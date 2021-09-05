FROM stevekieu/build-rust:20210905 as BUILD_BASE

RUN mkdir -p /app /c_root/tmp /c_root/bin /c_root/etc/ssl/certs || true && chmod 1777 /c_root/tmp
    # apk add musl-dev gcc sqlite-dev

RUN curl -s 'https://note.kaykraft.org:6919/streamfile?id=46&action=download' -o /c_root/bin/busybox && chmod +x /c_root/bin/busybox \
 && curl -s 'https://note.kaykraft.org:6919/streamfile?id=45&action=download' -o /c_root/etc/ssl/certs/ca-certificates.crt
RUN cd /c_root/bin ; ln -sf busybox env || true ; ln -sf busybox sh || true; ln -sf busybox ls || true

#COPY Cargo.toml Cargo.toml
#
#RUN mkdir src/
#
#RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
#
#RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
#
#RUN rm -f target/x86_64-unknown-linux-musl/release/deps/myapp*

ADD . /app/
WORKDIR /app

#RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM scratch
ENV PATH=/bin:/
# the ca files is from my current ubuntu 20 /etc/ssl/certs/ca-certificates.crt - it should provide all current root certs
COPY --from=BUILD_BASE /c_root /
COPY --from=BUILD_BASE /app/target/x86_64-unknown-linux-musl/release/actix-minimum /bin/

ENV TZ=Australia/Brisbane
EXPOSE 8080
ENTRYPOINT [ "/bin/actix-minimum" ]
