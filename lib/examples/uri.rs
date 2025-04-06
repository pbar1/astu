use std::fmt;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use fluent_uri::Uri;
use strum::EnumString;

// Target ---------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, EnumString)]
#[strum(ascii_case_insensitive)]
enum TargetKind {
    Cidr,
    Dns,
    File,
    Ip,
    Ssh,
    Tcp,
    K8s,
}

#[derive(Debug, PartialEq, Eq)]
struct Target {
    uri: Uri<String>,
    pub kind: TargetKind,
}

impl Target {
    pub fn as_str(&self) -> &str {
        self.uri.as_str()
    }

    pub fn user(&self) -> Option<&str> {
        self.uri.authority()?.userinfo()?.as_str().into()
    }

    pub fn host(&self) -> Option<Host> {
        let authority = self.uri.authority()?;
        use fluent_uri::component::Host as H;
        let host = match authority.host_parsed() {
            H::Ipv4(ipv4_addr) => Host::Ipv4(ipv4_addr),
            H::Ipv6(ipv6_addr) => Host::Ipv6(ipv6_addr),
            _other => Host::Domain(authority.host().to_string()),
        };
        Some(host)
    }

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

    pub fn path(&self) -> Option<PathBuf> {
        let path = self.uri.path().as_str();
        PathBuf::from_str(path).ok()
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
        let uri = Uri::from_str(s).with_context(|| format!("Failed to parse as URI: {s}"))?;
        // dbg!(&uri);
        let kind = TargetKind::from_str(uri.scheme().as_str())
            .with_context(|| format!("URI not supported: {s}"))?;
        Ok(Target { uri, kind })
    }
}

enum Host {
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
    Domain(String),
}

fn main() -> anyhow::Result<()> {
    for s in [
        "ip://127.0.0.1",
        "ip://127.0.0.1:22",
        "ip://user@127.0.0.1:22",
        "ip://[::]",
        "ip://[::]:22",
        "ip://user@[::]:22",
        "cidr://127.0.0.1/31",
        "cidr://127.0.0.1:22/31",
        "cidr://user@127.0.0.1:22/31",
        "tcp://127.0.0.1:22",
        "ssh://127.0.0.1",
        "ssh://127.0.0.1:22",
        "ssh://user@127.0.0.1",
        "file:relative.txt",
        "file:///absolute.txt",
        "dns://localhost",
        "dns://localhost:22",
        "dns://user@localhost:22",
        "k8s:coredns-ff8999cc5-x56jw",
        "k8s:coredns-ff8999cc5-x56jw#coredns",
        "k8s:kube-system/coredns-ff8999cc5-x56jw",
        "k8s:kube-system/coredns-ff8999cc5-x56jw#coredns",
        "k8s://user@default/kube-system/coredns-ff8999cc5-x56jw#coredns",
    ] {
        match Target::from_str(s) {
            Ok(t) => {
                println!("{t}");
                println!("    path: {:?}", t.path())
            }
            Err(err) => eprintln!("error: {err}"),
        }
    }
    Ok(())
}
