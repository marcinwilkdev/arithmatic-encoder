use std::fs::File;
use std::path::PathBuf;

use entropy_calculator::counter_pool::CounterPool;
use entropy_calculator::entropy_calculator::EntropyCalculator;
use entropy_calculator::messages::BytesChunk;
use entropy_calculator::symbols_reader::SymbolsReader;

pub fn show_file_entropy(filename: &PathBuf) {
    let file = File::open(filename).expect("File doesn't exist.");

    let (bytes_tx, bytes_rx) = crossbeam_channel::bounded::<BytesChunk>(1);

    SymbolsReader::new(file, bytes_tx).read_symbols();

    let mut counter_pool = CounterPool::new(bytes_rx);

    let counted_symbols = counter_pool.count_symbols(1);

    let mut entropy_calculator = EntropyCalculator::new(counted_symbols);

    let hx = entropy_calculator.calculate_hx();

    println!("Source entropy: {}", hx);
}

pub fn show_compression_ratio_and_symbol_len(symbols_count: usize, new_symbols_count: usize) {
    let compression_ratio = symbols_count as f64 / new_symbols_count as f64;

    println!("Compression ratio: {}", compression_ratio);

    println!(
        "Average symbol encoding length: {}",
        8.0 / compression_ratio
    );
}
