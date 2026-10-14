#!/usr/bin/env bash
# 三方腿模型目录里 tools/rerun.sh 的模板：腿把它拷成 <模型目录>/tools/rerun.sh，只改 rerun_steps 那一段。
# 仓根与冻结副本只从参数接，不从这份脚本自己的路径往上推：判决复跑时模型目录被拷进草稿目录跑
# （.claude/rules/three-way-inference.md 判决要引的复跑数那一条），从自身路径推出来的仓根在副本里就不是仓了。
#
# 用法（整条经内存包装跑，工作目录随意）：
#   bash scripts/hosts/run-with-memory-cap.sh <上限> bash <模型目录>/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> [--model <模型目录>]
#   bash <这份脚本> --selftest
#     --repo    仓根：要有 .git 与 scripts/hosts/run-with-memory-cap.sh（判是不是本仓）
#     --frozen  冻结副本（腿开工时拷出的那棵树）：要是一个目录
#     --work    新的草稿目录：不在、或是空目录；跑出来的全写在这里
#     --model   模型目录（原型、变异表、out/）：默认是本脚本的上一层——模型目录整份拷走时它跟着走；只拷这一份脚本时用它指过去
# 退出码：0 跑完；2 用法错（缺参数、仓根不像本仓、冻结副本不在、草稿目录已存在且非空、模型目录不在），stderr 列用法；
#         别的是 rerun_steps 里出错那一步的退出码。--selftest：0 全过，1 有格红。
#
# 准入与运行条件不写、不调 preflight：这份模板拷进模型目录、草稿目录跑，不从自身路径就找不到 .claude/scripts/preflight.sh；
# 环境要求（仓根、冻结副本、草稿目录）由 parse_arguments 自己判，不满足退 2。
#
# 弄坏开关（只给 --selftest 证红用）：
#   RERUN_TEMPLATE_BREAK=root-from-self-path  仓根改回从本脚本路径往上推四层（旧 rerun.sh 的写法），--repo 不认
#       「✗ 自检：拷进与仓无关的临时目录、传参应当跑通」那一格红
set -euo pipefail

refuse() {   # refuse <卡在哪>：打出卡在哪、出路与用法，退 2
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
  if [[ "${RERUN_TEMPLATE_BREAK:-}" == root-from-self-path ]]; then
    REPO_ROOT="$(dirname "${BASH_SOURCE[0]}")/../../../.."
  fi
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

# ── 这一腿要跑的步骤（m3-defs-b6-r2 辩方腿）──
# --frozen 是第二轮冻结副本（after）。第一轮冻结副本（before）用环境变量 BEFORE_FROZEN 给，零轮之前那一版（pre0）用 PRE0_FROZEN 给；
# 没给时取 --frozen 同级的 m3-defs-b6-r1/repo 与 m3-defs-b3-r2/repo。三份都只读，工具里 python 一律 -B，不往冻结副本里写。
# RERUN_PARTS="fm feed v5 v6 htd x6x7 misc" 选哪几步（默认全跑）；每一步的输出写 $WORK/out/，最后与 $MODEL/out/ 同名文件比（路径换成占位再比）。
# 不读 multi-host.env 的任何值：只有 x5 的 pv_path_probe 把它交给抽出来的取词代码数个数，输出里只有个数。
rerun_steps() {
  local after="$FROZEN" before="${BEFORE_FROZEN:-$(dirname "$(dirname "$FROZEN")")/m3-defs-b6-r1/repo}" pre0="${PRE0_FROZEN:-$(dirname "$(dirname "$FROZEN")")/m3-defs-b3-r2/repo}"
  local out="$WORK/out" tools="$MODEL/tools" parts=" ${RERUN_PARTS:-fm feed v5 v6 htd x6x7 misc} "
  [[ -d "$before/.claude/hooks" ]] || refuse "第一轮冻结副本 $before 不在（用 BEFORE_FROZEN 指过去）"
  [[ -d "$pre0/.claude/hooks" ]] || refuse "零轮之前那一版 $pre0 不在（用 PRE0_FROZEN 指过去）"
  export PYTHONDONTWRITEBYTECODE=1
  mkdir -p "$out"
  local roots=(--root "after=$after" --root "before=$before" --root "pre0=$pre0")
  local v dir prefix th b file base
  if [[ $parts == *" fm "* ]]; then   # X1/X6：派发闸 --check-prompt 三份副本矩阵（第一轮正推腿全部用例 + 第二轮新增）
    for prefix in X1 V1 V2 V8 X6; do
      nice -n 19 python3 -B "$tools/feed_matrix.py" "${roots[@]}" --work "$WORK/fm-$prefix" --only "$prefix" > "$out/fm-$prefix.txt" 2>&1 || true
    done
  fi
  if [[ $parts == *" feed "* ]]; then   # X1/X6：stdin 入口，类型表，攻方 V1/V8 变体，SINGLEFS_THREAD_CAP
    nice -n 19 python3 -B "$tools/feed_stdin.py" --frozen "$after" --old "$before" --work "$WORK/stdin-work" 2>&1 | cut -c1-210 > "$out/stdin.txt" || true
    { for v in after before; do dir=$after; [[ $v == before ]] && dir=$before; echo "=== $v"; nice -n 19 python3 -B -I "$tools/tables.py" "$dir" 2>&1 | cut -c1-200; done; } > "$out/tables.txt" || true
    for v in after before pre0; do
      dir=$after; [[ $v == before ]] && dir=$before; [[ $v == pre0 ]] && dir=$pre0
      V1_BASE="$MODEL/feeds/v1/base-investigator.txt" nice -n 19 bash "$tools/v1_feed.sh" "$dir" investigator "$MODEL/feeds/v1/variants.tsv" "$WORK/p-v1-$v" > "$out/o-v1-$v.txt" 2>&1 || true
      V1_BASE="$MODEL/feeds/v1/base-over2h.txt" nice -n 19 bash "$tools/v1_feed.sh" "$dir" investigator "$MODEL/feeds/v1/variants-hosts.tsv" "$WORK/p-v8-$v" > "$out/o-v8-$v.txt" 2>&1 || true
    done
    { for v in after before; do dir=$after; [[ $v == before ]] && dir=$before
        for th in "线程上限：16" "线程上限：31"; do
          printf '重型测试：不跑\n开工先读 `.claude/agent-common.md`\n草稿目录：%s/e/\n报告：%s/e/r.md\n预计挂钟：2 小时内\n%s\n' "$WORK" "$WORK" "$th" > "$WORK/env-p.md"
          (cd / && SINGLEFS_THREAD_CAP=16 bash "$dir/.claude/hooks/runner-dispatch-guard.sh" --check-prompt "$WORK/env-p.md" --type clerk --root "$dir" >/dev/null 2>&1; echo "$v SINGLEFS_THREAD_CAP=16 时 $th 退出=$?")
        done; done; } > "$out/env-cap.txt" 2>&1 || true
  fi
  if [[ $parts == *" v5 "* ]]; then   # X3：重型测试闸（仓副本自证、env -C 换目录、env 前缀拆法矩阵）
    for v in after before pre0; do
      dir=$after; [[ $v == before ]] && dir=$before; [[ $v == pre0 ]] && dir=$pre0
      mkdir -p "$WORK/v5-$v"
      nice -n 19 bash "$tools/v5_feed.sh" "$dir" "$WORK/v5-$v" > "$out/v5-copy-$v.txt" 2>&1 || true
      nice -n 19 bash "$tools/v5_chdir_feed.sh" "$dir" > "$out/v5-chdir-$v.txt" 2>&1 || true
    done
    nice -n 19 python3 -B "$tools/v5_env_matrix.py" "${roots[@]}" > "$out/v5-env-matrix.txt" 2>&1 || true
    nice -n 19 python3 -B "$tools/v5_env_malformed.py" "$after" > "$out/v5-env-malformed.txt" 2>&1 || true
  fi
  if [[ $parts == *" v6 "* ]]; then   # X4：叫停轮登记行（库 + ⑥ 端到端）、真表、小树、⑥ 读不到库、零散现查
    nice -n 19 python3 -B "$tools/v6_matrix.py" --root "after=$after" --root "before=$before" --work "$WORK/v6w" > "$out/v6-matrix.txt" 2>&1 || true
    { echo "--- after 库 默认读法（⑥ 与门禁同一套）"; python3 -B -I "$after/.claude/hooks/lib_abandoned_rounds.py" "$after" | cut -c1-160
      echo "--- before 库 默认读法(⑥)"; python3 -B -I "$before/.claude/hooks/lib_abandoned_rounds.py" "$before" | cut -c1-120
      echo "--- before 库 --legacy-ok(门禁)"; python3 -B -I "$before/.claude/hooks/lib_abandoned_rounds.py" "$before" --legacy-ok | cut -c1-120; } > "$out/x4-real-table.txt" 2>&1 || true
    nice -n 19 bash "$tools/make_gate_trees.sh" "$WORK/trees" > "$out/gate-trees.txt" 2>&1 || true
    { for v in after before; do dir=$after; [[ $v == before ]] && dir=$before; echo "=== $v"
        for t in legacy-sourceless legacy-listed other-name-unrecognised other-name-recognised other-form-nomaterial symlink-source; do
          echo "-- $t 默认读法"; python3 -B -I "$dir/.claude/hooks/lib_abandoned_rounds.py" "$WORK/trees/$t" 2>&1 | sed "s#$WORK#<WORK>#g" | cut -c1-150
          if [[ $v == before ]]; then echo "-- $t --legacy-ok"; python3 -B -I "$dir/.claude/hooks/lib_abandoned_rounds.py" "$WORK/trees/$t" --legacy-ok 2>&1 | sed "s#$WORK#<WORK>#g" | cut -c1-150; fi
        done
        echo "-- exempt-table"; python3 -B -I "$dir/.claude/hooks/lib_long_run.py" exempt-check "$WORK/trees/exempt-table" 2>&1 | sed "s#$WORK#<WORK>#g" | cut -c1-260; done; } > "$out/gate-trees-lib.txt" 2>&1 || true
    nice -n 19 bash "$tools/nolib.sh" "$after" "$WORK/trees/other-name-unrecognised" "$WORK/nolib-work" 2>&1 | cut -c1-200 > "$out/nolib.txt" || true
    OLD_FROZEN="$before" nice -n 19 bash "$tools/adhoc.sh" "$after" "$WORK/trees" "$WORK/adhoc-work" 2>&1 | cut -c1-230 > "$out/adhoc.txt" || true
  fi
  if [[ $parts == *" htd "* ]]; then   # X2：heavy-thread-defaults 格的 python 在三份副本的判法上各跑（抽自各自的 code-tooling.sh）
    for v in after before pre0; do
      dir=$after; [[ $v == before ]] && dir=$before; [[ $v == pre0 ]] && dir=$pre0
      nice -n 19 python3 -B "$tools/run_htd_cell.py" "$dir" "$MODEL/feeds/v3/sample" > "$out/v3-$v.txt" 2>&1 || true
      nice -n 19 python3 -B "$tools/run_htd_cell.py" "$dir" "$MODEL/feeds/x2/sample" > "$out/x2-fp-$v.txt" 2>&1 || true
      nice -n 19 python3 -B "$tools/run_htd_cell.py" "$dir" "$dir" 2>&1 | tail -n 3 > "$out/x2-real-$v.txt" || true
    done
    rm -rf "$WORK/xtree"; mkdir -p "$WORK/xtree"; rsync -a --exclude research --exclude .git --exclude multi-host.env "$after/" "$WORK/xtree/"
    printf '# 空表：看全部被判行\n' > "$WORK/xtree/.claude/heavy-thread-exempt"
    for v in after before pre0; do
      dir=$after; [[ $v == before ]] && dir=$before; [[ $v == pre0 ]] && dir=$pre0
      nice -n 19 python3 -B "$tools/run_htd_cell.py" "$dir" "$WORK/xtree" > "$WORK/x2-empty-$v.txt" 2>&1 || true
    done
    python3 -B -I "$tools/x2_diff_lists.py" "$WORK" > "$out/x2-rule-diff.txt" 2>&1 || true
    rm -rf "$WORK/xtree2"; mkdir -p "$WORK/xtree2"; rsync -a --exclude research --exclude .git --exclude multi-host.env "$after/" "$WORK/xtree2/"
    { for b in before pre0; do dir=$before; [[ $b == pre0 ]] && dir=$pre0; echo "== 基准=$b 的表, 被判=after（只缩不涨那一条）"
        HTD_BASE_TABLE="$dir/.claude/heavy-thread-exempt" nice -n 19 python3 -B "$tools/run_htd_cell.py" "$after" "$WORK/xtree2" 2>&1 | cut -c1-300 | tail -6; done; } > "$out/x2-base-growth.txt" 2>&1 || true
  fi
  if [[ $parts == *" x6x7 "* ]]; then   # X6/X7：E171 下一次修订入口三种头、豁免表 sha256 的各种改动、host-dispatch 的 header_hosts
    nice -n 19 bash "$tools/x6_rev12.sh" "$after" "$before" "$WORK/x6rev12" "$MODEL" 2>&1 | cut -c1-200 > "$out/x6-rev12.txt" || true
    nice -n 19 bash "$tools/x7_sha.sh" "$after" "$WORK/x7" 2>&1 | cut -c1-220 > "$out/x7-sha.txt" || true
    { for v in after before; do dir=$after; [[ $v == before ]] && dir=$before; echo "=== $v"; bash "$tools/x6_sweep.sh" "$dir" 2>&1 | sed "s#$dir#<ROOT>#g"; done; } > "$out/x6-sweep.txt" 2>&1 || true
    local E=scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes
    { for v in after before; do dir=$after; [[ $v == before ]] && dir=$before; mkdir -p "$WORK/hd-$v"
        cp "$dir/$E/run_segment_one_revision_11_multi_host.sh" "$WORK/hd-$v/copy_of_rev11.sh"; cp "$WORK/x6rev12/$v/$E/rev12_c_canonical_head.sh" "$WORK/hd-$v/canon_head.sh"
        echo "=== $v"; python3 -B -I "$tools/hd_header.py" "$dir" "$dir/$E/run_segment_one_revision_11_multi_host.sh" "$WORK/hd-$v/copy_of_rev11.sh" "$WORK/hd-$v/canon_head.sh" 2>&1 | sed "s#$WORK#<WORK>#g" | cut -c1-260; done; } > "$out/hd-header.txt" 2>&1 || true
  fi
  if [[ $parts == *" misc "* ]]; then   # X5：private-values 路径打码的形状；真配置只用来数个数
    nice -n 19 bash "$tools/x5_shapes.sh" "$after" "$before" "$pre0" "$WORK/x5" "$MODEL" > "$out/x5-shapes.txt" 2>&1 || true
    if [[ -f "$after/multi-host.env" ]]; then nice -n 19 python3 -B "$tools/pv_path_probe.py" "$after" "$after/multi-host.env" "$after" > "$out/x5-path-probe.txt" 2>&1 || true; fi
  fi
  echo "name=rerun_outputs work=$WORK"
  for file in "$out"/*.txt; do
    python3 -B -I - "$file" "$WORK" "$after" "$before" "$pre0" "$MODEL" <<'NORMEOF' || true
import sys
path, work, a, b, p, model = sys.argv[1:7]
text = open(path, encoding="utf-8", errors="replace").read()
for real, token in ((work, "<WORK>"), (a, "<AFTER>"), (b, "<BEFORE>"), (p, "<PRE0>"), (model, "<MODEL>")):
    text = text.replace(real, token)
open(path + ".normalized", "w", encoding="utf-8").write(text)
NORMEOF
    base="$(basename "$file")"
    if [[ -f "$MODEL/out/$base" ]]; then
      if diff -q "$MODEL/out/$base" "$file.normalized" >/dev/null; then echo "name=rerun_compare file=$base result=same"; else echo "name=rerun_compare file=$base result=differs"; fi
    else echo "name=rerun_compare file=$base result=no_saved_copy"; fi
  done
}

# 自证：模板拷进一个与仓无关的临时目录（<临时>/elsewhere/r1-opus-model/tools/rerun.sh），从另一个目录起它
selftest() {
  local scratch copy rc checked=0
  local -a failures=()
  # 删临时目录的 trap 在函数返回之后才触发，那时 local 已经没了：路径放进全局变量
  SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/rerun-template-selftest.XXXXXX")"
  trap 'rm -rf "${SELFTEST_SCRATCH:?}"' EXIT
  scratch="$SELFTEST_SCRATCH"
  mkdir -p "$scratch/fake-repo/.git" "$scratch/fake-repo/scripts/hosts" "$scratch/frozen/crates/target" \
    "$scratch/elsewhere/r1-opus-model/tools" "$scratch/cwd"
  : > "$scratch/fake-repo/scripts/hosts/run-with-memory-cap.sh"
  echo 冻结副本里的文件 > "$scratch/frozen/crates/marker.txt"
  echo 编译产物 > "$scratch/frozen/crates/target/junk"
  copy="$scratch/elsewhere/r1-opus-model/tools/rerun.sh"
  cp "${BASH_SOURCE[0]}" "$copy"

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1") \
      > "$scratch/run1.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 0 ]] || ! grep -qF "name=rerun_roots repo=$scratch/fake-repo frozen=$scratch/frozen model=$scratch/elsewhere/r1-opus-model " "$scratch/run1.out" \
      || [[ ! -f "$scratch/work1/tree/crates/marker.txt" || -e "$scratch/work1/tree/crates/target" ]]; then
    failures+=("拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 $rc：$(tail -n 4 "$scratch/run1.out")")
  fi

  checked=$((checked + 1))
  if (cd "$scratch/cwd" && bash "$copy") > "$scratch/run2.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "用法：rerun.sh --repo" "$scratch/run2.out"; then
    failures+=("不传参应当退 2 并列出用法，实际退 $rc：$(tail -n 4 "$scratch/run2.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --work "$scratch/work3" > "$scratch/run3.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "缺 --frozen" "$scratch/run3.out"; then
    failures+=("缺 --frozen 应当退 2 并点名缺它，实际退 $rc：$(tail -n 4 "$scratch/run3.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/cwd" --frozen "$scratch/frozen" --work "$scratch/work4" > "$scratch/run4.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "不像本仓" "$scratch/run4.out"; then
    failures+=("--repo 指到不是仓的目录应当退 2，实际退 $rc：$(tail -n 4 "$scratch/run4.out")")
  fi

  checked=$((checked + 1))
  if bash "$copy" --repo "$scratch/fake-repo" --frozen "$scratch/frozen" --work "$scratch/work1" > "$scratch/run5.out" 2>&1; then rc=0; else rc=$?; fi
  if [[ $rc -ne 2 ]] || ! grep -qF "已存在且非空" "$scratch/run5.out"; then
    failures+=("--work 指到已有输出的目录应当退 2、不在旧输出上接着写，实际退 $rc：$(tail -n 4 "$scratch/run5.out")")
  fi

  if ((${#failures[@]})); then
    local failure
    for failure in "${failures[@]}"; do
      echo "  ✗ 自检：$failure"   # gate-lint:detail
    done
    echo "    → 怎么办：看 parse_arguments() 与 rerun_steps()；RERUN_TEMPLATE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ 自检通过（查了 $checked 格）：拷进与仓无关的临时目录、从别的目录起，传参跑通；不传参、缺 --frozen、--repo 不是仓、--work 非空都退 2 并说明"
  return 0
}

if [[ "${1:-}" == --selftest ]]; then
  if selftest; then exit 0; else exit 1; fi
fi
parse_arguments "$@"
rerun_steps
