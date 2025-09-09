use std::collections::HashMap;

pub type Func = Box<dyn Fn(Vec<Primitive>) + Send + Sync>;

#[derive(Debug)]
pub enum Primitive {
    Text(String),
    Number(i32),
    Bool(bool),
}

pub struct Hub {
    name: String,
    registry: HashMap<String, Func>,
}

impl Hub {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.trim_start_matches("/").to_string(),
            registry: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, name: &str, function: F)
    where
        F: Fn(Vec<Primitive>) + Send + Sync + 'static,
    {
        let name = name.to_string();
        let boxed = Box::new(function);
        self.registry.insert(name, boxed);
    }

    pub fn call(&self, name: &str, args: Vec<Primitive>) {
        // Validate params
        if let Some(handler) = self.registry.get(name) {
            handler(args);
        }
    }
}
