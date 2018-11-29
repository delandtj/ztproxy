extern crate failure;
#[macro_use] extern crate failure_derive;


#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate ipnet; 

use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use failure::Error;

const ZT_ETHERTYPE_IPV4: u16 = 0x0800;
const ZT_ETHERTYPE_ARP: u16 = 0x0806;
const ZT_ETHERTYPE_IPV6: u16 = 0x86dd;

#[derive(Fail, Debug)]
#[fail(display = "Zerotier config validity error code {}. ({})",code,message)]
struct ZTError{
  code: i32,
  message: String,
}

#[derive(Debug,Serialize, Deserialize)]
struct IpAssignmentPools {
  #[serde(rename = "ipRangeStart")]
  ip_range_start: IpAddr,
  #[serde(rename = "ipRangeEnd")]
  ip_range_end: IpAddr,
}

impl Default for IpAssignmentPools{
  fn default() -> Self {
    IpAssignmentPools{
     ip_range_start : "169.255.0.10".parse().unwrap(),
     ip_range_end   : "169.255.0.100".parse().unwrap(),
    }
  }
}

impl IpAssignmentPools {
  pub fn set_range(&mut self, s: IpAddr, e:IpAddr) {
    self.ip_range_start = s;
    self.ip_range_end   = e;
  }
}

#[derive(Debug,Serialize, Deserialize)]
struct RootInterface {
  auth: Option<String>,
  name: String,
  private: bool,
  #[serde(rename = "allowPassiveBridging")]
  allow_passive_bridging: bool,
  #[serde(rename = "v4AssignMode")]
  v4_assign_mode: String,
  #[serde(rename = "v6AssignMode")]
  v6_assign_mode: String,
  routes: Vec<Routes>,
  #[serde(rename = "ipAssignmentPools")]
  ip_assignment_pools: Vec<IpAssignmentPools>,
  rules: Vec<Rules>,
  capabilities: Option<Vec<Rules>>,
  tags: Option<Vec<Rules>>,
}

impl Default for RootInterface {
  fn default() -> RootInterface {
    RootInterface {
      auth: None,
      name: "tfnet".to_owned(),
      private: true,
      allow_passive_bridging: false,
      v4_assign_mode: "zt".to_owned(),
      v6_assign_mode: "none".to_owned(),
      routes: vec!(Routes::default()),
      ip_assignment_pools: vec!(IpAssignmentPools::default()),
      rules: vec!(Rules::default()), 
      capabilities: None,
      tags: None,
    }
  }
}


impl RootInterface {
  /// Added routes for other networks that are not part of the network itself,
  /// need to use an IP __in__ the network to route to. This function verifies
  /// the validity of the routes in the request, before sending it to the
  /// zerotier-controller microservice.
  fn verify_routes(&self) -> Result<(),ZTError> {
    let nets = self.routes.clone() ;
    for r in &self.routes {
      if r.via.is_some() {
        // do we have a net that can contain that gw ?
        for n in &nets{
          if n.target.contains(&r.via.unwrap()){
            return Ok(())
          }
        }
      }
      return Err(ZTError{code:11i32 ,message: "no carrying net for gw".to_string()})
    }
    return Err(ZTError{code:13i32 ,message: "shouldn't get here (line 111)".to_string()})
  }
}

#[derive(Debug,Serialize, Deserialize)]
struct Rules {
  #[serde(rename = "etherType")]
  ethtype: u16,
  #[serde(rename = "not")]
  rnot: bool,
  #[serde(rename = "or")]
  ror: bool,
  #[serde(rename = "type")]
  rtype: String,
}

impl Default for Rules {
  fn default() -> Rules {
    Rules {
      ethtype: ZT_ETHERTYPE_IPV4,
      rnot : false,
      ror  : false,
      rtype: "ACTION_DROP".to_owned(),
    }
  }
}

#[derive(Clone, Debug,Serialize, Deserialize)]
struct Routes {
  target: IpNet,
  via: Option<IpAddr>,
  flags: u16 ,
  metric: u16 ,
}

impl Default for Routes {
  fn default() -> Self {
   Routes { 
     target: "169.255.0.0/16".parse::<IpNet>().unwrap(),
     via: None ,
     flags:0 ,
     metric: 0,
     }
  }
}

impl Routes {

  /// Create a new route entry
  pub fn with(&mut self, target: IpNet, gw: Option<IpAddr>){
    self.target = target;
    self.via = gw;
  }
  
  /// Eventually update flags .
  pub fn set_flag(&mut self, f: u16){
    self.flags = f;
  }

  /// set metric
  pub fn set_metric(&mut self, m: u16){
    self.metric = m;
  }
}

fn main() -> Result<(),Error> {
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