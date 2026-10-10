# 第 23 批代码轮第一轮 云端正推的模型与脚本

只在副本上量；仓内文件一个没改。副本的做法：把冻结副本 `/tmp/claude-1000/m3-formal-code-b23-r1/tree/crates` 连同仓根的 `Cargo.toml`、`Cargo.lock` 拷到一个草稿目录（下称 WS），把 `probes/*.rs` 拷进 `WS/crates/singlefs-harness/tests/`。

| 文件 | 干什么 | 起法（经内存包装，线程上限照派发） |
|---|---|---|
| `probes/b23_k3a_probe.rs` | K1：K3-a 用例的态在 harness 档复现，判官一层对两种接线的结论 | `cargo test -p singlefs-harness --test b23_k3a_probe -- --nocapture` |
| `probes/b23_segment_lengths_probe.rs` | K1：随机历史写表切段后的段长分布（10 个种子 × 2 档盘宽 × 16 步） | 同上换 `--test b23_segment_lengths_probe`，约 95 秒 |
| `probes/b23_k3_comment_probe.rs` | K3：盘 0 两槽撕裂的盘面上 c_见证 与所选那一版末条计数器 | 同上换 `--test b23_k3_comment_probe` |
| `probes/b23_h2_txg_probe.rs` | K3：只暖机挂载 k 次之后第一个文件版本的 txg（k 取 1、2、3、5、8、9、10、11、12） | 同上换 `--test b23_h2_txg_probe`，约 165 秒 |
| `f3c_identity.rs` | K1：F3-c 抽函数前后种子公式逐位比较 | `rustc --edition 2021 -O` 编，再经包装跑 |
| `f3c_weak_fingerprints.py` | K1：4095 断言对各种弱指纹的判别（纯算术） | `python3 f3c_weak_fingerprints.py` |
| `h2_sweep.sh` | K3：改副本里的 S（与放宽一个 txg 的变异）跑 H2 两条用例 | 环境变量 `WS`、`FROZEN`、`TARGET`，参数 `<S> <orig\|slack1>` |
| `h2_premise_mutants.sh` | K3：挂载次数改错一次，看红在前提断言还是落点断言 | 同上，参数 `<S> <ten-minus-one\|nine-plus-one>` |
| `mutation_anchors.py` | K1：第 2407–2413 行变异的锚点命中数（只读） | `python3 mutation_anchors.py <冻结副本 tree 根>` |
| `out/` | 上面各项这一轮真跑的原样输出 | — |

`probes/b23_h2_txg_probe.rs` 文件头注释写 k = 1..=22，现行代码里 k 取的是上表那几个值（22 个全跑要十几分钟）。
`out/h2-sweep.log` 里 S = 4、16 两节是编译期断言拒绝（`root_ring.rs` 的 `at_make_filesystem_checked_at_compile_time`），不是用例红。
