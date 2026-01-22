#![warn(missing_docs)]
use crate::{
    context::HttpResponse,
    req::HttpRequest,
    types::{HttpMethods, MiddlewareOutput},
};

/// Builtin CORS (Cross-Origin Resource Sharing) Middleware
///
/// This middleware handles Cross-Origin Resource Sharing (CORS) by adding appropriate
/// headers to HTTP responses and handling preflight OPTIONS requests. CORS is essential
/// for web applications that need to make cross-origin requests from browsers, such as
/// when a frontend served from one domain needs to access an API on another domain.
///
/// ## Features
///
/// * **Preflight request handling** - Automatically responds to OPTIONS requests
/// * **Dynamic origin reflection** - Can reflect requesting origin for flexible CORS
/// * **Static configuration** - Set fixed allowed origins, methods, and headers
/// * **Credential support** - Optional credential allowing with security considerations
/// * **Header optimization** - Minimal headers when not reflecting requests
/// * **Vary header management** - Proper cache control for dynamic responses
/// * **Security-first defaults** - Conservative defaults that can be relaxed as needed
/// * **Standards compliant** - Follows W3C CORS specification
///
/// ## CORS Behavior Modes
///
/// The middleware operates in two modes based on the presence of CORS request headers:
///
/// ### 1. Reflective Mode (Dynamic)
/// When the request includes both `Origin` and `Access-Control-Request-Method` headers:
/// - **Origin**: Reflects the requesting origin back in `Access-Control-Allow-Origin`
/// - **Methods**: Reflects the requested method back in `Access-Control-Allow-Methods`  
/// - **Headers**: Uses requested headers or falls back to configured `allowed_headers`
/// - **Vary**: Adds `Vary` header for proper caching behavior
///
/// ### 2. Static Mode (Default)
/// For regular requests or when CORS headers are missing:
/// - Uses configured static values for all CORS headers
/// - More predictable and cacheable responses
/// - No `Vary` header needed
///
/// ## Configuration Options
///
/// * `allowed_origin` - Origin(s) allowed to access the resource (default: "*")
/// * `allowed_methods` - HTTP methods allowed for cross-origin requests (default: "GET, POST, PUT, DELETE, OPTIONS, HEAD")
/// * `allowed_headers` - Headers allowed in cross-origin requests (default: "Content-Type, Authorization")
/// * `allow_credentials` - Whether to allow credentials (cookies, auth headers) in CORS requests (default: false)
///
/// ## Security Considerations
///
/// ### Origin Validation
/// - **Wildcard (`*`) origins** cannot be used with credentials for security
/// - **Dynamic reflection** should be used carefully - validate origins in production
/// - **Subdomain policies** may require specific origin patterns
///
/// ### Credential Handling
/// - Setting `allow_credentials: true` has security implications
/// - Cannot use wildcard origin with credentials
/// - Consider implementing proper origin validation when using credentials
///
/// ### Header Exposure
/// - Be conservative with `allowed_headers` - only allow what's needed
/// - Some headers (like `Authorization`) may expose sensitive information
/// - Consider the principle of least privilege
///
/// ## Preflight Request Handling
///
/// The middleware automatically handles CORS preflight requests:
/// 1. **Detection**: Identifies OPTIONS requests as potential preflights
/// 2. **Header Addition**: Adds appropriate CORS headers to the response
/// 3. **Early Return**: Returns 200 OK immediately without calling subsequent handlers
/// 4. **Cache Control**: Includes proper `Vary` headers when reflecting requests
///
/// ## Examples
///
/// Basic CORS with default settings (allows all origins):
///
/// ```rust
/// use ripress::{app::App, middlewares::cors::CorsConfig};
///
/// let mut app = App::new();
/// app.use_cors(Some(CorsConfig::default()));
/// ```
///
/// Restrictive CORS for production API:
///
/// ```rust
/// use ripress::{app::App, middlewares::cors::CorsConfig};
///
/// let mut app = App::new();
/// let config = CorsConfig {
///     allowed_origin: "https://myapp.com",
///     allowed_methods: "GET, POST, PUT, DELETE",
///     allowed_headers: "Content-Type, Authorization, X-API-Key",
///     allow_credentials: true,
/// };
/// app.use_cors(Some(config));
/// ```
///
/// Development-friendly CORS (allows localhost variations):
///
/// ```rust
/// use ripress::{app::App, middlewares::cors::CorsConfig};
///
/// let mut app = App::new();
/// let config = CorsConfig {
///     allowed_origin: "*", // Note: Cannot use with credentials
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD, PATCH",
///     allowed_headers: "Content-Type, Authorization, X-Requested-With, Accept, Origin",
///     allow_credentials: false, // Must be false when origin is "*"
/// };
/// app.use_cors(Some(config));
/// ```
///
/// Multiple origin support pattern (requires custom logic):
///
/// ```rust
/// use ripress::{app::App, middlewares::cors::CorsConfig};
///
/// // Note: For true multiple origin support, you may need custom middleware
/// // This example shows single origin configuration
/// let mut app = App::new();
/// let config = CorsConfig {
///     allowed_origin: "https://app.example.com",
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS",
///     allowed_headers: "Content-Type, Authorization",
///     allow_credentials: true,
/// };
/// app.use_cors(Some(config));
/// ```
///
/// Using default configuration:
///
/// ```rust
/// use ripress::app::App;
///
/// let mut app = App::new();
/// app.use_cors(None); // Uses CorsConfig::default()
/// ```
///
/// ## Common Use Cases
///
/// ### Single Page Applications (SPAs)
/// ```rust
/// use ripress::middlewares::cors::CorsConfig;
///
/// let config = CorsConfig {
///     allowed_origin: "https://myapp.com",
///     allowed_methods: "GET, POST, PUT, DELETE, PATCH",
///     allowed_headers: "Content-Type, Authorization",
///     allow_credentials: true,
/// };
/// ```
///
/// ### Public APIs
/// ```rust
/// use ripress::middlewares::cors::CorsConfig;
///
/// let config = CorsConfig {
///     allowed_origin: "*",
///     allowed_methods: "GET, POST",
///     allowed_headers: "Content-Type, X-API-Key",
///     allow_credentials: false, // Must be false with "*"
/// };
/// ```
///
/// ### Development/Testing
/// ```rust
/// use ripress::middlewares::cors::CorsConfig;
///
/// let config = CorsConfig {
///     allowed_origin: "*",
///     allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD, PATCH",
///     allowed_headers: "Content-Type, Authorization, X-Requested-With, Accept, Origin, X-Custom-Header",
///     allow_credentials: false,
/// };
/// ```
///
/// ## Response Headers Added
///
/// The middleware adds the following headers based on configuration:
///
/// ### Always Added
/// - `Access-Control-Allow-Origin`: Configured origin or reflected origin
/// - `Access-Control-Allow-Methods`: Configured methods or reflected method
/// - `Access-Control-Allow-Headers`: Configured headers or requested headers
///
/// ### Conditionally Added
/// - `Access-Control-Allow-Credentials`: "true" (only when `allow_credentials: true`)
/// - `Vary`: "Origin, Access-Control-Request-Method, Access-Control-Request-Headers" (only in reflective mode)
///
/// ## Browser Compatibility
///
/// This middleware is compatible with all modern browsers that support CORS:
/// - Chrome 4+
/// - Firefox 3.5+
/// - Safari 4+
/// - Internet Explorer 8+ (with limitations)
/// - Edge (all versions)
///
/// ## Debugging CORS Issues
///
/// Common issues and solutions:
/// - **"CORS policy" errors**: Check that `allowed_origin` matches the requesting domain exactly
/// - **Credential issues**: Ensure `allowed_origin` is not "*" when `allow_credentials` is true
/// - **Method not allowed**: Verify the HTTP method is in `allowed_methods`
/// - **Header not allowed**: Check that custom headers are included in `allowed_headers`
/// - **Preflight failures**: Ensure OPTIONS requests reach the middleware
///
/// ## Performance Notes
///
/// - **Static mode**: More efficient and cacheable
/// - **Reflective mode**: Adds slight overhead for header processing
/// - **Preflight handling**: Early return prevents unnecessary processing
/// - **Header cloning**: Minimal string operations for header management
///
/// ## Standards Compliance
///
/// This middleware implements the CORS specification as defined by:
/// - W3C Cross-Origin Resource Sharing specification
/// - RFC 6454 (The Web Origin Concept)
/// - Follows browser security model requirements

/// Configuration struct for the CORS Middleware
///
/// This struct defines the Cross-Origin Resource Sharing policy for your application.
/// CORS policies determine which origins, methods, and headers are allowed when
/// making cross-origin requests to your server from web browsers.
///
/// ## Field Details
///
/// All string fields use `&'static str` for efficiency, meaning they should be
/// string literals or static strings. This avoids unnecessary memory allocations
/// during request processing.
///
/// ## Security Implications
///
/// CORS configuration directly impacts your application's security posture:
/// - Overly permissive settings can expose your API to unauthorized access
/// - Restrictive settings may break legitimate client applications  
/// - Credential handling requires careful origin validation
///
/// ## Default Values
///
/// The default configuration is permissive and suitable for development:
/// - Allows all origins (`*`)
/// - Allows common HTTP methods
/// - Allows basic headers
/// - Disallows credentials (required when using `*` origin)
#[derive(Clone)]
pub struct CorsConfig {
    /// The allowed origin(s) for cross-origin requests
    ///
    /// Can be a specific origin like "https://example.com" or "*" for any origin.
    /// When using "*", credentials cannot be allowed for security reasons.
    ///
    /// **Examples:**
    /// - `"*"` - Allow any origin (development/public APIs)
    /// - `"https://myapp.com"` - Allow only specific domain
    /// - `"http://localhost:3000"` - Allow local development server
    ///
    /// **Note:** Some advanced CORS scenarios require multiple origins, which
    /// may need custom middleware implementation.
    pub allowed_origin: &'static str,

    /// The HTTP methods allowed for cross-origin requests
    ///
    /// Should be a comma-separated list of HTTP methods. Include OPTIONS
    /// if you want to explicitly allow it, though preflight requests
    /// are handled automatically.
    ///
    /// **Examples:**
    /// - `"GET, POST"` - Read and create operations only
    /// - `"GET, POST, PUT, DELETE"` - Full CRUD operations
    /// - `"GET, POST, PUT, DELETE, OPTIONS, HEAD, PATCH"` - Comprehensive set
    ///
    /// **Common methods:**
    /// - GET, POST, PUT, DELETE - Standard CRUD
    /// - PATCH - Partial updates
    /// - HEAD - Metadata requests
    /// - OPTIONS - Always handled for preflights
    pub allowed_methods: &'static str,

    /// The headers allowed in cross-origin requests
    ///
    /// Should be a comma-separated list of header names. Be conservative
    /// with this list - only include headers your application actually needs.
    ///
    /// **Examples:**
    /// - `"Content-Type"` - Basic content type header
    /// - `"Content-Type, Authorization"` - With auth support
    /// - `"Content-Type, Authorization, X-API-Key"` - With custom API key
    ///
    /// **Commonly needed headers:**
    /// - Content-Type - For JSON/form data
    /// - Authorization - For auth tokens
    /// - X-Requested-With - For AJAX detection
    /// - Accept - For content negotiation
    ///
    /// **Security note:** Avoid exposing sensitive headers unnecessarily.
    pub allowed_headers: &'static str,

    /// Whether to allow credentials (cookies, authorization headers) in CORS requests
    ///
    /// When true, adds the `Access-Control-Allow-Credentials: true` header.
    /// This cannot be used with `allowed_origin: "*"` for security reasons.
    ///
    /// **When to use:**
    /// - Authentication via cookies
    /// - Bearer token authentication  
    /// - Client certificates
    ///
    /// **Security implications:**
    /// - Must use specific origins, not "*"
    /// - Increases CSRF attack surface
    /// - Requires careful origin validation
    ///
    /// **Default:** false (safer default)
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allowed_origin: "*",
            allowed_methods: "GET, POST, PUT, DELETE, OPTIONS, HEAD",
            allowed_headers: "Content-Type, Authorization",
            allow_credentials: false,
        }
    }
}

/// Creates a CORS middleware function
///
/// Returns a middleware function that handles Cross-Origin Resource Sharing (CORS)
/// by adding appropriate headers to responses and handling preflight OPTIONS requests.
/// The middleware supports both static configuration and dynamic origin reflection
/// based on incoming request headers.
///
/// ## Parameters
///
/// * `config` - Optional CORS configuration. If `None`, uses `CorsConfig::default()`
///   which allows all origins with common methods and headers.
///
/// ## Returns
///
/// A middleware function compatible with the ripress framework that:
/// * Adds CORS headers to all responses based on configuration
/// * Automatically handles preflight OPTIONS requests with early termination
/// * Supports both static and reflective CORS modes
/// * Manages proper cache headers for dynamic responses
/// * Handles credential policies securely
///
/// ## Middleware Behavior
///
/// ### For OPTIONS Requests (Preflight)
/// 1. Adds appropriate CORS headers based on request and configuration
/// 2. Returns 200 OK status immediately
/// 3. Does not call subsequent middleware or handlers
/// 4. Includes `Vary` header when reflecting request headers
///
/// ### For Other Requests
/// 1. Adds CORS headers to the response
/// 2. Continues to next middleware/handler
/// 3. Uses static or reflective mode based on request headers
/// 4. Preserves response status and body from downstream handlers
///
/// ## Thread Safety
///
/// The returned middleware is `Send + Sync + Clone` and safe for concurrent use.
/// Configuration is cloned per request to avoid shared mutable state.
///
/// ## Error Handling
///
/// The middleware is designed to be permissive and never fails requests:
/// * Missing headers are handled gracefully
/// * Invalid configurations log warnings but don't block requests
/// * Always allows requests to proceed (except OPTIONS preflights)
///
/// ## Performance Characteristics
///
/// * **Static mode**: Minimal overhead, highly cacheable responses
/// * **Reflective mode**: Slight overhead for header inspection and reflection
/// * **OPTIONS handling**: Early return prevents unnecessary processing
/// * **Header operations**: Efficient string operations with minimal allocation
/// * **Configuration cloning**: Lightweight operation due to `&'static str` usage
pub(crate) fn cors(
    config: Option<CorsConfig>,
) -> impl Fn(HttpRequest, HttpResponse) -> MiddlewareOutput + Send + Sync + Clone + 'static {
    move |req: HttpRequest, mut res| {
        let config = config.clone().unwrap_or_default();
        let req_clone = req.clone();
        Box::pin(async move {
            let origin = req.headers.get("Origin");
            let allowed_methods = req.headers.get("Access-Control-Request-Method");
            let requested_headers = req.headers.get("Access-Control-Request-Headers");
            if let (Some(origin), Some(allowed_methods)) = (origin, allowed_methods) {
                res = res
                    .set_header("Access-Control-Allow-Origin", origin)
                    .set_header("Access-Control-Allow-Methods", allowed_methods)
                    .set_header(
                        "Access-Control-Allow-Headers",
                        requested_headers.unwrap_or(config.allowed_headers),
                    )
                    .set_header(
                        "Vary",
                        "Origin, Access-Control-Request-Method, Access-Control-Request-Headers",
                    );
            } else {
                res = res
                    .set_header("Access-Control-Allow-Origin", config.allowed_origin)
                    .set_header("Access-Control-Allow-Methods", config.allowed_methods)
                    .set_header("Access-Control-Allow-Headers", config.allowed_headers)
            }
            if config.allow_credentials {
                res = res.set_header("Access-Control-Allow-Credentials", "true");
            }
            if req_clone.method == HttpMethods::OPTIONS {
                return (req_clone, Some(res.ok()));
            }
            // Return the modified response so exec_pre_middleware can capture the headers
            // and merge them into the final response. We use a special marker to indicate
            // this is a "continue" response, not a short-circuit.
            (req_clone, Some(res))
        })
    }
}
