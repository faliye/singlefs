# scripts

工程反复要用的脚本、它们读的输入表与自证都放在这里，一律从这里跑。规矩在 `.claude/rules/directory-layout.md`。

| 子目录 | 放什么 |
|---|---|
| `agents/` | 派发、盯子 agent、三方材料与引文核对、定时提醒 |
| `crash/` | 崩溃点重放、崩溃放量、崩溃点重放双机分片、GPU 清场 |
| `devices/` | 虚机、QEMU、真块设备上的装置与探针 |
| `experiment-apparatus/` | 实验装置的 crate（`Cargo.toml` 在这一层）与复跑入口 `replay.sh` |
| `gate/` | 提交前的门禁编排、准入判定、补丁合入与合入后验证 |
| `hosts/` | 内存包装（每条上限与 slice 总上限的内核层兜底）、线程上限、第二台那几段（在同一个绝对路径的共享树里起命令，不拷树、不取回）、task-server 载入的预测与余量判法、长活脱离、每台只编一次的 C++ 依赖（RocksDB、snappy）；排队、挑机器、定额度与起任务在仓根 `task-server/`（不在 `scripts/` 下） |
| `kb/` | kb 的定点改写、引文与文献、回扫与变更史整理 |
| `mutation/` | 变异驱动、证红、广谱变异，变异表在 `mutation/tables/` |
| `prune-server/` | 判定库服务 prune-server 的 Python 一侧：被三家判定库载入的客户端、导入旧库、模拟负载（服务本体在仓根 `prune-server/`） |

- 层级：`scripts/` 下最多三层，`scripts/<子目录>/<装置目录>/<文件>`。
- 不放结果：产物、日志、实测表不进这里。实验产物进 `research/results/`；运行记录（峰值表这类）放多机配置 `NFS_SHARED_ROOTS` 点名的共享根下、两台读同一份，不经 `~` 取（`.claude/rules/verification.md`「判定库只有一份」）。
- 两台主机跑同一份脚本，差异只走参数与本地配置 `multi-host.env`，不为第二台另写一份。
