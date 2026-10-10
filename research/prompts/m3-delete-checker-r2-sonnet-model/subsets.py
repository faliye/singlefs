#!/usr/bin/env python3
"""bad_history_model 的补充：有复用的时刻里，甲 / 乙 / 丙之间的包含关系与「全没红」个数（窗口 K = 3、6、1000）。"""
import random, sys
sys.argv = ["x", "3000", "10", "refuse"]
import bad_history_model as b

for window in (3, 6, 1000):
    cnt = dict(total=0, jia_not_yi=0, yi_not_jia=0, bing_not_yi=0, yi_not_bing=0, none_of_four=0)
    for seed in range(3000):
        rng = random.Random(seed)
        w = b.BadWorld(rng)
        for _ in range(10):
            op = rng.choice(["create", "create", "unlink", "unlink", "rollback", "crash", "press"])
            ok = b.press(w) if op == "press" else {"create": w.create, "unlink": w.unlink, "rollback": w.rollback, "crash": w.crash_then_publish}[op]()
            if ok is False or not b.reissued(w):
                continue
            c = b.checks(w, window)
            cnt["total"] += 1
            cnt["jia_not_yi"] += int(c["jia"] and not c["yi"])
            cnt["yi_not_jia"] += int(c["yi"] and not c["jia"])
            cnt["bing_not_yi"] += int(c["bing"] and not c["yi"])
            cnt["yi_not_bing"] += int(c["yi"] and not c["bing"])
            cnt["none_of_four"] += int(not any(c.values()))
    print(window, cnt)
