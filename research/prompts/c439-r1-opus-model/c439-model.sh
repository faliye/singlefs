#!/usr/bin/env bash
# C439 第一轮攻方腿的模型：在一个新的临时 git 仓里照流程造一段历史，逐次打出每一次门禁用的基准、
# 被追踪的那一行（TRACK）在不在这一次判的行里、判红判绿，以及最后这段历史里它有没有进过一次窗口。
#
# 用法：bash c439-model.sh <冻结副本> <工作目录（新建）> <候选> <历史> [<续作序列>]
#   候选：today jia yi_shared yi_session yi_gate_mb yi_gate_h1 bing ding_a ding_rw ding_tree wu_set wu_count wu_fixed wu_cf geng geng2
#   历史：见 history_prefix 里的 case
#   续作序列：由用户决定的那几步，一个字母一步：N 不带 --staged 的整轮门禁、S gate-staged（--staged）、
#             c 改一个与正文无关的文件、暂存、提交，k 原样提交暂存区现有的（空就不提交），
#             o 会话甲开一轮（只对乙族有意义），p 推到上游（只在有上游的历史里），G 带 GATE_BASE=HEAD 跑 gate-staged
# 最后一行：result cand=<候选> hist=<历史> cont=<续作> judged_ever=yes|no runs=<进过窗口的那几次>
#
# 用的冻结代码（逐字，不改）：
#   scripts/gate/changed-paths.sh 整份 source（gate_diff_base、gate_added_lines）
#   .claude/singlefs-ai-sop/scripts/lib.sh 第 144–189 行的 diff_base（lib.sh 整份 source 要 preflight.sh，冻结副本里没有，只取这一个函数）
# 门禁本身（gate.sh、gate-staged.sh）不跑：照冻结副本里 --staged 的取树法（gate.sh 第 56、59、87、102、115 行）、
# 前移法（gate.sh 第 774–781 行、gate-staged.sh 第 28、38–46 行）在这里重写一遍，判红判绿只判 quoted-result-lines 一格的简化版：
# 判的行里以 E7RESULT 开头、而 research/results/*.out 里逐字找不到的，就红。
set -uo pipefail
FROZEN="$1"; WORK="$2"; CAND="$3"; HIST="$4"; CONT="${5:-}"
mkdir -p "$WORK" || exit 2
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=model GIT_AUTHOR_EMAIL=model@example.invalid GIT_COMMITTER_NAME=model GIT_COMMITTER_EMAIL=model@example.invalid
# 提交时刻写死：同一段历史每次跑出同一串提交号，复跑的输出逐字可比
export GIT_AUTHOR_DATE="2026-10-14T00:00:00+0900" GIT_COMMITTER_DATE="2026-10-14T00:00:00+0900"  # clock-times:allow 造历史用的固定提交时刻，是被测输入
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GATE_BASE GATE_DIFF_BASE

# shellcheck disable=SC1091
source "$FROZEN/scripts/gate/changed-paths.sh" || { echo "source changed-paths.sh 失败"; exit 2; }
awk 'NR>=144 && NR<=189' "$FROZEN/.claude/singlefs-ai-sop/scripts/lib.sh" > "$WORK/diff_base.sh"
[[ "$(head -1 "$WORK/diff_base.sh")" == "diff_base() {" && "$(tail -1 "$WORK/diff_base.sh")" == "}" ]] || { echo "lib.sh 第 144–189 行不是 diff_base"; exit 2; }
bad() { :; }; howto() { :; }
# shellcheck disable=SC1091
source "$WORK/diff_base.sh"

R="$WORK/repo"; REMOTE="$WORK/remote.git"
PAGE=.claude/kb/page.md; OTHER=.claude/kb/b.md; TABLE=.claude/kb/quoted-exempt.tsv; PRODUCT=research/results/e1.out
GOOD="E7RESULT 正确引文 value=1"
STOCK="E7RESULT 存量错引文 value=7"
L="E7RESULT 漏判样本 value=999"
G2="E7RESULT 会话乙判绿过的正确引文 value=2"
TRACK="$L"
EMPTY_TREE="$(git hash-object -t tree /dev/null)"
LOG="$WORK/log"; : > "$LOG"
RUN=0; JUDGED_RUNS=""; HCOUNT=0
log() { printf '%s\n' "$*" >> "$LOG"; }
g() { git -C "$R" "$@"; }

# ── 候选的基准 ──
anchored_base() {   # anchored_base <ref>：ref 是 HEAD 的祖先就用它；不是就用两者的 merge-base；都没有用空树
  local ref="$1" commit mb
  if commit="$(g rev-parse -q --verify "$ref^{commit}" 2>/dev/null)"; then
    if g merge-base --is-ancestor "$commit" HEAD 2>/dev/null; then echo "$commit"; return; fi
    if mb="$(g merge-base "$commit" HEAD 2>/dev/null)" && [[ -n "$mb" ]]; then echo "$mb"; return; fi
  fi
  echo "$EMPTY_TREE"
}
noted_base() {   # 最近一个带 refs/notes/gate-green 记号的祖先（含 HEAD），没有用空树
  local c
  for c in $(g rev-list HEAD); do
    if g notes --ref=gate-green show "$c" >/dev/null 2>&1; then echo "$c"; return; fi
  done
  echo "$EMPTY_TREE"
}
base_for() {   # base_for <会话> <N|S>：在源仓上算（--staged 那条路也在源仓上算，gate.sh 第 102 行）
  local who="$1" path="$2"
  case "$CAND" in
    today)
      if [[ "$path" == N ]]; then (cd "$R" && unset GATE_BASE && gate_diff_base gate); else diff_base "$R"; fi ;;
    jia) g rev-parse HEAD ;;
    yi_shared) anchored_base refs/sop/round-start ;;
    yi_session) anchored_base "refs/sop/round-start/$who" ;;
    yi_gate_mb) anchored_base refs/sop/gate-ok ;;
    yi_gate_h1) diff_base "$R" ;;
    bing|geng|geng2) g rev-parse -q --verify 'refs/sop/staged-green^{commit}' 2>/dev/null || echo "$EMPTY_TREE" ;;
    ding_a|ding_rw|ding_tree) noted_base ;;
    wu_*) echo FULL ;;
  esac
}

# ── 判：窗口里的行（候选的基准到这棵树的 gate_added_lines），或戊的全量减豁免 ──
judged_lines() {   # judged_lines <树的目录> <基准> <豁免表比对的基准提交>：打出「路径<TAB>行文」
  local dir="$1" base="$2" table_base="$3"
  if [[ "$base" == FULL ]]; then
    (cd "$dir" && python3 "$MODEL_DIR/wu_judge.py" "${CAND#wu_}" "$table_base")
  else
    if git -C "$dir" rev-parse -q --verify "$base^{commit}" >/dev/null 2>&1; then
      # 是提交：照阶段的取法经 GATE_BASE 交给 gate_diff_base gate（gate.sh 第 115 行设 GATE_BASE，阶段里 gate_diff_base gate 取它）
      (cd "$dir" && GATE_BASE="$base" gate_added_lines "$(GATE_BASE="$base" gate_diff_base gate)" .claude/kb ':(glob)research/**/*.md')
    else
      # 空树（候选的记号还不存在）：gate_diff_base 只认提交，直接拿树当基准
      (cd "$dir" && gate_added_lines "$base" .claude/kb ':(glob)research/**/*.md')
    fi
  fi
}
verdict_of() {   # verdict_of <树的目录> <判的行的文件>：red 或 green
  local dir="$1" lines="$2" text
  while IFS=$'\t' read -r _ text; do
    if [[ "$text" == GROWN* ]]; then echo red; return; fi
    [[ "${text# *}" == E7RESULT\ * ]] || continue
    grep -qxF -- "$text" "$dir"/research/results/*.out || { echo red; return; }
  done < "$lines"
  echo green
}
note_run() {   # note_run <路径> <会话> <基准> <判的行的文件> <判定>
  local path="$1" who="$2" base="$3" lines="$4" verdict="$5" in=no
  RUN=$((RUN + 1))
  grep -qF -- "$TRACK" "$lines" && in=yes
  [[ "$in" == yes ]] && JUDGED_RUNS+="${JUDGED_RUNS:+,}$RUN"
  log "run=$RUN path=$path who=$who head=$(g rev-parse --short HEAD) base=${base:0:12} track_in_window=$in verdict=$verdict"
}

# ── 写标记（各候选在全绿之后写什么） ──
mark_green() {   # mark_green <N|S> <开跑时的 HEAD>
  local path="$1" start="$2"
  case "$CAND" in
    today) [[ "$path" == N ]] && g update-ref refs/sop/gate-ok "$start" ;;   # gate.sh 第 774–781 行：只有不带 --staged、GATE_BASE 为空
    yi_gate_mb|yi_gate_h1) g update-ref refs/sop/gate-ok "$start" ;;
    ding_a|ding_rw) g notes --ref=gate-green add -f -m green "$start" >/dev/null 2>&1 ;;
  esac
}
staged_green_move() {   # gate-staged.sh 第 36–48 行：全绿、暂存区没变，就把 staged-green 前移到包着开跑时那棵暂存树的提交
  local tree28="$1" now
  now="$(g write-tree)"
  if [[ "$now" == "$tree28" ]]; then
    g update-ref refs/sop/staged-green "$(g commit-tree "$tree28" -m green)"
    log "  staged-green -> tree ${tree28:0:12}"
  else
    log "  staged-green 不动：暂存树 ${tree28:0:12} -> ${now:0:12}"
  fi
}

# ── 操作 ──
op_write() { printf '%s\n' "$2" >> "$R/$1"; }
op_remove_line() { grep -vxF -- "$2" "$R/$1" > "$R/$1.tmp"; mv "$R/$1.tmp" "$R/$1"; }
op_stage() { g add -- "$@"; }
op_commit() {   # op_commit [pathspec…]：提交（临时仓里没有钩子，提交都不经门禁）
  g commit -q -m "c$((++HCOUNT))" ${1+-- "$@"} || { log "commit 失败"; return 1; }
  if [[ "$CAND" == ding_tree ]]; then   # 丁的树对树变体：提交那一刻 HEAD 的树等于 staged-green 的树才记号
    local sg; sg="$(g rev-parse -q --verify 'refs/sop/staged-green^{tree}' 2>/dev/null || true)"
    [[ -n "$sg" && "$sg" == "$(g rev-parse 'HEAD^{tree}')" ]] && g notes --ref=gate-green add -f -m green HEAD >/dev/null 2>&1
  fi
  log "commit $(g rev-parse --short HEAD)"
}
op_gate_N() {   # 不带 --staged 的整轮：判工作区（含未跟踪），开跑时的 HEAD 记作 START_HEAD（gate.sh 第 140 行）
  local who="$1" start base lines="$WORK/lines.$RUN" verdict
  start="$(g rev-parse HEAD)"
  base="$(base_for "$who" N)"
  local table_base=HEAD; [[ "$CAND" == wu_fixed || "$CAND" == wu_cf ]] && table_base="$(g rev-parse refs/sop/wu-baseline)"
  judged_lines "$R" "$base" "$table_base" > "$lines" || { log "判的行取不到"; return 1; }
  verdict="$(verdict_of "$R" "$lines")"
  note_run N "$who" "$base" "$lines" "$verdict"
  [[ "$verdict" == green ]] && mark_green N "$start"
  return 0
}
op_gate_S() {   # gate-staged.sh → gate.sh --staged：write-tree、diff --cached、worktree add HEAD、apply --index、源仓上算基准
  local who="$1" hook_after_write_tree="${2:-}" hook_before_check="${3:-}" hook_after_patch="${4:-}" tree28 wt="$WORK/wt" start base lines="$WORK/lines.$RUN" verdict table_base
  tree28="$(g write-tree)"
  [[ -n "$hook_after_write_tree" ]] && { eval "$hook_after_write_tree"; log "  （交错）$hook_after_write_tree"; }
  g diff --cached --binary > "$WORK/staged.patch"
  [[ -n "$hook_after_patch" ]] && { eval "$hook_after_patch"; log "  （交错）$hook_after_patch"; }
  rm -rf "$wt"; g worktree add -q --detach "$wt" HEAD >/dev/null 2>&1 || { log "worktree add 失败"; return 1; }
  if [[ "$CAND" == geng || "$CAND" == geng2 ]]; then
    # 庚（攻方腿自己提的改法，被攻过零轮）：被判的树就是开跑时 write-tree 的那一棵，不再从 diff --cached 重套
    git -C "$wt" read-tree --reset -u "$tree28" || { log "read-tree 失败"; g worktree remove --force "$wt"; return 1; }
  elif [[ -s "$WORK/staged.patch" ]]; then git -C "$wt" apply --index "$WORK/staged.patch" || { log "暂存区的 diff 套不上 HEAD"; g worktree remove --force "$wt"; return 1; }; fi
  start="$(git -C "$wt" rev-parse HEAD)"
  base="$(base_for "$who" S)"
  # 人显式给了 GATE_BASE（gate.sh 第 102 行 ${GATE_BASE:-…} 先认它）：窗口取它，gate.sh 第 774 行的守卫不前移 gate-ok，
  # gate-staged.sh 第 36–48 行没有对应的守卫，照样前移 staged-green
  [[ -n "${OVERRIDE_BASE:-}" && "$CAND" != wu_* ]] && base="$(g rev-parse "$OVERRIDE_BASE")"
  table_base=HEAD; [[ "$CAND" == wu_fixed || "$CAND" == wu_cf ]] && table_base="$(g rev-parse refs/sop/wu-baseline)"
  judged_lines "$wt" "$base" "$table_base" > "$lines" || { log "判的行取不到"; g worktree remove --force "$wt"; return 1; }
  verdict="$(verdict_of "$wt" "$lines")"
  g worktree remove --force "$wt"
  note_run S "$who" "$base" "$lines" "$verdict"
  [[ -n "$hook_before_check" ]] && { eval "$hook_before_check"; log "  （交错）$hook_before_check"; }
  if [[ "$verdict" == green ]]; then
    [[ -z "${OVERRIDE_BASE:-}" ]] && mark_green S "$start"
    if [[ "$CAND" == geng2 && -n "${OVERRIDE_BASE:-}" ]]; then
      log "  庚二：GATE_BASE 是人给的，staged-green 不前移"   # 庚二（攻方腿自己提的改法，被攻过零轮）：gate-staged.sh 补上与 gate.sh 第 774 行同一种守卫
    else
      staged_green_move "$tree28"   # gate-staged.sh 第 36–48 行，每个候选都照做；只有丙、庚与丁的树对树变体拿它当基准
    fi
  fi
  return 0
}
op_open() { case "$CAND" in yi_shared) g update-ref refs/sop/round-start HEAD ;; yi_session) g update-ref "refs/sop/round-start/$1" HEAD ;; esac; log "open $1 at $(g rev-parse --short HEAD)"; }
op_push() { g push -q origin master 2>/dev/null && log "push $(g rev-parse --short HEAD)"; }

# ── 起点：存量一行错引文进根提交（戊的豁免表里有它），C0 上各候选的记号都已就位，然后照流程一次 gate-staged 全绿再提交（C1） ──
init_repo() {
  local with_upstream="$1"
  git init -q -b master --template= "$R"
  mkdir -p "$R/.claude/kb" "$R/research/results"
  printf '# 页\n\n%s\n%s\n' "$GOOD" "$STOCK" > "$R/$PAGE"
  printf '%s\n%s\n' "$GOOD" "$G2" > "$R/$PRODUCT"
  printf '%s\t%s\n' "$PAGE" "$STOCK" > "$R/$TABLE"
  printf 'base\n' > "$R/src.txt"
  g add -A && g commit -q -m root
  g update-ref refs/sop/wu-baseline HEAD
  printf 'c0\n' >> "$R/src.txt"; g add -A && g commit -q -m c0
  [[ "$CAND" == ding_rw ]] && g config notes.rewriteRef refs/notes/gate-green
  case "$CAND" in
    yi_gate_mb|yi_gate_h1) g update-ref refs/sop/gate-ok HEAD ;;
    bing|geng|geng2|ding_tree|today) g update-ref refs/sop/staged-green "$(g commit-tree "$(g rev-parse 'HEAD^{tree}')" -m green)" ;;
    ding_a|ding_rw) g notes --ref=gate-green add -m green HEAD ;;
  esac
  [[ "$CAND" == ding_tree ]] && g notes --ref=gate-green add -m green HEAD
  op_open A; op_open B
  if [[ "$with_upstream" == yes ]]; then
    git init -q --bare "$REMOTE"; g remote add origin "$REMOTE"; g push -q -u origin master 2>/dev/null
  fi
  op_write src.txt h0; op_stage src.txt; op_gate_S A; op_commit
  [[ "$with_upstream" == yes ]] && op_push
  log "--- 起点完 ---"
}

# ── 各段历史的前缀与故障（固定），之后接续作序列 ──
history_prefix() {
  case "$HIST" in
    h0_control)       init_repo no;  op_write $PAGE "$L"; op_stage $PAGE ;;   # 对照：错引文暂存着没提交，任何候选的下一次门禁都该判到它
    h1_c439)          init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit; op_gate_N A ;;
    h2_noverify)      init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit ;;
    h3_split)         init_repo no;  op_write src.txt a; op_write src2.txt b; op_stage src.txt src2.txt; op_gate_S A
                                     op_commit src.txt; op_write $PAGE "$L"; op_stage $PAGE; op_commit ;;
    h4_other)         init_repo no;  op_write src.txt a; op_stage src.txt; op_gate_S A; op_commit
                                     op_write $OTHER "$L"; op_stage $OTHER; op_commit ;;
    h5_pushed)        init_repo yes; op_write $PAGE "$L"; op_stage $PAGE; op_commit; op_push ;;
    h6_stamp_S)       init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit
                                     op_remove_line $PAGE "$L"; op_stage $PAGE; op_gate_S A; g restore -q --staged --worktree -- $PAGE; log "restore $PAGE" ;;
    h6_stamp_N)       init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit
                                     op_remove_line $PAGE "$L"; op_gate_N A; g restore -q --worktree -- $PAGE; log "restore $PAGE" ;;
    h6_stamp_revert)  init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit
                                     op_remove_line $PAGE "$L"; op_stage $PAGE; op_gate_S A; op_commit
                                     g revert --no-edit HEAD >/dev/null || log "revert 失败"; log "revert -> $(g rev-parse --short HEAD)" ;;
    h8_amend)         init_repo no;  op_write src.txt a; op_stage src.txt; op_gate_S A
                                     op_write $PAGE "$L"; op_stage $PAGE; g commit -q --amend --no-edit; log "amend -> $(g rev-parse --short HEAD)" ;;
    h9_aba)           init_repo no;  op_write $OTHER "$L"; op_stage $OTHER
                                     op_gate_S A "g rm -q --cached -- $OTHER" "g add -- $OTHER" ;;
    h9_stash)         init_repo no;  op_write $OTHER "$L"; op_stage $OTHER   # 会话乙在甲的 gate-staged 跑的当中 stash、再 stash pop --index
                                     op_gate_S A "g stash push -q" "g stash pop -q --index" ;;
    h9_commit_undo)   init_repo no;  op_write $OTHER "$L"; op_stage $OTHER   # 会话乙在甲的 gate-staged 跑的当中提交、又 reset --soft 撤回
                                     op_gate_S A "g commit -q -m tmp" "" "g reset -q --soft HEAD~1" ;;
    h2u_unpushed)     init_repo yes; op_write $PAGE "$L"; op_stage $PAGE; op_commit ;;   # 有上游、没推
    h15_override)     init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit   # 绕过门禁提交之后，有人带 GATE_BASE=HEAD 跑 gate-staged 把窗口收窄
                                     op_write src.txt x; op_stage src.txt; OVERRIDE_BASE=HEAD op_gate_S A ;;
    h11_dup)          init_repo no;  TRACK="$STOCK"; op_write $PAGE "$STOCK"; op_stage $PAGE; op_gate_S A; op_commit ;;
    h12_exempt_grow)  init_repo no;  op_write $PAGE "$L"; printf '%s\t%s\n' "$PAGE" "$L" >> "$R/$TABLE"; op_stage $PAGE $TABLE; op_commit ;;
    h14_rejudge_cost) init_repo no;  TRACK="$G2"; op_write $OTHER "$G2"; op_stage $OTHER; op_gate_S B; op_commit ;;
    h13_shared_open)  init_repo no;  op_write $PAGE "$L"; op_stage $PAGE; op_commit; op_open B ;;
    *) echo "认不出历史 $HIST"; exit 2 ;;
  esac
  log "--- 前缀与故障完 ---"
  PREFIX_RUNS=$RUN
}

continue_with() {
  local step k=0
  for ((k = 0; k < ${#CONT}; k++)); do
    step="${CONT:k:1}"
    case "$step" in
      N) op_gate_N A ;;
      S) op_gate_S A ;;
      G) OVERRIDE_BASE=HEAD op_gate_S A ;;
      c) op_write src.txt "k$k"; op_stage src.txt; op_commit ;;
      k) if g diff --cached --quiet; then log "k：暂存区是空的，不提交"; else op_commit; fi ;;
      o) op_open A ;;
      p) op_push ;;
      *) echo "认不出续作 $step"; exit 2 ;;
    esac
  done
}

MODEL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
history_prefix
continue_with
after_prefix=""; for r in ${JUDGED_RUNS//,/ }; do ((r > PREFIX_RUNS)) && after_prefix+="${after_prefix:+,}$r"; done
echo "result cand=$CAND hist=$HIST cont=${CONT:-none} judged_ever=$([[ -n "$JUDGED_RUNS" ]] && echo yes || echo no) runs=${JUDGED_RUNS:-none} prefix_runs=$PREFIX_RUNS after_prefix=${after_prefix:-none} total_runs=$RUN"
