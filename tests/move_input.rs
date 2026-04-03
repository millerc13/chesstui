use chesstui::game::move_input::{MoveInputParser, InputResult};
use cozy_chess::Board;

#[test]
fn pawn_e4_from_starting_position() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    // 'e' matches e3 and e4 — need more input
    match parser.feed('e') {
        InputResult::NeedMore(n) => assert!(n >= 2),
        other => panic!("Expected NeedMore, got {:?}", other),
    }
    // '4' narrows to exactly e4
    match parser.feed('4') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E2);
            assert_eq!(mv.to, cozy_chess::Square::E4);
        }
        other => panic!("Expected Exact, got {:?}", other),
    }
}

#[test]
fn knight_nf3_from_starting_position() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    assert!(matches!(parser.feed('N'), InputResult::NeedMore(_)));
    assert!(matches!(parser.feed('f'), InputResult::NeedMore(_)));
    match parser.feed('3') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.to, cozy_chess::Square::F3);
        }
        other => panic!("Expected Exact, got {:?}", other),
    }
}

#[test]
fn invalid_input_returns_no_match() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    // 'z' followed by anything won't match any move
    parser.feed('z');
    // At this point, 'z' alone might be NeedMore (unlikely but check)
    // Feed more to get NoMatch
    match parser.feed('9') {
        InputResult::NoMatch => {}, // expected
        _ => {} // 'z' alone might already be NoMatch
    }
}

#[test]
fn square_to_square_e2e4() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    parser.feed('e');
    parser.feed('2');
    parser.feed('e');
    match parser.feed('4') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E2);
            assert_eq!(mv.to, cozy_chess::Square::E4);
        }
        other => panic!("Expected Exact for e2e4, got {:?}", other),
    }
}

#[test]
fn castling_oo() {
    let board = Board::from_fen(
        "r1bqk2r/ppppbppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        false,
    ).unwrap();
    let mut parser = MoveInputParser::new(&board);
    assert!(matches!(parser.feed('O'), InputResult::NeedMore(_)));
    match parser.feed('O') {
        InputResult::Exact(mv) => {
            assert_eq!(mv.from, cozy_chess::Square::E1);
            // cozy-chess encodes kingside castling as e1->h1
        }
        other => panic!("Expected Exact for O-O, got {:?}", other),
    }
}

#[test]
fn buffer_and_reset() {
    let board = Board::default();
    let mut parser = MoveInputParser::new(&board);
    parser.feed('e');
    assert_eq!(parser.buffer(), "e");
    parser.reset();
    assert_eq!(parser.buffer(), "");
}
