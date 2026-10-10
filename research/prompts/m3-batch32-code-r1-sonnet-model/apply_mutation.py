#!/usr/bin/env python3
"""变异表一行的小工具（正推腿 Sonnet，m3-batch32-code-r1）。只读变异表、只改 <工作目录>/tree 里那份拷贝。

  apply   <tree> <行号>    把那一行的「原文」在「文件」里换成「替换文」（原文必须恰好命中一次）；原文件另存 .mut-backup-<行号>
  restore <tree> <行号>    从备份还原，还原后逐字节核与备份相同
  args    <tree> <行号>    打印那一行的 cargo test 参数（第 5 段）
  named   <tree> <行号>    打印那一行点名的测试名（第 6 段）
  anchors <tree> <首行> <末行>   每行一条 ANCHOR：锚点命中次数、点名的测试是不是 #[test] fn、参数里的子串各命中几个测试名、点名的测试在不在命中的里面
"""
import re
import sys
from pathlib import Path


def read_row(tree: Path, line: int):
    import os
    table = Path(os.environ["SONNET_MUTATION_TABLE"]) if os.environ.get("SONNET_MUTATION_TABLE") else tree / "crates" / "mutations.tsv"
    rows = table.read_text(encoding="utf-8").split("\n")
    fields = rows[line - 1].split("\t")
    assert len(fields) == 6, f"第 {line} 行不是六段：{len(fields)}"
    name, file, original, replacement, args, named = fields
    unescape = lambda text: text.replace("\\n", "\n")
    return name, file, unescape(original), unescape(replacement), args, named


def tests_of(source: str):
    return re.findall(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([A-Za-z0-9_]+)", source)


def main():
    command, tree = sys.argv[1], Path(sys.argv[2])
    if command == "anchors":
        first, last = int(sys.argv[3]), int(sys.argv[4])
        sources = {}
        for line in range(first, last + 1):
            name, file, original, replacement, args, named = read_row(tree, line)
            text = (tree / file).read_text(encoding="utf-8")
            hits = text.count(original)
            filters = []
            if " -- " in args:
                tail = args.split(" -- ", 1)[1].split()
                filters = [piece for piece in tail if not piece.startswith("--")]
            test_file = None
            package_args = args.split(" -- ")[0].split()
            if "--test" in package_args:
                test_name = package_args[package_args.index("--test") + 1]
                test_file = tree / "crates" / ("singlefs-harness" if "singlefs-harness" in package_args else "singlefs-checker-tier") / "tests" / f"{test_name}.rs"
            tests = tests_of(test_file.read_text(encoding="utf-8")) if test_file else []
            if "--exact" in args:
                matched = [t for t in tests if t == named]
                per_filter = "exact"
            else:
                matched_sets = {f: [t for t in tests if f in t] for f in filters}
                matched = sorted({t for found in matched_sets.values() for t in found})
                per_filter = ",".join(f"{len(found)}" for found in matched_sets.values())
            print(
                f"ANCHOR line={line} file={file} anchor_hits={hits} named_is_a_test={'yes' if named in tests else 'no'} "
                f"named_selected={'yes' if named in matched else 'no'} tests_selected={len(matched)} per_filter={per_filter}"
            )
        return
    line = int(sys.argv[3])
    name, file, original, replacement, args, named = read_row(tree, line)
    path = tree / file
    backup = tree / f".mut-backup-{line}"
    if command == "args":
        print(args)
    elif command == "named":
        print(named)
    elif command == "apply":
        text = path.read_text(encoding="utf-8")
        assert text.count(original) == 1, f"第 {line} 行原文在 {file} 里命中 {text.count(original)} 次"
        backup.write_bytes(path.read_bytes())
        path.write_text(text.replace(original, replacement, 1), encoding="utf-8")
        print(f"applied line {line}")
    elif command == "restore":
        path.write_bytes(backup.read_bytes())
        assert path.read_bytes() == backup.read_bytes()
        backup.unlink()
        print(f"restored line {line}")
    else:
        sys.exit(2)


main()
