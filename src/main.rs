mod config;
mod interface;
mod link;
pub mod arp;

use clap::{arg, command, Parser, Subcommand};
use config::Config;
use interface::ethernet::EthernetInterface;
use link::interface::Interface;
use futures::future::join_all;
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
	RELOAD {}
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

	println!("Configuring {} interfaces!", config.interfaces.len());
	let futures: Vec<_> = config.interfaces.into_iter().map(|entry| {
		let ifconfig = entry.1;
		let name = entry.0.clone();
		async move {
			println!("Configuring interface: {:?}", &name);

			match ifconfig.type_ {
				config::InterfaceType::Ethernet => {
					let interface = Interface::get_from_name(&name);
					if !interface.exists().await {
						panic!("Interface {name} does not exist!");
					}
					EthernetInterface::configure(&interface, &ifconfig).await;
				}
				config::InterfaceType::Bridge => {
					unimplemented!();
				}
			}
		}
	}).collect();

	join_all(futures).await;
}

async fn reset() {
	let config = Config::load();
	println!("Resetting {} interfaces!", config.interfaces.len());

	let futures: Vec<_> = config.interfaces.into_iter().map(|entry| {
		let ifconfig = entry.1;
		let name = entry.0.clone();
		async move {
			println!("Resetting interface: {:?}", &name);

			match ifconfig.type_ {
				config::InterfaceType::Ethernet => {
					let interface = Interface::get_from_name(&name);
					if !interface.exists().await {
						panic!("Interface {name} does not exist!");
					}
					interface.down().await;
					interface.flush_addresses().await;
				}
				config::InterfaceType::Bridge => {
					unimplemented!();
				}
			}
		}
	}).collect();

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
