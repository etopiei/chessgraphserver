extern crate rusted_cypher;
extern crate iron;

use std::collections::BTreeMap;
use std::env;
use std::io::Read;

use rusted_cypher::GraphClient;
use rusted_cypher::cypher::Statement;

use iron::prelude::*;
use iron::status;

fn main() {
    fn get_data_fen(req: &mut Request) -> IronResult<Response> {
        let mut fen_string: String = "".to_string();
        req.body.read_to_string(&mut fen_string).unwrap();
        Ok(Response::with((status::Ok, format!("Fetching results for: {}", fen_string))))
    }

    let user = env::var("NEO4J_USER").unwrap();
    let pw = env::var("NEO4J_PASSWORD").unwrap();
    let graph = GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

    let _server = Iron::new(get_data_fen).http("localhost:3000").unwrap();
    println!("On 3000");
}
