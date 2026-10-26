#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command git
# `gate-staged.sh` 的自证：在临时小仓里用一个假 gate.sh 造四种局面，逐格核 refs/sop/staged-green 前不前移、交给 gate.sh 的环境；格数由成功行现算。
#
#   gate-staged-selftest.sh [被测的 gate-staged.sh]    不给就测同目录那一份
#
# 四种局面，对着 gate-staged.sh 文件头那三条规矩与交给 gate.sh 的两样：
#   全绿                 ⇒ 退出码 0；ref 指向一个提交，那个提交的树就是开跑时的暂存树（不是 HEAD 的树）；
#                          假 gate.sh 收到的第一个参数是 --staged，环境里的 SINGLEFS_STAGED_TREE 与 GATE_STAGED_TREE 都是那棵暂存树，
#                          CARGO_TARGET_DIR 是没给 SINGLEFS_GATE_TARGET_DIR 时这一趟自己建的 <日志目录>/gate-staged-*/target，跑完不在了
#   有红                 ⇒ 退出码原样转手；ref 不动；这一趟的编译目录跑完不在了
#   被 TERM 停           ⇒ 假 gate.sh 睡着时只给外壳发 TERM：外壳等它退出后退 143，这一趟的编译目录不在了
#   跑的过程中暂存区变了 ⇒ ref 不动（这时退几由 gate-staged.sh 自己定，这里不钉）
#   给了 GATE_BASE       ⇒ 假 gate.sh 全绿、退出码 0，ref 不动，输出里有「不前移」；同一趟给了 SINGLEFS_GATE_TARGET_DIR，CARGO_TARGET_DIR 照它
#   轻档（--light）      ⇒ 临时仓的门禁目录里放一道绿、一道红、一道名字是重型的：不起假 gate.sh，退出码 1 转手，ref 不动，重型那一道没起、报本次未跑
#                          （轻档本身在 scripts/gate/staged-tree-worktree.sh light-stages，它的自证另核树与并行；这里只核 gate-staged.sh 那一层）
#
# 每一格都要先证明会红：把被测脚本拷一份，按 mutation 那几行逐条改坏（原文必须恰好命中一次，
# 命中不是一次就是锚点腐化、当场判红），改坏的每一份都必须让它点名的那一格判红；原样那一份每一格都要过。
# 假 gate.sh 与临时仓都在 mktemp 出来的目录里，碰不到本仓的暂存区与 ref。
# 日志出两份（gate-staged.sh 文件头「日志出两份」，日志目录经 SINGLEFS_GATE_STAGED_LOG_DIRECTORY 指到每个局面自己的临时目录）：假 gate.sh 打一行 ✓，退非 0 时再打 ✗ 与 → 出路；
#   全绿 ⇒ 日志目录里有 .log 与 .problems.log、输出里点名错误警告日志、它末行写退出码 0；有红 ⇒ 错误警告日志带行号收那一行 ✗ 与它的 → 出路、输出里的出路先指到它。
set -uo pipefail
# 门禁要是在 git 钩子里跑，这几个变量会指着本仓的索引与对象库；不清掉，临时仓的 git 命令会写到本仓头上。
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_PREFIX GIT_NAMESPACE
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
export GIT_AUTHOR_NAME=selftest GIT_AUTHOR_EMAIL=selftest@example.invalid GIT_COMMITTER_NAME=selftest GIT_COMMITTER_EMAIL=selftest@example.invalid
# 用户自己的全局配置（签名提交、默认分支名、钩子路径）不许影响临时仓里的判定
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
# 门禁 code-tooling 跑这份自证时，外层多半就是一趟 gate-staged.sh，环境里已经导出了 SINGLEFS_STAGED_TREE；
# 留着它，被测脚本漏了 export 也照样传得下去，「暂存树不交给门禁」那一条改坏就抓不到。
unset SINGLEFS_STAGED_TREE FAKE_GATE_RECORD FAKE_GATE_EXIT FAKE_GATE_TOUCH_INDEX
# 同理：外层那一趟导出的 GATE_STAGED_TREE、CARGO_TARGET_DIR 与调用方设的 GATE_BASE、SINGLEFS_GATE_TARGET_DIR 不许漏进被测的那一趟
unset GATE_STAGED_TREE CARGO_TARGET_DIR GATE_BASE SINGLEFS_GATE_TARGET_DIR

TARGET="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/gate-staged.sh}"
if [[ ! -f "$TARGET" ]]; then
  echo "  ✗ 找不到被测的 gate-staged.sh：$TARGET"
  echo "  → 怎么办：不给参数就测同目录那一份；给参数就写被测脚本的路径。"
  exit 2
fi
# 被测脚本所在的那棵树：被测的要正是 <树根>/scripts/gate/gate-staged.sh（run_cells 照这个相对路径起它）
TARGET_TREE="$(cd "$(dirname "$TARGET")/../.." && pwd)"
if [[ ! "$TARGET_TREE/scripts/gate/gate-staged.sh" -ef "$TARGET" ]]; then
  echo "  ✗ 被测的 gate-staged.sh 不在一棵树的 scripts/gate/ 底下：$TARGET"
  echo "  → 怎么办：给 <树根>/scripts/gate/gate-staged.sh 那样摆着的一份（副本也照这个相对位置摆，旁边要有 .claude/scripts 与规范副本）。"
  exit 2
fi
work="$(mktemp -d)"
trap 'rm -rf "${work:?}"' EXIT

MUTATION_NAMES=(); MUTATION_CELLS=(); MUTATION_ORIGINALS=(); MUTATION_REPLACEMENTS=()
mutation() { # mutation <改坏的是什么> <必须因此判红的那一格> <原文> <替换文>
  MUTATION_NAMES+=("$1"); MUTATION_CELLS+=("$2"); MUTATION_ORIGINALS+=("$3"); MUTATION_REPLACEMENTS+=("$4")
}
mutation "有红也前移"               "有红：ref 不动"                   'elif [[ $gate_rc -eq 0 ]]; then'                   'elif true; then'
mutation "给了 GATE_BASE 也前移"     "给了 GATE_BASE：ref 不动"         'if [[ $gate_rc -eq 0 && -n "${GATE_BASE:-}" ]]; then' 'if false; then'
mutation "暂存树不经 GATE_STAGED_TREE 交给上游" "全绿：环境里的 GATE_STAGED_TREE 对得上" 'export GATE_STAGED_TREE'                 'true'
mutation "编译目录不交给门禁"        "全绿：CARGO_TARGET_DIR 是这一趟自己建的" 'run_build_directory_begin "$gate_log_name" || exit 2' 'true'
mutation "跑完不删编译目录"          "全绿：跑完编译目录删了"           'run_build_directory_end ||'                       'true ||'
mutation "收尾不删编译目录"          "被 TERM 停：退 143、编译目录删了" 'trap gate_staged_finish EXIT'                     'true'
mutation "不照调用方给的编译目录"    "给了 GATE_BASE：CARGO_TARGET_DIR 照 SINGLEFS_GATE_TARGET_DIR" '${SINGLEFS_GATE_TARGET_DIR:-' '${SINGLEFS_GATE_TARGET_DIR_UNREAD:-'
mutation "不核跑的过程中暂存区变没变" "暂存区变了：ref 不动"             'if [[ "$now_tree" != "$staged_tree" ]]; then'   'if false; then'
mutation "ref 指向裸树不指向提交"    "全绿：ref 指向一个提交"           'git update-ref refs/sop/staged-green "$green_commit"' 'git update-ref refs/sop/staged-green "$staged_tree"'
mutation "判的是 HEAD 的树不是暂存树" "全绿：ref 的树就是开跑时的暂存树" 'staged_tree="$(git write-tree)"'                 'staged_tree="$(git rev-parse "HEAD^{tree}")"'
mutation "不带 --staged 跑门禁"      "全绿：假门禁收到 --staged"        'bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"' 'bash "$ROOT/.claude/scripts/gate.sh" "$@"'
mutation "退出码不转手"              "有红：退出码原样转手"             'exit $gate_rc'                                     'exit 0'
mutation "暂存树不交给门禁"          "全绿：环境里的暂存树对得上"       'export SINGLEFS_STAGED_TREE'                       'true'
mutation "全绿也退非 0"              "全绿：退出码 0"                   'exit $gate_rc'                                     'exit 1'
mutation "门禁输出不经 two-logs.sh"  "全绿：日志出两份、路径写进输出"   'bash "$two_logs_script" "$gate_log_name" -- bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"' 'bash "$ROOT/.claude/scripts/gate.sh" --staged "$@"'
mutation "红了不指到错误警告日志"    "有红：出路先指到错误警告日志"     '  echo "     → 怎么办：先看错误警告日志 $gate_log_name.problems.log' '  true "     → 怎么办：先看错误警告日志 $gate_log_name.problems.log'
mutation "轻档照跑整轮"              "轻档：不起 gate.sh"               'if ((light_tier)); then'                           'if false; then'
mutation "轻档退出码不转手"          "轻档：退出码原样转手"             'exit "$light_rc"'                                  'exit 0'

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
printf '%s\n' "${GATE_STAGED_TREE:-}" > "$FAKE_GATE_RECORD/gate-staged-tree-in-environment"
printf '%s\n' "${CARGO_TARGET_DIR:-}" > "$FAKE_GATE_RECORD/cargo-target-directory"
[[ -z "${FAKE_GATE_SLEEP:-}" ]] || sleep "$FAKE_GATE_SLEEP"
if [[ "${FAKE_GATE_TOUCH_INDEX:-0}" == 1 ]]; then
  printf 'another session staged this\n' >> tracked.txt
  git add tracked.txt
fi
echo "  ✓ 假门禁的一格过了"
if [[ "${FAKE_GATE_EXIT:-0}" != 0 ]]; then
  echo "  ✗ 假门禁的一格红了"
  echo "     → 怎么办：改假门禁（自证用）"
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

# run_cells <被测树根> <结果文件> <临时目录>：被测的是 <树根>/scripts/gate/gate-staged.sh；四种局面各起一个临时仓，逐格记过没过
run_cells() {
  local target="$1/scripts/gate/gate-staged.sh" results="$2" scratch="$3"
  local repository record log gate_rc expected_tree sentinel_commit ref_type ref_tree arguments environment_tree ref_now target_directory
  : > "$results"

  # ── 全绿 ──
  repository="$scratch/all-green"; record="$scratch/all-green-record"; log="$scratch/all-green.log"
  mkdir -p "$record"; make_repository "$repository"
  expected_tree="$(git -C "$repository" write-tree)"
  gate_rc=0
  (cd "$repository" && env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "全绿：退出码 0" "$([[ $gate_rc == 0 ]] && echo 1 || echo 0)" "退出码 $gate_rc"
  ref_type="$(git -C "$repository" cat-file -t refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "全绿：ref 指向一个提交" "$([[ "$ref_type" == commit ]] && echo 1 || echo 0)" "ref 的对象类型是「${ref_type:-没有这条 ref}」"
  ref_tree="$(git -C "$repository" rev-parse -q --verify 'refs/sop/staged-green^{tree}' 2>/dev/null)"
  record_cell "$results" "全绿：ref 的树就是开跑时的暂存树" "$([[ "$ref_tree" == "$expected_tree" ]] && echo 1 || echo 0)" "ref 的树 ${ref_tree:-（没有）}，开跑时的暂存树 $expected_tree"
  arguments="$(head -1 "$record/arguments" 2>/dev/null)"
  record_cell "$results" "全绿：假门禁收到 --staged" "$([[ "$arguments" == --staged ]] && echo 1 || echo 0)" "第一个参数是「${arguments:-（没收到）}」"
  environment_tree="$(head -1 "$record/staged-tree-in-environment" 2>/dev/null)"
  record_cell "$results" "全绿：环境里的暂存树对得上" "$([[ "$environment_tree" == "$expected_tree" ]] && echo 1 || echo 0)" "SINGLEFS_STAGED_TREE=「${environment_tree}」，开跑时的暂存树 $expected_tree"
  environment_tree="$(head -1 "$record/gate-staged-tree-in-environment" 2>/dev/null)"
  record_cell "$results" "全绿：环境里的 GATE_STAGED_TREE 对得上" "$([[ "$environment_tree" == "$expected_tree" ]] && echo 1 || echo 0)" "GATE_STAGED_TREE=「${environment_tree}」，外壳 write-tree 那一棵 $expected_tree"
  target_directory="$(head -1 "$record/cargo-target-directory" 2>/dev/null)"
  record_cell "$results" "全绿：CARGO_TARGET_DIR 是这一趟自己建的" "$([[ "$target_directory" == "$record/logs/gate-staged-"*/target ]] && echo 1 || echo 0)" \
    "CARGO_TARGET_DIR=「${target_directory}」，没给 SINGLEFS_GATE_TARGET_DIR 时应当是 $record/logs/gate-staged-<…>/target"
  record_cell "$results" "全绿：跑完编译目录删了" "$([[ -n "$target_directory" && ! -e "$target_directory" ]] && echo 1 || echo 0)" "跑完 $target_directory 还在"

  local log_problems logs_ok logs_last_line
  log_problems="$(ls "$record"/logs/*.problems.log 2>/dev/null | head -n 1)"
  logs_ok=0
  logs_last_line="$(tail -n 1 "$log_problems" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ -n "$log_problems" && -f "${log_problems%.problems.log}.log" ]] && grep -qF "错误警告 $log_problems" "$log" \
     && grep -qE '收了 [0-9]+ 行，全量日志共 [0-9]+ 行，被包的命令退出码 0' <<<"$logs_last_line"; then logs_ok=1; fi
  record_cell "$results" "全绿：日志出两份、路径写进输出" "$logs_ok" \
    "日志目录 $record/logs 里是「$(ls "$record/logs" 2>&1 | tr '\n' ' ')」，输出里点名的错误警告日志是「$(grep -o '错误警告 [^ ]*' "$log" | head -n 1)」"

  # ── 有红 ──
  repository="$scratch/red"; record="$scratch/red-record"; log="$scratch/red.log"
  mkdir -p "$record"; make_repository "$repository"
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=1 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "有红：退出码原样转手" "$([[ $gate_rc == 1 ]] && echo 1 || echo 0)" "假门禁退 1，外壳退 $gate_rc"
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "有红：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}"
  target_directory="$(head -1 "$record/cargo-target-directory" 2>/dev/null)"
  record_cell "$results" "有红：跑完编译目录删了" "$([[ "$target_directory" == "$record/logs/gate-staged-"*/target && ! -e "$target_directory" ]] && echo 1 || echo 0)" "CARGO_TARGET_DIR=「${target_directory}」，跑完$([[ -e "$target_directory" ]] && echo 还在 || echo 不在)"
  log_problems="$(ls "$record"/logs/*.problems.log 2>/dev/null | head -n 1)"
  logs_ok=0
  if [[ -n "$log_problems" ]] && grep -qE '^[0-9]+:  ✗ 假门禁的一格红了$' "$log_problems" && grep -qE '^[0-9]+:     → 怎么办：改假门禁（自证用）$' "$log_problems"; then logs_ok=1; fi
  record_cell "$results" "有红：错误警告日志收了 ✗ 与 → 出路" "$logs_ok" "错误警告日志「${log_problems:-没有}」里没有带行号的「✗ 假门禁的一格红了」与它的「→ 怎么办」"
  logs_ok=0
  [[ -n "$log_problems" ]] && grep -qF "先看错误警告日志 $log_problems" "$log" && logs_ok=1
  record_cell "$results" "有红：出路先指到错误警告日志" "$logs_ok" "输出里没有「先看错误警告日志 ${log_problems:-（没有）}」"

  # ── 被 TERM 停 ──（假 gate.sh 睡 3 秒；记下编译目录之后只给外壳那一个进程号发 TERM）
  repository="$scratch/terminated"; record="$scratch/terminated-record"; log="$scratch/terminated.log"
  mkdir -p "$record"; make_repository "$repository"
  local shell_pid attempt
  (cd "$repository" && exec env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 FAKE_GATE_SLEEP=3 bash "$target" "$repository") > "$log" 2>&1 &
  shell_pid=$!
  for (( attempt = 0; attempt < 600; attempt++ )); do [[ -s "$record/cargo-target-directory" ]] && break; sleep 0.1; done
  kill -TERM "$shell_pid" 2>/dev/null
  if wait "$shell_pid"; then gate_rc=0; else gate_rc=$?; fi
  target_directory="$(head -1 "$record/cargo-target-directory" 2>/dev/null)"
  record_cell "$results" "被 TERM 停：退 143、编译目录删了" "$([[ $gate_rc == 143 && "$target_directory" == "$record/logs/gate-staged-"*/target && ! -e "$target_directory" ]] && echo 1 || echo 0)" \
    "外壳退 $gate_rc（要 143），CARGO_TARGET_DIR=「${target_directory}」，跑完$([[ -e "$target_directory" ]] && echo 还在 || echo 不在)"

  # ── 跑的过程中暂存区变了 ──
  repository="$scratch/index-moved"; record="$scratch/index-moved-record"; log="$scratch/index-moved.log"
  mkdir -p "$record"; make_repository "$repository"
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 FAKE_GATE_TOUCH_INDEX=1 bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "暂存区变了：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}（外壳退 $gate_rc）"

  # ── 给了 GATE_BASE ──
  repository="$scratch/gate-base"; record="$scratch/gate-base-record"; log="$scratch/gate-base.log"
  mkdir -p "$record"; make_repository "$repository"
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 GATE_BASE=HEAD SINGLEFS_GATE_TARGET_DIR="$scratch/given-target" \
    bash "$target" "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "给了 GATE_BASE：退出码 0" "$([[ $gate_rc == 0 ]] && echo 1 || echo 0)" "假门禁退 0，外壳退 $gate_rc"
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "给了 GATE_BASE：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}"
  record_cell "$results" "给了 GATE_BASE：输出写明不前移" "$(grep -q 'GATE_BASE.*不前移' "$log" && echo 1 || echo 0)" "输出里没有「…GATE_BASE…不前移」那一句"
  target_directory="$(head -1 "$record/cargo-target-directory" 2>/dev/null)"
  record_cell "$results" "给了 GATE_BASE：CARGO_TARGET_DIR 照 SINGLEFS_GATE_TARGET_DIR" "$([[ "$target_directory" == "$scratch/given-target" ]] && echo 1 || echo 0)" \
    "CARGO_TARGET_DIR=「${target_directory}」，调用方给的是 $scratch/given-target"

  # ── 轻档 ──
  repository="$scratch/light"; record="$scratch/light-record"; log="$scratch/light.log"
  mkdir -p "$record"; make_repository "$repository"
  mkdir -p "$repository/.claude/gate.d"
  printf '#!/usr/bin/env bash\necho "  ✓ 轻档样本绿（查了 1 项）"\n' > "$repository/.claude/gate.d/doc-green.sh"
  printf '#!/usr/bin/env bash\necho "  ✗ 轻档样本红"\necho "  → 怎么办：样本"\nexit 1\n' > "$repository/.claude/gate.d/doc-red.sh"
  printf '#!/usr/bin/env bash\ntouch "%s/heavy-ran"\n' "$record" > "$repository/.claude/gate.d/checker-tier-crash-point-replay.sh"
  git -C "$repository" add .claude/gate.d
  sentinel_commit="$(git -C "$repository" commit-tree 'HEAD^{tree}' -m sentinel)"
  git -C "$repository" update-ref refs/sop/staged-green "$sentinel_commit"
  gate_rc=0
  (cd "$repository" && env SINGLEFS_GATE_STAGED_LOG_DIRECTORY="$record/logs" FAKE_GATE_RECORD="$record" FAKE_GATE_EXIT=0 bash "$target" --light "$repository") > "$log" 2>&1 || gate_rc=$?
  record_cell "$results" "轻档：不起 gate.sh" "$([[ ! -e "$record/arguments" ]] && echo 1 || echo 0)" "假 gate.sh 被起了，收到的参数是「$(tr '\n' ' ' < "$record/arguments" 2>/dev/null)」"
  record_cell "$results" "轻档：退出码原样转手" "$([[ $gate_rc == 1 ]] && echo 1 || echo 0)" "一道红，外壳退 $gate_rc"
  ref_now="$(git -C "$repository" rev-parse -q --verify refs/sop/staged-green 2>/dev/null)"
  record_cell "$results" "轻档：ref 不动" "$([[ "$ref_now" == "$sentinel_commit" ]] && echo 1 || echo 0)" "ref 从 $sentinel_commit 变成了 ${ref_now:-（没有）}"
  record_cell "$results" "轻档：重型那一道没起、报本次未跑" \
    "$([[ ! -e "$record/heavy-ran" ]] && grep -qF 'checker-tier-crash-point-replay.sh：本次未跑（重型' "$log" && grep -qF 'doc-red.sh：红（退出 1）' "$log" && echo 1 || echo 0)" \
    "重型那一道$([[ -e "$record/heavy-ran" ]] && echo 起了 || echo 没起)；输出：$(tail -n 8 "$log" | tr '\n' ' ')"
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
  # 副本摆在与原件相同的相对位置（scripts/ 底下，旁边链上 .claude/scripts 与规范副本）：原件按相对路径 source .claude/scripts/preflight.sh，
  # 摆在别处的话 source 落空、preflight 一步没判还把参数清掉，副本就因为这个而不是因为改坏的那一处出结果
  mkdir -p "$scratch/scripts" "$scratch/scripts/gate" "$scratch/.claude"
  ln -s "$(cd "$(dirname "$TARGET")/../.." && pwd)/.claude/singlefs-ai-sop" "$scratch/.claude/singlefs-ai-sop"
  ln -s "$(cd "$(dirname "$TARGET")/../.." && pwd)/.claude/scripts" "$scratch/.claude/scripts"
  ln -s "$(cd "$(dirname "$TARGET")/../.." && pwd)/scripts/hosts" "$scratch/scripts/hosts"   # 门禁输出出两份日志的包装 scripts/hosts/two-logs.sh
  # 轻档那一层调的 staged-tree-worktree.sh 与它读的重型名单（.claude/hooks/lib_heavy_tests.py）：副本里也要照原样跑得起，轻档那几格才只因改坏的那一处出结果
  ln -s "$(cd "$(dirname "$TARGET")" && pwd)/staged-tree-worktree.sh" "$scratch/scripts/gate/staged-tree-worktree.sh"
  ln -s "$(cd "$(dirname "$TARGET")/../.." && pwd)/.claude/hooks" "$scratch/.claude/hooks"
  if ! occurrences="$(apply_mutation "$TARGET" "$scratch/scripts/gate/gate-staged.sh" "${MUTATION_ORIGINALS[$mutation_index]}" "${MUTATION_REPLACEMENTS[$mutation_index]}")"; then
    echo "  ✗ 改坏「$name」没施加上：原文在 $TARGET 里命中 ${occurrences:-?} 次（要恰好 1 次），这一格的判别力没证"   # gate-lint:detail
    failures=$((failures + 1)); continue
  fi
  run_cells "$scratch" "$scratch/results.tsv" "$scratch"
  if ! grep -qF "FAIL	$cell	" "$scratch/results.tsv"; then
    echo "  ✗ 改坏「$name」之后，「$cell」那一格照样过——这一格分不出好坏"   # gate-lint:detail
    failures=$((failures + 1))
  fi
done

# ② 原样那一份，每一格都要过
run_cells "$TARGET_TREE" "$work/results.tsv" "$work/intact"
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
cells_gate_base="$(grep -c $'\t给了 GATE_BASE：' "$work/results.tsv")"
cells_light="$(grep -c $'\t轻档：' "$work/results.tsv")"
cells_terminated="$(grep -c $'\t被 TERM 停：' "$work/results.tsv")"
echo "  ✓ gate-staged.sh 自证通过（查了 $cells_checked 格：全绿 $cells_all_green 格、有红 $cells_red 格、跑的过程中暂存区变了 $cells_index_moved 格、给了 GATE_BASE $cells_gate_base 格、轻档 $cells_light 格、被 TERM 停 $cells_terminated 格；${#MUTATION_NAMES[@]} 份改坏的副本各让点名的那一格判红）"
