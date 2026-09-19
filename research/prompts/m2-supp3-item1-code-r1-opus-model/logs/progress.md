# 进度记录（攻方腿 Opus，m2-supp3-item1-code-r1；时刻 UTC）
- 15:33 开工；快照 11 个文件 sha256 全 OK；ps 无性能测量，无别的 cargo。
- 15:36 slot-A / slot-B 快档 18 条变异（release，overflow-checks 开）；15:42 起大档 1000x60、3000x40。
- 15:45 ps 看到：另一会话 `second_transaction_step_zero_layer0 --include-ignored`（pid 691123）、sonnet 腿的大档（pid 712386），没有 qemu / fio / e152 / vm-bench；负载 39 → 65。
- 15:54 slot-P 够达探针（PA3/PA4/PB2/PN5/PN2），slot-D 机理探针（gap）。
- 16:06 ps：另一会话 /tmp/claude-1000/m2-layer0-parallel 的 layer0 全量（pid 2087215），非性能测量；负载 104。本腿 5 路 16 线程在跑。run-gap.sh / run-z2.sh 还原用 mv 不 touch，跨 crate 连跑时 cargo 不重编被还原的 crate：A5（gap 96x30）、A8（gap 96x30 首跑）、A2（gap 1000x30 首跑）、z2 首跑的 A5/A8 作废重跑（脚本已加 touch）。
- 16:20 slot-G：proposal-patch.py（P1 收窄第 1 条、P2 冷启动 Failed 算失败、P3 装得下的长度各至少一次 Ok）在 BASE 与 A1/A2/B6/B1/A5/A8 上量快档与两档大档。
