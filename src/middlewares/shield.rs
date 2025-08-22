#![warn(missing_docs)]
use std::collections::HashMap;

use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};

#[derive(Clone)]
pub struct ShieldConfig {
    pub content_security_policy: ContentSecurityPolicy,
    pub hsts: Hsts,
    pub frameguard: Frameguard,
    pub no_sniff: NoSniff,
    pub xss_filter: XssFilter,
    pub referrer_policy: ReferrerPolicy,
    pub dns_prefetch_control: DnsPrefetchControl,
    pub ie_no_open: IENoOpen,
    pub hide_powered_by: HidePoweredBy,
    pub permissions_policy: PermissionsPolicy,
    pub cross_origin_opener_policy: CrossOriginOpenerPolicy,
    pub cross_origin_resource_policy: CrossOriginResourcePolicy,
    pub cross_origin_embedder_policy: CrossOriginEmbedderPolicy,
    pub origin_agent_cluster: OriginAgentCluster,
    pub cross_domain_policy: CrossDomainPolicy,
}

#[derive(Clone)]
pub struct Hsts {
    pub enabled: bool,
    pub max_age: u64,
    pub include_subdomains: bool,
    pub preload: bool,
}

impl Default for Hsts {
    fn default() -> Self {
        Self {
            enabled: true,
            max_age: 31536000, // 1 year in seconds
            include_subdomains: true,
            preload: false,
        }
    }
}

#[derive(Clone)]
pub struct Frameguard {
    pub enabled: bool,
    pub action: String,
}

impl Default for Frameguard {
    fn default() -> Self {
        Self {
            enabled: true,
            action: "deny".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct NoSniff {
    pub enabled: bool,
}

impl Default for NoSniff {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone)]
pub struct XssFilter {
    pub enabled: bool,
    pub mode: String,
    pub report_uri: Option<String>,
}

impl Default for XssFilter {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: "block".to_string(),
            report_uri: None,
        }
    }
}

#[derive(Clone)]
pub struct ReferrerPolicy {
    pub enabled: bool,
    pub policy: String,
}

impl Default for ReferrerPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            policy: "no-referrer-when-downgrade".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct DnsPrefetchControl {
    pub enabled: bool,
    pub allow: bool,
}

impl Default for DnsPrefetchControl {
    fn default() -> Self {
        Self {
            enabled: false,
            allow: false,
        }
    }
}

#[derive(Clone)]
pub struct IENoOpen {
    pub enabled: bool,
}

impl Default for IENoOpen {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone)]
pub struct HidePoweredBy {
    pub enabled: bool,
}

impl Default for HidePoweredBy {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone)]
pub struct PermissionsPolicy {
    pub enabled: bool,
    pub features: HashMap<String, Vec<String>>,
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            features: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub enum CrossOriginOpenerPolicy {
    SameOrigin,
    SameOriginAllowPopups,
    SameOriginAllowPopupsAndRedirects,
    UnsafeNone,
}

impl Default for CrossOriginOpenerPolicy {
    fn default() -> Self {
        Self::UnsafeNone
    }
}

#[derive(Clone)]
pub enum CrossOriginResourcePolicy {
    SameOrigin,
    SameOriginAllowPopups,
    SameOriginAllowPopupsAndRedirects,
    CrossOrigin,
    CrossOriginAllowPopups,
    CrossOriginAllowPopupsAndRedirects,
    UnsafeNone,
}

impl Default for CrossOriginResourcePolicy {
    fn default() -> Self {
        Self::UnsafeNone
    }
}

#[derive(Clone)]
pub struct ContentSecurityPolicy {
    pub enabled: bool,
    pub directives: HashMap<String, String>,
    pub report_only: bool,
}

impl Default for ContentSecurityPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            directives: HashMap::new(),
            report_only: false,
        }
    }
}

#[derive(Clone)]
pub enum CrossOriginEmbedderPolicy {
    RequireCorp,
    RequireCorpIframe,
    UnsafeNone,
}

impl Default for CrossOriginEmbedderPolicy {
    fn default() -> Self {
        Self::UnsafeNone
    }
}

#[derive(Clone)]
pub struct OriginAgentCluster {
    pub enabled: bool,
}

impl Default for OriginAgentCluster {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Clone)]
pub struct CrossDomainPolicy {
    pub enabled: bool,
    pub policy: String,
}

impl Default for CrossDomainPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            policy: "none".to_string(),
        }
    }
}

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            content_security_policy: ContentSecurityPolicy::default(),
            hsts: Hsts::default(),
            frameguard: Frameguard::default(),
            no_sniff: NoSniff::default(),
            xss_filter: XssFilter::default(),
            referrer_policy: ReferrerPolicy::default(),
            dns_prefetch_control: DnsPrefetchControl::default(),
            ie_no_open: IENoOpen::default(),
            hide_powered_by: HidePoweredBy::default(),
            permissions_policy: PermissionsPolicy::default(),
            cross_origin_opener_policy: CrossOriginOpenerPolicy::default(),
            cross_origin_resource_policy: CrossOriginResourcePolicy::default(),
            cross_origin_embedder_policy: CrossOriginEmbedderPolicy::default(),
            origin_agent_cluster: OriginAgentCluster::default(),
            cross_domain_policy: CrossDomainPolicy::default(),
        }
    }
}

pub(crate) fn shield(
    config: Option<ShieldConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let config = config.unwrap_or_default();
    move |req, mut res| {
        let config = config.clone();
        Box::pin(async move {
            let content_security_policy = config.content_security_policy;
            let hsts = config.hsts;
            let frameguard = config.frameguard;
            let no_sniff = config.no_sniff;
            let xss_filter = config.xss_filter;
            let referrer_policy = config.referrer_policy;
            let dns_prefetch_control = config.dns_prefetch_control;
            let ie_no_open = config.ie_no_open;
            let hide_powered_by = config.hide_powered_by;
            let permissions_policy = config.permissions_policy;
            let cross_origin_opener_policy = config.cross_origin_opener_policy;
            let cross_origin_resource_policy = config.cross_origin_resource_policy;
            let cross_origin_embedder_policy = config.cross_origin_embedder_policy;
            let origin_agent_cluster = config.origin_agent_cluster;
            let cross_domain_policy = config.cross_domain_policy;

            set_content_security_policy(&mut res, content_security_policy);
            set_hsts(&mut res, hsts);
            set_frameguard(&mut res, frameguard);
            set_no_sniff(&mut res, no_sniff);
            set_xss_filter(&mut res, xss_filter);
            set_referrer_policy(&mut res, referrer_policy);
            set_dns_prefetch_control(&mut res, dns_prefetch_control);
            set_ie_no_open(&mut res, ie_no_open);
            set_hide_powered_by(&mut res, hide_powered_by);
            set_permissions_policy(&mut res, permissions_policy);
            set_cross_origin_opener_policy(&mut res, cross_origin_opener_policy);
            set_cross_origin_resource_policy(&mut res, cross_origin_resource_policy);
            set_cross_origin_embedder_policy(&mut res, cross_origin_embedder_policy);
            set_origin_agent_cluster(&mut res, origin_agent_cluster);
            set_cross_domain_policy(&mut res, cross_domain_policy);

            (req, None)
        })
    }
}

fn set_content_security_policy(res: &mut HttpResponse, csp: ContentSecurityPolicy) {
    if !csp.enabled {
        return;
    }

    let header_name = if csp.report_only {
        "Content-Security-Policy-Report-Only"
    } else {
        "Content-Security-Policy"
    };

    let header_value = csp
        .directives
        .iter()
        .map(|(k, v)| format!("{} {}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    res.headers.insert(header_name, header_value);
}

fn set_hsts(res: &mut HttpResponse, hsts: Hsts) {
    if !hsts.enabled {
        return;
    };

    let mut value = format!("max-age={}", hsts.max_age);

    if hsts.include_subdomains {
        value += "; includeSubDomains"
    };
    if hsts.preload {
        value += "; preload"
    };

    res.headers.insert("Strict-Transport-Security", value);
}

fn set_frameguard(res: &mut HttpResponse, frameguard: Frameguard) {}

fn set_no_sniff(res: &mut HttpResponse, no_sniff: NoSniff) {
    if !no_sniff.enabled {
        return;
    };

    res.headers.insert("X-Content-Type-Options", "nosniff");
}

fn set_xss_filter(res: &mut HttpResponse, xss_filter: XssFilter) {
    if !xss_filter.enabled {
        return;
    }

    let mut value = String::from("1");

    if xss_filter.mode == String::from("block") {
        value += "; mode=block";
    }

    if xss_filter.report_uri.is_some() {
        value += format!("; report=${}", xss_filter.report_uri.unwrap()).as_str();
    }

    res.headers.insert("X-XSS-Protection", value);
}

fn set_referrer_policy(res: &mut HttpResponse, referrer_policy: ReferrerPolicy) {
    if !referrer_policy.enabled {
        return;
    }
    res.headers
        .insert("Referrer-Policy", referrer_policy.policy);
}

fn set_dns_prefetch_control(res: &mut HttpResponse, dns_prefetch_control: DnsPrefetchControl) {
    if !dns_prefetch_control.enabled {
        return;
    }

    let header_value = if dns_prefetch_control.allow {
        "on"
    } else {
        "off"
    };

    res.headers.insert("DNS-Prefetch-Control", header_value);
}

fn set_ie_no_open(res: &mut HttpResponse, ie_no_open: IENoOpen) {
    if !ie_no_open.enabled {
        return;
    }

    res.headers.insert("X-Download-Options", "noopen");
}

fn set_hide_powered_by(res: &mut HttpResponse, hide_powered_by: HidePoweredBy) {
    if hide_powered_by.enabled {
        return;
    };
    res.headers.remove("X-Powered-By");
}

fn set_permissions_policy(res: &mut HttpResponse, permissions_policy: PermissionsPolicy) {
    if !permissions_policy.enabled {
        return;
    }

    let mut header_value = String::new();

    for feature in permissions_policy.features.keys() {
        header_value.push_str(feature);
        header_value.push_str(", ");
    }
    let mut policies = Vec::new();

    for (feature, allowlist) in &permissions_policy.features {
        let policy_str = if allowlist.is_empty() {
            format!("{}=()", feature)
        } else {
            let origins: Vec<String> = allowlist
                .iter()
                .map(|origin| {
                    if origin == "self" {
                        "self".to_string()
                    } else {
                        format!("\"{}\"", origin)
                    }
                })
                .collect();
            format!("{}=({})", feature, origins.join(" "))
        };
        policies.push(policy_str);
    }

    if !policies.is_empty() {
        let header_value = policies.join(", ");
        res.headers.insert("Permissions-Policy", header_value);
    }
}

fn set_cross_origin_opener_policy(
    res: &mut HttpResponse,
    cross_origin_opener_policy: CrossOriginOpenerPolicy,
) {
    let header_value = match cross_origin_opener_policy {
        CrossOriginOpenerPolicy::SameOrigin => String::from("same-origin"),
        CrossOriginOpenerPolicy::SameOriginAllowPopups => String::from("same-origin-allow-popups"),
        CrossOriginOpenerPolicy::SameOriginAllowPopupsAndRedirects => {
            String::from("same-origin-allow-popups-and-redirects")
        }
        CrossOriginOpenerPolicy::UnsafeNone => String::from("unsafe-none"),
    };

    res.headers
        .insert("Cross-Origin-Opener-Policy", header_value);
}

fn set_cross_origin_resource_policy(
    res: &mut HttpResponse,
    cross_origin_resource_policy: CrossOriginResourcePolicy,
) {
    let header_value = match cross_origin_resource_policy {
        CrossOriginResourcePolicy::SameOrigin => String::from("same-origin"),
        CrossOriginResourcePolicy::SameOriginAllowPopups => {
            String::from("same-origin-allow-popups")
        }
        CrossOriginResourcePolicy::SameOriginAllowPopupsAndRedirects => {
            String::from("same-origin-allow-popups-and-redirects")
        }
        CrossOriginResourcePolicy::CrossOrigin => String::from("cross-origin"),
        CrossOriginResourcePolicy::CrossOriginAllowPopups => {
            String::from("cross-origin-allow-popups")
        }
        CrossOriginResourcePolicy::CrossOriginAllowPopupsAndRedirects => {
            String::from("cross-origin-allow-popups-and-redirects")
        }
        CrossOriginResourcePolicy::UnsafeNone => String::from("unsafe-none"),
    };

    res.headers
        .insert("Cross-Origin-Resource-Policy", header_value);
}

fn set_cross_origin_embedder_policy(
    res: &mut HttpResponse,
    cross_origin_embedder_policy: CrossOriginEmbedderPolicy,
) {
    let header_value = match cross_origin_embedder_policy {
        CrossOriginEmbedderPolicy::RequireCorp => String::from("require-corp"),
        CrossOriginEmbedderPolicy::RequireCorpIframe => String::from("require-corp-iframe"),
        CrossOriginEmbedderPolicy::UnsafeNone => String::from("unsafe-none"),
    };

    res.headers
        .insert("Cross-Origin-Embedder-Policy", header_value);
}

fn set_origin_agent_cluster(res: &mut HttpResponse, origin_agent_cluster: OriginAgentCluster) {
    if !origin_agent_cluster.enabled {
        return;
    }

    res.headers.insert("Origin-Agent-Cluster", "?1");
}

fn set_cross_domain_policy(res: &mut HttpResponse, cross_domain_policy: CrossDomainPolicy) {
    if !cross_domain_policy.enabled {
        return;
    }

    res.headers
        .insert("Cross-Domain-Policy", cross_domain_policy.policy);
}
