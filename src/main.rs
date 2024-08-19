mod tcpserver;
mod http;
mod router;
use std::collections::HashMap;
use lazy_static::lazy_static;

#[tokio::main]
async fn main() {
    let routes: Vec<router::Route> = vec![
        router::Route::new("/".to_string(), || Box::pin(home())),
        router::Route::new("/home".to_string(), || Box::pin(home())),
        router::Route::new("/favicon.ico".to_string(), || Box::pin(icon()))
    ];
    let router = router::Router::new(routes);
    let server = tcpserver::Server::new("192.168.91.29:8080".to_string(), router);
    server.run().await;
}



// 文件加载复用
lazy_static! {
    static ref HTMLFile: HashMap<&'static str, Vec<u8>> = {
        preload_files(vec![
            "src/resource/html/home.html",
            "src/resource/img/icon.png",
        ])
    };
}
fn preload_files(files: Vec<&'static str>) -> HashMap<&'static str, Vec<u8>> {
    use std::io::Read;
    let mut m = HashMap::new();
    for f in files {
        let mut fs = std::fs::File::open(f).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        fs.read_to_end(&mut buf).unwrap();
        m.insert(f, buf);
    }
    m
}


// 路由函数
async fn home() ->http::Response {
    let status_line: String = "HTTP/1.1 200 OK\r\n".to_string();
    let mut headers: HashMap<String, String> = HashMap::new();
    let body: &Vec<u8> = HTMLFile.get("src/resource/html/home.html").unwrap();
    headers.insert("Content-Length".to_string(), body.len().to_string());
    let res: http::Response = http::Response::new(status_line, headers, body.clone());
    return res;
}

async fn icon() ->http::Response {
    let status_line: String = "HTTP/1.1 200 OK\r\n".to_string();
    let mut headers: HashMap<String, String> = HashMap::new();
    let body: &Vec<u8> = HTMLFile.get("src/resource/img/icon.png").unwrap();
    headers.insert("Content-Length".to_string(), body.len().to_string());
    let res: http::Response = http::Response::new(status_line, headers, body.clone());
    return res;
}