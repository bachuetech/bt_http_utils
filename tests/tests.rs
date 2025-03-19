#[cfg(test)]
mod http_utils_tests {
use std::collections::HashMap;

use bt_http_utils::{self, ContentType, HttpClient, DANGER_ACCEPT_INVALID_CERTS, DANGER_ACCEPT_INVALID_HOSTNAMES};
use bt_logger::{build_logger, LogLevel, LogTarget};

#[cfg(test)]
const SERVER: &str = "://localhost";


#[tokio::test]
async fn test_request_err_notfound_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/uh/api.php","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().status_code,404);
}

#[tokio::test]
async fn test_request_err_missparam_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/uh/api.php/posts/","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("post",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().status_code,400);
}

#[tokio::test]
async fn test_request_post_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/uh/api.php/users/","http",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "C.Brown".to_string());
    let test_content = "{\"id\":3,\"name\":\"C.Brown\"}";

    let http_client = HttpClient::new(false, true, None);

    let resp = http_client.request("post",&url, None, Some(param), None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}


#[tokio::test]
async fn test_request_post_all(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/test/param.php/{{name}}/","http",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "omega".to_string());
    param.insert("lastname".to_string(),"alpha".to_string());
    let mut bodyp: HashMap<String, String> = HashMap::new();
    bodyp.insert("age".to_string(), "25".to_string());
    let mut headerp: HashMap<String, String> = HashMap::new();
    headerp.insert("key".to_string(), "Key1".to_string());

    let test_content = "Key: Key1, Name: omega, Last Name: No last name provided, Age: 25";

    let http_client = HttpClient::new(false, true, None);

    let resp = http_client.request("post",&url, Some(headerp), Some(bodyp), Some(param), ContentType::TEXT).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_all(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/test/param.php/{{name}}/","http",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "omega".to_string());
    param.insert("lastname".to_string(),"alpha".to_string());
    let mut bodyp: HashMap<String, String> = HashMap::new();
    bodyp.insert("age".to_string(), "25".to_string());
    let mut headerp: HashMap<String, String> = HashMap::new();
    headerp.insert("key".to_string(), "Key1".to_string());

    let test_content = "Key: Key1, Name: omega, Last Name: alpha, Age: No age provided";

    let http_client = HttpClient::new(false, true, None);

    let resp = http_client.request("get",&url, Some(headerp), Some(bodyp), Some(param), ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_sec(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let url = "https://www.bachuetech.biz/";
    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, None, ContentType::TEXT).await;
    println!("Body: {:?}",&resp);
    assert!(resp.is_ok());
    assert!(resp.unwrap().body.len() > 0);
}

#[tokio::test]
async fn test_request_post_text_no_hickory_sec(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    //env_logger::init();
    //tracing_subscriber::fmt::init();

    let url = format!("{}{}:8081/uh/apitxt1.php/users/","https",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "John".to_string());
    param.insert("last_name".to_string(), "Smith".to_string());
    let test_content = "John Smith";
    let dar = vec![(DANGER_ACCEPT_INVALID_HOSTNAMES.to_string() , true)];

    let http_client = HttpClient::new(false, true, Some(dar));

    let resp = http_client.request("post",&url, None, Some(param), None, ContentType::TEXT).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_post_text_no_hickory_sec_invalid(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    //env_logger::init();
    //tracing_subscriber::fmt::init();

    let url = format!("{}{}:8081/uh/apitxt1.php/users/","https",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "John".to_string());
    param.insert("last_name".to_string(), "Smith".to_string());
    let test_content = "John Smith";
    let dar = vec![(DANGER_ACCEPT_INVALID_CERTS.to_string(), true), ("invalid_key".to_string(),true)];

    let http_client = HttpClient::new(false, true, Some(dar));

    let resp = http_client.request("post",&url, None, Some(param), None, ContentType::TEXT).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_post_text_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let url = format!("{}{}/uh/apitxt1.php/users/","http",SERVER);

    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("name".to_string(), "John".to_string());
    param.insert("last_name".to_string(), "Smith".to_string());
    let test_content = "John Smith";

    let http_client = HttpClient::new(false, true, None);

    let resp = http_client.request("post",&url, None, Some(param), None, ContentType::TEXT).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_delete_direct_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "{\"status\":\"success\",\"message\":\"User deleted\"}";
    let url = format!("{}{}/uh/api.php/delete/1","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("delete",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_delete_qry_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "{\"status\":\"success\",\"message\":\"User deleted\"}";
    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("userid".to_string(), "1".to_string());
    let url = format!("{}{}/uh/api.php/delete/{{userid}}","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("delete",&url, None, None, Some(param), ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_noparam_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "[{\"id\":1,\"userId\":1},{\"id\":2,\"userId\":2}]";
    let url = format!("{}{}/uh/api.php/get","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_put_with_direct_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "[{\"id\":3,\"userId\":2,\"title\":\"Write Documentation\",\"completed\":false}]";
    let url = format!("{}{}/uh/api.php/todos/2","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("put",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_patch_with_direct_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "[{\"id\":3,\"userId\":2,\"title\":\"Write Documentation\",\"completed\":false}]";
    let url = format!("{}{}/uh/api.php/todos/2","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("patch",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_with_direct_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "{\"id\":2,\"userId\":2}";
    let url = format!("{}{}/uh/api.php/get/2","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, None, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_qry_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "{\"id\":2,\"userId\":2}";
    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("id".to_string(), "2".to_string());

    let url = format!("{}{}/uh/api.php/get/{{id}}","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, Some(param), ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_request_get_extra_qry_param_no_hickory(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "{\"id\":2,\"userId\":2}";
    let mut param: HashMap<String, String> = HashMap::new();
    param.insert("id".to_string(), "2".to_string());
    param.insert("qry".to_string(), "Extra".to_string());

    let url = format!("{}{}/uh/api.php/get/{{id}}","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.request("get",&url, None, None, Some(param), ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_no_hickory(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World! to Bachuetech!";
    let url = format!("{}{}/test_get.php?name=Bachuetech","http",SERVER);

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}


#[tokio::test]
async fn test_plain_get_no_hickory_new_header(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}{}/test_get.html","http",SERVER);
    let mut extra: HashMap<String, String> = HashMap::new();
    extra.insert("btai_session_id".to_string(), "A_12dkk3dsd".to_string());

    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}


#[tokio::test]
async fn test_plain_get_hickory_new_header(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World! BachueTech!";
    let url = format!("{}{}/test_get.php","http",SERVER);
    let mut extra: HashMap<String, String> = HashMap::new();
    extra.insert("Last-Name".to_string(), "BachueTech".to_string());

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_hickory_new_header_chk(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "hello BachueTech";
    let url = format!("{}{}/test_get.php","http",SERVER);
    let mut extra: HashMap<String, String> = HashMap::new();
    extra.insert("Last-Name".to_string(), "BachueTech".to_string());

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().header.get("answer").unwrap(),test_content);
}

#[tokio::test]
async fn test_plain_get_hickory(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}{}/test_get.html","http",SERVER);

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_dns_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.get("http://http://www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.is_err(),true);
}

#[tokio::test]
async fn test_plain_get_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, true, None);
    let resp = http_client.get("http:/www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[tokio::test]
async fn test_json_post_hickory(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}{}/test_post.php","http",SERVER);

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.post(&url, None, body, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}{}/test_post.php","http",SERVER);

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.post(&url, None, body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory_extra_headers(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}{}/test_post.php","http",SERVER);

    let mut extra: HashMap<String, String> = HashMap::new();
    extra.insert("btai_session_id".to_string(), "A_12dkk3dsd".to_string());

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.post(&url, Some(extra), body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_json_post_hickory_fail(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );

    let url = format!("{}{}/test_post_fake.php","http",SERVER);

    let http_client = HttpClient::new(true, true, None);
    let resp = http_client.post(&url, None, "", ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[test]
fn test_set_headers(){
    let header_val =  "HEADER_VALUE";
    let header_name = "bt_header";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, true, None);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

#[test]
fn test_change_headers(){
    let header_val =  "HEADER_VALUE";
    let header_name = "user-agent";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, true, None);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

//NO COOKIES TESTS
#[tokio::test]
async fn test_plain_get_no_hickory_nc(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}{}/test_get.html","http",SERVER);

    let http_client = HttpClient::new(false, false, None);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_hickory_nc(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}{}/test_get.html","http",SERVER);

    let http_client = HttpClient::new(true, false, None);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_dns_fail_nc(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, false, None);
    let resp = http_client.get("http://http://www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.is_err(),true);
}

#[tokio::test]
async fn test_plain_get_fail_nc(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, false, None);
    let resp = http_client.get("http:/www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[tokio::test]
async fn test_json_post_hickory_nc(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}{}/test_post.php","http",SERVER);

    let http_client = HttpClient::new(true, false, None);
    let resp = http_client.post(&url, None, body, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory_nc(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}{}/test_post.php","http",SERVER);

    let http_client = HttpClient::new(true, false, None);
    let resp = http_client.post(&url, None, body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_json_post_hickory_fail_nc(){
    //const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );

    let url = format!("{}{}/test_post_fake.php","http",SERVER);

    let http_client = HttpClient::new(true, false, None);
    let resp = http_client.post(&url, None, "", ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[test]
fn test_set_headers_nc(){
    let header_val =  "HEADER_VALUE";
    let header_name = "bt_header";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, false, None);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

#[test]
fn test_change_headers_nc(){
    let header_val =  "HEADER_VALUE";
    let header_name = "user-agent";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, false, None);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}
}