// crates/varex-core/src/ssrf.rs
//! Resolve → validate every IP → connect to the validated IP. A hostname
//! never reaches `TcpStream::connect` / `UdpSocket::connect`.
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use crate::config::SecurityConfig;
use crate::error::{Result, VarexError};

pub fn is_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => is_blocked_v6(v6),
    }
}

fn is_blocked_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    if o[0] == 0 { return true; }
    if o[0] == 10 { return true; }
    if o[0] == 100 && (64..=127).contains(&o[1]) { return true; }
    if o[0] == 127 { return true; }
    if o[0] == 169 && o[1] == 254 { return true; }
    if o[0] == 172 && (16..=31).contains(&o[1]) { return true; }
    if o[0] == 192 && o[1] == 0 && o[2] == 0 { return true; }
    if o[0] == 192 && o[1] == 0 && o[2] == 2 { return true; }
    if o[0] == 192 && o[1] == 168 { return true; }
    if o[0] == 198 && (o[1] == 18 || o[1] == 19) { return true; }
    if o[0] == 198 && o[1] == 51 && o[2] == 100 { return true; }
    if o[0] == 203 && o[1] == 0 && o[2] == 113 { return true; }
    if (224..=239).contains(&o[0]) { return true; }
    if o[0] >= 240 { return true; }
    false
}

fn is_blocked_v6(ip: Ipv6Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() { return true; }
    let seg = ip.segments();
    if (seg[0] & 0xfe00) == 0xfc00 { return true; }
    if (seg[0] & 0xffc0) == 0xfe80 { return true; }
    if let Some(v4) = ip.to_ipv4_mapped() { return is_blocked_v4(v4); }
    false
}

pub struct SsrfPolicy {
    allowed_ports: Vec<u16>,
    allowed_hosts: Option<globset::GlobSet>,
    denied_hosts: globset::GlobSet,
}

impl SsrfPolicy {
    pub fn new(cfg: &SecurityConfig) -> Result<Self> {
        let denied = build_globset(&cfg.denied_hosts)?;
        let allowed = if cfg.allowed_hosts.is_empty() { None } else { Some(build_globset(&cfg.allowed_hosts)?) };
        Ok(Self { allowed_ports: cfg.allowed_ports.clone(), allowed_hosts: allowed, denied_hosts: denied })
    }

    pub fn check_port(&self, port: u16) -> Result<()> {
        if self.allowed_ports.is_empty() || self.allowed_ports.contains(&port) { Ok(()) }
        else { Err(VarexError::PortDenied { port }) }
    }

    pub fn check_hostname(&self, host: &str) -> Result<()> {
        let h = host.to_ascii_lowercase();
        if let Some(allow) = &self.allowed_hosts {
            if !allow.is_match(&h) { return Err(VarexError::HostDenied(host.into())); }
        }
        if self.denied_hosts.is_match(&h) { return Err(VarexError::HostDenied(host.into())); }
        Ok(())
    }

    pub fn check_resolved(&self, addrs: &[IpAddr]) -> Result<()> {
        for ip in addrs {
            if is_blocked(*ip) { return Err(VarexError::AddressBlocked { addr: ip.to_string() }); }
        }
        Ok(())
    }
}

fn build_globset(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut b = globset::GlobSetBuilder::new();
    for p in patterns {
        b.add(globset::Glob::new(p).map_err(|e| VarexError::Config(format!("glob {p}: {e}")))?);
    }
    b.build().map_err(|e| VarexError::Config(format!("globset: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocks_required_ranges() {
        for s in ["0.0.0.0","10.1.2.3","100.64.0.1","127.0.0.1","169.254.169.254",
                  "172.16.0.1","192.168.1.1","224.0.0.1","255.255.255.255",
                  "::1","fc00::1","fe80::1","::ffff:127.0.0.1","::"] {
            assert!(is_blocked(s.parse().unwrap()), "{s}");
        }
    }
    #[test]
    fn allows_public() {
        for s in ["1.1.1.1","8.8.8.8","2606:4700:4700::1111"] {
            assert!(!is_blocked(s.parse().unwrap()), "{s}");
        }
    }
}
