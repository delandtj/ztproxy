extern crate failure;
extern crate reqwest;
use failure::Error;
use super::{RootInterface, Auth, serde_json};

const BASE_URL: &'static str = "http://127.0.0.1:9993";

pub fn new_network(r: RootInterface, auth: Auth) -> Result<(RootInterface), Error> {
    let net_url: String = format!("{}/network",BASE_URL).to_owned();
    let installed_net = call_zt_get(serde_json::to_string(&r)?)?;
    Ok(serde_json::from_value(installed_net)?)
}
    
fn get_network(i: String) -> Result<RootInterface,Error> {
    let net_url: String = format!("{}/network/{}",BASE_URL,i).to_owned();
    let v: serde_json::Value = call_zt_get(net_url)?;
    println!("{:?}",v);
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_get(){
        let v = get_network("65a8d1a59587fee4".to_owned());
        println!("{:?}",v);
        assert!(true);
    }
}