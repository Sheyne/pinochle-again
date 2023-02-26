####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=pinochle
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /pinochle

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM node:latest as frontend_builder

WORKDIR /pinochle
COPY ./www/ .

RUN npm run-script build

####################################################################################################
## Final image
####################################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /pinochle

COPY --from=frontend_builder /pinochle/build /pinochle/www/build

# Copy our build
COPY --from=builder /pinochle/target/x86_64-unknown-linux-musl/release/pinochle-again ./

# Use an unprivileged user.
USER pinochle:pinochle

CMD ["/pinochle/pinochle-again"]