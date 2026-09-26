#!/usr/bin/env bash
# 用法：bash g2-cargo-discovery.sh <仓根> <草稿目录>
# 把 crates/ Cargo.toml Cargo.lock 拷进草稿目录（不带 target），在副本里放两个「别的会话还没 git add 的」文件，
# 用 cargo metadata（不编译）看 cargo 认不认它们为目标：认了，59 号拷进每一片的 crates/ 里就有它们，`-p singlefs-harness` 的每一行都要把 bin 编进去。
set -uo pipefail
root="$1"; draft="$2"; copy="$draft/g2-cargo-copy"
rm -rf "$copy"; mkdir -p "$copy"
( cd "$root" && tar --exclude=target -cf - crates Cargo.toml Cargo.lock ) | tar -xf - -C "$copy"
echo '#[test] fn wip() { panic!() }' > "$copy/crates/singlefs-harness/tests/wip_from_other_session.rs"
echo 'fn main() { this_does_not_compile }' > "$copy/crates/singlefs-harness/src/bin/e999_other_session_wip.rs"
( cd "$copy" && cargo metadata --no-deps --offline --format-version 1 2>/dev/null ) | python3 -c '
import json, sys
for package in json.load(sys.stdin)["packages"]:
    for target in package["targets"]:
        if "wip" in target["name"]:
            print(package["name"], target["kind"], target["name"])'
printf 'crates/mutations.tsv 里 cargo 参数带 -p singlefs-harness 的行：%s（非注释非空行共 %s）\n' \
  "$(cut -f5 "$root/crates/mutations.tsv" | grep -v '^#' | grep -c -- '-p singlefs-harness')" \
  "$(grep -v '^#' "$root/crates/mutations.tsv" | grep -vc '^$')"
