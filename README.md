# Project Title
BT HTTP UTILS

## Description
A simple HTTP wrapper to simplify POST and GET calls.

## Usage
```
    let http_client = HttpClient::new(false);

    let resp_get = http_client.get(&url).await; 
    let resp_post_txt = http_client.post(&url, body, ContentType::TEXT).await;
    let resp_post_json = http_client.post(&url, body, ContentType::JSON).await;

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

## License
GPL-3.0-only