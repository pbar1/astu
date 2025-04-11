use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use anyhow::Result;
use borrow_or_share::Bos;
use fluent_uri::Uri;
use ipnet::IpNet;
use strum::EnumString;

/// All target scheme variants supported by [`Target`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
#[non_exhaustive]
pub enum TargetKind {
    File,
    Cidr,
    Ip,
    Dns,
    Ssh,
    K8s,
}

/// A generic address that may be targeted by actions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    File {
        path: PathBuf,
    },
    Cidr {
        user: Option<String>,
        network: IpNet,
        port: Option<u16>,
    },
    Ip {
        user: Option<String>,
        ip: IpAddr,
        port: Option<u16>,
    },
    Dns {
        user: Option<String>,
        name: String,
        port: Option<u16>,
    },
    Ssh {
        user: Option<String>,
        password: Option<String>,
        host: String,
        port: Option<u16>,
    },
    K8s {
        user: Option<String>,
        cluster: Option<String>,
        namespace: Option<String>,
        pod: Option<String>,
        container: Option<String>,
    },
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let uri = Uri::parse(s)?;
        Self::try_from(uri)
    }
}

impl<T> TryFrom<Uri<T>> for Target
where
    T: Bos<str>,
{
    type Error = anyhow::Error;

    fn try_from(uri: Uri<T>) -> Result<Self, Self::Error> {
        let kind = TargetKind::from_str(uri.scheme().as_str())?;
        let uri = uri.normalize();

        match kind {
            TargetKind::File => uri_to_file(&uri),
            TargetKind::Cidr => uri_to_cidr(&uri),
            TargetKind::Ip => uri_to_ip(&uri),
            TargetKind::Dns => uri_to_dns(&uri),
            TargetKind::Ssh => uri_to_ssh(&uri),
            TargetKind::K8s => uri_to_k8s(&uri),
        }
    }
}

fn uri_to_file(uri: &Uri<String>) -> Result<Target> {
    let path = uri_utils::path(uri).context("URI file: must have a path")?;

    Ok(Target::File { path })
}

fn uri_to_cidr(uri: &Uri<String>) -> Result<Target> {
    let ip = uri_utils::ip(uri).context("URI cidr: must have a valid IP")?;
    let prefix = uri_utils::path_segments(uri)
        .next()
        .context("URI cidr: first path component must be prefix len")?
        .as_str()
        .parse::<u8>()
        .context("URI cidr: prefix len must be u8")?;
    let network = IpNet::new(ip, prefix).context("URI cidr: invalid CIDR specification")?;

    Ok(Target::Cidr {
        user: uri_utils::user(uri),
        network,
        port: uri_utils::port(uri),
    })
}

fn uri_to_ip(uri: &Uri<String>) -> Result<Target> {
    let ip = uri_utils::ip(uri).context("URI ip: must have a valid IP")?;

    Ok(Target::Ip {
        user: uri_utils::user(uri),
        ip,
        port: uri_utils::port(uri),
    })
}

fn uri_to_dns(uri: &Uri<String>) -> Result<Target> {
    let name = uri_utils::domain_name(uri).context("URI dns: must have a valid domain name")?;

    Ok(Target::Dns {
        user: uri_utils::user(uri),
        name,
        port: uri_utils::port(uri),
    })
}

fn uri_to_ssh(uri: &Uri<String>) -> Result<Target> {
    let host = uri_utils::host(uri).context("URI ssh: must have a host")?;

    Ok(Target::Ssh {
        user: uri_utils::user(uri),
        password: uri_utils::password(uri),
        host,
        port: uri_utils::port(uri),
    })
}

fn uri_to_k8s(uri: &Uri<String>) -> Result<Target> {
    todo!()
}

mod uri_utils {
    use std::net::IpAddr;
    use std::path::PathBuf;
    use std::str::FromStr;

    use fluent_uri::component::Host;
    use fluent_uri::encoding::encoder::Path;
    use fluent_uri::encoding::Split;
    use fluent_uri::Uri;

    pub fn port(uri: &Uri<String>) -> Option<u16> {
        uri.authority()?.port_to_u16().ok()?
    }

    pub fn user(uri: &Uri<String>) -> Option<String> {
        uri.authority()?
            .userinfo()?
            .as_str()
            .split(':')
            .next()?
            .to_owned()
            .into()
    }

    pub fn password(uri: &Uri<String>) -> Option<String> {
        uri.authority()?
            .userinfo()?
            .as_str()
            .split(':')
            .nth(1)?
            .to_owned()
            .into()
    }

    pub fn host(uri: &Uri<String>) -> Option<String> {
        uri.authority()?.host().to_owned().into()
    }

    pub fn ip(uri: &Uri<String>) -> Option<IpAddr> {
        match uri.authority()?.host_parsed() {
            Host::Ipv4(ip) => Some(ip.into()),
            Host::Ipv6(ip) => Some(ip.into()),
            _ => None,
        }
    }

    pub fn domain_name(uri: &Uri<String>) -> Option<String> {
        match uri.authority()?.host_parsed() {
            Host::RegName(name) => Some(name.to_string()),
            _ => None,
        }
    }

    pub fn path_segments(uri: &Uri<String>) -> Split<'_, Path> {
        match uri.path().segments_if_absolute() {
            Some(segments) => segments,
            None => uri.path().split('/'),
        }
    }

    pub fn path(uri: &Uri<String>) -> Option<PathBuf> {
        let path = uri.path().as_str();
        if path.is_empty() {
            None
        } else {
            PathBuf::from_str(path).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("file:relative/file.txt", "relative/file.txt")]
    #[case("file:///absolute/file.txt", "/absolute/file.txt")]
    fn target2_file(#[case] uri: &str, #[case] path: &str) {
        let target = Target::from_str(uri).unwrap();
        let path_should = PathBuf::from_str(path).unwrap();
        match target {
            Target::File { path } => assert_eq!(path, path_should),
            _ => panic!("target type incorrect"),
        };
    }

    #[rstest]
    #[case("cidr://127.0.0.0/32", "127.0.0.0/32", None, None)]
    #[case("cidr://root@127.0.0.0:22/32", "127.0.0.0/32", "root", 22)]
    #[case("cidr://[::1]/128", "::1/128", None, None)]
    #[case("cidr://root@[::1]:22/128", "::1/128", "root", 22)]
    fn target2_cidr(
        #[case] uri: &str,
        #[case] cidr: &str,
        #[case] user: impl Into<Option<&'static str>>,
        #[case] port: impl Into<Option<u16>>,
    ) {
        let target = Target::from_str(uri).unwrap();
        let network_should = IpNet::from_str(cidr).unwrap();
        let user_should: Option<String> = user.into().map(ToOwned::to_owned);
        let port_should: Option<u16> = port.into();
        match target {
            Target::Cidr {
                user,
                network,
                port,
            } => {
                assert_eq!(network, network_should);
                assert_eq!(user, user_should);
                assert_eq!(port, port_should);
            }
            _ => panic!("target type incorrect"),
        };
    }
}
