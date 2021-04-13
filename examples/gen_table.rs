extern crate basehanja;

use basehanja::{encode_utf8, get_encodings};

static SAMPLES: &[(&str, &str)] = &[
    ("hello_world_012345", "ASCII"),
    ("俺の日本語は下手", "2 byte"),
    ("𠜎𠜱𠝹𠱓𠱸𠲖𠳏", "3 byte"),
    ("🐵🙈🙉🙊", "emoji"),
];

fn main() {
    println!("|encoding|chars_kind|%c_incr|%b_incr|Δc|Δb|example_text|char_change|byte_change|");
    println!("|--------|----------|------:|------:|-:|-:|:-----------|:---------:|:---------:|");
    for codec in get_encodings() {
        for (sample, desc) in SAMPLES {
            let enc = encode_utf8(sample, codec).unwrap();
            let c_len = sample.chars().count();
            let b_len = sample.len();
            let c_len2 = enc.chars().count();
            let b_len2 = enc.len();
            let c_eff = 100.0 * (c_len2 as f32) / (c_len as f32) - 100.0;
            let b_eff = 100.0 * (b_len2 as f32) / (b_len as f32) - 100.0;
            let enc64 = encode_utf8(sample, "base64").unwrap();
            let c_len64 = enc64.chars().count();
            let b_len64 = enc64.len();
            let dt_c_eff = c_eff - 100.0 * (c_len64 as f32) / (c_len as f32) + 100.0;
            let dt_b_eff = b_eff - 100.0 * (b_len64 as f32) / (b_len as f32) + 100.0;
            println!(
                "|{}|{}|{:.2}|{:.2}|{:.2}|{:.2}|{}|{} -> {}|{} -> {}|",
                codec, desc, c_eff, b_eff, dt_c_eff, dt_b_eff, sample, c_len, c_len2, b_len, b_len2
            );
        }
    }
}
