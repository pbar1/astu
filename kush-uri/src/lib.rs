use core::fmt;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

use url::Url;

/// Friendly wrapper around [`Url`] that infers what you mean.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uri {
    pub inner: Url,
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl FromStr for Uri {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with('[') && s.ends_with(']') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        let inner = if s.contains("://") {
            Url::parse(s)?
        } else if let Ok(ip) = IpAddr::from_str(s) {
            parse_ssh_ip(s, ip)?
        } else if let Ok(sock) = SocketAddr::from_str(s) {
            parse_ssh_ip(s, sock.ip())?
        } else {
            Url::parse(s)?
        };

        // FIXME: We will have still missed some IPv4-as-domain cases if given scheme

        Ok(Self { inner })
    }
}

/// [`Url`] parses IPv4 hosts as domains, because they are. We don't want that,
/// so we force the host to be IP.
fn parse_ssh_ip(s: &str, ip: IpAddr) -> anyhow::Result<Url> {
    let s = match (ip, s.starts_with('[')) {
        (IpAddr::V4(_), _) => format!("ssh://{s}"),
        (IpAddr::V6(_), true) => format!("ssh://{s}"),
        (IpAddr::V6(_), false) => format!("ssh://[{s}]"),
    };
    let mut inner = Url::parse(&s)?;
    inner
        .set_ip_host(ip)
        .map_err(|_| anyhow::anyhow!("failed to set url ip address"))?;
    Ok(inner)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ipv4("127.0.0.1", "ssh://127.0.0.1")]
    #[case::ipv6("::1", "ssh://[::1]")]
    #[case::ipv6("[::1]", "ssh://[::1]")]
    #[case::sock4("127.0.0.1:22", "ssh://127.0.0.1:22")]
    #[case::sock6("[::1]:22", "ssh://[::1]:22")]
    #[case::scheme("ssh://127.0.0.1", "ssh://127.0.0.1")]
    fn uri_roundtrip_works(#[case] input: &str, #[case] should: &str) {
        let got = Uri::from_str(input).unwrap().to_string();
        assert_eq!(got, should);
    }

    #[rstest]
    #[case::ipv4_compressed("127.1")] // TODO: Support compressed IPv4
    #[case::ipv6_ambiguous_port("0:1:2:3:4:5:6:7:8")]
    fn uri_roundtrip_fails(#[case] input: &str) {
        let result = Uri::from_str(input);
        assert!(result.is_err());
    }
}
