extern crate hyper;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;

fn main() {
    // Create a client.
    let mut client = Client::new();

    // Creating an outgoing request.
    let mut res = client.get("http://www.gooogle.com/")
        // set a header
        .header(Connection(vec![ConnectionOption::Close]))
        // let 'er go!
        .send().unwrap();

    // Read the Response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("Response: {}", body);
}
