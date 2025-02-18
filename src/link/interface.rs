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

	pub async fn create(name: &str, r#type: &str) -> Interface {
		let output = Command::new("ip")
			.arg("link")
			.arg("add")
			.arg(name)
			.arg("type")
			.arg(r#type)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
		Interface {
			name: name.to_string(),
		}
	}

	/// Check if the interface exists
	pub async fn exists(&self) -> bool {
		let output = Command::new("ip")
			.arg("link")
			.arg("show")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		output.status.success()
	}

	/// Rename the interface.
	/// Must be done before bringing the interface up, otherwise it will fail
	pub async fn rename(&mut self, new_name: &str) {
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

	/// Bring the interface up
	pub async fn up(&self) {
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

	/// Bring the interface down
	pub async fn down(&self) {
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

	/// Check if the interface is up
	pub async fn is_up(&self) -> bool {
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

	/// Add an IP address to the interface
	pub async fn add_address(&self, address: &String, mask: u8) {
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

	/// Flush all addresses from the interface.
	/// Removes all IP addresses from the interface
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

	/// Get the gateway for the interface.
	/// This checks the routing table for this interface and extracts the default route
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

	/// Set the interface description
	pub async fn set_description(&self, description: &str) {
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

	/// Get the interface description
	pub async fn get_description(&self) -> String {
		let output = Command::new("ip")
			.arg("link")
			.arg("show")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");

		let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
		let parts: Vec<&str> = output.split_whitespace().collect();
		let description = parts[parts.iter().position(|&x| x == "alias").unwrap_or_default() + 1];
		let description = if description.len() > 2 {
			&description[1..description.len() - 1]
		} else {
			""
		};
		description.to_string()
	}

	pub async fn get_mac(&self) -> String {
		let output = Command::new("ip")
			.arg("link")
			.arg("show")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");

		let output = String::from_utf8(output.stdout).expect("Invalid UTF-8");
		let parts: Vec<&str> = output.split_whitespace().collect();
		let mac = parts[parts.iter().position(|&x| x == "link/ether").unwrap() + 1];
		mac.to_string()
	}

	/// Delete the interface
	pub async fn delete(&self) {
		let output = Command::new("ip")
			.arg("link")
			.arg("delete")
			.arg("dev")
			.arg(&self.name)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	/// Sets the master of the interface
	pub async fn set_master(&self, master: &str) {
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg("dev")
			.arg(&self.name)
			.arg("master")
			.arg(master)
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success());
	}

	/// Removes the master of the interface
	pub async fn set_nomaster(&self) {
		let output = Command::new("ip")
			.arg("link")
			.arg("set")
			.arg("dev")
			.arg(&self.name)
			.arg("nomaster")
			.output()
			.await
			.expect("Failed to execute command");
		assert!(output.status.success())
	}
}
