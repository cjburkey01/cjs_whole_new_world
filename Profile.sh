# Mac requires sudo!!!
# Linux requires additional setup, see https://github.com/flamegraph-rs/flamegraph#installation
rm -rf target/release/assets
cp -R ./assets target/release/
RUSTFLAGS='-C force-frame-pointers=y' cargo flamegraph
