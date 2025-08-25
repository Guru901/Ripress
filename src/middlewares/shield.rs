#![warn(missing_docs)]
use std::collections::HashMap;

use crate::{context::HttpResponse, req::HttpRequest, types::FutMiddleware};

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

/// Main configuration struct for the Shield middleware
///
/// This struct contains configuration for all security headers that can be set by the Shield middleware.
/// Each field represents a different security feature and can be individually configured or disabled.
/// All security features use secure defaults when using `ShieldConfig::default()`.
///
/// ## Field Overview
///
/// * `content_security_policy` - Controls CSP header generation and directives
/// * `hsts` - Configures HTTP Strict Transport Security behavior  
/// * `frameguard` - Controls X-Frame-Options header for clickjacking protection
/// * `no_sniff` - Enables X-Content-Type-Options: nosniff header
/// * `xss_filter` - Configures X-XSS-Protection header (legacy)
/// * `referrer_policy` - Controls Referrer-Policy header for privacy
/// * `dns_prefetch_control` - Manages X-DNS-Prefetch-Control header
/// * `ie_no_open` - Sets X-Download-Options: noopen for IE protection
/// * `hide_powered_by` - Removes X-Powered-By header to hide server info
/// * `permissions_policy` - Configures Permissions-Policy for browser features
/// * `cross_origin_opener_policy` - Sets Cross-Origin-Opener-Policy header
/// * `cross_origin_resource_policy` - Sets Cross-Origin-Resource-Policy header
/// * `cross_origin_embedder_policy` - Sets Cross-Origin-Embedder-Policy header
/// * `origin_agent_cluster` - Controls Origin-Agent-Cluster header
/// * `cross_domain_policy` - Sets X-Permitted-Cross-Domain-Policies header
#[derive(Clone)]
pub struct ShieldConfig {
    /// Content Security Policy configuration
    pub content_security_policy: ContentSecurityPolicy,
    /// HTTP Strict Transport Security configuration
    pub hsts: Hsts,
    /// X-Frame-Options configuration for clickjacking protection
    pub frameguard: Frameguard,
    /// X-Content-Type-Options configuration
    pub no_sniff: NoSniff,
    /// X-XSS-Protection configuration (deprecated but included for legacy support)
    pub xss_filter: XssFilter,
    /// Referrer-Policy configuration
    pub referrer_policy: ReferrerPolicy,
    /// X-DNS-Prefetch-Control configuration
    pub dns_prefetch_control: DnsPrefetchControl,
    /// X-Download-Options configuration for Internet Explorer
    pub ie_no_open: IENoOpen,
    /// X-Powered-By header removal configuration
    pub hide_powered_by: HidePoweredBy,
    /// Permissions-Policy configuration for browser features
    pub permissions_policy: PermissionsPolicy,
    /// Cross-Origin-Opener-Policy configuration
    pub cross_origin_opener_policy: CrossOriginOpenerPolicy,
    /// Cross-Origin-Resource-Policy configuration
    pub cross_origin_resource_policy: CrossOriginResourcePolicy,
    /// Cross-Origin-Embedder-Policy configuration
    pub cross_origin_embedder_policy: CrossOriginEmbedderPolicy,
    /// Origin-Agent-Cluster header configuration
    pub origin_agent_cluster: OriginAgentCluster,
    /// X-Permitted-Cross-Domain-Policies configuration
    pub cross_domain_policy: CrossDomainPolicy,
}

/// HTTP Strict Transport Security (HSTS) configuration
///
/// HSTS forces browsers to use HTTPS connections and prevents protocol downgrade attacks.
/// Once a browser receives an HSTS header, it will refuse to connect to the server over HTTP
/// for the specified duration, providing protection against man-in-the-middle attacks.
///
/// ## Security Benefits
///
/// * **Prevents protocol downgrade attacks** - Blocks HTTP connections after first HTTPS visit
/// * **Mitigates SSL stripping** - Protects against attacks that try to downgrade to HTTP
/// * **Improves user privacy** - Ensures all connections are encrypted
/// * **Reduces attack surface** - Eliminates HTTP as a potential attack vector
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the HSTS header (default: true)
/// * `max_age` - How long browsers should enforce HTTPS in seconds (default: 1 year)
/// * `include_subdomains` - Whether HSTS applies to all subdomains (default: true)
/// * `preload` - Whether to indicate the site should be included in browser preload lists (default: false)
///
/// ## Important Notes
///
/// * **HTTPS required** - HSTS headers are ignored when served over HTTP
/// * **Long-term commitment** - Setting a long max_age creates a long-term commitment to HTTPS
/// * **Preload considerations** - Enabling preload requires manual submission to browser vendors
/// * **Subdomain impact** - `include_subdomains` affects all current and future subdomains
#[derive(Clone)]
pub struct Hsts {
    /// Whether to enable HSTS header
    pub enabled: bool,
    /// Maximum age in seconds for HSTS enforcement (default: 1 year)
    pub max_age: u64,
    /// Whether to include subdomains in HSTS enforcement
    pub include_subdomains: bool,
    /// Whether to indicate preload list eligibility
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

/// X-Frame-Options (Frameguard) configuration  
///
/// The X-Frame-Options header prevents clickjacking attacks by controlling whether
/// the current page can be embedded within frames, iframes, embeds, or objects on other websites.
/// This header is crucial for preventing UI redressing attacks where malicious sites
/// overlay transparent frames to trick users into clicking on hidden elements.
///
/// ## Protection Modes
///
/// * **DENY** - Never allow the page to be framed (most secure, default)
/// * **SAMEORIGIN** - Only allow framing from the same origin
/// * **ALLOW-FROM** - Only allow framing from a specific domain (requires domain parameter)
///
/// ## Security Considerations
///
/// * **DENY is most secure** - Provides complete protection against clickjacking
/// * **SAMEORIGIN for internal frames** - Use when you need to frame your own content
/// * **ALLOW-FROM for partners** - Use only for trusted partner domains
/// * **Modern alternative** - Consider using CSP frame-ancestors directive instead
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the X-Frame-Options header (default: true)
/// * `action` - The framing policy: "deny", "sameorigin", or "allow-from" (default: "deny")
/// * `domain` - Specific domain for "allow-from" action (required when action is "allow-from")
#[derive(Clone)]
pub struct Frameguard {
    /// Whether to enable X-Frame-Options header
    pub enabled: bool,
    /// Framing action: "deny", "sameorigin", or "allow-from"
    pub action: String,
    /// Domain for allow-from action (required if action is "allow-from")
    pub domain: Option<String>,
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

/// X-Content-Type-Options configuration
///
/// The X-Content-Type-Options header prevents MIME type sniffing attacks by instructing
/// browsers to strictly follow the Content-Type header rather than trying to guess
/// the content type based on file content. This prevents malicious files from being
/// executed when they're disguised as safe content types.
///
/// ## Security Benefits
///
/// * **Prevents MIME confusion attacks** - Stops browsers from executing malicious content
/// * **Enforces Content-Type headers** - Ensures proper content type handling
/// * **Reduces attack surface** - Eliminates content sniffing as an attack vector
/// * **Simple and effective** - Single value provides comprehensive protection
///
/// ## How It Works
///
/// When `nosniff` is set, browsers will:
/// * Refuse to execute scripts if Content-Type is not a JavaScript MIME type
/// * Refuse to apply stylesheets if Content-Type is not `text/css`
/// * Not try to guess content types for other resources
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the X-Content-Type-Options: nosniff header (default: true)
#[derive(Clone)]
pub struct NoSniff {
    /// Whether to enable X-Content-Type-Options: nosniff header
    pub enabled: bool,
}

impl Default for NoSniff {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// X-XSS-Protection configuration
///
/// The X-XSS-Protection header enables and configures the browser's built-in XSS filter.
/// **Note: This header is deprecated in modern browsers** as Content Security Policy
/// provides superior XSS protection. However, it's included for legacy browser support.
///
/// ## Deprecation Notice
///
/// Modern browsers have removed or disabled XSS filtering due to:
/// * **Security vulnerabilities** - The filter itself could be exploited
/// * **False positives** - Legitimate content was sometimes blocked
/// * **Better alternatives** - CSP provides more robust XSS protection
///
/// ## Legacy Support
///
/// This header is maintained for:
/// * **Older browsers** - Browsers that don't support modern CSP features
/// * **Defense in depth** - Additional layer of protection alongside CSP
/// * **Compatibility** - Applications that haven't migrated to CSP-only protection
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the X-XSS-Protection header (default: true)
/// * `mode` - Protection mode: "block" or "filter" (default: "block")
/// * `report_uri` - Optional URI for violation reporting
///
/// ## Recommendation
///
/// Use Content Security Policy instead of relying on X-XSS-Protection for new applications.
/// This header should be considered a legacy compatibility feature.
#[derive(Clone)]
pub struct XssFilter {
    /// Whether to enable X-XSS-Protection header
    pub enabled: bool,
    /// XSS protection mode: "block" or "filter"
    pub mode: String,
    /// Optional URI for XSS violation reporting
    pub report_uri: Option<String>,
}

impl Default for XssFilter {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "0".to_string(),
            report_uri: None,
        }
    }
}

/// Referrer-Policy configuration
///
/// The Referrer-Policy header controls how much referrer information is included
/// when navigating from your site to other sites. This helps protect user privacy
/// and prevents sensitive information in URLs from being leaked to third parties.
///
/// ## Privacy Benefits
///
/// * **Protects sensitive URLs** - Prevents leaking URLs containing tokens or personal data
/// * **Reduces tracking** - Limits the ability of sites to track user navigation patterns
/// * **Improves user privacy** - Gives users more control over their browsing information
/// * **Prevents information disclosure** - Stops accidental leaking of internal URLs
///
/// ## Policy Options
///
/// * **no-referrer** - Never send referrer information
/// * **no-referrer-when-downgrade** - Send referrer for HTTPS→HTTPS, not HTTPS→HTTP (default)
/// * **origin** - Only send the origin, not the full URL
/// * **origin-when-cross-origin** - Send full URL for same-origin, origin only for cross-origin
/// * **same-origin** - Only send referrer for same-origin requests
/// * **strict-origin** - Send origin only, and only for HTTPS→HTTPS
/// * **strict-origin-when-cross-origin** - Full URL same-origin, origin cross-origin, respect HTTPS
/// * **unsafe-url** - Always send full referrer (not recommended)
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the Referrer-Policy header (default: true)
/// * `policy` - The referrer policy to use (default: "no-referrer-when-downgrade")
#[derive(Clone)]
pub struct ReferrerPolicy {
    /// Whether to enable Referrer-Policy header
    pub enabled: bool,
    /// The referrer policy to apply
    pub policy: String,
}

impl Default for ReferrerPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            policy: "strict-origin-when-cross-origin".to_string(),
        }
    }
}

/// X-DNS-Prefetch-Control configuration
///
/// The X-DNS-Prefetch-Control header controls DNS prefetching behavior in browsers.
/// DNS prefetching allows browsers to resolve domain names before they're actually needed,
/// improving perceived performance but potentially impacting user privacy.
///
/// ## Privacy vs Performance Trade-off
///
/// **Allowing DNS prefetching:**
/// * **Performance benefit** - Faster loading of external resources
/// * **Privacy cost** - DNS queries reveal user browsing patterns
/// * **Bandwidth usage** - Additional DNS queries consume bandwidth
///
/// **Disabling DNS prefetching:**
/// * **Privacy benefit** - Reduces DNS query leakage
/// * **Performance cost** - Slower loading of external resources
/// * **Bandwidth saving** - Fewer unnecessary DNS queries
///
/// ## Default Behavior
///
/// By default, this middleware **disables** DNS prefetching (`allow: false`) to prioritize
/// user privacy over performance. This is appropriate for most privacy-conscious applications.
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the X-DNS-Prefetch-Control header (default: true)
/// * `allow` - Whether to allow DNS prefetching (default: false for privacy)
#[derive(Clone)]
pub struct DnsPrefetchControl {
    /// Whether to enable X-DNS-Prefetch-Control header
    pub enabled: bool,
    /// Whether to allow DNS prefetching (false = more private, true = faster)
    pub allow: bool,
}

impl Default for DnsPrefetchControl {
    fn default() -> Self {
        Self {
            enabled: true,
            allow: false, // Prioritize privacy over performance
        }
    }
}

/// X-Download-Options configuration for Internet Explorer
///
/// The X-Download-Options header is specific to Internet Explorer and prevents IE
/// from executing downloaded files in the context of your site. When set to "noopen",
/// IE will not show the "Open" button in the download dialog, forcing users to save
/// files to disk before opening them.
///
/// ## Security Benefits
///
/// * **Prevents execution in site context** - Downloaded files can't execute with site privileges
/// * **Reduces malware risk** - Forces users to explicitly save and open files
/// * **Protects against drive-by downloads** - Prevents automatic execution of malicious files
/// * **Legacy IE protection** - Specifically addresses IE security vulnerabilities
///
/// ## Modern Relevance
///
/// This header is primarily relevant for:
/// * **Legacy IE support** - Organizations still supporting Internet Explorer
/// * **Comprehensive security** - Defense-in-depth strategies
/// * **File download applications** - Sites that frequently serve downloadable files
/// * **Minimal overhead** - No performance cost for modern browsers (ignored)
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the X-Download-Options: noopen header (default: true)
///
/// ## Note
///
/// Modern browsers ignore this header, but including it provides protection for legacy IE users
/// with minimal overhead.
#[derive(Clone)]
pub struct IENoOpen {
    /// Whether to enable X-Download-Options: noopen header
    pub enabled: bool,
}

impl Default for IENoOpen {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// X-Powered-By header removal configuration
///
/// The HidePoweredBy feature removes the X-Powered-By header from HTTP responses.
/// This header typically reveals information about the server technology stack,
/// which can be useful for attackers performing reconnaissance.
///
/// ## Security Through Obscurity
///
/// While not a primary security measure, hiding server information:
/// * **Reduces attack surface discovery** - Attackers have less information about your stack
/// * **Prevents targeted attacks** - Makes it harder to identify specific vulnerabilities
/// * **Improves operational security** - Reduces information disclosure
/// * **Industry best practice** - Commonly recommended security hardening step
///
/// ## Information Disclosure Prevention
///
/// Common X-Powered-By headers reveal:
/// * **Server technology** - "Express", "ASP.NET", "PHP", etc.
/// * **Version information** - Specific versions of frameworks
/// * **Technology stack** - Programming languages and frameworks used
///
/// ## Configuration Options
///
/// * `enabled` - Whether to remove X-Powered-By headers (default: true)
///
/// ## Implementation Note
///
/// When enabled, this feature removes any existing X-Powered-By header that might be
/// set by the application framework or server. The removal happens during response
/// processing, ensuring no powered-by information is leaked.
#[derive(Clone)]
pub struct HidePoweredBy {
    /// Whether to remove X-Powered-By headers from responses
    pub enabled: bool,
}

impl Default for HidePoweredBy {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Permissions-Policy configuration
///
/// The Permissions-Policy header (formerly Feature-Policy) controls which browser features
/// and APIs can be used in the current document and any embedded frames. This provides
/// fine-grained control over powerful browser capabilities that could be abused if
/// accessed without permission.
///
/// ## Controllable Features
///
/// Common browser features that can be controlled:
/// * **camera** - Access to camera devices
/// * **microphone** - Access to microphone devices  
/// * **geolocation** - Access to location information
/// * **payment** - Access to Payment Request API
/// * **usb** - Access to USB devices
/// * **bluetooth** - Access to Bluetooth devices
/// * **accelerometer** - Access to device motion sensors
/// * **gyroscope** - Access to device orientation sensors
/// * **magnetometer** - Access to magnetometer sensors
/// * **fullscreen** - Ability to enter fullscreen mode
/// * **picture-in-picture** - Picture-in-picture video capability
///
/// ## Allowlist Values
///
/// For each feature, you can specify an allowlist:
/// * **Empty list** `()` - Completely disable the feature
/// * **"self"** - Allow only for the current origin
/// * **Specific origins** - Allow for listed origins only
/// * **"\*"** - Allow for all origins (not recommended)
///
/// ## Default Configuration
///
/// The default configuration disables high-risk features:
/// * **camera** - Disabled (empty allowlist)
/// * **microphone** - Disabled (empty allowlist)
/// * **geolocation** - Allowed for same origin only
/// * **payment** - Disabled (empty allowlist)
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the Permissions-Policy header (default: true)
/// * `features` - HashMap of feature names to allowlist vectors
///
/// ## Security Benefits
///
/// * **Prevents feature abuse** - Malicious scripts can't access dangerous APIs
/// * **Reduces attack surface** - Disables unnecessary browser capabilities
/// * **Protects user privacy** - Controls access to sensitive device features
/// * **Granular control** - Different policies for different features
#[derive(Clone)]
pub struct PermissionsPolicy {
    /// Whether to enable Permissions-Policy header
    pub enabled: bool,
    /// Map of feature names to allowlist origins
    pub features: HashMap<String, Vec<String>>,
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        let mut features = HashMap::new();
        features.insert("camera".to_string(), vec![]); // Disabled
        features.insert("microphone".to_string(), vec![]); // Disabled
        features.insert("geolocation".to_string(), vec!["self".to_string()]); // Same origin only
        features.insert("payment".to_string(), vec![]); // Disabled

        Self {
            enabled: true,
            features,
        }
    }
}

/// Cross-Origin-Opener-Policy configuration
///
/// The Cross-Origin-Opener-Policy (COOP) header allows you to ensure that a top-level
/// document does not share a browsing context group with cross-origin documents.
/// This provides protection against cross-origin attacks that rely on having a
/// reference to your window object.
///
/// ## Security Benefits
///
/// * **Prevents window object access** - Cross-origin pages can't access your window
/// * **Protects against Spectre** - Helps isolate sensitive content in separate processes
/// * **Reduces attack surface** - Limits cross-origin interactions
/// * **Enables advanced security features** - Required for some security-sensitive APIs
///
/// ## Policy Options
///
/// * **SameOrigin** - Isolate from cross-origin documents (most secure, default)
/// * **SameOriginAllowPopups** - Isolate but allow popups to maintain references
/// * **UnsafeNone** - No isolation (least secure, legacy compatibility)
///
/// ## Compatibility Considerations
///
/// * **Breaking changes** - May break legitimate cross-origin interactions
/// * **Popup compatibility** - SameOrigin policy may break popup functionality
/// * **iframe interactions** - May affect cross-origin iframe communication
/// * **Third-party integrations** - May require adjustment for payment processors, etc.
///
/// ## Default Configuration
///
/// The default is `SameOrigin` for maximum security. Consider `SameOriginAllowPopups`
/// if your application uses popups for authentication or payments.
#[derive(Clone)]
pub enum CrossOriginOpenerPolicy {
    /// Isolate from all cross-origin documents (most secure)
    SameOrigin,
    /// Isolate but allow popups to maintain references
    SameOriginAllowPopups,
    /// No isolation (legacy compatibility, least secure)
    UnsafeNone,
}

impl Default for CrossOriginOpenerPolicy {
    fn default() -> Self {
        Self::SameOrigin // Most secure option
    }
}

/// Cross-Origin-Resource-Policy configuration
///
/// The Cross-Origin-Resource-Policy (CORP) header allows you to control which
/// websites can embed your resources (images, scripts, stylesheets, etc.) as subresources.
/// This provides protection against side-channel attacks like Spectre by controlling
/// cross-origin resource access at the network level.
///
/// ## Security Benefits
///
/// * **Prevents resource inclusion attacks** - Controls which sites can embed your resources
/// * **Protects against side-channel attacks** - Helps mitigate Spectre-class vulnerabilities
/// * **Reduces data leakage** - Prevents unauthorized cross-origin resource access
/// * **Complements CORS** - Provides additional layer beyond CORS policy
///
/// ## Policy Options
///
/// * **SameOrigin** - Only allow same-origin resource inclusion (most secure, default)
/// * **SameSite** - Allow same-site resource inclusion (includes subdomains)
/// * **CrossOrigin** - Allow cross-origin resource inclusion (least secure)
///
/// ## Use Cases
///
/// * **SameOrigin** - Internal applications, admin panels, sensitive content
/// * **SameSite** - Multi-subdomain applications with shared resources  
/// * **CrossOrigin** - Public APIs, CDN resources, embeddable widgets
///
/// ## Compatibility Notes
///
/// * **May break CDN usage** - SameOrigin policy blocks legitimate cross-origin embedding
/// * **Image/media sharing** - Consider CrossOrigin for publicly shareable media
/// * **API endpoints** - APIs typically need CrossOrigin policy
/// * **Widget/embed services** - Embeddable content requires CrossOrigin
#[derive(Clone)]
pub enum CrossOriginResourcePolicy {
    /// Only allow same-origin resource inclusion (most secure)
    SameOrigin,
    /// Allow same-site resource inclusion (includes subdomains)
    SameSite,
    /// Allow cross-origin resource inclusion (required for public resources)
    CrossOrigin,
}

impl Default for CrossOriginResourcePolicy {
    fn default() -> Self {
        Self::SameOrigin // Most secure and simple default
    }
}

/// Content Security Policy configuration
///
/// Content Security Policy (CSP) is the most powerful security header available,
/// providing comprehensive protection against XSS attacks, data injection, and
/// other code injection vulnerabilities. CSP works by defining a whitelist of
/// trusted sources for various types of content.
///
/// ## How CSP Works
///
/// CSP uses directives to control resource loading:
/// * **default-src** - Fallback for other directives
/// * **script-src** - Controls JavaScript execution
/// * **style-src** - Controls CSS loading and inline styles
/// * **img-src** - Controls image loading
/// * **font-src** - Controls font loading
/// * **connect-src** - Controls AJAX, WebSocket, EventSource connections
/// * **media-src** - Controls audio and video loading
/// * **object-src** - Controls plugins (Flash, Java, etc.)
/// * **frame-src** - Controls iframe sources
/// * **worker-src** - Controls web workers and service workers
/// * **manifest-src** - Controls web app manifest loading
///
/// ## Common Source Values
///
/// * **'self'** - Same origin as the document
/// * **'unsafe-inline'** - Allow inline scripts/styles (not recommended)
/// * **'unsafe-eval'** - Allow eval() and similar (not recommended)  
/// * **'none'** - Block all sources
/// * **https:** - Allow any HTTPS source
/// * **data:** - Allow data: URIs
/// * **blob:** - Allow blob: URIs
/// * **Specific domains** - https://example.com, https://cdn.jsdelivr.net
/// * **'nonce-<value>'** - Allow resources with matching nonce attribute
/// * **'sha256-<hash>'** - Allow resources matching specific hash
///
/// ## Security Benefits
///
/// * **XSS prevention** - Primary defense against cross-site scripting
/// * **Data injection protection** - Prevents malicious resource loading
/// * **Clickjacking mitigation** - frame-ancestors directive controls embedding
/// * **Mixed content prevention** - Enforces HTTPS-only resource loading
/// * **Exfiltration prevention** - Controls where data can be sent
///
/// ## Default Configuration
///
/// The default CSP is restrictive but functional for most web applications:
/// * **default-src 'self'** - Only allow same-origin resources by default
/// * **script-src 'self'** - Only allow same-origin scripts
/// * **style-src 'self' 'unsafe-inline'** - Same-origin styles plus inline (necessary for many apps)
/// * **img-src 'self' data: https:** - Same-origin images, data URIs, and HTTPS images
/// * **object-src 'none'** - Block all plugins and embedded objects
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include CSP header (default: true)
/// * `directives` - HashMap of directive names to source values
/// * `report_only` - Whether to use report-only mode for testing (default: false)
///
/// ## Testing and Deployment
///
/// 1. **Start with report-only mode** - Set `report_only: true` to test without breaking functionality
/// 2. **Monitor violations** - Set up violation reporting to catch policy issues
/// 3. **Gradually tighten policy** - Remove 'unsafe-inline' and 'unsafe-eval' when possible
/// 4. **Use nonces or hashes** - For inline scripts/styles that can't be moved to external files
/// 5. **Deploy enforcement** - Set `report_only: false` when confident in policy
#[derive(Clone)]
pub struct ContentSecurityPolicy {
    /// Whether to enable Content Security Policy header
    pub enabled: bool,
    /// Map of CSP directive names to their values
    pub directives: HashMap<String, String>,
    /// Whether to use Content-Security-Policy-Report-Only header instead of enforcing
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

/// Cross-Origin-Embedder-Policy configuration
///
/// The Cross-Origin-Embedder-Policy (COEP) header allows you to prevent a document
/// from loading any cross-origin resources that don't explicitly grant permission.
/// This enables powerful browser features like SharedArrayBuffer by ensuring that
/// all cross-origin resources are loaded with explicit consent.
///
/// ## Security and Performance Benefits
///
/// * **Enables SharedArrayBuffer** - Required for high-performance web applications
/// * **Prevents data leakage** - Ensures explicit consent for cross-origin resources
/// * **Improves process isolation** - Helps browser implement stronger security boundaries
/// * **Enables precise timing APIs** - Required for performance.now() with high resolution
///
/// ## Policy Options
///
/// * **RequireCorp** - Require Cross-Origin-Resource-Policy header on all cross-origin resources
/// * **UnsafeNone** - No requirements (default, maintains compatibility)
///
/// ## Compatibility Impact
///
/// **RequireCorp policy may break:**
/// * **Third-party images** - Without proper CORP headers
/// * **External stylesheets** - From CDNs without CORP headers
/// * **Embedded content** - Widgets, ads, social media embeds
/// * **API responses** - Cross-origin API calls without CORP
///
/// ## Migration Strategy
///
/// 1. **Audit cross-origin resources** - Identify all external resources
/// 2. **Add CORP headers** - Ensure external resources have proper headers
/// 3. **Test thoroughly** - Verify all functionality works with COEP enabled  
/// 4. **Enable incrementally** - Start with non-critical pages
///
/// ## Default Configuration
///
/// Default is `UnsafeNone` to maintain compatibility. Only enable `RequireCorp`
/// if you need SharedArrayBuffer or high-resolution timing APIs and have ensured
/// all cross-origin resources have appropriate CORP headers.
#[derive(Clone)]
pub enum CrossOriginEmbedderPolicy {
    /// Require CORP header on all cross-origin resources
    RequireCorp,
    /// No requirements (maintains compatibility)
    UnsafeNone,
}

impl Default for CrossOriginEmbedderPolicy {
    fn default() -> Self {
        Self::UnsafeNone // Maintain compatibility by default
    }
}

/// Origin-Agent-Cluster configuration
///
/// The Origin-Agent-Cluster header requests that the browser place the current
/// document in an origin-keyed agent cluster. This provides additional process
/// isolation by ensuring that documents from different origins are placed in
/// separate agent clusters, improving security and enabling certain performance
/// optimizations.
///
/// ## Security Benefits
///
/// * **Process isolation** - Better separation between different origins
/// * **Reduces side-channel attacks** - Harder for malicious sites to infer information
/// * **Improves memory safety** - Separate memory spaces for different origins
/// * **Future-proofs security** - Enables stronger isolation features as browsers evolve
///
/// ## Performance Implications
///
/// * **Memory overhead** - Each agent cluster requires separate resources
/// * **Slower cross-origin communication** - May impact legitimate cross-origin interactions
/// * **Browser optimization** - May enable better browser optimizations for single-origin content
///
/// ## When to Enable
///
/// Enable Origin-Agent-Cluster when:
/// * **Security is paramount** - High-value applications requiring maximum isolation
/// * **Single-origin applications** - Apps that don't heavily interact with other origins
/// * **Memory is not constrained** - Systems that can handle additional memory overhead
/// * **Modern browser targets** - Applications targeting browsers with good OAC support
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include the Origin-Agent-Cluster: ?1 header (default: true)
///
/// ## Browser Support
///
/// This is a relatively new header with growing browser support. It's safe to enable
/// as unsupporting browsers will simply ignore it.
#[derive(Clone)]
pub struct OriginAgentCluster {
    /// Whether to enable Origin-Agent-Cluster: ?1 header
    pub enabled: bool,
}

impl Default for OriginAgentCluster {
    fn default() -> Self {
        Self { enabled: true } // Enable by default for better security
    }
}

/// Cross-Domain Policy configuration
///
/// The X-Permitted-Cross-Domain-Policies header controls whether Adobe Flash Player
/// and Adobe Reader can load content from your domain via cross-domain policy files.
/// While Flash is largely obsolete, this header is still relevant for legacy content
/// and provides defense-in-depth security.
///
/// ## Legacy Technology Context
///
/// This header was primarily important for:
/// * **Adobe Flash Player** - Now deprecated and removed from most browsers
/// * **Adobe Reader** - Legacy PDF handling in browsers
/// * **Silverlight** - Microsoft's Flash alternative, also deprecated
///
/// ## Policy Options
///
/// * **none** - Prohibit all cross-domain policy files (most secure, default)
/// * **master-only** - Only allow master cross-domain policy file
/// * **by-content-type** - Allow cross-domain policy files served with appropriate content type
/// * **by-ftp-filename** - Allow files named crossdomain.xml via FTP
/// * **all** - Allow all cross-domain policy files (least secure)
///
/// ## Security Benefits
///
/// * **Prevents Flash-based attacks** - Even for legacy Flash content
/// * **Reduces attack surface** - Blocks potential vector for old vulnerabilities
/// * **Defense in depth** - Additional security layer with minimal overhead
/// * **Compliance** - May be required for certain security standards
///
/// ## Default Configuration
///
/// Default policy is "none" which provides maximum security by completely
/// disabling cross-domain policy files. This is appropriate for modern
/// web applications that don't use legacy Flash/Silverlight content.
///
/// ## Configuration Options
///
/// * `enabled` - Whether to include X-Permitted-Cross-Domain-Policies header (default: true)
/// * `policy` - The cross-domain policy: "none", "master-only", etc. (default: "none")
#[derive(Clone)]
pub struct CrossDomainPolicy {
    /// Whether to enable X-Permitted-Cross-Domain-Policies header
    pub enabled: bool,
    /// The cross-domain policy to enforce
    pub policy: String,
}

impl Default for CrossDomainPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            policy: "none".to_string(), // Most secure: prohibit all cross-domain policies
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
    move |req, mut res| {
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
