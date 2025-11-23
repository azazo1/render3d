default: ray_tracing

ray_tracing:
    cargo run --bin ray_tracing --release

alias s := wasm_server
wasm_server:
    @if command -v simple-http-server &>/dev/null; \
    then cd www && simple-http-server; \
    else cargo run --bin wasm_server --features=wasm_server; \
    fi

alias b := build
build profile='release':
    cargo build --profile {{profile}}
    wasm-pack build --target web