use std::{collections::HashMap, io::{Error, ErrorKind}, str::FromStr, sync::Arc};

use bt_logger::{get_error, log_error, log_trace};
use reqwest::{cookie::Jar, header::{self, HeaderMap, HeaderName, HeaderValue}, Client, Response, StatusCode};

pub struct HttpClient {
    client: Client,
    headers: HeaderMap,
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
    pub fn new(use_hickory_dns: bool, use_cookies: bool) -> Self {
        let c =             
        if use_cookies {
            let cookie_store = Arc::new(Jar::default());

            Client::builder()
                    .cookie_store(true)
                    .cookie_provider(cookie_store.clone())
                    .hickory_dns(use_hickory_dns)
                    .build().unwrap()
        }else{
            Client::builder()
                    .cookie_store(false)
                    .hickory_dns(use_hickory_dns)
                    .build().unwrap()
        };
        let mut h =  HeaderMap::new();
        h.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (compatible; BachueTech/1.0)"));

        Self {
                client: c,
                headers: h,
        }
    }

    pub fn set_header(&mut self, header_name: &str, header_value: &str) {
        self.headers.insert(HeaderName::from_str(&header_name).unwrap(), HeaderValue::from_str(&header_value).unwrap());
    }

    pub fn get_default_headers(&self) -> HashMap<String, String>{
        Self::convert_headers(&self.headers)
    }

    fn get_extra_headers(&self, extra_headers: Option<HashMap<&str, &str>>) -> HeaderMap{
        let mut local_headers = self.headers.clone();
        if let Some(new_headers) = extra_headers {
            // Add headers from HashMap into the existing HeaderMap
            for (key, value) in new_headers {
                local_headers.insert(HeaderName::from_str(&key).unwrap(), HeaderValue::from_str(value).unwrap());
            }
        }

        local_headers
    }

    pub async fn get(&self, url: &str, extra_headers: Option<HashMap<&str, &str>>) -> Result<HttpResponse, Error> {
        let local_headers = self.get_extra_headers(extra_headers);


        match self
            .client
            .get(url)
            .headers(local_headers)
            .send()
            .await
            {
                Ok(resp) => return Ok(Self::extract_response(resp, url, "GET").await),
                Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("get","Failed to get response from GET: {}. Error: {}",url,e.to_string()))),
            }
    }

    pub async fn post(&self, url: &str, extra_headers: Option<HashMap<&str, &str>>, body_request: &str, content_type: ContentType) -> Result<HttpResponse, Error>{
        log_trace!("post","Getting {} with payload: {}", url, body_request);
        let mut local_headers = self.get_extra_headers(extra_headers); //self.headers.clone();
        match content_type{
            ContentType::JSON =>    {local_headers.insert(header::CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
                                        match self.client
                                        .post(url)
                                        .headers(local_headers)
                                        .body(body_request.to_owned())
                                        .send()
                                        .await {
                                            Ok(resp) => return Ok(Self::extract_response(resp, url, "POST").await),
                                            Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("post","Failed to get response from POST(JSON): {}. Error: {}",url,e.to_string()))),
                                        }
                                    },
            ContentType::TEXT =>    {local_headers.insert(header::CONTENT_TYPE, HeaderValue::from_str("application/text").unwrap());
                                        match self.client
                                        .post(url)
                                        .headers(local_headers)
                                        .body(body_request.to_string())
                                        .send()
                                        .await{
                                            Ok(resp) => return Ok(Self::extract_response(resp, url, "POST").await),
                                            Err(e )=> return Err(Error::new(ErrorKind::Other,get_error!("post","Failed to get response from POST (TEXT): {}. Error: {}",url,e.to_string()))),
                                        }
                                    },
        }

    }

    async fn extract_response(resp: Response, url: &str, method: &str) -> HttpResponse{
        if resp.status().is_client_error() || resp.status().is_server_error() {
            log_error!("extract_response", "ERROR: Failed to get response from {}: {} Status Code: {}", method, url, resp.status());
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
