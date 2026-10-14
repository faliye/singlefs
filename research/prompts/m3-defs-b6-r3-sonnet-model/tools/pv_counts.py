#!/usr/bin/env python3
"""只数不打印：用冻结副本 doc-text.sh 里的 PRIVATE_VALUES_LIBRARY 取词，输出词数、带 / 的词数、门槛字数。
一个值也不写出来。用法：pv_counts.py <冻结副本根> [配置文件]；配置文件默认 <冻结副本根>/multi-host.env。"""
import sys, os, types
root = os.path.abspath(sys.argv[1]); conf = sys.argv[2] if len(sys.argv) > 2 else os.path.join(root, "multi-host.env")
lines = open(os.path.join(root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('PRIVATE_VALUES_LIBRARY="$(cat <<\'PY_PRIVATE_VALUES\''))
end = next(i for i in range(start + 1, len(lines)) if lines[i] == "PY_PRIVATE_VALUES")
mod = types.ModuleType("pv"); exec(compile("\n".join(lines[start + 1:end]), "pv", "exec"), mod.__dict__)
targets, short, cfg_words, host_note, minimum = mod.load_private_targets(conf, "", "")
cfg = [t for t in targets if t[0] != mod.HOSTNAME_KEY]
host = [t for t in targets if t[0] == mod.HOSTNAME_KEY]
print(f"门槛 {minimum} 个字；配置里取到的词 {len(cfg)}（其中带 / {sum('/' in w for _, w in cfg)}）；本机主机名词 {len(host)}（带 / {sum('/' in w for _, w in host)}）；合计 {len(targets)}（带 / {sum('/' in w for _, w in targets)}）；不足门槛没取 {sum(short.values())}")
