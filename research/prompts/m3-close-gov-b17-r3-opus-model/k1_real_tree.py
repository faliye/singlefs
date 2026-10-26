#!/usr/bin/env python3
"""K1 第二条路：在真 crates/ 源码上（只读），用一份脚本副本（--scripts）里的 lib_mutation_units.UnitKeys，
给 crates/ 下每个 fn 单元造一条「执行过」的函数（区域起在 fn 那个词所在的行），交给真 executed_units 落单元；
逐个打「<文件>:<fn 行> <函数单元> -> <落进的单元>」与每个单元的（种类、限定名、指纹）。不起 cargo、不编译。"""
import argparse, importlib.util, os, sys

parser = argparse.ArgumentParser()
parser.add_argument("--scripts", required=True)
parser.add_argument("--root", required=True)
parser.add_argument("--out", required=True)
arguments = parser.parse_args()
spec = importlib.util.spec_from_file_location("lmu_k1", os.path.join(arguments.scripts, "scripts", "mutation", "lib_mutation_units.py"))
lmu = importlib.util.module_from_spec(spec); sys.modules[spec.name] = lmu; spec.loader.exec_module(lmu)
keys = lmu.UnitKeys(arguments.root)
root = os.path.abspath(arguments.root)
functions, identities, fingerprints = [], [], []
for directory, names, files in os.walk(os.path.join(root, "crates")):
    names[:] = sorted(name for name in names if name != "target")
    for name in sorted(files):
        if not name.endswith(".rs"):
            continue
        path = os.path.relpath(os.path.join(directory, name), root)
        rust_file = keys.rust_file(path)
        for unit in rust_file.units:
            fingerprints.append(f"{path}\t{unit.kind}\t{unit.name}\t{unit.fingerprint}")
            if unit.kind != "fn":
                continue
            line = next((rust_file.lines[index] for index in range(unit.first_token, unit.last_token) if rust_file.texts[index] == "fn"), None)
            if line is None:
                continue
            functions.append({"count": 1, "filenames": [os.path.join(root, path)], "regions": [[line, 1, line, 2, 1, 0, 0, 0]]})
            identities.append(f"{path}:{line}\t{unit.name}")
lines = []
for function, identity in zip(functions, identities):
    placed, unplaced = keys.executed_units({"data": [{"functions": [function]}]}, root)
    target = ",".join(f"{reference.kind}:{reference.name}" for reference in placed) or "-"
    lines.append(f"{identity}\t->\t{target}")
with open(arguments.out + ".placement", "w", encoding="utf-8") as handle:
    handle.write("".join(line + "\n" for line in lines))
with open(arguments.out + ".fingerprints", "w", encoding="utf-8") as handle:
    handle.write("".join(line + "\n" for line in sorted(fingerprints)))
wrong = sum(1 for line in lines if line.split("\t")[1] != line.split("\t")[3].split(":", 1)[-1])
print(f"name=real_tree scripts={os.path.basename(arguments.scripts)} fn_units={len(lines)} units={len(fingerprints)} placed_elsewhere={wrong}")
