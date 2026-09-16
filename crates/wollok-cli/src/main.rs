use std::fs;
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

fn init_tracing() {
    // Configurar tracing simple a stdout
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("wollok=debug".parse().unwrap()),
        )
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}

fn main() {
    init_tracing();

    info!("Wollok CLI starting up");
    debug!("Debug information available");

    // Intentar leer y procesar un archivo de ejemplo
    if let Ok(content) = fs::read_to_string("../example.wlk") {
        info!("Processing test file");

        // Crear TokenStream y recolectar tokens
        let token_stream = wollok_lexer::lexer::TokenStream::new(&content);
        if token_stream.collect_all().is_ok() {
            info!("Lexing completed successfully");

            // Intentar parsear AST
            let token_stream = wollok_lexer::lexer::TokenStream::new(&content);
            let _ = wollok_ast::ast::Scope::from_tokens("example.wlk", token_stream);
            info!("AST parsing completed");
        } else {
            info!("Lexing failed");
        }
    } else {
        info!("No test file found, running basic example");
    }

    println!("Hello, world!");

    info!("Wollok CLI shutting down");
}
