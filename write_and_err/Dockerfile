FROM archlinux:latest
RUN echo Install packages && \
pacman -Syyu --noconfirm git base-devel rustup vim neovim screen tmux && \
rustup default stable && \
rustup component add clippy && \
rustup component add rustfmt
RUN echo Install repo and prerequisits && \
cargo install --version "0.1.2" cargo-dylint dylint-link && \
git clone https://github.com/gww-parity/substrate_lints.git && \
cd substrate_lints/write_and_err && \
make
