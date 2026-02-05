# How to contribute

## Environment Setup

### Rust

#### Nix

If you use [nix-direnv](https://github.com/nix-community/nix-direnv), you can use `direnv allow` to get a ready-to-use development environment.

```bash
direnv allow
```

This includes the Rust toolchain and some cli tools, but does not include the database.

#### Manual

Refer to the Rust website to install: https://rust-lang.org/learn/get-started/

Then install the cli tools using cargo:

```bash
cargo install --locked bacon
cargo install sea-orm-cli@^2.0.0-rc
```

### Database

Here, I only recommend using containers, with Docker as the reference.

```bash
docker compose up -d
```

> TODO
