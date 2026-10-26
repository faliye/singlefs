# scripts/crash

崩溃点重放、崩溃放量、崩溃点重放双机分片与 GPU 清场。checker 档的重型，只由主 agent 带 `SINGLEFS_HEAVY_TESTS` 前缀跑；轻型与 `--selftest` 照 `.claude/rules/verification.md`「两档：轻型与重型」。

| 文件 | 做什么 |
|---|---|
| `crash-point-replay.sh`、`lib_crash_point_replay_items.py` | 崩溃点重放的入口与合表的读法、核法 |
| `crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-two-hosts.sh`、`crash-amplification-lib.sh`、`lib_verdict_library.py` | 崩溃放量：全量入口、一个分项、双机跑法、共用段、判定库 |
| `crash-point-replay-shard-run.sh`、`crash-point-replay-shard-configuration-check.sh` | 崩溃枚举用例的双机跑法（把门禁交的一条用例交给崩溃放量的双机驱动，导入那一趟的日志交回门禁；自证在 `--selftest`）、多机配置的判法 |
| `gpu-quiesce-run.sh`、`gpu-exemptions-check.py` | 用 GPU 之前清场并回读（命令起过的那一趟出去时发一条资源释放广播 gpu-released，`scripts/hosts/resource_release_broadcast.py`）；不走 GPU 的计算的豁免登记核对 |
| `cpu-judging-sites-check.py` | 判器调用点（池级 checker、记录核对器、RefFS 允许集合比对与登记的判器本体）要么在判定段里、要么带豁免或 CPU 参照的标记（门禁 checker-independence-and-sync 的 cpu-judging-sites 格调它） |
| `crash-flow-coverage.py` | 崩溃流覆盖：从崩溃点重放的日志读覆盖行与计数行，出覆盖表（入口 × StepKind）、ignore 开着的节点、评估 0 次的（流，不变量）对与上一次全量的标记；门禁 checker-tier-crash-point-replay 的出口 4 那四格调它判，入口每趟拿它落覆盖表 |
| `crash-case-check.py`、`check-segment-registry.py`、`check-segment-registry-product-sample.txt` | 测试源码与崩溃点测试登记表的静态判据（含每条崩溃枚举用例的必红变异、旋钮登记表）；段序列登记表与钉住活代码的用例的逐字核对（钉不到用例的行才与 E142 产物比，产物不在报本次未跑），样本是它自证测产物行解析用的摘抄 |
