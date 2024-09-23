FROM rust:bookworm AS build
ARG BUILD_FLAGS=

WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/src/target,sharing=locked cargo build ${BUILD_FLAGS}
RUN --mount=type=cache,target=/src/target,sharing=locked <<EOF
set -e
dir=debug
[[ "${BUILD_FLAGS}" =~ .*--release.* ]] && dir=release
cp /src/target/$dir/watchdock /
EOF

FROM debian:bookworm AS runtime
COPY --from=build /watchdock /usr/local/bin/
COPY --from=build /src/tests/inside.sh /usr/local/bin/

USER 1000
ENTRYPOINT [ "watchdock" ]
CMD [ "listen", "/run/test/sock", "inside.sh", "2", "0" ]
