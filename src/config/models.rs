use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/*
    Idea
        Create a trait for functions that has a call method for all
            (Strategy pattern)
            This is used to we can call our generic functions
        Create a function wrapper
            holds any function
        Implement the handler call for function wrapper
            now we can place any function in side a wrapper
            implement the trait on how the funciton is called
            able to call it

*/

pub type FnRegistry = HashMap<String, Box<dyn Function>>;

pub enum PrimitiveParam {
    Text(String),
    Number(i32),
    Bool(bool),
}

pub trait Function: Send + Sync {
    fn call(&self, args: Vec<PrimitiveParam>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

pub struct FunctionWrapper<F> {
    function: F,
    signature: Vec<PrimitiveParam>,
}

impl<F> Function for FunctionWrapper<F>
where
    F: Function,
{
    fn call(&self, args: Vec<PrimitiveParam>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        // Validate params towards the signature
        // Call the function
        // (self.function)(p1, p2, p3)
        Box::pin(async move {
            println!("Hello");
        })
    }
}

pub struct Hub {
    name: String,
    registry: FnRegistry,
}

impl Hub {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.trim_start_matches("/").to_string(),
            registry: HashMap::new(),
        }
    }

    pub fn register(&mut self, function_name: &str, function: Box<dyn Function>) {
        let function_name = function_name.to_string();
        self.registry.insert(function_name, function);
    }

    pub async fn call(&self, function_name: &str, args: Vec<PrimitiveParam>) {
        if let Some(handler) = self.registry.get(function_name) {
            handler.call(args).await;
        }
    }
}

fn test123() {
    async fn greet() {
        println!("Hello");
    }

    async fn greet_me(name: &str) {
        println!("Hello {}", name);
    }

    let mut hub = Hub::new("test");

    let greet_handler = hub.register("greet", Pin::new(Box::new(greet)));
    // hub.register("greet_me", greet_me(name));
}
