#!/usr/bin/env bash
# gate-stage: 改了规则、agent、hook、门禁、脚本或实现之后，有没有写阶段同步记录
#
# 为什么：2026-09-17 一批改动做完后没人回头同步知识——records/2026-09-16-subagent拆分提案.md 与 .claude/agent-common.md
# 里两句「还没用过」在用过之后都留着，另一个会话在那句正下方追加新一批也没改它。用户定：阶段任务结束时有一个同步知识的任务点，
# 派 sweep 做阶段同步，主 agent 判完每处命中写一份同步记录。这一道判的是那份记录的形式。
#
# 改动范围：GATE_BASE 给了就与它比，否则与 @{upstream} 的 merge-base 比，都没有就与 HEAD 比；工作区、暂存区与未跟踪文件都算
# （取法与 56-crates-adversarial-review.sh 相同，只多一个 core.quotepath=false：不加的话中文路径被 git 转成带引号的八进制，载体对不上）。
#
# 判据：
#   ① 触发文件：改动范围里匹配下面任一条的——^\.claude/agents/、^\.claude/agent-common\.md$、^\.claude/hooks/、
#      ^\.claude/settings\.json$、^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$、^\.claude/rules/、^research/scripts/、
#      ^crates/[^/]+/src/、^CLAUDE\.md$。一个都没有 ⇒ 无对象可判，退 77（不记通过）。
#   ② 同步记录：改动范围里的 research/prompts/<阶段>-sync.md（prompts 顶层），且文件里有一行只写 <!-- knowledge-sync -->。
#      每个触发文件都要在某份同步记录里按路径逐字出现（等同 grep -F）；漏的逐个列出 ⇒ 红。
#   ③ 每份同步记录有「## 搜索」小节且不空 ⇒ 否则红。
#   ④ 每份同步记录有「## 命中处置」小节，小节里恰好一张表，表头 | 载体 | 原句 | 处置 |。逐行：
#      载体格是「路径:行号」，路径从仓库根起（可包一层反引号），在仓里存在或在改动范围里；
#      处置格以「改了」「补了」「不改：」之一开头，「不改：」后面要写理由；
#      处置是「改了」「补了」的，载体路径要在改动范围里（记录说改了而文件没动）。任一不合 ⇒ 红。表可以 0 行。
#
# 同步记录的格式（research/prompts/<阶段>-sync.md）：
#   <!-- knowledge-sync -->
#   # <阶段> 阶段同步
#
#   触发文件：.claude/agents/sweep.md、research/scripts/agent-watch.py（这一阶段改动范围里的触发文件，逐个按路径写全）
#
#   ## 搜索
#   回扫用的每条命令与它的命中计数，例：grep -rn "<旧说法>" .claude records research | wc -l → 3
#
#   ## 命中处置
#   | 载体 | 原句 | 处置 |
#   |---|---|---|
#   | .claude/agent-common.md:120 | <那一行的原句> | 改了：<改成了什么> |
#   | .claude/kb/pitfalls.md:40 | （新增） | 补了：<补了什么> |
#   | records/2026-09-16-subagent拆分提案.md:88 | <那一行的原句> | 不改：<理由，例：说的是那一次发生的事> |
#   两个小节标题逐字写，后面不加字；原句里的 | 写成 \|；代码围栏里的 # 行不算标题。
#
# 管不到的：记录写得对不对（原句是不是那一行、「不改」的理由站不站得住）、回扫搜没搜全、
# 只用过而没改动的东西（用过之后该改的句子，只要没碰触发范围就不触发）——这些靠阶段收尾的任务点与人。
# 「改了」「补了」只核载体文件在改动范围里，不核那一行真的动了；行号不核是否越界；路径里带制表符、换行或引号的，git 仍会转义，认不出。
# 判别力：fixtures/68-knowledge-sync.sh/red 放一个没被点名的触发文件、一个只在不带标记的文件里点名的触发文件、
# 说改了而载体没动、开头不合法、不改没理由、载体没行号、载体不存在、缺「## 搜索」、表头写错，必须判红；
# green 放三个触发文件分在两份记录里点名、三种处置各一行、中文文件名的载体、原句里的 \|、表后围栏里的竖线行、一份 0 行的表，必须判绿并报对数。
#
#   bash .claude/gate.d/68-knowledge-sync.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
git rev-parse --is-inside-work-tree >/dev/null 2>&1 || { echo "  ! $ROOT 不是 git 仓，本阶段跳过"; exit 77; }
base="HEAD"
if [[ -n "${GATE_BASE:-}" ]] && git rev-parse --verify -q "${GATE_BASE}^{commit}" >/dev/null 2>&1; then
  base="$GATE_BASE"
elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1; then
  base="$(git merge-base HEAD '@{upstream}')"
fi
changed="$( { git -c core.quotepath=false diff --name-only "$base" -- ; git -c core.quotepath=false diff --name-only --cached -- ; git -c core.quotepath=false ls-files --others --exclude-standard -- ; } | sort -u )"
# 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
python3 - "$base" <(printf '%s\n' "$changed") <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
with open(sys.argv[2], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})
changed_set = set(changed_files)

trigger_patterns = [re.compile(pattern) for pattern in (
    r"^\.claude/agents/",
    r"^\.claude/agent-common\.md$",
    r"^\.claude/hooks/",
    r"^\.claude/settings\.json$",
    r"^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$",
    r"^\.claude/rules/",
    r"^research/scripts/",
    r"^crates/[^/]+/src/",
    r"^CLAUDE\.md$",
)]
trigger_files = [path for path in changed_files if any(pattern.search(path) for pattern in trigger_patterns)]
outside_trigger_count = len(changed_files) - len(trigger_files)
if not trigger_files:
    print(f"  ! 这次改动没碰规则、agent、hook、门禁、脚本与实现，本阶段无对象可判（改动范围 {len(changed_files)} 个文件，基准 {base}）")
    sys.exit(77)

marker_line = "<!-- knowledge-sync -->"
record_name = re.compile(r"^research/prompts/[^/]+-sync\.md$")
records = []
fresh_records = set()
unmarked_candidates = []
for path in changed_files:
    if not record_name.search(path) or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if any(line.strip() == marker_line for line in text.split("\n")):
        records.append((path, text))
        # 这一轮**新写**的那几份：处置行的「改了 / 补了」只对它们判。
        # ⚠️ 跨轮留存的旧记录点名的载体是更早一轮改的，相对今天的基准当然不在范围里，
        # 而那些行说的都是真话——改写它们才是假的（C450 实测 2026-09-21）。
        # 判据用同一个基准：相对基准是新增（A）的才算这一轮写的；只是被改过（M，例如
        # 按 C450 追加一句「为什么这一轮还留着」）的不算。
        # 判据是「基准那一版里有没有这个文件」，不是 `git diff --name-status`：
        # 后者看不见**未跟踪**的新文件，而这一轮刚写出来还没 add 的记录正是那一种
        # （门禁自己的红样本就这么造的，用 diff 判会把它当成旧记录放行）。
        exists_at_base = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_base:
            fresh_records.add(path)
    else:
        unmarked_candidates.append(path)

fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def second_level_sections(text):
    """二级标题整行 → 小节体 [(行号, 行, 是否在代码围栏里)]；同名标题只认第一次，代码围栏里的 # 行不算标题。"""
    sections = {}
    current_body = None
    open_fence = None
    for line_number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            fence_match = fence_open.match(line)
            if fence_match:
                open_fence = fence_match.group(1)
                if current_body is not None:
                    current_body.append((line_number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                if title.startswith("## ") and title not in sections:
                    sections[title] = []
                    current_body = sections[title]
                else:
                    current_body = None
                continue
            if current_body is not None:
                current_body.append((line_number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current_body is not None:
                current_body.append((line_number, line, True))
    return sections


def table_cells(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")

missing_triggers = [path for path in trigger_files if not any(path in text for _record_path, text in records)]
search_problems = []
table_problems = []
carrier_problems = []
disposition_start_problems = []
empty_reason_problems = []
out_of_range_problems = []
disposition_counts = {"改了": 0, "补了": 0, "不改": 0}

for record_path, text in records:
    sections = second_level_sections(text)
    if "## 搜索" not in sections:
        search_problems.append(f"{record_path}：没有「## 搜索」小节")
    elif not any(line.strip() for _number, line, _fenced in sections["## 搜索"]):
        search_problems.append(f"{record_path}：「## 搜索」小节是空的")
    if "## 命中处置" not in sections:
        table_problems.append(f"{record_path}：没有「## 命中处置」小节")
        continue
    table_runs = []
    previous_was_table_line = False
    for line_number, line, fenced in sections["## 命中处置"]:
        is_table_line = (not fenced) and line.lstrip().startswith("|")
        if is_table_line and not previous_was_table_line:
            table_runs.append([])
        if is_table_line:
            table_runs[-1].append((line_number, line))
        previous_was_table_line = is_table_line
    if not table_runs:
        table_problems.append(f"{record_path}：「## 命中处置」小节里没有表")
        continue
    if len(table_runs) > 1:
        table_problems.append(f"{record_path}：「## 命中处置」小节里有 {len(table_runs)} 张表，只许一张（第二张从第 {table_runs[1][0][0]} 行起）")
        continue
    table_rows = table_runs[0]
    header_number, header_line = table_rows[0]
    if table_cells(header_line) != ["载体", "原句", "处置"]:
        table_problems.append(f"{record_path}:{header_number}：「## 命中处置」的表头不是 | 载体 | 原句 | 处置 |，实际是 {header_line.strip()}")
        continue
    if len(table_rows) < 2 or len(table_cells(table_rows[1][1])) != 3 or not all(separator_cell.match(cell) for cell in table_cells(table_rows[1][1])):
        table_problems.append(f"{record_path}:{header_number + 1}：表头下面一行不是三格的分隔行 |---|---|---|")
        continue
    for row_number, row_line in table_rows[2:]:
        cells = table_cells(row_line)
        location = f"{record_path}:{row_number}"
        if len(cells) != 3:
            table_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
            continue
        carrier_text, _original_sentence, disposition = cells
        if len(carrier_text) >= 2 and carrier_text.startswith("`") and carrier_text.endswith("`"):
            carrier_text = carrier_text[1:-1].strip()
        carrier_match = carrier_form.match(carrier_text)
        carrier_path = None
        if not carrier_match:
            carrier_problems.append(f"{location}：载体格不是「路径:行号」：{carrier_text}")
        else:
            candidate_path = carrier_match.group("path")
            if candidate_path.startswith(("/", "./", "../")) or "/../" in candidate_path:
                carrier_problems.append(f"{location}：载体路径不是从仓库根起写的：{candidate_path}")
            elif not os.path.exists(candidate_path) and candidate_path not in changed_set:
                carrier_problems.append(f"{location}：载体路径 {candidate_path} 在仓里不存在，也不在改动范围里")
            else:
                carrier_path = candidate_path
        if disposition.startswith("改了") or disposition.startswith("补了"):
            disposition_counts[disposition[:2]] += 1
            if (record_path in fresh_records
                    and carrier_path is not None and carrier_path not in changed_set):
                out_of_range_problems.append(f"{location}：处置写「{disposition[:2]}」，载体 {carrier_path} 不在改动范围里")
        elif disposition.startswith("不改："):
            disposition_counts["不改"] += 1
            if not disposition[len("不改："):].strip():
                empty_reason_problems.append(f"{location}：「不改：」后面没写理由")
        else:
            disposition_start_problems.append(f"{location}：处置开头不合法：{disposition or '（空）'}")

sync_step = ("阶段同步怎么做：阶段任务结束时派 sweep 做阶段同步（sweep 定义里的第四种活），主 agent 判完每处命中写 "
             "research/prompts/<阶段>-sync.md，格式见 .claude/gate.d/68-knowledge-sync.sh 文件头。")
failed = False
if missing_triggers:
    failed = True
    print(f"  ✗ {len(missing_triggers)} 个触发文件没在任何同步记录里点名（改动范围里带 <!-- knowledge-sync --> 的 research/prompts/*-sync.md 共 {len(records)} 份，一份都没按路径提到它）：")
    for path in missing_triggers:
        print(f"      {path}")
    if unmarked_candidates:
        print("     这几份文件名像同步记录，但没有只写 <!-- knowledge-sync --> 的那一行，不算：")
        for path in unmarked_candidates:
            print(f"      {path}")
    print("     → 怎么办：这一阶段还没做知识同步就先做；做过的，把上面每个文件按路径写全补进同步记录的「触发文件」一行。")
if search_problems:
    failed = True
    print(f"  ✗ {len(search_problems)} 份同步记录缺「## 搜索」或它是空的：")
    for entry in search_problems:
        print(f"      {entry}")
    print("     → 怎么办：加「## 搜索」小节（标题逐字写、后面不加字），写回扫用的每条命令与它的命中计数。")
if table_problems:
    failed = True
    print(f"  ✗ {len(table_problems)} 处「## 命中处置」的表写坏了：")
    for entry in table_problems:
        print(f"      {entry}")
    print("     → 怎么办：「## 命中处置」下面恰好一张表，表头逐字是 | 载体 | 原句 | 处置 |，第二行 |---|---|---|；每行三格，原句里的 | 写成 \\|；没有命中也留表头（0 行）。")
if carrier_problems:
    failed = True
    print(f"  ✗ {len(carrier_problems)} 行载体格不是仓库根起的「路径:行号」或指不到文件：")
    for entry in carrier_problems:
        print(f"      {entry}")
    print("     → 怎么办：载体格写仓库根起的 路径:行号（例 .claude/agent-common.md:120），路径要在仓里现存；别写成相对 research/prompts/ 的路径。")
if disposition_start_problems:
    failed = True
    print(f"  ✗ {len(disposition_start_problems)} 行处置开头不合法（只认「改了」「补了」「不改：」）：")
    for entry in disposition_start_problems:
        print(f"      {entry}")
    print("     → 怎么办：每处命中先判完再写：改了写「改了：…」，新加的写「补了：…」，不动写「不改：理由」；别写「待定」「看过」。")
if empty_reason_problems:
    failed = True
    print(f"  ✗ {len(empty_reason_problems)} 行「不改：」没写理由：")
    for entry in empty_reason_problems:
        print(f"      {entry}")
    print("     → 怎么办：「不改：」后面写不改的理由，例：冻结证据 / 说的是那一次发生的事 / 同一个词、不同的事。")
if out_of_range_problems:
    failed = True
    print(f"  ✗ {len(out_of_range_problems)} 行处置写「改了」「补了」而载体不在改动范围里（记录说改了，文件没动）：")
    for entry in out_of_range_problems:
        print(f"      {entry}")
    print("     → 怎么办：去把那一处真的改了；判下来不该改的，处置写成「不改：理由」；也对一遍载体路径是不是写成了别的文件。")
if failed:
    print(f"     → {sync_step}")
    sys.exit(1)
hit_rows = sum(disposition_counts.values())
print(f"  ✓ 触发文件 {len(trigger_files)} 个都在同步记录里点名（同步记录 {len(records)} 份；命中 {hit_rows} 行："
      f"改了 {disposition_counts['改了']} / 补了 {disposition_counts['补了']} / 不改 {disposition_counts['不改']}；基准 {base}）；"
      f"改动范围里另有 {outside_trigger_count} 个文件不在触发范围")
PY
