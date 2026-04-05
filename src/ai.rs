use cozy_chess::{Board, Color, Move, Piece};

const DEPTH: u8 = 3;

pub fn choose_move(board: &Board) -> Option<Move> {
    let mut legal = Vec::new();
    board.generate_moves(|mvs| { legal.extend(mvs); false });
    if legal.is_empty() { return None; }

    let maximizing = board.side_to_move() == Color::White;
    let mut best_move = legal[0];
    let mut best_score = if maximizing { i32::MIN } else { i32::MAX };

    for mv in &legal {
        let mut next = board.clone();
        next.play_unchecked(*mv);
        let score = minimax(&next, DEPTH - 1, i32::MIN, i32::MAX, !maximizing);
        if maximizing && score > best_score || !maximizing && score < best_score {
            best_score = score;
            best_move = *mv;
        }
    }

    Some(best_move)
}

fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut legal = Vec::new();
    board.generate_moves(|mvs| { legal.extend(mvs); false });

    if legal.is_empty() {
        return if !board.checkers().is_empty() {
            // Checkmate
            if maximizing { -100_000 } else { 100_000 }
        } else {
            0 // Stalemate
        };
    }

    if maximizing {
        let mut best = i32::MIN;
        for mv in &legal {
            let mut next = board.clone();
            next.play_unchecked(*mv);
            let score = minimax(&next, depth - 1, alpha, beta, false);
            best = best.max(score);
            alpha = alpha.max(score);
            if beta <= alpha { break; }
        }
        best
    } else {
        let mut best = i32::MAX;
        for mv in &legal {
            let mut next = board.clone();
            next.play_unchecked(*mv);
            let score = minimax(&next, depth - 1, alpha, beta, true);
            best = best.min(score);
            beta = beta.min(score);
            if beta <= alpha { break; }
        }
        best
    }
}

fn evaluate(board: &Board) -> i32 {
    let mut score: i32 = 0;

    for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
        let val = piece_value(piece);
        let white = (board.pieces(piece) & board.colors(Color::White)).len() as i32;
        let black = (board.pieces(piece) & board.colors(Color::Black)).len() as i32;
        score += val * (white - black);
    }

    // Bonus for center control (pawns/knights on d4/d5/e4/e5)
    let center = cozy_chess::BitBoard::EMPTY
        | cozy_chess::Square::D4.bitboard()
        | cozy_chess::Square::D5.bitboard()
        | cozy_chess::Square::E4.bitboard()
        | cozy_chess::Square::E5.bitboard();

    let white_center = (board.colors(Color::White) & center).len() as i32;
    let black_center = (board.colors(Color::Black) & center).len() as i32;
    score += 10 * (white_center - black_center);

    score
}

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}
