// use std::process::Command;

use tokio::process::Command;

pub struct Interface {
	pub name: String,
}

impl Interface {
	pub fn get_from_name(name: &str) -> Interface {
		Interface {
			name: name.to_string(),
		}
	}

	pub async fn exists(&self) -> bool {
		// Check if the interface exists
		// run ip link show <name> command
		// and check exit code
		let output = Command::new("ip")
			.arg("link")
			.arg("show")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		output.status.success()
	}

	pub async fn rename(&mut self, new_name: &str) {
		// Rename the interface
		// run ip link set <name> name <new_name> command
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg(&self.name)
			.arg("name")
			.arg(new_name)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
		self.name = new_name.to_string();
	}

	pub async fn up(&self) {
		// Bring the interface up
		// run ip link set <name> up command
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg(&self.name)
			.arg("up")
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	pub async fn down(&self) {
		// Bring the interface down
		// run ip link set <name> down command
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg(&self.name)
			.arg("down")
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	pub async fn is_up(&self) -> bool {
		// Check if the interface is up
		// run ip link show <name> command
		// and check if the output contains "UP"
		let output = Command::new("ip")
			.arg("link")
			.arg("show")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
		output.contains("UP")
	}

	pub async fn add_address(&self, address: &String, mask: u8) {
		// Add an IP address to the interface
		// run ip addr add <address>/<mask> dev <name> command
		let output = Command::new("ip")
			.arg("addr")
			.arg("add")
			.arg(format!("{}/{}", address, mask))
			.arg("dev")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	pub async fn flush_addresses(&self) {
		let output = Command::new("ip")
			.arg("addr")
			.arg("flush")
			.arg("dev")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	pub async fn get_gateway(&self) -> String {
		let output = Command::new("ip")
			.arg("route")
			.arg("show")
			.arg("dev")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");

		// STDOUT looks like this:
		// default via <gateway> dev <name> ......
		// We want to extract the <gateway> part
		let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
		let parts: Vec<&str> = output.split_whitespace().collect();
		let gateway = parts[2];
		gateway.to_string()
	}

	pub async fn set_description(&self, description: &str) {
		// Set the interface description
		// run ip link set <name> alias <description> command
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg("dev")
			.arg(&self.name)
			.arg("alias")
			.arg(format!("\"{}\"", description))
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}
}
