FROM rust:latest as rust-build
RUN apt update && apt -y install lld musl-tools
COPY src/ ./src/
COPY Cargo.toml ./
COPY db.sqlite3 ./
COPY log.yml ./
COPY tmp ./
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo test --workspace --target x86_64-unknown-linux-musl && \
    cargo build --workspace --target x86_64-unknown-linux-musl --release

FROM gcr.io/distroless/static-debian12:latest
ENV TELOXIDE_TOKEN=0000000
ENV ROOT_DIR=/1С/1C_IS/
ENV ADMIN_ID=00000000
ENV ACCESS_KEY=123
ENV YAUTH=000000
ENV DB_PATH=/usr/local/bin/db.sqlite3
ENV BOT_NAME=Bot
ENV LOG_PATH=/usr/local/bin/log.yml
ENV TMP_DIR=/usr/local/bin/tmp
COPY --from=rust-build /tmp /usr/local/bin/tmp
COPY --from=rust-build /log.yml /usr/local/bin/log.yml
COPY --from=rust-build /db.sqlite3 /usr/local/bin/db.sqlite3
COPY --from=rust-build /target/x86_64-unknown-linux-musl/release/yandex-cloud-bot /usr/local/bin/yandex-cloud-bot
ENTRYPOINT ["/usr/local/bin/yandex-cloud-bot"]
CMD ["yandex-cloud-bot"]