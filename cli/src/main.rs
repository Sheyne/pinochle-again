use pinochle::ai::Bot;
use pinochle::{Action, Game, GameInfo, Phase, Player};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    let seed: [u8; 32] = serde_json::from_str(include_str!("../../seed.json")).unwrap();
    let actions: Vec<(Player, Action)> =
        serde_json::from_str(include_str!("../../actions.json")).unwrap();

    let mut game = Game::new(StdRng::from_seed(seed));
    for (player, action) in &actions {
        game.act(*player, action.clone()).unwrap();
    }
    let bot_player = game.current_player();

    let mut game = Game::new(StdRng::from_seed(seed));
    let mut bot: Option<Bot> = None;

    for (current_player, action) in &actions {
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
                    let player_hand = game.player_hand(*current_player);

                    bot.update(
                        *current_player,
                        player_hand[*card],
                        playing_phase.trump,
                        &playing_phase.trick.cards,
                    );
                }
            }
        }
        game.act(*current_player, action.clone()).unwrap();
    }
    let chosen_card = &bot.as_ref().unwrap().get_move();
    dbg!(chosen_card);
    let action = Action::Play(
        game.player_hand(bot_player)
            .iter()
            .position(|x| x == chosen_card)
            .unwrap(),
    );

    game.act(bot_player, action).unwrap();
}
