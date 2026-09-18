#!/usr/bin/env bash
# m2-agentdef-r1 Opus 攻方腿：implementation-writer 第 3 步删掉的前提（`rsync -a` 带回旧修改时间，cargo 不重编）
# 在「同一份副本、自己的 target、用 rsync -a 从原件还原」这条路上还成不成立。玩具 crate，不碰仓里任何代码。
# 用法：bash mtime_toy.sh <工作目录>     （工作目录会被清空重建）
set -euo pipefail
work="${1:?用法：bash mtime_toy.sh <工作目录>}"
rm -rf "$work"; mkdir -p "$work/pristine/src" "$work/pristine/tests"
cd "$work/pristine"
printf '[package]\nname = "mtime_toy"\nversion = "0.1.0"\nedition = "2021"\n' > Cargo.toml
printf 'pub fn value() -> u32 { 1 }\n' > src/lib.rs
printf '#[test]\nfn value_is_one() { assert_eq!(mtime_toy::value(), 1); }\n' > tests/pin.rs
touch -d '2026-09-01 00:00:00' Cargo.toml src/lib.rs tests/pin.rs      # 原件的修改时间早于任何一次编译

verdict() {   # 在副本里跑那条测试，只取判定：pass / FAILED
  local copy="$1"
  if ( cd "$copy" && CARGO_TARGET_DIR="$copy/target" nice -n 19 cargo test --quiet --offline >"$copy.log" 2>&1 ); then
    echo pass
  else
    grep -q 'value_is_one --- FAILED\|test result: FAILED' "$copy.log" && echo FAILED || echo "error(see $copy.log)"
  fi
}

echo "path A：同一份副本、自己的 target，改坏后用 rsync -a 从原件还原"
rsync -a --exclude target "$work/pristine/" "$work/copy-a/"
echo "  A1 基线（未改）            → $(verdict "$work/copy-a")"
sed -i 's/{ 1 }/{ 2 }/' "$work/copy-a/src/lib.rs"
echo "  A2 改坏一处（sed -i）       → $(verdict "$work/copy-a")"
rsync -a --exclude target "$work/pristine/" "$work/copy-a/"
cmp -s "$work/pristine/src/lib.rs" "$work/copy-a/src/lib.rs" && same=same || same=differs
echo "  A3 rsync -a 还原：内容与原件 $same，mtime $(stat -c %y "$work/copy-a/src/lib.rs" | cut -c1-19) → $(verdict "$work/copy-a")"

echo "path B：同一份副本、自己的 target，改坏后原地改回（Edit 的效果：新修改时间）"
rsync -a --exclude target "$work/pristine/" "$work/copy-b/"
echo "  B1 基线（未改）            → $(verdict "$work/copy-b")"
sed -i 's/{ 1 }/{ 2 }/' "$work/copy-b/src/lib.rs"
echo "  B2 改坏一处（sed -i）       → $(verdict "$work/copy-b")"
sed -i 's/{ 2 }/{ 1 }/' "$work/copy-b/src/lib.rs"
echo "  B3 原地改回                 → $(verdict "$work/copy-b")"

echo "path C：改坏后不还原，另起一份新副本（自己的 target）"
rsync -a --exclude target "$work/pristine/" "$work/copy-c/"
echo "  C1 新副本                   → $(verdict "$work/copy-c")"
echo "cargo: $(cargo --version)"
