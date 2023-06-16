SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."
cargo run -- examples/ --rust-out examples/rust-server/src --csharp-out examples/cs-server/src
