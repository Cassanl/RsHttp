pub enum WhatToHandle {
    Echo,
    UserAgent,
    PostFile,
    GetFile,
    Wildcard,
    Error
}

#[derive(PartialEq)]
pub enum RequestMethod {
    Get, 
    Post,
    Error
}

pub struct Request {
    pub method: RequestMethod,
    pub url: String,
    pub version: String,
    pub headers: String,
    pub body: String
}

impl Request {
    pub fn new(
        method: RequestMethod,
        url: String,
        version: String,
        headers: String,
        body: String
    ) -> Self {
        Request { method, url, version, headers, body }
    }

    pub fn default() -> Self {
        Request { 
            method: RequestMethod::Get,
            url: "/".into(),
            version: "HTTP/1.1".into(),
            headers: "".into(),
            body: "".into()
        }
    }
}

pub struct Response {
    pub core: String,
    pub headers: String,
    pub body: String
}

impl Response {
    pub fn new(core: String, headers: String, body: String) -> Self {
        Response { core, headers, body }
    }

    pub fn default_ok() -> Self {
        Response { core: "HTTP/1.1 200 OK\r\n\r\n".into(), headers: "".into(), body: "".into()}
    }

    pub fn default_error() -> Self {
        Response { core: "HTTP/1.1 404 Not Found\r\n\r\n".into(), headers: "".into(), body: "".into()}
    }

    pub fn format_to_send(&self) -> String {
        format!("{}{}{}", self.core, self.headers, self.body)
    }
}
