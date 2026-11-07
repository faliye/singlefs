#!/usr/bin/env bash
# admission: always 每趟重跑三组配置、写新的输出，上一次的输出不替这一次作保
# run-condition: command cargo
# E56 的跑批。三组配置，一次跑完：
#   1. 测量：5 个种子 × 缓存 66 个节点（约占树 6.4%）
#   2. 自检：把两条攒批臂的核心判据换成常量（缓冲=1），1 个种子 —— 结构性检查，不是测量
#   3. 缓存敏感性：缓存 264 / 1057 个节点，1 个种子
#      ⚠️ 第 3 组是两条独立论证腿都指出来的那个混淆：
#      点查那一列在高 ε 上贵起来，是「内部节点数逼近缓存容量」造成的，
#      不是消息缓冲本身让单次查询变贵。不扫缓存就分不开这两件事。
#
#   task-server/task submit -- bash scripts/experiment-apparatus/e56-sweep.sh <输出文件>
#
# 编译：脚本在同一条任务里自己编（cargo build 再逐个配置 cargo run --release），不要求仓里先有预编的二进制；
# 编译目录归任务服务（本机一支脚本里的 cargo 不在命令位置上，是 scripts/experiment-apparatus/ 自成工作区的 target；
# 派到第二台的，CARGO_TARGET_DIR 由 multi-host-run.sh 在第二台本地盘上这一趟新建、跑完删），脚本不设、不猜。
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 每一轮的收尾行必须是 `name=done emitted=N`，且 N 必须等于该轮实际 E7RESULT 行数；
#   3. 任一轮退出码非 0 或条数对不上，整脚本判红，不许当成「跑过了」。
#
# 自证：bash scripts/experiment-apparatus/e56-sweep.sh --selftest
#   把本脚本拷进临时树（.claude 指向真仓的，preflight 照读），PATH 最前面放假 cargo（build 退 0；run 打两行 E7RESULT，收尾行的 emitted 照 E56_SELFTEST_CARGO 写对或写错），
#   E56_WORK 指到临时目录（镜像也落那里），调用时的工作目录是另一个临时目录：不起真 cargo、不碰仓里的东西，跑完整个临时目录删掉。
# 弄坏开关 E56_SWEEP_BREAK=<项>（只在 --selftest 起的那几趟里认：自证设 E56_SWEEP_SELFTEST_MARK 指到它建的标记文件，别处设了拒、退 2）：
#   E56_SWEEP_BREAK=output-relative-to-apparatus  输出路径不先转绝对路径，cd 进装置目录之后按相对路径写
#       「相对路径的输出落在调用时的工作目录下、不落在装置目录下」
#   E56_SWEEP_BREAK=emitted-count-unchecked  闸 2 不核收尾行的 emitted 与实际条数
#       「收尾行的 emitted 与实际条数对不上：退 1、输出里点名条数对不上」
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SELFTEST_MARK_TEXT="e56-sweep-selftest"

# 弄坏开关只在自证起的那几趟里认（标记文件在、内容对得上）；别处设了就拒。
break_is() {
  [[ ",${E56_SWEEP_BREAK:-}," == *",$1,"* ]]
}
if [[ -n "${E56_SWEEP_BREAK:-}" && "${1:-}" != --selftest ]] \
   && [[ -z "${E56_SWEEP_SELFTEST_MARK:-}" || "$(cat -- "${E56_SWEEP_SELFTEST_MARK}" 2> /dev/null)" != "$SELFTEST_MARK_TEXT" ]]; then
  echo "  ✗ E56_SWEEP_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：平常跑时不设它" >&2
  exit 2
fi

run_selftest() {
  local work failures=0 cell_count=0 exit_code
  work=$(mktemp -d) || return 1
  mkdir -p -- "$work/tree/scripts/experiment-apparatus" "$work/fakes" "$work/caller" "$work/scratch"
  echo "$SELFTEST_MARK_TEXT" > "$work/mark"
  cp -- "$SCRIPT_PATH" "$work/tree/scripts/experiment-apparatus/e56-sweep.sh"
  ln -s -- "$(cd "$(dirname "$SCRIPT_PATH")/../.." && pwd)/.claude" "$work/tree/.claude"
  cat > "$work/fakes/cargo" <<'FAKE'
#!/usr/bin/env bash
# 假 cargo：build 什么都不做；run 打一行数据、一行收尾行（E56_SELFTEST_CARGO=short 时收尾行多报一条）
case "$1" in
  run)
    printf 'E7RESULT name=row selftest=1\n'
    if [[ "${E56_SELFTEST_CARGO:-ok}" == short ]]; then printf 'E7RESULT name=done emitted=3\n'; else printf 'E7RESULT name=done emitted=2\n'; fi ;;
esac
exit 0
FAKE
  chmod +x "$work/fakes/cargo"
  invoke() { # <输出文件参数>：在调用时的工作目录里起树里那一份
    ( cd -- "$work/caller" && env PATH="$work/fakes:$PATH" E56_WORK="$work/scratch" E56_SWEEP_SELFTEST_MARK="$work/mark" \
        bash "$work/tree/scripts/experiment-apparatus/e56-sweep.sh" "$@" > "$work/invoke.log" 2>&1 )
  }
  expect() { # <格名> <条件>
    cell_count=$((cell_count + 1))
    if eval "$2"; then
      echo "  ✓ $1（第 $cell_count 格）"
    else
      echo "  ✗ $1：条件「$2」不成立；输出尾部 $(tail -n 4 -- "$work/invoke.log" 2> /dev/null | tr '\n' ' ')"
      echo "     → 怎么办：照这一格点名的条件改本脚本，再单独起 bash scripts/experiment-apparatus/e56-sweep.sh --selftest 到它退 0"
      failures=$((failures + 1))
    fi
  }
  invoke e56-relative.out; exit_code=$?
  expect "相对路径的输出落在调用时的工作目录下、不落在装置目录下" \
    '((exit_code == 0)) && [[ -f "$work/caller/e56-relative.out" && ! -e "$work/tree/scripts/experiment-apparatus/e56-relative.out" ]] && (($(grep -c "^=== mode=" "$work/caller/e56-relative.out") == 11))'
  E56_SELFTEST_CARGO=short invoke "$work/caller/e56-short.out"; exit_code=$?   # 绝对路径：这一格只看闸 2，不随相对路径那一条开关红
  expect "收尾行的 emitted 与实际条数对不上：退 1、输出里点名条数对不上" \
    '((exit_code == 1)) && grep -qF "条数对不上：抓到 2，收尾行说 3" "$work/caller/e56-short.out"'
  invoke; exit_code=$?
  expect "不给输出文件：退 2、打用法" '((exit_code == 2)) && grep -qF "用法：e56-sweep.sh <输出文件>" "$work/invoke.log"'
  rm -rf -- "${work:?}"
  if ((failures > 0)); then
    echo "  ✗ e56-sweep.sh 自证：$failures 格红（共 $cell_count 格）"
    echo "     → 怎么办：看上面红的那几格各自的出路；E56_SWEEP_BREAK 设着的话点名的那一格本来就该红"
    return 1
  fi
  echo "  ✓ e56-sweep.sh 自证：$cell_count 格全绿"
  return 0
}
if [[ "${1:-}" == --selftest ]]; then
  run_selftest
  exit $?
fi

if [[ -z "${1:-}" ]]; then
  echo "用法：e56-sweep.sh <输出文件>"; echo "→ 怎么办：照文件头的用法给一个输出文件路径（相对路径按调用方的工作目录算）"; exit 2
fi
OUT="$1"
# 先转绝对路径再 cd：相对路径按调用方的工作目录算，不落到装置目录底下
if ! break_is output-relative-to-apparatus; then
  case "$OUT" in /*) ;; *) OUT="$PWD/$OUT" ;; esac
fi
cd "$(dirname "$0")" || { echo "进不去装置目录 $(dirname "$0")"; echo "→ 怎么办：从仓根照文件头的用法起"; exit 2; }
if ! cargo build --release --manifest-path Cargo.toml --bin e56_epsilon; then
  echo "e56_epsilon 编不过，见上面 cargo 的输出"; echo "→ 怎么办：先修编译错误再重交 task-server/task submit -- bash scripts/experiment-apparatus/e56-sweep.sh <输出文件>"; exit 2
fi

# 镜像一律落临时目录，不许进仓（command-safety.md）
WORK="${E56_WORK:-${TMPDIR:-/tmp}/singlefs-e56}"
mkdir -p "$WORK"
IMG="$WORK/e56.img"
truncate -s 512M "$IMG"   # 8192 叶 × ε=0.95 的几何要约 143 MB

# 模式 种子 缓存节点数 叶数
#
# 第 3 组（叶数 4096 / 8192）是两条独立论证腿都点名的那个外推威胁：
# 「树只有 1024 个叶、缓存占 6.4%」。缓存预算不变而树长大 4 倍 / 8 倍，
# 缓存占比从 6.4% 掉到 1.6% / 0.8% —— 收益站不站得住，量出来而不是让步。
CONFIGS="
none 101 66 1024
none 202 66 1024
none 303 66 1024
none 404 66 1024
none 505 66 1024
none 101 66 8192
none 202 66 8192
selfcheck 101 66 1024
none 101 264 1024
none 101 1057 1024
none 101 66 4096
"

rm -f "$OUT"                      # 闸 1：跑前删本轮输出
: > "$OUT"
fail=0
while read -r mode seed cache nleaf; do
  [[ -z "$mode" ]] && continue
  tmp="$WORK/round-$mode-$seed-c$cache-l$nleaf.out"
  rm -f "$tmp"
  echo "=== mode=$mode seed=$seed cache=$cache nleaf=$nleaf ===" >> "$OUT"
  if ! cargo run --release --quiet --manifest-path Cargo.toml --bin e56_epsilon -- "$IMG" "$seed" "$mode" "$cache" "$nleaf" > "$tmp" 2>"$tmp.err"; then
    echo "!! mode=$mode seed=$seed cache=$cache nleaf=$nleaf 退出码非 0" >> "$OUT"; cat "$tmp.err" >> "$OUT"; fail=1; continue
  fi
  got=$(grep -c '^E7RESULT ' "$tmp")
  want=$(sed -n 's/^E7RESULT name=done emitted=\([0-9]*\)$/\1/p' "$tmp" | tail -1)
  if [[ -z "$want" || "$got" != "$want" ]] && ! break_is emitted-count-unchecked; then
    echo "!! mode=$mode seed=$seed cache=$cache nleaf=$nleaf 条数对不上：抓到 $got，收尾行说 ${want:-无}" >> "$OUT"; fail=1
  fi
  cat "$tmp" >> "$OUT"
done <<< "$CONFIGS"
if (( fail )); then echo "E56 跑批判红：有轮次失败或条数对不上，见 $OUT" >&2; exit 1; fi
echo "E56 跑批完成：$OUT"
