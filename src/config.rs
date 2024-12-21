use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub dhcp: DhcpConfig,
	#[serde_inline_default(HashMap::new())]
	pub renames: HashMap<String, String>,
	pub interfaces: HashMap<String, InterfaceConfig>,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct DhcpConfig {
	#[serde_inline_default(7200)]
	pub max_lease_time: u32,
	#[serde_inline_default(3600)]
	pub default_lease_time: u32,
}

impl Default for DhcpConfig {
	fn default() -> Self {
		Self {
			max_lease_time: 7200,
			default_lease_time: 3600,
		}
	}
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceType {
	Ethernet, Bridge
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceMode {
	Static, Dhcp
}

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct InterfaceConfig {
	#[serde_inline_default(InterfaceType::Ethernet)]
	pub type_: InterfaceType,
	pub mode: InterfaceMode,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub address: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub netmask: Option<u8>,
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	#[serde_inline_default(false)]
	pub do_failover: bool,
	#[serde(skip_serializing_if = "InterfaceDhcpConfig::is_disabled")]
	#[serde(default)]
	pub dhcp: InterfaceDhcpConfig,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct InterfaceDhcpConfig {
	#[serde_inline_default(false)]
	pub enabled: bool,
	pub subnet: String,
	pub netmask: String,
	pub router: String,
	pub start: String,
	pub end: String,
	pub dns: String,
}

impl InterfaceDhcpConfig {
	pub fn is_disabled(&self) -> bool {
		!self.enabled
	}
}

impl Default for InterfaceDhcpConfig {
	fn default() -> Self {
		Self {
			enabled: false,
			subnet: "".to_string(),
			netmask: "".to_string(),
			router: "".to_string(),
			start: "".to_string(),
			end: "".to_string(),
			dns: "".to_string()
		}
	}
}

impl Config {
	pub fn load() -> Self {
		let config = std::fs::read_to_string("/etc/config.toml").unwrap();
		toml::from_str(&config).unwrap()
	}
}
