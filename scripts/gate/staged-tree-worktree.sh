#!/usr/bin/env bash
# admission: always 每一次调都按此刻的 HEAD 与暂存区建树、按此刻的门禁目录跑，上一次建的树与判过的结论不替这一次作保
# run-condition: command git python3
# HEAD + 暂存区的 git worktree：给只读检查用的那棵树。
# 为什么：手建的 HEAD 加暂存区的树没带被忽略的规范副本，静态判据两次退 1；这里建树时把它们一并拷进去。
#
#   staged-tree-worktree.sh create [仓根] [--tree <树>] [--path <目录>]
#       建一棵 worktree，内容是那棵树（不给 --tree 取 git write-tree：HEAD + 暂存区；别的会话没暂存的改动与未跟踪文件都不在里面），
#       再把被忽略而跑门禁要的几样从源仓拷进去（COPIED_IGNORED：规范副本 .claude/<族名>/、multi-host.env、qemu.env；源仓有、树里没有的才拷）。
#       路径不给取 ${TMPDIR:-/tmp}/staged-tree-worktree-<仓根路径的哈希前 12 位>；那里已有一棵这份脚本建的（有标记文件 .staged-tree-worktree）就先删掉再建，
#       有别的东西就拒绝（退 2）。末行打「STAGED_TREE_WORKTREE path=<路径> tree=<树>」。树里不提交、不写回源仓，用完 remove。
#   staged-tree-worktree.sh remove [仓根] [--path <目录>]
#       git worktree remove --force 删掉（只删带标记文件的；树里只有拷进去的东西），再删它的目录。
#   staged-tree-worktree.sh --selftest   临时小仓里造暂存区、没暂存的改动、未跟踪文件与被忽略的副本，核 create / remove
#
# 弄坏开关（只给 --selftest 证红用；一次设一个；--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认）。
# 下一行起写打开后自证红的项（失败行 `✗ ` 之后的句首）：
#   STAGED_TREE_WORKTREE_BREAK=head-tree  create 铺 HEAD 的树、不铺暂存树
#       「自证：树里是 HEAD + 暂存区」
#   STAGED_TREE_WORKTREE_BREAK=ignored-not-copied  create 不拷被忽略的规范副本与本地配置
#       「自证：被忽略的副本与本地配置拷进去了」
set -uo pipefail
# 弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道：命令行第一个词是 --selftest，或祖先里有一个的命令行里带 --selftest（自证起的子进程）才放行；
# 否则下面这些变量有设了的，点名它、说它只在自证里认，退 2
BREAK_VARIABLES=(STAGED_TREE_WORKTREE_BREAK)
inside_selftest() { # 往上数这个进程的祖先，有一个的命令行里带 --selftest 就退 0
  local pid="$PPID" stat words word fields
  while ((pid > 1)); do
    mapfile -d '' -t words 2> /dev/null < "/proc/$pid/cmdline" || return 1
    for word in "${words[@]}"; do [[ "$word" == --selftest ]] && return 0; done
    IFS= read -r stat 2> /dev/null < "/proc/$pid/stat" || return 1
    read -ra fields <<< "${stat##*) }"
    pid="${fields[1]:-0}"
  done
  return 1
}
settle_break_switches() { # settle_break_switches <命令行的词…>
  local name setting=()
  for name in "${BREAK_VARIABLES[@]}"; do [[ -n "${!name:-}" ]] && setting+=("$name"); done
  ((${#setting[@]} == 0)) && return 0
  [[ "${1:-}" == --selftest ]] && return 0
  inside_selftest && return 0
  echo "  ✗ ${setting[*]} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做" >&2
  echo "     → 怎么办：unset ${setting[*]} 再来；要看开关红在哪一项，跑 ${setting[0]}=<项> bash scripts/gate/staged-tree-worktree.sh --selftest" >&2
  exit 2
}
settle_break_switches "$@"
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SCRIPT_REPOSITORY="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MARKER_NAME=".staged-tree-worktree"
LOCAL_CONFIGS=(multi-host.env qemu.env)
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
        cp -dR --preserve=timestamps,links "$root/$item" "$path/$item" || refuse "拷 $item 进 $path 没成" "看磁盘空间与权限"
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

selftest() {
  local work failures=() root out code tree path
  work="$(mktemp -d "${TMPDIR:-/tmp}/staged-tree-worktree-selftest-XXXXXX")" || { echo "  ✗ 建不了临时目录"; echo "  → 怎么办：看 TMPDIR"; return 1; }
  # 门禁在 git 钩子里跑时这几个变量指着本仓：清掉，临时仓的 git 命令不写到本仓头上；用户的全局配置不影响临时仓
  unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_PREFIX GIT_NAMESPACE GATE_BASE GATE_STAGED_TREE
  export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_AUTHOR_NAME=s GIT_AUTHOR_EMAIL=s@s GIT_COMMITTER_NAME=s GIT_COMMITTER_EMAIL=s@s
  root="$work/source"
  mkdir -p "$root/.claude/singlefs-ai-sop/scripts" "$root/scripts/gate" "$root/.claude/scripts"
  printf '/.claude/singlefs-ai-sop/\n/multi-host.env\n' > "$root/.gitignore"
  echo v1 > "$root/a.txt"
  # 被测的脚本与它 source 的几样拷进样本仓；规范副本拷进去当被忽略的那一样（create 要把它拷进树）
  cp "$SCRIPT_PATH" "$root/scripts/gate/staged-tree-worktree.sh"
  cp "$SCRIPT_REPOSITORY/.claude/scripts/preflight.sh" "$SCRIPT_REPOSITORY/.claude/scripts/project_preflight.py" "$root/.claude/scripts/"
  cp -dR --preserve=timestamps,links "$SCRIPT_REPOSITORY/.claude/singlefs-ai-sop/scripts/." "$root/.claude/singlefs-ai-sop/scripts/"   # preflight.sh 按相对路径取规范副本里的 preflight
  printf 'family=singlefs-ai-sop\n' > "$root/.claude/singlefs-ai-sop/I18N"
  echo 'PEER=样本' > "$root/multi-host.env"
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
  rm -rf "${work:?}"
  local cells=3
  if ((${#failures[@]})); then
    printf '  ✗ 自证：%s\n' "${failures[@]}"   # gate-lint:detail
    echo "  ✗ 自证 $cells 格里 ${#failures[@]} 处判错"   # gate-lint:summary
    echo "  → 怎么办：看 create_tree 的判法；STAGED_TREE_WORKTREE_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ staged-tree-worktree 自证通过（$cells 格）：树里是 HEAD + 暂存区、被忽略的副本与本地配置拷进去、remove 删干净"
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
  *) refuse "认不出的子命令「$command」" "用 create / remove / --selftest，写法见文件头" ;;
esac
