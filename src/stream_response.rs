use std::collections::HashMap;

use bt_logger::{log_error, log_warning};
use reqwest::Response;

use crate::{convert_headers, HttpResponse};

const MAX_NUMBER_ERROR: i8 = 5;

///The HttpStreamResponse struct and its associated methods are designed to handle HTTP responses, including logging errors, 
/// checking response statuses, and reading the response stream asynchronously.
/// 
///Fields:
/// - ini_status_str: Initial status string of the HTTP response.
/// - ini_header: Initial headers of the HTTP response.
/// - url: URL from which the response was received.
/// - remote_address: Remote address of the server that sent the response.
/// - error_count: Counter for errors encountered during stream reading.
/// - resp: The actual reqwest::Response object.
/// 
/// Methods:
/// - new(http_resp: Response) -> Self: Initializes a new instance of HttpStreamResponse from a reqwest::Response.
/// - is_error() -> bool: Checks if the HTTP status code indicates an error.
/// - get_status() -> u16: Returns the HTTP status code as an unsigned 16-bit integer.
/// - get_ini_header() -> HashMap: Returns a copy of the initial headers.
/// - read_stream(&mut self) -> Option: Asynchronously reads and processes the stream from the HTTP response. 
///     It handles errors by logging them and optionally stopping execution if too many errors occur.
///     The read_stream method uses asynchronous I/O to read chunks from the response stream.
///     It processes each chunk individually, converting it to a string if possible.
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
                let chunk = self.resp.chunk().await;
                match chunk { 
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
                            None => return None, //Stop
                        }
                    },
                    Err(e) => {
                        self.error_count += 1;
                        if self.error_count > MAX_NUMBER_ERROR{
                           log_error!("read_stream","Error reading streaming (>{} errors) from {}. Stop Executing returning None. Error {}",MAX_NUMBER_ERROR, self.url, e);
                           return None //Stop
                        }

                        log_error!("read_stream","Error reading streaming from {}. Return Empty body but continue. Error {}", &self.url, e);
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