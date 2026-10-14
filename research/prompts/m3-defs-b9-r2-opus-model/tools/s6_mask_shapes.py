#!/usr/bin/env python3
"""S6 喂法：从 <副本根>/.claude/gate.d/doc-text.sh 抽出 PRIVATE_VALUES_LIBRARY，拿编的词表直接喂 shown_line（term-renames 转出的 sweep-term 输出那一路）、
shown_path（date-once 红行那一路）、snippet_bounds + shown_text（clock-times 这类片段那一路），看输出里还剩不剩整值或值的半截。词表全是编的。
用法：s6_mask_shapes.py <副本根>"""
import os, sys, types
root = sys.argv[1]
lines = open(os.path.join(root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('PRIVATE_VALUES_LIBRARY="$(cat <<'))
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "PY_PRIVATE_VALUES")
library = types.ModuleType("lib_private_values")
exec(compile("\n".join(lines[start + 1:end]), "private-values-library", "exec"), library.__dict__)
targets = [("PEER_SSH_HOST", "demo-peer-host-42"), ("PEER_REPOSITORY_DIRECTORY", "/srv/demo-private/peer-checkout"),
           ("QUIESCE_STOP_COMMAND", "demo-gpu-service"), ("QUIESCE_STOPPED_CHECK_COMMAND", "service-watchdog"), ("本机 hostname", "demo-local-host-09")]
values = [word for _key, word in targets]
def leaks(text):
    found = [w for w in values if w in text]
    halves = [w[:k] for w in values for k in range(6, len(w)) if w[:k] in text and w not in text] + [w[-k:] for w in values for k in range(6, len(w)) if w[-k:] in text and w not in text]
    return f"整值 {found}；半截（≥6 字）{sorted(set(halves), key=len)[-1:] if halves else []}"
sweep_lines = [
    "      research/results/from-peer/srv/demo-private/peer-checkout/notes.md       1 处",
    "     没扫的 3 处（.claude/term-rename-exempt 登记，各有理由）：research/prompts/、records/demo-peer-host-42-对拍.md、scripts/kb/sweep-term.py",
    "      「research/results/demo-peer-host-42」（2 处）：Permission denied",
    "      records/x.md:「第二台 DEMO-PEER-HOST-42 上跑」 锚命中 0 行",
    "      records/x.md:「demo-gpu-service-watchdog 停了」 锚命中 0 行",
]
for line in sweep_lines:
    shown = library.shown_line(line, targets, "")
    print(f"shown_line\t{shown}\t{leaks(shown)}")
for path in ("records/2026-10-13-demo-peer-host-42-对拍.md", "records/peer.demo-local-host-09.md"):
    shown = library.shown_path(path, targets, "")
    print(f"shown_path\t{shown}\t{leaks(shown)}")
for line, position in (("跑完：demo-gpu-service-watchdog 在 2026-10-13T10:31:05 停了。", 28), ("第二台 DEMO-PEER-HOST-42 上 2026-10-13T11:02:00 开跑", 24)):
    start, end = library.snippet_bounds(line, max(0, position - 20), position + 10, targets)
    shown = library.shown_text(line[start:end].strip(), targets)
    print(f"片段\t{shown}\t{leaks(shown)}")
