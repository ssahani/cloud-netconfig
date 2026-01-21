
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::net::{IpAddr, SocketAddr};
use anyhow::{anyhow, Result};

pub fn parse_ip(ip: &str) -> Result<IpAddr> {
    ip.parse::<IpAddr>()
        .map_err(|_| anyhow!("invalid IP"))
}

pub fn parse_port(port: &str) -> Result<u16> {
    port.parse::<u16>()
        .map_err(|_| anyhow!("invalid port"))
}

pub fn parse_ip_port(s: &str) -> Result<(String, String)> {
    let addr = s.parse::<SocketAddr>()
        .map_err(|_| anyhow!("invalid socket address"))?;

    Ok((addr.ip().to_string(), addr.port().to_string()))
}

/// Splits MAC address without ':' or '-' into MAC address format by inserting ':'
pub fn parse_mac(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            result.push(':');
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mac() {
        assert_eq!(parse_mac("001122334455"), "00:11:22:33:44:55");
    }

    #[test]
    fn test_parse_ip_port() {
        let (ip, port) = parse_ip_port("127.0.0.1:5209").unwrap();
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, "5209");
    }
}
