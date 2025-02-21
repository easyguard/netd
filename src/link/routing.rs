use tokio::process::Command;

pub async fn add_route_via(net: &str, route: &str) {
	let output = Command::new("ip")
		.arg("route")
		.arg("add")
		.arg(route)
		.arg("via")
		.arg(net)
		.output()
		.await
		.expect("Failed to execute command");
	assert!(output.status.success());
}
