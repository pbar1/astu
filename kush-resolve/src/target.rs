use anyhow::bail;

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
}

impl std::str::FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s;

        // Assume surrounding brackets is IPv6 - this will not parse correctly later, so
        // remove them upfront
        let s = if s.starts_with('[') && s.ends_with(']') {
            &s[1..s.len() - 1]
        } else {
            s
        };

        if let Ok(x) = std::net::Ipv4Addr::from_str(s) {
            Ok(Self::Ipv4Addr(x))
        } else if let Ok(x) = std::net::Ipv6Addr::from_str(s) {
            Ok(Self::Ipv6Addr(x))
        } else if let Ok(x) = std::net::SocketAddrV4::from_str(s) {
            Ok(Self::SocketAddrV4(x))
        } else if let Ok(x) = std::net::SocketAddrV6::from_str(s) {
            Ok(Self::SocketAddrV6(x))
        } else if let Ok(x) = ipnet::Ipv4Net::from_str(s) {
            Ok(Self::Ipv4Net(x))
        } else if let Ok(x) = ipnet::Ipv6Net::from_str(s) {
            Ok(Self::Ipv6Net(x))
        } else if let Ok(x) = hickory_resolver::Name::from_str(s) {
            Ok(Self::Domain(x))
        } else if let Some((name, port)) = s.split_once(':') {
            let name = hickory_resolver::Name::from_str(name)?;
            let port = u16::from_str(port)?;
            Ok(Self::DomainPort { name, port })
        } else {
            bail!("unknown target: {input}");
        }
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
        };
        write!(f, "{s}")
    }
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

    #[rstest]
    #[case("example.test/path")]
    fn target_fails(#[case] input: &str) {
        let result = Target::from_str(input);
        assert!(result.is_err());
    }
}
