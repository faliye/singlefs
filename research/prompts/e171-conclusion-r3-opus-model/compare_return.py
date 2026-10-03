"""e171-conclusion-r3 攻方：返回值探针一行变异的输出与基线比。
用法：python3 compare_return.py <基线 tsv> <行输出目录> <行表>
每行一条：写流（op）相同与否、每次调用的结局相同与否、哪几次调用交回值的摘要不同、末了分配器 / 现行那一版的摘要同不同。
"""
import os
import sys

base_path, rows_dir, table = sys.argv[1:4]


def load(path):
    ops, calls, final = [], [], None
    for line in open(path, encoding="utf-8"):
        c = line.rstrip("\n").split("\t")
        if c[1] == "op":
            ops.append(tuple(c[2:]))
        elif c[1] == "call":
            calls.append((c[2], c[3], c[4], c[5], c[6], c[8]))
        elif c[1] == "final":
            final = (c[2], c[3])
    return ops, calls, final


base = load(base_path)
counts = {}
for line in open(table, encoding="utf-8"):
    row, file, name = line.rstrip("\n").split("\t")
    path = os.path.join(rows_dir, f"row-{row}.tsv")
    if not os.path.exists(path):
        print(f"name=return_row row={row} status=no_output")
        counts["no_output"] = counts.get("no_output", 0) + 1
        continue
    ops, calls, final = load(path)
    ops_same = ops == base[0]
    words_same = [(c[0], c[1], c[4]) for c in calls] == [(c[0], c[1], c[4]) for c in base[1]]
    returned_diff = [f"{c[0]}:{c[1]}" for c, b in zip(calls, base[1]) if c[5] != b[5]] if words_same else ["calls_differ"]
    final_allocator = final is not None and final[0] == base[2][0]
    final_output = final is not None and final[1] == base[2][1]
    hidden = ops_same and words_same and (returned_diff or not final_allocator or not final_output)
    status = "hidden_difference" if hidden else ("identical" if ops_same and words_same else "visible_difference")
    counts[status] = counts.get(status, 0) + 1
    print(f"name=return_row row={row} status={status} ops_same={ops_same} call_words_same={words_same} "
          f"returned_differs_at={','.join(returned_diff) or 'none'} final_allocator_same={final_allocator} final_output_same={final_output} "
          f"file={file} row_name={name}")
print("name=return_summary " + " ".join(f"{k}={v}" for k, v in sorted(counts.items())))
