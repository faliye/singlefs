"""真仓扫描（只读）：改动范围里有多少路径被判成只差搬迁、剔掉了；被剔的 crates 与钩子、门禁脚本里，路径改在注释里还是改在代码里。
用法：python3 real_scan.py --repo <仓根>。仓是活的（别的会话还在改），数字只对这一次跑；产物一行 name=… 。"""
import argparse
import difflib
import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from common import load_library, say  # noqa: E402

COMMENT = re.compile(r"^\s*(//|///|//!|#|\*|/\*|<!--)")


def listed(repo, *arguments):
    done = subprocess.run(["git", "-c", "core.quotepath=false", *arguments], cwd=repo, capture_output=True, text=True, check=True)
    return [line for line in done.stdout.split("\n") if line]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    repo = os.path.abspath(parser.parse_args().repo)
    library, _path = load_library(repo)
    changed = sorted(set(listed(repo, "diff", "--name-only", "HEAD") + listed(repo, "diff", "--name-only", "--cached") + listed(repo, "ls-files", "--others", "--exclude-standard")))
    moved, problem = library.moved_only_paths("HEAD", changed, cwd=repo)
    say("scan-totals", changed=len(changed), dropped_as_moved_only=len(moved), kept=len(changed) - len(moved), problem=problem)
    kept_apparatus = [path for path in changed if path not in moved and path.startswith("scripts/experiment-apparatus/")]
    moves, _ = library.path_moves(repo)
    categories = {"only-sweep-forms": 0, "other": 0, "new-file": 0}
    examples = {"only-sweep-forms": [], "other": []}
    for path in kept_apparatus:
        before, taken, _problem = library.baseline_file_bytes("HEAD", path, cwd=repo)
        if before is None:
            categories["new-file"] += 1
            continue
        after = library.rewrite_moved_bytes(before, moves).decode("utf-8", "replace").split("\n")
        current = open(os.path.join(repo, path), encoding="utf-8", errors="replace").read().split("\n")
        residue = [line[2:] for line in difflib.ndiff(after, current) if line.startswith("+ ")]
        only_forms = bool(residue) and all(("experiment-apparatus" in line) for line in residue)
        key = "only-sweep-forms" if only_forms else "other"
        categories[key] += 1
        if len(examples[key]) < 4:
            examples[key].append((path, residue[:2]))
    say("scan-kept-under-apparatus", total=len(kept_apparatus), **categories)
    for key, items in examples.items():
        for path, residue in items:
            say(f"scan-example-{key}", path=path, residue=[line[:100] for line in residue])
    sensitive = [path for path in moved if re.match(r"^(crates/[^/]+/src/.*\.rs|\.claude/(hooks|gate\.d)/[^/]+|\.claude/(agents|rules)/.*|\.claude/(agent-common|main-agent)\.md|\.claude/settings\.json)$", path)]
    say("scan-dropped-sensitive", count=len(sensitive), paths=sensitive[:40])
    for path in sensitive:
        if not path.startswith("crates/"):
            continue
        before, _taken, _problem = library.baseline_file_bytes("HEAD", path, cwd=repo)
        now = open(os.path.join(repo, path), encoding="utf-8", errors="replace").read().split("\n")
        old = before.decode("utf-8", "replace").split("\n") if before else []
        lines = [line[2:] for line in difflib.ndiff(old, now) if line.startswith("+ ")]
        code_lines = [line for line in lines if not COMMENT.match(line)]
        say("scan-crates-dropped", path=path, added_lines=len(lines), non_comment_lines=len(code_lines), non_comment=[line.strip()[:110] for line in code_lines[:3]])


if __name__ == "__main__":
    main()
