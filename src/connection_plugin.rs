use std::net::{SocketAddr, ToSocketAddrs};
use regex::Regex;
use std::collections::HashMap;
use crate::address_matcher::IpAddrMatcher;
use hex;
    

pub trait ConnectionPlugin {
    fn onlySingleTarget(&self) -> Option<SocketAddr>;
    fn decideTarget(&self, buf: &[u8], addr: SocketAddr) -> Option<SocketAddr>;
    fn testipaddr(&self, addr: &SocketAddr) -> bool;
    fn transform(&mut self, buf: &[u8]) -> Option<Vec<u8>>;
}

pub struct RegexMultiplexer {
    utarget: Option<SocketAddr>,
    rules: Vec<(Box<dyn Fn(&str)->bool + Send + Sync>,SocketAddr)>,
    ip_matcher: IpAddrMatcher
}

impl From<(Vec<(String,String)>, Vec<String>)> for RegexMultiplexer {
    fn from(regexPlusAllowed: (Vec<(String,String)>, Vec<String>)) -> Self {
        let proto2regex: HashMap<&str,&str> = vec![
            ("[ssh]", "^SSH-2\\.0-.+"),
            ("[http]", "^(GET|POST|PUT|DELETE|OPTIONS|HEAD|CONNECT|TRACE).*HTTP.*")
        ].into_iter().collect();

        let rules = regexPlusAllowed.0.iter().map(|pair| {
            let gexp = match proto2regex.get(&pair.0.as_str()) {
                Some(re) => *re,
                None => &pair.0
            };

            let addr = pair.1.to_socket_addrs().unwrap().next().unwrap();
            if gexp == "[socks5]" {
                let func: Box<dyn Fn(&str)->bool + Send + Sync> = Box::new(|s: &str| {
                    match hex::decode(s) {
                        Ok(buf) => {
                            if buf.len() < 3 || buf[0] != 0x05 || buf.len() != usize::from(buf[1]) + 2 {
                                false
                            } else {
                                let mut is_valid = true;
                                for octet_ in &buf[2..] {
                                    let octet = *octet_;
                                    if octet != 0 && octet != 1 && octet != 2 && octet != 3 && octet != 0x80 && octet != 0xFF {
                                        is_valid = false;
                                        break;
                                    }
                                }
                                is_valid
                            }
                        },
                        Err(_) => false,
                    }
                });
                return (func, addr);
            } else {
                let exp = if gexp.starts_with("[http:") && gexp.ends_with("]") {
                    let domain_name = &gexp[6..gexp.len()-1];
                    "^(GET|POST|PUT|DELETE|OPTIONS|HEAD|CONNECT|TRACE).*HTTP.*(.\r\n.*)*".to_string() + domain_name
                } else if gexp.starts_with("[https:") && gexp.ends_with("]") {
                    let domain_name = &gexp[7..gexp.len()-1];
                    "^160301.*".to_string() + &hex::encode(domain_name) + &".*".to_string()
                } else {
                    gexp.to_string()
                };

                let regex = Regex::new(&exp).unwrap();
                let func: Box<dyn Fn(&str)->bool + Send + Sync> = Box::new(move |s: &str| regex.is_match(s));
                return (func, addr);
            }
        }).collect();
        let ip_matcher= IpAddrMatcher::from(&regexPlusAllowed.1);
        let utarget = if regexPlusAllowed.0.len() == 1 && regexPlusAllowed.0.get(0).unwrap().0 == ".*" {
            Some(regexPlusAllowed.0.get(0).unwrap().1.to_socket_addrs().unwrap().next().unwrap())
        } else {
            None
        };
        RegexMultiplexer { utarget, rules, ip_matcher }
    }
}

impl ConnectionPlugin for RegexMultiplexer {
    fn onlySingleTarget(&self) -> Option<SocketAddr> {
        return self.utarget;
    }

    fn decideTarget(&self, buf: &[u8], _addr: SocketAddr) -> Option<SocketAddr> {
        let s1 = hex::encode(buf);
        let s2 = String::from_utf8_lossy(buf);
        for rule in &self.rules {
            if rule.0(&s1) || rule.0(&s2) {
                return Some(rule.1);
            }
        }
        None
    }

    fn testipaddr(&self, addr: &SocketAddr) -> bool {
        self.ip_matcher.testipaddr(&addr.ip())
    }

    fn transform(&mut self, _: &[u8]) -> Option<Vec<u8>> {
        None
    }
}
