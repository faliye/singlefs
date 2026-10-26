#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r2 H1/H3：判法单元闭包按文件数单元（只算 python 闭包，不起 cargo）。
用法：python3 closure_by_file.py --root <仓副本根>
打：broad-mutation 的 JUDGING_ENTRIES 与 GPU_JUDGING_ENTRIES 两个闭包各按文件数单元，与照驱动写法算出的两把键；
crates-mutation-rows.py judging-units 照常与开关 judging-files-included 下的键、按文件数单元。"""
import argparse, collections, hashlib, importlib.util, os, subprocess, sys

parser = argparse.ArgumentParser()
parser.add_argument("--root", required=True)
root = os.path.realpath(parser.parse_args().root)

def load(name, path):
    specification = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module

lfu = load("lfu_probe", os.path.join(root, "scripts/gate/lib_function_units.py"))
driver_text = open(os.path.join(root, "scripts/mutation/broad-mutation.py"), encoding="utf-8").read()
namespace = {}
for name in ("JUDGING_ENTRIES", "JUDGING_SCAN_FILES", "GPU_JUDGING_ENTRIES"):
    start = driver_text.index(f"\n{name} = (") + 1
    end = driver_text.index(")\n", driver_text.index("(", start)) + 1
    while driver_text[start:end].count("(") != driver_text[start:end].count(")"):
        end = driver_text.index(")\n", end) + 1
    exec(driver_text[start:end], namespace)

def broad_key(entries):
    options = lfu.Options()
    options.under = [place for place in namespace["JUDGING_SCAN_FILES"] if os.path.isfile(os.path.join(root, place))]
    lines, distinct = [], set()
    for path, name in entries:
        closure = lfu.closure_of(root, f"{path}::{name}", options)
        for unit in closure.order:
            distinct.add((unit.file, unit.kind, unit.name, unit.first_line))
        key, refusals = lfu.closure_key(closure, allow_refused=True)
        lines.append(f"{path}::{name}\t{key}\t{','.join(sorted({refusal.form for refusal in refusals}))}")
    return hashlib.sha256("".join(line + "\n" for line in lines).encode()).hexdigest(), collections.Counter(unit[0] for unit in distinct)

driver = load("broad_mutation_probe", os.path.join(root, "scripts/mutation/broad-mutation.py"))
print(f"name=driver_keys judging_units_key={driver.judging_units_key(root)} gpu_judging_key={driver.gpu_judging_key(root)}")
for label, entries in (("broad_cpu", namespace["JUDGING_ENTRIES"]), ("broad_gpu", namespace["GPU_JUDGING_ENTRIES"])):
    key, by_file = broad_key(entries)
    print(f"name={label}_key key={key} entries={len(entries)}")
    for path, count in sorted(by_file.items(), key=lambda pair: (-pair[1], pair[0])):
        print(f"name={label}_units file={path} distinct_units={count}")

for label, extra in (("crates_default", {}), ("crates_judging_files_included", {"CRATES_MUTATION_ROWS_BREAK": "judging-files-included"})):
    environment = {key: value for key, value in os.environ.items() if not key.endswith("_BREAK")}
    environment.update(extra)
    done = subprocess.run([sys.executable, os.path.join(root, "scripts/mutation/crates-mutation-rows.py"), "judging-units", root, "--units"],
                          capture_output=True, text=True, env=environment)
    lines = done.stdout.splitlines()
    print(f"name={label} exit={done.returncode} head={lines[0] if lines else ''}")
    counts = collections.Counter(line.split("\t")[2].rsplit(":", 1)[0] for line in lines[1:] if line.count("\t") >= 2)
    for path, count in sorted(counts.items(), key=lambda pair: (-pair[1], pair[0])):
        print(f"name={label}_units file={path} units={count}")
    if done.returncode:
        print(f"name={label}_stderr {done.stderr.strip()[-400:]}")
