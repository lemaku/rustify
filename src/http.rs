#[derive(Debug)]
pub struct Header {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: Vec<Header>,
}

#[derive(Debug)]
pub struct Response {
    pub status: i16,
    pub status_message: String,
    pub headers: Vec<Header>,
    pub body: String,
}
