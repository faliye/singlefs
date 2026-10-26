#!/usr/bin/env bash
# 辩方腿 T6：数核心层 block_device 测试跑完后临时目录里留下几份。调用方自己设 TMPDIR（包装不删调用方的），
# 先在没弄坏的树里跑，再在弄坏守卫的树里跑（mutate_guard.sh 留下的 file_arm_does_nothing），各数 TMPDIR 里剩的。
# 用法（整条经内存包装）：bash scripts/hosts/run-with-memory-cap.sh 8G bash guard_leftovers.sh --repo <仓根> --tree <rerun 的 tree> --mutant <弄坏的树> --work <新目录>
set -euo pipefail
REPO_ROOT="" TREE="" MUTANT="" WORK=""
while (($#)); do
  case "$1" in
    --repo) REPO_ROOT="$2" ;; --tree) TREE="$2" ;; --mutant) MUTANT="$2" ;; --work) WORK="$2" ;;
    *) echo "认不出参数 $1" >&2; exit 2 ;;
  esac
  shift 2
done
[[ -d "$TREE/crates" && -d "$MUTANT/crates" && -n "$WORK" ]] || { echo "用法错" >&2; exit 2; }
[[ ! -e "$WORK" || -z "$(ls -A "$WORK")" ]] || { echo "--work 非空" >&2; exit 2; }
mkdir -p "$WORK"
target="${RERUN_TARGET_DIR:-$WORK/target}"
for which in intact mutant; do
  [[ $which == intact ]] && tree="$TREE" || tree="$MUTANT"
  mkdir -p "$WORK/tmp-$which"
  find "$tree/crates" -type f -exec touch {} +   # 单元哈希不含绝对路径，不碰新会拿到别的树编的二进制
  cd "$tree"
  TMPDIR="$WORK/tmp-$which" \
  SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-core 的 lib 测试目标按名字过滤 block_device::tests（$which）; 范围=1 个测试目标; 为什么=辩方腿数测试跑完后临时目录里留下几份，核 T6 守卫" \
    CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
    cargo test --offline -p singlefs-core --lib block_device::tests -- --test-threads=4 \
    > "$WORK/run-$which.txt" 2>&1 && rc=0 || rc=$?
  echo "name=leftovers tree=$which rc=$rc $(grep -h '^test result' "$WORK/run-$which.txt" | head -1) files_left_in_TMPDIR=$(find "$WORK/tmp-$which" -mindepth 1 | wc -l)"
done
