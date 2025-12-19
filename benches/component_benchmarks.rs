use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use zed::{Cache, Capsule, ReactiveSystem, Reducer, SimpleCache, configure_store, create_reducer};

#[derive(Clone, Debug)]
struct BenchState {
    counter: i64,
    data: Vec<String>,
    is_active: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum BenchAction {
    Increment,
    AddData(String),
    Toggle,
    Reset,
}

fn bench_reactive_system_creation(c: &mut Criterion) {
    c.bench_function("reactive_system_creation", |b| {
        b.iter(|| {
            let initial_state = BenchState {
                counter: 0,
                data: vec![],
                is_active: false,
            };
            ReactiveSystem::new(initial_state)
        })
    });
}

fn bench_reactive_system_trigger(c: &mut Criterion) {
    let mut group = c.benchmark_group("reactive_system_trigger");

    for num_reactions in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*num_reactions as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_reactions),
            num_reactions,
            |b, &num_reactions| {
                let mut system = ReactiveSystem::new(BenchState {
                    counter: 0,
                    data: vec![],
                    is_active: false,
                });

                // Add multiple reactions
                for i in 0..num_reactions {
                    system.on(format!("action_{i}"), |state: &mut BenchState| {
                        state.counter += 1;
                    });
                }

                b.iter(|| {
                    for i in 0..num_reactions {
                        system.trigger(black_box(format!("action_{i}")));
                    }
                })
            },
        );
    }
    group.finish();
}

fn bench_capsule_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("capsule_operations");

    group.bench_function("capsule_creation", |b| {
        b.iter(|| {
            let capsule: Capsule<BenchState, BenchAction> = Capsule::new(BenchState {
                counter: 0,
                data: vec![],
                is_active: false,
            });
            capsule
        })
    });

    group.bench_function("capsule_with_cache", |b| {
        b.iter(|| {
            let capsule: Capsule<BenchState, BenchAction> = Capsule::new(BenchState {
                counter: 0,
                data: vec![],
                is_active: false,
            })
            .with_cache(SimpleCache::new());
            capsule
        })
    });

    group.bench_function("capsule_dispatch", |b| {
        let mut capsule = Capsule::new(BenchState {
            counter: 0,
            data: vec![],
            is_active: false,
        })
        .with_logic(|state: &mut BenchState, action: BenchAction| match action {
            BenchAction::Increment => state.counter += 1,
            BenchAction::AddData(s) => state.data.push(s),
            BenchAction::Toggle => state.is_active = !state.is_active,
            BenchAction::Reset => {
                state.counter = 0;
                state.data.clear();
                state.is_active = false;
            }
        })
        .with_cache(SimpleCache::new());

        b.iter(|| {
            capsule.dispatch(black_box(BenchAction::Increment));
        })
    });

    group.finish();
}

fn bench_simple_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_cache_operations");

    group.bench_function("cache_creation", |b| {
        b.iter(|| {
            let cache: SimpleCache<BenchState> = SimpleCache::new();
            cache
        })
    });

    group.bench_function("cache_set", |b| {
        let mut cache = SimpleCache::new();
        let state = BenchState {
            counter: 42,
            data: vec!["test".to_string()],
            is_active: true,
        };

        b.iter(|| {
            cache.set(black_box(state.clone()));
        })
    });

    group.bench_function("cache_get", |b| {
        let mut cache = SimpleCache::new();
        cache.set(BenchState {
            counter: 42,
            data: vec!["test".to_string()],
            is_active: true,
        });

        b.iter(|| {
            let _result = cache.get();
        })
    });

    group.finish();
}

fn bench_configure_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configure_store_operations");

    group.bench_function("configure_store_creation", |b| {
        b.iter(|| {
            configure_store(
                BenchState {
                    counter: 0,
                    data: vec![],
                    is_active: false,
                },
                create_reducer(|state: &BenchState, action: &BenchAction| {
                    let mut new_state = state.clone();
                    match action {
                        BenchAction::Increment => new_state.counter += 1,
                        BenchAction::AddData(s) => new_state.data.push(s.clone()),
                        BenchAction::Toggle => new_state.is_active = !new_state.is_active,
                        BenchAction::Reset => {
                            new_state.counter = 0;
                            new_state.data.clear();
                            new_state.is_active = false;
                        }
                    }
                    new_state
                }),
            )
        })
    });

    group.bench_function("configure_store_dispatch", |b| {
        let store = configure_store(
            BenchState {
                counter: 0,
                data: vec![],
                is_active: false,
            },
            create_reducer(|state: &BenchState, action: &BenchAction| {
                let mut new_state = state.clone();
                match action {
                    BenchAction::Increment => new_state.counter += 1,
                    BenchAction::AddData(s) => new_state.data.push(s.clone()),
                    BenchAction::Toggle => new_state.is_active = !new_state.is_active,
                    BenchAction::Reset => {
                        new_state.counter = 0;
                        new_state.data.clear();
                        new_state.is_active = false;
                    }
                }
                new_state
            }),
        );

        b.iter(|| {
            store.dispatch(black_box(BenchAction::Increment));
        })
    });

    group.finish();
}

fn bench_reducer_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("reducer_operations");

    group.bench_function("reducer_creation", |b| {
        b.iter(|| {
            create_reducer(|state: &BenchState, action: &BenchAction| {
                let mut new_state = state.clone();
                match action {
                    BenchAction::Increment => new_state.counter += 1,
                    BenchAction::AddData(s) => new_state.data.push(s.clone()),
                    BenchAction::Toggle => new_state.is_active = !new_state.is_active,
                    BenchAction::Reset => {
                        new_state.counter = 0;
                        new_state.data.clear();
                        new_state.is_active = false;
                    }
                }
                new_state
            })
        })
    });

    group.bench_function("reducer_execution", |b| {
        let reducer = create_reducer(|state: &BenchState, action: &BenchAction| {
            let mut new_state = state.clone();
            match action {
                BenchAction::Increment => new_state.counter += 1,
                BenchAction::AddData(s) => new_state.data.push(s.clone()),
                BenchAction::Toggle => new_state.is_active = !new_state.is_active,
                BenchAction::Reset => {
                    new_state.counter = 0;
                    new_state.data.clear();
                    new_state.is_active = false;
                }
            }
            new_state
        });

        let state = BenchState {
            counter: 0,
            data: vec![],
            is_active: false,
        };

        b.iter(|| {
            let _result = reducer.reduce(black_box(&state), black_box(&BenchAction::Increment));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_reactive_system_creation,
    bench_reactive_system_trigger,
    bench_capsule_operations,
    bench_simple_cache_operations,
    bench_configure_store_operations,
    bench_reducer_operations,
);
criterion_main!(benches);
