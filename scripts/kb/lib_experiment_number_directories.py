"""实验号住在哪：读目录登记 .claude/gate.d/experiment-number-directories.tsv，列出一棵树里出现过的实验号。

给 scripts/kb/claim-experiment.sh（查号、--next）与门禁 doc-experiments 的 experiment-orphans 格（出现过的号在 kb 里有没有正文）共用：
两处读同一份登记、同一套名字形态，不各写一份目录表。登记的写法在那份登记的文件头。
没有自己的自证；两个调用方各带一组：claim-experiment.sh 自证里「目录登记」那几处判，门禁 doc-experiments 的 experiment-orphans-registry-red 样本。
弄坏开关：这份库没有；两个调用方各有「不读登记、退回改前写死的那组目录」的开关（CLAIM_EXPERIMENT_BREAK=registry-ignored、EXPERIMENT_PAGES_AND_PRODUCTS_BREAK=orphans-registry-ignored）。

用法：
  python3 scripts/kb/lib_experiment_number_directories.py numbers <登记> <树根>
    <登记> 写 - 时从标准输入读登记正文。逐行打，制表符分隔：
      entry  <路径>  <名字形态>        登记里的每一处，照登记的次序
      absent <路径>                    登记了、这棵树里不在的那一处
      number <号> <相对树根的路径>      每个命中（号去掉前导 0；index-rows 的路径写成 <文件>:<行号>），按号、路径排序
    退 0。登记写坏（列数不对、名字形态认不出、理由不到 4 个字、路径不是相对路径或带 ..、同一路径登两行、一行都没有、
    目录形态的登记指到文件或 index-rows 指到目录）或扫的时候读不了目录：标准错误打 ✗ 与 →，标准输出什么都不打，退 1。参数不对退 2。
"""
import os
import re
import sys

REGISTRY_RELATIVE_PATH = ".claude/gate.d/experiment-number-directories.tsv"
MINIMUM_REASON_CHARACTERS = 4
E_NUMBER_NAME = re.compile(r"^e([0-9]+)(?![0-9])")
NUMBER_NAME = re.compile(r"^([0-9]+)-")
INDEX_ROW = re.compile(r"^\| E([0-9]+)（")
DIRECTORY_FORMS = ("e-number-names", "number-names")
FORMS = DIRECTORY_FORMS + ("index-rows",)
PRUNED_DIRECTORY = "target"


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


def main(arguments):
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


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
