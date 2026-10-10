# m3-formal-code-b36-r1 正推腿（Sonnet）模型与脚本

报告：`research/prompts/m3-formal-code-b36-r1-sonnet-output.md`。冻结副本：`~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/`（sha256 清单同目录 `crates-sha256.txt`，316 行）。

复跑：`S=<自己的草稿目录> bash research/prompts/m3-formal-code-b36-r1-sonnet-model/tools/rerun.sh <阶段…>`，阶段与次序见 `tools/rerun.sh` 文件头：
`setup build tests probes-build p1-a p1-b p1-c probes-small probes-r2a probes-r2b r1-off rows wgsl`。
每条 cargo 经 `run-with-memory-cap.sh` 8G 与 `capped.sh 16`；`wgsl` 阶段的 naga 二进制整条校验脚本经内存包装 4G；同一时刻只起一条。

| 目录 / 文件 | 内容 |
|---|---|
| `tools/rerun.sh`、`tools/env.sh`、`tools/run-tests.sh`、`tools/build.sh`、`tools/rerun_probe.sh` | 复跑入口与公共环境 |
| `tools/build_probes.py`、`probes/probe_b36_tail*.rs` | 探针（在副本里拼成 `zz_probe_b36.rs` = `checker_known_bad_images.rs` 整份 + 各 tail，不进仓）：`tail.rs` P1 矩阵、H2 / H3；`tail5.rs` H3 第一条记录带旧根；`tail_a.rs` P1b 网格、P1c、P1d；`tail_r3.rs` I-5.5 理由文字；`tail_r2.rs` R2 新旧造法对照 |
| `tools/summarize_p1.py`、`tools/summarize_p1b.py` | P1 矩阵与 P1b 网格的交叉表 |
| `tools/select_rows36.py`、`tools/mutate_rows2.py`、`tools/patch_b36.py` | 第 36 批 8 行变异：核行与锚点；harness 档 4 行逐行施加、跑、还原；R1 那一行临时施加（关掉 R1 对照）与还原 |
| `tools/wgsl_validate.py`、`naga-check/` | 两个池级 checker 内核照 `crash_judge_gpu.rs` 的拼法交 naga（30.0.1，`=` 钉版本、仓的 `Cargo.lock`）解析与校验；GPU 四行变异施加后再校验；不跑 GPU |
| `results/` | 本腿那一趟的原样输出：`t-p1-*.log` 矩阵（`t-p1-all.log` 是三趟合并）、`t-p1bgrid.log`、`t-p1c.log`、`t-p1d.log`、`t-r3.log`、`t-h2-*.log` / `t-h3-*.log` / `t-h3s-*.log`（on = R1 开，off = 关）、`t-p1-r1-off.log`、`t-r2*.log`、`t-base-rows.log`、`mut-results.jsonl`、`wgsl-validate.log` |

探针与变异都只在副本里；仓里的 `crates/` 一个字没动。`results/` 里的逐状态签名行（`SIG …`）带探针里的 SipHash 固定键哈希，两次跑可比。
