use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;
use std::thread;
use zed::{Store, create_reducer};

#[derive(Clone, Debug)]
struct BenchState {
    counter: i64,
    data: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
enum BenchAction {
    Increment,
    AddData(String),
    Reset,
}

fn bench_reducer(state: &BenchState, action: &BenchAction) -> BenchState {
    match action {
        BenchAction::Increment => BenchState {
            counter: state.counter + 1,
            data: state.data.clone(),
        },
        BenchAction::AddData(s) => BenchState {
            counter: state.counter,
            data: {
                let mut new_data = state.data.clone();
                new_data.push(s.clone());
                new_data
            },
        },
        BenchAction::Reset => BenchState {
            counter: 0,
            data: vec![],
        },
    }
}

fn bench_store_creation(c: &mut Criterion) {
    c.bench_function("store_creation", |b| {
        b.iter(|| {
            let initial_state = BenchState {
                counter: 0,
                data: vec![],
            };
            Store::new(initial_state, Box::new(create_reducer(bench_reducer)))
        })
    });
}

fn bench_store_dispatch(c: &mut Criterion) {
    let initial_state = BenchState {
        counter: 0,
        data: vec![],
    };
    let store = Store::new(initial_state, Box::new(create_reducer(bench_reducer)));

    c.bench_function("store_dispatch_increment", |b| {
        b.iter(|| {
            store.dispatch(black_box(BenchAction::Increment));
        })
    });
}

fn bench_store_dispatch_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_dispatch_throughput");

    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let initial_state = BenchState {
                counter: 0,
                data: vec![],
            };
            let store = Store::new(initial_state, Box::new(create_reducer(bench_reducer)));

            b.iter(|| {
                for _ in 0..size {
                    store.dispatch(black_box(BenchAction::Increment));
                }
            })
        });
    }
    group.finish();
}

fn bench_store_subscribers(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_subscribers");

    for subscriber_count in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(subscriber_count),
            subscriber_count,
            |b, &subscriber_count| {
                let initial_state = BenchState {
                    counter: 0,
                    data: vec![],
                };
                let store = Store::new(initial_state, Box::new(create_reducer(bench_reducer)));

                // Add subscribers
                for _ in 0..subscriber_count {
                    store.subscribe(|_state: &BenchState| {
                        // Minimal work in subscriber
                    });
                }

                b.iter(|| {
                    store.dispatch(black_box(BenchAction::Increment));
                })
            },
        );
    }
    group.finish();
}

fn bench_store_concurrent_access(c: &mut Criterion) {
    c.bench_function("store_concurrent_dispatch", |b| {
        let initial_state = BenchState {
            counter: 0,
            data: vec![],
        };
        let store = Arc::new(Store::new(
            initial_state,
            Box::new(create_reducer(bench_reducer)),
        ));

        b.iter(|| {
            let mut handles = vec![];

            for _ in 0..4 {
                let store_clone = Arc::clone(&store);
                let handle = thread::spawn(move || {
                    for _ in 0..100 {
                        store_clone.dispatch(BenchAction::Increment);
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

fn bench_store_state_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_state_size");

    for data_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(data_size),
            data_size,
            |b, &data_size| {
                let initial_data: Vec<String> =
                    (0..data_size).map(|i| format!("data_item_{i}")).collect();

                let initial_state = BenchState {
                    counter: 0,
                    data: initial_data,
                };
                let store = Store::new(initial_state, Box::new(create_reducer(bench_reducer)));

                b.iter(|| {
                    store.dispatch(black_box(BenchAction::Increment));
                })
            },
        );
    }
    group.finish();
}

fn bench_store_get_state(c: &mut Criterion) {
    let initial_state = BenchState {
        counter: 42,
        data: vec!["test".to_string(); 100],
    };
    let store = Store::new(initial_state, Box::new(create_reducer(bench_reducer)));

    c.bench_function("store_get_state", |b| {
        b.iter(|| {
            black_box(store.get_state());
        })
    });
}

criterion_group!(
    benches,
    bench_store_creation,
    bench_store_dispatch,
    bench_store_dispatch_throughput,
    bench_store_subscribers,
    bench_store_concurrent_access,
    bench_store_state_size,
    bench_store_get_state
);
criterion_main!(benches);
