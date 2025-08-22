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
    pub domain: Option<String>, // For allow-from action
}

impl Default for Frameguard {
    fn default() -> Self {
        Self {
            enabled: true,
            action: "deny".to_string(),
            domain: None,
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
            enabled: true, // Changed from false
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
        let mut features = HashMap::new();
        features.insert("camera".to_string(), vec![]);
        features.insert("microphone".to_string(), vec![]);
        features.insert("geolocation".to_string(), vec!["self".to_string()]);
        features.insert("payment".to_string(), vec![]);

        Self {
            enabled: true,
            features,
        }
    }
}

#[derive(Clone)]
pub enum CrossOriginOpenerPolicy {
    SameOrigin,
    SameOriginAllowPopups,
    UnsafeNone,
}

impl Default for CrossOriginOpenerPolicy {
    fn default() -> Self {
        Self::SameOrigin // Changed from UnsafeNone for better security
    }
}

#[derive(Clone)]
pub enum CrossOriginResourcePolicy {
    SameOrigin,
    SameSite,
    CrossOrigin,
}

impl Default for CrossOriginResourcePolicy {
    fn default() -> Self {
        Self::SameOrigin // Simplified and more secure
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
        let mut directives = HashMap::new();
        directives.insert("default-src".to_string(), "'self'".to_string());
        directives.insert("script-src".to_string(), "'self'".to_string());
        directives.insert(
            "style-src".to_string(),
            "'self' 'unsafe-inline'".to_string(),
        );
        directives.insert("img-src".to_string(), "'self' data: https:".to_string());
        directives.insert("object-src".to_string(), "'none'".to_string());

        Self {
            enabled: true,
            directives,
            report_only: false,
        }
    }
}

#[derive(Clone)]
pub enum CrossOriginEmbedderPolicy {
    RequireCorp,
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
        Self { enabled: true } // Changed from false
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
            enabled: true, // Changed from false
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
            set_content_security_policy(&mut res, &config.content_security_policy);
            set_hsts(&mut res, &config.hsts);
            set_frameguard(&mut res, &config.frameguard);
            set_no_sniff(&mut res, &config.no_sniff);
            set_xss_filter(&mut res, &config.xss_filter);
            set_referrer_policy(&mut res, &config.referrer_policy);
            set_dns_prefetch_control(&mut res, &config.dns_prefetch_control);
            set_ie_no_open(&mut res, &config.ie_no_open);
            set_hide_powered_by(&mut res, &config.hide_powered_by);
            set_permissions_policy(&mut res, &config.permissions_policy);
            set_cross_origin_opener_policy(&mut res, &config.cross_origin_opener_policy);
            set_cross_origin_resource_policy(&mut res, &config.cross_origin_resource_policy);
            set_cross_origin_embedder_policy(&mut res, &config.cross_origin_embedder_policy);
            set_origin_agent_cluster(&mut res, &config.origin_agent_cluster);
            set_cross_domain_policy(&mut res, &config.cross_domain_policy);

            (req, None)
        })
    }
}

fn set_content_security_policy(res: &mut HttpResponse, csp: &ContentSecurityPolicy) {
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

fn set_hsts(res: &mut HttpResponse, hsts: &Hsts) {
    if !hsts.enabled {
        return;
    }

    let mut value = format!("max-age={}", hsts.max_age);

    if hsts.include_subdomains {
        value.push_str("; includeSubDomains");
    }
    if hsts.preload {
        value.push_str("; preload");
    }

    res.headers.insert("Strict-Transport-Security", value);
}

fn set_frameguard(res: &mut HttpResponse, frameguard: &Frameguard) {
    if !frameguard.enabled {
        return;
    }

    let value = match frameguard.action.as_str() {
        "deny" => "DENY".to_string(),
        "sameorigin" => "SAMEORIGIN".to_string(),
        "allow-from" => {
            if let Some(domain) = &frameguard.domain {
                format!("ALLOW-FROM {}", domain)
            } else {
                "DENY".to_string() // Fallback if no domain specified
            }
        }
        _ => "DENY".to_string(), // Default fallback
    };

    res.headers.insert("X-Frame-Options", value);
}

fn set_no_sniff(res: &mut HttpResponse, no_sniff: &NoSniff) {
    if !no_sniff.enabled {
        return;
    }

    res.headers.insert("X-Content-Type-Options", "nosniff");
}

fn set_xss_filter(res: &mut HttpResponse, xss_filter: &XssFilter) {
    if !xss_filter.enabled {
        return;
    }

    let mut value = "1".to_string();

    if xss_filter.mode == "block" {
        value.push_str("; mode=block");
    }

    if let Some(report_uri) = &xss_filter.report_uri {
        value.push_str(&format!("; report={}", report_uri));
    }

    res.headers.insert("X-XSS-Protection", value);
}

fn set_referrer_policy(res: &mut HttpResponse, referrer_policy: &ReferrerPolicy) {
    if !referrer_policy.enabled {
        return;
    }
    res.headers
        .insert("Referrer-Policy", referrer_policy.policy.clone());
}

fn set_dns_prefetch_control(res: &mut HttpResponse, dns_prefetch_control: &DnsPrefetchControl) {
    if !dns_prefetch_control.enabled {
        return;
    }

    let header_value = if dns_prefetch_control.allow {
        "on"
    } else {
        "off"
    };

    res.headers.insert("X-DNS-Prefetch-Control", header_value); // Fixed header name
}

fn set_ie_no_open(res: &mut HttpResponse, ie_no_open: &IENoOpen) {
    if !ie_no_open.enabled {
        return;
    }

    res.headers.insert("X-Download-Options", "noopen");
}

fn set_hide_powered_by(res: &mut HttpResponse, hide_powered_by: &HidePoweredBy) {
    if !hide_powered_by.enabled {
        // Fixed logic - was backwards
        return;
    }
    res.headers.remove("X-Powered-By");
}

fn set_permissions_policy(res: &mut HttpResponse, permissions_policy: &PermissionsPolicy) {
    if !permissions_policy.enabled {
        return;
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
    cross_origin_opener_policy: &CrossOriginOpenerPolicy,
) {
    let header_value = match cross_origin_opener_policy {
        CrossOriginOpenerPolicy::SameOrigin => "same-origin",
        CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
        CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
    };

    res.headers
        .insert("Cross-Origin-Opener-Policy", header_value.to_string());
}

fn set_cross_origin_resource_policy(
    res: &mut HttpResponse,
    cross_origin_resource_policy: &CrossOriginResourcePolicy,
) {
    let header_value = match cross_origin_resource_policy {
        CrossOriginResourcePolicy::SameOrigin => "same-origin",
        CrossOriginResourcePolicy::SameSite => "same-site",
        CrossOriginResourcePolicy::CrossOrigin => "cross-origin",
    };

    res.headers
        .insert("Cross-Origin-Resource-Policy", header_value.to_string());
}

fn set_cross_origin_embedder_policy(
    res: &mut HttpResponse,
    cross_origin_embedder_policy: &CrossOriginEmbedderPolicy,
) {
    let header_value = match cross_origin_embedder_policy {
        CrossOriginEmbedderPolicy::RequireCorp => "require-corp",
        CrossOriginEmbedderPolicy::UnsafeNone => "unsafe-none",
    };

    res.headers
        .insert("Cross-Origin-Embedder-Policy", header_value.to_string());
}

fn set_origin_agent_cluster(res: &mut HttpResponse, origin_agent_cluster: &OriginAgentCluster) {
    if !origin_agent_cluster.enabled {
        return;
    }

    res.headers.insert("Origin-Agent-Cluster", "?1");
}

fn set_cross_domain_policy(res: &mut HttpResponse, cross_domain_policy: &CrossDomainPolicy) {
    if !cross_domain_policy.enabled {
        return;
    }

    res.headers.insert(
        "X-Permitted-Cross-Domain-Policies",
        cross_domain_policy.policy.clone(),
    ); // Fixed header name
}
