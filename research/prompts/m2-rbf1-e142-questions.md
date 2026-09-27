# 实一（回退改形态：格式与系统配置）之后的 E142 重跑问题单（2026-09-26）

实一照回退改形态的 kb 写回改了格式：系统配置字段表加回退下界 F，incompat 布局身份换位，根记录 flags 位 0 定为卸载记号。第一个事务写出的字节跟着变。实一的实现员报告里有它自报的差异（报告位置与主 agent 的判定见 `records/2026-09-24-里程碑二收尾调度.md` 第三节「实一交回」那一行），实验要独立算，不许照抄，设计时也不读那一节。

下面每一行只写问题与候选，不写倾向。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | E142（第一个事务的干跑） 的独立装置照三条条款改完之后，第一个事务写出的每个区域与 `crates/` 实装逐字节比，全等还是不等。三条条款是：D22（单元原子性怎么合成） 已定项 9（系统配置的字段表）、D15（格式冻结政策） 已定项 4（feature bit 的位分配登记表）、D22（单元原子性怎么合成） 已定项 7（根记录的字段表）。装置不读 `crates/` | 全等 / 不等 | 任一区域有一个字节不等 | 两块盘的全部区域都比完，产物里每个区域一行比对结论 | **够判：全等**（2026-09-26 第十八次跑；`research/results/e142-first-txn-dry-run-2026-09-26-r18-main.out` 第 393 行 `name=impl_bytes_equal_summary model_regions=29 crates_regions=29 matched=29 equal=29 unequal=0`，独立 bin `research/results/e142-region-diff-independent-2026-09-26-r18.out` 第 30 行 `equal=29 unequal=0`；阳性对照 `e142-r18-controls-2026-09-26.out` 第 31 行 `equal=28 unequal=1` 分得出差别。比的是实一加实二那一版 `crates/` 的快照。F 与卸载记号只在值 0 上比过） |
| 2 | 与第十七次跑主产物 `research/results/e142-first-txn-dry-run-2026-09-25-r17-main.out` 比，第一个事务写出的字节在哪些区域、哪些偏移上变了；每一处变化能不能对到第 1 行那三条条款中的某一条 | 全部对得到 / 有对不到的 | 有一处变化对不到这三条条款中的任一条 | 每处变化都列出区域、偏移、改前值、改后值与对到的条款 | **够判：全部对得到**（2026-09-26 第十八次跑；`research/results/e142-region-old-new-independent-2026-09-26-r18.out` 第 41 行 `name=old_new_independent_verdict all_changes_mapped=true`：29 个区域里变了 2 个（两份系统配置槽），变的是 D15（格式冻结政策） 已定项 4 的 incompat 第一个字节与它带动的整槽校验和；阳性对照 PC6 (i)–(iii) 判 `all_changes_mapped=false`，`e142-r18-controls-2026-09-26.out` 第 101、140、179 行） |
