prefix  := env_var_or_default("PREFIX", "/usr/local")
destdir := env_var_or_default("DESTDIR", "")
bindir  := env_var_or_default("BINDIR", prefix + "/bin")
mandir  := env_var_or_default("MANDIR", prefix + "/share/man")

default:
    @just --list

# Check the code
check *args:
    cargo fmt --all -- --check
    cargo clippy --locked --all-targets --features man {{args}}
    cargo test --locked {{args}}

# Build the release binary.
build:
    cargo build --locked --release

# Build and install (honors PREFIX, BINDIR, MANDIR, DESTDIR).
install: build
    install -m755 target/release/i3rustus "{{destdir}}{{bindir}}/i3rustus"

# Remove installed files.
uninstall:
    rm -f "{{destdir}}{{bindir}}/i3rustus"

clean:
    cargo clean
