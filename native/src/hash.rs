

#[napi]
pub fn string_sha1(input: String) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let ml = (len as u64) * 8;

    let mut padded_len = len + 1;
    while padded_len % 64 != 56 {
        padded_len += 1;
    }
    let total_len = padded_len + 8;

    let mut msg = vec![0u8; total_len];
    msg[..len].copy_from_slice(bytes);
    msg[len] = 0x80;

    let be_len = ml.to_be_bytes();
    msg[total_len - 8..total_len].copy_from_slice(&be_len);

    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for t in 0..16 {
            w[t] = u32::from_be_bytes([
                chunk[t * 4],
                chunk[t * 4 + 1],
                chunk[t * 4 + 2],
                chunk[t * 4 + 3],
            ]);
        }
        for t in 16..80 {
            w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);

        for t in 0..80 {
            let (f, k) = match t {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[t]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}",
        h0, h1, h2, h3, h4
    )
}

#[napi]
pub fn string_hash(s: String) -> i32 {
    let mut hash: i32 = 0;
    for ch in s.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(ch as i32);
    }
    hash
}

#[napi]
pub fn number_hash(val: i32, initial_hash: i32) -> i32 {
    (initial_hash << 5).wrapping_sub(initial_hash).wrapping_add(val)
}

#[napi]
pub fn object_hash(obj: serde_json::Value) -> i32 {
    do_hash(&obj, 0)
}

fn do_hash(value: &serde_json::Value, mut hash: i32) -> i32 {
    match value {
        serde_json::Value::Null => {
            hash = number_hash(0, hash);
            hash = number_hash(0, hash);
            hash
        }
        serde_json::Value::Bool(b) => {
            hash = number_hash(if *b { 1 } else { 0 }, hash);
            hash
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                hash = number_hash(i as i32, hash);
            } else if let Some(f) = n.as_f64() {
                hash = number_hash(f as i32, hash);
            }
            hash
        }
        serde_json::Value::String(s) => {
            hash = string_hash_inner(s, hash);
            hash
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                hash = do_hash(item, hash);
            }
            hash
        }
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                hash = string_hash_inner(key, hash);
                hash = do_hash(&map[key], hash);
            }
            hash
        }
    }
}

fn string_hash_inner(s: &str, init: i32) -> i32 {
    let mut hash = init;
    for ch in s.chars() {
        hash = number_hash(ch as i32, hash);
    }
    hash
}
