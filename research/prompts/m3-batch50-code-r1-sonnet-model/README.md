# m3-batch50-code-r1 正推腿（Sonnet）的草稿装置

这里的东西都只在冻结副本的**拷贝**上跑过，冻结副本 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree/` 一个字节没动。报告：`research/prompts/m3-batch50-code-r1-sonnet-output.md`。

- `probe_m3_oracle.rs`：放进拷贝的 `crates/singlefs-core/tests/`。`#[path = "e176_model.rs"]` 引同目录下的 `e176_model.rs`（= `scripts/experiment-apparatus/e176_inode_leaf_merge_capacity_and_rewrites.rs` 原文件，sha256 `a873f91ea844aed2b8cc6aba21405f6ae948e8cf47cbd25e7c8caeb06fcb82b6`，一个字没改）。三条用例：随机对拍 M3 臂（`PROBE_SEEDS`、`PROBE_STEPS` 两个环境变量）、三片各 117 条的定向形、135 片 × 2 条的连锁吸收。
  跑法：`cd <拷贝> && PROBE_SEEDS=40 PROBE_STEPS=9000 bash scripts/hosts/run-with-memory-cap.sh 10G bash scripts/hosts/capped.sh 8 cargo test --release --offline -p singlefs-core --test probe_m3_oracle --target-dir <自己的编译目录> -- --nocapture --test-threads=3`
- `probe_k52.rs`：放进拷贝的 `crates/singlefs-harness/tests/`（需要同目录的 `common/`、`common_admission/`、`common_admission_pool/`）。扫第一个文件的数据单元数 du（`K52_LO`、`K52_HI`、`K52_STEP`），每个 du：写 → 贴边建 inode 到被拒 → unlink 2 号、3 号、1 号，打 `K52 du=…` 行。
- `scripts/variant3.py`（core lib + 探针一起跑）、`variant4.py`（任意 cargo 参数，harness 用）、`variant6.py`（可以同时套几处替换、带 `--nocapture`）、`rows.py`（逐行重放 `logs/mut-unstaged.diff` 里本批的变异行，结果 `logs/rows-results.tsv`）、`cite.sh` / `cite2.sh`（按行号现取一行、打成报告里的引文格式）。脚本里的路径写死了草稿目录 `~/.local/share/singlefs/work/m3-batch50-code-r1-sonnet/`，换地方要改。
- `variants/`：变体的 `.old` / `.new` 替换串（order = 先右后左对调；floor116 / floor118 = 借的下限 ±1；ceilborrow；strict233；ra / rb / rc / rd = 标签 6 的退役常驻三项 / 35 × n_记录 / W 的 6 + H_退役 / ckpt_cost 的退役项；retwidth = I-1.7 对类型 3 不判宽；death = 死亡 txg + 1；opt2inv = Opt2 的滤取反；carrypointer / carrykey = R3 照抄取错叶序）。
- `logs/`：每次运行的原样输出；`mut-unstaged.diff` 是 `git diff -- crates/mutations.tsv`（工作区对暂存区）。

注意：探针用的是拷贝里的 `inode_tree.rs`（与冻结副本逐字节相同，`diff -q` 过）。变体脚本在改完源文件之后 `touch`，还原之后再 `touch`（cargo 按 mtime 判，不 touch 会拿旧变体的编译产物当新的，我自己踩过一次）。
