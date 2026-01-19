#[cfg(test)]
mod tests {
    use crate::{
        middlewares::shield::config::{
            ContentSecurityPolicy, CrossDomainPolicy, CrossOriginEmbedderPolicy,
            CrossOriginOpenerPolicy, CrossOriginResourcePolicy, DnsPrefetchControl, Frameguard,
            HidePoweredBy, Hsts, IENoOpen, NoSniff, OriginAgentCluster, PermissionsPolicy,
            ReferrerPolicy, XssFilter,
        },
        middlewares::shield::{
            set_content_security_policy, set_cross_domain_policy, set_cross_origin_embedder_policy,
            set_cross_origin_opener_policy, set_cross_origin_resource_policy,
            set_dns_prefetch_control, set_frameguard, set_hide_powered_by, set_hsts,
            set_ie_no_open, set_no_sniff, set_origin_agent_cluster, set_permissions_policy,
            set_referrer_policy, set_xss_filter,
        },
        res::HttpResponse,
    };

    use std::collections::HashMap;

    #[test]
    fn test_set_content_security_policy_enabled() {
        let mut res = HttpResponse::new();
        let mut directives = HashMap::new();
        directives.insert("default-src".to_string(), "'self'".to_string());
        directives.insert("script-src".to_string(), "'self'".to_string());
        let csp = ContentSecurityPolicy {
            enabled: true,
            directives: directives.clone(),
            report_only: false,
        };
        set_content_security_policy(&mut res, &csp);
        let val = res.headers.get("content-security-policy").unwrap();
        // Order is sorted by key
        assert_eq!(val, "default-src 'self'; script-src 'self'");
    }

    #[test]
    fn test_set_content_security_policy_report_only() {
        let mut res = HttpResponse::new();
        let mut directives = HashMap::new();
        directives.insert("default-src".to_string(), "'self'".to_string());
        let csp = ContentSecurityPolicy {
            enabled: true,
            directives,
            report_only: true,
        };
        set_content_security_policy(&mut res, &csp);
        assert!(res
            .headers
            .get("content-security-policy-report-only")
            .is_some());
    }

    #[test]
    fn test_set_content_security_policy_disabled() {
        let mut res = HttpResponse::new();
        let csp = ContentSecurityPolicy {
            enabled: false,
            directives: HashMap::new(),
            report_only: false,
        };
        set_content_security_policy(&mut res, &csp);
        assert!(res.headers.get("content-security-policy").is_none());
    }

    #[test]
    fn test_set_hsts_enabled() {
        let mut res = HttpResponse::new();
        let hsts = Hsts {
            enabled: true,
            max_age: 123,
            include_subdomains: true,
            preload: true,
        };
        set_hsts(&mut res, &hsts);
        let val = res.headers.get("strict-transport-security").unwrap();
        assert_eq!(val, "max-age=123; includeSubDomains; preload");
    }

    #[test]
    fn test_set_hsts_disabled() {
        let mut res = HttpResponse::new();
        let hsts = Hsts {
            enabled: false,
            max_age: 123,
            include_subdomains: true,
            preload: true,
        };
        set_hsts(&mut res, &hsts);
        assert!(res.headers.get("strict-transport-security").is_none());
    }

    #[test]
    fn test_set_frameguard_deny() {
        let mut res = HttpResponse::new();
        let fg = Frameguard {
            enabled: true,
            action: "deny".to_string(),
            domain: None,
        };
        set_frameguard(&mut res, &fg);
        assert_eq!(res.headers.get("x-frame-options").unwrap(), "DENY");
    }

    #[test]
    fn test_set_frameguard_sameorigin() {
        let mut res = HttpResponse::new();
        let fg = Frameguard {
            enabled: true,
            action: "sameorigin".to_string(),
            domain: None,
        };
        set_frameguard(&mut res, &fg);
        assert_eq!(res.headers.get("x-frame-options").unwrap(), "SAMEORIGIN");
    }

    #[test]
    fn test_set_frameguard_allow_from_with_domain() {
        let mut res = HttpResponse::new();
        let fg = Frameguard {
            enabled: true,
            action: "allow-from".to_string(),
            domain: Some("https://example.com".to_string()),
        };
        set_frameguard(&mut res, &fg);
        assert_eq!(
            res.headers.get("x-frame-options").unwrap(),
            "ALLOW-FROM https://example.com"
        );
    }

    #[test]
    fn test_set_frameguard_allow_from_without_domain() {
        let mut res = HttpResponse::new();
        let fg = Frameguard {
            enabled: true,
            action: "allow-from".to_string(),
            domain: None,
        };
        set_frameguard(&mut res, &fg);
        assert_eq!(res.headers.get("x-frame-options").unwrap(), "DENY");
    }

    #[test]
    fn test_set_no_sniff_enabled() {
        let mut res = HttpResponse::new();
        let ns = NoSniff { enabled: true };
        set_no_sniff(&mut res, &ns);
        assert_eq!(
            res.headers.get("x-content-type-options").unwrap(),
            "nosniff"
        );
    }

    #[test]
    fn test_set_no_sniff_disabled() {
        let mut res = HttpResponse::new();
        let ns = NoSniff { enabled: false };
        set_no_sniff(&mut res, &ns);
        assert!(res.headers.get("x-content-type-options").is_none());
    }

    #[test]
    fn test_set_xss_filter_enabled_block() {
        let mut res = HttpResponse::new();
        let xf = XssFilter {
            enabled: true,
            mode: "block".to_string(),
            report_uri: None,
        };
        set_xss_filter(&mut res, &xf);
        assert_eq!(
            res.headers.get("x-xss-protection").unwrap(),
            "1; mode=block"
        );
    }

    #[test]
    fn test_set_xss_filter_enabled_block_with_report() {
        let mut res = HttpResponse::new();
        let xf = XssFilter {
            enabled: true,
            mode: "block".to_string(),
            report_uri: Some("https://report".to_string()),
        };
        set_xss_filter(&mut res, &xf);
        assert_eq!(
            res.headers.get("x-xss-protection").unwrap(),
            "1; mode=block; report=https://report"
        );
    }

    #[test]
    fn test_set_xss_filter_disabled() {
        let mut res = HttpResponse::new();
        let xf = XssFilter {
            enabled: false,
            mode: "block".to_string(),
            report_uri: None,
        };
        set_xss_filter(&mut res, &xf);
        assert!(res.headers.get("x-xss-protection").is_none());
    }

    #[test]
    fn test_set_referrer_policy_enabled() {
        let mut res = HttpResponse::new();
        let rp = ReferrerPolicy {
            enabled: true,
            policy: "no-referrer".to_string(),
        };
        set_referrer_policy(&mut res, &rp);
        assert_eq!(res.headers.get("referrer-policy").unwrap(), "no-referrer");
    }

    #[test]
    fn test_set_referrer_policy_disabled() {
        let mut res = HttpResponse::new();
        let rp = ReferrerPolicy {
            enabled: false,
            policy: "no-referrer".to_string(),
        };
        set_referrer_policy(&mut res, &rp);
        assert!(res.headers.get("referrer-policy").is_none());
    }

    #[test]
    fn test_set_dns_prefetch_control_on() {
        let mut res = HttpResponse::new();
        let dpc = DnsPrefetchControl {
            enabled: true,
            allow: true,
        };
        set_dns_prefetch_control(&mut res, &dpc);
        assert_eq!(res.headers.get("x-dns-prefetch-control").unwrap(), "on");
    }

    #[test]
    fn test_set_dns_prefetch_control_off() {
        let mut res = HttpResponse::new();
        let dpc = DnsPrefetchControl {
            enabled: true,
            allow: false,
        };
        set_dns_prefetch_control(&mut res, &dpc);
        assert_eq!(res.headers.get("x-dns-prefetch-control").unwrap(), "off");
    }

    #[test]
    fn test_set_dns_prefetch_control_disabled() {
        let mut res = HttpResponse::new();
        let dpc = DnsPrefetchControl {
            enabled: false,
            allow: true,
        };
        set_dns_prefetch_control(&mut res, &dpc);
        assert!(res.headers.get("x-dns-prefetch-control").is_none());
    }

    #[test]
    fn test_set_ie_no_open_enabled() {
        let mut res = HttpResponse::new();
        let ie = IENoOpen { enabled: true };
        set_ie_no_open(&mut res, &ie);
        assert_eq!(res.headers.get("x-download-options").unwrap(), "noopen");
    }

    #[test]
    fn test_set_ie_no_open_disabled() {
        let mut res = HttpResponse::new();
        let ie = IENoOpen { enabled: false };
        set_ie_no_open(&mut res, &ie);
        assert!(res.headers.get("x-download-options").is_none());
    }

    #[test]
    fn test_set_hide_powered_by_enabled() {
        let mut res = HttpResponse::new();
        res.headers.insert("x-powered-by", "Express");
        res.headers.insert("X-Powered-By", "PHP");
        let hp = HidePoweredBy { enabled: true };
        set_hide_powered_by(&mut res, &hp);
        assert!(res.headers.get("x-powered-by").is_none());
        assert!(res.headers.get("X-Powered-By").is_none());
    }

    #[test]
    fn test_set_hide_powered_by_disabled() {
        let mut res = HttpResponse::new();
        res.headers.insert("x-powered-by", "Express");
        let hp = HidePoweredBy { enabled: false };
        set_hide_powered_by(&mut res, &hp);
        assert_eq!(res.headers.get("x-powered-by").unwrap(), "Express");
    }

    #[test]
    fn test_set_permissions_policy_enabled() {
        let mut res = HttpResponse::new();
        let mut features = HashMap::new();
        features.insert("camera".to_string(), vec![]);
        features.insert(
            "geolocation".to_string(),
            vec!["self".to_string(), "https://foo.com".to_string()],
        );
        let pp = PermissionsPolicy {
            enabled: true,
            features,
        };
        set_permissions_policy(&mut res, &pp);
        let val = res.headers.get("permissions-policy").unwrap();
        // Order is not guaranteed, so check both substrings
        assert!(val.contains("camera=()"));
        assert!(
            val.contains("geolocation=(self \"https://foo.com\")")
                || val.contains("geolocation=(\"https://foo.com\" self)")
        );
    }

    #[test]
    fn test_set_permissions_policy_disabled() {
        let mut res = HttpResponse::new();
        let pp = PermissionsPolicy {
            enabled: false,
            features: HashMap::new(),
        };
        set_permissions_policy(&mut res, &pp);
        assert!(res.headers.get("permissions-policy").is_none());
    }

    #[test]
    fn test_set_cross_origin_opener_policy() {
        let mut res = HttpResponse::new();
        set_cross_origin_opener_policy(&mut res, &CrossOriginOpenerPolicy::SameOrigin);
        assert_eq!(
            res.headers.get("cross-origin-opener-policy").unwrap(),
            "same-origin"
        );
        set_cross_origin_opener_policy(&mut res, &CrossOriginOpenerPolicy::SameOriginAllowPopups);
        assert_eq!(
            res.headers.get("cross-origin-opener-policy").unwrap(),
            "same-origin-allow-popups"
        );
        set_cross_origin_opener_policy(&mut res, &CrossOriginOpenerPolicy::UnsafeNone);
        assert_eq!(
            res.headers.get("cross-origin-opener-policy").unwrap(),
            "unsafe-none"
        );
    }

    #[test]
    fn test_set_cross_origin_resource_policy() {
        let mut res = HttpResponse::new();
        set_cross_origin_resource_policy(&mut res, &CrossOriginResourcePolicy::SameOrigin);
        assert_eq!(
            res.headers.get("cross-origin-resource-policy").unwrap(),
            "same-origin"
        );
        set_cross_origin_resource_policy(&mut res, &CrossOriginResourcePolicy::SameSite);
        assert_eq!(
            res.headers.get("cross-origin-resource-policy").unwrap(),
            "same-site"
        );
        set_cross_origin_resource_policy(&mut res, &CrossOriginResourcePolicy::CrossOrigin);
        assert_eq!(
            res.headers.get("cross-origin-resource-policy").unwrap(),
            "cross-origin"
        );
    }

    #[test]
    fn test_set_cross_origin_embedder_policy() {
        let mut res = HttpResponse::new();
        set_cross_origin_embedder_policy(&mut res, &CrossOriginEmbedderPolicy::RequireCorp);
        assert_eq!(
            res.headers.get("cross-origin-embedder-policy").unwrap(),
            "require-corp"
        );
        set_cross_origin_embedder_policy(&mut res, &CrossOriginEmbedderPolicy::UnsafeNone);
        assert_eq!(
            res.headers.get("cross-origin-embedder-policy").unwrap(),
            "unsafe-none"
        );
    }

    #[test]
    fn test_set_origin_agent_cluster_enabled() {
        let mut res = HttpResponse::new();
        let oac = OriginAgentCluster { enabled: true };
        set_origin_agent_cluster(&mut res, &oac);
        assert_eq!(res.headers.get("origin-agent-cluster").unwrap(), "?1");
    }

    #[test]
    fn test_set_origin_agent_cluster_disabled() {
        let mut res = HttpResponse::new();
        let oac = OriginAgentCluster { enabled: false };
        set_origin_agent_cluster(&mut res, &oac);
        assert!(res.headers.get("origin-agent-cluster").is_none());
    }

    #[test]
    fn test_set_cross_domain_policy_enabled() {
        let mut res = HttpResponse::new();
        let cdp = CrossDomainPolicy {
            enabled: true,
            policy: "none".to_string(),
        };
        set_cross_domain_policy(&mut res, &cdp);
        assert_eq!(
            res.headers
                .get("x-permitted-cross-domain-policies")
                .unwrap(),
            "none"
        );
    }

    #[test]
    fn test_set_cross_domain_policy_disabled() {
        let mut res = HttpResponse::new();
        let cdp = CrossDomainPolicy {
            enabled: false,
            policy: "none".to_string(),
        };
        set_cross_domain_policy(&mut res, &cdp);
        assert!(res
            .headers
            .get("x-permitted-cross-domain-policies")
            .is_none());
    }
}
