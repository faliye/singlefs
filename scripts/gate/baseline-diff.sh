#!/usr/bin/env bash
# admission: always 判的是此刻的主工作区对 --base 给的提交，上一次的结论不替这一次作保
# run-condition: command git python3
# 判「这一格的红是不是这一轮带的」：用主仓里的那一份判法，分别判基准提交的树与主工作区，只报工作区里新增的红行。
# 替掉手工做的那一套（在基准提交的 worktree 里跑那棵树自己的门禁退 127，改成主仓的判法在基准树上跑，再手工比两份输出），每批收尾都要做一次。
#
#   baseline-diff.sh <门禁脚本> [--check <格>[,<格>…]] --base <提交> [--force-gate] [--keep] [--work-directory <目录>] [--ignore <正则>]…
#   baseline-diff.sh --selftest            # 临时仓里造「基准就红」的格与「这一次新红」的格，只报后一个；BASELINE_DIFF_BREAK=<项> 打开后自证退 1
#
#   <门禁脚本>        门禁阶段（.claude/gate.d/<阶段>.sh）或共享 doc-lint（.claude/singlefs-ai-sop/scripts/doc-lint.sh）：
#                     调法一律是 `bash <脚本> [--check <格>] <项目根>`，红行打在 stdout 或 stderr 里（两路并在一起读）。
#                     比 doc-lint 点名共享的那一份：项目里的包装 .claude/scripts/doc-lint.sh 自己写死了项目根，传根给它没用，基准那一趟会判成主工作区。
#   --check           原样转给门禁脚本（可给几次、逗号隔，合并成一个 --check）；不给就是脚本的全部格。门禁脚本不认 --check 的（doc-lint）别给。
#   --base <提交>     基准提交；基准树是它的检出，用 git worktree add --detach 建在工作目录里，跑完删掉。
#   --force-gate      给门禁脚本两趟都带 --force（准入没满足时它退 78；--force 是门禁脚本自己的，baseline-diff 自己的准入是 always，用不上）。
#   --keep            收尾不删工作目录与 worktree，留下两趟的完整输出（base.out、workspace.out），打出路径与删法。
#   --work-directory  工作目录的上一级，默认 $TMPDIR（没设是 /tmp）；里面建 baseline-diff.XXXXXX。
#   --ignore <正则>   比较前把每行里匹配的片段当成会漂的段抹掉，可给几次（正则按 python re）；不给只抹下面几种。
#
# 做法：
#   ① 基准树：git worktree add --detach <工作目录>/base <提交>。基准树里没有的判法脚本（.sh、.py，在 scripts、.claude/gate.d、.claude/hooks、.claude/scripts、
#      .claude/singlefs-ai-sop 底下，不含 fixtures、target、__pycache__）从主工作区拷过去，只补没有的、不覆盖基准树自己有的：这一轮新加的判法依赖的脚本
#      基准树里本来没有，不补会退 127，那是工具缺而不是红。补了几份在报告里写。
#   ② 两趟判的都是主仓里的那一份门禁脚本（绝对路径起），项目根参数不同：基准树的根 / 主工作区的根。基准那一趟清掉 GATE_BASE、GATE_STAGED_FROM、
#      GATE_DIFF_BASE、GATE_STAGED_TREE、SINGLEFS_STAGED_TREE（基准树是干净检出，「这次改动」本来为空），上次红的清单落在工作目录里（SINGLEFS_LAST_RUNS_DIRECTORY），
#      不冲掉主工作区那一份；工作区那一趟设 GATE_BASE=<基准提交>，「这次改动」就是基准到工作区（含未提交、未跟踪）的全部。
#   ③ 红行的认法：去掉颜色码之后，开头是 ✗ 的行，加上同一组里的缩进明细行（组 = 一个 ✗ 行到下一个 ✗ ✓ ! · ── 行或空行；
#      「→」出路行与它缩进更深的续行不算红行；✓ 组底下的缩进行不算）。
#   ④ 逐行比较前先抹掉会漂的段（只用来比，报告里打的仍是工作区那一行的原文）：行号（`文件:12:`、`:5,12`、`第 12 行`、`line 12`、`行号 12`、`#L12`）
#      一律成 #；基准树与主工作区的绝对路径成 <根>；--ignore 给的片段成 #。先逐字比；逐字对不上的，两边剩下的红行再把明确的整数计数换成 # 比一遍：
#      汇总行里只是计数变了（基准 `查了 3 个`、工作区 `查了 4 个`）的，掩数字后基准里找得到，单列「计数变了的红行 N 条（不算新红）」并把基准与工作区两版各打一行，
#      不计入新红、不影响退出码；只掩带「个、条、处、份、项、行、道、笔、格、次、批、件」量词的整数计数，文件名、标识符、校验值的数字保留。
#   ⑤ 按行数比较（多重集）：基准有 2 条同样的红行、工作区有 3 条，报 1 条。工作区有而基准没有的，就是这一轮带的。
#
# 退出码：0 没有新增的红行（基准红的原样留着、只差数字的计数变了的红行不算）；1 有新增的红行（逐行列出）；2 参数或环境不对；
#         3 判不了——某一趟的门禁脚本退出码不是 0 / 1 / 77（工具缺、崩了、准入没满足退 78），或退 1 却一行红行都抽不出；
#         77 本次未跑——工作区那一趟退 77，或两趟都退 77。基准那一趟退 77 不碍事（基准没有对象，工作区的红行全算新增）。
# 起它的命令里带 SINGLEFS_RUN_SCOPE（照 .claude/hooks/run-scope-guard.sh 文件头）：门禁脚本是从这里面起的，声明由起它的这一条命令负责。
# 弄坏开关 BASELINE_DIFF_BREAK=<项>（只给证红用，一次一个；自证退 1）：
#   report-all          不比，工作区所有红行都报：自证里「基准就红的格不报」那一项红
#   keep-line-numbers   不抹行号：基准那一格的红行在工作区里挪了行号，被当成新红行，「基准就红的格不报」与「只挪了行号不报」两项红
#   no-tool-overlay     基准树里没有的判法脚本不补：基准那一趟退 127，「基准树缺判法脚本照用主仓的」与「只报新红的格」两项红
#   counts-as-new       不做掩数字那一遍：只差数字的红行被当成新红，「计数变了不报新红」那几项红
#   worktree-left       收尾不删 worktree 与工作目录：「收尾删干净」那一项红
#   sandbox-left        自证只在里面那一趟（自证跑自证，TMPDIR 指到新空目录）退出时不删自己的临时目录：「自证跑完不留临时目录」那一项红；
#                       外面这一趟的临时目录照删，所以开着它跑完也不会在 $TMPDIR 里留东西
set -uo pipefail
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_PREFIX GIT_NAMESPACE
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
BREAK="${BASELINE_DIFF_BREAK:-}"
TOOLING_DIRECTORIES=(scripts .claude/gate.d .claude/hooks .claude/scripts .claude/singlefs-ai-sop)

die() { # die <卡在哪> <下一步>
  echo "  ✗ $1"
  echo "     → 怎么办：$2"
  exit 2
}

# overlay_missing_tools <主工作区根> <基准树根>：基准树里没有的判法脚本从主工作区拷过去；打出补了几份（只数字走 stdout）
overlay_missing_tools() {
  local main_root="$1" base_tree="$2" tooling_directory relative copied=0
  [[ "$BREAK" != no-tool-overlay ]] || { echo 0; return 0; }
  for tooling_directory in "${TOOLING_DIRECTORIES[@]}"; do
    [[ -d "$main_root/$tooling_directory" ]] || continue
    while IFS= read -r -d '' relative; do
      relative="${relative#"$main_root"/}"
      [[ -e "$base_tree/$relative" ]] && continue
      mkdir -p "$base_tree/$(dirname "$relative")" || return 1
      cp -p "$main_root/$relative" "$base_tree/$relative" || return 1
      copied=$((copied + 1))
    done < <(find "$main_root/$tooling_directory" \( -name fixtures -o -name target -o -name 'target-*' -o -name __pycache__ -o -name .git -o -name node_modules \) -prune \
               -o -type f \( -name '*.sh' -o -name '*.py' \) -print0)
  done
  echo "$copied"
}

# compare_outputs：读两趟的输出、抽红行、逐行比、打报告，退出码就是整个脚本的退出码（0 / 1 / 3 / 77）
compare_outputs() {
  python3 - "$@" <<'PY'
import collections
import re
import sys

(base_out, work_out, base_root, work_root, base_rc, work_rc, gate_label, base_label, tool_count, *extra_ignores) = sys.argv[1:]
base_rc, work_rc, tool_count = int(base_rc), int(work_rc), int(tool_count)
import os
break_item = os.environ.get('BASELINE_DIFF_BREAK', '')

ANSI = re.compile(r'\x1b\[[0-9;?]*[A-Za-z]')
HEADER_MARKS = ('✗', '✓', '!', '·', '⚠', '──')


def read_lines(path):
    with open(path, encoding='utf-8', errors='replace') as handle:
        return ANSI.sub('', handle.read()).split('\n')


def indent_of(line):
    return len(line) - len(line.lstrip(' '))


def red_lines(lines):
    """红行：✗ 行，加同一组里的缩进明细行；「→」出路行与它更深的续行不算。"""
    found = []
    in_red_group = False
    howto_indent = None
    for line in lines:
        stripped = line.strip()
        if not stripped:
            in_red_group, howto_indent = False, None
            continue
        lead = indent_of(line)
        is_red_header = stripped.startswith('✗')
        is_other_header = lead <= 3 and stripped.startswith(HEADER_MARKS) and not is_red_header
        if is_red_header:
            in_red_group, howto_indent = True, None
            found.append(line)
        elif is_other_header:
            in_red_group, howto_indent = False, None
        elif not in_red_group:
            continue
        elif stripped.startswith('→'):
            howto_indent = lead
        elif howto_indent is not None and lead >= howto_indent + 4:
            continue
        elif lead <= 3:
            in_red_group, howto_indent = False, None
        else:
            howto_indent = None
            found.append(line)
    return found


ROOTS = sorted({root for root in (base_root, work_root) if root}, key=len, reverse=True)


def normalize(line):
    """只用来比：抹掉行号这类会漂的段与两棵树的绝对路径。"""
    text = line.strip()
    for root in ROOTS:
        text = text.replace(root, '<根>')
    if break_item != 'keep-line-numbers':
        text = re.sub(r'(?<=[^\s:]):\d+(?:[,，\-]\d+)*(?=[:：\s)）,，]|$)', ':#', text)
        text = re.sub(r'第\s*\d+\s*行', '第#行', text)
        text = re.sub(r'(?i)\bline\s+\d+', 'line #', text)
        text = re.sub(r'行号\s*\d+', '行号#', text)
        text = re.sub(r'#L\d+', '#L#', text)
    for pattern in extra_ignores:
        text = re.sub(pattern, '#', text)
    return text


def tail_of(path, count=8):
    lines = [line for line in read_lines(path) if line.strip()]
    return lines[-count:]


print(f'baseline-diff：{gate_label}')
print(f'  基准 {base_label}：门禁脚本退 {base_rc}')
print(f'  工作区 {work_root}：门禁脚本退 {work_rc}')
if tool_count:
    print(f'  基准树里没有、照用主仓那一份的判法脚本 {tool_count} 份（拷进基准树的临时检出里，跑完随 worktree 一起删）')

abnormal = [(name, rc, path) for name, rc, path in (('基准树', base_rc, base_out), ('工作区', work_rc, work_out)) if rc not in (0, 1, 77)]
if abnormal:
    for name, rc, path in abnormal:
        print(f'  ✗ {name}那一趟门禁脚本退 {rc}（不是 0 / 1 / 77），判法本身没跑成，不是「没有新增的红」')   # gate-lint:detail
        for line in tail_of(path):
            print(f'      | {line}')   # gate-lint:detail
    print('     → 怎么办：看上面输出的末尾——127 是命令找不到（判法依赖的东西基准树里没有、又不在补的范围里），78 是准入没满足（加 --force-gate），')
    print('               别的是脚本自己崩了；修好它再跑，别拿这一次当通过。要看完整输出加 --keep。')
    sys.exit(3)

if work_rc == 77:
    print('  ! 工作区那一趟门禁脚本退 77（本次未跑，无对象可判），不是通过')
    sys.exit(77)

base_red = red_lines(read_lines(base_out))
work_red = red_lines(read_lines(work_out))
print(f'  红行：基准 {len(base_red)} 条，工作区 {len(work_red)} 条')

if work_rc == 1 and not work_red:
    print('  ✗ 工作区那一趟退 1，却一行红行（开头是 ✗ 的行）都抽不出')
    print('     → 怎么办：这道门禁脚本的红不是用叉号打的，这个比对认不得；直接看它的输出（加 --keep 留下 workspace.out）。')
    sys.exit(3)
if base_rc == 1 and not base_red:
    print('  ! 基准那一趟退 1，却一行红行都抽不出：基准按「没有红行」比，工作区的红行会全部算新增')

def mask_numbers(line):
    """只抹明确带量词的整数计数；文件身份与实际数值不同必须报新红。"""
    return re.sub(r'(?<![A-Za-z0-9_./])\d+(?=\s*(?:个|条|处|份|项|行|道|笔|格|次|批|件))', '#', normalize(line))


# 第一遍逐字比；第二遍只抹明确的整数计数。产物名 e9- 换成 e10-、实际校验值变化仍算新红。
remaining = collections.Counter(normalize(line) for line in base_red)
base_by_exact = collections.defaultdict(list)
for line in base_red:
    base_by_exact[normalize(line)].append(line)
if break_item == 'report-all':
    remaining = collections.Counter()
unmatched = []
for line in work_red:
    key = normalize(line)
    if remaining[key] > 0:
        remaining[key] -= 1
        base_by_exact[key].pop(0)
    else:
        unmatched.append(line)
kept = len(work_red) - len(unmatched)

base_left = collections.defaultdict(list)
for lines in base_by_exact.values():
    for line in lines:
        base_left[mask_numbers(line)].append(line)
if break_item == 'report-all':
    base_left = collections.defaultdict(list)
count_changed, new_lines = [], []
for line in unmatched:
    candidates = base_left.get(mask_numbers(line)) if break_item != 'counts-as-new' else None
    if candidates:
        count_changed.append((candidates.pop(0), line))
    else:
        new_lines.append(line)
vanished = sum(len(lines) for lines in base_left.values())
changed_note = f'；计数变了的红行 {len(count_changed)} 条不算新红' if count_changed else ''

if count_changed:
    print(f'  · 计数变了的红行 {len(count_changed)} 条（只差带量词的整数计数，不算新红；基准与工作区两版都列在下面，数变得对不对自己看一眼）：')
    for base_line, work_line in count_changed:
        print(f'      基准：{base_line.strip()}')   # gate-lint:detail
        print(f'      工作区：{work_line.strip()}')   # gate-lint:detail
if new_lines:
    print(f'  ✗ 工作区新增的红行 {len(new_lines)} 条（基准没有、工作区有；行号这类会漂的段已忽略；基准红的原样留着 {kept} 条、已消 {vanished} 条{changed_note}）：')
    for line in new_lines:
        print(f'      {line.strip()}')   # gate-lint:detail
    print('     → 怎么办：这几条是这一轮带来的，按它们自己的出路修；基准就红的那些不在这里，不归这一轮。要看某一格的完整输出加 --keep。')
    sys.exit(1)
print(f'  ✓ 工作区没有新增的红行（新增的红行 0 条；基准红的原样留着 {kept} 条、已消 {vanished} 条{changed_note}）')
sys.exit(0)
PY
}

# run_cells <门禁脚本> <基准树根> <主工作区根> <基准提交> <工作目录>：两趟输出落 <工作目录>/base.out 与 workspace.out，退出码落 base.rc 与 workspace.rc
run_gates() {
  local gate="$1" base_tree="$2" main_root="$3" base_commit="$4" work="$5" base_rc workspace_rc
  shift 5
  (
    cd "$base_tree" || exit 2
    env -u GATE_BASE -u GATE_STAGED_FROM -u GATE_DIFF_BASE -u GATE_STAGED_TREE -u SINGLEFS_STAGED_TREE \
      SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
      bash "$gate" ${gate_arguments[@]+"${gate_arguments[@]}"} "$base_tree" ${gate_force[@]+"${gate_force[@]}"}
  ) > "$work/base.out" 2>&1
  base_rc=$?
  printf '%s\n' "$base_rc" > "$work/base.rc"
  (
    cd "$main_root" || exit 2
    GATE_BASE="$base_commit" bash "$gate" ${gate_arguments[@]+"${gate_arguments[@]}"} "$main_root" ${gate_force[@]+"${gate_force[@]}"}
  ) > "$work/workspace.out" 2>&1
  workspace_rc=$?
  printf '%s\n' "$workspace_rc" > "$work/workspace.rc"
}

# ── 自证 ────────────────────────────────────────────────────────────────────
# 临时仓里造几种局面，逐项核；格数由成功行现算。假门禁三个：fake-count-gate.sh（汇总红行带计数，基准 3、工作区 4；extra.txt 在就多一条不带数的明细）、fake-gate.sh（带 --check，格甲判 a.txt、格乙判 b.txt 且靠 scripts/judge-b.sh）、
# fake-doc-lint.sh（像 doc-lint：只收项目根，不认 --check）。
# 临时目录的清理：EXIT 陷阱在函数返回之后才跑，那时函数的 local 变量已经出了作用域，陷阱里的 "${sandbox:?}" 会报 parameter null or not set、目录也删不掉。
# 所以目录名放进不带 local 的全局变量 selftest_sandbox，陷阱函数 selftest_cleanup 在脚本退出时读它；
# 函数里的提前返回与信号退出（INT / TERM / HUP 转成 exit，EXIT 陷阱照跑）走同一条清理路径。
selftest_sandbox=""
selftest_cleanup() {
  [[ -n "$selftest_sandbox" ]] || return 0
  [[ "$BREAK" == sandbox-left && -n "${BASELINE_DIFF_SELFTEST_NESTED:-}" ]] && return 0
  rm -rf "${selftest_sandbox:?}"
}
selftest() {
  local sandbox repository sample results="" total=0 failed=0 output rc worktrees leftovers
  sandbox="$(mktemp -d)" || { echo "  ✗ 自证建不了临时目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败）"; echo "     → 怎么办：让 \$TMPDIR 指到一个能写的目录再跑。"; return 1; }
  selftest_sandbox="$sandbox"
  trap selftest_cleanup EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM HUP
  export GIT_AUTHOR_NAME=selftest GIT_AUTHOR_EMAIL=selftest@example.invalid GIT_COMMITTER_NAME=selftest GIT_COMMITTER_EMAIL=selftest@example.invalid
  export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
  unset GATE_BASE GATE_STAGED_FROM GATE_DIFF_BASE GATE_STAGED_TREE SINGLEFS_STAGED_TREE FAKE_GATE_CRASH FAKE_GATE_EXIT
  repository="$sandbox/repo"
  mkdir -p "$repository/scripts" "$sandbox/tmp"
  git init -q -b master "$repository"
  cat > "$repository/fake-gate.sh" <<'FAKE_GATE_EOF'
#!/usr/bin/env bash
# 假门禁：格甲判 <根>/a.txt、格乙判 <根>/b.txt（靠 <根>/scripts/judge-b.sh）里有没有 bad 行；--check 点名格；--force 吃掉
[[ "${FAKE_GATE_CRASH:-0}" == 1 ]] && { echo "  假门禁自己崩了"; exit 2; }
[[ -n "${FAKE_GATE_EXIT:-}" ]] && exit "$FAKE_GATE_EXIT"
cells=(a b); root=""
while (($#)); do
  case "$1" in
    --check) IFS=, read -r -a cells <<<"$2"; shift 2 ;;
    --force) shift ;;
    *) root="$1"; shift ;;
  esac
done
cd "$root" || exit 2
status=0
for cell in "${cells[@]}"; do
  case "$cell" in
    a)
      line="$(grep -n bad a.txt | head -n 1 | cut -d: -f1)"
      if [[ -n "$line" ]]; then echo "  ✗ 格甲 a.txt:$line: 有 bad（第 $line 行）"; echo "     → 怎么办：删掉 bad"; status=1; else echo "  ✓ 格甲"; fi ;;
    b)
      bash scripts/judge-b.sh b.txt; judged=$?
      [[ "$judged" -eq 127 ]] && exit 127
      [[ "$judged" -eq 0 ]] || status=1 ;;
  esac
done
[[ "$status" -eq 0 ]] || { echo "  ✗ fake-gate：有格红"; echo "     → 怎么办：看上面"; }
exit "$status"
FAKE_GATE_EOF
  cat > "$repository/fake-doc-lint.sh" <<'FAKE_LINT_EOF'
#!/usr/bin/env bash
# 假 doc-lint：只收项目根，判 <根>/c.md 里每个 TODO 行各红一条
root="${1:?}"; status=0
while IFS=: read -r line _; do
  echo "  ✗ c.md:$line: 留着 TODO"; echo "     → 怎么办：写完或删掉"; status=1
done < <(grep -n TODO "$root/c.md")
exit "$status"
FAKE_LINT_EOF
  cat > "$repository/fake-count-gate.sh" <<'FAKE_COUNT_EOF'
#!/usr/bin/env bash
# 假门禁（汇总行带计数）：<根>/n.txt 里的数写进一条汇总红行；<根>/extra.txt 在就再红一条不带数的明细
root="${1:?}"; n="$(cat "$root/n.txt")"
echo "  ✗ 门禁自检失败：${n} 个脚本里有 2 处不对（共 ${n}0 条拒绝）"; echo "     → 怎么办：看明细"
[[ -f "$root/extra.txt" ]] && { echo "  ✗ 新坏了 extra.txt：有 oops"; echo "     → 怎么办：删掉 oops"; }
exit 1
FAKE_COUNT_EOF
  printf '3\n' > "$repository/n.txt"
  printf 'x\ny\nbad\n' > "$repository/a.txt"
  printf 'ok\n' > "$repository/b.txt"
  printf 'TODO 一\nfine\n' > "$repository/c.md"
  git -C "$repository" add -A && git -C "$repository" commit -q -m base
  local base_commit
  base_commit="$(git -C "$repository" rev-parse HEAD)"
  # 工作区：格甲基准就红、行号挪了；格乙这一次新红，靠基准树里没有的 scripts/judge-b.sh；c.md 多一条 TODO
  printf 'new1\nnew2\nx\ny\nbad\n' > "$repository/a.txt"
  printf 'ok\nbad\n' > "$repository/b.txt"
  printf '#!/usr/bin/env bash\n# 格乙的判法：b.txt 里有 bad 就红\nline="$(grep -n bad "$1" | head -n 1 | cut -d: -f1)"\n[[ -z "$line" ]] && { echo "  ✓ 格乙"; exit 0; }\necho "  ✗ 格乙 $1:$line: 有 bad（第 $line 行）"; echo "     → 怎么办：删掉 bad"; exit 1\n' > "$repository/scripts/judge-b.sh"
  printf 'TODO 一\nfine\nTODO 二\n' > "$repository/c.md"
  printf '4\n' > "$repository/n.txt"

  check() { # check <名字> <条件：0 成立> <不成立时写什么>
    total=$((total + 1))
    if [[ "$2" == 0 ]]; then results+="  ✓ $1"$'\n'; else results+="  ✗ $1：$3"$'\n'; failed=$((failed + 1)); fi
  }
  run_case() { # run_case <工作目录上一级> <门禁脚本> <别的参数…>：在临时仓里起自己，输出进 $output，退出码进 $rc
    local parent="$1" gate="$2"; shift 2
    output="$(cd "$repository" && bash "$SELF" "$gate" --base "$base_commit" --work-directory "$parent" "$@" 2>&1)"; rc=$?
  }

  # ① 带 --check 的假门禁：基准就红的格甲不报（行号挪了也不报），这一次新红的格乙报；格乙的判法基准树里没有，照用主仓那一份
  run_case "$sandbox/tmp" "$repository/fake-gate.sh"
  check "只报新红的格：退出 1" "$([[ "$rc" == 1 ]]; echo $?)" "退出 $rc，输出：$output"
  check "只报新红的格：新增 1 条" "$(grep -qF '工作区新增的红行 1 条' <<<"$output"; echo $?)" "输出：$output"
  check "只报新红的格：报了格乙" "$(grep -qF '✗ 格乙 b.txt:2' <<<"$output"; echo $?)" "输出：$output"
  check "基准就红的格不报" "$(! grep -qF '格甲' <<<"${output#*工作区新增的红行}"; echo $?)" "格甲被当成新红：$output"
  check "只挪了行号不报" "$(grep -qF '基准红的原样留着 2 条' <<<"$output"; echo $?)" "输出：$output"
  check "基准树缺判法脚本照用主仓的" "$(grep -qF '照用主仓那一份的判法脚本 1 份' <<<"$output"; echo $?)" "输出：$output"
  worktrees="$(git -C "$repository" worktree list | wc -l)"
  leftovers="$(find "$sandbox/tmp" -mindepth 1 -maxdepth 1 | wc -l)"
  check "收尾删干净" "$([[ "$worktrees" == 1 && "$leftovers" == 0 ]]; echo $?)" "worktree ${worktrees} 个、工作目录上一级里剩 ${leftovers} 项"

  # ② 点名只跑格甲：基准就红、工作区同样红（行号挪了）→ 没有新增，退 0
  run_case "$sandbox/tmp" "$repository/fake-gate.sh" --check a
  check "只跑基准就红的格：退出 0" "$([[ "$rc" == 0 ]]; echo $?)" "退出 $rc，输出：$output"
  check "只跑基准就红的格：新增 0 条" "$(grep -qF '新增的红行 0 条' <<<"$output"; echo $?)" "输出：$output"

  # ③ 像 doc-lint 的门禁（只收项目根，不认 --check）：c.md 多一条 TODO 二，只报它
  run_case "$sandbox/tmp" "$repository/fake-doc-lint.sh"
  check "像 doc-lint 的门禁：新增 1 条" "$([[ "$rc" == 1 ]] && grep -qF '工作区新增的红行 1 条' <<<"$output"; echo $?)" "退出 $rc，输出：$output"
  check "像 doc-lint 的门禁：报的是新的那条" "$(grep -qF 'c.md:3' <<<"$output"; echo $?)" "输出：$output"

  # ④ 门禁脚本崩了：退 3，不当成通过
  output="$(cd "$repository" && FAKE_GATE_CRASH=1 bash "$SELF" "$repository/fake-gate.sh" --base "$base_commit" --work-directory "$sandbox/tmp" 2>&1)"; rc=$?
  check "门禁脚本崩了：退 3" "$([[ "$rc" == 3 ]] && grep -qF '判法本身没跑成' <<<"$output"; echo $?)" "退出 $rc，输出：$output"
  # 两趟都退 77：退 77
  output="$(cd "$repository" && FAKE_GATE_EXIT=77 bash "$SELF" "$repository/fake-gate.sh" --base "$base_commit" --work-directory "$sandbox/tmp" 2>&1)"; rc=$?
  check "两趟都本次未跑：退 77" "$([[ "$rc" == 77 ]]; echo $?)" "退出 $rc，输出：$output"

  # ⑤ 汇总行只是计数变了（基准 3、工作区 4）：单列、不算新红、退 0，基准与工作区两版各打一行；真新红（extra.txt 只在工作区）照报、退 1
  local red_mark=$'\xe2\x9c\x97'   # 假门禁打的红行开头那个字；下面几格拿它拼要找的整行，字面量不写进这些检查行里
  run_case "$sandbox/tmp" "$repository/fake-count-gate.sh"
  check "计数变了不报新红：退出 0" "$([[ "$rc" == 0 ]]; echo $?)" "退出 $rc，输出：$output"
  check "计数变了不报新红：新增 0 条" "$(grep -qF '新增的红行 0 条' <<<"$output"; echo $?)" "输出：$output"
  check "计数变了单列：1 条" "$(grep -qF '计数变了的红行 1 条' <<<"$output"; echo $?)" "输出：$output"
  check "计数变了单列：基准与工作区两版各一行" "$(grep -qF "基准：${red_mark} 门禁自检失败：3 个脚本里有 2 处不对（共 30 条拒绝）" <<<"$output" && grep -qF "工作区：${red_mark} 门禁自检失败：4 个脚本里有 2 处不对（共 40 条拒绝）" <<<"$output"; echo $?)" "输出：$output"
  printf 'oops\n' > "$repository/extra.txt"
  run_case "$sandbox/tmp" "$repository/fake-count-gate.sh"
  check "计数变了之外有真新红：退出 1" "$([[ "$rc" == 1 ]]; echo $?)" "退出 $rc，输出：$output"
  check "计数变了之外有真新红：只新增 1 条" "$(grep -qF '工作区新增的红行 1 条' <<<"$output"; echo $?)" "输出：$output"
  check "计数变了之外有真新红：报的是新坏了那条" "$(grep -qF "${red_mark} 新坏了 extra.txt：有 oops" <<<"${output#*工作区新增的红行}"; echo $?)" "输出：$output"
  check "计数变了之外有真新红：汇总行仍单列、不在新增里" "$(grep -qF '计数变了的红行 1 条' <<<"$output" && ! grep -qF '门禁自检失败' <<<"${output#*工作区新增的红行}"; echo $?)" "输出：$output"
  rm -f "$repository/extra.txt"

  # ⑥ 自证本体跑完不在 $TMPDIR 里留东西：里面起一趟自证（子 shell，TMPDIR 指到新空目录），跑完那个目录要空、stderr 里没有 parameter null or not set。
  #    里面那一趟带 BASELINE_DIFF_SELFTEST_NESTED=1，不再往下起；弄坏开关 sandbox-left 只递给它，外面这一趟的目录照删
  if [[ -z "${BASELINE_DIFF_SELFTEST_NESTED:-}" ]]; then
    local nested_tmp="$sandbox/nested-tmp" nested_break="" nested_rc nested_left nested_error
    [[ "$BREAK" != sandbox-left ]] || nested_break=sandbox-left
    mkdir -p "$nested_tmp"
    (
      export TMPDIR="$nested_tmp" BASELINE_DIFF_SELFTEST_NESTED=1 BASELINE_DIFF_BREAK="$nested_break"
      bash "$SELF" --selftest > "$sandbox/nested.out" 2> "$sandbox/nested.err"
    )
    nested_rc=$?
    nested_left="$(find "$nested_tmp" -mindepth 1 -maxdepth 1 | wc -l)"
    nested_error="$(cat "$sandbox/nested.err")"
    check "自证跑完不留临时目录" "$([[ "$nested_rc" == 0 && "$nested_left" == 0 ]] && ! grep -qF 'parameter null or not set' <<<"$nested_error"; echo $?)" \
      "里面那一趟退 ${nested_rc}、TMPDIR 里剩 ${nested_left} 项，stderr：${nested_error:-（空）}；剩下的：$(find "$nested_tmp" -mindepth 1 -maxdepth 1 -printf '%f ')"
  fi

  printf '%s' "$results"
  if ((failed)); then
    echo "  ✗ baseline-diff 自证没过：${total} 项里 ${failed} 项红${BREAK:+（弄坏开关 BASELINE_DIFF_BREAK=$BREAK 打开着）}"
    echo "     → 怎么办：看上面每一项写着的实测；开关没打开却红，是 baseline-diff.sh 自己坏了。"
    return 1
  fi
  echo "  ✓ baseline-diff 自证 ${total} 项全过（基准就红的格不报、这一次新红的格报、挪行号不报、缺判法脚本照用主仓的、收尾删干净、崩了退 3、两趟未跑退 77、只差计数的汇总行单列不算新红且真新红照报、自证跑完不留临时目录）"
  return 0
}

# ── 参数 ────────────────────────────────────────────────────────────────────
gate_script="" base_ref="" keep=0 work_parent="${TMPDIR:-/tmp}"
gate_cell_list=() gate_force=() extra_ignores=() gate_arguments=()
while (($#)); do
  case "$1" in
    --selftest) selftest; exit $? ;;
    --base)
      (($# >= 2)) || die "--base 后面没给提交" "写成 --base <提交>（分支名、标签、哈希都行）。"
      base_ref="$2"; shift 2 ;;
    --check)
      (($# >= 2)) || die "--check 后面没给格名" "写成 --check <格>[,<格>…]，格名用 bash <门禁脚本> --list 看。"
      gate_cell_list+=("$2"); shift 2 ;;
    --force-gate) gate_force=(--force); shift ;;
    --keep) keep=1; shift ;;
    --work-directory)
      (($# >= 2)) || die "--work-directory 后面没给目录" "写成 --work-directory <目录>（工作目录的上一级，里面建 baseline-diff.XXXXXX）。"
      work_parent="$2"; shift 2 ;;
    --ignore)
      (($# >= 2)) || die "--ignore 后面没给正则" "写成 --ignore <正则>（python re，匹配到的片段比较前抹成 #）。"
      extra_ignores+=("$2"); shift 2 ;;
    -*) die "认不出的选项 $1" "只认 --base、--check、--force-gate、--keep、--work-directory、--ignore、--selftest；用法见这个脚本的文件头。" ;;
    *)
      [[ -z "$gate_script" ]] || die "给了两个门禁脚本：$gate_script 与 $1" "一次比一个门禁脚本；要比几道就起几次。"
      gate_script="$1"; shift ;;
  esac
done
[[ -n "$gate_script" ]] || die "没给门禁脚本" "第一个参数写门禁脚本，例：bash scripts/gate/baseline-diff.sh .claude/gate.d/doc-experiments.sh --check results-cited --base HEAD。"
[[ -n "$base_ref" ]] || die "没给 --base" "写 --base <提交>：基准是上一次提交（HEAD）、上游（@{upstream}）还是某个哈希，由你定，这里不猜。"
[[ -f "$gate_script" ]] || die "门禁脚本不存在：$gate_script" "写门禁脚本的路径（相对当前目录或绝对路径）。"
gate_script="$(cd "$(dirname "$gate_script")" && pwd)/$(basename "$gate_script")"
if ((${#gate_cell_list[@]})); then
  joined_cells="$(IFS=,; echo "${gate_cell_list[*]}")"
  gate_arguments=(--check "$joined_cells")
fi

main_root="$(git rev-parse --show-toplevel 2>/dev/null)" || die "当前目录不在 git 仓里" "在要判的仓里跑它（基准树是 git worktree，要有仓）。"
base_commit="$(git -C "$main_root" rev-parse --verify --quiet "${base_ref}^{commit}")" || die "--base $base_ref 不是仓里的提交" "写分支名、标签或提交哈希；git log --oneline 看有哪些。"
[[ -d "$work_parent" ]] || die "工作目录的上一级不存在：$work_parent" "先建它，或不给 --work-directory 用 \$TMPDIR。"

work="$(mktemp -d "${work_parent%/}/baseline-diff.XXXXXX")" || die "建不了工作目录（在 $work_parent 下）" "换一个能写的 --work-directory。"
base_tree="$work/base"
cleanup() {
  [[ "$BREAK" != worktree-left ]] || return 0
  ((keep)) && return 0
  if [[ -d "$base_tree" ]]; then
    git -C "$main_root" worktree remove --force "$base_tree" >/dev/null 2>&1 || echo "  ! 删不掉 worktree $base_tree（git worktree remove 失败），手工 git worktree remove --force 它" >&2
  fi
  git -C "$main_root" worktree prune >/dev/null 2>&1 || echo "  ! git worktree prune 失败，手工跑一遍" >&2
  rm -rf "${work:?}"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

git -C "$main_root" worktree add -q --detach "$base_tree" "$base_commit" || die "建不了基准树的 worktree（git worktree add --detach $base_tree $base_commit 失败）" "按上面 git 的报错修（磁盘满、仓坏了、这个提交的检出有冲突）。"
tool_count="$(overlay_missing_tools "$main_root" "$base_tree")" || die "往基准树补判法脚本失败" "看基准树 $base_tree 是否可写、磁盘有没有满。"

run_gates "$gate_script" "$base_tree" "$main_root" "$base_commit" "$work"
base_rc="$(cat "$work/base.rc")"
workspace_rc="$(cat "$work/workspace.rc")"
gate_label="$(basename "$gate_script")${gate_arguments[*]:+ ${gate_arguments[*]}}"
base_label="${base_commit:0:12}（$(git -C "$main_root" log -1 --format=%s "$base_commit")）"

compare_outputs "$work/base.out" "$work/workspace.out" "$base_tree" "$main_root" "$base_rc" "$workspace_rc" "$gate_label" "$base_label" "$tool_count" ${extra_ignores[@]+"${extra_ignores[@]}"}
compare_rc=$?
if ((keep)); then
  echo "  留着的现场（--keep）：$work（base.out、workspace.out 是两趟的完整输出；删法：git -C $main_root worktree remove --force $base_tree && git -C $main_root worktree prune && rm -rf $work）"
fi
exit "$compare_rc"
