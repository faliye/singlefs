#!/usr/bin/env bash
# m3-batch27-code-r1 攻方（Opus）模型的复跑。
# 用法：
#   SINGLEFS_REPO=<被判的仓树（冻结副本）> SINGLEFS_TOOLS=<带 scripts/hosts/run-with-memory-cap.sh 与 capped.sh 的仓根> \
#     bash rerun.sh <空的工作目录> all        # 拷两份仓副本（new = 被判的样子；old = walk.rs 退回第 27 批之前），追加原型，编、跑，写 <工作目录>/summary.txt
#   bash rerun.sh compare <summary.txt>      # 与本目录的 expected-summary.txt 比（两边都先排序），不读 SINGLEFS_REPO
# 环境变量：SINGLEFS_REPO（仓树）、SINGLEFS_TOOLS（包装脚本所在的仓根；包装脚本另读的变量在它自己文件头）、
#   OPUS_B27_THREADS（原型里判的线程数，默认 12，不超过派发的线程上限 15）；OPUS_B27_VERSIONS（j3 / j3types 跑哪几份，默认 "new old"）。
#   前台一条命令有时限时分步跑：build → j3（OPUS_B27_VERSIONS=new、=old、=fix 各一次）→ j3types → fixcheck → j1，各步把行追加进 summary.txt；all 一条跑完全部。
#   三份副本：new = 被判的样子；old = walk.rs 退回第 27 批之前（walk-rs.patch 反打）；fix = new 上打攻方 F-α（walk-fix-alpha.patch，零轮）。
# 包装脚本另读的变量：同一会话另有一条经它在跑时它退 252，照它的出路设 RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由>（本脚本不替调用方设）。
# 编译经内存包装 6G、跑经 4G（派发提示定的，显式写，不写 auto）；两份副本各用自己的编译目录（同一编译目录会让 cargo 把两份当同一份）。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ "${1:-}" == compare ]]; then
  [[ -f "${2:-}" ]] || { echo "用法：rerun.sh compare <summary.txt>" >&2; exit 2; }
  if diff <(LC_ALL=C sort "$here/expected-summary.txt") <(LC_ALL=C sort "$2") > /dev/null; then
    echo "SUMMARY_MATCHES $(wc -l < "$2") 行"
    exit 0
  fi
  echo "SUMMARY_DIFFERS" >&2
  diff <(LC_ALL=C sort "$here/expected-summary.txt") <(LC_ALL=C sort "$2") | head -n 40 >&2
  exit 1
fi
work="${1:?工作目录}"
what="${2:-all}"
: "${SINGLEFS_REPO:?设 SINGLEFS_REPO}"
: "${SINGLEFS_TOOLS:?设 SINGLEFS_TOOLS}"
threads="${OPUS_B27_THREADS:-12}"
cap="$SINGLEFS_TOOLS/scripts/hosts/run-with-memory-cap.sh"
capped="$SINGLEFS_TOOLS/scripts/hosts/capped.sh"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示规定上限显式写（编译 6G、跑 4G），不写 auto"
mkdir -p "$work"
prepare() {
  for v in new old fix; do
    if [[ ! -d "$work/$v" ]]; then
      mkdir -p "$work/$v"
      rsync -a --exclude target --exclude .git "$SINGLEFS_REPO/" "$work/$v/"
      if [[ $v == old ]]; then
        (cd "$work/old" && patch -R -p1 --quiet < "$here/walk-rs.patch")
      fi
      if [[ $v == fix ]]; then # 攻方 F-α（零轮）：类型段不认得那一支照判 I-2.1、两份都对不上记走读失败
        (cd "$work/fix" && patch -p1 --quiet < "$here/walk-fix-alpha.patch")
      fi
      cat "$here/attack.rs" >> "$work/$v/crates/singlefs-harness/tests/checker_known_bad_images.rs"
    fi
  done
}
build() {
  for v in new old fix; do
    (cd "$work/$v" && CARGO_TARGET_DIR="$work/target-$v" nice -n 19 bash "$cap" 6G bash "$capped" 15 \
      cargo test --offline -p singlefs-harness --test checker_known_bad_images --no-run) > "$work/build-$v.log" 2>&1
    grep -o 'Executable tests/checker_known_bad_images.rs ([^)]*)' "$work/build-$v.log" | sed 's/.*(\(.*\))/\1/' > "$work/bin-$v.txt"
    [[ -s "$work/bin-$v.txt" ]] || { echo "编译 $v 没交出测试二进制，见 $work/build-$v.log" >&2; exit 1; }
  done
}
run_test() { # $1 版本 $2 测试名 $3 日志；其余是环境变量赋值
  local v="$1" name="$2" log="$3"; shift 3
  (cd "$work/$v/crates/singlefs-harness" && env "$@" OPUS_B27_THREADS="$threads" nice -n 19 bash "$cap" 4G \
    "$(cat "$work/bin-$v.txt")" "$name" --exact --nocapture --test-threads 1) > "$log" 2>&1
}
j3() {
  for v in ${OPUS_B27_VERSIONS:-new old}; do
    run_test "$v" opus_b27_attack::opus_b27_j3_matrix "$work/j3-$v.log"
    grep '^J3	' "$work/j3-$v.log" | sed "s/^/$v	/" >> "$work/summary.txt"
  done
}
j3types() { # 类型段 1、65535 两种不登记的码：只在 B1、B2 上跑（下标 0..504：两份底各 9×7×2×2 = 252 形）
  for v in ${OPUS_B27_VERSIONS:-new old}; do
    run_test "$v" opus_b27_attack::opus_b27_j3_matrix "$work/j3types-$v.log" OPUS_B27_J3_TYPES=1,65535 OPUS_B27_J3_TO=504
    grep '^J3	' "$work/j3types-$v.log" | sed "s/^/$v-types	/" >> "$work/summary.txt"
  done
}
fixcheck() { # F-α 副本上跑第 26、27 批那几条坏镜像（变异行点名的两个筛子）
  (cd "$work/fix/crates/singlefs-harness" && nice -n 19 bash "$cap" 4G "$(cat "$work/bin-fix.txt")" stale_copy_of_the_child child_that_does_not_read_back --test-threads 4) > "$work/fixcheck.log" 2>&1 || true
  grep "^test .* \.\.\. \|^test result" "$work/fixcheck.log" | LC_ALL=C sort | sed "s/^/fix	FIXCHECK	/" | sed "s/ finished in [0-9.]*s//" >> "$work/summary.txt"
}
j1() {
  run_test new opus_b27_attack::opus_b27_j1_chains "$work/j1-new.log"
  grep '^J1	' "$work/j1-new.log" | sed "s/^/new	/" >> "$work/summary.txt"
}
case "$what" in
  all) prepare; build; : > "$work/summary.txt"; j3; OPUS_B27_VERSIONS=fix j3; j3types; fixcheck; j1; echo "summary：$(wc -l < "$work/summary.txt") 行 → $work/summary.txt" ;;
  build) prepare; build; : > "$work/summary.txt" ;;
  j3|j3types|j1|fixcheck) prepare; build; "$what" ;;
  *) echo "不认得的子命令 $what" >&2; exit 2 ;;
esac
