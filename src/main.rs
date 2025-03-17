use zed::reducer::Reducer;
use zed::store::Store;

#[derive(Clone, Debug)]
struct MyState {
    counter: i32,
}

#[derive(Clone, Debug)]
enum MyAction {
    Increment,
    Decrement,
    SetValue(i32),
}

struct MyReducer;

impl Reducer<MyState, MyAction> for MyReducer {
    fn reduce(&self, state: &MyState, action: &MyAction) -> MyState {
        let mut new_state = state.clone();
        match action {
            MyAction::Increment => new_state.counter += 1,
            MyAction::Decrement => new_state.counter -= 1,
            MyAction::SetValue(val) => new_state.counter = *val,
        }
        new_state
    }
}

fn main() {
    let initial_state = MyState { counter: 0 };

    let store = Store::new(initial_state, Box::new(MyReducer));

    store.subscribe(|state| {
        println!("Estado atualizado: {:?}", state);
    });

    store.dispatch(MyAction::Increment);
    store.dispatch(MyAction::Increment);
    store.dispatch(MyAction::Decrement);
    store.dispatch(MyAction::SetValue(42));

    println!("Estado final: {:?}", store.get_state());
}
