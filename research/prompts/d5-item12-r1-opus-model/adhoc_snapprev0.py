#!/usr/bin/env python3
# 按 05-快照-空间记账机制.md:247（已定项 9）字面读：快照条目的 previous_snapshot_txg 写 0，
# prev(S) 只能从同头的活快照枚举出来；克隆头的第一个快照枚举不到前驱，取 0（origin 在树表字段里没有落点）。
# 与臂无关的共同前置：对 REF / 甲 / 乙 / 丙 各跑一次 BFS，找最短的真值判红历史。
import json
import dlmodel as m


def patch(arm_class):
    class Patched(arm_class):
        def snapshot(self, world, head, sid):
            super().snapshot(world, head, sid)
            own = world.head_snaps(head)
            if own and own[0] == sid and world.heads[head]["origin"] is not None:
                self.ps[sid] = 0          # 枚举不到前驱、origin 链接不在字段里 ⇒ 0
    return Patched


for name, factory in (("REF", lambda w, mut=(): patch(m.Ref)(w, mut)),
                      ("JIA", lambda w, mut=(): patch(m.Jia)(w, mut)),
                      ("YI_min", lambda w, mut=(): patch(m.Yi)(w, mut, merge="min")),
                      ("BING_R", lambda w, mut=(): patch(m.Bing)(w, mut, mode="R")),
                      ("BING_T", lambda w, mut=(): patch(m.BingTid)(w, mut))):
    m.ARMS[name + "_snapprev0"] = factory
    print(json.dumps({"section": "adhoc-snapprev0", **m.bfs_shortest(name + "_snapprev0", m.SMALL, 7)}, default=str))
print(json.dumps({"section": "END", "rows": 5}))
