#!/usr/bin/env python3
"""把 <门禁根>/.claude/gate.d/doc-text.sh 里 private-values 格的 python 抽出来，在 <样本 git 仓> 里原样跑：
基准取 HEAD，改动范围取 git 的未跟踪加已改动文件（代替 scripts/gate/changed-paths.sh 的 gate_changed_paths），配置与主机名替身取样本根里的。
用法：run_pv_cell.py <门禁根> <样本 git 仓>"""
import os, subprocess, sys, tempfile
gate_root, sample = map(os.path.abspath, sys.argv[1:3])
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('  python3 - "$base" "$configuration_file" "$hostname_file"'))
stop = next(i for i in range(start + 1, len(lines)) if lines[i] == "PY")
code = "\n".join(lines[start + 1:stop])
git = lambda *a: subprocess.run(["git", "-c", "core.quotepath=false", *a], cwd=sample, capture_output=True, text=True).stdout
base = git("rev-parse", "HEAD").strip()
changed = sorted(set(git("ls-files", "--others", "--exclude-standard").split("\n") + git("diff", "--name-only", base).split("\n")) - {""})
listing = tempfile.NamedTemporaryFile("w", delete=False, suffix=".txt"); listing.write("\n".join(changed) + "\n"); listing.close()
hostname = os.path.join(sample, ".private-values-hostname")
limit = os.path.join(sample, ".private-values-size-limit")
args = [sys.executable, "-", base, os.path.join(sample, "multi-host.env"), hostname if os.path.isfile(hostname) else "", listing.name,
        os.environ.get("GATE_TEXT_NOTATIONS_BREAK", ""), limit if os.path.isfile(limit) else ""]
result = subprocess.run(args, input=code, text=True, cwd=sample, capture_output=True)
os.unlink(listing.name)
sys.stdout.write(result.stdout); sys.stderr.write(result.stderr)
sys.exit(result.returncode)
