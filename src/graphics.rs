use std::convert::TryFrom;
use std::cmp;

use crate::chess::{
    Board, ChessPiece, ChessTeam, Coord, GameEndState, GameState, Move, MoveError, Tile,
};
use macroquad::input;
use macroquad::prelude::*;

use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Group},
    Drag, Ui,
};

const PIECE_DISPLAY_SIZE: u32 = 80;
const BOARD_PADDING: u32 = 30;
const MOVES_LIST_WIDTH: u32 = 200;

// Color used for legal move indicators, arrows, tile where the piece will end up, etc...

const HIGHLIGHT_COLOR_RGB: (f32, f32, f32) = (0.89, 0.596, 0.850); //pinki

const HIGHLIGHT_COLOR: Color = Color {
    r: HIGHLIGHT_COLOR_RGB.0,
    g: HIGHLIGHT_COLOR_RGB.1,
    b: HIGHLIGHT_COLOR_RGB.2,
    a: 0.6,
}; //pinki
   // const HIGHLIGHT_COLOR: Color = Color{r:0.4, g:0.4, b:0.4, a: 0.4};

// Color used for last move tiles
const LAST_MOVE_COLOR: Color = Color {
    r: 0.00,
    g: 1.0,
    b: 0.0,
    a: 0.2,
};

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

const BOARD_TEXTURES: [&str; 33] = [
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
    "assets/boards/walnut.png",
];

pub fn get_mq_conf() -> Conf {
    Conf {
        window_title: String::from("Chess-rs"),
        window_width: (PIECE_DISPLAY_SIZE * 8 + BOARD_PADDING * 2
                        + MOVES_LIST_WIDTH + BOARD_PADDING) as i32,
        window_height: (PIECE_DISPLAY_SIZE * 8 + BOARD_PADDING * 2) as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[derive(PartialEq)]
enum Arrow {
    Arrow(Coord, Coord), //x,y -> u,v
    Circle(Coord),       //x,y
}

#[cfg(feature = "assets")]
pub struct GfxState {
    dragged_piece_i: usize,
    is_dragged: bool,
    board_col: ColBox,
    pieces: Vec<Piece>,
    board_tex: Texture2D,
    pieces_tex: Texture2D,
    piece_tex_index: usize,
    board_tex_index: usize,
    dragged_legal_moves: Vec<Coord>,
    //arrows and stuff
    coord_on_right_click_press: Option<Coord>,
    coord_on_right_click_release: Option<Coord>,
    arrows: Vec<Arrow>,
    viewed_move: usize
}

#[cfg(not(feature = "assets"))]
pub struct GfxState {
    dragged_piece_i: usize,
    is_dragged: bool,
    board_col: ColBox,
    pieces: Vec<Piece>,
    dragged_legal_moves: Vec<Coord>,
}

fn get_board_coord(tile: Tile) -> Coord {
    let mut coord = Coord::from(tile);
    coord.y = 7 - coord.y;
    coord
}

impl GfxState {
    #[cfg(not(feature = "assets"))]
    pub async fn init(game: &GameState) -> GfxState {
        let dragged_piece_i = 0;
        let is_dragged = false;

        let board_col = ColBox {
            x: BOARD_PADDING as f32,
            y: BOARD_PADDING as f32,
            w: PIECE_DISPLAY_SIZE as f32 * 8.0,
            h: PIECE_DISPLAY_SIZE as f32 * 8.0,
        };

        let mut state = GfxState {
            dragged_piece_i,
            is_dragged,
            board_col,
            pieces: vec![],
            dragged_legal_moves: vec![],
        };

        state.sync_board(&game.get_board());

        state
    }

    #[cfg(feature = "assets")]
    pub async fn init(game: &mut GameState) -> GfxState {
        let piece_tex_index = 5;
        let board_tex_index = 14;

        let dragged_piece_i = 0;
        let is_dragged = false;

        let board_col = ColBox {
            x: BOARD_PADDING as f32,
            y: BOARD_PADDING as f32,
            w: PIECE_DISPLAY_SIZE as f32 * 8.0,
            h: PIECE_DISPLAY_SIZE as f32 * 8.0,
        };

        let board_tex = load_texture(BOARD_TEXTURES[board_tex_index]).await;
        let pieces_tex = load_texture(PIECE_TEXTURES[piece_tex_index]).await;

        let viewed_move = cmp::max(0 as usize, game.move_count()) as usize;

        let mut state = GfxState {
            dragged_piece_i,
            is_dragged,
            board_col,
            board_tex,
            pieces_tex,
            pieces: vec![],
            piece_tex_index,
            board_tex_index,
            dragged_legal_moves: vec![],
            coord_on_right_click_press: None,
            coord_on_right_click_release: None,
            arrows: vec![],
            viewed_move
        };

        state.sync_board(&mut game.get_board());

        state
    }

    //display board position at move [move_i]
    fn show_move(&mut self, game: &GameState, move_i : usize) {

        assert!(move_i <= game.move_count());
        let board = game.get_board_at(move_i);
        self.viewed_move = move_i;

        //updating last move


        self.sync_board(&board);

        // if it is not the last move, lock the board (can't make any move)
    }

    fn get_coord_col(&self, coord: Coord) -> ColBox {
        ColBox {
            x: self.board_col.x + (self.board_col.w / 8.0) * coord.x as f32,
            y: self.board_col.y + (self.board_col.h / 8.0) * (7 - coord.y) as f32,
            w: self.board_col.w / 8.0,
            h: self.board_col.w / 8.0,
        }
    }

    fn sync_board(&mut self, board: &Board) {
        self.pieces.clear();

        for (piece, tile) in board.find_all_pieces() {
            self.pieces.push(Piece::init(
                &self.board_col,
                Coord::from(tile),
                piece.0,
                piece.1,
            ));
        }
    }

    //so this constructs the move and calls gamestate.perform_move
    fn attempt_move_execution(
        &mut self,
        piece_i: usize,
        coord_to: Coord,
        game: &mut GameState,
    ) -> Result<(), MoveError> {
        let piece = &self.pieces[piece_i];

        let tile_from = Tile::try_from(piece.pos).unwrap();
        let tile_to = Tile::try_from(coord_to).unwrap();

        // is it a castle?
        let king_spawn_coord_y = match game.whose_turn() {
            ChessTeam::Black => 7,
            ChessTeam::White => 0,
        };

        let finish_line = match game.whose_turn() {
            ChessTeam::Black => 0,
            ChessTeam::White => 7,
        };

        let the_move;

        if piece.piece_type == ChessPiece::King
            && piece.pos
                == (Coord {
                    x: 4,
                    y: king_spawn_coord_y,
                })
            && (coord_to
                == Coord {
                    x: 2,
                    y: king_spawn_coord_y,
                }
                || coord_to
                    == Coord {
                        x: 6,
                        y: king_spawn_coord_y,
                    })
        {
            if coord_to.x == 2 {
                the_move = Move::CastleLong;
            } else {
                the_move = Move::CastleShort;
            }
        }
        //promotion?
        else if piece.piece_type == ChessPiece::Pawn && coord_to.y == finish_line {
            // TODO(lucypero): Auto promotes to a queen for now
            the_move = Move::PieceMoveWithPromotion {
                tile_from,
                tile_to,
                promotion: ChessPiece::Queen,
            };
        }
        //normal move
        else {
            // we pass false to is_en_passant every time because perform_move modifies the field if the move is actually en passant before executing it
            the_move = Move::PieceMove {
                piece: piece.piece_type,
                tile_from,
                tile_to,
                is_en_passant: false,
            };
        }

        game.perform_move(the_move)
    }

    fn handle_end_state(&self, game: &mut GameState) {
        match game.get_end_state() {
            GameEndState::Checkmate => {
                println!(
                    "It's checkmate! {} has won!",
                    game.whose_turn().the_other_one()
                );
                // TODO(lucypero): Yeah I panic just to end the program hah
                panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Stalemate => {
                println!("It's a stalemate! Game is drawn!");
                // TODO(lucypero): Yeah I panic just to end the program hah
                panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Running => {}
        }
    }

    fn draw_moves_ui(&mut self, game: &mut GameState) {

        const move_no_w :f32 = 30.;
        
        widgets::Window::new(hash!(), vec2(PIECE_DISPLAY_SIZE as f32 * 8. + BOARD_PADDING as f32 * 2.,
                           BOARD_PADDING as f32), vec2(MOVES_LIST_WIDTH as f32, 
                                          PIECE_DISPLAY_SIZE as f32 * 8.))
            .movable(false)
            .titlebar(false)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, "Moves");

                // 0 -- 0
                // 1 -- 1
                // 2 -- 1
                // 3 -- 2
                // 4 -- 2
                // 5 -- 3
                let group_count = (game.move_count() + 1) / 2;
                let move_count = game.move_count();

                for i in 1..=group_count {
                    Group::new(hash!("moves", i), vec2(MOVES_LIST_WIDTH as f32, 30.)).ui( ui, |ui| {
                        ui.label(None, &format!("{}", i));

                        for j in 0..=1 {
                            let move_i = (i as i32 *2 + (j as i32 - 1)) as usize - 1;
                            if move_count > move_i && ui.button(Vec2::new(move_no_w + j as f32 * ((MOVES_LIST_WIDTH as f32 - move_no_w) / 2.), 0.), &game.get_move_in_chess_notation(move_i)) {
                                self.show_move(game, move_i + 1);
                            }
                        }
                    });
                }
            });

    }

    fn draw_legal_move_tiles_at(&self, coords: &Vec<Coord>) {
        for coord in coords {
            draw_circle(
                self.board_col.x
                    + (self.board_col.w / 8.0 / 2.0)
                    + coord.x as f32 * (self.board_col.w / 8.0),
                self.board_col.y
                    + (self.board_col.h / 8.0 / 2.0)
                    + (7 - coord.y) as f32 * (self.board_col.h / 8.0),
                12.0,
                HIGHLIGHT_COLOR,
            );
        }
    }

    #[cfg(feature = "assets")]
    async fn keys_swap_textures(&mut self) {
        if input::is_key_pressed(KeyCode::B) {
            self.board_tex_index += 1;
            if self.board_tex_index >= BOARD_TEXTURES.len() {
                self.board_tex_index = 0;
            }

            self.board_tex = load_texture(BOARD_TEXTURES[self.board_tex_index]).await;

            println!("board tex: {}", BOARD_TEXTURES[self.board_tex_index]);
        }

        if input::is_key_pressed(KeyCode::P) {
            self.piece_tex_index += 1;
            if self.piece_tex_index >= PIECE_TEXTURES.len() {
                self.piece_tex_index = 0;
            }

            self.pieces_tex = load_texture(PIECE_TEXTURES[self.piece_tex_index]).await;
            println!("piece tex: {}", PIECE_TEXTURES[self.piece_tex_index]);
        }
    }

    #[cfg(not(feature = "assets"))]
    fn keys_swap_textures(&mut self) {}

    #[cfg(feature = "assets")]
    fn draw_board(&self) {
        let board_params = DrawTextureParams {
            dest_size: Some(Vec2 {
                x: self.board_col.w,
                y: self.board_col.h,
            }),
            source: None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };
        draw_texture_ex(
            self.board_tex,
            self.board_col.x,
            self.board_col.y,
            WHITE,
            board_params,
        );
    }

    #[cfg(not(feature = "assets"))]
    fn draw_board(&self) {
        draw_rectangle(
            self.board_col.x,
            self.board_col.y,
            self.board_col.w,
            self.board_col.h,
            BROWN,
        );
    }

    pub async fn draw(&mut self, game: &mut GameState) {
        self.keys_swap_textures().await;

        clear_background(DARKBROWN);

        self.draw_board();

        //draw tiles of last move

        if self.viewed_move > 0 {
            let last_move = game.get_move(self.viewed_move - 1);
            let coord_from: Coord;
            let coord_to: Coord;

            match last_move {
                Move::PieceMove {
                    piece: _,
                    tile_from,
                    tile_to,
                    is_en_passant: _,
                } => {
                    coord_from = Coord::from(tile_from);
                    coord_to = Coord::from(tile_to);
                }
                Move::PieceMoveWithPromotion {
                    tile_from,
                    tile_to,
                    promotion: _,
                } => {
                    coord_from = Coord::from(tile_from);
                    coord_to = Coord::from(tile_to);
                }
                Move::CastleShort => match game.whose_turn().the_other_one() {
                    ChessTeam::Black => {
                        coord_from = Coord { x: 4, y: 7 };
                        coord_to = Coord { x: 6, y: 7 };
                    }
                    ChessTeam::White => {
                        coord_from = Coord { x: 4, y: 0 };
                        coord_to = Coord { x: 6, y: 0 };
                    }
                },
                Move::CastleLong => match game.whose_turn().the_other_one() {
                    ChessTeam::Black => {
                        coord_from = Coord { x: 4, y: 7 };
                        coord_to = Coord { x: 2, y: 7 };
                    }
                    ChessTeam::White => {
                        coord_from = Coord { x: 4, y: 0 };
                        coord_to = Coord { x: 2, y: 0 };
                    }
                },
            }

            let col_from = self.get_coord_col(coord_from);
            let col_to = self.get_coord_col(coord_to);

            draw_rectangle(
                col_from.x,
                col_from.y,
                col_from.w,
                col_from.h,
                LAST_MOVE_COLOR,
            );
            draw_rectangle(col_to.x, col_to.y, col_to.w, col_to.h, LAST_MOVE_COLOR);
        }

        // cycle move display with arrow_left and arrow_right
        if input::is_key_pressed(KeyCode::Left) { 
            self.show_move(game, cmp::max(0,self.viewed_move as i32 - 1) as usize);
        }

        if input::is_key_pressed(KeyCode::Right) { 
            self.show_move(game, cmp::min(game.move_count(),self.viewed_move + 1));
        }

        if input::is_key_pressed(KeyCode::Down) { 
            self.show_move(game, game.move_count());
        }

        if input::is_key_pressed(KeyCode::Up) { 
            self.show_move(game, 0);
        }

        if !input::is_mouse_button_down(MouseButton::Left) && self.is_dragged {
            // println!("dragged stopped!");
            self.is_dragged = false;

            let mouse_vec = input::mouse_position();
            let mouse_vec = Vec2 {
                x: mouse_vec.0,
                y: mouse_vec.1,
            };
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;
                let board_coord = Coord {
                    x: coord_x,
                    y: 7 - coord_y,
                };

                let res = self.attempt_move_execution(self.dragged_piece_i, board_coord, game);

                if res.is_ok() {
                    self.viewed_move = game.move_count();
                    self.handle_end_state(game);
                }
            }

            self.sync_board(&game.get_board());
        }

        if input::is_mouse_button_pressed(MouseButton::Left) && !self.is_dragged {
            // println!("mouse click! at {:?}", input::mouse_position());
            //check if u clicked the box
            let mouse_vec = input::mouse_position();

            let mouse_vec = Vec2 {
                x: mouse_vec.0,
                y: mouse_vec.1,
            };

            for (i, piece) in self.pieces.iter().enumerate() {
                if piece.col.is_in_box(mouse_vec) {
                    // println!("clicked on box! dragged = true");
                    if game.whose_turn() != piece.team {
                        continue;
                    }
                    self.is_dragged = true;

                    // populate dragged legal moves
                    let last_move = game.get_last_move();
                    let board = game.get_board();
                    self.dragged_legal_moves = board
                        .get_legal_moves_of_piece_in_tile(
                            Tile::try_from(self.pieces[i].pos).unwrap(),
                            last_move,
                        )
                        .unwrap();

                    // self.drag_offset.x = mouse_vec.x - piece.col.x;
                    // self.drag_offset.y = mouse_vec.y - piece.col.y;
                    self.dragged_piece_i = i;

                    // println!("drag offset {}", drag_offset);
                    break;
                }
            }
        }

        for (i, piece) in self.pieces.iter().enumerate() {
            if self.is_dragged && i != self.dragged_piece_i || !self.is_dragged {
                #[cfg(feature = "assets")]
                {
                    piece.draw(&self.pieces_tex);
                }
                #[cfg(not(feature = "assets"))]
                {
                    piece.draw();
                }
            }
        }

        if self.is_dragged {
            let mouse_vec = input::mouse_position();

            let mut pi = &mut self.pieces[self.dragged_piece_i];

            pi.col.x = mouse_vec.0 - pi.col.w / 2.0;
            pi.col.y = mouse_vec.1 - pi.col.h / 2.0;

            self.draw_legal_move_tiles_at(&self.dragged_legal_moves);

            //frame the tile on hover
            let mouse_vec = Vec2 {
                x: mouse_vec.0,
                y: mouse_vec.1,
            };
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;
                let board_coord = Coord {
                    x: coord_x,
                    y: 7 - coord_y,
                };

                let col = self.get_coord_col(board_coord);

                draw_rectangle_lines(col.x, col.y, col.w, col.h, 15.0, HIGHLIGHT_COLOR);
            }
        }

        if input::is_mouse_button_down(MouseButton::Right) && self.is_dragged {
            self.is_dragged = false;
            self.sync_board(&game.get_board());
        }

        if self.is_dragged {
            #[cfg(feature = "assets")]
            {
                self.pieces[self.dragged_piece_i].draw(&self.pieces_tex);
            }
            #[cfg(not(feature = "assets"))]
            {
                self.pieces[self.dragged_piece_i].draw();
            }
        }

        //handle arrow input
        if input::is_mouse_button_pressed(MouseButton::Right) && !self.is_dragged {
            let mouse_vec = input::mouse_position();
            let mouse_vec = Vec2 {
                x: mouse_vec.0,
                y: mouse_vec.1,
            };
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;

                self.coord_on_right_click_press = Some(Coord {
                    x: coord_x,
                    y: 7 - coord_y,
                });

                draw_rectangle(
                    self.board_col.x + (self.board_col.w / 8.0) * coord_x as f32,
                    self.board_col.y + (self.board_col.h / 8.0) * coord_y as f32,
                    self.board_col.w / 8.0,
                    self.board_col.h / 8.0,
                    Color {
                        r: HIGHLIGHT_COLOR_RGB.0,
                        g: HIGHLIGHT_COLOR_RGB.1,
                        b: HIGHLIGHT_COLOR_RGB.2,
                        a: 0.2,
                    },
                );
            }
        }

        if input::is_mouse_button_released(MouseButton::Right) {
            let mouse_vec = input::mouse_position();
            let mouse_vec = Vec2 {
                x: mouse_vec.0,
                y: mouse_vec.1,
            };
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;
                let coord_to = Coord {
                    x: coord_x,
                    y: 7 - coord_y,
                };

                self.coord_on_right_click_release = Some(coord_to);

                if let Some(coord_from) = self.coord_on_right_click_press {
                    if coord_from == coord_to {
                        let val = Arrow::Circle(coord_from);
                        if self.arrows.contains(&val) {
                            self.arrows.retain(|a| *a != val);
                        } else {
                            self.arrows.push(val);
                        }
                    } else if (coord_to - coord_from).magnitude().is_some() {
                        let val = Arrow::Arrow(coord_from, coord_to);
                        if self.arrows.contains(&val) {
                            self.arrows.retain(|a| *a != val);
                        } else {
                            self.arrows.push(val);
                        }
                    }
                }
            }
        }

        //draw arrows
        for arrow in &self.arrows {
            match arrow {
                Arrow::Arrow(coord_from, coord_to) => {
                    let t = 10.0; //stem thiccness
                    let t_offset;

                    // y goes top bottom
                    if coord_to.x > coord_from.x && coord_to.y > coord_from.y {
                        // arrow -> top right
                        t_offset = Vec2 { x: -1.0, y: 0.0 }.normalize();
                    } else if coord_to.x > coord_from.x && coord_to.y < coord_from.y {
                        // arrow -> bottom right
                        t_offset = Vec2 { x: 0.0, y: -1.0 }.normalize();
                    } else if coord_to.x > coord_from.x && coord_to.y == coord_to.y {
                        // arrow -> right
                        t_offset = Vec2 { x: -1.0, y: -1.0 }.normalize();
                    } else if coord_to.x < coord_from.x && coord_to.y > coord_from.y {
                        // arrow -> top left
                        t_offset = Vec2 { x: 0.0, y: 1.0 }.normalize();
                    } else if coord_to.x < coord_from.x && coord_to.y < coord_from.y {
                        // arrow -> bottom left
                        t_offset = Vec2 { x: 1.0, y: 0.0 }.normalize();
                    } else if coord_to.x == coord_from.x && coord_to.y > coord_from.y {
                        // arrow -> up
                        t_offset = Vec2 { x: -1.0, y: 1.0 }.normalize();
                    } else if coord_to.x == coord_from.x && coord_to.y < coord_from.y {
                        // arrow -> down
                        t_offset = Vec2 { x: 1.0, y: -1.0 }.normalize();
                    } else {
                        //arrow -> left
                        t_offset = Vec2 { x: 1.0, y: 1.0 }.normalize();
                    }

                    let tile_w = self.board_col.w / 8.0;

                    // arrow stem

                    // y:self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_from.y) as f32 + t,
                    // if it is a diagonal to the top right
                    draw_triangle(
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_from.x as f32
                                + t_offset.x * t,
                            y: self.board_col.y
                                + tile_w / 2.0
                                + tile_w * (7 - coord_from.y) as f32
                                + t_offset.y * t,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_to.x as f32
                                + t_offset.x * t,
                            y: self.board_col.y
                                + tile_w / 2.0
                                + tile_w * (7 - coord_to.y) as f32
                                + t_offset.y * t,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_to.x as f32
                                + t_offset.y * t,
                            y: self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                                - t_offset.x * t,
                        },
                        HIGHLIGHT_COLOR,
                    );
                    draw_triangle(
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_from.x as f32
                                + t_offset.x * t,
                            y: self.board_col.y
                                + tile_w / 2.0
                                + tile_w * (7 - coord_from.y) as f32
                                + t_offset.y * t,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_from.x as f32
                                + t_offset.y * t,
                            y: self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_from.y) as f32
                                - t_offset.x * t,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_to.x as f32
                                + t_offset.y * t,
                            y: self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                                - t_offset.x * t,
                        },
                        HIGHLIGHT_COLOR,
                    );
                    // the tip
                    draw_triangle(
                        Vec2 {
                            x: self.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32,
                            y: self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_to.x as f32
                                + t_offset.x * t,
                            y: self.board_col.y
                                + tile_w / 2.0
                                + tile_w * (7 - coord_to.y) as f32
                                + t_offset.y * t,
                        },
                        Vec2 {
                            x: self.board_col.x
                                + tile_w / 2.0
                                + tile_w * coord_to.x as f32
                                + t_offset.y * t,
                            y: self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                                - t_offset.x * t,
                        },
                        HIGHLIGHT_COLOR,
                    );
                }
                Arrow::Circle(coord) => {
                    draw_rectangle(
                        self.board_col.x + (self.board_col.w / 8.0) * coord.x as f32,
                        self.board_col.y + (self.board_col.h / 8.0) * (7 - coord.y) as f32,
                        self.board_col.w / 8.0,
                        self.board_col.h / 8.0,
                        HIGHLIGHT_COLOR,
                    );
                }
            }
        }

        //draw moves ui
        self.draw_moves_ui(game);
    }
}

// in pixels
struct ColBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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
    col: ColBox,
    pos: Coord,
    team: ChessTeam,
    piece_type: ChessPiece,
}

impl Piece {
    fn init(board_col: &ColBox, pos: Coord, team: ChessTeam, piece_type: ChessPiece) -> Piece {
        let mut piece = Piece {
            col: ColBox {
                x: 0.0,
                y: 0.0,
                w: board_col.w / 8.0,
                h: board_col.w / 8.0,
            },
            pos,
            team,
            piece_type,
        };
        piece.update_col(board_col);
        piece
    }

    fn update_col(&mut self, board_col: &ColBox) {
        self.col.x = (board_col.w / 8.0) * (self.pos.x) as f32 + board_col.x;
        self.col.y = (board_col.h / 8.0) * (7 - self.pos.y) as f32 + board_col.y;
    }

    #[cfg(not(feature = "assets"))]
    fn draw(&self) {
        let color = match self.team {
            ChessTeam::Black => BLACK,
            ChessTeam::White => WHITE,
        };

        let text_color = match self.team {
            ChessTeam::Black => WHITE,
            ChessTeam::White => BLACK,
        };

        self.col.draw(color);
        let piece_str = format!("{}", self.piece_type);
        draw_text(
            piece_str.as_str(),
            self.col.x,
            self.col.y + self.col.h / 2.0,
            15.0,
            text_color,
        );
    }

    #[cfg(feature = "assets")]
    fn draw(&self, atlas_tex: &Texture2D) {
        let mut atlas_pos = match self.piece_type {
            ChessPiece::Pawn => 3,
            ChessPiece::Rook => 5,
            ChessPiece::Knight => 2,
            ChessPiece::Bishop => 0,
            ChessPiece::Queen => 4,
            ChessPiece::King => 1,
        };

        if self.team == ChessTeam::White {
            atlas_pos += 6;
        }

        let params = DrawTextureParams {
            dest_size: Some(Vec2 {
                x: self.col.w,
                y: self.col.h,
            }),
            source: Some(Rect {
                x: 150.0 * atlas_pos as f32,
                y: 0.0,
                w: 150.0,
                h: 150.0,
            }),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: None,
        };

        draw_texture_ex(*atlas_tex, self.col.x, self.col.y, WHITE, params);
    }
}
