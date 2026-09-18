use criterion::{Criterion, criterion_group, criterion_main};

use std::hint::black_box;

use hashbrown::HashMap;
use wollok_vm::bytecode::MethodNameIdx;
use wollok_vm::dispatch::MethodRef;
use wollok_vm::heap::ClassId;
use wollok_vm::program::ClassTable;

fn build_class_table(class_count: usize, methods_per_class: usize) -> (ClassTable, Vec<ClassId>) {
  let mut classes = ClassTable::default();
  let mut ids = Vec::with_capacity(class_count);

  for class_idx in 0..class_count {
    let mut methods = HashMap::with_capacity(methods_per_class);

    for method_idx in 0..methods_per_class {
      methods.insert(
        MethodNameIdx(u32::try_from(method_idx).unwrap()),
        MethodRef(u32::try_from(class_idx * methods_per_class + method_idx).unwrap()),
      );
    }

    ids.push(classes.define(format!("Class{class_idx}"), 0, None, methods));
  }

  (classes, ids)
}

fn bench_lookup(c: &mut Criterion) {
  let (classes, ids) = build_class_table(10_000, 16);
  c.bench_function("lookup on classes", |b| {
    let mut i = 0usize;
    b.iter(|| {
      let class = ids[i % ids.len()];
      let selector = MethodNameIdx(u32::try_from(i % 16).unwrap());

      i = i.wrapping_add(1);
      black_box(classes.lookup(black_box(class), black_box(selector)));
    });
  });
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
