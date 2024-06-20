use spin_sdk::http::{send, RequestBuilder, Response};

#[allow(warnings)]
mod bindings;

fn main() {
    spin_executor::run(async move {
        let req =
            RequestBuilder::new(spin_sdk::http::Method::Get, "https://myip.fermyon.app").build();
        let res: Response = send(req).await.unwrap();
        println!("Your IP is: {}", String::from_utf8_lossy(res.body()));
    });
}
