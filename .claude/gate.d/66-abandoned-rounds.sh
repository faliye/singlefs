#!/usr/bin/env bash
# gate-stage: 发过腿的三方轮次，要么有判决，要么登记撂下
#
# 实测（2026-09-17 核出）：c245-r3 与 c355-c363-r3 两轮 2026-09-16 18:04–18:13 UTC 发了云端腿提示、本地腿交了样本，
# 云端腿没交、没有判决，之后没人收尾，也没有任何东西报警（里程碑「第二个事务」增补 2 收口表第 33 行）。
#
# 判据（只看 research/prompts/ 顶层文件，不进子目录）：
#   ① 认轮次。文件名是下面五种之一，就算这一轮发过腿；轮名必须以 -r<N> 结尾：
#        _<轮>-body.md          主 agent 写的三方正文（CLAUDE.md 派发表）
#        <轮>-opus.md           云端攻方腿的提示
#        <轮>-sonnet.md         云端正推 / 辩方腿的提示
#        <轮>-local-attack.md   本地攻方腿的提示（.claude/agents/three-way-local-attack.md）
#        <轮>-local-defense.md  本地辩方腿的提示（.claude/agents/three-way-local-defense.md）
#   ② 认判决。两种任一：
#        research/prompts/<轮>-main-verification.md；
#        .claude/kb/ 下某个 .md 的同一小节（一个标题到下一个标题之间）既点名这一轮的材料
#        （出现 research/prompts/<轮>- 或 research/prompts/_<轮>-），又有「**判决」——
#        2026-09-11 有判决文件之前，判决直接写进决策正文或变更史，d16-item1-r2、d26-item4-r1、d26-item5-r4 三轮是这样。
#   ③ 认撂下。research/prompts/abandoned-rounds.tsv 一行一轮，四列用制表符分隔：轮名、日期、为什么撂下、去向。
#   发过腿、没有判决、也没登记 ⇒ 红。登记表写坏（不是四列、有空列、日期不是 YYYY-MM-DD、同一轮登记两行）、
#   登记了一个目录里认不出的轮、登记了已经有判决的轮 ⇒ 也红。
#
# 不判的，成功行逐个列出前缀（现算）：腿提示是旧形态的轮——<题>-local.md、<题>-forward-sonnet.md / -reverse-opus.md
# 这类，或轮名不以 -r<N> 结尾（arm3、d16-items134）。2026-09-17 现查这些文件的首次提交全部在 2026-09-11 14:10 UTC 或更早，
# 那时还没有判决文件（第一份 2026-09-11 16:35 提交），判决直接写进 kb、形态不一，按 ② 认会把「判决」读成任意一句提到材料的话；
# 此后仍用 -local.md 写本地腿提示的轮（到 2026-09-16 为止）都另有 _<轮>-body.md 或 <轮>-opus.md / -sonnet.md，已被 ① 认出；agent 定义要求写成 -local-attack / -local-defense。
#
# ⚠️ 管不到的：判决写得对不对、登记的理由真不真（靠人）；② 的第二种只认「同一小节里既点名又有 **判决」，
# 一段写着「**判决**：没判」的小节也算判过；新轮若既不写正文、腿提示又用了上面五种之外的名字，它落进不判的那一列而不红。
# 判别力：fixtures/66-abandoned-rounds.sh/red 放一轮发了腿没判决没登记、一轮只在 kb 里被提到而没有判决、
# 写坏与说反话的登记（认不出的轮、有判决的轮、日期写坏、少一列、同一轮两行），必须判红；green 放判决文件、kb 小节判决、登记撂下各一轮与三个旧形态前缀，必须判绿并报对数。
#
#   bash .claude/gate.d/66-abandoned-rounds.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" 2>/dev/null || exit 2
python3 - <<'PY'
import os, re, sys

prompts_dir = "research/prompts"
registry_path = "research/prompts/abandoned-rounds.tsv"
kb_dir = ".claude/kb"

if not os.path.isdir(prompts_dir):
    print(f"  ! 没有 {prompts_dir}，本阶段无对象可判")
    sys.exit(77)

round_name = r"(?P<round>[a-z0-9][a-z0-9-]*-r[0-9]+)"
dispatch_forms = (
    re.compile(r"^_" + round_name + r"-body\.md$"),
    re.compile(r"^" + round_name + r"-opus\.md$"),
    re.compile(r"^" + round_name + r"-sonnet\.md$"),
    re.compile(r"^" + round_name + r"-local-attack\.md$"),
    re.compile(r"^" + round_name + r"-local-defense\.md$"),
)
legacy_body_form = re.compile(r"^_(?P<prefix>.+)-body\.md$")
legacy_leg_form = re.compile(r"^(?P<prefix>[^_].*?)(?:(?:-(?:forward|reverse|attack|defense|admission|ledger))?-(?:opus|sonnet)|-local|-local-attack|-local-defense)\.md$")

prompt_files = sorted(name for name in os.listdir(prompts_dir) if os.path.isfile(os.path.join(prompts_dir, name)))
dispatch_files_by_round = {}
unrecognized_files = []
for name in prompt_files:
    matched_round = None
    for form in dispatch_forms:
        match = form.match(name)
        if match:
            matched_round = match.group("round")
            break
    if matched_round is not None:
        dispatch_files_by_round.setdefault(matched_round, []).append(name)
    elif legacy_body_form.match(name) or legacy_leg_form.match(name):
        unrecognized_files.append(name)

legacy_prefixes = set()
for name in unrecognized_files:
    match = legacy_body_form.match(name) or legacy_leg_form.match(name)
    prefix = match.group("prefix")
    if prefix not in dispatch_files_by_round:
        legacy_prefixes.add(prefix)

if not dispatch_files_by_round:
    print(f"  ! {prompts_dir} 下没有一个文件名认得出发过腿的轮次（_<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md），本阶段无对象可判")
    sys.exit(77)

# kb 小节：一个标题行到下一个标题行之间。只读还没有判决文件的轮要用到的那几条。
rounds_without_verdict_file = [name for name in dispatch_files_by_round if not os.path.isfile(os.path.join(prompts_dir, f"{name}-main-verification.md"))]
kb_sections = []
if rounds_without_verdict_file and os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            section_lines = []
            section_heading = ""
            for line in open(path, encoding="utf-8"):
                if re.match(r"^#{1,6} ", line):
                    if section_lines:
                        kb_sections.append((path, section_heading, "".join(section_lines)))
                    section_lines = [line]
                    section_heading = line.strip()
                else:
                    section_lines.append(line)
            if section_lines:
                kb_sections.append((path, section_heading, "".join(section_lines)))

def kb_verdict_location(name):
    citation = re.compile(r"research/prompts/_?" + re.escape(name) + r"-")
    for path, heading, text in kb_sections:
        if citation.search(text) and "**判决" in text:
            return f"{path}「{heading}」"
    return None

verdict_file_rounds = []
kb_verdict_rounds = []
rounds_without_any_verdict = []
for name in sorted(dispatch_files_by_round):
    if name not in rounds_without_verdict_file:
        verdict_file_rounds.append(name)
    elif kb_verdict_location(name):
        kb_verdict_rounds.append(name)
    else:
        rounds_without_any_verdict.append(name)

registered_line_numbers = {}
malformed_registry_rows = []
if os.path.isfile(registry_path):
    for line_number, line in enumerate(open(registry_path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4 or any(not field.strip() for field in fields):
            malformed_registry_rows.append(f"第 {line_number} 行不是四列、或有空列：{line}")
            continue
        if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", fields[1].strip()):
            malformed_registry_rows.append(f"第 {line_number} 行日期不是 YYYY-MM-DD：{fields[1]}")
            continue
        registered_line_numbers.setdefault(fields[0].strip(), []).append(line_number)

duplicated_registrations = [f"{name}（第 {', '.join(map(str, numbers))} 行）" for name, numbers in sorted(registered_line_numbers.items()) if len(numbers) > 1]
registered_unknown_rounds = [name for name in sorted(registered_line_numbers) if name not in dispatch_files_by_round]
registered_rounds_with_verdict = [name for name in sorted(registered_line_numbers) if name in verdict_file_rounds or name in kb_verdict_rounds]
unjudged_unregistered_rounds = [name for name in rounds_without_any_verdict if name not in registered_line_numbers]
abandoned_rounds = [name for name in rounds_without_any_verdict if name in registered_line_numbers]

failed = False
if unjudged_unregistered_rounds:
    failed = True
    print("  ✗ 这些三方轮次发过腿，却没有判决、也不在撂下登记表里：")  # gate-lint:summary
    for name in unjudged_unregistered_rounds:
        print(f"     {name}（{'、'.join(dispatch_files_by_round[name])}）")  # gate-lint:detail
    print(f"     → 怎么办：补判决（写 {prompts_dir}/<轮>-main-verification.md），或在 {registry_path} 加一行：轮名、日期、为什么撂下、去向（四列，制表符分隔）。")
    print("               腿还在跑的轮次，等腿交齐、判决写完再跑门禁；只在 kb 里提到这一轮而没有「**判决」的小节不算判过。")
if malformed_registry_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行写坏了：")  # gate-lint:summary
    for entry in malformed_registry_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：轮名、日期（YYYY-MM-DD）、为什么撂下、去向；四列都要有内容，# 开头的行是注释。")
if duplicated_registrations:
    failed = True
    print("  ✗ 这些轮次在撂下登记表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated_registrations:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，理由与去向写在同一行里。")
if registered_unknown_rounds:
    failed = True
    print(f"  ✗ 撂下登记表登记了这些轮，{prompts_dir} 里认不出它们发过腿（没有 _<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md）：")  # gate-lint:summary
    for name in registered_unknown_rounds:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：轮名多半写错了，改成目录里文件名的前缀；那一轮的文件搬过家就照 .claude/rules/path-moves.md 改；确实没有这一轮就删掉这一行。")
if registered_rounds_with_verdict:
    failed = True
    print("  ✗ 这些轮次登记了撂下，却已经有判决——两处说反话：")  # gate-lint:summary
    for name in registered_rounds_with_verdict:
        where = f"{prompts_dir}/{name}-main-verification.md" if name in verdict_file_rounds else kb_verdict_location(name)
        print(f"     {name}：判决在 {where}")  # gate-lint:detail
    print("     → 怎么办：判决是这一轮的现状，就删掉登记表里那一行；判决那一处其实没判，就把它改成不再写「**判决」或删掉那份判决文件，再留登记。")
if failed:
    sys.exit(1)

checked = len(dispatch_files_by_round)
print(f"  ✓ 发过腿的三方轮次都有判决或撂下登记（查了 {checked} 轮：判决文件 {len(verdict_file_rounds)} 轮、kb 小节里的判决 {len(kb_verdict_rounds)} 轮、登记撂下 {len(abandoned_rounds)} 轮）")
if legacy_prefixes:
    print(f"    没判 {len(legacy_prefixes)} 个前缀（腿提示是旧形态：<题>-local.md、<题>-forward-sonnet.md 这类，或轮名不以 -r<N> 结尾）：{'、'.join(sorted(legacy_prefixes))}")
else:
    print("    没判 0 个前缀")
PY
