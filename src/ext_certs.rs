use std::{env, fs, path::Path};

use bt_logger::{log_error, log_info, log_warning};
use native_tls::{Certificate, TlsConnector};

use crate::{DANGER_ACCEPT_INVALID_CERTS, DANGER_ACCEPT_INVALID_HOSTNAMES};

const LOCAL_CERTIFICATES: &str = "certs";
const LOCAL_CERTIFICATES_ENV_VAR_NAME: &str = "BTLOCALPEMCERTIFICATES";



/// Scans the "certs" directory and returns all `.pem` file paths.
fn get_cert_files() -> Vec<String> {
    let mut certs = Vec::new();
    let cert_dir: String;
    match env::var(LOCAL_CERTIFICATES_ENV_VAR_NAME){
        Ok(d) => cert_dir = d,
        Err(_) => cert_dir = LOCAL_CERTIFICATES.to_owned(),
    }


    if let Ok(entries) = fs::read_dir(&cert_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "pem") {
                if let Some(path_str) = path.to_str() {
                    certs.push(path_str.to_string());
                }
            }
        }
    } else {
        log_info!("get_cert_files","Could not read directory '{}'. Assuming no local certificates (PEM files)", &cert_dir);
    }

    certs
}

pub(crate) fn get_local_certificates(danger_accept_invalid: Option<Vec<(&str,bool)>>) -> Option<TlsConnector> {
    let certs_str = get_cert_files();
    if certs_str.len() <= 0{
        return None
    }

    let mut tls_builder = TlsConnector::builder();
    for cert_path in certs_str {
        if Path::new(&cert_path).exists() {
            match fs::read(&cert_path){
                Ok(cert_bytes) => {
                    match Certificate::from_pem(&cert_bytes){
                        Ok(cert) => {
                                tls_builder.add_root_certificate(cert);
                        },
                        Err(e) => log_error!("get_local_certificate", "Could not read PEM file at path: {}. Error: {}",cert_path,e),
                    }
                },
                Err(e) => log_error!("get_local_certificate", "Could not read PEM file at path: {}. Error: {}",cert_path,e),
            }
        }else{
            log_error!("get_local_certificate", "Invalid certificate path: {}",cert_path);
        }
    }

    if let Some(daiv) = danger_accept_invalid{
        if daiv.len() > 0 {
            for item in daiv {
                if item.0 ==  DANGER_ACCEPT_INVALID_HOSTNAMES {
                    tls_builder.danger_accept_invalid_hostnames(item.1);
                } else {
                    if item.0 == DANGER_ACCEPT_INVALID_CERTS {
                        tls_builder.danger_accept_invalid_certs(item.1);
                    }else{
                        log_warning!("get_local_certificates","Inalid Danger Accept Invalid key {}",item.0);
                    }
                }
            }
        }
    }

    match tls_builder.build(){
        Ok(conn) => return Some(conn),
        Err(e) => {
            log_error!("get_local_certificate","Could not built TLS Connector. Error {}",&e);
            None
        }
    }
}