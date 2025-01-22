// use tokio::process::Command;

use crate::{config::EthernetConfig, hooks::run_hook, link::interface::Interface};

use super::generic;

pub struct EthernetInterface {}

impl EthernetInterface {
	pub async fn configure(interface: &Interface, ifconfig: &EthernetConfig) {
		let ifname = interface.name.clone();
		run_hook(format!("pre-up.{ifname}"));
		interface.up().await;
		run_hook(format!("post-up.{ifname}"));
		generic::generic_configuration(&ifconfig.generic, interface).await;
	}
}
