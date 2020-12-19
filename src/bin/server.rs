use std::env::current_dir;
use cykv::NoCacheManager;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::str::FromStr;

const SERVER_PORT: u16 = 2958;

fn main() {
	let engine = cykv::CyStore::open(current_dir().unwrap(),
	Box::new(NoCacheManager)).unwrap();
	let addr = SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), SERVER_PORT);
	let server = cykv::Server::new(engine, addr).unwrap();
	server.run().unwrap();
}