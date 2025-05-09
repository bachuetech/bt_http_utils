/// Defines a HttpClient struct and its associated methods, which provides a simple and efficient way to make HTTP requests.
/// It includes methods to set custom headers and retrieve default headers, as well as handling cookies if needed.
/// It also defines an HttpResponse struct to represent the response from a HTTP request.
mod ext_certs;
pub mod stream_response;

pub const DANGER_ACCEPT_INVALID_HOSTNAMES: &str = "danger_accept_invalid_hostnames";
pub const DANGER_ACCEPT_INVALID_CERTS: &str = "danger_accept_invalid_certs";

use std::{
    collections::HashMap, str::FromStr, sync::Arc
};

use bt_logger::{get_error, log_error, log_verbose, log_warning};
use ext_certs::get_local_certificates;
use reqwest::{
    cookie::Jar, header::{self, HeaderMap, HeaderName, HeaderValue}, Client, Method, Response, StatusCode
};
use stream_response::HttpStreamResponse;

///HttpClient:
///client: A Client instance from the reqwest crate for making HTTP requests.
///headers: A HeaderMap to store custom headers.
pub struct HttpClient {
    client: Client,
    headers: HeaderMap,
}

///HttpResponse: Represents the response from an HTTP request.
/// status_code: The status code of the HTTP response.
/// header: A HashMap containing the headers from the response.
/// body: The body content of the HTTP response as a string.
#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub header: HashMap<String, String>,
    pub body: String,
    pub remote_address: String
}


///ContentType: An enum to specify the content type of the request or response. Currently supports JSON and TEXT.
#[derive(Debug)]
pub enum ContentType {
    JSON,
    TEXT,
}

impl HttpClient {
    ///Constructor new: 
    /// The new method is used to create a new instance of the HttpClient struct
    /// It takes two boolean parameters: use_hickory_dns and use_cookies.
    /// If use_cookies is true, it enables cookie support in the client. It creates a cookie store using Arc, sets the client to use cookies with a default custom user agent.
    /// If use_cookies is false, it builds a client without cookie support but still sets a default user agent.
    /// If use_hickory_dns is true, it enables Hickory DNS resolution in the client.
    /// danger_accept_invalid: If true removes any validation to digital certificates. Useful with some self-signed certificate sites or when hostname doesn't match the certificate.
    ///                         Possible values: const DANGER_ACCEPT_INVALID_HOSTNAMES: &str = "danger_accept_invalid_hostnames" OR
    ///                                          const DANGER_ACCEPT_INVALID_CERTS: &str = "danger_accept_invalid_certs" OR
    pub fn new(use_hickory_dns: bool, use_cookies: bool, danger_accept_invalid: Option<Vec<(String,bool)>>) -> Self {
        let tls_conn = get_local_certificates(danger_accept_invalid);
        let mut cb = Client::builder();

        if use_cookies {
            let cookie_store = Arc::new(Jar::default());
            if let Some (reqwest_tc) = tls_conn{
                cb = cb
                    .use_native_tls()
                    .use_preconfigured_tls(reqwest_tc);
            }
                cb = cb.cookie_provider(cookie_store.clone())
                /*.hickory_dns(use_hickory_dns)
                .build()
                .unwrap()*/
        } else {
            if let Some (reqwest_tc) = tls_conn{
                cb = cb
                    .use_native_tls()
                    .use_preconfigured_tls(reqwest_tc);
            }
            cb = cb.cookie_store(false)
                /*.hickory_dns(use_hickory_dns)
                .build()
                .unwrap()*/
        };

        let c = cb
        .connection_verbose(true)
        //.danger_accept_invalid_certs(true)
        //.danger_accept_invalid_hostnames(true)
        .hickory_dns(use_hickory_dns)
        .build()
        .unwrap();

        let mut h = HeaderMap::new();
        h.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (compatible; BachueTech/1.0)"),
        );

        Self {
            client: c,
            headers: h,
        }
    }

    ///Method set_header: Allows adding custom headers to the HTTP client dynamically.
    pub fn set_header(&mut self, header_name: &str, header_value: &str) {
        self.headers.insert(
            HeaderName::from_str(&header_name).unwrap(),
            HeaderValue::from_str(&header_value).unwrap(),
        );
    }

    ///Method get_default_headers: Converts the internal HeaderMap to a HashMap for easy access and manipulation.
    pub fn get_default_headers(&self) -> HashMap<String, String> {
        convert_headers(&self.headers)
    }

    ///Helper Method: Merge current/default headers with extra headers
    //fn get_extra_headers(&self, extra_headers: Option<HashMap<&str, &str>>) -> HeaderMap {
    fn get_extra_headers(&self, extra_headers: Option<HashMap<String, String>>) -> HeaderMap {
        let mut local_headers = self.headers.clone();
        if let Some(new_headers) = extra_headers {
            // Add headers from HashMap into the existing HeaderMap
            for (key, value) in new_headers {
                local_headers.insert(
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(&value).unwrap(),
                );
            }
        }

        local_headers
    }

///Method: get
///The get method is used to make a GET request to a specific URL
///It takes two parameters: url and extra_headers. If extra_headers is Some, it adds the headers to the existing headers in the client. 
/// The method returns an HttpResponse instance containing the response from the GET request. 
//    pub async fn get( &self, url: &str, extra_headers: Option<HashMap<&str, &str>>, ) -> Result<HttpResponse, Error> {
    pub async fn get( &self, url: &str, extra_headers: Option<HashMap<String, String>>, ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let local_headers = self.get_extra_headers(extra_headers);
        match self.client.get(url).headers(local_headers).send().await {
            Ok(resp) => return Ok(Self::extract_response(resp, url, "GET").await),
            Err(e) => {
                return Err(get_error!( "get", "Failed to get response from GET: {}. Error: {}", url, e).into())
            }
        }
    }

///Method: post
///The post method is used to make a POST request to a specific URL
///It takes four parameters: url, extra_headers, body_request, and content_type. 
/// The method returns an HttpResponse instance containing the response from the POST request. 
//    pub async fn post( &self, url: &str, extra_headers: Option<HashMap<&str, &str>>, body_request: &str, content_type: ContentType, ) -> Result<HttpResponse, Error> {
    pub async fn post( &self, url: &str, extra_headers: Option<HashMap<String, String>>, body_request: &str, content_type: ContentType, ) 
                        -> Result<HttpResponse,  Box<dyn std::error::Error>> {
        //log_verbose!("post", "Getting {} with payload: {}", url, body_request);
        let mut local_headers = self.get_extra_headers(extra_headers); //self.headers.clone();
        match content_type {
            ContentType::JSON => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
            }
            ContentType::TEXT => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/text").unwrap(),
                );
            }
        }

        match self
            .client
            .post(url)
            .headers(local_headers)
            .body(body_request.to_string())
            .send()
            .await
        {
            Ok(resp) => return Ok(Self::extract_response(resp, url, "POST").await),
            Err(e) => {
                return Err(get_error!( "post", "Failed to get response from POST ({:?}): {}. Error: {}", content_type, url, e ).into() )
            }
        }
    }

    pub async fn post_stream( &self, url: &str, extra_headers: Option<HashMap<String, String>>, body_request: &str, content_type: ContentType, ) -> Result<HttpStreamResponse,  Box<dyn std::error::Error>> {
        //log_verbose!("post", "Getting {} with payload: {}", url, body_request);
        let mut local_headers = self.get_extra_headers(extra_headers); //self.headers.clone();
        match content_type {
            ContentType::JSON => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
            }
            ContentType::TEXT => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/text").unwrap(),
                );
            }
        }

        match self
            .client
            .post(url)
            .headers(local_headers)
            .body(body_request.to_string())
            .send()
            .await
        {
            Ok(resp) => return Ok(HttpStreamResponse::new(resp)),
            Err(e) => {
                return Err(get_error!( "post_stream", "Failed to get stream response from POST ({:?}): {}. Error: {}", content_type, url, e ).into() )
            }
        }
    }

///Method: request
/// The request method is used to make a request to a specific URL using a specific HTTP method: currently tested, get, post, put, delete, patch, delete
/// It takes six parameters: request_method, url_with_ep_path (URL with endpoint: path, path parameters), extra_headers, body_params, query_params, and content_type. 
/// The method returns an HttpResponse instance containing the response from the request.
//    pub async fn request( &self, request_method: &str, url_with_ep_path: &str, extra_headers: Option<HashMap<&str, &str>>, body_params: Option<HashMap<String, String>>, 
    pub async fn request( &self, request_method: &str, url_with_ep_path: &str, extra_headers: Option<HashMap<String, String>>, body_params: Option<HashMap<String, String>>, 
                        query_params: Option<HashMap<String, String>>, content_type: ContentType, ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let method = match request_method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            _ => return Err(get_error!("request", "Unsupported HTTP method: {}", &request_method).into()),
        };

        let mut url = url_with_ep_path.to_string();
        let mut qry_params: HashMap<String, String>;

        // Handle path parameters
        if let Some(path_params) = query_params {
            qry_params = path_params.clone();
            for path_param in path_params {
                if url.contains(&format!("{{{}}}", &path_param.0)) {
                    url = url.replace(&format!("{{{}}}", &path_param.0), &path_param.1);
                    qry_params.remove(&path_param.0); //Remove used path_param to use remaining params as query parameters
                } else {
                    log_verbose!("request","Path parameter '{:?}' not provided. Parameter will be used as Query parameter", &path_param.0);
                }
            }
        }else{
            qry_params = HashMap::new();
        }


        let mut local_headers = self.get_extra_headers(extra_headers); //self.headers.clone();

        match content_type {
            ContentType::JSON => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
            }
            ContentType::TEXT => {
                local_headers.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("application/text").unwrap(),
                );
            }
        }

        //Removed 03/28/25: Cause issues!
        //if !url.ends_with('/') && qry_params.len() > 0 {
        //    url = format!("{}{}",url,"/");
        //}

        let mut request = self.client.request(method.clone(), &url).headers(local_headers);
        if method == Method::GET{
            request = request.query(&qry_params); // Use remaining params as query parameters if any
        }else{
            if let Some(b_params) = body_params{
                    match content_type {
                        ContentType::JSON => request = request.json(&b_params),
                        _ => {let body_data = b_params
                                .iter()
                                .map(|(k, v)| format!("{}={}", k, v))
                                .collect::<Vec<String>>()
                                .join("&");
                                request = request.body(body_data)
                            }
                    }       
            }
        }

        match request
            .send()
            .await
        {
            Ok(resp) => return Ok(Self::extract_response(resp, &url, request_method.to_uppercase().as_str()).await),
            Err(e) => {
                return Err(get_error!( "request", "Failed to get response from {} ({:?}): {}. Error: {}", &method, content_type, url, e)
                                    .into())
            }
        }
    }

 ///Helper Method: extract_response
 /// The extract_response method is used to extract the response from a Response instance
 /// It takes three parameters: resp, url, and method. The method returns an HttpResponse instance containing the response from the request.
    async fn extract_response(mut resp: Response, url: &str, method: &str) -> HttpResponse {
        let ra = match resp.remote_addr() {
            Some(ip) => ip.ip().to_string(),
            None => {
                log_warning!("extract_response", "Remote Address not found. Using default 0.0.0.0");
                "0.0.0.0".to_owned()
            },
        };

        if resp.status().is_client_error() || resp.status().is_server_error() {
            log_error!( "extract_response", "ERROR: Failed to get response from {}: {} Status Code: {}", method, url, resp.status() );
            return HttpResponse {
                status_code: resp.status().as_u16(),
                header: convert_headers(resp.headers()),
                body: format!( "ERROR: Failed to get response from {}:{} -Error: {}", method, url, resp.status().canonical_reason().unwrap_or("UNKNOWN ERROR!") ),
                remote_address: ra
            };
        } else {
            let mut full_body = String::new();
            let mut error_count = 0;
            let rstatus = resp.status().as_u16();
            let rheader = convert_headers(resp.headers());

            if resp.status().is_success() {
                let mut read_resp: bool = true;
                // Process the response body as it's being streamed
                while read_resp {
                    match resp.chunk().await { 
                        Ok(r) => {
                            match r{
                                Some(chunk) => full_body.push_str(&String::from_utf8_lossy(&chunk)),
                                None => read_resp = false,
                            }
                        },
                        Err(e) => {
                            if error_count > 3{
                                log_error!("extract_response","Too many errors (>3 times) reading answer body. Stop Executing and return what was collected. Error {}",e);
                                return HttpResponse {
                                    status_code: resp.status().as_u16(),
                                    header: convert_headers(resp.headers()),
                                    body: resp.text().await.expect(full_body.as_str() ),
                                        //get_error!("extract_response","ERROR: Failed to get payload from {}:{}",method,url)
                                        //    .as_str(),
                                        //),
                                    remote_address: ra,
                                };
                            }
                            error_count = error_count + 1;
                            log_error!("extract_response","Error reading answer body (error count={}). Error {}",error_count,e);                
                        },
                    }
                }
            }else{
                full_body = resp.text().await.expect(
                        get_error!("extract_response","ERROR: Failed to get payload when status = {} from {}:{}",rstatus, method,url)
                        .as_str(),
                    );
            }
            return HttpResponse {
                status_code: rstatus, // resp.status().as_u16(),
                header: rheader, //Self::convert_headers(resp.headers()),
                //body: resp.text().await.expect(
                //    get_error!("extract_response","ERROR: Failed to get payload from {}:{}",method,url)
                //    .as_str(),
                //),
                body: full_body,
                remote_address: ra,
            };
        }
    }


}

    ///Helper Method convert_headers: A private method to convert HeaderMap to HashMap.
    fn convert_headers(headers: &HeaderMap) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect()
    }
    
impl HttpResponse {
///The is_error method is used to check if the response is an error:    
    pub fn is_error(&self) -> bool {
        let sc = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::FORBIDDEN);
        sc.is_client_error() || sc.is_server_error()
    }
}
