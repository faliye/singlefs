#!/usr/bin/env python3
"""V2 模拟丙：在一棵副本里把 HostLookup.query 起 host-resources.py 的那一条边切掉（Popen 那一处换成 None），别的一个字不动。
只用来量「切这一条边之后闭包还剩什么」，不是实现；用法：cut_lookup_edge.py <副本根>"""
import sys

path = sys.argv[1] + "/scripts/hosts/run-with-memory-cap.sh"
text = open(path, encoding="utf-8").read()
old = ('process = self.process = subprocess.Popen([sys.executable, "-B", script, "--json", "--", *self.command], stdout=output, stderr=errors,\n'
       '                                                              stdin=subprocess.DEVNULL, env=environment)')
assert text.count(old) == 1, "旧串不是恰好一处"
with open(path + ".new", "w", encoding="utf-8") as handle:
    handle.write(text.replace(old, "process = self.process = None"))
import os
os.replace(path + ".new", path)
