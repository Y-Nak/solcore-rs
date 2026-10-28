//! Compatibility entry point for the standalone Solcore language server.

fn main() {
    std::process::exit(solcore_lsp::native::run_cli(
        "solcore-lsp",
        std::env::args_os().skip(1).collect(),
    ));
}
