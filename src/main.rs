use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command as ProcessCommand,
};

use clap::Parser;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use mcp_payloadcms_rs::{cli, metadata, server};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

const SETTINGS_PATH: &str = "settings.json";

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Command::Start(args) => {
            if !args.foreground {
                let mut child_args: Vec<String> = std::env::args().skip(1).collect();
                if !child_args.iter().any(|a| a == "--foreground") {
                    child_args.push("--foreground".to_string());
                }
                match std::process::Command::new(std::env::current_exe().unwrap())
                    .args(child_args)
                    .spawn()
                {
                    Ok(child) => {
                        println!(
                            "{} started in background (pid {})",
                            "OK".green().bold(),
                            child.id()
                        );
                    }
                    Err(err) => {
                        eprintln!("{}", format!("Failed to spawn server: {err}").red().bold())
                    }
                }
                return;
            }
            let mut effective = load_settings();
            overlay_args(&mut effective, &args);
            if let Err(err) = effective.validate() {
                eprintln!("{}", format!("Error: {err}").red().bold());
                return;
            };

            let has_network = effective.enable_tcp
                || effective.enable_unix
                || effective.enable_http
                || effective.enable_sse
                || effective.enable_ws;
            if has_network {
                if let Some(true) = running_status(&effective) {
                    eprintln!(
                        "{} {}",
                        "Error:".red().bold(),
                        "another instance is already running (non-stdio transport enabled)"
                    );
                    eprintln!(
                        "Stop the existing server or disable network transports to run stdio-only."
                    );
                    return;
                }
            }

            if let Err(error) = server::start_server(effective.clone()).await {
                eprintln!("{}", format!("{error}").red().bold());
            } else {
                if let Err(err) = fs::write(&effective.pid_file, format!("{}", std::process::id()))
                {
                    eprintln!(
                        "{}",
                        format!("Warning: could not write pid file: {err}").yellow()
                    );
                } else {
                    println!(
                        "{} pid file written to {}",
                        "OK".green().bold(),
                        &effective.pid_file
                    );
                }
                save_settings(&effective);
            }
        }
        cli::Command::Status => {
            let args = load_settings();
            status_report(&args);
        }
        cli::Command::Shutdown => {
            let args = load_settings();
            shutdown_server(&args);
        }
        cli::Command::Version => {
            println!(
                "{} v{} - {}",
                metadata::PKG_NAME,
                metadata::PKG_VERSION,
                metadata::PKG_DESCRIPTION
            );
        }
        cli::Command::Setup => {
            let mut args = load_settings();
            interactive_setup(&mut args);
            save_settings(&args);
        }
        cli::Command::Config => {
            let mut args = load_settings();
            config_tui(&mut args);
            save_settings(&args);
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SettingsFile {
    server_name: Option<String>,
    server_description: Option<String>,
    enable_stdio: Option<bool>,
    enable_tcp: Option<bool>,
    enable_unix: Option<bool>,
    enable_http: Option<bool>,
    enable_sse: Option<bool>,
    enable_ws: Option<bool>,
    tcp_addr: Option<String>,
    http_addr: Option<String>,
    sse_addr: Option<String>,
    ws_addr: Option<String>,
    unix_path: Option<String>,
    pid_file: Option<String>,
}

fn load_settings() -> cli::CommandArguments {
    let defaults = cli::CommandArguments::default_settings();
    if let Ok(contents) = fs::read_to_string(SETTINGS_PATH) {
        if let Ok(settings) = serde_json::from_str::<SettingsFile>(&contents) {
            return apply_settings(settings, defaults);
        }
    }
    defaults
}

fn save_settings(args: &cli::CommandArguments) {
    let settings = SettingsFile {
        server_name: Some(args.server_name.clone()),
        server_description: Some(args.server_description.clone()),
        enable_stdio: Some(args.enable_stdio),
        enable_tcp: Some(args.enable_tcp),
        enable_unix: Some(args.enable_unix),
        enable_http: Some(args.enable_http),
        enable_sse: Some(args.enable_sse),
        enable_ws: Some(args.enable_ws),
        tcp_addr: Some(args.tcp_addr.clone()),
        http_addr: Some(args.http_addr.clone()),
        sse_addr: Some(args.sse_addr.clone()),
        ws_addr: Some(args.ws_addr.clone()),
        unix_path: Some(args.unix_path.clone()),
        pid_file: Some(args.pid_file.clone()),
    };
    if let Err(err) = fs::write(
        SETTINGS_PATH,
        serde_json::to_string_pretty(&settings).unwrap_or_default(),
    ) {
        eprintln!(
            "{}",
            format!("Warning: could not persist settings: {err}").yellow()
        );
    } else {
        println!(
            "{} settings saved to {}",
            "OK".green().bold(),
            SETTINGS_PATH
        );
    }
}

fn apply_settings(
    settings: SettingsFile,
    mut base: cli::CommandArguments,
) -> cli::CommandArguments {
    if let Some(v) = settings.server_name {
        base.server_name = v;
    }
    if let Some(v) = settings.server_description {
        base.server_description = v;
    }
    if let Some(v) = settings.enable_stdio {
        base.enable_stdio = v;
    }
    if let Some(v) = settings.enable_tcp {
        base.enable_tcp = v;
    }
    if let Some(v) = settings.enable_unix {
        base.enable_unix = v;
    }
    if let Some(v) = settings.enable_http {
        base.enable_http = v;
    }
    if let Some(v) = settings.enable_sse {
        base.enable_sse = v;
    }
    if let Some(v) = settings.enable_ws {
        base.enable_ws = v;
    }
    if let Some(v) = settings.tcp_addr {
        base.tcp_addr = v;
    }
    if let Some(v) = settings.http_addr {
        base.http_addr = v;
    }
    if let Some(v) = settings.sse_addr {
        base.sse_addr = v;
    }
    if let Some(v) = settings.ws_addr {
        base.ws_addr = v;
    }
    if let Some(v) = settings.unix_path {
        base.unix_path = v;
    }
    if let Some(v) = settings.pid_file {
        base.pid_file = v;
    }
    base
}

fn overlay_args(target: &mut cli::CommandArguments, overrides: &cli::CommandArguments) {
    let defaults = cli::CommandArguments::default_settings();
    if overrides.server_name != defaults.server_name {
        target.server_name = overrides.server_name.clone();
    }
    if overrides.server_description != defaults.server_description {
        target.server_description = overrides.server_description.clone();
    }
    if overrides.enable_stdio != defaults.enable_stdio {
        target.enable_stdio = overrides.enable_stdio;
    }
    if overrides.enable_tcp != defaults.enable_tcp {
        target.enable_tcp = overrides.enable_tcp;
    }
    if overrides.enable_unix != defaults.enable_unix {
        target.enable_unix = overrides.enable_unix;
    }
    if overrides.enable_http != defaults.enable_http {
        target.enable_http = overrides.enable_http;
    }
    if overrides.enable_sse != defaults.enable_sse {
        target.enable_sse = overrides.enable_sse;
    }
    if overrides.enable_ws != defaults.enable_ws {
        target.enable_ws = overrides.enable_ws;
    }
    if overrides.tcp_addr != defaults.tcp_addr {
        target.tcp_addr = overrides.tcp_addr.clone();
    }
    if overrides.http_addr != defaults.http_addr {
        target.http_addr = overrides.http_addr.clone();
    }
    if overrides.sse_addr != defaults.sse_addr {
        target.sse_addr = overrides.sse_addr.clone();
    }
    if overrides.ws_addr != defaults.ws_addr {
        target.ws_addr = overrides.ws_addr.clone();
    }
    if overrides.unix_path != defaults.unix_path {
        target.unix_path = overrides.unix_path.clone();
    }
    if overrides.pid_file != defaults.pid_file {
        target.pid_file = overrides.pid_file.clone();
    }
}

fn status_report(args: &cli::CommandArguments) {
    let running = running_status(args);
    let (state_icon, state_msg) = match running {
        Some(true) => ("✅".green().bold(), "running".green().bold()),
        Some(false) => ("🟥".red().bold(), "stopped".red().bold()),
        None => ("⬜".normal(), "unknown".yellow().bold()),
    };

    println!("{} {}", state_icon, state_msg);
    println!(
        "  {} {}",
        "Version:".blue().bold(),
        format!("{} {}", metadata::PKG_NAME, metadata::PKG_VERSION)
    );
    println!(
        "  {} {}",
        "Server:".blue().bold(),
        format!("{} -- {}", args.server_name, args.server_description)
    );

    match server::TransportState::from_args(args) {
        Ok(transports) => {
            let endpoints = transports.active_endpoints();
            let (tx_icon, tx_label) = if endpoints.is_empty() {
                ("⬜".normal(), "no transports".red().bold())
            } else {
                (
                    "✅".green().bold(),
                    format!("{} transports", endpoints.len()).green().bold(),
                )
            };
            println!("  {} {}", tx_icon, tx_label);
            for ep in endpoints {
                println!("    - {ep}");
            }
            println!(
                "  {}",
                "Ports set to 0 will be auto-assigned at runtime.".yellow()
            );
        }
        Err(err) => eprintln!("  {} {}", "Error:".red().bold(), err),
    }
}

fn interactive_setup(args: &mut cli::CommandArguments) {
    println!("{}", "Setup assistant".green().bold());
    let clients = ["vscode", "codex", "gemini", "zed", "roo", "other"];
    println!("Known clients:");
    for (i, c) in clients.iter().enumerate() {
        println!("{:>2}. {}", i + 1, c);
    }
    let client_idx = prompt_choice("Select client [1]: ", &clients, 0);
    let client = clients[client_idx];

    let transport_options = vec![
        (
            "http+streamable-sse",
            args.enable_http,
            args.http_addr.as_str(),
        ),
        ("sse", args.enable_sse, args.sse_addr.as_str()),
        ("tcp", args.enable_tcp, args.tcp_addr.as_str()),
        ("unix", args.enable_unix, args.unix_path.as_str()),
        ("stdio", args.enable_stdio, "stdio"),
        ("ws", args.enable_ws, args.ws_addr.as_str()),
    ];
    let default_transport_idx = transport_options
        .iter()
        .position(|(name, enabled, _)| {
            *enabled && (*name == "http+streamable-sse" || *name == "sse")
        })
        .or_else(|| {
            transport_options
                .iter()
                .position(|(_, enabled, _)| *enabled)
        })
        .unwrap_or(0);

    println!("Available transports (enabled marked with *):");
    for (i, (name, enabled, addr)) in transport_options.iter().enumerate() {
        let mark = if *enabled { "*" } else { " " };
        let addr_display = if *name == "stdio" {
            "".to_string()
        } else {
            format!(" ({addr})")
        };
        println!("{:>2}. [{}] {}{}", i + 1, mark, name, addr_display);
    }
    let transport_idx = prompt_choice(
        &format!("Select transport [{}]: ", default_transport_idx + 1),
        &transport_options
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>(),
        default_transport_idx,
    );
    let (transport_name, _, addr) = &transport_options[transport_idx];

    println!(
        "\n{} {} using transport {}",
        "Configuring".green().bold(),
        client,
        transport_name
    );
    match *transport_name {
        "stdio" => args.enable_stdio = true,
        "unix" => args.enable_unix = true,
        "http+streamable-sse" => args.enable_http = true,
        "sse" => args.enable_sse = true,
        "tcp" => args.enable_tcp = true,
        "ws" => args.enable_ws = true,
        _ => {}
    }
    match *transport_name {
        "stdio" => println!("Use stdio mode; no host/port required."),
        "unix" => println!("Socket path: {}", addr),
        _ => {
            if addr.ends_with(":0") {
                println!("Address: {} (port auto-assigned at runtime)", addr);
            } else {
                println!("Address: {}", addr);
            }
        }
    }
    if !matches!(*transport_name, "http+streamable-sse" | "sse" | "stdio") {
        println!(
            "{} Non-default transport selected; enabling it in settings.",
            "Note:".yellow()
        );
    }
    println!(
        "{} Settings updated. HTTP/SSE/stdio remain preferred defaults unless you disable them.",
        "Info".blue().bold()
    );
}

fn config_tui(args: &mut cli::CommandArguments) {
    let theme = ColorfulTheme::default();
    loop {
        println!("\n{} {}", "Config".green().bold(), "(settings.json)".blue());
        println!(
            "  transports: stdio={}, http={}, sse={}, tcp={}, unix={}, ws={}",
            args.enable_stdio,
            args.enable_http,
            args.enable_sse,
            args.enable_tcp,
            args.enable_unix,
            args.enable_ws
        );
        println!(
            "  addresses: http={}, sse={}, tcp={}, ws={}, unix={}",
            args.http_addr, args.sse_addr, args.tcp_addr, args.ws_addr, args.unix_path
        );
        println!("  pid file: {}", args.pid_file);
        println!(
            "  server: {} -- {}",
            args.server_name, args.server_description
        );

        let choices = vec![
            "Edit server name/description",
            "Toggle transports",
            "Edit addresses",
            "Edit pid file",
            "Save and exit",
            "Exit without saving",
        ];
        let selection = Select::with_theme(&theme)
            .with_prompt("Choose an option")
            .items(&choices)
            .default(0)
            .interact();
        let selection = match selection {
            Ok(idx) => idx,
            Err(_) => return,
        };
        match selection {
            0 => edit_server_metadata(args, &theme),
            1 => toggle_transports(args, &theme),
            2 => edit_addresses(args, &theme),
            3 => {
                if let Ok(pid) = Input::with_theme(&theme)
                    .with_prompt("PID file path")
                    .default(args.pid_file.clone())
                    .interact_text()
                {
                    args.pid_file = pid;
                }
            }
            4 => break,
            5 => return,
            _ => {}
        }
    }
}

fn edit_server_metadata(args: &mut cli::CommandArguments, theme: &ColorfulTheme) {
    if let Ok(name) = Input::with_theme(theme)
        .with_prompt("Server name")
        .default(args.server_name.clone())
        .interact_text()
    {
        args.server_name = name;
    }
    if let Ok(desc) = Input::with_theme(theme)
        .with_prompt("Server description")
        .default(args.server_description.clone())
        .interact_text()
    {
        args.server_description = desc;
    }
}

fn toggle_transports(args: &mut cli::CommandArguments, theme: &ColorfulTheme) {
    let toggles = [
        ("stdio", &mut args.enable_stdio),
        ("http", &mut args.enable_http),
        ("sse", &mut args.enable_sse),
        ("tcp", &mut args.enable_tcp),
        ("unix", &mut args.enable_unix),
        ("ws", &mut args.enable_ws),
    ];
    for (name, flag) in toggles {
        if let Ok(val) = Confirm::with_theme(theme)
            .with_prompt(format!("Enable {name}? (currently {flag})"))
            .default(*flag)
            .interact()
        {
            *flag = val;
        }
    }
}

fn edit_addresses(args: &mut cli::CommandArguments, theme: &ColorfulTheme) {
    if let Ok(val) = Input::with_theme(theme)
        .with_prompt("HTTP address")
        .default(args.http_addr.clone())
        .interact_text()
    {
        args.http_addr = val;
    }
    if let Ok(val) = Input::with_theme(theme)
        .with_prompt("SSE address")
        .default(args.sse_addr.clone())
        .interact_text()
    {
        args.sse_addr = val;
    }
    if let Ok(val) = Input::with_theme(theme)
        .with_prompt("TCP address")
        .default(args.tcp_addr.clone())
        .interact_text()
    {
        args.tcp_addr = val;
    }
    if let Ok(val) = Input::with_theme(theme)
        .with_prompt("Websocket address")
        .default(args.ws_addr.clone())
        .interact_text()
    {
        args.ws_addr = val;
    }
    if let Ok(val) = Input::with_theme(theme)
        .with_prompt("Unix socket path")
        .default(args.unix_path.clone())
        .interact_text()
    {
        args.unix_path = val;
    }
}

fn prompt_choice(prompt: &str, options: &[&str], default_idx: usize) -> usize {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        if let Ok(num) = input.trim().parse::<usize>() {
            if num >= 1 && num <= options.len() {
                return num - 1;
            }
        }
    }
    default_idx.min(options.len().saturating_sub(1))
}

fn shutdown_server(args: &cli::CommandArguments) {
    let pid_path = Path::new(&args.pid_file);
    match fs::read_to_string(pid_path) {
        Ok(contents) => match contents.trim().parse::<i32>() {
            Ok(pid) => {
                println!(
                    "{} sending shutdown to pid {}",
                    "Shutdown".yellow().bold(),
                    pid
                );
                if let Err(err) = ProcessCommand::new("kill").arg(pid.to_string()).status() {
                    eprintln!(
                        "{}",
                        format!("Failed to signal process: {err}").red().bold()
                    );
                }
            }
            Err(_) => eprintln!("{}", "Invalid pid file contents.".red().bold()),
        },
        Err(err) => eprintln!(
            "{}",
            format!("Could not read pid file {}: {err}", pid_path.display())
                .red()
                .bold()
        ),
    }
}

fn running_status(args: &cli::CommandArguments) -> Option<bool> {
    let pid_path = Path::new(&args.pid_file);
    let contents = match fs::read_to_string(pid_path) {
        Ok(c) => c,
        Err(_) => return None,
    };
    let pid_raw: i32 = match contents.trim().parse() {
        Ok(p) => p,
        Err(_) => return None,
    };
    let pid = Pid::from_u32(pid_raw as u32);
    let sys = System::new_all();
    if sys.process(pid).is_some() {
        return Some(true);
    }
    // Fallback: check any process name matching our package name
    let name = metadata::PKG_NAME;
    for process in sys.processes().values() {
        let pname = process.name().to_string_lossy();
        if pname.contains(name) {
            return Some(true);
        }
    }
    Some(false)
}
