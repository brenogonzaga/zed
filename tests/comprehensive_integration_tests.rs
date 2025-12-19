/// Integration tests combining multiple Zed features.
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zed::*;

// ============== E-Commerce App State ==============

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    quantity: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EcommerceState {
    cart: Vec<Product>,
    user_id: String,
    total_price: f64,
    status: String,
}

#[derive(Clone, Debug)]
enum EcommerceAction {
    AddToCart { product: Product },
    RemoveFromCart { product_id: u32 },
    UpdateQuantity { product_id: u32, quantity: u32 },
    Checkout,
    ClearCart,
}

fn ecommerce_reducer(state: &EcommerceState, action: &EcommerceAction) -> EcommerceState {
    match action {
        EcommerceAction::AddToCart { product } => {
            let mut new_cart = state.cart.clone();

            // Check if product already exists
            if let Some(pos) = new_cart.iter().position(|p| p.id == product.id) {
                new_cart[pos].quantity += product.quantity;
            } else {
                new_cart.push(product.clone());
            }

            let total = new_cart.iter().map(|p| p.price * p.quantity as f64).sum();

            EcommerceState {
                cart: new_cart,
                user_id: state.user_id.clone(),
                total_price: total,
                status: "cart_updated".to_string(),
            }
        }
        EcommerceAction::RemoveFromCart { product_id } => {
            let new_cart: Vec<Product> = state
                .cart
                .iter()
                .filter(|p| p.id != *product_id)
                .cloned()
                .collect();

            let total = new_cart.iter().map(|p| p.price * p.quantity as f64).sum();

            EcommerceState {
                cart: new_cart,
                user_id: state.user_id.clone(),
                total_price: total,
                status: "item_removed".to_string(),
            }
        }
        EcommerceAction::UpdateQuantity {
            product_id,
            quantity,
        } => {
            let mut new_cart = state.cart.clone();
            if let Some(pos) = new_cart.iter().position(|p| p.id == *product_id) {
                new_cart[pos].quantity = *quantity;
            }

            let total = new_cart.iter().map(|p| p.price * p.quantity as f64).sum();

            EcommerceState {
                cart: new_cart,
                user_id: state.user_id.clone(),
                total_price: total,
                status: "quantity_updated".to_string(),
            }
        }
        EcommerceAction::Checkout => EcommerceState {
            cart: state.cart.clone(),
            user_id: state.user_id.clone(),
            total_price: state.total_price,
            status: "checked_out".to_string(),
        },
        EcommerceAction::ClearCart => EcommerceState {
            cart: vec![],
            user_id: state.user_id.clone(),
            total_price: 0.0,
            status: "cart_cleared".to_string(),
        },
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    // ============== E-Commerce Integration ==============

    #[test]
    fn test_ecommerce_workflow() {
        let initial_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store = Store::new(initial_state, Box::new(create_reducer(ecommerce_reducer)));

        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);

        store.subscribe(move |state: &EcommerceState| {
            events_clone.lock().unwrap().push(state.status.clone());
        });

        // Simulate user shopping
        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 1,
                name: "Laptop".to_string(),
                price: 999.99,
                quantity: 1,
            },
        });

        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 2,
                name: "Mouse".to_string(),
                price: 29.99,
                quantity: 2,
            },
        });

        let state = store.get_state();
        assert_eq!(state.cart.len(), 2);
        assert!((state.total_price - 1059.97).abs() < 0.01);

        // Update quantity
        store.dispatch(EcommerceAction::UpdateQuantity {
            product_id: 1,
            quantity: 2,
        });

        let state = store.get_state();
        assert!((state.total_price - 2059.96).abs() < 0.01);

        // Checkout
        store.dispatch(EcommerceAction::Checkout);
        assert_eq!(store.get_state().status, "checked_out");

        // Clear cart
        store.dispatch(EcommerceAction::ClearCart);
        assert_eq!(store.get_state().cart.len(), 0);
        assert_eq!(store.get_state().total_price, 0.0);

        thread::sleep(Duration::from_millis(50));
        let events_list = events.lock().unwrap();
        assert!(events_list.len() >= 5);
    }

    // ============== Multi-Store Coordination ==============

    #[test]
    fn test_multi_store_coordination() {
        let user_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store1 = Arc::new(Store::new(
            user_state.clone(),
            Box::new(create_reducer(ecommerce_reducer)),
        ));

        let store2 = Arc::new(Store::new(
            user_state,
            Box::new(create_reducer(ecommerce_reducer)),
        ));

        // Store 1 operations
        store1.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 1,
                name: "Product A".to_string(),
                price: 100.0,
                quantity: 1,
            },
        });

        // Store 2 operations
        store2.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 2,
                name: "Product B".to_string(),
                price: 200.0,
                quantity: 1,
            },
        });

        // Verify independence
        assert_eq!(store1.get_state().cart.len(), 1);
        assert_eq!(store2.get_state().cart.len(), 1);

        assert_eq!(store1.get_state().cart[0].name, "Product A");
        assert_eq!(store2.get_state().cart[0].name, "Product B");
    }

    // ============== State Snapshots & Recovery ==============

    #[test]
    fn test_state_snapshots() {
        let initial_state = EcommerceState {
            cart: vec![Product {
                id: 1,
                name: "Initial Product".to_string(),
                price: 50.0,
                quantity: 1,
            }],
            user_id: "user_123".to_string(),
            total_price: 50.0,
            status: "initialized".to_string(),
        };

        let store = Store::new(
            initial_state.clone(),
            Box::new(create_reducer(ecommerce_reducer)),
        );

        // Take snapshot
        let snapshot1 = store.get_state();
        assert_eq!(snapshot1.cart.len(), 1);

        // Modify state
        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 2,
                name: "New Product".to_string(),
                price: 75.0,
                quantity: 1,
            },
        });

        let snapshot2 = store.get_state();
        assert_eq!(snapshot2.cart.len(), 2);

        // Verify snapshot1 is independent (it's a clone)
        assert_eq!(snapshot1.cart.len(), 1);
        assert_ne!(snapshot1, snapshot2);
    }

    // ============== Reducer Replacement ==============

    #[test]
    fn test_reducer_replacement_integration() {
        let initial_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store = Store::new(initial_state, Box::new(create_reducer(ecommerce_reducer)));

        // Original behavior
        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 1,
                name: "Product".to_string(),
                price: 100.0,
                quantity: 1,
            },
        });

        assert_eq!(store.get_state().cart.len(), 1);

        // Create a modified reducer that doubles prices
        let double_price_reducer = |state: &EcommerceState, action: &EcommerceAction| match action {
            EcommerceAction::AddToCart { product } => {
                let mut modified_product = product.clone();
                modified_product.price *= 2.0;
                ecommerce_reducer(
                    state,
                    &EcommerceAction::AddToCart {
                        product: modified_product,
                    },
                )
            }
            _ => ecommerce_reducer(state, action),
        };

        store.replace_reducer(Box::new(create_reducer(double_price_reducer)));

        // New behavior with doubled prices
        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 2,
                name: "Expensive Product".to_string(),
                price: 100.0,
                quantity: 1,
            },
        });

        let state = store.get_state();
        assert_eq!(state.cart.len(), 2);
        // Total: 100 + (100 * 2) = 300
        assert!((state.total_price - 300.0).abs() < 0.01);
    }

    // ============== Concurrent Shopping ==============

    #[test]
    fn test_concurrent_shopping() {
        let initial_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store = Arc::new(Store::new(
            initial_state,
            Box::new(create_reducer(ecommerce_reducer)),
        ));

        let mut handles = vec![];

        // Multiple threads shopping concurrently
        for thread_id in 0..5 {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                for i in 0..10 {
                    let product_id = thread_id * 10 + i;
                    store_clone.dispatch(EcommerceAction::AddToCart {
                        product: Product {
                            id: product_id,
                            name: format!("Product_{}", product_id),
                            price: (product_id as f64) * 10.0,
                            quantity: 1,
                        },
                    });
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_state = store.get_state();
        assert_eq!(final_state.cart.len(), 50);
        assert!(final_state.total_price > 0.0);
    }

    // ============== Serialization & Persistence ==============

    #[test]
    fn test_state_persistence_workflow() {
        let initial_state = EcommerceState {
            cart: vec![
                Product {
                    id: 1,
                    name: "Laptop".to_string(),
                    price: 999.99,
                    quantity: 1,
                },
                Product {
                    id: 2,
                    name: "Mouse".to_string(),
                    price: 29.99,
                    quantity: 2,
                },
            ],
            user_id: "user_123".to_string(),
            total_price: 1059.97,
            status: "persisted".to_string(),
        };

        // Simulate saving state
        let json = serde_json::to_string_pretty(&initial_state).expect("Serialization failed");
        assert!(json.contains("Laptop"));
        assert!(json.contains("user_123"));

        // Simulate loading state
        let loaded_state: EcommerceState =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(loaded_state, initial_state);

        // Use loaded state in store
        let store = Store::new(loaded_state, Box::new(create_reducer(ecommerce_reducer)));

        assert_eq!(store.get_state().cart.len(), 2);
        assert_eq!(store.get_state().user_id, "user_123");
    }

    // ============== Subscriber Chaining ==============

    #[test]
    fn test_subscriber_chaining() {
        let initial_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store = Store::new(initial_state, Box::new(create_reducer(ecommerce_reducer)));

        let action_log = Arc::new(Mutex::new(Vec::new()));
        let validation_log = Arc::new(Mutex::new(Vec::new()));

        // First subscriber - logs all actions
        let log_clone = Arc::clone(&action_log);
        store.subscribe(move |state: &EcommerceState| {
            log_clone.lock().unwrap().push(format!(
                "Action: {} | Items: {} | Total: ${:.2}",
                state.status,
                state.cart.len(),
                state.total_price
            ));
        });

        // Second subscriber - validates state
        let val_clone = Arc::clone(&validation_log);
        store.subscribe(move |state: &EcommerceState| {
            let total_calculated: f64 =
                state.cart.iter().map(|p| p.price * p.quantity as f64).sum();

            if (total_calculated - state.total_price).abs() < 0.01 {
                val_clone.lock().unwrap().push("VALID".to_string());
            } else {
                val_clone.lock().unwrap().push("INVALID".to_string());
            }
        });

        // Perform operations
        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 1,
                name: "Test".to_string(),
                price: 50.0,
                quantity: 1,
            },
        });

        store.dispatch(EcommerceAction::AddToCart {
            product: Product {
                id: 2,
                name: "Test2".to_string(),
                price: 75.0,
                quantity: 1,
            },
        });

        thread::sleep(Duration::from_millis(50));

        let logs = action_log.lock().unwrap();
        let validations = validation_log.lock().unwrap();

        assert!(logs.len() >= 2);
        assert!(validations.len() >= 2);
        assert!(validations.iter().all(|v| v == "VALID"));
    }

    // ============== Complex State Transitions ==============

    #[test]
    fn test_complex_state_transitions() {
        let initial_state = EcommerceState {
            cart: vec![],
            user_id: "user_123".to_string(),
            total_price: 0.0,
            status: "initialized".to_string(),
        };

        let store = Store::new(initial_state, Box::new(create_reducer(ecommerce_reducer)));

        let products = vec![
            Product {
                id: 1,
                name: "Product A".to_string(),
                price: 100.0,
                quantity: 1,
            },
            Product {
                id: 2,
                name: "Product B".to_string(),
                price: 200.0,
                quantity: 2,
            },
            Product {
                id: 3,
                name: "Product C".to_string(),
                price: 150.0,
                quantity: 1,
            },
        ];

        // Add products
        for product in products {
            store.dispatch(EcommerceAction::AddToCart { product });
        }

        assert_eq!(store.get_state().cart.len(), 3);

        // Update quantities
        store.dispatch(EcommerceAction::UpdateQuantity {
            product_id: 2,
            quantity: 5,
        });

        let state = store.get_state();
        if let Some(product) = state.cart.iter().find(|p| p.id == 2) {
            assert_eq!(product.quantity, 5);
        }

        // Remove items
        store.dispatch(EcommerceAction::RemoveFromCart { product_id: 1 });
        assert_eq!(store.get_state().cart.len(), 2);

        store.dispatch(EcommerceAction::RemoveFromCart { product_id: 3 });
        assert_eq!(store.get_state().cart.len(), 1);

        // Verify final state
        let final_state = store.get_state();
        assert_eq!(final_state.cart.len(), 1);
        assert_eq!(final_state.cart[0].id, 2);
        assert_eq!(final_state.cart[0].quantity, 5);
    }
}
