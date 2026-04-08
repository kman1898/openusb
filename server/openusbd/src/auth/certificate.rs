// Mutual TLS / client certificate authentication.
//
// When tls_client_certs is enabled in config, the server requires
// clients to present a valid certificate signed by the configured CA.
// The certificate's Common Name (CN) is used as the username for ACL.

/// Extract the Common Name from a client certificate's subject.
pub fn extract_cn(subject: &str) -> Option<String> {
    for part in subject.split(',') {
        let trimmed = part.trim();
        if let Some(cn) = trimmed.strip_prefix("CN=") {
            return Some(cn.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cn() {
        assert_eq!(
            extract_cn("CN=alice, O=OpenUSB, C=US"),
            Some("alice".to_string())
        );
        assert_eq!(extract_cn("O=OpenUSB"), None);
    }
}
