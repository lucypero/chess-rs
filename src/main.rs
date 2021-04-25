#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod chess;
mod graphics;
mod move_parser;

use chess::GameState;

fn get_mq_conf() -> macroquad::prelude::Conf {
    graphics::get_mq_conf()
}

struct Args {
    test: Option<GameState>,
    fen: Option<GameState>
}

fn parse_args(args : Vec<String>) -> Args {

    fn safe_index(vec: &Vec<String>, i: usize) -> Result<String, ()> {
        if i >= vec.len() {
            Err(())
        } else {
            Ok(vec[i].clone())
        }
    }

    let mut args_p = Args{test: None, fen: None};


    let mut i = 1;

    loop {
        if args.len() <= i {
            break;
        }

        let arg = args[i].as_str();

        match arg {
            "--test" | "-t" => {
                i+=1;
                let test_arg = safe_index(&args, i).expect("specify the test");
                args_p.test = chess::get_test(test_arg);
                // let test_arg = args[i+1].as_str();
            }
            "--fen" | "-f" => {
                i+=1;
                let fen_arg = safe_index(&args, i).expect("specify the fen");
                args_p.fen = chess::parse_fen(fen_arg);
            }
            _ => {
                panic!("invalid argument: {}", arg);
            }
        }

        i+=1;
    }

    args_p
}

#[macroquad::main(get_mq_conf)]
async fn main() {
    // test scenarios
    let args: Vec<String> = std::env::args().collect();
    let args = parse_args(args);

    //initializing game state
    let mut game = if let Some(fen_game) = args.fen {
        fen_game
    } else {
        GameState::init()
    };

    // game_cli_loop(&mut game);

    let mut gfx_state = graphics::GfxState::init(&mut game).await;

    loop {
        game_loop(&mut game, &mut gfx_state).await;
        macroquad::prelude::next_frame().await
    }
}

async fn game_loop(game: &mut GameState, gfx_state: &mut graphics::GfxState) {
    gfx_state.draw(game).await
}
