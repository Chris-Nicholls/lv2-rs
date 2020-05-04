#! /bin/bash
set -x -e
rustup run nightly cargo build --release  --target=armv7-unknown-linux-gnueabihf  --features plugin
rustup run nightly cargo build --release  --target=aarch64-unknown-linux-gnu  --features plugin
cp target/armv7-unknown-linux-gnueabihf/release/libeffect.so reverb-rs.lv2/
cp target/aarch64-unknown-linux-gnu/release/libeffect.so reverb-rs.lv2/libreverb64.so





rm -rf ~/.lv2/reverb-rs.lv2/
cp -r reverb-rs.lv2/ ~/.lv2/
tar cz reverb-rs.lv2 | base64 | curl -F 'package=@-' http://192.168.51.1/sdk/install