use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use zed::StateNode;

#[derive(Clone, Debug, PartialEq)]
struct MeshState {
    value: i64,
    version: u64,
    data: Vec<String>,
}

impl MeshState {
    fn new(value: i64) -> Self {
        Self {
            value,
            version: 1,
            data: vec![],
        }
    }
}

fn bench_node_creation(c: &mut Criterion) {
    c.bench_function("state_node_creation", |b| {
        b.iter(|| {
            StateNode::new(
                black_box("node_1".to_string()),
                black_box(MeshState::new(42)),
            )
        })
    });
}

fn bench_node_connection(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_node_connection");

    for node_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            node_count,
            |b, &node_count| {
                b.iter(|| {
                    let mut main_node = StateNode::new("main".to_string(), MeshState::new(0));

                    for i in 0..node_count {
                        let node = StateNode::new(format!("node_{i}"), MeshState::new(i));
                        main_node.connect(node);
                    }

                    black_box(main_node);
                })
            },
        );
    }
    group.finish();
}

fn bench_conflict_resolution(c: &mut Criterion) {
    c.bench_function("state_node_conflict_resolution", |b| {
        let mut node = StateNode::new("node_1".to_string(), MeshState::new(10));

        // Set up last-write-wins conflict resolution
        node.set_conflict_resolver(|current: &mut MeshState, remote: &MeshState| {
            if remote.version > current.version {
                *current = remote.clone();
            }
        });

        b.iter(|| {
            let remote_state = MeshState {
                value: black_box(42),
                version: black_box(100),
                data: vec!["test".to_string()],
            };
            node.resolve_conflict(remote_state);
        })
    });
}

fn bench_state_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_mesh_propagation");

    for node_count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*node_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(node_count),
            node_count,
            |b, &node_count| {
                let mut main_node = StateNode::new("main".to_string(), MeshState::new(0));

                // Create and connect nodes
                for i in 0..node_count {
                    let mut node = StateNode::new(format!("node_{i}"), MeshState::new(i));

                    // Set up conflict resolution for each node
                    node.set_conflict_resolver(|current: &mut MeshState, remote: &MeshState| {
                        if remote.version >= current.version {
                            *current = remote.clone();
                        }
                    });

                    main_node.connect(node);
                }

                b.iter(|| {
                    main_node.propagate_update();
                })
            },
        );
    }
    group.finish();
}

fn bench_mesh_merge(c: &mut Criterion) {
    c.bench_function("state_mesh_merge", |b| {
        let mut node1 = StateNode::new("node1".to_string(), MeshState::new(10));
        let node2 = StateNode::new("node2".to_string(), MeshState::new(20));

        node1.set_conflict_resolver(|current: &mut MeshState, remote: &MeshState| {
            current.value = (current.value + remote.value) / 2; // Average merge
            current.version = current.version.max(remote.version) + 1;
        });

        b.iter(|| {
            node1.merge(black_box(&node2));
        })
    });
}

fn bench_complex_mesh_topology(c: &mut Criterion) {
    c.bench_function("complex_mesh_topology", |b| {
        b.iter(|| {
            // Create a fully connected mesh of 20 nodes
            let mut nodes: Vec<StateNode<MeshState>> = (0..20)
                .map(|i| {
                    let mut node = StateNode::new(format!("node_{i}"), MeshState::new(i));
                    node.set_conflict_resolver(|current: &mut MeshState, remote: &MeshState| {
                        if remote.version > current.version {
                            *current = remote.clone();
                        }
                    });
                    node
                })
                .collect();

            // Connect each node to every other node (full mesh)
            for i in 0..nodes.len() {
                for j in 0..nodes.len() {
                    if i != j {
                        let other_node = nodes[j].clone();
                        nodes[i].connect(other_node);
                    }
                }
            }

            // Propagate updates from first node
            nodes[0].propagate_update();

            black_box(nodes);
        })
    });
}

fn bench_large_state_propagation(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_state_propagation");

    for data_size in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(data_size),
            data_size,
            |b, &data_size| {
                let large_data: Vec<String> =
                    (0..data_size).map(|i| format!("data_item_{i}")).collect();

                let initial_state = MeshState {
                    value: 0,
                    version: 1,
                    data: large_data,
                };

                let mut main_node = StateNode::new("main".to_string(), initial_state);

                // Connect 10 nodes
                for i in 0..10 {
                    let node = StateNode::new(format!("node_{i}"), MeshState::new(i));
                    main_node.connect(node);
                }

                b.iter(|| {
                    main_node.propagate_update();
                })
            },
        );
    }
    group.finish();
}

fn bench_concurrent_conflict_resolution(c: &mut Criterion) {
    c.bench_function("concurrent_conflict_resolution", |b| {
        b.iter(|| {
            let mut node = StateNode::new("node".to_string(), MeshState::new(0));

            // Complex conflict resolution that does more work
            node.set_conflict_resolver(|current: &mut MeshState, remote: &MeshState| {
                // Simulate complex merge logic
                current.value = current.value.max(remote.value);
                current.version = current.version.max(remote.version) + 1;

                // Merge data arrays
                let mut merged_data = current.data.clone();
                merged_data.extend(remote.data.clone());
                merged_data.sort();
                merged_data.dedup();
                current.data = merged_data;
            });

            // Simulate multiple concurrent updates
            for i in 0..100 {
                let remote_state = MeshState {
                    value: i,
                    version: i as u64,
                    data: vec![format!("item_{}", i)],
                };
                node.resolve_conflict(remote_state);
            }

            black_box(node);
        })
    });
}

criterion_group!(
    benches,
    bench_node_creation,
    bench_node_connection,
    bench_conflict_resolution,
    bench_state_propagation,
    bench_mesh_merge,
    bench_complex_mesh_topology,
    bench_large_state_propagation,
    bench_concurrent_conflict_resolution
);
criterion_main!(benches);
