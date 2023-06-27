use anyhow::{anyhow, bail, Result};
use config::Config;
use directories::ProjectDirs;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, fmt::Display, path::PathBuf};


#[derive(Debug, Deserialize)]
pub struct Profile {
    pub username: String,
    pub password: String,
    url: String,
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

impl Profile {
    pub fn url(&self) -> Result<Url> {
        Ok(Url::parse(&self.url)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct JenkinsConfig {
    profile: String,
    profiles: HashMap<String, Profile>,
}

impl JenkinsConfig {
    pub fn config_path(path: &Option<String>) -> Result<PathBuf> {
        if let Some(p) = path {
            // if a config file path is provided, we use it
            return Ok(PathBuf::from(p));
        }
        if let Some(dirs) = ProjectDirs::from("", "", "jenkins-cli") {
            // if no config file was provided, but a standard profile exists
            // then we use that
            for ext in ["toml", "yaml", "yml"] {
                let p = dirs.config_dir().join(format!("config.{}", ext));
                if p.exists() {
                    return Ok(p);
                }
            }
        }
        bail!("No config path provided, and no standard config directory found")
    }

    pub fn new(config_path: &Option<String>) -> Result<Self> {
        let mut builder = Config::builder();

        let cfg_path = Self::config_path(config_path)?;

        if config_path.is_none() {
            if cfg_path.exists() {
                builder = builder.add_source(config::File::from(cfg_path));
            }
        } else {
            builder = builder.add_source(config::File::from(cfg_path));
        }

        builder = builder.add_source(config::Environment::with_prefix("JENKINS_CLI"));

        let config = builder.build()?;

        config.try_deserialize().map_err(anyhow::Error::from)
    }
    
    pub fn select_profile(&mut self, profile: &str) {
        self.profile = profile.to_owned();
    }

    pub fn profile(&self) -> Result<&Profile> {
        self.profiles
            .get(&self.profile)
            .ok_or_else(|| anyhow!("profile not found"))
    }
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "url: {}", self.url)?;
        writeln!(f, "username: {}", self.username)?;
        writeln!(f, "password: {}", "*".repeat(self.password.len()))?;
        write!(f, "aliases: {:?}", self.aliases)
    }
}
