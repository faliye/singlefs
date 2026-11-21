#!/usr/bin/env python3
"""攻方腿核「已经量的」那一节的辅助：对开工快照点名的几份被判文件，列出命令里点到它的自证、开关证红、样本档与真仓单格那几趟，
逐趟标那一趟收尾早于还是晚于这份文件此刻那一版的改动时间（早于的那一趟量的是更早的一版，不是被判的这一版）。
只读任务服务的尝试目录（spec.json、result.json）与文件的改动时间；不读服务的库、不写任何文件。打出来的行带任务号，只给主 agent 现查用，不抄进仓。
用法：python3 measured_versus_judged_versions.py <仓根> <尝试目录>（尝试目录是任务服务状态目录下的 attempts）"""
import json
import os
import sys

root, attempts = sys.argv[1], sys.argv[2]
# 被判文件 → 命令里认它的字串（自证、开关证红、样本档、门禁单格）
WATCHED = {
    ".claude/hooks/write-guard.sh": ("write-guard.sh --selftest",),
    ".claude/hooks/write-guard-selftest.py": ("write-guard.sh --selftest",),
    ".claude/hooks/bash-command-detector.sh": ("bash-command-detector.sh --selftest", "--item .claude/hooks/bash-command-detector.sh"),
    "scripts/gate/merge-and-verify.sh": ("merge-and-verify.sh --selftest", "--item scripts/gate/merge-and-verify.sh"),
    "scripts/gate/merge-and-verify-selftest-lib.sh": ("merge-and-verify.sh --selftest",),
    ".claude/gate.d/code-tooling.sh": ("code-tooling.sh --check crate-roles",),
    "scripts/gate/crate-cross-references-check.py": ("crate-cross-references-check.py --selftest", "--item checker-independence-and-sync.sh",
                                                     "checker-independence-and-sync.sh --check crate-cross-references"),
    ".claude/hooks/ask-user-placement-guard.sh": ("ask-user-placement-guard.sh --selftest", "--item .claude/hooks/ask-user-placement-guard.sh"),
}
since = os.stat(os.path.join(root, "research", "prompts", "after-split-governance-r1-snapshot", "judged-sha256.txt")).st_mtime - 2 * 86400
rows = []
for name in os.listdir(attempts):
    directory = os.path.join(attempts, name)
    try:
        if os.stat(directory).st_mtime < since:
            continue
        with open(os.path.join(directory, "spec.json"), encoding="utf-8") as handle:
            command = " ".join(json.load(handle).get("argv") or [])
        with open(os.path.join(directory, "result.json"), encoding="utf-8") as handle:
            result = json.load(handle)
    except (OSError, ValueError):
        continue
    rows.append((result.get("ended_at") or 0, name, result.get("exit_code"), command))
rows.sort()
for path, needles in WATCHED.items():
    changed = os.stat(os.path.join(root, path)).st_mtime
    for ended, name, code, command in rows:
        if any(needle in command for needle in needles):
            breaking = next((word for word in command.split() if "_BREAK=" in word), "")
            relation = "earlier-version" if ended < changed else "judged-version"
            print(f"name=measured file={path} attempt={name} exit={code} {relation} {breaking}".rstrip())
