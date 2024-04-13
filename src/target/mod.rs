use std::net::IpAddr;
use std::net::SocketAddr;
use std::str::FromStr;

use thiserror::Error;
use url::Url;

mod ssh;

/// Errors regarding [`Target`].
#[derive(Debug, Clone, Error)]
pub enum TargetError {
    #[error("unsupported target URI scheme: {0}")]
    UnsupportedUriScheme(String),

    #[error("missing host in target URI")]
    NoHostInUri,

    #[error("unable to parse target from string: {0}")]
    Unparsable(String),
}

/// [`Target`] is a single receiver of an action to be executed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// Not a real target. Actions against this target will be ignored.
    Dummy,

    /// The local machine. Actions against this target will be executed
    /// within the current process.
    Local,

    /// An SSH connection to a remote machine. Actions against this target will
    /// be executed using an SSH session.
    Ssh {
        host: String,
        port: Option<u16>,
        user: Option<String>,
    },
}

impl Target {
    fn run_command(&self, _command: &str) -> anyhow::Result<()> {
        match self.to_owned() {
            Target::Dummy => Ok(()),
            Target::Local => todo!(),
            Target::Ssh {
                host: _,
                port: _,
                user: _,
            } => todo!(),
        }
    }
}

// URI parsing ----------------------------------------------------------------

impl TryFrom<Url> for Target {
    type Error = TargetError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match value.scheme() {
            "dummy" => parse_dummy_uri(value),
            "local" => parse_local_uri(value),
            "ssh" => parse_ssh_uri(value),
            scheme => Err(TargetError::UnsupportedUriScheme(scheme.to_owned())),
        }
    }
}

fn parse_dummy_uri(_value: Url) -> Result<Target, TargetError> {
    Ok(Target::Dummy)
}

fn parse_local_uri(_value: Url) -> Result<Target, TargetError> {
    Ok(Target::Local)
}

fn parse_ssh_uri(value: Url) -> Result<Target, TargetError> {
    let Some(host) = value.host_str() else {
        return Err(TargetError::NoHostInUri);
    };
    let host = host.to_owned();

    let port = value.port();

    let user = if value.username().is_empty() {
        None
    } else {
        Some(value.username().to_owned())
    };

    Ok(Target::Ssh { host, port, user })
}

// String parsing -------------------------------------------------------------

impl FromStr for Target {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Interpret IP addresses as SSH
        if let Ok(ip_addr) = IpAddr::from_str(s) {
            return Ok(Target::Ssh {
                host: ip_addr.to_string(),
                port: None,
                user: None,
            });
        }

        // Interpret socket addresses as SSH
        if let Ok(socket_addr) = SocketAddr::from_str(s) {
            return Ok(Target::Ssh {
                host: socket_addr.ip().to_string(),
                port: Some(socket_addr.port()),
                user: None,
            });
        }

        // Interpret URIs accordingly
        if let Ok(uri) = Url::from_str(s) {
            return Target::try_from(uri);
        }

        Err(TargetError::Unparsable(s.to_owned()))
    }
}

// Tests ----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use url::Url;

    use super::*;

    #[rstest]
    #[case::dummy("dummy:", Target::Dummy)]
    #[case::local("local:", Target::Local)]
    #[case::ssh("ssh://host", Target::Ssh { host: "host".to_owned(), port: None, user: None })]
    #[case::ssh("ssh://host:22", Target::Ssh { host: "host".to_owned(), port: Some(22), user: None })]
    #[case::ssh("ssh://user@host", Target::Ssh { host: "host".to_owned(), port: None, user: Some("user".to_owned()) })]
    #[case::ssh("ssh://user@host:22", Target::Ssh { host: "host".to_owned(), port: Some(22), user: Some("user".to_owned()) })]
    fn target_from_uri(#[case] target_uri: Url, #[case] target_should: Target) {
        let target_got = Target::try_from(target_uri).unwrap();
        assert_eq!(target_got, target_should);
    }

    #[rstest]
    #[case::unsupported_scheme("nosuchscheme:")]
    fn target_from_uri_error(#[case] target_uri: Url) {
        let result = Target::try_from(target_uri);
        assert!(result.is_err());
    }

    #[rstest]
    #[case::ssh_ipv4("127.0.0.1", Target::Ssh { host: "127.0.0.1".to_owned(), port: None, user: None })]
    #[case::ssh_socketv4("127.0.0.1:22", Target::Ssh { host: "127.0.0.1".to_owned(), port: Some(22), user: None })]
    #[case::ssh_ipv6("::1", Target::Ssh { host: "::1".to_owned(), port: None, user: None })]
    #[case::ssh_socketv6("[::1]:22", Target::Ssh { host: "::1".to_owned(), port: Some(22), user: None })]
    #[case::ssh("ssh://user@host:22", Target::Ssh { host: "host".to_owned(), port: Some(22), user: Some("user".to_owned()) })]
    fn target_from_str(#[case] target_str: &str, #[case] target_should: Target) {
        let target_got = Target::from_str(target_str).unwrap();
        assert_eq!(target_got, target_should);
    }
}
