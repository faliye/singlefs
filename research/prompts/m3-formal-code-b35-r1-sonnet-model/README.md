# m3-formal-code-b35-r1 正推腿（Sonnet）模型与脚本

报告：`research/prompts/m3-formal-code-b35-r1-sonnet-output.md`。冻结副本：`/tmp/claude-1000/m3-formal-code-b35-r1/tree/`（sha256 清单同目录 `crates-sha256.txt`）。

复跑：`S=<自己的草稿目录> bash research/prompts/m3-formal-code-b35-r1-sonnet-model/tools/rerun.sh setup build tests probes fl-off rows b4`
（阶段说明在 `tools/rerun.sh` 文件头；每条 cargo 经 `run-with-memory-cap.sh` 8G 与 `capped.sh 10`；`probes` 用 opt-level 2 的另一个编译目录，debug 版逐状态判太慢）。

| 目录 / 文件 | 内容 |
|---|---|
| `tools/rerun.sh`、`tools/env.sh`、`tools/run-tests.sh`、`tools/build.sh`、`tools/rerun_probe.sh` | 复跑入口与公共环境 |
| `tools/select_rows35.py`、`tools/mutate_rows2.py` | 第 35 批 21 行变异：核行（整行只出现一次、与改动全文逐字相同）与锚点；harness 档 13 行逐行施加、跑、还原，记红在断言还是越界 panic、点名的测试是否红 |
| `tools/patch_b35.py` | 把 `mutations.tsv` 里点名的行当探针补丁临时施加 / 还原（用来把 FL 关掉做对照） |
| `tools/b4_windows2.py` | B4 窗口敏感性（换窗口看 35-3 取样点罩不罩得住，另打第一条 panic） |
| `tools/build_probes.py`、`probes/probe_b35_tail*.rs` | 探针测试（在副本里拼成 `zz_probe_b35.rs` = `checker_known_bad_images.rs` 整份 + 各 tail，不进仓）：tail 逐文件内容见各文件头注释 |
| `results/` | 本腿那一趟的原样输出（日志名与 `tools/rerun.sh` 里的 `$S/t-*.log` 一一对应，`t-base-row0.log` 是行 0 那一组的基线）；`mut-results.jsonl` 13 行变异；`b4-windows2.log` |

探针与补丁都只在副本里；仓里的 `crates/` 一个字没动。

补：`probes/probe_b35_tail8.rs` 是对拍用例 4（新池那次发布的根与记录里树表指针全零）的 CPU 一侧，`probe_b35_tail9.rs` 是 I-5.5 那张 extent 树根写零的镜像与干净镜像逐条比；`tools/rerun.sh probes` 都会起。第一次跑 flow4 时内存包装排不上（退出码 252，别的会话占着 slice），之后用 2G 上限与 `RUN_WITH_MEMORY_CAP_WAIT_SECONDS=200` 直接跑编好的探针二进制（`results/t-flow4.log`、`results/t-p4b.log`）。
