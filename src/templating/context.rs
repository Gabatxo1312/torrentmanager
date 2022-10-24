use rocket_dyn_templates::tera::{to_value, Value};

use std::collections::HashMap;

use crate::{Config, Database};
use crate::utils::find_it;

pub struct GlobalContext {
    pub errors: Vec<String>,
    pub database: Database,
    pub is_qbt_setup: bool,
    pub is_database_loaded: bool,
}

impl GlobalContext {
    pub fn from_config(config: &Config) -> GlobalContext {
        let mut errors: Vec<String> = Vec::new();
        let database = Database::from_dir(&config.collections_dir, &mut errors);
        GlobalContext {
            is_database_loaded: errors.is_empty(),
            is_qbt_setup: find_it("qbt"),
            errors,
            database,
        }
    }

    pub fn local(&self) -> Context {
        let mut context = Context::from_error_slice(&self.errors);
        context.insert("collections", self.database.collections().iter().map(|x| x.name.to_string()).collect());
        context.insert_bool("is_qbt_setup", self.is_qbt_setup);
        context.insert_bool("is_database_loaded", self.is_database_loaded);
        context
    }

    pub fn has_errors(&self) -> bool {
        ! self.errors.is_empty()
    }

    pub fn is_loaded(&self) -> bool {
        self.is_database_loaded && self.is_qbt_setup
    }
}

#[derive(Debug, Serialize)]
pub struct Context {
    pub errors: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Context {
    pub fn from_error_slice(errors: &[String]) -> Context {
        Context {
            errors: errors.iter().map(|x| x.clone()).collect(),
            extra: HashMap::new(),
        }
    }

    pub fn error(&mut self, value: &str) {
        self.errors.push(value.to_string());
    }

    pub fn error_owned(&mut self, value: String) {
        self.errors.push(value);
    }

    pub fn has_errors(&self) -> bool {
        ! self.errors.is_empty()
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.extra.insert(key.to_string(), value);
    }

    pub fn insert_bool(&mut self, key: &str, value: bool) {
        self.extra.insert(key.to_string(), to_value(value).unwrap());
    }

    pub fn insert_string<T: AsRef<str>>(&mut self, key: &str, value: T) {
        self.extra.insert(key.to_string(), to_value(value.as_ref()).unwrap());
    }
}


