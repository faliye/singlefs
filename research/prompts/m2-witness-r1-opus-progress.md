# m2-witness-r1 云端攻方腿进度（时刻 UTC）
- batch1（H3 G0 L=3 |F|≤2 全族 + w-min）完成：none 213 复现（512/4096），a/b 0 违例。
- batch2：c 全族 0 违例；w-crash 全部 panic（目标不在时间线上），已修。
- batch3（w-crash 全量深度 1）：收到主 agent 转用户令「全量崩溃枚举最后统一跑」，按 pid 停掉 931783 931791 931815 931816 932582 953681（proc.py stop）。停之前完成的两臂：none-512、a-late-512。
- 下一步：改成挑的历史（每候选两条回退边 + 撕裂 / 崩溃 / 两根读不出的构造历史），不做全量枚举。
- 挑的历史 batch4 跑完（w-targeted 17 臂 + w-stale 4 行），recheck 两臂与 batch1 逐字段相同；报告写完 research/prompts/m2-witness-r1-opus-output.md。
