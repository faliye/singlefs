#!/usr/bin/env python3
"""A1 被验的身份「<代码文件名>::<验证目标名>::…」在今天的仓上撞不撞（m3-prune-gpu-r3 云端攻方腿的原型，不入库）。
「代码文件名」取文件名本身（不带目录）与取仓内路径各算一遍；「验证目标名」取 #[test] 标的函数名（不带模块路径）。
另数：同一份源文件被几个测试目标用 mod / #[path] / include! 带进去（流程写在共用文件里时，身份里的「代码文件名」是哪一份就不唯一）。
用法：a1_scan.py <仓根>
"""
import collections
import pathlib
import re
import sys

TEST_FN = re.compile(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)")


def main():
    root = pathlib.Path(sys.argv[1])
    by_path = collections.Counter()
    by_basename = collections.defaultdict(set)
    files_by_basename = collections.defaultdict(set)
    for path in sorted(root.glob("crates/*/**/*.rs")):
        relative = str(path.relative_to(root))
        if "/target/" in relative:
            continue
        for name in TEST_FN.findall(path.read_text()):
            by_path[(relative, name)] += 1
            by_basename[(path.name, name)].add(relative)
            files_by_basename[path.name].add(relative)
    duplicate_within_file = sorted(f"{relative}::{name}x{count}" for (relative, name), count in by_path.items() if count > 1)
    basename_collisions = sorted(f"{basename}::{name}<-{sorted(paths)}" for (basename, name), paths in by_basename.items() if len(paths) > 1)
    basenames_in_several_files = sorted(f"{basename}<-{len(paths)}" for basename, paths in files_by_basename.items() if len(paths) > 1)
    targets = sorted(root.glob("crates/*/tests/*.rs")) + sorted(root.glob("crates/*/src/bin/*.rs"))
    shared = collections.Counter()
    for target in targets:
        text = target.read_text()
        for included in re.findall(r'#\[path\s*=\s*"([^"]+)"\]', text) + re.findall(r'include!\("([^"]+)"\)', text):
            shared[str((target.parent / included).resolve().relative_to(root.resolve()))] += 1
        if re.search(r"^\s*mod\s+common\s*;", text, re.M):
            shared[str((target.parent / "common/mod.rs").relative_to(root))] += 1
    shared_files = sorted(f"{name}<-{count}" for name, count in shared.items() if count > 1)
    print(f"E7RESULT name=r3_a1_scan test_functions={sum(by_path.values())} files_with_tests={len({relative for relative, _ in by_path})} "
          f"same_file_same_target_name={len(duplicate_within_file)} detail={duplicate_within_file} "
          f"same_basename_same_target_name_in_different_files={len(basename_collisions)} detail={basename_collisions} "
          f"basenames_carried_by_several_files={basenames_in_several_files} "
          f"source_files_compiled_into_several_targets={shared_files} "
          f"must_be_nonzero={len(duplicate_within_file) + len(basename_collisions)}")


if __name__ == "__main__":
    main()
