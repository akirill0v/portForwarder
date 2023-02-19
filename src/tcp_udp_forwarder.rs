use crate::tcp_forwarder::TcpForwarder;
use crate::udp_forwarder::UdpForwarder;
use crate::forward_config::ForwardSessionConfig;
use std::net::ToSocketAddrs;
use std::error::Error;
use std::result::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct TcpUdpForwarder {
    tcp: Arc<Option<TcpForwarder>>,
    udp: Arc<Option<UdpForwarder>>
}

impl TcpUdpForwarder {
    pub fn from<T>(config: &ForwardSessionConfig<T>)
        -> Result<TcpUdpForwarder, Box<dyn Error>> where T: ToSocketAddrs
    {
        assert!(config.enable_tcp || config.enable_udp);

        let mut tcpi = None;
        let mut udpi = None;
        if config.enable_tcp {
            tcpi = Some(TcpForwarder::from(&config)?);
        }
        if config.enable_udp {
            udpi = Some(UdpForwarder::from(&config)?);
        }

        Ok(TcpUdpForwarder {
            tcp: Arc::from(tcpi),
            udp: Arc::from(udpi)
        })
    }

    pub fn listen(&self) {
        let mut tt = None;
        let mut tu = None;
        if self.tcp.is_some() {
            let m = self.tcp.clone();
            tt = Some(std::thread::spawn(move || {
                match m.as_ref() {
                    Some(l) => {
                        l.listen().unwrap_or_default();
                    },
                    None => {}
                }
            }));
        }
        if self.udp.is_some() {
            let m = self.udp.clone();
            tu = Some(std::thread::spawn(move || {
                match m.as_ref() {
                    Some(l) => l.listen().unwrap_or_default(),
                    None => {}
                }
            }));
        }

        if tt.is_some() {
            tt.unwrap().join().unwrap_or_default();
        }
        if tu.is_some() {
            tu.unwrap().join().unwrap_or_default();
        }
    }

#[allow(dead_code)]
    pub fn close(&self) {
        match self.tcp.as_ref() {
            Some(tcp) => tcp.close(),
            None => {}
        }

        match self.udp.as_ref() {
            Some(udp) => udp.close(),
            None => {}
        }
    }
}

