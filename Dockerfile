FROM rust:1.82-bullseye@sha256:c42c8ca762560c182ba30edda0e0d71a8604040af2672370559d7e854653c66d AS builder

ARG BUILD_PROFILE=reproducible
ENV BUILD_PROFILE=$BUILD_PROFILE

# Extra Cargo flags
ARG RUSTFLAGS="-C target-feature=+crt-static -C link-arg=-Wl,--build-id=none -Clink-arg=-static-libgcc -C metadata='' --remap-path-prefix $(pwd)=."
ENV RUSTFLAGS="$RUSTFLAGS"

# Extra Cargo features
ARG FEATURES=""
ENV FEATURES=$FEATURES

RUN apt-get update && apt-get install -y \
    libclang-dev=1:11.0-51+nmu5 \
    protobuf-compiler=3.12.4-1+deb11u1

# Clone the repository at the specific branch
WORKDIR /app
COPY ./ /app

# Get the latest commit timestamp and set SOURCE_DATE_EPOCH
RUN SOURCE_DATE_EPOCH=$(git log -1 --pretty=%ct) && \
    echo "SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH" >> /etc/environment

# Set environment variables for reproducibility
ENV SOURCE_DATE_EPOCH=$SOURCE_DATE_EPOCH \
    CARGO_INCREMENTAL=0 \
    LC_ALL=C \
    TZ=UTC \
    RUSTFLAGS="${RUSTFLAGS}"

# Build the project with the reproducible settings
RUN . /etc/environment && \
    cargo build --features "${FEATURES}" --profile "${BUILD_PROFILE}" --locked --target x86_64-unknown-linux-gnu

RUN . /etc/environment && mv /app/target/x86_64-unknown-linux-gnu/"${BUILD_PROFILE}"/rbuilder /rbuilder

FROM gcr.io/distroless/cc-debian12:nonroot-6755e21ccd99ddead6edc8106ba03888cbeed41a
COPY --from=builder /rbuilder /rbuilder
ENTRYPOINT [ "/rbuilder" ]
