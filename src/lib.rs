/// Defines a HttpClient struct and its associated methods, which provides a simple and efficient way to make HTTP requests.
/// It includes methods to set custom headers and retrieve default headers, as well as handling cookies if needed.
/// It also defines an HttpResponse struct to represent the response from a HTTP request.

use std::{
    collections::HashMap, io::{Error, ErrorKind}, str::FromStr, sync::Arc
};

use bt_logger::{get_error, log_debug, log_error, log_verbose};
use reqwest::{
    cookie::Jar,
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response, StatusCode,
};

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
    pub fn new(use_hickory_dns: bool, use_cookies: bool) -> Self {
        let c = if use_cookies {
            let cookie_store = Arc::new(Jar::default());
            Client::builder()
                .cookie_store(true)
                .cookie_provider(cookie_store.clone())
                .hickory_dns(use_hickory_dns)
                .build()
                .unwrap()
        } else {
            Client::builder()
                .cookie_store(false)
                .hickory_dns(use_hickory_dns)
                .build()
                .unwrap()
        };
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
        Self::convert_headers(&self.headers)
    }

    ///Helper Method: Merge current/default headers with extra headers
    fn get_extra_headers(&self, extra_headers: Option<HashMap<&str, &str>>) -> HeaderMap {
        let mut local_headers = self.headers.clone();
        if let Some(new_headers) = extra_headers {
            // Add headers from HashMap into the existing HeaderMap
            for (key, value) in new_headers {
                local_headers.insert(
                    HeaderName::from_str(&key).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                );
            }
        }

        local_headers
    }

///Method: get
///The get method is used to make a GET request to a specific URL
///It takes two parameters: url and extra_headers. If extra_headers is Some, it adds the headers to the existing headers in the client. 
/// The method returns an HttpResponse instance containing the response from the GET request. 
    pub async fn get(
        &self,
        url: &str,
        extra_headers: Option<HashMap<&str, &str>>,
    ) -> Result<HttpResponse, Error> {
        let local_headers = self.get_extra_headers(extra_headers);

        match self.client.get(url).headers(local_headers).send().await {
            Ok(resp) => return Ok(Self::extract_response(resp, url, "GET").await),
            Err(e) => {
                return Err(Error::new( ErrorKind::Other, get_error!( "get", "Failed to get response from GET: {}. Error: {}", url, e.to_string() ), ))
            }
        }
    }

///Method: post
///The post method is used to make a POST request to a specific URL
///It takes four parameters: url, extra_headers, body_request, and content_type. 
/// The method returns an HttpResponse instance containing the response from the POST request. 
    pub async fn post( &self, url: &str, extra_headers: Option<HashMap<&str, &str>>, body_request: &str, content_type: ContentType, ) -> Result<HttpResponse, Error> {
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
                return Err(Error::new(
                    ErrorKind::Other,
                    get_error!( "post", "Failed to get response from POST (TEXT): {}. Error: {}", url, e.to_string() ),
                ))
            }
        }
    }

///Method: request
/// The request method is used to make a request to a specific URL using a specific HTTP method: currently tested, get, post, put, delete, patch, delete
/// It takes six parameters: request_method, url_with_ep_path (URL with endpoint: path, path parameters), extra_headers, body_params, query_params, and content_type. 
/// The method returns an HttpResponse instance containing the response from the request.
    pub async fn request( &self, request_method: &str, url_with_ep_path: &str, extra_headers: Option<HashMap<&str, &str>>, body_params: Option<HashMap<String, String>>, 
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
                    log_debug!("request","Path parameter '{:?}' not provided. Parameter will be used as Query parameter", &path_param.0);
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

        if !url.ends_with('/') && qry_params.len() > 0 {
            url = format!("{}{}",url,"/");
        }

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
                                log_verbose!("request","TEXT: Body {}",&body_data);
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
                return Err(get_error!( "request", "Failed to get response from {} ({:?}): {}. Error: {}", &method, content_type, url, e.to_string())
                                    .into())
            }
        }
    }

 ///Helper Method: extract_response
 /// The extract_response method is used to extract the response from a Response instance
 /// It takes three parameters: resp, url, and method. The method returns an HttpResponse instance containing the response from the request.
    async fn extract_response(resp: Response, url: &str, method: &str) -> HttpResponse {
        if resp.status().is_client_error() || resp.status().is_server_error() {
            log_error!( "extract_response", "ERROR: Failed to get response from {}: {} Status Code: {}", method, url, resp.status() );
            return HttpResponse {
                status_code: resp.status().as_u16(),
                header: Self::convert_headers(resp.headers()),
                body: format!( "ERROR: Failed to get response from {}:{} -Error: {}", method, url, resp.status().canonical_reason().unwrap_or("UNKNOWN ERROR!") ),
            };
        } else {
            return HttpResponse {
                status_code: resp.status().as_u16(),
                header: Self::convert_headers(resp.headers()),
                body: resp.text().await.expect(
                    get_error!(
                        "extract_response",
                        "ERROR: Failed to get payload from {}:{}",
                        method,
                        url
                    )
                    .as_str(),
                ),
            };
        }
    }

    ///Helper Method convert_headers: A private method to convert HeaderMap to HashMap suitable for public use.
    fn convert_headers(headers: &HeaderMap) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect()
    }
}

impl HttpResponse {
///The is_error method is used to check if the response is an error:    
    pub fn is_error(&self) -> bool {
        let sc = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::FORBIDDEN);
        sc.is_client_error() || sc.is_server_error()
    }
}
