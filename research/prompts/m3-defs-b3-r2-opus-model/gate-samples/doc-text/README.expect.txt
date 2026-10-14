攻方腿推的结局（没跑：门禁格子 agent 不起，要主 agent 跑 run-gate-samples.sh）：
pv-big-untracked         退 0，「✓ 查了这次改动里 1 份文本文件新加的 … 行 … 没有一处命中」（第 8 MiB 之后那一行没读到）
pv-filename-only         退 0，「✓ … 没有一处命中」（文件名不判）
pv-filename-and-content  退 1，红行「research/results/e999-<假主机名>.out:1  键名：PEER_SSH_HOST、QUIESCE_STOP_COMMAND（值不打印）」——值经文件名打了出来
pv-nul-head              退 0，「✓ 查了这次改动里 1 份文本文件新加的 1 行 …」「二进制 1 份」（带 NUL 的日志整份跳过，里面的假本机主机名没判）
