use zed::*;

#[derive(Clone, Debug)]
struct CapsuleCounterState {
    count: i32,
}

#[derive(Clone, Debug)]
enum CapsuleCounterAction {
    Increment,
    Decrement,
}

fn capsule_logic(state: &mut CapsuleCounterState, action: CapsuleCounterAction) {
    match action {
        CapsuleCounterAction::Increment => state.count += 1,
        CapsuleCounterAction::Decrement => state.count -= 1,
    }
}

fn main() {
    println!("=== Example Domain Capsules (Counter) ===");

    let mut capsule = capsule::Capsule::new(CapsuleCounterState { count: 0 })
        .with_logic(capsule_logic)
        .with_cache(simple_cache::SimpleCache::new());

    capsule.dispatch(CapsuleCounterAction::Increment);
    capsule.dispatch(CapsuleCounterAction::Increment);
    capsule.dispatch(CapsuleCounterAction::Decrement);

    println!("[Capsule] Current state: {:?}", capsule.get_state());
}
