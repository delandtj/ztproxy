/// Command line for setting up a new controller on the
/// ZerotTier daemon that listens on `localhost:9993`
///
/// parameters -
///     -i --netid    : network-id String[16]
///     -s --start-ip : IpAddr
///     -e --end-ip   : IpAddr
///     -n --netmask  : u8
///     -p --private  : true , Default=true
///
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
extern crate failure;
extern crate ipnet;

// extern crate ztproxy;
use ztproxy::*;

// use std::net::IpAddr;
use failure::Error;

// use std::error::Error;

use clap::{App, Arg, SubCommand};

fn get_params() -> clap::ArgMatches<'static> {
    let matches = App::new("ZeroTier proxy")
        .version("0.1")
        .author("Jan De Landtsheer <jan@threefoldtech.com>")
        .about("cli and api to control local zerotier daemon")
        .usage("ztproxy <COMMAND> [params]\n    Use --help (-h) to see your options")
        .subcommand(
            SubCommand::with_name("create")
                .about("Create a new network")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .long("name")
                        .takes_value(true)
                        .required(true)
                        .help("Give it a name"),
                ).arg(
                    Arg::with_name("start")
                        .short("s")
                        .long("start")
                        .takes_value(true)
                        .required(true)
                        .help("Start ip of range in network"),
                ).arg(
                    Arg::with_name("end")
                        .short("e")
                        .long("end")
                        .takes_value(true)
                        .required(true)
                        .help("End ip of range in network"),
                ).arg(
                    Arg::with_name("mask")
                        .short("m")
                        .long("mask")
                        .takes_value(true)
                        .required(true)
                        .help("Network mask in bits "),
                ).arg(
                    Arg::with_name("private")
                        .short("p")
                        .long("private")
                        .takes_value(false)
                        .required(false)
                        .help("Is that network private"),
                ),
        ).subcommand(
            SubCommand::with_name("addsubnet")
                .about("Add a new subnet rand range")
                .arg(
                    Arg::with_name("nwid")
                        .short("i")
                        .long("nwid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier address of network"),
                ).arg(
                    Arg::with_name("start")
                        .short("s")
                        .long("start")
                        .takes_value(true)
                        .required(true)
                        .help("Start ip of range in network"),
                ).arg(
                    Arg::with_name("end")
                        .short("e")
                        .long("end")
                        .takes_value(true)
                        .required(true)
                        .help("End ip of range in network"),
                ).arg(
                    Arg::with_name("mask")
                        .short("n")
                        .long("mask")
                        .takes_value(true)
                        .required(true)
                        .help("Network mask in bits "),
                ),
        ).subcommand(
            SubCommand::with_name("addroute")
                .about("Add a route for a network")
                .arg(
                    Arg::with_name("destination net")
                        .short("d")
                        .long("destnet")
                        .takes_value(true)
                        .required(true)
                        .help("Subnet to reach"),
                ).arg(
                    Arg::with_name("mask")
                        .short("n")
                        .long("mask")
                        .takes_value(true)
                        .required(true)
                        .help("Network mask in bits "),
                ).arg(
                    Arg::with_name("gateway")
                        .short("g")
                        .long("gateway")
                        .takes_value(true)
                        .required(true)
                        .help("Through which ip to reach that net"),
                ),
        ).subcommand(
            SubCommand::with_name("auth")
                .about("Authorize a client node")
                .arg(
                    Arg::with_name("ztnetid")
                        .short("i")
                        .long("ztnetid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier address of network"),
                ).arg(
                    Arg::with_name("clientid")
                        .short("c")
                        .long("clid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier client id"),
                ),
        ).subcommand(
            SubCommand::with_name("deauth")
                .about("Un-authorize a client node")
                .arg(
                    Arg::with_name("ztnetid")
                        .short("i")
                        .long("ztnetid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier address of network"),
                ).arg(
                    Arg::with_name("clientid")
                        .short("c")
                        .long("clid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier client id"),
                ),
        ).subcommand(
            SubCommand::with_name("destroy")
                .about("Destroy a network on the local controller")
                .arg(
                    Arg::with_name("ztnetid")
                        .short("i")
                        .long("ztnetid")
                        .takes_value(true)
                        .required(true)
                        .help("Zerotier address of network"),
                ).arg(
                    Arg::with_name("need to be sure")
                        .long("force")
                        .required(true)
                        .help("No force ? no delete!"),
                ),
        ).get_matches();
    matches
}

fn main() -> Result<(), Error> {
    let matches = get_params();
    let mut p: bool = false;
    match matches.subcommand() {
        // Create a NEW network
        ("create", Some(m)) => {
            let name = m.value_of("name").unwrap();
            let start = m.value_of("start").unwrap();
            let end = m.value_of("end").unwrap();
            let mask = m.value_of("mask").unwrap();
            if m.is_present("private") {
                p = true;
            }
            let r = RootInterface::with(
                Some(name.to_string()),
                p,
                start.parse().unwrap(),
                end.parse().unwrap(),
                mask.parse().unwrap(),
                None,
            );

            let j = serde_json::to_string(&r)?;
            println!("{}", j);
            //return Ok(commands::newnet(&r));
        }

        // Add a subnet to an nwid
        ("addsubnet", Some(m)) => {
            let start = m.value_of("start").unwrap();
            let end = m.value_of("end").unwrap();
            let mask = m.value_of("mask").unwrap();
            let nwid = m.value_of("nwid").unwrap();
            let mut r = RootInterface::with(
                None,
                p,
                start.parse().unwrap(),
                end.parse().unwrap(),
                mask.parse().unwrap(),
                None,
            );
            r.nwid = Some(nwid.to_string());
            //return Ok(commands::update_net(&r));
        }
        ("", None) => println!("No command entered \n{}",matches.usage()),
        //println!("no command used"),
        _ => println!("unknown command! \n{}",matches.usage()),
    }

    //  let start   : IpAddr = "10.10.10.10".parse()?;
    //  let end     : IpAddr = "10.10.10.100".parse()?;
    //  let mask    : u8     = 24u8;
    //  let private : bool   = true;
    //
    //  let network = RootInterface::with(
    //    "testnet".into(),
    //    private,
    //    start,
    //    end,
    //    mask
    //    );
    //  let j = serde_json::to_string(&network)?;
    //println!("{}",j);
    //println!("{:?}",network);

    Ok(())
}
