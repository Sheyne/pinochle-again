use actix_cors::Cors;
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use pinochle::ai::Bot;
use pinochle::{Action, Error, Game, GameInfo, Phase, Player};
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct GameState {
    seed: [u8; 32],
    actions: Vec<Action>,
}

impl GameState {
    fn random() -> Self {
        Self {
            seed: thread_rng().gen(),
            actions: Default::default(),
        }
    }

    fn game(&self) -> Game<StdRng> {
        let mut game = Game::new(StdRng::from_seed(self.seed));
        for action in &self.actions {
            game.act(action.clone()).unwrap();
        }
        game
    }
}

struct AppState {
    games: Mutex<HashMap<String, GameState>>,
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
        let info: GameInfo = (&game.game()).into();
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
        HttpResponse::Ok().json(&game.game().player_hand(player))
    } else {
        HttpResponse::NotFound().body("")
    }
}

#[post("/game/{game}/trigger-bot")]
async fn trigger_bot(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let mut games = data.games.lock().unwrap();
    let name = game.into_inner();
    if let Some(game_init) = games.get_mut(&name) {
        let bot_player = game_init.game().current_player();

        let mut game = Game::new(StdRng::from_seed(game_init.seed));

        let mut bot: Option<Bot> = None;

        for action in &game_init.actions {
            let info: GameInfo = (&game).into();
            if let Phase::Play(playing_phase) = info.phase {
                if bot.is_none() {
                    bot = Some(Bot::new(
                        bot_player,
                        game.player_hand(bot_player),
                        playing_phase.clone(),
                    ));
                }
                if let Action::Play(card) = action {
                    if let Some(bot) = &mut bot {
                        let player_hand = game.player_hand(game.current_player());

                        bot.update(
                            game.current_player(),
                            player_hand[*card],
                            playing_phase.trump,
                            &playing_phase.trick.cards,
                        );
                    }
                }
            }
            game.act(action.clone()).unwrap();
        }
        let chosen_card = bot.unwrap().get_move();

        let chosen_action = game
            .player_hand(bot_player)
            .iter()
            .position(|x| *x == chosen_card)
            .unwrap();

        game_init.actions.push(Action::Play(chosen_action));
    }
    HttpResponse::Ok().body("")
}

#[post("/game/{game}/{player}/act")]
async fn act(
    game: web::Path<(String, Player)>,
    info: web::Json<Action>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut games = data.games.lock().unwrap();
    let (name, player) = game.into_inner();
    if let Some(game_state) = games.get_mut(&name) {
        let action = info.0.clone();
        let mut game = game_state.game();
        if let Action::Continue(action_player) = action {
            if action_player != player {
                return HttpResponse::BadRequest().body("NotTheCurrentPlayer");
            }
        } else if player != game.current_player() {
            return HttpResponse::BadRequest().body("NotTheCurrentPlayer");
        }

        if let Some(err) = game.act(action).err() {
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
            game_state.actions.push(info.0);
            HttpResponse::Ok().body("")
        }
    } else {
        HttpResponse::NotFound().body("Game not found")
    }
}

fn create(game: web::Path<String>, info: GameState, data: web::Data<AppState>) {
    let mut games = data.games.lock().unwrap();
    let name = game.into_inner();
    games.insert(name, info);
}

#[put("/game/{game}")]
async fn create_with(
    game: web::Path<String>,
    info: web::Json<GameState>,
    data: web::Data<AppState>,
) -> impl Responder {
    create(game, info.into_inner(), data);
    HttpResponse::Ok()
}

#[post("/game/{game}")]
async fn create_without(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    create(game, GameState::random(), data);
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
            .service(trigger_bot)
            .service(create_with)
            .service(actix_files::Files::new("/", "./www/build").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
