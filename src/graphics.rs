use std::convert::TryFrom;

use crate::chess::{
    Coord,
    GameState,
    ChessTeam,
    ChessPiece,
    Tile,
    Move,
    Board,
    MoveError,
    is_piece_move_legal,
    TeamedChessPiece,
    GameEndState
};
use macroquad::prelude::*;
use macroquad::input;

pub fn get_mq_conf() -> Conf {
    Conf {
        window_title: String::from("Chess-rs"),
        window_width: 500,
        window_height: 500,
        fullscreen: false,
        ..Default::default()
    }
}

pub struct GfxState {
    board_w: f32,
    dragged_piece_i: usize,
    is_dragged: bool,
    drag_offset: Vec2,
    board_col: ColBox,
    pieces: Vec<Piece>
}

fn get_board_coord(tile: Tile) -> Coord {
    let mut coord = Coord::from(tile);
    coord.y = 7 - coord.y;
    coord
}

impl GfxState {

    pub fn init(game: &GameState) -> GfxState {

        let board_w : f32 = 400.0;

        // let tex = load_texture("assets/smiley.png").await;
        let dragged_piece_i = 0;
        let is_dragged = false;
        let drag_offset = Vec2{x:0.0, y:0.0};

        let board_col = ColBox {x: 30.0, y: 30.0, w: board_w, h: board_w};

        let mut state = GfxState {
            board_w,
            drag_offset,
            dragged_piece_i,
            is_dragged,
            board_col,
            pieces: vec![]
        };

        state.sync_board(&game.get_board());

        state
    }

    fn sync_board(&mut self, board: &Board) {

        self.pieces.clear();

        for (piece, tile) in board.find_all_pieces() {
            self.pieces.push(
                Piece::init(&self.board_col, Coord::from(tile), piece.0, piece.1)
            );
        }
    }

    //so this constructs the move and calls gamestate.perform_move
    fn attempt_move_execution(&mut self, piece_i : usize, coord_to: Coord, game: &mut GameState) -> Result<(), MoveError> {

        let piece = &self.pieces[piece_i];

        let tile_from = Tile::try_from(piece.pos).unwrap();
        let tile_to = Tile::try_from(coord_to).unwrap();


        // is it a castle?
        let king_spawn_coord_y = match game.whose_turn() {
            ChessTeam::Black => 7,
            ChessTeam::White => 0
        };

        let finish_line = match game.whose_turn() {
            ChessTeam::Black => 0,
            ChessTeam::White => 7
        };

        let the_move;

        if piece.piece_type == ChessPiece::King &&
            piece.pos == (Coord{x:4,y:king_spawn_coord_y}) &&
            (coord_to == Coord{x:2, y:king_spawn_coord_y} || 
             coord_to == Coord{x:6, y:king_spawn_coord_y}) {

            if coord_to.x == 2 {
                the_move = Move::CastleLong;
            } else {
                the_move = Move::CastleShort;
            }
        } 
        //promotion?
        else if piece.piece_type == ChessPiece::Pawn &&
            coord_to.y == finish_line {
            the_move = Move::PieceMoveWithPromotion{tile_from, tile_to, promotion: ChessPiece::Queen};
        }
        //normal move
        else {
            // TODO(lucypero): Auto promotes to a queen for now

            //checking if it is en passant
            let mut is_en_passant = false;
            is_piece_move_legal(TeamedChessPiece(game.whose_turn(), ChessPiece::Pawn), tile_from, tile_to, game.get_last_move(), &game.get_board(), &mut is_en_passant);

            the_move = Move::PieceMove{piece:piece.piece_type, tile_from, tile_to, is_en_passant};
        }

        game.perform_move(the_move)
    }

    fn handle_end_state(&self, game: &GameState) {
        match game.get_end_state() {
            GameEndState::Checkmate => {
                println!("It's checkmate! {} has won!", game.whose_turn().the_other_one());
                // TODO(lucypero): Yeah I panic just to end the program hah
                panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Stalemate => {
                println!("It's a stalemate! Game is drawn!");
                // TODO(lucypero): Yeah I panic just to end the program hah
                panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Running => {
                print!("\n\n{} to move. What's your move? ...\n", game.whose_turn());
            }
        }
    }

    pub fn draw(&mut self, game: &mut GameState) {
        clear_background(DARKBROWN);

        // draw_texture(tex, screen_width() / 2.0 , screen_height() / 2.0, WHITE);
        // draw_rectangle(col_box.x, col_box.y, col_box.w, col_box.h, BLUE);
        self.board_col.draw(BROWN);


        if !input::is_mouse_button_down(MouseButton::Left) && self.is_dragged{
            // println!("dragged stopped!");
            self.is_dragged = false;

            let mouse_vec = input::mouse_position();
            let mouse_vec = Vec2{x:mouse_vec.0, y: mouse_vec.1};
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;
                let board_coord = Coord{x:coord_x, y: 7 - coord_y};

                let res = self.attempt_move_execution(self.dragged_piece_i, board_coord, game);

                if res.is_ok() {
                    println!("Move Executed successfully!");
                    self.handle_end_state(&game);
                } else {
                    println!("Move not valid");
                }
            }

            self.sync_board(&game.get_board());
        }

        if input::is_mouse_button_pressed(MouseButton::Left) {
            // println!("mouse click! at {:?}", input::mouse_position());
            //check if u clicked the box
            let mouse_vec = input::mouse_position();

            let mouse_vec = Vec2{x:mouse_vec.0, y: mouse_vec.1};

            for (i, piece) in self.pieces.iter().enumerate() {
                if piece.col.is_in_box(mouse_vec) {
                    // println!("clicked on box! dragged = true");
                    if game.whose_turn() != piece.team {
                        continue;
                    }
                    self.is_dragged = true;
                    self.drag_offset.x = mouse_vec.x - piece.col.x;
                    self.drag_offset.y = mouse_vec.y - piece.col.y;
                    self.dragged_piece_i = i;
                    // println!("drag offset {}", drag_offset);
                    break;
                }
            }

        }

        if self.is_dragged {
            let mouse_vec = input::mouse_position();
            self.pieces[self.dragged_piece_i].col.x = mouse_vec.0 - self.drag_offset.x;
            self.pieces[self.dragged_piece_i].col.y = mouse_vec.1 - self.drag_offset.y;
        }

        for piece in &self.pieces {
            piece.draw();
        }
    }
}

// in pixels
struct ColBox {
    x : f32,
    y : f32,
    w : f32,
    h : f32
}

impl ColBox {

    fn is_in_box(&self, vec: Vec2) -> bool {
        vec.x >= self.x && vec.y >= self.y && vec.x <= self.x + self.w && vec.y <= self.y + self.h
    }

    fn draw(&self, col: Color) {
        draw_rectangle(self.x, self.y, self.w, self.h, col);
    }
}

struct Piece {
    col : ColBox,
    pos : Coord,
    team: ChessTeam,
    piece_type: ChessPiece
}

impl Piece {

    fn init(board_col: &ColBox, pos:Coord, team: ChessTeam, piece_type: ChessPiece) -> Piece {
        let mut piece = Piece{ col: ColBox{x:0.0, y:0.0, w: board_col.w / 8.0, h: board_col.w / 8.0}, pos, team, piece_type};
        piece.update_col(board_col);
        piece
    }

    fn update_col(&mut self, board_col : &ColBox) {
        self.col.x = (board_col.w / 8.0) * (self.pos.x) as f32 + board_col.x;
        self.col.y = (board_col.h / 8.0) * (7 - self.pos.y) as f32 + board_col.y;
    }

    fn draw(&self) {

        let color = match self.team {
            ChessTeam::Black => BLACK,
            ChessTeam::White => WHITE
        };

        let text_color = match self.team {
            ChessTeam::Black => WHITE,
            ChessTeam::White => BLACK
        };

        self.col.draw(color);
        let piece_str = format!("{}", self.piece_type);
        draw_text(piece_str.as_str(), self.col.x, self.col.y + self.col.h / 2.0, 15.0, text_color);
    }
}
