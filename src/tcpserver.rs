use crate::{http, router};
//use http::ParserErr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

pub struct Server {
    router: Arc<router::Router>,
    socket_addr: String,
}

impl Server {
    pub fn new(addr: String, router: router::Router) -> Server {
        return Server {
            router: Arc::new(router),
            socket_addr: addr,
        };
    }

    pub async fn run(&self) {
        let listener: TcpListener = TcpListener::bind(&self.socket_addr).await.unwrap();
        println!(
            "{}:{} - 正在监听http://{}...",
            file!(),
            line!(),
            self.socket_addr
        );
        let mut i = 1;
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let router = Arc::clone(&self.router);
            println!("第{i}个连接");
            i = i + 1;
            task::spawn(Self::client_handler(router, stream));
        }
    }

    async fn client_handler(router: Arc<router::Router>, mut stream: TcpStream) {
        let mut parser = http::Parser::new();
        parser.set_buf_size(8192); // default 4096
        loop {
            if parser.is_end() {
                println!("{}:{} - 关闭tcp", file!(), line!());
                break; 
            } else {
                match parser.parse(&mut stream).await {
                    Ok(req) => {
                        let rps = router.create_response(&req).await;
        
                        stream.write(&rps.as_bytes()).await.unwrap();
                    }
                    Err(e) => {
                        match e {
                            _ => {
                                //println!("{}:{} - 发生错误: {:?}", file!(), line!(), e);
                                break;
                            },
                        }
                    }
                }
            }
        }
    }
}
