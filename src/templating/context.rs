use rocket_dyn_templates::tera::{to_value, Value};

use std::collections::HashMap;

#[derive(Debug, Default, Serialize)]
pub struct Context {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl Context {
    pub fn from_error_slice(errors: &[String]) -> Context {
        Context {
            errors: errors.to_vec(),
            warnings: Vec::new(),
            extra: HashMap::new(),
        }
    }

    //    pub fn error<T: ToString>(&mut self, value: T) {
    //        self.errors.push(value.to_string());
    //    }
    pub fn error<T: ToString>(&mut self, value: T) {
        self.errors.push(value.to_string().replace('\n', "<br>"));
    }

    pub fn warning<T: ToString>(&mut self, value: T) {
        self.warnings.push(value.to_string().replace('\n', "<br>"));
    }

    pub fn error_owned(&mut self, value: String) {
        self.errors.push(value);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn insert(&mut self, key: &str, value: Value) {
        self.extra.insert(key.to_string(), value);
    }

    pub fn insert_bool(&mut self, key: &str, value: bool) {
        self.extra.insert(key.to_string(), to_value(value).unwrap());
    }

    pub fn insert_string<T: AsRef<str>>(&mut self, key: &str, value: T) {
        self.extra
            .insert(key.to_string(), to_value(value.as_ref()).unwrap());
    }

    pub fn insert_vec<T: serde::Serialize>(&mut self, key: &str, value: Vec<T>) {
        self.extra
            .insert(key.to_string(), to_value(&value).unwrap());
    }
}
