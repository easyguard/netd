use tokio::process::Command;

use crate::link::interface::Interface;

/// Runs a DHCP client on the specified interface
/// Optionally, it can be told to not background itself after obtaining an IP address (or not)
/// That is useful for failover scenarios
/// Note: Check the return value to see if the command was successful
pub async fn dhcp_client(interface: &Interface, no_bg: bool) -> bool {
	let mut cmd = Command::new("udhcpc");
	cmd.arg("-i")
		.arg(&interface.name);
	if no_bg {
		cmd.arg("-n");
		cmd.arg("-q");
	}
	let output = cmd.output().await.expect("Failed to execute command");
	output.status.success()
}