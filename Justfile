
cli:
    cargo run --release -p matex-cli

debug:
    RUST_LOG=matex cargo run --release -p matex-cli

gui:
    cargo run -p matex-gui

web:
    echo "hello"