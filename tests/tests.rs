use std::collections::HashMap;

use bt_http_utils::{self, ContentType, HttpClient};
use bt_logger::{build_logger, LogLevel, LogTarget};



#[tokio::test]
async fn test_plain_get_no_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World! to Bachuetech!";
    let url = format!("{}/test_get.php?name=Bachuetech",SERVER);

    let http_client = HttpClient::new(false, true);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}


#[tokio::test]
async fn test_plain_get_no_hickory_new_header(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);
    let mut extra: HashMap<&str, &str> = HashMap::new();
    extra.insert("btai_session_id", "A_12dkk3dsd");

    let http_client = HttpClient::new(false, true);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}


#[tokio::test]
async fn test_plain_get_hickory_new_header(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World! BachueTech!";
    let url = format!("{}/test_get.php",SERVER);
    let mut extra: HashMap<&str, &str> = HashMap::new();
    extra.insert("Last-Name", "BachueTech");

    let http_client = HttpClient::new(true, true);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_hickory_new_header_chk(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "hello BachueTech";
    let url = format!("{}/test_get.php",SERVER);
    let mut extra: HashMap<&str, &str> = HashMap::new();
    extra.insert("Last-Name", "BachueTech");

    let http_client = HttpClient::new(true, true);
    let resp = http_client.get(&url, Some(extra)).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().header.get("answer").unwrap(),test_content);
}

#[tokio::test]
async fn test_plain_get_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);

    let http_client = HttpClient::new(true, true);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_dns_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, true);
    let resp = http_client.get("http://http://www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.is_err(),true);
}

#[tokio::test]
async fn test_plain_get_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, true);
    let resp = http_client.get("http:/www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[tokio::test]
async fn test_json_post_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}/test_post.php",SERVER);

    let http_client = HttpClient::new(true, true);
    let resp = http_client.post(&url, None, body, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}/test_post.php",SERVER);

    let http_client = HttpClient::new(true, true);
    let resp = http_client.post(&url, None, body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory_extra_headers(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}/test_post.php",SERVER);

    let mut extra: HashMap<&str, &str> = HashMap::new();
    extra.insert("btai_session_id", "A_12dkk3dsd");

    let http_client = HttpClient::new(true, true);
    let resp = http_client.post(&url, Some(extra), body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_json_post_hickory_fail(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );

    let url = format!("{}/test_post_fake.php",SERVER);

    let http_client = HttpClient::new(true, true);
    let resp = http_client.post(&url, None, "", ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[test]
fn test_set_headers(){
    let header_val =  "HEADER_VALUE";
    let header_name = "bt_header";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, true);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

#[test]
fn test_change_headers(){
    let header_val =  "HEADER_VALUE";
    let header_name = "user-agent";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, true);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

//NO COOKIES TESTS
#[tokio::test]
async fn test_plain_get_no_hickory_nc(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);

    let http_client = HttpClient::new(false, false);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_hickory_nc(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);

    let http_client = HttpClient::new(true, false);
    let resp = http_client.get(&url, None).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_dns_fail_nc(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, false);
    let resp = http_client.get("http://http://www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.is_err(),true);
}

#[tokio::test]
async fn test_plain_get_fail_nc(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false, false);
    let resp = http_client.get("http:/www.google.com/page/", None).await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[tokio::test]
async fn test_json_post_hickory_nc(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}/test_post.php",SERVER);

    let http_client = HttpClient::new(true, false);
    let resp = http_client.post(&url, None, body, ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_text_post_hickory_nc(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello Bachuetech";
    let body = "{\"name\":\"Bachuetech\"}";
    let url = format!("{}/test_post.php",SERVER);

    let http_client = HttpClient::new(true, false);
    let resp = http_client.post(&url, None, body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_json_post_hickory_fail_nc(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );

    let url = format!("{}/test_post_fake.php",SERVER);

    let http_client = HttpClient::new(true, false);
    let resp = http_client.post(&url, None, "", ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}

#[test]
fn test_set_headers_nc(){
    let header_val =  "HEADER_VALUE";
    let header_name = "bt_header";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, false);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}

#[test]
fn test_change_headers_nc(){
    let header_val =  "HEADER_VALUE";
    let header_name = "user-agent";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    let mut http_client = HttpClient::new(false, false);
    http_client.set_header(&header_name, &header_val);

    println!("Headers: {:?}",&http_client.get_default_headers());
    assert_eq!(http_client.get_default_headers().get(header_name).unwrap(),header_val); 
}