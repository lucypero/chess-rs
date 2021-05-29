#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use std::{net::{TcpListener, TcpStream}};
use serde::{Serialize, Deserialize};
use bincode::Options;
use std::io::{self, Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    GameStart(chess::ChessTeam),
    Move(chess::Move),
}

struct Match {
    clients: [TcpStream;2],
    game: chess::GameState,
}

fn main() {

    let mut clients:Vec<TcpStream> = Vec::with_capacity(2);

    let listener = TcpListener::bind("0.0.0.0:3333").unwrap();
    println!("Server listening on port 3333");

    let mut client_count = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                stream.set_nonblocking(true).expect("set_nonblocking call failed");
                clients.push(stream);
                client_count += 1;

                if client_count == 2 {
                    break;
                }
            }
            Err(e) => {
                println!("error atconnecting {}", e);
                /* connection failed */
            }
        }
    }

    // create game
    
    let game = chess::GameState::init();
    // let match = Match{clients: [clients[0], clients[1]], game};

    let my_options = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes();
    
    let msg = Message::GameStart(chess::ChessTeam::White);
    let msg_e: Vec<u8> = my_options.serialize(&msg).unwrap();

    //let them know
    clients[0].write(&msg_e).unwrap();


    let msg = Message::GameStart(chess::ChessTeam::Black);
    let msg_e: Vec<u8> = my_options.serialize(&msg).unwrap();

    clients[1].write(&msg_e).unwrap();


    let message_size;
    //this just calculates a move size
    {
        let the_move = chess::Move::PieceMove {
            piece: chess::ChessPiece::Queen,
            tile_from: chess::Tile::A1,
            tile_to: chess::Tile::A2,
            is_en_passant: false
        };

        let message = Message::Move(the_move);

        let my_options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let move_encoded: Vec<u8> = my_options.serialize(&message).unwrap();
        message_size = move_encoded.len();
    }


    //main loop
    //waiting for messages
    loop {

        let mut msg_buffer: Vec<u8> = vec![0;message_size];
        match clients[0].read(&mut msg_buffer) {
            Ok(_) => {
                //recieved message
                let msg_decoded : Message = bincode::deserialize(&msg_buffer).unwrap();
                println!("recved message from client 0: {:?}", msg_decoded);
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }


        match clients[1].read(&mut msg_buffer) {
            Ok(_) => {
                //recieved message
                let msg_decoded : Message = bincode::deserialize(&msg_buffer).unwrap();
                println!("recved message from client 1: {:?}", msg_decoded);
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
            }
            Err(e) => panic!("encountered IO error: {}", e),
        }

    }
}


