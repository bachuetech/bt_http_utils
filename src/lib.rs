use std::{collections::HashMap, io::{Error, ErrorKind}};

use bt_logger::{get_error, log_error, log_trace};
use reqwest::{header::{self, HeaderMap}, Client, Response, StatusCode};

pub struct HttpClient {
    client: Client,
}


#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub header: HashMap<String, String>,
    pub body: String,
}

pub enum ContentType{
    JSON,
    TEXT,
}

impl HttpClient {
    pub fn new(use_hickory_dns: bool) -> Self {
        let c =             
        if use_hickory_dns{
            Client::new()
        }else{
            Client::builder().hickory_dns(true).build().unwrap()
        };

        Self {
                client: c,
        }
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse, Error> {
        match self
            .client
            .get( url)
            .send()
            .await
            {
                Ok(resp) => return Ok(Self::extract_response(resp, url, "GET").await),
                Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("get","Failed to get response from GET: {}. Error: {}",url,e.to_string()))),
            }
    }

    pub async fn post(&self, url: &str, body_request: &str, content_type: ContentType) -> Result<HttpResponse, Error>{
        log_trace!("post","Getting {} with payload: {}", url, body_request);
        match content_type{
            ContentType::JSON =>    {match self.client
                                        .post(url)
                                        .header(header::CONTENT_TYPE, "application/json")
                                        .body(body_request.to_owned())
                                        .send()
                                        .await {
                                            Ok(resp) => return Ok(Self::extract_response(resp, url, "POST").await),
                                            Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("get","Failed to get response from POST(JSON): {}. Error: {}",url,e.to_string()))),
                                        }
                                    },
            ContentType::TEXT =>    {match self.client
                                        .post(url)
                                        .body(body_request.to_string())
                                        .send()
                                        .await{
                                            Ok(resp) => return Ok(Self::extract_response(resp, url, "POST").await),
                                            Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("get","Failed to get response from POST (TEXT): {}. Error: {}",url,e.to_string()))),
                                        }
                                    },
        }

    }

    async fn extract_response(resp: Response, url: &str, method: &str) -> HttpResponse{
        if resp.status().is_client_error() || resp.status().is_server_error() {
            log_error!("extract_response", "ERROR: Failed to get response from {}: {} Satus Code: {}", method, url, resp.status());
            return HttpResponse{
                status_code: resp.status().as_u16(),
                header: Self::convert_headers(resp.headers()),
                body: format!("ERROR: Failed to get response from {}:{} -Error: {}",method, url, resp.status().canonical_reason().unwrap_or("UNKNOWN ERROR!")),
            }
        }else{
            return HttpResponse{
                status_code: resp.status().as_u16(),
                header:Self::convert_headers(resp.headers()),
                body: resp
                        .text()
                        .await
                        .expect(get_error!("extract_response","ERROR: Failed to get payload from {}:{}",method,url).as_str()),
            }
        }
    }

    fn convert_headers(headers: &HeaderMap) -> HashMap<String, String> {
        headers.iter().map(|(k, v)| {
            (
                k.to_string(),
                v.to_str().unwrap_or_default().to_string(),
            )
        }).collect()
    }
}

impl HttpResponse {
    pub fn is_error(&self) -> bool{
        let sc = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::FORBIDDEN);
        sc.is_client_error() || sc.is_server_error()
    }
}
