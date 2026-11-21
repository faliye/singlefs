# E202 造池改成一次发布写一整个文件：问题单（2026-11-18）

只写问题与候选定义，不写倾向。实验 E202（GPU与CPU两种模式GC对照组），跑前登记 `research/prompts/e202-preregistration.md`。

| # | 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| 1 | 起步池照登记第五节「负载：池怎么造」的造法造不造得出登记的三档 | 现造法（造数据挂载一次发布一个单元）；一次发布写一整个文件的 16 个单元（用户 2026-11-18 定改成这一种）；发布合并的个数取别的值 | 一档在造数据或造碎片挂载里被 `SlotPairsOutsideTheClusterSegmentsForDataUnits` 拒；或造完而几何核那几条（至少 90% 的段活槽数恰好 32、greedy 选的段活槽数不超过 32 且未释放记录全是码 1 数据单元）不过 | 三档各造完一次、几何核过，或点名哪一档在哪一步过不了 | 开着 |
| 2 | 改了造法之后起步池还是不是登记要的那种池 | 同一个量的两种造法 | 新造法造出的池，每段活槽数的直方图、全空段数、`mapped_units` 条数、每次量挂载整理要搬的批数，与登记第五节写的目标对不上 | 三档的这几样读数各报一次，与登记的目标逐条比 | 开着 |
| 3 | 造池要多久、占多少内存 | 同上 | 一档造池超过任务服务的四小时上限，或三档同时造时常驻内存超过一台机器装得下的 | 三档各自的造池用时与常驻峰值；在不动形态的两批造池提速补丁打与不打两种树上各量一次 | 开着 |

已知的读数与出处（供设计员写修订时引，不代表倾向）：2288MiB 档、16384 个活数据单元、现造法、`construction-timing --chunks shared`：任务 20071 的探针行「label=refusal publishes=21987 segments=2288 cluster_segments=1614 … core_pairs_outside=0 … scan_cluster_live_histogram=0:8,1:1416,16-31:1,2:140,3-7:46,8-15:3」，重挂之后「cluster_segments=1 … core_pairs_outside=49503」；定位报告 `~/.local/share/singlefs/work/e202-pool-construction-no-space-investigation/result.md`。
