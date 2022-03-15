mkdir -p videos
TSTAMP=$(date "+%Y-%m-%d-%H-%M-%S")
cargo run --release --example $1 && \
    ffmpeg -y -r 60 -i $1/%d.png -c:v libx264 -vf "fps=60,format=yuv420p" "videos/$TSTAMP-$1.mp4" && \
    rm -rf $1
