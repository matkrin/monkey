build-wasm:
    cd wasm/ && wasm-pack build

run-dev:
    cd wasm/www/ && npm run dev
