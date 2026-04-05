//! Renders the chess board as a pixel-perfect image via Kitty/Sixel protocol.
//! Falls back to halfblock rendering if the terminal doesn't support graphics.

use cozy_chess::{Color as ChessColor, Move, Piece, Square};
use image::{ImageBuffer, Rgb, RgbImage};

use super::pieces;
use super::sprites;
use crate::config::PieceStyle;
use crate::theme::Theme;

/// Generate a full board image at the given pixel dimensions.
/// Each square is sq_px × sq_px pixels. Total image is 8*sq_px × 8*sq_px.
pub fn render_board_image(
    board: &cozy_chess::Board,
    theme: &Theme,
    sq_px: u32,
    piece_cache: &PieceCache,
    flipped: bool,
    cursor: Option<(u8, u8)>,
    selected: Option<Square>,
    legal_moves: &[Move],
    last_move: Option<(Square, Square)>,
) -> RgbImage {
    let board_px = sq_px * 8;
    let mut img = ImageBuffer::new(board_px, board_px);

    let in_check = !board.checkers().is_empty();
    let king_square = if in_check {
        let side = board.side_to_move();
        let kings = board.pieces(Piece::King) & board.colors(side);
        let mut sq = None;
        for s in kings {
            sq = Some(s);
        }
        sq
    } else {
        None
    };

    for dr in 0..8u8 {
        for dc in 0..8u8 {
            let (file, rank) = if flipped { (7 - dc, dr) } else { (dc, 7 - dr) };
            let sq = Square::new(
                cozy_chess::File::index(file as usize),
                cozy_chess::Rank::index(rank as usize),
            );

            let is_light = (file + rank) % 2 != 0;

            // Determine square background color
            let sq_bg = highlight_color(
                file,
                rank,
                sq,
                theme,
                flipped,
                cursor,
                selected,
                legal_moves,
                last_move,
                in_check,
                king_square,
            )
            .unwrap_or_else(|| {
                if is_light {
                    color_to_rgb_tuple(theme.light_square)
                } else {
                    color_to_rgb_tuple(theme.dark_square)
                }
            });

            let px_x = dc as u32 * sq_px;
            let px_y = dr as u32 * sq_px;

            // Fill square background
            for y in 0..sq_px {
                for x in 0..sq_px {
                    img.put_pixel(px_x + x, px_y + y, Rgb([sq_bg.0, sq_bg.1, sq_bg.2]));
                }
            }

            // Render piece if present
            let piece_info = get_piece(board, sq);
            if let Some((piece, color)) = piece_info {
                let piece_img = piece_cache.get(piece, color);
                // Composite piece onto square (skip transparent/background pixels)
                stamp_piece(&mut img, piece_img, px_x, px_y, sq_px, sq_bg);
            } else if selected.is_some() && legal_moves.iter().any(|m| m.to == sq) {
                // Draw legal move dot
                draw_dot(
                    &mut img,
                    px_x,
                    px_y,
                    sq_px,
                    color_to_rgb_tuple(theme.accent),
                );
            }
        }
    }

    img
}

/// Render a preview image showing all 12 pieces (2 rows × 6 columns).
/// Top row: white pieces (K Q R B N P), bottom row: black pieces.
pub fn render_piece_preview(theme: &Theme, sq_px: u32, style: PieceStyle) -> RgbImage {
    let pieces = [
        Piece::King,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ];
    let cache = PieceCache::new(sq_px, theme, style);

    let width = sq_px * 6;
    let height = sq_px * 2;
    let mut img = ImageBuffer::new(width, height);

    for row in 0..2u32 {
        let color = if row == 0 {
            ChessColor::White
        } else {
            ChessColor::Black
        };
        for col in 0..6u32 {
            let is_light = (row + col) % 2 == 0;
            let bg = if is_light {
                color_to_rgb_tuple(theme.light_square)
            } else {
                color_to_rgb_tuple(theme.dark_square)
            };

            let px_x = col * sq_px;
            let px_y = row * sq_px;

            // Fill background
            for y in 0..sq_px {
                for x in 0..sq_px {
                    img.put_pixel(px_x + x, px_y + y, Rgb([bg.0, bg.1, bg.2]));
                }
            }

            // Stamp piece
            let piece_img = cache.get(pieces[col as usize], color);
            stamp_piece(&mut img, piece_img, px_x, px_y, sq_px, bg);
        }
    }

    img
}

/// Stamp a pre-rendered piece image onto the board, skipping background pixels.
fn stamp_piece(
    board_img: &mut RgbImage,
    piece_img: &RgbImage,
    px_x: u32,
    px_y: u32,
    sq_px: u32,
    _sq_bg: (u8, u8, u8),
) {
    let pw = piece_img.width();
    let ph = piece_img.height();
    // Center piece in square
    let off_x = (sq_px.saturating_sub(pw)) / 2;
    let off_y = (sq_px.saturating_sub(ph)) / 2;

    for y in 0..ph {
        for x in 0..pw {
            let px = piece_img.get_pixel(x, y);
            // Skip transparent marker (pixel [0,0,0] with alpha=0 equivalent — we use [1,1,1] as "empty")
            if px[0] <= 1 && px[1] <= 1 && px[2] <= 1 {
                continue; // transparent
            }
            let bx = px_x + off_x + x;
            let by = px_y + off_y + y;
            if bx < board_img.width() && by < board_img.height() {
                board_img.put_pixel(bx, by, *px);
            }
        }
    }
}

/// Draw a centered dot for legal moves.
fn draw_dot(img: &mut RgbImage, px_x: u32, px_y: u32, sq_px: u32, color: (u8, u8, u8)) {
    let cx = px_x + sq_px / 2;
    let cy = px_y + sq_px / 2;
    let r = (sq_px / 6).max(2);
    let r2 = (r * r) as i32;
    for dy in -(r as i32)..=(r as i32) {
        for dx in -(r as i32)..=(r as i32) {
            if dx * dx + dy * dy <= r2 {
                let x = (cx as i32 + dx) as u32;
                let y = (cy as i32 + dy) as u32;
                if x < img.width() && y < img.height() {
                    img.put_pixel(x, y, Rgb([color.0, color.1, color.2]));
                }
            }
        }
    }
}

// ─── Piece image cache ───

pub struct PieceCache {
    /// [piece_index * 2 + color_index] → rendered image
    images: Vec<RgbImage>,
    pub sq_px: u32,
    pub style: PieceStyle,
}

impl PieceCache {
    pub fn new(sq_px: u32, theme: &Theme, style: PieceStyle) -> Self {
        let pieces = [
            Piece::King,
            Piece::Queen,
            Piece::Rook,
            Piece::Bishop,
            Piece::Knight,
            Piece::Pawn,
        ];
        let colors = [ChessColor::White, ChessColor::Black];
        let mut images = Vec::with_capacity(12);

        for piece in &pieces {
            for color in &colors {
                let img = match sprites::get_sprite_data(style, *piece, *color) {
                    Some(png_bytes) => load_and_resize_sprite(png_bytes, sq_px, style),
                    None => {
                        let base_color = if *color == ChessColor::White {
                            theme.white_piece
                        } else {
                            theme.black_piece
                        };
                        render_piece_image(*piece, sq_px, base_color)
                    }
                };
                images.push(img);
            }
        }

        Self {
            images,
            sq_px,
            style,
        }
    }

    pub fn get(&self, piece: Piece, color: ChessColor) -> &RgbImage {
        let pi = match piece {
            Piece::King => 0,
            Piece::Queen => 1,
            Piece::Rook => 2,
            Piece::Bishop => 3,
            Piece::Knight => 4,
            Piece::Pawn => 5,
        };
        let ci = if color == ChessColor::White { 0 } else { 1 };
        &self.images[pi * 2 + ci]
    }
}

// ─── 3D piece renderer (surface of revolution + Phong) ───

fn render_piece_image(piece: Piece, sq_px: u32, base_color: ratatui::style::Color) -> RgbImage {
    let size = (sq_px as f64 * 0.85) as u32; // piece occupies 85% of square
    if size < 4 {
        return ImageBuffer::new(1, 1);
    }

    let profile = pieces::get_profile(piece);
    let base_rgb = pieces::color_to_rgb(base_color);

    let w = size as usize;
    let h = size as usize;

    let mut lum_buf = vec![vec![0.0f64; w]; h];
    let mut zbuffer = vec![vec![0.0f64; w]; h];
    let mut filled = vec![vec![false; w]; h];

    let lengths = arc_lengths(profile);
    let y_max = profile.iter().map(|p| p.1).fold(0.0f64, f64::max);
    let r_max = profile.iter().map(|p| p.0).fold(0.0f64, f64::max);
    let y_center = y_max / 2.0;

    let a: f64 = 0.3;
    let b: f64 = 0.12;
    let (sin_a, cos_a) = a.sin_cos();
    let (sin_b, cos_b) = b.sin_cos();

    let k2 = 8.0;
    let scale_y = (h as f64 * 0.9 * k2) / y_max;
    let scale_x = (w as f64 * 0.9 * k2) / (r_max * 2.0);
    let scale = scale_y.min(scale_x);

    let (lx, ly, lz) = {
        let (a, b, c): (f64, f64, f64) = (0.35, 0.65, -0.65);
        let len = (a * a + b * b + c * c).sqrt();
        (a / len, b / len, c / len)
    };

    let theta_steps = 250;
    let phi_steps = 300;

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

            let x1 = x0;
            let y1 = y * cos_a - z0 * sin_a;
            let z1 = y * sin_a + z0 * cos_a;
            let nx1 = nx0;
            let ny1 = ny_p * cos_a - nz0 * sin_a;
            let nz1 = ny_p * sin_a + nz0 * cos_a;

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
                    let specular = rz.abs().powi(16) * 0.3;
                    lum_buf[yi][xi] = (ambient + diffuse + specular).min(1.0);
                } else {
                    lum_buf[yi][xi] = 0.08;
                }
            }
        }
    }

    // Edge detection
    let mut edge = vec![vec![false; w]; h];
    for y in 0..h {
        for x in 0..w {
            if !filled[y][x] {
                continue;
            }
            'outer: for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0
                        || nx >= w as i32
                        || ny < 0
                        || ny >= h as i32
                        || !filled[ny as usize][nx as usize]
                    {
                        edge[y][x] = true;
                        break 'outer;
                    }
                }
            }
        }
    }

    // Convert to image
    let outline = (
        (base_rgb.0 as f64 * 0.08) as u8,
        (base_rgb.1 as f64 * 0.08) as u8,
        (base_rgb.2 as f64 * 0.08) as u8,
    );

    ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let (xi, yi) = (x as usize, y as usize);
        if !filled[yi][xi] {
            Rgb([0, 0, 0]) // transparent marker
        } else if edge[yi][xi] {
            Rgb([outline.0.max(2), outline.1.max(2), outline.2.max(2)])
        } else {
            let lum = lum_buf[yi][xi];
            let c = lum_to_color(lum, base_rgb);
            Rgb([c.0, c.1, c.2])
        }
    })
}

fn lum_to_color(lum: f64, base: (u8, u8, u8)) -> (u8, u8, u8) {
    let shadow = (
        (base.0 as f64 * 0.12) as u8,
        (base.1 as f64 * 0.12) as u8,
        (base.2 as f64 * 0.12) as u8,
    );
    let highlight = (
        (base.0 as f64 * 0.3 + 255.0 * 0.7) as u8,
        (base.1 as f64 * 0.3 + 255.0 * 0.7) as u8,
        (base.2 as f64 * 0.3 + 255.0 * 0.7) as u8,
    );
    if lum < 0.4 {
        let t = lum / 0.4;
        (
            (shadow.0 as f64 + (base.0 as f64 - shadow.0 as f64) * t) as u8,
            (shadow.1 as f64 + (base.1 as f64 - shadow.1 as f64) * t) as u8,
            (shadow.2 as f64 + (base.2 as f64 - shadow.2 as f64) * t) as u8,
        )
    } else {
        let t = (lum - 0.4) / 0.6;
        (
            (base.0 as f64 + (highlight.0 as f64 - base.0 as f64) * t) as u8,
            (base.1 as f64 + (highlight.1 as f64 - base.1 as f64) * t) as u8,
            (base.2 as f64 + (highlight.2 as f64 - base.2 as f64) * t) as u8,
        )
    }
}

// ─── Sprite loading ───

fn load_and_resize_sprite(png_bytes: &[u8], sq_px: u32, style: PieceStyle) -> RgbImage {
    let dyn_img = image::load_from_memory(png_bytes).expect("embedded PNG is valid");
    let rgba = dyn_img.to_rgba8();

    // Nearest-neighbor for pixel art (preserves crispness), Lanczos3 for hi-res
    let filter = match style {
        PieceStyle::Classic => image::imageops::FilterType::Lanczos3,
        _ => image::imageops::FilterType::Nearest,
    };

    let target = (sq_px as f64 * 0.85) as u32;
    let target = target.max(4);
    let resized = image::imageops::resize(&rgba, target, target, filter);

    // Convert RGBA to RGB with transparency convention (alpha < 128 → [0,0,0])
    let mut rgb = RgbImage::new(target, target);
    for (x, y, pixel) in resized.enumerate_pixels() {
        if pixel[3] < 128 {
            rgb.put_pixel(x, y, Rgb([0, 0, 0]));
        } else {
            rgb.put_pixel(
                x,
                y,
                Rgb([pixel[0].max(2), pixel[1].max(2), pixel[2].max(2)]),
            );
        }
    }
    rgb
}

// ─── Helpers ───

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

fn get_piece(board: &cozy_chess::Board, sq: Square) -> Option<(Piece, ChessColor)> {
    for piece in [
        Piece::King,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ] {
        if board.pieces(piece).has(sq) {
            let color = if board.colors(ChessColor::White).has(sq) {
                ChessColor::White
            } else {
                ChessColor::Black
            };
            return Some((piece, color));
        }
    }
    None
}

fn highlight_color(
    file: u8,
    rank: u8,
    sq: Square,
    theme: &Theme,
    flipped: bool,
    cursor: Option<(u8, u8)>,
    selected: Option<Square>,
    legal_moves: &[Move],
    last_move: Option<(Square, Square)>,
    in_check: bool,
    king_square: Option<Square>,
) -> Option<(u8, u8, u8)> {
    if in_check {
        if let Some(ksq) = king_square {
            if sq == ksq {
                return Some(color_to_rgb_tuple(theme.check_bg));
            }
        }
    }
    if let Some((cf, cr)) = cursor {
        let (f, r) = if flipped { (7 - cf, 7 - cr) } else { (cf, cr) };
        let csq = Square::new(
            cozy_chess::File::index(f as usize),
            cozy_chess::Rank::index(r as usize),
        );
        if sq == csq {
            return Some(color_to_rgb_tuple(theme.cursor_bg));
        }
    }
    if let Some(sel) = selected {
        if sq == sel {
            return Some(color_to_rgb_tuple(theme.selected_bg));
        }
    }
    if selected.is_some() && legal_moves.iter().any(|m| m.to == sq) {
        return Some(color_to_rgb_tuple(theme.legal_move_bg));
    }
    if let Some((from, to)) = last_move {
        if sq == from || sq == to {
            let is_light = (file + rank) % 2 != 0;
            return Some(color_to_rgb_tuple(if is_light {
                theme.last_move_light
            } else {
                theme.last_move_dark
            }));
        }
    }
    None
}

fn color_to_rgb_tuple(c: ratatui::style::Color) -> (u8, u8, u8) {
    pieces::color_to_rgb(c)
}
