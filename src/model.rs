use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Response = Vec<FileConfig>;

#[derive(Serialize, Deserialize)]
pub struct FileConfig {
    pub targets: Vec<String>,
    pub labels: HashMap<String, String>,
}
