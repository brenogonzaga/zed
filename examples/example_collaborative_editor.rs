use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zed::StateNode;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DocumentState {
    pub content: String,
    pub cursor_positions: HashMap<String, usize>,
    pub version: u64,
    pub last_modified: u64,
    pub author: String,
    pub is_online: bool,
}

impl DocumentState {
    pub fn new(author: String) -> Self {
        Self {
            content: String::new(),
            cursor_positions: HashMap::new(),
            version: 1,
            last_modified: current_timestamp(),
            author,
            is_online: true,
        }
    }

    pub fn insert_text(&mut self, position: usize, text: &str) {
        if position <= self.content.len() {
            self.content.insert_str(position, text);
            self.version += 1;
            self.last_modified = current_timestamp();

            // Update cursor positions for all users
            for cursor_pos in self.cursor_positions.values_mut() {
                if *cursor_pos > position {
                    *cursor_pos += text.len();
                }
            }
        }
    }

    pub fn delete_text(&mut self, start: usize, end: usize) {
        if start < end && end <= self.content.len() {
            self.content.drain(start..end);
            self.version += 1;
            self.last_modified = current_timestamp();

            let deleted_len = end - start;
            // Update cursor positions
            for cursor_pos in self.cursor_positions.values_mut() {
                if *cursor_pos > end {
                    *cursor_pos -= deleted_len;
                } else if *cursor_pos > start {
                    *cursor_pos = start;
                }
            }
        }
    }

    pub fn set_cursor(&mut self, user: String, position: usize) {
        if position <= self.content.len() {
            self.cursor_positions.insert(user, position);
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

// Operational Transform-like conflict resolution
fn resolve_document_conflict(current: &mut DocumentState, remote: &DocumentState) {
    println!(
        "🔄 Resolving conflict between {} and {}",
        current.author, remote.author
    );

    // Last-write-wins based on timestamp and version
    if remote.last_modified > current.last_modified
        || (remote.last_modified == current.last_modified && remote.version > current.version)
    {
        println!("   📝 Accepting remote changes from {}", remote.author);

        // Merge cursor positions
        for (user, pos) in &remote.cursor_positions {
            current.cursor_positions.insert(user.clone(), *pos);
        }

        // Update content and metadata
        current.content = remote.content.clone();
        current.version = remote.version.max(current.version) + 1;
        current.last_modified = current_timestamp();
    } else {
        println!("   📝 Keeping local changes from {}", current.author);

        // Merge just the cursor positions from remote
        for (user, pos) in &remote.cursor_positions {
            if user != &current.author {
                current.cursor_positions.insert(user.clone(), *pos);
            }
        }

        current.version += 1;
        current.last_modified = current_timestamp();
    }
}

fn simulate_user_typing(mut user_node: StateNode<DocumentState>, user_name: &str, text: &str) {
    println!("👤 {user_name} starts typing...");

    for (i, ch) in text.chars().enumerate() {
        thread::sleep(Duration::from_millis(100)); // Simulate typing speed

        let insert_pos = user_node.state.content.len();
        user_node.state.insert_text(insert_pos, &ch.to_string());
        user_node
            .state
            .set_cursor(user_name.to_string(), insert_pos + 1);

        if i % 5 == 0 {
            // Periodically propagate changes
            user_node.propagate_update();
        }
    }

    // Final propagation
    user_node.propagate_update();
    println!("✅ {user_name} finished typing");
}

fn main() {
    println!("=== Collaborative Document Editor with State Mesh ===\n");

    // Create users
    let mut alice_node =
        StateNode::new("alice".to_string(), DocumentState::new("Alice".to_string()));

    let mut bob_node = StateNode::new("bob".to_string(), DocumentState::new("Bob".to_string()));

    let mut charlie_node = StateNode::new(
        "charlie".to_string(),
        DocumentState::new("Charlie".to_string()),
    );

    // Set up conflict resolution for all nodes
    alice_node.set_conflict_resolver(resolve_document_conflict);
    bob_node.set_conflict_resolver(resolve_document_conflict);
    charlie_node.set_conflict_resolver(resolve_document_conflict);

    // Connect all users (full mesh network)
    alice_node.connect(bob_node.clone());
    alice_node.connect(charlie_node.clone());

    bob_node.connect(alice_node.clone());
    bob_node.connect(charlie_node.clone());

    charlie_node.connect(alice_node.clone());
    charlie_node.connect(bob_node.clone());

    println!("🌐 Users connected in mesh network");
    println!("📄 Starting collaborative editing session...\n");

    // Simulate initial document creation by Alice
    alice_node.state.insert_text(0, "Hello World! ");
    alice_node
        .state
        .set_cursor("Alice".to_string(), alice_node.state.content.len());
    alice_node.propagate_update();

    println!(
        "📝 Alice created initial document: \"{}\"",
        alice_node.state.content
    );

    // Simulate concurrent editing
    let handles = vec![
        thread::spawn(move || {
            simulate_user_typing(alice_node, "Alice", "This is a collaborative document. ");
        }),
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200)); // Slight delay
            simulate_user_typing(bob_node, "Bob", "Everyone can edit together! ");
        }),
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(400)); // More delay
            simulate_user_typing(charlie_node, "Charlie", "Amazing real-time collaboration!");
        }),
    ];

    // Wait for all users to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Create a final node to check the merged state
    let mut final_node = StateNode::new(
        "observer".to_string(),
        DocumentState::new("Observer".to_string()),
    );

    final_node.set_conflict_resolver(resolve_document_conflict);

    // Simulate getting updates from all users
    println!("\n🔍 Observer joining and syncing with all users...");

    // In a real scenario, this would come from network synchronization
    // Here we simulate by creating representative final states
    let final_content = "Hello World! This is a collaborative document. Everyone can edit together! Amazing real-time collaboration!";
    let mut final_state = DocumentState::new("Merged".to_string());
    final_state.content = final_content.to_string();
    final_state.cursor_positions.insert("Alice".to_string(), 50);
    final_state.cursor_positions.insert("Bob".to_string(), 80);
    final_state
        .cursor_positions
        .insert("Charlie".to_string(), final_content.len());
    final_state.version = 10;

    final_node.resolve_conflict(final_state);

    // Display final results
    println!("\n📊 Final Document State:");
    println!("Content: \"{}\"", final_node.state.content);
    println!("Version: {}", final_node.state.version);
    println!("Cursor Positions:");
    for (user, pos) in &final_node.state.cursor_positions {
        println!("  - {user}: position {pos}");
    }

    // Demonstrate conflict resolution with simultaneous edits
    println!("\n🔥 Simulating conflict scenario...");

    let mut node_a = StateNode::new(
        "node_a".to_string(),
        DocumentState::new("UserA".to_string()),
    );
    let mut node_b = StateNode::new(
        "node_b".to_string(),
        DocumentState::new("UserB".to_string()),
    );

    node_a.set_conflict_resolver(resolve_document_conflict);
    node_b.set_conflict_resolver(resolve_document_conflict);

    // Both users start with same document
    node_a.state.content = "Original text".to_string();
    node_b.state.content = "Original text".to_string();

    // Simulate offline editing (no immediate sync)
    println!("📱 UserA (offline): Adding at beginning");
    node_a.state.insert_text(0, "PREFIX: ");

    thread::sleep(Duration::from_millis(10)); // Ensure different timestamp

    println!("📱 UserB (offline): Adding at end");
    node_b
        .state
        .insert_text(node_b.state.content.len(), " SUFFIX");

    // Now they come back online and sync
    println!("🌐 Users come back online and sync...");
    node_a.merge(&node_b);

    println!("🎯 Resolved content: \"{}\"", node_a.state.content);
    println!("✅ Collaborative editing session completed!");

    // Serialize final state
    match serde_json::to_string_pretty(&final_node.state) {
        Ok(json) => {
            println!("\n📄 Final state as JSON:");
            println!("{json}");
        }
        Err(e) => println!("❌ Failed to serialize: {e}"),
    }
}
