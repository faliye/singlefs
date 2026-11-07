"""只在临时目录复现文档契约；不修改仓库源文件，不连接外部服务。

退出 0 表示观察到了这里明确列出的现象，不表示产品契约通过。
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/kb/replace-batch.py"


def observe(directory, name, *, legacy, second_anchor_missing=False):
    target = directory / (name + ".txt")
    target.write_text("original\n", encoding="utf-8")
    edits = [{"file": str(target), "old": "original", "new": "replacement"}]
    if second_anchor_missing:
        edits.append({"file": str(target), "old": "absent", "new": "other"})
    specification = directory / (name + ".json")
    specification.write_text(json.dumps(edits), encoding="utf-8")
    environment = dict(os.environ)
    for key in ("REPLACE_BATCH_BREAK", "REPLACE_BATCH_WRITE_EACH", "LIB_ATOMIC_REPLACE_BREAK"):
        environment.pop(key, None)
    if legacy:
        environment["REPLACE_BATCH_WRITE_EACH"] = "1"
    before_inode = target.stat().st_ino
    result = subprocess.run(
        [sys.executable, "-B", str(SCRIPT), "--dry-run", str(specification)],
        cwd=ROOT, env=environment, capture_output=True, text=True, timeout=30,
    )
    observed = {
        "case": name, "exit": result.returncode,
        "content": target.read_text(encoding="utf-8"),
        "same_inode": target.stat().st_ino == before_inode,
        "stdout": result.stdout, "stderr": result.stderr,
    }
    print(json.dumps(observed, ensure_ascii=False), flush=True)
    return observed


with tempfile.TemporaryDirectory(prefix="singlefs-doc-audit-") as temporary:
    directory = Path(temporary)
    control = observe(directory, "normal_dry_run", legacy=False)
    legacy = observe(directory, "legacy_dry_run", legacy=True)
    incomplete = observe(directory, "legacy_dry_run_later_anchor_missing", legacy=True,
                         second_anchor_missing=True)
    assert (control["exit"], control["content"]) == (0, "original\n"), control
    assert (legacy["exit"], legacy["content"], legacy["same_inode"]) == (
        0, "replacement\n", True), legacy
    assert (incomplete["exit"], incomplete["content"]) == (1, "replacement\n"), incomplete
print("REPRODUCED: legacy switch bypasses --dry-run and writes before checking all anchors")
