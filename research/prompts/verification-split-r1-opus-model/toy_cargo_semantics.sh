#!/usr/bin/env bash
# Q1 / Q4：在一个与仓无关的玩具工作区（toy-core、toy-harness、toy-checker）上量 cargo 的真行为：
# -p 通配、-p 名@版本、-p pkgid URL、别名（--config 与 CARGO_ALIAS_）、cargo bench --test；dev-dependency 进不进库单测与 doctest、#[path] 抄实现源码进库。
set -uo pipefail
REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
T="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/toyws}"
mkdir -p "$T" && cd "$T" && mkdir -p crates/toy-core/src crates/toy-harness/src crates/toy-checker/src crates/toy-checker/tests
printf '[workspace]\nresolver = "2"\nmembers = ["crates/toy-core", "crates/toy-harness", "crates/toy-checker"]\n' > Cargo.toml
for p in core harness; do printf '[package]\nname = "toy-%s"\nversion = "0.1.0"\nedition = "2021"\n' $p > crates/toy-$p/Cargo.toml; done
echo 'pub fn f() -> u32 { 7 }' > crates/toy-core/src/lib.rs; echo 'pub fn f() {}' > crates/toy-harness/src/lib.rs
printf '[package]\nname = "toy-checker"\nversion = "0.1.0"\nedition = "2021"\n\n[dev-dependencies]\nimplementation = { package = "toy-core", path = "../toy-core" }\n' > crates/toy-checker/Cargo.toml
printf '#[test]\nfn integration_test_in_checker_ran() { println!("MARKER-CHECKER-INTEGRATION-RAN"); }\n' > crates/toy-checker/tests/big_stream.rs
cat > crates/toy-checker/src/lib.rs <<'RS'
/// ```
/// assert_eq!(implementation::f(), 7);
/// ```
pub fn judge() {}

#[path = "../../toy-core/src/lib.rs"]
mod copied_from_the_implementation;
pub fn uses_copied() -> u32 { copied_from_the_implementation::f() }

#[cfg(test)]
mod unit {
    #[test]
    fn lib_unit_test_links_the_implementation() { assert_eq!(implementation::f(), 7); println!("MARKER-LIB-UNIT-LINKED-CORE"); }
}
RS
export CARGO_TARGET_DIR="$T/target"
run() { echo "== $*"; nice -n 19 bash "$REPOSITORY/research/scripts/run-with-memory-cap.sh" 2G bash "$REPOSITORY/research/scripts/capped.sh" 16 "$@" 2>&1 \
  | grep -E 'MARKER|error|test result|Doc-tests|Running'; echo "rc=${PIPESTATUS[0]}"; }
run cargo test --offline -p 'toy-check*' -- --nocapture
run cargo test --offline -p toy-checker@0.1.0 -- --nocapture
run cargo test --offline -p "path+file://$T/crates/toy-checker#toy-checker@0.1.0" -- --nocapture
( cd crates/toy-harness && run cargo --config 'alias.xt="test -p toy-checker"' xt --offline -- --nocapture )
( cd crates/toy-harness && CARGO_ALIAS_XT='test -p toy-checker' run cargo xt --offline -- --nocapture )
run cargo bench --offline -p toy-checker --test big_stream -- --nocapture
echo "== cargo build -p toy-checker"; nice -n 19 bash "$REPOSITORY/research/scripts/capped.sh" 16 cargo build --offline -p toy-checker 2>&1 | tail -1
run cargo test --offline -p toy-checker --lib -- --nocapture
run cargo test --offline -p toy-checker --doc
