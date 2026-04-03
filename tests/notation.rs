use chesstui::game::notation::to_algebraic;
use cozy_chess::{Board, Move, Piece, Square};

// ─── Basic piece moves ────────────────────────────────────────────────────────

#[test]
fn pawn_move_e4() {
    let board = Board::default();
    let mv = Move { from: Square::E2, to: Square::E4, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "e4");
}

#[test]
fn knight_move_nf3() {
    let board = Board::default();
    let mv = Move { from: Square::G1, to: Square::F3, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "Nf3");
}

// ─── Pawn capture ─────────────────────────────────────────────────────────────

#[test]
fn pawn_capture_exd5() {
    let board = Board::from_fen(
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
        false,
    )
    .unwrap();
    let mv = Move { from: Square::E4, to: Square::D5, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "exd5");
}

// ─── Promotion ────────────────────────────────────────────────────────────────

#[test]
fn promotion_e8_queen() {
    let board = Board::from_fen("8/4P3/8/8/8/8/8/4K2k w - - 0 1", false).unwrap();
    let mv = Move { from: Square::E7, to: Square::E8, promotion: Some(Piece::Queen) };
    assert_eq!(to_algebraic(&board, &mv), "e8=Q");
}

#[test]
fn promotion_e8_knight() {
    let board = Board::from_fen("8/4P3/8/8/8/8/8/4K2k w - - 0 1", false).unwrap();
    let mv = Move { from: Square::E7, to: Square::E8, promotion: Some(Piece::Knight) };
    assert_eq!(to_algebraic(&board, &mv), "e8=N");
}

// ─── Castling ─────────────────────────────────────────────────────────────────
//
// cozy-chess represents castling as king -> rook's square:
//   White kingside:  E1 -> H1
//   White queenside: E1 -> A1
//   Black kingside:  E8 -> H8
//   Black queenside: E8 -> A8

#[test]
fn white_kingside_castling() {
    // Position where White can castle kingside (Nf3, Bc4 developed).
    let board = Board::from_fen(
        "r1bqk2r/ppppbppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        false,
    )
    .unwrap();
    // Kingside castling: king E1 -> H1 (rook square)
    let mv = Move { from: Square::E1, to: Square::H1, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "O-O");
}

#[test]
fn white_queenside_castling() {
    // Position where White can castle queenside.
    let board = Board::from_fen(
        "r3k2r/ppppbppp/2n2n2/4p3/2B1P3/3B1N2/PPPPQPPP/R3K2R w KQkq - 4 4",
        false,
    )
    .unwrap();
    // Queenside castling: king E1 -> A1 (rook square)
    let mv = Move { from: Square::E1, to: Square::A1, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "O-O-O");
}

#[test]
fn black_kingside_castling() {
    let board = Board::from_fen(
        "r3k2r/ppppbppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/R3K2R b KQkq - 4 4",
        false,
    )
    .unwrap();
    // Black kingside castling: king E8 -> H8
    let mv = Move { from: Square::E8, to: Square::H8, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "O-O");
}

#[test]
fn black_queenside_castling() {
    let board = Board::from_fen(
        "r3k2r/ppppbppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/R3K2R b KQkq - 4 4",
        false,
    )
    .unwrap();
    // Black queenside castling: king E8 -> A8
    let mv = Move { from: Square::E8, to: Square::A8, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "O-O-O");
}

// ─── Check / Checkmate suffixes ───────────────────────────────────────────────

#[test]
fn check_suffix() {
    // Fool's mate setup: after 1.f3 e5 2.g4, Qh4# is available.
    // Test a simple check: Scholar's mate threat setup.
    // Position: white queen can give check from h5.
    let board = Board::from_fen(
        "rnbqkbnr/pppp1ppp/8/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 2 3",
        false,
    )
    .unwrap();
    // Qh5 to f7 gives checkmate (Scholar's mate)
    let mv = Move { from: Square::H5, to: Square::F7, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "Qxf7#");
}

#[test]
fn move_gives_check_not_checkmate() {
    // After 1.e4 e5 2.Bc4, white bishop can check from f7
    let board = Board::from_fen(
        "rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3",
        false,
    )
    .unwrap();
    // Bxf7+ gives check but not checkmate
    let mv = Move { from: Square::C4, to: Square::F7, promotion: None };
    let result = to_algebraic(&board, &mv);
    assert_eq!(result, "Bxf7+");
}

// ─── Disambiguation ───────────────────────────────────────────────────────────

#[test]
fn rook_disambiguation_by_file() {
    // Two rooks on a1 and h1, king on e3 (not blocking), both rooks can reach d1.
    let board = Board::from_fen("4k3/8/8/8/8/4K3/8/R6R w - - 0 1", false).unwrap();
    // Ra1d1 — rook from a1 to d1; the h1 rook can also reach d1
    let mv = Move { from: Square::A1, to: Square::D1, promotion: None };
    let result = to_algebraic(&board, &mv);
    assert_eq!(result, "Rad1");
}

#[test]
fn rook_disambiguation_by_rank() {
    // Two rooks on a2 and a7, king on e1 and black king on e8 — both rooks reach a4.
    let board = Board::from_fen("4k3/R7/8/8/8/8/R7/4K3 w - - 0 1", false).unwrap();
    let mv = Move { from: Square::A2, to: Square::A4, promotion: None };
    let result = to_algebraic(&board, &mv);
    assert_eq!(result, "R2a4");
}

// ─── En-passant ───────────────────────────────────────────────────────────────

#[test]
fn en_passant_exd6() {
    // After 1.e4 d5 2.e5 d4? — actually use standard en-passant setup.
    // White pawn on e5, black just played d7-d5 (en-passant target on d6).
    let board = Board::from_fen(
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
        false,
    )
    .unwrap();
    let mv = Move { from: Square::E5, to: Square::D6, promotion: None };
    assert_eq!(to_algebraic(&board, &mv), "exd6");
}
