#[macro_use]
extern crate rusted_cypher;
extern crate iron;
extern crate router;

use serde::{Deserialize, Serialize};

use std::env;
use std::io::Read;

use rusted_cypher::GraphClient;

use iron::mime::Mime;
use iron::prelude::*;
use iron::{status, Chain};
use iron_cors::CorsMiddleware;
use router::Router;

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    event: String,
    game_id: String,
    result: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    move_string: String,
    move_number: usize,
    position: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerData {
    name: String,
    player_id: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct SearchData {
    result_type: String,
    data: PlayerData,
}

fn main() {
    let cors = CorsMiddleware::with_allow_any();

    let mut router = Router::new();
    router.post("/games/fen", get_games_with_fen, "list_games");
    router.post("/search/player", search, "search");
    router.post("/games/player", get_games_for_player_id, "player_games");
    router.post("/games/moves", from_game_id_get_moves, "game_moves");

    let mut chain = Chain::new(router);
    chain.link_around(cors);

    Iron::new(chain).http(("127.0.0.1", 3000)).unwrap();
}

fn search(req: &mut Request) -> IronResult<Response> {
    let user = env::var("NEO4J_USER").unwrap();
    let pw = env::var("NEO4J_PASSWORD").unwrap();
    let graph =
        GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

    let mut search_string: String = "".to_string();
    req.body.read_to_string(&mut search_string).unwrap();
    println!("Search term: {}", search_string);

    // To start off the search will only be able to search players
    let statement =
        cypher_stmt!("MATCH (p:Player) WHERE p.name CONTAINS {search} RETURN p.name, ID(p)", {
            "search" => &search_string
        })
        .unwrap();

    let result = graph.exec(statement).unwrap();
    println!("Players Found: {}", result.data.len());

    let mut player_data: Vec<SearchData> = Vec::new();
    for row in result.rows() {
        let player = SearchData {
            result_type: "Player".to_string(),
            data: PlayerData {
                name: row.get("p.name").unwrap(),
                player_id: row.get("ID(p)").unwrap(),
            },
        };
        player_data.push(player);
    }

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((
        content_type,
        status::Ok,
        serde_json::to_string(&player_data).unwrap(),
    )))
}

fn get_games_for_player_id(req: &mut Request) -> IronResult<Response> {
    let user = env::var("NEO4J_USER").unwrap();
    let pw = env::var("NEO4J_PASSWORD").unwrap();
    let graph =
        GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

    let mut player_id: String = "".to_string();
    req.body.read_to_string(&mut player_id).unwrap();
    let player_id: usize = player_id.parse().unwrap();
    println!("Searching for player ID: {}", player_id);

    // Need to account for player id being an int here
    let statement = cypher_stmt!("MATCH (p:Player)-[:PLAYED_WHITE_IN | :PLAYED_BLACK_IN]->(g:Game) WHERE ID(p) = {player_id} RETURN g.game_id, g.event, g.result LIMIT 20", {
        "player_id" => player_id
    }).unwrap();

    let result = graph.exec(statement).unwrap();
    println!("Games Found: {}", result.data.len());

    let mut game_data: Vec<GameData> = Vec::new();
    for row in result.rows() {
        let game = GameData {
            event: row.get("g.event").unwrap(),
            game_id: row.get("g.game_id").unwrap(),
            result: row.get("g.result").unwrap(),
        };
        game_data.push(game);
    }

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((
        content_type,
        status::Ok,
        serde_json::to_string(&game_data).unwrap(),
    )))
}

fn get_games_with_fen(req: &mut Request) -> IronResult<Response> {
    let user = env::var("NEO4J_USER").unwrap();
    let pw = env::var("NEO4J_PASSWORD").unwrap();
    let graph =
        GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

    let mut fen_string: String = "".to_string();
    req.body.read_to_string(&mut fen_string).unwrap();
    println!("FEN String: {}", fen_string);

    let statement = cypher_stmt!("MATCH (g:Game)-[:HAD_POSITION]->(p:Position) WHERE p.FEN = {game_fen} RETURN g.game_id, g.event, g.result LIMIT 20", {
        "game_fen" => &fen_string
    }).unwrap();

    let result = graph.exec(statement).unwrap();
    println!("Games Found: {}", result.data.len());

    let mut game_data: Vec<GameData> = Vec::new();
    for row in result.rows() {
        let game = GameData {
            event: row.get("g.event").unwrap(),
            game_id: row.get("g.game_id").unwrap(),
            result: row.get("g.result").unwrap(),
        };
        game_data.push(game);
    }

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((
        content_type,
        status::Ok,
        serde_json::to_string(&game_data).unwrap(),
    )))
}

fn from_game_id_get_moves(req: &mut Request) -> IronResult<Response> {
    let user = env::var("NEO4J_USER").unwrap();
    let pw = env::var("NEO4J_PASSWORD").unwrap();
    let graph =
        GraphClient::connect(format!("http://{}:{}@localhost:7474/db/data", user, pw)).unwrap();

    let mut game_id = "".to_string();
    req.body.read_to_string(&mut game_id).unwrap();
    println!("Finding moves for game: {}", game_id);

    let statement = cypher_stmt!("MATCH (g:Game)-[i:HAD_POSITION]->(p:Position) WHERE g.game_id = {game_id} RETURN p.FEN, i.move, i.move_number", {
        "game_id" => &game_id
    }).unwrap();

    let result = graph.exec(statement).unwrap();
    println!("Moves Found: {}", result.data.len());

    let mut move_data: Vec<Move> = Vec::new();
    for row in result.rows() {
        let current_move = Move {
            move_string: row.get("i.move").unwrap_or_else(|_| "NULL".to_string()),
            move_number: row
                .get("i.move_number")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap(),
            position: row.get("p.FEN").unwrap_or_else(|_| "NULL".to_string()),
        };
        move_data.push(current_move);
    }

    // Maybe here sort the data? Lessen the load on the JS
    move_data.sort_by(|a, b| a.move_number.cmp(&b.move_number));

    let content_type = "application/json".parse::<Mime>().unwrap();
    Ok(Response::with((
        content_type,
        status::Ok,
        serde_json::to_string(&move_data).unwrap(),
    )))
}
