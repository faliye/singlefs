#!/usr/bin/env python3
# admission: always 每一次调都比此刻点名的两份第二段产物，上一次的结论不替这一次作保
# run-condition: none 只读点名的两份产物文本，除了 python3 之外没有环境要求，不起 cargo、不编不跑
"""E165 第 1 次重跑（research/prompts/e165-r1-prereg.md）R3 的比对：第二段两份产物，每个变体（不含基线）按 title= 认，
比 outcome、first_clause_red 每个域的 N、first_anchor_red 每个域的 N、not_red_domains 的集合（登记第六节 RQ8 到 RQ10）。
table_row、rows_sharing_this_replacement、table、rerun、run_exit、copy_restored、red_examples* 与 name=config、name=restored、name=verdict、name=done 不比。
首红那个 N 上不同的判据名、other_difference_before_any_red、legal_family_minimum、adversarial_family_minimum（RQ11）与 verdict 的结局计数（RQ13）另附。
执行员在第三段草稿目录写成（compare-kill-bounds.py，sha256 ef2871c2…），入库时补了准入头、内嵌的合成样本与三条弄坏开关，比对的判法与输出不变。

用法：
  e165-compare-kill-bounds.py <入库那份> <今天那份> [--expected 27]
      每个变体一行 name=r3_variant，末尾一行 name=r3_summary；比了几行不等于 --expected（默认 27）退 2，有一个变体一格不同退 1，全同退 0。
  e165-compare-kill-bounds.py --selftest [<一份第二段产物>]
      登记 5.2 R3 那四格 + 比满行数那一格 + not_red_domains、first_anchor_red 各一格，共 7 项：自己比自己 0 行不同；改一个变体的 outcome= 恰 1 行不同且点名变体与字段；
      只改一个域的 N 恰 1 行不同；只改 table_row= 与 rerun= 0 行不同；漏读最后一行时比满那一格退 2；只改 not_red_domains、只改 first_anchor_red 一个域的 N 各恰 1 行不同。
      不给产物就拿内嵌的合成产物（SAMPLE_PRODUCT）；不开弄坏开关时另把三条弄坏开关逐个打开重跑一遍，每条都要让它点名的那一项判错。

弄坏开关 E165_COMPARE_KILL_BOUNDS_BREAK=<项>（登记 9.5 末三行；只给证红用）：
  match-by-table-row     按 table_row= 认变体、不按 title=                      「只改 table_row= 与 rerun=」那一项判错（由 0 行不同转非 0）
  clause-size-unchecked  不比 first_clause_red 每个域的 N                      「只改一个域的 N」那一项判错（由 1 行不同转 0）
  last-row-dropped       读产物时漏读最后一行 name=kill_bound                  「自己比自己」那一项判错（比的行数不满，第七节乙类「比了 27 行」那一格）
"""
import os
import re
import subprocess
import sys
import tempfile

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

TOKEN = re.compile(r'(\w+)=("[^"]*"|\S+)')
BASELINE_TITLE = "基线（不改）"
BREAK = {item for item in os.environ.get("E165_COMPARE_KILL_BOUNDS_BREAK", "").split(",") if item}
# 每条弄坏开关要让哪一项判错（自证里那一项的标签开头）
BREAK_NAMED_CHECK = {
    "match-by-table-row": "只改 table_row= 与 rerun=",
    "clause-size-unchecked": "只改一个域的 N",
    "last-row-dropped": "自己比自己",
}

# 内嵌的合成第二段产物：基线一行、四个变体（红、红且有实现锚点、只被实现锚点抓到、界内不红），字段的写法照第二段驱动的产物
SAMPLE_PRODUCT = "\n".join([
    "E7RESULT name=config experiment=E165 segment=2 bounds=segment2 items=4 table_rows=3 custom_rows=1 widening=none",
    'E7RESULT name=kill_bound item=baseline table=none table_row=none rows_sharing_this_replacement=none outcome=baseline result_rows=9 run_exit=0 '
    'rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --recheck --item baseline" title=基线（不改） complete=yes',
    'E7RESULT name=kill_bound item=tsv-23 table=crates/mutations.tsv table_row=23 rows_sharing_this_replacement=23 outcome=red first_clause_red=X:2:J1+J2 '
    'first_anchor_red=none other_difference_before_any_red=none not_red_domains=L(≤8),C(≤5) legal_family_minimum=2 adversarial_family_minimum=none run_exit=0 '
    'copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --recheck --item tsv-23" title=合成变体一 complete=yes',
    'E7RESULT name=kill_bound item=tsv-40 table=crates/mutations.tsv table_row=40 rows_sharing_this_replacement=40 outcome=red first_clause_red=L:3:J4;X:2:J1 '
    'first_anchor_red=L:2:failed_states other_difference_before_any_red=none not_red_domains=none legal_family_minimum=3 adversarial_family_minimum=2 run_exit=0 '
    'copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --recheck --item tsv-40" title=合成变体二 complete=yes',
    'E7RESULT name=kill_bound item=Z1 table=research/mutations/e165_replay_bounded_enumeration_custom.tsv table_row=2 rows_sharing_this_replacement=2 '
    'outcome=red_only_by_implementation_anchor first_clause_red=none first_anchor_red=U_verify_off:2:IA2 other_difference_before_any_red=none '
    'not_red_domains=X(≤5) legal_family_minimum=none adversarial_family_minimum=none run_exit=0 copy_restored=true '
    'rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --recheck --item Z1" title=合成变体三 complete=yes',
    'E7RESULT name=kill_bound item=tsv-50 table=crates/mutations.tsv table_row=50 rows_sharing_this_replacement=50 outcome=not_red_within_bounds '
    'first_clause_red=none first_anchor_red=none other_difference_before_any_red=none not_red_domains=X(≤5),L(≤8) legal_family_minimum=none '
    'adversarial_family_minimum=none run_exit=0 copy_restored=true rerun="bash research/scripts/e165-replay-mutation-kill-bounds.sh --recheck --item tsv-50" '
    'title=合成变体四 complete=yes',
    "E7RESULT name=verdict scope=segment2 variants=5 variants_excluding_baseline=4 red=2 red_only_by_implementation_anchor=1 not_red_within_bounds=1 invalid=0 run_did_not_finish=0",
    "E7RESULT name=done emitted=8",
    "",
])


class Unreadable(Exception):
    """产物里一行没有 title= 或 title 重复：message 是卡在哪。"""


def parse_rows(path):
    rows, order = {}, []
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    numbered = [(number, line) for number, line in enumerate(lines, 1) if line.startswith("E7RESULT name=kill_bound ")]
    if "last-row-dropped" in BREAK:
        numbered = numbered[:-1]
    for number, line in numbered:
        fields = dict(TOKEN.findall(line))
        fields["_line"] = str(number)
        title = fields.get("title")
        if "match-by-table-row" in BREAK and title != BASELINE_TITLE:
            title = fields.get("table_row")
        if title is None or title in rows:
            raise Unreadable(f"{path}:{number} 没有 title= 或 title 重复：{title}")
        rows[title] = fields
        order.append(title)
    verdict = [dict(TOKEN.findall(line)) for line in lines if line.startswith("E7RESULT name=verdict ")]
    return rows, order, (verdict[0] if verdict else {})


def by_domain(text):
    """'X:2:J1+J2;L:3:J4' -> {'X': (2, 'J1+J2'), 'L': (3, 'J4')}；'none' -> {}"""
    if text in (None, "none"):
        return {}
    result = {}
    for part in text.split(";"):
        domain, size, criteria = part.split(":", 2)
        result[domain] = (size, criteria)
    return result


def compare_variant(old, new):
    differences = []
    if old.get("outcome") != new.get("outcome"):
        differences.append(f"outcome:{old.get('outcome')}→{new.get('outcome')}")
    for field in ("first_clause_red", "first_anchor_red"):
        if field == "first_clause_red" and "clause-size-unchecked" in BREAK:
            continue
        old_map, new_map = by_domain(old.get(field)), by_domain(new.get(field))
        for domain in sorted(set(old_map) | set(new_map)):
            old_size = old_map[domain][0] if domain in old_map else "无"
            new_size = new_map[domain][0] if domain in new_map else "无"
            if old_size != new_size:
                differences.append(f"{field}.{domain}:N={old_size}→{new_size}")
    if set(filter(None, old.get("not_red_domains", "").split(","))) != set(filter(None, new.get("not_red_domains", "").split(","))):
        differences.append(f"not_red_domains:{old.get('not_red_domains')}→{new.get('not_red_domains')}")
    return differences


def side_notes(old, new):
    notes = []
    for field in ("first_clause_red", "first_anchor_red"):
        old_map, new_map = by_domain(old.get(field)), by_domain(new.get(field))
        for domain in sorted(set(old_map) & set(new_map)):
            if old_map[domain][0] == new_map[domain][0] and old_map[domain][1] != new_map[domain][1]:
                notes.append(f"{field}.{domain}:N={old_map[domain][0]}上判据名 {old_map[domain][1]}→{new_map[domain][1]}")
    for field in ("other_difference_before_any_red", "legal_family_minimum", "adversarial_family_minimum"):
        if old.get(field) != new.get(field):
            notes.append(f"{field}:{old.get(field)}→{new.get(field)}")
    return notes


def run_compare(old_path, new_path, expected, out):
    old_rows, old_order, old_verdict = parse_rows(old_path)
    new_rows, new_order, new_verdict = parse_rows(new_path)
    old_titles = [title for title in old_order if title != BASELINE_TITLE]
    new_titles = [title for title in new_order if title != BASELINE_TITLE]
    problems = []
    if BASELINE_TITLE not in old_rows or BASELINE_TITLE not in new_rows:
        problems.append("两份里有一份没有基线行")
    if set(old_titles) != set(new_titles):
        problems.append(f"两份的变体名单不同：只在入库那份 {sorted(set(old_titles) - set(new_titles))}，只在今天那份 {sorted(set(new_titles) - set(old_titles))}")
    compared, differing, differing_titles = 0, 0, []
    for title in old_titles:
        if title not in new_rows:
            continue
        old, new = old_rows[title], new_rows[title]
        differences = compare_variant(old, new)
        notes = side_notes(old, new)
        compared += 1
        if differences:
            differing += 1
            differing_titles.append(title)
        out.append(f"E7RESULT name=r3_variant title={title} old_line={old['_line']} new_line={new['_line']} old_table_row={old.get('table_row')} new_table_row={new.get('table_row')} "
                   f"outcome_old={old.get('outcome')} outcome_new={new.get('outcome')} same={'false' if differences else 'true'} "
                   f"differences={'|'.join(differences) if differences else 'none'} side_notes={'|'.join(notes) if notes else 'none'}")
    verdict_fields = ("variants", "red", "red_only_by_implementation_anchor", "not_red_within_bounds", "invalid", "run_did_not_finish")
    verdict_old = " ".join(f"{field}={old_verdict.get(field)}" for field in verdict_fields)
    verdict_new = " ".join(f"{field}={new_verdict.get(field)}" for field in verdict_fields)
    out.append(f"E7RESULT name=r3_summary compared={compared} expected={expected} differing={differing} "
               f"differing_titles={'+'.join(differing_titles) if differing_titles else 'none'} old_baseline_present={BASELINE_TITLE in old_rows} new_baseline_present={BASELINE_TITLE in new_rows} "
               f"verdict_same={'true' if verdict_old == verdict_new else 'false'} verdict_old=\"{verdict_old}\" verdict_new=\"{verdict_new}\"")
    if compared != expected:
        problems.append(f"比了 {compared} 行，应为 {expected}")
    for problem in problems:
        out.append(f"E7RESULT name=r3_problem text=\"{problem}\"")
    if problems:
        return 2
    return 1 if differing else 0


def summary_fields(lines):
    return dict(TOKEN.findall(next(line for line in lines if line.startswith("E7RESULT name=r3_summary "))))


def selftest_on(product, directory):
    """在一份产物上跑 7 项；交回 (查了几项, 判错的标签列表)。"""
    with open(product, encoding="utf-8") as handle:
        original = handle.read()
    lines = original.split("\n")
    variant_indexes = [i for i, line in enumerate(lines) if line.startswith("E7RESULT name=kill_bound ") and "title=基线" not in line]
    checked, wrong = 0, []

    def check(label, condition):
        nonlocal checked
        checked += 1
        if not condition:
            wrong.append(label)
        print(f"  {'✓' if condition else '✗'} {label}")  # gate-lint:detail

    def write(name, new_lines):
        path = os.path.join(directory, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("\n".join(new_lines))
        return path

    def compare(path, expected=len(variant_indexes)):
        out = []
        try:
            code = run_compare(product, path, expected, out)
        except Unreadable as unreadable:
            return 2, [f"E7RESULT name=r3_summary compared=读不出 differing=读不出 text=\"{unreadable}\""]
        return code, out

    code, out = compare(product)
    check(f"自己比自己：比满 {len(variant_indexes)} 个变体行、0 行不同、退 0",
          code == 0 and summary_fields(out)["compared"] == str(len(variant_indexes)) and summary_fields(out)["differing"] == "0")
    # 换一个 outcome=（取第一个 outcome=red 的变体）
    target = next(i for i in variant_indexes if " outcome=red " in lines[i])
    changed = list(lines)
    changed[target] = changed[target].replace(" outcome=red ", " outcome=not_red_within_bounds ", 1)
    code, out = compare(write("outcome.out", changed))
    title = dict(TOKEN.findall(lines[target]))["title"]
    check("改一个变体的 outcome=：恰 1 行不同、点名变体与 outcome", code == 1 and summary_fields(out)["differing"] == "1"
          and any(f"title={title} " in line and "differences=outcome:red→not_red_within_bounds" in line for line in out))
    # 只把一个域的 N 加一
    target = next(i for i in variant_indexes if re.search(r" first_clause_red=[A-Za-z0-9_]+:\d+:", lines[i]))
    changed = list(lines)
    match = re.search(r"( first_clause_red=[A-Za-z0-9_]+:)(\d+)(:)", changed[target])
    changed[target] = changed[target][:match.start()] + match.group(1) + str(int(match.group(2)) + 1) + match.group(3) + changed[target][match.end():]
    code, out = compare(write("clause-n.out", changed))
    check("只改一个域的 N：恰 1 行不同", code == 1 and summary_fields(out)["differing"] == "1")
    # 只改 table_row= 与 rerun=
    target = next(i for i in variant_indexes if " table_row=" in lines[i] and " rerun=" in lines[i])
    changed = list(lines)
    changed[target] = re.sub(r" table_row=\S+", " table_row=99999", changed[target], 1)
    changed[target] = re.sub(r' rerun="[^"]*"', ' rerun="换了的复跑命令"', changed[target], 1)
    code, out = compare(write("rerun.out", changed))
    check("只改 table_row= 与 rerun=：0 行不同", code == 0 and summary_fields(out)["differing"] == "0")
    # 漏读最后一行：比满行数那一格要红
    dropped = list(lines)
    del dropped[variant_indexes[-1]]
    code, out = compare(write("dropped.out", dropped))
    check("漏掉最后一个变体行：比的行数不满、退 2", code == 2 and summary_fields(out)["compared"] == str(len(variant_indexes) - 1))
    # 只改 not_red_domains
    target = next(i for i in variant_indexes if " not_red_domains=" in lines[i] and "not_red_domains=none" not in lines[i])
    changed = list(lines)
    changed[target] = re.sub(r" not_red_domains=\S+", " not_red_domains=none", changed[target], 1)
    code, out = compare(write("notred.out", changed))
    check("只改 not_red_domains：恰 1 行不同", code == 1 and summary_fields(out)["differing"] == "1")
    # 只改 first_anchor_red 里一个域的 N（取有 anchor 的变体）
    target = next(i for i in variant_indexes if re.search(r" first_anchor_red=[A-Za-z0-9_]+:\d+:", lines[i]))
    changed = list(lines)
    match = re.search(r"( first_anchor_red=[A-Za-z0-9_]+:)(\d+)(:)", changed[target])
    changed[target] = changed[target][:match.start()] + match.group(1) + str(int(match.group(2)) + 1) + match.group(3) + changed[target][match.end():]
    code, out = compare(write("anchor-n.out", changed))
    check("只改 first_anchor_red 一个域的 N：恰 1 行不同", code == 1 and summary_fields(out)["differing"] == "1")
    return checked, wrong


def selftest(product=None):
    with tempfile.TemporaryDirectory(prefix="e165-compare-kill-bounds-selftest-") as directory:
        if product is None:
            product = os.path.join(directory, "sample-kill-bounds.out")
            with open(product, "w", encoding="utf-8") as handle:
                handle.write(SAMPLE_PRODUCT)
        checked, wrong = selftest_on(product, directory)
    print(f"自检：查了 {checked} 项，{len(wrong)} 项不过")
    if wrong:
        print(f"  ✗ e165-compare-kill-bounds 自证 {checked} 项里 {len(wrong)} 项判错：{'；'.join(wrong)}")
        print("     → 怎么办：按判错那几项的标签找对应的比对函数（compare_variant、parse_rows、run_compare）修；判据写在文件头")
        return 1
    if BREAK:
        print("（开着弄坏开关，没重跑开关）")
        return 0
    missed = []
    for item, named in BREAK_NAMED_CHECK.items():
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--selftest"], capture_output=True, text=True,
                                   env=dict(os.environ, E165_COMPARE_KILL_BOUNDS_BREAK=item))
        if completed.returncode == 0 or not any(line.startswith(f"  ✗ {named}") for line in completed.stdout.split("\n")):
            missed.append(f"{item}（点名「{named}」）")
    if missed:
        print(f"  ✗ 弄坏开关打开之后点名的那一项没判错：{' '.join(missed)}")
        print("     → 怎么办：那条开关管的那一段没有自证盯着：补一项会被它弄错的自证，或修开关")
        return 1
    print(f"  ✓ e165-compare-kill-bounds 自证 {checked} 项都判对；弄坏开关 {len(BREAK_NAMED_CHECK)} 个，每个都让它点名的那一项判错")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"] and len(argv) <= 2:
        return selftest(argv[1] if len(argv) == 2 else None)
    expected = 27
    if "--expected" in argv:
        position = argv.index("--expected")
        expected = int(argv[position + 1])
        argv = argv[:position] + argv[position + 2:]
    if len(argv) != 2:
        print(__doc__)
        return 2
    out = []
    try:
        code = run_compare(argv[0], argv[1], expected, out)
    except Unreadable as unreadable:
        print(f"  ✗ {unreadable}")
        print("     → 怎么办：两份都要是第二段驱动写的产物（每行 name=kill_bound 带唯一的 title=）；手改坏了从 git 找回")
        return 2
    print("\n".join(out))
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
