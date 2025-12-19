/// Generates a `match` expression for dispatching `ToolBox` (or other provided enum) variants to their respective `run_tool` methods.
///
/// This macro reduces boilerplate in matching `FileSystemTools` enum variants by generating a `match` arm
/// for each specified tool. Each arm calls the tool's `run_tool` method with the provided parameters and
/// filesystem service, handling the async dispatch uniformly.
///
/// # Parameters
/// - `$params:expr`: The expression to match against, expected to be an enum value (e.g., `ToolBox`).
/// - `$toolbox_service:expr`: The filesystem service reference (e.g., `&self.fs_service`) to pass to each tool's `run_tool` method.
/// - `$enum:path` (optional): The enum type to match against (defaults to `ToolBox`).
/// - `$($tool:ident),*`: A comma-separated list of tool identifiers (e.g., `ReadMediaFileTool`, `WriteFileTool`) corresponding to
///   `FileSystemTools` variants and their associated types.
///
/// # Usage
/// The macro is typically used within an async method that dispatches filesystem operations based on a `ToolBox`-style enum.
/// Each tool must have a `run_tool` method with the signature:
/// ```rust
/// async fn run_tool(params: ParamsType, fs_service: &FsService) -> ServiceResult<()>
/// ```
/// where `ParamsType` is the parameter type for the specific tool, and `FsService` is the filesystem service type.
///
/// # Example
/// ```rust
/// invoke_tools!(
///     tool_params,
///     &self.fs_service,
///     ReadMediaFileTool,
///     WriteFileTool,
/// )
/// ```
/// This expands to:
/// ```rust
/// match tool_params {
///     ToolBox::ReadMediaFileTool(params) => ReadMediaFileTool::run_tool(params, &self.fs_service).await,
///     ToolBox::WriteFileTool(params) => WriteFileTool::run_tool(params, &self.fs_service).await,
/// }
/// ```
///
/// # Notes
/// - Ensure each tool identifier matches a variant of the `FileSystemTools` enum and has a corresponding `run_tool` method.
/// - The macro assumes all `run_tool` methods are `async` and return `ServiceResult<()>`.
/// - To add a new tool, include its identifier in the macro invocation list.
/// - If a tool has a different `run_tool` signature, a separate macro or manual `match` arm may be needed.
#[macro_export]
macro_rules! invoke_tools {
    // Default form using `ToolBox::<Variant>` enum path
    ($params:expr, $toolbox_service:expr, $($tool:ident),* $(,)?) => {
        match $params {
            $(
                ToolBox::$tool(params) => $tool::run_tool(params, $toolbox_service).await,
            )*
        }
    };

    // Explicit enum path form to support other enums
    ($params:expr, $toolbox_service:expr, $enum:path, $($tool:ident),* $(,)?) => {
        match $params {
            $(
                $enum::$tool(params) => $tool::run_tool(params, $toolbox_service).await,
            )*
        }
    };
}
