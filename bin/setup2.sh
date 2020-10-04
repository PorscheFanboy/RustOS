
#!/bin/bash

set -e

TOP=$(git rev-parse --show-toplevel)
BIN=$TOP/bin
DEP=$TOP/.dep
VER=nightly-2019-07-01


rustup default $VER
rustup component add rust-src llvm-tools-preview clippy

# install cargo xbuild
mkdir -p $DEP
pushd $DEP
if ! [ -e cargo-xbuild ]; then
  git clone https://github.com/rust-osdev/cargo-xbuild
  pushd cargo-xbuild
  git checkout v0.5.20
  # https://github.com/rust-osdev/cargo-xbuild/pull/75
  git cherry-pick b24c849028eb7da2375288b1b8ab6a7538162bd7
  popd
fi
cargo install -f --path cargo-xbuild --locked
popd

# install cargo binutils
pushd $DEP
if ! [ -e cargo-objcopy ]; then
  git clone https://github.com/man9ourah/cargo-binutils.git
  cargo install -f --path cargo-binutils --locked
fi
popd

echo "[!] Please add '$HOME/.cargo/bin' in your PATH"
