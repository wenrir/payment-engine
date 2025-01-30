FROM rust:slim-bookworm as base
ARG MODULE
ARG OPTIONAL_DEPS
ARG ADDITIONAL_PACKAGES="curl ca-certificates libssl-dev pkg-config libpq-dev musl-dev"

WORKDIR /${MODULE}

RUN --mount=type=bind,source=./,target=/${MODULE}/,readwrite\
 rm -f /etc/apt/apt.conf.d/docker-clean \
 && apt-get update \
 && rustup target add x86_64-unknown-linux-musl \
 && DEBIAN_FRONTEND=noninteractive\
    apt-get install --no-install-recommends --assume-yes ${ADDITIONAL_PACKAGES} \
 && cargo install --path .