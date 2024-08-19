use std::usize;
use std::{collections::HashMap, vec};

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

// 暂时只支持 get
/* enum Method {
    GET,
    POST,
    UNKNOWN,
} */
pub struct Request {
    start_line: Vec<String>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    //method: Method,
}

impl Request {
    /* pub fn get_method(&self) -> Method {
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
    } */
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

#[derive(Debug)]
pub enum ParserErr {
    UnKnown,
    SockClose,
    NoEnd,
    DataLoss,
    ReadFail,
    InvalidUtf8,
}



impl Parser {
    pub fn new() -> Self {
        Self {
            header_buf_size: 4096,
            is_end: false,
        }
    }

    pub fn set_buf_size(&mut self, bsz: usize) {
        self.header_buf_size = bsz;
    }

    pub fn is_end(&self) -> bool {
        self.is_end
    }
    pub async fn parse(&mut self, stream: &mut TcpStream) -> Result<Request, ParserErr> {
        let mut buf: Vec<u8> = vec![0; self.header_buf_size];
        let num_read: usize = stream.read(&mut buf).await.map_err(|_| ParserErr::ReadFail)?;

        if num_read == 0 {
            self.is_end = true;
            return Err(ParserErr::SockClose);
        }

        buf.truncate(num_read);
        let msg = std::str::from_utf8(&buf).map_err(|_| ParserErr::InvalidUtf8)?;
        
        Self::build_req(stream, msg, &buf).await
    }

    async fn build_req(
        stream: &mut TcpStream,
        msg: &str,
        buf: &[u8],
    ) -> Result<Request, ParserErr> {
        let id_end = msg.find("\r\n\r\n").ok_or(ParserErr::NoEnd)?;

        let mut request = Request {
            start_line: vec![],
            headers: HashMap::new(),
            body: vec![],
        };

        let mut lines = msg[..id_end].split("\r\n");

        // Parse start line
        let start_line = lines.next().ok_or(ParserErr::UnKnown)?;
        request.start_line = start_line.split_whitespace().map(String::from).collect();
        if request.start_line.len() != 3 {
            return Err(ParserErr::UnKnown);
        }

        // Parse headers
        for line in lines {
            let (key, value) = line.split_once(':').ok_or(ParserErr::UnKnown)?;
            request.headers.insert(key.trim().to_string(), value.trim().to_string());
        }

        // Parse body
        let body_start = id_end + 4;
        let content_length: usize = request.headers
            .get("Content-Length")
            .and_then(|len| len.parse().ok())
            .unwrap_or(0);

        request.body.extend_from_slice(&buf[body_start..]);
        let remaining = content_length.saturating_sub(request.body.len());

        if remaining > 0 {
            let mut additional_buf = vec![0; remaining];
            let n = stream.read_exact(&mut additional_buf).await.map_err(|_| ParserErr::ReadFail)?;
            if n < remaining {
                return Err(ParserErr::DataLoss);
            }
            request.body.extend_from_slice(&additional_buf);
        }

        Ok(request)
    }
} 