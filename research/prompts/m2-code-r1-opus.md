你是发布 B（覆盖写 + 释放）那批代码三方对抗第一轮的**攻方腿**。攻背景材料第三节的 X1（释放的口径）、X2（记账口径）、X4（层 0 多版本 oracle 的假阴性）、X5（对照够不够）。目标是一段按条款走不通的历史、一个 oracle 判绿而实际错了的状态、或一条改坏了没有断言红的变异。

## 先读

1. 背景材料：`research/prompts/_m2-code-r1-background.md`，附录二 `research/prompts/_m2-code-r1-diff.md`。
2. 自己去读代码（原仓只读）：`crates/singlefs-core/src/allocator.rs`、`transaction.rs`、`recovery.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-checker/src/walk.rs`。
3. 要改代码跑用例，把仓拷一份到 `/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/m2-code-r1-opus-copy/`（`rsync -a --exclude target --exclude .git`），在副本上改、在副本上 cargo test；副本上的数照实报、注明是副本。

## 要攻的

- X1：写一段历史（发布序列 + 每次发布的 txg）让已释放的槽被分配器重新发出去，或让该释放的没释放；核 `release` 的两盘同槽假设与 `placements` 的跨度推法。
- X2：S2 把已释放的算在已分配里——与 D28 已定项 1 的式子逐字对，哪一边扣了两次；构造一个 I-3.1 或 I-5.2 会红的状态（复用之后、回退之后、A 的根被轮转覆写之后）。
- X4：在 `evaluate_state_for_versions` 上找假阴性：某个持久集合让恢复读出错的东西而 oracle 判绿；核 `newest_persisted_root_txg` 与 `root_persisted_states` 的口径。
- X5：逐条给新测试的断言找它抓不到的变异（释放代写错、映射条目没删、树表诞生 txg 改了、超级块世代号、反向链……），在副本上改坏跑一遍，报「红在哪条 / 一条都不红」。

## 交付

- **分段写**：报告写进 `research/prompts/m2-code-r1-opus-output.md`，每一次写文件严格不超过 150 行；第一段用排他方式新建（`set -o noclobber` 后 `cat > 文件 <<'EOF'`），之后用 `>>` 追加。
- 引条款整行抄、行号写 kb 文件的；引代码写路径与行号；副本上的改动与数照实报。不许用「本条」「本节」「上文」这类指代。原仓不许改任何已有文件、不许 git 的任何写操作。请在大约 45 分钟内交出报告（剧本与变异结果优先）。最后回复只写一句指向报告。
