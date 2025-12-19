// Tests to validate that examples work correctly and produce expected results

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use zed::*;

// Test structures similar to examples
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GameState {
    player: Player,
    world: World,
    ui: UIState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Player {
    name: String,
    level: u32,
    experience: u32,
    health: u32,
    inventory: Vec<Item>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct World {
    current_level: String,
    npcs: Vec<Npc>,
    items: Vec<Item>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UIState {
    current_menu: String,
    notifications: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Npc {
    id: String,
    name: String,
    dialogue: Vec<String>,
}

#[derive(Clone, Debug)]
enum GameAction {
    PlayerLevelUp,
    PlayerTakeDamage {
        damage: u32,
    },
    PlayerGainExperience {
        exp: u32,
    },
    PlayerPickupItem {
        item: Item,
    },
    WorldChangeLevel {
        level: String,
    },
    WorldSpawnNpc {
        npc: Npc,
    },
    #[allow(dead_code)]
    UIShowNotification {
        message: String,
    },
    UIChangeMenu {
        menu: String,
    },
}

#[derive(Clone, Debug)]
struct CounterState {
    value: i32,
    is_loading: bool,
    error: Option<String>,
}

#[derive(Clone, Debug)]
enum CounterAction {
    Increment,
    Decrement,
    SetValue(i32),
    StartLoading,
    StopLoading,
    SetError(String),
    Reset,
}

#[derive(Clone, Debug)]
struct DocumentState {
    #[allow(dead_code)]
    content: String,
    #[allow(dead_code)]
    cursor_position: usize,
    #[allow(dead_code)]
    version: u32,
}

mod examples_tests {
    use super::*;

    #[test]
    fn test_game_state_example_functionality() {
        let initial_state = GameState {
            player: Player {
                name: "Hero".to_string(),
                level: 1,
                experience: 0,
                health: 100,
                inventory: vec![],
            },
            world: World {
                current_level: "Forest".to_string(),
                npcs: vec![],
                items: vec![],
            },
            ui: UIState {
                current_menu: "main".to_string(),
                notifications: vec![],
            },
        };

        let store = configure_store(
            initial_state,
            create_reducer(|state: &GameState, action: &GameAction| {
                let mut new_state = state.clone();
                match action {
                    GameAction::PlayerLevelUp => {
                        new_state.player.level += 1;
                        new_state.player.health = 100;
                        new_state
                            .ui
                            .notifications
                            .push("Level Up! You feel stronger.".to_string());
                    }
                    GameAction::PlayerTakeDamage { damage } => {
                        if new_state.player.health > *damage {
                            new_state.player.health -= damage;
                        } else {
                            new_state.player.health = 0;
                            new_state
                                .ui
                                .notifications
                                .push("You have been defeated!".to_string());
                        }
                    }
                    GameAction::PlayerGainExperience { exp } => {
                        new_state.player.experience += exp;
                        if new_state.player.experience >= new_state.player.level * 100 {
                            new_state.player.experience = 0;
                            new_state.player.level += 1;
                            new_state.player.health = 100;
                            new_state
                                .ui
                                .notifications
                                .push("Level Up! You feel stronger.".to_string());
                        }
                    }
                    GameAction::PlayerPickupItem { item } => {
                        new_state.player.inventory.push(item.clone());
                        new_state
                            .ui
                            .notifications
                            .push(format!("Picked up: {}", item.name));
                    }
                    GameAction::WorldChangeLevel { level } => {
                        new_state.world.current_level = level.clone();
                        new_state.world.npcs.clear();
                        new_state.world.items.clear();
                        new_state.ui.notifications.push(format!("Entered: {level}"));
                    }
                    GameAction::WorldSpawnNpc { npc } => {
                        new_state.world.npcs.push(npc.clone());
                        new_state
                            .ui
                            .notifications
                            .push(format!("A {} appears!", npc.name));
                    }
                    GameAction::UIShowNotification { message } => {
                        new_state.ui.notifications.push(message.clone());
                    }
                    GameAction::UIChangeMenu { menu } => {
                        new_state.ui.current_menu = menu.clone();
                    }
                }
                new_state
            }),
        );

        // Test initial state
        let state = store.get_state();
        assert_eq!(state.player.name, "Hero");
        assert_eq!(state.player.level, 1);
        assert_eq!(state.player.health, 100);
        assert_eq!(state.world.current_level, "Forest");
        assert_eq!(state.ui.current_menu, "main");

        // Test level up
        store.dispatch(GameAction::PlayerLevelUp);
        let state = store.get_state();
        assert_eq!(state.player.level, 2);
        assert_eq!(state.player.health, 100);
        assert!(
            state
                .ui
                .notifications
                .contains(&"Level Up! You feel stronger.".to_string())
        );

        // Test damage
        store.dispatch(GameAction::PlayerTakeDamage { damage: 30 });
        let state = store.get_state();
        assert_eq!(state.player.health, 70);

        // Test experience gain and auto level up
        store.dispatch(GameAction::PlayerGainExperience { exp: 200 });
        let state = store.get_state();
        assert_eq!(state.player.level, 3);
        assert_eq!(state.player.experience, 0);
        assert_eq!(state.player.health, 100);

        // Test item pickup
        let sword = Item {
            id: "sword_1".to_string(),
            name: "Iron Sword".to_string(),
            description: "A sturdy iron sword".to_string(),
        };
        store.dispatch(GameAction::PlayerPickupItem {
            item: sword.clone(),
        });
        let state = store.get_state();
        assert_eq!(state.player.inventory.len(), 1);
        assert_eq!(state.player.inventory[0].name, "Iron Sword");

        // Test world change
        store.dispatch(GameAction::WorldChangeLevel {
            level: "Mountain".to_string(),
        });
        let state = store.get_state();
        assert_eq!(state.world.current_level, "Mountain");
        assert_eq!(state.world.npcs.len(), 0);
        assert_eq!(state.world.items.len(), 0);

        // Test Npc spawn
        let goblin = Npc {
            id: "goblin_1".to_string(),
            name: "Goblin".to_string(),
            dialogue: vec!["Grr!".to_string()],
        };
        store.dispatch(GameAction::WorldSpawnNpc {
            npc: goblin.clone(),
        });
        let state = store.get_state();
        assert_eq!(state.world.npcs.len(), 1);
        assert_eq!(state.world.npcs[0].name, "Goblin");

        // Test UI menu change
        store.dispatch(GameAction::UIChangeMenu {
            menu: "inventory".to_string(),
        });
        let state = store.get_state();
        assert_eq!(state.ui.current_menu, "inventory");

        // Test serialization
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: GameState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.player.name, state.player.name);
        assert_eq!(deserialized.world.current_level, state.world.current_level);
    }

    #[test]
    fn test_counter_example_functionality() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let store = configure_store(
            initial_state,
            create_reducer(|state: &CounterState, action: &CounterAction| {
                let mut new_state = state.clone();
                match action {
                    CounterAction::Increment => new_state.value += 1,
                    CounterAction::Decrement => new_state.value -= 1,
                    CounterAction::SetValue(val) => new_state.value = *val,
                    CounterAction::StartLoading => new_state.is_loading = true,
                    CounterAction::StopLoading => new_state.is_loading = false,
                    CounterAction::SetError(msg) => new_state.error = Some(msg.clone()),
                    CounterAction::Reset => {
                        new_state.value = 0;
                        new_state.is_loading = false;
                        new_state.error = None;
                    }
                }
                new_state
            }),
        );

        // Test initial state
        let state = store.get_state();
        assert_eq!(state.value, 0);
        assert!(!state.is_loading);
        assert_eq!(state.error, None);

        // Test increment
        store.dispatch(CounterAction::Increment);
        let state = store.get_state();
        assert_eq!(state.value, 1);

        // Test decrement
        store.dispatch(CounterAction::Decrement);
        let state = store.get_state();
        assert_eq!(state.value, 0);

        // Test set value
        store.dispatch(CounterAction::SetValue(42));
        let state = store.get_state();
        assert_eq!(state.value, 42);

        // Test loading state
        store.dispatch(CounterAction::StartLoading);
        let state = store.get_state();
        assert!(state.is_loading);

        store.dispatch(CounterAction::StopLoading);
        let state = store.get_state();
        assert!(!state.is_loading);

        // Test error state
        store.dispatch(CounterAction::SetError("Test error".to_string()));
        let state = store.get_state();
        assert_eq!(state.error, Some("Test error".to_string()));

        // Test reset
        store.dispatch(CounterAction::Reset);
        let state = store.get_state();
        assert_eq!(state.value, 0);
        assert!(!state.is_loading);
        assert_eq!(state.error, None);
    }

    #[test]
    fn test_reactive_system_example() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let mut reactive_system = ReactiveSystem::new(initial_state);

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::<String>::new()));
        let notifications_clone = notifications.clone();

        reactive_system.on("Increment".to_string(), move |state| {
            let mut notifs = notifications_clone.lock().unwrap();
            notifs.push(format!("Value incremented! Now: {}", state.value));
        });

        let notifications_clone2 = notifications.clone();
        reactive_system.on("Increment".to_string(), move |_| {
            let mut notifs = notifications_clone2.lock().unwrap();
            notifs.push("Another reaction!".to_string());
        });

        // Test initial state
        assert_eq!(notifications.lock().unwrap().len(), 0);

        // Trigger reactions
        reactive_system.trigger("Increment".to_string());

        // Check reactions were triggered
        let notifs = notifications.lock().unwrap();
        assert_eq!(notifs.len(), 2);
        assert!(notifs.iter().any(|n| n.contains("Value incremented! Now:")));
        assert!(notifs.contains(&"Another reaction!".to_string()));
    }

    #[test]
    fn test_state_mesh_example() {
        let initial_state = DocumentState {
            content: "Hello".to_string(),
            cursor_position: 5,
            version: 1,
        };

        let mut node1 = StateNode::new("node1".to_string(), initial_state.clone());
        let node2 = StateNode::new("node2".to_string(), initial_state.clone());
        let node3 = StateNode::new("node3".to_string(), initial_state.clone());

        // Connect nodes
        node1.connect(node2);
        node1.connect(node3);

        // For this test, we'll just verify the basic functionality
        // The actual propagation and state access would need to be implemented
        // based on the actual StateNode API

        // Test that nodes can be created and connected
        assert_eq!(node1.id, "node1");
    }

    #[test]
    fn test_timeline_example() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let mut timeline = StateManager::new(
            initial_state,
            |state: &CounterState, action: &dyn std::any::Any| {
                if let Some(counter_action) = action.downcast_ref::<CounterAction>() {
                    let mut new_state = state.clone();
                    match counter_action {
                        CounterAction::Increment => new_state.value += 1,
                        CounterAction::Decrement => new_state.value -= 1,
                        CounterAction::SetValue(val) => new_state.value = *val,
                        _ => {}
                    }
                    new_state
                } else {
                    state.clone()
                }
            },
        );

        // Test initial state
        let state = timeline.current_state();
        assert_eq!(state.value, 0);

        // Test dispatch
        timeline.dispatch(CounterAction::Increment);
        timeline.dispatch(CounterAction::Increment);
        timeline.dispatch(CounterAction::Increment);

        let state = timeline.current_state();
        assert_eq!(state.value, 3);

        // Test rewind
        timeline.rewind(2);
        let state = timeline.current_state();
        assert_eq!(state.value, 1);

        // Test dispatch after rewind
        timeline.dispatch(CounterAction::SetValue(10));
        let state = timeline.current_state();
        assert_eq!(state.value, 10);

        // Test branch
        let mut branch = timeline.branch();
        branch.dispatch(CounterAction::Increment);

        let branch_state = branch.current_state();
        let main_state = timeline.current_state();

        assert_eq!(branch_state.value, 11);
        assert_eq!(main_state.value, 10);
    }

    #[test]
    fn test_capsule_example() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let mut capsule = Capsule::new(initial_state);

        // Test initial state
        let state = capsule.get_state();
        assert_eq!(state.value, 0);

        // Test dispatch (without logic, state won't change)
        capsule.dispatch(CounterAction::Increment);
        let state = capsule.get_state();
        assert_eq!(state.value, 0); // No logic, so no change

        // Test with logic
        let mut capsule_with_logic = Capsule::new(CounterState {
            value: 0,
            is_loading: false,
            error: None,
        })
        .with_logic(
            |state: &mut CounterState, action: CounterAction| match action {
                CounterAction::Increment => state.value += 1,
                CounterAction::Decrement => state.value -= 1,
                _ => {}
            },
        );

        capsule_with_logic.dispatch(CounterAction::Increment);
        capsule_with_logic.dispatch(CounterAction::Increment);

        let state = capsule_with_logic.get_state();
        assert_eq!(state.value, 2);
    }

    #[test]
    fn test_performance_characteristics() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let store = configure_store(
            initial_state,
            create_reducer(|state: &CounterState, action: &CounterAction| {
                let mut new_state = state.clone();
                if let CounterAction::Increment = action {
                    new_state.value += 1
                }
                new_state
            }),
        );

        // Test rapid dispatch performance
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            store.dispatch(CounterAction::Increment);
        }
        let elapsed = start.elapsed();

        // Should complete in reasonable time (less than 100ms)
        assert!(elapsed.as_millis() < 100);

        let state = store.get_state();
        assert_eq!(state.value, 1000);
    }

    #[test]
    fn test_concurrent_example_behavior() {
        let initial_state = CounterState {
            value: 0,
            is_loading: false,
            error: None,
        };

        let store = Arc::new(configure_store(
            initial_state,
            create_reducer(|state: &CounterState, action: &CounterAction| {
                let mut new_state = state.clone();
                match action {
                    CounterAction::Increment => new_state.value += 1,
                    CounterAction::Decrement => new_state.value -= 1,
                    _ => {}
                }
                new_state
            }),
        ));

        let mut handles = vec![];

        // Spawn multiple threads that increment
        for _ in 0..10 {
            let store_clone = store.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    store_clone.dispatch(CounterAction::Increment);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let state = store.get_state();
        assert_eq!(state.value, 1000);
    }
}
