//! This is a library for managing your local zerotier daemon, adding
//! controls for the embedded controller and moon features.
//!
//! ## basic usage
//!
//! Once a zerotier daemon runs, you can join a zerotier network ID with
//!     
//!     zerotier-cli join $networkid
//!
//! That command lets the daemon request the information for that network id
//! and register it's own id as a client in that network.
//!
//! The world controllers then allocate a free ip address and send it to the
//! requesting client. If the network is flagged as private, you need to use
//! the ZT GUI to find the id, and allow it to that network so it can receive
//! and send packets on that network.
//!
//! The zerotier daemons of `Planet` are exactly the same as the ones of the
//! clients, except for the fact that they share id's and network
//! configurations in a Rethinkdb Database.
//!
//! So we can run our own controller -that is, activate the embedded controller
//! that is already there- but that one saves its config in Json files.
//!
//!

// TODO: 
//   - is_sibling
//   - vec!(default_rules)
mod commands;

extern crate failure;
#[macro_use]
extern crate failure_derive;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate ipnet;

use ipnet::PrefixLenError;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use failure::Error;

const ZT_ETHERTYPE_IPV4: u16 = 0x0800;
const ZT_ETHERTYPE_ARP: u16 = 0x0806;
const ZT_ETHERTYPE_IPV6: u16 = 0x86dd;

/// we use a specifi error struct so we can give a code for easy location
/// of a failure in the library/binary/api
///
/// where : 1xx = library errors  
///         2xx = cli errors  
///         3xx = api/http errors  
#[derive(Fail, Debug)]
#[fail(
    display = "Zerotier config validity error code {}. ({})",
    code,
    message
)]
pub struct ZTError {
    code: i32,
    message: String,
}

/// Range of addresses to allocate from IPv4/6
#[derive(Debug, Serialize, Deserialize)]
pub struct IpAssignmentPools {
    #[serde(rename = "ipRangeStart")]
    pub ip_range_start: IpAddr,
    #[serde(rename = "ipRangeEnd")]
    pub ip_range_end: IpAddr,
}

/// There is no default for IpAddr, so we set ipv4 LL as default
impl Default for IpAssignmentPools {
    fn default() -> Self {
        IpAssignmentPools {
            ip_range_start: "169.254.0.10".parse().unwrap(),
            ip_range_end: "169.254.0.100".parse().unwrap(),
        }
    }
}

/// If you would want to update your struct with something else
impl IpAssignmentPools {
    pub fn set_range(&mut self, s: IpAddr, e: IpAddr) {
        self.ip_range_start = s;
        self.ip_range_end = e;
    }
}

/// The main struct from which we create the JSON that we're going to send to
/// the zerotier daemon. The serde renames are there 'just' to make the compiler
/// happy and conform to the default snake_case of how to Rust.
#[derive(Debug, Serialize, Deserialize)]
pub struct RootInterface {
    pub auth: Option<String>,
    pub name: Option<String>,
    pub private: bool,
    pub id: Option<String>,
    pub nwid: Option<String>,
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

/// we would not need a default per se, but it can come in handy for the API
impl Default for RootInterface {
    fn default() -> RootInterface {
        RootInterface {
            auth: None,
            name: Some("tfnet".to_owned()),
            private: true,
            id: None,
            nwid: None,
            allow_passive_bridging: false,
            v4_assign_mode: "zt".to_owned(),
            v6_assign_mode: "none".to_owned(),
            routes: vec![Routes::default()],
            ip_assignment_pools: vec![IpAssignmentPools::default()],
            rules: vec![Rules::default()],
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
    pub fn verify_routes(&self) -> Result<(), ZTError> {
        let nets = self.routes.clone();
        for r in &self.routes {
            if r.via.is_some() {
                // do we have a net that can contain that gw ?
                for n in &nets {
                    // we can unwrap here, as it's typechecked before
                    if n.target.contains(&r.via.unwrap()) {
                        return Ok(());
                    }
                }
            }
            return Err(ZTError {
                code: 101i32,
                message: "no carrying net for gw".to_string(),
            });
        }
        return Err(ZTError {
            code: 102i32,
            message: "shouldn't get here (fn verify_routes)".to_string(),
        });
    }

    /// There is no into() for an IpAddr/mask, so we add it here.
    /// TBD: issue pull request for that to the ipnet maintainer.
    fn new_ipnet(a: IpAddr, m: u8) -> Result<IpNet, PrefixLenError> {
        Ok(match a {
            IpAddr::V4(v4) => IpNet::V4(Ipv4Net::new(v4, m)?),
            IpAddr::V6(v6) => IpNet::V6(Ipv6Net::new(v6, m)?),
        })
    }

    /// Are the 2 ipaddrs valid for that range?
    fn validate_sibling() {}

    /// We create a rootinterface with some reasonable defaults,
    /// with private = true
    pub fn with(
        n: Option<String>,
        p: bool,
        s: IpAddr,
        e: IpAddr,
        m: u8,
        nwid: Option<String>,
    ) -> Self {
        let subnet = RootInterface::new_ipnet(s, m);

        // yeah baby !!
        let route = Routes {
            target: subnet.unwrap(),
            ..Default::default()
        };

        let mut pool = IpAssignmentPools::default();
        pool.set_range(s, e);

        let mut r: RootInterface = RootInterface::default();
        if nwid.is_some() {
            r.nwid = nwid.to_owned();
        }
        r.name = n;
        r.private = p;
        r.ip_assignment_pools = vec![pool];
        r.routes = vec![route];
        r
    }

    // end RootInterface
}

/// Zerotier's notion for rules in this format is for 1.2.x clients,
/// We'll adhere to **only** that.
///
/// This means that nodes (clients) that are version <1.2,  
/// won't be able to forward packets in the nets managed
/// by this controller
#[derive(Default, Debug, Serialize, Deserialize)]
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
    /// Create a rule
    pub fn with(e: u16, n: bool, o: bool, t: String) -> Self {
        Rules {
            ethtype: e,
            rnot: n,
            ror: o,
            rtype: t,
        }
    }
}

/// Every net has at least one route, the one that holds the IpAssignmentPool
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Routes {
    pub target: IpNet,
    pub via: Option<IpAddr>,
    pub flags: u16,
    pub metric: u16,
}

impl Default for Routes {
    /// A default for an unset route is the ipv4 LL network
    fn default() -> Self {
        Routes {
            target: "169.254.0.0/16".parse().unwrap(),
            via: None,
            flags: 0,
            metric: 0,
        }
    }
}

impl Routes {
    /// Create a new route entry
    pub fn with(&mut self, target: IpNet, gw: Option<IpAddr>) {
        self.target = target;
        self.via = gw;
    }

    /// Eventually update flags .
    pub fn set_flag(&mut self, f: u16) {
        self.flags = f;
    }

    /// set metric
    pub fn set_metric(&mut self, m: u16) {
        self.metric = m;
    }
}
