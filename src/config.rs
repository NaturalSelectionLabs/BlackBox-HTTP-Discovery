use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub target: Vec<Target>,
    pub endpoint: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Target {
    pub module: String,
    pub url: String,
    #[serde(default = "default_labels")]
    pub labels: HashMap<String, String>,
}

fn default_labels() -> HashMap<String, String> {
    let mut labels = HashMap::new();
    labels.insert("default".to_string(), "true".to_string());
    labels
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Endpoint {
    pub address: String,
    pub geohash: String,
    pub name: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Targets:")?;
        for target in &self.target {
            writeln!(
                f,
                "  Module: {}, URL: {}, Labels: {:?}",
                target.module, target.url, target.labels
            )?;
        }

        writeln!(f, "Endpoints:")?;
        for endpoint in &self.endpoint {
            writeln!(
                f,
                "  Name: {}, Address: {}, Geohash: {}",
                endpoint.name, endpoint.address, endpoint.geohash
            )?;
        }

        Ok(())
    }
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct Args {
    #[arg(short, long)]
    config: String,
}

impl Args {
    pub fn load_config(&self) -> Result<Config, Box<dyn std::error::Error>> {
        load(&self.config)
    }
}

fn load(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut f = File::open(file_path)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let config: Config = serde_yaml::from_str(&contents)?;

    Ok(config)
}
