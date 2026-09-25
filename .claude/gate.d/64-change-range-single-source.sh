#!/usr/bin/env bash
# gate-stage: 改动范围只许一份取法：阶段不自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false
#
# 判据（射程是 `.claude/gate.d/` 顶层的 *.sh 与 *.py，本阶段自己除外）。三条，任一条不成立判红：
#   ① 算 diff 基准只许在 research/scripts/changed-paths.sh 里：代码行里出现 merge-base（`--is-ancestor` 核祖先除外）、
#      @{upstream} 或 @{u}、gate-ok、读 GATE_BASE 或 GATE_DIFF_BASE（`$GATE_BASE`、`${GATE_BASE`、environ）的，
#      以及跑 git 的那一行上写死 HEAD~N、用三点号（A...B 就是取 merge-base）的，判红。`unset GATE_BASE` 这类不读它的不算。
#      阶段要改动范围就 source 那份共用脚本、调 gate_diff_base。
#   ② 列路径的 git 调用（--name-only、--name-status、ls-files）要带 -c core.quotepath=false（键不分大小写，值必须是 false）：
#      同一行写了它；或者这一行用的变量 / 列表在同一个文件里带着它定义（例 `GIT = ["git", "-c", "core.quotepath=false"]`）；
#      或者这一行调的是同一个文件里带着它的包装函数（python 写成「名字(」，bash 写在命令位置上；名字恰好是 git 时，
#      裸的 git 命令与 ['git', …] 列表不算调了它——例 75 号的 `def git(*args)`）；或者 git 自己带 -z（-z 下不转义路径）。
#      参数列表换了行、这一行以引号开头的，往上看三行找调用前缀。
#      中文路径不带这个选项会被转成带引号的八进制串，与仓里的路径逐字对不上，而 git 不报错。
#   ③ 调共用取法（gate_changed_paths、gate_added_lines）要判退出码：同一行带 `||`，或放在 if / while 里。
#      git 失败时共用脚本退非 0、什么都不交，不判就会把「没取到」读成「这次什么都没改」。
#   「代码行」：去掉前导空白后不以 #、echo、printf、print(、bad、howto、die、引号开头的行，或者以这些开头、但这一行里
#   跑了 `$(git …)` 或带引号括起的列路径参数的——出路文字与注释里提到这些词不算。
#
# 为什么：改动范围的取法收成共用脚本之前有六份以上拷贝，已经分叉过（一半漏了 quotepath；61 号的基准在默认分支上
# 就是 HEAD，定案分两次提交、第一次没推，它就退 77 不判）。收完之后没有东西拦下一份拷贝。
# 三方判决：research/prompts/gate-fix-forks-r1-main-verification.md 的 T5；判据按第二轮攻方打中的写法收严（gate-fix-forks-r2）。
#
# 没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下的脚本不是门禁阶段，成功行把其中碰到这两条的逐个列名，清单现算。
# 样本：fixtures/64-change-range-single-source.sh/red 把三条判据该抓的写法各放几种（自己取 merge-base、@{u}...、HEAD~1、
# 不带或带 =true 的 quotepath、包装函数名叫 git 而这一行没调它、test 的 -z、printf "$(git …)"、换行的参数列表、
# 不判退出码的共用取法），逐条点名判红；green 放五种合法写法与 unset GATE_BASE、merge-base --is-ancestor、判了退出码的调用，判绿。
#
#   bash .claude/gate.d/64-change-range-single-source.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
[[ -d .claude/gate.d ]] || { echo "  ! 没有 .claude/gate.d，本阶段无对象可判"; exit 77; }
python3 - "$(basename "$0")" <<'PY'
import glob, os, re, sys

own_name = sys.argv[1]
BASE_PATTERN = re.compile(r'merge-base|@\{u(pstream)?\}|gate-ok|\$\{?GATE_(DIFF_)?BASE\b|environ[^\n]*GATE_(DIFF_)?BASE')
# 写死的基准（HEAD~N）与三点号（A...B 就是取 merge-base）只在跑 git 的那一行上算：文档字符串里的「...」不算
BASE_IN_GIT_PATTERN = re.compile(r'\bHEAD~\d|\S\.\.\.(\s|$|[^.])')
RUNS_GIT_ANYWHERE = re.compile(r'\bgit\b|\bGIT\b')
BASE_ALLOWED = re.compile(r'merge-base\s+--is-ancestor')
LIST_PATTERN = re.compile(r'--name-only|--name-status|\bls-files\b')
QUOTED_LIST_FLAG = re.compile(r"""["'](--name-only|--name-status|ls-files)["']""")
TEXT_PREFIX = re.compile(r"""^(#|echo\b|printf\b|print\(|bad\b|howto\b|die\b|["'])""")
RUNS_GIT = re.compile(r'\$\(\s*git\b')
QUOTEPATH_FALSE = re.compile(r'quotepath=false', re.I)
QUOTE_VARIABLE = re.compile(r'^\s*([A-Za-z_][A-Za-z_0-9]*)\s*\+?=.*quotepath=false', re.I)
WRAPPER_HEAD = re.compile(r'^\s*(?:def\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(|(?:function\s+)?([A-Za-z_][A-Za-z_0-9]*)\s*\(\)\s*\{?)')
GIT_Z = re.compile(r"""\bgit\b[^|;&]*\s-z\b|["']-z["']""")
SHARED_CALL = re.compile(r'\b(gate_changed_paths|gate_added_lines)\b')
STATUS_HANDLED = re.compile(r'\|\||^\s*(if|elif|while|until|!)\b')
WRAPPER_REACH = 3
LOOKBACK = 3
CONTINUATION = re.compile(r"""^\s*["'\[]""")

def is_text(stripped):
    """出路文字与注释：以这些开头、而且这一行不跑 git、不带引号括起的列路径参数的。"""
    return bool(TEXT_PREFIX.match(stripped)) and not RUNS_GIT.search(stripped) and not QUOTED_LIST_FLAG.search(stripped)

def quoting_names(all_lines):
    """同一个文件里带着 quotepath=false 定义的变量 / 列表，与带着它的包装函数：(变量名集合, 函数名集合)。"""
    variables, wrappers = set(), set()
    for index, line in enumerate(all_lines):
        variable = QUOTE_VARIABLE.match(line)
        if variable:
            variables.add(variable.group(1))
        wrapper = WRAPPER_HEAD.match(line)
        if wrapper and any(QUOTEPATH_FALSE.search(body) for body in all_lines[index:index + 1 + WRAPPER_REACH]):
            wrappers.add(wrapper.group(1) or wrapper.group(2))
    return variables, wrappers

def uses_quoting(line, variables, wrappers):
    if QUOTEPATH_FALSE.search(line) or GIT_Z.search(line):
        return True
    if any(re.search(r'\$\{?' + re.escape(name) + r'\b|\*' + re.escape(name) + r'\b|\b' + re.escape(name) + r'\s*(\+|\[)', line)
           for name in variables):
        return True
    # 包装函数认「这一行调的是它」：python 写成「名字(」，bash 写在命令位置上。名字恰好是 git 时，
    # 「git(」认，裸的 git 命令与 ['git', …] 列表不认——75 号的 def git(*args) 就是这种。
    for name in wrappers:
        if re.search(r'(?<![\w\'"])' + re.escape(name) + r'\(', line):
            return True
        if name != 'git' and re.search(r'(^|[;&|(]|\$\()\s*' + re.escape(name) + r'\s', line.strip()):
            return True
    return False

def judge(path):
    """返回 (算基准的行, 不带 quotepath 的列路径行, 不判退出码的共用取法调用)。"""
    with open(path, encoding='utf-8', errors='replace') as handle:
        all_lines = handle.read().split('\n')
    variables, wrappers = quoting_names(all_lines)
    base_hits, list_hits, status_hits = [], [], []
    for index, line in enumerate(all_lines):
        stripped = line.strip()
        if not stripped or is_text(stripped):
            continue
        number = index + 1
        if (BASE_PATTERN.search(line) or BASE_IN_GIT_PATTERN.search(line) and RUNS_GIT_ANYWHERE.search(line)) \
                and not BASE_ALLOWED.search(line):
            base_hits.append((number, stripped))
        if LIST_PATTERN.search(line) and not uses_quoting(line, variables, wrappers):
            # 参数列表换了行（这一行以引号或方括号开头）：往上找这条语句开头的那一行，看调用前缀带没带选项
            head = index
            while head > 0 and index - head < LOOKBACK and CONTINUATION.match(all_lines[head]):
                head -= 1
            if head == index or not uses_quoting(all_lines[head], variables, wrappers):
                list_hits.append((number, stripped))
        if SHARED_CALL.search(line) and not re.match(r'^\s*(gate_changed_paths|gate_added_lines)\s*\(\)', line) \
                and not STATUS_HANDLED.search(line):
            status_hits.append((number, stripped))
    return base_hits, list_hits, status_hits

scanned = sorted(path for pattern in ('.claude/gate.d/*.sh', '.claude/gate.d/*.py')
                 for path in glob.glob(pattern) if os.path.basename(path) != own_name)
base_bad, list_bad, status_bad = [], [], []
for path in scanned:
    base_hits, list_hits, status_hits = judge(path)
    base_bad += [(path, number, line) for number, line in base_hits]
    list_bad += [(path, number, line) for number, line in list_hits]
    status_bad += [(path, number, line) for number, line in status_hits]

failed = False
if base_bad:
    failed = True
    print(f'  ✗ {len(base_bad)} 行在阶段里自己算 diff 基准（改动范围只许一份取法：research/scripts/changed-paths.sh）：')
    for path, number, line in base_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：阶段里 source research/scripts/changed-paths.sh，基准写 base="$(gate_diff_base gate)"（问「这一次提交带哪些」的写 head），')
    print('               路径写 gate_changed_paths、新增行写 gate_added_lines；共用脚本缺哪种取法就往它里面加，连同它的 --selftest 一起改。')
if list_bad:
    failed = True
    print(f'  ✗ {len(list_bad)} 行列路径的 git 调用没带 -c core.quotepath=false：')
    for path, number, line in list_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：在那一行的 git 后面加 -c core.quotepath=false（python 里是 ["git", "-c", "core.quotepath=false", …]），')
    print('               或者把带它的调用前缀收进一个变量 / 列表再用；不带它时中文路径变成带引号的八进制串，与仓里的路径逐字对不上。')
if status_bad:
    failed = True
    print(f'  ✗ {len(status_bad)} 处调共用取法（gate_changed_paths / gate_added_lines）没判退出码：')
    for path, number, line in status_bad:
        print(f'      {path}:{number}  {line[:120]}')   # gate-lint:detail
    print('     → 怎么办：写成 x="$(gate_changed_paths …)" || { echo 取不到改动范围…; echo 出路…; exit 1; }，或放进 if；')
    print('               git 失败时共用脚本退非 0、什么都不交，不判退出码就会把「没取到」读成「这次什么都没改」。')
if failed:
    sys.exit(1)

outside = []
for pattern in ('research/scripts/*.sh', 'research/scripts/*.py', '.claude/scripts/*', '.claude/hooks/*'):
    for path in sorted(glob.glob(pattern)):
        if os.path.isfile(path) and path != 'research/scripts/changed-paths.sh':
            base_hits, list_hits, status_hits = judge(path)
            if base_hits or list_hits or status_hits:
                outside.append(f'{path}（算基准 {len(base_hits)} 行、不带 quotepath 的列路径 {len(list_hits)} 行、不判退出码 {len(status_hits)} 处）')
print(f'  ✓ 查了 .claude/gate.d/ 下 {len(scanned)} 份：没有阶段自己算 diff 基准，列路径的 git 调用都带 core.quotepath=false，调共用取法都判了退出码')
if outside:
    print(f'     没扫的 {len(outside)} 份（不是门禁阶段，射程之外）：' + '；'.join(outside))
else:
    print('     没扫的：research/scripts/、.claude/scripts/、.claude/hooks/ 下没有碰到这两条的脚本')
PY
