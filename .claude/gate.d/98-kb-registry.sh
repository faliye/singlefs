#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本与 git 记录，除了跑门禁本身就要的 bash、git、python3 之外没有环境要求
# gate-stage: kb 目录里的每一份都要在 CLAUDE.md 的「项目本地事实」表里有一行
#
# 判据：三条，任一条不成立判红。
#   ① `.claude/kb/` 根目录下每一份 `.md`，都要在 `CLAUDE.md` 的「## 项目本地事实」表里被按路径点名；
#   ② `.claude/kb/` 下每一个子目录也要被点名（形态是带斜杠的路径，例如 `.claude/kb/decisions/`）；
#   ③ 反过来，那张表里点到的每一个 `.claude/kb/…` 路径都要真的存在。
#
# 为什么：那张表是 kb 各文件职责的唯一登记位——一份文件是什么、归谁读、与别处什么关系，只写在那里一行。
# 新建一份 kb 文件而不登记，它对别的会话与派出去的 agent 就是个来历不明的东西；删掉一份而不撤行，
# 表就指向空处。门禁 62 号早就在盯 `stage-owners.tsv` 与 `.claude/gate.d/` 逐项一致，kb 这一侧一直没有对应的闸。
#
# ⚠️ **这条是实测出来的**（2026-09-21）：建 `.claude/kb/feature-bits.md` 时漏了登记，一查那张表还同时漏着
# `INDEX.md`、`term-renames.md`、`tooling.md` 三份——四份文件没有任何东西说得出它们是什么。
#
# 「登记」只认那张表里一行的**第一格**写的路径（逐字相等）：表外散文里提到的反引号路径不算，
# 一行 `.claude/kb/` 也不替它下面的每一份登记。带月份占位的形态（`.claude/kb/decisions-history/<年-月>.md`，
# 写在表里任一格都算）替它所在的那个子目录登记。「指得到东西」那一条照旧对那一节里点到的每个 `.claude/kb/…` 路径判。
#
# ⚠️ 射程：判的是「登记了没有」，判不了那一行写得对不对——后者要人看。
# `decisions-history/` 这类由别的阶段管形状的子目录同样要有一行，表里写它与谁同进退。
#
# 双向比对用共用库 lib-manifest.py，与 50、62、63 号同一份代码。
#
# 判别力：fixtures/98-kb-registry.sh/red 放一份没登记的 kb 文件与一行指向空处的登记，必须判红；
# 它的表里另有一行只写 `.claude/kb/`、散文里另提一份文件，这两处都不许替那两份没登记的文件登记；green 两边对齐。
#
#   bash .claude/gate.d/98-kb-registry.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || { echo "  ✗ 进不去项目根 $ROOT"; echo "     → 怎么办：第一个参数给项目根（仓的顶层目录），不给就取这个脚本往上两级；路径写错或没有权限进去时这一道什么都没判。"; exit 2; }
python3 - "$(cd "$(dirname "$0")" && pwd)/lib-manifest.py" <<'PY'
import glob, importlib.util, os, re, sys

try:
    spec = importlib.util.spec_from_file_location("lib_manifest", sys.argv[1])
    manifest = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(manifest)
except OSError as error:
    print(f"  ✗ 读不到共用库 {sys.argv[1]}：{error}")
    print("     → 怎么办：目录与清单的双向比对只有那一份，恢复它，别在阶段里再抄一份。")
    sys.exit(1)

KB = ".claude/kb"
MANIFEST = "CLAUDE.md"
SECTION = "## 项目本地事实"

def fail(message, steps):
    print(f"  ✗ {message}")
    for step in steps:
        print(f"     → {step}")
    sys.exit(1)

if not os.path.isdir(KB):
    print(f"  ! 没有 {KB}，本阶段无对象可判")
    sys.exit(77)
if not os.path.isfile(MANIFEST):
    fail(f"找不到 {MANIFEST}", ["怎么办：项目说明改了名就同步改这个阶段里的路径。"])

text = open(MANIFEST, encoding="utf-8").read()
start = text.find(SECTION)
if start < 0:
    fail(f"{MANIFEST} 里没有「{SECTION}」这一节", [
        f"怎么办：这一节是 kb 各文件职责的登记位。改了标题就同步改这个阶段里的 SECTION，别让这道闸扫空。",
    ])
end = text.find("\n## ", start + len(SECTION))
section = text[start:end if end > 0 else len(text)]
registered = set(re.findall(r"`(\.claude/kb/[^`]*)`", section))
# 登记只认表里一行的第一格；带占位（<年-月>）的形态写在表里任一格都替它所在的子目录登记
table_lines = [line for line in section.split("\n") if line.strip().startswith("|")]
first_cells = set()
placeholder_directories = set()
for line in table_lines:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    first_cells |= set(re.findall(r"`(\.claude/kb/[^`]*)`", cells[0])) if cells else set()
    for entry in re.findall(r"`(\.claude/kb/[^`]*<[^`]*)`", line):
        placeholder_directories.add(entry[:entry.index("<")].rsplit("/", 1)[0] + "/")
if not registered:
    fail(f"「{SECTION}」那一节里一个 `.claude/kb/…` 路径都没点到", [
        "怎么办：扫到 0 项而报绿，与判过了一模一样。那一节的表每行第一格写路径，用反引号包起来。",
    ])

on_disk = set()
for path in sorted(glob.glob(f"{KB}/*.md")):
    on_disk.add(path)
for entry in sorted(os.listdir(KB)):
    full = os.path.join(KB, entry)
    if os.path.isdir(full):
        on_disk.add(full + "/")

def is_registered(path):
    # 逐字等于表里某一行的第一格才算登记；前缀不算——一行 `.claude/kb/` 会让每一项都满足前缀
    if path in first_cells:
        return True
    # 表里写成带月份占位的形态（`.claude/kb/decisions-history/<年-月>.md`）时，替它所在的那个子目录登记
    return path in placeholder_directories

def points_at_something(entry):
    # 带占位的形态（`<年-月>`）不是一个具体路径，不判它指不指得到
    return os.path.exists(entry) or os.path.exists(entry.rstrip("/")) or "<" in entry

missing, dangling = manifest.two_way(sorted(on_disk), sorted(registered),
                                     is_registered=is_registered, exists=points_at_something)

failed = False
if missing:
    failed = True
    print(f"  ✗ kb 里 {len(missing)} 份没有在 {MANIFEST} 的「{SECTION}」表里登记：")  # gate-lint:summary
    for path in missing:
        print(f"      {path}")  # gate-lint:detail
    print(f"     → 怎么办：往那张表加一行，第一格写路径（反引号包起来）、第二格写它是什么、归谁读、与别处什么关系。")
    print("               写不出那一行，多半说明这份文件该并进已有的某一份，或者压根不该单开。")
if dangling:
    failed = True
    print(f"  ✗ 那张表里 {len(dangling)} 个路径指向不存在的东西：")  # gate-lint:summary
    for entry in dangling:
        print(f"      {entry}")  # gate-lint:detail
    print("     → 怎么办：文件搬家或改名了就同步改那一行（.claude/rules/path-moves.md）；删掉了就把那一行撤掉——")
    print("               指向空处的登记比没有登记更糟，它让人以为那一格有人管着。")
if failed:
    sys.exit(1)

print(f"  ✓ kb 里 {len(on_disk)} 项（{len([p for p in on_disk if p.endswith('.md')])} 份 .md、"
      f"{len([p for p in on_disk if p.endswith('/')])} 个子目录）都在 {MANIFEST} 的「{SECTION}」表里登记着，"
      f"表里 {len(registered)} 个路径也都指得到东西")
PY
