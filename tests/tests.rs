use bt_http_utils::{self, ContentType, HttpClient};
use bt_logger::{build_logger, LogLevel, LogTarget};



#[tokio::test]
async fn test_plain_get_no_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);

    let http_client = HttpClient::new(false);
    let resp = http_client.get(&url).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_hickory(){
    const SERVER: &str = "http://localhost";

    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let test_content = "Hello World!";
    let url = format!("{}/test_get.html",SERVER);

    let http_client = HttpClient::new(true);
    let resp = http_client.get(&url).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_plain_get_dns_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false);
    let resp = http_client.get("http://http://www.google.com/page/").await;
    println!("Staus: {:?}",&resp);
    assert_eq!(resp.is_err(),true);
}

#[tokio::test]
async fn test_plain_get_fail(){
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );
    
    let http_client = HttpClient::new(false);
    let resp = http_client.get("http:/www.google.com/page/").await;
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

    let http_client = HttpClient::new(true);
    let resp = http_client.post(&url, body, ContentType::JSON).await;
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

    let http_client = HttpClient::new(true);
    let resp = http_client.post(&url, body, ContentType::TEXT).await;
    println!("Body: {:?}",&resp); 


    assert_eq!(resp.unwrap().body,test_content);
}

#[tokio::test]
async fn test_json_post_hickory_fail(){
    const SERVER: &str = "http://localhost";
    
    build_logger("BACHUETECH", "BT.HTTP.UTILS", LogLevel::VERBOSE, LogTarget::STD_ERROR );

    let url = format!("{}/test_post_fake.php",SERVER);

    let http_client = HttpClient::new(true);
    let resp = http_client.post(&url, "", ContentType::JSON).await;
    println!("Body: {:?}",&resp);
    assert_eq!(resp.unwrap().is_error(),true);
}