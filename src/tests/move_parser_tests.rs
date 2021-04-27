use crate::move_parser::*;

mod move_parser_tests {

    use super::{parse, Move, MovePrimary};

    fn test_moves(input: &str) -> Result<Vec<Move>, ()> {
        let mut input: String = input.to_string();
        input.retain(|c| !c.is_whitespace());
        return parse(input.chars().collect());
    }

    fn assert_move_vec_eq(ms: &str, mv: Move) {
        assert_eq!(test_moves(ms), Ok(vec![mv]));
    }

    fn assert_move_mul_vec_eq(ms: &str, mv: Vec<Move>) {
        assert_eq!(test_moves(ms), Ok(mv));
    }

    #[test]
    fn parsing_moves() {
        assert_move_vec_eq(
            "Bbxb4 e.p.#",
            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('B', 'b', '-'), 
                    destination: ('b', '4'),
                    promotion: '-'
                },
                check: false,
                checkmate: true,
                en_passant: true,
                captures: true,
            }
            // vec![
            //     Node::Piece('B', 'b', '-'),
            //     Node::Captures,
            //     Node::Destination('b', '4'),
            //     Node::EnPassant,
            //     Node::Checkmate,
            // ],
        );

        // castling
        assert_move_vec_eq("O-O-O+", 
            Move{
                primary: MovePrimary::CastleLong,
                check: true,
                checkmate: false,
                en_passant: false,
                captures: false,
            }
            // vec![Node::CastleLong, Node::Check]
            );

        // pawn move
        assert_move_vec_eq("e4", 

            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', '-', '-'), 
                    destination: ('e', '4'),
                    promotion: '-'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: false,
            }
            // vec![Node::Destination('e', '4')]
            );

        //pawn captures
        assert_move_vec_eq(
            "exd5",
            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', 'e', '-'), 
                    destination: ('d', '5'),
                    promotion: '-'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: true,
            }
            // vec![
            //     Node::Piece('-', 'e', '-'),
            //     Node::Captures,
            //     Node::Destination('d', '5'),
            // ],
        );

        assert_move_vec_eq(
            "ed#",
            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', 'e', '-'), 
                    destination: ('d', '-'),
                    promotion: '-'
                },
                check: false,
                checkmate: true,
                en_passant: false,
                captures: false,
            }
            // vec![
            //     Node::Piece('-', 'e', '-'),
            //     Node::Destination('d', '-'),
            //     Node::Checkmate,
            // ],
        );

        //promoting
        assert_move_vec_eq(
            "e8=Q",

            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', '-', '-'), 
                    destination: ('e', '8'),
                    promotion: 'Q'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: false,
            }

            // vec![Node::Destination('e', '8'), Node::PawnPromotion('Q')],
        );

        assert_move_vec_eq(
            "edQ#",

            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', 'e', '-'), 
                    destination: ('d', '-'),
                    promotion: 'Q'
                },
                check: false,
                checkmate: true,
                en_passant: false,
                captures: false,
            }

            // vec![
            //     Node::Piece('-', 'e', '-'),
            //     Node::Destination('d', '-'),
            //     Node::PawnPromotion('Q'),
            //     Node::Checkmate,
            // ],
        );

        //multiple interpretations
        assert_move_mul_vec_eq(
            "bc4",
            vec![
            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('-', 'b', '-'), 
                    destination: ('c', '4'),
                    promotion: '-'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: false,
            },
            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('b', '-', '-'), 
                    destination: ('c', '4'),
                    promotion: '-'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: false,
            }

            ]
            // vec![
            //     vec![Node::Piece('-', 'b', '-'), Node::Destination('c', '4')],
            //     vec![Node::Piece('b', '-', '-'), Node::Destination('c', '4')],
            // ],
        );

        //only one interpretation because B is capitalized (bishop move)
        assert_move_vec_eq(
            "Bc4",

            Move{
                primary: MovePrimary::PieceMove{
                    piece: ('B', '-', '-'), 
                    destination: ('c', '4'),
                    promotion: '-'
                },
                check: false,
                checkmate: false,
                en_passant: false,
                captures: false,
            }

            // vec![Node::Piece('B', '-', '-'), Node::Destination('c', '4')],
        );
    }
}
