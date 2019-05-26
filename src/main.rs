#[macro_use]
extern crate rusted_cypher;
extern crate iron;

use serde::{Serialize, Deserialize};

use std::env;
use std::io::Read;

use rusted_cypher::GraphClient;

use iron::prelude::*;
use iron::status;
use iron::mime::Mime;

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    event: String,
    game_id: String,
    result: String,
}

fn main() {
    fn get_data_fen(req: &mut Request) -> IronResult<Response> {
        let user = env::var("NEO4J_USER").unwrap();
        let pw = env::var("NEO4J_PASSWORD").unwrap();
        let graph = GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

        let mut fen_string: String = "".to_string();
        req.body.read_to_string(&mut fen_string).unwrap();

        let statement = cypher_stmt!("MATCH (g:Game)-[:HAD_POSITION]->(p:Position) WHERE p.FEN = {game_fen} RETURN g.game_id, g.event, g.result LIMIT 20", {
            "game_fen" => &fen_string
        }).unwrap();
        
        let result = graph.exec(statement).unwrap();
        let mut game_data: Vec<GameData> = Vec::new();

        for row in result.rows() {
            let game = GameData {
                event: row.get("g.event").unwrap(),
                game_id: row.get("g.game_id").unwrap(),
                result: row.get("g.result").unwrap()
            };
            game_data.push(game);
        }
        let content_type = "application/json".parse::<Mime>().unwrap();
        Ok(Response::with((content_type, status::Ok, serde_json::to_string(&game_data).unwrap())))
    }

    let _server = Iron::new(get_data_fen).http("localhost:3000").unwrap();
    println!("On 3000");
}
