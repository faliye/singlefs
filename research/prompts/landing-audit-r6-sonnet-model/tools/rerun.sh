#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh：landing-audit-r6 辩方腿（K1 未定项清单的完整性）。从 scripts/agents/three-way-rerun-template.sh 拷来，只改 rerun_steps。
# 用法：bash scripts/hosts/run-with-memory-cap.sh 4G bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
set -euo pipefail

refuse() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：仓根与冻结副本都从参数给（脚本不从自己的路径推）；草稿目录给一个新的；整条经 scripts/hosts/run-with-memory-cap.sh 跑" >&2
  cat >&2 <<'USAGE'
用法：rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
      rerun.sh --selftest
USAGE
  exit 2
}

parse_arguments() {
  REPO_ROOT="" FROZEN="" WORK="" MODEL=""
  while (($#)); do
    case "$1" in
      --repo|--frozen|--work|--model)
        (($# >= 2)) || refuse "$1 后面要跟一个路径"
        case "$1" in
          --repo) REPO_ROOT="$2" ;;
          --frozen) FROZEN="$2" ;;
          --work) WORK="$2" ;;
          --model) MODEL="$2" ;;
        esac
        shift 2 ;;
      *) refuse "认不出参数：$1" ;;
    esac
  done
  [[ -n "$REPO_ROOT" ]] || refuse "缺 --repo <仓根>"
  [[ -n "$FROZEN" ]] || refuse "缺 --frozen <冻结副本>"
  [[ -n "$WORK" ]] || refuse "缺 --work <新的草稿目录>"
  [[ -e "$REPO_ROOT/.git" && -f "$REPO_ROOT/scripts/hosts/run-with-memory-cap.sh" ]] \
    || refuse "--repo $REPO_ROOT 不像本仓（要有 .git 与 scripts/hosts/run-with-memory-cap.sh）"
  REPO_ROOT="$(cd "$REPO_ROOT" && pwd)"
  [[ -d "$FROZEN" ]] || refuse "--frozen $FROZEN 不是目录"
  FROZEN="$(cd "$FROZEN" && pwd)"
  MODEL="${MODEL:-$(dirname "${BASH_SOURCE[0]}")/..}"
  [[ -d "$MODEL" ]] || refuse "模型目录 $MODEL 不在（只拷了这一份脚本的，用 --model 指过去）"
  MODEL="$(cd "$MODEL" && pwd)"
  if [[ -e "$WORK" && -n "$(ls -A "$WORK")" ]]; then
    refuse "--work $WORK 已存在且非空：给一个新的草稿目录，不在旧输出上接着写"
  fi
  mkdir -p "$WORK"
  WORK="$(cd "$WORK" && pwd)"
}

# ── 这一腿要跑的步骤 ──
# 两步：去向表（destinations.sh：关键字符串在九十节、调度记录、两份判决、r6 正文、欠账表里各出现几次）；
# gate-overlap.py 的写入目标认法探针（overlap_targets.py：mv、cp 加 mv、hook-swap.py 三种命令各认出哪些路径）。
# 九十节与调度记录读模型目录 inputs/ 里的快照；判决与正文、欠账表读 --repo 里现在的文件；gate-overlap.py 读 --frozen 里的上游拷贝。
rerun_steps() {
  mkdir -p "$WORK/out"
  nice -n 19 bash "$MODEL/probes/destinations.sh" "$REPO_ROOT" "$MODEL" > "$WORK/out/destinations.out"
  nice -n 19 python3 -I "$MODEL/probes/overlap_targets.py" "$FROZEN/.claude/singlefs-ai-sop/scripts/gate-overlap.py" > "$WORK/out/overlap_targets.out"
  local name
  for name in destinations overlap_targets; do
    if cmp -s "$WORK/out/$name.out" "$MODEL/out/$name.out"; then
      echo "name=$name 与腿交回的 out/$name.out 逐字节相同"
    else
      echo "name=$name 与腿交回的 out/$name.out 不同（欠账表是现读的，别的会话改过它时会不同；逐行 diff 看）"
      diff "$WORK/out/$name.out" "$MODEL/out/$name.out" | head -n 20 || true
    fi
  done
}

main() {
  parse_arguments "$@"
  rerun_steps
}

main "$@"
