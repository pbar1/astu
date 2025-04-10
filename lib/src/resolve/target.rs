use std::fmt;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

use anyhow::bail;
use anyhow::Context;
use fluent_uri::Uri;
use internment::Intern;
use ipnet::IpNet;
use strum::EnumString;

/// All target scheme variants supported by [`Target`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
#[non_exhaustive]
pub enum TargetKind {
    Cidr,
    Dns,
    File,
    Ip,
    Ssh,
    Tcp,
    K8s,
}

/// Hostnames may be either IP addresses or domain names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Host {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
    Domain(String),
}

/// A generic address that may be targeted by actions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Target {
    uri: Uri<String>,

    /// Scheme of the target.
    pub kind: TargetKind,
}

impl Target {
    /// Interns the target. This is so it can implement [`Copy`] for use with
    /// the target graph.
    #[must_use]
    pub fn intern(self) -> Intern<Self> {
        self.into()
    }

    #[must_use]
    pub fn user(&self) -> Option<&str> {
        self.uri.authority()?.userinfo()?.as_str().into()
    }

    #[must_use]
    pub fn host(&self) -> Option<Host> {
        use fluent_uri::component::Host as H;

        let authority = self.uri.authority()?;
        let host = match authority.host_parsed() {
            H::Ipv4(ipv4_addr) => Host::Ipv4(ipv4_addr),
            H::Ipv6(ipv6_addr) => Host::Ipv6(ipv6_addr),
            _other => Host::Domain(authority.host().to_string()),
        };
        Some(host)
    }

    #[must_use]
    pub fn port(&self) -> Option<u16> {
        let port = self.uri.authority()?.port_to_u16().ok()?;
        if port.is_some() {
            port
        } else {
            self.default_scheme_port()
        }
    }

    fn default_scheme_port(&self) -> Option<u16> {
        match &self.kind {
            TargetKind::Ssh => Some(22),
            _other => None,
        }
    }

    #[must_use]
    pub fn path(&self) -> Option<PathBuf> {
        if self.kind != TargetKind::File {
            return None;
        }
        let path = self.uri.path().as_str();
        if path.is_empty() {
            None
        } else {
            PathBuf::from_str(path).ok()
        }
    }

    #[must_use]
    pub fn ip_addr(&self) -> Option<IpAddr> {
        #[allow(clippy::match_wildcard_for_single_variants)]
        match self.host()? {
            Host::Ipv4(ip) => Some(ip.into()),
            Host::Ipv6(ip) => Some(ip.into()),
            _other => None,
        }
    }

    #[must_use]
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        let ip = self.ip_addr()?;
        let port = self.port()?;
        Some(SocketAddr::new(ip, port))
    }

    #[must_use]
    pub fn cidr(&self) -> Option<IpNet> {
        if self.kind != TargetKind::Cidr {
            return None;
        }
        let ip = self.ip_addr()?;
        let mut path_iter = self.uri.path().split('/');
        let prefix_str = if self.uri.path().is_rootless() {
            path_iter.next()?
        } else {
            path_iter.nth(1)?
        };
        let prefix_len = u8::from_str(prefix_str.as_str()).ok()?;
        IpNet::new(ip, prefix_len).ok()
    }

    #[must_use]
    pub fn k8s_namespace(&self) -> Option<String> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        let mut segments = match self.uri.path().segments_if_absolute() {
            Some(segments) => segments,
            None => self.uri.path().split('/'),
        };
        segments.next().map(ToString::to_string)
    }

    #[must_use]
    pub fn k8s_resource(&self) -> Option<String> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        let mut segments = match self.uri.path().segments_if_absolute() {
            Some(segments) => segments,
            None => self.uri.path().split('/'),
        };
        segments.nth(1).map(ToString::to_string)
    }

    #[must_use]
    pub fn k8s_container(&self) -> Option<String> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        self.uri.fragment().map(ToString::to_string)
    }

    #[must_use]
    pub fn k8s_cluster(&self) -> Option<&str> {
        let host = self.uri.authority()?.host();
        if host.is_empty() {
            None
        } else {
            Some(host)
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

// Conversions

impl Target {
    /// # Errors
    ///
    /// If the string does not conform to any of the supported short forms.
    pub fn parse_short_form(s: &str) -> anyhow::Result<Self> {
        if s.starts_with("localhost") {
            return Target::from_str(&format!("dns://{s}"));
        }

        let target = if let Ok(value) = IpAddr::from_str(s) {
            Self::from(value)
        } else if let Ok(value) = SocketAddr::from_str(s) {
            Self::from(value)
        } else if let Ok(value) = IpNet::from_str(s) {
            Self::from(value)
        } else {
            bail!("Unsupported target short form: {s}");
        };

        Ok(target)
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(target) = Self::parse_short_form(s) {
            return Ok(target);
        }

        let uri = Uri::from_str(s).with_context(|| format!("Failed to parse as URI: {s}"))?;
        let kind = TargetKind::from_str(uri.scheme().as_str())
            .with_context(|| format!("URI not supported: {s}"))?;
        Ok(Target { uri, kind })
    }
}

impl From<IpAddr> for Target {
    fn from(value: IpAddr) -> Self {
        let s = match value {
            IpAddr::V4(ip) => format!("ip://{ip}"),
            IpAddr::V6(ip) => format!("ip://[{ip}]"),
        };
        Self::from_str(&s).expect("URI invariant not upheld")
    }
}

impl From<SocketAddr> for Target {
    fn from(value: SocketAddr) -> Self {
        Self::from_str(&format!("ip://{value}")).expect("URI invariant not upheld")
    }
}

impl From<IpNet> for Target {
    fn from(value: IpNet) -> Self {
        let s = match value {
            IpNet::V4(cidr) => format!("cidr://{cidr}"),
            IpNet::V6(cidr) => format!("cidr://[{}]/{}", cidr.network(), cidr.prefix_len()),
        };
        Self::from_str(&s).expect("URI invariant not upheld")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::ipv4("127.0.0.1", "ip://127.0.0.1")]
    #[case::ipv6("::1", "ip://[::1]")]
    #[case::sock4("127.0.0.1:22", "ip://127.0.0.1:22")]
    #[case::sock6("[::1]:22", "ip://[::1]:22")]
    #[case::net4("0.0.0.0/0", "cidr://0.0.0.0/0")]
    #[case::net6("::/0", "cidr://[::]/0")]
    #[case::domain("localhost", "dns://localhost")]
    #[case::domain("dns://domain.test", "dns://domain.test")]
    #[case::domainport("localhost:22", "dns://localhost:22")]
    #[case::domainport("dns://domain.test:22", "dns://domain.test:22")]
    #[case::ssh("ssh://127.0.0.1", "ssh://127.0.0.1")]
    #[case::ssh("ssh://user@127.0.0.1", "ssh://user@127.0.0.1")]
    // #[case::sshport("ssh://[::1]:22", "ssh://[::1]")]
    #[case::sshport("ssh://[::1]:2222", "ssh://[::1]:2222")]
    #[case::sshport("ssh://user@[::1]:2222", "ssh://user@[::1]:2222")]
    fn target_roundtrip(#[case] input: &str, #[case] should: &str) {
        let target = Target::from_str(input).unwrap();
        let output = target.to_string();
        assert_eq!(output, should);
    }

    #[rstest]
    #[case("k8s:kube-system", Some("kube-system"), None, None, None, None)]
    #[case(
        "k8s:kube-system/coredns-0",
        Some("kube-system"),
        Some("coredns-0"),
        None,
        None,
        None
    )]
    #[case(
        "k8s:kube-system/coredns-0#coredns",
        Some("kube-system"),
        Some("coredns-0"),
        Some("coredns"),
        None,
        None
    )]
    #[case("k8s:///kube-system", Some("kube-system"), None, None, None, None)]
    #[case(
        "k8s:///kube-system/coredns-0",
        Some("kube-system"),
        Some("coredns-0"),
        None,
        None,
        None
    )]
    #[case(
        "k8s:///kube-system/coredns-0#coredns",
        Some("kube-system"),
        Some("coredns-0"),
        Some("coredns"),
        None,
        None
    )]
    #[case(
        "k8s://cluster/kube-system/coredns-0#coredns",
        Some("kube-system"),
        Some("coredns-0"),
        Some("coredns"),
        Some("cluster"),
        None
    )]
    #[case(
        "k8s://user@cluster/kube-system/coredns-0#coredns",
        Some("kube-system"),
        Some("coredns-0"),
        Some("coredns"),
        Some("cluster"),
        Some("user")
    )]
    fn target_k8s(
        #[case] input: &str,
        #[case] namespace: Option<&str>,
        #[case] resource: Option<&str>,
        #[case] container: Option<&str>,
        #[case] cluster: Option<&str>,
        #[case] user: Option<&str>,
    ) {
        let namespace = namespace.map(ToOwned::to_owned);
        let resource = resource.map(ToOwned::to_owned);
        let container = container.map(ToOwned::to_owned);

        let target = Target::from_str(input).unwrap();

        assert_eq!(target.k8s_namespace(), namespace);
        assert_eq!(target.k8s_resource(), resource);
        assert_eq!(target.k8s_container(), container);
        assert_eq!(target.k8s_cluster(), cluster);
        assert_eq!(target.user(), user);
    }
}
