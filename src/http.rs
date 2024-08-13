use std::usize;
use std::{collections::HashMap, vec};

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
enum Method {
    GET,
    POST,
    UNKNOWN,
}
pub struct Request {
    start_line: Vec<String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    //method: Method,
}

impl Request {
    pub fn get_method(&self) -> Method {
        match self.start_line.get(0) {
            Some(m) => {
                let m_lower = &m.to_lowercase();
                if m_lower == "get" {
                    return Method::GET;
                } else if m_lower == "post" {
                    return Method::POST;
                } else {
                    return Method::UNKNOWN;
                }
            }
            None => {
                println!("{}:{} - 请求方法解析错误", file!(), line!());
                return Method::UNKNOWN;
            }
        }
    }
    pub fn get_path(&self) -> Option<String> {
        match self.start_line.get(1) {
            Some(path) => return Some(path.to_string()),
            None => {
                return None;
            }
        }
    }
}

pub struct Response {
    status_line: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
impl Response {
    pub fn new(status_line: String, headers: HashMap<String, String>, body: Vec<u8>) -> Response {
        return Response {
            status_line,
            headers,
            body,
        };
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = self.status_line.clone().into_bytes();

        // 添加 headers，每个 header 后面跟一个\r\n
        for (key, value) in &self.headers {
            bytes.extend_from_slice(key.as_str().as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value.as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }

        // 添加一个额外的\r\n来结束 headers 部分
        bytes.extend_from_slice(b"\r\n");

        // 连接 body
        bytes.extend_from_slice(&self.body);

        return bytes;
    }
}

pub struct Parser {
    header_buf_size: usize,
    is_end: bool,
}

pub enum ParserErr {
    UnKnown,
    SockClose,
    NoEnd,
    DataLoss,
    ReadFail,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            header_buf_size: 4096,
            is_end: false,
        }
    }
    pub fn set_buf_size(&mut self, bsz: usize) {
        self.header_buf_size = bsz;
    }
    pub fn is_end(&self) -> bool {
        return self.is_end;
    }
    pub async fn parse(&mut self, stream: &mut TcpStream) -> Result<Request, ParserErr> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.header_buf_size);
        /* if let Ok(num_read) = stream.read_buf(&mut buf).await {
            if num_read == 0 {
                self.is_end = true;
            }
        } else {
            println!("{}:{} - 读取失败", file!(), line!());
            return Err(ParserErr::ReadFail);
        } */
       match stream.read_buf(&mut buf).await {
           Ok(n) => {
            if n == 0 {
                self.is_end = true;
            }
           },
           Err(e) => {
            self.is_end = true;
            return Err(ParserErr::ReadFail);
           }
       }

        match std::str::from_utf8(&buf) {
            Ok(msg) => {
                return Self::build_req(stream, msg, &buf).await;
            }
            Err(e) => {
                println!("{}:{} - 转换utf-8失败", file!(), line!());
                let valid_id = e.valid_up_to();
                match std::str::from_utf8(&buf[..valid_id]) {
                    Ok(msg) => {
                        return Self::build_req(stream, msg, &buf).await;
                    }
                    Err(err) => {
                        return Err(ParserErr::UnKnown);
                    }
                }
            }
        }
    }
    async fn build_req(
        stream: &mut TcpStream,
        msg: &str,
        buf: &Vec<u8>,
    ) -> Result<Request, ParserErr> {
        match msg.find("\r\n\r\n") {
            None => {
                println!("{}:{} - 找不到结束符", file!(), line!());
                return Err(ParserErr::NoEnd);
            }

            Some(id_end) => {
                let mut request: Request = Request {
                    start_line: vec![],
                    headers: HashMap::new(),
                    body: vec![],
                };
                let items: Vec<&str> = msg.split("\r\n").collect();
                if let Some(&first_line) = items.first() {
                    // 用空格分隔请求行
                    request.start_line = first_line.split(" ").map(|tmp| tmp.to_string()).collect();
                    if request.start_line.len() != 3 {
                        return Err(ParserErr::UnKnown);
                    }
                } else {
                    return Err(ParserErr::UnKnown);
                }
                for &item in items.iter().skip(1) {
                    if item == "" {
                        break;
                    }
                    match item.split_once(':') {
                        Some((k, v)) => {
                            request.headers.insert(k.to_string(), v.to_string());
                        }
                        None => {
                            return Err(ParserErr::UnKnown);
                        }
                    }
                }
                match request.headers.get(&"Content-Length".to_string()) {
                    Some(len) => match len.parse::<usize>() {
                        Ok(len) => {
                            request.body.extend_from_slice(&buf[(id_end + 4)..]);
                            let res = len - request.body.len();
                            if res > 0 {
                                let mut buf_2: Vec<u8> = Vec::with_capacity(res);
                                match stream.read_buf(&mut buf_2).await {
                                    Ok(n) => {
                                        if n < res {
                                            return Err(ParserErr::DataLoss);
                                        } else {
                                            request.body.extend_from_slice(&buf_2);
                                            return Ok(request);
                                        }
                                    }
                                    Err(e) => {
                                        return Err(ParserErr::ReadFail);
                                    }
                                }
                            } else {
                                return Ok(request);
                            }
                        }
                        Err(e) => {
                            return Err(ParserErr::UnKnown);
                        }
                    },
                    None => {
                        request.body = Vec::with_capacity(0);
                        // 解析成功 没有请求体
                        return Ok(request);
                    }
                }
            }
        }
    }
}

/* impl Parser {
    pub fn new() -> Parser{
        Parser {
            header_buf_size: 4096,
        }
    }
    pub fn set_buf_size(&mut self, bsz: usize) {
        self.header_buf_size = bsz;
    }
    pub async fn parse(&self, stream: &mut TcpStream) -> Result<Request, ParserErr> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.header_buf_size);
        let num: usize = stream.read(&mut buf).await.unwrap();
        if num <= 0 {
            println!("{}:{} - socket closed", file!(), line!());
            return Err(ParserErr::SockClose);
        } else {
            match std::str::from_utf8(&buf) {
                Ok(msg) => {
                    match msg.find("Content-Length:") {
                        Some(id) => {
                            match msg[id..].find("\r\n") {
                                Some(id_e) => {
                                    match msg[(id + "Content-Length:".len())..id_e].parse::<usize>() {
                                        Ok(len) => {
                                            return Err(ParserErr::NoEnd);
                                        },
                                        Err(e) => {
                                            return Err(ParserErr::UnKnown);
                                        },
                                    }
                                },
                                None => {
                                    return Err(ParserErr::UnKnown);
                                },
                            }
                        },
                        None => {
                            match msg.find("\r\n\r\n") {
                                Some(end) => {
                                    return Ok(Self::build_request_only_header(&msg[..(end + 2)].to_string()));
                                },
                                None => {
                                    return Err(ParserErr::NoEnd);
                                },
                            }
                        },
                    }
                },
                Err(e) => {
                    let id: usize = e.valid_up_to();
                    let head = String::from_utf8(buf[..id].to_vec()).unwrap();
                    match head.find("Content-Length:") {
                        Some(id) => {
                            match head[id..].find("\r\n") {
                                Some(id_e) => {
                                    match head[(id + "Content-Length:".len())..id_e].parse::<usize>() {
                                        Ok(len) => {
                                            return Err(ParserErr::NoEnd);
                                        },
                                        Err(e) => {
                                            return Err(ParserErr::UnKnown);
                                        },
                                    }
                                },
                                None => {
                                    return Err(ParserErr::UnKnown);
                                },
                            }
                        },
                        None => {
                            match head.find("\r\n\r\n") {
                                Some(end) => {
                                    return Ok(Self::build_request_only_header(&head[..(end + 2)].to_string()));
                                },
                                None => {
                                    return Err(ParserErr::NoEnd);
                                },
                            }
                        },
                    }
                },
            }
        }

    }
    fn build_request_only_header(h: &String) -> Request {
        let mut request = Request {
            start_line: Vec::with_capacity(3),
            headers: HashMap::new(),
            body: Vec::with_capacity(0),
        };
        let items: Vec<&str> = h.split("\r\n").collect();
        let mut flag: bool = true;
        for item in items {
            if flag {
                let start_line: Vec<&str> = item.split(" ").collect();
                let start_line: Vec<String> =
                    start_line.into_iter().map(|s| s.to_string()).collect();
                request.start_line = start_line;
                flag = false;
            } else {
                if item.is_empty() == false {
                    match item.find(":") {
                        Some(id) => {
                            request.headers.insert(
                                item[..id].to_string(),
                                item[(id + 1)..].to_string(),
                            );
                        }
                        None => {
                            println!("{}:{} - header解析出错", file!(), line!());
                        }
                    }
                }
            }
        }
        return request;
    }
}
 */
