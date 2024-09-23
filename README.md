# `watchdock`
> Trigger a build inside a build container (e.g. using `cargo-watch`), then optionally run another command outside the container when the inner build command succeeds.

Inside your build container (with --volume ./target/watchdock:/run/watchdock):
```shell
cd /src
watchdock listen /run/watchdock/sock cargo build
```

Then outside your container:
```shell
cargo watch -s "watchdock trigger-run ./target/watchdock/sock echo 'DONE - now do my useful thing'"
```
