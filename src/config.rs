use libc::exit;
use serde::Deserialize;
use std::fs;
use std::process;

#[derive(Deserialize)]
pub struct Config {
    pub leader_key: String,
    pub groups: Vec<Group>,
}

#[derive(Deserialize)]
pub struct Group {
    pub name: String,
    pub mappings: Vec<Mapping>,
}

#[derive(Deserialize,Debug)]
pub struct Mapping {
    pub keys: String,
    pub kind: String,
    pub command: String,
}

impl Config {
    pub fn new() -> Self {
      // TODO: change to config file from $HOME directory
      let content = match fs::read_to_string("config.toml") {
        Ok(c) => c,
        Err(e) => {
          log::error!("Failed to read file: {}", e);
          process::exit(1);
        }
      };

      let config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
          log::error!("Failed to deserialize config: {}", e);
          process::exit(1);
        }
      };

      return config;
    }

}
