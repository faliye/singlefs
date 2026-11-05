# scripts/crash

崩溃点重放、崩溃放量、崩溃点重放分片与 GPU 清场。验证包的跑只由主 agent 经 `task-server/task submit -- <命令>` 交（内存上限、机器、显卡与分片都由服务定，命令里不写）；崩溃点重放只有一档，不带参数跑全部项，缩小范围用 `--item <项>`（`--list-items` 列可点的项，`--list-items --with-keys` 另按项打 `ITEM_PLAN` 行与 `ITEM_DOMAIN` 行、可与 `--item` 合用），缩崩溃域用 `--crash-expand-up-to <n>`（缩过域的那一趟结果行写出实际展开、不算门禁通过），跑法与 `--selftest` 照 `.claude/rules/verification.md`。

`--item`（指定范围）没缩域时，开跑之前先估点名的项这一趟要现判多少崩溃状态（只数键对不上的；`ITEM_DOMAIN` 的 `states_to_judge=` 就是这个数），超过 200000 态或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 `--crash-expand-up-to <n>` 缩域再交（说明里写得出能压到上限以内的 n 就写出来，缩过域的那一趟不算门禁通过），或留给提交时不指定范围的那一趟。不带 `--item` 的不指定范围那一趟不管上限。崩溃点重放与崩溃放量两个入口共用这一判，估法、记录来源、上限常量只登记在 `lib_crash_point_replay_items.py` 文件头「指定范围的崩溃域上限」。

| 文件 | 做什么 |
|---|---|
| `crash-point-replay.sh`、`lib_crash_point_replay_items.py` | 崩溃点重放的入口与合表的读法、核法 |
| `crash-amplification.sh`、`crash-amplification-item.sh`、`crash-amplification-two-hosts.sh`、`crash-amplification-lib.sh`、`lib_verdict_library.py` | 崩溃放量：全量入口、一个分项、照 task-server 分片协议两片分着判（计划、各片、合并三步）、共用段、判定库 |
| `crash-point-replay-shard-run.sh`、`crash-point-replay-shard-configuration-check.sh` | 崩溃枚举用例照 task-server 分片协议两片分着判（经 `task-server/task submit` 单独交，计划、各片、合并三步交给崩溃放量的分片驱动；门禁不在一件任务里面调它；自证在 `--selftest`）、多机配置的判法 |
| `gpu-quiesce-run.sh`、`gpu-exemptions-check.py` | 用 GPU 之前清场并回读（task-server 给 GPU 任务分卡、经它清场再起）；不走 GPU 的计算的豁免登记核对 |
| `cpu-judging-sites-check.py` | 判器调用点（池级 checker、记录核对器、RefFS 允许集合比对与登记的判器本体）要么在判定段里、要么带豁免或 CPU 参照的标记（门禁 checker-independence-and-sync 的 cpu-judging-sites 格调它） |
| `crash-flow-coverage.py` | 崩溃流覆盖：从崩溃点重放的日志读覆盖行与计数行，出覆盖表（入口 × StepKind）、ignore 开着的节点、评估 0 次的（流，不变量）对与上一次全量的标记；门禁 crash-point-replay 的出口 4 那四格调它判，入口每趟拿它落覆盖表 |
| `crash-case-check.py`、`check-segment-registry.py`、`check-segment-registry-product-sample.txt` | 测试源码与崩溃点测试登记表的静态判据（含每条崩溃枚举用例的必红变异、旋钮登记表；调了枚举一族函数的用例都登记成 `crash-case:`，确实不是崩溃用例的在测试函数上面写注释 `crash-case-check:not-a-crash-case <理由>`）；段序列登记表与钉住活代码的用例的逐字核对（钉不到用例的行才与 E142 产物比，产物不在报本次未跑），样本是它自证测产物行解析用的摘抄 |
