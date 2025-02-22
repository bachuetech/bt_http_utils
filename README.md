# Project Title
BT HTTP UTILS

## Description
A simple HTTP wrapper to simplify POST and GET calls.

## Usage
```
    let http_client = HttpClient::new(false, false);

    let resp_get = http_client.get(&url, None).await; 
    let resp_post_txt = http_client.post(&url, None, body, ContentType::TEXT).await;
    let resp_post_json = http_client.post(&url, None, body, ContentType::JSON).await;

    ///Fields available
    resp.unwrap().body;
    resp.unwrap().header;
    resp.unwrap().is_error();
    resp.unwrap().status_code;
```

## Version History
* 0.1.0
    * Initial Release
* 0.2.0
    * Include set and get default headers
* 0.2.1
    * Fix typos
* 0.3.0
    * Breaking  Change. Support for cookies. HttpClient::new(use hickory dns, use cookies)
* 0.4.0
    * Breaking  Change. Support for extra headers in get and post calls    
* 0.4.1
    * Fix content of error message   

## License
GPL-3.0-only