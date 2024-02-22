use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

pub type Response = Vec<FileConfig>;

#[derive(Serialize)]
pub struct FileConfig {
    pub targets: Vec<String>,
    pub labels: HashMap<String,String>,
}