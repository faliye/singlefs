#!/usr/bin/env bash
# 核「54 号快档红 = C577 之后钉值没改」：三份副本上录第一条流（内存稀疏盘，不枚举），打段长与闭式数；再跑 harness 档里钉这条流段序列的四个测试目标。
#   A = 快照的 crates；B = A 只把 crates/singlefs-core/src/transaction.rs:1208 的 C577 屏障换成 Ok(())；E = 45d49aaf（[26, 3, 2] 钉进去的那次提交）的 crates。
# 三份各用自己的 CARGO_TARGET_DIR（共用一份时 cargo 按工作区相对路径算元数据哈希，B 会拿到 A 的产物）。
set -uo pipefail
REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
S="${1:-/tmp/claude-1000/three-way-attack/verification-split-r1/snap}"
W="${2:-/tmp/claude-1000/three-way-attack/verification-split-r1/c577-rerun}"
mkdir -p "$W/tmp-images"
for variant in A B; do rm -rf "${W:?}/$variant"; mkdir -p "$W/$variant"; cp "$S/Cargo.toml" "$S/Cargo.lock" "$W/$variant/"; rsync -a --exclude target "$S/crates" "$W/$variant/"; done
python3 - "$W/B/crates/singlefs-core/src/transaction.rs" <<'PY'
import sys
path = sys.argv[1]; lines = open(path, encoding="utf-8").read().split("\n")
assert lines[1207].strip() == "writer.perform(CommitStep::Barrier)", lines[1207]
lines[1207] = "    Ok(()) // attack-leg variant B: C577 end-of-publish barrier removed"
open(path, "w", encoding="utf-8").write("\n".join(lines))
PY
rm -rf "${W:?}/E"; mkdir -p "$W/E"; git -C "$REPOSITORY" archive 45d49aaf Cargo.toml Cargo.lock crates | tar -x -C "$W/E"
for variant in A B E; do
  cp "$MODEL/zz_attack_first_stream_segment_counts.rs" "$W/$variant/crates/singlefs-harness/tests/"
  (cd "$W/$variant" && TMPDIR="$W/tmp-images" CARGO_TARGET_DIR="$W/target-$variant" nice -n 19 bash "$REPOSITORY/research/scripts/run-with-memory-cap.sh" 8G \
     bash "$REPOSITORY/research/scripts/capped.sh" 16 cargo test --offline -p singlefs-harness --test zz_attack_first_stream_segment_counts -- --nocapture) > "$W/prototype-$variant.log" 2>&1
  echo "== $variant prototype rc=$?"; grep -E 'ATTACK_FIRST_STREAM' "$W/prototype-$variant.log"
done
for variant in A B; do
  (cd "$W/$variant" && TMPDIR="$W/tmp-images" CARGO_TARGET_DIR="$W/target-$variant" nice -n 19 bash "$REPOSITORY/research/scripts/run-with-memory-cap.sh" 8G \
     bash "$REPOSITORY/research/scripts/capped.sh" 16 cargo test --offline --no-fail-fast -p singlefs-harness --test crash_segments_per_device_and_torn_in_place_overwrites \
     --test crash_enumeration_sharded_across_processes --test first_transaction_step_five_publish --test in_place_overwrite_torn_state_count) > "$W/harness-$variant.log" 2>&1
  echo "== $variant harness rc=$?"; grep -E '^     Running|^test result|\.\.\. FAILED$' "$W/harness-$variant.log" | sed -E 's|\(/[^)]*\)||'
done
