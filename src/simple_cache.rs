#[derive(Clone)]
pub struct SimpleCache<T: Clone> {
    value: Option<T>,
}

impl<T: Clone> SimpleCache<T> {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl<T: Clone> Default for SimpleCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> super::capsule::Cache<T> for SimpleCache<T> {
    fn get(&self) -> Option<T> {
        self.value.clone()
    }
    fn set(&mut self, value: T) {
        self.value = Some(value);
    }
}
