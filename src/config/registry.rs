use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type Func =
    Arc<dyn Fn(Vec<Primitive>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Debug)]
pub enum Primitive {
    Text(String),
    Number(i32),
    Bool(bool),
}

pub struct Registry {
    registry: HashMap<String, Func>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    pub fn add_fn<F, Fut>(&mut self, name: &str, function: F)
    where
        F: Fn(Vec<Primitive>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let name = name.to_string();
        let boxed: Func = Arc::new(move |args| {
            let fut = function(args);
            Box::pin(fut)
        });
        self.registry.insert(name, boxed);
    }

    pub async fn call(&self, name: &str, args: Vec<Primitive>) {
        if let Some(handler) = self.registry.get(name) {
            handler(args).await;
        }
    }
}
