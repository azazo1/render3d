default: ray_tracing

ray_tracing features='rayon':
    cargo run --bin ray_tracing --release --features={{features}}

alias s := wasm_server
wasm_server: build
    @if command -v simple-http-server &>/dev/null; \
    then simple-http-server -- ./www; \
    else cargo run --bin wasm_server --features=wasm_server; \
    fi

alias b := build
build profile='release':
    cargo build --profile {{profile}}
    wasm-pack build --target web

bench:
    @if command -v samply &>/dev/null; \
    then samply record cargo bench; \
    else cargo bench; \
    fi