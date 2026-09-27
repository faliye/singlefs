"""K4 探针：Y1 让整份 admission.py（与整份 54 号）进每条崩溃枚举用例的指纹。量三样：
① admission.py 里真正决定「日志判不判绿、标记作不作数、登记行怎么读」的那几段（从入口按 ast 顺着引用的模块级名字求闭包）占全文多少行；
② 这一份窄的「判法摘要」（闭包里每个定义的源码按名字排好拼起来的 sha256，我的模型，不在被判代码里）对几种改动变不变：
   改实验准入那几段、改自证、改判法本身、改判法调的小函数；另拿改法之前的备份与今天的比；
③ 这一趟 Y1–Y8 对 admission.py 的每一处改动（附录二的 hunk）落在闭包里还是闭包外。
复跑：python3 probe_k4_cost.py [--before <改法之前的 admission.py>]"""
import ast
import hashlib
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

ENTRY_POINTS = ("judge_crash_case_log", "judge_worker_threads", "crash_case_marker_problems", "read_crash_case_marker",
                "check_crash_case_marker", "write_crash_case_marker", "crash_case_marker_path", "parse_crash_case", "crash_case_of_key",
                "crash_cases_of", "command_crash_case_judge", "command_crash_case_record", "command_crash_case_marker_check")
BEFORE = sys.argv[sys.argv.index("--before") + 1] if "--before" in sys.argv else "/tmp/claude-1000/gate-batch-m2-r1-fixes/backup/research/scripts/admission.py"
DIFF = os.path.join(pc.REPOSITORY, "research/prompts/_defs-gatebatch-m2-r2-diff.md")


def top_level_definitions(tree):
    """模块级的 def / class / 赋值：名字 → 节点。"""
    found = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            found[node.name] = node
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            for target in (node.targets if isinstance(node, ast.Assign) else [node.target]):
                if isinstance(target, ast.Name):
                    found[target.id] = node
    return found


def closure(source):
    tree = ast.parse(source)
    definitions = top_level_definitions(tree)
    wanted, queue = set(), [name for name in ENTRY_POINTS if name in definitions]
    while queue:
        name = queue.pop()
        if name in wanted:
            continue
        wanted.add(name)
        for inner in ast.walk(definitions[name]):
            if isinstance(inner, ast.Name) and inner.id in definitions and inner.id not in wanted:
                queue.append(inner.id)
    return definitions, wanted


def judge_digest(source):
    definitions, wanted = closure(source)
    lines = source.split("\n")
    pieces = []
    for name in sorted(wanted):
        node = definitions[name]
        pieces.append(f"## {name}\n" + "\n".join(lines[node.lineno - 1:node.end_lineno]))
    return hashlib.sha256("\n".join(pieces).encode()).hexdigest(), wanted, definitions


with open(pc.ADMISSION, encoding="utf-8") as handle:
    today = handle.read()
digest_today, wanted, definitions = judge_digest(today)
closure_lines = sum(definitions[name].end_lineno - definitions[name].lineno + 1 for name in wanted)
total_lines = today.count("\n")
selftest_start = min(node.lineno for name, node in definitions.items() if name in ("Selftest",))
print(f"MEASURE admission.py 全文 {total_lines} 行；闭包 {len(wanted)} 个定义、{closure_lines} 行（{closure_lines * 100 // total_lines}%）；"
      f"自证从第 {selftest_start} 行起到文件尾共 {total_lines - selftest_start + 1} 行")
print("MEASURE 闭包里的名字：" + " ".join(sorted(wanted)))
cells = pc.Cells("K4 窄的判法摘要")


def variant(label, edit, want_change):
    edited = edit(today)
    assert edited != today, label
    changed_digest = judge_digest(edited)[0] != digest_today
    changed_whole = hashlib.sha256(edited.encode()).hexdigest() != hashlib.sha256(today.encode()).hexdigest()
    cells.expect("CONTROL" if want_change else "COST", f"{label} ⇒ 判法摘要应当{'变' if want_change else '不变'}",
                 changed_digest == want_change, f"判法摘要{'变了' if changed_digest else '没变'}；整份 admission.py 的哈希（Y1 进指纹的那一份）{'变了' if changed_whole else '没变'}")


def replace_once(old, new):
    def edit(text):
        assert text.count(old) == 1, old
        return text.replace(old, new)
    return edit


variant("V1 实验准入：admit_experiment 里加一行注释", replace_once("def admit_experiment(root, key):\n", "def admit_experiment(root, key):\n    # 注释\n"), False)
variant("V2 门禁复用：gate_reuse 的一句说明改字", replace_once("按要跑处理\"\n    staged_tree", "按要跑处理（改字）\"\n    staged_tree"), False)
variant("V3 自证：run_selftest 里加一行注释", replace_once("def run_selftest():\n", "def run_selftest():\n    # 注释\n"), False)
variant("V4 排除规则：files_crash_cases_do_not_read 改一句（输出的清单变了会自己进指纹）",
        replace_once("def files_crash_cases_do_not_read(root, listed_names, code_texts):\n", "def files_crash_cases_do_not_read(root, listed_names, code_texts):\n    # 注释\n"), False)
variant("V5 判法：judge_worker_threads 把「至少两片」改成「至少三片」",
        replace_once("judged_on_one_thread = freshly_run_slices >= 2 and worker_threads == 1", "judged_on_one_thread = freshly_run_slices >= 3 and worker_threads == 1"), True)
variant("V6 判法调的小函数：fields_of_line 改写法", replace_once('    return dict(token.split("=", 1) for token in line.split() if "=" in token)',
                                                 '    return dict(token.split("=", 1) for token in line.split(" ") if "=" in token)'), True)
variant("V7 判法用的常量：PASSED_ONE_TEST_FORM 放宽", replace_once('PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\\. 1 passed; 0 failed; ")',
                                                  'PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\\. [0-9]+ passed; 0 failed; ")'), True)
variant("V8 登记行怎么读：parse_crash_case 不再要求 exhaustive= 先登记成 count-line=",
        replace_once("    unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]",
                     "    unregistered = []"), True)
if os.path.isfile(BEFORE):
    with open(BEFORE, encoding="utf-8") as handle:
        before = handle.read()
    digest_before, wanted_before, _definitions = judge_digest(before)
    print(f"MEASURE 改法之前的备份 {BEFORE}：判法摘要 {digest_before[:16]}…，今天 {digest_today[:16]}…，"
          f"{'不同' if digest_before != digest_today else '相同'}；闭包名字差：多 {sorted(wanted - wanted_before)} 少 {sorted(wanted_before - wanted)}")
# ③ 附录二里 admission.py 的每个 hunk：新文件一侧的行号落在哪个模块级定义里
with open(DIFF, encoding="utf-8") as handle:
    diff_lines = handle.read().split("\n")
start = next(index for index, line in enumerate(diff_lines) if line == "+++ b/research/scripts/admission.py")
hunks, new_line = [], None
for line in diff_lines[start + 1:]:
    if line.startswith("diff --git") or line.startswith("```"):
        break
    header = re.match(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@", line)
    if header:
        new_line = int(header.group(1))
        hunks.append(set())
        continue
    if new_line is None:
        continue
    if line.startswith("-"):
        hunks[-1].add(new_line)
        continue
    if line.startswith("+"):
        hunks[-1].add(new_line)
    new_line += 1
spans = sorted((node.lineno, node.end_lineno, name) for name, node in definitions.items())
inside, outside = 0, 0
for touched in hunks:
    names = {name for first, last, name in spans for number in touched if first <= number <= last}
    hit = bool(names & wanted)
    inside, outside = inside + hit, outside + (not hit)
    print(f"HUNK 新文件第 {min(touched)}–{max(touched)} 行：{'闭包内' if hit else '闭包外'}（{'、'.join(sorted(names)) or '模块级文字 / 文件头'}）")
print(f"MEASURE 附录二 admission.py 的 {len(hunks)} 个 hunk：落在判法闭包里 {inside} 个，闭包外 {outside} 个")
sys.exit(cells.finish())
