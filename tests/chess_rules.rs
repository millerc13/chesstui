use chesstui::game::state::{GameResult, GameState, GameStatus};

#[test]
fn new_game_starts_with_white_to_move() {
    let game = GameState::new();
    assert_eq!(game.side_to_move(), cozy_chess::Color::White);
    assert_eq!(game.fullmove_number(), 1);
    assert!(game.move_history().is_empty());
    assert_eq!(game.status(), GameStatus::InProgress);
}

#[test]
fn making_a_move_switches_turn() {
    let mut game = GameState::new();
    let mv = find_move(&game, "e2", "e4", None);
    assert!(game.try_make_move(mv).is_ok());
    assert_eq!(game.side_to_move(), cozy_chess::Color::Black);
}

#[test]
fn illegal_move_is_rejected() {
    let mut game = GameState::new();
    let mv = cozy_chess::Move {
        from: cozy_chess::Square::E7,
        to: cozy_chess::Square::E5,
        promotion: None,
    };
    assert!(game.try_make_move(mv).is_err());
}

#[test]
fn captured_pieces_are_tracked() {
    let mut game = GameState::new();
    make_uci(&mut game, "e2e4");
    make_uci(&mut game, "e7e5");
    make_uci(&mut game, "f1c4");
    make_uci(&mut game, "b8c6");
    make_uci(&mut game, "d1h5");
    make_uci(&mut game, "g8f6");
    make_uci(&mut game, "h5f7"); // Qxf7#
    assert_eq!(game.captured_by(cozy_chess::Color::White).len(), 1);
    assert_eq!(
        game.status(),
        GameStatus::Finished(GameResult::Checkmate(cozy_chess::Color::White))
    );
}

#[test]
fn undo_restores_previous_position() {
    let mut game = GameState::new();
    make_uci(&mut game, "e2e4");
    make_uci(&mut game, "e7e5");
    assert_eq!(game.move_history().len(), 2);
    game.undo();
    assert_eq!(game.move_history().len(), 1);
    assert_eq!(game.side_to_move(), cozy_chess::Color::Black);
    game.undo();
    assert_eq!(game.move_history().len(), 0);
    assert_eq!(game.side_to_move(), cozy_chess::Color::White);
}

fn find_move(
    _game: &GameState,
    from: &str,
    to: &str,
    promo: Option<cozy_chess::Piece>,
) -> cozy_chess::Move {
    let from_sq = from.parse::<cozy_chess::Square>().unwrap();
    let to_sq = to.parse::<cozy_chess::Square>().unwrap();
    cozy_chess::Move {
        from: from_sq,
        to: to_sq,
        promotion: promo,
    }
}

fn make_uci(game: &mut GameState, uci: &str) {
    let bytes = uci.as_bytes();
    let from = format!("{}{}", bytes[0] as char, bytes[1] as char);
    let to = format!("{}{}", bytes[2] as char, bytes[3] as char);
    let promo = if bytes.len() == 5 {
        match bytes[4] {
            b'q' => Some(cozy_chess::Piece::Queen),
            b'r' => Some(cozy_chess::Piece::Rook),
            b'b' => Some(cozy_chess::Piece::Bishop),
            b'n' => Some(cozy_chess::Piece::Knight),
            _ => None,
        }
    } else {
        None
    };
    let mv = find_move(game, &from, &to, promo);
    game.try_make_move(mv)
        .expect(&format!("Move {} should be legal", uci));
}
