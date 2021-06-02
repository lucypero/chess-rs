use bincode::Options;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use chess::{ChessPiece, ChessTeam, GameState, Move, Tile};

use crate::graphics::{GfxState, PlayerInput};

pub struct MPState {
    team: ChessTeam,
    game: GameState,
    gfx_state: GfxState,
    rx_recv: Receiver<Message>,
    tx_send: Sender<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    GameStart(chess::ChessTeam),
    Move(Move),
}

impl MPState {
    //connect to server and wait for game start
    pub fn init(ip: String) -> MPState {
        let mut game = GameState::init();

        let tcp_stream_op;

        match TcpStream::connect(ip) {
            Ok(stream) => {
                println!("Successfully connected to server");
                tcp_stream_op = stream;
            }
            Err(e) => {
                panic!("error atconnecting {}", e);
            }
        }

        let mut stream1 = tcp_stream_op;
        let mut stream2 = stream1.try_clone().unwrap();

        let (tx_send, rx_send): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        //thread that sends the messages
        thread::spawn(move || {
            let my_options = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .allow_trailing_bytes();

            loop {
                let msg_1 = rx_send.recv().unwrap();
                let msg_1_encoded: Vec<u8> = my_options.serialize(&msg_1).unwrap();

                stream1.write(&msg_1_encoded).unwrap();
                // match stream1.write(&msg_1_encoded) {
                //     Ok(_) => { }
                //     Err(e) => {
                //         println!("broken pipe... attempting to reconnect..., {}", e);
                //     }
                // }
            }
        });

        //thread that recvs the messages
        let (tx_recv, rx_recv): (Sender<Message>, Receiver<Message>) = mpsc::channel();
        thread::spawn(move || {
            let message_size;
            //this just calculates a move size
            {
                let the_move = Move::PieceMove {
                    piece: ChessPiece::Queen,
                    tile_from: Tile::A1,
                    tile_to: Tile::A2,
                    is_en_passant: false,
                };

                let message = Message::Move(the_move);

                let my_options = bincode::DefaultOptions::new()
                    .with_fixint_encoding()
                    .allow_trailing_bytes();

                let move_encoded: Vec<u8> = my_options.serialize(&message).unwrap();
                message_size = move_encoded.len();
            }

            //recv value from socket and send it thru channel
            loop {
                let mut msg_buffer: Vec<u8> = vec![0; message_size];
                stream2
                    .read(&mut msg_buffer)
                    .expect("error while reading socket");

                let msg_decoded: Message = bincode::deserialize(&msg_buffer).unwrap();
                tx_recv.send(msg_decoded).unwrap();
            }
        });

        let team;

        loop {
            match rx_recv.recv() {
                Ok(message) => match message {
                    Message::GameStart(the_team) => {
                        team = the_team;
                        println!("Game started!!! team is {:?}", team);
                        break;
                    }
                    Message::Move(the_move) => {
                        println!("recieved move! but game didn's start yet!?? {:?}", the_move);
                    }
                },
                Err(_) => {
                    panic!("recv error");
                }
            }
        }

        let gfx_state = GfxState::init(&mut game, Some(team));

        let game = chess::GameState::init();

        MPState {
            game,
            gfx_state,
            rx_recv,
            tx_send,
            team,
        }
    }

    fn send_move(&mut self, the_move: Move) {
        let message = Message::Move(the_move);
        self.tx_send.send(message).unwrap();
    }

    fn recieve_move_maybe(&mut self) -> Option<Move> {
        //read from channel
        match self.rx_recv.try_recv() {
            Ok(message) => match message {
                Message::GameStart(_) => {
                    println!("games start recieved.. this shouldn't happen");
                    return None;
                }
                Message::Move(the_move) => return Some(the_move),
            },
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(_) => {
                panic!();
            }
        }
    }

    //true to go back to menu
    pub fn mp_loop(&mut self) -> bool {
        let mut res = false;

        let the_move = self.recieve_move_maybe();
        if let Some(some_move) = the_move {
            println!("recieved a move omg {}", some_move.clone());
            if let Ok(()) = self.game.perform_move(some_move) {
                self.gfx_state.move_was_made(&mut self.game);
            }
        }

        let player_input = self.gfx_state.draw(&mut self.game);

        if let Some(input) = player_input {
            match input {
                PlayerInput::GoBack => {
                    res = true;
                }
                PlayerInput::Move(chess_move, move_res) => {
                    if let Ok(()) = move_res {
                        println!("sent move {}", chess_move.clone());
                        self.send_move(chess_move);
                    }
                }
            }
        }

        res
    }
}
