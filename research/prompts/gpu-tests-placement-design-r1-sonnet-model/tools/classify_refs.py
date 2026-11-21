#!/usr/bin/env python3
"""读 `git grep -c` 的「路径:命中数」行（标准输入），按目录分类计文件数与命中数。"""
import collections
import sys

rows = [line.rstrip("\n").rsplit(":", 1) for line in sys.stdin if line.strip()]
classes = collections.defaultdict(lambda: [0, 0])


def class_of(path: str) -> str:
    if path.startswith("research/prompts/"):
        return "research/prompts"
    if path.startswith("research/results/"):
        return "research/results"
    if path.startswith("records/"):
        return "records"
    if path.startswith(".claude/gate.d/fixtures/"):
        return ".claude/gate.d/fixtures (样本档)"
    if path.startswith(".claude/gate.d/"):
        return ".claude/gate.d (门禁常量与登记表)"
    if path.startswith(".claude/kb/"):
        return ".claude/kb"
    if path.startswith(".claude/"):
        return ".claude/其余"
    if path.startswith("crates/singlefs-background-gpu-tests/"):
        return "crates/测试包自己"
    if path == "crates/mutations.tsv":
        return "crates/mutations.tsv"
    if path.startswith("crates/"):
        return "crates/别的包"
    if path.startswith("scripts/"):
        return "scripts"
    return "仓根与别处"


total_files = total_hits = 0
for path, hits in rows:
    key = class_of(path)
    classes[key][0] += 1
    classes[key][1] += int(hits)
    total_files += 1
    total_hits += int(hits)
for key, (files, hits) in sorted(classes.items(), key=lambda item: -item[1][1]):
    print(f"class={key} files={files} hits={hits}")
print(f"name=refs_total files={total_files} hits={total_hits}")
