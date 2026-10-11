#!/bin/bash
# m3-batch10-code-r2 攻方第二次抽样（opus-s2）K2 模型复跑。
# 用法：bash scripts/hosts/run-with-memory-cap.sh 4000M bash rerun.sh <草稿目录> [冻结副本根]
#   草稿目录要不存在或为空；冻结副本根默认 /tmp/claude-1000/m3-batch10-code-r2/tree。
# 做的事：把冻结副本的 Cargo.toml、Cargo.lock、crates/ 拷进草稿目录，打上 mount-rs-attack-arms.patch（ATTACK_RR2_ARM 臂开关），
# 放进原型测试，release 编译，按 臂（rr2 / before / merge）× 盘宽（4g / 384）× 前置覆盖写（1 / 3）跑 12 次 k2_attack_grid，
# 再跑一次 k2_young_ring_rereads；逐份比对每行 name= 与 out/ 里的原件，最后一行报几份不同。
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
W=${1:?草稿目录}
FROZEN=${2:-/tmp/claude-1000/m3-batch10-code-r2/tree}
mkdir -p "$W"
if [ -n "$(ls -A "$W")" ]; then echo "草稿目录不空：$W" >&2; exit 2; fi
mkdir -p "$W/tree" "$W/out"
rsync -a --exclude target --exclude .git "$FROZEN/Cargo.toml" "$FROZEN/Cargo.lock" "$FROZEN/crates" "$W/tree/"
(cd "$W/tree" && patch -p1 < "$HERE/mount-rs-attack-arms.patch")
cp "$HERE/k2_attack_reread_replaces_the_first_pass.rs" "$W/tree/crates/singlefs-harness/tests/"
cd "$W/tree"
CARGO_TARGET_DIR="$W/target" cargo test --offline --release -p singlefs-harness --test k2_attack_reread_replaces_the_first_pass --no-run > "$W/out/build.log" 2>&1
B=$(ls -t "$W"/target/release/deps/k2_attack_reread_replaces_the_first_pass-* | grep -v '\.d$' | head -1)
cd "$W/tree/crates/singlefs-harness"
for width in 4g 384; do
  for before in 1 3; do
    for arm in rr2 before merge; do
      env ATTACK_RR2_ARM=$arm ATTACK_WIDTH=$width ATTACK_OVERWRITES_BEFORE=$before ATTACK_CONTINUE=40 \
        "$B" --nocapture --test-threads=1 > "$W/out/grid-$width-$before-$arm.log" 2>&1
    done
  done
done
"$B" --nocapture --test-threads=1 k2_young_ring_rereads > "$W/out/young.log" 2>&1
differ=0
total=0
for original in "$HERE"/out/grid-*.log "$HERE/out/young.log"; do
  name=$(basename "$original")
  total=$((total + 1))
  if ! diff <(grep -o 'name=.*' "$original") <(grep -o 'name=.*' "$W/out/$name") > /dev/null; then
    echo "不同：$name"
    differ=$((differ + 1))
  fi
done
echo "比了 $total 份，$differ 份不同"
