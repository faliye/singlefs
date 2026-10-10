# m3-formal-code-b21-r1 云端正推腿的模型与脚本

副本：草稿目录 `/tmp/claude-1000/m3-formal-code-b21-r1-sonnet/work/`（冻结树 `crates/` 的拷贝 + 仓根 `Cargo.toml` / `Cargo.lock`，workspace 去掉 checker-tier 一员）。
副本相对冻结树的改动全文：`副本相对冻结树的改动.patch`（四处探针钩子：`allocator.rs` 回收门槛读环境变量 `B21_SLACK`；`walk.rs` 的 `of_tree_kind` 读 `B21_DROP`，把那一种丢到 NotJudged；
`mount_transparency.rs` 把 `witnessed_counter_holds_against_the_selected_version` 改成 pub；两份测试文件末尾加探针）。另有两个新文件 `b21_scan.rs`、`b21_k3_witness.rs`（本目录）。
仓内文件一个没改。

| 文件 | 用途 | 对应格 |
|---|---|---|
| `b21_scan.rs` | 不同每区槽数 S、环长、只暖机挂载次数 k 上，第一个文件的 txg、数据单元落点（相对 U）、I-5.5 与池级红条 | K1 H2 |
| `driver-h2.sh` | H2 用例在回收门槛放宽 0 / 1 / 2 / 5 个 txg 下各跑一次 | K1 H2 |
| `driver-scan2.sh` | S = 12、16 的扫描 | K1 H2 |
| `driver-k1b.sh` | 28 张坏镜像逐张与干净镜像比，六种「不归 Zero」各丢一次，原用例各跑一次 | K1 六张坏镜像、判完再断言 |
| `b21_k3_witness.rs` | 手写日志世界上直接调真函数，核判据对所选那一版的单调，并与五步纯函数抄本逐点比 | K3 |
| `out/` | 上面各条的原样输出 | |

起法（工作目录是副本，全经包装；线程上限按派发提示）：
`CARGO_TARGET_DIR=<草稿目录>/target bash research/scripts/capped.sh 7 cargo test --offline -p singlefs-harness --test <名> --no-run`，
再 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON=… RUN_WITH_MEMORY_CAP_PARALLEL_REASON=… bash research/scripts/capped.sh 7 bash research/scripts/run-with-memory-cap.sh 4G <测试二进制> …`。
`b21_scan.rs` 吃环境变量 `B21_SLACK`、`B21_S`（逗号分隔）、`B21_J`（`default` 或 `j128`）。

## 复现（副本与编译目录交回前已删）

1. `cp -r <冻结树>/crates <新副本>/`；把仓根 `Cargo.toml`、`Cargo.lock` 拷进去，`Cargo.toml` 的 `members` 里去掉 `crates/singlefs-checker-tier`。
2. 在副本根下 `patch -p0`（按 `副本相对冻结树的改动.patch` 里的路径，先把路径前缀换成副本里的）把五处改动打上；`b21_scan.rs`、`b21_k3_witness.rs` 拷到 `crates/singlefs-harness/tests/`。
3. 各 `driver-*.sh` 里的二进制路径写死了当时的草稿目录，换成自己的；`out/` 是当时的原样输出（`k3.log` sha256 `7f508296abf58ff16eb7d9c422f25d291b999c17aeb3d03f292d9809e70bd11b`）。
4. `check-mutation-old-strings.py <冻结树根>` 复核 mutations.tsv 七行旧串的命中次数（输出在 `out/mutation-old-strings.log`）。
