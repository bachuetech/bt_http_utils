use std::{env, fs};
use bt_logger::{log_error, log_info, log_warning};
use native_tls::{Certificate, TlsConnector};
use crate::{DANGER_ACCEPT_INVALID_CERTS, DANGER_ACCEPT_INVALID_HOSTNAMES};

const LOCAL_CERTIFICATES: &str = "certs";
const LOCAL_CERTIFICATES_ENV_VAR_NAME: &str = "BT_LOCALPEMCERTIFICATES_DIR";

#[inline]
fn get_cert_files() -> Vec<String> {
    let cert_dir = env::var(LOCAL_CERTIFICATES_ENV_VAR_NAME)
        .unwrap_or_else(|_| LOCAL_CERTIFICATES.to_owned());

    let entries = match fs::read_dir(&cert_dir) {
        Ok(e) => e,
        Err(_) => {
            log_info!("", "Could not read directory '{}'. Assuming no local certificates", &cert_dir);
            return Vec::new();
        }
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "pem"))
        .filter_map(|e| e.path().to_str().map(str::to_owned))
        .collect()
}

pub(crate) fn get_local_certificates(danger_accept_invalid: Option<Vec<(String, bool)>>) -> Option<TlsConnector> {
    let cert_files = get_cert_files();
    if cert_files.is_empty() {
        return None;
    }

    let mut builder = TlsConnector::builder();

    for path in &cert_files {
        match fs::read(path).map_err(|e| e.to_string())
            .and_then(|bytes| Certificate::from_pem(&bytes).map_err(|e| e.to_string())) 
        {
            Ok(cert) => { builder.add_root_certificate(cert); }
            Err(e) => log_error!("", "Failed to load certificate '{}': {}", path, e),
        }
    }

    if let Some(daiv) = danger_accept_invalid {
        for (key, value) in daiv {
            match key.as_str() {
                DANGER_ACCEPT_INVALID_HOSTNAMES => {builder.danger_accept_invalid_hostnames(value);},
                DANGER_ACCEPT_INVALID_CERTS => {builder.danger_accept_invalid_certs(value);},
                _ => {log_warning!("", "Invalid danger_accept_invalid key: {}", key);},
            };
        }
    }

    builder.build().map(Some).unwrap_or_else(|e| {
        log_error!("", "Failed to build TLS connector: {}", e);
        None
    })
}


/******************* */
//unit tests      */
//***************** */


#[cfg(test)]
mod ext_certs_tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
use std::path::Path;
    use tempfile::TempDir;

    const VALID_PEM: &str = r#"-----BEGIN CERTIFICATE-----
MIIBszCCAVmgAwIBAgIUWYz6z8UOKz9GxMHnR2Z5HvF8HVEwCgYIKoZIzj0EAwIw
FDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI0MDEwMTAwMDAwMFoXDTI1MDEwMTAw
MDAwMFowFDESMBAGA1UEAwwJbG9jYWxob3N0MFkwEwYHKoZIzj0CAQYIKoZIzj0D
AQcDQgAE8QUVPIYBbBwFzKPRPHUPVpTZzQbxVnBvHhpKrCaI8/E1sF2yjjPJJOqv
bJ9U6WK8QzEBKTnBWBC5+5oDA9sDM6NTMFEwHQYDVR0OBBYEFOGH7UvBqLSqAwHg
PSRfZZEE2r/NMB8GA1UdIwQYMBaAFOGH7UvBqLSqAwHgPSRfZZEE2r/NMA8GA1Ud
EwEB/wQFMAMBAf8wCgYIKoZIzj0EAwIDSQAwRgIhAN2/NbTDO9oPKqlDqkjHNKdd
EO5RkWjfVdROrBnPrNVPAiEA4lPQqJYEhxpHBH/BjLEhWbTiA5LBcOJz6DNnUy0z
cYY=
-----END CERTIFICATE-----"#;

    const INVALID_PEM: &str = "not a valid certificate";

    fn setup_test_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    fn create_pem_file(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut file = File::create(&path).expect("Failed to create file");
        file.write_all(content.as_bytes()).expect("Failed to write file");
    }

    fn with_env_var<F, R>(dir: &Path, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe { env::set_var(LOCAL_CERTIFICATES_ENV_VAR_NAME, dir.to_str().unwrap()) };
        let result = f();
        unsafe { env::remove_var(LOCAL_CERTIFICATES_ENV_VAR_NAME) };
        result
    }

    mod get_cert_files_tests {
        use super::*;

        #[test]
        fn test_empty_directory() {
            let temp = setup_test_dir();
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert!(files.is_empty());
            });
        }

        #[test]
        fn test_single_pem_file() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert_eq!(files.len(), 1);
                assert!(files[0].ends_with("cert.pem"));
            });
        }

        #[test]
        fn test_multiple_pem_files() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert1.pem", VALID_PEM);
            create_pem_file(temp.path(), "cert2.pem", VALID_PEM);
            create_pem_file(temp.path(), "cert3.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert_eq!(files.len(), 3);
            });
        }

        #[test]
        fn test_ignores_non_pem_files() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert.pem", VALID_PEM);
            create_pem_file(temp.path(), "cert.txt", "text file");
            create_pem_file(temp.path(), "cert.key", "key file");
            create_pem_file(temp.path(), "cert.crt", "crt file");
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert_eq!(files.len(), 1);
                assert!(files[0].ends_with(".pem"));
            });
        }

        #[test]
        fn test_ignores_directories() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert.pem", VALID_PEM);
            fs::create_dir(temp.path().join("subdir.pem")).unwrap();
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert_eq!(files.len(), 1);
            });
        }

        #[test]
        fn test_nonexistent_directory() {
            unsafe { env::set_var(LOCAL_CERTIFICATES_ENV_VAR_NAME, "/nonexistent/path/12345") };
            let files = get_cert_files();
            assert!(files.is_empty());
            unsafe { env::remove_var(LOCAL_CERTIFICATES_ENV_VAR_NAME) };
        }

        #[test]
        fn test_file_without_extension() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert", VALID_PEM);
            create_pem_file(temp.path(), "cert.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                assert_eq!(files.len(), 1);
                assert!(files[0].ends_with(".pem"));
            });
        }

        #[test]
        fn test_case_sensitive_extension() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert.PEM", VALID_PEM);
            create_pem_file(temp.path(), "cert.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let files = get_cert_files();
                // Only lowercase .pem should match
                assert_eq!(files.len(), 1);
                assert!(files[0].ends_with(".pem"));
            });
        }
    }

    mod get_local_certificates_tests {
        use super::*;

        #[test]
        fn test_no_cert_files_returns_none() {
            let temp = setup_test_dir();
            with_env_var(temp.path(), || {
                let result = get_local_certificates(None);
                assert!(result.is_none());
            });
        }

        #[test]
        fn test_valid_cert_returns_connector() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let result = get_local_certificates(None);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_invalid_cert_returns_connector_without_cert() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "invalid.pem", INVALID_PEM);
            
            with_env_var(temp.path(), || {
                let result = get_local_certificates(None);
                // Should still return Some because builder builds successfully
                // but the invalid cert is skipped
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_mixed_valid_and_invalid_certs() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            create_pem_file(temp.path(), "invalid.pem", INVALID_PEM);
            
            with_env_var(temp.path(), || {
                let result = get_local_certificates(None);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_danger_accept_invalid_hostnames() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![
                    (DANGER_ACCEPT_INVALID_HOSTNAMES.to_string(), true)
                ]);
                let result = get_local_certificates(danger);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_danger_accept_invalid_certs() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![
                    (DANGER_ACCEPT_INVALID_CERTS.to_string(), true)
                ]);
                let result = get_local_certificates(danger);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_both_danger_options() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![
                    (DANGER_ACCEPT_INVALID_HOSTNAMES.to_string(), true),
                    (DANGER_ACCEPT_INVALID_CERTS.to_string(), true),
                ]);
                let result = get_local_certificates(danger);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_invalid_danger_key() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![
                    ("invalid_key".to_string(), true)
                ]);
                let result = get_local_certificates(danger);
                // Should still work, just logs a warning
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_empty_danger_vec() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![]);
                let result = get_local_certificates(danger);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_with_danger_false_values() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "valid.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let danger = Some(vec![
                    (DANGER_ACCEPT_INVALID_HOSTNAMES.to_string(), false),
                    (DANGER_ACCEPT_INVALID_CERTS.to_string(), false),
                ]);
                let result = get_local_certificates(danger);
                assert!(result.is_some());
            });
        }

        #[test]
        fn test_multiple_valid_certs() {
            let temp = setup_test_dir();
            create_pem_file(temp.path(), "cert1.pem", VALID_PEM);
            create_pem_file(temp.path(), "cert2.pem", VALID_PEM);
            create_pem_file(temp.path(), "cert3.pem", VALID_PEM);
            
            with_env_var(temp.path(), || {
                let result = get_local_certificates(None);
                assert!(result.is_some());
            });
        }
    }

    mod constants_tests {
        use super::*;

        #[test]
        fn test_constants_values() {
            assert_eq!(LOCAL_CERTIFICATES, "certs");
            assert_eq!(LOCAL_CERTIFICATES_ENV_VAR_NAME, "BT_LOCALPEMCERTIFICATES_DIR");
            assert_eq!(DANGER_ACCEPT_INVALID_HOSTNAMES, "danger_accept_invalid_hostnames");
            assert_eq!(DANGER_ACCEPT_INVALID_CERTS, "danger_accept_invalid_certs");
        }
    }
}