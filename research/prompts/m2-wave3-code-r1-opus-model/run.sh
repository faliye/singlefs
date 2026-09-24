#!/usr/bin/env bash
# m2-wave3-code-r1 云端攻方腿（Opus）复跑：在仓的副本上跑，不动主工作区。
# 用法：bash run.sh <仓根> <草稿目录>
set -euo pipefail
repo_root=${1:?仓根}
draft=${2:?草稿目录}
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$draft"
# 1) 今天代码的副本（Y1 / Y3 的扫描）
rsync -a --exclude target --exclude .git "$repo_root"/ "$draft"/repo/
cp "$here"/opus_attack_y1.rs "$here"/opus_attack_y3.rs "$draft"/repo/crates/singlefs-harness/tests/
(cd "$draft"/repo && CARGO_TARGET_DIR="$draft"/target CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y1 -- --nocapture --test-threads=2 > "$draft"/y1-run1.log 2>&1)
(cd "$draft"/repo && CARGO_TARGET_DIR="$draft"/target CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y3 -- --nocapture --test-threads=1 y3_crash_inside_overwrite > "$draft"/y3-run1.log 2>&1)
(cd "$draft"/repo && CARGO_TARGET_DIR="$draft"/target CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y3 -- --nocapture --test-threads=1 y3_crash_inside_first_writable > "$draft"/y3-formatted.log 2>&1)
# 2) Y4：分配器变异副本（只在发布 B 那一次把新记的分配代写成 txg + 1）
rsync -a --exclude target --exclude .git "$draft"/repo/ "$draft"/repo-mutY4/
(cd "$draft"/repo-mutY4 && patch -p1 < "$here"/mutY4-allocator.diff && \
  cd crates/singlefs-harness/tests && patch opus_attack_y3.rs < "$here"/mutY4-driver.diff)
(cd "$draft"/repo-mutY4 && OPUS_Y4=1 CARGO_TARGET_DIR="$draft"/target-mutY4 CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y3 -- --nocapture --test-threads=1 y4_ > "$draft"/mutY4-run2.log 2>&1)
# 3) 改法 D（I-7.8 扫描方向按实例表行排除崩溃孤儿）副本
rsync -a --exclude target --exclude .git "$draft"/repo/ "$draft"/repo-fixD/
(cd "$draft"/repo-fixD && patch -p1 < "$here"/fixD-walk.diff)
(cd "$draft"/repo-fixD && CARGO_TARGET_DIR="$draft"/target-fixD CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test checker_known_bad_images --test second_transaction_step_four_rollback -- --test-threads=4 > "$draft"/fixD-tests.log 2>&1)
(cd "$draft"/repo-fixD && CARGO_TARGET_DIR="$draft"/target-fixD CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y1 -- --nocapture --test-threads=1 > "$draft"/fixD-y1.log 2>&1)
# 4) 改法 E（I-3.10 另读「下一次挂载要施加的那一版」）：带 Y4 变异跑一遍，干净副本上跑 Y3 两条扫描看有没有误报
rsync -a --exclude target --exclude .git "$draft"/repo-mutY4/ "$draft"/repo-fixE-mut/
(cd "$draft"/repo-fixE-mut && patch -p1 < "$here"/fixE-walk.diff)
(cd "$draft"/repo-fixE-mut && OPUS_Y4=1 CARGO_TARGET_DIR="$draft"/target-fixE-mut CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y3 -- --nocapture --test-threads=1 y4_ > "$draft"/fixE-mutY4.log 2>&1)
rsync -a --exclude target --exclude .git "$draft"/repo/ "$draft"/repo-fixE/
(cd "$draft"/repo-fixE && patch -p1 < "$here"/fixE-walk.diff)
(cd "$draft"/repo-fixE && CARGO_TARGET_DIR="$draft"/target-fixE CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test opus_attack_y3 -- --nocapture --test-threads=2 y3_ > "$draft"/fixE-y3.log 2>&1)
# 5) 改法 D 之下变异 356（回退行水位不取环 max）点名的用例还红不红
rsync -a --exclude target --exclude .git "$draft"/repo-fixD/ "$draft"/repo-fixD-m356/
(cd "$draft"/repo-fixD-m356 && python3 - <<'PY'
p='crates/singlefs-core/src/mount.rs'
s=open(p).read()
old="        ring.max(version_to_build_on.tree_identifier_watermark)"
assert s.count(old)==1
open(p,'w').write(s.replace(old,"        version_to_build_on.tree_identifier_watermark"))
PY
)
(cd "$draft"/repo-fixD-m356 && CARGO_TARGET_DIR="$draft"/target-fixD-m356 CARGO_BUILD_JOBS=4 nice -n 19 \
  cargo test -p singlefs-harness --test second_transaction_step_four_rollback -- after_rolling_back_to_a_warm_up_root > "$draft"/fixD-m356.log 2>&1) || true
