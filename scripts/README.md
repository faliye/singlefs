# scripts

工程反复要用的脚本、它们读的输入表与自证都放在这里，一律从这里跑。规矩在 `.claude/rules/directory-layout.md`。

| 子目录 | 放什么 |
|---|---|
| `agents/` | 派发、盯子 agent、三方材料与引文核对、本地模型网关 |
| `crash/` | 崩溃点重放、崩溃放量、崩溃点重放双机分片、GPU 清场 |
| `devices/` | 虚机、QEMU、真块设备上的装置与探针 |
| `experiments/` | 实验装置的 crate（`Cargo.toml` 在这一层）与复跑入口 `replay.sh` |
| `gate/` | 提交前的门禁编排、准入判定、补丁合入与合入后验证 |
| `hosts/` | 内存包装、线程上限、双机派发与资源测量、长活脱离 |
| `kb/` | kb 的定点改写、引文与文献、回扫与变更史整理 |
| `mutation/` | 变异驱动、证红、广谱变异，变异表在 `mutation/tables/` |

- 层级：`scripts/` 下最多三层，`scripts/<子目录>/<装置目录>/<文件>`。
- 不放结果：产物、日志、实测表不进这里。实验产物进 `research/results/`；只在本机成立的运行状态（内存峰值表这类）放 `~/.local/share/singlefs/`。
- 两台主机跑同一份脚本，差异只走参数与本地配置 `multi-host.env`，不为第二台另写一份。
