package ping

import (
	"fmt"
	"net"
	"sync"
	"time"

	"github.com/goware/urlx"
	"github.com/pbar1/astu/internal/pkg/printer"
)

type PingOptions struct {
	// Raw URL that will be parsed
	RawURL string

	// Override the parsed URL scheme with this value
	Scheme string

	// Duration after which the connectivity check will fail
	Timeout time.Duration

	// Enable to allow IPv6 IP addresses to be used in connectivity check
	AllowIPv6 bool
}

const (
	SchemeTCP = "tcp"
	SchemeUDP = "udp"
)

/*
Go through the whole chain. Given an input URI:
- Assess it is an IP or a DNS entry
- If DNS, resolve to list of IPs
- Test IP can make a TCP connection (and do something with UDP)
- Test TCP connection can make an HTTP connection (with TLS)
*/

func Ping(opts *PingOptions) error {
	u, err := urlx.Parse(opts.RawURL)
	if err != nil {
		return err
	}

	port := u.Port()
	if port == "" {
		return fmt.Errorf("no port found: %s", opts.RawURL)
	}

	scheme := u.Scheme
	if opts.Scheme != "" {
		scheme = opts.Scheme
	}

	ips, err := resolveIPs(u.Hostname())
	if err != nil {
		return err
	}

	wg := new(sync.WaitGroup)
	wg.Add(len(ips))
	for _, ip := range ips {
		go checkConnect(wg, ip, port, scheme, opts.Timeout, opts.AllowIPv6)
	}
	wg.Wait()

	return nil
}

func resolveIPs(hostname string) ([]net.IP, error) {
	return net.LookupIP(hostname)
}

func checkConnect(wg *sync.WaitGroup, ip net.IP, port, scheme string, timeout time.Duration, allowIPv6 bool) {
	defer wg.Done()

	var addr string
	if ip.To4() != nil {
		addr = fmt.Sprintf("%s:%s", ip, port)
	} else if ip.To16() != nil {
		if allowIPv6 {
			addr = fmt.Sprintf("[%s]:%s", ip, port)
		} else {
			printer.Neutralf(nil, "%s → Skipped\n", ip)
			return
		}
	} else {
		printer.Badf(nil, "%s → Not a valid IP address\n", ip)
		return
	}

	switch scheme {
	case SchemeUDP:
		fallthrough
	case SchemeTCP:
		_, err := net.DialTimeout(scheme, addr, timeout)
		if err != nil {
			printer.Badf(nil, "%s → %v\n", ip, err)
			return
		}
		printer.Goodf(nil, "%s → Open\n", addr)
	default:
		printer.Badf(nil, "%s → Unsupported scheme: %s\n", addr, scheme)
		return
	}

	return
}
