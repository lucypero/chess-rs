use macroquad::prelude::*;
use macroquad::input;

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

struct Coord {
    x : i32,
    y : i32
}

struct Piece<'a> {
    col : ColBox,
    pos : Coord,
    color : Color,
    board_col : &'a ColBox
}

impl<'a> Piece<'a> {

    fn init(board_col: &'a ColBox, pos:Coord,  color:Color) -> Piece {
        let mut piece = Piece{ board_col, col: ColBox{x:0.0, y:0.0, w: board_col.w / 8.0, h: board_col.w / 8.0}, pos, color};
        piece.update_col();
        piece
    }

    fn update_col(&mut self) {
        self.col.x = (self.board_col.w / 8.0) * self.pos.x as f32 + self.board_col.x;
        self.col.y = (self.board_col.h / 8.0) * self.pos.y as f32 + self.board_col.y;
    }

    fn draw(&self) {
        self.col.draw(self.color);
        draw_text("Pawn", self.col.x, self.col.y + self.col.h / 2.0, 20.0, WHITE);
    }
}

fn conf() -> Conf
{
    Conf {
        window_title: String::from("Chess-rs"),
        window_width: 1260,
        window_height: 768,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {

    let board_w : f32 = 400.0;

    // let tex = load_texture("assets/smiley.png").await;
    let mut dragged_piece_i = 0;
    let mut is_dragged = false;
    let mut drag_offset = Vec2{x:0.0, y:0.0};

    let board_col = ColBox {x: 30.0, y: 30.0, w: board_w, h: board_w};

    let mut pieces : Vec<Piece> = vec![
        Piece::init(&board_col, Coord{x:0, y:0}, RED),
        Piece::init(&board_col, Coord{x:1, y:0}, BLUE),
    ];

    loop {
        clear_background(WHITE);
 
        // draw_texture(tex, screen_width() / 2.0 , screen_height() / 2.0, WHITE);
        // draw_rectangle(col_box.x, col_box.y, col_box.w, col_box.h, BLUE);
        board_col.draw(BLACK);


        if !input::is_mouse_button_down(MouseButton::Left) && is_dragged{
            println!("dragged stopped!");
            is_dragged = false;

            let mouse_vec = input::mouse_position();
            let mouse_vec = Vec2{x:mouse_vec.0, y: mouse_vec.1};
            if board_col.is_in_box(mouse_vec) {
                //get tile where the mouse was in
                let coord_x = (((mouse_vec.x - board_col.x) / board_col.w) * 8.0) as i32;
                let coord_y = (((mouse_vec.y - board_col.y) / board_col.h) * 8.0) as i32;

                pieces[dragged_piece_i].pos.x = coord_x;
                pieces[dragged_piece_i].pos.y = coord_y;
                pieces[dragged_piece_i].update_col();

                println!("coord x: {}, coord y: {}", coord_x, coord_y);
            }
        }

        if input::is_mouse_button_pressed(MouseButton::Left) {
            // println!("mouse click! at {:?}", input::mouse_position());
            //check if u clicked the box
            let mouse_vec = input::mouse_position();

            println!("mouse pos: {:?}", mouse_vec);

            let mouse_vec = Vec2{x:mouse_vec.0, y: mouse_vec.1};

            for (i, piece) in pieces.iter().enumerate() {
                if piece.col.is_in_box(mouse_vec) {
                    println!("clicked on box! dragged = true");
                    is_dragged = true;
                    drag_offset.x = mouse_vec.x - piece.col.x;
                    drag_offset.y = mouse_vec.y - piece.col.y;
                    dragged_piece_i = i;
                    println!("drag offset {}", drag_offset);
                    break;
                }
            }

        }

        if is_dragged {
            let mouse_vec = input::mouse_position();
            pieces[dragged_piece_i].col.x = mouse_vec.0 - drag_offset.x;
            pieces[dragged_piece_i].col.y = mouse_vec.1 - drag_offset.y;
        }

        for piece in &pieces {
            piece.draw();
        }

        next_frame().await
    }
}
