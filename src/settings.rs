use toml;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io::Error};

#[derive(Deserialize, Serialize, Debug)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: String,
    pub main_script: String,
}

impl Project {
    pub fn new(name: String, version: String, description: String, main_script: String) -> Project {
        Project {
            name,
            version,
            description,
            main_script,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub project: Project,
    pub packages: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

impl Config {
    pub fn new(project: Project, packages: HashMap<String, String>, scripts: HashMap<String, String>) -> Config {
        Config {
            project,
            packages,
            scripts,
        }
    }

    pub fn write_to_file(&self, path: &str) -> Result<(), Error> {
        let toml_string = toml::to_string(&self).unwrap();
        fs::write(path, toml_string)
    }

    pub fn load_from_file(path: &str) -> Result<Config, Error> {
        let toml_string = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&toml_string)?;
        Ok(config)
    }
    
}