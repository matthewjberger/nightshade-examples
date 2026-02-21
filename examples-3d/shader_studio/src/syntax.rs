use egui::Color32;
use egui::text::LayoutJob;
use nightshade::prelude::egui;

const KEYWORDS: &[&str] = &[
    "fn",
    "var",
    "let",
    "const",
    "return",
    "if",
    "else",
    "for",
    "while",
    "loop",
    "break",
    "continue",
    "discard",
    "struct",
    "switch",
    "case",
    "default",
    "override",
    "enable",
    "diagnostic",
    "true",
    "false",
];

const TYPES: &[&str] = &[
    "f32",
    "f16",
    "i32",
    "u32",
    "bool",
    "vec2",
    "vec3",
    "vec4",
    "mat2x2",
    "mat3x3",
    "mat4x4",
    "mat2x3",
    "mat2x4",
    "mat3x2",
    "mat3x4",
    "mat4x2",
    "mat4x3",
    "array",
    "atomic",
    "ptr",
    "texture_1d",
    "texture_2d",
    "texture_2d_array",
    "texture_3d",
    "texture_cube",
    "texture_cube_array",
    "texture_multisampled_2d",
    "texture_storage_1d",
    "texture_storage_2d",
    "texture_storage_2d_array",
    "texture_storage_3d",
    "texture_depth_2d",
    "texture_depth_2d_array",
    "texture_depth_cube",
    "texture_depth_cube_array",
    "texture_depth_multisampled_2d",
    "sampler",
    "sampler_comparison",
];

const BUILTINS: &[&str] = &[
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "sinh",
    "cosh",
    "tanh",
    "sqrt",
    "inverseSqrt",
    "exp",
    "exp2",
    "log",
    "log2",
    "pow",
    "abs",
    "sign",
    "floor",
    "ceil",
    "round",
    "trunc",
    "fract",
    "min",
    "max",
    "clamp",
    "saturate",
    "mix",
    "step",
    "smoothstep",
    "length",
    "distance",
    "dot",
    "cross",
    "normalize",
    "faceForward",
    "reflect",
    "refract",
    "select",
    "transpose",
    "determinant",
    "all",
    "any",
    "countOneBits",
    "reverseBits",
    "extractBits",
    "insertBits",
    "firstLeadingBit",
    "firstTrailingBit",
    "pack4x8snorm",
    "pack4x8unorm",
    "unpack4x8snorm",
    "unpack4x8unorm",
    "textureSample",
    "textureLoad",
    "textureStore",
    "textureDimensions",
    "textureNumLevels",
    "textureNumLayers",
    "textureNumSamples",
    "textureSampleLevel",
    "textureSampleBias",
    "textureSampleCompare",
    "textureSampleCompareLevel",
    "textureGather",
    "textureGatherCompare",
    "storageBarrier",
    "workgroupBarrier",
    "workgroupUniformLoad",
    "arrayLength",
    "dpdx",
    "dpdy",
    "fwidth",
    "dpdxCoarse",
    "dpdyCoarse",
    "fwidthCoarse",
    "dpdxFine",
    "dpdyFine",
    "fwidthFine",
];

struct Theme {
    keyword: Color32,
    type_name: Color32,
    builtin: Color32,
    attribute: Color32,
    number: Color32,
    string: Color32,
    comment: Color32,
    punctuation: Color32,
    default: Color32,
}

const DARK_THEME: Theme = Theme {
    keyword: Color32::from_rgb(0xC5, 0x86, 0xC0),
    type_name: Color32::from_rgb(0x4E, 0xC9, 0xB0),
    builtin: Color32::from_rgb(0xDC, 0xDC, 0xAA),
    attribute: Color32::from_rgb(0x9C, 0xDC, 0xFE),
    number: Color32::from_rgb(0xB5, 0xCE, 0xA8),
    string: Color32::from_rgb(0xCE, 0x91, 0x78),
    comment: Color32::from_rgb(0x6A, 0x99, 0x55),
    punctuation: Color32::from_rgb(0xD4, 0xD4, 0xD4),
    default: Color32::from_rgb(0xD4, 0xD4, 0xD4),
};

pub fn highlight_wgsl(ui: &egui::Ui, text: &str, wrap_width: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;

    let font_id = egui::TextStyle::Monospace.resolve(ui.style());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut position = 0;

    while position < len {
        let ch = chars[position];

        if ch == '/' && position + 1 < len && chars[position + 1] == '/' {
            let start = position;
            while position < len && chars[position] != '\n' {
                position += 1;
            }
            let span: String = chars[start..position].iter().collect();
            append(&mut job, &span, &font_id, DARK_THEME.comment);
            continue;
        }

        if ch == '/' && position + 1 < len && chars[position + 1] == '*' {
            let start = position;
            position += 2;
            while position + 1 < len && !(chars[position] == '*' && chars[position + 1] == '/') {
                position += 1;
            }
            if position + 1 < len {
                position += 2;
            }
            let span: String = chars[start..position].iter().collect();
            append(&mut job, &span, &font_id, DARK_THEME.comment);
            continue;
        }

        if ch == '@' {
            let start = position;
            position += 1;
            while position < len && (chars[position].is_alphanumeric() || chars[position] == '_') {
                position += 1;
            }
            let span: String = chars[start..position].iter().collect();
            append(&mut job, &span, &font_id, DARK_THEME.attribute);
            continue;
        }

        if ch == '"' {
            let start = position;
            position += 1;
            while position < len && chars[position] != '"' {
                if chars[position] == '\\' {
                    position += 1;
                }
                position += 1;
            }
            if position < len {
                position += 1;
            }
            let span: String = chars[start..position].iter().collect();
            append(&mut job, &span, &font_id, DARK_THEME.string);
            continue;
        }

        if ch.is_ascii_digit()
            || (ch == '.' && position + 1 < len && chars[position + 1].is_ascii_digit())
        {
            let start = position;
            let mut has_dot = ch == '.';
            position += 1;
            while position < len {
                let next = chars[position];
                if next.is_ascii_digit() {
                    position += 1;
                } else if next == '.' && !has_dot {
                    has_dot = true;
                    position += 1;
                } else if next == 'e' || next == 'E' {
                    position += 1;
                    if position < len && (chars[position] == '+' || chars[position] == '-') {
                        position += 1;
                    }
                } else if next == 'u' || next == 'i' || next == 'f' || next == 'h' {
                    position += 1;
                    break;
                } else {
                    break;
                }
            }
            let span: String = chars[start..position].iter().collect();
            append(&mut job, &span, &font_id, DARK_THEME.number);
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let start = position;
            while position < len && (chars[position].is_alphanumeric() || chars[position] == '_') {
                position += 1;
            }
            let word: String = chars[start..position].iter().collect();

            let color = if KEYWORDS.contains(&word.as_str()) {
                DARK_THEME.keyword
            } else if TYPES.contains(&word.as_str()) {
                DARK_THEME.type_name
            } else if BUILTINS.contains(&word.as_str()) {
                DARK_THEME.builtin
            } else {
                DARK_THEME.default
            };

            append(&mut job, &word, &font_id, color);
            continue;
        }

        if "{}[]();:,.<>+-*/%&|^!~=".contains(ch) {
            let mut span = String::new();
            span.push(ch);
            position += 1;
            if position < len {
                let next = chars[position];
                if matches!(
                    (ch, next),
                    ('-', '>')
                        | ('=', '=')
                        | ('!', '=')
                        | ('<', '=')
                        | ('>', '=')
                        | ('&', '&')
                        | ('|', '|')
                        | ('<', '<')
                        | ('>', '>')
                        | ('+', '=')
                        | ('-', '=')
                        | ('*', '=')
                        | ('/', '=')
                        | ('%', '=')
                ) {
                    span.push(next);
                    position += 1;
                }
            }
            append(&mut job, &span, &font_id, DARK_THEME.punctuation);
            continue;
        }

        let mut span = String::new();
        span.push(ch);
        position += 1;
        append(&mut job, &span, &font_id, DARK_THEME.default);
    }

    job
}

fn append(job: &mut LayoutJob, text: &str, font_id: &egui::FontId, color: Color32) {
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: font_id.clone(),
            color,
            ..Default::default()
        },
    );
}
