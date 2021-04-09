extern crate basehanja;
use basehanja::{decode_utf8, encode_utf8, get_encodings};
use clap::{App, Arg};

fn main() {
    let matches = App::new("CLI example")
        .arg(
            Arg::with_name("action")
                .possible_values(&["enc", "dec"])
                .required(true),
        )
        .arg(
            Arg::with_name("encoding")
                .possible_values(&get_encodings())
                .required(true),
        )
        .arg(Arg::with_name("text").required(true))
        .get_matches();

    let codec = matches.value_of("encoding").unwrap();
    let text = matches.value_of("text").unwrap();
    // note: will panic if any error, because it tries to construct a JsValue on non-wasm...
    let res = match matches.value_of("action") {
        Some("enc") => encode_utf8(text, codec),
        Some("dec") => decode_utf8(text, codec),
        _ => unreachable!(),
    };

    if let Ok(res) = res {
        println!("{}", res);
    }
}
