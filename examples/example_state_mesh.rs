use zed::*;

#[derive(Clone, Debug)]
struct DocumentState {
    content: String,
}

fn main() {
    println!("=== State Mesh Example (Document) ===");

    let mut node1 = state_mesh::StateNode::new(
        "node1".to_string(),
        DocumentState {
            content: "Hello".into(),
        },
    );
    let node2 = state_mesh::StateNode::new(
        "node2".to_string(),
        DocumentState {
            content: "World".into(),
        },
    );

    node1.connect(node2.clone());

    node1.set_conflict_resolver(|local, remote| {
        local.content = format!("{} {}", local.content, remote.content);
    });

    node1.resolve_conflict(DocumentState {
        content: "from Mesh".into(),
    });

    node1.propagate_update();

    node1.merge(&node2);

    let removed = node1.remove_connection(&"node2".to_string());

    println!("[State Mesh] Node1 state: {:?}", node1.state);
    if let Some(removed_node) = removed {
        println!("[State Mesh] Removed node2 state: {:?}", removed_node.state);
    }
}
