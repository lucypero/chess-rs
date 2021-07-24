#![warn(rust_2018_idioms)]
#![allow(dead_code)]
//#![windows_subsystem = "windows"]

mod graphics;
mod multiplayer;

use crate::multiplayer::MPState;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

fn get_mq_conf() -> macroquad::prelude::Conf {
    graphics::get_mq_conf()
}

pub struct Audio {
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    audio_data: HashMap<String, Arc<[u8]>>,
}

impl Audio {
    fn init() -> Audio {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        // let file = File::open("assets/sounds/standard/Berserk.ogg").unwrap();

        let mut audio_data: HashMap<String, Arc<[u8]>> = HashMap::new();

        let paths = std::fs::read_dir("assets/sounds/standard/").unwrap();

        for path in paths {
            let p = path.unwrap();
            let p_file_name = p.file_name();
            let file_name = std::path::Path::new(&p_file_name)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            let audio_file: Vec<u8> = std::fs::read(p.path()).unwrap();
            audio_data.insert(file_name.to_string(), audio_file.into());
        }

        Audio {
            stream,
            stream_handle,
            audio_data,
        }
    }

    fn play_sound(&self, name: &str) {
        // play_audio(&stream_handle, audio_arr.get("Move").unwrap().clone());
        // fn play_audio(stream_handle: &rodio::OutputStreamHandle, audio: Arc<[u8]>)
        //getting arc
        let audio = self.audio_data.get(name).unwrap().clone();
        let c = std::io::Cursor::new(audio);
        let beep1 = self.stream_handle.play_once(c).unwrap();
        beep1.set_volume(1.);
        beep1.detach();
    }
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
    Main,
    PlayMenu { fen_string: String },
    OptionsMenu,
}

pub enum GameState {
    MainMenu(MainMenuState),
    SinglePlayer(chess::GameState, graphics::GfxState),
    MultiplayerSession(MPState),
}

impl GameState {
    //called every time game state is switched

    fn init_mm() -> GameState {
        GameState::MainMenu(MainMenuState::Main {})
    }

    fn swap_to_in_game(&mut self, mut game: chess::GameState, audio: Rc<Audio>) {
        let gfx_state = graphics::GfxState::init(&mut game, None, audio);
        *self = GameState::SinglePlayer(game, gfx_state);
    }

    fn swap_to_mm(&mut self) {
        *self = GameState::init_mm();
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

    // audio_arr.insert(AudioFiles::Move, audio_file.into());

    // #[derive(PartialEq, Eq, Hash)]
    // enum AudioFiles {
    //    Move,
    //    Draw,
    //    Defeat
    // }

    let audio = Audio::init();
    let audio_p = Rc::new(audio);

    // let c = std::io::Cursor::new(a_p);
    let mut game_state = GameState::init_mm();

    loop {
        game_loop(&mut game_state, audio_p.clone());
        macroquad::prelude::next_frame().await;
    }
}

pub enum MenuChange {
    Menu(MainMenuState),
    Game(chess::GameState),
    MultiplayerGame(MPState),
    None,
}

// async fn game_loop(game: &mut GameState, gfx_state: &mut graphics::GfxState) {
fn game_loop(game_state: &mut GameState, audio: Rc<Audio>) {
    match game_state {
        GameState::MainMenu(mm_s) => match graphics::draw_main_menu(mm_s, audio.clone()) {
            MenuChange::Menu(menu) => {
                *game_state = GameState::MainMenu(menu);
            }
            MenuChange::Game(gs) => {
                game_state.swap_to_in_game(gs, audio.clone());
            }
            MenuChange::MultiplayerGame(mp_state) => {
                game_state.swap_to_multiplayer(mp_state);
            }
            MenuChange::None => {}
        },
        GameState::SinglePlayer(game, gfx_state) => {
            gfx_state.draw(game);
            let player_input = gfx_state.consume_player_input_buffer();

            if let Some(pl_input) = player_input {
                match pl_input {
                    graphics::PlayerInput::GoBack => {
                        game_state.swap_to_mm();
                    }
                    graphics::PlayerInput::Move(_chess_move, _move_res) => {
                        //ok so here u do stuff with the move
                        // if u are the client u send the move to the server and stuff
                        // let move_res = game.perform_move(chess_move);
                        // if let Ok(()) = move_res {
                        // gfx_state.move_was_made(game);
                        // }
                    }
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
