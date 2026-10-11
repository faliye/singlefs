# m3-root-ring-code-r2 正推腿（Sonnet）的模型与探针

报告：`research/prompts/m3-root-ring-code-r2-sonnet-output.md`。这里的东西只用来复核报告里的数，不进 `crates/`。

## 怎么复跑

- `e2_window_lemma.py`：K1 引理的逐点枚举，纯 Python，`nice -n 19 python3 e2_window_lemma.py`，输出 `checked=17399790 violations=0`（`logs/e2_window_lemma.out`）。
- `tests/`：三份探针测试（harness 档）。拷进一份冻结副本（`/tmp/claude-1000/m3-root-ring-code-r2/tree/`，在你自己的草稿目录里再拷一份）的 `crates/singlefs-harness/tests/` 下，加 `.cargo/config.toml`，再 `cargo test -p singlefs-harness --test <名字> -- --nocapture`（经 `capped.sh` 与内存包装）：
  - `probe_k1_raise_to_the_ceiling_over_an_unreadable_x.rs`：准入抬 F 走「抬到上限」入口，X 那一槽读不出与读得出各一次（报告 1.5）。
  - `probe_k3_i35_on_legit_hole_histories.rs`：写行崩溃留洞的合法历史 330 张镜像逐版看 I-3.5（报告 2.5）。
  - `probe_k3_i35_rollback_after_a_hole.rs`：洞之后回退再发布 84 张镜像（报告 2.5）。
- `probes/*.old`、`*.new`：变异探针的旧串与新串（`scripts/patch2.py apply <相对文件> <旧串文件> <新串文件>` 在副本里就地改，`restore` 还原并 `touch`，别漏掉 `touch`：cargo 按 mtime 认产物）。名字对照：p1/p1b/p2/p3/p4 改 `mount.rs` 的 `reread_of_a_root_ring_slot_unreadable_at_mount`（报告 1.7）；q1/q2/q3 把三处记根点换成不重读（1.7）；ba–bg 改 `walk.rs` 的洞判法与引用不全跳过（2.2）；bh 读不出头当 0（2.1）；gap 去掉缺口（2.5）；tf 去掉 TF (b)（2.5）；f 去掉 I-8.10 的 F 一对（3.1）；j1–j4 改 `pointer.rs` 的 `judged()`（4.1）；dbg 只加打印（5.1）。
- `scripts/prove.sh <crate> <mutations.tsv 行号…>`：按行号取变异名交 `scripts/mutation/prove-red.sh --copy <副本>`；`scripts/runt3.sh <整条命令>` 在副本里经 `capped.sh 15` 与 6G 内存包装跑；`scripts/e2-targets.txt` 是 E2 相关的 17 个 harness 二进制。脚本里的副本路径写死在 `/tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree/`，换目录要改。
- `logs/`：每一条报告里引的原样输出；`prove-*.out` 是 prove-red 的逐行结果。
