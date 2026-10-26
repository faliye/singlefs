#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H3：crates 变异判法的六个入口各自的闭包里，出自 lib_function_units.py 与 lib_verdict_library.py 的单元数（只算 python 闭包）。
用法：python3 crates_entries_by_file.py --root <根>"""
import argparse, ast, importlib.util, os
parser = argparse.ArgumentParser(); parser.add_argument("--root", required=True)
root = os.path.realpath(parser.parse_args().root)
specification = importlib.util.spec_from_file_location("lfu_probe", os.path.join(root, "scripts/gate/lib_function_units.py"))
lfu = importlib.util.module_from_spec(specification); specification.loader.exec_module(lfu)
tree = ast.parse(open(os.path.join(root, "scripts/mutation/crates-mutation-rows.py"), encoding="utf-8").read())
entries = next(ast.literal_eval(node.value) for node in tree.body if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", "") == "JUDGING_ENTRIES")
for entry in entries:
    closure = lfu.closure_of(root, entry)
    files = [unit.file for unit in closure.order]
    print(f"entry={entry} units={len(files)} lib_function_units={files.count('scripts/gate/lib_function_units.py')} "
          f"lib_verdict_library={files.count('scripts/crash/lib_verdict_library.py')} refusals={len(closure.refusals)}")
