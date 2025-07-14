use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::any::Any;
use std::hint::black_box;
use zed::StateManager;

#[derive(Clone, Debug)]
struct TimelineState {
    counter: i64,
    history: Vec<String>,
}

#[derive(Clone, Debug)]
enum TimelineAction {
    Increment,
    AddHistory(String),
    Reset,
}

fn timeline_reducer(state: &TimelineState, action: &dyn Any) -> TimelineState {
    if let Some(action) = action.downcast_ref::<TimelineAction>() {
        match action {
            TimelineAction::Increment => TimelineState {
                counter: state.counter + 1,
                history: state.history.clone(),
            },
            TimelineAction::AddHistory(s) => TimelineState {
                counter: state.counter,
                history: {
                    let mut new_history = state.history.clone();
                    new_history.push(s.clone());
                    new_history
                },
            },
            TimelineAction::Reset => TimelineState {
                counter: 0,
                history: vec![],
            },
        }
    } else {
        state.clone()
    }
}

fn bench_timeline_creation(c: &mut Criterion) {
    c.bench_function("timeline_creation", |b| {
        b.iter(|| {
            let initial_state = TimelineState {
                counter: 0,
                history: vec![],
            };
            StateManager::new(initial_state, timeline_reducer)
        })
    });
}

fn bench_timeline_dispatch(c: &mut Criterion) {
    c.bench_function("timeline_dispatch", |b| {
        let initial_state = TimelineState {
            counter: 0,
            history: vec![],
        };
        let mut timeline = StateManager::new(initial_state, timeline_reducer);

        b.iter(|| {
            timeline.dispatch(black_box(TimelineAction::Increment));
        })
    });
}

fn bench_timeline_history_growth(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_history_growth");

    for history_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*history_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(history_size),
            history_size,
            |b, &history_size| {
                let initial_state = TimelineState {
                    counter: 0,
                    history: vec![],
                };
                let mut timeline = StateManager::new(initial_state, timeline_reducer);

                // Build up history
                for i in 0..history_size {
                    timeline.dispatch(TimelineAction::AddHistory(format!("entry_{i}")));
                }

                b.iter(|| {
                    timeline.dispatch(black_box(TimelineAction::Increment));
                })
            },
        );
    }
    group.finish();
}

fn bench_timeline_rewind(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_rewind");

    for history_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(history_size),
            history_size,
            |b, &history_size| {
                let initial_state = TimelineState {
                    counter: 0,
                    history: vec![],
                };
                let mut timeline = StateManager::new(initial_state, timeline_reducer);

                // Build up history
                for _ in 0..history_size {
                    timeline.dispatch(TimelineAction::Increment);
                }

                b.iter(|| {
                    let mut timeline_clone = timeline.clone();
                    timeline_clone.rewind(black_box(history_size / 2));
                })
            },
        );
    }
    group.finish();
}

fn bench_timeline_branch(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_branch");

    for history_size in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(history_size),
            history_size,
            |b, &history_size| {
                let initial_state = TimelineState {
                    counter: 0,
                    history: vec![],
                };
                let mut timeline = StateManager::new(initial_state, timeline_reducer);

                // Build up history
                for _ in 0..history_size {
                    timeline.dispatch(TimelineAction::Increment);
                }

                b.iter(|| {
                    black_box(timeline.branch());
                })
            },
        );
    }
    group.finish();
}

fn bench_timeline_current_state(c: &mut Criterion) {
    let initial_state = TimelineState {
        counter: 0,
        history: vec!["test".to_string(); 100],
    };
    let timeline = StateManager::new(initial_state, timeline_reducer);

    c.bench_function("timeline_current_state", |b| {
        b.iter(|| {
            black_box(timeline.current_state());
        })
    });
}

fn bench_timeline_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("timeline_memory_usage");

    for actions in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(actions),
            actions,
            |b, &actions| {
                b.iter(|| {
                    let initial_state = TimelineState {
                        counter: 0,
                        history: vec![],
                    };
                    let mut timeline = StateManager::new(initial_state, timeline_reducer);

                    for i in 0..actions {
                        timeline.dispatch(TimelineAction::AddHistory(format!("action_{i}")));
                    }

                    black_box(timeline.current_state());
                })
            },
        );
    }
    group.finish();
}

fn bench_timeline_dispatch_after_rewind(c: &mut Criterion) {
    c.bench_function("timeline_dispatch_after_rewind", |b| {
        let initial_state = TimelineState {
            counter: 0,
            history: vec![],
        };

        b.iter(|| {
            let mut timeline = StateManager::new(initial_state.clone(), timeline_reducer);

            // Build history
            for _ in 0..100 {
                timeline.dispatch(TimelineAction::Increment);
            }

            // Rewind
            timeline.rewind(50);

            // Dispatch new action (truncates future history)
            timeline.dispatch(black_box(TimelineAction::Reset));
        })
    });
}

criterion_group!(
    benches,
    bench_timeline_creation,
    bench_timeline_dispatch,
    bench_timeline_history_growth,
    bench_timeline_rewind,
    bench_timeline_branch,
    bench_timeline_current_state,
    bench_timeline_memory_usage,
    bench_timeline_dispatch_after_rewind
);
criterion_main!(benches);
