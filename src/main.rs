use ansi_term::Colour;
use ansi_term::Style;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::env;
use std::fmt;
use std::io::{self, BufRead};
use std::ops;
use std::str::FromStr;

mod move_parser;

#[derive(Debug, Eq, PartialEq)]
enum ChessTeam {
    Black,
    White,
}

impl fmt::Display for ChessTeam {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            ChessTeam::Black => "Black",
            ChessTeam::White => "White",
        };

        write!(f, "{}", disp)
    }
}

#[derive(Eq, PartialEq)]
enum ChessPiece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

// Represents coordinates and distances on the board
//  the origin (0,0) is at the bottom left corner of the board (A1)
#[derive(Clone, Copy)]
struct Coord {
    x: i32,
    y: i32,
}

impl From<Tile> for Coord {
    fn from(tile: Tile) -> Self {
        match tile {
            Tile::A1 => Coord { x: 0, y: 0 },
            Tile::A2 => Coord { x: 0, y: 1 },
            Tile::A3 => Coord { x: 0, y: 2 },
            Tile::A4 => Coord { x: 0, y: 3 },
            Tile::A5 => Coord { x: 0, y: 4 },
            Tile::A6 => Coord { x: 0, y: 5 },
            Tile::A7 => Coord { x: 0, y: 6 },
            Tile::A8 => Coord { x: 0, y: 7 },

            Tile::B1 => Coord { x: 1, y: 0 },
            Tile::B2 => Coord { x: 1, y: 1 },
            Tile::B3 => Coord { x: 1, y: 2 },
            Tile::B4 => Coord { x: 1, y: 3 },
            Tile::B5 => Coord { x: 1, y: 4 },
            Tile::B6 => Coord { x: 1, y: 5 },
            Tile::B7 => Coord { x: 1, y: 6 },
            Tile::B8 => Coord { x: 1, y: 7 },

            Tile::C1 => Coord { x: 2, y: 0 },
            Tile::C2 => Coord { x: 2, y: 1 },
            Tile::C3 => Coord { x: 2, y: 2 },
            Tile::C4 => Coord { x: 2, y: 3 },
            Tile::C5 => Coord { x: 2, y: 4 },
            Tile::C6 => Coord { x: 2, y: 5 },
            Tile::C7 => Coord { x: 2, y: 6 },
            Tile::C8 => Coord { x: 2, y: 7 },

            Tile::D1 => Coord { x: 3, y: 0 },
            Tile::D2 => Coord { x: 3, y: 1 },
            Tile::D3 => Coord { x: 3, y: 2 },
            Tile::D4 => Coord { x: 3, y: 3 },
            Tile::D5 => Coord { x: 3, y: 4 },
            Tile::D6 => Coord { x: 3, y: 5 },
            Tile::D7 => Coord { x: 3, y: 6 },
            Tile::D8 => Coord { x: 3, y: 7 },

            Tile::E1 => Coord { x: 4, y: 0 },
            Tile::E2 => Coord { x: 4, y: 1 },
            Tile::E3 => Coord { x: 4, y: 2 },
            Tile::E4 => Coord { x: 4, y: 3 },
            Tile::E5 => Coord { x: 4, y: 4 },
            Tile::E6 => Coord { x: 4, y: 5 },
            Tile::E7 => Coord { x: 4, y: 6 },
            Tile::E8 => Coord { x: 4, y: 7 },

            Tile::F1 => Coord { x: 5, y: 0 },
            Tile::F2 => Coord { x: 5, y: 1 },
            Tile::F3 => Coord { x: 5, y: 2 },
            Tile::F4 => Coord { x: 5, y: 3 },
            Tile::F5 => Coord { x: 5, y: 4 },
            Tile::F6 => Coord { x: 5, y: 5 },
            Tile::F7 => Coord { x: 5, y: 6 },
            Tile::F8 => Coord { x: 5, y: 7 },

            Tile::G1 => Coord { x: 6, y: 0 },
            Tile::G2 => Coord { x: 6, y: 1 },
            Tile::G3 => Coord { x: 6, y: 2 },
            Tile::G4 => Coord { x: 6, y: 3 },
            Tile::G5 => Coord { x: 6, y: 4 },
            Tile::G6 => Coord { x: 6, y: 5 },
            Tile::G7 => Coord { x: 6, y: 6 },
            Tile::G8 => Coord { x: 6, y: 7 },

            Tile::H1 => Coord { x: 7, y: 0 },
            Tile::H2 => Coord { x: 7, y: 1 },
            Tile::H3 => Coord { x: 7, y: 2 },
            Tile::H4 => Coord { x: 7, y: 3 },
            Tile::H5 => Coord { x: 7, y: 4 },
            Tile::H6 => Coord { x: 7, y: 5 },
            Tile::H7 => Coord { x: 7, y: 6 },
            Tile::H8 => Coord { x: 7, y: 7 },
        }
    }
}

// Operator overloading for coordinates
impl ops::Add<Coord> for Coord {
    type Output = Self;

    fn add(self, _rhs: Self) -> Self {
        Coord {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
        }
    }
}

impl ops::Sub<Coord> for Coord {
    type Output = Self;

    fn sub(self, _rhs: Self) -> Self {
        Coord {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
        }
    }
}

impl ops::Mul<Coord> for Coord {
    type Output = Self;

    fn mul(self, _rhs: Self) -> Self {
        Coord {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
        }
    }
}

impl ops::Div<Coord> for Coord {
    type Output = Self;

    fn div(self, _rhs: Self) -> Self {
        Coord {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
        }
    }
}

impl ops::Div<i32> for Coord {
    type Output = Self;

    fn div(self, _rhs: i32) -> Self {
        Coord {
            x: self.x / _rhs,
            y: self.y / _rhs,
        }
    }
}

impl ops::Mul<i32> for Coord {
    type Output = Self;

    fn mul(self, _rhs: i32) -> Self {
        Coord {
            x: self.x * _rhs,
            y: self.y * _rhs,
        }
    }
}

impl Coord {
    //  other - self
    fn distance(&self, other: Coord) -> Coord {
        Coord {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    // for now just return the highest component
    // Returns a value only if the vector is either a horizontal,
    //    vertical or diagonal line (←↑→↓↖↗↘↙), otherwise returns none
    fn magnitude(&self) -> Option<i32> {
        if self.x == 0 || //vertical
           self.y == 0 || //horizontal
           self.y.abs() == self.x.abs()
        // diagonal
        {
            Some(std::cmp::max(self.x.abs(), self.y.abs()))
        } else {
            None
        }
    }

    fn unit(&self) -> Self {
        *self / self.magnitude().unwrap()
    }
}

struct TeamedChessPiece(ChessTeam, ChessPiece);

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Hash, Copy, Clone)]
#[repr(u32)]
enum Tile {
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
}

impl fmt::Display for Tile {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let disp = match self {
            Tile::A1 => "A1",
            Tile::A2 => "A2",
            Tile::A3 => "A3",
            Tile::A4 => "A4",
            Tile::A5 => "A5",
            Tile::A6 => "A6",
            Tile::A7 => "A7",
            Tile::A8 => "A8",
            Tile::B1 => "B1",
            Tile::B2 => "B2",
            Tile::B3 => "B3",
            Tile::B4 => "B4",
            Tile::B5 => "B5",
            Tile::B6 => "B6",
            Tile::B7 => "B7",
            Tile::B8 => "B8",
            Tile::C1 => "C1",
            Tile::C2 => "C2",
            Tile::C3 => "C3",
            Tile::C4 => "C4",
            Tile::C5 => "C5",
            Tile::C6 => "C6",
            Tile::C7 => "C7",
            Tile::C8 => "C8",
            Tile::D1 => "D1",
            Tile::D2 => "D2",
            Tile::D3 => "D3",
            Tile::D4 => "D4",
            Tile::D5 => "D5",
            Tile::D6 => "D6",
            Tile::D7 => "D7",
            Tile::D8 => "D8",
            Tile::E1 => "E1",
            Tile::E2 => "E2",
            Tile::E3 => "E3",
            Tile::E4 => "E4",
            Tile::E5 => "E5",
            Tile::E6 => "E6",
            Tile::E7 => "E7",
            Tile::E8 => "E8",
            Tile::F1 => "F1",
            Tile::F2 => "F2",
            Tile::F3 => "F3",
            Tile::F4 => "F4",
            Tile::F5 => "F5",
            Tile::F6 => "F6",
            Tile::F7 => "F7",
            Tile::F8 => "F8",
            Tile::G1 => "G1",
            Tile::G2 => "G2",
            Tile::G3 => "G3",
            Tile::G4 => "G4",
            Tile::G5 => "G5",
            Tile::G6 => "G6",
            Tile::G7 => "G7",
            Tile::G8 => "G8",
            Tile::H1 => "H1",
            Tile::H2 => "H2",
            Tile::H3 => "H3",
            Tile::H4 => "H4",
            Tile::H5 => "H5",
            Tile::H6 => "H6",
            Tile::H7 => "H7",
            Tile::H8 => "H8",
        };

        write!(f, "{}", disp)
    }
}

impl TryFrom<Coord> for Tile {
    type Error = &'static str;

    fn try_from(coord: Coord) -> Result<Self, Self::Error> {
        match coord {
            Coord { x: 0, y: 0 } => Ok(Tile::A1),
            Coord { x: 0, y: 1 } => Ok(Tile::A2),
            Coord { x: 0, y: 2 } => Ok(Tile::A3),
            Coord { x: 0, y: 3 } => Ok(Tile::A4),
            Coord { x: 0, y: 4 } => Ok(Tile::A5),
            Coord { x: 0, y: 5 } => Ok(Tile::A6),
            Coord { x: 0, y: 6 } => Ok(Tile::A7),
            Coord { x: 0, y: 7 } => Ok(Tile::A8),

            Coord { x: 1, y: 0 } => Ok(Tile::B1),
            Coord { x: 1, y: 1 } => Ok(Tile::B2),
            Coord { x: 1, y: 2 } => Ok(Tile::B3),
            Coord { x: 1, y: 3 } => Ok(Tile::B4),
            Coord { x: 1, y: 4 } => Ok(Tile::B5),
            Coord { x: 1, y: 5 } => Ok(Tile::B6),
            Coord { x: 1, y: 6 } => Ok(Tile::B7),
            Coord { x: 1, y: 7 } => Ok(Tile::B8),

            Coord { x: 2, y: 0 } => Ok(Tile::C1),
            Coord { x: 2, y: 1 } => Ok(Tile::C2),
            Coord { x: 2, y: 2 } => Ok(Tile::C3),
            Coord { x: 2, y: 3 } => Ok(Tile::C4),
            Coord { x: 2, y: 4 } => Ok(Tile::C5),
            Coord { x: 2, y: 5 } => Ok(Tile::C6),
            Coord { x: 2, y: 6 } => Ok(Tile::C7),
            Coord { x: 2, y: 7 } => Ok(Tile::C8),

            Coord { x: 3, y: 0 } => Ok(Tile::D1),
            Coord { x: 3, y: 1 } => Ok(Tile::D2),
            Coord { x: 3, y: 2 } => Ok(Tile::D3),
            Coord { x: 3, y: 3 } => Ok(Tile::D4),
            Coord { x: 3, y: 4 } => Ok(Tile::D5),
            Coord { x: 3, y: 5 } => Ok(Tile::D6),
            Coord { x: 3, y: 6 } => Ok(Tile::D7),
            Coord { x: 3, y: 7 } => Ok(Tile::D8),

            Coord { x: 4, y: 0 } => Ok(Tile::E1),
            Coord { x: 4, y: 1 } => Ok(Tile::E2),
            Coord { x: 4, y: 2 } => Ok(Tile::E3),
            Coord { x: 4, y: 3 } => Ok(Tile::E4),
            Coord { x: 4, y: 4 } => Ok(Tile::E5),
            Coord { x: 4, y: 5 } => Ok(Tile::E6),
            Coord { x: 4, y: 6 } => Ok(Tile::E7),
            Coord { x: 4, y: 7 } => Ok(Tile::E8),

            Coord { x: 5, y: 0 } => Ok(Tile::F1),
            Coord { x: 5, y: 1 } => Ok(Tile::F2),
            Coord { x: 5, y: 2 } => Ok(Tile::F3),
            Coord { x: 5, y: 3 } => Ok(Tile::F4),
            Coord { x: 5, y: 4 } => Ok(Tile::F5),
            Coord { x: 5, y: 5 } => Ok(Tile::F6),
            Coord { x: 5, y: 6 } => Ok(Tile::F7),
            Coord { x: 5, y: 7 } => Ok(Tile::F8),

            Coord { x: 6, y: 0 } => Ok(Tile::G1),
            Coord { x: 6, y: 1 } => Ok(Tile::G2),
            Coord { x: 6, y: 2 } => Ok(Tile::G3),
            Coord { x: 6, y: 3 } => Ok(Tile::G4),
            Coord { x: 6, y: 4 } => Ok(Tile::G5),
            Coord { x: 6, y: 5 } => Ok(Tile::G6),
            Coord { x: 6, y: 6 } => Ok(Tile::G7),
            Coord { x: 6, y: 7 } => Ok(Tile::G8),

            Coord { x: 7, y: 0 } => Ok(Tile::H1),
            Coord { x: 7, y: 1 } => Ok(Tile::H2),
            Coord { x: 7, y: 2 } => Ok(Tile::H3),
            Coord { x: 7, y: 3 } => Ok(Tile::H4),
            Coord { x: 7, y: 4 } => Ok(Tile::H5),
            Coord { x: 7, y: 5 } => Ok(Tile::H6),
            Coord { x: 7, y: 6 } => Ok(Tile::H7),
            Coord { x: 7, y: 7 } => Ok(Tile::H8),

            _ => Err("This coordinate cannot be a tile."),
        }
    }
}

impl FromStr for Tile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A1" => Ok(Tile::A1),
            "A2" => Ok(Tile::A2),
            "A3" => Ok(Tile::A3),
            "A4" => Ok(Tile::A4),
            "A5" => Ok(Tile::A5),
            "A6" => Ok(Tile::A6),
            "A7" => Ok(Tile::A7),
            "A8" => Ok(Tile::A8),

            "B1" => Ok(Tile::B1),
            "B2" => Ok(Tile::B2),
            "B3" => Ok(Tile::B3),
            "B4" => Ok(Tile::B4),
            "B5" => Ok(Tile::B5),
            "B6" => Ok(Tile::B6),
            "B7" => Ok(Tile::B7),
            "B8" => Ok(Tile::B8),

            "C1" => Ok(Tile::C1),
            "C2" => Ok(Tile::C2),
            "C3" => Ok(Tile::C3),
            "C4" => Ok(Tile::C4),
            "C5" => Ok(Tile::C5),
            "C6" => Ok(Tile::C6),
            "C7" => Ok(Tile::C7),
            "C8" => Ok(Tile::C8),

            "D1" => Ok(Tile::D1),
            "D2" => Ok(Tile::D2),
            "D3" => Ok(Tile::D3),
            "D4" => Ok(Tile::D4),
            "D5" => Ok(Tile::D5),
            "D6" => Ok(Tile::D6),
            "D7" => Ok(Tile::D7),
            "D8" => Ok(Tile::D8),

            "E1" => Ok(Tile::E1),
            "E2" => Ok(Tile::E2),
            "E3" => Ok(Tile::E3),
            "E4" => Ok(Tile::E4),
            "E5" => Ok(Tile::E5),
            "E6" => Ok(Tile::E6),
            "E7" => Ok(Tile::E7),
            "E8" => Ok(Tile::E8),

            "F1" => Ok(Tile::F1),
            "F2" => Ok(Tile::F2),
            "F3" => Ok(Tile::F3),
            "F4" => Ok(Tile::F4),
            "F5" => Ok(Tile::F5),
            "F6" => Ok(Tile::F6),
            "F7" => Ok(Tile::F7),
            "F8" => Ok(Tile::F8),

            "G1" => Ok(Tile::G1),
            "G2" => Ok(Tile::G2),
            "G3" => Ok(Tile::G3),
            "G4" => Ok(Tile::G4),
            "G5" => Ok(Tile::G5),
            "G6" => Ok(Tile::G6),
            "G7" => Ok(Tile::G7),
            "G8" => Ok(Tile::G8),

            "H1" => Ok(Tile::H1),
            "H2" => Ok(Tile::H2),
            "H3" => Ok(Tile::H3),
            "H4" => Ok(Tile::H4),
            "H5" => Ok(Tile::H5),
            "H6" => Ok(Tile::H6),
            "H7" => Ok(Tile::H7),
            "H8" => Ok(Tile::H8),

            _ => Err("invalid tile".to_string()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Move {
    tile_from: Tile,
    tile_to: Tile,
}

impl fmt::Display for Move {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Move: {} to {}", self.tile_from, self.tile_to)
    }
}

struct GameState {
    _is_running: bool,
    moves: Vec<Move>,
}

impl GameState {
    fn init() -> GameState {
        GameState {
            _is_running: true,
            moves: vec![],
        }
    }

    fn move_count(&self) -> u32 {
        return self.moves.len() as u32;
    }

    //Returns the current board position
    fn get_board(&self) -> Board {
        //Start with the starting board position then you start mutating it with each
        //  move until you get the current position

        let mut board = Board::start_position();
        for chess_move in self.moves.iter() {
            board.apply_move(chess_move);
        }

        board
    }

    fn perform_move(&mut self, chess_move: Move) -> Result<(), String> {
        //performs all move validation here. If it is legal,
        //    the move is added to self.moves

        let board = self.get_board();

        // 1: Is the player grabbing a piece?
        let piece = board
            .get_piece(chess_move.tile_from)
            .ok_or("there is nothing at that tile!".to_string())?;

        // 2: Is the Player grabbing their own piece?
        if piece.0 != self.whose_turn() {
            return Err("hey! you can only grab your own pieces!".to_string());
        }

        // 3: Is the move legal according to how the piece moves?
        if !is_piece_move_legal(&piece, &chess_move, &board) {
            return Err("That piece does not move that way.".to_string());
        }

        //Everything is good. adding move to self.moves
        self.moves.push(chess_move);

        Ok(())
    }

    // whose turn is it?
    fn whose_turn(&self) -> ChessTeam {
        if self.moves.len() % 2 == 0 {
            ChessTeam::White
        } else {
            ChessTeam::Black
        }
    }
}

fn capitalize_coord_str(str: String) -> String {
    let mut v: Vec<char> = str.as_str().chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    v.into_iter().collect()
}

// TODO(lucypero): move this to game or board
//  only understands pawn moves for now

// This uses our move parser in move_parser::parse() then processes the output
//  It finds the right piece to move, and the destination tile, and constructs a Move
fn parse_move(mut move_input: String, game: &GameState) -> Result<Move, String> {
    //Basically:

    move_input.retain(|c| !c.is_whitespace());
    move_input = move_input.to_lowercase();

    let nodes = move_parser::parse(move_input.chars().collect());

    if nodes.is_err() {
        return Err("Move could not be parsed.".to_string());
    }

    //Processing parser output

    let board = game.get_board();

    println!("{:?}", nodes);

    Err("can't parse".to_string())
}

// Part of move validation. Validates chess piece move logic
fn is_piece_move_legal(piece: &TeamedChessPiece, chess_move: &Move, board: &Board) -> bool {
    let tile_from_coord = Coord::from(chess_move.tile_from);
    let tile_to_coord = Coord::from(chess_move.tile_to);

    let coord_distance = tile_from_coord.distance(tile_to_coord);

    match piece {
        TeamedChessPiece(team, ChessPiece::Pawn) => {
            //team factor (if black, it moves down, if white it moves up)
            let team_factor = match team {
                ChessTeam::White => 1,
                ChessTeam::Black => -1,
            };

            if coord_distance.y == 1 * team_factor && coord_distance.x == 0 {
                // NOTE(lucypero): going to have to do this some day
                // board.is_path_blocked(chess_move.tile_from, chess_move.tile_to)

                //check if there is a piece in the way
                let piece_option = board.get_piece(chess_move.tile_to);

                if piece_option.is_none() {
                    return true;
                }
            }

            //check if it is on the starting position
            let starting_position_y = match team {
                ChessTeam::White => 1,
                ChessTeam::Black => 6,
            };

            // if that is the case, it can move 2 tile units
            if starting_position_y == tile_from_coord.y
                && coord_distance.y == 2 * team_factor
                && coord_distance.x == 0
            {
                //check if there is a piece in the way(gotta check 2 squares)
                let piece_option = board.get_piece(
                    Tile::try_from(Coord {
                        x: tile_from_coord.x,
                        y: tile_from_coord.y + team_factor,
                    })
                    .unwrap(),
                );
                let piece_option_2 = board.get_piece(chess_move.tile_to);

                if piece_option.is_none() && piece_option_2.is_none() {
                    return true;
                }
            }

            // diagonal move when it takes a piece
            if coord_distance.x == 1 && coord_distance.y == 1 * team_factor {
                //get the targeted tile
                //if there is a piece and it is the other team's, the move is valid
                let target_piece_option = board.get_piece(chess_move.tile_to);

                if target_piece_option.is_some() {
                    let target_piece = target_piece_option.unwrap();
                    if *team != target_piece.0 {
                        return true;
                    }
                }
            }

            // TODO(lucypero): en passant
            //en passant rule (leave this for last.. this one will be complicated)

            false
        }
        TeamedChessPiece(_, ChessPiece::King) => {
            let magn = coord_distance.magnitude();

            //check movement
            if magn.is_none() {
                return false;
            }

            //magnitude has to be one
            let magn = magn.unwrap();
            if magn != 1 {
                return false;
            }

            //check if there is a friendly piece in the way
            if !board.is_path_clear(piece, chess_move) {
                return false;
            }

            true
        }
        TeamedChessPiece(_, ChessPiece::Rook) => {
            let magn = coord_distance.magnitude();

            //check movement
            if magn.is_none() {
                return false;
            }

            //line has to be a horizontal or vertical line
            if !(coord_distance.x == 0 || coord_distance.y == 0) {
                return false;
            }

            //check if there is a friendly piece in the way
            if !board.is_path_clear(piece, chess_move) {
                return false;
            }

            true
        }
        TeamedChessPiece(_, ChessPiece::Bishop) => {
            let magn = coord_distance.magnitude();

            //check movement
            if magn.is_none() {
                return false;
            }

            //line has to be a diagonal
            if !(coord_distance.x.abs() == coord_distance.y.abs()) {
                return false;
            }

            //check if there is a friendly piece in the way
            if !board.is_path_clear(piece, chess_move) {
                return false;
            }

            true
        }
        TeamedChessPiece(_, ChessPiece::Queen) => {
            let magn = coord_distance.magnitude();

            //check movement
            if magn.is_none() {
                return false;
            }

            // if magnitude returns something then it is a valid queen move

            //check if there is a friendly piece in the way
            if !board.is_path_clear(piece, chess_move) {
                return false;
            }

            true
        }
        TeamedChessPiece(_, ChessPiece::Knight) => {
            //checking if the move is an L

            if !((coord_distance.x.abs() == 2 && coord_distance.y.abs() == 1)
                || (coord_distance.x.abs() == 1 && coord_distance.y.abs() == 2))
            {
                return false;
            }

            //check if there is a friendly piece in the way
            if board.is_friendly_piece_at_destination(piece, chess_move) {
                return false;
            }

            true
        }
    }
}

// Describes a snapshot of the board on a given position
// Basically, what pieces there are and where they are
struct Board {
    piece_locations: HashMap<Tile, TeamedChessPiece>,
}

impl Board {
    fn start_position() -> Board {
        let mut piece_locations = HashMap::new();

        piece_locations.insert(
            Tile::A1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Rook),
        );
        piece_locations.insert(
            Tile::B1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Knight),
        );
        piece_locations.insert(
            Tile::C1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Bishop),
        );
        piece_locations.insert(
            Tile::D1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Queen),
        );
        piece_locations.insert(
            Tile::E1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::King),
        );
        piece_locations.insert(
            Tile::F1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Bishop),
        );
        piece_locations.insert(
            Tile::G1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Knight),
        );
        piece_locations.insert(
            Tile::H1,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Rook),
        );

        piece_locations.insert(
            Tile::A2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::B2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::C2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::D2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::E2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::F2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::G2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::H2,
            TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
        );

        piece_locations.insert(
            Tile::A8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Rook),
        );
        piece_locations.insert(
            Tile::B8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Knight),
        );
        piece_locations.insert(
            Tile::C8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Bishop),
        );
        piece_locations.insert(
            Tile::D8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Queen),
        );
        piece_locations.insert(
            Tile::E8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::King),
        );
        piece_locations.insert(
            Tile::F8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Bishop),
        );
        piece_locations.insert(
            Tile::G8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Knight),
        );
        piece_locations.insert(
            Tile::H8,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Rook),
        );

        piece_locations.insert(
            Tile::A7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::B7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::C7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::D7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::E7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::F7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::G7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );
        piece_locations.insert(
            Tile::H7,
            TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
        );

        Board { piece_locations }
    }

    fn apply_move(&mut self, chess_move: &Move) {
        // TODO(lucypero): error checking (if there is a piece there, if it is a legal move, etc)

        //after all the checking, perform the chess_move
        //get the piece at tile_from

        //rechess_move the piece from tile_from and put it on tile_to
        let piece = self.piece_locations.remove(&chess_move.tile_from).unwrap();
        self.piece_locations.insert(chess_move.tile_to, piece);
    }

    fn get_piece(&self, tile: Tile) -> Option<&TeamedChessPiece> {
        self.piece_locations.get(&tile)
    }

    // Checks if the path is clear for the piece.
    //   false if there is a friendly piece in (tile_from, tile_to]
    //   false if there is an enemy piece in (tile_from, tile_to) (not including tile_to)
    //   otherwise, true
    fn is_path_clear(&self, piece: &TeamedChessPiece, chess_move: &Move) -> bool {
        //ensure that the distance is a "good line", otherwise it makes no sense.
        let tile_from_coord = Coord::from(chess_move.tile_from);
        let tile_to_coord = Coord::from(chess_move.tile_to);

        let coord_distance = tile_from_coord.distance(tile_to_coord);
        let magn = coord_distance.magnitude().unwrap();

        //loop through the distance
        for i in 1..magn {
            //get direction to destination
            let coord_iter = tile_from_coord + (tile_to_coord - tile_from_coord).unit() * i;

            let tile_iter = Tile::try_from(coord_iter).unwrap();
            print!("is_path clear: is piece at {}? ", tile_iter);

            let piece_in_path_res = self.get_piece(tile_iter);

            println!("{}", piece_in_path_res.is_some());

            if piece_in_path_res.is_some() {
                return false;
            }
        }

        // finally, check if there is a friendly piece at the destination tile
        if self.is_friendly_piece_at_destination(piece, chess_move) {
            return false;
        }

        true
    }

    fn is_friendly_piece_at_destination(
        &self,
        piece: &TeamedChessPiece,
        chess_move: &Move,
    ) -> bool {
        let piece_dest = self.get_piece(chess_move.tile_to);

        if let Some(TeamedChessPiece(team, _)) = piece_dest {
            if *team == piece.0 {
                return true;
            }
        }

        false
    }

    fn print(&self) {
        println!();

        //print 8 tiles then new line
        for tile_num in 0..64u32 {
            let tile = Tile::try_from(tile_num).unwrap();
            let piece = self.get_piece(tile);
            let bg_style = Style::new().on(Colour::RGB(215, 135, 0));

            match piece {
                Some(TeamedChessPiece(team, piece)) => {
                    let used_style = match team {
                        ChessTeam::White => bg_style.fg(Colour::White),
                        ChessTeam::Black => bg_style.fg(Colour::Black),
                    };

                    match piece {
                        ChessPiece::Pawn => print!("{}", used_style.paint("P")),
                        ChessPiece::Rook => print!("{}", used_style.paint("R")),
                        ChessPiece::Knight => print!("{}", used_style.paint("N")),
                        ChessPiece::Bishop => print!("{}", used_style.paint("B")),
                        ChessPiece::Queen => print!("{}", used_style.paint("Q")),
                        ChessPiece::King => print!("{}", used_style.paint("K")),
                    }
                }
                None => {
                    print!("{}", bg_style.fg(Colour::RGB(178, 178, 178)).paint("-"));
                }
            }

            if (tile_num + 1) % 8 == 0 {
                println!();
            }
        }

        println!();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    //read move from stdin, you write the move and press enter
    println!("Welcome to chess! Type !help for all the commands");

    //initializing game state
    let mut game = GameState::init();
    game.get_board().print();
    print!("\n\nWhite to move. What's your move? ...\n");

    let stdin = io::stdin();
    for line_res in stdin.lock().lines() {
        //printing board:

        let mut line = line_res.unwrap();

        if line.len() <= 0 {
            println!("Type the move!");
            continue;
        }

        let first_character = line.chars().nth(0);

        //parsing commands
        if first_character.unwrap() == '!' {
            let command = line.split_off(1);
            match command.as_str() {
                "help" => {
                    print!("Welcome to chess!\n\nJust type the move you want to make!\n\nAvailable commands:\n\n!help - show this dialog\n!test: test command\n");
                }
                "move_count" => {
                    println!(
                        "{} moves were made. {} to move.",
                        game.move_count(),
                        "White"
                    );
                }
                "test" => {
                    println!("test command");
                }
                str => {
                    println!("{} command doesn't exist. Try again.", str);
                }
            }
        } else {
            // This should be a chess move

            //capitalize letters

            //trying to parse real chess move notation...

            //first try to understand E4

            // if it is just a coordinate, it means that it is a pawn move to that coordinate
            // you gotta get the pawn of that file
            // move from target tile downwards until you find the pawn

            //let tile_to_str = line.split_off(2);

            let parse_res = parse_move(line, &game);
            if parse_res.is_err() {
                println!("error while parsing move: {}", parse_res.unwrap_err());
                continue;
            }

            let the_move = parse_res.unwrap();

            // old way of parsing moves "E2E4" (it's bad)
            {
                // let tile_to_str = line.split_off(2);

                // let tile_from_res = Tile::from_str(capitalize_coord_str(line).as_str());
                // let tile_to_res = Tile::from_str(capitalize_coord_str(tile_to_str).as_str());

                // if !tile_from_res.is_ok() || !tile_to_res.is_ok() {
                //     println!("Move doesn't make sense. The tiles are wrong. Try again.");
                //     continue;
                // }

                // let tile_from = tile_from_res.unwrap();
                // let tile_to = tile_to_res.unwrap();

                // let the_move = Move {tile_from, tile_to};
            }

            let move_res = game.perform_move(the_move);
            if move_res.is_err() {
                println!("Error while attempting move: {}", move_res.unwrap_err());
                continue;
            }

            //move done successfully. move on to the next move
            //after move, print board again

            println!("{}", the_move);
            game.get_board().print();
            print!("\n\n{} to move. What's your move? ...\n", game.whose_turn());
        }
    }
}
