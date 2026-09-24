#!/usr/bin/env bash
# c355-c363-r3 攻方腿（W2）复跑：计数模型（只用 std）+ 仓副本探针（副本上的数，不是入库装置上的数）。
#   bash run.sh [源仓目录] [草稿目录] [输出目录]
# 默认：源仓 = 本脚本往上三层的仓根；草稿 = /tmp/claude-1000/c355-r3-opus/rerun；输出 = 本目录的 results/。
# 源仓只读：rsync 进草稿目录的三份副本（base、v1、v2），v1 / v2 各打一个补丁，探针测试拷进 crates/singlefs-harness/tests/。
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SRC="${1:-$(cd "$HERE/../../.." && pwd)}"
DRAFT="${2:-/tmp/claude-1000/c355-r3-opus/rerun}"
OUT="${3:-$HERE/results}"
mkdir -p "$DRAFT" "$OUT"

# 一、计数模型
rustc -O --edition 2021 -o "$DRAFT/w2_count" "$HERE/w2_count.rs"
nice -n 19 "$DRAFT/w2_count" > "$OUT/w2_count.out"
for mutation in --mutate-ignore-device-cover --mutate-split-pays-no-fixed-point; do
  set +e
  nice -n 19 "$DRAFT/w2_count" "$mutation" > "$OUT/w2_count${mutation}.out" 2>&1
  status=$?
  set -e
  echo "w2_count $mutation exit=$status" >> "$OUT/w2_count-mutations.txt"
  if [ "$status" -eq 0 ]; then echo "变异 $mutation 没有让断言红" >&2; exit 1; fi
done

# 二、仓副本探针
for arm in base v1 v2; do
  rsync -a --delete --exclude target --exclude .git --exclude research "$SRC/" "$DRAFT/$arm/"
  cp "$HERE/copy-probe/w2_probe.rs" "$DRAFT/$arm/crates/singlefs-harness/tests/w2_probe.rs"
done
(cd "$DRAFT/base" && find crates Cargo.toml Cargo.lock -type f ! -path 'crates/singlefs-harness/tests/w2_probe.rs' -print0 | sort -z | xargs -0 sha256sum) > "$OUT/copy-manifest.txt"
(cd "$DRAFT/v1" && patch -p1 < "$HERE/copy-probe/v1-release-holds-on-failure.patch")
(cd "$DRAFT/v2" && patch -p1 < "$HERE/copy-probe/v2-reclaim-old-floor-immediately.patch")
for arm in base v1 v2; do
  (cd "$DRAFT/$arm" && nice -n 19 cargo build --release -p singlefs-harness --tests)
done
run() { # arm test env out
  (cd "$DRAFT/$1" && env $3 nice -n 19 cargo test --release -p singlefs-harness --test w2_probe "$2" -- --exact --nocapture) > "$OUT/$4" 2>&1
}
run base w2_probe W2_P2=1 w2_probe_p1p2.base.out
run base w2_v1_probe W2_V1=base w2_v1.base.out
run v1 w2_v1_probe W2_V1=v1 w2_v1.v1.out
run v2 w2_probe W2_NONE=1 w2_probe_p1.v2.out
run v2 w2_v1_probe W2_V1=v2 w2_v1.v2.out

# 三、汇总（报告里引的那几行由这里算）
{
  echo "## P1（base）：抬 F 被拒且原因是每块盘上都没有，按这一步写了几次分组"
  grep '^P1' "$OUT/w2_probe_p1p2.base.out" | awk -F'\t' '$7 ~ /NoFreeSlot/ {w[$8]++} END{for(k in w) print "raise_writes="k, w[k]}' | sort
  echo "## P1（base）总段数 / 抬 F 做成"
  grep -c '^P1' "$OUT/w2_probe_p1p2.base.out"
  grep '^P1' "$OUT/w2_probe_p1p2.base.out" | awk -F'\t' '$7 ~ /^RaisedFloor/' | wc -l
  echo "## P2（base）"
  grep '^# P2 summary' "$OUT/w2_probe_p1p2.base.out"
  grep '^P2' "$OUT/w2_probe_p1p2.base.out" | awk -F'\t' '{n=split($5,a,"+"); inproc=1; for(i=1;i<=n;i++) if(a[i]!~/^(Overwrite|Raise)/) inproc=0; if(inproc){t++; if($8=="true") e++}} END{print "in-process suffixes:", t, "escaped:", e+0}'
  grep '^P2' "$OUT/w2_probe_p1p2.base.out" | awk -F'\t' '$8=="true"{n=split($5,a,"+"); m=0; for(i=1;i<=n;i++) if(a[i]~/^(MountWritable|Rollback)/) m=1; if(m) c++; else o++} END{print "escaped with a mount in the suffix:", c+0, "escaped without:", o+0}'
  echo "## V1 对拍：各臂结局"
  for arm in base v1 v2; do
    printf '%s ' "$arm"; grep '^V1' "$OUT/w2_v1.$arm.out" | awk -F'\t' '{print ($NF ~ /^Completed/)?"Completed":"NewFinding"}' | sort | uniq -c | tr '\n' ' '; echo
  done
  echo "## V1（v1 臂）判红的段按抬 F 那一步的结局分组"
  grep '^V1' "$OUT/w2_v1.v1.out" | awk -F'\t' '{split($7,o," "); k=o[1]; red=($NF ~ /^Completed/)?0:1; t[k]++; r[k]+=red} END{for(k in t) print k, t[k], r[k]}' | sort
  echo "## V2：P1 里抬 F 的结局"
  grep '^P1' "$OUT/w2_probe_p1.v2.out" | awk -F'\t' '{print $7}' | sed 's/(F=[0-9]*,/(F=*,/' | sort | uniq -c
} > "$OUT/summary.txt"
cat "$OUT/summary.txt"
