use criterion::{Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;

use wollok_ast::ast::Scope;
use wollok_lexer::lexer::TokenStream;
use wollok_vm::value::Value;
use wollok_vm::vm::Vm;

/// No `console.println` in these on purpose: criterion runs the body
/// hundreds of times per sample, and real stdout I/O inside that loop
/// is exactly what causes noisy outliers (buffering/flush/scheduling),
/// not VM variance. `bench/corpus/` (with printing) is for the
/// one-shot CLI comparison against wollok-ts, not this.
fn fixture(name: &str) -> String {
  fs::read_to_string(format!("benches/fixtures/{name}.wlk")).expect("fixture missing")
}

fn bench_fixture(c: &mut Criterion, name: &str) {
  let src = fixture(name);

  c.bench_function(&format!("parse/{name}"), |b| {
    b.iter(|| {
      let tokens = TokenStream::new(black_box(&src));
      black_box(Scope::from_tokens(name, tokens))
    });
  });

  c.bench_function(&format!("parse+compile/{name}"), |b| {
    b.iter(|| {
      let tokens = TokenStream::new(black_box(&src));
      let scope = Scope::from_tokens(name, tokens);
      let mut vm = Vm::new();
      wollok_std::install(&mut vm);
      black_box(wollok_compiler::compile(&scope, &mut vm).expect("compile failed"))
    });
  });

  c.bench_function(&format!("parse+compile+run/{name}"), |b| {
    b.iter(|| {
      let tokens = TokenStream::new(black_box(&src));
      let scope = Scope::from_tokens(name, tokens);
      let mut vm = Vm::new();
      wollok_std::install(&mut vm);
      let compiled = wollok_compiler::compile(&scope, &mut vm).expect("compile failed");
      black_box(vm.run_method(&compiled.program, compiled.main, Value::Null, vec![]))
    });
  });
}

fn benches(c: &mut Criterion) {
  bench_fixture(c, "aritmetica");
  bench_fixture(c, "mensajes");
}

criterion_group!(pipeline, benches);
criterion_main!(pipeline);
