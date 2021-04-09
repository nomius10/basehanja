extern crate basehanja;
use std::io;

fn read_line() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line
}

macro_rules! pop_arg {
    ($line:expr) => {{
        let trimmed = $line.trim_start();
        if let Some(arg) = trimmed.split_whitespace().next() {
            let rest = trimmed.strip_prefix(arg).unwrap();
            (arg, rest)
        } else {
            ("", "")
        }
    }};
}

fn usage() {
    eprint!(
        "Usage:
exit
enc <encoding> <text>
dec <encoding> <text>
\n"
    )
}

fn main() {
    loop {
        eprint!(">>> ");
        let line = read_line();
        let line = &line[0..line.len() - 1]; // remove \n

        let (arg, rest) = pop_arg!(line);
        let encode = match arg {
            "exit" => {
                break;
            }
            "enc" => true,
            "dec" => false,
            _ => {
                usage();
                continue;
            }
        };

        let (arg, rest) = pop_arg!(rest);
        // if any error is raised, it will crash, because it constructs a JsValue...
        let res = if encode {
            basehanja::encode_utf8(rest, arg)
        } else {
            basehanja::decode_utf8(rest, arg)
        };

        if let Ok(s) = res {
            println!("{}", s);
        }
    }
}
