use napi::bindgen_prelude::*;

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_URL_SAFE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn decode_base64_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

fn decode_hex_char(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[napi]
pub fn native_encode_hex(input: Buffer) -> String {
    let bytes = input.as_ref();
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    result
}

#[napi]
pub fn native_decode_hex(hex: String) -> Vec<u8> {
    let hex = hex.as_bytes();
    if hex.len() % 2 != 0 {
        return Vec::new();
    }
    let mut out = vec![0u8; hex.len() / 2];
    for i in (0..hex.len()).step_by(2) {
        match (decode_hex_char(hex[i]), decode_hex_char(hex[i + 1])) {
            (Some(hi), Some(lo)) => out[i >> 1] = (hi << 4) | lo,
            _ => return Vec::new(),
        }
    }
    out
}

#[napi]
pub fn native_encode_base64(input: Buffer, padded: Option<bool>, url_safe: Option<bool>) -> String {
    let padded = padded.unwrap_or(true);
    let url_safe = url_safe.unwrap_or(false);
    let dict = if url_safe {
        BASE64_URL_SAFE
    } else {
        BASE64_CHARS
    };
    let bytes = input.as_ref();
    let len = bytes.len();
    let remainder = len % 3;
    let full = len - remainder;
    let cap = (len / 3) * 4 + if remainder > 0 { 4 } else { 0 };
    let mut r = String::with_capacity(cap);
    let mut i = 0;
    while i < full {
        let a = bytes[i];
        let b = bytes[i + 1];
        let c = bytes[i + 2];
        r.push(dict[(a >> 2) as usize] as char);
        r.push(dict[((a << 4 | b >> 4) & 0b111111) as usize] as char);
        r.push(dict[((b << 2 | c >> 6) & 0b111111) as usize] as char);
        r.push(dict[(c & 0b111111) as usize] as char);
        i += 3;
    }
    if remainder == 1 {
        let a = bytes[i];
        r.push(dict[(a >> 2) as usize] as char);
        r.push(dict[((a << 4) & 0b111111) as usize] as char);
        if padded {
            r.push_str("==");
        }
    } else if remainder == 2 {
        let a = bytes[i];
        let b = bytes[i + 1];
        r.push(dict[(a >> 2) as usize] as char);
        r.push(dict[((a << 4 | b >> 4) & 0b111111) as usize] as char);
        r.push(dict[((b << 2) & 0b111111) as usize] as char);
        if padded {
            r.push('=');
        }
    }
    r
}

#[napi]
pub fn native_decode_base64(input: String) -> Vec<u8> {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return vec![];
    }
    let mut padding = 0;
    if len > 0 && bytes[len - 1] == b'=' {
        padding += 1;
    }
    if len > 1 && bytes[len - 2] == b'=' {
        padding += 1;
    }
    let olen = (len * 6) / 8 - padding;
    if olen == 0 {
        return vec![];
    }
    let mut out = vec![0u8; olen];
    let mut pos = 0;
    let mut i = 0;
    while i < len && bytes[i] != b'=' {
        let rem = len - i - padding;
        let cs = std::cmp::min(rem, 4);
        let mut acc = 0u32;
        for j in 0..cs {
            match decode_base64_char(bytes[i + j]) {
                Some(v) => acc = (acc << 6) | (v as u32),
                None => return vec![],
            }
        }
        acc <<= (4 - cs) * 6;
        if cs >= 2 && pos < olen {
            out[pos] = (acc >> 16) as u8;
            pos += 1;
        }
        if cs >= 3 && pos < olen {
            out[pos] = (acc >> 8) as u8;
            pos += 1;
        }
        if cs >= 4 && pos < olen {
            out[pos] = acc as u8;
            pos += 1;
        }
        i += cs;
    }
    if pos != olen {
        return vec![];
    }
    out
}
