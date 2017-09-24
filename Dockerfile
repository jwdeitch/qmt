FROM rust:1.5.0

CMD cargo build --release .

EXPOSE 80
CMD "./target/release/qmt"
