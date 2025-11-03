# List available recipes
default:
  @just --list

# Run cargo doc and open result in browser
[group('build')]
doc:
  cargo doc --all-features --no-deps --open

# Run cargo build
[group('build')]
build:
  cargo build --verbose

# Run cargo clean
[group('build')]
clean:
  cargo clean --verbose

# Install cargo tools used in package maintenance
[group('build')]
install_dev_tools:
  echo "Installing optional tools for depelopment."
  cargo install --locked release-plz
  cargo install --locked cargo-audit
  cargo install --locked cargo-outdated
  cargo install --locked cargo-llvm-cov
  cargo install --locked cargo-expand

# Format source code with cargo fmt
[group('lint')]
fmt:
  cargo fmt --all

# Lint source code CI linter
[group('lint')]
lint:
  cargo check
  cargo clippy --all -- -D warnings

# Lint source code with strict linter
[group('lint')]
pedantic:
  cargo clippy -- -W clippy::pedantic

# Run cargo audit to vet dependencies
[group('lint')]
audit:
  cargo audit
# Run cargo audit to vet dependencies
[group('lint')]
outdated:
  cargo outdated

# Run example that parses hard codded buffer and print byte slices that contain valid frames.
[group('examples')]
example_raw_frame:
  cargo run --example=simple_raw_frame

# Run example that parses hard codded buffer and print parsed frames.
[group('examples')]
example_frame:
  cargo run --example=simple_frame


set positional-arguments
# Run tests for all features
[group('test')]
test args='':
  cargo test --all-features $1 -- --show-output

# Run llvm-cov code coverage tool and open report in browser
[group('test')]
cov:
  cargo llvm-cov --open

# Run same testing commands as on CI server
[group('test')]
ci:
  cargo clippy --all -- -D warnings
  cargo build --verbose
  cargo test --all-features --verbose
  cargo test --examples
