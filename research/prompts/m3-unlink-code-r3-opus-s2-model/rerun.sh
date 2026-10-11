#!/usr/bin/env bash
# m3-unlink-code-r3 攻方第二次抽样（s2）的复跑脚本。分段跑（每段前台几分钟之内），同一个工作目录与产物目录依次跑完全部段：
#   bash rerun.sh build <工作目录>                 拷冻结副本三份（c1 原样、held 改回第 2259 行、perdisk 攻方自提的按盘扣，零轮）、加原型、各编一份
#   bash rerun.sh scan  <工作目录> <产物目录>       N1 静态扫（三族 × 两臂，不枚举）
#   bash rerun.sh enum  <工作目录> <产物目录> <x|a|b|c> [臂]   N1 层 0 全量枚举一段历史（不给臂两臂都跑；c 那一段两臂合起来近 4 分钟，分两次跑）
#   bash rerun.sh n2    <工作目录> <产物目录>       N2 注错点放开扫（五个起点，c1 臂）
#   bash rerun.sh compare <产物目录>               与本目录 out/ 里的产物比判定行（去掉耗时与进度行）
# 冻结副本：/tmp/claude-1000/m3-unlink-code-r3/tree/（只读，拷出来改）。编译与跑都经 scripts/hosts/run-with-memory-cap.sh。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
cap="$repo/scripts/hosts/run-with-memory-cap.sh"
capped="$repo/scripts/hosts/capped.sh"
frozen=/tmp/claude-1000/m3-unlink-code-r3/tree
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="原型每次改过重编，峰值表里的旧读数不代表这一版"
export RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-200}"
stage="${1:?段名}"
case "$stage" in
  build)
    work="${2:?工作目录}"
    for arm in c1 held perdisk; do
      mkdir -p "$work/$arm"
      rsync -a --delete --exclude target --exclude .git "$frozen/" "$work/$arm/"
      python3 "$here/patch-tree.py" "$work/$arm" "$arm"
      rsync -a "$here/probe-s2/" "$work/$arm/crates/probe-s2/"
      (cd "$work/$arm" && PROBE_ARM="$arm" CARGO_TARGET_DIR="$work/target-$arm" \
        nice -n 19 bash "$cap" 8G bash "$capped" 15 cargo build --release -p probe-s2)
    done
    ;;
  scan)
    work="${2:?工作目录}"; out="${3:?产物目录}"; mkdir -p "$out"
    for arm in c1 held perdisk; do
      bin="$work/target-$arm/release/probe-s2"
      nice -n 19 bash "$cap" 2G "$bin" n1-scan 0 192,256,320,384 0-8 2,3,5,8,14,20 0-4 > "$out/n1-scan-f0-$arm.log" 2>&1
      nice -n 19 bash "$cap" 2G "$bin" n1-scan 2 128,160,192,256 0-30 1,2,3,5 0 > "$out/n1-scan-f2-$arm.log" 2>&1
      nice -n 19 bash "$cap" 2G "$bin" n1-scan 1 240,256,384 0,5,10,20,30,40 1,3,14 0-1 > "$out/n1-scan-f1-$arm.log" 2>&1
    done
    ;;
  enum)
    work="${2:?工作目录}"; out="${3:?产物目录}"; which="${4:?x|a|b|c}"; mkdir -p "$out"
    case "$which" in
      x) shape="0 256 2 14 1"; name="n1-enum-x-0-256-2-14-1" ;;
      a) shape="2 128 4 2 0"; name="n1-enum-a-2-128-4-2-0" ;;
      b) shape="0 256 1 5 1"; name="n1-enum-b-0-256-1-5-1" ;;
      c) shape="0 256 3 5 2"; name="n1-enum-c-0-256-3-5-2" ;;
      *) echo "x|a|b|c" >&2; exit 2 ;;
    esac
    for arm in ${5:-c1 held}; do
      # shellcheck disable=SC2086
      nice -n 19 bash "$cap" 3G bash "$capped" 15 "$work/target-$arm/release/probe-s2" n1 $shape > "$out/$name-$arm.log" 2>&1
    done
    ;;
  n2)
    work="${2:?工作目录}"; out="${3:?产物目录}"; mkdir -p "$out"
    for spec in "256 2 14 0" "256 2 14 2" "192 2 14 2" "256 1 5 1" "256 2 14 1"; do
      # shellcheck disable=SC2086
      nice -n 19 bash "$cap" 3G "$work/target-c1/release/probe-s2" n2 $spec > "$out/n2-sweep-${spec// /-}.log" 2>&1
    done
    ;;
  compare)
    out="${2:?产物目录}"; status=0
    for reference in "$here"/out/*.log; do
      name="$(basename "$reference")"
      if ! diff <(grep -E '^(SCAN|CLASS|pre:|states|NO-WRITE|X-CUT|  detail|N2CLASS|MISMATCH|baseline)' "$reference") \
                <(grep -E '^(SCAN|CLASS|pre:|states|NO-WRITE|X-CUT|  detail|N2CLASS|MISMATCH|baseline)' "$out/$name") > /dev/null; then
        echo "✗ $name 判定行不同"; status=1
      else
        echo "✓ $name 判定行相同（$(grep -cE '^(SCAN|CLASS|pre:|states|NO-WRITE|X-CUT|  detail|N2CLASS|MISMATCH|baseline)' "$reference") 行）"
      fi
    done
    exit "$status"
    ;;
  *) echo "段名：build | scan | enum | n2 | compare" >&2; exit 2 ;;
esac
