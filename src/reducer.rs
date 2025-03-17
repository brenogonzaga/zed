pub trait Reducer<State, Action> {
    fn reduce(&self, state: &State, action: &Action) -> State;
}
