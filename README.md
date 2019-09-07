# fivedice

A Rust/WebAssembly implementation of Yahtzee. WIP.

## Dependencies

- Node
- Rust
- wasm-pack

## Usage

Open two terminals. In one, execute `make` to build the WASM module. In the other, execute `npm install` if this is the first run, and then `npm run start`. Re-run `make` each time you change the Rust, and the webpack dev server will hot reload your changes. Use `make clean` to clear build artifacts.
