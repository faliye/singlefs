攻方腿 m3-admission-r1-opus 的探针输出（原样，只留从 PROBE / DF / FP / FPB / GEO / CAP / QD / ==== 起的那一截行）。
每个 <步名>.lines 由 tools/rerun.sh 的同名一步产出；腿自己跑时设 RERUN_TREE / RERUN_TARGET 指到已拷好探针的冻结副本拷贝，
一步一个新的 --work，经 scripts/hosts/run-with-memory-cap.sh 跑（held_small、df_small_*、fp_small、geometry、qd 用 2G，其余 8G）。
腿量到的单步挂钟（2026-10-12，本机，release，线程 15）：held_small 约 9 秒；held_wide 约 59 秒；df_small_equal、df_small_unequal、
fp_small、geometry 四步合计约 21 秒；df_wide_big 与 fp_wide 合计约 96 秒；fp_wide_single（五档）与 fp_which（两档）合计约 32 秒；
df_wide_long 约 136 秒；qd 约 1 秒。从冻结副本现拷现编（不设 RERUN_TREE）时编译约 26 秒。
held_wide 第一次用 2G 上限跑，raw 日志停在 big_write 那一行、包装的退出码当时没收（原因没核，不算结果）；改 8G 重跑的才是这里这一份。
