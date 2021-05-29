#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use chess::GameState;

fn main() {
    let game = GameState::init();
    println!("hello world");
}
