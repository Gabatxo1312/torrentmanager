use rocket_dyn_templates::tera::{Function as TeraFunction, Result as TeraResult, Value};
use rocket_dyn_templates::tera::{from_value, to_value};

use std::collections::HashMap;
use std::str::FromStr;

use crate::required_arg;

#[derive(Debug, Serialize)]
pub struct MenuEntry {
    pub link: String,
    pub icon: String,
    pub name: String,
}

impl FromStr for MenuEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<MenuEntry, Self::Err> {
        match s {
            "collections" => Ok(MenuEntry {
                link: "/collections".to_string(),
                icon: "fa-dropbox".to_string(),
                name: "Collections".to_string(),
            }), "progress" => Ok(MenuEntry {
                link: "/progress".to_string(),
                icon: "fa-spinner".to_string(),
                name: "Progress".to_string(),
            }), "logs" => Ok(MenuEntry {
                link: "/logs".to_string(),
                icon: "fa-file-text-o".to_string(),
                name: "Logs".to_string(),
            }), "translations" => Ok(MenuEntry {
                link: "/translations".to_string(),
                icon: "fa-language".to_string(),
                name: "Translations".to_string(),
            }), "upload" => Ok(MenuEntry {
                link: "/upload".to_string(),
                icon: "fa-plus-square".to_string(),
                name: "Upload".to_string(),
            }), "reviews" => Ok(MenuEntry {
                link: "/reviews".to_string(),
                icon: "fa-stethoscope".to_string(),
                name: "Reviews".to_string(),
            }), "chores1" => Ok(MenuEntry {
                link: "/chores".to_string(),
                icon: "fa-medkit".to_string(),
                name: "Chores".to_string(),
            }), "chores2" => Ok(MenuEntry {
                link: "/chores".to_string(),
                icon: "fa-textpattern".to_string(),
                name: "Chores".to_string(),
            }), _ => {
                Err(format!("Unknown menu entry: {}", s))
            }
        }
    }
}

pub struct EntryFn;

impl TeraFunction for EntryFn {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let name = required_arg!(
            String,
            args.get("name"),
            "`` requires a `name` argument with a string value"
        );

        let entry = MenuEntry::from_str(&name)?;

        Ok(to_value(&entry).unwrap())
    }
}

pub struct MenuFn;

impl TeraFunction for MenuFn {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let entries: Vec<MenuEntry> = required_arg!(
            Vec<String>,
            args.get("entries"),
            "`` requires an `entries` argument with a list of string values"
        ).iter().map(|x| MenuEntry::from_str(x).expect(&format!("Invalid menu entry {}", x))).collect();

        Ok(to_value(&entries).unwrap())
    }
}
