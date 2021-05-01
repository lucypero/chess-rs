/*

EBNF grammar

piece_type = "r" | "R" | "b" | "B" | "n" | "N" | "k" | "K" | "q" | "Q";
piece = piece_type [file] [rank]
tile = "a1" ...
captures = "x"
check = "+"
checkmate = "#"
en_passant = "e.p"

move = [piece] [captures] tile [en_passant] [check | checkmate]

*/

#[derive(Debug, PartialEq)]
enum Node {
    Piece(char, char, char), // (piece_type, file, rank) '-' if unspecified
    Destination(char, char), // (file, rank) '-' if unspecified. rank may be '-' if pawn capture
    Captures,
    Check,
    Checkmate,
    CastleShort,
    CastleLong,
    EnPassant,
    PawnPromotion(char),
}

#[derive(Debug, PartialEq)]
pub enum MovePrimary {
    PieceMove {
        piece: (char, char, char),
        destination: (char, char),
        promotion: char,
    }, // (piece_type, file, rank) '-' if unspecified
    CastleShort,
    CastleLong,
}

#[derive(Debug, PartialEq)]
pub struct Move {
    pub primary: MovePrimary, //primary, or required info
    pub check: bool,
    pub checkmate: bool,
    pub en_passant: bool,
    pub captures: bool,
}

impl Move {
    fn construct(nodes: Vec<Node>) -> Move {
        let mut primary = MovePrimary::CastleShort;

        let mut piece = ('-', '-', '-');
        let mut destination = ('-', '-');
        let mut promotion = '-';

        let mut check = false;
        let mut checkmate = false;
        let mut en_passant = false;
        let mut captures = false;

        let mut is_piece_move = false;

        for node in nodes {
            match node {
                Node::Piece(p, f, r) => {
                    piece = (p, f, r);
                    is_piece_move = true;
                }
                Node::Destination(f, r) => {
                    destination = (f, r);
                    is_piece_move = true;
                }
                Node::Captures => {
                    captures = true;
                }
                Node::Check => {
                    check = true;
                }
                Node::Checkmate => {
                    checkmate = true;
                }
                Node::CastleShort => {
                    primary = MovePrimary::CastleShort;
                }
                Node::CastleLong => {
                    primary = MovePrimary::CastleLong;
                }
                Node::EnPassant => {
                    en_passant = true;
                }
                Node::PawnPromotion(p) => {
                    promotion = p;
                    is_piece_move = true;
                }
            }
        }

        if is_piece_move {
            primary = MovePrimary::PieceMove {
                piece,
                destination,
                promotion,
            }
        }

        Move {
            primary,
            check,
            checkmate,
            en_passant,
            captures,
        }
    }
}

fn is_there_tile_ahead(input: &[char], index: usize) -> bool {
    let mut i = index;

    while i < input.len() {
        let res = parse_destination(input, &mut i);
        if res.is_ok() {
            return true;
        }
        i += 1;
    }
    false
}

fn parse_pawn_capture(
    input: &[char],
    i: &mut usize,
    could_be_something_else: &mut bool,
) -> Result<Vec<Node>, ()> {
    let mut result: Vec<Node> = vec![];
    let mut j = *i;

    if is_file(input[j]) {
        if is_chess_piece(input[j]) {
            *could_be_something_else = true;
        } else {
            *could_be_something_else = false;
        }

        result.push(Node::Piece('-', input[j], '-'));
        j += 1;
        let mut char_ahead = safe_index(input, j)?;
        if let Ok(cap) = parse_captures(input, &mut j) {
            result.push(cap);
            char_ahead = safe_index(input, j)?;
        }
        if is_file(char_ahead) {
            j += 1;
            let mut rank: char = '-';
            if let Ok(char_ahead_ahead) = safe_index(input, j) {
                if is_rank(char_ahead_ahead) {
                    rank = char_ahead_ahead;
                    j += 1;
                }
            }

            //if there is a tile ahead, this is not a pawn capture
            if is_there_tile_ahead(input, j) {
                return Err(());
            }

            result.push(Node::Destination(char_ahead, rank));

            *i = j;
            return Ok(result);
        }
    }
    Err(())
}

// returns Err if found no piece. parsing could still succeed because this is optional
fn parse_piece(input: &[char], i: &mut usize) -> Result<Node, ()> {
    // -> look ahead, if rank
    //     -> look ahead, if there is tile anywhere:
    //        -> then piece = piece_type rank
    //        -> if not, then Err()
    // -> if not rank, if file
    //     -> look ahead, if rank
    //        -> look ahead, if there is tile anywhere:
    //          -> then piece = piece_type file rank
    //          -> if not, then piece = piece_type
    //     -> if not rank, then piece = piece_type file
    // -> if not rank or file
    //    -> then piece = piece_type

    let name = input[*i];
    if is_chess_piece(input[*i]) {
        let char_ahead = safe_index(input, *i + 1)?;
        if is_rank(char_ahead) {
            if is_there_tile_ahead(input, *i + 2) {
                *i += 2;
                return Ok(Node::Piece(name, '-', char_ahead));
            } else {
                return Err(());
            }
        } else if is_file(char_ahead) {
            let char_ahead_ahead = safe_index(input, *i + 2)?;
            if is_rank(char_ahead_ahead) {
                if is_there_tile_ahead(input, *i + 3) {
                    *i += 3;
                    return Ok(Node::Piece(name, char_ahead, char_ahead_ahead));
                } else {
                    *i += 1;
                    return Ok(Node::Piece(name, '-', '-'));
                }
            } else {
                *i += 2;
                return Ok(Node::Piece(name, char_ahead, '-'));
            }
        } else {
            return Ok(Node::Piece(name, '-', '-'));
        }
    }

    Err(())
}

fn parse_captures(input: &[char], i: &mut usize) -> Result<Node, ()> {
    if input[*i] == 'x' {
        *i += 1;
        Ok(Node::Captures)
    } else {
        Err(())
    }
}

fn parse_en_passant(input: &[char], i: &mut usize) -> Result<Node, ()> {
    if safe_index(input, *i + 3).is_ok() && input[*i..=*i + 3].iter().collect::<String>() == "e.p."
    {
        *i += 4;
        return Ok(Node::EnPassant);
    }

    Err(())
}

fn parse_check_or_mate(input: &[char], i: &mut usize) -> Result<Node, ()> {
    if safe_index(input, *i).is_err() {
        return Err(());
    }

    if input[*i] == '#' {
        *i += 1;
        Ok(Node::Checkmate)
    } else if input[*i] == '+' {
        *i += 1;
        Ok(Node::Check)
    } else {
        Err(())
    }
}

fn is_chess_piece(c: char) -> bool {
    c == 'r'
        || c == 'n'
        || c == 'b'
        || c == 'k'
        || c == 'q'
        || c == 'R'
        || c == 'N'
        || c == 'B'
        || c == 'K'
        || c == 'Q'
}

fn parse_pawn_promotion(input: &[char], i: &mut usize) -> Result<Node, ()> {
    let mut j = *i;
    let mut char_next = safe_index(input, j)?;

    // = is optional
    if char_next == '=' {
        j += 1;
        char_next = safe_index(input, j)?;
    }

    if is_chess_piece(char_next) {
        j += 1;
        *i = j;
        return Ok(Node::PawnPromotion(char_next));
    }
    Err(())
}

fn parse_castle(input: &[char], i: &mut usize) -> Result<Node, ()> {
    let input_string: String = input.iter().collect();
    if input_string.starts_with("o-o") || input_string.starts_with("O-O") {
        if input_string.starts_with("o-o-o") || input_string.starts_with("O-O-O") {
            *i += 5;
            return Ok(Node::CastleLong);
        } else {
            *i += 3;
            return Ok(Node::CastleShort);
        }
    }
    Err(())
}

//Parses a string representing a chess move in FIDE move notation ("eg: Be4")
// Returns a Vec<Node> representing the move. Err(()) if it could not be parsed
//recommended pre-processing of input: strip whitespace
pub fn parse(input: Vec<char>) -> Result<Vec<Move>, ()> {
    let mut results: Vec<Move> = vec![];

    let mut could_be_something_else = true;
    let mut pawn_branch_done = false;

    while could_be_something_else {
        let mut i = 0;
        let mut result: Vec<Node> = vec![];
        let mut pawn_move_parsed = false;

        if let Ok(castle) = parse_castle(&input, &mut i) {
            result.push(castle);
            could_be_something_else = false;
        } else {
            // pawn capture (eg: "exd5, ed")
            if !pawn_branch_done {
                if let Ok(mut pawn_capture) =
                    parse_pawn_capture(&input, &mut i, &mut could_be_something_else)
                {
                    pawn_move_parsed = true;
                    result.append(&mut pawn_capture);
                }
                pawn_branch_done = true;
            }

            if !pawn_move_parsed {
                could_be_something_else = false;

                if let Ok(piece) = parse_piece(&input, &mut i) {
                    result.push(piece);
                }

                if let Ok(captures) = parse_captures(&input, &mut i) {
                    result.push(captures);
                }

                let dest_tile_res = parse_destination(&input, &mut i);
                if dest_tile_res.is_err() {
                    continue;
                }

                result.push(dest_tile_res.unwrap());
            }

            if let Ok(promotion) = parse_pawn_promotion(&input, &mut i) {
                result.push(promotion);
            }

            if let Ok(ep) = parse_en_passant(&input, &mut i) {
                result.push(ep);
            }
        }

        if let Ok(check_or_mate) = parse_check_or_mate(&input, &mut i) {
            result.push(check_or_mate);
        }

        results.push(Move::construct(result));
    }

    Ok(results)
}

fn is_file(c: char) -> bool {
    ('a'..='h').contains(&c)
}

fn is_rank(c: char) -> bool {
    ('1'..='8').contains(&c)
}

fn safe_index(vec: &[char], i: usize) -> Result<char, ()> {
    if i >= vec.len() {
        Err(())
    } else {
        Ok(vec[i])
    }
}

fn parse_destination(input: &[char], final_i: &mut usize) -> Result<Node, ()> {
    let i = *final_i;

    if is_file(input[i]) {
        let char_ahead = safe_index(input, i + 1)?;
        if is_rank(char_ahead) {
            *final_i = i + 2;
            return Ok(Node::Destination(input[i], char_ahead));
        }
    }

    Err(())
}

#[cfg(test)]
#[path = "./tests/move_parser_tests.rs"]
mod move_parser_tests;
