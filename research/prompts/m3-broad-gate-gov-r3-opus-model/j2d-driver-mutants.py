#!/usr/bin/env python3
"""攻方 J2-d：驱动 escalation_evidence_problems（升档证据那一判）的判红分支逐个删掉（每种一份树拷贝，只改 research/scripts/broad-mutation.py 一处），
交主 agent 在每份拷贝上跑驱动 --selftest（子 agent 起拷贝的自证被重型闸拒），看 74 格里有没有一格红。
用法：j2d-driver-mutants.py <原样树根> <放拷贝的目录>；每行打「名字<TAB>删的是哪一支」。旧串在文件里必须恰好命中一次。"""
import os, shutil, subprocess, sys
base, into = sys.argv[1:3]
TARGET = "research/scripts/broad-mutation.py"
MUTANTS = [
    ("d1-control-same-exit", "（对照）两趟结局不同不算证据不齐：自证第 4 条（253|250）应当红",
     '    if escalated_exit != first_exit:\n        problems.append', '    if False:\n        problems.append'),
    ("d2-no-baseline-ok", "这一版没有这一步的基线也算证据齐",
     '        problems.append("这一版没有这一步在没改过的树上的基线（超时要 baseline.tsv 的绿记录，撞顶要 baseline-peaks 里的峰值）")', '        pass'),
    ("d3-no-multiple", "升档那趟不到基线的 K 倍也算证据齐",
     '    elif escalated_amount is None or escalated_amount < multiple * baseline:', '    elif escalated_amount is None:'),
    ("d4-not-larger-ok", "升档那趟没比原来的大也算证据齐",
     '    if escalated_amount is None or first_amount is None or escalated_amount <= first_amount:', '    if escalated_amount is None or first_amount is None:'),
    ("d5-no-wrapper-first", "原来那趟的日志里没有包装那一句也算证据齐",
     '    if wrapper_hit_text(first_exit, first_text) not in first_log_text:', '    if False:'),
    ("d6-no-wrapper-escalated", "升档那趟的日志里没有包装那一句也算证据齐",
     '    if wrapper_hit_text(escalated_exit, escalated_text) not in escalated_log_text:', '    if False:'),
]
for name, what, old, new in MUTANTS:
    target = os.path.join(into, name)
    shutil.rmtree(target, ignore_errors=True)
    subprocess.run(["rsync", "-a", base.rstrip("/") + "/", target + "/"], check=True)
    path = os.path.join(target, TARGET)
    text = open(path, encoding="utf-8").read()
    if text.count(old) != 1:
        sys.exit(f"{name}：旧串命中 {text.count(old)} 次，不是 1")
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
    print(f"{name}\t{what}")
