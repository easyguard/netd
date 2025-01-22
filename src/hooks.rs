pub fn run_hook(hook: String) {
	let path = format!("/etc/config/network/hooks/{}", hook);
	let path = path.as_str();
	// Check if the hook exists
	if !std::path::Path::new(path).exists() {
		return;
	}
	// Run the hook (the shebang will determine how it's run)
	let output = std::process::Command::new(path)
		.output()
		.expect("Failed to run hook");
	if !output.status.success() {
		println!("[{hook}] Hook failed: {}", String::from_utf8_lossy(&output.stderr));
	}
	println!("[{hook}] Hook ran successfully");
}
