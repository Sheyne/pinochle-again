use actix_cors::Cors;
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use base64::{engine::general_purpose::STANDARD, Engine};
use bitvec::prelude::*;
use pinochle::ai::Bot;
use pinochle::{Action, Error, Game, Phase, Player};
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct GameState {
    player_names: [String; 4],
    seed: [u8; 32],
    actions: Vec<Action>,
}

impl GameState {
    fn random() -> Self {
        Self {
            player_names: [
                "A".to_owned(),
                "B".to_owned(),
                "C".to_owned(),
                "D".to_owned(),
            ],
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

    fn to_bytes(&self) -> Vec<u8> {
        let mut actions = bitvec![u8, Lsb0; 0; 8];
        actions.extend_from_bitslice(self.player_names[0].as_bits::<Lsb0>());
        actions.extend_from_bitslice(bits!(0; 8));
        actions.extend_from_bitslice(self.player_names[1].as_bits::<Lsb0>());
        actions.extend_from_bitslice(bits!(0; 8));
        actions.extend_from_bitslice(self.player_names[2].as_bits::<Lsb0>());
        actions.extend_from_bitslice(bits!(0; 8));
        actions.extend_from_bitslice(self.player_names[3].as_bits::<Lsb0>());
        actions.extend_from_bitslice(bits!(0; 8));
        actions.extend_from_bitslice(self.seed.as_bits::<Lsb0>());
        actions.extend_from_bitslice((self.actions.len() as u32).to_le_bytes().as_bits::<Lsb0>());
        let mut game = Game::new(StdRng::from_seed(self.seed));
        for action in &self.actions {
            action.encode(&mut actions, &game);
            game.act(action.clone()).unwrap();
        }
        actions.into_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        fn get_str(bytes: &[u8]) -> Option<(&str, &[u8])> {
            let idx = bytes.iter().position(|x| *x == 0)?;
            Some((
                std::str::from_utf8(&bytes[..idx]).ok()?,
                &bytes[(idx + 1)..],
            ))
        }
        let _version = bytes[0];
        let bytes = &bytes[1..];

        let (a, bytes) = get_str(bytes)?;
        let (b, bytes) = get_str(bytes)?;
        let (c, bytes) = get_str(bytes)?;
        let (d, bytes) = get_str(bytes)?;

        let seed = bytes.get(0..32)?.try_into().ok()?;
        let bytes = &bytes[32..];
        let length = bytes.get(0..4)?;
        let length = u32::from_le_bytes(length.try_into().ok()?);
        let bytes = &bytes[4..];
        let mut actions = vec![];
        let mut game = Game::new(StdRng::from_seed(seed));
        let mut bits = bytes.as_bits::<Lsb0>();
        while actions.len() < length as usize {
            let (new_bits, action) = Action::decode(bits, &game)?;
            actions.push(action.clone());
            game.act(action).ok()?;
            if let Some(new_bits) = new_bits {
                bits = new_bits;
            } else {
                break;
            }
        }
        Some(Self {
            player_names: [
                if a.len() == 0 { "A" } else { a }.to_owned(),
                if b.len() == 0 { "B" } else { b }.to_owned(),
                if c.len() == 0 { "C" } else { c }.to_owned(),
                if d.len() == 0 { "D" } else { d }.to_owned(),
            ],
            actions,
            seed,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameInfo {
    player_names: [String; 4],
    first_bidder: Player,
    current_player: Player,
    phase: Phase,
    scores: [i32; 2],
}

impl GameInfo {
    fn from_game<R: Rng>(player_names: &[String; 4], game: &Game<R>) -> Self {
        GameInfo {
            player_names: player_names.clone(),
            first_bidder: game.first_bidder(),
            current_player: game.current_player(),
            phase: game.phase().clone(),
            scores: game.scores(),
        }
    }
}

impl<R: Rng> From<&Game<R>> for GameInfo {
    fn from(value: &Game<R>) -> Self {
        Self::from_game(&Default::default(), value)
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
        let info = GameInfo::from_game(&game.player_names, &game.game());
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

#[get("/game/{game}/base64")]
async fn get_b64_game(game: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let games = data.games.lock().unwrap();
    let name = game.into_inner();

    if let Some(game) = games.get(&name) {
        HttpResponse::Ok().body(STANDARD.encode(game.to_bytes()))
    } else {
        HttpResponse::NotFound().body("")
    }
}

#[put("/game/{game}/base64")]
async fn create_with_b64(
    game_name: web::Path<String>,
    info: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    if let Some(game) = STANDARD
        .decode(info)
        .ok()
        .and_then(|bytes| GameState::from_bytes(&bytes))
    {
        create(game_name, game, data);
        HttpResponse::Ok()
    } else {
        HttpResponse::NotAcceptable()
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
            if let Phase::Play(playing_phase) = game.phase() {
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

#[put("/game/{game}/{player}/name")]
async fn set_name(
    game: web::Path<(String, Player)>,
    info: web::Bytes,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut games = data.games.lock().unwrap();
    let (name, player) = game.into_inner();
    if let Some(game_state) = games.get_mut(&name) {
        match String::from_utf8(info.into()) {
            Ok(name) => {
                game_state.player_names[player as usize] = name;
                HttpResponse::Ok().body("")
            }
            Err(err) => HttpResponse::NotAcceptable().body(format!("{err}")),
        }
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
            .service(set_name)
            .service(get_full_game)
            .service(get_game)
            .service(get_games)
            .service(get_hand)
            .service(get_b64_game)
            .service(create_with_b64)
            .service(create_without)
            .service(trigger_bot)
            .service(create_with)
            .service(actix_files::Files::new("/game/{name}/{player}", "./www/build").index_file("index.html"))
            .service(actix_files::Files::new("/", "./www/build").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
