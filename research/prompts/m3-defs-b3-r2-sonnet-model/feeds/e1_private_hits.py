# E1（第一轮攻方 D4）：冻结副本根上被忽略的 multi-host.env 里 8 个字以上的词与本机主机名，在冻结副本其余文件里出现在哪几份。
# 只打键名（或「本机 hostname」）、命中份数与文件路径，不打值、不打长度、不打种类。没有 multi-host.env 时退 77。
# 用法：python3 -I e1_private_hits.py <冻结副本>
import os, re, shlex, socket, sys
frozen = os.path.abspath(sys.argv[1])
config = os.path.join(frozen, "multi-host.env")
if not os.path.isfile(config):
    print("  ! 冻结副本根下没有 multi-host.env，没有私有值可比（这不是通过）"); sys.exit(77)
COMMON = {"ssh", "sudo", "systemctl", "--user", "bash", "stop", "start", "is-active", "--quiet", "-o", "BatchMode=yes", "true", "false", "&&", "||", "env", "sh", "-c", "exec", "test", "status", "is-enabled", "nvidia-smi", "grep", "-q"}
words = []
for line in open(config, encoding="utf-8"):
    line = line.strip()
    if not line or line.startswith("#") or "=" not in line: continue
    key, value = line.split("=", 1)
    if key in ("PEER_MEMORY_CAP", "ENABLE_ACROSS_MACHINES", "ENABLE_GPU"): continue
    value = value.strip().strip("'\"")
    try: parts = shlex.split(value)
    except ValueError: parts = value.split()
    for word in parts:
        for piece in re.split(r"[,;:@]", word):
            piece = piece.strip()
            if len(piece) >= 8 and piece not in COMMON: words.append((key, piece))
local = socket.gethostname()
if len(local) >= 8 and local != "localhost": words.append(("本机 hostname", local))
hits = {}
for directory, subdirectories, files in os.walk(frozen):
    subdirectories[:] = [n for n in subdirectories if n not in {".git", "target", "__pycache__"}]
    for name in files:
        path = os.path.join(directory, name)
        if path in {os.path.join(frozen, "multi-host.env"), os.path.join(frozen, "qemu.env")}: continue
        try: data = open(path, "rb").read(8 * 1024 * 1024)
        except OSError: continue
        for key, word in words:
            if word.encode() in data:
                lines = [i for i, l in enumerate(data.split(b"\n"), 1) if word.encode() in l]
                hits.setdefault(key, []).append((os.path.relpath(path, frozen), lines))
for key in sorted({k for k, _ in words}):
    found = sorted(hits.get(key, []))
    print(f"name=E1 key={key} files={len(found)}")
    for path, lines in found: print(f"  {path} 行 {lines}")
