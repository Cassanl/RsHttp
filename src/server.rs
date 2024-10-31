use core::str;
use std::net::TcpStream;
use std::io::{prelude::*, Write};
use std::fs;
use std::env;

use crate::model::{Request, RequestMethod, Response, WhatToHandle};

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n";
// const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 200 Not Found\r\n";
// const CREATED_RESPONSE: &str = "HTTP/1.1 201 Created\r\n";

 fn get_what_to_handle(request: &Request) -> WhatToHandle {
    let what_to_handle = if request.url.starts_with("/echo/") {
        WhatToHandle::Echo
    } else if request.url.starts_with("/files/") && request.method == RequestMethod::Get {
        WhatToHandle::GetFile
    } else if request.url.starts_with("/files/") && request.method == RequestMethod::Post {
        WhatToHandle::PostFile
    } else if request.url.starts_with("/user-agent") {
        WhatToHandle::UserAgent
    } else if request.url == "/" {
        WhatToHandle::Wildcard
    } else {
        WhatToHandle::Error
    };
    what_to_handle
}

fn handle_compression(request: &Request) -> Option<String> {
    if request.headers.contains("Accept-Encoding") {
        let compression: Vec<&str> = request
        .headers
        .split("\r\n")
        .filter(|x| x.starts_with("Accept-Encoding"))
        .map(|x| x.strip_suffix("Accept-Encoding: ").unwrap())
        .collect();
        match compression[0] {
            "gzip" => return Some("Content-Encoding: gzip".into()),
            _ => return None
        };
    }
    None
}

fn parse_request(buffer: &[u8]) -> Request {
    match str::from_utf8(buffer) {
        Ok(content) => {
            let (first_line, rest) = content
                .split_once("\r\n")
                .unwrap();
            let method_url_version: Vec<&str> = first_line
                .split(" ")
                .collect();
            let method = match method_url_version[0] {
                "GET" => RequestMethod::Get,
                "POST" => RequestMethod::Post,
                _ => RequestMethod::Error
            };
            let url = method_url_version[1];
            let version: &str = method_url_version[2];
            let (headers, body) = rest
                .split_once("\r\n\r\n")
                .unwrap();
            println!("{url}");
            println!("{version}");
            println!("{headers}");
            println!("{body}");
            Request::new(
                method,
                url.into(),
                version.into(),
                headers.into(), 
                body.into())
        },
        Err(_) => Request::default()
    } 
}

pub fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");
    let mut buffer = [0 as u8; 2048];
    let read = stream.read(&mut buffer).unwrap();
    if read >= buffer.len() {
        let response = "HTTP/1.1 414 Uri Too Long\r\n\r\n";
        let _bytes_written = stream.write_all(response.as_bytes());
        return;
    }
    let parsed_request = parse_request(&buffer[..]);
    let response: Response = match get_what_to_handle(&parsed_request) {
        WhatToHandle::Echo => handle_echo(parsed_request),
        WhatToHandle::UserAgent => handle_user_agent(parsed_request),
        WhatToHandle::PostFile => handle_post_file(parsed_request),
        WhatToHandle::GetFile => handle_get_file(parsed_request),
        WhatToHandle::Wildcard => Response::default_ok(),
        WhatToHandle::Error => Response::default_error()
    };
    let response = response.format_to_send();
    let _bytes_written = stream.write_all(response.as_bytes());
}

fn handle_echo(request: Request) -> Response {
    let message = request.url.replace("/echo/", "");
    let headers = format!("Content-Length: {}\r\n\r\n", message.len());
    match handle_compression(&request) {
        Some(compression) => {
            let headers = format!("Content-Length: {}\r\n{}\r\n\r\n", message.len(), compression);
            Response::new(OK_RESPONSE.into(), headers, message)
        },
        None => Response::new(OK_RESPONSE.into(), headers, message)
    }
}

fn handle_user_agent(request: Request) -> Response {
    let user_agent: Vec<&str> = request
        .headers
        .split("\r\n")
        .filter(|x| x.starts_with("User-Agent"))
        .map(|x| x.strip_prefix("User-Agent: ").unwrap())
        .collect();
    let headers = format!("Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n", user_agent[0]);
    Response::new(OK_RESPONSE.into(), headers, user_agent[0].into())
}

fn handle_get_file(request: Request) -> Response {
    let file_name = request.url.replace("/files/", "");
    let args: Vec<String> = env::args().collect();
    let mut directory = args[2].clone();
    directory.push_str(&file_name);
    let file = fs::read_to_string(directory);
    match file {
        Ok(content) => {
            let headers = format!("Content-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n", content.len());
            Response::new(OK_RESPONSE.into(), headers, content)
        },
        Err(_) => Response::default_error()
    }
}

fn handle_post_file(request: Request) -> Response {
    let file_name = request.url.replace("/files/", "");
    let args: Vec<String> = env::args().collect();
    let mut directory = args[2].clone();
    directory.push_str(&file_name);
    let written_file = fs::write(directory, request.body);
    match written_file {
        Ok(_) => Response::default_ok(),
        Err(_) => Response::default_error()
    }
}