# arithmatic-encoder
Encoder and decorder created using adaptive arithmatic encoding.

## Setup
You need cargo to build and run this program.
You can install it using rustup: https://rustup.rs/

To run this project locally, compile it using cargo:
```
cargo build
````

## Code examples
Encode 'file_to_encode' and put output in 'output_file'.
```
cargo run --release -- --file 'file_to_encode' --output 'output_file'
```

Decode 'file_to_decode' and put output in 'output_file':
```
cargo run --release -- --file 'file_to_decode' --output 'output_file' --decode
