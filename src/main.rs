#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod chess;
mod graphics;
mod move_parser;
mod multiplayer;

use crate::multiplayer::MPState;
use futures::executor::block_on;


fn get_mq_conf() -> macroquad::prelude::Conf {
    graphics::get_mq_conf()
}

struct Args {
    test: Option<chess::GameState>,
    fen: Option<chess::GameState>,
}

fn parse_args(args: Vec<String>) -> Args {
    fn safe_index(vec: &Vec<String>, i: usize) -> Result<String, ()> {
        if i >= vec.len() {
            Err(())
        } else {
            Ok(vec[i].clone())
        }
    }

    let mut args_p = Args {
        test: None,
        fen: None,
    };

    let mut i = 1;

    loop {
        if args.len() <= i {
            break;
        }

        let arg = args[i].as_str();

        match arg {
            "--test" | "-t" => {
                i += 1;
                let test_arg = safe_index(&args, i).expect("specify the test");
                args_p.test = chess::get_test(test_arg);
                // let test_arg = args[i+1].as_str();
            }
            "--fen" | "-f" => {
                i += 1;
                let fen_arg = safe_index(&args, i).expect("specify the fen");
                args_p.fen = chess::parse_fen(fen_arg);
            }
            _ => {
                panic!("invalid argument: {}", arg);
            }
        }

        i += 1;
    }

    args_p
}

pub enum MainMenuState {
    Main{ ip_string : String },
    PlayMenu{ fen_string: String },
    OptionsMenu
}

pub enum GameState {
    MainMenu(MainMenuState),
    InGame(chess::GameState, graphics::GfxState),
    MultiplayerSession(MPState)
}

impl GameState {
    //called every time game state is switched
    fn init_mm() -> GameState {
        GameState::MainMenu(MainMenuState::Main{ip_string:"0.0.0.0:3333".to_string()})
    }

    fn swap_to_in_game(&mut self, mut game : chess::GameState) {
        let gfx_state = graphics::GfxState::init(&mut game, false);
        *self = GameState::InGame(game, gfx_state);
    }

    fn swap_to_mm(&mut self) {
        *self = GameState::MainMenu(MainMenuState::Main{ip_string:"0.0.0.0:3333".to_string()})
    }

    fn swap_to_multiplayer(&mut self, mp_state: MPState) {
        *self = GameState::MultiplayerSession(mp_state);
    }
}

#[macroquad::main(get_mq_conf)]
async fn main() {
    // test scenarios
    // let args: Vec<String> = std::env::args().collect();
    // let args = parse_args(args);

    //initializing game state
    // let mut game = if let Some(fen_game) = args.fen {
    //     fen_game
    // } else if let Some(test_game) = args.test {
    //     test_game
    // } else {
    //     chess::GameState::init()
    // };

    let mut game_state = GameState::init_mm();

    loop {
        game_loop(&mut game_state);
        macroquad::prelude::next_frame().await;
    }
}

pub enum MenuChange {
    Menu(MainMenuState),
    Game(chess::GameState),
    MultiplayerGame(MPState),
    None
}

// async fn game_loop(game: &mut GameState, gfx_state: &mut graphics::GfxState) {
fn game_loop(game_state : &mut GameState) {

    match game_state {
        GameState::MainMenu(mm_s) => {

            match graphics::draw_main_menu(mm_s) {
                MenuChange::Menu(menu) => {
                    *game_state = GameState::MainMenu(menu);
                }
                MenuChange::Game(gs) => {
                    game_state.swap_to_in_game(gs);
                }
                MenuChange::MultiplayerGame(mp_state) => {
                    game_state.swap_to_multiplayer(mp_state);
                }
                MenuChange::None => {

                }
            }
        }
        GameState::InGame(game, gfx_state) => {
            let player_input = gfx_state.draw(game);
            if let Some(pl_input) = player_input {
                match pl_input {
                    graphics::PlayerInput::GoBack => {
                        game_state.swap_to_mm();
                    }
                    graphics::PlayerInput::Move(_) => {}
                }
            }
        }
        GameState::MultiplayerSession(mp_state) => {
            let go_back = mp_state.mp_loop();
            if go_back {
                game_state.swap_to_mm();
            }
        }
    }

}
