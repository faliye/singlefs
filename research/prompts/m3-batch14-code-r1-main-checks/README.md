# m3-batch14-code-r1 主 agent 复跑（2026-10-04）

攻方腿模型目录只读，工作目录在会话草稿目录，命令（仓根）：
`ATTACK_THREADS=31 bash research/prompts/m3-batch14-code-r1-opus-model/rerun.sh <阶段> <空工作目录>`，阶段依次 `build-ga build-fx r-h2-detail r-narrow-ga-0 r-narrow-fx-0 renum-h2-1`，每段退出码 0。

| 产物 | 与腿的比 |
|---|---|
| `r-h2-ga-detail.log` | 与腿的 `out/r-h2-ga-detail.log` 排序后只差两行：内存包装的放行说明（腿那份多一行）与 `RDONE` 的耗时；腿报告引的 `RCUT … n=1 … m=6 …` 那一行逐字在（这份第 3 行），`RTALLY` 行全同 |
| `renum-h2-ga-1.log` | `RENUM`、`RENUMV` 行与腿的 `out/renum-h2-ga.log` n = 1 那一段逐行相同（20511 个状态，丢盘 0 读不出 2048 + 4096 + 6158 = 12302） |
| `remount-narrow-overwrite0-summary.txt` | 腿没单出覆盖写 0 这一档；GA 48 个起点里 5 个读不出、FX 0 个、两臂再挂被拒 0，落在腿 `out/remount-summary.txt` 窄盘族四档合计（GA 13 / 186、FX 0 / 186）之内 |
