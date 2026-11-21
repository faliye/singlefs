#!/usr/bin/env bash
# 复用三问的闭包外改动：构建环境与依赖版本换了值（.cargo/config.toml 的 rustflags、Cargo.lock 里 tiny 的版本），这两样不在这一格的登记输入里，两组的键都不变（要复用）
set -euo pipefail
printf '[build]\nrustflags = ["-Cdebuginfo=1"]\n' > .cargo/config.toml
printf 'version = 4\n\n[[package]]\nname = "tiny"\nversion = "0.1.1"\n' > Cargo.lock
