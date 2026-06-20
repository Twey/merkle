use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use merkle::Tree;
use sha2::Sha256;

type Sha256Tree = Tree<Sha256>;

const SIZES: &[usize] = &[10, 100, 1_000, 10_000];

fn leaves(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("leaf_{i}")).collect()
}

fn bench_construct(c: &mut Criterion) {
    let mut group = c.benchmark_group("construct");
    for &size in SIZES {
        let items = leaves(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &items, |b, items| {
            b.iter(|| items.iter().collect::<Sha256Tree>());
        });
    }
    group.finish();
}

fn bench_prove(c: &mut Criterion) {
    let mut group = c.benchmark_group("prove");
    for &size in SIZES {
        let tree: Sha256Tree = leaves(size).iter().collect();
        let index = size / 2;
        group.bench_with_input(BenchmarkId::from_parameter(size), &tree, |b, tree| {
            b.iter(|| tree.prove(index).unwrap());
        });
    }
    group.finish();
}

fn bench_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify");
    for &size in SIZES {
        let tree: Sha256Tree = leaves(size).iter().collect();
        let index = size / 2;
        group.bench_with_input(BenchmarkId::from_parameter(size), &tree, |b, tree| {
            b.iter(|| {
                let proof = tree.prove(index).unwrap();
                proof.preproof.verify().unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_construct, bench_prove, bench_verify);
criterion_main!(benches);
