#[macro_use] extern crate failure;

#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[macro_use] extern crate ipnet; 
use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use failure::Error;

const ZT_ETHERTYPE_IPV4: u16 = 0x0800;
const ZT_ETHERTYPE_ARP: u16 = 0x0806;
const ZT_ETHERTYPE_IPV6: u16 = 0x86dd;

#[derive(Debug,Serialize, Deserialize)]
struct IpAssignmentPools {
  #[serde(rename = "ipRangeStart")]
  ip_range_start: IpAddr,
  #[serde(rename = "ipRangeEnd")]
  ip_range_end: IpAddr,
}

#[derive(Debug,Serialize, Deserialize)]
struct RootInterface {
  auth: String,
  name: String,
  private: i64,
  #[serde(rename = "allowPassiveBridging")]
  allow_passive_bridging: i8,
  #[serde(rename = "v4AssignMode")]
  v4_assign_mode: String,
  #[serde(rename = "v6AssignMode")]
  v6_assign_mode: String,
  routes: Vec<Routes>,
  #[serde(rename = "ipAssignmentPools")]
  ip_assignment_pools: Vec<IpAssignmentPools>,
  rules: Vec<Rules>,
  capabilities: Vec<Rules>,
  tags: Vec<Rules>,
}

#[derive(Debug,Serialize, Deserialize)]
struct Routes {
  target: IpNet,
  via: Option<IpAddr>,
  flags: u16 ,
  metric: u16 ,
}

#[derive(Debug,Serialize, Deserialize)]
struct Rules {
  #[serde(rename = "etherType")]
  ethtype: u16,
  not: bool,
  or: bool,
  #[serde(rename = "type")]
  rtype: String,
}

impl Routes {
  fn new() -> Self {
   Routes { 
     target: "169.255.0.0/16".parse::<IpNet>().unwrap(),
     via: None ,
     flags:0 ,
     metric: 0,
     }
  }
}

fn main() -> Result<(),Error> {
  let s: IpAddr = "10.10.10.10".parse()?;
  let e: IpAddr = "10.10.10.100".parse()?;
  let n: IpNet = "10.10.10.0/24".parse()?;
  println!("{:?}",n);
  let p: IpAssignmentPools = IpAssignmentPools {ip_range_start: s, ip_range_end: e};
  let r: Routes = Routes {target: n , via: Some(s),flags: 0, metric: 0};
  let j = serde_json::to_string(&p)?;
  let j2 = serde_json::to_string(&r)?;
  let rt = Routes::new();
  let routes = vec![Routes::new(),Routes::new()];
  println!("{:?}",routes);
  println!("{}",j2);
  let j2 = serde_json::to_string(&rt)?;
  println!("{}",j2);
  Ok(())
}