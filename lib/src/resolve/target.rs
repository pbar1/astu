use std::fmt;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;
use std::string::ToString;

use anyhow::bail;
use anyhow::Context;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use fluent_uri::encoding::encoder::Path;
use fluent_uri::encoding::Split;
use fluent_uri::Uri;
use ipnet::IpNet;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumString;

/// Hostnames may be either IP addresses or domain names.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Host {
    Ip(IpAddr),
    Domain(String),
}

impl FromStr for Host {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(ip) = IpAddr::from_str(s) {
            Ok(Self::Ip(ip))
        } else {
            Ok(Self::Domain(s.to_owned()))
        }
    }
}

/// All target scheme variants supported by [`Target`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Serialize, Deserialize,
)]
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

/// A generic address that may be targeted by actions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Target {
    uri: Uri<String>,
    kind: TargetKind,
}

/// Accessors
impl Target {
    #[must_use]
    pub fn kind(&self) -> TargetKind {
        self.kind
    }

    #[must_use]
    pub fn user(&self) -> Option<&str> {
        let value: Option<_> = self
            .uri
            .authority()?
            .userinfo()?
            .as_str()
            .split(':')
            .next()?
            .into();
        value.filter(|x| !x.is_empty())
    }

    #[must_use]
    pub fn password(&self) -> Option<&str> {
        let value: Option<_> = self
            .uri
            .authority()?
            .userinfo()?
            .as_str()
            .split(':')
            .nth(1)?
            .into();
        value.filter(|x| !x.is_empty())
    }

    #[must_use]
    pub fn host(&self) -> Option<Host> {
        use fluent_uri::component::Host as H;
        let authority = self.uri.authority()?;
        let host = match authority.host_parsed() {
            H::Ipv4(ip) => Host::Ip(ip.into()),
            H::Ipv6(ip) => Host::Ip(ip.into()),
            _other => Host::Domain(authority.host().to_string()),
        };
        Some(host)
    }

    #[must_use]
    pub fn domain(&self) -> Option<&str> {
        use fluent_uri::component::Host as H;
        let authority = self.uri.authority()?;
        let value: Option<_> = match authority.host_parsed() {
            H::RegName(name) => name.as_str().into(),
            _ => None,
        };
        value.filter(|x| !x.is_empty())
    }

    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.uri.authority()?.port_to_u16().ok()?
    }

    #[must_use]
    pub fn path(&self) -> Option<Utf8PathBuf> {
        let path = self.uri.path().as_str();
        if path.is_empty() {
            None
        } else {
            Utf8PathBuf::from_str(path).ok()
        }
    }

    pub fn path_segments(&self) -> Split<'_, Path> {
        match self.uri.path().segments_if_absolute() {
            Some(segments) => segments,
            None => self.uri.path().split('/'),
        }
    }

    #[must_use]
    pub fn fragment(&self) -> Option<&str> {
        let value: Option<_> = self.uri.fragment()?.as_str().into();
        value.filter(|x| !x.is_empty())
    }

    #[must_use]
    pub fn ip(&self) -> Option<IpAddr> {
        match self.host()? {
            Host::Ip(ip) => ip.into(),
            Host::Domain(_) => None,
        }
    }

    #[must_use]
    pub fn socket_addr(&self) -> Option<SocketAddr> {
        let ip = self.ip()?;
        let port = self.port()?;
        Some(SocketAddr::new(ip, port))
    }

    #[must_use]
    pub fn cidr(&self) -> Option<IpNet> {
        if self.kind != TargetKind::Cidr {
            return None;
        }
        let ip = self.ip()?;
        let prefix_len = self.path_segments().next()?.as_str().parse::<u8>().ok()?;
        IpNet::new(ip, prefix_len).ok()
    }

    #[must_use]
    pub fn k8s_user(&self) -> Option<&str> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        self.user()
    }

    #[must_use]
    pub fn k8s_cluster(&self) -> Option<&str> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        self.domain()
    }

    #[must_use]
    pub fn k8s_namespace(&self) -> Option<&str> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        self.path_segments().nth_back(1)?.as_str().into()
    }

    #[must_use]
    pub fn k8s_pod(&self) -> Option<&str> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        let pod: Option<_> = self.path_segments().next_back()?.as_str().into();
        pod.filter(|x| !x.is_empty())
    }

    #[must_use]
    pub fn k8s_container(&self) -> Option<&str> {
        if self.kind != TargetKind::K8s {
            return None;
        }
        self.fragment()
    }
}

/// Constructors
impl Target {
    /// # Errors
    ///
    /// If the URI is malformed
    pub fn new_file(path: &Utf8Path) -> anyhow::Result<Self> {
        let uri = if path.is_absolute() {
            format!("file://{path}")
        } else {
            format!("file:{path}")
        };
        Self::from_str(&uri)
    }

    /// # Errors
    ///
    /// If the URI is malformed
    pub fn new_cidr(cidr: &IpNet, port: Option<u16>, user: Option<&str>) -> anyhow::Result<Self> {
        let mut uri = "cidr://".to_owned();
        if let Some(user) = user {
            uri.push_str(user);
            uri.push('@');
        }
        uri.push_str(&cidr.addr().to_string());
        if let Some(port) = port {
            uri.push(':');
            uri.push_str(&port.to_string());
        }
        uri.push('/');
        uri.push_str(&cidr.prefix_len().to_string());
        Self::from_str(&uri)
    }

    /// # Errors
    ///
    /// If the URI is malformed
    pub fn new_ip(ip: &IpAddr, port: Option<u16>, user: Option<&str>) -> anyhow::Result<Self> {
        let mut uri = "ip://".to_owned();
        if let Some(user) = user {
            uri.push_str(user);
            uri.push('@');
        }
        uri.push_str(&ip.to_string());
        if let Some(port) = port {
            uri.push(':');
            uri.push_str(&port.to_string());
        }
        Self::from_str(&uri)
    }

    /// # Errors
    ///
    /// If the URI is malformed
    pub fn new_dns(domain: &str, port: Option<u16>, user: Option<&str>) -> anyhow::Result<Self> {
        let mut uri = "dns://".to_owned();
        if let Some(user) = user {
            uri.push_str(user);
            uri.push('@');
        }
        uri.push_str(domain);
        if let Some(port) = port {
            uri.push(':');
            uri.push_str(&port.to_string());
        }
        Self::from_str(&uri)
    }
}

/// Conversions
impl Target {
    /// # Errors
    ///
    /// If the string does not conform to any of the supported short forms.
    pub fn parse_short_form(s: &str) -> anyhow::Result<Self> {
        if s.starts_with("localhost") {
            return Target::from_str(&format!("dns://{s}"));
        }

        if let Ok(value) = IpNet::from_str(s) {
            return Ok(Self::from(value));
        }

        if let Ok(value) = IpAddr::from_str(s) {
            return Ok(Self::from(value));
        }

        if let Ok(value) = SocketAddr::from_str(s) {
            return Ok(Self::from(value));
        }

        bail!("Unsupported target short form: {s}");
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)
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

    use super::TargetKind as K;
    use super::*;

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("0.0.0.0/0",                                  K::Cidr, "cidr://0.0.0.0/0")]
    #[case("::/0",                                       K::Cidr, "cidr://[::]/0")]
    #[case("0.0.0.0",                                    K::Ip,   "ip://0.0.0.0")]
    #[case("::",                                         K::Ip,   "ip://[::]")]
    #[case("0.0.0.0:0",                                  K::Ip,   "ip://0.0.0.0:0")]
    #[case("[::]:0",                                     K::Ip,   "ip://[::]:0")]
    #[case("localhost",                                  K::Dns,  "dns://localhost")]
    #[case("file:relative.txt",                          K::File, "file:relative.txt")]
    #[case("file:///absolute.txt",                       K::File, "file:///absolute.txt")]
    #[case("cidr://user@0.0.0.0:0/0",                    K::Cidr, "cidr://user@0.0.0.0:0/0")]
    #[case("ip://user@0.0.0.0:0",                        K::Ip,   "ip://user@0.0.0.0:0")]
    #[case("dns://user@localhost:0",                     K::Dns,  "dns://user@localhost:0")]
    #[case("ssh://user:password@localhost:2222",         K::Ssh,  "ssh://user:password@localhost:2222")]
    #[case("k8s:pod#container",                          K::K8s,  "k8s:pod#container")]
    #[case("k8s://user@cluster/namespace/pod#container", K::K8s,  "k8s://user@cluster/namespace/pod#container")]
    fn roundtrip_works(#[case] uri: &str, #[case] kind_should: K, #[case] output_should: &str) {
        let target = Target::from_str(uri).unwrap();
        assert_eq!(target.kind(), kind_should);
        let output = target.to_string();
        assert_eq!(output, output_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("file:relative/file.txt",    "relative/file.txt")]
    #[case("file:///absolute/file.txt", "/absolute/file.txt")]
    fn file_works(#[case] uri: &str, #[case] path_should: &str) {
        let target = Target::from_str(uri).unwrap();
        let path = target.path().unwrap();
        assert_eq!(path, path_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("0.0.0.0/0",                   "0.0.0.0/0",    None,   None)]
    #[case("::/0",                        "::/0",         None,   None)]
    #[case("cidr://127.0.0.0/32",         "127.0.0.0/32", None,   None)]
    #[case("cidr://root@127.0.0.0:22/32", "127.0.0.0/32", "root", 22)]
    #[case("cidr://[::1]/128",            "::1/128",      None,   None)]
    #[case("cidr://root@[::1]:22/128",    "::1/128",      "root", 22)]
    fn cidr_works(
        #[case] uri: &str,
        #[case] cidr_should: &str,
        #[case] user_should: impl Into<Option<&'static str>>,
        #[case] port_should: impl Into<Option<u16>>,
    ) {
        let cidr_should = IpNet::from_str(cidr_should).unwrap();
        let user_should = user_should.into();
        let port_should = port_should.into();

        let target = Target::from_str(uri).unwrap();

        let cidr = target.cidr().unwrap();
        let user = target.user();
        let port = target.port();

        assert_eq!(cidr, cidr_should);
        assert_eq!(user, user_should);
        assert_eq!(port, port_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("0.0.0.0",                "0.0.0.0",   None,   None)]
    #[case("::",                     "::",        None,   None)]
    #[case("0.0.0.0:0",              "0.0.0.0",   None,   0)]
    #[case("[::]:0",                 "::",        None,   0)]
    #[case("ip://127.0.0.1",         "127.0.0.1", None,   None)]
    #[case("ip://root@127.0.0.1:22", "127.0.0.1", "root", 22)]
    #[case("ip://[::1]",             "::1",       None,   None)]
    #[case("ip://root@[::1]:22",     "::1",       "root", 22)]
    fn ip_works(
        #[case] uri: &str,
        #[case] ip_should: &str,
        #[case] user_should: impl Into<Option<&'static str>>,
        #[case] port_should: impl Into<Option<u16>>,
    ) {
        let ip_should = IpAddr::from_str(ip_should).unwrap();
        let user_should = user_should.into();
        let port_should = port_should.into();

        let target = Target::from_str(uri).unwrap();

        let ip = target.ip().unwrap();
        let user = target.user();
        let port = target.port();

        assert_eq!(ip, ip_should);
        assert_eq!(user, user_should);
        assert_eq!(port, port_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("localhost",               "localhost", None, None)]
    #[case("dns://localhost",         "localhost", None, None)]
    #[case("dns://root@localhost:22", "localhost", 22,   "root")]
    fn dns_works(
        #[case] uri: &str,
        #[case] domain_should: &str,
        #[case] port_should: impl Into<Option<u16>>,
        #[case] user_should: impl Into<Option<&'static str>>,
    ) {
        let port_should = port_should.into();
        let user_should = user_should.into();

        let target = Target::from_str(uri).unwrap();

        let domain = target.domain().unwrap();
        let user = target.user();
        let port = target.port();

        assert_eq!(domain, domain_should);
        assert_eq!(port, port_should);
        assert_eq!(user, user_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("ssh://127.0.0.1",                    "127.0.0.1",  None, None,   None)]
    #[case("ssh://localhost",                    "localhost",  None, None,   None)]
    #[case("ssh://root:password@localhost:2222", "localhost",  2222, "root", "password")]
    #[case("ssh://root@[::1]",                   "::1",        None, "root", None)]
    fn ssh_works(
        #[case] uri: &str,
        #[case] host_should: &str,
        #[case] port_should: impl Into<Option<u16>>,
        #[case] user_should: impl Into<Option<&'static str>>,
        #[case] password_should: impl Into<Option<&'static str>>,
    ) {
        let host_should = Host::from_str(host_should).unwrap();
        let port_should = port_should.into();
        let user_should = user_should.into();
        let password_should = password_should.into();

        let target = Target::from_str(uri).unwrap();

        let host = target.host().unwrap();
        let port = target.port();
        let user = target.user();
        let password = target.password();

        assert_eq!(host, host_should);
        assert_eq!(port, port_should);
        assert_eq!(user, user_should);
        assert_eq!(password, password_should);
    }

    #[rustfmt::skip::attributes(case)]
    #[rstest]
    #[case("k8s:kube-system/",                                 "kube-system", None,        None,      None,      None)]
    #[case("k8s:coredns-0",                                    None,          "coredns-0", None,      None,      None)]
    #[case("k8s:kube-system/coredns-0",                        "kube-system", "coredns-0", None,      None,      None)]
    #[case("k8s:kube-system/coredns-0#coredns",                "kube-system", "coredns-0", "coredns", None,      None)]
    #[case("k8s:///kube-system/",                              "kube-system", None,        None,      None,      None)]
    #[case("k8s:///kube-system/coredns-0",                     "kube-system", "coredns-0", None,      None,      None)]
    #[case("k8s:///kube-system/coredns-0#coredns",             "kube-system", "coredns-0", "coredns", None,      None)]
    #[case("k8s://cluster/kube-system/coredns-0#coredns",      "kube-system", "coredns-0", "coredns", "cluster", None)]
    #[case("k8s://user@cluster/kube-system/coredns-0#coredns", "kube-system", "coredns-0", "coredns", "cluster", "user")]
    fn k8s_works(
        #[case] input: &str,
        #[case] namespace: impl Into<Option<&'static str>>,
        #[case] resource: impl Into<Option<&'static str>>,
        #[case] container: impl Into<Option<&'static str>>,
        #[case] cluster: impl Into<Option<&'static str>>,
        #[case] user: impl Into<Option<&'static str>>,
    ) {
        let namespace_should = namespace.into();
        let pod_should = resource.into();
        let container_should = container.into();
        let cluster_should = cluster.into();
        let user_should = user.into();

        let target = Target::from_str(input).unwrap();

        assert_eq!(target.k8s_namespace(), namespace_should);
        assert_eq!(target.k8s_pod(), pod_should);
        assert_eq!(target.k8s_container(), container_should);
        assert_eq!(target.k8s_cluster(), cluster_should);
        assert_eq!(target.user(), user_should);
    }
}
