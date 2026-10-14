#!/usr/bin/env python3 -I
"""数私有值命中：只打份数与行号，不打值。
用法：value_hits.py <配置文件 multi-host.env> <标签=目录或git引用> ...
  标签=目录路径  : 在该目录下的 records/2026-09-30-里程碑三开工第一轮.md 与
                  research/prompts/m3-batch10x-code-r1-opus-output.md 里数两种值的命中行号
  标签=git:<仓根>:<提交>: 从 git 对象里读同样两份
值：PEER_SSH_HOST 的整个值、本机 hostname；不写出来。
"""
import os, subprocess, sys, socket

def load(cfg):
    peer = None
    for line in open(cfg, encoding="utf-8"):
        line = line.strip()
        if line.startswith("PEER_SSH_HOST="):
            peer = line.split("=", 1)[1].strip().strip("'\"")
    return peer, socket.gethostname()

FILES = ["records/2026-09-30-里程碑三开工第一轮.md",
         "research/prompts/m3-batch10x-code-r1-opus-output.md"]

def hits(text, needle):
    if not needle:
        return None
    return [i for i, l in enumerate(text.splitlines(), 1) if needle in l]

def main():
    cfg = sys.argv[1]
    peer, host = load(cfg)
    print("name=value_hits peer_value_loaded=%s hostname_loaded=%s" % (bool(peer), bool(host)))
    for spec in sys.argv[2:]:
        label, _, where = spec.partition("=")
        for f in FILES:
            try:
                if where.startswith("git:"):
                    _, root, rev = where.split(":", 2)
                    text = subprocess.run(["git", "-C", root, "show", "%s:%s" % (rev, f)],
                                          capture_output=True, text=True, check=True).stdout
                else:
                    text = open(os.path.join(where, f), encoding="utf-8").read()
            except Exception as e:
                print("name=value_hits label=%s file=%s missing=%s" % (label, f, type(e).__name__)); continue
            print("name=value_hits label=%s file=%s peer_lines=%s host_lines=%s" % (
                label, f, hits(text, peer), hits(text, host)))

main()
