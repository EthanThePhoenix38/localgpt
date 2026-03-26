//! SSRF (Server-Side Request Forgery) protection for the web_fetch tool.
//!
//! Validates URLs before any HTTP request is made:
//! - Blocks private/internal IP ranges (IPv4 and IPv6)
//! - Blocks known metadata endpoints (AWS, GCP, Azure)
//! - Blocks dangerous hostnames (localhost, *.local, *.internal)
//! - DNS pinning: resolves hostnames and checks resolved IPs before connecting
//! - Only allows http:// and https:// schemes

use anyhow::Result;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Check whether an IP address belongs to a private, reserved, or internal range
/// that should not be reachable from web_fetch.
///
/// Blocked IPv4 ranges:
/// - 0.0.0.0/8       (current network)
/// - 10.0.0.0/8      (private, RFC 1918)
/// - 100.64.0.0/10   (carrier-grade NAT, RFC 6598)
/// - 127.0.0.0/8     (loopback)
/// - 169.254.0.0/16  (link-local)
/// - 172.16.0.0/12   (private, RFC 1918)
/// - 192.0.0.0/24    (IETF protocol assignments)
/// - 192.0.2.0/24    (documentation, TEST-NET-1)
/// - 192.168.0.0/16  (private, RFC 1918)
/// - 198.18.0.0/15   (benchmarking)
/// - 198.51.100.0/24 (documentation, TEST-NET-2)
/// - 203.0.113.0/24  (documentation, TEST-NET-3)
/// - 224.0.0.0/4     (multicast)
/// - 240.0.0.0/4     (reserved/broadcast)
///
/// Blocked IPv6 ranges:
/// - ::1             (loopback)
/// - ::              (unspecified)
/// - fe80::/10       (link-local)
/// - fc00::/7        (unique local)
/// - ::ffff:0:0/96   (IPv4-mapped — checked against IPv4 rules)
/// - 2001:db8::/32   (documentation)
/// - ff00::/8        (multicast)
pub fn is_private_ip(addr: &IpAddr) -> bool {
    match addr {
        IpAddr::V4(ip) => is_private_ipv4(ip),
        IpAddr::V6(ip) => is_private_ipv6(ip),
    }
}

fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();

    // 0.0.0.0/8 — current network
    octets[0] == 0
    // 10.0.0.0/8 — private
    || octets[0] == 10
    // 100.64.0.0/10 — carrier-grade NAT (RFC 6598)
    || (octets[0] == 100 && (octets[1] & 0xC0) == 64)
    // 127.0.0.0/8 — loopback
    || octets[0] == 127
    // 169.254.0.0/16 — link-local
    || (octets[0] == 169 && octets[1] == 254)
    // 172.16.0.0/12 — private
    || (octets[0] == 172 && (octets[1] & 0xF0) == 16)
    // 192.0.0.0/24 — IETF protocol assignments
    || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
    // 192.0.2.0/24 — documentation (TEST-NET-1)
    || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
    // 192.168.0.0/16 — private
    || (octets[0] == 192 && octets[1] == 168)
    // 198.18.0.0/15 — benchmarking
    || (octets[0] == 198 && (octets[1] & 0xFE) == 18)
    // 198.51.100.0/24 — documentation (TEST-NET-2)
    || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
    // 203.0.113.0/24 — documentation (TEST-NET-3)
    || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
    // 224.0.0.0/4 — multicast
    || (octets[0] & 0xF0) == 224
    // 240.0.0.0/4 — reserved/broadcast (includes 255.255.255.255)
    || (octets[0] & 0xF0) == 240
}

fn is_private_ipv6(ip: &Ipv6Addr) -> bool {
    // Check IPv4-mapped addresses (::ffff:x.x.x.x) first
    if let Some(mapped_v4) = ip.to_ipv4_mapped() {
        return is_private_ipv4(&mapped_v4);
    }

    let segments = ip.segments();

    // ::1 — loopback
    ip.is_loopback()
    // :: — unspecified
    || ip.is_unspecified()
    // fe80::/10 — link-local
    || (segments[0] & 0xFFC0) == 0xFE80
    // fc00::/7 — unique local address
    || (segments[0] & 0xFE00) == 0xFC00
    // 2001:db8::/32 — documentation
    || (segments[0] == 0x2001 && segments[1] == 0x0DB8)
    // ff00::/8 — multicast
    || (segments[0] & 0xFF00) == 0xFF00
}

/// Check whether a hostname is blocked (case-insensitive).
///
/// Blocked hostnames:
/// - localhost
/// - metadata.google.internal (GCP metadata)
/// - 169.254.169.254 (AWS/GCP/Azure metadata IP as hostname)
///
/// Blocked suffixes:
/// - *.local
/// - *.internal
/// - *.localhost
pub fn is_blocked_hostname(host: &str) -> bool {
    let host = host.to_ascii_lowercase();

    const BLOCKED_EXACT: &[&str] = &["localhost", "metadata.google.internal", "169.254.169.254"];
    const BLOCKED_SUFFIXES: &[&str] = &[".local", ".internal", ".localhost"];

    BLOCKED_EXACT.contains(&host.as_str())
        || BLOCKED_SUFFIXES.iter().any(|suffix| host.ends_with(suffix))
}

/// Validate a URL for SSRF safety before making any HTTP request.
///
/// Performs the following checks in order:
/// 1. Only http:// and https:// schemes are allowed
/// 2. Hostname must not be in the blocked list
/// 3. If the host is a literal IP, check it against private ranges
/// 4. DNS pinning: resolve the hostname and check all resolved IPs
///
/// Returns the parsed URL on success, or a descriptive error on failure.
pub async fn validate_url(url: &str) -> Result<reqwest::Url> {
    let parsed = reqwest::Url::parse(url)?;

    // 1. Scheme check
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!(
            "Only http/https URLs are allowed, got '{}'",
            parsed.scheme()
        );
    }

    // 2. Host extraction
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("No host in URL"))?;

    // 3. Blocked hostname check
    if is_blocked_hostname(host) {
        anyhow::bail!(
            "Blocked hostname: {} — access to internal/metadata hosts is not allowed",
            host
        );
    }

    // 4. Check if host is a literal IP address.
    //    host_str() returns IPv6 without brackets (e.g. "::1" not "[::1]"),
    //    so we can parse it directly as IpAddr.
    //    However, some url crate versions may include brackets — handle both.
    let ip_candidate = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ip) = ip_candidate.parse::<IpAddr>() {
        if is_private_ip(&ip) {
            anyhow::bail!(
                "URL resolves to private/reserved IP {} — blocked for SSRF protection",
                ip
            );
        }
        return Ok(parsed);
    }

    // 5. DNS pinning: resolve the hostname and check every resolved address
    let port = parsed.port_or_known_default().unwrap_or(443);
    let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| anyhow::anyhow!("DNS resolution failed for '{}': {}", host, e))?
        .collect();

    if addrs.is_empty() {
        anyhow::bail!("DNS resolution for '{}' returned no addresses", host);
    }

    for addr in &addrs {
        if is_private_ip(&addr.ip()) {
            anyhow::bail!(
                "URL '{}' resolves to private/reserved IP {} — blocked for SSRF protection",
                url,
                addr.ip()
            );
        }
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------
    // is_private_ip — IPv4
    // ---------------------------------------------------------------

    #[test]
    fn ipv4_loopback_blocked() {
        assert!(is_private_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_private_ip(&"127.255.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_10_range_blocked() {
        assert!(is_private_ip(&"10.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"10.0.0.5".parse().unwrap()));
        assert!(is_private_ip(&"10.255.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_172_16_range_blocked() {
        assert!(is_private_ip(&"172.16.0.0".parse().unwrap()));
        assert!(is_private_ip(&"172.16.0.1".parse().unwrap()));
        assert!(is_private_ip(&"172.31.255.255".parse().unwrap()));
        // 172.15.x and 172.32.x should NOT be blocked
        assert!(!is_private_ip(&"172.15.255.255".parse().unwrap()));
        assert!(!is_private_ip(&"172.32.0.0".parse().unwrap()));
    }

    #[test]
    fn ipv4_192_168_range_blocked() {
        assert!(is_private_ip(&"192.168.0.0".parse().unwrap()));
        assert!(is_private_ip(&"192.168.1.1".parse().unwrap()));
        assert!(is_private_ip(&"192.168.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_link_local_blocked() {
        assert!(is_private_ip(&"169.254.0.0".parse().unwrap()));
        assert!(is_private_ip(&"169.254.0.10".parse().unwrap()));
        assert!(is_private_ip(&"169.254.169.254".parse().unwrap()));
        assert!(is_private_ip(&"169.254.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_zero_network_blocked() {
        assert!(is_private_ip(&"0.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"0.1.2.3".parse().unwrap()));
        assert!(is_private_ip(&"0.255.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_cgnat_blocked() {
        // 100.64.0.0/10 — carrier-grade NAT
        assert!(is_private_ip(&"100.64.0.0".parse().unwrap()));
        assert!(is_private_ip(&"100.127.255.255".parse().unwrap()));
        // Just outside the range
        assert!(!is_private_ip(&"100.128.0.0".parse().unwrap()));
        assert!(!is_private_ip(&"100.63.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_multicast_blocked() {
        assert!(is_private_ip(&"224.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"239.255.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_reserved_broadcast_blocked() {
        assert!(is_private_ip(&"240.0.0.0".parse().unwrap()));
        assert!(is_private_ip(&"255.255.255.255".parse().unwrap()));
    }

    #[test]
    fn ipv4_documentation_ranges_blocked() {
        assert!(is_private_ip(&"192.0.2.1".parse().unwrap())); // TEST-NET-1
        assert!(is_private_ip(&"198.51.100.1".parse().unwrap())); // TEST-NET-2
        assert!(is_private_ip(&"203.0.113.1".parse().unwrap())); // TEST-NET-3
    }

    #[test]
    fn ipv4_benchmarking_blocked() {
        assert!(is_private_ip(&"198.18.0.0".parse().unwrap()));
        assert!(is_private_ip(&"198.19.255.255".parse().unwrap()));
        assert!(!is_private_ip(&"198.20.0.0".parse().unwrap()));
    }

    #[test]
    fn ipv4_public_allowed() {
        assert!(!is_private_ip(&"8.8.8.8".parse().unwrap()));
        assert!(!is_private_ip(&"1.1.1.1".parse().unwrap()));
        assert!(!is_private_ip(&"93.184.216.34".parse().unwrap()));
        assert!(!is_private_ip(&"142.250.80.46".parse().unwrap()));
    }

    // ---------------------------------------------------------------
    // is_private_ip — IPv6
    // ---------------------------------------------------------------

    #[test]
    fn ipv6_loopback_blocked() {
        assert!(is_private_ip(&"::1".parse().unwrap()));
    }

    #[test]
    fn ipv6_unspecified_blocked() {
        assert!(is_private_ip(&"::".parse().unwrap()));
    }

    #[test]
    fn ipv6_link_local_blocked() {
        assert!(is_private_ip(&"fe80::1".parse().unwrap()));
        assert!(is_private_ip(&"fe80::abcd:ef01:2345:6789".parse().unwrap()));
    }

    #[test]
    fn ipv6_unique_local_blocked() {
        assert!(is_private_ip(&"fc00::1".parse().unwrap()));
        assert!(is_private_ip(&"fd00::1".parse().unwrap()));
        assert!(is_private_ip(&"fdff::1".parse().unwrap()));
    }

    #[test]
    fn ipv6_documentation_blocked() {
        assert!(is_private_ip(&"2001:db8::1".parse().unwrap()));
        assert!(is_private_ip(&"2001:db8:ffff::1".parse().unwrap()));
    }

    #[test]
    fn ipv6_multicast_blocked() {
        assert!(is_private_ip(&"ff02::1".parse().unwrap()));
        assert!(is_private_ip(&"ff0e::1".parse().unwrap()));
    }

    #[test]
    fn ipv6_mapped_ipv4_private_blocked() {
        // ::ffff:127.0.0.1 — IPv4-mapped loopback
        assert!(is_private_ip(
            &"::ffff:127.0.0.1".parse::<IpAddr>().unwrap()
        ));
        // ::ffff:10.0.0.1 — IPv4-mapped private
        assert!(is_private_ip(&"::ffff:10.0.0.1".parse::<IpAddr>().unwrap()));
        // ::ffff:192.168.1.1 — IPv4-mapped private
        assert!(is_private_ip(
            &"::ffff:192.168.1.1".parse::<IpAddr>().unwrap()
        ));
        // ::ffff:169.254.169.254 — IPv4-mapped metadata
        assert!(is_private_ip(
            &"::ffff:169.254.169.254".parse::<IpAddr>().unwrap()
        ));
    }

    #[test]
    fn ipv6_mapped_ipv4_public_allowed() {
        // ::ffff:8.8.8.8 — IPv4-mapped public
        assert!(!is_private_ip(&"::ffff:8.8.8.8".parse::<IpAddr>().unwrap()));
    }

    #[test]
    fn ipv6_public_allowed() {
        assert!(!is_private_ip(&"2606:4700:4700::1111".parse().unwrap()));
        assert!(!is_private_ip(&"2001:4860:4860::8888".parse().unwrap()));
    }

    // ---------------------------------------------------------------
    // is_blocked_hostname
    // ---------------------------------------------------------------

    #[test]
    fn blocked_hostnames_exact() {
        assert!(is_blocked_hostname("localhost"));
        assert!(is_blocked_hostname("LOCALHOST"));
        assert!(is_blocked_hostname("metadata.google.internal"));
        assert!(is_blocked_hostname("169.254.169.254"));
    }

    #[test]
    fn blocked_hostnames_suffixes() {
        assert!(is_blocked_hostname("printer.local"));
        assert!(is_blocked_hostname("my-service.internal"));
        assert!(is_blocked_hostname("test.localhost"));
        assert!(is_blocked_hostname("deep.nested.localhost"));
    }

    #[test]
    fn allowed_hostnames() {
        assert!(!is_blocked_hostname("example.com"));
        assert!(!is_blocked_hostname("api.github.com"));
        assert!(!is_blocked_hostname("google.com"));
        // "localhosted.com" should not be blocked
        assert!(!is_blocked_hostname("localhosted.com"));
        // "internalize.com" should not be blocked
        assert!(!is_blocked_hostname("internalize.com"));
    }

    // ---------------------------------------------------------------
    // validate_url — async integration tests
    // ---------------------------------------------------------------

    #[tokio::test]
    async fn rejects_non_http_schemes() {
        let err = validate_url("file:///etc/passwd").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Only http/https"));

        let err = validate_url("ftp://example.com/file").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Only http/https"));

        let err = validate_url("gopher://example.com").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn rejects_private_ip_urls() {
        let err = validate_url("http://127.0.0.1/admin").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("private/reserved IP"));

        let err = validate_url("http://10.0.0.1/internal").await;
        assert!(err.is_err());

        let err = validate_url("http://192.168.1.1/router").await;
        assert!(err.is_err());

        let err = validate_url("http://0.0.0.0/").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn rejects_metadata_ip() {
        let err = validate_url("http://169.254.169.254/latest/meta-data/").await;
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        // Could match either hostname block or IP block
        assert!(
            msg.contains("Blocked hostname") || msg.contains("private/reserved IP"),
            "unexpected error: {}",
            msg
        );
    }

    #[tokio::test]
    async fn rejects_blocked_hostnames() {
        let err = validate_url("http://localhost/api").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Blocked hostname"));

        let err = validate_url("http://metadata.google.internal/computeMetadata/v1/").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Blocked hostname"));

        let err = validate_url("http://service.internal/api").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn rejects_ipv6_loopback_url() {
        let err = validate_url("http://[::1]/admin").await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("private/reserved IP"));
    }

    #[tokio::test]
    async fn rejects_url_without_host() {
        let err = validate_url("http:///no-host").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn allows_public_ip_url() {
        // 93.184.216.34 is example.com's IP — a literal IP so no DNS lookup needed
        let result = validate_url("https://93.184.216.34/").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn allows_public_domain() {
        // This requires DNS resolution — if it fails in CI due to no network,
        // the test should still pass since DNS failure is a different error path.
        let result = validate_url("https://example.com/").await;
        // In environments with DNS, this should succeed.
        // In environments without DNS, it will fail with a DNS error, not an SSRF error.
        match result {
            Ok(url) => assert_eq!(url.host_str(), Some("example.com")),
            Err(e) => {
                let msg = e.to_string();
                // DNS failure is acceptable in sandboxed test environments
                assert!(
                    msg.contains("DNS resolution failed"),
                    "unexpected error: {}",
                    msg
                );
            }
        }
    }

    #[tokio::test]
    async fn rejects_ipv6_private_in_bracket_notation() {
        let err = validate_url("http://[fe80::1]/").await;
        assert!(err.is_err());

        let err = validate_url("http://[fc00::1]/").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn preserves_url_components() {
        // Ensure the returned URL preserves path, query, fragment
        let result = validate_url("https://93.184.216.34/path?q=1#frag").await;
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.path(), "/path");
        assert_eq!(url.query(), Some("q=1"));
        assert_eq!(url.fragment(), Some("frag"));
    }
}
