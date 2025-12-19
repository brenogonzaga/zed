use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use zed::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TodoState {
    pub items: Vec<Todo>,
    pub filter: TodoFilter,
    pub is_loading: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub completed: bool,
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

create_slice! {
    enum_name: TodoActions,
    fn_base: todo,
    state: TodoState,
    initial_state: TodoState {
        items: vec![],
        filter: TodoFilter::All,
        is_loading: false,
        error: None,
    },
    actions: {
        StartLoading,
        AddTodo { text: String },
        ToggleTodo { id: u32 },
        RemoveTodo { id: u32 },
        EditTodo { id: u32, text: String },
        SetFilter { filter: TodoFilter },
        ClearCompleted,
        ToggleAll,
        LoadTodosSuccess { todos: Vec<Todo> },
        LoadTodosError { error: String },
    },
    reducer: |state: &mut TodoState, action: &TodoActions| {
        match action {
            TodoActions::StartLoading => {
                state.is_loading = true;
                state.error = None;
            },
            TodoActions::AddTodo { text } => {
                let new_id = state.items.iter().map(|t| t.id).max().unwrap_or(0) + 1;
                state.items.push(Todo {
                    id: new_id,
                    text: text.clone(),
                    completed: false,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                });
                state.is_loading = false;
                state.error = None;
            },
            TodoActions::ToggleTodo { id } => {
                if let Some(todo) = state.items.iter_mut().find(|t| t.id == *id) {
                    todo.completed = !todo.completed;
                }
                state.error = None;
            },
            TodoActions::RemoveTodo { id } => {
                state.items.retain(|t| t.id != *id);
                state.error = None;
            },
            TodoActions::EditTodo { id, text } => {
                if let Some(todo) = state.items.iter_mut().find(|t| t.id == *id) {
                    todo.text = text.clone();
                }
                state.error = None;
            },
            TodoActions::SetFilter { filter } => {
                state.filter = filter.clone();
            },
            TodoActions::ClearCompleted => {
                state.items.retain(|t| !t.completed);
                state.error = None;
            },
            TodoActions::ToggleAll => {
                let all_completed = state.items.iter().all(|t| t.completed);
                for todo in &mut state.items {
                    todo.completed = !all_completed;
                }
                state.error = None;
            },
            TodoActions::LoadTodosSuccess { todos } => {
                state.items = todos.clone();
                state.is_loading = false;
                state.error = None;
            },
            TodoActions::LoadTodosError { error } => {
                state.is_loading = false;
                state.error = Some(error.clone());
            },
        }
    }
}

impl TodoState {
    pub fn filtered_items(&self) -> Vec<&Todo> {
        match self.filter {
            TodoFilter::All => self.items.iter().collect(),
            TodoFilter::Active => self.items.iter().filter(|t| !t.completed).collect(),
            TodoFilter::Completed => self.items.iter().filter(|t| t.completed).collect(),
        }
    }

    pub fn active_count(&self) -> usize {
        self.items.iter().filter(|t| !t.completed).count()
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|t| t.completed).count()
    }
}

fn simulate_load_todos() -> Result<Vec<Todo>, String> {
    thread::sleep(Duration::from_millis(500));

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    if now.is_multiple_of(10) {
        return Err("Network error".to_string());
    }

    Ok(vec![
        Todo {
            id: 1,
            text: "Learn Rust".to_string(),
            completed: false,
            created_at: 1600000000,
        },
        Todo {
            id: 2,
            text: "Build amazing apps".to_string(),
            completed: true,
            created_at: 1600000001,
        },
    ])
}

fn main() {
    println!("=== Advanced Todo App with Zed ===\n");

    let store = todo_store();

    store.subscribe(|state: &TodoState| {
        println!("📊 State Update:");
        println!("   Items: {} total", state.items.len());
        println!("   Active: {}", state.active_count());
        println!("   Completed: {}", state.completed_count());
        println!("   Filter: {:?}", state.filter);
        println!("   Loading: {}", state.is_loading);
        if let Some(error) = &state.error {
            println!("   ❌ Error: {error}");
        }
        println!();
    });

    let store_clone = Arc::new(store);
    let store_for_async = Arc::clone(&store_clone);

    println!("🔄 Loading initial todos...");
    store_clone.dispatch(TodoActions::StartLoading);

    thread::spawn(move || match simulate_load_todos() {
        Ok(todos) => {
            store_for_async.dispatch(TodoActions::LoadTodosSuccess { todos });
        }
        Err(error) => {
            store_for_async.dispatch(TodoActions::LoadTodosError { error });
        }
    });

    thread::sleep(Duration::from_millis(600));

    println!("➕ Adding new todos...");
    store_clone.dispatch(TodoActions::AddTodo {
        text: "Master state management".to_string(),
    });

    store_clone.dispatch(TodoActions::AddTodo {
        text: "Write comprehensive tests".to_string(),
    });

    println!("✅ Completing a todo...");
    let current_state = store_clone.get_state();
    if let Some(todo) = current_state.items.first() {
        store_clone.dispatch(TodoActions::ToggleTodo { id: todo.id });
    }

    println!("🔍 Filtering active todos...");
    store_clone.dispatch(TodoActions::SetFilter {
        filter: TodoFilter::Active,
    });

    println!("📝 Editing a todo...");
    let current_state = store_clone.get_state();
    if let Some(todo) = current_state.filtered_items().first() {
        store_clone.dispatch(TodoActions::EditTodo {
            id: todo.id,
            text: format!("{} (edited)", todo.text),
        });
    }

    println!("🧹 Clearing completed todos...");
    store_clone.dispatch(TodoActions::SetFilter {
        filter: TodoFilter::All,
    });
    store_clone.dispatch(TodoActions::ClearCompleted);

    println!("🔄 Toggle all remaining todos...");
    store_clone.dispatch(TodoActions::ToggleAll);

    let final_state = store_clone.get_state();
    println!("🏁 Final Summary:");
    println!("   Total todos: {}", final_state.items.len());
    println!(
        "   All completed: {}",
        final_state.items.iter().all(|t| t.completed)
    );

    match serde_json::to_string_pretty(&final_state) {
        Ok(json) => {
            println!("\n📄 Current state as JSON:");
            println!("{json}");
        }
        Err(e) => println!("❌ Failed to serialize state: {e}"),
    }
}
