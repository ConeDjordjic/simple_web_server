use std::{
    collections::HashMap,
    fs,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

pub struct HTMLResponse(String);

impl HTMLResponse {
    pub fn new(s: String) -> HTMLResponse {
        HTMLResponse(s)
    }
}

pub trait IntoResponse {
    fn into_response(self: Box<Self>) -> String;
}

impl IntoResponse for String {
    fn into_response(self: Box<Self>) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
            self.len(),
            self
        )
    }
}

impl IntoResponse for HTMLResponse {
    fn into_response(self: Box<Self>) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}",
            self.0.len(),
            self.0
        )
    }
}

impl IntoResponse for &'static str {
    fn into_response(self: Box<Self>) -> String {
        Box::new(self.to_string()).into_response()
    }
}

pub struct Router {
    ip_addr: String,
    routes: HashMap<String, Box<dyn Fn() -> Box<dyn IntoResponse>>>,
}

impl Router {
    pub fn new(ip_address: String) -> Router {
        Router {
            ip_addr: ip_address,
            routes: HashMap::new(),
        }
    }

    pub fn new_route<F, R>(&mut self, route: &str, handler: F) -> anyhow::Result<()>
    where
        F: Fn() -> R + 'static,
        R: IntoResponse + 'static,
    {
        if self.routes.contains_key(route) {
            return Err(anyhow::anyhow!("Route already exists"));
        }
        self.routes
            .insert(route.to_string(), Box::new(move || Box::new(handler())));
        Ok(())
    }

    pub fn start_routing(&mut self) {
        let listener = TcpListener::bind(&self.ip_addr)
            .expect(&format!("Error while binding to {}", &self.ip_addr));

        if self.routes.len() != 0 {
            for route in &self.routes {
                println!("Establishing route: {}", route.0);
            }
        }

        println!("Listening for connections on {}", &self.ip_addr);

        for stream in listener.incoming() {
            let Ok(s) = stream else {
                println!("Error while reading from listener");
                continue;
            };

            match self.handle_connection(s) {
                Ok(r) => r,
                Err(err) => {
                    println!("Error while serving connection. Error: {}", err);
                    continue;
                }
            };
        }
    }

    fn handle_connection(&mut self, mut stream: TcpStream) -> anyhow::Result<()> {
        let reader = BufReader::new(&stream);
        let request_line = reader.lines().next().ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "EOF while reading request!")
        })??;

        self.handle_routing(&request_line, &mut stream)?;
        Ok(())
    }

    fn handle_routing(
        &mut self,
        request_line: &String,
        stream: &mut TcpStream,
    ) -> anyhow::Result<()> {
        let route_vec: Vec<&str> = request_line.split_whitespace().collect();
        let key_route = route_vec[1];
        let mut response = String::new();

        if !self.routes.contains_key(key_route) {
            let html_404 =
                fs::read_to_string("404.html").expect("Couldn't read from file 404.html!");
            let html_404_len = html_404.len();
            response = format!(
                "HTTP/1.1 404 NOT FOUND\r\nContent-Length: {}\r\n\r\n{}",
                html_404_len, html_404
            );
        }

        if let Some(handler) = self.routes.get(key_route) {
            response = handler().into_response();
        };

        stream.write_all(response.as_bytes()).unwrap();

        Ok(())
    }
}
