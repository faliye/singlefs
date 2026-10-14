# scripts/crash

崩溃点重放、崩溃放量、崩溃点重放双机分片与 GPU 清场。checker 档的重型，只由主 agent 带 `SINGLEFS_HEAVY_TESTS` 前缀跑；轻型与 `--selftest` 照 `.claude/rules/verification.md`「两档：轻型与重型」。

| 文件 | 做什么 |
|---|---|
| `crash-point-replay.sh`、`lib_crash_point_replay_items.py` | 崩溃点重放的入口与合表的读法、核法 |
| `crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-two-hosts.sh`、`crash-amplification-lib.sh`、`lib_verdict_library.py` | 崩溃放量：全量入口、一个分项、双机跑法、共用段、判定库 |
| `crash-point-replay-shard-run.sh`、`crash-point-replay-shard-configuration-check.sh` | 崩溃枚举用例的双机跑法（把门禁交的一条用例交给崩溃放量的双机驱动，导入那一趟的日志交回门禁；自证在 `--selftest`）、多机配置的判法 |
| `gpu-quiesce-run.sh`、`gpu-exemptions-check.py` | 用 GPU 之前清场并回读；不走 GPU 的计算的豁免登记核对 |
| `cpu-judging-sites-check.py` | 判器调用点（池级 checker、记录核对器、RefFS 允许集合比对与登记的判器本体）要么在判定段里、要么带豁免或 CPU 参照的标记（门禁 checker-independence-and-sync 的 cpu-judging-sites 格调它） |
| `crash-case-check.py`、`check-segment-registry.py` | 测试源码的静态判据；段序列登记表与 E142 产物的逐字核对 |
