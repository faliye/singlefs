# m3-formal-code-b34-r1 正推腿（Sonnet）模型与脚本

报告：`research/prompts/m3-formal-code-b34-r1-sonnet-output.md`。冻结副本：`/tmp/claude-1000/m3-formal-code-b34-r1/tree/`（sha256 清单同目录 `crates-sha256.txt`）。

复跑：`S=<自己的草稿目录> bash research/prompts/m3-formal-code-b34-r1-sonnet-model/tools/rerun.sh setup build tests rows custom b4 probes patches`
（阶段说明在 `tools/rerun.sh` 文件头；每条 cargo 经 `run-with-memory-cap.sh` 8G 与 `capped.sh 10`；`rows` 与 `patches` 各超过 4 分钟，主 agent 用后台跑）。

| 目录 / 文件 | 内容 |
|---|---|
| `tools/rerun.sh`、`tools/env.sh`、`tools/run-tests.sh`、`tools/build.sh` | 复跑入口与公共环境 |
| `tools/select_rows.py`、`tools/mutate_rows2.py` | 第 34 批 36 行变异：核行与锚点；harness 档 34 行逐行施加、跑、还原，记红在断言还是越界 panic、点名的测试是否红 |
| `tools/custom_mut.py`、`data/custom*.json` | 自拟变异：M1–M4（新根段取错字段）、ALT1（不越界的替代变异） |
| `tools/b4_windows.py` | B4 窗口敏感性（换窗口看 34-3 取样点罩不罩得住） |
| `tools/patch_walk.py` | 临时补丁 q1a / q2 / q4 / rvoff / instrument（跑完还原并核 sha256） |
| `tools/build_probes.py`、`probes/*.rs` | 探针测试（在副本里拼成 `zz_probe_*.rs`，不进仓）：全零树表指针、Q1 两形、I-5.5 两形、合法历史前缀状态、I-5.4 逐状态计数、低于下界的根 |
| `results/` | 本腿那一趟的原样输出：`mut-results3.jsonl`（34 行变异）、`custom*.out`、`b4-windows.log`、`probes.out`、`patches.out`、`tests-summary.txt`、`build1.log` |

探针与补丁都只在副本里；仓里的 `crates/` 一个字没动。
