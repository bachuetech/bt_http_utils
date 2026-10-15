//! Defines a HttpClient struct and its associated methods, which provides a simple and efficient way to make HTTP requests.
//! It includes methods to set custom headers and retrieve default headers, as well as handling cookies if needed.
//! It also defines an HttpResponse struct to represent the response from a HTTP request.

mod ext_certs;
pub mod stream_response;

pub const DANGER_ACCEPT_INVALID_HOSTNAMES: &str = "danger_accept_invalid_hostnames";
pub const DANGER_ACCEPT_INVALID_CERTS: &str = "danger_accept_invalid_certs";

use std::{collections::HashMap, sync::Arc};
use bt_any_error::any_err::AnyErr;
use bt_logger::{get_error, log_error, log_warning};
use ext_certs::get_local_certificates;
use reqwest::{Client, Method, Response, StatusCode, Url, cookie::Jar, header::{self, HeaderMap, HeaderName, HeaderValue}};
use stream_response::HttpStreamResponse;

///HttpClient:
///client: A Client instance from the reqwest crate for making HTTP requests.
///headers: A HeaderMap to store custom headers.
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    headers: Arc<HeaderMap>,
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
    pub remote_address: String,
}

///ContentType: An enum to specify the content type of the request or response. Currently supports JSON and TEXT.
#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    Json,
    Text,
}

impl ContentType {
    #[inline]
    fn as_str(&self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Text => "text/plain",
        }
    }
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
    pub fn new(use_hickory_dns: bool, use_cookies: bool, danger_accept_invalid: Option<Vec<(String, bool)>>) -> Self {
        let tls_conn = get_local_certificates(danger_accept_invalid);
        let mut cb = Client::builder()
            .connection_verbose(true)
            .hickory_dns(use_hickory_dns)
            .pool_max_idle_per_host(32)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .http2_keep_alive_interval(std::time::Duration::from_secs(30))
            .http2_keep_alive_timeout(std::time::Duration::from_secs(10));

        if let Some(tc) = tls_conn {
            cb = cb.use_native_tls().use_preconfigured_tls(tc);
        }

        if use_cookies {
            cb = cb.cookie_provider(Arc::new(Jar::default()));
        } else {
            cb = cb.cookie_store(false);
        }

        let client = cb.build().expect("Failed to build HTTP client");

        let mut headers = HeaderMap::with_capacity(4);
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (compatible; BachueTech/1.0)"),
        );

        Self {
            client,
            headers: Arc::new(headers),
        }
    }

    /// set_header: Allows adding custom headers to the HTTP client dynamically.    
    #[inline]
    pub fn set_header(&mut self, name: &str, value: &str) {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value)
        ) {
            Arc::make_mut(&mut self.headers).append(n, v);
        }
    }

    #[inline]
    pub fn set_header_replace(&mut self, name: &str, value: &str) {
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value)
        ) {
            Arc::make_mut(&mut self.headers).insert(n, v);
        }
    }

    #[inline]
    pub fn remove_header(&mut self, name: &str) {
        if let Ok(n) = HeaderName::from_bytes(name.as_bytes()) {
            Arc::make_mut(&mut self.headers).remove(&n);
        }
    }

    #[inline]
    pub fn get_header_values(&self, name: &str) -> Vec<String> {
        HeaderName::from_bytes(name.as_bytes())
            .ok()
            .map(|n| {
                self.headers
                    .get_all(&n)
                    .iter()
                    .filter_map(|v| v.to_str().ok().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// get_default_headers: Converts the internal HeaderMap to a HashMap for easy access and manipulation.    
    #[inline]
    pub fn get_default_headers(&self) -> HashMap<String, String> {
        convert_headers(&self.headers)
    }

    ///Helper Method: Merge current/default headers with extra headers
    //fn get_extra_headers(&self, extra_headers: Option<HashMap<&str, &str>>) -> HeaderMap {    
    #[inline]
    fn build_headers(&self, extra: Option<HashMap<String, String>>, content_type: Option<ContentType>) -> HeaderMap {
        let mut headers = (*self.headers).clone();
        
        if let Some(ct) = content_type {
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(ct.as_str()));
        }
        
        if let Some(extra) = extra {
            for (k, v) in extra {
                if let (Ok(name), Ok(value)) = (
                    HeaderName::from_bytes(k.as_bytes()), 
                    HeaderValue::from_str(&v)
                ) {
                    headers.append(name, value);
                }
            }
        }
        
        headers
    }
    /// get
    ///The get method is used to make a GET request to a specific URL
    ///It takes two parameters: url and extra_headers. If extra_headers is Some, it adds the headers to the existing headers in the client. 
    /// The method returns an HttpResponse instance containing the response from the GET request.     
    pub async fn get(&self, url: &str, extra_headers: Option<HashMap<String, String>>) -> Result<HttpResponse, AnyErr> {
        let headers = self.build_headers(extra_headers, None);
        let resp = self.client.get(url).headers(headers).send().await?;
        Ok(Self::extract_response(resp, url, "GET").await)
    }

    /// post
    ///The post method is used to make a POST request to a specific URL
    ///It takes four parameters: url, extra_headers, body_request, and content_type. 
    /// The method returns an HttpResponse instance containing the response from the POST request.     
    pub async fn post(&self, url: &str, extra_headers: Option<HashMap<String, String>>, body: &str, content_type: ContentType) -> Result<HttpResponse, AnyErr> {
        let headers = self.build_headers(extra_headers, Some(content_type));
        let resp = self.client.post(url).headers(headers).body(body.to_owned()).send().await?;
        Ok(Self::extract_response(resp, url, "POST").await)
    }

    pub async fn post_stream(&self, url: &str, extra_headers: Option<HashMap<String, String>>, body: &str, content_type: ContentType) -> Result<HttpStreamResponse, AnyErr> {
        let headers = self.build_headers(extra_headers, Some(content_type));
        let resp = self.client.post(url).headers(headers).body(body.to_owned()).send().await?;
        Ok(HttpStreamResponse::new(resp))
    }

    /// request
    /// The request method is used to make a request to a specific URL using a specific HTTP method: currently tested, get, post, put, delete, patch, delete
    /// It takes six parameters: request_method, url_with_ep_path (URL with endpoint: path, path parameters), extra_headers, body_params, query_params, and content_type. 
    /// The method returns an HttpResponse instance containing the response from the request.
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        extra_headers: Option<HashMap<String, String>>,
        body_params: Option<HashMap<String, String>>,
        query_params: Option<HashMap<String, String>>,
        content_type: ContentType,
    ) -> Result<HttpResponse, AnyErr> {
        let method = Method::from_bytes(method.to_uppercase().as_bytes())
            .map_err(|_| get_error!("request", "Unsupported HTTP method: {}", method))?;

        let url = Self::build_url(url, query_params)?;
        let url_str = url.as_str().to_owned();        
        let headers = self.build_headers(extra_headers, Some(content_type));

        let mut req = self.client.request(method.clone(), url).headers(headers);

        if method != Method::GET && let Some(params) = body_params {
            req = match content_type {
                ContentType::Json => req.json(&params),
                ContentType::Text => {
                    let body: String = params.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("&");
                    req.body(body)
                }
            };
        }


        let resp = req.send().await?;
        Ok(Self::extract_response(resp, &url_str, method.as_str()).await)
    }

    fn build_url(base: &str, params: Option<HashMap<String, String>>) -> Result<Url, AnyErr> {
        let mut url = base.to_owned();
        let mut query_params: Vec<(String, String)> = Vec::new();

        if let Some(params) = params {
            for (k, v) in params {
                let placeholder = format!("{{{}}}", k);
                if url.contains(&placeholder) {
                    url = url.replacen(&placeholder, &v, 1);
                } else {
                    query_params.push((k, v));
                }
            }
        }

        let parsed = if query_params.is_empty() {
            Url::parse(&url)?
        } else {
            Url::parse_with_params(&url, &query_params)?
        };

        Ok(parsed)
    }

    ///Helper Method: extract_response
    /// The extract_response method is used to extract the response from a Response instance
    /// It takes three parameters: resp, url, and method. The method returns an HttpResponse instance containing the response from the request.    
    async fn extract_response(resp: Response, url: &str, method: &str) -> HttpResponse {
        let status = resp.status();
        let status_code = status.as_u16();
        let remote_address = resp.remote_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_else(|| {
                log_warning!("", "Remote Address not found, using 0.0.0.0");
                "0.0.0.0".into()
            });
        let headers = convert_headers(resp.headers());

        if status.is_client_error() || status.is_server_error() {
            log_error!("", "ERROR: {} {} failed with status {}", method, url, status);
            return HttpResponse {
                status_code,
                header: headers,
                body: format!("ERROR: {} {} - {}", method, url, status.canonical_reason().unwrap_or("UNKNOWN")),
                remote_address,
            };
        }

        let body = resp.text().await.unwrap_or_else(|e| {
            log_error!("", "Failed to read body from {}:{} - {}", method, url, e);
            String::new()
        });

        HttpResponse {
            status_code,
            header: headers,
            body,
            remote_address,
        }
    }
}

///Helper Method convert_headers: A private method to convert HeaderMap to HashMap.
#[inline]
fn convert_headers(headers: &HeaderMap) -> HashMap<String, String> {
    headers.iter()
        .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or_default().to_owned()))
        .collect()
}

///The is_error method is used to check if the response is an error:  
impl HttpResponse {
    #[inline]
    pub fn is_error(&self) -> bool {
        let sc = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        sc.is_client_error() || sc.is_server_error()
    }
}

/************* */
/* unit tests */
/************ */

// ... existing code ...

#[cfg(test)]
mod http_utils_lib_tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_client() -> HttpClient {
        HttpClient::new(false, false, None)
    }

    mod http_client_tests {
        use super::*;

        #[test]
        fn test_set_header_appends() {
            let mut client = create_test_client();
            client.set_header("X-Custom", "value1");
            client.set_header("X-Custom", "value2");
            
            let values = client.get_header_values("x-custom");
            assert_eq!(values.len(), 2);
            assert!(values.contains(&"value1".to_string()));
            assert!(values.contains(&"value2".to_string()));
        }

        #[test]
        fn test_set_header_replace_replaces() {
            let mut client = create_test_client();
            client.set_header("X-Custom", "value1");
            client.set_header("X-Custom", "value2");
            client.set_header_replace("X-Custom", "final");
            
            let values = client.get_header_values("x-custom");
            assert_eq!(values.len(), 1);
            assert_eq!(values[0], "final");
        }

        #[test]
        fn test_header_case_insensitive() {
            let mut client = create_test_client();
            client.set_header("X-Custom-Header", "value1");
            client.set_header("x-custom-header", "value2");
            client.set_header("X-CUSTOM-HEADER", "value3");
            
            let values = client.get_header_values("X-Custom-Header");
            assert_eq!(values.len(), 3);
            
            let values_lower = client.get_header_values("x-custom-header");
            assert_eq!(values_lower.len(), 3);
            assert_eq!(values, values_lower);
        }

        #[test]
        fn test_get_header_values_case_variations() {
            let mut client = create_test_client();
            client.set_header("Content-Type", "application/json");
            
            assert_eq!(client.get_header_values("content-type").len(), 
                       client.get_header_values("Content-Type").len());
            assert_eq!(client.get_header_values("CONTENT-TYPE").len(), 
                       client.get_header_values("content-TYPE").len());
        }

        #[test]
        fn test_remove_header() {
            let mut client = create_test_client();
            client.set_header("X-Remove", "value1");
            client.set_header("X-Remove", "value2");
            
            assert_eq!(client.get_header_values("x-remove").len(), 2);
            
            client.remove_header("X-Remove");
            
            assert!(client.get_header_values("x-remove").is_empty());
            assert!(client.get_header_values("X-REMOVE").is_empty());
        }

        #[test]
        fn test_headers_preserved_across_requests() {
            let mut client = create_test_client();
            client.set_header("Authorization", "Bearer token123");
            client.set_header("X-Request-Id", "req-001");
            
            let headers1 = client.build_headers(None, None);
            let headers2 = client.build_headers(None, None);
            
            assert!(headers1.contains_key("authorization"));
            assert!(headers2.contains_key("authorization"));
            assert!(headers1.contains_key("x-request-id"));
            assert!(headers2.contains_key("x-request-id"));
        }

        #[test]
        fn test_new_client_no_cookies() {
            let client = HttpClient::new(false, false, None);
            assert!(client.get_default_headers().contains_key("user-agent"));
        }

        #[test]
        fn test_new_client_with_cookies() {
            let client = HttpClient::new(false, true, None);
            assert!(client.get_default_headers().contains_key("user-agent"));
        }

        #[test]
        fn test_new_client_with_hickory_dns() {
            let client = HttpClient::new(true, false, None);
            assert!(client.get_default_headers().contains_key("user-agent"));
        }

        #[test]
        fn test_set_header() {
            let mut client = create_test_client();
            client.set_header("X-Custom-Header", "test-value");
            
            let headers = client.get_default_headers();
            assert_eq!(headers.get("x-custom-header"), Some(&"test-value".to_string()));
        }

        #[test]
        fn test_set_multiple_headers() {
            let mut client = create_test_client();
            client.set_header("X-Header-1", "value1");
            client.set_header("X-Header-2", "value2");
            client.set_header("Authorization", "Bearer token123");
            
            let headers = client.get_default_headers();
            assert_eq!(headers.get("x-header-1"), Some(&"value1".to_string()));
            assert_eq!(headers.get("x-header-2"), Some(&"value2".to_string()));
            assert_eq!(headers.get("authorization"), Some(&"Bearer token123".to_string()));
        }

        #[test]
        fn test_get_default_headers_contains_user_agent() {
            let client = create_test_client();
            let headers = client.get_default_headers();
            
            assert!(headers.contains_key("user-agent"));
            assert!(headers.get("user-agent").unwrap().contains("BachueTech"));
        }

    }

    mod content_type_tests {
        use super::*;

        #[test]
        fn test_content_type_json_as_str() {
            assert_eq!(ContentType::Json.as_str(), "application/json");
        }

        #[test]
        fn test_content_type_text_as_str() {
            assert_eq!(ContentType::Text.as_str(), "text/plain");
        }

        #[test]
        fn test_content_type_clone() {
            let ct = ContentType::Json;
            let cloned = ct;
            assert_eq!(ct.as_str(), cloned.as_str());
        }

        #[test]
        fn test_content_type_debug() {
            let json = ContentType::Json;
            let text = ContentType::Text;
            assert_eq!(format!("{:?}", json), "Json");
            assert_eq!(format!("{:?}", text), "Text");
        }
    }

    mod http_response_tests {
        use super::*;

        fn create_response(status_code: u16) -> HttpResponse {
            HttpResponse {
                status_code,
                header: HashMap::new(),
                body: String::new(),
                remote_address: "127.0.0.1".to_string(),
            }
        }

        #[test]
        fn test_is_error_200_ok() {
            let resp = create_response(200);
            assert!(!resp.is_error());
        }

        #[test]
        fn test_is_error_201_created() {
            let resp = create_response(201);
            assert!(!resp.is_error());
        }

        #[test]
        fn test_is_error_204_no_content() {
            let resp = create_response(204);
            assert!(!resp.is_error());
        }

        #[test]
        fn test_is_error_301_redirect() {
            let resp = create_response(301);
            assert!(!resp.is_error());
        }

        #[test]
        fn test_is_error_400_bad_request() {
            let resp = create_response(400);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_401_unauthorized() {
            let resp = create_response(401);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_403_forbidden() {
            let resp = create_response(403);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_404_not_found() {
            let resp = create_response(404);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_500_internal_server_error() {
            let resp = create_response(500);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_502_bad_gateway() {
            let resp = create_response(502);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_503_service_unavailable() {
            let resp = create_response(503);
            assert!(resp.is_error());
        }

        #[test]
        fn test_is_error_invalid_status_code() {
            let resp = create_response(599);
            assert!(resp.is_error());
        }

        #[test]
        fn test_response_clone() {
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());
            
            let original = HttpResponse {
                status_code: 200,
                header: headers,
                body: "{\"key\":\"value\"}".to_string(),
                remote_address: "192.168.1.1".to_string(),
            };
            
            let cloned = original.clone();
            
            assert_eq!(original.status_code, cloned.status_code);
            assert_eq!(original.body, cloned.body);
            assert_eq!(original.remote_address, cloned.remote_address);
            assert_eq!(original.header, cloned.header);
        }
    }

    mod convert_headers_tests {
        use super::*;
        use reqwest::header::{HeaderMap, HeaderValue};


        
        #[test]
        fn test_convert_headers_empty() {
            let headers = HeaderMap::new();
            let result = convert_headers(&headers);
            assert!(result.is_empty());
        }

        #[test]
        fn test_convert_headers_single() {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", HeaderValue::from_static("application/json"));
            
            let result = convert_headers(&headers);
            
            assert_eq!(result.len(), 1);
            assert_eq!(result.get("content-type"), Some(&"application/json".to_string()));
        }

        #[test]
        fn test_convert_headers_multiple() {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", HeaderValue::from_static("text/html"));
            headers.insert("authorization", HeaderValue::from_static("Bearer token"));
            headers.insert("x-custom", HeaderValue::from_static("custom-value"));
            
            let result = convert_headers(&headers);
            
            assert_eq!(result.len(), 3);
            assert_eq!(result.get("content-type"), Some(&"text/html".to_string()));
            assert_eq!(result.get("authorization"), Some(&"Bearer token".to_string()));
            assert_eq!(result.get("x-custom"), Some(&"custom-value".to_string()));
        }
    }

    mod build_url_tests {
        use super::*;

        #[test]
        fn test_build_url_no_params() {
            let url = HttpClient::build_url("https://api.example.com/users", None).unwrap();
            assert_eq!(url.as_str(), "https://api.example.com/users");
        }

        #[test]
        fn test_build_url_with_path_param() {
            let mut params = HashMap::new();
            params.insert("id".to_string(), "123".to_string());
            
            let url = HttpClient::build_url("https://api.example.com/users/{id}", Some(params)).unwrap();
            assert_eq!(url.as_str(), "https://api.example.com/users/123");
        }

        #[test]
        fn test_build_url_with_multiple_path_params() {
            let mut params = HashMap::new();
            params.insert("user_id".to_string(), "42".to_string());
            params.insert("post_id".to_string(), "100".to_string());
            
            let url = HttpClient::build_url(
                "https://api.example.com/users/{user_id}/posts/{post_id}",
                Some(params)
            ).unwrap();
            
            assert_eq!(url.as_str(), "https://api.example.com/users/42/posts/100");
        }

        #[test]
        fn test_build_url_with_query_params() {
            let mut params = HashMap::new();
            params.insert("page".to_string(), "1".to_string());
            params.insert("limit".to_string(), "10".to_string());
            
            let url = HttpClient::build_url("https://api.example.com/users", Some(params)).unwrap();
            
            assert!(url.as_str().contains("page=1"));
            assert!(url.as_str().contains("limit=10"));
        }

        #[test]
        fn test_build_url_mixed_path_and_query_params() {
            let mut params = HashMap::new();
            params.insert("id".to_string(), "123".to_string());
            params.insert("expand".to_string(), "true".to_string());
            
            let url = HttpClient::build_url("https://api.example.com/users/{id}", Some(params)).unwrap();
            
            assert!(url.as_str().contains("/users/123"));
            assert!(url.as_str().contains("expand=true"));
        }

        #[test]
        fn test_build_url_empty_params() {
            let params = HashMap::new();
            let url = HttpClient::build_url("https://api.example.com/users", Some(params)).unwrap();
            assert_eq!(url.as_str(), "https://api.example.com/users");
        }

        #[test]
        fn test_build_url_invalid_url() {
            let result = HttpClient::build_url("not-a-valid-url", None);
            assert!(result.is_err());
        }

        #[test]
        fn test_build_url_special_characters_in_query() {
            let mut params = HashMap::new();
            params.insert("query".to_string(), "hello world".to_string());
            
            let url = HttpClient::build_url("https://api.example.com/search", Some(params)).unwrap();
            assert!(url.as_str().contains("query=hello"));
        }

        #[test]
        fn test_build_url_duplicate_placeholder_only_replaces_first() {
            let mut params = HashMap::new();
            params.insert("id".to_string(), "123".to_string());
            
            let url = HttpClient::build_url("https://api.example.com/{id}/items/{id}", Some(params)).unwrap();
            assert!(url.as_str().contains("123"));
        }
    }

    mod build_headers_tests {
        use super::*;
        #[test]
        fn test_build_headers_extra_appends() {
            let mut client = create_test_client();
            client.set_header("Accept", "text/html");
            
            let mut extra = HashMap::new();
            extra.insert("Accept".to_string(), "application/json".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            
            let accept_values: Vec<_> = headers
                .get_all("accept")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect();
            
            assert_eq!(accept_values.len(), 2);
            assert!(accept_values.contains(&"text/html"));
            assert!(accept_values.contains(&"application/json"));
        }

        #[test]
        fn test_build_headers_case_insensitive_append() {
            let mut client = create_test_client();
            client.set_header("X-Custom", "original");
            
            let mut extra = HashMap::new();
            extra.insert("x-custom".to_string(), "appended".to_string());
            extra.insert("X-CUSTOM".to_string(), "also-appended".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            
            let values: Vec<_> = headers
                .get_all("x-custom")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect();
            
            assert_eq!(values.len(), 3);
            assert!(values.contains(&"original"));
            assert!(values.contains(&"appended"));
            assert!(values.contains(&"also-appended"));
        }

        #[test]
        fn test_build_headers_content_type_replaces() {
            let mut client = create_test_client();
            client.set_header("Content-Type", "text/plain");
            
            let headers = client.build_headers(None, Some(ContentType::Json));
            
            let values: Vec<_> = headers
                .get_all("content-type")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect();
            
            assert!(values.contains(&"application/json"));
        }

        #[test]
        fn test_build_headers_invalid_headers_ignored() {
            let client = create_test_client();
            
            let mut extra = HashMap::new();
            extra.insert("Invalid Header Name!".to_string(), "value".to_string());
            extra.insert("X-Valid".to_string(), "valid".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            
            assert!(headers.contains_key("x-valid"));
            assert!(!headers.contains_key("invalid header name!"));
        }

        #[test]
        fn test_build_headers_multiple_same_name_in_extra() {
            let client = create_test_client();
            
            // HashMap doesn't allow duplicate keys, so test single value
            let mut extra = HashMap::new();
            extra.insert("X-Multi".to_string(), "value1".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            assert_eq!(headers.get_all("x-multi").iter().count(), 1);
        }

        #[test]
        fn test_get_all_returns_all_values() {
            let mut client = create_test_client();
            client.set_header("Set-Cookie", "session=abc");
            client.set_header("Set-Cookie", "user=xyz");
            client.set_header("Set-Cookie", "pref=123");
            
            let headers = client.build_headers(None, None);
            let cookies: Vec<_> = headers
                .get_all("set-cookie")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect();
            
            assert_eq!(cookies.len(), 3);
            assert!(cookies.contains(&"session=abc"));
            assert!(cookies.contains(&"user=xyz"));
            assert!(cookies.contains(&"pref=123"));
        }

        #[test]
        fn test_get_returns_first_value_only() {
            let mut client = create_test_client();
            client.set_header("X-Order", "first");
            client.set_header("X-Order", "second");
            client.set_header("X-Order", "third");
            
            let headers = client.build_headers(None, None);
            
            // get() returns first value
            assert_eq!(headers.get("x-order").unwrap().to_str().unwrap(), "first");
            
            // get_all() returns all
            assert_eq!(headers.get_all("x-order").iter().count(), 3);
        }

        #[test]
        fn test_build_headers_no_extra_no_content_type() {
            let client = create_test_client();
            let headers = client.build_headers(None, None);
            
            assert!(headers.contains_key(header::USER_AGENT));
            assert!(!headers.contains_key(header::CONTENT_TYPE));
        }

        #[test]
        fn test_build_headers_with_content_type_json() {
            let client = create_test_client();
            let headers = client.build_headers(None, Some(ContentType::Json));
            
            assert_eq!(
                headers.get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
                "application/json"
            );
        }

        #[test]
        fn test_build_headers_with_content_type_text() {
            let client = create_test_client();
            let headers = client.build_headers(None, Some(ContentType::Text));
            
            assert_eq!(
                headers.get(header::CONTENT_TYPE).unwrap().to_str().unwrap(),
                "text/plain"
            );
        }

        #[test]
        fn test_build_headers_with_extra_headers() {
            let client = create_test_client();
            let mut extra = HashMap::new();
            extra.insert("X-Custom".to_string(), "custom-value".to_string());
            extra.insert("Authorization".to_string(), "Bearer token".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            
            assert_eq!(headers.get("x-custom").unwrap().to_str().unwrap(), "custom-value");
            assert_eq!(headers.get("authorization").unwrap().to_str().unwrap(), "Bearer token");
        }

        #[test]
        fn test_build_headers_invalid_extra_header_ignored() {
            let client = create_test_client();
            let mut extra = HashMap::new();
            extra.insert("Invalid Header!".to_string(), "value".to_string());
            extra.insert("X-Valid".to_string(), "valid".to_string());
            
            let headers = client.build_headers(Some(extra), None);
            
            assert!(headers.contains_key("x-valid"));
            assert!(headers.contains_key(header::USER_AGENT));
        }

        #[test]
        fn test_build_headers_preserves_user_agent() {
            let client = create_test_client();
            let headers = client.build_headers(None, Some(ContentType::Json));
            
            let user_agent = headers.get(header::USER_AGENT).unwrap().to_str().unwrap();
            assert!(user_agent.contains("BachueTech"));
        }
    }
}