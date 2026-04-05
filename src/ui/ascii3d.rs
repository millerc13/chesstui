use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy)]
pub enum Piece3D {
    King,
    Queen,
    Rook,
    Bishop,
    Pawn,
}

// Profiles: (radius, y_height) defining silhouette revolved around Y axis.
// More exaggerated proportions for clear silhouettes at terminal resolution.

const KING_PROFILE: &[(f64, f64)] = &[
    // Wide base platform
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    // Taper to stem
    (2.2, 0.8),
    (1.8, 1.1),
    (1.5, 1.4),
    (1.3, 1.7),
    // Narrow stem
    (1.2, 2.0),
    (1.15, 2.5),
    (1.15, 3.0),
    (1.2, 3.2),
    // Crown flare — wide dramatic crown
    (1.5, 3.4),
    (2.0, 3.6),
    (2.5, 3.8),
    (2.8, 4.0),
    (2.8, 4.15),
    // Crown rim
    (2.7, 4.3),
    (2.4, 4.5),
    (2.0, 4.7),
    (1.5, 4.9),
    // Neck before cross
    (1.0, 5.1),
    (0.8, 5.2),
    // Cross arms
    (1.6, 5.3),
    (1.6, 5.4),
    (0.8, 5.5),
    // Cross vertical
    (0.5, 5.6),
    (0.5, 6.0),
    (0.3, 6.1),
    (0.0, 6.1),
];

const QUEEN_PROFILE: &[(f64, f64)] = &[
    // Wide base
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    // Taper
    (2.2, 0.8),
    (1.8, 1.1),
    (1.5, 1.4),
    (1.3, 1.7),
    // Stem
    (1.2, 2.0),
    (1.15, 2.5),
    (1.15, 3.0),
    (1.2, 3.2),
    // Crown — tall dramatic flare
    (1.6, 3.4),
    (2.2, 3.7),
    (2.8, 4.0),
    (3.0, 4.2),
    (3.0, 4.3),
    // Crown peak taper
    (2.6, 4.6),
    (2.0, 4.9),
    (1.4, 5.2),
    (0.8, 5.5),
    // Ball finial on top
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
    // Wide base
    (0.0, 0.0),
    (3.2, 0.0),
    (3.4, 0.05),
    (3.4, 0.3),
    (3.2, 0.45),
    (2.8, 0.55),
    // Taper
    (2.2, 0.8),
    (1.8, 1.1),
    // Thick stem (rook is stocky)
    (1.6, 1.5),
    (1.6, 2.0),
    (1.6, 2.5),
    (1.6, 3.0),
    (1.6, 3.5),
    // Turret flare
    (1.8, 3.6),
    (2.4, 3.7),
    (2.4, 3.8),
    // Turret wall
    (2.4, 4.2),
    // Battlement notch
    (1.7, 4.2),
    (1.7, 4.6),
    // Battlement merlon
    (2.4, 4.6),
    (2.4, 5.0),
    // Flat top
    (1.8, 5.0),
    (0.0, 5.0),
];

const BISHOP_PROFILE: &[(f64, f64)] = &[
    // Base
    (0.0, 0.0),
    (3.0, 0.0),
    (3.2, 0.05),
    (3.2, 0.3),
    (3.0, 0.45),
    (2.6, 0.55),
    // Taper
    (2.0, 0.8),
    (1.6, 1.1),
    (1.3, 1.5),
    // Stem
    (1.2, 1.8),
    (1.15, 2.2),
    (1.15, 2.8),
    (1.2, 3.0),
    // Mitre bulge
    (1.5, 3.2),
    (1.8, 3.5),
    (2.0, 3.7),
    (2.0, 3.9),
    // Mitre taper — long elegant point
    (1.8, 4.2),
    (1.5, 4.5),
    (1.2, 4.8),
    (0.9, 5.1),
    (0.6, 5.4),
    (0.35, 5.7),
    (0.15, 6.0),
    // Tiny ball on tip
    (0.3, 6.1),
    (0.3, 6.2),
    (0.15, 6.3),
    (0.0, 6.3),
];

const PAWN_PROFILE: &[(f64, f64)] = &[
    // Wide base
    (0.0, 0.0),
    (2.8, 0.0),
    (3.0, 0.05),
    (3.0, 0.3),
    (2.8, 0.45),
    (2.4, 0.55),
    // Taper to narrow stem
    (1.8, 0.8),
    (1.4, 1.1),
    (1.1, 1.4),
    // Very narrow neck
    (0.85, 1.7),
    (0.75, 2.0),
    (0.75, 2.2),
    // Head — prominent round ball
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

impl Piece3D {
    fn profile(&self) -> &'static [(f64, f64)] {
        match self {
            Piece3D::King => KING_PROFILE,
            Piece3D::Queen => QUEEN_PROFILE,
            Piece3D::Rook => ROOK_PROFILE,
            Piece3D::Bishop => BISHOP_PROFILE,
            Piece3D::Pawn => PAWN_PROFILE,
        }
    }
}

const PIECE_CYCLE: [Piece3D; 5] = [
    Piece3D::King,
    Piece3D::Queen,
    Piece3D::Rook,
    Piece3D::Bishop,
    Piece3D::Pawn,
];
const CYCLE_TICKS: u64 = 80; // ~8 seconds per piece at 10fps

/// Cycles through pieces for the left display.
pub fn current_display_piece(tick: u64) -> Piece3D {
    PIECE_CYCLE[((tick / CYCLE_TICKS) as usize) % PIECE_CYCLE.len()]
}

/// Cycles through pieces for the right display (offset so it shows a different piece).
pub fn current_display_piece_alt(tick: u64) -> Piece3D {
    PIECE_CYCLE[((tick / CYCLE_TICKS) as usize + 2) % PIECE_CYCLE.len()]
}

fn arc_lengths(profile: &[(f64, f64)]) -> Vec<f64> {
    let mut lengths = Vec::with_capacity(profile.len());
    lengths.push(0.0);
    for i in 1..profile.len() {
        let dr = profile[i].0 - profile[i - 1].0;
        let dy = profile[i].1 - profile[i - 1].1;
        let seg = (dr * dr + dy * dy).sqrt();
        lengths.push(lengths[i - 1] + seg);
    }
    lengths
}

fn interpolate_profile(profile: &[(f64, f64)], lengths: &[f64], t: f64) -> (f64, f64, usize) {
    let total = *lengths.last().unwrap();
    let target = (t * total).min(total - 1e-9);

    let seg = lengths
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, &l)| l >= target)
        .map(|(i, _)| i - 1)
        .unwrap_or(0);

    let seg_start = lengths[seg];
    let next = (seg + 1).min(profile.len() - 1);
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

/// Pixel in the high-res buffer. 0.0 = empty, >0 = filled with luminance.
fn render_hires(piece: Piece3D, width: usize, hires_height: usize, tick: u64) -> Vec<Vec<f64>> {
    let w = width;
    let h = hires_height;

    let mut lum_buf = vec![vec![0.0f64; w]; h];
    let mut zbuffer = vec![vec![0.0f64; w]; h];

    let profile = piece.profile();
    let lengths = arc_lengths(profile);

    let y_max = profile.iter().map(|p| p.1).fold(0.0f64, f64::max);
    let r_max = profile.iter().map(|p| p.0).fold(0.0f64, f64::max);
    let y_center = y_max / 2.0;

    // Rotation: gentle tilt with slow oscillation + steady Y spin
    let a = 0.45 + (tick as f64 * 0.018).sin() * 0.12;
    let b = tick as f64 * 0.05;

    let (sin_a, cos_a) = a.sin_cos();
    let (sin_b, cos_b) = b.sin_cos();

    let k2 = 8.0;
    let scale_y = (h as f64 * 0.75 * k2) / y_max;
    let scale_x = (w as f64 * 0.75 * k2) / (r_max * 2.0);
    let scale = scale_y.min(scale_x);

    // Light direction (normalized): upper-right-front
    let lx = 0.4_f64;
    let ly = 0.6;
    let lz = -0.7;
    let llen = (lx * lx + ly * ly + lz * lz).sqrt();
    let (lx, ly, lz) = (lx / llen, ly / llen, lz / llen);

    // View direction for specular (straight at screen)
    let (vx, vy, vz) = (0.0, 0.0, -1.0);

    let theta_steps = 300;
    let phi_steps = 350;

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

            // Rotate X
            let x1 = x0;
            let y1 = y * cos_a - z0 * sin_a;
            let z1 = y * sin_a + z0 * cos_a;
            let nx1 = nx0;
            let ny1 = ny_p * cos_a - nz0 * sin_a;
            let nz1 = ny_p * sin_a + nz0 * cos_a;

            // Rotate Y
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

            let xi = xp as usize;
            let yi = yp as usize;

            if ooz > zbuffer[yi][xi] {
                zbuffer[yi][xi] = ooz;

                // Phong lighting: ambient + diffuse + specular
                let n_dot_l = nx2 * lx + ny2 * ly + nz2 * lz;

                if n_dot_l > 0.0 {
                    let ambient = 0.12;
                    let diffuse = n_dot_l * 0.65;

                    // Specular: reflect light across normal, dot with view
                    let rx = 2.0 * n_dot_l * nx2 - lx;
                    let ry = 2.0 * n_dot_l * ny2 - ly;
                    let rz = 2.0 * n_dot_l * nz2 - lz;
                    let r_dot_v = (rx * vx + ry * vy + rz * vz).max(0.0);
                    let specular = r_dot_v.powi(8) * 0.4;

                    let total = (ambient + diffuse + specular).min(1.0);
                    lum_buf[yi][xi] = total;
                } else {
                    // Back-face: very dim ambient only
                    lum_buf[yi][xi] = 0.06;
                }
            }
        }
    }

    lum_buf
}

/// Blend between two colors by factor t (0.0 = a, 1.0 = b).
fn lerp_color(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> Color {
    let t = t.clamp(0.0, 1.0);
    let r = (a.0 as f64 + (b.0 as f64 - a.0 as f64) * t) as u8;
    let g = (a.1 as f64 + (b.1 as f64 - a.1 as f64) * t) as u8;
    let b_val = (a.2 as f64 + (b.2 as f64 - a.2 as f64) * t) as u8;
    Color::Rgb(r, g, b_val)
}

/// Extract RGB from a ratatui Color, falling back to gray.
fn color_to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(idx) => {
            // Approximate 256-color palette for common values
            match idx {
                0 => (0, 0, 0),
                7 => (192, 192, 192),
                15 | 231 => (255, 255, 255),
                232..=255 => {
                    let v = (idx - 232) * 10 + 8;
                    (v, v, v)
                }
                16..=231 => {
                    let idx = idx - 16;
                    let b = (idx % 6) * 51;
                    let g = ((idx / 6) % 6) * 51;
                    let r = (idx / 36) * 51;
                    (r, g, b)
                }
                _ => (128, 128, 128),
            }
        }
        _ => (128, 128, 128),
    }
}

/// Map luminance to a smooth RGB color using the theme's accent as base hue.
fn lum_to_color(lum: f64, base_rgb: (u8, u8, u8)) -> Color {
    // Dark shadow color (very dark version of base hue)
    let shadow = (
        (base_rgb.0 as f64 * 0.12) as u8,
        (base_rgb.1 as f64 * 0.12) as u8,
        (base_rgb.2 as f64 * 0.12) as u8,
    );

    // Mid-tone: the base color itself
    let mid = base_rgb;

    // Highlight: bright white-ish tinted version
    let highlight = (
        (base_rgb.0 as f64 * 0.4 + 255.0 * 0.6) as u8,
        (base_rgb.1 as f64 * 0.4 + 255.0 * 0.6) as u8,
        (base_rgb.2 as f64 * 0.4 + 255.0 * 0.6) as u8,
    );

    if lum < 0.4 {
        // Shadow → mid
        lerp_color(shadow, mid, lum / 0.4)
    } else {
        // Mid → highlight
        lerp_color(mid, highlight, (lum - 0.4) / 0.6)
    }
}

/// Render a rotating 3D chess piece as themed ratatui Lines using half-block characters.
pub fn render_to_lines(
    piece: Piece3D,
    width: u16,
    height: u16,
    tick: u64,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let w = width as usize;
    let h = height as usize;
    if w < 4 || h < 2 {
        return vec![];
    }

    let hires_h = h * 2;
    let hires = render_hires(piece, w, hires_h, tick);

    // Use theme accent as the base color for shading
    let base_rgb = color_to_rgb(theme.accent);

    let mut lines = Vec::with_capacity(h);
    for row in 0..h {
        let top_row = row * 2;
        let bot_row = row * 2 + 1;

        let mut spans: Vec<Span> = Vec::with_capacity(w);
        for (col, top_val) in hires[top_row].iter().enumerate() {
            let top = *top_val;
            let bot = if bot_row < hires_h {
                hires[bot_row][col]
            } else {
                0.0
            };

            let top_filled = top > 0.01;
            let bot_filled = bot > 0.01;

            match (top_filled, bot_filled) {
                (false, false) => {
                    spans.push(Span::raw(" "));
                }
                (true, false) => {
                    spans.push(Span::styled(
                        "\u{2580}", // ▀
                        Style::default().fg(lum_to_color(top, base_rgb)),
                    ));
                }
                (false, true) => {
                    spans.push(Span::styled(
                        "\u{2584}", // ▄
                        Style::default().fg(lum_to_color(bot, base_rgb)),
                    ));
                }
                (true, true) => {
                    spans.push(Span::styled(
                        "\u{2580}", // ▀ — fg=top, bg=bottom
                        Style::default()
                            .fg(lum_to_color(top, base_rgb))
                            .bg(lum_to_color(bot, base_rgb)),
                    ));
                }
            }
        }

        lines.push(Line::from(spans));
    }

    lines
}
