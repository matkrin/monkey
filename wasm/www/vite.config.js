import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm"

export default defineConfig({
    base: "/monkey/",
    plugins: [
        wasm(),
    ],
    build: {
        target: "esnext",
    },
})
