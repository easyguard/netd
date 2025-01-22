pub mod arp;
mod config;
mod interface;
mod link;

use clap::{arg, command, Parser, Subcommand};
use config::Config;
use futures::future::join_all;
use interface::{bridge::BridgeInterface, ethernet::EthernetInterface};
use link::interface::Interface;
use tokio::process::Command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
	#[arg(short, long)]
	verbose: bool,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Launch the TUI
	TUI {},

	/// Run (do not use, service only)
	RUN {},

	/// Reset networking
	RESET {},

	/// Reload
	RELOAD {},
}

#[tokio::main]
async fn main() {
	let args: Args = Args::parse();

	match args.command {
		Commands::TUI {} => {
			unimplemented!("TUI not implemented yet");
		}
		Commands::RUN {} => {
			run().await;
		}
		Commands::RESET {} => {
			reset().await;
		}
		Commands::RELOAD {} => {
			// Confirm that the user wants to reload
			let confirm = dialoguer::Confirm::new()
				.with_prompt("Are you sure you want to reload?")
				.interact()
				.unwrap();
			if !confirm {
				return;
			}
			reset().await;
			run().await;
		}
	}
}

async fn run() {
	let config = Config::load();
	if config.renames.len() != 0 {
		println!("Renaming {} interfaces!", config.renames.len());
		for entry in config.renames {
			let old_name = entry.0;
			let new_name = entry.1;
			let mut interface = Interface::get_from_name(&old_name);
			if !interface.exists().await {
				panic!("Interface {old_name} does not exist!");
			}
			interface.rename(&new_name).await;
		}
	}

	if std::env::var("NO_LO_UP").is_err() {
		let lo = Interface::get_from_name("lo");
		if !lo.exists().await {
			panic!("Interface lo does not exist!");
		}
		lo.up().await;
	}

	println!("Configuring {} interfaces!", config.interfaces.len());
	let futures: Vec<_> = config
		.interfaces
		.into_iter()
		.map(|entry| {
			let ifconfig = entry.1;
			let name = entry.0.clone();
			async move {
				println!("Configuring interface: {:?}", &name);

				// Wait for all depends to be CONFIGURED
				if let Some(depends) = &ifconfig.shared.depends {
					for depend in depends {
						let depend_interface = Interface::get_from_name(&depend);
						if !depend_interface.exists().await {
							panic!("Interface {depend} does not exist!");
						}
						while depend_interface.get_description().await != "CONFIGURED" {
							println!("[{name}] Waiting for {depend} to be CONFIGURED");
							tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
						}
					}
				}

				match ifconfig.specific {
					config::InterfaceTypeConfig::Ethernet(specific) => {
						let interface = Interface::get_from_name(&name);
						if !interface.exists().await {
							panic!("Interface {name} does not exist!");
						}
						EthernetInterface::configure(&interface, &specific).await;
					}
					config::InterfaceTypeConfig::Bridge(specific) => {
						BridgeInterface::configure(&name, &specific).await;
					}
				}

				// Start services
				for service in &ifconfig.shared.services {
					println!("[{name}] Starting service: {service}");
					let output = Command::new("rc-service")
						.arg(service)
						.arg("start")
						.output()
						.await
						.expect("Failed to execute command");
					println!(
						"[{name}] RC-Service log: {}",
						String::from_utf8_lossy(&output.stdout)
					);
				}
			}
		})
		.collect();

	join_all(futures).await;
}

async fn reset() {
	let config = Config::load();
	println!("Resetting {} interfaces!", config.interfaces.len());

	let futures: Vec<_> = config
		.interfaces
		.into_iter()
		.map(|entry| {
			let ifconfig = entry.1;
			let name = entry.0.clone();
			async move {
				println!("Resetting interface: {:?}", &name);

				// Stop services
				for service in &ifconfig.shared.services {
					println!("[{name}] Stopping service: {service}");
					let output = Command::new("rc-service")
						.arg(service)
						.arg("stop")
						.output()
						.await
						.expect("Failed to execute command");
					println!(
						"[{name}] RC-Service log: {}",
						String::from_utf8_lossy(&output.stdout)
					);
				}

				match ifconfig.specific {
					config::InterfaceTypeConfig::Ethernet(_) => {
						let interface = Interface::get_from_name(&name);
						if !interface.exists().await {
							panic!("Interface {name} does not exist!");
						}
						interface.down().await;
						interface.flush_addresses().await;
						interface.set_description("").await;
					}
					config::InterfaceTypeConfig::Bridge(specific) => {
						let bridge = Interface::get_from_name(&name);
						if !bridge.exists().await {
							return;
						}
						// Bring all subinterfaces down
						for ifname in &specific.interfaces {
							let iface = Interface::get_from_name(ifname);
							if !iface.exists().await {
								continue;
							}
							iface.set_nomaster().await;
							iface.down().await;
						}
						bridge.down().await;
						bridge.flush_addresses().await;
						bridge.set_description("").await;
						bridge.delete().await;
					}
				}
			}
		})
		.collect();

	join_all(futures).await;

	println!("Renaming {} interfaces!", config.renames.len());
	for entry in config.renames {
		let old_name = entry.0;
		let new_name = entry.1;
		let mut interface = Interface::get_from_name(&new_name);
		if !interface.exists().await {
			println!("Interface {new_name} does not exist, ignoring");
		}
		interface.rename(&old_name).await;
	}
}
