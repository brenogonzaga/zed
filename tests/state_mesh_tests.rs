use zed::StateNode;

#[derive(Clone, Debug, PartialEq)]
struct TestData {
    value: i32,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_node_creation() {
        let data = TestData {
            value: 42,
            name: "test".to_string(),
        };

        let node = StateNode::new("node1".to_string(), data.clone());

        assert_eq!(node.id, "node1");
        assert_eq!(node.state, data);
        assert!(node.connections.is_empty());
        assert!(node.on_conflict.is_none());
    }

    #[test]
    fn test_state_node_connection() {
        let data1 = TestData {
            value: 1,
            name: "node1".to_string(),
        };
        let data2 = TestData {
            value: 2,
            name: "node2".to_string(),
        };

        let mut node1 = StateNode::new("node1".to_string(), data1);
        let node2 = StateNode::new("node2".to_string(), data2.clone());

        node1.connect(node2);

        assert_eq!(node1.connections.len(), 1);
        assert!(node1.connections.contains_key("node2"));
        assert_eq!(node1.connections["node2"].state, data2);
    }

    #[test]
    fn test_state_node_remove_connection() {
        let data1 = TestData {
            value: 1,
            name: "node1".to_string(),
        };
        let data2 = TestData {
            value: 2,
            name: "node2".to_string(),
        };

        let mut node1 = StateNode::new("node1".to_string(), data1);
        let node2 = StateNode::new("node2".to_string(), data2.clone());

        node1.connect(node2);
        assert_eq!(node1.connections.len(), 1);

        let removed = node1.remove_connection(&"node2".to_string());
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().state, data2);
        assert!(node1.connections.is_empty());
    }

    #[test]
    fn test_state_node_remove_nonexistent_connection() {
        let data = TestData {
            value: 1,
            name: "node1".to_string(),
        };

        let mut node = StateNode::new("node1".to_string(), data);
        let removed = node.remove_connection(&"nonexistent".to_string());

        assert!(removed.is_none());
    }

    #[test]
    fn test_state_node_conflict_resolution_default() {
        let data1 = TestData {
            value: 1,
            name: "original".to_string(),
        };
        let data2 = TestData {
            value: 2,
            name: "updated".to_string(),
        };

        let mut node = StateNode::new("node1".to_string(), data1);
        node.resolve_conflict(data2.clone());

        // Default behavior should replace the state
        assert_eq!(node.state, data2);
    }

    #[test]
    fn test_state_node_conflict_resolution_custom() {
        let data1 = TestData {
            value: 10,
            name: "original".to_string(),
        };
        let data2 = TestData {
            value: 5,
            name: "updated".to_string(),
        };

        let mut node = StateNode::new("node1".to_string(), data1);

        // Set custom conflict resolver that takes the maximum value
        node.set_conflict_resolver(|current: &mut TestData, remote: &TestData| {
            if remote.value > current.value {
                current.value = remote.value;
                current.name = remote.name.clone();
            }
        });

        node.resolve_conflict(data2);

        // Should keep original data since 10 > 5
        assert_eq!(node.state.value, 10);
        assert_eq!(node.state.name, "original");

        // Now try with a larger value
        let data3 = TestData {
            value: 15,
            name: "larger".to_string(),
        };

        node.resolve_conflict(data3.clone());
        assert_eq!(node.state, data3);
    }

    #[test]
    fn test_state_node_merge() {
        let data1 = TestData {
            value: 10,
            name: "node1".to_string(),
        };
        let data2 = TestData {
            value: 20,
            name: "node2".to_string(),
        };

        let mut node1 = StateNode::new("node1".to_string(), data1);
        let node2 = StateNode::new("node2".to_string(), data2.clone());

        node1.merge(&node2);

        // Default merge should replace state
        assert_eq!(node1.state, data2);
    }

    #[test]
    fn test_state_node_propagate_update() {
        let data1 = TestData {
            value: 100,
            name: "master".to_string(),
        };
        let data2 = TestData {
            value: 1,
            name: "slave1".to_string(),
        };
        let data3 = TestData {
            value: 2,
            name: "slave2".to_string(),
        };

        let mut master = StateNode::new("master".to_string(), data1.clone());
        let slave1 = StateNode::new("slave1".to_string(), data2);
        let slave2 = StateNode::new("slave2".to_string(), data3);

        master.connect(slave1);
        master.connect(slave2);

        master.propagate_update();

        // All connected nodes should have master's state
        assert_eq!(master.connections["slave1"].state, data1);
        assert_eq!(master.connections["slave2"].state, data1);
    }

    #[test]
    fn test_state_node_complex_mesh() {
        let data_a = TestData {
            value: 1,
            name: "A".to_string(),
        };
        let data_b = TestData {
            value: 2,
            name: "B".to_string(),
        };
        let data_c = TestData {
            value: 3,
            name: "C".to_string(),
        };

        let mut node_a = StateNode::new("A".to_string(), data_a);
        let mut node_b = StateNode::new("B".to_string(), data_b);
        let node_c = StateNode::new("C".to_string(), data_c.clone());

        // Set up conflict resolution that always takes the higher value
        let resolver = |current: &mut TestData, remote: &TestData| {
            if remote.value > current.value {
                *current = remote.clone();
            }
        };

        node_a.set_conflict_resolver(resolver);
        node_b.set_conflict_resolver(resolver);

        // Connect A -> B, A -> C
        node_a.connect(node_b.clone());
        node_a.connect(node_c.clone());

        // Connect B -> A, B -> C
        node_b.connect(node_a.clone());
        node_b.connect(node_c.clone());

        // Propagate from A (value: 1)
        node_a.propagate_update();

        // B and C should get A's value only if it's higher (it's not)
        assert_eq!(node_a.connections["B"].state.value, 2); // B keeps its value
        assert_eq!(node_a.connections["C"].state.value, 1); // C gets A's value

        // Now update A to have the highest value
        node_a.state.value = 10;
        node_a.propagate_update();

        // Now all should have A's value
        assert_eq!(node_a.connections["B"].state.value, 10);
        assert_eq!(node_a.connections["C"].state.value, 10);
    }
}
