# fivedice

A Rust/WebAssembly implementation of Yahtzee. WIP.

## Dependencies

- Node
- Rust
- wasm-pack

## Usage

Open two terminals. In one, execute `wasm-pack build` to build the WASM module. In the other, execute `npm install` if this is the first run, and then `npm run start`. Re-run `wasm-pack build` each time you change the Rust, and the webpack dev server will hot reload your changes.
