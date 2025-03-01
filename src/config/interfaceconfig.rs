use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

//
// Basic types
//

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InterfaceMode {
	Static,
	Dhcp,
}

//
// Generic
//

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct GenericInterfaceConfig {
	pub mode: InterfaceMode,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub address: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub netmask: Option<u8>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub gateway: Option<String>,
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
	pub start: String,
	pub end: String,
	pub dns: String,
	pub router: String,
	pub netmask: String,
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
			netmask: "".to_string(),
			router: "".to_string(),
			start: "".to_string(),
			end: "".to_string(),
			dns: "".to_string(),
		}
	}
}

//
// Specific Interface Types
//

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct EthernetConfig {
	#[serde(flatten)]
	pub generic: GenericInterfaceConfig,
}

#[serde_inline_default]
#[derive(Serialize, Deserialize)]
pub struct BridgeConfig {
	pub interfaces: Vec<String>,
	#[serde(flatten)]
	pub generic: GenericInterfaceConfig,
}

//
//
//

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum InterfaceTypeConfig {
	Ethernet(EthernetConfig),
	Bridge(BridgeConfig),
}

//
// Shared Interface Config
//

#[derive(Serialize, Deserialize)]
pub struct SharedInterfaceConfig {
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub depends: Option<Vec<String>>,
	#[serde(default)]
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub services: Vec<String>,
}

//
// Interface Config
//

#[derive(Serialize, Deserialize)]
#[serde_inline_default]
pub struct InterfaceConfig {
	#[serde(flatten)]
	pub shared: SharedInterfaceConfig,
	#[serde(flatten)]
	pub specific: InterfaceTypeConfig,
}
