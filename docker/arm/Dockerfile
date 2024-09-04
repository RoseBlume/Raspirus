FROM rust:1.79.0-bookworm

WORKDIR /usr/app/raspirus
COPY . .

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get clean && apt-get update && apt-get upgrade -y
RUN apt-get update && apt-get install -y build-essential \
                                           curl \
                                           wget \
                                           file \
                                           libxdo-dev \
                                           libayatana-appindicator3-dev \
                                           librsvg2-dev \
                                           gcc-aarch64-linux-gnu \
                                           libssl-dev \
                                           libwebkit2gtk-4.1:arm64


# Add architecture
RUN dpkg --add-architecture arm64
RUN apt-get -qq update
RUN apt-get -qq install -y libwebkit2gtk-4.1:arm64 libssl:arm64

# Add Rust target
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown

# Create the dist directory
RUN mkdir dist

# Install app deps
RUN cargo install trunk
RUN cargo install --path src-tauri/
RUN cargo install tauri-cli@^2.0.0-beta --locked

# Set environment variables
ENV PKG_CONFIG_SYSROOT_DIR=/usr/aarch64-linux-gnu/

# Build app
RUN cargo tauri build --target aarch64-unknown-linux-gnu -b deb
