use crate::chess::*;

mod fen_tests {

    use super::*;

    use std::array::IntoIter;
    use std::iter::FromIterator;

    use ChessPiece::*;
    use ChessTeam::*;
    use Tile::*;

    fn assert_game(
        game: &mut GameState,
        piece_locations: HashMap<Tile, TeamedChessPiece>,
        whose_turn: ChessTeam,
        castling_rights: (bool, bool, bool, bool),
        ep_square: Option<Tile>,
        fifty_move_counter: u32,
        full_move_counter: u32,
    ) {
        //checking if hashmaps are equal

        let g_piece_locations = game.get_board().piece_locations.clone();

        assert!(
            g_piece_locations.len() == piece_locations.len()
                && g_piece_locations.keys().all(|k| {
                    let v1 = g_piece_locations.get(k).copied().unwrap();
                    let v2 = piece_locations.get(k).copied().unwrap();
                    v1 == v2
                })
        );

        assert_eq!(game.whose_turn(), whose_turn);
        assert_eq!(game.get_board().castling_rights, castling_rights);
        assert_eq!(game.en_passant_square, ep_square);
        assert_eq!(game.fifty_move_counter, fifty_move_counter);
        assert_eq!(game.starting_move_count, full_move_counter);
    }

    #[test]
    fn correct_fens() {
        let chess_pieces = HashMap::<_, _>::from_iter(IntoIter::new([
            (F8, TeamedChessPiece(Black, King)),
            (F7, TeamedChessPiece(Black, Pawn)),
            (G6, TeamedChessPiece(Black, Pawn)),
            (C4, TeamedChessPiece(Black, Knight)),
            (H3, TeamedChessPiece(Black, Pawn)),
            (G2, TeamedChessPiece(Black, Rook)),
            (G4, TeamedChessPiece(White, Pawn)),
            (F3, TeamedChessPiece(White, Pawn)),
            (H2, TeamedChessPiece(White, Pawn)),
            (D3, TeamedChessPiece(White, Rook)),
            (G3, TeamedChessPiece(White, Knight)),
            (H1, TeamedChessPiece(White, King)),
        ]));

        // no ep square, no castling
        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w - - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (false, false, false, false),
            None,
            0,
            54,
        );

        // testing castling, no ep square

        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w KQ - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (true, true, false, false),
            None,
            0,
            54,
        );

        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w KQkq - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (true, true, true, true),
            None,
            0,
            54,
        );

        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w kq - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (false, false, true, true),
            None,
            0,
            54,
        );

        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w KQk - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (true, true, true, false),
            None,
            0,
            54,
        );

        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w Qq - 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (false, true, false, true),
            None,
            0,
            54,
        );

        // ep square and castling
        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w Qq e3 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (false, true, false, true),
            Some(E3),
            0,
            54,
        );

        // ep square and no castling
        let mut game =
            parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w - e3 0 54".to_string()).unwrap();
        assert_game(
            &mut game,
            chess_pieces.clone(),
            ChessTeam::White,
            (false, false, false, false),
            Some(E3),
            0,
            54,
        );
    }

    #[test]
    fn incorrect_fens() {
        // ep square and no castling
        assert!(parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rP/7K w - e3 0 ".to_string()).is_none());
        assert!(parse_fen("5k2/5p2/6p1/8/2n3P1/3R1Pp/6rP/7K w - e3 0 54".to_string()).is_none());
        assert!(parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/2rP/7K w - e3 0 54".to_string()).is_none());
        assert!(parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rPw/7K e3 0 54".to_string()).is_none());
        assert!(parse_fen("5k2/5p2/6p1/8/2n3P1/3R1PNp/6rPw/7K e".to_string()).is_none());
        assert!(parse_fen("w - e3 0 2".to_string()).is_none());
    }
}
