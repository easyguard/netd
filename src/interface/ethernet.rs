// use tokio::process::Command;

use crate::{config::EthernetConfig, link::interface::Interface};

use super::generic;

pub struct EthernetInterface {}

impl EthernetInterface {
	pub async fn configure(interface: &Interface, ifconfig: &EthernetConfig) {
		// let ifname = interface.name.clone();
		interface.up().await;
		generic::generic_configuration(&ifconfig.generic, interface).await;
	}
}
