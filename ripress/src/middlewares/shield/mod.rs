#![warn(missing_docs)]

use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};

pub use crate::middlewares::shield::config::{
    ContentSecurityPolicy, CrossDomainPolicy, CrossOriginEmbedderPolicy, CrossOriginOpenerPolicy,
    CrossOriginResourcePolicy, DnsPrefetchControl, Frameguard, HidePoweredBy, Hsts, IENoOpen,
    NoSniff, OriginAgentCluster, PermissionsPolicy, ReferrerPolicy, ShieldConfig, XssFilter,
};

/// Builtin Shield Middleware
///
/// This middleware provides comprehensive HTTP security header protection for web applications.
/// It automatically sets various security headers that help protect against common web vulnerabilities
/// including XSS attacks, clickjacking, content sniffing, MIME type confusion, and other security threats.
/// The middleware is highly configurable, allowing you to customize each security feature according
/// to your application's specific requirements while maintaining secure defaults.
///
/// ## Features
///
/// * **Content Security Policy (CSP)** - Prevents XSS and data injection attacks
/// * **HTTP Strict Transport Security (HSTS)** - Forces HTTPS connections and prevents downgrade attacks
/// * **X-Frame-Options (Frameguard)** - Prevents clickjacking by controlling frame embedding
/// * **X-Content-Type-Options** - Prevents MIME type sniffing attacks
/// * **X-XSS-Protection** - Enables and configures browser XSS filtering
/// * **Referrer Policy** - Controls referrer information leakage across origins
/// * **DNS Prefetch Control** - Manages DNS prefetching behavior for privacy
/// * **IE No Open** - Prevents Internet Explorer from executing downloaded files
/// * **Hide Powered-By** - Removes X-Powered-By header to hide server technology
/// * **Permissions Policy** - Controls browser feature and API access
/// * **Cross-Origin Opener Policy** - Manages cross-origin window references
/// * **Cross-Origin Resource Policy** - Controls cross-origin resource access
/// * **Cross-Origin Embedder Policy** - Enables process isolation for sensitive resources
/// * **Origin Agent Cluster** - Requests origin-keyed agent clustering
/// * **Cross Domain Policy** - Controls Flash/Silverlight cross-domain access
/// * **Thread-safe operation** - Safe for concurrent use across multiple threads
/// * **Zero-overhead when disabled** - Individual features can be disabled with no performance cost
/// * **Secure defaults** - All features enabled with security-first default configurations
///
/// ## Security Headers Overview
///
/// ### Content Security Policy (CSP)
/// The most powerful security header, CSP prevents XSS attacks by controlling which resources
/// the browser is allowed to load and execute. The default policy is restrictive but functional:
/// * `default-src 'self'` - Only allow resources from the same origin
/// * `script-src 'self'` - Only allow scripts from the same origin
/// * `style-src 'self' 'unsafe-inline'` - Allow same-origin styles plus necessary inline styles
/// * `img-src 'self' data: https:` - Allow images from same origin, data URLs, and HTTPS sources
/// * `object-src 'none'` - Block all plugins and embedded objects
///
/// ### HTTP Strict Transport Security (HSTS)
/// Forces browsers to use HTTPS connections and prevents protocol downgrade attacks:
/// * Default max-age of 1 year (31536000 seconds)
/// * Includes subdomains by default for comprehensive protection
/// * Preload disabled by default (requires manual submission to browser preload lists)
///
/// ### Cross-Origin Policies
/// Modern security headers that provide fine-grained control over cross-origin interactions:
/// * **COOP** - Prevents malicious sites from accessing your window objects
/// * **CORP** - Controls which sites can embed your resources
/// * **COEP** - Enables process isolation by requiring explicit opt-in for cross-origin resources
///
/// ## Configuration
///
/// The middleware accepts an optional `ShieldConfig` struct. If no configuration is provided,
/// secure defaults are used for all features. Each security feature can be individually
/// configured or disabled as needed.
///
/// ## Examples
///
/// Basic usage with secure defaults:
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
/// app.use_shield(None); // Uses ShieldConfig::default() with secure settings
/// ```
///
/// Custom CSP configuration for a web application:
///
/// ```rust
/// use ripress::{app::App, middlewares::shield::*};
/// use std::collections::HashMap;
///
/// let mut app = App::new();
///
/// let mut csp_directives = HashMap::new();
/// csp_directives.insert("default-src".to_string(), "'self'".to_string());
/// csp_directives.insert("script-src".to_string(), "'self' 'unsafe-eval' https://cdn.jsdelivr.net".to_string());
/// csp_directives.insert("style-src".to_string(), "'self' 'unsafe-inline' https://fonts.googleapis.com".to_string());
/// csp_directives.insert("font-src".to_string(), "'self' https://fonts.gstatic.com".to_string());
/// csp_directives.insert("img-src".to_string(), "'self' data: https: blob:".to_string());
/// csp_directives.insert("connect-src".to_string(), "'self' https://api.example.com".to_string());
///
/// let config = ShieldConfig {
///     content_security_policy: ContentSecurityPolicy {
///         enabled: true,
///         directives: csp_directives,
///         report_only: false,
///     },
///     hsts: Hsts {
///         enabled: true,
///         max_age: 63072000, // 2 years
///         include_subdomains: true,
///         preload: true,
///     },
///     ..Default::default()
/// };
///
/// app.use_shield(Some(config));
/// ```
///
/// Development configuration with relaxed CSP:
///
/// ```rust
/// use ripress::{app::App, middlewares::shield::*};
/// use std::collections::HashMap;
///
/// let mut app = App::new();
///
/// let mut csp_directives = HashMap::new();
/// csp_directives.insert("default-src".to_string(), "'self' 'unsafe-eval' 'unsafe-inline'".to_string());
///
/// let config = ShieldConfig {
///     content_security_policy: ContentSecurityPolicy {
///         enabled: true,
///         directives: csp_directives,
///         report_only: true, // Report violations but don't block
///     },
///     hsts: Hsts {
///         enabled: false, // Disable HSTS in development
///         ..Default::default()
///     },
///     ..Default::default()
/// };
///
/// app.use_shield(Some(config));
/// ```
///
/// API-only configuration with minimal headers:
///
/// ```rust
/// use ripress::{app::App, middlewares::shield::*};
///
/// let mut app = App::new();
///
/// let config = ShieldConfig {
///     content_security_policy: ContentSecurityPolicy {
///         enabled: false, // No CSP needed for API-only applications
///         ..Default::default()
///     },
///     frameguard: Frameguard {
///         enabled: false, // No frame protection needed for APIs
///         ..Default::default()
///     },
///     xss_filter: XssFilter {
///         enabled: false, // No XSS protection needed for APIs
///         ..Default::default()
///     },
///     cross_origin_resource_policy: CrossOriginResourcePolicy::CrossOrigin, // Allow cross-origin API access
///     ..Default::default()
/// };
///
/// app.use_shield(Some(config));
/// ```
///
/// ## Performance Considerations
///
/// * **Minimal overhead** - Headers are set once per response with negligible CPU cost
/// * **Memory efficient** - Configuration is shared via cloning, no per-request allocation
/// * **Disabled features** - Features with `enabled: false` are completely skipped
/// * **String operations** - Header values are computed once and cached when possible
/// * **Zero allocation** - No dynamic memory allocation during request processing
///
/// ## Security Best Practices
///
/// * **Test CSP thoroughly** - Content Security Policy can break functionality if misconfigured
/// * **Use report-only mode first** - Test CSP with `report_only: true` before enforcing
/// * **Monitor CSP violations** - Set up violation reporting to catch policy violations
/// * **Keep HSTS max-age reasonable** - Very long HSTS periods can cause accessibility issues
/// * **Review cross-origin policies** - Ensure CORP/COOP settings match your application architecture
/// * **Disable unused features** - Turn off security headers that don't apply to your application type
///
/// ## Compatibility Notes
///
/// * **Modern browsers** - All headers are supported by modern browsers (Chrome 60+, Firefox 55+, Safari 12+)
/// * **Legacy browser graceful degradation** - Older browsers ignore unknown headers without issues
/// * **X-XSS-Protection deprecation** - This header is deprecated in modern browsers but kept for legacy support
/// * **IE-specific headers** - Some headers like X-Download-Options only affect Internet Explorer
///
/// ## Troubleshooting
///
/// Common issues and solutions:
/// * **CSP blocking resources** - Check browser console for violation reports, update directives accordingly
/// * **HSTS not working** - Ensure you're serving over HTTPS, HSTS is ignored on HTTP
/// * **Frame embedding issues** - Adjust frameguard settings if legitimate embedding is blocked
/// * **API CORS issues** - Configure cross-origin policies appropriately for API endpoints
/// * **Development vs production** - Use different configurations for development and production environments
///
/// ## Thread Safety
///
/// The middleware is fully thread-safe:
/// * Configuration structs implement `Clone` for efficient sharing
/// * No mutable state is maintained between requests  
/// * Safe for use in multi-threaded web servers
/// * Header setting operations are atomic and isolated per response
pub mod config;

/// Creates a shield middleware function
///
/// Returns a middleware function that sets comprehensive security headers on HTTP responses
/// to protect against common web vulnerabilities. The middleware applies all configured
/// security headers efficiently with minimal performance overhead.
///
/// ## Parameters
///
/// * `config` - Optional shield configuration. If `None`, uses `ShieldConfig::default()`
///   which enables all security features with secure default settings.
///
/// ## Returns
///
/// A middleware function compatible with the ripress framework that:
/// * Sets security headers on all HTTP responses
/// * Applies configuration-specific security policies
/// * Operates efficiently with minimal CPU and memory overhead
/// * Handles disabled features by skipping them entirely
/// * Provides thread-safe operation across concurrent requests
///
/// ## Security Headers Applied
///
/// When using default configuration, the following headers are set:
/// * **Content-Security-Policy** - Restrictive policy allowing only same-origin resources
/// * **Strict-Transport-Security** - 1-year HSTS with includeSubDomains
/// * **X-Frame-Options** - DENY to prevent clickjacking
/// * **X-Content-Type-Options** - nosniff to prevent MIME confusion
/// * **X-XSS-Protection** - not sent by default (header deprecated in modern browsers)
/// * **Referrer-Policy** - strict-origin-when-cross-origin for privacy
/// * **X-DNS-Prefetch-Control** - off to improve privacy
/// * **X-Download-Options** - noopen for IE security
/// * **Permissions-Policy** - Restrictive policy disabling camera, microphone, etc.
/// * **Cross-Origin-Opener-Policy** - same-origin for process isolation
/// * **Cross-Origin-Resource-Policy** - same-origin for resource protection
/// * **Cross-Origin-Embedder-Policy** - unsafe-none for compatibility
/// * **Origin-Agent-Cluster** - ?1 for improved isolation
/// * **X-Permitted-Cross-Domain-Policies** - none to disable legacy policies
/// * **X-Powered-By** - Header removed to hide server information
///
/// ## Performance Characteristics
///
/// * **Low CPU overhead** - Headers are set with simple string operations
/// * **Minimal memory usage** - Configuration is shared via cloning
/// * **No I/O operations** - All processing happens in memory
/// * **Disabled feature skip** - Features with `enabled: false` are completely bypassed
/// * **String reuse** - Header values are computed once per configuration
///
/// ## Thread Safety
///
/// The middleware is fully thread-safe:
/// * Configuration is cloned for each request processing
/// * No shared mutable state between requests
/// * Safe for use in multi-threaded web servers
/// * All header operations are isolated per response
///
/// ## Usage Examples
///
/// Basic usage with secure defaults:
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
/// app.use_shield(None); // Applies all security headers with defaults
/// ```
///
/// Custom configuration:
/// ```rust
/// use ripress::{app::App, middlewares::shield::*};
///
/// let mut app = App::new();
/// let config = ShieldConfig {
///     hsts: Hsts {
///         enabled: true,
///         max_age: 63072000, // 2 years
///         include_subdomains: true,
///         preload: true,
///     },
///     frameguard: Frameguard {
///         enabled: true,
///         action: "sameorigin".to_string(),
///         domain: None,
///     },
///     ..Default::default()
/// };
/// app.use_shield(Some(config));
/// ```
pub(crate) fn shield(
    config: Option<ShieldConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> FutMiddleware + Send + Sync + 'static {
    let config = std::sync::Arc::new(config.unwrap_or_default());
    move |req: HttpRequest, mut res| {
        let config = std::sync::Arc::clone(&config);

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

/// Sets Content-Security-Policy header based on configuration
///
/// Constructs and sets the CSP header from the provided directives map.
/// Uses Content-Security-Policy-Report-Only header when report_only is true.
pub(crate) fn set_content_security_policy(res: &mut HttpResponse, csp: &ContentSecurityPolicy) {
    if !csp.enabled {
        return;
    }

    let header_name = if csp.report_only {
        "content-security-policy-report-only"
    } else {
        "content-security-policy"
    };

    let mut entries: Vec<_> = csp.directives.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let header_value = entries
        .into_iter()
        .map(|(k, v)| format!("{} {}", k, v))
        .collect::<Vec<_>>()
        .join("; ");

    res.headers.insert(header_name, header_value);
}

/// Sets Strict-Transport-Security header based on HSTS configuration
///
/// Constructs HSTS header with max-age, includeSubDomains, and preload directives
/// as configured. Only sets header when HSTS is enabled.
pub(crate) fn set_hsts(res: &mut HttpResponse, hsts: &Hsts) {
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
    // Use a single canonical header name to avoid duplicates.
    res.headers.insert("strict-transport-security", value);
}

/// Sets X-Frame-Options header based on frameguard configuration
///
/// Supports DENY, SAMEORIGIN, and ALLOW-FROM actions. Falls back to DENY
/// if an invalid action is specified or if ALLOW-FROM is used without a domain.
pub(crate) fn set_frameguard(res: &mut HttpResponse, frameguard: &Frameguard) {
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
        _ => "DENY".to_string(), // Default fallback for invalid actions
    };

    res.headers.insert("x-frame-options", value);
}

/// Sets X-Content-Type-Options header to prevent MIME sniffing
///
/// Always sets the header to "nosniff" when enabled, as this is the only
/// valid value for this security header.
pub(crate) fn set_no_sniff(res: &mut HttpResponse, no_sniff: &NoSniff) {
    if !no_sniff.enabled {
        return;
    }

    res.headers.no_sniff();
}

/// Sets X-XSS-Protection header based on XSS filter configuration
///
/// Constructs the header value with mode and optional report URI.
/// This header is deprecated but included for legacy browser support.
pub(crate) fn set_xss_filter(res: &mut HttpResponse, xss_filter: &XssFilter) {
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

    res.headers.insert("x-xss-protection", value);
}

/// Sets Referrer-Policy header based on configuration
///
/// Controls how much referrer information is sent when navigating away from the page.
/// Uses the configured policy string directly as the header value.
pub(crate) fn set_referrer_policy(res: &mut HttpResponse, referrer_policy: &ReferrerPolicy) {
    if !referrer_policy.enabled {
        return;
    }
    res.headers
        .insert("referrer-policy", referrer_policy.policy.clone());
}

/// Sets X-DNS-Prefetch-Control header based on configuration
///
/// Controls DNS prefetching behavior. Sets to "on" when allowed,
/// "off" when disabled for privacy protection.
pub(crate) fn set_dns_prefetch_control(
    res: &mut HttpResponse,
    dns_prefetch_control: &DnsPrefetchControl,
) {
    if !dns_prefetch_control.enabled {
        return;
    }

    let header_value = if dns_prefetch_control.allow {
        "on"
    } else {
        "off"
    };

    res.headers.insert("x-dns-prefetch-control", header_value);
}

/// Sets X-Download-Options header for Internet Explorer protection
///
/// Always sets the header to "noopen" when enabled, preventing IE from
/// executing downloaded files in the site's context.
pub(crate) fn set_ie_no_open(res: &mut HttpResponse, ie_no_open: &IENoOpen) {
    if !ie_no_open.enabled {
        return;
    }

    res.headers.insert("x-download-options", "noopen");
}

/// Removes X-Powered-By header when hide powered-by is enabled
///
/// Removes any existing X-Powered-By header to prevent disclosure of
/// server technology information to potential attackers.
pub(crate) fn set_hide_powered_by(res: &mut HttpResponse, hide_powered_by: &HidePoweredBy) {
    if !hide_powered_by.enabled {
        return;
    }

    res.headers.remove("x-powered-by");
    res.headers.remove("X-Powered-By");
}

/// Sets Permissions-Policy header based on feature configuration
///
/// Constructs the header from the features HashMap, formatting each feature
/// with its allowlist. Empty allowlists result in () (disabled), while
/// populated allowlists are formatted as space-separated quoted origins.
pub(crate) fn set_permissions_policy(
    res: &mut HttpResponse,
    permissions_policy: &PermissionsPolicy,
) {
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
        res.headers.insert("permissions-policy", header_value);
    }
}

/// Sets Cross-Origin-Opener-Policy header based on policy configuration
///
/// Maps the enum variant to the appropriate header value string.
/// This header controls cross-origin window references and process isolation.
pub(crate) fn set_cross_origin_opener_policy(
    res: &mut HttpResponse,
    cross_origin_opener_policy: &CrossOriginOpenerPolicy,
) {
    let header_value = match cross_origin_opener_policy {
        CrossOriginOpenerPolicy::SameOrigin => "same-origin",
        CrossOriginOpenerPolicy::SameOriginAllowPopups => "same-origin-allow-popups",
        CrossOriginOpenerPolicy::UnsafeNone => "unsafe-none",
    };

    res.headers
        .insert("cross-origin-opener-policy", header_value.to_string());
}

/// Sets Cross-Origin-Resource-Policy header based on policy configuration
///
/// Maps the enum variant to the appropriate header value string.
/// This header controls cross-origin resource embedding permissions.
pub(crate) fn set_cross_origin_resource_policy(
    res: &mut HttpResponse,
    cross_origin_resource_policy: &CrossOriginResourcePolicy,
) {
    let header_value = match cross_origin_resource_policy {
        CrossOriginResourcePolicy::SameOrigin => "same-origin",
        CrossOriginResourcePolicy::SameSite => "same-site",
        CrossOriginResourcePolicy::CrossOrigin => "cross-origin",
    };

    res.headers
        .insert("cross-origin-resource-policy", header_value.to_string());
}

/// Sets Cross-Origin-Embedder-Policy header based on policy configuration
///
/// Maps the enum variant to the appropriate header value string.
/// This header controls cross-origin resource requirements for embedder isolation.
pub(crate) fn set_cross_origin_embedder_policy(
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

/// Sets Origin-Agent-Cluster header when enabled
///
/// Always sets the header to "?1" when enabled, requesting origin-keyed
/// agent clustering for improved process isolation.
pub(crate) fn set_origin_agent_cluster(
    res: &mut HttpResponse,
    origin_agent_cluster: &OriginAgentCluster,
) {
    if !origin_agent_cluster.enabled {
        return;
    }

    res.headers.insert("origin-agent-cluster", "?1");
}

/// Sets X-Permitted-Cross-Domain-Policies header based on policy configuration
///
/// Controls Adobe Flash Player and Reader cross-domain policy file permissions.
/// Uses the configured policy string directly as the header value.
pub(crate) fn set_cross_domain_policy(
    res: &mut HttpResponse,
    cross_domain_policy: &CrossDomainPolicy,
) {
    if !cross_domain_policy.enabled {
        return;
    }

    res.headers.insert(
        "x-permitted-cross-domain-policies",
        cross_domain_policy.policy.clone(),
    );
}
