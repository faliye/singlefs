#!/usr/bin/env bash
# 门禁阶段共用：问一道「要不要跑」的判定脚本，照它的退出码往下跑、退 77 或判红。阶段 source 它，不各抄一份。
#
#   stage_run_or_skip <判定脚本> <跳过时括号里说的依据> <跳过时的出路> -- <判定脚本的参数…>
#
# 判定脚本是 research/scripts/stage-must-run.sh（复用上一次整轮全绿的判定）与 research/scripts/change-touches-crates.sh
# （这次改动碰没碰这一道读的路径），两份约定一样：退 0 要跑，退 1 可跳过，自己判不出来也退 0。
# 除此之外的退出码只可能是判定脚本没跑成：文件不在退 127、不可读退 126、被信号打断退 128 以上。
# 这些不许读成「可跳过」：那样这一道会一直退 77，而 77 在整轮汇总里只记「本次未跑」、不挡末句，门禁照说全部通过。
# 所以只认 1 为跳过（退 77），0 往下跑，其余判红（退 1）。
#
# 退出码 77 与 1 都由这个函数直接 exit，调用它的阶段不用再接；返回 0 时阶段接着往下跑。
#
#   bash research/scripts/stage-run-or-skip.sh --selftest   自证四格：判定脚本退 0 往下跑、退 1 退 77、
#   退 2 判红、文件不在（127）判红；格数由成功行现算。STAGE_RUN_OR_SKIP_BREAK=treat-unknown-as-skip 把「其余判红」
#   换回「非 0 一律跳过」的旧写法，自证必须判红。
stage_run_or_skip() {
  local judge_script="$1" skip_basis="$2" skip_howto="$3"
  shift 3
  if [[ "${1:-}" == "--" ]]; then shift; fi
  local judge_reason="" judge_exit_code=0
  judge_reason="$(bash "$judge_script" "$@")" || judge_exit_code=$?
  case "$judge_exit_code" in
    0)
      return 0 ;;
    1)
      echo "  ! 本阶段跳过（${skip_basis}）：$judge_reason"
      echo "     → $skip_howto"
      exit 77 ;;
    *)
      echo "  ✗ 判不出这一道要不要跑：$judge_script 退 $judge_exit_code（它只许退 0 要跑、1 可跳过），这一道按红记"
      echo "     → 怎么办：单独执行 bash $judge_script $* 看它停在哪；文件不在或起不来就先修它，修好之前这一道一直红，不退 77。"
      exit 1 ;;
  esac
}

# 弄坏开关：只给自证用，把「只认 1 为跳过」换回审核抓到的旧写法「非 0 一律跳过」，自证必须判红
if [[ "${STAGE_RUN_OR_SKIP_BREAK:-}" == "treat-unknown-as-skip" ]]; then
  stage_run_or_skip() {
    local judge_script="$1" skip_basis="$2" skip_howto="$3"; shift 3
    if [[ "${1:-}" == "--" ]]; then shift; fi
    local judge_reason="" judge_exit_code=0
    judge_reason="$(bash "$judge_script" "$@")" || judge_exit_code=$?
    if [[ "$judge_exit_code" != 0 ]]; then echo "  ! 本阶段跳过（${skip_basis}）：$judge_reason"; echo "     → $skip_howto"; exit 77; fi
  }
fi

stage_run_or_skip_selftest() {
  local scratch helper_path="${BASH_SOURCE[0]}" cases=0 wrong=0
  scratch="$(mktemp -d)"
  trap 'rm -rf "${scratch:?}"' RETURN
  printf 'echo "要跑：输入变了"; exit 0\n' > "$scratch/judge-run.sh"
  printf 'echo "可跳过：输入没变"; exit 1\n' > "$scratch/judge-skip.sh"
  printf 'echo "内部出错"; exit 2\n' > "$scratch/judge-broken.sh"
  local label judge want_exit want_text got_exit output
  while IFS='|' read -r label judge want_exit want_text; do
    cases=$((cases + 1)); got_exit=0
    output="$(bash -c 'source "$1"; stage_run_or_skip "$2" "样本依据" "样本出路" -- a b; echo "往下跑了"' _ "$helper_path" "$scratch/$judge" 2>&1)" || got_exit=$?
    if [[ "$got_exit" != "$want_exit" || "$output" != *"$want_text"* ]]; then
      wrong=$((wrong + 1))
      echo "  ✗ 自证「$label」：期望退 $want_exit 且输出含「$want_text」，实得退 $got_exit：$output"   # gate-lint:detail
    fi
  done <<'CASES'
判定退 0 就往下跑|judge-run.sh|0|往下跑了
判定退 1 就退 77|judge-skip.sh|77|本阶段跳过（样本依据）：可跳过：输入没变
判定退 2 判红|judge-broken.sh|1|退 2（它只许退 0 要跑、1 可跳过）
判定脚本不在判红|judge-missing.sh|1|退 127（它只许退 0 要跑、1 可跳过）
CASES
  if ((wrong)); then
    echo "  ✗ stage-run-or-skip 自证：$wrong / $cases 格不中"   # gate-lint:summary
    echo "     → 怎么办：照上面不中的那几格修 stage_run_or_skip；设了 STAGE_RUN_OR_SKIP_BREAK 时判红是对的。"
    return 1
  fi
  echo "  ✓ stage-run-or-skip 自证通过：$cases 格（退 0 往下跑、退 1 退 77、其余判红）"
}

if [[ "${BASH_SOURCE[0]}" == "$0" && "${1:-}" == "--selftest" ]]; then
  stage_run_or_skip_selftest
  exit $?
fi
