#!/usr/bin/env python3
"""B1 扫描的分类计数（m3-formal-code-b24-r1 opus）：每份输入按 class 计数，另按 (覆盖写次数, 覆盖写字节, 单元, 批) 一组列出
今天的用例绿而「建 0 个」不红（NOW-GREEN-ZERO-SURVIVES）与新前提先红而旧用例本来分得开（NOW-RED-AT-B1C-OLD-KILLED-ZERO）的槽区间。"""
import collections, re, sys
for path in sys.argv[1:]:
    classes = collections.Counter(); groups = collections.defaultdict(lambda: collections.defaultdict(list)); pre_reasons = collections.Counter()
    for line in open(path, encoding="utf-8"):
        m = re.match(r"b1 slots=(\d+) ow=(\d+) owbytes=(\d+) units=(\d+) batches=(\d+) class=(\S+)", line)
        if not m:
            continue
        slots, cls = int(m.group(1)), m.group(6)
        classes[cls] += 1
        groups[m.group(2, 3, 4, 5)][cls].append(slots)
        r = re.search(r"b1c_publish0=(\S+)", line)
        if r and cls.startswith("NOW-RED-AT-B1C"):
            pre_reasons[(cls, r.group(1))] += 1
    print(f"{path.rsplit('/', 1)[-1]} disks={sum(classes.values())} " + " ".join(f"{k}={v}" for k, v in sorted(classes.items())))
    for (cls, reason), v in sorted(pre_reasons.items()):
        print(f"  b1c_refused class={cls} publish0={reason} disks={v}")
    for key, by in sorted(groups.items(), key=lambda kv: tuple(int(x) for x in kv[0])):
        def span(c):
            s = sorted(by.get(c, []))
            return f"{s[0]}..{s[-1]}({len(s)})" if s else "-"
        if any(c.startswith("NOW") for c in by):
            print(f"  ow={key[0]} owbytes={key[1]} units={key[2]} batches={key[3]} kills_both={span('NOW-GREEN-KILLS-BOTH')} "
                  f"zero_survives={span('NOW-GREEN-ZERO-SURVIVES')} b1c_red_old_silent={span('NOW-RED-AT-B1C-OLD-ZERO-SURVIVED')} "
                  f"b1c_red_old_killed={span('NOW-RED-AT-B1C-OLD-KILLED-ZERO')}")
