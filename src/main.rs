use std::fs;

mod cone_server;
fn main() {
    let ip_addr = String::from("127.0.0.1:7878");

    let mut router = cone_server::Router::new(ip_addr);
    router.new_route("/cone", cone).unwrap();
    router.new_route("/", cone2).unwrap();
    router.new_route("/conestr", cone_str).unwrap();

    router.start_routing();
}

fn cone() -> String {
    "cone string".to_string()
}

fn cone2() -> cone_server::HTMLResponse {
    cone_server::HTMLResponse::new(fs::read_to_string("hello.html").unwrap())
}

fn cone_str() -> &'static str {
    "cone str"
}
