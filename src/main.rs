use std::path::PathBuf;

use structopt::StructOpt;

use arithmatic_compressor::{self, EncodedData};

#[derive(StructOpt, Debug)]
#[structopt(name = "compressor")]
struct Opt {
    #[structopt(short, long, parse(from_os_str))]
    file: PathBuf,
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
    #[structopt(short, long)]
    decode: bool,
}

fn main() {
    let opt = Opt::from_args();

    let file_content = std::fs::read(&opt.file).expect("error reading file");

    if opt.decode {
        let len = arithmatic_compressor::cast_bytes_to_u64(&file_content[..8]);
        let symbols_count = arithmatic_compressor::cast_bytes_to_u64(&file_content[8..16]);

        let data = &file_content[16..];

        let decoded =
            arithmatic_compressor::decompress(len as usize, symbols_count as usize, data, 256);

        std::fs::write(opt.output, decoded).expect("couldn't write result");
    } else {
        arithmatic_compressor::show_file_entropy(&opt.file);

        let start = std::time::Instant::now();

        let EncodedData { len, mut data } = arithmatic_compressor::compress(&file_content[..], 256);

        println!("Compressing time: {:?}", std::time::Instant::now() - start);

        let symbols_count = file_content.len();

        arithmatic_compressor::show_compression_ratio_and_symbol_len(symbols_count, data.len());

        let mut len_bytes = arithmatic_compressor::cast_u64_to_bytes(len as u64);
        let mut symbols_count_bytes =
            arithmatic_compressor::cast_u64_to_bytes(symbols_count as u64);

        len_bytes.append(&mut symbols_count_bytes);
        len_bytes.append(&mut data);

        std::fs::write(opt.output, len_bytes).expect("couldn't write result");
    }
}
