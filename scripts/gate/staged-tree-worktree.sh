#!/usr/bin/env bash
# admission: always 每一次调都按此刻的 HEAD 与暂存区建树、按此刻的门禁目录跑，上一次建的树与判过的结论不替这一次作保
# run-condition: command git python3
# HEAD + 暂存区的 git worktree：给只读检查用的那棵树，与门禁 gate-staged.sh 的轻档（只跑不重的门禁）。
# 为什么：.claude/kb/milestone/03-third-txn.md「待优化」第 56、48 行——2026-10-18 收尾只想在暂存树上跑不重的门禁，gate-staged.sh 只能整轮跑、
#   跳不掉刚跑过的重型几道，改成手建 HEAD 加暂存区的 worktree 并行跑；手建的树没带被忽略的规范副本，静态判据两次退 1。
#
#   staged-tree-worktree.sh create [仓根] [--tree <树>] [--path <目录>]
#       建一棵 worktree，内容是那棵树（不给 --tree 取 git write-tree：HEAD + 暂存区；别的会话没暂存的改动与未跟踪文件都不在里面），
#       再把被忽略而跑门禁要的几样从源仓拷进去（COPIED_IGNORED：规范副本 .claude/<族名>/、multi-host.env、qemu.env；源仓有、树里没有的才拷）。
#       路径不给取 ${TMPDIR:-/tmp}/staged-tree-worktree-<仓根路径的哈希前 12 位>；那里已有一棵这份脚本建的（有标记文件 .staged-tree-worktree）就先删掉再建，
#       有别的东西就拒绝（退 2）。末行打「STAGED_TREE_WORKTREE path=<路径> tree=<树>」。树里不提交、不写回源仓，用完 remove。
#   staged-tree-worktree.sh remove [仓根] [--path <目录>]
#       git worktree remove --force 删掉（只删带标记文件的；树里只有拷进去的东西），再删它的目录。
#   staged-tree-worktree.sh light-stages [仓根] --tree <树> --log-directory <目录>
#       gate-staged.sh --light 调的轻档：在一棵私有路径的这种树上跑门禁目录（.claude/gate.d/ 顶层的 *.sh，lib-* 不是阶段）里不重的每一道，
#       重的那几道（.claude/hooks/lib_heavy_tests.py 的 STAGE_KIND，取这份脚本所在仓里那一份：认重型只有那一处）不起、逐道报「本次未跑（重型）」。
#       各道并行起，环境检查那一道（ENVIRONMENT_STAGE，code-tooling 的 stage-order 格要它排在跑测试的阶段之后）等别的都跑完再起；
#       每道的输出写 <日志目录>/<阶段>.log、退出码写 <阶段>.exit，跑完按阶段名的次序逐道回读、转打、数份数（份数对不上整趟判红）。
#       交给各道的环境同 gate.sh --staged 的里层：GATE_BASE 与 GATE_DIFF_BASE 是在源仓上算的 diff 基准（这份脚本所在仓的规范副本里上游 lib.sh 的 diff_base，
#       调用方给了 GATE_BASE 照它），
#       TMPDIR 是这一趟自己的临时目录，GATE_NOT_RUN_FILE 收各道报的「本次未跑」。退 0 绿、77 本次未跑；别的（含准入没过的 78）算红。
#       退出码：0 没有一道红；1 有红或份数对不上；2 用法错、建不了树。跑完删掉那棵树。
#   staged-tree-worktree.sh --selftest   临时小仓里造暂存区、没暂存的改动、未跟踪文件与被忽略的副本，核 create / remove / light-stages
#                                       （gate-staged.sh --light 那一层的格在 gate-staged-selftest.sh 的「轻档」局面里）
#
# 弄坏开关（只给 --selftest 证红用；一次设一个）：STAGED_TREE_WORKTREE_BREAK=
#   head-tree             create 铺 HEAD 的树、不铺暂存树                      自证「树里是 HEAD + 暂存区」红
#   ignored-not-copied    create 不拷被忽略的规范副本与本地配置                自证「被忽略的副本与本地配置拷进去了」红
#   heavy-run             light-stages 不跳重型几道                            自证「重型几道不起、报本次未跑」红
#   red-swallowed         light-stages 红的那道按绿记、整趟退 0                自证「红的那道照报、退 1」红
#   environment-parallel  环境检查那一道与别的同时起                          自证「环境检查那一道等别的跑完再起」红
#   worktree-kept         light-stages 跑完不删树                              自证「跑完树删了」红
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SCRIPT_REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MARKER_NAME=".staged-tree-worktree"
ENVIRONMENT_STAGE="harness-test-environment.sh"
LOCAL_CONFIGS=(multi-host.env qemu.env)
HEAVY_LIBRARY="$SCRIPT_REPOSITORY/.claude/hooks/lib_heavy_tests.py"
BREAK="${STAGED_TREE_WORKTREE_BREAK:-}"

refuse() {
  echo "  ✗ $1"
  echo "  → 怎么办：$2"
  exit 2
}

# 规范副本的族名：源仓 .claude/singlefs-ai-sop/I18N 的 family=，读不到取 singlefs-ai-sop（与 gate.sh --staged 同一个取法）
package_family() {
  local family
  family="$(sed -n 's/^family=//p' "$1/.claude/singlefs-ai-sop/I18N" 2>/dev/null || true)"
  printf '%s' "${family:-singlefs-ai-sop}"
}

default_path() {
  printf '%s/staged-tree-worktree-%s' "${TMPDIR:-/tmp}" "$(printf '%s' "$1" | git -C "$1" hash-object --stdin | cut -c1-12)"
}

# TREE 从调用方索引取出之后，worktree 操作必须使用目标自己的索引与仓定位。
worktree_git() (
  unset GIT_INDEX_FILE GIT_DIR GIT_WORK_TREE GIT_COMMON_DIR
  git "$@"
)

# remove_tree <仓根> <路径>：只删带标记文件的
remove_tree() {
  [[ -e "$2" ]] || return 0
  [[ -f "$2/$MARKER_NAME" ]] || refuse "$2 不是这份脚本建的树（没有 $MARKER_NAME），不删" "换一个 --path；那个目录是谁的交用户看"
  worktree_git -C "$1" worktree remove --force "$2" >/dev/null 2>&1 || true
  rm -rf "${2:?}"
  worktree_git -C "$1" worktree prune >/dev/null 2>&1 || true
  [[ ! -e "$2" ]] || refuse "删不掉 $2" "看权限，或有进程开着它（ls -l /proc/*/cwd 2>/dev/null | grep '$2'）"
}

# create_tree <仓根> <树> <路径>
create_tree() {
  local root="$1" tree="$2" path="$3" family item
  [[ "$(worktree_git -C "$root" cat-file -t "$tree" 2>/dev/null)" == tree ]] || refuse "$tree 不是 $root 里的一棵树" "给 git write-tree 算出来的树对象名，或不给 --tree"
  remove_tree "$root" "$path"
  mkdir -p "$(dirname "$path")" || refuse "建不了 $(dirname "$path")" "看 TMPDIR 指的目录能不能写"
  worktree_git -C "$root" worktree add -q --detach "$path" HEAD >/dev/null 2>&1 || refuse "git worktree add $path 没成" "仓里要有提交；有残留登记就 git -C $root worktree prune 再跑"
  printf 'tree=%s\nsource=%s\n' "$tree" "$root" > "$path/$MARKER_NAME"
  if [[ "$BREAK" != head-tree ]]; then
    worktree_git -C "$path" read-tree --reset -u "$tree" || refuse "把 $tree 铺进 $path 没成" "确认那棵树还在仓里（git cat-file -t $tree）再跑"
  fi
  if [[ "$BREAK" != ignored-not-copied ]]; then
    family="$(package_family "$root")"
    for item in ".claude/$family" "${LOCAL_CONFIGS[@]}"; do
      if [[ -e "$root/$item" && ! -e "$path/$item" ]]; then
        mkdir -p "$(dirname "$path/$item")"
        cp -a "$root/$item" "$path/$item" || refuse "拷 $item 进 $path 没成" "看磁盘空间与权限"
      fi
    done
  fi
}

parse_common() { # 读 [仓根] 与 --tree / --path / --log-directory，落进 ROOT、TREE、TREE_PATH、LOG_DIRECTORY
  ROOT="" TREE="" TREE_PATH="" LOG_DIRECTORY=""
  while (($#)); do
    case "$1" in
      --tree) [[ $# -ge 2 ]] || refuse "--tree 后面缺树" "写成 --tree <树对象名>"; TREE="$2"; shift 2 ;;
      --path) [[ $# -ge 2 ]] || refuse "--path 后面缺目录" "写成 --path <目录>"; TREE_PATH="$2"; shift 2 ;;
      --log-directory) [[ $# -ge 2 ]] || refuse "--log-directory 后面缺目录" "写成 --log-directory <目录>"; LOG_DIRECTORY="$2"; shift 2 ;;
      -*) refuse "认不出的参数 $1" "用法见这份脚本文件头" ;;
      *) [[ -z "$ROOT" ]] || refuse "多给了一个仓根「$1」" "只给一个仓根"; ROOT="$1"; shift ;;
    esac
  done
  ROOT="$(cd "${ROOT:-$SCRIPT_REPOSITORY}" 2>/dev/null && pwd)" || refuse "进不去仓根" "第一个参数给仓根的路径"
  git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1 || refuse "$ROOT 不是 git 工作树" "在这个项目的 git 仓里跑"
}

light_stages() {
  local root="$1" tree="$2" log_directory="$3" run_directory path base heavy_names stage name code
  local -a light=() heavy=() order=() pids=()
  mkdir -p "$log_directory" || refuse "建不了日志目录 $log_directory" "看路径与权限"
  run_directory="$(mktemp -d "${TMPDIR:-/tmp}/staged-light-XXXXXX")" || refuse "建不了临时目录" "看 TMPDIR"
  path="$run_directory/tree"
  create_tree "$root" "$tree" "$path"
  if [[ "$BREAK" != worktree-kept ]]; then
    # 退出时 light_stages 的局部变量已出了作用域：三个值在设 trap 的这一刻展开进去
    trap "remove_tree $(printf '%q' "$root") $(printf '%q' "$path") >/dev/null 2>&1; rm -rf $(printf '%q' "${run_directory:?}")" EXIT
  fi
  if ! heavy_names="$(python3 -c 'import importlib.util, sys
spec = importlib.util.spec_from_file_location("lib_heavy_tests", sys.argv[1]); module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
print("\n".join(sorted(module.STAGE_KIND)))' "$HEAVY_LIBRARY")"; then
    echo "  ✗ 读不出重型的门禁名单（$HEAVY_LIBRARY 的 STAGE_KIND）：分不出哪几道不重，一道都不起"
    echo "  → 怎么办：认重型只在那一处，修好它再跑；要整轮就照 .claude/skills/gate/staged-gate.md 跑整轮门禁"
    return 1
  fi
  for stage in "$path"/.claude/gate.d/*.sh; do
    name="${stage##*/}"
    [[ "$name" == lib-* ]] && continue
    if grep -qxF "$name" <<<"$heavy_names" && [[ "$BREAK" != heavy-run ]]; then heavy+=("$name"); else light+=("$name"); fi
  done
  base="${GATE_BASE:-$(bash -c 'source "$1/.claude/singlefs-ai-sop/scripts/lib.sh" && diff_base "$2"' _ "$SCRIPT_REPOSITORY" "$root")}" \
    || { echo "  ✗ 在源仓上算不出 diff 基准"; echo "  → 怎么办：看上面 diff_base 的原话（GATE_BASE 写错、仓里没有提交）；修好再跑"; return 1; }
  : > "$run_directory/not-run"
  echo "  轻档：在 HEAD + 暂存区的树 ${tree:0:12}（$path）上跑不重的 ${#light[@]} 道，重型 ${#heavy[@]} 道不起；diff 基准 ${base:0:12}；每道的输出在 $log_directory"
  run_one() { # run_one <阶段名>：输出与退出码各写自己的文件
    local exit_code
    if (unset GIT_INDEX_FILE GIT_DIR GIT_WORK_TREE GIT_COMMON_DIR; cd "$path" && GATE_BASE="$base" GATE_DIFF_BASE="$base" TMPDIR="$run_directory" GATE_NOT_RUN_FILE="$run_directory/not-run" \
        bash "$path/.claude/gate.d/$1" "$path") > "$log_directory/$1.log" 2>&1; then exit_code=0; else exit_code=$?; fi
    printf '%s\n' "$exit_code" > "$log_directory/$1.exit"
  }
  rm -f "$log_directory"/*.exit
  for name in "${light[@]}"; do
    if [[ "$name" == "$ENVIRONMENT_STAGE" && "$BREAK" != environment-parallel ]]; then continue; fi
    order+=("$name"); run_one "$name" & pids+=("$!")
  done
  for code in "${pids[@]}"; do wait "$code"; done
  for name in "${light[@]}"; do
    if [[ "$name" == "$ENVIRONMENT_STAGE" && "$BREAK" != environment-parallel ]]; then order+=("$name"); run_one "$name"; fi
  done
  local green=0 red=0 not_run=0 collected=0
  local -a red_names=() not_run_names=()
  for name in $(printf '%s\n' "${order[@]}" | sort); do
    echo "── $name ──"
    cat "$log_directory/$name.log" 2>/dev/null
    [[ -f "$log_directory/$name.exit" ]] || { echo "  ✗ $name 没留退出码"; continue; }   # gate-lint:detail
    collected=$((collected + 1)); code="$(cat "$log_directory/$name.exit")"
    case "$code" in
      0) green=$((green + 1)); echo "$name：绿" ;;
      77) not_run=$((not_run + 1)); not_run_names+=("$name"); echo "$name：本次未跑" ;;
      *) if [[ "$BREAK" == red-swallowed ]]; then green=$((green + 1)); echo "$name：绿"; else red=$((red + 1)); red_names+=("$name"); echo "$name：红（退出 $code）"; fi ;;
    esac
  done
  for name in "${heavy[@]}"; do echo "$name：本次未跑（重型，轻档不起：照 .claude/skills/gate/staged-gate.md 跑整轮，或照 .claude/skills/crash-test/commit-stages.md 跑这几道）"; done
  if [[ -s "$run_directory/not-run" ]]; then sed 's/^/  本次未跑的项：/' "$run_directory/not-run"; fi
  echo "── 汇总：轻档跑了 ${#order[@]} 道（收回 $collected 份退出码）：绿 $green、红 $red、本次未跑 $not_run；重型 ${#heavy[@]} 道本次未跑"
  if (( collected != ${#order[@]} )); then
    echo "  ✗ 派出去 ${#order[@]} 道、收回 $collected 份退出码：有一道的结局丢了，这一趟不算数"
    echo "  → 怎么办：看 $log_directory 里少了哪一份 .exit（那一道被杀、磁盘写不进），整趟重跑"
    return 1
  fi
  if (( red )); then
    echo "  ✗ 红的 $red 道：${red_names[*]}"
    echo "  → 怎么办：逐道看 $log_directory/<阶段>.log 里的红行与它的 → 出路；只重跑红的那几道：bash .claude/gate.d/<阶段>.sh --check <格名>"
    return 1
  fi
  echo "  ✓ 轻档 ${#order[@]} 道里没有红的（绿 $green、本次未跑 $not_run${not_run_names[*]:+：${not_run_names[*]}}）；重型 ${#heavy[@]} 道没跑，不算整轮全绿"
  return 0
}

selftest() {
  local work failures=() root out code tree path log
  work="$(mktemp -d "${TMPDIR:-/tmp}/staged-tree-worktree-selftest-XXXXXX")" || { echo "  ✗ 建不了临时目录"; echo "  → 怎么办：看 TMPDIR"; return 1; }
  # 门禁在 git 钩子里跑时这几个变量指着本仓：清掉，临时仓的 git 命令不写到本仓头上；用户的全局配置不影响临时仓
  unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_PREFIX GIT_NAMESPACE GATE_BASE GATE_STAGED_TREE
  export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_AUTHOR_NAME=s GIT_AUTHOR_EMAIL=s@s GIT_COMMITTER_NAME=s GIT_COMMITTER_EMAIL=s@s
  root="$work/source"
  mkdir -p "$root/.claude/gate.d" "$root/.claude/singlefs-ai-sop/scripts" "$root/scripts/gate" "$root/.claude/scripts" "$work/records"
  printf '/.claude/singlefs-ai-sop/\n/multi-host.env\n' > "$root/.gitignore"
  echo v1 > "$root/a.txt"
  # 被测的脚本与它 source 的几样拷进样本仓；规范副本拷进去当被忽略的那一样（create 要把它拷进树），上游 lib.sh 也从这里取
  cp "$SCRIPT_PATH" "$root/scripts/gate/staged-tree-worktree.sh"
  cp "$SCRIPT_REPOSITORY/.claude/scripts/preflight.sh" "$SCRIPT_REPOSITORY/.claude/scripts/project_preflight.py" "$root/.claude/scripts/"
  mkdir -p "$root/.claude/hooks"
  cp -a "$SCRIPT_REPOSITORY/.claude/hooks/." "$root/.claude/hooks/"   # lib_heavy_tests.py 与它按路径载入的同目录库
  cp -a "$SCRIPT_REPOSITORY/.claude/singlefs-ai-sop/scripts/." "$root/.claude/singlefs-ai-sop/scripts/"
  printf 'family=singlefs-ai-sop\n' > "$root/.claude/singlefs-ai-sop/I18N"
  echo 'PEER=样本' > "$root/multi-host.env"
  # 样本阶段：记起止时刻（环境检查那一道要排在别的都跑完之后）
  stage() { # stage <文件名> <退出码> <正文>
    printf '#!/usr/bin/env bash\necho "start $(date +%%s.%%N)" >> "%s/$(basename "$0").times"\n%s\nsleep 1\necho "end $(date +%%s.%%N)" >> "%s/$(basename "$0").times"\nexit %s\n' \
      "$work/records" "$3" "$work/records" "$2" > "$root/.claude/gate.d/$1"
  }
  stage doc-green.sh 0 '[[ "$(cat staged.txt)" == staged && "$(cat a.txt)" == v1 && ! -e untracked.txt ]] || { echo "  ✗ 树里不是 HEAD + 暂存区"; echo "  → 怎么办：样本"; exit 1; }; echo "  ✓ 样本绿（查了 1 项）"'
  stage doc-red.sh 1 'echo "  ✗ 样本红"; echo "  → 怎么办：样本"'
  stage doc-notrun.sh 77 'echo "doc-notrun：样本本次未跑的一项" >> "$GATE_NOT_RUN_FILE"'
  stage checker-tier-crash-point-replay.sh 0 "touch '$work/records/heavy-ran'"
  stage "$ENVIRONMENT_STAGE" 0 'echo "  ✓ 样本环境（查了 1 项）"'
  (cd "$root" && git init -q && git add -A && git commit -qm base) || { echo "  ✗ 样本仓没建成"; echo "  → 怎么办：看 git 能不能用"; rm -rf "${work:?}"; return 1; }
  echo staged > "$root/staged.txt"; git -C "$root" add staged.txt
  echo dirty > "$root/a.txt"; echo 没跟踪 > "$root/untracked.txt"
  tree="$(git -C "$root" write-tree)"
  # create / remove
  path="$work/created"
  out="$(bash "$root/scripts/gate/staged-tree-worktree.sh" create "$root" --path "$path" 2>&1)"; code=$?
  if (( code != 0 )) || [[ "$(cat "$path/staged.txt" 2>/dev/null)" != staged || "$(cat "$path/a.txt" 2>/dev/null)" != v1 || -e "$path/untracked.txt" ]] \
      || [[ "$out" != *"STAGED_TREE_WORKTREE path=$path tree=$tree"* ]]; then
    failures+=("树里是 HEAD + 暂存区：退 $code，staged.txt「$(cat "$path/staged.txt" 2>/dev/null)」a.txt「$(cat "$path/a.txt" 2>/dev/null)」，输出 $out")
  fi
  if [[ ! -f "$path/.claude/singlefs-ai-sop/I18N" || ! -f "$path/multi-host.env" ]]; then
    failures+=("被忽略的副本与本地配置拷进去了：$path 里规范副本 $([[ -f "$path/.claude/singlefs-ai-sop/I18N" ]] && echo 在 || echo 不在)、multi-host.env $([[ -f "$path/multi-host.env" ]] && echo 在 || echo 不在)")
  fi
  bash "$root/scripts/gate/staged-tree-worktree.sh" remove "$root" --path "$path" >/dev/null 2>&1
  local listed
  listed="$(git -C "$root" worktree list --porcelain)"   # 先落到变量再 grep：pipefail 下以 grep -q 收尾的管道会把命中读成没命中
  if [[ -e "$path" ]] || grep -qF "$path" <<<"$listed"; then failures+=("remove 删了树、不在 worktree list 里：$listed"); fi
  # light-stages
  log="$work/light-logs"
  out="$(bash "$root/scripts/gate/staged-tree-worktree.sh" light-stages "$root" --tree "$tree" --log-directory "$log" 2>&1)"; code=$?
  if (( code != 1 )) || [[ "$out" != *"doc-red.sh：红（退出 1）"* || "$out" != *"doc-green.sh：绿"* ]]; then failures+=("红的那道照报、退 1：退 $code，输出 $out"); fi
  if [[ -e "$work/records/heavy-ran" || "$out" != *"checker-tier-crash-point-replay.sh：本次未跑（重型"* ]]; then failures+=("重型几道不起、报本次未跑：输出 $out"); fi
  if [[ "$out" != *"doc-notrun.sh：本次未跑"* || "$out" != *"本次未跑的项：doc-notrun：样本本次未跑的一项"* || "$out" != *"轻档跑了 4 道（收回 4 份退出码）：绿 2、红 1、本次未跑 1；重型 1 道本次未跑"* ]]; then
    failures+=("本次未跑与份数照报：输出 $out")
  fi
  local environment_start latest_end
  environment_start="$(sed -n 's/^start //p' "$work/records/$ENVIRONMENT_STAGE.times" 2>/dev/null | head -1)"
  latest_end="$(cat "$work/records"/doc-*.times 2>/dev/null | sed -n 's/^end //p' | sort -n | tail -1)"
  if [[ -z "$environment_start" || -z "$latest_end" ]] || ! python3 -c 'import sys; sys.exit(0 if float(sys.argv[1]) >= float(sys.argv[2]) else 1)' "$environment_start" "$latest_end"; then
    failures+=("环境检查那一道等别的跑完再起：它起于 ${environment_start:-?}，别的最晚终于 ${latest_end:-?}")
  fi
  listed="$(git -C "$root" worktree list --porcelain)"
  if grep -q 'staged-light-' <<<"$listed"; then failures+=("跑完树删了：$listed"); fi
  rm -rf "${work:?}"
  local cells=7
  if ((${#failures[@]})); then
    printf '  ✗ 自证：%s\n' "${failures[@]}"   # gate-lint:detail
    echo "  ✗ 自证 $cells 格里 ${#failures[@]} 处判错"   # gate-lint:summary
    echo "  → 怎么办：看 create_tree / light_stages 的判法；STAGED_TREE_WORKTREE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ staged-tree-worktree 自证通过（$cells 格）：树里是 HEAD + 暂存区、被忽略的副本与本地配置拷进去、remove 删干净；轻档红的照报退 1、重型不起报本次未跑、本次未跑与份数照报、环境检查那一道最后起、跑完树删了"
  return 0
}

command="${1:-}"
shift 2>/dev/null || true
case "$command" in
  --selftest) selftest; exit $? ;;
  create)
    parse_common "$@"
    TREE="${TREE:-$(git -C "$ROOT" write-tree)}" || refuse "算不出暂存树（git write-tree 失败）" "先看 git status"
    TREE_PATH="${TREE_PATH:-$(default_path "$ROOT")}"
    create_tree "$ROOT" "$TREE" "$TREE_PATH"
    echo "  只读检查用这棵树；不提交、不往源仓写，用完：bash $SCRIPT_PATH remove $ROOT --path $TREE_PATH"
    echo "STAGED_TREE_WORKTREE path=$TREE_PATH tree=$TREE" ;;
  remove)
    parse_common "$@"
    TREE_PATH="${TREE_PATH:-$(default_path "$ROOT")}"
    trees_before="$(git -C "$ROOT" worktree list | wc -l)"
    remove_tree "$ROOT" "$TREE_PATH"
    trees_after="$(git -C "$ROOT" worktree list | wc -l)"
    echo "  ✓ 删了 $TREE_PATH（git worktree list 里也没了；登记的工作树 $trees_before 棵 → $trees_after 棵）" ;;
  light-stages)
    parse_common "$@"
    [[ -n "$TREE" && -n "$LOG_DIRECTORY" ]] || refuse "light-stages 要 --tree 与 --log-directory" "写成 light-stages <仓根> --tree <树> --log-directory <目录>（门禁外壳的轻档替你给）"
    light_stages "$ROOT" "$TREE" "$LOG_DIRECTORY"; exit $? ;;
  *) refuse "认不出的子命令「$command」" "用 create / remove / light-stages / --selftest，写法见文件头" ;;
esac
