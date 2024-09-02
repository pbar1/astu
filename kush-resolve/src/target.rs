#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    Ipv4Addr(std::net::Ipv4Addr),
    Ipv6Addr(std::net::Ipv6Addr),
    SocketAddrV4(std::net::SocketAddrV4),
    SocketAddrV6(std::net::SocketAddrV6),
    Ipv4Net(ipnet::Ipv4Net),
    Ipv6Net(ipnet::Ipv6Net),
    Domain(hickory_resolver::Name),
    DomainPort {
        name: hickory_resolver::Name,
        port: u16,
    },
    Uri(fluent_uri::UriRef<String>),
    Unknown(String),
}

impl std::str::FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.to_owned();

        // Assume surrounding brackets is IPv6 - this will not parse correctly later, so
        // remove them upfront
        let s = if s.starts_with('[') && s.ends_with(']') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        // Eagerly detect URI
        if s.contains("://") {
            let uri = fluent_uri::UriRef::from_str(s)?;
            return Ok(Self::Uri(uri));
        }

        let target = if let Ok(x) = std::net::Ipv4Addr::from_str(s) {
            Self::Ipv4Addr(x)
        } else if let Ok(x) = std::net::Ipv6Addr::from_str(s) {
            Self::Ipv6Addr(x)
        } else if let Ok(x) = std::net::SocketAddrV4::from_str(s) {
            Self::SocketAddrV4(x)
        } else if let Ok(x) = std::net::SocketAddrV6::from_str(s) {
            Self::SocketAddrV6(x)
        } else if let Ok(x) = ipnet::Ipv4Net::from_str(s) {
            Self::Ipv4Net(x)
        } else if let Ok(x) = ipnet::Ipv6Net::from_str(s) {
            Self::Ipv6Net(x)
        } else if let Ok(x) = hickory_resolver::Name::from_str(s) {
            Self::Domain(x)
        } else if let Some((name, port)) = s.split_once(':') {
            let name = hickory_resolver::Name::from_str(name)?;
            let port = u16::from_str(port)?;
            Self::DomainPort { name, port }
        } else {
            Self::Unknown(input)
        };

        Ok(target)
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Target::Ipv4Addr(x) => x.to_string(),
            Target::Ipv6Addr(x) => x.to_string(),
            Target::SocketAddrV4(x) => x.to_string(),
            Target::SocketAddrV6(x) => x.to_string(),
            Target::Ipv4Net(x) => x.to_string(),
            Target::Ipv6Net(x) => x.to_string(),
            Target::Domain(x) => x.to_string(),
            Target::DomainPort { name, port } => format!("{name}:{port}"),
            Target::Uri(x) => x.to_string(),
            Target::Unknown(x) => x.to_string(),
        };
        write!(f, "{s}")
    }
}

/// Number of known unique targets that a target can be divided into discretely.
///
/// For example:
/// - IP and socket addresses are atomic - they cannot be divided further.
/// - CIDR blocks are not atomic - they can be divided into their constituent IP
///   addresses.
/// - DNS names are indeterminate - they are impossible to divide
///   deterministically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Atoms {
    Known(u128),
    KnownMax,
    Unknown,
}

impl Target {
    /// Returns the total number of individual atomic targets that this target
    /// can further resolve into. If that is not possible to determine, returns
    /// [`None`].
    pub fn atoms(&self) -> Atoms {
        match self {
            Target::Ipv4Addr(_) => Atoms::Known(1),
            Target::Ipv6Addr(_) => Atoms::Known(1),
            Target::SocketAddrV4(_) => Atoms::Known(1),
            Target::SocketAddrV6(_) => Atoms::Known(1),
            Target::Ipv4Net(x) => ip_atoms(ipnet::IpNet::V4(*x)),
            Target::Ipv6Net(x) => ip_atoms(ipnet::IpNet::V6(*x)),
            _unknown => Atoms::Unknown,
        }
    }
}

fn ip_atoms(ip_net: ipnet::IpNet) -> Atoms {
    let host_bits = ip_net.max_prefix_len() - ip_net.prefix_len();
    // u128 will overflow if a bit shift this large is attempted
    if host_bits >= 128 {
        return Atoms::KnownMax;
    }
    Atoms::Known(1u128 << host_bits)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("127.0.0.1", "127.0.0.1")]
    #[case("::1", "::1")]
    #[case("[::1]", "::1")]
    #[case("127.0.0.1:22", "127.0.0.1:22")]
    #[case("[::1]:22", "[::1]:22")]
    #[case("0.0.0.0/0", "0.0.0.0/0")]
    #[case("::/0", "::/0")]
    #[case("localhost", "localhost")]
    #[case("domain.test", "domain.test")]
    #[case("localhost:22", "localhost:22")]
    #[case("domain.test:22", "domain.test:22")]
    fn target_roundtrip(#[case] input: &str, #[case] should: &str) {
        let target = Target::from_str(input).unwrap();
        let output = target.to_string();
        assert_eq!(output, should);
    }

    // FIXME: Check for unknown
    // #[rstest]
    // #[case("example.test/path")]
    // fn target_fails(#[case] input: &str) {
    //     let result = Target::from_str(input);
    //     assert!(result.is_err());
    // }

    #[rstest]
    #[case("localhost", Atoms::Unknown)]
    #[case("127.0.0.1", Atoms::Known(1))]
    #[case("::1", Atoms::Known(1))]
    #[case("127.0.0.1:22", Atoms::Known(1))]
    #[case("[::1]:22", Atoms::Known(1))]
    #[case("0.0.0.0/0", Atoms::Known(u32::MAX as u128 + 1))]
    #[case("::/1", Atoms::Known(170141183460469231731687303715884105728))]
    #[case("::/0", Atoms::KnownMax)]
    fn target_atoms(#[case] input: &str, #[case] should: Atoms) {
        let target = Target::from_str(input).unwrap();
        let got = target.atoms();
        assert_eq!(got, should);
    }
}
