use anyhow::bail;
use anyhow::Context;
use internment::Intern;

/// A generic address that may be targeted by actions (ie, connect).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Target {
    // Atoms
    IpAddr(std::net::IpAddr),
    SocketAddr(std::net::SocketAddr),
    Ssh {
        addr: std::net::SocketAddr,
        user: Option<String>,
    },

    // Aggregates
    Cidr(ipnet::IpNet),
    Domain {
        name: hickory_resolver::Name,
        port: Option<u16>,
    },
    File(camino::Utf8PathBuf),
}

impl std::str::FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let original = s.to_owned();

        // Capture known stdin abbreviation of single dash
        if s == "-" {
            let path = camino::Utf8Path::new("/dev/fd/0").to_owned();
            return Ok(path.into());
        }

        // Assume surrounding brackets is IPv6, and remove them to allow correct parsing
        let s = if s.starts_with('[') && s.ends_with(']') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        // Try URI first if scheme is detected
        if s.contains("://") {
            if let Ok(uri) = fluent_uri::UriRef::from_str(s) {
                return uri.try_into();
            }
        }

        if let Ok(ip) = std::net::IpAddr::from_str(s) {
            return Ok(ip.into());
        }

        if let Ok(sock) = std::net::SocketAddr::from_str(s) {
            return Ok(sock.into());
        }

        if let Ok(cidr) = ipnet::IpNet::from_str(s) {
            return Ok(cidr.into());
        }

        if let Ok(name) = hickory_resolver::Name::from_str(s) {
            return Ok(name.into());
        }

        if let Some((name, port)) = s.split_once(':') {
            let name = hickory_resolver::Name::from_str(name)?;
            let port = Some(u16::from_str(port)?);
            return Ok(Self::Domain { name, port });
        }

        if camino::Utf8Path::new(s).exists() {
            let path = camino::Utf8Path::new(s).to_owned();
            return Ok(path.into());
        }

        bail!("unknown target type: {original}");
    }
}

impl From<std::net::IpAddr> for Target {
    fn from(ip: std::net::IpAddr) -> Self {
        Self::IpAddr(ip)
    }
}

impl From<std::net::SocketAddr> for Target {
    fn from(sock: std::net::SocketAddr) -> Self {
        Self::SocketAddr(sock)
    }
}

impl From<ipnet::IpNet> for Target {
    fn from(cidr: ipnet::IpNet) -> Self {
        Self::Cidr(cidr)
    }
}

impl From<hickory_resolver::Name> for Target {
    fn from(name: hickory_resolver::Name) -> Self {
        Self::Domain { name, port: None }
    }
}

impl From<camino::Utf8PathBuf> for Target {
    fn from(path: camino::Utf8PathBuf) -> Self {
        Self::File(path)
    }
}

impl TryFrom<fluent_uri::UriRef<String>> for Target {
    type Error = anyhow::Error;

    fn try_from(uri: fluent_uri::UriRef<String>) -> Result<Self, Self::Error> {
        let scheme = uri
            .scheme()
            .map(fluent_uri::component::Scheme::as_str)
            .unwrap_or_default();

        if scheme == "ssh" {
            let authority = uri.authority().context("ssh uri had no authority")?;
            let user = authority.userinfo().map(std::string::ToString::to_string);
            let host = authority.host_parsed();
            let port = authority.port_to_u16()?.unwrap_or(22);
            let addr = match host {
                fluent_uri::component::Host::Ipv4(ipv4) => {
                    std::net::SocketAddr::new(ipv4.into(), port)
                }
                fluent_uri::component::Host::Ipv6(ipv6) => {
                    std::net::SocketAddr::new(ipv6.into(), port)
                }
                _unsupported => bail!("ssh host type unsupported: {}", authority.host()),
            };
            return Ok(Self::Ssh { addr, user });
        }

        bail!("unknown uri: {uri}");
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Target::IpAddr(ip) => ip.to_string(),
            Target::SocketAddr(sock) => sock.to_string(),
            Target::Ssh { addr, user } => display_ssh(addr, user.as_ref()),
            Target::Cidr(cidr) => cidr.to_string(),
            Target::Domain { name, port } => display_domain(name, *port),
            Target::File(path) => path.to_string(),
        };
        write!(f, "{s}")
    }
}

fn display_domain(name: &hickory_resolver::Name, port: Option<u16>) -> String {
    let mut s = name.to_string();
    if let Some(port) = port {
        s.push(':');
        s.push_str(&port.to_string());
    }
    s
}

fn display_ssh(addr: &std::net::SocketAddr, user: Option<&String>) -> String {
    let mut s = "ssh://".to_string();
    if let Some(user) = user {
        s.push_str(user);
        s.push('@');
    }
    match addr.ip() {
        std::net::IpAddr::V4(ip) => s.push_str(&ip.to_string()),
        std::net::IpAddr::V6(ip) => {
            s.push('[');
            s.push_str(&ip.to_string());
            s.push(']');
        }
    }
    match addr.port() {
        22 => return s,
        port => s.push_str(&format!(":{port}")),
    }
    s
}

impl Target {
    /// Interns the target. This is so it can implement [`Copy`] for use with
    /// the target graph.
    #[must_use]
    pub fn intern(self) -> Intern<Self> {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ipv4("127.0.0.1", "127.0.0.1")]
    #[case::ipv6("::1", "::1")]
    #[case::ipv6("[::1]", "::1")]
    #[case::sock4("127.0.0.1:22", "127.0.0.1:22")]
    #[case::sock6("[::1]:22", "[::1]:22")]
    #[case::net4("0.0.0.0/0", "0.0.0.0/0")]
    #[case::net6("::/0", "::/0")]
    #[case::domain("localhost", "localhost")]
    #[case::domain("domain.test", "domain.test")]
    #[case::domainport("localhost:22", "localhost:22")]
    #[case::domainport("domain.test:22", "domain.test:22")]
    #[case::ssh("ssh://127.0.0.1", "ssh://127.0.0.1")]
    #[case::ssh("ssh://user@127.0.0.1", "ssh://user@127.0.0.1")]
    #[case::sshport("ssh://[::1]:22", "ssh://[::1]")]
    #[case::sshport("ssh://[::1]:2222", "ssh://[::1]:2222")]
    #[case::sshport("ssh://user@[::1]:2222", "ssh://user@[::1]:2222")]
    fn target_roundtrip(#[case] input: &str, #[case] should: &str) {
        let target = Target::from_str(input).unwrap();
        let output = target.to_string();
        assert_eq!(output, should);
    }
}
