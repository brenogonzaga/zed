use zed::*;

#[derive(Clone, Debug)]
struct ReactiveCounter {
    value: i32,
}

fn main() {
    println!("=== Reactive Cascade Tree Example (Counter) ===");

    let mut reactive_system = reactive::ReactiveSystem::new(ReactiveCounter { value: 0 });

    reactive_system.on("increment".to_string(), |state| {
        state.value += 1;
        println!("[Reactive] Value incremented: {}", state.value);
    });

    reactive_system.on("increment".to_string(), |state| {
        if state.value == 2 {
            println!("[Reactive] Alert: value reached 2!");
        }
    });

    reactive_system.trigger("increment".to_string());
    reactive_system.trigger("increment".to_string());
}
