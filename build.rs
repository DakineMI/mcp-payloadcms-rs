use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
    println!("cargo:rerun-if-changed={}", cargo_toml_path.display());

    let content = fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("Failed to read Cargo.toml: {e}"));
    let parsed: toml::Value =
        toml::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse Cargo.toml: {e}"));
    let pkg = parsed
        .get("package")
        .and_then(|p| p.as_table())
        .expect("Cargo.toml missing [package]");

    let name = pkg
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("mcp-server");
    let version = pkg
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0");
    let description = pkg
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("pkg_info.rs");
    let contents = format!(
        r#"pub const PKG_NAME: &str = "{name}";
pub const PKG_VERSION: &str = "{version}";
pub const PKG_DESCRIPTION: &str = "{description}";
"#
    );
    fs::write(&dest, contents).expect("Failed to write pkg_info.rs");
}
