#!/usr/bin/env bash
# 红样本现场：crate-sources-reachable 那一格——同一份平铺的 crate 里多一份既没登记、也没被谁引的 forgotten.rs（cargo 不编它）。
set -e
mkdir -p scripts/experiment-apparatus
printf '[package]\nname = "sample"\nversion = "0.1.0"\n\n[[bin]]\nname = "first"\npath = "first.rs"\n' > scripts/experiment-apparatus/Cargo.toml
printf 'fn main() {}\n' > scripts/experiment-apparatus/first.rs
printf 'fn main() {}\n' > scripts/experiment-apparatus/forgotten.rs
git init -q . && git add -A
