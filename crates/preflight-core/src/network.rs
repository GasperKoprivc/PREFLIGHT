use crate::model::ReasonCode;
use std::{
    future::Future,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use url::{Host, Url};

/// Conservative public-address policy, shared by literal URLs and resolved DNS.
pub fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let [a, b, c, _] = v.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || (a == 100 && (64..=127).contains(&b))
                || (a == 169 && b == 254)
                || (a == 172 && (16..=31).contains(&b))
                || (a == 192
                    && (b == 168 || (b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
                || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
                || (a == 203 && b == 0 && c == 113))
        }
        IpAddr::V6(v) => {
            let s = v.segments();
            (s[0] & 0xe000) == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x0200 || s[1] == 0x0db8))
                && s[0] != 0x2002
                && !(s[0] == 0x3fff && (s[1] & 0xf000) == 0)
        }
    }
}
pub fn validate_url(input: &str) -> Result<Url, ReasonCode> {
    if input.len() > 2048 || input.chars().any(char::is_control) {
        return Err(ReasonCode::UnsafeUrl);
    }
    let u = Url::parse(input).map_err(|_| ReasonCode::UnsafeUrl)?;
    if u.scheme() != "https"
        || !u.username().is_empty()
        || u.password().is_some()
        || u.port_or_known_default() != Some(443)
        || u.fragment().is_some()
    {
        return Err(ReasonCode::UnsafeUrl);
    }
    match u.host().ok_or(ReasonCode::UnsafeUrl)? {
        Host::Domain(d)
            if d.trim_end_matches('.').eq_ignore_ascii_case("localhost")
                || d.ends_with(".localhost")
                || !d.contains('.') =>
        {
            return Err(ReasonCode::UnsafeUrl)
        }
        Host::Ipv4(ip) if !public_ip(ip.into()) => return Err(ReasonCode::UnsafeUrl),
        Host::Ipv6(ip) if !public_ip(ip.into()) => return Err(ReasonCode::UnsafeUrl),
        _ => {}
    }
    Ok(u)
}
pub struct Hop {
    pub status: u16,
    pub location: Option<String>,
}
pub trait Transport: Sync {
    fn resolve(
        &self,
        host: &str,
    ) -> impl Future<Output = Result<Vec<SocketAddr>, ReasonCode>> + Send;
    fn head(
        &self,
        url: &Url,
        pinned: &[SocketAddr],
    ) -> impl Future<Output = Result<Hop, ReasonCode>> + Send;
}
pub struct SystemTransport;
impl Transport for SystemTransport {
    async fn resolve(&self, host: &str) -> Result<Vec<SocketAddr>, ReasonCode> {
        tokio::net::lookup_host((host, 443))
            .await
            .map(|a| a.collect())
            .map_err(|_| ReasonCode::NetworkUnavailable)
    }
    async fn head(&self, url: &Url, pinned: &[SocketAddr]) -> Result<Hop, ReasonCode> {
        let host = url.host_str().ok_or(ReasonCode::UnsafeUrl)?;
        let client = reqwest::Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .referer(false)
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .resolve_to_addrs(host, pinned)
            .build()
            .map_err(|_| ReasonCode::NetworkUnavailable)?;
        let response = client.head(url.clone()).send().await.map_err(|e| {
            if e.is_timeout() {
                ReasonCode::Timeout
            } else {
                ReasonCode::NetworkUnavailable
            }
        })?;
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|h| h.to_str().ok())
            .map(str::to_owned);
        Ok(Hop {
            status: response.status().as_u16(),
            location,
        })
    }
}
pub async fn check_https(input: &str, transport: &impl Transport) -> ReasonCode {
    tokio::time::timeout(Duration::from_secs(12), async {
        let mut url = match validate_url(input) {
            Ok(u) => u,
            Err(e) => return e,
        };
        for hop in 0..=3 {
            let addresses = match url.host() {
                Some(Host::Ipv4(ip)) => vec![SocketAddr::new(ip.into(), 443)],
                Some(Host::Ipv6(ip)) => vec![SocketAddr::new(ip.into(), 443)],
                Some(Host::Domain(domain)) => match transport.resolve(domain).await {
                    Ok(a) => a,
                    Err(e) => return e,
                },
                None => return ReasonCode::UnsafeUrl,
            };
            if addresses.is_empty() {
                return ReasonCode::NetworkUnavailable;
            }
            if addresses
                .iter()
                .any(|a| !public_ip(a.ip()) || a.port() != 443)
            {
                return ReasonCode::UnsafeUrl;
            }
            // The request MUST use these vetted addresses, with the original TLS hostname.
            let response = match transport.head(&url, &addresses).await {
                Ok(r) => r,
                Err(e) => return e,
            };
            if (200..300).contains(&response.status) {
                return ReasonCode::HttpsReachable;
            }
            if [301, 302, 303, 307, 308].contains(&response.status) {
                if hop == 3 {
                    return ReasonCode::RedirectLimit;
                }
                let Some(location) = response.location else {
                    return ReasonCode::NetworkUnavailable;
                };
                let next = match url.join(&location) {
                    Ok(u) => u,
                    Err(_) => return ReasonCode::UnsafeUrl,
                };
                url = match validate_url(next.as_str()) {
                    Ok(u) => u,
                    Err(e) => return e,
                };
            } else {
                return ReasonCode::HttpsStatus;
            }
        }
        ReasonCode::RedirectLimit
    })
    .await
    .unwrap_or(ReasonCode::Timeout)
}
