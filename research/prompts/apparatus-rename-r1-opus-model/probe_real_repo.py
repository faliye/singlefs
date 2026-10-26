#!/usr/bin/env python3
"""只读探针：在主工作区（--repo）上照 changed-paths.sh 的取法列改动范围，调冻结树里的 moved_only_paths，
再在「只差搬迁」被剔掉的文件里找换了新前缀、而那一处是指 git 历史的写法（git log/diff/blame/rev-list 的 -- 路径、<非十六进制引用>:路径）。"""
import importlib.util, os, re, subprocess, sys
sys.dont_write_bytecode = True
tree, repo, base = sys.argv[1], sys.argv[2], sys.argv[3]
spec = importlib.util.spec_from_file_location("lag", os.path.join(tree, ".claude/gate.d/lib-lag-baseline.py"))
lib = importlib.util.module_from_spec(spec); spec.loader.exec_module(lib)
def git(*a):
    return subprocess.run(["git", "-c", "core.quotepath=false", *a], cwd=repo, capture_output=True, text=True, check=True).stdout
changed = sorted(set(git("diff", "--name-only", base, "--").split("\n")) | set(git("diff", "--name-only", "--cached", "--").split("\n"))
                 | set(git("ls-files", "--others", "--exclude-standard").split("\n")) - {""})
moved, problem = lib.moved_only_paths(base, changed, cwd=repo)
print(f"name=real_repo base={base} changed={len(changed)} moved_only={len(moved)} problem={problem}")
history = re.compile(r"git (?:log|diff|blame|rev-list|show)\b[^`\n]*?(?:-- [^`\n]*scripts/experiment-apparatus|(?<![0-9a-fA-F])[^\s:`'\"]*[^0-9a-fA-F\s:`'\"][^\s:`'\"]*:(?:\./)?scripts/experiment-apparatus)")
hits = []
for path in sorted(moved):
    full = os.path.join(repo, path)
    if not os.path.isfile(full):
        continue
    try:
        text = open(full, encoding="utf-8").read()
    except UnicodeDecodeError:
        continue
    for number, line in enumerate(text.split("\n"), 1):
        if "scripts/experiment-apparatus" in line and history.search(line):
            hits.append((path, number, history.search(line).group(0)[:160]))
print(f"name=real_repo.history_bound_lines_in_moved_only_files count={len(hits)}")
for path, number, snippet in hits:
    print(f"name=real_repo.history_bound_line path={path} line={number} snippet={snippet!r}")
