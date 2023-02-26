use actix_cors::Cors;
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use std::collections::HashMap;
use std::sync::Mutex;

mod pinochle;
use pinochle::{Action, Error, Game, GameInfo, Player};

struct AppState {
    games: Mutex<HashMap<String, Game>>,
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
                Error::IncorrectAction => HttpResponse::BadRequest().body("IncorrectAction"),
            }
        } else {
            HttpResponse::Ok().body("")
        }
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

#[put("/game/{game}")]
async fn create(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let mut games = data.games.lock().unwrap();
    let name = game.into_inner();
    games.insert(name, Game::default());
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
            .service(get_game)
            .service(get_hand)
            .service(create)
            .service(actix_files::Files::new("/", "./www/build").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
