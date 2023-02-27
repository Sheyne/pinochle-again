use actix_cors::Cors;
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use std::collections::HashMap;
use std::sync::Mutex;

mod pinochle;
use pinochle::{Action, Error, Game, GameInfo, Player};
struct AppState {
    games: Mutex<HashMap<String, Game>>,
}

#[get("/game")]
async fn get_games(data: web::Data<AppState>) -> impl Responder {
    let games = data.games.lock().unwrap();
    let names: Vec<_> = games.keys().collect();
    HttpResponse::Ok().json(names)
}

#[get("/game/{game}")]
async fn get_game(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let games = data.games.lock().unwrap();
    let name = game.into_inner();
    if let Some(game) = games.get(&name) {
        let info: GameInfo = game.into();
        HttpResponse::Ok().json(&info)
    } else {
        HttpResponse::NotFound().body("")
    }
}

#[get("/game/{game}/full")]
async fn get_full_game(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let games = data.games.lock().unwrap();
    let name = game.into_inner();
    if let Some(game) = games.get(&name) {
        HttpResponse::Ok().json(game)
    } else {
        HttpResponse::NotFound().body("")
    }
}

#[get("/game/{game}/hand/{player}")]
async fn get_hand(game: web::Path<(String, Player)>, data: web::Data<AppState>) -> impl Responder {
    let games = data.games.lock().unwrap();
    let (name, player) = game.into_inner();
    if let Some(game) = games.get(&name) {
        HttpResponse::Ok().json(&game.player_hand(player))
    } else {
        HttpResponse::NotFound().body("")
    }
}

#[post("/game/{game}/{player}/act")]
async fn act(
    game: web::Path<(String, Player)>,
    info: web::Json<Action>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut games = data.games.lock().unwrap();
    let (name, player) = game.into_inner();
    if let Some(game) = games.get_mut(&name) {
        if let Some(err) = game.act(player, info.0).err() {
            match err {
                Error::PlayingNonExtantCard => {
                    HttpResponse::BadRequest().body("PlayingNonExtantCard")
                }
                Error::PassingWrongNumberOfCards => {
                    HttpResponse::BadRequest().body("PassingWrongNumberOfCards")
                }
                Error::NotTheCurrentPlayer => {
                    HttpResponse::BadRequest().body("NotTheCurrentPlayer")
                }
                Error::CardIsNotLegalToPlay => {
                    HttpResponse::BadRequest().body("CardIsNotLegalToPlay")
                }
                Error::IncorrectAction => HttpResponse::BadRequest().body("IncorrectAction"),
            }
        } else {
            HttpResponse::Ok().body("")
        }
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

fn create(game: web::Path<String>, info: Game, data: web::Data<AppState>) {
    let mut games = data.games.lock().unwrap();
    let name = game.into_inner();
    games.insert(name, info);
}

#[put("/game/{game}")]
async fn create_with(
    game: web::Path<String>,
    info: web::Json<Game>,
    data: web::Data<AppState>,
) -> impl Responder {
    create(game, info.into_inner(), data);
    HttpResponse::Ok()
}

#[post("/game/{game}")]
async fn create_without(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    create(game, Game::default(), data);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared = web::Data::new(AppState {
        games: Mutex::default(),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .app_data(shared.clone())
            .service(act)
            .service(get_full_game)
            .service(get_game)
            .service(get_games)
            .service(get_hand)
            .service(create_without)
            .service(create_with)
            .service(actix_files::Files::new("/", "./www/build").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
