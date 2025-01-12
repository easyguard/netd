use crate::link::{dhcpc::dhcp_client, interface::Interface};

pub async fn failover(interface: &Interface, ifname: &String) -> bool {
	let mut failover_reconfigured = false;
	interface.set_description("FAILOVER_PROBING").await;
	println!("[{ifname}] Probing for existing router on network");
	let result = dhcp_client(interface, true).await;
	// If the DHCP client failed, no router is on this network, so we should continue configuring the interface like normal
	// If the DHCP client succeeded, we should not configure the interface further and start pinging the router until it stops responding
	//   thats when we should start configuring the interface like normal
	if result {
		interface.set_description("FAILOVER_WAITING").await;
		println!("[{ifname}] Router already on this network, failover mode enabled");
		// Get the gateway IP for this interface
		let gateway = interface.get_gateway().await;
		println!("[{ifname}] Detected gateway: {}", gateway);
		let ping_payload = [0; 8];
		loop {
			let ping = surge_ping::ping(gateway.parse().unwrap(), &ping_payload).await;
			if ping.is_err() {
				println!("[{ifname}] Router is down, beginning normal configuration");
				interface.flush_addresses().await;
				failover_reconfigured = true;
				break;
			} else {
				println!("[{ifname}] Router is still up, continuing failover mode");
				tokio::time::sleep(std::time::Duration::from_secs(2)).await;
			}
		}
	} else {
		println!(
			"[{ifname}] No router found on this network, beginning normal configuration"
		);
	}
	return failover_reconfigured;
}