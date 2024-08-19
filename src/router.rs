use std::collections::HashMap;

use crate::http;
use std::future::Future;
use std::pin::Pin;

// 应该使用hashmap
pub struct Router {
    routes: Vec<Route>,
}
impl Router {
    pub fn new(routes: Vec<Route>) -> Router {
        return Router {
            routes: routes,
        };
    }
    pub async fn create_response(&self, rq: &http::Request) -> http::Response {
        //let method = rq.get_method();
        // GET or POST
        let  path: String;
        match rq.get_path() {
            Some(p) => {path = p;}
            None => {
                let body: Vec<u8> = "<html><div>Page Not Found</div></html>".as_bytes().to_vec();
                let mut headers: HashMap<String, String> = HashMap::new();
                headers.insert("Content-Length".to_string(), body.len().to_string());
                return http::Response::new("HTTP/1.1 404 NOT-FOUND\r\n".to_string(), headers, body);
            }
        }
        return self.route(&path).await;
    }
    pub async  fn route(&self, path: &str) -> http::Response {
        for item in self.routes.iter() {
            if item.path == path {
                return (item.func)().await;
            }
        }
        println!("{}:{} - 找不到路由:{path}", file!(), line!());
        let body: Vec<u8> = "<html><div>Page Not Found</div></html>".as_bytes().to_vec();
        let mut headers: HashMap<String, String> = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());
        return http::Response::new("HTTP/1.1 404 NOT-FOUND\r\n".to_string(), headers, body);
    }
}

/* pub struct Route {
    path: String,
    func: fn() -> http::Response,
}

impl Route {
    pub fn new(path: String, func: fn() -> http::Response) -> Route {
        return Route {
            path: path,
            func: func,
        };
    }
} */

/* fn home() -> http::Response {
    let tmp: http::Response = http::Response {

    };
    return tmp;
} */

pub struct Route {
    path: String,
    func: fn() -> Pin<Box<dyn Future<Output = http::Response> + Send>>,
}
impl Route {
    pub fn new(path: String, func: fn() -> Pin<Box<dyn Future<Output = http::Response> + Send>>) -> Route {
        Route {
            path,
            func,
        }
    }
}