use std::env;
use std::sync::{Arc, RwLock};
use crate::prelude::*;

pub struct KeyChain {
    keys: Arc<RwLock<HashMap<String, String>>>,
}

impl KeyChain {
    pub fn from_env() -> Self {
        let keys = env::vars()
            .into_iter()
            .filter(|(name, _)| name.ends_with("_API_KEY"))
            .map(|(name, value)| {
                let name = name.trim_end_matches("_API_KEY");
                (name.to_string(), value)
            })
            .collect::<HashMap<String, String>>();
        dbg!(&keys);
        Self { keys: Arc::new(RwLock::new(keys)) }
    }

    pub fn get_api_key(&self, name: &str) -> Option<String> {
        self.keys.read().unwrap().get(name).cloned()
    }
}
