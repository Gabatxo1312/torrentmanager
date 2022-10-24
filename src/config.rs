use rocket::serde::{Serialize, Deserialize};

use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub collections_dir: PathBuf,
    pub uploads_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            collections_dir: PathBuf::from("../collections"),
            uploads_dir: PathBuf::from("../uploads"),
        }
    }
}
