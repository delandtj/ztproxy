extern crate failure;
#[macro_use] extern crate failure_derive;


#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate ipnet; 

use ipnet::{IpNet,Ipv4Net,Ipv6Net};
use ipnet::PrefixLenError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use failure::Error;

const ZT_ETHERTYPE_IPV4: u16 = 0x0800;
const ZT_ETHERTYPE_ARP: u16 = 0x0806;
const ZT_ETHERTYPE_IPV6: u16 = 0x86dd;

#[derive(Fail, Debug)]
#[fail(display = "Zerotier config validity error code {}. ({})",code,message)]
pub struct ZTError{
  code: i32,
  message: String,
}

#[derive(Debug,Serialize, Deserialize)]
pub struct IpAssignmentPools {
  #[serde(rename = "ipRangeStart")]
  pub ip_range_start: IpAddr,
  #[serde(rename = "ipRangeEnd")]
  pub ip_range_end: IpAddr,
}

impl Default for IpAssignmentPools{
  fn default() -> Self {
    IpAssignmentPools{
     ip_range_start : "169.254.0.10".parse().unwrap(),
     ip_range_end   : "169.254.0.100".parse().unwrap(),
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
pub struct RootInterface {
  pub auth: Option<String>,
  pub name: String,
  pub private: bool,
  #[serde(rename = "allowPassiveBridging")]
  allow_passive_bridging: bool,
  #[serde(rename = "v4AssignMode")]
  v4_assign_mode: String,
  #[serde(rename = "v6AssignMode")]
  v6_assign_mode: String,
  pub routes: Vec<Routes>,
  #[serde(rename = "ipAssignmentPools")]
  pub ip_assignment_pools: Vec<IpAssignmentPools>,
  pub rules: Vec<Rules>,
  pub capabilities: Option<Vec<Rules>>,
  pub tags: Option<Vec<Rules>>,
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
  pub fn verify_routes(&self) -> Result<(),ZTError> {
    let nets = self.routes.clone() ;
    for r in &self.routes {
      if r.via.is_some() {
        // do we have a net that can contain that gw ?
        for n in &nets{
          // we can unwrap here, as it's typechecked before
          if n.target.contains(&r.via.unwrap()){
            return Ok(())
          }
        }
      }
      return Err(ZTError{code:11i32 ,message: "no carrying net for gw".to_string()})
    }
    return Err(ZTError{code:13i32 ,message: "shouldn't get here (fn verify_routes)".to_string()})
  }

  fn new_ipnet(a: IpAddr, m: u8) -> Result<IpNet, PrefixLenError>{
     Ok( match a {
       IpAddr::V4(v4) => IpNet::V4(Ipv4Net::new(v4, m)?),
       IpAddr::V6(v6) => IpNet::V6(Ipv6Net::new(v6, m)?),
     })
  }

//  pub fn with(mut self, n: String, s: IpAddr, e: IpAddr, m: u8 , p: bool) -> Result<Self, ZTError> {
//    // create a zt network with 'reasonable' defaults
//    self.name = n;
//    self.ip_assignment_pools = vec!(IpAssignmentPools {ip_range_start: s, ip_range_end: e});
//    self.private = true;
//    let default_rules = vec!(
//    );
//    Ok(self)
//  } 

#[derive(Default,Debug,Serialize, Deserialize)]
pub struct Rules {
  #[serde(rename = "etherType")]
  pub ethtype: u16,
  #[serde(rename = "not")]
  pub rnot: bool,
  #[serde(rename = "or")]
  pub ror: bool,
  #[serde(rename = "type")]
  pub rtype: String,
}

impl Rules {
  pub fn with(mut self, e: u16, n: bool, o: bool, t: String) -> Result<Self,Error> {
    self.ethtype = e;
    self.rnot = n;
    self.ror = o ;
    self.rtype = t;
    Ok(self)
  }
}

#[derive(Clone, Debug,Serialize, Deserialize)]
pub struct Routes {
  pub target: IpNet,
  pub via: Option<IpAddr>,
  pub flags: u16 ,
  pub metric: u16 ,
}

impl Default for Routes {
  fn default() -> Self {
   Routes { 
     target: "169.255.0.0/16".parse().unwrap(),
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
