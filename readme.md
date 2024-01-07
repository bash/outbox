# Outbox 📤
A mail queue daemon for msmtp.

## Installing
TODO

## Building from Source
```
meson _build
ninja -C _build
```

## Using during Development
Use `make run` to run `outboxd` using the session bus.
Instead of sending emails it will open them in the default email program.

