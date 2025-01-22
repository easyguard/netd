use crate::{config::BridgeConfig, hooks::run_hook, link::interface::Interface};

use super::generic;

pub struct BridgeInterface {}

impl BridgeInterface {
	pub async fn configure(ifname: &String, ifconfig: &BridgeConfig) {
		let interface = Interface::create(ifname, "bridge").await;
		run_hook(format!("pre-up.{ifname}"));
		interface.up().await;
		run_hook(format!("post-up.{ifname}"));

		// Add all interfaces to the bridge
		for member in ifconfig.interfaces.iter() {
			let member_interface = Interface::get_from_name(member);
			if !member_interface.exists().await {
				panic!("[{ifname}] Subinterface {member} does not exist!");
			}
			member_interface.up().await;
			member_interface.set_master(ifname).await;
		}

		generic::generic_configuration(&ifconfig.generic, &interface).await;
	}
}
