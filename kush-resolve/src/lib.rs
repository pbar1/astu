mod dns;
mod ip;
mod target;

use async_stream::stream;
pub use dns::DnsResolver;
use futures::Stream;
pub use ip::IpResolver;
pub use target::Target;

pub trait Resolve {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target>;
}

pub struct ForwardResolveChain {
    ip: IpResolver,
    dns: DnsResolver,
}

impl ForwardResolveChain {
    pub fn try_default() -> anyhow::Result<ForwardResolveChain> {
        let ip = IpResolver;
        let dns = DnsResolver::system()?;
        Ok(Self { ip, dns })
    }
}

impl Resolve for ForwardResolveChain {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                Target::Ipv4Addr(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::Ipv6Addr(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::SocketAddrV4(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::SocketAddrV6(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::Ipv4Net(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::Ipv6Net(_) => {
                    for await x in self.ip.resolve(target) {
                        yield x;
                    }
                },
                Target::Domain(_) => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
                Target::DomainPort{ .. } => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
            };
        }
    }
}

pub struct ReverseResolveChain {
    dns: DnsResolver,
}

impl ReverseResolveChain {
    pub fn try_default() -> anyhow::Result<ReverseResolveChain> {
        let dns = DnsResolver::system()?;
        Ok(Self { dns })
    }
}

impl Resolve for ReverseResolveChain {
    fn resolve(&self, target: Target) -> impl Stream<Item = Target> {
        stream! {
            match target {
                Target::Ipv4Addr(_) => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
                Target::Ipv6Addr(_) => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
                Target::SocketAddrV4(_) => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
                Target::SocketAddrV6(_) => {
                    for await x in self.dns.resolve(target) {
                        yield x;
                    }
                },
                // TODO: Resolving IPs in CIDRs to domains should be possible
                // Target::Ipv4Net(_) => {
                //     for await x in self.dns.resolve(target) {
                //         yield x;
                //     }
                // },
                // Target::Ipv6Net(_) => {
                //     for await x in self.dns.resolve(target) {
                //         yield x;
                //     }
                // },
                _unsupported => return,
            };
        }
    }
}
