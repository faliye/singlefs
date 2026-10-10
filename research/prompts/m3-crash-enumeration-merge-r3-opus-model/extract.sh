#!/usr/bin/env bash
# 从仓里的 crash_identity.rs 按行抽出闭包算法，拼成单文件程序 <输出目录>/src/main.rs。行区间对应的那份文件钉 sha256，对不上退 3。
set -euo pipefail
repo="$1"; out="$2"; model="$(cd "$(dirname "$0")" && pwd)"
src="$repo/crates/singlefs-checker-tier/src/crash_identity.rs"
pinned=488f75ff6ffae9c81bd94188ca17df87a371194ef699b042bab8f7e12968d2f0
actual="$(sha256sum "$src" | cut -d' ' -f1)"
[[ "$actual" == "$pinned" ]] || { echo "crash_identity.rs 变了（$actual），行区间要重核" >&2; exit 3; }
mkdir -p "$out/src"
{
  cat "$model/closure_head.rs"
  sed -n '278,875p' "$src"      # lexical_tokens … FILES_THE_JUDGES_MUST_NOT_REACH
  sed -n '1009,1235p' "$src"    # judging_code_of_the_judges … rust_files_under
  cat "$model/closure_main.rs"
} > "$out/src/main.rs"
printf '[package]\nname = "closure-model"\nversion = "0.1.0"\nedition = "2021"\n[workspace]\n' > "$out/Cargo.toml"
