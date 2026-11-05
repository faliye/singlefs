#!/usr/bin/env python3
# admission: always 判的是此刻树里的目录与登记，上一次的结论不替这一次作保
# run-condition: none 只读树里的目录名与登记文本；直接执行只列号或跑自证，除了 python3 没有环境要求
"""实验号住在哪：读目录登记 .claude/gate.d/experiment-number-directories.tsv，列出一棵树里出现过的实验号。

给 scripts/kb/claim-experiment.sh（查号、--next）与门禁 doc-experiments 的 experiment-orphans 格（出现过的号在 kb 里有没有正文）共用：
两处读同一份登记、同一套名字形态，不各写一份目录表。登记的写法在那份登记的文件头。

目录形态逐层往下时不进两类目录：名叫 target 的子目录；三方腿的模型目录——research/prompts/ 直接底下名字以 -model 结尾的目录
（<轮>-<模型>-model、<轮>-<模型>-s2-model，命名见 .claude/rules/three-way-inference.md）：里面是腿自己造的样本世界（worlds/ 底下整棵仓的样子，
攻方造的假 e990-preregistration.md、假 e990_judge_sample.rs 在里面），不是实验。模型目录自己的名字照常认（名字带号的那个号算用过），只是不往里走。
别处名叫 -model 的目录（research/prompts/ 更深一层的、research/results/ 底下的）照常往下走。

自证：python3 scripts/kb/lib_experiment_number_directories.py --selftest
  在临时目录里摆一棵树：腿模型目录里的假样本号不算、模型目录外同名的照算、模型目录自己的名字带号的照算，再走命令行入口核同一棵树，再核平常跑设开关被拒。

弄坏开关（只给 --selftest 证红用；直接执行而不是 --selftest 时设了它，参数解析之前就拒、退 2，点名它只在自证里认）。
逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ lib_experiment_number_directories 自证：<格名>：…」）：
  EXPERIMENT_NUMBER_DIRECTORIES_BREAK=model-directories-scanned  numbers_in_tree() 不跳过三方腿的模型目录，照旧逐层扫进去（改前的样子）
      「✗ lib_experiment_number_directories 自证：腿模型目录里的假样本号不算用过」
      「✗ lib_experiment_number_directories 自证：模型目录自己的名字带号时那个号照算、里面的不算」
      「✗ lib_experiment_number_directories 自证：命令行入口：腿模型目录里的号不列」
两个调用方各有「不读登记、退回改前写死的那组目录」的开关（CLAIM_EXPERIMENT_BREAK=registry-ignored、EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored）。

用法：
  python3 scripts/kb/lib_experiment_number_directories.py numbers <登记> <树根>
    <登记> 写 - 时从标准输入读登记正文。逐行打，制表符分隔：
      entry  <路径>  <名字形态>        登记里的每一处，照登记的次序
      absent <路径>                    登记了、这棵树里不在的那一处
      number <号> <相对树根的路径>      每个命中（号去掉前导 0；index-rows 的路径写成 <文件>:<行号>），按号、路径排序
    退 0。登记写坏（列数不对、名字形态认不出、理由不到 4 个字、路径不是相对路径或带 ..、同一路径登两行、一行都没有、
    目录形态的登记指到文件或 index-rows 指到目录）或扫的时候读不了目录：标准错误打 ✗ 与 →，标准输出什么都不打，退 1。参数不对退 2。
  python3 scripts/kb/lib_experiment_number_directories.py --selftest
"""
import contextlib
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile

REGISTRY_RELATIVE_PATH = ".claude/gate.d/experiment-number-directories.tsv"
MINIMUM_REASON_CHARACTERS = 4
E_NUMBER_NAME = re.compile(r"^e([0-9]+)(?![0-9])")
NUMBER_NAME = re.compile(r"^([0-9]+)-")
INDEX_ROW = re.compile(r"^\| E([0-9]+)（")
DIRECTORY_FORMS = ("e-number-names", "number-names")
FORMS = DIRECTORY_FORMS + ("index-rows",)
PRUNED_DIRECTORY = "target"
# 三方腿的模型目录：这个目录直接底下名字以 MODEL_DIRECTORY_SUFFIX 结尾的子目录，只认名字、不往里走
MODEL_DIRECTORY_PARENT = "research/prompts"
MODEL_DIRECTORY_SUFFIX = "-model"
BREAK_VARIABLE = "EXPERIMENT_NUMBER_DIRECTORIES_BREAK"
KNOWN_BREAKS = ("model-directories-scanned",)


def break_mode():
    """此刻设的弄坏开关值（每次现读）；认不出的值抛 ValueError。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAKS:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 认不出（只认 {'、'.join(KNOWN_BREAKS)}）")
    return mode


def read_registry(text, source):
    """→ (entries, problems)：entries 是 [(路径, 名字形态)]，照登记的次序；problems 是写坏的那几行的说明。"""
    entries, problems, seen = [], [], {}
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        columns = line.split("\t")
        if len(columns) != 3:
            problems.append(f"{source} 第 {number} 行：不是「<路径><TAB><名字形态><TAB># 为什么>」三列：{line[:90]}")
            continue
        path, form, reason = (column.strip() for column in columns)
        path = path.rstrip("/")
        if not reason.startswith("#") or len(reason.lstrip("#").strip()) < MINIMUM_REASON_CHARACTERS:
            problems.append(f"{source} 第 {number} 行：第三列要写「# 为什么」，理由至少 {MINIMUM_REASON_CHARACTERS} 个字：{reason[:60]}")
            continue
        if form not in FORMS:
            problems.append(f"{source} 第 {number} 行：名字形态「{form}」认不出，只认 {'、'.join(FORMS)}")
            continue
        if not path or os.path.isabs(path) or ".." in path.split("/"):
            problems.append(f"{source} 第 {number} 行：路径要写相对仓根的路径、不带 ..：{path or '（空）'}")
            continue
        if path in seen:
            problems.append(f"{source} 第 {number} 行：{path} 在第 {seen[path]} 行已经登过")
            continue
        seen[path] = number
        entries.append((path, form))
    if not entries and not problems:
        problems.append(f"{source} 一行登记都没有：不知道实验号住在哪几处")
    return entries, problems


def numbers_in_tree(root, entries):
    """→ (hits, absent, problems)：hits 是 [(号, 相对树根的路径)]，absent 是登记了而不在的路径。"""
    hits, absent, problems = [], [], []
    scan_model_directories = break_mode() == "model-directories-scanned"

    def unreadable(error):
        problems.append(f"读不了 {error.filename}：{error.strerror}")

    for path, form in entries:
        full = os.path.join(root, path)
        if not os.path.lexists(full):
            absent.append(path)
            continue
        if form in DIRECTORY_FORMS:
            if not os.path.isdir(full):
                problems.append(f"{path} 登记成 {form}（目录形态），树里却不是目录")
                continue
            pattern = E_NUMBER_NAME if form == "e-number-names" else NUMBER_NAME
            for base, directories, files in os.walk(full, onerror=unreadable):
                directories[:] = sorted(name for name in directories if name != PRUNED_DIRECTORY)
                names = directories + files if form == "e-number-names" else files
                for name in names:
                    found = pattern.match(name)
                    if found:
                        hits.append((int(found.group(1)), os.path.relpath(os.path.join(base, name), root)))
                if not scan_model_directories and os.path.relpath(base, root) == MODEL_DIRECTORY_PARENT:
                    directories[:] = [name for name in directories if not name.endswith(MODEL_DIRECTORY_SUFFIX)]
            continue
        if not os.path.isfile(full):
            problems.append(f"{path} 登记成 index-rows（文件形态），树里却不是文件")
            continue
        try:
            with open(full, encoding="utf-8", errors="replace") as handle:
                for line_number, line in enumerate(handle, 1):
                    found = INDEX_ROW.match(line)
                    if found:
                        hits.append((int(found.group(1)), f"{path}:{line_number}"))
        except OSError as error:
            unreadable(error)
    return sorted(set(hits)), absent, problems


def refuse_break_outside_selftest(arguments):
    """直接执行时、参数解析之前调：不是 --selftest 而设了弄坏开关就打拒绝、交 2；没设或是 --selftest 交 None。"""
    mode = os.environ.get(BREAK_VARIABLE, "")
    if not mode or arguments == ["--selftest"]:
        return None
    print(f"  ✗ {BREAK_VARIABLE}={mode} 只在自证里认（只给 --selftest 证红用），这一趟不是 --selftest，不往下走：设着它会少查实验号", file=sys.stderr)
    print(f"    → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/kb/lib_experiment_number_directories.py --selftest", file=sys.stderr)
    return 2


def run_numbers(arguments):
    if len(arguments) != 3 or arguments[0] != "numbers":
        print("  ✗ 用法：python3 scripts/kb/lib_experiment_number_directories.py numbers <登记或 -> <树根>", file=sys.stderr)
        print("    → 登记写仓里那份 .claude/gate.d/experiment-number-directories.tsv，或写 - 从标准输入读", file=sys.stderr)
        return 2
    _, registry, root = arguments
    if registry == "-":
        text, source = sys.stdin.read(), "标准输入里的登记"
    else:
        try:
            with open(registry, encoding="utf-8") as handle:
                text = handle.read()
        except OSError as error:
            print(f"  ✗ 读不了目录登记 {registry}：{error.strerror}；不知道实验号住在哪几处，查不全", file=sys.stderr)
            print(f"    → 从 git 找回这份登记（git log --diff-filter=D --name-only -- {REGISTRY_RELATIVE_PATH}），写法在它的文件头", file=sys.stderr)
            return 1
        source = registry
    entries, problems = read_registry(text, source)
    if not problems:
        if not os.path.isdir(root):
            problems.append(f"树根 {root} 不是目录")
        else:
            hits, absent, problems = numbers_in_tree(root, entries)
    if problems:
        for problem in problems:
            print(f"  ✗ {problem}", file=sys.stderr)  # gate-lint:detail
        print(f"  ✗ 实验号住在哪：{len(problems)} 处不对，号没查全", file=sys.stderr)  # gate-lint:summary
        print(f"    → 照 {REGISTRY_RELATIVE_PATH} 文件头的写法改登记；读不了的目录先修权限。查不全的时候不给号、不判孤儿", file=sys.stderr)
        return 1
    for path, form in entries:
        print(f"entry\t{path}\t{form}")
    for path in absent:
        print(f"absent\t{path}")
    for number, path in hits:
        print(f"number\t{number}\t{path}")
    return 0


def build_selftest_tree(work):
    """在 work 里摆一棵树，返回登记文件路径。号与它们该不该算：
    990 991  腿模型目录（-model、-s2-model）里的假样本，不算
    992      research/prompts/ 底下模型目录之外的，算
    993      research/results/ 底下名叫 -model 的目录里的，算（不是 research/prompts/ 直接底下的）
    994      research/prompts/ 再深一层的 -model 目录里的，算（同上）
    995      模型目录自己的名字带号，算；996 在那个目录里面，不算"""
    files = (
        "research/prompts/d137-sample-r1-opus-model/worlds/w1/research/prompts/e990-preregistration.md",
        "research/prompts/d137-sample-r1-opus-s2-model/worlds/w1/crates/singlefs-verification/src/bin/e991_judge_sample.rs",
        "research/prompts/e992-preregistration.md",
        "research/results/leg-model/e993-sample.out",
        "research/prompts/nested/leg-model/e994-sample.md",
        "research/prompts/e995-sample-r1-opus-model/e996-inside.md",
    )
    for relative in files:
        full = os.path.join(work, relative)
        os.makedirs(os.path.dirname(full), exist_ok=True)
        with open(full, "w", encoding="utf-8") as handle:
            handle.write("sample\n")
    registry = os.path.join(work, "registry.tsv")
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write("research\te-number-names\t# 样本：跑前登记与实验产物\n")
    return registry


def run_selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="experiment-number-directories-selftest-")
    try:
        registry = build_selftest_tree(work)
        with open(registry, encoding="utf-8") as handle:
            entries, problems = read_registry(handle.read(), registry)
        hits, _, scan_problems = numbers_in_tree(work, entries)
        found = {number for number, _ in hits}
        checked += 1
        if problems or scan_problems:
            failures.append(f"样本树读不了：{problems + scan_problems}")
        checked += 1
        if found & {990, 991}:
            failures.append(f"腿模型目录里的假样本号不算用过：E990、E991 在 research/prompts/ 直接底下名字以 -model 结尾的目录里，却被列成用过的号（列出的号 {sorted(found)}）")
        checked += 1
        missing = {992, 993, 994} - found
        if missing:
            failures.append(f"模型目录外同名的照算：E992（research/prompts/ 底下）、E993（research/results/ 底下的 -model 目录）、E994（research/prompts/ 更深一层的 -model 目录）"
                            f"该列没列：缺 {sorted(missing)}（列出的号 {sorted(found)}）")
        checked += 1
        if 995 not in found or 996 in found:
            failures.append(f"模型目录自己的名字带号时那个号照算、里面的不算：E995 该列、E996 不该列（列出的号 {sorted(found)}）")
        # 命令行入口走同一棵树
        checked += 1
        captured, error_text = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(captured), contextlib.redirect_stderr(error_text):
            code = run_numbers(["numbers", registry, work])
        listed = {int(row.split("\t")[1]) for row in captured.getvalue().splitlines() if row.startswith("number\t")}
        if code != 0 or listed & {990, 991} or not {992, 993, 994, 995} <= listed:
            failures.append(f"命令行入口：腿模型目录里的号不列、模型目录外的列：应当退 0、不含 990 991、含 992 到 995，实际退 {code}、列出 {sorted(listed)}、标准错误 {error_text.getvalue().strip()[-120:]!r}")
        # 平常跑设开关要被拒：起自己一次（不是 --selftest），设上开关要退 2、点名它只在自证里认；不设就不拒（照常报用法错）
        checked += 1
        probes = {}
        for label, value in (("设上", "model-directories-scanned"), ("不设", None)):
            probe_environment = {name: text for name, text in os.environ.items() if name != BREAK_VARIABLE}
            if value is not None:
                probe_environment[BREAK_VARIABLE] = value
            done = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--break-guard-probe-unknown-option"], env=probe_environment,
                                  stdin=subprocess.DEVNULL, capture_output=True, text=True, errors="replace", timeout=60)
            named = [row for row in (done.stdout + done.stderr).splitlines() if BREAK_VARIABLE in row and "自证" in row]
            probes[label] = (done.returncode, named)
        if probes["设上"][0] != 2 or not probes["设上"][1] or probes["不设"][1]:
            failures.append(f"平常跑设开关要被拒：设上 {BREAK_VARIABLE} 应当退 2 并点名它只在自证里认、不设不点名，"
                            f"实际设上退 {probes['设上'][0]}、点名 {len(probes['设上'][1])} 行，不设点名 {len(probes['不设'][1])} 行")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ lib_experiment_number_directories 自证：{failure}")  # gate-lint:detail
    if failures:
        print("    → 怎么办：看 numbers_in_tree() 逐层往下时跳模型目录的那一句与 refuse_break_outside_selftest()；"
              f"{BREAK_VARIABLE}=model-directories-scanned 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_experiment_number_directories 自证通过（查了 {checked} 项）：腿模型目录里的假样本号不算用过，模型目录外同名的照算，"
          "模型目录自己的名字带号的照算，命令行入口列同一批号，平常跑设开关被拒、退 2")
    return 0


def main(arguments):
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    if arguments == ["--selftest"]:
        return run_selftest()
    return run_numbers(arguments)


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
