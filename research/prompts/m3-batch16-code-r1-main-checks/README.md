# m3-batch16-code-r1 主 agent 复跑（2026-10-04）

攻方腿模型目录只读，工作目录在会话草稿目录，命令（仓根）：
`ATTACK_THREADS_BUILD=31 bash research/prompts/m3-batch16-code-r1-opus-model/rerun.sh <阶段> <空工作目录>`，阶段依次 `build-jia build-nh build-n7clamp l1-jia-hide1 l1-nh-hide1 tests-n7clamp-build tests-n7clamp`，每段退出码 0。

| 产物 | 与腿的比 |
|---|---|
| `l1-jia-hide1.log` | 去掉耗时与内存包装说明两类行后，排序与腿的 `out/l1-jia-hide1.log` 逐行相同（306 行）；层 0 窄盘 `264:6:4:0` n = 15 那一段 20511 个状态，丢盘 0 读不出 6144 + 8195 = 14339 |
| `l1-nh-hide1.log` | 同一比法逐行相同；同一段 20511 个状态读不出 0 |
| `tests-n7clamp.log` | 五行 `test result` 去掉耗时之后与腿的逐行相同：错实现「意图时间取 max(给的, 上一版)」下 33、5、8、16、9 条全过 |
