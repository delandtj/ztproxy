/// Command line for setting up a new controller on the
/// ZerotTier daemon that listens on `localhost:9993`
/// 
/// parameters - 
///     -i --netid    : network-id String[16]
///     -s --start-ip : IpAddr
///     -e --end-ip   : IpAddr
///     -n --netmask  : u8
///     -p --private  : true , Default=true
    
///  Create a new network
/// ex: ztnet create -s 10.10.10.10 -e 10.10.10.100 -n 24 -p true # will request creation of a ztnet
///     return : 0, ztnetid or error
///     
///  Add a network range (1 IPv4 and 1 IPv6)
///     ztnet addnet -i ztnetid -s fdab:1234::1:1 -e fdab:1234::f:ff00 -n 64
///  
///  Add a route to a net via (note: carrier net needs to exist)
///     ztnet addroute -i ztnetid 172.22.2.0 -n 24 -g 10.10.10.123
/// 
///  Athorize a client to connect (note: network has to be private)
///     ztnet auth -i ztnetid -c ztclientid
///     return: 0 , warn_netnotprivate or error
/// 
///  Remove node from net
///     ztnet deauth -i ztnetid -c ztclientid
///     return: 0 or error
/// 
///  Remove net from controller
///     ztnet destroy -i ztnetid
///     return: 0 or error
/// 
///  Start a moon for a ztnetid
///     ztnet addmoon -i ztnetid
/// 

extern crate clap;
extern crate ipnet;
extern crate failure;
// extern crate ztproxy;
use ztproxy::*;

use std::net::IpAddr;
use ipnet::IpNet;
use failure::Error;

//use std::error::Error;

use clap::{Arg, App, SubCommand};


fn main() -> Result<(),Error> {
    let matches = App::new("ZeroTier proxy")
            .version("0.1")
            .author("Jan De Landtsheer")
            .about("cli and api to control local zerotier daemon")
            .arg(Arg::with_name("auth"));


  let s: IpAddr = "10.10.10.10".parse()?;
  let e: IpAddr = "10.10.10.100".parse()?;
  let n: IpNet = "10.10.10.0/24".parse()?;
  let p: IpAssignmentPools = IpAssignmentPools {ip_range_start: s, ip_range_end: e};

  let mut rt = Routes::default();
  rt.target = "172.16.1.0/24".parse()?;
  rt.via = Some("10.10.10.254".parse()?);
  let mut rt2 = Routes::default();
  rt2.target = n;
  rt2.set_flag(20u16);
  let rt3 = Routes::default();

  let mut rules = Rules::default();

  let mut network = RootInterface::default();
  network.routes = vec!(rt);
  network.routes.push(rt2);
  network.rules = vec!(rules);
  network.ip_assignment_pools = vec!(p);
  network.verify_routes()?;
  let j = serde_json::to_string(&network)?;
  println!("{}",j);
  Ok(())
}