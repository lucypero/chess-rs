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
    GameEndState
};
use macroquad::prelude::*;
use macroquad::input;

const PIECE_DISPLAY_SIZE: f32 = 80.0;
const BOARD_PADDING: f32 = 30.0;

const PIECE_TEXTURES: [&str; 33] = [
    "assets/pieces/neo.png",
    "assets/pieces/8_bit.png",
    "assets/pieces/alpha.png",
    "assets/pieces/bases.png",
    "assets/pieces/book.png",
    "assets/pieces/bubblegum.png",
    "assets/pieces/cases.png",
    "assets/pieces/classic.png",
    "assets/pieces/club.png",
    "assets/pieces/condal.png",
    "assets/pieces/dash.png",
    "assets/pieces/game_room.png",
    "assets/pieces/glass.png",
    "assets/pieces/gothic.png",
    "assets/pieces/graffiti.png",
    "assets/pieces/icy_sea.png",
    "assets/pieces/light.png",
    "assets/pieces/lolz.png",
    "assets/pieces/marble.png",
    "assets/pieces/maya.png",
    "assets/pieces/metal.png",
    "assets/pieces/modern.png",
    "assets/pieces/nature.png",
    "assets/pieces/neon.png",
    "assets/pieces/neo_wood.png",
    "assets/pieces/newspaper.png",
    "assets/pieces/ocean.png",
    "assets/pieces/sky.png",
    "assets/pieces/space.png",
    "assets/pieces/tigers.png",
    "assets/pieces/tournament.png",
    "assets/pieces/vintage.png",
    "assets/pieces/wood.png",
];

const BOARD_TEXTURES: [&str;33] = [
    "assets/boards/8_bit.png",
    "assets/boards/bases.png",
    "assets/boards/blue.png",
    "assets/boards/brown.png",
    "assets/boards/bubblegum.png",
    "assets/boards/burled_wood.png",
    "assets/boards/christmas.png",
    "assets/boards/christmas_alt.png",
    "assets/boards/dark_wood.png",
    "assets/boards/dash.png",
    "assets/boards/glass.png",
    "assets/boards/graffiti.png",
    "assets/boards/green.png",
    "assets/boards/icy_sea.png",
    "assets/boards/light.png",
    "assets/boards/lolz.png",
    "assets/boards/marble.png",
    "assets/boards/metal.png",
    "assets/boards/neon.png",
    "assets/boards/newpaper.png",
    "assets/boards/orange.png",
    "assets/boards/overlay.png",
    "assets/boards/parchment.png",
    "assets/boards/purple.png",
    "assets/boards/red.png",
    "assets/boards/sand.png",
    "assets/boards/sea.png",
    "assets/boards/sky.png",
    "assets/boards/stone.png",
    "assets/boards/tan.png",
    "assets/boards/tournament.png",
    "assets/boards/translucent.png",
    "assets/boards/walnut.png"
];

pub fn get_mq_conf() -> Conf {
    Conf {
        window_title: String::from("Chess-rs"),
        window_width: PIECE_DISPLAY_SIZE as i32 * 8 + BOARD_PADDING as i32 * 2,
        window_height: PIECE_DISPLAY_SIZE as i32 * 8 + BOARD_PADDING as i32 * 2,
        fullscreen: false,
        ..Default::default()
    }
}

pub struct GfxState {
    dragged_piece_i: usize,
    is_dragged: bool,
    drag_offset: Vec2,
    board_col: ColBox,
    pieces: Vec<Piece>,
    board_tex: Texture2D,
    pieces_tex: Texture2D,
    piece_tex_index: usize,
    board_tex_index: usize,
}

fn get_board_coord(tile: Tile) -> Coord {
    let mut coord = Coord::from(tile);
    coord.y = 7 - coord.y;
    coord
}

impl GfxState {

    pub async fn init(game: &GameState) -> GfxState {

        let piece_tex_index = 3;
        let board_tex_index = 22;

        // let tex = load_texture("assets/smiley.png").await;
        let dragged_piece_i = 0;
        let is_dragged = false;
        let drag_offset = Vec2{x:0.0, y:0.0};

        let board_col = ColBox {x: BOARD_PADDING, y: BOARD_PADDING,
                                w: PIECE_DISPLAY_SIZE * 8.0, h: PIECE_DISPLAY_SIZE * 8.0};

        let board_tex = load_texture(BOARD_TEXTURES[board_tex_index]).await;
        let pieces_tex = load_texture(PIECE_TEXTURES[piece_tex_index]).await;

        let mut state = GfxState {
            drag_offset,
            dragged_piece_i,
            is_dragged,
            board_col,
            board_tex,
            pieces_tex,
            pieces: vec![],
            piece_tex_index,
            board_tex_index,
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
            // TODO(lucypero): Auto promotes to a queen for now
            the_move = Move::PieceMoveWithPromotion{tile_from, tile_to, promotion: ChessPiece::Queen};
        }
        //normal move
        else {
            // we pass false to is_en_passant every time because perform_move modifies the field if the move is actually en passant before executing it
            the_move = Move::PieceMove{piece:piece.piece_type, tile_from, tile_to, is_en_passant: false};
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

    pub async fn draw(&mut self, game: &mut GameState) {

        if input::is_key_pressed(KeyCode::B) {
            self.board_tex_index +=1;
            if self.board_tex_index >= BOARD_TEXTURES.len() {
                self.board_tex_index = 0;
            }

            self.board_tex = load_texture(BOARD_TEXTURES[self.board_tex_index]).await;

            println!("board tex: {}", BOARD_TEXTURES[self.board_tex_index]);
        }

        if input::is_key_pressed(KeyCode::P) {

            self.piece_tex_index +=1;
            if self.piece_tex_index >= PIECE_TEXTURES.len() {
                self.piece_tex_index = 0;
            }

            self.pieces_tex = load_texture(PIECE_TEXTURES[self.piece_tex_index]).await;
            println!("piece tex: {}", PIECE_TEXTURES[self.piece_tex_index]);
        }


        clear_background(DARKBROWN);

        // draw_texture(tex, screen_width() / 2.0 , screen_height() / 2.0, WHITE);
        // draw_rectangle(col_box.x, col_box.y, col_box.w, col_box.h, BLUE);

        let board_params = DrawTextureParams { 
            dest_size: Some(Vec2{x:self.board_col.w, y:self.board_col.h}), 
            source: None,
            rotation: 0.0, flip_x: false, flip_y: false, pivot: None
        };

        draw_texture_ex(self.board_tex, self.board_col.x, self.board_col.y, WHITE, board_params);

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
            piece.draw(self.pieces_tex);
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

    fn draw(&self, atlas_tex: Texture2D) {

        let mut atlas_pos = match self.piece_type {
            ChessPiece::Pawn => 3,
            ChessPiece::Rook => 5,
            ChessPiece::Knight => 2,
            ChessPiece::Bishop => 0,
            ChessPiece::Queen => 4,
            ChessPiece::King => 1
        };

        if self.team == ChessTeam::White {
            atlas_pos += 6;
        }

        let params = DrawTextureParams { 
            dest_size: Some(Vec2{x:self.col.w, y:self.col.h}), 
            source: Some(Rect{ x: 150.0 * atlas_pos as f32, y: 0.0, w: 150.0, h: 150.0}), 
            rotation: 0.0, flip_x: false, flip_y: false, pivot: None
        };

        draw_texture_ex(atlas_tex, self.col.x, self.col.y, WHITE, params);
    }
}
