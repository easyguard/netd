use tokio::process::Command;

use crate::{config::{InterfaceConfig, InterfaceMode}, link::{dhcpc::dhcp_client, dhcpd, interface::Interface}};

pub struct EthernetInterface {}

impl EthernetInterface {
	pub async fn configure(interface: &Interface, ifconfig: &InterfaceConfig) {
		let ifname = interface.name.clone();
		interface.up().await;
		if ifconfig.do_failover {
			// unimplemented!("Failover not implemented yet");
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
						break;
					} else {
						println!("[{ifname}] Router is still up, continuing failover mode");
						tokio::time::sleep(std::time::Duration::from_secs(2)).await;
					}
				}
			} else {
				println!("[{ifname}] No router found on this network, beginning normal configuration");
			}
		}
		interface.set_description("CONFIGURING").await;
		if ifconfig.mode == InterfaceMode::Dhcp {
			println!("[{ifname}] Obtaining DHCP lease");
			let result = dhcp_client(interface, false).await;
			if !result {
				println!("[{ifname}] Could not get DHCP lease");
			} else {
				println!("[{ifname}] Got DHCP lease"); // TODO: show the IP address
			}
		} else {
			if ifconfig.address.is_none() || ifconfig.netmask.is_none() {
				panic!("[{ifname}] Static interface configuration requires an address and a netmask");
			}
			interface.add_address(&ifconfig.address.as_ref().unwrap(), ifconfig.netmask.unwrap()).await;
		}

		println!("[{ifname}] DHCP: {:?}", ifconfig.dhcp.enabled);

		if ifconfig.dhcp.enabled {
			// Run a DHCP Server
			// println!("WARNING: DHCP Server not fully implemented yet! Running dhcpd service");
			// let mut status = Command::new("rc-service")
			// 	.arg("dhcpd")
			// 	.arg("start")
			// 	.spawn()
			// 	.expect("Failed to start service!");
			let dhcpserver = dhcpd::DHCPServer::new(
				ifconfig.dhcp.start.clone(),
				ifconfig.dhcp.end.clone(),
				ifname.clone(),
				ifconfig.dhcp.dns.clone(),
				ifconfig.dhcp.netmask.clone(),
				ifconfig.dhcp.router.clone(),
				3600
			);
			dhcpserver.start();
		}
		interface.set_description("CONFIGURED").await;
	}
}