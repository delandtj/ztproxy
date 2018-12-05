extern crate failure;
extern crate reqwest;
use failure::Error;
#[macro_use] use serde_json::{Deserializer, Serializer};
use super::*;

const BASE_URL: &str = "http://127.0.0.1:9993";

pub fn new_network(r: RootInterface) -> Result<(RootInterface), Error> {
    let net_url: String = format!("{}/network",BASE_URL).to_owned();
    let r = RootInterface::default();
    Ok(r)
}
    
fn get_network(i: String) -> Result<RootInterface,Error> {
    let net_url: String = format!("{}/network/{}",BASE_URL,i).to_owned();
    let v: serde_json::Value = call_zt_get(net_url)?;
    let r: RootInterface = serde_json::from_value(v)?;
    Ok(r)
}

pub fn update_network(r: RootInterface) -> Result<RootInterface,Error> {
    let ret = RootInterface::default();
    Ok(ret)
}

fn call_zt_get(u: String) -> Result<serde_json::Value, Error> {
    let v = reqwest::get(&*u)?.text()?;
    let v: serde_json::Value = serde_json::from_str(&v)?;
    Ok(v)
}

