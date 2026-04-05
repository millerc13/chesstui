//! Custom Kitty graphics protocol transmitter using PNG compression.
//! Bypasses ratatui-image to use f=100 (PNG) instead of f=32 (raw RGBA),
//! reducing transmission from ~1.4MB to ~100KB per board image.

use std::fmt::Write;
use image::RgbImage;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Encode an RgbImage as PNG bytes.
pub fn encode_png(img: &RgbImage) -> Vec<u8> {
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut buf,
        image::codecs::png::CompressionType::Fast,
        image::codecs::png::FilterType::Sub,
    );
    image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgb8,
    )
    .expect("PNG encoding should not fail");
    buf
}

/// A simple hash of image content for change detection.
/// Uses FNV-1a on a sampled subset of pixels for speed.
pub fn image_hash(img: &RgbImage) -> u64 {
    let raw = img.as_raw();
    let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
    let step = 64.max(raw.len() / 16384);
    let mut i = 0;
    while i < raw.len() {
        hash ^= raw[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        i += step;
    }
    hash
}

/// Cached Kitty board state.
pub struct KittyBoardCache {
    pub transmit_str: String,
    pub image_hash: u64,
    pub image_id: u32,
    pub area: Rect,
}

/// Build the Kitty escape sequence to transmit a PNG image with virtual placement.
/// Uses `f=100` (PNG format) and `U=1` (virtual/unicode placement).
pub fn build_transmit_sequence(png_bytes: &[u8], image_id: u32) -> String {
    use base64::Engine;

    const CHARS_PER_CHUNK: usize = 4096;
    const CHUNK_SIZE: usize = (CHARS_PER_CHUNK / 4) * 3;

    let chunks: Vec<&[u8]> = png_bytes.chunks(CHUNK_SIZE).collect();
    let chunk_count = chunks.len();

    let mut data = String::with_capacity(chunk_count * (CHARS_PER_CHUNK + 60));

    for (i, chunk) in chunks.iter().enumerate() {
        write!(data, "\x1b_Gq=2,").unwrap();

        if i == 0 {
            write!(data, "i={image_id},a=T,U=1,f=100,t=d,").unwrap();
        }

        let more = if i + 1 < chunk_count { 1 } else { 0 };
        write!(data, "m={more};").unwrap();

        base64::engine::general_purpose::STANDARD.encode_string(chunk, &mut data);

        data.push_str("\x1b\\");
    }

    data
}

/// Render the Kitty board image into ratatui's buffer using unicode placeholders.
pub fn render_to_buffer(
    cache: &KittyBoardCache,
    area: Rect,
    buf: &mut Buffer,
    needs_transmit: bool,
) {
    let image_id = cache.image_id;
    let [id_extra, id_r, id_g, id_b] = image_id.to_be_bytes();
    let id_color = format!("\x1b[38;2;{id_r};{id_g};{id_b}m");
    let id_extra_val = u16::from(id_extra);

    let full_width = area.width.min(cache.area.width);
    let height = area.height.min(cache.area.height).min(297);

    let row_diacritics: String =
        std::iter::repeat_n('\u{10EEEE}', full_width.saturating_sub(1) as usize).collect();

    let right = area.width - 1;
    let down = area.height - 1;
    let restore_cursor = format!("\x1b[u\x1b[{right}C\x1b[{down}B");

    for y in 0..height {
        let mut symbol = String::with_capacity(if needs_transmit && y == 0 {
            cache.transmit_str.len() + 200
        } else {
            200
        });

        if needs_transmit && y == 0 {
            symbol.push_str(&cache.transmit_str);
        }

        write!(
            symbol,
            "\x1b[s{id_color}\u{10EEEE}{}{}{}",
            diacritic(y),
            diacritic(0),
            diacritic(id_extra_val),
        )
        .unwrap();

        symbol.push_str(&row_diacritics);

        for x in 1..full_width {
            if let Some(cell) = buf.cell_mut((area.left() + x, area.top() + y)) {
                cell.set_skip(true);
            }
        }

        symbol.push_str(&restore_cursor);

        if let Some(cell) = buf.cell_mut((area.left(), area.top() + y)) {
            cell.set_symbol(&symbol);
        }
    }
}

fn diacritic(index: u16) -> char {
    const TABLE: [char; 297] = [
        '\u{305}','\u{30D}','\u{30E}','\u{310}','\u{312}','\u{33D}','\u{33E}','\u{33F}',
        '\u{346}','\u{34A}','\u{34B}','\u{34C}','\u{350}','\u{351}','\u{352}','\u{357}',
        '\u{35B}','\u{363}','\u{364}','\u{365}','\u{366}','\u{367}','\u{368}','\u{369}',
        '\u{36A}','\u{36B}','\u{36C}','\u{36D}','\u{36E}','\u{36F}','\u{483}','\u{484}',
        '\u{485}','\u{486}','\u{487}','\u{592}','\u{593}','\u{594}','\u{595}','\u{597}',
        '\u{598}','\u{599}','\u{59C}','\u{59D}','\u{59E}','\u{59F}','\u{5A0}','\u{5A1}',
        '\u{5A8}','\u{5A9}','\u{5AB}','\u{5AC}','\u{5AF}','\u{5C4}','\u{610}','\u{611}',
        '\u{612}','\u{613}','\u{614}','\u{615}','\u{616}','\u{617}','\u{657}','\u{658}',
        '\u{659}','\u{65A}','\u{65B}','\u{65D}','\u{65E}','\u{6D6}','\u{6D7}','\u{6D8}',
        '\u{6D9}','\u{6DA}','\u{6DB}','\u{6DC}','\u{6DF}','\u{6E0}','\u{6E1}','\u{6E2}',
        '\u{6E4}','\u{6E7}','\u{6E8}','\u{6EB}','\u{6EC}','\u{730}','\u{732}','\u{733}',
        '\u{735}','\u{736}','\u{73A}','\u{73D}','\u{73F}','\u{740}','\u{741}','\u{743}',
        '\u{745}','\u{747}','\u{749}','\u{74A}','\u{7EB}','\u{7EC}','\u{7ED}','\u{7EE}',
        '\u{7EF}','\u{7F0}','\u{7F1}','\u{7F3}','\u{816}','\u{817}','\u{818}','\u{819}',
        '\u{81B}','\u{81C}','\u{81D}','\u{81E}','\u{81F}','\u{820}','\u{821}','\u{822}',
        '\u{823}','\u{825}','\u{826}','\u{827}','\u{829}','\u{82A}','\u{82B}','\u{82C}',
        '\u{82D}','\u{951}','\u{953}','\u{954}','\u{F82}','\u{F83}','\u{F86}','\u{F87}',
        '\u{135D}','\u{135E}','\u{135F}','\u{17DD}','\u{193A}','\u{1A17}','\u{1A75}','\u{1A76}',
        '\u{1A77}','\u{1A78}','\u{1A79}','\u{1A7A}','\u{1A7B}','\u{1A7C}','\u{1B6B}','\u{1B6D}',
        '\u{1B6E}','\u{1B6F}','\u{1B70}','\u{1B71}','\u{1B72}','\u{1B73}','\u{1CD0}','\u{1CD1}',
        '\u{1CD2}','\u{1CDA}','\u{1CDB}','\u{1CE0}','\u{1DC0}','\u{1DC1}','\u{1DC3}','\u{1DC4}',
        '\u{1DC5}','\u{1DC6}','\u{1DC7}','\u{1DC8}','\u{1DC9}','\u{1DCB}','\u{1DCC}','\u{1DD1}',
        '\u{1DD2}','\u{1DD3}','\u{1DD4}','\u{1DD5}','\u{1DD6}','\u{1DD7}','\u{1DD8}','\u{1DD9}',
        '\u{1DDA}','\u{1DDB}','\u{1DDC}','\u{1DDD}','\u{1DDE}','\u{1DDF}','\u{1DE0}','\u{1DE1}',
        '\u{1DE2}','\u{1DE3}','\u{1DE4}','\u{1DE5}','\u{1DE6}','\u{1DFE}','\u{20D0}','\u{20D1}',
        '\u{20D4}','\u{20D5}','\u{20D6}','\u{20D7}','\u{20DB}','\u{20DC}','\u{20E1}','\u{20E7}',
        '\u{20E9}','\u{20F0}','\u{2CEF}','\u{2CF0}','\u{2CF1}','\u{2DE0}','\u{2DE1}','\u{2DE2}',
        '\u{2DE3}','\u{2DE4}','\u{2DE5}','\u{2DE6}','\u{2DE7}','\u{2DE8}','\u{2DE9}','\u{2DEA}',
        '\u{2DEB}','\u{2DEC}','\u{2DED}','\u{2DEE}','\u{2DEF}','\u{2DF0}','\u{2DF1}','\u{2DF2}',
        '\u{2DF3}','\u{2DF4}','\u{2DF5}','\u{2DF6}','\u{2DF7}','\u{2DF8}','\u{2DF9}','\u{2DFA}',
        '\u{2DFB}','\u{2DFC}','\u{2DFD}','\u{2DFE}','\u{2DFF}','\u{A66F}','\u{A67C}','\u{A67D}',
        '\u{A6F0}','\u{A6F1}','\u{A8E0}','\u{A8E1}','\u{A8E2}','\u{A8E3}','\u{A8E4}','\u{A8E5}',
        '\u{A8E6}','\u{A8E7}','\u{A8E8}','\u{A8E9}','\u{A8EA}','\u{A8EB}','\u{A8EC}','\u{A8ED}',
        '\u{A8EE}','\u{A8EF}','\u{A8F0}','\u{A8F1}','\u{AAB0}','\u{AAB2}','\u{AAB3}','\u{AAB7}',
        '\u{AAB8}','\u{AABE}','\u{AABF}','\u{AAC1}','\u{FE20}','\u{FE21}','\u{FE22}','\u{FE23}',
        '\u{FE24}','\u{FE25}','\u{FE26}','\u{10A0F}','\u{10A38}','\u{1D185}','\u{1D186}','\u{1D187}',
        '\u{1D188}','\u{1D189}','\u{1D1AA}','\u{1D1AB}','\u{1D1AC}','\u{1D1AD}','\u{1D242}','\u{1D243}',
        '\u{1D244}',
    ];
    TABLE.get(index as usize).copied().unwrap_or(TABLE[0])
}
