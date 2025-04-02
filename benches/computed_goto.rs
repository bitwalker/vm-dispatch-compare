use vm_bench::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_computed_goto(c: &mut Criterion) {
    // The program we want to execute
    let program: Vec<Inst> = black_box(vec![
        Inst::Begin,
        Inst::Push(1),
        Inst::If(If{
            if_true: 3,
            if_false: 4,
        }),
        Inst::Push(2),
        Inst::Push(0),
        Inst::Add,
        Inst::Stop,
    ]);
    
    // The bytecode of that program
    let code = program.into_iter().map(Opcode::compile).collect::<Vec<_>>();

    let id = criterion::BenchmarkId::new("computed goto", 1);
    c.bench_with_input(id, &code, |b, code| b.iter(|| computed_goto(code)));
}

criterion_group!(benches, bench_computed_goto);
criterion_main!(benches);
