# MacOS
TUN_LOCAL_ADDR=2.1.1.1 TUN_REMOTE_ADDR=2.1.1.2 UDP_LISTEN=0.0.0.0:2111 UDP_DST_ADDR=192.168.1.180:2112 sudo -E cargo test -- --nocapture

# Linux
cargo test --no-run # change below deps/sring-hash...
sudo TUN_LOCAL_ADDR=2.1.1.2 TUN_REMOTE_ADDR=2.1.1.1 UDP_LISTEN=0.0.0.0:2112 UDP_DST_ADDR=192.168.1.183:2111 /home/boris/projects/sring/target/debug/deps/sring-cdf200da8c773ff2

