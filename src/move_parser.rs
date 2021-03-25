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
pub enum Node {
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

fn is_there_tile_ahead(input: &Vec<char>, index: usize) -> bool {
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
    input: &Vec<char>,
    i: &mut usize,
    could_be_something_else: &mut bool,
) -> Result<Vec<Node>, ()> {
    let mut result: Vec<Node> = vec![];
    let mut j = *i;

    if is_file(input[j]) {
        if is_chess_piece(input[j]) {
            *could_be_something_else = true;
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
fn parse_piece(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
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

fn parse_captures(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
    if input[*i] == 'x' {
        *i += 1;
        Ok(Node::Captures)
    } else {
        Err(())
    }
}

fn parse_en_passant(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
    if safe_index(input, *i + 3).is_ok() {
        if input[*i..=*i + 3].iter().collect::<String>() == "e.p." {
            *i += 4;
            return Ok(Node::EnPassant);
        }
    }

    Err(())
}

fn parse_check_or_mate(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
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
    if c == 'r'
        || c == 'n'
        || c == 'b'
        || c == 'k'
        || c == 'q'
        || c == 'R'
        || c == 'N'
        || c == 'B'
        || c == 'K'
        || c == 'Q'
    {
        true
    } else {
        false
    }
}

fn parse_pawn_promotion(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
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

fn parse_castle(input: &Vec<char>, i: &mut usize) -> Result<Node, ()> {
    let input_string: String = input.iter().collect();
    if input_string.starts_with("o-o") {
        if input_string.starts_with("o-o-o") {
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
//recommended pre-processing of input: strip whitespace and lowercase everything
pub fn parse(input: Vec<char>) -> Result<Vec<Vec<Node>>, ()> {
    let mut results: Vec<Vec<Node>> = vec![];

    let mut could_be_something_else = true;
    let mut pawn_branch_done = false;

    while could_be_something_else {
        let mut i = 0 as usize;
        let mut result: Vec<Node> = vec![];
        let mut pawn_move_parsed = false;

        if let Ok(castle) = parse_castle(&input, &mut i) {
            result.push(castle);
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

                let dest_tile = parse_destination(&input, &mut i)?;
                result.push(dest_tile);
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

        results.push(result);
    }

    Ok(results)
}

fn is_file(c: char) -> bool {
    c >= 'a' && c <= 'h'
}

fn is_rank(c: char) -> bool {
    c >= '1' && c <= '8'
}

fn safe_index(vec: &Vec<char>, i: usize) -> Result<char, ()> {
    if i >= vec.len() {
        Err(())
    } else {
        Ok(vec[i])
    }
}

fn parse_destination(input: &Vec<char>, final_i: &mut usize) -> Result<Node, ()> {
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
mod tests {

    use super::{parse, Node};

    fn test_moves(input: &str) -> Result<Vec<Node>, ()> {
        let mut input: String = input.to_string();

        input.retain(|c| !c.is_whitespace());
        input = input.to_lowercase();

        return parse(input.chars().collect());
    }

    fn assert_move_vec_eq(ms: &str, mv: Vec<Node>) {
        assert_eq!(test_moves(ms), Ok(mv));
    }

    #[test]
    fn parsing_moves() {
        assert_move_vec_eq(
            "Bbxb4 e.p.#",
            vec![
                Node::Piece('b', 'b', '-'),
                Node::Captures,
                Node::Destination('b', '4'),
                Node::EnPassant,
                Node::Checkmate,
            ],
        );

        // castling
        assert_move_vec_eq("O-O-O+", vec![Node::CastleLong, Node::Check]);

        // pawn move
        assert_move_vec_eq("e4", vec![Node::Destination('e', '4')]);

        //pawn captures
        assert_move_vec_eq(
            "exd5",
            vec![
                Node::Piece('-', 'e', '-'),
                Node::Captures,
                Node::Destination('d', '5'),
            ],
        );

        assert_move_vec_eq(
            "ed#",
            vec![
                Node::Piece('-', 'e', '-'),
                Node::Destination('d', '-'),
                Node::Checkmate,
            ],
        );

        //promoting
        assert_move_vec_eq(
            "e8=Q",
            vec![Node::Destination('e', '8'), Node::PawnPromotion('q')],
        );
        assert_move_vec_eq(
            "edQ#",
            vec![
                Node::Piece('-', 'e', '-'),
                Node::Destination('d', '-'),
                Node::PawnPromotion('q'),
                Node::Checkmate,
            ],
        );
    }
}
