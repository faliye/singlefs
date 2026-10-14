#!/usr/bin/env python3
"""X6：仓里每份带 # long-run-hosts: 文件头的脚本，用 <仓根>/.claude/hooks/lib_long_run.py 的 judge_script_header 判（与 detached-run.sh long-run-check、host-dispatch.py 同一个函数），
打印读法（live / registered）、问题条数与第一条问题的前 160 字。用法：x6_headers.py <仓根> <脚本清单文件>"""
import os, sys
root = os.path.abspath(sys.argv[1])
sys.path.insert(0, os.path.join(root, ".claude/hooks"))
import lib_long_run as L
for relative in open(sys.argv[2], encoding="utf-8").read().split():
    path = os.path.join(root, relative)
    declared = L.script_header_declaration(path)
    problems = L.judge_script_header(path, declared, root)
    print(f"{relative}\t{L.header_hosts_reading(path, root)}\t问题 {len(problems)} 条\t{(problems[0][:160] if problems else '')}")
