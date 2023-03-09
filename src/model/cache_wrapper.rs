pub struct CacheWrapper<T> {
    content: Option<T>,
}

impl<T> CacheWrapper<T> {
    pub fn new() -> Self {
        Self { content: None }
    }

    pub fn content(&self) -> &T {
        match &self.content {
            Some(val) => val,
            None => panic!("Content requested while empty"),
        }
    }

    pub fn set(&mut self, value: T) {
        self.content = Some(value);
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_none()
    }
}
