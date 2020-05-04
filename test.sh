set -x -e

rustup run nightly cargo build --release   --features plugin
cp target/release/*.so lv2-package
rm -rf ~/.lv2/effect/

mkdir -p ~/.lv2/effect/
cp -r lv2-package/* ~/.lv2/effect/
lv2bm -n 30000  -i sweep https://github.com/Chris-Nicholls/lv2-rs