#!/usr/bin/env python3
"""攻方腿 m3-close-gov-b17-r1 G2：完工清理闸认副本的那一步（scripts/agents/agent-watch.py 的 scratch_hog_kind / find_scratch_hogs，
completion-cleanup-guard.sh 文件头「仓副本」那一行说从它导入）认不认判决复跑留下的东西。
造的目录照 research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh 的形状：<work>/judgement-rerun/repo 里有 Cargo.toml、Cargo.lock、crates/，
cargo 编过会有 repo/target（带 CACHEDIR.TAG）。只建空目录与两三个小文件，不拷仓、不编。
用法：python3 probe_cleanup_kind.py --repo <仓根> --work <新的草稿目录>"""
import argparse, importlib.util, os

parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
repo = os.path.realpath(arguments.repo)
work = os.path.realpath(arguments.work)
spec = importlib.util.spec_from_file_location("agent_watch", os.path.join(repo, "scripts", "agents", "agent-watch.py"))
agent_watch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent_watch)

copy = os.path.join(work, "judgement-rerun", "repo")
os.makedirs(os.path.join(copy, "crates", "singlefs-harness"), exist_ok=True)
for name in ("Cargo.toml", "Cargo.lock"):
    open(os.path.join(copy, name), "w").close()
target = os.path.join(copy, "target")
os.makedirs(target, exist_ok=True)
with open(os.path.join(target, "CACHEDIR.TAG"), "wb") as handle:
    handle.write(agent_watch.CACHEDIR_TAG_SIGNATURE + b"\n")
model_copy = os.path.join(work, "judgement-rerun", "model-copy")
os.makedirs(os.path.join(model_copy, "out"), exist_ok=True)
open(os.path.join(model_copy, "rows.tsv"), "w").close()

for label, path in (("repo-copy", copy), ("its-target", target), ("model-copy-only", model_copy)):
    print(f"kind {label} = {agent_watch.scratch_hog_kind(path, os.listdir(path))}")
found, visited, truncated = agent_watch.find_scratch_hogs([os.path.join(work, "judgement-rerun")])
for path, kind in sorted(found):
    print(f"found {os.path.relpath(path, work)} {kind}")
print(f"visited={visited} truncated={truncated}")
