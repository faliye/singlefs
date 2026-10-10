#!/usr/bin/env python3
# 用法：table2.py <候选,候选…> <文件…>  每个文件一块：头部几个总数 + 所列候选的几列
import sys
want = sys.argv[1].split(",")
for path in sys.argv[2:]:
    head = None; rows = {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("W3MODEL"):
            head = dict(kv.split("=", 1) for kv in line.split()[1:])
        elif line.startswith("W3COUNT"):
            d = dict(kv.split("=", 1) for kv in line.split()[1:]); rows[d["candidate"]] = d
    keys = ["arm","mutation","ring","sweep_threshold","longest","images","legal_images","events","events_where_I-9.29_holds","events_where_existing_checks_silent","events_existing_silent_mounted_since_the_last_rollback","events_existing_silent_not_mounted_since_the_last_rollback","residual_images","residual_where_existing_checks_silent"]
    print(path, " ".join(f"{k}={head[k]}" for k in keys if k in head))
    cols = ["false_positive","event_missed","event_missed_where_existing_checks_silent","event_missed_where_existing_silent_and_mounted_since_the_last_rollback","event_missed_where_existing_silent_and_not_mounted_since_the_last_rollback","residual_missed","residual_missed_where_existing_checks_silent","mutated_history_undetected","mutated_images"]
    short = ["FP","evt_miss","evt_miss_exist","exist_after_mount","exist_no_mount","resid","resid_exist","mut_undet","mut_imgs"]
    print("  " + "candidate".ljust(8) + "".join(s.rjust(len(s)+2) for s in short))
    for c in want:
        d = rows[c]
        print("  " + c.ljust(8) + "".join(d[k].rjust(len(s)+2) for k, s in zip(cols, short)))
