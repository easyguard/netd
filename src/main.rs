pub mod arp;
mod config;
mod interface;
mod link;
pub mod hooks;

use std::{io::{Read as _, Write as _}, os::unix::net::{UnixListener, UnixStream}, sync::Arc};

use clap::{arg, command, Parser, Subcommand};
use config::Config;
use futures::future::join_all;
use hooks::run_hook;
use interface::{bridge::BridgeInterface, ethernet::EthernetInterface};
use link::interface::Interface;
use tokio::{process::Command, sync::Mutex};

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
			// reset().await;
			let confirm = dialoguer::Confirm::new()
				.with_prompt("Are you sure you want to reset? All interfaces will be brought down!")
				.interact()
				.unwrap();
			if !confirm {
				return;
			}
			let mut unix_stream =
			UnixStream::connect("/tmp/netd.sock").expect("Could not connect to daemon socket. Is netd running?");
			unix_stream
				.write(b"reset")
				.expect("Failed to write to unix stream");
			println!("Sent reset command to daemon");
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
			// reset().await;
			// run().await;
			let mut unix_stream =
        UnixStream::connect("/tmp/netd.sock").expect("Could not connect to daemon socket. Is netd running?");
				unix_stream
				.write(b"reload")
				.expect("Failed to write to unix stream");
			println!("Sent reload command to daemon");
		}
	}
}

async fn run() {
	let current_config = Arc::new(Mutex::new(Config::load()));
	let configure_thread_cfg = Arc::clone(&current_config);
	let socket_cfg = Arc::clone(&current_config);
	tokio::spawn(async move {
		let config = configure_thread_cfg.lock().await;
		configure(&config).await;
	});
	let socket_path = "/tmp/netd.sock";
	let unix_listener = UnixListener::bind(socket_path).expect("Failed to bind to socket. Is netd already running?");
	loop {
		let (mut unix_stream, socket_addr) = unix_listener
			.accept()
			.expect("Failed to accept connection");
		println!("Accepted connection from {:?}", socket_addr);
		let config = socket_cfg.lock().await;
		handle_stream(&mut unix_stream, &config).await;
	}
}

async fn handle_stream(mut stream: &UnixStream, mut config: &Config) {
	let mut message = String::new();
	stream
		.read_to_string(&mut message)
		.expect("Failed at reading the unix stream");
	let message = message.trim();
	println!("Received message: {:?}", message);
	if message == "reload" {
		reset(&config).await;
		let new_config = Config::load();
		config = &new_config;
		configure(&config).await;
	} else if message == "reset" {
		reset(&config).await;
	}
}

async fn configure(config: &Config) {
	if config.renames.len() != 0 {
		println!("Renaming {} interfaces!", config.renames.len());
		for entry in config.renames.clone() {
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
		.iter()
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

				match &ifconfig.specific {
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

async fn reset(config: &Config) {
	println!("Resetting {} interfaces!", config.interfaces.len());

	let futures: Vec<_> = config
		.interfaces
		.iter()
		.map(|entry| {
			let ifconfig = entry.1;
			let name = entry.0.clone();
			async move {
				println!("Resetting interface: {:?}", &name);

				run_hook(format!("pre-down.{name}"));

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

				match &ifconfig.specific {
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

				run_hook(format!("post-down.{name}"));
			}
		})
		.collect();

	join_all(futures).await;

	println!("Renaming {} interfaces!", config.renames.len());
	for entry in &config.renames {
		let old_name = entry.0;
		let new_name = entry.1;
		let mut interface = Interface::get_from_name(&new_name);
		if !interface.exists().await {
			println!("Interface {new_name} does not exist, ignoring");
		}
		interface.rename(&old_name).await;
	}
}
