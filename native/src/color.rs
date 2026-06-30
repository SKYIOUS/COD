use serde::Serialize;

#[derive(Serialize)]
#[napi(object)]
pub struct CssColor {
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub a: f64,
}

fn hex_digit(c: u8) -> i32 {
    match c {
        b'0'..=b'9' => (c - b'0') as i32,
        b'a'..=b'f' => (c - b'a' + 10) as i32,
        b'A'..=b'F' => (c - b'A' + 10) as i32,
        _ => 0,
    }
}

#[napi]
pub fn parse_css_color(css: String) -> Option<CssColor> {
    let css = css.trim();
    if css.is_empty() || css == "transparent" {
        return Some(CssColor { r: 0, g: 0, b: 0, a: 0.0 });
    }

    if css.starts_with('#') {
        return parse_hex(css);
    }

    if css.starts_with("rgba(") || css.starts_with("rgb(") {
        return parse_rgb(css);
    }

    if css.starts_with("hsla(") || css.starts_with("hsl(") {
        return parse_hsl(css);
    }

    parse_named(css)
}

fn parse_hex(hex: &str) -> Option<CssColor> {
    let bytes = hex.as_bytes();
    let len = bytes.len();

    if len < 2 || bytes[0] != b'#' {
        return None;
    }

    match len {
        7 => {
            let r = hex_digit(bytes[1]) * 16 + hex_digit(bytes[2]);
            let g = hex_digit(bytes[3]) * 16 + hex_digit(bytes[4]);
            let b = hex_digit(bytes[5]) * 16 + hex_digit(bytes[6]);
            Some(CssColor { r, g, b, a: 1.0 })
        }
        9 => {
            let r = hex_digit(bytes[1]) * 16 + hex_digit(bytes[2]);
            let g = hex_digit(bytes[3]) * 16 + hex_digit(bytes[4]);
            let b = hex_digit(bytes[5]) * 16 + hex_digit(bytes[6]);
            let a = (hex_digit(bytes[7]) * 16 + hex_digit(bytes[8])) as f64 / 255.0;
            Some(CssColor { r, g, b, a })
        }
        4 => {
            let r = hex_digit(bytes[1]) * 17;
            let g = hex_digit(bytes[2]) * 17;
            let b = hex_digit(bytes[3]) * 17;
            Some(CssColor { r, g, b, a: 1.0 })
        }
        5 => {
            let r = hex_digit(bytes[1]) * 17;
            let g = hex_digit(bytes[2]) * 17;
            let b = hex_digit(bytes[3]) * 17;
            let a = (hex_digit(bytes[4]) * 17) as f64 / 255.0;
            Some(CssColor { r, g, b, a })
        }
        _ => None,
    }
}

fn parse_rgb(css: &str) -> Option<CssColor> {
    let inner = if css.starts_with("rgba(") {
        css.strip_prefix("rgba(")?.strip_suffix(')')?
    } else {
        css.strip_prefix("rgb(")?.strip_suffix(')')?
    };

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.len() < 3 {
        return None;
    }

    let r = parts[0].parse::<i32>().ok()?;
    let g = parts[1].parse::<i32>().ok()?;
    let b = parts[2].parse::<i32>().ok()?;
    let a = if parts.len() > 3 {
        parts[3].parse::<f64>().ok().unwrap_or(1.0)
    } else {
        1.0
    };

    Some(CssColor { r, g, b, a })
}

fn parse_hsl(css: &str) -> Option<CssColor> {
    let inner = if css.starts_with("hsla(") {
        css.strip_prefix("hsla(")?.strip_suffix(')')?
    } else {
        css.strip_prefix("hsl(")?.strip_suffix(')')?
    };

    let parts: Vec<&str> = inner.split(&[',', ' '][..]).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if parts.len() < 3 { return None; }

    let h = parts[0].trim_end_matches("deg").parse::<f64>().ok()?;
    let s = parts[1].trim_end_matches('%').parse::<f64>().ok()?;
    let l = parts[2].trim_end_matches('%').parse::<f64>().ok()?;
    let a = if parts.len() > 3 { parts[3].parse::<f64>().ok().unwrap_or(1.0) } else { 1.0 };

    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (rp, gp, bp) = match h as i32 % 360 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Some(CssColor {
        r: ((rp + m) * 255.0).round() as i32,
        g: ((gp + m) * 255.0).round() as i32,
        b: ((bp + m) * 255.0).round() as i32,
        a,
    })
}

fn parse_named(name: &str) -> Option<CssColor> {
    let c = match name {
        "aliceblue" => (240, 248, 255),
        "antiquewhite" => (250, 235, 215),
        "aqua" => (0, 255, 255),
        "aquamarine" => (127, 255, 212),
        "azure" => (240, 255, 255),
        "beige" => (245, 245, 220),
        "bisque" => (255, 228, 196),
        "black" => (0, 0, 0),
        "blanchedalmond" => (255, 235, 205),
        "blue" => (0, 0, 255),
        "blueviolet" => (138, 43, 226),
        "brown" => (165, 42, 42),
        "burlywood" => (222, 184, 135),
        "cadetblue" => (95, 158, 160),
        "chartreuse" => (127, 255, 0),
        "chocolate" => (210, 105, 30),
        "coral" => (255, 127, 80),
        "cornflowerblue" => (100, 149, 237),
        "cornsilk" => (255, 248, 220),
        "crimson" => (220, 20, 60),
        "cyan" => (0, 255, 255),
        "darkblue" => (0, 0, 139),
        "darkcyan" => (0, 139, 139),
        "darkgoldenrod" => (184, 134, 11),
        "darkgray" => (169, 169, 169),
        "darkgreen" => (0, 100, 0),
        "darkgrey" => (169, 169, 169),
        "darkkhaki" => (189, 183, 107),
        "darkmagenta" => (139, 0, 139),
        "darkolivegreen" => (85, 107, 47),
        "darkorange" => (255, 140, 0),
        "darkorchid" => (153, 50, 204),
        "darkred" => (139, 0, 0),
        "darksalmon" => (233, 150, 122),
        "darkseagreen" => (143, 188, 143),
        "darkslateblue" => (72, 61, 139),
        "darkslategray" => (47, 79, 79),
        "darkslategrey" => (47, 79, 79),
        "darkturquoise" => (0, 206, 209),
        "darkviolet" => (148, 0, 211),
        "deeppink" => (255, 20, 147),
        "deepskyblue" => (0, 191, 255),
        "dimgray" => (105, 105, 105),
        "dimgrey" => (105, 105, 105),
        "dodgerblue" => (30, 144, 255),
        "firebrick" => (178, 34, 34),
        "floralwhite" => (255, 250, 240),
        "forestgreen" => (34, 139, 34),
        "fuchsia" => (255, 0, 255),
        "gainsboro" => (220, 220, 220),
        "ghostwhite" => (248, 248, 255),
        "gold" => (255, 215, 0),
        "goldenrod" => (218, 165, 32),
        "gray" => (128, 128, 128),
        "green" => (0, 128, 0),
        "greenyellow" => (173, 255, 47),
        "grey" => (128, 128, 128),
        "honeydew" => (240, 255, 240),
        "hotpink" => (255, 105, 180),
        "indianred" => (205, 92, 92),
        "indigo" => (75, 0, 130),
        "ivory" => (255, 255, 240),
        "khaki" => (240, 230, 140),
        "lavender" => (230, 230, 250),
        "lavenderblush" => (255, 240, 245),
        "lawngreen" => (124, 252, 0),
        "lemonchiffon" => (255, 250, 205),
        "lightblue" => (173, 216, 230),
        "lightcoral" => (240, 128, 128),
        "lightcyan" => (224, 255, 255),
        "lightgoldenrodyellow" => (250, 250, 210),
        "lightgray" => (211, 211, 211),
        "lightgreen" => (144, 238, 144),
        "lightgrey" => (211, 211, 211),
        "lightpink" => (255, 182, 193),
        "lightsalmon" => (255, 160, 122),
        "lightseagreen" => (32, 178, 170),
        "lightskyblue" => (135, 206, 250),
        "lightslategray" => (119, 136, 153),
        "lightslategrey" => (119, 136, 153),
        "lightsteelblue" => (176, 196, 222),
        "lightyellow" => (255, 255, 224),
        "lime" => (0, 255, 0),
        "limegreen" => (50, 205, 50),
        "linen" => (250, 240, 230),
        "magenta" => (255, 0, 255),
        "maroon" => (128, 0, 0),
        "mediumaquamarine" => (102, 205, 170),
        "mediumblue" => (0, 0, 205),
        "mediumorchid" => (186, 85, 211),
        "mediumpurple" => (147, 112, 219),
        "mediumseagreen" => (60, 179, 113),
        "mediumslateblue" => (123, 104, 238),
        "mediumspringgreen" => (0, 250, 154),
        "mediumturquoise" => (72, 209, 204),
        "mediumvioletred" => (199, 21, 133),
        "midnightblue" => (25, 25, 112),
        "mintcream" => (245, 255, 250),
        "mistyrose" => (255, 228, 225),
        "moccasin" => (255, 228, 181),
        "navajowhite" => (255, 222, 173),
        "navy" => (0, 0, 128),
        "oldlace" => (253, 245, 230),
        "olive" => (128, 128, 0),
        "olivedrab" => (107, 142, 35),
        "orange" => (255, 165, 0),
        "orangered" => (255, 69, 0),
        "orchid" => (218, 112, 214),
        "palegoldenrod" => (238, 232, 170),
        "palegreen" => (152, 251, 152),
        "paleturquoise" => (175, 238, 238),
        "palevioletred" => (219, 112, 147),
        "papayawhip" => (255, 239, 213),
        "peachpuff" => (255, 218, 185),
        "peru" => (205, 133, 63),
        "pink" => (255, 192, 203),
        "plum" => (221, 160, 221),
        "powderblue" => (176, 224, 230),
        "purple" => (128, 0, 128),
        "rebeccapurple" => (102, 51, 153),
        "red" => (255, 0, 0),
        "rosybrown" => (188, 143, 143),
        "royalblue" => (65, 105, 225),
        "saddlebrown" => (139, 69, 19),
        "salmon" => (250, 128, 114),
        "sandybrown" => (244, 164, 96),
        "seagreen" => (46, 139, 87),
        "seashell" => (255, 245, 238),
        "sienna" => (160, 82, 45),
        "silver" => (192, 192, 192),
        "skyblue" => (135, 206, 235),
        "slateblue" => (106, 90, 205),
        "slategray" => (112, 128, 144),
        "slategrey" => (112, 128, 144),
        "snow" => (255, 250, 250),
        "springgreen" => (0, 255, 127),
        "steelblue" => (70, 130, 180),
        "tan" => (210, 180, 140),
        "teal" => (0, 128, 128),
        "thistle" => (216, 191, 216),
        "tomato" => (255, 99, 71),
        "turquoise" => (64, 224, 208),
        "violet" => (238, 130, 238),
        "wheat" => (245, 222, 179),
        "white" => (255, 255, 255),
        "whitesmoke" => (245, 245, 245),
        "yellow" => (255, 255, 0),
        "yellowgreen" => (154, 205, 50),
        _ => return None,
    };
    Some(CssColor { r: c.0, g: c.1, b: c.2, a: 1.0 })
}
