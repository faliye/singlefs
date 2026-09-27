#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的 Rust 源码与两张登记表，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: 函数名与类型名不许只由空泛词拼成（run_cell、CellRun、check_cell 这一类）：每个词都在 .claude/naming-vague-words 里就判违规
# gate-similar: 12-no-prime-marks.sh 也是全仓扫名字的写法，但判的是撇号类字符当角标，对象是全部文本文件；这里判 .rs 里我们声明的名字由哪些词拼成，读一张词表
# gate-similar: .claude/singlefs-ai-sop/scripts/naming-lint.sh 是上游的命名纪律，判单字母与常见缩写、词表写死在上游；空泛词表是本项目定的（用户 2026-09-27「run_cell 这种类似的命名是最差劲的命名」），上游这一批不发版，先放项目本地
#
# 判据：`git ls-files` 列出的 crates/**/*.rs 与 research/e7-index-bench/src/**/*.rs 里，每一处 `fn <名>` 与 `struct|enum|type|trait <名>`：
# 函数名按 `_` 切词、类型名按大写字母切词，切出来的每一个词（小写）都在 .claude/naming-vague-words 里，判违规，逐处列「文件:行 名字」。
# 不判的：`impl <trait> for <类型>` 块里的方法（名字是 trait 定的）、`main`、宏展开出来的名字（认不出）。
# 还没改完名的文件逐个登记在 .claude/naming-vague-exclude（一行 <路径>  # 理由），那份文件里的违规不判；排除项指向不存在的文件、
# 或那份文件里已经一处违规都没有了，判红（排除只缩不涨）。
# 一份 .rs 都没扫到：无对象可判，退 77（不记通过）。
# 管不到的：名字里夹了一个非空泛词、而那个词本身也什么都没说（run_the_cell_again）——词表只认整名全空泛，这一类靠 review。
# 样本：fixtures/13-vague-names.sh/red 有一份带 fn run_cell 与 struct CellRun 的 rs、一条指向不存在文件的排除项；green 只有带领域词的名字与一个 trait 方法 get。
#
#   bash .claude/gate.d/13-vague-names.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级。"; exit 2; }
python3 - <<'PY'
import os, re, subprocess, sys

WORDS_FILE = ".claude/naming-vague-words"
EXCLUDE_FILE = ".claude/naming-vague-exclude"
BROKEN = os.environ.get("GATE_VAGUE_NAMES_BREAK", "")

def registry(path):
    entries = []
    if not os.path.isfile(path):
        return None
    for number, line in enumerate(open(path, encoding="utf-8"), 1):
        body = line.split("#", 1)[0].strip()
        if body:
            entries.append((number, body, line.split("#", 1)[1].strip() if "#" in line else ""))
    return entries

words = registry(WORDS_FILE)
if words is None:
    print(f"  ✗ 读不到空泛词表 {WORDS_FILE}")
    print("     → 怎么办：从 git 恢复它；这一道没有词表就什么都判不了，不是通过。")
    sys.exit(1)
vague = {word.lower() for _number, word, _reason in words}
excluded = registry(EXCLUDE_FILE) or []

listed = subprocess.run(["git", "-c", "core.quotepath=false", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "crates", "research/e7-index-bench/src"],
                        capture_output=True)
paths = sorted(path for path in listed.stdout.decode("utf-8", "replace").split("\0") if path.endswith(".rs") and os.path.isfile(path))
if not paths:
    print("  ! 本次无对象可判：crates/ 与 research/e7-index-bench/src/ 下一份 .rs 都没列出来")
    sys.exit(77)

DECLARATION = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*(fn|struct|enum|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)")
TRAIT_IMPL = re.compile(r"^\s*impl\b.*\bfor\b")

def words_of(kind, name):
    if kind == "fn":
        return [part.lower() for part in name.split("_") if part]
    return [part.lower() for part in re.findall(r"[A-Z][a-z0-9]*|[a-z0-9]+", name)]

def violations_in(path):
    found = []
    depth = 0
    trait_impl_depths = []
    for number, line in enumerate(open(path, encoding="utf-8", errors="replace"), 1):
        code = line.split("//", 1)[0]
        if TRAIT_IMPL.match(code) and BROKEN != "trait-methods-judged":
            trait_impl_depths.append(depth)
        match = DECLARATION.match(code)
        inside_trait_impl = bool(trait_impl_depths) and depth > trait_impl_depths[-1]
        if match and not (match.group(1) == "fn" and inside_trait_impl) and match.group(2) != "main":
            parts = words_of(match.group(1), match.group(2))
            if parts and all(part in vague for part in parts):
                found.append(f"{path}:{number} {match.group(1)} {match.group(2)}")
        depth += code.count("{") - code.count("}")
        while trait_impl_depths and depth <= trait_impl_depths[-1] and "}" in code:
            trait_impl_depths.pop()
    return found

excluded_paths = {entry for _number, entry, _reason in excluded}
problems, exclusion_problems = [], []
scanned = 0
for path in paths:
    scanned += 1
    found = violations_in(path)
    if path in excluded_paths and BROKEN != "exclusions-ignored":
        if not found:
            exclusion_problems.append(f"{EXCLUDE_FILE}：{path} 里已经一个空泛名都没有了，这一行删掉")
        continue
    problems += found
for number, entry, reason in excluded:
    if not os.path.isfile(entry):
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} 指向不存在的文件 {entry}")
    elif len(reason) < 4:
        exclusion_problems.append(f"{EXCLUDE_FILE}:{number} {entry} 没写为什么先不改（# 后面至少 4 个字）")

for problem in problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if problems:
    print(f"  ✗ {len(problems)} 个函数名或类型名只由空泛词拼成（每个词都在 {WORDS_FILE} 里）")  # gate-lint:summary
    print("     → 怎么办：按 .claude/singlefs-ai-sop/rules/code-discipline.md「名字：光看名字，就说得出它是什么、做什么」改名：说出跑的是什么、判的是哪条、取的是哪个量；")
    print(f"                一时改不完的文件登记进 {EXCLUDE_FILE}（一行 <路径>  # 理由），改完删那一行。")
for problem in exclusion_problems:
    print(f"  ✗ {problem}")  # gate-lint:detail
if exclusion_problems:
    print(f"  ✗ {EXCLUDE_FILE} 有 {len(exclusion_problems)} 行不起作用或没写理由")  # gate-lint:summary
    print("     → 怎么办：指向不存在的与已经改完的那几行删掉；留着的每一行写明为什么先不改。排除只缩不涨。")
if problems or exclusion_problems:
    sys.exit(1)
print(f"  ✓ 没有只由空泛词拼成的函数名与类型名（扫了 {scanned} 份 .rs，词表 {len(vague)} 个词，排除 {len(excluded)} 份还没改完的文件）")
PY
