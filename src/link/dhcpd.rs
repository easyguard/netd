use rand::{thread_rng, Rng};

pub struct DHCPServer {
	pub start: String,
	pub end: String,
	pub interface: String,
	pub dns: String,
	pub netmask: String,
	pub router: String,
	pub lease: u16,
}

pub fn generate_random_string(length: usize) -> String {
	let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
	let mut rng = thread_rng();
	let random_string: String = (0..length)
		.map(|_| {
			let idx = rng.gen_range(0..charset.len());
			char::from(charset[idx])
		})
		.collect();
	random_string
}

impl DHCPServer {
	pub fn new(
		start: String,
		end: String,
		interface: String,
		dns: String,
		netmask: String,
		router: String,
		lease: u16,
	) -> DHCPServer {
		DHCPServer {
			start,
			end,
			interface,
			dns,
			netmask,
			router,
			lease,
		}
	}
	pub fn start(&self) {
		let _ = std::fs::create_dir("/tmp/dhcp");
		let name = generate_random_string(5);
		let _ = std::fs::write(
			format!("/tmp/dhcp/{name}.conf"),
			format!(
				"start {start}
end {end}
interface {interface}
option dns {dns}
option subnet {netmask}
option router {router}
option lease {lease}",
				start = self.start,
				end = self.end,
				interface = self.interface,
				dns = self.dns,
				netmask = self.netmask,
				router = self.router,
				lease = self.lease
			),
		);
		let _ = std::process::Command::new("udhcpd")
			.arg(format!("/tmp/dhcp/{name}.conf"))
			.spawn();
	}
}
