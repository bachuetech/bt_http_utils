use std::collections::HashMap;

use bt_logger::{log_error, log_warning};
use reqwest::Response;

use crate::{convert_headers, HttpResponse};

#[derive(Debug)]
pub struct HttpStreamResponse {
    //ini_status_code: u16,
    ini_status_str: String,
    ini_header: HashMap<String, String>,
    url: String,
    remote_address: String,
    error_count: i8,
    resp: Response,
}

impl HttpStreamResponse {
    pub fn new(http_resp: Response) -> Self{
        let ra = match http_resp.remote_addr() {
            Some(ip) => ip.ip().to_string(),
            None => {
                log_warning!("new", "Remote Address not found. Using default 0.0.0.0");
                "0.0.0.0".to_owned()
            },
        };


        Self { 
            ini_status_str: http_resp.status().canonical_reason().unwrap_or("UNKNOWN ERROR!").to_owned(),
            ini_header: convert_headers(http_resp.headers()), 
            url: http_resp.url().to_string(), 
            remote_address: ra, 
            error_count: 0,
            resp: http_resp,
        }
    }

    pub fn is_error(&self) -> bool{
        self.resp.status().is_client_error() || self.resp.status().is_server_error()
    }

    pub fn get_status(&self) -> u16{
        self.resp.status().as_u16()
    }

    pub fn get_ini_header(&self) -> HashMap<String, String>{
        self.ini_header.clone()
    }

    pub async fn read_stream(&mut self) -> Option<HttpResponse> {
        if self.is_error() { //if response.status().is_client_error() || response.status().is_server_error() {
            log_error!( "read_stream", "ERROR: Failed to read stream response from {}. Status Code: {} ({})", self.url,self.get_status(),self.ini_status_str );
            return Some(HttpResponse {
                status_code: self.get_status(),//response.status().as_u16(),
                header: self.get_ini_header(), //convert_headers(response.headers()),
                body: format!( "ERROR: Failed to read stream response from {}. Status: {}.", self.url, self.ini_status_str ),
                remote_address: self.remote_address.clone(),
            });
        } else {
                    match self.resp.chunk().await { 
                        Ok(r) => {
                            match r{
                                Some(chunk) => {
                                    return Some(HttpResponse {
                                        status_code: self.get_status(),
                                        header: convert_headers(self.resp.headers()),
                                        body: (&String::from_utf8_lossy(&chunk)).to_string(),
                                        remote_address: self.remote_address.clone(),
                                    })
                                },
                                None => return None,
                            }
                        },
                        Err(e) => {
                            self.error_count += 1;
                            if  self.error_count > 3{
                                log_error!("read_stream","Error reading streaming (>3 errors) from {}. Stop Executing return None. Error {}",self.url, e);
                                return None
                            }

                            log_error!("read_stream","Error reading streaming from {}. Return Empty body. Error {}",self.url, e);
                            return Some(HttpResponse {
                                status_code: self.get_status(),
                                header: convert_headers(self.resp.headers()),
                                body: "".to_owned(),
                                remote_address: self.remote_address.clone(),
                            });
                        },
                    }
            }
        }        
}