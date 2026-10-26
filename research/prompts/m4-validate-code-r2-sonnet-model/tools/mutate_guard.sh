#!/usr/bin/env bash
# 辩方腿 T6：测试临时目录守卫的判别力。把 rerun.sh 拷出来的 tree（--tree）再拷一份到 --work，
# 逐个弄坏守卫 Drop 里的一个分支，跑两条新用例，必须红。用法（整条经内存包装）：
#   bash scripts/hosts/run-with-memory-cap.sh 8G bash mutate_guard.sh --repo <仓根> --tree <rerun 的 tree> --work <新目录>
set -euo pipefail
REPO_ROOT="" TREE="" WORK=""
while (($#)); do
  case "$1" in
    --repo) REPO_ROOT="$2" ;; --tree) TREE="$2" ;; --work) WORK="$2" ;;
    *) echo "认不出参数 $1" >&2; exit 2 ;;
  esac
  shift 2
done
[[ -d "$REPO_ROOT/scripts/hosts" && -d "$TREE/crates" && -n "$WORK" ]] || { echo "用法错" >&2; exit 2; }
[[ ! -e "$WORK" || -z "$(ls -A "$WORK")" ]] || { echo "--work 非空" >&2; exit 2; }
mkdir -p "$WORK"
target="${RERUN_TARGET_DIR:-$WORK/target}"
declare -A mutants=(
  [directory_arm_does_nothing]='Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(&self.0),|Ok(metadata) if metadata.is_dir() => { let _ = metadata; Ok(()) },'
  [file_arm_does_nothing]='Ok(_not_a_directory) => std::fs::remove_file(&self.0),|Ok(_not_a_directory) => Ok(()),'
)
for name in directory_arm_does_nothing file_arm_does_nothing; do
  rsync -a --exclude target "$TREE/" "$WORK/$name/"
  find "$WORK/$name/crates" -type f -exec touch {} +   # 单元哈希不含绝对路径，不碰新会拿到别的树编的二进制
  python3 - "$WORK/$name/crates/singlefs-core/src/block_device.rs" "${mutants[$name]}" <<'PY'
import sys
path, pair = sys.argv[1], sys.argv[2]
old, new = pair.split("|", 1)
text = open(path, encoding="utf-8").read()
assert text.count(old) == 1, "旧串恰好一次"
open(path, "w", encoding="utf-8").write(text.replace(old, new))
PY
  cd "$WORK/$name"
  SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-core 的 lib 测试目标按名字过滤 block_device::tests::a_temporary（弄坏守卫 $name）; 范围=1 个测试目标; 为什么=辩方腿核测试临时目录守卫两条新用例有判别力" \
    CARGO_TARGET_DIR="$target" bash "$REPO_ROOT/scripts/hosts/capped.sh" 31 \
    cargo test --offline -p singlefs-core --lib block_device::tests::a_temporary -- --test-threads=2 \
    > "$WORK/mutant_$name.txt" 2>&1 && rc=0 || rc=$?
  echo "name=mutant mutant=$name rc=$rc $(grep -h '^test result' "$WORK/mutant_$name.txt" | head -1)"
  grep -h '^test .* \(ok\|FAILED\)$' "$WORK/mutant_$name.txt" | sed "s/^/name=mutant_case mutant=$name /"
done
