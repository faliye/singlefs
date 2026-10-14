#!/usr/bin/env python3
"""D4 探针：冻结副本根上被 git 忽略的 multi-host.env 里的私有值（第二台主机名、第二台上的路径、清场命令里的词）与本机主机名，
在冻结副本的其余文件里出现在哪几份。只打键名、词的种类与长度、命中的文件路径，一个值都不打。
用法：python3 d4-private-values.py <冻结副本>"""
import os, re, shlex, socket, sys

frozen = os.path.abspath(sys.argv[1])
config = os.path.join(frozen, "multi-host.env")
COMMON = {"ssh", "sudo", "systemctl", "--user", "bash", "stop", "start", "is-active", "--quiet", "-o", "BatchMode=yes",
          "true", "false", "&&", "||", "env", "sh", "-c", "exec", "test", "status", "is-enabled", "!", "nvidia-smi", "grep", "-q"}
tokens = []   # (键, 第几个词, 种类, 值)
for line in open(config, encoding="utf-8"):
    line = line.strip()
    if not line or line.startswith("#") or "=" not in line:
        continue
    key, value = line.split("=", 1)
    if key in ("PEER_MEMORY_CAP", "ENABLE_ACROSS_MACHINES", "ENABLE_GPU"):
        continue
    value = value.strip().strip("'\"")
    try:
        words = shlex.split(value)
    except ValueError:
        words = value.split()
    for index, word in enumerate(words):
        for piece in re.split(r"[,;:@]", word) if key.startswith("GPU_CARDS") else [word]:
            piece = piece.strip()
            if len(piece) < 4 or piece in COMMON or piece.isdigit() or re.fullmatch(r"[0-9.]+[GMK]?i?B?", piece):
                continue
            kind = "路径" if "/" in piece else ("服务单元" if piece.endswith((".service", ".target", ".socket")) else "其他词")
            tokens.append((key, index, kind, piece))
local = socket.gethostname()
tokens.append(("本机 hostname", 0, "主机名", local))
skip_dirs = {".git", "target", "__pycache__"}
skip_files = {os.path.join(frozen, "multi-host.env"), os.path.join(frozen, "qemu.env")}
hits = {}
for directory, subdirectories, files in os.walk(frozen):
    subdirectories[:] = [name for name in subdirectories if name not in skip_dirs]
    for name in files:
        path = os.path.join(directory, name)
        if path in skip_files:
            continue
        try:
            with open(path, "rb") as handle:
                data = handle.read(8 * 1024 * 1024)
        except OSError:
            continue
        for number, (key, index, kind, value) in enumerate(tokens):
            if value.encode() in data:
                hits.setdefault(number, []).append(os.path.relpath(path, frozen))
for number, (key, index, kind, value) in enumerate(tokens):
    files = sorted(hits.get(number, []))
    print(f"name=d4 key={key} word={index} kind={kind} length={len(value)} files={len(files)}")
    if len(value) >= 8:   # 短词（4–7 个字）只报份数：它们是常用词，逐份列出来上千行，见报告 D4 检查形态一节
        for path in files:
            print(f"  {path}")
