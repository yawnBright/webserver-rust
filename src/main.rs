mod tcpserver;
mod http;
mod router;
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    let routes: Vec<router::Route> = vec![
        router::Route::new("/home".to_string(), || Box::pin(home())),
        router::Route::new("/favicon.ico".to_string(), || Box::pin(icon()))
    ];
    let router = router::Router::new(routes);
    let server = tcpserver::Server::new("192.168.215.29:8080".to_string(), router);
    server.run().await;
}



async fn home() ->http::Response {
    let mut fs = File::open("src/resource/html/home.html").await.unwrap();
    let mut tmp: Vec<u8> = vec![];
    
    fs.read_to_end(&mut tmp).await.unwrap();
    let status_line: String = "HTTP/1.1 200 OK\r\n".to_string();
    let mut headers: HashMap<String, String> = HashMap::new();
    let body: Vec<u8> = tmp;
    headers.insert("Content-Length".to_string(), body.len().to_string());
    let res: http::Response = http::Response::new(status_line, headers, body);
    return res;
}

async fn icon() ->http::Response {
    let mut fs = File::open("src/resource/img/icon.png").await.unwrap();
    let mut tmp: Vec<u8> = vec![];
    
    fs.read_to_end(&mut tmp).await.unwrap();
    let status_line: String = "HTTP/1.1 200 OK\r\n".to_string();
    let mut headers: HashMap<String, String> = HashMap::new();
    let body: Vec<u8> = tmp;
    headers.insert("Content-Length".to_string(), body.len().to_string());
    let res: http::Response = http::Response::new(status_line, headers, body);
    return res;
}