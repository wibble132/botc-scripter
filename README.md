
Simple tool to prepare datapack data for our Minecraft BOTC server's scripts.

# Running

### Cargo

If you have a standard Rust installation, just use cargo

```shell
# Show the command line options
cargo run --release -- --help

# Simple usage (downloads a script and creates the functions for this script in ./out)
cargo run --release -- tb
```

### Nix

This is also packaged with a Nix flake package and dev environment. If you don't know what this means, it's probably not for you (yet).

The default nix package is for a statically linked executable.

# Extra

See https://github.com/ThePandemoniumInstitute/botc-release for some cool useful stuff like json schema and character data. Notably the `resources/data/` folder has roles and their descriptions, all the jinxes, and the all the night orders.
