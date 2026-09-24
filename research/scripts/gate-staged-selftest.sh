#!/usr/bin/env bash
# `gate-staged.sh` 的自证：在临时小仓里用一个假 gate.sh 造三种局面，逐格核 refs/sop/staged-green 前不前移；格数由成功行现算。
#
#   gate-staged-selftest.sh [被测的 gate-staged.sh]    不给就测同目录那一份
#
# 三种局面，对着 gate-staged.sh 文件头那三条规矩：
#   全绿                 ⇒ 退出码 0；ref 指向一个提交，那个提交的树就是开跑时的暂存树（不是 HEAD 的树）；
#                          假 gate.sh 收到的第一个参数是 --staged，环境里的 SINGLEFS_STAGED_TREE 就是那棵暂存树
#   有红                 ⇒ 退出码原样转手；ref 不动
#   跑的过程中暂存区变了 ⇒ ref 不动（这时退几由 gate-staged.sh 自己定，这里不钉）
#
# 每一格都要先证明会红：把被测脚本拷一份，按 mutation 那几行逐条改坏（原文必须恰好命中一次，
# 命中不是一次就是锚点腐化、当场判红），改坏的每一份都必须让它点名的那一格判红；原样那一份每一格都要过。
# 假 gate.sh 与临时仓都在 mktemp 出来的目录里，碰不到本仓的暂存区与 ref。
set -uo pipefail
# 门禁要是在 git 钩子里跑，这几个变量会指着本仓的索引与对象库；不清掉，临时仓的 git 命令会写到本仓头上。
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_PREFIX GIT_NAMESPACE
export GIT_AUTHOR_NAME=selftest GIT_AUTHOR_EMAIL=selftest@example.invalid GIT_COMMITTER_NAME=selftest GIT_COMMITTER_EMAIL=selftest@example.invalid
# 用户自己的全局配置（签名提交、默认分支名、钩子路径）不许影响临时仓里的判定
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
# 门禁 47 号跑这份自证时，外层多半就是一趟 gate-staged.sh，环境里已经导出了 SINGLEFS_STAGED_TREE；
# 留着它，被测脚本漏了 export 也照样传得下去，「暂存树不交给门禁」那一条改坏就抓不到。
unset SINGLEFS_STAGED_TREE FAKE_GATE_RECORD FAKE_GATE_EXIT FAKE_GATE_TOUCH_INDEX

TARGET="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/gate-staged.sh}"
if [[ ! -f "$TARGET" ]]; then
  echo "  ✗ 找不到被测的 gate-staged.sh：$TARGET"
  echo "  → 怎么办：不给参数就测同目录那一份；给参数就写被测脚本的路径。"
  exit 2
fi
work="$(mktemp -d)"
trap 'rm -rf "${work:?}"' EXIT

MUTATION_NAMES=(); MUTATION_CELLS=(); MUTATION_ORIGINALS=(); MUTATION_REPLACEMENTS=()
mutation() { # mutation <改坏的是什么> <必须因此判红的那一格> <原文> <替换文>
  MUTATION_NAMES+=("$1"); MUTATION_CELLS+=("$2"); MUTATION_ORIGINALS+=("$3"); MUTATION_REPLACEMENTS+=("$4")
}
mutation "有红也前移"               "有红：ref 不动"                   'if [[ $gate_rc -eq 0 ]]; then'                     'if true; then'
mutation "不核跑的过程中暂存区变没变" "暂存区变了：ref 不动"             'if [[ "$now_tree" != "$staged_tree" ]]; then'   'if false; then'
mutation "ref 指向裸树不指向提交"    "全绿：ref 指向一个提交"           'git update-ref refs/sop/staged-green "$green_commit"' 'git update-ref refs/sop/staged-green "$staged_tree"'
mutation "判的是 HEAD 的树不是暂存树" "全绿：ref 的树就是开跑时的暂存树" 'staged_tree="$(git write-tree)"'                 'staged_tree="$(git rev-parse "HEAD^{tree}")"'
mutation "不带 --staged 跑门禁"      "全绿：假门禁收到 --staged"        'bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"' 'bash "$ROOT/.claude/scripts/gate.sh" "$@"'
mutation "退出码不转手"              "有红：退出码原样转手"             'exit $gate_rc'                                     'exit 0'
mutation "暂存树不交给门禁"          "全绿：环境里的暂存树对得上"       'export SINGLEFS_STAGED_TREE'                       'true'
mutation "全绿也退非 0"              "全绿：退出码 0"                   'exit $gate_rc'                                     'exit 1'

# make_repository <目录>：一个提交、一处已暂存的改动（暂存树与 HEAD 的树不同），外加一个假 gate.sh。
make_repository() {
  local repository="$1"
  mkdir -p "$repository/.claude/scripts"
  git init -q -b master "$repository"
  printf 'base\n' > "$repository/tracked.txt"
  git -C "$repository" add tracked.txt && git -C "$repository" commit -q -m base
  printf 'staged change\n' >> "$repository/tracked.txt"
  git -C "$repository" add tracked.txt
  # 假门禁：记下收到的参数与环境里的暂存树，按 FAKE_GATE_EXIT 退出；
  # FAKE_GATE_TOUCH_INDEX=1 时在跑的过程中往暂存区再放一处改动（别的会话在这段时间里暂存了东西）。
  cat > "$repository/.claude/scripts/gate.sh" <<'FAKE_GATE_EOF'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$FAKE_GATE_RECORD/arguments"
printf '%s\n' "${SINGLEFS_STAGED_TREE:-}" > "$FAKE_GATE_RECORD/staged-tree-in-environment"
if [[ "${FAKE_GATE_TOUCH_INDEX:-0}" == 1 ]]; then
  printf 'another session staged this\n' >> tracked.txt
  git add tracked.txt
fi
exit "${FAKE_GATE_EXIT:-0}"
FAKE_GATE_EOF
}

# record_cell <结果文件> <格名> <过没过：1 过 0 没过> <没过时写什么>
record_cell() {
  if [[ "$3" == 1 ]]; then
    printf 'PASS\t%s\t\n' "$2" >> "$1"
  else
    printf 'FAIL\t%s\t%s\n' "$2" "$4" >> "$1"
  fi
}

# run_cells <被测脚本> <结果文件> <临时目录>：三种局面各起一个临时仓，逐格记过没过
run_cells() {
  local target="$1" results="$2" scratch="$3"
  local repository record log gate_rc expected_tree sentinel_commit ref_type ref_tree arguments environment_tree ref_now
  : > "$results"

  # ── 全绿 ──
  repository="$scratch/all-green"; record="$scratch/all-green-record"; log="$scratch/all-green.log"
  mkdir -p "$record"; make_repository "$repository"
  expected_tree="$(git -C "$repository" write-tree)"
  gate_rc=0
  (cd "$repository" && env FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "全绿：退出码 0" "$([[ $gate_rc == 0 ]] && echo 1 || echo 0)" "退出码 $gate_rc"
  ref_type="$(git -C "$repository" cat-file -t refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "全绿：ref 指向一个提交" "$([[ "$ref_type" == commit ]] && echo 1 || echo 0)" "ref 的对象类型是「${ref_type:-没有这条 ref}」"
  ref_tree="$(git -C "$repository" rev-parse -q --verify 'refs/sop/staged-green^{tree}' 2>/dev/null)"
  record_cell "$results" "全绿：ref 的树就是开跑时的暂存树" "$([[ "$ref_tree" == "$expected_tree" ]] && echo 1 || echo 0)" "ref 的树 ${ref_tree:-（没有）}，开跑时的暂存树 $expected_tree"
  arguments="$(head -1 "$record/arguments" 2>/dev/null)"
  record_cell "$results" "全绿：假门禁收到 --staged" "$([[ "$arguments" == --staged ]] && echo 1 || echo 0)" "第一个参数是「${arguments:-（没收到）}」"
  environment_tree="$(head -1 "$record/staged-tree-in-environment" 2>/dev/null)"
  record_cell "$results" "全绿：环境里的暂存树对得上" "$([[ "$environment_tree" == "$expected_tree" ]] && echo 1 || echo 0)" "SINGLEFS_STAGED_TREE=「${environment_tree}」，开跑时的暂存树 $expected_tree"

  # ── 有红 ──
  repository="$scratch/red"; record="$scratch/red-record"; log="$scratch/red.log"
  mkdir -p "$record"; make_repository "$repository"
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=1 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "有红：退出码原样转手" "$([[ $gate_rc == 1 ]] && echo 1 || echo 0)" "假门禁退 1，外壳退 $gate_rc"
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "有红：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}"

  # ── 跑的过程中暂存区变了 ──
  repository="$scratch/index-moved"; record="$scratch/index-moved-record"; log="$scratch/index-moved.log"
  mkdir -p "$record"; make_repository "$repository"
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 FAKE_GATE_TOUCH_INDEX=1 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "暂存区变了：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}（外壳退 $gate_rc）"
}

# apply_mutation <原脚本> <副本> <原文> <替换文>：原文恰好命中一次才写副本；否则退 3 并打印命中次数
apply_mutation() {
  python3 - "$@" <<'PY'
import sys
source_path, copy_path, original, replacement = sys.argv[1:5]
text = open(source_path, encoding='utf-8').read()
occurrences = text.count(original)
if occurrences != 1:
    print(occurrences)
    sys.exit(3)
with open(copy_path, 'w', encoding='utf-8') as copy_file:
    copy_file.write(text.replace(original, replacement, 1))
PY
}

failures=0

# ① 每一条改坏的副本，都必须让它点名的那一格判红
for mutation_index in "${!MUTATION_NAMES[@]}"; do
  name="${MUTATION_NAMES[$mutation_index]}"; cell="${MUTATION_CELLS[$mutation_index]}"
  scratch="$work/mutation-$mutation_index"; mkdir -p "$scratch"
  if ! occurrences="$(apply_mutation "$TARGET" "$scratch/gate-staged.sh" "${MUTATION_ORIGINALS[$mutation_index]}" "${MUTATION_REPLACEMENTS[$mutation_index]}")"; then
    echo "  ✗ 改坏「$name」没施加上：原文在 $TARGET 里命中 ${occurrences:-?} 次（要恰好 1 次），这一格的判别力没证"   # gate-lint:detail
    failures=$((failures + 1)); continue
  fi
  run_cells "$scratch/gate-staged.sh" "$scratch/results.tsv" "$scratch"
  if ! grep -qF "FAIL	$cell	" "$scratch/results.tsv"; then
    echo "  ✗ 改坏「$name」之后，「$cell」那一格照样过——这一格分不出好坏"   # gate-lint:detail
    failures=$((failures + 1))
  fi
done

# ② 原样那一份，每一格都要过
run_cells "$TARGET" "$work/results.tsv" "$work/intact"
cells_checked="$(wc -l < "$work/results.tsv")"
while IFS=$'\t' read -r verdict cell detail; do
  [[ "$verdict" == FAIL ]] || continue
  echo "  ✗ 「$cell」没过：$detail"   # gate-lint:detail
  failures=$((failures + 1))
done < "$work/results.tsv"

if (( failures )); then
  echo "  ✗ gate-staged.sh 自证没过：$failures 处（上面逐条列出）"   # gate-lint:summary
  echo "  → 怎么办：原样那一份有格没过，就修 gate-staged.sh 让它守回文件头那三条规矩；"
  echo "    改坏的副本某一格照样过，就是那一格分不出好坏，补一个会被它抓到的局面；原文命中不是一次，就把 mutation 那一行的原文照 gate-staged.sh 今天的写法改。"
  exit 1
fi
cells_all_green="$(grep -c $'\t全绿：' "$work/results.tsv")"
cells_red="$(grep -c $'\t有红：' "$work/results.tsv")"
cells_index_moved="$(grep -c $'\t暂存区变了：' "$work/results.tsv")"
echo "  ✓ gate-staged.sh 自证通过（查了 $cells_checked 格：全绿 $cells_all_green 格、有红 $cells_red 格、跑的过程中暂存区变了 $cells_index_moved 格；${#MUTATION_NAMES[@]} 份改坏的副本各让点名的那一格判红）"
