#!/usr/bin/env python3
"""只读：照门禁第一格的算法（global_key_lines，工具版本 tool_versions(llvm_bin=None)，限时取那一版记的那一行）在真库此刻那一版的清单上现算全局键，
与那一版 global-key.txt、结论表里记的 global_key 比。不写库目录。用法：real_key.py <驱动> <仓根> <库目录>"""
import hashlib, importlib.util, json, os, subprocess, sys, types
driver_path, repository, home = sys.argv[1:4]
spec = importlib.util.spec_from_file_location("bm", driver_path); bm = importlib.util.module_from_spec(spec); spec.loader.exec_module(bm)
bm.REPOSITORY = repository
source = json.load(open(os.path.join(home, "tree.source")))
version = os.path.join(home, "versions", source["fingerprint"])
manifest = bm.read_manifest(os.path.join(version, "manifest.tsv"))
toolchain = subprocess.run(["rustc", "-Vv"], capture_output=True, text=True, cwd=os.path.join(home, "tree")).stdout.strip() + "\n"
tools = bm.tool_versions(types.SimpleNamespace(llvm_bin=None))
recorded = open(os.path.join(version, "global-key.txt"), encoding="utf-8").read()
limits = next(line for line in recorded.splitlines() if line.startswith("limits "))
current = bm.global_key_lines(bm.JUDGE_RULES_VERSION, toolchain, manifest, os.environ, tools, bm.exclusions_digest_of(bm.EXCLUSIONS_FILE), limits)
conclusion_keys = sorted({line.split("\t")[6] for line in open(os.path.join(home, "conclusions.tsv"), encoding="utf-8").read().splitlines()[1:] if line})
print(f"tree.source fingerprint={source['fingerprint'][:16]} global_key_in_pointer={source.get('global_key')}")
print(f"recorded global-key.txt sha256={hashlib.sha256(recorded.encode()).hexdigest()[:16]} lines={len(recorded.splitlines())}")
print(f"current (gate algorithm) sha256={hashlib.sha256(current.encode()).hexdigest()[:16]} lines={len(current.splitlines())}")
print(f"conclusions.tsv global_key values={[key[:16] for key in conclusion_keys]}")
print(f"byte-identical={current == recorded}")
for line in recorded.splitlines():
    if line not in current.splitlines():
        print(f"only-recorded\t{line}")
for line in current.splitlines():
    if line not in recorded.splitlines():
        print(f"only-current\t{line}")
