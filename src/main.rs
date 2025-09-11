use std::{
    env,
    fs::{File, read_to_string},
    io::{self, Error, Read},
};

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

fn args() -> Vec<String> {
    env::args().collect()
}

fn main() -> io::Result<()> {
    init_tracing();
    let [_, ref file] = args()[..] else {
        return Err(Error::new(io::ErrorKind::InvalidInput, "File input needed"));
    };
    let path = read_to_string(file)?;

    let tokens = TokenStream::new(&path);
    let scope = Scope::from_tokens(&path, tokens);

    println!("{scope}");

    debug!("AST Scope: {:#?}", scope);

    Ok(())
}
