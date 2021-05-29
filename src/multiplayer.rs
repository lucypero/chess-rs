use std::{net::{TcpListener, TcpStream}, time::Duration};
use serde::{Serialize, Deserialize};
use bincode::Options;
use std::io::{Read, Write};
use std::{thread, time};
use std::sync::mpsc::{self, Sender, Receiver};

use crate::{chess::{ChessPiece, ChessTeam, GameState, Move, Tile}, graphics::{
        GfxState,
        PlayerInput
    }};

pub struct MPState {
    is_host : bool,
    team: ChessTeam,
    game: GameState,
    gfx_state: GfxState,
    rx_recv: Receiver<Message>,
    tx_send: Sender<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Move(Move),
}

impl MPState {

    pub fn init(is_host: bool, ip: String) -> MPState {
        let mut  game = GameState::init();

        let flipped_board;
        if is_host {
            flipped_board = false;
        } else {
            flipped_board = true;
        }

        let mut gfx_state = GfxState::init(&mut game, flipped_board);

        let mut tcp_stream_op = None;

        //gotta connect etc
        if is_host {
            let listener = TcpListener::bind(ip).unwrap();
            // let listener = TcpListener::bind("0.0.0.0:3333").unwrap();

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
            // match TcpStream::connect("localhost:3333") {
            match TcpStream::connect(ip) {
                Ok(stream) => {
                    println!("Successfully connected to server in port 3333");
                    tcp_stream_op = Some(stream);
                },
                Err(e) => {
                    panic!("error atconnecting {}", e);
                }
            }
        }

        let stream = tcp_stream_op.unwrap();
        let mut stream1 = stream.try_clone().unwrap();
        let mut stream2 = stream.try_clone().unwrap();

        let (tx_send, rx_send): (Sender<Message>, Receiver<Message>) = mpsc::channel();

        //thread that sends the messages
        thread::spawn(move || {
            let my_options = bincode::DefaultOptions::new()
                .with_fixint_encoding()
                .allow_trailing_bytes();

            loop {
                let msg_1 = rx_send.recv().unwrap();
                // get value and send it through the socket
                let msg_1_encoded: Vec<u8> = my_options.serialize(&msg_1).unwrap();

                //how do i use the stream here.. oh boy
                // TODO(lucypero): you are here

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
        let (tx_recv, rx_recv): (Sender<Message>, Receiver<Message>)  = mpsc::channel();
        thread::spawn(move || {

            let message_size;
            //this just calculates a move size
            {
                let the_move = Move::PieceMove {
                    piece: ChessPiece::Queen,
                    tile_from: Tile::A1,
                    tile_to: Tile::A2,
                    is_en_passant: false
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
                let mut msg_buffer: Vec<u8> = vec![0;message_size];
                stream2.read(&mut msg_buffer);

                let msg_decoded : Message = bincode::deserialize(&msg_buffer).unwrap();
                tx_recv.send(msg_decoded).unwrap();
            }
        });

        let team = if is_host {
            ChessTeam::White
        } else {
            ChessTeam::Black
        };

        gfx_state.set_team_lock(Some(team.the_other_one()));

        MPState{is_host, game, gfx_state, rx_recv, tx_send, team }
    }

    // pub fn init_with_game(is_host: bool, mut game:GameState) -> MPState {
    //     let gfx_state = GfxState::init(&mut game);
    //     MPState{is_host, game, gfx_state}
    // }

    fn send_move(&mut self, the_move: Move) {

        let my_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let message = Message::Move(the_move);

        // let msg_encoded: Vec<u8> = my_options.serialize(&message).unwrap();
        self.tx_send.send(message).unwrap();
        // self.tcp_stream.write(&move_encoded).unwrap();
    }

    fn recieve_move_maybe(&mut self) -> Option<Move> {

        //read from channel
        // let read_res = self.tcp_stream.read(&mut msg_buffer);

        match self.rx_recv.try_recv() {
            Ok(message) => {match message {
                Message::Move(the_move) => { return Some(the_move)}
            }}
            Err(mpsc::TryRecvError::Empty) => {return None}
            Err(_) => {panic!();}
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
                PlayerInput::GoBack => {res = true;}
                PlayerInput::Move(chess_move, move_res) => {
                    //ok so here u do stuff with the move
                    // if u are the client u send the move to the server and stuff

                    // let move_res = self.game.perform_move(chess_move);
                    if let Ok(()) = move_res {

                        // self.gfx_state.move_was_made(&mut self.game);
                        println!("sent move {}", chess_move.clone());
                        self.send_move(chess_move);
                    }
                }
            }
        }

        res
    }
}

