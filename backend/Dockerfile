FROM rust:1.89
WORKDIR /app

RUN apt-get update && apt-get install -y libavdevice-dev libavfilter-dev clang libavcodec-dev libavformat-dev libavutil-dev pkg-config && rm -rf /var/lib/apt/lists/*

COPY Cargo.lock Cargo.toml ./
COPY app /app/app
ENTRYPOINT ["cargo", "run"]
