use std::{net::Ipv4Addr, str::FromStr};

use pnet::{packet::arp::ArpOperations, util::MacAddr};

use crate::{
	arp::send_arp_packet, config::{GenericInterfaceConfig, InterfaceMode}, hooks::run_hook, interface::failover, link::{dhcpc::dhcp_client, dhcpd, interface::Interface, routing}
};

pub async fn generic_configuration(ifconfig: &GenericInterfaceConfig, interface: &Interface) {
	let ifname = interface.name.clone();
	let mut failover_reconfigured = false;
	if ifconfig.do_failover {
		failover_reconfigured = failover::failover(interface, &ifname).await;
		run_hook(format!("post-failover.{ifname}"));
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
		interface
			.add_address(
				&ifconfig.address.as_ref().unwrap(),
				ifconfig.netmask.unwrap(),
			)
			.await;
	}

	if ifconfig.gateway.is_some() {
		routing::add_route_via(ifconfig.gateway.as_ref().unwrap().as_str(), "default").await;
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
		run_hook(format!("pre-dhcp-server.{ifname}"));
		let dhcpserver = dhcpd::DHCPServer::new(
			ifconfig.dhcp.start.clone(),
			ifconfig.dhcp.end.clone(),
			ifname.clone(),
			ifconfig.dhcp.dns.clone(),
			ifconfig.dhcp.netmask.clone(),
			ifconfig.dhcp.router.clone(),
			3600,
		);
		dhcpserver.start();
		run_hook(format!("post-dhcp-server.{ifname}"));
	}
	interface.set_description("CONFIGURED").await;
	run_hook(format!("post-configure.{ifname}"));

	if failover_reconfigured {
		for _ in 0..3 {
			println!("[{ifname}] Sending gratuitous ARP packet");
			send_arp_packet(
				interface,
				Ipv4Addr::from_str("10.10.99.1").unwrap(),
				MacAddr::from_str(&interface.get_mac().await).unwrap(),
				Ipv4Addr::from_str("10.10.99.1").unwrap(),
				MacAddr::from_str("ff:ff:ff:ff:ff:ff").unwrap(),
				ArpOperations::Request,
			);
			tokio::time::sleep(std::time::Duration::from_secs(1)).await;
		}
		run_hook(format!("post-garp.{ifname}"));
	}
}
