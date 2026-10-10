#!/usr/bin/env python3
"""真历史的段号与段长（m3-formal-code-b24-r1 opus）：读 opus-b24-probe real 的输出，每份一行。
候选段 = 与 draw_crash_points 同一条界（整段落在 mkfs 之后、最后一个跑完的操作之前）；摆出 = 这一档的摆法真摆出的第一次崩溃态。"""
import collections, sys
for path in sys.argv[1:]:
    histories = 0; cand_idx = []; cand_len = collections.Counter(); drawn_idx = []; drawn_len = collections.Counter(); config = ""
    for line in open(path, encoding="utf-8"):
        if line.startswith("H "):
            histories += 1
            fields = dict(f.split("=", 1) for f in line.split()[1:])
            config = f"operations={fields['operations']} weights={fields['weights']} width={fields['width']} points={fields['points']}"
            for item in filter(None, fields["cand"].split(",")):
                i, l = map(int, item.split(":"))
                cand_idx.append(i); cand_len[l] += 1
        elif line.startswith("P "):
            _, _, seg, bits = line.split()
            drawn_idx.append(int(seg)); drawn_len[len(bits)] += 1
    def over(counter, n): return sum(c for l, c in counter.items() if l > n)
    print(f"{path.rsplit('/', 1)[-1]} {config} histories={histories} candidate_segments={len(cand_idx)} max_candidate_index={max(cand_idx)} "
          f"candidates_with_index_over_63={sum(i > 63 for i in cand_idx)} candidates_with_index_over_255={sum(i > 255 for i in cand_idx)} "
          f"longest_candidate={max(cand_len)} candidates_longer_than_12={over(cand_len, 12)} longer_than_28={over(cand_len, 28)} "
          f"drawn_states={len(drawn_idx)} max_drawn_index={max(drawn_idx)} drawn_with_index_over_63={sum(i > 63 for i in drawn_idx)} "
          f"longest_drawn={max(drawn_len)} drawn_longer_than_12={over(drawn_len, 12)} drawn_longer_than_28={over(drawn_len, 28)} "
          f"candidate_length_histogram={dict(sorted(cand_len.items()))}")
