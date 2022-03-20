use std::path::PathBuf;

use structopt::StructOpt;

use arithmatic_compressor::{adaptive_decode, adaptive_encode};

#[derive(StructOpt, Debug)]
#[structopt(name = "compressor")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let file_content = std::fs::read_to_string(opt.file).expect("erro reading file");

    let mut cumulative = [0; 257];

    for i in 0..257 {
        cumulative[i] = i as u32;
    }

    let encoded = adaptive_encode(file_content.as_bytes(), &mut cumulative, 256);

    let mut cumulative = [0; 257];

    for i in 0..257 {
        cumulative[i] = i as u32;
    }

    let decoded = adaptive_decode(encoded.len, file_content.len(), &encoded.data, &mut cumulative, 256);

    assert_eq!(decoded, file_content.as_bytes());

    println!("{:?}", String::from_utf8_lossy(&decoded));
}
