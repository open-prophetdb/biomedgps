# BiomedGPS Graph

This project provides a Rust implementation for calculating various centrality measures of a graph, compiled to WebAssembly (Wasm) for use in a web frontend.

## Getting Started

### Prerequisites

Ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Build Instructions

1. Clone the repository:

   ```sh
   git clone https://github.com/yjcyxky/biomedgps.git
   cd biomedgps/wasm
   ```

2. Build the project:

   ```sh
   wasm-pack build --target web
   ```

3. Run example.js:

   ```sh
   python3 -m http.server 8000
   ```

   Open `http://localhost:8000` in your browser and see the output.
