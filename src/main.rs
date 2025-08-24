use tracing::debug;
use tracing_subscriber::EnvFilter;
use wollok_ast::ast::Scope;
use wollok_lexer::lexer::TokenStream;

fn init_tracing() {
    // Configurar tracing simple a stdout
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(true)
        .init();
}

fn main() {
    init_tracing();
    let base = include_str!("../example.wlk");
    let tokens = TokenStream::new(base);
    let scope = Scope::from_tokens(base, tokens);
    // println!("{scope:#?}");
    debug!("Parsed AST scope: {:#?}", scope);
}
