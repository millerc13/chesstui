use cozy_chess::{Color as ChessColor, Piece};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};

// ─── Piece profiles: (radius, y_height) defining silhouette revolved around Y axis ───
// Same profiles as ascii3d.rs rotating pieces, tuned for recognizable shapes.

const KING_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    (2.2, 0.8),
    (1.8, 1.1),
    (1.5, 1.4),
    (1.3, 1.7),
    (1.2, 2.0),
    (1.15, 2.5),
    (1.15, 3.0),
    (1.2, 3.2),
    (1.5, 3.4),
    (2.0, 3.6),
    (2.5, 3.8),
    (2.8, 4.0),
    (2.8, 4.15),
    (2.7, 4.3),
    (2.4, 4.5),
    (2.0, 4.7),
    (1.5, 4.9),
    (1.0, 5.1),
    (0.8, 5.2),
    (1.6, 5.3),
    (1.6, 5.4),
    (0.8, 5.5),
    (0.5, 5.6),
    (0.5, 6.0),
    (0.3, 6.1),
    (0.0, 6.1),
];

const QUEEN_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    (2.2, 0.8),
    (1.8, 1.1),
    (1.5, 1.4),
    (1.3, 1.7),
    (1.2, 2.0),
    (1.15, 2.5),
    (1.15, 3.0),
    (1.2, 3.2),
    (1.6, 3.4),
    (2.2, 3.7),
    (2.8, 4.0),
    (3.0, 4.2),
    (3.0, 4.3),
    (2.6, 4.6),
    (2.0, 4.9),
    (1.4, 5.2),
    (0.8, 5.5),
    (1.0, 5.6),
    (1.4, 5.8),
    (1.5, 6.0),
    (1.5, 6.2),
    (1.4, 6.4),
    (1.1, 6.6),
    (0.7, 6.7),
    (0.0, 6.8),
];

const ROOK_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    (2.2, 0.8),
    (1.8, 1.1),
    (1.6, 1.5),
    (1.6, 2.0),
    (1.6, 2.5),
    (1.6, 3.0),
    (1.6, 3.5),
    (1.8, 3.6),
    (2.4, 3.7),
    (2.4, 3.8),
    (2.4, 4.2),
    (1.7, 4.2),
    (1.7, 4.6),
    (2.4, 4.6),
    (2.4, 5.0),
    (1.8, 5.0),
    (0.0, 5.0),
];

const BISHOP_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (3.0, 0.0),
    (3.2, 0.05),
    (3.2, 0.3),
    (3.0, 0.45),
    (2.6, 0.55),
    (2.0, 0.8),
    (1.6, 1.1),
    (1.3, 1.5),
    (1.2, 1.8),
    (1.15, 2.2),
    (1.15, 2.8),
    (1.2, 3.0),
    (1.5, 3.2),
    (1.8, 3.5),
    (2.0, 3.7),
    (2.0, 3.9),
    (1.8, 4.2),
    (1.5, 4.5),
    (1.2, 4.8),
    (0.9, 5.1),
    (0.6, 5.4),
    (0.35, 5.7),
    (0.15, 6.0),
    (0.3, 6.1),
    (0.3, 6.2),
    (0.15, 6.3),
    (0.0, 6.3),
];

const KNIGHT_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (3.0, 0.0),
    (3.2, 0.05),
    (3.2, 0.3),
    (3.0, 0.45),
    (2.6, 0.55),
    (2.0, 0.8),
    (1.6, 1.1),
    (1.3, 1.4),
    (1.2, 1.7),
    (1.15, 2.0),
    (1.15, 2.5),
    (1.3, 2.8),
    (1.6, 3.1),
    (2.0, 3.4),
    (2.3, 3.7),
    (2.4, 4.0),
    (2.3, 4.3),
    (2.0, 4.6),
    (1.6, 4.9),
    (1.2, 5.2),
    (0.8, 5.4),
    (0.5, 5.6),
    (0.3, 5.8),
    (0.0, 5.9),
];

const PAWN_PROFILE: &[(f64, f64)] = &[
    (0.0, 0.0),
    (2.8, 0.0),
    (3.0, 0.05),
    (3.0, 0.3),
    (2.8, 0.45),
    (2.4, 0.55),
    (1.8, 0.8),
    (1.4, 1.1),
    (1.1, 1.4),
    (0.85, 1.7),
    (0.75, 2.0),
    (0.75, 2.2),
    (0.9, 2.4),
    (1.1, 2.6),
    (1.4, 2.8),
    (1.6, 3.0),
    (1.7, 3.2),
    (1.75, 3.4),
    (1.75, 3.6),
    (1.7, 3.8),
    (1.6, 4.0),
    (1.4, 4.2),
    (1.1, 4.4),
    (0.8, 4.5),
    (0.4, 4.6),
    (0.0, 4.65),
];

pub fn get_profile(piece: Piece) -> &'static [(f64, f64)] {
    match piece {
        Piece::King => KING_PROFILE,
        Piece::Queen => QUEEN_PROFILE,
        Piece::Rook => ROOK_PROFILE,
        Piece::Bishop => BISHOP_PROFILE,
        Piece::Knight => KNIGHT_PROFILE,
        Piece::Pawn => PAWN_PROFILE,
    }
}

// ─── Arc-length parameterized interpolation ───

fn arc_lengths(profile: &[(f64, f64)]) -> Vec<f64> {
    let mut lengths = Vec::with_capacity(profile.len());
    lengths.push(0.0);
    for i in 1..profile.len() {
        let dr = profile[i].0 - profile[i - 1].0;
        let dy = profile[i].1 - profile[i - 1].1;
        lengths.push(lengths[i - 1] + (dr * dr + dy * dy).sqrt());
    }
    lengths
}

fn interpolate_profile(profile: &[(f64, f64)], lengths: &[f64], t: f64) -> (f64, f64, usize) {
    let total = *lengths.last().unwrap();
    let target = (t * total).min(total - 1e-9);
    let mut seg = 0;
    for i in 1..lengths.len() {
        if lengths[i] >= target {
            seg = i - 1;
            break;
        }
    }
    let next = (seg + 1).min(profile.len() - 1);
    let seg_start = lengths[seg];
    let seg_end = lengths[next];
    let seg_len = seg_end - seg_start;
    let frac = if seg_len > 1e-9 {
        (target - seg_start) / seg_len
    } else {
        0.0
    };
    let r = profile[seg].0 + frac * (profile[next].0 - profile[seg].0);
    let y = profile[seg].1 + frac * (profile[next].1 - profile[seg].1);
    (r, y, seg)
}

// ─── Static front-facing 3D render ───

/// Render a piece as a static front-facing 3D view into a luminance buffer.
/// Returns (lum_buf, edge_buf) — luminance values and edge detection.
fn render_static(
    piece: Piece,
    width: usize,
    hires_height: usize,
) -> (Vec<Vec<f64>>, Vec<Vec<bool>>) {
    let w = width;
    let h = hires_height;
    let mut lum_buf = vec![vec![0.0f64; w]; h];
    let mut zbuffer = vec![vec![0.0f64; w]; h];
    let mut filled = vec![vec![false; w]; h];

    let profile = get_profile(piece);
    let lengths = arc_lengths(profile);

    let y_max = profile.iter().map(|p| p.1).fold(0.0f64, f64::max);
    let r_max = profile.iter().map(|p| p.0).fold(0.0f64, f64::max);
    let y_center = y_max / 2.0;

    // Fixed viewing angle: slight tilt to show some 3D depth
    let a: f64 = 0.35; // tilt angle (shows a bit of the top)
    let b: f64 = 0.15; // slight Y rotation for depth
    let (sin_a, cos_a) = a.sin_cos();
    let (sin_b, cos_b) = b.sin_cos();

    let k2 = 8.0;
    let scale_y = (h as f64 * 0.85 * k2) / y_max;
    let scale_x = (w as f64 * 0.85 * k2) / (r_max * 2.0);
    let scale = scale_y.min(scale_x);

    // Light from upper-right-front
    let (lx, ly, lz) = {
        let (a, b, c): (f64, f64, f64) = (0.4, 0.6, -0.7);
        let len = (a * a + b * b + c * c).sqrt();
        (a / len, b / len, c / len)
    };

    let theta_steps = 150;
    let phi_steps = 200;

    for ti in 0..theta_steps {
        let t = ti as f64 / theta_steps as f64;
        let (r, y_local, seg) = interpolate_profile(profile, &lengths, t);
        if r < 1e-6 {
            continue;
        }

        let next = (seg + 1).min(profile.len() - 1);
        let dr = profile[next].0 - profile[seg].0;
        let dy = profile[next].1 - profile[seg].1;
        let tang_len = (dr * dr + dy * dy).sqrt();
        let (nr_p, ny_p) = if tang_len > 1e-9 {
            (dy / tang_len, -dr / tang_len)
        } else {
            (1.0, 0.0)
        };

        let y = y_local - y_center;

        for pi in 0..phi_steps {
            let phi = std::f64::consts::TAU * pi as f64 / phi_steps as f64;
            let (sin_phi, cos_phi) = phi.sin_cos();

            let x0 = r * cos_phi;
            let z0 = r * sin_phi;
            let nx0 = nr_p * cos_phi;
            let nz0 = nr_p * sin_phi;

            // Rotate X (tilt)
            let x1 = x0;
            let y1 = y * cos_a - z0 * sin_a;
            let z1 = y * sin_a + z0 * cos_a;
            let nx1 = nx0;
            let ny1 = ny_p * cos_a - nz0 * sin_a;
            let nz1 = ny_p * sin_a + nz0 * cos_a;

            // Rotate Y (slight turn)
            let x2 = x1 * cos_b + z1 * sin_b;
            let y2 = y1;
            let z2 = -x1 * sin_b + z1 * cos_b;
            let nx2 = nx1 * cos_b + nz1 * sin_b;
            let ny2 = ny1;
            let nz2 = -nx1 * sin_b + nz1 * cos_b;

            let denom = z2 + k2;
            if denom < 0.5 {
                continue;
            }
            let ooz = 1.0 / denom;

            let xp = (w as f64 / 2.0 + scale * ooz * x2) as i32;
            let yp = (h as f64 / 2.0 - scale * ooz * y2) as i32;

            if xp < 0 || xp >= w as i32 || yp < 0 || yp >= h as i32 {
                continue;
            }
            let (xi, yi) = (xp as usize, yp as usize);

            if ooz > zbuffer[yi][xi] {
                zbuffer[yi][xi] = ooz;
                filled[yi][xi] = true;

                let n_dot_l = nx2 * lx + ny2 * ly + nz2 * lz;
                if n_dot_l > 0.0 {
                    let ambient = 0.15;
                    let diffuse = n_dot_l * 0.6;
                    let rz: f64 = 2.0 * n_dot_l * nz2 - lz;
                    let r_dot_v = rz.abs(); // view along -Z
                    let specular = r_dot_v.powi(12) * 0.35;
                    lum_buf[yi][xi] = (ambient + diffuse + specular).min(1.0);
                } else {
                    lum_buf[yi][xi] = 0.08;
                }
            }
        }
    }

    // Edge detection: find pixels on the silhouette boundary
    let mut edge_buf = vec![vec![false; w]; h];
    for y in 0..h {
        for x in 0..w {
            if !filled[y][x] {
                continue;
            }
            // Check if any neighbor is empty → this is an edge pixel
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || nx >= w as i32 || ny < 0 || ny >= h as i32 {
                        edge_buf[y][x] = true;
                        break;
                    }
                    if !filled[ny as usize][nx as usize] {
                        edge_buf[y][x] = true;
                        break;
                    }
                }
                if edge_buf[y][x] {
                    break;
                }
            }
        }
    }

    (lum_buf, edge_buf)
}

// ─── Color helpers ───

fn lerp_color_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8,
        (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8,
        (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8,
    )
}

/// Map luminance to a color: shadow → piece_color → highlight
fn lum_to_piece_color(lum: f64, base: (u8, u8, u8)) -> Color {
    let shadow = (
        (base.0 as f64 * 0.15) as u8,
        (base.1 as f64 * 0.15) as u8,
        (base.2 as f64 * 0.15) as u8,
    );
    let highlight = (
        (base.0 as f64 * 0.35 + 255.0 * 0.65) as u8,
        (base.1 as f64 * 0.35 + 255.0 * 0.65) as u8,
        (base.2 as f64 * 0.35 + 255.0 * 0.65) as u8,
    );
    if lum < 0.4 {
        lerp_color_rgb(shadow, base, lum / 0.4)
    } else {
        lerp_color_rgb(base, highlight, (lum - 0.4) / 0.6)
    }
}

/// Extract RGB from a ratatui Color.
pub fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(idx) => match idx {
            232..=255 => {
                let v = (idx - 232) * 10 + 8;
                (v, v, v)
            }
            16..=231 => {
                let i = idx - 16;
                let b = (i % 6) * 51;
                let g = ((i / 6) % 6) * 51;
                let r = (i / 36) * 51;
                (r, g, b)
            }
            _ => (180, 180, 180),
        },
        Color::White => (240, 240, 240),
        Color::Black => (30, 30, 30),
        _ => (160, 160, 160),
    }
}

// ─── Public API ───

/// Draw a 3D-shaded piece into a board square using half-block characters.
/// Uses surface-of-revolution profiles with Phong lighting and edge outlines.
pub fn draw_piece(
    buf: &mut Buffer,
    area: Rect,
    x: u16,
    y: u16,
    cw: u16,
    ch: u16,
    piece: Piece,
    piece_color: Color,
    sq_bg: Color,
) {
    let w = cw as usize;
    let hires_h = (ch as usize) * 2;
    if w < 2 || hires_h < 2 {
        return;
    }

    let (lum, edges) = render_static(piece, w, hires_h);
    let base_rgb = color_to_rgb(piece_color);

    // Dark outline color (near-black, slightly tinted)
    let outline_color = Color::Rgb(
        (base_rgb.0 as f64 * 0.1) as u8,
        (base_rgb.1 as f64 * 0.1) as u8,
        (base_rgb.2 as f64 * 0.1) as u8,
    );

    for row in 0..ch {
        for col in 0..cw {
            let px_x = x + col;
            let px_y = y + row;
            if px_x >= area.x + area.width
                || px_y >= area.y + area.height
                || px_x < area.x
                || px_y < area.y
            {
                continue;
            }

            let top_r = (row * 2) as usize;
            let bot_r = (row * 2 + 1) as usize;
            let c = col as usize;

            let top_lum = lum[top_r][c];
            let bot_lum = if bot_r < hires_h { lum[bot_r][c] } else { 0.0 };
            let top_on = top_lum > 0.01;
            let bot_on = bot_lum > 0.01;

            let top_edge = top_on && edges[top_r][c];
            let bot_edge = bot_on && bot_r < hires_h && edges[bot_r][c];

            let top_col = if top_edge {
                outline_color
            } else if top_on {
                lum_to_piece_color(top_lum, base_rgb)
            } else {
                sq_bg
            };
            let bot_col = if bot_edge {
                outline_color
            } else if bot_on {
                lum_to_piece_color(bot_lum, base_rgb)
            } else {
                sq_bg
            };

            let (ch_char, style) = match (top_on, bot_on) {
                (true, true) => ('\u{2580}', Style::default().fg(top_col).bg(bot_col)),
                (true, false) => ('\u{2580}', Style::default().fg(top_col).bg(sq_bg)),
                (false, true) => ('\u{2584}', Style::default().fg(bot_col).bg(sq_bg)),
                (false, false) => (' ', Style::default().bg(sq_bg)),
            };

            if let Some(cell) = buf.cell_mut(Position::new(px_x, px_y)) {
                cell.set_char(ch_char);
                cell.set_style(style);
            }
        }
    }
}

/// Unicode chess symbol for a piece (used in captured lists, status, etc.)
pub fn piece_char(piece: Piece, color: ChessColor) -> char {
    match (color, piece) {
        (ChessColor::White, Piece::King) => '\u{2654}',
        (ChessColor::White, Piece::Queen) => '\u{2655}',
        (ChessColor::White, Piece::Rook) => '\u{2656}',
        (ChessColor::White, Piece::Bishop) => '\u{2657}',
        (ChessColor::White, Piece::Knight) => '\u{2658}',
        (ChessColor::White, Piece::Pawn) => '\u{2659}',
        (ChessColor::Black, Piece::King) => '\u{265a}',
        (ChessColor::Black, Piece::Queen) => '\u{265b}',
        (ChessColor::Black, Piece::Rook) => '\u{265c}',
        (ChessColor::Black, Piece::Bishop) => '\u{265d}',
        (ChessColor::Black, Piece::Knight) => '\u{265e}',
        (ChessColor::Black, Piece::Pawn) => '\u{265f}',
    }
}
