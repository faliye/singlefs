# C794 盘上隔离位 第一轮 正推腿模型目录

报告：`research/prompts/c794-bit-r1-sonnet-output.md`。

- `src/probe-instrumentation.diff`：对冻结副本五个源文件加的探针（环境变量开关；默认行为不变）。
- `src/tests/`：探针测试。`forward_probe_q1.tail.rs` 与 `forward_probe_rollback.tail.rs` 是接在冻结副本里两份现有测试文件后面的探针，`forward_probe_q1b.rs` 是独立文件；`tools/rerun.sh` 负责拼起来。
- `tools/rerun.sh`：整条经 `scripts/hosts/run-with-memory-cap.sh` 跑（用法见报告「要主 agent 跑的命令」）；`RERUN_STEPS` 选步骤。
- `out/*.txt`：每个步骤的结论行（已去掉耗时字段与并行输出里次序不稳的行），复跑逐行对这里。
