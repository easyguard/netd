mod interfaceconfig;
pub use interfaceconfig::*;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct Config {
	#[serde_inline_default(HashMap::new())]
	pub renames: HashMap<String, String>,
	pub interfaces: HashMap<String, InterfaceConfig>,
}

impl Config {
	pub fn load() -> Self {
		let config = std::fs::read_to_string("/etc/config/network.toml").unwrap();
		toml::from_str(&config).unwrap()
	}
}
