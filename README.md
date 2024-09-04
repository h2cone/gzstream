# gzstream

This project demonstrates how to stream process a Gzip compressed file using Rust. It fetches a Gzip compressed file from a URL, decompresses it, processes each line, and then recompresses the output.

## Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)
- Node.js (for running the preprocessing script)

## Usage

0. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/gzstream.git
    cd gzstream
    ```

1. Run the preprocessing script using Node.js:
    ```sh
    node scripts/fs.mjs
    ```

2. Run the project using Cargo:
    ```sh
    cargo run > assets/transformed.vcf.gz
    ```

3. The program will fetch the Gzip file from the specified URL, decompress it, process each line, and recompress the output to `assets/transformed.vcf.gz`.