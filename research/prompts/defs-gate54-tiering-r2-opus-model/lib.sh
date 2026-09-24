#!/usr/bin/env bash
# defs-gate54-tiering-r2 攻方模型的公共函数。只在 $RUNS 下的合成仓里动 git；真仓一处不碰。
set -uo pipefail
MODEL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNS="${RUNS:-/tmp/claude-1000/defs54-r2-attack/runs}"
mkdir -p "$RUNS/tmp"
export TMPDIR="$RUNS/tmp"
export PATH="$MODEL/fake-bin:$PATH"
export FAKE_TOOLCHAIN_FILE="$RUNS/toolchain-version"
unset SINGLEFS_LAYER0_THREADS SINGLEFS_STAGED_TREE GATE_BASE SINGLEFS_GATE_FULL FAKE_CARGO_HOLD
export GIT_AUTHOR_DATE="2026-09-24T00:00:00Z" GIT_COMMITTER_DATE="2026-09-24T00:00:00Z"
STAGE=".claude/gate.d/54-layer0-replay.sh"
trim() { LC_ALL=C.UTF-8 awk -v n="$1" '{ print substr($0, 1, n) }'; }
THREAD_CHECK_LINE='  if [[ "$2" == 1 && "$machine_cores" -gt 1 && ! ( "$threads_origin" == "显式设的" && "$SINGLEFS_LAYER0_THREADS" == 1 ) ]]; then'

# variant_54 <arm:post|pre> <version:new|old|wip-strict> ⇒ 把那一版 54 号打到 stdout
#   new        入库形态（post＝工作区 sha256 232c8c0d…；pre＝HEAD 那一份）
#   old        同一份，只把「多核只起 1 个工作线程判红」那一判换成 if false（仿 fc7942f 之前没有这一判的 54 号）
#   wip-strict 别的会话在主工作区里改到一半的 54 号：LAYER0 行另要 journal_differing=0（合成日志里是 3，于是 --full 判红）
variant_54() {
  local source="$MODEL/inputs/54-$1.sh"
  # POST_54_OVERRIDE：拿打了改法的一份 54 号当分档之后那一臂（old / wip-strict 照样从它派生）
  [[ "$1" == post && -n "${POST_54_OVERRIDE:-}" ]] && source="$POST_54_OVERRIDE"
  case "$2" in
    new) cat "$source" ;;
    old)
      [[ "$(grep -cxF "$THREAD_CHECK_LINE" "$source")" == 1 ]] || { echo "variant_54: 线程那一行在 $source 里不是恰好一次" >&2; return 1; }
      awk -v target="$THREAD_CHECK_LINE" '$0 == target { print "  if false; then # 模型：旧版 54 号没有这一判"; next } { print }' "$source" ;;
    wip-strict)
      [[ "$(grep -cxF 'worker_threads_are_acceptable "第一个事务那条流" "$worker_threads" || exit 1' "$source")" == 1 ]] || { echo "variant_54: 锚点不是恰好一次" >&2; return 1; }
      awk '$0 == "worker_threads_are_acceptable \"第一个事务那条流\" \"$worker_threads\" || exit 1" { print; print "[[ \"$line\" == *\"journal_differing=0\"* ]] || { echo \"  ✗ （别的会话改到一半的 54 号）LAYER0 行 journal_differing 不是 0\"; exit 1; }"; next } { print }' "$source" ;;
  esac
}

# want <格名>：CASES 没设就全跑；设了只跑点名的那几格（改法那一轮只重跑打中的格与对照）
want() { [[ -z "${CASES:-}" || " $CASES " == *" $1 "* ]]; }

# make_repo <目录> <arm> <54 的版本> <crash.rs 里的记号，可空>
make_repo() {
  local repo="$1" arm="$2" version="$3" token="${4:-}"
  rm -rf "$repo"; mkdir -p "$repo"
  git -C "$repo" init -q -b master
  git -C "$repo" config user.email model@example.com; git -C "$repo" config user.name model
  mkdir -p "$repo/.claude/gate.d" "$repo/research/scripts" "$repo/crates/singlefs-harness/src" "$repo/docs"
  variant_54 "$arm" "$version" > "$repo/$STAGE"
  if [[ "$arm" == post ]]; then
    cp "$MODEL/inputs/stage-inputs.tsv" "$repo/.claude/gate.d/stage-inputs.tsv"
    cp "$MODEL/inputs/stage-must-run.sh" "$repo/research/scripts/stage-must-run.sh"
  else
    cp "$MODEL/inputs/stage-inputs-pre.tsv" "$repo/.claude/gate.d/stage-inputs.tsv"
    cp "$MODEL/inputs/stage-must-run-pre.sh" "$repo/research/scripts/stage-must-run.sh"
  fi
  cp "$MODEL/inputs/change-touches-crates.sh" "$repo/research/scripts/change-touches-crates.sh"
  printf '[workspace]\nmembers = ["crates/singlefs-harness"]\n' > "$repo/Cargo.toml"
  printf '# lock v1\n' > "$repo/Cargo.lock"
  printf 'pub mod crash;\n' > "$repo/crates/singlefs-harness/src/lib.rs"
  set_crash "$repo" base "$token"
  printf 'note\n' > "$repo/docs/note.md"
  printf 'target/\n' > "$repo/.gitignore"
  git -C "$repo" add -A && git -C "$repo" commit -q -m base
}

# set_crash <仓> <版本名> <记号>：改工作区里的 crash.rs（内容随版本名变，记号决定假 cargo 的行为）
set_crash() { printf '// crash.rs version=%s\n// tokens: %s\npub fn enumerate() {}\n' "$2" "${3:-none}" > "$1/crates/singlefs-harness/src/crash.rs"; }
set_54() { variant_54 "$2" "$3" > "$1/$STAGE"; }
common_dir() { git -C "$1" rev-parse --path-format=absolute --git-common-dir; }

# cells <仓>：common-dir 里的全绿格，每格一行（哈希前 12 位、judged_root 的尾巴、线程那一行）
cells() {
  local found=0 cell
  for cell in "$(common_dir "$1")"/singlefs-layer0-full-green.*; do
    [[ -f "$cell" ]] || continue
    [[ "$cell" == *.partial.* ]] && { echo "      [残留 partial] $(basename "$cell")"; continue; }
    found=1
    echo "      [格] ${cell##*.}" | cut -c1-24 | tr -d '\n'; echo " worker_threads=$(sed -n 's/^worker_threads=第一个事务那条流 \([0-9]*\)、.*/\1/p' "$cell") judged_root=…/$(sed -n 's/^judged_root=//p' "$cell" | awk -F/ '{print $(NF-1)"/"$NF}')"
  done
  (( found )) || echo "      [格] 一格都没有"
}

# gate_staged_54 <仓>：照共享 gate.sh --staged 的建法（diff --cached --binary、worktree add --detach HEAD、apply --index）
# 加 gate-staged.sh 的 SINGLEFS_STAGED_TREE，只跑 54 号那一段。打 54 号的 ✓ / ✗ / ! 行与退出码。
gate_staged_54() {
  local repo="$1" base tree_object rc
  tree_object="$(git -C "$repo" write-tree)"
  base="$(mktemp -d)"
  git -C "$repo" diff --cached --binary > "$base/staged.patch"
  git -C "$repo" worktree add -q --detach "$base/tree" HEAD
  if [[ -s "$base/staged.patch" ]]; then git -C "$base/tree" apply --index "$base/staged.patch"; fi
  SINGLEFS_STAGED_TREE="$tree_object" bash "$base/tree/$STAGE" "$base/tree" > "$base/out" 2>&1; rc=$?
  grep -E '^  (✓|✗|!) ' "$base/out" | trim 150
  echo "    [gate --staged 里 54 号退出码 $rc]"
  git -C "$repo" worktree remove --force "$base/tree"; rm -rf "${base:?}"
  return "$rc"
}

# outpath_file <54 版本文件> <写到哪>：从 54 号出路函数的三行 echo 里取出它打给人照跑的命令，原样写成脚本
outpath_file() {
  awk '/^print_staged_worktree_full_commands\(\) \{/ { inside = 1; next } inside && /^\}/ { inside = 0 } inside' "$1" \
    | sed -e "s/^  echo '//" -e "s/'\$//" -e 's/^ *在项目根、暂存之后：//' -e 's/^ *//' > "$2"
  [[ "$(wc -l < "$2")" == 3 ]] || { echo "outpath_file: 取出来不是三行" >&2; return 1; }
}

# truth_full <仓>：同一份「HEAD + 暂存区」在另一个 clone 里（另一个 common-dir，不碰这个仓的格）用那棵树里的 54 号跑 --full
truth_full() {
  local repo="$1" truth rc
  truth="$(mktemp -d)/truth"
  git clone -q --no-hardlinks "$repo" "$truth"
  git -C "$repo" diff --cached --binary > "$truth.patch"
  if [[ -s "$truth.patch" ]]; then git -C "$truth" apply --index "$truth.patch"; fi
  # 分档之前的 54 号不认 --full（它把参数当项目根），全量是它的默认
  if grep -qF -- '--full) layer0_tier="full" ;;' "$truth/$STAGE"; then bash "$truth/$STAGE" --full "$truth" > "$truth.out" 2>&1; rc=$?
  else SINGLEFS_GATE_FULL=1 bash "$truth/$STAGE" "$truth" > "$truth.out" 2>&1; rc=$?; fi  # 真值要的是「跑全量」，越过它的改动范围那一问
  grep -E '^  ✗ ' "$truth.out" | head -2 | trim 150
  echo "    [真值：这份 HEAD + 暂存区用它自己的 54 号跑 --full，退出码 $rc]"
  rm -rf "$(dirname "$truth")"
  return "$rc"
}
