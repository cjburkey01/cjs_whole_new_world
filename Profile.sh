# Mac requires sudo
# Linux requires additional setup, see https://github.com/flamegraph-rs/flamegraph#installation
sudo rm -rf target/release/assets
sudo cp -R ./assets target/release/
sudo RUSTFLAGS='-C force-frame-pointers=y' cargo flamegraph
