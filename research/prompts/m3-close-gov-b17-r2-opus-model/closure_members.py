#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2：列 broad-mutation 判法闭包里点名文件的单元名（只算 python 闭包）。
用法：python3 closure_members.py --root <根> --file <相对路径> [--names 名,名]"""
import argparse, importlib.util, os
parser = argparse.ArgumentParser()
parser.add_argument("--root", required=True); parser.add_argument("--file", required=True); parser.add_argument("--names", default="")
arguments = parser.parse_args()
root = os.path.realpath(arguments.root)
def load(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module
driver = load("broad_mutation_probe", os.path.join(root, "scripts/mutation/broad-mutation.py"))
lfu = driver.prune_rules().units_library()
options = lfu.Options()
options.under = [place for place in driver.JUDGING_SCAN_FILES if os.path.isfile(os.path.join(root, place))]
wanted = set(filter(None, arguments.names.split(",")))
for label, entries in (("broad_cpu", driver.JUDGING_ENTRIES), ("broad_gpu", driver.GPU_JUDGING_ENTRIES)):
    names = set()
    for path, name in entries:
        closure = lfu.closure_of(root, f"{path}::{name}", options)
        names |= {unit.name for unit in closure.order if unit.file == arguments.file}
    hits = sorted(names & wanted) if wanted else sorted(names)[:12]
    print(f"name={label}_members file={arguments.file} total={len(names)} {'asked' if wanted else 'first12'}={','.join(hits)}")
