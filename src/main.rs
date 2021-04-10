#![warn(rust_2018_idioms)]
#![allow(dead_code)]

mod chess;
mod move_parser;
mod graphics;

use chess::GameState;

fn get_mq_conf() -> macroquad::prelude::Conf {
    graphics::get_mq_conf()
}

#[macroquad::main(get_mq_conf)]
async fn main() {
    // test scenarios
    let args: Vec<String> = std::env::args().collect();
    let test_game = chess::get_test(args);


    //initializing game state
    let mut game = if let Some(test_game) = test_game {
        test_game
    } else {
        GameState::init()
    };

    // game_cli_loop(&mut game);

    let mut gfx_state = graphics::GfxState::init(&game).await;

    loop {
        game_loop(&mut game, &mut gfx_state).await;
        macroquad::prelude::next_frame().await
    }
}

async fn game_loop(game: &mut GameState, gfx_state: &mut graphics::GfxState) {
    gfx_state.draw(game).await
}
