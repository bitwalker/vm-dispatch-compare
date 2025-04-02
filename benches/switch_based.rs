use vm_bench::*;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_switch_based(c: &mut Criterion) {
    // The program we want to execute
    let program: Vec<Inst> = vec![
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
    ];
    
    let id = criterion::BenchmarkId::new("switch based", 1);
    c.bench_with_input(id, &program, |b, code| b.iter(|| switch_based(code)));
}

criterion_group!(benches, bench_switch_based);
criterion_main!(benches);
