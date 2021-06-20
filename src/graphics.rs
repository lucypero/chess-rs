use std::cmp;
use std::convert::TryFrom;

use chess::{
    self, Board, ChessPiece, ChessTeam, Coord, GameEndState, GameState, Move, MoveError, Tile,
};

use crate::MenuChange;
use egui::CtxRef;
use futures::executor::block_on;
use macroquad::input;
use macroquad::prelude::*;

use macroquad::ui::{
    hash,
    root_ui,
    widgets::{self, Group},
    Skin, // Drag, Ui,
};

use crate::multiplayer::MPState;
use crate::MainMenuState;

// player input that code outside graphics may be interested in.
// input is caught by this graphics module but it won't try to modify the game state
// that's why this enum is returned every frame to the caller so the caller can fiddle
// with the game state however it wants.
#[derive(Clone)]
pub enum PlayerInput {
    GoBack,
    Move(Move, Result<(), MoveError>),
}

// use crate::GameState as ProgramState;

const PIECE_DISPLAY_SIZE: u32 = 80;
const BOARD_PADDING: u32 = 30;
const MOVES_LIST_WIDTH: u32 = 200;

// Color used for legal move indicators, arrows, tile where the piece will end up, etc...

const HIGHLIGHT_COLOR_RGB: (f32, f32, f32) = (0.89, 0.596, 0.850); //pinki
const BACKGROUND_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

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

const PIECE_TEXTURES: [&str; 34] = [
    "assets/pieces/madware_tileset_128.png",
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

const BOARD_TEXTURES: [&str; 34] = [
    "assets/boards/cute_board.png",
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
        window_width: (PIECE_DISPLAY_SIZE * 8
            + BOARD_PADDING * 2
            + MOVES_LIST_WIDTH
            + BOARD_PADDING) as i32,
        window_height: (PIECE_DISPLAY_SIZE * 8 + BOARD_PADDING * 2) as i32,
        fullscreen: false,
        ..Default::default()
    }
}

#[derive(PartialEq)]
enum Arrow {
    Arrow(Coord, Coord), //x,y -> u,v
    KnightArrow(Coord, Coord),
    Circle(Coord), //x,y
}

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

    //viewed move (could be any move, not just the last one)
    viewed_move: usize,

    //promotion
    is_promotion_ui_shown: bool,
    promotion_move: Move,

    //ui styles
    skin1: Skin,

    //cache moves as strings
    moves_str: Vec<String>,

    //is board flipped (white or black perspective)
    is_board_flipped: bool,

    locked_team: Option<ChessTeam>,

    player_input_buffer: Option<PlayerInput>,

    is_board_locked: bool,
}

fn get_board_coord(tile: Tile) -> Coord {
    let mut coord = Coord::from(tile);
    coord.y = 7 - coord.y;
    coord
}

impl GfxState {
    // team: chess team that the client is playing as
    //   used for flipping the board and to lock input for the other team
    pub fn init(game: &mut GameState, team: Option<chess::ChessTeam>) -> GfxState {
        let piece_tex_index = 0;
        let board_tex_index = 0;

        let dragged_piece_i = 0;
        let is_dragged = false;

        let board_col = ColBox {
            x: BOARD_PADDING as f32,
            y: BOARD_PADDING as f32,
            w: PIECE_DISPLAY_SIZE as f32 * 8.0,
            h: PIECE_DISPLAY_SIZE as f32 * 8.0,
        };

        let skin1 = {
            let label_style = root_ui()
                .style_builder()
                // .text_color(Color::from_rgba(180, 180, 120, 255))
                .font_size(18)
                .build();

            let button_style = root_ui()
                .style_builder()
                // .background(Image::from_file_with_format(
                //     include_bytes!("../examples/ui_assets/button_background.png"),
                //     None,
                // ))
                // .background_margin(RectOffset::new(37.0, 37.0, 5.0, 5.0))
                .margin(RectOffset::new(5.0, 5.0, 5.0, 5.0))
                .color_hovered(Color::from_rgba(200, 200, 200, 255))
                .color_selected(Color::from_rgba(200, 200, 200, 255))
                // .text_color(Color::from_rgba(180, 180, 100, 255))
                .font_size(18)
                .build();

            Skin {
                label_style,
                button_style,
                ..root_ui().default_skin()
            }
        };

        let board_tex = block_on(load_texture(BOARD_TEXTURES[board_tex_index])).unwrap();
        let pieces_tex = block_on(load_texture(PIECE_TEXTURES[piece_tex_index])).unwrap();

        let viewed_move = cmp::max(0 as usize, game.move_count()) as usize;

        let mut is_board_flipped = false;
        let mut locked_team = None;

        if let Some(team) = team {
            match team {
                ChessTeam::Black => {
                    is_board_flipped = true;
                    locked_team = Some(ChessTeam::White);
                }
                ChessTeam::White => {
                    locked_team = Some(ChessTeam::Black);
                }
            }
        }

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
            viewed_move,
            promotion_move: Move::CastleLong,
            is_promotion_ui_shown: false,
            skin1,
            moves_str: vec![],
            is_board_flipped,
            locked_team,
            player_input_buffer: None,
            is_board_locked: false
        };

        state.sync_board(&mut game.get_board());

        state
    }

    pub fn set_team_lock(&mut self, team: Option<ChessTeam>) {
        self.locked_team = team;
    }

    //display board position at move [move_i]
    fn show_move(&mut self, game: &GameState, move_i: usize) {
        assert!(move_i <= game.move_count());
        let board = game.get_board_at(move_i);
        self.viewed_move = move_i;

        //updating last move

        self.sync_board(&board);

        self.clear_arrows();

        // if it is not the last move, lock the board (can't make any move)
    }

    fn get_coord_col(&self, coord: Coord) -> ColBox {
        if self.is_board_flipped {
            ColBox {
                x: self.board_col.x + (self.board_col.w / 8.0) * (7 - coord.x) as f32,
                y: self.board_col.y + (self.board_col.h / 8.0) * (coord.y) as f32,
                w: self.board_col.w / 8.0,
                h: self.board_col.w / 8.0,
            }
        } else {
            ColBox {
                x: self.board_col.x + (self.board_col.w / 8.0) * coord.x as f32,
                y: self.board_col.y + (self.board_col.h / 8.0) * (7 - coord.y) as f32,
                w: self.board_col.w / 8.0,
                h: self.board_col.w / 8.0,
            }
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
                self.is_board_flipped,
            ));
        }
    }

    //so this constructs the move and calls gamestate.perform_move
    fn construct_move(
        &mut self,
        piece_i: usize,
        coord_to: Coord,
        game: &mut GameState,
    ) -> Result<Move, MoveError> {
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
            the_move = Move::PieceMoveWithPromotion {
                tile_from,
                tile_to,
                promotion: ChessPiece::Queen,
            };

            let ep_square = game.en_passant_square;
            let moves = game
                .get_board()
                .get_legal_moves_of_piece_in_tile(tile_from, ep_square);

            if let Some(moves) = moves {
                if moves.contains(&coord_to) {
                    self.is_promotion_ui_shown = true;
                    self.is_board_locked = true;
                    self.promotion_move = the_move;

                    return Err(MoveError::PromotionPieceNotSpecified);
                }
            }
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

        Ok(the_move)
        // game.perform_move(the_move)
    }

    fn draw_promotion(&mut self, game: &mut GameState, egui_ctx: &CtxRef) {

        let mut pro_p = None;

        egui::Window::new("Promotion")
            .resizable(false)
            .title_bar(false)
            .fixed_pos(egui::pos2(
                    PIECE_DISPLAY_SIZE as f32 * 3. + BOARD_PADDING as f32,
                    PIECE_DISPLAY_SIZE as f32 * 3. + BOARD_PADDING as f32,
                    ))
            .show(egui_ctx, |ui| {
                if ui.add(egui::Button::new("Queen")).clicked() {
                    pro_p = Some(ChessPiece::Queen);
                }
                if ui.add(egui::Button::new("Knight")).clicked() {
                    pro_p = Some(ChessPiece::Knight);
                }
                if ui.add(egui::Button::new("Rook")).clicked() {
                    pro_p = Some(ChessPiece::Rook);
                }
                if ui.add(egui::Button::new("Bishop")).clicked() {
                    pro_p = Some(ChessPiece::Bishop);
                }
            });

        if let Some(p) = pro_p {
            self.is_promotion_ui_shown = false;
            self.is_board_locked = false;

            if let Move::PieceMoveWithPromotion {
                tile_from: _,
                tile_to: _,
                promotion: ref mut prom_piece,
            } = self.promotion_move
            {
                *prom_piece = p;
            }

            let res = game.perform_move(self.promotion_move);

            if res.is_ok() {
                self.move_was_made(game);
                self.sync_board(&game.get_board());
            }
        }
    }

    fn handle_end_state(&self, game: &mut GameState) {
        match game.get_end_state() {
            GameEndState::Checkmate => {
                println!(
                    "It's checkmate! {} has won!",
                    game.whose_turn().the_other_one()
                );
                // TODO(lucypero): Yeah I panic just to end the program hah
                // panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Draw => {
                println!("It's a draw!");
                // TODO(lucypero): Yeah I panic just to end the program hah
                // panic!("nothing went wrong, it's just that the game ended");
            }
            GameEndState::Running => {}
        }
    }

    fn draw_moves_ui(&mut self, game: &mut GameState, egui_ctx: &CtxRef) {
        // TODO(lucypero): this is crashing the game when you promote a pawn..

        const MOVE_NO_W: f32 = 30.;

        let group_count = (game.move_count() + 1) / 2;
        let move_count = game.move_count();

        egui::Window::new("Moves")
            // .anchor(egui::Align2::LEFT_TOP,
            //     egui::vec2(
            //         PIECE_DISPLAY_SIZE as f32 * 8. + BOARD_PADDING as f32 * 2.,
            //         BOARD_PADDING as f32,
            //     ))
            // .fixed_size(egui::vec2(MOVES_LIST_WIDTH as f32 + 2000.,
            //         PIECE_DISPLAY_SIZE as f32 * 8.))
            .fixed_pos(egui::pos2(
                PIECE_DISPLAY_SIZE as f32 * 8. + BOARD_PADDING as f32 * 2.,
                BOARD_PADDING as f32,
            ))
            .resizable(false)
            .title_bar(false)
            .show(egui_ctx, |ui| {
                ui.set_min_size(egui::vec2(
                    MOVES_LIST_WIDTH as f32,
                    PIECE_DISPLAY_SIZE as f32 * 8.,
                ));

                egui::Grid::new("my_grid")
                    .striped(true)
                    // .min_col_width(self.min_col_width)
                    // .max_col_width(self.max_col_width)
                    .show(ui, |ui| {
                        for i in 1..=group_count {
                            ui.label(&format!("{}", i));
                            for j in 0..=1 {
                                let move_i = (i as i32 * 2 + (j as i32 - 1)) as usize - 1;
                                if move_count > move_i
                                    && ui
                                        .add(egui::Button::new(self.moves_str[move_i].as_str()))
                                        .clicked()
                                {
                                    self.show_move(game, move_i + 1);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
    }

    fn draw_moves_ui_old(&mut self, game: &mut GameState) {
        const MOVE_NO_W: f32 = 30.;

        root_ui().push_skin(&self.skin1);
        widgets::Window::new(
            hash!(),
            vec2(
                PIECE_DISPLAY_SIZE as f32 * 8. + BOARD_PADDING as f32 * 2.,
                BOARD_PADDING as f32,
            ),
            vec2(MOVES_LIST_WIDTH as f32, PIECE_DISPLAY_SIZE as f32 * 8.),
        )
        .movable(false)
        .titlebar(false)
        .ui(&mut *root_ui(), |ui| {
            ui.label(None, "Moves");

            let group_count = (game.move_count() + 1) / 2;
            let move_count = game.move_count();

            for i in 1..=group_count {
                Group::new(hash!("moves", i), vec2(MOVES_LIST_WIDTH as f32, 30.))
                    .highlight(false)
                    .ui(ui, |ui| {
                        ui.label(None, &format!("{}", i));

                        for j in 0..=1 {
                            let move_i = (i as i32 * 2 + (j as i32 - 1)) as usize - 1;
                            if move_count > move_i
                                && ui.button(
                                    Vec2::new(
                                        MOVE_NO_W
                                            + j as f32
                                                * ((MOVES_LIST_WIDTH as f32 - MOVE_NO_W) / 2.),
                                        0.,
                                    ),
                                    self.moves_str[move_i].as_str(),
                                )
                            {
                                self.show_move(game, move_i + 1);
                            }
                        }
                    });
            }
        });
        root_ui().pop_skin();
    }

    fn draw_legal_move_tiles_at(&self, coords: &Vec<Coord>) {
        for coord in coords {
            if self.is_board_flipped {
                draw_circle(
                    self.board_col.x
                        + (self.board_col.w / 8.0 / 2.0)
                        + (7 - coord.x) as f32 * (self.board_col.w / 8.0),
                    self.board_col.y
                        + (self.board_col.h / 8.0 / 2.0)
                        + (coord.y) as f32 * (self.board_col.h / 8.0),
                    12.0,
                    HIGHLIGHT_COLOR,
                );
            } else {
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
    }

    fn keys_swap_textures(&mut self) {
        if input::is_key_pressed(KeyCode::B) {
            self.board_tex_index += 1;
            if self.board_tex_index >= BOARD_TEXTURES.len() {
                self.board_tex_index = 0;
            }

            self.board_tex = block_on(load_texture(BOARD_TEXTURES[self.board_tex_index])).unwrap();

            println!("board tex: {}", BOARD_TEXTURES[self.board_tex_index]);
        }

        if input::is_key_pressed(KeyCode::P) {
            self.piece_tex_index += 1;
            if self.piece_tex_index >= PIECE_TEXTURES.len() {
                self.piece_tex_index = 0;
            }

            self.pieces_tex = block_on(load_texture(PIECE_TEXTURES[self.piece_tex_index])).unwrap();
            println!("piece tex: {}", PIECE_TEXTURES[self.piece_tex_index]);
        }
    }

    fn draw_board(&self) {
        let board_params = DrawTextureParams {
            dest_size: Some(vec2(self.board_col.w, self.board_col.h)),
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

    fn flip_board(&mut self) {
        self.is_board_flipped = !self.is_board_flipped;
        for piece in &mut self.pieces {
            piece.flip(&self.board_col);
        }
    }

    fn move_was_made(&mut self, game: &mut GameState) {
        self.viewed_move = game.move_count();
        self.moves_str
            .push(game.get_move_in_chess_notation(self.viewed_move - 1));
        self.clear_arrows();
        self.handle_end_state(game);
    }

    pub fn move_was_made_from_other_client(&mut self, game: &mut GameState) {
        self.move_was_made(game);
        self.sync_board(&game.get_board());
    }

    fn clear_arrows(&mut self) {
        self.arrows.clear();
    }

    fn draw_arrows(&mut self) {
        fn draw_line(gfx: &GfxState, coord_from: Coord, coord_to: Coord, arrow_head: bool) {
            let t = 10.0; //stem thiccness
            let t_offset;

            // y goes top bottom
            if coord_to.x > coord_from.x && coord_to.y > coord_from.y {
                // arrow -> top right
                t_offset = vec2(-1.0, 0.0).normalize();
            } else if coord_to.x > coord_from.x && coord_to.y < coord_from.y {
                // arrow -> bottom right
                t_offset = vec2(0.0, -1.0).normalize();
            } else if coord_to.x > coord_from.x && coord_to.y == coord_to.y {
                // arrow -> right
                t_offset = vec2(-1.0, -1.0).normalize();
            } else if coord_to.x < coord_from.x && coord_to.y > coord_from.y {
                // arrow -> top left
                t_offset = vec2(0.0, 1.0).normalize();
            } else if coord_to.x < coord_from.x && coord_to.y < coord_from.y {
                // arrow -> bottom left
                t_offset = vec2(1.0, 0.0).normalize();
            } else if coord_to.x == coord_from.x && coord_to.y > coord_from.y {
                // arrow -> up
                t_offset = vec2(-1.0, 1.0).normalize();
            } else if coord_to.x == coord_from.x && coord_to.y < coord_from.y {
                // arrow -> down
                t_offset = vec2(1.0, -1.0).normalize();
            } else {
                //arrow -> left
                t_offset = vec2(1.0, 1.0).normalize();
            }

            let tile_w = gfx.board_col.w / 8.0;

            // arrow stem

            // y:self.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_from.y) as f32 + t,
            // if it is a diagonal to the top right
            draw_triangle(
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_from.x as f32 + t_offset.x * t,
                    gfx.board_col.y
                        + tile_w / 2.0
                        + tile_w * (7 - coord_from.y) as f32
                        + t_offset.y * t,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32 + t_offset.x * t,
                    gfx.board_col.y
                        + tile_w / 2.0
                        + tile_w * (7 - coord_to.y) as f32
                        + t_offset.y * t,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32 + t_offset.y * t,
                    gfx.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                        - t_offset.x * t,
                ),
                HIGHLIGHT_COLOR,
            );
            draw_triangle(
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_from.x as f32 + t_offset.x * t,
                    gfx.board_col.y
                        + tile_w / 2.0
                        + tile_w * (7 - coord_from.y) as f32
                        + t_offset.y * t,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_from.x as f32 + t_offset.y * t,
                    gfx.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_from.y) as f32
                        - t_offset.x * t,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32 + t_offset.y * t,
                    gfx.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                        - t_offset.x * t,
                ),
                HIGHLIGHT_COLOR,
            );

            if !arrow_head {
                return;
            }

            // the arrow head
            draw_triangle(
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32,
                    gfx.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32 + t_offset.x * t,
                    gfx.board_col.y
                        + tile_w / 2.0
                        + tile_w * (7 - coord_to.y) as f32
                        + t_offset.y * t,
                ),
                vec2(
                    gfx.board_col.x + tile_w / 2.0 + tile_w * coord_to.x as f32 + t_offset.y * t,
                    gfx.board_col.y + tile_w / 2.0 + tile_w * (7 - coord_to.y) as f32
                        - t_offset.x * t,
                ),
                HIGHLIGHT_COLOR,
            );
        }

        //draw arrows
        for arrow in &self.arrows {
            match arrow {
                Arrow::Arrow(coord_from_orig, coord_to_orig) => {
                    let coord_from: Coord;
                    let coord_to: Coord;

                    if self.is_board_flipped {
                        coord_from = Coord {
                            x: 7 - coord_from_orig.x,
                            y: 7 - coord_from_orig.y,
                        };
                        coord_to = Coord {
                            x: 7 - coord_to_orig.x,
                            y: 7 - coord_to_orig.y,
                        };
                    } else {
                        coord_from = *coord_from_orig;
                        coord_to = *coord_to_orig;
                    }

                    draw_line(self, coord_from, coord_to, true);
                }
                Arrow::Circle(coord_orig) => {
                    let coord: Coord;

                    if self.is_board_flipped {
                        coord = Coord {
                            x: 7 - coord_orig.x,
                            y: 7 - coord_orig.y,
                        };
                    } else {
                        coord = *coord_orig;
                    }

                    draw_rectangle(
                        self.board_col.x + (self.board_col.w / 8.0) * coord.x as f32,
                        self.board_col.y + (self.board_col.h / 8.0) * (7 - coord.y) as f32,
                        self.board_col.w / 8.0,
                        self.board_col.h / 8.0,
                        HIGHLIGHT_COLOR,
                    );
                }
                Arrow::KnightArrow(coord_from_orig, coord_to_orig) => {
                    let coord_from: Coord;
                    let coord_to: Coord;

                    if self.is_board_flipped {
                        coord_from = Coord {
                            x: 7 - coord_from_orig.x,
                            y: 7 - coord_from_orig.y,
                        };
                        coord_to = Coord {
                            x: 7 - coord_to_orig.x,
                            y: 7 - coord_to_orig.y,
                        };
                    } else {
                        coord_from = *coord_from_orig;
                        coord_to = *coord_to_orig;
                    }

                    let first_line_to = Coord {
                        x: coord_to.x,
                        y: coord_from.y,
                    };

                    draw_line(self, coord_from, first_line_to, false);
                    draw_line(self, first_line_to, coord_to, true);
                }
            }
        }
    }

    fn handle_arrows_input(&mut self) {
        // start arrow registration when pressing right mouse button
        if input::is_mouse_button_pressed(MouseButton::Right) && !self.is_dragged {
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;

                if self.is_board_flipped {
                    self.coord_on_right_click_press = Some(Coord {
                        x: 7 - coord_x,
                        y: coord_y,
                    });
                } else {
                    self.coord_on_right_click_press = Some(Coord {
                        x: coord_x,
                        y: 7 - coord_y,
                    });
                }
            }
        }

        // register arrow when right mouse button is released
        if input::is_mouse_button_released(MouseButton::Right) {
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32;

                let coord_to;

                if self.is_board_flipped {
                    coord_to = Coord {
                        x: 7 - coord_x,
                        y: coord_y,
                    };
                } else {
                    coord_to = Coord {
                        x: coord_x,
                        y: 7 - coord_y,
                    };
                }

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
                    } else {
                        let coord_distance = coord_from.distance(coord_to);
                        if (coord_distance.x.abs() == 2 && coord_distance.y.abs() == 1)
                            || (coord_distance.x.abs() == 1 && coord_distance.y.abs() == 2)
                        {
                            let val = Arrow::KnightArrow(coord_from, coord_to);
                            if self.arrows.contains(&val) {
                                self.arrows.retain(|a| *a != val);
                            } else {
                                self.arrows.push(val);
                            }
                        }
                    }
                }
            }
        }

        self.draw_arrows();
    }

    pub fn draw(&mut self, game: &mut GameState) {

        clear_background(BACKGROUND_COLOR);

        if input::is_key_pressed(KeyCode::F) {
            println!("fen output: {}", game.get_fen());
        }

        if input::is_key_pressed(KeyCode::Backspace) {
            self.player_input_buffer =  Some(PlayerInput::GoBack);
        }

        if input::is_key_pressed(KeyCode::T) {
            self.flip_board();
        }

        egui_macroquad::ui(|egui_ctx| {
            self.draw_moves_ui(game, egui_ctx);

            if self.is_promotion_ui_shown {
                self.draw_promotion(game, egui_ctx);
            }
        });

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

        // if you click the board and you are watching a previous move, jump to last move
        if input::is_mouse_button_down(MouseButton::Left) && self.viewed_move != game.move_count() {
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);

            if self.board_col.is_in_box(mouse_vec) {
                self.show_move(game, game.move_count());
            }
        }

        // clear arrows if u click the board
        if input::is_mouse_button_down(MouseButton::Left) {
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);

            if self.board_col.is_in_box(mouse_vec) {
                self.clear_arrows();
            }
        }

        // cycle move display with arrow_left and arrow_right
        if input::is_key_pressed(KeyCode::Left) {
            self.show_move(game, cmp::max(0, self.viewed_move as i32 - 1) as usize);
        }

        if input::is_key_pressed(KeyCode::Right) {
            self.show_move(game, cmp::min(game.move_count(), self.viewed_move + 1));
        }

        if input::is_key_pressed(KeyCode::Down) {
            self.show_move(game, game.move_count());
        }

        if input::is_key_pressed(KeyCode::Up) {
            self.show_move(game, 0);
        }

        // When you stop pressing the mouse click to commit to a move
        if !input::is_mouse_button_down(MouseButton::Left) && self.is_dragged {
            self.is_dragged = false;

            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = std::cmp::min(
                    (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as u32,
                    7,
                ) as i32;
                let coord_y = std::cmp::min(
                    (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as u32,
                    7,
                ) as i32;
                let board_coord;
                if self.is_board_flipped {
                    board_coord = Coord {
                        x: 7 - coord_x,
                        y: coord_y,
                    };
                } else {
                    board_coord = Coord {
                        x: coord_x,
                        y: 7 - coord_y,
                    };
                }

                let the_move = self.construct_move(self.dragged_piece_i, board_coord, game);
                if let Ok(the_move) = the_move {
                    let move_res = game.perform_move(the_move);

                    if move_res.is_ok() {
                        self.move_was_made(game);
                        self.player_input_buffer = Some(PlayerInput::Move(the_move, move_res));
                    }
                }

                // let res = self.attempt_move_execution(self.dragged_piece_i, board_coord, game);

                // if res.is_ok() {
                //     self.viewed_move = game.move_count();
                //     self.moves_str
                //         .push(game.get_move_in_chess_notation(self.viewed_move - 1));
                //     self.handle_end_state(game);
                // }
            }

            self.sync_board(&game.get_board());
        }

        // When you try to grab a piece with the mouse and mouse click
        if input::is_mouse_button_pressed(MouseButton::Left)
            && !self.is_dragged
            && self.viewed_move == game.move_count()
        {
            // println!("mouse click! at {:?}", input::mouse_position());
            //check if u clicked the box
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);

            for (i, piece) in self.pieces.iter().enumerate() {
                if piece.col.is_in_box(mouse_vec) {
                    // println!("clicked on box! dragged = true");
                    if game.whose_turn() != piece.team  || 
                        self.is_board_locked
                    {
                        continue;
                    }


                    // check if the team is not locked
                    if let Some(team) = self.locked_team {
                        if piece.team == team {
                            continue;
                        }
                    }

                    self.is_dragged = true;

                    // populate dragged legal moves
                    let the_tile = Tile::try_from(self.pieces[i].pos).unwrap();

                    let ep_square = game.en_passant_square;
                    let board = game.get_board();
                    self.dragged_legal_moves = board
                        .get_legal_moves_of_piece_in_tile(
                            the_tile,
                            ep_square,
                        )
                        .unwrap();
                    //adding castling move
                    self.dragged_legal_moves.append(
                        &mut board.get_king_casle_moves(the_tile,ep_square));

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
                piece.draw(&self.pieces_tex);
            }
        }

        if self.is_dragged {
            let mouse_vec = input::mouse_position();
            let mouse_vec = vec2(mouse_vec.0, mouse_vec.1);

            let mut pi = &mut self.pieces[self.dragged_piece_i];

            pi.col.x = mouse_vec.x - pi.col.w / 2.0;
            pi.col.y = mouse_vec.y - pi.col.h / 2.0;

            self.draw_legal_move_tiles_at(&self.dragged_legal_moves);

            //frame the tile on hover
            if self.board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = std::cmp::min(
                    (((mouse_vec.x - self.board_col.x) / self.board_col.w) * 8.0) as i32,
                    7,
                );
                let coord_y = std::cmp::min(
                    (((mouse_vec.y - self.board_col.y) / self.board_col.h) * 8.0) as i32,
                    7,
                );
                let board_coord = Coord {
                    x: coord_x,
                    y: 7 - coord_y,
                };

                let col = ColBox {
                    x: self.board_col.x + (self.board_col.w / 8.0) * board_coord.x as f32,
                    y: self.board_col.y + (self.board_col.h / 8.0) * (7 - board_coord.y) as f32,
                    w: self.board_col.w / 8.0,
                    h: self.board_col.w / 8.0,
                };

                draw_rectangle_lines(col.x, col.y, col.w, col.h, 15.0, HIGHLIGHT_COLOR);
            }
        }

        if input::is_mouse_button_down(MouseButton::Right) && self.is_dragged {
            self.is_dragged = false;
            self.sync_board(&game.get_board());
        }

        if self.is_dragged {
            self.pieces[self.dragged_piece_i].draw(&self.pieces_tex);
        }

        self.handle_arrows_input();

        egui_macroquad::draw();
    }

    pub fn consume_player_input_buffer(&mut self) -> Option<PlayerInput> {
        let pi = self.player_input_buffer.clone();
        self.player_input_buffer = None;
        return pi;
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
    flipped_board: bool,
}

impl Piece {
    fn init(
        board_col: &ColBox,
        pos: Coord,
        team: ChessTeam,
        piece_type: ChessPiece,
        flipped_board: bool,
    ) -> Piece {
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
            flipped_board,
        };
        piece.update_col(board_col);
        piece
    }

    fn flip(&mut self, board_col: &ColBox) {
        self.flipped_board = !self.flipped_board;
        self.update_col(board_col)
    }

    fn update_col(&mut self, board_col: &ColBox) {
        if self.flipped_board {
            self.col.x = (board_col.w / 8.0) * (7 - self.pos.x) as f32 + board_col.x;
            self.col.y = (board_col.h / 8.0) * (self.pos.y) as f32 + board_col.y;
        } else {
            self.col.x = (board_col.w / 8.0) * (self.pos.x) as f32 + board_col.x;
            self.col.y = (board_col.h / 8.0) * (7 - self.pos.y) as f32 + board_col.y;
        }
    }

    fn draw(&self, atlas_tex: &Texture2D) {
        let params;

        if atlas_tex.width() == 1536. && atlas_tex.height() == 512. {
            let atlas_pos_x = match self.piece_type {
                ChessPiece::Pawn => 5,
                ChessPiece::Rook => 0,
                ChessPiece::Knight => 1,
                ChessPiece::Bishop => 2,
                ChessPiece::Queen => 3,
                ChessPiece::King => 4,
            };

            let atlas_pos_y = match self.team {
                ChessTeam::Black => 0,
                ChessTeam::White => 1,
            };

            params = DrawTextureParams {
                dest_size: Some(vec2(self.col.w, self.col.h)),
                source: Some(Rect {
                    x: 256.0 * atlas_pos_x as f32,
                    y: 256.0 * atlas_pos_y as f32,
                    w: 256.0,
                    h: 256.0,
                }),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            };
        } else if atlas_tex.width() == 768. && atlas_tex.height() == 256. {
            let atlas_pos_x = match self.piece_type {
                ChessPiece::Pawn => 5,
                ChessPiece::Rook => 0,
                ChessPiece::Knight => 1,
                ChessPiece::Bishop => 2,
                ChessPiece::Queen => 3,
                ChessPiece::King => 4,
            };

            let atlas_pos_y = match self.team {
                ChessTeam::Black => 0,
                ChessTeam::White => 1,
            };

            params = DrawTextureParams {
                dest_size: Some(vec2(self.col.w, self.col.h)),
                source: Some(Rect {
                    x: 128.0 * atlas_pos_x as f32,
                    y: 128.0 * atlas_pos_y as f32,
                    w: 128.0,
                    h: 128.0,
                }),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            };
        } else {
            let mut atlas_pos_x = match self.piece_type {
                ChessPiece::Pawn => 3,
                ChessPiece::Rook => 5,
                ChessPiece::Knight => 2,
                ChessPiece::Bishop => 0,
                ChessPiece::Queen => 4,
                ChessPiece::King => 1,
            };

            if self.team == ChessTeam::White {
                atlas_pos_x += 6;
            }

            params = DrawTextureParams {
                dest_size: Some(vec2(self.col.w, self.col.h)),
                source: Some(Rect {
                    x: 150.0 * atlas_pos_x as f32,
                    y: 0.0,
                    w: 150.0,
                    h: 150.0,
                }),
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            };
        }

        draw_texture_ex(*atlas_tex, self.col.x, self.col.y, WHITE, params);
    }
}

pub fn draw_main_menu(mm_state: &mut MainMenuState) -> MenuChange {
    let mut play_button_clicked = false;
    let mut play_fen_clicked = false;

    let mut play_client_clicked = false;

    let mut res = MenuChange::None;

    #[cfg(feature = "cheats")]
    let mut preset_position: Option<GameState> = None;

    clear_background(BACKGROUND_COLOR);

    //listen for keyboard events 


    egui_macroquad::ui(|egui_ctx| match mm_state {
        MainMenuState::Main { ip_string: _ } => {
            egui::Window::new("Main Menu!").show(egui_ctx, |ui| {
                if ui.add(egui::Button::new("Play against yourself")).clicked() {
                    res = MenuChange::Menu(MainMenuState::PlayMenu {
                        fen_string: String::new(),
                    });
                }
                // ui.add(egui::TextEdit::singleline(ip_string));
                if ui.add(egui::Button::new("Look for a player online")).clicked() {
                    play_client_clicked = true;
                }
            });
        }
        MainMenuState::PlayMenu { fen_string } => {
            egui::Window::new("Play").show(egui_ctx, |ui| {
                if ui.add(egui::Button::new("Play normal game")).clicked() {
                    play_button_clicked = true;
                }
                ui.add(egui::TextEdit::singleline(fen_string));
                if ui
                    .add(egui::Button::new("Play from FEN position"))
                    .clicked()
                {
                    play_fen_clicked = true;
                }
            });

            #[cfg(feature = "cheats")]
            egui::Window::new("Cheats").show(egui_ctx,|ui| {

                egui::CollapsingHeader::new("Play some preset positions")
                    .default_open(false)
                    .show(ui, |ui| {
                        if ui.add(egui::Button::new("Promotion Test")).clicked() {
                            preset_position = Some(chess::parse_fen("k4r2/3PP3/8/8/2p3p1/7P/1P1pp3/K7 w - - 0 1".to_string()).unwrap())
                        }
                        if ui.add(egui::Button::new("Notation Test")).clicked() {
                            preset_position = Some(chess::parse_fen("3r3r/1K1k4/8/R7/4Q2Q/8/8/R6Q w - - 0 54".to_string()).unwrap())
                        }
                    });
            });
        }
        MainMenuState::OptionsMenu => {}
    });

    egui_macroquad::draw();

    if play_button_clicked {
        res = MenuChange::Game(GameState::init());
    } else if play_fen_clicked {
        if let MainMenuState::PlayMenu { fen_string } = mm_state {
            let game = chess::parse_fen(fen_string.to_string());
            if let Some(game) = game {
                res = MenuChange::Game(game);
            }
        }
    } else if play_client_clicked {
        if let MainMenuState::Main { ip_string: _ } = mm_state {
            // let mp_state = MPState::init(ip_string.clone());
            // let mp_state = MPState::init(ip_string.clone());
            let mp_state = MPState::init("193.200.238.76:3333".to_string());
            res = MenuChange::MultiplayerGame(mp_state);
        }
    } else if preset_position.is_some() {
        res = MenuChange::Game(preset_position.unwrap());
    }

    res
}
