use std::{
  env,
  fs::read_to_string,
  io::{self, Error},
};

use tracing::debug;
use tracing_subscriber::EnvFilter;
use wollok_ast::ast::Scope;
use wollok_lexer::lexer::TokenStream;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

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

  let mut vm = Vm::new();
  wollok_std::install(&mut vm);
  match wollok_compiler::compile(&scope, &mut vm) {
    Ok(compiled) => {
      let result = vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]);
      match result.as_str_idx() {
        Some(idx) => println!("=> {:?}", vm.strings.get(idx)),
        None => println!("=> {result:?}"),
      }
    }
    Err(err) => eprintln!("compile error: {err}"),
  }

  Ok(())
}
