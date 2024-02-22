use std::fmt;
use clap::{arg, command, ArgAction};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub target: Vec<Target>,
    pub endpoint: Vec<Endpoint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Target {
    pub module: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Endpoint {
    pub address: String,
    pub geohash: String,
    pub name: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Targets:\n")?;
        for target in &self.target {
            write!(f, "  Module: {}, URL: {}\n", target.module, target.url)?;
        }

        write!(f, "Endpoints:\n")?;
        for endpoint in &self.endpoint {
            write!(
                f,
                "  Name: {}, Address: {}, Geohash: {}\n",
                endpoint.name, endpoint.address, endpoint.geohash
            )?;
        }

        Ok(())
    }
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let matches = command!("server")
            .arg(
                arg!(--config)
                    .long("config")
                    .short('c')
                    .action(ArgAction::Set)
                    .default_value("config.yaml"),
            )
            .get_matches();

        let config_file = matches.get_one::<String>("config").expect("required");
        load(config_file).expect(&*format!("load file {} error", config_file))
    };
}

fn load(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut f = File::open(file_path)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let config: Config = serde_yaml::from_str(&contents)?;

    Ok(config)
}
