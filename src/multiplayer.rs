use std::{net::{TcpListener, TcpStream}, time::Duration};
use serde::{Serialize, Deserialize};
use bincode::Options;
use std::io::{Read, Write};

use crate::{
    chess::{
        GameState,
        Move,
        Tile,
        ChessPiece
    },
    graphics::{
        GfxState,
        PlayerInput
    }
};

pub struct MPState {
    is_host : bool,
    game: GameState,
    gfx_state: GfxState,
    tcp_stream: TcpStream
}

impl MPState {

    pub async fn init(is_host: bool) -> MPState {
        let mut  game = GameState::init();

        let flipped_board;
        if is_host {
            flipped_board = false;
        } else {
            flipped_board = true;
        }

        let gfx_state = GfxState::init(&mut game, flipped_board).await;

        let mut tcp_stream_op = None;

        //gotta connect etc
        if is_host {
            let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

            println!("Server listening on port 3333");

            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        println!("New connection: {}", stream.peer_addr().unwrap());
                        tcp_stream_op = Some(stream);
                        break;
                        // handle_client(stream)
                    }
                    Err(e) => {
                        panic!("error atconnecting {}", e);
                        /* connection failed */
                    }
                }
            }
            // close the socket server
            // drop(listener);
        }
        else {
            // side = Side::Client;
            match TcpStream::connect("localhost:3333") {
                Ok(stream) => {
                    println!("Successfully connected to server in port 3333");
                    tcp_stream_op = Some(stream);
                },
                Err(e) => {
                    panic!("error atconnecting {}", e);
                }
            }
        }

        let tcp_stream = tcp_stream_op.unwrap();
        tcp_stream.set_read_timeout(Some(Duration::new(0, 1000000)));

        MPState{is_host, game, gfx_state, tcp_stream}
    }

    // pub async fn init_with_game(is_host: bool, mut game:GameState) -> MPState {
    //     let gfx_state = GfxState::init(&mut game).await;
    //     MPState{is_host, game, gfx_state}
    // }
    fn send_move(&mut self, the_move: Move) {

        let my_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let move_encoded: Vec<u8> = my_options.serialize(&the_move).unwrap();
        self.tcp_stream.write(&move_encoded).unwrap();
    }

    fn recieve_move_maybe(&mut self) -> Option<Move> {

        let mut the_move_res = None;

        let move_1 = Move::PieceMove {
            piece: ChessPiece::Queen,
            tile_from: Tile::A1,
            tile_to: Tile::A2,
            is_en_passant: false
        };

        let my_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let move_encoded: Vec<u8> = my_options.serialize(&move_1).unwrap();
        let move_size = move_encoded.len();
   
        let mut msg_buffer: Vec<u8> = vec![0;move_size];
        let read_res = self.tcp_stream.read(&mut msg_buffer);

        let msg_decoded = bincode::deserialize(&msg_buffer);
        if let Ok(the_move) = msg_decoded {
            if read_res.is_ok() {
                println!("got move: {:?}", the_move);
                the_move_res = Some(the_move);
            }
        }

        the_move_res
    }

    //true to go back to menu
    pub async fn mp_loop(&mut self) -> bool {

        let mut res = false;

        let the_move = self.recieve_move_maybe();
        if let Some(some_move) = the_move {
            println!("recieved a move omg {}", some_move.clone());
            if let Ok(()) = self.game.perform_move(some_move) {
                self.gfx_state.move_was_made(&mut self.game);
            }
        }

        let player_input = self.gfx_state.draw(&mut self.game).await;
        
        if let Some(input) = player_input {
            match input {
                PlayerInput::GoBack => {res = true;}
                PlayerInput::Move(chess_move) => {
                    //ok so here u do stuff with the move
                    // if u are the client u send the move to the server and stuff

                    let move_res = self.game.perform_move(chess_move);
                    if let Ok(()) = move_res {

                        self.gfx_state.move_was_made(&mut self.game);
                        println!("sent move {}", chess_move.clone());
                        self.send_move(chess_move);
                    }
                }
            }
        }

        res
    }
}

