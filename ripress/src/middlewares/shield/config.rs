use std::collections::HashMap;

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
