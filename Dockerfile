FROM debian:trixie-20260112
RUN apt-get update && apt-get upgrade -y && apt-get install -y git curl build-essential

# Installs Rust.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain=1.93.0 -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Installs Gipfelkreuzer.
RUN mkdir /tmp_downloads && \
    git -C /tmp_downloads clone https://github.com/at-robins/Gipfelkreuzer.git && \
    git -C /tmp_downloads/Gipfelkreuzer checkout 175a048
WORKDIR /tmp_downloads/Gipfelkreuzer
RUN cargo build --release && \
    cp /tmp_downloads/Gipfelkreuzer/target/release/Gipfelkreuzer /usr/bin && \
    rm -r /tmp_downloads
WORKDIR /

ENTRYPOINT ["Gipfelkreuzer"]