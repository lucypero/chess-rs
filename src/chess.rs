//ALl the actual chess game logic

use crate::move_parser;
use ansi_term::Colour;
use ansi_term::Style;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::io::{self, BufRead};
use std::ops;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChessTeam {
    Black,
    White,
}

impl ChessTeam {
    pub fn the_other_one(&self) -> ChessTeam {
        match self {
            ChessTeam::White => ChessTeam::Black,
            ChessTeam::Black => ChessTeam::White,
        }
    }
}

impl fmt::Display for ChessTeam {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let disp = match self {
            ChessTeam::Black => "Black",
            ChessTeam::White => "White",
        };

        write!(f, "{}", disp)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChessPiece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl fmt::Display for ChessPiece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let piece = match self {
            ChessPiece::Pawn => "Pawn",
            ChessPiece::Rook => "Rook",
            ChessPiece::Knight => "Knight",
            ChessPiece::Bishop => "Bishop",
            ChessPiece::Queen => "Queen",
            ChessPiece::King => "King",
        };

        write!(f, "{}", piece)
    }
}

// NOTE(lucypero): Most of these will never happen because most of these cases
//   are caught by MoveParseError (assuming we use chess move notation as input)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MoveError {
    // there is nothing at that tile!
    TileFromIsEmpty,
    // hey! you can only grab your own pieces!
    TileFromIsEnemyPiece,
    //That piece does not move that way.
    PieceDoesNotMoveLikeThat,
    //You must specify the promotion piece. e.g: e8=q
    PromotionPieceNotSpecified,
    //The pawn has to reach the back rank to promote.
    PromotionNotLegal,
    //You can't promote to a pawn or a king... try another piece
    PromotionWrongPiece,
    //Can't castle. The player has no castling rights
    CastlingNoRights,
    //Can't castle. The tiles in between are not free
    CastlingTilesInBetweenNotFree,
    //Can't castle while in or through check.
    CastlingThroughCheck,
    //Your King would be in check. King can't be in check.
    InCheck,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::TileFromIsEmpty => write!(f, "there is nothing at that tile!"),
            MoveError::TileFromIsEnemyPiece => write!(f, "hey! you can only grab your own pieces!"),
            MoveError::PieceDoesNotMoveLikeThat => write!(f, "That piece does not move that way."),
            MoveError::PromotionPieceNotSpecified => {
                write!(f, "You must specify the promotion piece. e.g: e8=q")
            }
            MoveError::PromotionNotLegal => {
                write!(f, "The pawn has to reach the back rank to promote.")
            }
            MoveError::PromotionWrongPiece => write!(
                f,
                "You can't promote to a pawn or a king... try another piece."
            ),
            MoveError::CastlingNoRights => {
                write!(f, "Can't castle. The player has no castling rights.")
            }
            MoveError::CastlingTilesInBetweenNotFree => {
                write!(f, "Can't castle. The tiles in between are not free.")
            }
            MoveError::CastlingThroughCheck => write!(f, "Can't castle while in or through check."),
            MoveError::InCheck => write!(f, "Your King would be in check. King can't be in check."),
        }
    }
}

// Represents coordinates and distances on the board
//  the origin (0,0) is at the bottom left corner of the board (A1)
#[derive(Clone, Copy, PartialEq)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
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
    pub fn distance(&self, other: Coord) -> Coord {
        Coord {
            x: other.x - self.x,
            y: other.y - self.y,
        }
    }

    // for now just return the highest component
    // Returns a value only if the vector is either a horizontal,
    //    vertical or diagonal line (←↑→↓↖↗↘↙), otherwise returns none
    pub fn magnitude(&self) -> Option<i32> {
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

    pub fn unit(&self) -> Self {
        *self / self.magnitude().unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct TeamedChessPiece(pub ChessTeam, pub ChessPiece);

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Hash, Copy, Clone)]
#[repr(u32)]
#[rustfmt::skip]
pub enum Tile {
    A8, B8, C8, D8, E8, F8, G8, H8, A7, B7, C7, D7, E7, F7, G7, H7, A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5, A4, B4, C4, D4, E4, F4, G4, H4, A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2, A1, B1, C1, D1, E1, F1, G1, H1,
}

#[rustfmt::skip]
impl fmt::Display for Tile {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let disp = match self {
            Tile::A1 => "A1", Tile::A2 => "A2", Tile::A3 => "A3", Tile::A4 => "A4", Tile::A5 => "A5",
            Tile::A6 => "A6", Tile::A7 => "A7", Tile::A8 => "A8", Tile::B1 => "B1", Tile::B2 => "B2",
            Tile::B3 => "B3", Tile::B4 => "B4", Tile::B5 => "B5", Tile::B6 => "B6", Tile::B7 => "B7",
            Tile::B8 => "B8", Tile::C1 => "C1", Tile::C2 => "C2", Tile::C3 => "C3", Tile::C4 => "C4",
            Tile::C5 => "C5", Tile::C6 => "C6", Tile::C7 => "C7", Tile::C8 => "C8", Tile::D1 => "D1",
            Tile::D2 => "D2", Tile::D3 => "D3", Tile::D4 => "D4", Tile::D5 => "D5", Tile::D6 => "D6",
            Tile::D7 => "D7", Tile::D8 => "D8", Tile::E1 => "E1", Tile::E2 => "E2", Tile::E3 => "E3",
            Tile::E4 => "E4", Tile::E5 => "E5", Tile::E6 => "E6", Tile::E7 => "E7", Tile::E8 => "E8",
            Tile::F1 => "F1", Tile::F2 => "F2", Tile::F3 => "F3", Tile::F4 => "F4", Tile::F5 => "F5",
            Tile::F6 => "F6", Tile::F7 => "F7", Tile::F8 => "F8", Tile::G1 => "G1", Tile::G2 => "G2",
            Tile::G3 => "G3", Tile::G4 => "G4", Tile::G5 => "G5", Tile::G6 => "G6", Tile::G7 => "G7",
            Tile::G8 => "G8", Tile::H1 => "H1", Tile::H2 => "H2", Tile::H3 => "H3", Tile::H4 => "H4",
            Tile::H5 => "H5", Tile::H6 => "H6", Tile::H7 => "H7", Tile::H8 => "H8",
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Move {
    PieceMove {
        piece: ChessPiece,
        tile_from: Tile,
        tile_to: Tile,
        is_en_passant: bool,
    },
    PieceMoveWithPromotion {
        tile_from: Tile,
        tile_to: Tile,
        promotion: ChessPiece,
    },
    CastleShort,
    CastleLong,
}

impl fmt::Display for Move {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::PieceMove {
                piece,
                tile_from,
                tile_to,
                is_en_passant,
            } => {
                let mut en_passant_str = "";
                if *is_en_passant {
                    en_passant_str = ", takes en passant";
                }
                write!(
                    f,
                    "Move: {} in {} to {}{}",
                    piece, tile_from, tile_to, en_passant_str
                )
            }
            Move::PieceMoveWithPromotion {
                tile_from,
                tile_to,
                promotion,
            } => {
                write!(
                    f,
                    "Move: Pawn in {} to {}, promoted to {}",
                    tile_from, tile_to, promotion
                )
            }
            Move::CastleShort => {
                write!(f, "Move: Short Castle")
            }
            Move::CastleLong => {
                write!(f, "Move: Long Castle")
            }
        }
    }
}

pub struct GameState {
    moves: Vec<Move>,
    starting_board: Board,
    cached_current_board: Option<Board>,
}

pub enum GameEndState {
    Checkmate,
    Stalemate,
    Running,
}

impl GameState {
    pub fn init() -> GameState {
        GameState {
            moves: vec![],
            starting_board: Board::start_position(),
            cached_current_board: None
        }
    }

    fn init_from_custom_position(board: Board) -> GameState {
        GameState {
            moves: vec![],
            starting_board: board,
            cached_current_board: None
        }
    }

    pub fn get_end_state(&mut self) -> GameEndState {
        // 1. check if team has any legal moves

        let mut has_legal_moves = false;

        let whose_turn = self.whose_turn();
        let last_move = self.get_last_move();

        let team_pieces = self.get_board().find_pieces_of_team(whose_turn);
        for piece in team_pieces {
            if !self.get_board()
                .get_legal_moves_of_piece_in_tile(piece.1, last_move)
                .unwrap()
                .is_empty()
            {
                has_legal_moves = true;
                break;
            }
        }

        //    1.1 if it does, game is Running
        if has_legal_moves {
            return GameEndState::Running;
        }

        // 2. if team is in check, it is checkmate and current team has lost
        //    2.2 if not, it is stalemate
        if self.get_board().is_team_in_check(whose_turn, last_move) {
            GameEndState::Checkmate
        } else {
            GameEndState::Stalemate
        }
    }

    pub fn get_last_move(&self) -> Option<Move> {
        self.moves.last().copied()
    }

    fn move_count(&self) -> u32 {
        self.moves.len() as u32
    }

    //Returns the current board position
    // TODO(lucypero): Cache the board for subsequent calls to this on the same position
    //   will save a lot of cpu
    pub fn get_board(&mut self) -> &Board {
        //Start with the starting board position then you start mutating it with each
        //  move until you get the current position
        if self.cached_current_board.is_none() {
            let mut board = self.starting_board.clone();
            for chess_move in self.moves.iter() {
                board.apply_move(*chess_move);
            }
            self.cached_current_board = Some(board.clone());
        }

        self.cached_current_board.as_ref().unwrap()
    }

    pub fn perform_move(&mut self, mut chess_move: Move) -> Result<(), MoveError> {
        //performs all move validation here. If it is legal,
        //    the move is added to self.moves

        let whose_turn = self.whose_turn();
        let last_move = self.get_last_move();
        let board = self.get_board();

        match chess_move {
            Move::PieceMove {
                piece: piece_type,
                tile_from,
                tile_to,
                is_en_passant: _,
            } => {
                // 1: Is the player grabbing a piece?
                let piece = board
                    .get_piece(tile_from)
                    .ok_or(MoveError::TileFromIsEmpty)?;

                // 2: Is the Player grabbing their own piece?
                if piece.0 != board.whose_turn {
                    return Err(MoveError::TileFromIsEnemyPiece);
                }

                let mut is_en_passant = false;

                // 3: Is the move legal according to how the piece moves?
                if !is_piece_move_legal(
                    piece,
                    tile_from,
                    tile_to,
                    last_move,
                    board,
                    &mut is_en_passant,
                ) {
                    return Err(MoveError::PieceDoesNotMoveLikeThat);
                }

                chess_move = Move::PieceMove {
                    piece: piece_type,
                    tile_from,
                    tile_to,
                    is_en_passant,
                };

                // promotion check: error if it is a pawn move that reached the back rank_spec
                let tile_to_coord = Coord::from(tile_to);

                let back_rank = match whose_turn {
                    ChessTeam::Black => 0,
                    ChessTeam::White => 7,
                };

                if piece_type == ChessPiece::Pawn && tile_to_coord.y == back_rank {
                    return Err(MoveError::PromotionPieceNotSpecified);
                }
            }
            Move::PieceMoveWithPromotion {
                tile_from,
                tile_to,
                promotion,
            } => {
                // 1: Is the player grabbing a piece?
                let piece = board
                    .get_piece(tile_from)
                    .ok_or(MoveError::TileFromIsEmpty)?;

                // 2: Is the Player grabbing their own piece?
                if piece.0 != board.whose_turn {
                    return Err(MoveError::TileFromIsEnemyPiece);
                }

                // 3: Is the move legal according to how the piece moves?
                if !is_piece_move_legal(piece, tile_from, tile_to, last_move, &board, &mut false) {
                    return Err(MoveError::PieceDoesNotMoveLikeThat);
                }

                //rank has to be the back rank
                let tile_to_coord = Coord::from(tile_to);

                let back_rank = match whose_turn {
                    ChessTeam::Black => 0,
                    ChessTeam::White => 7,
                };

                if tile_to_coord.y != back_rank {
                    return Err(MoveError::PromotionNotLegal);
                }

                // promotion can't be a pawn or a king
                if promotion == ChessPiece::Pawn || promotion == ChessPiece::King {
                    return Err(MoveError::PromotionWrongPiece);
                }
            }
            Move::CastleShort | Move::CastleLong => {
                //1. check if the player has castling rights
                let the_err = Err(MoveError::CastlingNoRights);

                match whose_turn {
                    ChessTeam::Black => {
                        if (chess_move == Move::CastleShort && !board.castling_rights.2)
                            || (chess_move == Move::CastleLong && !board.castling_rights.3)
                        {
                            return the_err;
                        }
                    }
                    ChessTeam::White => {
                        if (chess_move == Move::CastleShort && !board.castling_rights.0)
                            || (chess_move == Move::CastleLong && !board.castling_rights.1)
                        {
                            return the_err;
                        }
                    }
                }

                //2. check if tiles in between are free

                // tiles in between for white, short castle: F1 G1
                // tiles in between for black, short castle: F8 G8

                // tiles in between for white, long castle: B1 C1 D1
                // tiles in between for black, long castle: B8 C8 D8

                let tiles_in_btw = match whose_turn {
                    ChessTeam::Black => {
                        if chess_move == Move::CastleShort {
                            vec![Tile::F8, Tile::G8]
                        } else {
                            vec![Tile::B8, Tile::C8, Tile::D8]
                        }
                    }
                    ChessTeam::White => {
                        if chess_move == Move::CastleShort {
                            vec![Tile::F1, Tile::G1]
                        } else {
                            vec![Tile::B1, Tile::C1, Tile::D1]
                        }
                    }
                };

                for tile in tiles_in_btw {
                    if board.get_piece(tile).is_some() {
                        return Err(MoveError::CastlingTilesInBetweenNotFree);
                    }
                }

                //3. check if king is not in check and does not go through check
                let tiles_king = match whose_turn {
                    ChessTeam::Black => {
                        if chess_move == Move::CastleShort {
                            vec![Tile::E8, Tile::F8, Tile::G8]
                        } else {
                            vec![Tile::C8, Tile::D8, Tile::E8]
                        }
                    }
                    ChessTeam::White => {
                        if chess_move == Move::CastleShort {
                            vec![Tile::E1, Tile::F1, Tile::G1]
                        } else {
                            vec![Tile::C1, Tile::D1, Tile::E1]
                        }
                    }
                };

                for tile in tiles_king {
                    if board.is_tile_attacked_by(whose_turn.the_other_one(), tile, last_move)
                    {
                        return Err(MoveError::CastlingThroughCheck);
                    }
                }
            }
        }

        // would the move put the player's king in check?

        // 1. get a hypothetical board where this move is performed anyway
        let mut future_board = board.clone();
        future_board.apply_move(chess_move);
        // 2. in that board, check if the king is attacked

        //finding king tile
        let king_coord = future_board.find_pieces(self.whose_turn(), ChessPiece::King)[0];
        if future_board.is_tile_attacked_by(
            self.whose_turn().the_other_one(),
            Tile::try_from(king_coord).unwrap(),
            Some(chess_move),
        ) {
            return Err(MoveError::InCheck);
        }

        //Everything is good. adding move to self.moves
        self.moves.push(chess_move);
        self.cached_current_board = None;

        Ok(())
    }

    // whose turn is it?
    pub fn whose_turn(&mut self) -> ChessTeam {
        self.get_board().whose_turn
    }
}

// fn capitalize_coord_str(str: String) -> String {
//     let mut v: Vec<char> = str.as_str().chars().collect();
//     v[0] = v[0].to_uppercase().nth(0).unwrap();
//     v.into_iter().collect()
// }

// functions that process moves from user input
mod move_processor {

    // Errors while parsing move

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum MoveParseError {
        //Move is ambiguous. More than one of that piece type can move there. Try specifying the rank and/or file of the piece.
        Ambiguous,
        //no piece of that type can make that move
        NoPiece,
        //destination tile is incomplete
        NoDestination,
        //Move could not be parsed.
        CantParse,
    }

    impl fmt::Display for MoveParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                MoveParseError::Ambiguous => write!(f, "Move is ambiguous. More than one of that piece type can move there. Try specifying the rank and/or file of the piece."),
                MoveParseError::NoPiece => write!(f, "no piece of that type can make that move"),
                MoveParseError::NoDestination => write!(f, "destination tile is incomplete"),
                MoveParseError::CantParse => write!(f, "Move could not be parsed."),
            }
        }
    }

    use super::*;

    //returns a tile if destination tuple contains both a file and a rank
    fn get_dest_tile(the_move: &move_parser::Move) -> Result<Coord, MoveParseError> {
        if let move_parser::MovePrimary::PieceMove {
            piece: _,
            destination,
            promotion: _,
        } = the_move.primary
        {
            let file_coord = match destination.0 {
                'a' => 0,
                'b' => 1,
                'c' => 2,
                'd' => 3,
                'e' => 4,
                'f' => 5,
                'g' => 6,
                'h' => 7,
                _ => return Err(MoveParseError::NoDestination),
            };

            let rank_coord = destination.1.to_digit(10);
            if rank_coord.is_none() {
                return Err(MoveParseError::NoDestination);
            }

            let rank_coord = (rank_coord.unwrap() - 1) as i32;

            return Ok(Coord {
                x: file_coord,
                y: rank_coord,
            });
        }

        Err(MoveParseError::NoDestination)
    }

    //part of the get_moved_* series. it tries
    //  to get the tile where the piece that made the move to dest is.
    fn get_moved_pawn(dest: Coord, board: &Board) -> Result<Coord, MoveParseError> {
        //walks backwards from dest (max 2 tiles) until there's a pawn.

        let team_factor = match board.whose_turn {
            ChessTeam::White => -1,
            ChessTeam::Black => 1,
        };

        for i in 1..=2 {
            let coord_iter = Coord {
                x: dest.x,
                y: dest.y + i * team_factor,
            };

            let piece = board.get_piece(Tile::try_from(coord_iter).unwrap());

            match piece {
                Some(TeamedChessPiece(t, p)) => {
                    //if found piece is not our pawn, return err()
                    if p == ChessPiece::Pawn && t == board.whose_turn {
                        return Ok(coord_iter);
                    } else {
                        return Err(MoveParseError::NoPiece);
                    }
                }
                None => continue,
            }
        }

        Err(MoveParseError::NoPiece)
    }

    fn get_piece(p: char) -> Option<ChessPiece> {
        match p {
            'r' | 'R' => Some(ChessPiece::Rook),
            'q' | 'Q' => Some(ChessPiece::Queen),
            'b' | 'B' => Some(ChessPiece::Bishop),
            'n' | 'N' => Some(ChessPiece::Knight),
            'k' | 'K' => Some(ChessPiece::King),
            _ => None,
        }
    }

    fn file_to_coord(f: char) -> Result<i32, ()> {
        match f {
            'a' => Ok(0),
            'b' => Ok(1),
            'c' => Ok(2),
            'd' => Ok(3),
            'e' => Ok(4),
            'f' => Ok(5),
            'g' => Ok(6),
            'h' => Ok(7),
            _ => Err(()),
        }
    }

    fn rank_to_coord(r: char) -> Result<i32, ()> {
        let rank_coord = r.to_digit(10);
        if rank_coord.is_none() {
            return Err(());
        }
        Ok(rank_coord.unwrap() as i32)
    }

    fn get_non_pawn_move(
        the_move: move_parser::Move,
        board: &Board,
    ) -> Result<Move, MoveParseError> {
        let piece = match the_move.primary {
            move_parser::MovePrimary::PieceMove {
                piece: p,
                destination: _,
                promotion: _,
            } => p,
            _ => ('-', '-', '-'),
        };

        //check for piece specification
        let file_spec = file_to_coord(piece.1);
        let rank_spec = rank_to_coord(piece.2);

        let piece = get_piece(piece.0).unwrap();

        let coord_dest = get_dest_tile(&the_move).unwrap();
        let tile_to = Tile::try_from(coord_dest).unwrap();
        let teamed_piece = TeamedChessPiece(board.whose_turn, piece);

        //1. get all pieces of that type that can make the move
        let mut pieces = board.find_pieces(board.whose_turn, piece);

        // filter if there is specification
        if let Ok(file_spec) = file_spec {
            pieces.retain(move |&p| p.x == file_spec)
        }

        if let Ok(rank_spec) = rank_spec {
            pieces.retain(move |&p| p.y == rank_spec);
        }

        pieces.retain(move |&p| {
            let tile_from = Tile::try_from(p).unwrap();
            is_piece_move_legal(teamed_piece, tile_from, tile_to, None, board, &mut false)
        });

        //2. if more than 1, ask to specify
        //   if 1, u have the move
        match pieces.len().cmp(&1) {
            std::cmp::Ordering::Greater => Err(MoveParseError::Ambiguous),
            std::cmp::Ordering::Less => Err(MoveParseError::NoPiece),
            std::cmp::Ordering::Equal => {
                let tile_from = Tile::try_from(pieces[0]).unwrap();
                Ok(Move::PieceMove {
                    piece,
                    tile_from,
                    tile_to,
                    is_en_passant: false,
                })
            }
        }
    }

    fn get_pawn_capture(
        the_move: move_parser::Move,
        last_move: Option<Move>,
        board: &Board,
    ) -> Result<Move, MoveParseError> {
        let mut file_from = '-';
        let mut destination = ('-', '-');
        let mut promotion = '-';

        if let move_parser::MovePrimary::PieceMove {
            piece,
            destination: d,
            promotion: p,
        } = the_move.primary
        {
            destination = d;
            promotion = p;
            file_from = piece.1
        }

        let team_factor = match board.whose_turn {
            ChessTeam::White => 1,
            ChessTeam::Black => -1,
        };

        let file = file_to_coord(destination.0).unwrap();
        let rank = rank_to_coord(destination.1);

        let file_from = file_to_coord(file_from).unwrap();

        let mut tile_from: Option<Tile> = None;
        let mut tile_to: Option<Tile> = None;

        let mut is_en_passant = false;

        //1. get all friendly pawns in file the_move.primary.piece.1
        let mut pawns: Vec<Coord> =
            board.find_pieces_in_file(board.whose_turn, ChessPiece::Pawn, file_from);
        //2. filter pawns by if they can take a piece on file the_move.primary.destination.2, or on tile if it is specified
        pawns.retain(|&p| {
            let coord_pawn_target = Coord {
                x: file,
                y: p.y + team_factor,
            };

            // if the rank is specified, test if it is the same as coord_pawn_target
            if rank.is_ok()
                && ((Coord {
                    x: file,
                    y: rank.unwrap(),
                }) != coord_pawn_target)
            {
                return false;
            }

            tile_from = Some(Tile::try_from(p).unwrap());
            tile_to = Some(Tile::try_from(coord_pawn_target).unwrap());

            is_piece_move_legal(
                TeamedChessPiece(board.whose_turn, ChessPiece::Pawn),
                tile_from.unwrap(),
                tile_to.unwrap(),
                last_move,
                board,
                &mut is_en_passant,
            )
        });

        //3. if more than one pawn can take, return error "need to specify tile to take"
        //   if only one pawn can take, return the move
        match pawns.len().cmp(&1) {
            std::cmp::Ordering::Greater => Err(MoveParseError::Ambiguous),
            std::cmp::Ordering::Less => Err(MoveParseError::NoPiece),
            std::cmp::Ordering::Equal => {
                let tile_from = tile_from.unwrap();
                let tile_to = tile_to.unwrap();

                if promotion != '-' {
                    let promoted_piece_type = get_piece(promotion).unwrap();
                    Ok(Move::PieceMoveWithPromotion {
                        tile_from,
                        tile_to,
                        promotion: promoted_piece_type,
                    })
                } else {
                    Ok(Move::PieceMove {
                        piece: ChessPiece::Pawn,
                        tile_from,
                        tile_to,
                        is_en_passant,
                    })
                }
            }
        }
    }

    fn get_pawn_move(the_move: move_parser::Move, board: &Board) -> Result<Move, MoveParseError> {
        let coord_dest = get_dest_tile(&the_move)?;

        // Get the moved pawn location
        let coord_from = get_moved_pawn(coord_dest, board)?;

        if let move_parser::MovePrimary::PieceMove {
            piece: _,
            destination: _,
            promotion,
        } = the_move.primary
        {
            if promotion != '-' {
                let promoted_piece_type = get_piece(promotion).unwrap();

                return Ok(Move::PieceMoveWithPromotion {
                    tile_from: Tile::try_from(coord_from).unwrap(),
                    tile_to: Tile::try_from(coord_dest).unwrap(),
                    promotion: promoted_piece_type,
                });
            } else {
                return Ok(Move::PieceMove {
                    piece: ChessPiece::Pawn,
                    tile_from: Tile::try_from(coord_from).unwrap(),
                    tile_to: Tile::try_from(coord_dest).unwrap(),
                    is_en_passant: false,
                });
            }
        }

        Err(MoveParseError::NoPiece)
    }

    // This uses our move parser in move_parser::parse() then processes the output
    //  It finds the right piece to move, and the destination tile, and constructs a Move
    pub fn parse_move(mut move_input: String, game: &mut GameState) -> Result<Move, MoveParseError> {
        move_input.retain(|c| !c.is_whitespace());

        let moves = move_parser::parse(move_input.chars().collect());

        if moves.is_err() {
            return Err(MoveParseError::CantParse);
        }

        let moves = moves.unwrap();

        //Processing parser output
        let last_move = game.get_last_move();
        let board = game.get_board();

        let mut the_move: Option<Move> = None;

        let mut last_error = MoveParseError::CantParse;

        for move_i in moves {
            match move_i.primary {
                move_parser::MovePrimary::PieceMove {
                    piece,
                    destination: _,
                    promotion: _,
                } => {
                    let piece_move;
                    // TODO(lucypero): try if u can generalize every case
                    //    and handle pawn moves and captures also in get_non_pawn_move
                    //    everything would have to go through is_piece_move_legal and
                    //    it would be a lot less code

                    //pawn move (no capture)
                    if piece == ('-', '-', '-') {
                        piece_move = get_pawn_move(move_i, &board);
                    }
                    // pawn capture
                    else if piece.0 == '-' {
                        piece_move = get_pawn_capture(move_i, last_move, &board);
                    }
                    // non-pawn move
                    else {
                        piece_move = get_non_pawn_move(move_i, &board);
                    }

                    if let Ok(piece_move) = piece_move {
                        the_move = Some(piece_move);
                    } else {
                        last_error = piece_move.unwrap_err();
                    }
                }
                move_parser::MovePrimary::CastleShort => {
                    the_move = Some(Move::CastleShort);
                }
                move_parser::MovePrimary::CastleLong => {
                    the_move = Some(Move::CastleLong);
                }
            }

            if the_move.is_some() {
                break;
            }
        }

        if let Some(the_move) = the_move {
            return Ok(the_move);
        }

        Err(last_error)
    }
}

// Part of move validation. Validates chess piece move logic
// note: it does not take into account if the move puts the player's king in check
pub fn is_piece_move_legal(
    piece: TeamedChessPiece,
    tile_from: Tile,
    tile_to: Tile,
    last_move: Option<Move>,
    board: &Board,
    is_en_passant: &mut bool,
) -> bool {
    *is_en_passant = false;

    let tile_from_coord = Coord::from(tile_from);
    let tile_to_coord = Coord::from(tile_to);

    let coord_distance = tile_from_coord.distance(tile_to_coord);

    match piece {
        TeamedChessPiece(team, ChessPiece::Pawn) => {
            //team factor (if black, it moves down, if white it moves up)
            let team_factor = match team {
                ChessTeam::White => 1,
                ChessTeam::Black => -1,
            };

            if coord_distance.y == team_factor && coord_distance.x == 0 {
                // NOTE(lucypero): going to have to do this some day
                // board.is_path_blocked(chess_move.tile_from, chess_move.tile_to)

                //check if there is a piece in the way
                let piece_option = board.get_piece(tile_to);

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
                let piece_option_2 = board.get_piece(tile_to);

                if piece_option.is_none() && piece_option_2.is_none() {
                    return true;
                }
            }

            // diagonal move when it takes a piece
            if coord_distance.x.abs() == 1 && coord_distance.y == team_factor {
                //get the targeted tile
                //if there is a piece and it is the other team's, the move is valid
                let target_piece_option = board.get_piece(tile_to);

                if let Some(target_piece) = target_piece_option {
                    if team != target_piece.0 {
                        return true;
                    }
                }
                //en passant
                else if (board.whose_turn == ChessTeam::Black && tile_to_coord.y == 2)
                    || (board.whose_turn == ChessTeam::White && tile_to_coord.y == 5)
                {
                    // get last move
                    if let Some(Move::PieceMove {
                        piece: ChessPiece::Pawn,
                        tile_from: last_tile_from,
                        tile_to: last_tile_to,
                        is_en_passant: _,
                    }) = last_move
                    {
                        let last_tile_from = Coord::from(last_tile_from);
                        let last_tile_to = Coord::from(last_tile_to);

                        if (board.whose_turn == ChessTeam::Black
                            && last_tile_from
                                == Coord {
                                    x: tile_to_coord.x,
                                    y: 1,
                                }
                            && last_tile_to
                                == Coord {
                                    x: tile_to_coord.x,
                                    y: 3,
                                })
                            || (board.whose_turn == ChessTeam::White
                                && last_tile_from
                                    == Coord {
                                        x: tile_to_coord.x,
                                        y: 6,
                                    }
                                && last_tile_to
                                    == Coord {
                                        x: tile_to_coord.x,
                                        y: 4,
                                    })
                        {
                            //this is en pasant, return true
                            *is_en_passant = true;
                            return true;
                        }
                    }
                }
            }

            false
        }
        TeamedChessPiece(_, ChessPiece::King) => {
            let magn = coord_distance.magnitude();

            !(magn.is_none() ||
            //magnitude has to be one
              (magn.unwrap() != 1) ||
            //check if there is a friendly piece in the way
              (!board.is_path_clear(piece, tile_from, tile_to)))
        }
        TeamedChessPiece(_, ChessPiece::Rook) => {
            let magn = coord_distance.magnitude();

            //check movement
            !(magn.is_none() || 
            //line has to be a horizontal or vertical line
              (!(coord_distance.x == 0 || coord_distance.y == 0)) ||
            //check if there is a friendly piece in the way
              (!board.is_path_clear(piece, tile_from, tile_to)))
        }
        TeamedChessPiece(_, ChessPiece::Bishop) => {
            let magn = coord_distance.magnitude();

            //check movement
            !(magn.is_none() ||
            //line has to be a diagonal
               (coord_distance.x.abs() != coord_distance.y.abs()) ||
            //check if there is a friendly piece in the way
               (!board.is_path_clear(piece, tile_from, tile_to)))
        }
        TeamedChessPiece(_, ChessPiece::Queen) => {
            let magn = coord_distance.magnitude();

            //check movement
            !(magn.is_none() ||
            //check if there is a friendly piece in the way
              (!board.is_path_clear(piece, tile_from, tile_to)))
        }
        TeamedChessPiece(_, ChessPiece::Knight) => {
            //checking if the move is an L
            !(!((coord_distance.x.abs() == 2 && coord_distance.y.abs() == 1)
                || (coord_distance.x.abs() == 1 && coord_distance.y.abs() == 2)) ||
            //check if there is a friendly piece at destination
             (board.is_friendly_piece_at_destination(piece, tile_to)))
        }
    }
}

// Describes a snapshot of the board on a given position
// Basically, what pieces there are and where they are
#[derive(Clone)]
pub struct Board {
    pub whose_turn: ChessTeam,
    piece_locations: HashMap<Tile, TeamedChessPiece>,
    castling_rights: (bool, bool, bool, bool), // (white short castle, white long castle, black short castle, black long castle)
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

        Board {
            whose_turn: ChessTeam::White,
            piece_locations,
            castling_rights: (true, true, true, true),
        }
    }

    pub fn find_all_pieces(&self) -> Vec<(TeamedChessPiece, Tile)> {
        let mut result = vec![];
        for piece in &self.piece_locations {
            result.push((*piece.1, *piece.0));
        }
        result
    }

    pub fn find_pieces_of_team(&self, team: ChessTeam) -> Vec<(TeamedChessPiece, Tile)> {
        let mut result = vec![];

        for piece in &self.piece_locations {
            if piece.1 .0 == team {
                result.push((*piece.1, *piece.0));
            }
        }

        result
    }

    pub fn find_pieces(&self, team: ChessTeam, piece_type: ChessPiece) -> Vec<Coord> {
        let mut result = vec![];

        for file in 0..=7 {
            let mut result_file = self.find_pieces_in_file(team, piece_type, file);
            result.append(&mut result_file);
        }
        result
    }

    pub fn get_legal_moves_of_piece_in_tile(
        &self,
        tile: Tile,
        last_move: Option<Move>,
    ) -> Option<Vec<Coord>> {
        let piece = self.get_piece(tile)?;
        let piece_type = piece.1;

        let coord_from = Coord::from(tile);

        //1. get all the moves that the piece can make according to how it moves.
        //   disregarding if the piece can actually make that move in the board.
        //   all wrong moves will be filtered out later.
        let mut moves: Vec<Coord> = vec![];

        fn safe_coord_add(coord: Coord, moves: &mut Vec<Coord>) {
            if Tile::try_from(coord).is_ok() {
                moves.push(coord);
            }
        }

        fn add_horizontal_and_vertical(coord: Coord, moves: &mut Vec<Coord>) {
            for i in 0..=7 {
                moves.push(Coord { x: coord.x, y: i });
                moves.push(Coord { x: i, y: coord.y });
            }

            // remove the tile where the piece is sitting on
            moves.retain(|c| *c != coord);
        }

        fn add_diagonals(coord: Coord, moves: &mut Vec<Coord>) {
            // diagonal that goes from coord to to right top
            for i in 1..=7 {
                let possible_coord = coord + Coord { x: i, y: i };
                if Tile::try_from(possible_coord).is_ok() {
                    moves.push(possible_coord);
                } else {
                    break;
                }
            }

            // diagonal that goes from coord to left bottom
            for i in 1..=7 {
                let possible_coord = coord + Coord { x: -i, y: -i };
                if Tile::try_from(possible_coord).is_ok() {
                    moves.push(possible_coord);
                } else {
                    break;
                }
            }

            // diagonal that goes from coord to left top
            for i in 1..=7 {
                let possible_coord = coord + Coord { x: -i, y: i };
                if Tile::try_from(possible_coord).is_ok() {
                    moves.push(possible_coord);
                } else {
                    break;
                }
            }

            // diagonal that goes from coord to right bottom
            for i in 1..=7 {
                let possible_coord = coord + Coord { x: i, y: -i };
                if Tile::try_from(possible_coord).is_ok() {
                    moves.push(possible_coord);
                } else {
                    break;
                }
            }
        }

        match piece_type {
            ChessPiece::Pawn => {
                //1 or 2 moves ahead, and diagonally
                let team_factor = match self.whose_turn {
                    ChessTeam::Black => -1,
                    ChessTeam::White => 1,
                };

                // 1 and 2 moves ahead
                safe_coord_add(
                    coord_from
                        + Coord {
                            x: 0,
                            y: team_factor,
                        },
                    &mut moves,
                );
                safe_coord_add(
                    coord_from
                        + Coord {
                            x: 0,
                            y: 2 * team_factor,
                        },
                    &mut moves,
                );
                // diagonals
                safe_coord_add(
                    coord_from
                        + Coord {
                            x: 1,
                            y: team_factor,
                        },
                    &mut moves,
                );
                safe_coord_add(
                    coord_from
                        + Coord {
                            x: -1,
                            y: team_factor,
                        },
                    &mut moves,
                );
            }
            ChessPiece::Rook => {
                // all other tiles on its file and rank
                add_horizontal_and_vertical(coord_from, &mut moves);
            }
            ChessPiece::Knight => {
                // the L's
                safe_coord_add(coord_from + Coord { x: 1, y: 2 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: -1, y: 2 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 2, y: 1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 2, y: -1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: -2, y: 1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: -2, y: -1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 1, y: -2 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: -1, y: -2 }, &mut moves);
            }
            ChessPiece::Bishop => {
                // all other tiles on its diagonals
                add_diagonals(coord_from, &mut moves);
            }
            ChessPiece::Queen => {
                // all other tiles on its file, rank and diagonals
                add_horizontal_and_vertical(coord_from, &mut moves);
                add_diagonals(coord_from, &mut moves);
            }
            ChessPiece::King => {
                safe_coord_add(coord_from + Coord { x: -1, y: 1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 0, y: 1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 1, y: 1 }, &mut moves);

                safe_coord_add(coord_from + Coord { x: -1, y: 0 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 0, y: 0 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 1, y: 0 }, &mut moves);

                safe_coord_add(coord_from + Coord { x: -1, y: -1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 0, y: -1 }, &mut moves);
                safe_coord_add(coord_from + Coord { x: 1, y: -1 }, &mut moves);
            }
        }

        //2. filter out all moves where !is_legal_move
        moves.retain(|coord_to| {
            is_piece_move_legal(
                piece,
                tile,
                Tile::try_from(*coord_to).unwrap(),
                last_move,
                &self,
                &mut false,
            )
        });

        //3. filter out all moves that put the king in check
        moves.retain(|coord_to| {
            let mut future_board: Board = self.clone();

            //the move
            let the_move = Move::PieceMove {
                piece: piece_type,
                tile_from: tile,
                tile_to: Tile::try_from(*coord_to).unwrap(),
                is_en_passant: false,
            };

            // TODO(lucypero): castle moves. not sure if it is necessary.

            // NOTE(lucypero): we ignore things like en_passant and promotion piece because
            //  that would not affect if the player's king is in check.
            future_board.apply_move(the_move);
            !future_board.is_team_in_check(self.whose_turn, Some(the_move))
        });

        Some(moves)
    }

    pub fn is_team_in_check(&self, team: ChessTeam, last_move: Option<Move>) -> bool {
        let king_coord = self.find_pieces(team, ChessPiece::King)[0];
        self.is_tile_attacked_by(
            team.the_other_one(),
            Tile::try_from(king_coord).unwrap(),
            last_move,
        )
    }

    //check if tile is being under attack by any piece in team
    pub fn is_tile_attacked_by(
        &self,
        team: ChessTeam,
        tile: Tile,
        last_move: Option<Move>,
    ) -> bool {
        //.1 get all pieces of team
        let pieces = self.find_pieces_of_team(team);

        //.2 for each of that piece, check if it is legal to take on tile
        for piece in pieces {
            if is_piece_move_legal(piece.0, piece.1, tile, last_move, &self, &mut false) {
                return true;
            }
        }

        false
    }

    pub fn find_pieces_in_file(
        &self,
        team: ChessTeam,
        piece_type: ChessPiece,
        file: i32,
    ) -> Vec<Coord> {
        let mut result = vec![];

        for rank in 0..=7 {
            let coord = Coord { x: file, y: rank };
            let tile = Tile::try_from(coord).unwrap();
            let piece = self.get_piece(tile);

            if piece.is_none() {
                continue;
            }

            let teamed_piece = piece.unwrap();

            if teamed_piece.0 == team && teamed_piece.1 == piece_type {
                result.push(coord);
            }
        }

        result
    }

    pub fn apply_move(&mut self, chess_move: Move) {
        match chess_move {
            Move::PieceMove {
                piece: piece_type,
                tile_from,
                tile_to,
                is_en_passant,
            } => {
                let piece = self.piece_locations.remove(&tile_from).unwrap();
                self.piece_locations.insert(tile_to, piece);

                //must remove captured pawn if en_passant
                if is_en_passant {
                    let mut captured_pawn_coord = Coord::from(tile_to);
                    match self.whose_turn {
                        ChessTeam::Black => {
                            captured_pawn_coord.y += 1;
                        }
                        ChessTeam::White => {
                            captured_pawn_coord.y -= 1;
                        }
                    }
                    self.piece_locations
                        .remove(&Tile::try_from(captured_pawn_coord).unwrap());
                }

                //update castling rights if necessary
                if piece_type == ChessPiece::Rook {
                    match self.whose_turn {
                        ChessTeam::Black => {
                            if tile_from == Tile::A8 {
                                self.castling_rights.3 = false;
                            } else if tile_from == Tile::H8 {
                                self.castling_rights.2 = false;
                            }
                        }
                        ChessTeam::White => {
                            if tile_from == Tile::A1 {
                                self.castling_rights.1 = false;
                            } else if tile_from == Tile::H1 {
                                self.castling_rights.0 = false;
                            }
                        }
                    }
                }

                if piece_type == ChessPiece::King {
                    match self.whose_turn {
                        ChessTeam::Black => {
                            self.castling_rights.2 = false;
                            self.castling_rights.3 = false;
                        }
                        ChessTeam::White => {
                            self.castling_rights.0 = false;
                            self.castling_rights.1 = false;
                        }
                    }
                }
            }
            Move::PieceMoveWithPromotion {
                tile_from,
                tile_to,
                promotion,
            } => {
                self.piece_locations.remove(&tile_from);
                self.piece_locations
                    .insert(tile_to, TeamedChessPiece(self.whose_turn, promotion));
            }
            Move::CastleShort => match self.whose_turn {
                ChessTeam::Black => {
                    self.piece_locations.remove(&Tile::H8);
                    self.piece_locations.remove(&Tile::E8);
                    self.piece_locations.insert(
                        Tile::F8,
                        TeamedChessPiece(self.whose_turn, ChessPiece::Rook),
                    );
                    self.piece_locations.insert(
                        Tile::G8,
                        TeamedChessPiece(self.whose_turn, ChessPiece::King),
                    );

                    self.castling_rights.2 = false;
                    self.castling_rights.3 = false;
                }
                ChessTeam::White => {
                    self.piece_locations.remove(&Tile::H1);
                    self.piece_locations.remove(&Tile::E1);
                    self.piece_locations.insert(
                        Tile::F1,
                        TeamedChessPiece(self.whose_turn, ChessPiece::Rook),
                    );
                    self.piece_locations.insert(
                        Tile::G1,
                        TeamedChessPiece(self.whose_turn, ChessPiece::King),
                    );

                    self.castling_rights.0 = false;
                    self.castling_rights.1 = false;
                }
            },
            Move::CastleLong => match self.whose_turn {
                ChessTeam::Black => {
                    self.piece_locations.remove(&Tile::A8);
                    self.piece_locations.remove(&Tile::E8);
                    self.piece_locations.insert(
                        Tile::D8,
                        TeamedChessPiece(self.whose_turn, ChessPiece::Rook),
                    );
                    self.piece_locations.insert(
                        Tile::C8,
                        TeamedChessPiece(self.whose_turn, ChessPiece::King),
                    );

                    self.castling_rights.2 = false;
                    self.castling_rights.3 = false;
                }
                ChessTeam::White => {
                    self.piece_locations.remove(&Tile::A1);
                    self.piece_locations.remove(&Tile::E1);
                    self.piece_locations.insert(
                        Tile::D1,
                        TeamedChessPiece(self.whose_turn, ChessPiece::Rook),
                    );
                    self.piece_locations.insert(
                        Tile::C1,
                        TeamedChessPiece(self.whose_turn, ChessPiece::King),
                    );

                    self.castling_rights.0 = false;
                    self.castling_rights.1 = false;
                }
            },
        }

        self.whose_turn = self.whose_turn.the_other_one();
    }

    pub fn get_piece(&self, tile: Tile) -> Option<TeamedChessPiece> {
        self.piece_locations.get(&tile).copied()
    }

    // Checks if the path is clear for the piece.
    //   false if there is a friendly piece in (tile_from, tile_to]
    //   false if there is an enemy piece in (tile_from, tile_to) (not including tile_to)
    //   otherwise, true
    pub fn is_path_clear(&self, piece: TeamedChessPiece, tile_from: Tile, tile_to: Tile) -> bool {
        //ensure that the distance is a "good line", otherwise it makes no sense.
        let tile_from_coord = Coord::from(tile_from);
        let tile_to_coord = Coord::from(tile_to);

        let coord_distance = tile_from_coord.distance(tile_to_coord);
        let magn = coord_distance.magnitude().unwrap();

        //loop through the distance
        for i in 1..magn {
            //get direction to destination
            let coord_iter = tile_from_coord + (tile_to_coord - tile_from_coord).unit() * i;

            let tile_iter = Tile::try_from(coord_iter).unwrap();

            let piece_in_path_res = self.get_piece(tile_iter);

            if piece_in_path_res.is_some() {
                return false;
            }
        }

        // finally, check if there is a friendly piece at the destination tile
        if self.is_friendly_piece_at_destination(piece, tile_to) {
            return false;
        }

        true
    }

    pub fn is_friendly_piece_at_destination(&self, piece: TeamedChessPiece, tile: Tile) -> bool {
        let piece_dest = self.get_piece(tile);

        if let Some(TeamedChessPiece(team, _)) = piece_dest {
            if team == piece.0 {
                return true;
            }
        }

        false
    }

    pub fn print(&self) {
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

// this is basically the terminal interface of the game
fn game_cli_loop(game: &mut GameState) {
    //read move from stdin, you write the move and press enter
    println!("Welcome to chess! Type !help for all the commands");

    game.get_board().print();

    print!("\n\n{} to move. What's your move? ...\n", game.whose_turn());

    let stdin = io::stdin();
    for line_res in stdin.lock().lines() {
        //printing board:

        let mut line = line_res.unwrap();

        if line.is_empty() {
            println!("Type the move!");
            continue;
        }

        let first_character = line.chars().next();

        //parsing commands
        if first_character.unwrap() == '!' {
            let command = line.split_off(1);
            match command.as_str() {
                "help" => {
                    print!(
                        r#"
Welcome to chess!
Just type the move you want to make! (in algebraic chess notation: https://en.wikipedia.org/wiki/Algebraic_notation_(chess))

Available commands:

!help - show this dialog
!move_count - show how many moves were made
!test: test command
"#
                    );
                }
                "move_count" => {
                    println!(
                        "{} moves were made. {} to move.",
                        game.move_count(),
                        game.whose_turn()
                    );
                }
                "test" => {
                    println!("test command");
                }
                cmd => {
                    println!("{} command doesn't exist. Try again.", cmd);
                }
            }
        } else {
            let parse_res = move_processor::parse_move(line, game);
            if parse_res.is_err() {
                println!("Error while parsing move: {}", parse_res.unwrap_err());
                continue;
            }

            let the_move = parse_res.unwrap();

            let move_res = game.perform_move(the_move);
            if move_res.is_err() {
                println!("Illegal move: {}", move_res.unwrap_err());
                continue;
            }

            println!("{}", the_move);

            game.get_board().print();
            //move done successfully. check for end game condition
            match game.get_end_state() {
                GameEndState::Checkmate => {
                    println!(
                        "It's checkmate! {} has won!",
                        game.whose_turn().the_other_one()
                    );
                    break;
                }
                GameEndState::Stalemate => {
                    println!("It's a stalemate! Game is drawn!");
                    break;
                }
                GameEndState::Running => {
                    print!("\n\n{} to move. What's your move? ...\n", game.whose_turn());
                }
            }
        }
    }
}

pub fn get_test(args: Vec<String>) -> Option<GameState> {
    if args.len() <= 1 {
        return None;
    }

    if args.get(1).unwrap() == "test" {
        let test_arg = args.get(2)?;

        #[allow(clippy::single_match)]
        match test_arg.as_str() {
            "promotion-test" => {
                let mut piece_locations = HashMap::new();

                piece_locations.insert(
                    Tile::A1,
                    TeamedChessPiece(ChessTeam::White, ChessPiece::King),
                );
                piece_locations.insert(
                    Tile::A8,
                    TeamedChessPiece(ChessTeam::Black, ChessPiece::King),
                );
                piece_locations.insert(
                    Tile::D7,
                    TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
                );
                piece_locations.insert(
                    Tile::E7,
                    TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
                );
                piece_locations.insert(
                    Tile::D2,
                    TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
                );
                piece_locations.insert(
                    Tile::E2,
                    TeamedChessPiece(ChessTeam::Black, ChessPiece::Pawn),
                );
                piece_locations.insert(
                    Tile::H3,
                    TeamedChessPiece(ChessTeam::White, ChessPiece::Pawn),
                );

                let board = Board {
                    whose_turn: ChessTeam::White,
                    piece_locations,
                    castling_rights: (false, false, false, false),
                };

                return Some(GameState::init_from_custom_position(board));
            }
            _ => {}
        }
    }

    None
}
