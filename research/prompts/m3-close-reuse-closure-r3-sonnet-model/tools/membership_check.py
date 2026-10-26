#!/usr/bin/env python3
"""membership_check.py <复跑 out 目录> <入口全名>…：几棵树的闭包集合（scan-*.sets）里，某些单元在不在：
memory_peaks_lib.py 的 import subprocess、lib_long_run.py 的单元数与其中 import 数、host-dispatch.py 的 module-body、host-resources.py 的单元数。"""
import os, sys
out = sys.argv[1]; entries = sys.argv[2:]
trees = ["current", "cut-src", "cut-algo", "narrow", "modlevel", "cut-modlevel"]
def load(tree):
    rows = {}
    for line in open(os.path.join(out, f"scan-{tree}.tsv"), encoding="utf-8"):
        f = line.rstrip("\n").split("\t"); rows[f[1]] = os.path.join(out, f"scan-{tree}.sets", f"{int(f[0]):02d}.set")
    return rows
for entry in entries:
    for tree in trees:
        members = open(load(tree)[entry], encoding="utf-8").read().splitlines()
        mp = sum(1 for m in members if m.startswith("import|subprocess|scripts/hosts/memory_peaks_lib.py|"))
        lr = [m for m in members if m.split("|")[2] == ".claude/hooks/lib_long_run.py"]
        lri = sum(1 for m in lr if m.startswith("import|"))
        hd = sum(1 for m in members if m.startswith("module-body|<module-body>|scripts/hosts/host-dispatch.py|"))
        hr = sum(1 for m in members if m.split("|")[2] == "scripts/hosts/host-resources.py")
        print(f"name=membership entry={entry.split('::')[1]} tree={tree} memory_peaks_lib_import_subprocess={mp} lib_long_run_units={len(lr)} lib_long_run_imports={lri} host_dispatch_module_body={hd} host_resources_units={hr}")
