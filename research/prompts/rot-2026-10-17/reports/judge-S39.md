# 知识腐烂治理第二轮逐行判：S39（183 组）

草稿目录：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-S39`（认领过；里面只有生成报告用的 `gen.py`）。没建副本、没起编译、没跑重型测试。

## 取组与现查的命令

- 取组：`awk -F'\t' '$1=="S39"{print $2}' segments2.tsv`，183 个组：P3 84、P4 57、P5 12、P7 6、P9 24（`cut -c1-2 | sort | uniq -c` 数出）。
- P3：每个名字 `git grep -F -l -- <名> -- ':!records' ':!.claude/kb'` 与 `-- research/results research/*.* litmus Cargo.toml`、`grep -rlF -- <名> research/results` 各数一遍；再读载体行。命中在 `research/results/`（装置源码 `device-source.rs`、产物 `.out`）的，是产物字段 / 装置里的名字，判不相干；零命中的按句子是不是把它当现有代码写来判。
- P4：`sed -n '581,$p' checks-owed.md` 取「### 已还清」表，对 40 个 C 号逐个 `grep -c "^| C<n> "` 已还清表 1、开着的表 0，全部确是已还清；再逐处读那个 C 号前后各 200 字，看句子是不是在说它还开着。
- P5：`grep -h "^| I-x.y " crates/singlefs-checker/invariants/*.md` 取状态列；`IMPLEMENTED_INVARIANTS`（image.rs:112 起，92 条）核 I-2.2、I-7.6；决策分项读 `.claude/kb/decisions/22-*.md` 表格行「进度」。
- P7：`grep -n 'missing-commands\|outer-memory-cap-to-driver\|file-names\|harness-test-granularity\|status-sync'` 在 `scripts/crash/`、`.claude/gate.d/` 里找；`ls .claude/gate.d/*.sh | wc -l` 17。
- P9：读 `.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 8 条「带日期的现状快照也是历史」，逐行删日期读；能现查的（crate 数、`make_filesystem` 的 `write_zeroes_at`、`strip_timing`、模块位置、函数名）都现查。
- 被钩子拒过一次：`cargo mutants --version`（重型测试闸），没有改写法绕过，所以 02-second-txn.md:427 的工具版本没复核（见下）。

## 判定的口径（写在前面，免得每行重复）

- P3 的 84 行里 81 行判不相干（63）/ 事件句不改（18）：名字是产物字段（E165、E166、E172、E175、E178 的 .out 与 `device-source.rs`）、别家实现（Linux 内核、jbd2、QEMU、vLLM）、或句子自己写着「已删」「改指，原 X」「随之改名」「当时叫」。判要改的只有 3 行：49-反向链宽度32还是64.md:203、02-second-txn.md:198、:485。
- P4 的 57 行里 3 行判要改（03-空间分配.md:239、02-second-txn.md:62、:359），不相干 28、事件句不改 26：引 C 号作题名 / 依据 / 事件，或同一格已写「已还清」。
- 同一载体行命中几次，每组各写一行相同判定。
- 「要改」的最后一格写改后的整句（或改法，P5 一行同时指出源头行）。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| P30170 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:490 | 不相干 | 句子说的是 .out 产物里的行标签（「各 1 行加注释头」「`pq1_changed` 342 行」「`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑」），是产物里的行标签 / 字段名（产物已归档，git log 里可查，或仍在 research/results/），不是现有代码名；pq1_changed、gate_a_rerun 在 research/results/e165-...-r5-segment1-comparison.out 等里现数命中，不是已删代码 |
| P30171 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:490 | 不相干 | 句子说的是 .out 产物里的行标签（「各 1 行加注释头」「`pq1_changed` 342 行」「`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑」），是产物里的行标签 / 字段名（产物已归档，git log 里可查，或仍在 research/results/），不是现有代码名；pq1_changed、gate_a_rerun 在 research/results/e165-...-r5-segment1-comparison.out 等里现数命中，不是已删代码 |
| P30172 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:490 | 不相干 | 句子说的是 .out 产物里的行标签（「各 1 行加注释头」「`pq1_changed` 342 行」「`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑」），是产物里的行标签 / 字段名（产物已归档，git log 里可查，或仍在 research/results/），不是现有代码名；pq1_changed、gate_a_rerun 在 research/results/e165-...-r5-segment1-comparison.out 等里现数命中，不是已删代码 |
| P30173 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:989 | 不相干 | 句子说的是 .out 产物里的行标签（「各 1 行加注释头」「`pq1_changed` 342 行」「`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑」），是产物里的行标签 / 字段名（产物已归档，git log 里可查，或仍在 research/results/），不是现有代码名；pq1_changed、gate_a_rerun 在 research/results/e165-...-r5-segment1-comparison.out 等里现数命中，不是已删代码 |
| P30174 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1115 | 不相干 | 句子说的是 .out 产物里的行标签（「各 1 行加注释头」「`pq1_changed` 342 行」「`gate_a_rerun` 是第 5 次第一段在同一份根上取甲的重跑」），是产物里的行标签 / 字段名（产物已归档，git log 里可查，或仍在 research/results/），不是现有代码名；pq1_changed、gate_a_rerun 在 research/results/e165-...-r5-segment1-comparison.out 等里现数命中，不是已删代码 |
| P30175 | .claude/kb/experiments/166-并行提交的共享状态分法.md:194 | 不相干 | `T_max_single` 是 E166 产物 .out 的字段名（`research/results/e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out` 命中 432 次），句子说「`cell` 16 行的 `T_total`、`T_max_single`」是产物字段，不是代码名 |
| P30176 | .claude/kb/experiments/170-O3规约执行器第一版.md:769 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30177 | .claude/kb/experiments/170-O3规约执行器第一版.md:769 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30178 | .claude/kb/experiments/170-O3规约执行器第一版.md:769 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30179 | .claude/kb/experiments/170-O3规约执行器第一版.md:769 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30180 | .claude/kb/experiments/170-O3规约执行器第一版.md:795 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30181 | .claude/kb/experiments/170-O3规约执行器第一版.md:795 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30182 | .claude/kb/experiments/170-O3规约执行器第一版.md:795 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30183 | .claude/kb/experiments/170-O3规约执行器第一版.md:795 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30184 | .claude/kb/experiments/170-O3规约执行器第一版.md:802 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30185 | .claude/kb/experiments/170-O3规约执行器第一版.md:805 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30186 | .claude/kb/experiments/170-O3规约执行器第一版.md:808 | 事件句不改 | 说的是第 2 次重跑收尾时驱动怎么「跟着换了」「改指，原 `driver_e170_r1_s3_*`」，是那一次换名的记录；旧名登记在 `.claude/data-name-lint-exclude`（「kb 表格逐字引它，改名要连产物一起重跑」），现名 `driver_e170_r2_*` 在 scripts/experiments/replay.sh 里现存 |
| P30187 | .claude/kb/experiments/170-O3规约执行器第一版.md:811 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30188 | .claude/kb/experiments/170-O3规约执行器第一版.md:811 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30189 | .claude/kb/experiments/170-O3规约执行器第一版.md:811 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30190 | .claude/kb/experiments/170-O3规约执行器第一版.md:811 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30191 | .claude/kb/experiments/170-O3规约执行器第一版.md:811 | 不相干 | 这一句自己写着这几个驱动「已从 `replay.sh` 删掉」「从 `replay.sh` 删掉，历史在 git 里」，没有把它们当现有代码写；`git grep` 在 crates/scripts/research 零命中与句子一致 |
| P30192 | .claude/kb/experiments/171-三oracle合谋测试.md:1064 | 不相干 | 句子写「第一段的 `driver_e171_s1_comparator` 已由 s2 那一份取代、不再登记」，把它当已取代的旧驱动写，不是现有代码；crates/scripts 零命中与句子一致 |
| P30193 | .claude/kb/experiments/172-删除写出的字节的干跑.md:132 | 不相干 | `mount_instance_row_txg8` 是 E172 产物 .out 里的段名（`research/results/e172-deletion-write-dry-run-2026-10-05.out` 命中），句子说「段是 `mount_instance_row_txg8`」描述产物，不是代码名 |
| P30194 | .claude/kb/experiments/174-借据丢掉时等待与显存回还.md:315 | 不相干 | `judged_us` 是句子里自己定义的分析量（「超出剩下的活的部分（`judged_us`）」），出自 E174 的产物栏名，不是代码标识符；说的是 G2-乙-等待的取数口径 |
| P30195 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:15 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30196 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:27 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30197 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:27 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30198 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:28 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30199 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:57 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30200 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:79 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30201 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:79 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30202 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:79 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30203 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:79 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30204 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:118 | 不相干 | `code_baseline` 是 E175 结果文件头与汇总行的字段（句子写「两份汇总去掉 `code_baseline` 后比」），装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`git grep` 命中 64 个文件），不是 crates/ 里的代码名 |
| P30205 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:154 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30206 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:154 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30207 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:154 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30208 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:162 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30209 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:163 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30210 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:163 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30211 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:166 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30212 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:166 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30213 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:174 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30214 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:176 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30215 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:176 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30216 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:176 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30217 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:182 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30218 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:202 | 不相干 | 这是 E175 装置的测试名 / 结果字段名，装置源码与产物在 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/`（`device-source.rs`、`summary-segment-*.out`、`d-*.out`，`git grep` 命中），不在 crates/ 里是因为装置住在产物目录，句子没有说它在 crates/ 里 |
| P30219 | .claude/kb/experiments/178-叶容器与记录死亡的计数.md:343 | 不相干 | `o2_smallest`、`o2_largest`、`share_ppm` 是 E178 产物 .out 的栏名（`research/results/e178-leaf-and-record-death-counts-segment-three.out`、`...-segment-four.out` 命中），句子描述产物里的量，不是代码名 |
| P30220 | .claude/kb/experiments/178-叶容器与记录死亡的计数.md:343 | 不相干 | `o2_smallest`、`o2_largest`、`share_ppm` 是 E178 产物 .out 的栏名（`research/results/e178-leaf-and-record-death-counts-segment-three.out`、`...-segment-four.out` 命中），句子描述产物里的量，不是代码名 |
| P30221 | .claude/kb/experiments/178-叶容器与记录死亡的计数.md:439 | 不相干 | `o2_smallest`、`o2_largest`、`share_ppm` 是 E178 产物 .out 的栏名（`research/results/e178-leaf-and-record-death-counts-segment-three.out`、`...-segment-four.out` 命中），句子描述产物里的量，不是代码名 |
| P30222 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:117 | 不相干 | `xlog_state_switch_iclogs`、`jbd2_journal_recover` 是 Linux 内核（`xfs_log.c`、`fs/jbd2/recovery.c`）的函数，句子是「现查 jbd2（本机 7.2.0 树…）」的别家实现，不是本仓代码 |
| P30223 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:133 | 不相干 | `xlog_state_switch_iclogs`、`jbd2_journal_recover` 是 Linux 内核（`xfs_log.c`、`fs/jbd2/recovery.c`）的函数，句子是「现查 jbd2（本机 7.2.0 树…）」的别家实现，不是本仓代码 |
| P30224 | .claude/kb/experiments/36-槽位映射那一维.md:78 | 不相干 | `next_commit_ID` 是 jbd2 的概念，句子说「**没有建模 jbd2 的真实恢复算法**（PASS_SCAN 跟链、`next_commit_ID` 的推进）」，是别家实现 |
| P30225 | .claude/kb/experiments/41-根环槽几何2x4x256.md:67 | 不相干 | `blk_stack_limits` 是 Linux `block/blk-settings.c` 的函数，句子是「现查 `block/blk-settings.c`」，别家实现 |
| P30226 | .claude/kb/experiments/43-扩展点字节上限.md:155 | 不相干 | 句子写「当时叫 `JOURNAL_HDR`，现名 `JOURNAL_HEADER_BYTES`」，已经给出现名（`git grep JOURNAL_HEADER_BYTES` 命中 crates/singlefs-format/src/lib.rs） |
| P30227 | .claude/kb/experiments/43-扩展点字节上限.md:199 | 事件句不改 | 说的是 2026-08-31 重跑里「单测随之改名 `the_self_witness_bound_is_…_not_864`」，是那一次改名的记录；今天该单测叫 `the_self_witness_bound_is_25_not_864`（scripts/experiments/e43_extension_point_budget.rs:639），把旧名换成新名这句话就不再是当时发生的事 |
| P30228 | .claude/kb/experiments/43-扩展点字节上限.md:216 | 事件句不改 | 说的是 2026-08-31 重跑里「单测随之改名 `the_self_witness_bound_is_…_not_864`」，是那一次改名的记录；今天该单测叫 `the_self_witness_bound_is_25_not_864`（scripts/experiments/e43_extension_point_budget.rs:639），把旧名换成新名这句话就不再是当时发生的事 |
| P30229 | .claude/kb/experiments/43-扩展点字节上限.md:231 | 事件句不改 | 说的是 2026-08-31 重跑里「单测随之改名 `the_self_witness_bound_is_…_not_864`」，是那一次改名的记录；今天该单测叫 `the_self_witness_bound_is_25_not_864`（scripts/experiments/e43_extension_point_budget.rs:639），把旧名换成新名这句话就不再是当时发生的事 |
| P30230 | .claude/kb/experiments/49-反向链宽度32还是64.md:203 | 要改 | 今天的单测叫 `fixed_record_reading_diverges_only_at_these_bases_in_440_to_520`（scripts/experiments/e49_chain_width.rs:642，范围随记录头长改到 base 440..=520），`..._in_280_to_345` 零命中。改后的句子：…所以同判「免费」（当时的单测 `fixed_record_reading_diverges_only_at_these_bases_in_280_to_345`，记录头改成 487 字节后改名为 `fixed_record_reading_diverges_only_at_these_bases_in_440_to_520`，范围随 base 移到 440..=520）。 |
| P30231 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:107 | 不相干 | `device_del` 是 QEMU 的热拔盘命令，句子写「真正的设备消失（`device_del`）也没做」，说的是实验没做这件事，不是本仓代码名 |
| P30232 | .claude/kb/experiments/95-老化的节点布局臂.md:118 | 不相干 | `fmt_group_gN` 是产物里 `fmt_group_g{2,8,32}` 三条臂名的通称（scripts/experiments/e127_group_identity_under_split_merge.rs:19 写作 `fmt_group_g{2,8,32}`，产物 e95-node-layout-arms-2026-09-09.out），不是本仓现有标识符 |
| P30233 | .claude/kb/experiments/95-老化的节点布局臂.md:132 | 不相干 | `fmt_group_gN` 是产物里 `fmt_group_g{2,8,32}` 三条臂名的通称（scripts/experiments/e127_group_identity_under_split_merge.rs:19 写作 `fmt_group_g{2,8,32}`，产物 e95-node-layout-arms-2026-09-09.out），不是本仓现有标识符 |
| P30234 | .claude/kb/milestone/02-second-txn.md:145 | 事件句不改 | 这一段开头已带「（2026-10-11 注：这一段点名的四个 `MountError` 成员今天都不在 `crates/` 里…今天的拒绝形态以 `mount_writable` 的代码为准）」，`FormattedPoolMountNotShapedLikeTheNewPoolFileCreation` 是 2026-09-16 落地那一版的拒绝成员，句子是那一天的现状记录且已自带「今天不在」的注 |
| P30235 | .claude/kb/milestone/02-second-txn.md:198 | 要改 | `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 与用例 `raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking` 在 crates/ 里零命中（抬 F 现在是 `mount::raise_rollback_floor` 一族），这一句把它们当现状写。改后的句子：没做过可写挂载的进程抬 F 被拒绝而不 panic（当时的错误成员 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 与用例 `raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking` 今天都不在 `crates/` 里，以 `mount::raise_rollback_floor` 的代码为准）。 |
| P30236 | .claude/kb/milestone/02-second-txn.md:365 | 事件句不改 | 说的是「2026-09-22 逐笔重核可达性：两笔已可达，各做成一条会红的用例」，是那一天做了什么；两条用例后来改了名（今天同文件里叫 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable` 与 `the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot`），旧名是当时的名字 |
| P30237 | .claude/kb/milestone/02-second-txn.md:365 | 事件句不改 | 说的是「2026-09-22 逐笔重核可达性：两笔已可达，各做成一条会红的用例」，是那一天做了什么；两条用例后来改了名（今天同文件里叫 `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable` 与 `the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot`），旧名是当时的名字 |
| P30238 | .claude/kb/milestone/02-second-txn.md:374 | 事件句不改 | 说的是「2026-09-18 用户定案…同日实现（`PublishShape::EMPTY_PUBLISH`、`publish_sequence_admission`、`MountError::WarmUpAdmissionRefusedBeforeAcquisition`…）」，是那一天实现了什么；`publish_sequence_admission` 今天零命中，`WarmUpAdmissionRefusedBeforeAcquisition` 仍在 checker-tier 里命中 |
| P30239 | .claude/kb/milestone/02-second-txn.md:379 | 不相干 | 句子自己写「`STILL_UNDECIDED_TODAY` 那两个成员已删、拒绝分支已拿掉（`grep -rn STILL_UNDECIDED_TODAY crates/` 零命中）」，没有把它们当现有代码 |
| P30240 | .claude/kb/milestone/02-second-txn.md:379 | 不相干 | 句子自己写「`STILL_UNDECIDED_TODAY` 那两个成员已删、拒绝分支已拿掉（`grep -rn STILL_UNDECIDED_TODAY crates/` 零命中）」，没有把它们当现有代码 |
| P30241 | .claude/kb/milestone/02-second-txn.md:381 | 事件句不改 | 说的是「2026-09-18 已改（`mount.rs` 的 `MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported`…）」，是那一天的改动；同一格后文写了「用户 2026-09-19 定第二片进这个里程碑」，这个成员今天在 crates/ 里零命中 |
| P30242 | .claude/kb/milestone/02-second-txn.md:485 | 要改 | `JudgingCodes` 在 crates/ 零命中，今天 `crates/singlefs-checker-tier/src/crash_amplification.rs` 里的是 `PipelineJudgingCode`（1157 行）。改后的句子：…判法代码按点的文件各算一份（`crash_amplification.rs` 的 `PipelineJudgingCode`），几条流的公共前缀在库里只核一次。 |
| P30243 | .claude/kb/milestone/03-third-txn.md:111 | 事件句不改 | 说的是「里程碑二收尾留下的已知红钉值（用户 2026-10-01 转进这个里程碑；提交 dbcc0d2f 上就有）」：那个提交上的三条红用例名，是当时的名字，这一格后文写「三样按记录已清」；换成今天的名字这句话就不再是 dbcc0d2f 上发生的事 |
| P30244 | .claude/kb/milestone/03-third-txn.md:111 | 事件句不改 | 说的是「里程碑二收尾留下的已知红钉值（用户 2026-10-01 转进这个里程碑；提交 dbcc0d2f 上就有）」：那个提交上的三条红用例名，是当时的名字，这一格后文写「三样按记录已清」；换成今天的名字这句话就不再是 dbcc0d2f 上发生的事 |
| P30245 | .claude/kb/milestone/03-third-txn.md:111 | 事件句不改 | 说的是「里程碑二收尾留下的已知红钉值（用户 2026-10-01 转进这个里程碑；提交 dbcc0d2f 上就有）」：那个提交上的三条红用例名，是当时的名字，这一格后文写「三样按记录已清」；换成今天的名字这句话就不再是 dbcc0d2f 上发生的事 |
| P30246 | .claude/kb/milestone/10-tenth-txn.md:64 | 不相干 | `CAP_DAC_OVERRIDE` 是 Linux 能力位，句子说「POSIX ACL、能力位 `CAP_DAC_OVERRIDE` 这类认不认」，是设计轮要对齐的内核判法，不是本仓代码 |
| P30620 | .claude/kb/tooling.md:13 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30621 | .claude/kb/tooling.md:14 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30622 | .claude/kb/tooling.md:16 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30623 | .claude/kb/tooling.md:16 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30624 | .claude/kb/tooling.md:20 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30625 | .claude/kb/tooling.md:20 | 不相干 | 这是外部本地模型服务（`~/code/ai-center/.env.tenants`、vLLM 请求参数 `max_model_len`、`max_tokens`、`presence_penalty` 等）的名字，不是本仓 crates/scripts 里的代码标识符 |
| P30626 | .claude/kb/tooling.md:154 | 不相干 | `scsi_debug` 是 Linux 内核模块，表格是「任意扇区大小 + ZBC」的模块参数说明，不是本仓代码 |
| P40001 | .claude/kb/checks-owed.md:22 | 不相干 | C14 在这一句里是「布局清单文件随 C14（格式变了 checker 没跟） 有了：`.claude/gate.d/layouts.tsv`」，说的是已有；命中的「欠」是「C8（门禁范围判不出来） 还欠「专属路径与共享路径怎么分」那一层」，欠的是 C8 不是 C14 |
| P40002 | .claude/kb/checks-owed.md:262 | 不相干 | C14 写的是「布局清单随 C14 有了」、C20 写的是「已并进它」，都没写成开着；「目录约定仍欠」欠的是目录约定（C8 的前置） |
| P40003 | .claude/kb/checks-owed.md:270 | 不相干 | C634 只是「C634（单次 unlink 的墓碑字节没有 trace 检查） 同域」的交叉指，命中的「还没」来自「量还没算」说 D3 已定项 2，不是说 C634 开着 |
| P40004 | .claude/kb/checks-owed.md:275 | 不相干 | 「2026-09-18 C374（释放代与树表诞生 txg 只有验收断言盯着） 的 checker 实现」是那一天做了什么的事件句（并且括号里是 C374 的标题），没有说 C374 开着 |
| P40005 | .claude/kb/checks-owed.md:298 | 不相干 | C12 写的是「那一条 2026-09-21 落成的门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格」，已落成；命中的「仍」是「调用图仍要另建」，说的是本行要的工装，不是 C12 |
| P40006 | .claude/kb/checks-owed.md:317 | 不相干 | C25 在这一句里是立账依据「C25（三个 oracle 零代码） 的判据「每个 oracle 各立一条欠检查」：2026-09-21 现查…」，引的是它的判据与当天现查；命中的「欠」是「欠检查」这个词，没有把 C25 写成开着 |
| P40007 | .claude/kb/checks-owed.md:318 | 不相干 | C25 在这一句里是立账依据「C25（三个 oracle 零代码） 的判据「每个 oracle 各立一条欠检查」：2026-09-21 现查…」，引的是它的判据与当天现查；命中的「欠」是「欠检查」这个词，没有把 C25 写成开着 |
| P40008 | .claude/kb/checks-owed.md:319 | 不相干 | C25 在这一句里是立账依据「C25（三个 oracle 零代码） 的判据「每个 oracle 各立一条欠检查」：2026-09-21 现查…」，引的是它的判据与当天现查；命中的「欠」是「欠检查」这个词，没有把 C25 写成开着 |
| P40009 | .claude/kb/checks-owed.md:340 | 不相干 | C481 只在「那一行是 C481（坏盘输入的基线镜像取不到树表 0 条的盘面） 的 `newest_root_tree_table_has_no_entries`，另一回事」里，用来区分另一条变异；「还欠」「仍没有」说的是本行（模型对拍排除加回来的变异） |
| P40010 | .claude/kb/checks-owed.md:414 | 不相干 | C1 在这一句里是「D8 已定项 5 第 2 条与三态表与 C1（意图续做做成常规路径） 按承诺类 / 提示类分开写」，整理落地那天要动的条款之一，命中的「要先」不是说 C1 开着 |
| P40011 | .claude/kb/checks-owed.md:427 | 不相干 | 这一句写的是「C646（弄坏开关文件头点名的判错项与实测不等） 2026-10-13 已还清（点名的判错项已与实测逐项相等）」，已经说了已还清；「等」是「不等」这个词 |
| P40012 | .claude/kb/checks-owed.md:575 | 不相干 | 「C8 的前置还剩一半：布局清单随 C14 有了（`.claude/gate.d/layouts.tsv`），C8 还欠…」，C14 写成已有，欠的是 C8 |
| P40013 | .claude/kb/decisions/03-空间分配.md:239 | 要改 | C324 已在 checks-owed.md「已还清」表（2026-09-14 定案写进 D3 已定项 10 ⑤），而这一行挂在「**欠**：」后面。改后的句子：**欠**：无（C324（码 3 打包容器要不要 32768 对齐没写） 已由 ⑤ 收口，已还清）。 |
| P40014 | .claude/kb/decisions/04-校验和位置.md:122 | 不相干 | C581 在这一句里是「`extent_off` 第一版恒 0 的检查已有：池级 checker 的 `judge_pointer_extent_offset_is_zero`…归 C581（extent 偏移非 0 没有判据）」，说检查已有，没把 C581 写成开着 |
| P40015 | .claude/kb/decisions/09-加密.md:225 | 不相干 | 「**欠**：C177（C34 作废之后被解锁的三处没复核）」欠的是 C177（仍在 checks-owed.md 开着的表里）；C34 只出现在 C177 的标题里，且 C34 本身已销账作废 |
| P40016 | .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md:23 | 不相干 | C12 在这一句里是「若某一类账做不到幂等，那一类必须另有一条与运行时不共享判定代码的独立重算路径（C12（增量语义共用））」，引它作条款依据；命中的「等」来自「等」字（「幂等」），不是说 C12 开着 |
| P40017 | .claude/kb/decisions/16-发布语义.md:210 | 不相干 | 「实现已合入（…，实 C577（系统配置没见证到的最新根，乙罩不到） 报告 …），代价仍另登记小实验量」，C577 写成已实现；「仍」说的是代价另登记 |
| P40018 | .claude/kb/decisions/23-journal的角色与格式.md:57 | 不相干 | 「恢复期仍有「续做未完成意图」这条常规路径（C1（意图续做做成常规路径））」说的是这条路径存在，C1 已还清（`establish_instance` 无条件续做）与句子一致 |
| P40019 | .claude/kb/decisions/23-journal的角色与格式.md:136 | 不相干 | C577 在这一句里是 N-配置判据的依据之一（「C554 乙、C577（系统配置没见证到的最新根，乙罩不到）」），没写成开着；命中的「等」是「等它持久」 |
| P40020 | .claude/kb/decisions/23-journal的角色与格式.md:365 | 不相干 | 「用户知情认下（C519（丢一整块盘时恢复丢掉刚确认的那一版） 那一格…」，C519 已还清的内容正是「用户 2026-09-25 定…认下这个代价」，与句子一致；命中的「挡」是「要挡的一份新一份旧的嫁接」，不是说 C519 开着 |
| P40021 | .claude/kb/decisions/23-journal的角色与格式.md:390 | 不相干 | 「C393（被抛弃根的账读不出时修复没有条款） 取 (b)-从映射现算，用户 2026-09-28 定，被攻过零轮」，写成已定已做；命中的「仍」是「仍不成在取号之前拒可写」，说的是现算不成时的行为 |
| P40022 | .claude/kb/decisions/23-journal的角色与格式.md:392 | 不相干 | 「C694（被抛弃的根的账读不出时正常卸载被拒） 的 C，用户 2026-10-07 定「C 与 B 都做」」，写成已定；命中的「仍」是「只除去现行账里仍分配着的槽」，与 C694 开没开着无关 |
| P40023 | .claude/kb/decisions/23-journal的角色与格式.md:419 | 不相干 | 「C579（判据在单故障合法状态上也拒可写） 用户定维持乙字面」，C579 已还清的内容正是「用户 2026-09-27 弹窗定维持 C554 乙的字面」；命中的「仍」是「重读一次仍为真」 |
| P40024 | .claude/kb/experiments/126-系统配置槽宽四条候选的代价.md:3 | 事件句不改 | E126 页开头写的是这个实验当时要答什么（「它要答的是 C232（系统配置槽宽：类规则给了答案，字段表说未定） 挡着的那一格」），是实验目的的记录；把它换成「C232 已还清」这句话就不再是这个实验当时的动机 |
| P40025 | .claude/kb/experiments/142-新池新建文件的干跑.md:199 | 事件句不改 | 说的是 E142 第十三次跑（2026-09-23）当天的发现「`crates/` 还没有实现 C480」，同页第 308 行写了「已在第十四次跑（2026-09-25，`crates/` 补上 C480 之后）收口」「不再是当前状态」，所以这是那一次跑的事件，不是现状 |
| P40026 | .claude/kb/experiments/142-新池新建文件的干跑.md:308 | 事件句不改 | 说的是 E142 第十三次跑（2026-09-23）当天的发现「`crates/` 还没有实现 C480」，同页第 308 行写了「已在第十四次跑（2026-09-25，`crates/` 补上 C480 之后）收口」「不再是当前状态」，所以这是那一次跑的事件，不是现状 |
| P40027 | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:84 | 事件句不改 | 这是 E150 页对各决策的复核记录，「2026-10-01 不受影响：正文里欠账的简称随改名换新（C143…C314…C498…C542…）」说的是那一天复核改了什么，不是说这些账今天开着；命中的「欠」是「欠账」这个词 |
| P40028 | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:85 | 事件句不改 | 这是 E150 页对各决策的复核记录，「2026-10-01 不受影响：正文里欠账的简称随改名换新（C143…C314…C498…C542…）」说的是那一天复核改了什么，不是说这些账今天开着；命中的「欠」是「欠账」这个词 |
| P40029 | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:86 | 事件句不改 | 这是 E150 页对各决策的复核记录，「2026-10-01 不受影响：正文里欠账的简称随改名换新（C143…C314…C498…C542…）」说的是那一天复核改了什么，不是说这些账今天开着；命中的「欠」是「欠账」这个词 |
| P40030 | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:87 | 事件句不改 | 这是 E150 页对各决策的复核记录，「2026-10-01 不受影响：正文里欠账的简称随改名换新（C143…C314…C498…C542…）」说的是那一天复核改了什么，不是说这些账今天开着；命中的「欠」是「欠账」这个词 |
| P40031 | .claude/kb/experiments/151-用户数据落点的到达序与容器臂.md:4 | 事件句不改 | E151 页的「问题」写的是这个实验当时要补的基线臂（「D3 已定项 8 与 D14 已定项 2 欠的基线臂（用户定向…C239 第 ② 笔）」），是实验动机；C239 已还清不改变当时为什么跑它 |
| P40032 | .claude/kb/experiments/157-并行线一两条条款的计数模型.md:1 | 事件句不改 | E157 页标题写的是 2026-09-22 第一段跑完的状态「第 2 行挡着的 C491 本段未碰」，「本段」指那一段，是那一段的事件；C491 2026-09-23 已定（D23 已定项 17） |
| P40033 | .claude/kb/experiments/157-并行线一两条条款的计数模型.md:6 | 事件句不改 | E157 页的「问题」写的是跑前登记时被测的对象（「并行线一挡着的两条未定条款之一…C490」），是跑前登记那天的事；C490 2026-09-23 已定（D8 已定项 3） |
| P40034 | .claude/kb/experiments/158-择根与修复四岔路.md:369 | 事件句不改 | 在「### 2026-09-26：第 2 次跑第一段」小节里，「前提 2（C331 今天仍打得中、乙要重定义）｜ 成立」，「今天」指那一天（该小节标题的日期），是那天验前提的结果；C331 此后由系统配置见证 R2 挡住 |
| P40035 | .claude/kb/experiments/158-择根与修复四岔路.md:450 | 事件句不改 | E158 页的「问题」写的是跑前登记的四条岔路（「C393、C331 的修法、C332、C334 配代价数」），是跑前登记那天的事；这四笔的去向以 checks-owed.md 为准（C393、C331 已在已还清表） |
| P40036 | .claude/kb/experiments/158-择根与修复四岔路.md:801 | 不相干 | `H-C1` 是判决 K2 里给丙的具体历史的编号（「判决 K2 给丙的具体历史 H-C1 需要 12 个特定故障」），不是欠账 C1 |
| P40037 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:622 | 事件句不改 | 写的是「那一天 D23 已定项 14 的原文写…这里的实现还是 2026-09-28 取甲的那一版，欠 C331」，是 2026-10-03 那一版实现当天的事；同一句后文写了「根环那条线第 7 批 7-2 之后实现取 R2」 |
| P40038 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:631 | 不相干 | C331 在这一句里是「KG 上 C331（择根倒挂压过已确认的写） 的判别力自证」「C331 自证与整轮的判决行」，引的是 E165 产物里的自证行（`name=c331_proof`），没有把 C331 写成开着；命中的「等」是「等于」「相等」 |
| P40039 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1333 | 不相干 | C331 在这一句里是「KG 上 C331（择根倒挂压过已确认的写） 的判别力自证」「C331 自证与整轮的判决行」，引的是 E165 产物里的自证行（`name=c331_proof`），没有把 C331 写成开着；命中的「等」是「等于」「相等」 |
| P40040 | .claude/kb/experiments/79-根记录的容量.md:66 | 事件句不改 | 「上一次回看 2026-09-19：不受影响：…树表单元每层容量那一格按旧条目宽算，重算欠在 C157」，是 2026-09-19 那次复核的记录；此后回看行（2026-09-27、2026-10-11、2026-10-17）写的都是结论不变 |
| P40041 | .claude/kb/experiments/84-打包墓碑的钉住量.md:4 | 事件句不改 | E84 页开头交代实验动机「D18 已定项 10 …自陈钉住量欠实验（C84（墓碑单元的粒度没人定）剩的那半）。E84 把两种装载纪律各跑一遍」，是这个实验当时为什么跑；E84 标题已写「已跑」，C84 已由 D18 已定项 24 定了粒度 |
| P40042 | .claude/kb/milestone/02-second-txn.md:62 | 要改 | C338 已在 checks-owed.md「已还清」表（认购表在 D8 已定项 8 预留那一段，合计 48、余 28，三个 8 字节宽度已登记为 livelist 每头计数器 / 第二道销毁闸的活快照计数 / origin 链接，门禁 doc-registries 的 tree-table-reserve 格判），「仍要定」已不真。改后的句子：**会碰到的决策点**：C338（树表条目预留没有余量，已还清：认购表与三个 8 字节宽度已登记在 D8 已定项 8 预留那一段）；D8（核心索引结构） 已定项 8；D22（单元原子性怎么合成） 已定项 2。 |
| P40043 | .claude/kb/milestone/02-second-txn.md:336 | 事件句不改 | 这是收口表第 ① 行的第一格，写的是立账时要解决什么（「连带 C124 仍欠的那一半」）；同一行末尾已写「2026-09-26 回退改成挂着时的一次向前发布…这一行三格的对象都不在了」，C124 已作废 |
| P40044 | .claude/kb/milestone/02-second-txn.md:337 | 事件句不改 | 「C380 的前置写明与这一行一起判（2026-09-18 C374（释放代与树表诞生 txg 只有验收断言盯着） 的 checker 实现时立）」，是 2026-09-18 立账那一天的事件；命中的「前置」属于 C380 |
| P40045 | .claude/kb/milestone/02-second-txn.md:338 | 事件句不改 | 这一格写「欠账 C393…。**2026-09-28 用户定 (b)-从映射现算**…已实现，…C393 挪进已还清」，同一格已写明已还清；「欠账 C393」是当时立账的句子 |
| P40046 | .claude/kb/milestone/02-second-txn.md:342 | 事件句不改 | 这一格已写「（C331 已还清，修法动记录号的那一支也已做完）」，「C331 甲、乙挡不住，丁 挡得住」是 2026-09-22 三方判决的结论，是那一轮的事 |
| P40047 | .claude/kb/milestone/02-second-txn.md:344 | 事件句不改 | 「P7：2026-09-23 用户定案等整段重议时一起定，账记在 C514（回退行的 W 等整段重议）」是 2026-09-23 定案的事件，同一格后文写「C514 已作废」 |
| P40048 | .claude/kb/milestone/02-second-txn.md:348 | 事件句不改 | 「账在 C394…。**2026-09-25 实十六接续已打进主工作区**…」，这一格按日期记录 C394 的经过；命中的「还没有条款」属于另起一题，是 2026-09-23 三方判完那一天的状态 |
| P40049 | .claude/kb/milestone/02-second-txn.md:356 | 事件句不改 | 这一格写「运行时计数…已删除（C515（政策不一致计数恒 0，口径没改） 2026-09-25 已还清）」，已写已还清；「仍恒 0」是 2026-09-23 用户定案那天的状态 |
| P40050 | .claude/kb/milestone/02-second-txn.md:357 | 事件句不改 | 这一格末尾已写「C366 在 `.claude/kb/checks-owed.md` 已还清表里」；「怎么拦…没做」是 2026-09-19 时的状态，同格后文记了 2026-09-23 做完 |
| P40051 | .claude/kb/milestone/02-second-txn.md:358 | 事件句不改 | 这一格已写「C282（环里最旧根没有定义） 2026-10-11 已还清（改由 `crates/mutations.tsv` 的变异行…判红）」；「欠」是 2026-09-22/24 用户定案那天的状态 |
| P40052 | .claude/kb/milestone/02-second-txn.md:359 | 要改 | C29 已在 checks-owed.md「已还清」表，第 22 行同样写了「C29 2026-09-28 核实已还清」，这一句却写「「先信 tail」C29 那一形态仍没有会红的检查」。改后的句子：…「先信 tail」C29（恢复先信 tail 会丢数据） 那一形态当时仍没有会红的检查（2026-09-22 做出来，用例 `a_stale_tail_does_not_hide_the_record_above_it_that_recovery_must_apply`，C29 已还清）。 |
| P40053 | .claude/kb/milestone/02-second-txn.md:374 | 事件句不改 | 这一格记 2026-09-23 的状态「C378 按「认了」断言的用例已落（2026-09-23）…待代码三方与崩溃验证」；C378 已于 2026-09-28 还清。「待代码三方与崩溃验证」是否仍待，见「要主 agent 现查」 |
| P40054 | .claude/kb/milestone/02-second-txn.md:377 | 不相干 | 「不在 C318（影子账隔离的单元没进准入不等式） 射程内（那笔账的题面是准入不等式）」，用 C318 的题面划界，没把 C318 写成开着；命中的「等」是「不等式」 |
| P40055 | .claude/kb/milestone/02-second-txn.md:389 | 事件句不改 | 这一格是 2026-09-23 的定案记录（「E142 第十三次跑已照条款写 blocks = 6…这一句是 C480 实现的验收之一」）；C480 此后在 E142 第十四次跑之后收口 |
| P40056 | .claude/kb/milestone/02-second-txn.md:427 | 不相干 | 「判定三…（当天修，见 C461（checker 判「挂不上的镜像」整片报不适用））」，写的是当天修了，没写 C461 开着；命中的「仍」来自别处的「仍有 22 条新发现」 |
| P40057 | .claude/rules/three-way-inference.md:68 | 不相干 | 「（C273（小节清单不认一级标题），自证用 `KB_SECTIONS_NO_H1=1`）」只是引 C273 作一级标题要有自己行的依据，没写成开着；命中的「等」是「等」字 |
| P50001 | .claude/kb/checks-owed.md:122 | 事件句不改 | 「**2026-09-06 当日已把不变量写下来：I-7.8…**，欠的收窄成「它还没实现」（checker 不存在）」说的是 2026-09-06 当天的状态，紧接着同一格写「2026-10-11 现查已实现」，不变量文件里 I-7.8 状态列也是「已实现」 |
| P50002 | .claude/kb/checks-owed.md:235 | 不相干 | 这一格是 C309 的「怎么拦」：「读槽时算法标识读到 0 或未登记的码一律拒绝挂载，checker 同判，I-2.2 转已实现」，是修好之后的目标；I-2.2 状态列今天写「未实现（条款已写、检查仍欠）」，`IMPLEMENTED_INVARIANTS` 里也没有 I-2.2，句子与现状一致 |
| P50003 | .claude/kb/checks-owed.md:343 | 不相干 | 「D22 已定项 1 要的「由 I-7.4 扣住的块数挂载时算」没实现」说的是 C506 的上界现算这一件事没做（`crates/singlefs-format/src/lib.rs:285` 仍是 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16`，282 行 placeholder 注释写着 C506），不是说 D22 已定项 1 或 I-7.4 整体未实现；I-7.4 状态列「已实现」与它不冲突 |
| P50004 | .claude/kb/checks-owed.md:343 | 不相干 | 「D22 已定项 1 要的「由 I-7.4 扣住的块数挂载时算」没实现」说的是 C506 的上界现算这一件事没做（`crates/singlefs-format/src/lib.rs:285` 仍是 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16`，282 行 placeholder 注释写着 C506），不是说 D22 已定项 1 或 I-7.4 整体未实现；I-7.4 状态列「已实现」与它不冲突 |
| P50005 | .claude/kb/decisions.md:392 | 要改 | D22 已定项 14 的内容（devs < R 时区域设备身份取满 min(R, devs) 个不同值、每盘不超过 ⌈R/devs⌉）正是 I-7.6 在 `crates/singlefs-checker/invariants/02-根环.md` 第 13 行写的已实现判法，`IMPLEMENTED_INVARIANTS` 含 I-7.6（image.rs:140），所以「未实现」已不真。改后的句子：- 14. devs < R 时 I-7.6（根环区域落在互不相同的盘上） 判什么 —— 已定 · 已实现（decisions.md 这一行由 D22 已定项 14 的表格行「进度：未实现」生成，源头 `.claude/kb/decisions/22-单元原子性怎么合成.md` 第 24 行要同改成「进度：已实现」） |
| P50006 | .claude/kb/decisions/17-实现分层与第三方管道.md:140 | 不相干 | 「第 3 笔的条款已写、检查仍欠…落成不变量 I-1.5（系统配置记管道身份），checker 侧还没实现」说的是 checker 侧，I-1.5 状态列今天写「未实现（条款已写、检查仍欠）」，句子与现状一致；D22 已定项 9「进度：已实现」指字段表 |
| P50007 | .claude/kb/milestone/02-second-txn.md:592 | 不相干 | 句子写「原写「未实现，前置：树的种类」，2026-09-22 都已实现」，I-9.6 与 I-9.12 状态列今天都是「已实现」，句子与现状一致 |
| P50008 | .claude/kb/milestone/03-third-txn.md:38 | 不相干 | 句子写 I-7.15 等「已实现」，`02-根环.md` 第 22 行 I-7.15 状态列也是「已实现」，与现状一致 |
| P50009 | .claude/kb/milestone/03-third-txn.md:61 | 不相干 | 句子写 I-8.5 等「已实现」，「未实现的是 I-8.4（重放幂等）（实现在途）与 I-4.3（提交原子）」，03-journal.md 里 I-8.5「已实现」、I-8.4「未实现（实现在途…）」、06 里 I-4.3「未实现」，逐条一致 |
| P50010 | .claude/kb/milestone/03-third-txn.md:61 | 不相干 | 句子写 I-8.5 等「已实现」，「未实现的是 I-8.4（重放幂等）（实现在途）与 I-4.3（提交原子）」，03-journal.md 里 I-8.5「已实现」、I-8.4「未实现（实现在途…）」、06 里 I-4.3「未实现」，逐条一致 |
| P50011 | .claude/kb/milestone/03-third-txn.md:66 | 不相干 | 这一格列的是 C28 等「（已还清…）」与仍开着的 C147、C395、C446，I-8.5 在 C367 名下写「已还清」，与 03-journal.md 里 I-8.5「已实现」一致；C147 在 checks-owed.md 仍开着（仍欠第 ② 格坏镜像），也一致 |
| P50012 | .claude/kb/milestone/04-fourth-txn.md:57 | 不相干 | 引的是 D3 已定项 7 登记表第 7 行的逐字原话「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」，「整理未实现」说的是后台整理这件事，不是已定项 7 的进度；03-空间分配.md 第 17 行已定项 7「进度：已实现」说的是分配记录条目编码 |
| P70001 | .claude/kb/checks-owed.md:387 | 事件句不改 | 句子写「立账时点名的变更史现状行 `--write` 2026-09-29 随 status-sync 格删掉」，是把 status-sync 格当已删写的事件；status-sync 在 `.claude/gate.d/*.sh` 零命中与之一致。同一格里「`doc-decisions.sh` 第 1640、1653 行」「第 1644、1679 行」的行号已对不上（见「要主 agent 现查」） |
| P70002 | .claude/kb/checks-owed.md:405 | 要改 | `file-names` 是 `scripts/crash/crash-case-check.py` 自己的格（第 8、34 行、选项 `--only file-names`），不在 gate.d 登记里没问题；但「出路已改成「来历进 commit message 或 .claude/kb/」（第 286 行）」的行号已不对：该文字在 crash-case-check.py 第 360 行，第 286 行是 `for loop in FOR_LOOP.finditer(body):`。改后的句子：`scripts/crash/crash-case-check.py` 的 file-names 格出路已改成「来历进 commit message 或 .claude/kb/」（第 360 行） |
| P70003 | .claude/kb/checks-owed.md:425 | 不相干 | `harness-test-granularity` 是用例上方的标记注释的前缀（`// harness-test-granularity:one-scenario <理由>`），`scripts/crash/crash-case-check.py` 第 37 行与 harness-model-differential-and-scenarios.sh 第 27 行都用它；不是门禁阶段名，句子也没把它当阶段写 |
| P70004 | .claude/kb/checks-owed.md:774 | 不相干 | `missing-commands` 是 `scripts/crash/gpu-quiesce-run.sh --selftest` 的格（第 71、362 行），句子写「缺键退 78 由 `gpu-quiesce-run.sh --selftest` 的 missing-commands 格钉住」，不是 gate.d 的 gate-cell，现存 |
| P70005 | .claude/kb/checks-owed.md:796 | 不相干 | `outer-memory-cap-to-driver` 是 `scripts/crash/crash-point-replay-shard-run.sh --selftest` 的格（第 85、321 行），弄坏开关 `ignore-outer-cap` 也在该脚本里命中 4 次，不是 gate.d 的 gate-cell，句子与现状一致 |
| P70006 | .claude/rules/verification.md:28 | 不相干 | `harness-test-granularity` 是用例上方的标记注释的前缀（`// harness-test-granularity:one-scenario <理由>`），`scripts/crash/crash-case-check.py` 第 37 行与 harness-model-differential-and-scenarios.sh 第 27 行都用它；不是门禁阶段名，句子也没把它当阶段写 |
| P90001 | .claude/agents/sweep.md:41 | 不相干 | 这是 sweep 定义自己的判据说明（「「现状（日期……）：」「2026-09-14 现状：」这种冒号句，冒号后面每个分句单独这样判」），举例用的格式，不是对仓的现状断言 |
| P90002 | .claude/kb/checks-owed.md:106 | 不相干 | 删日期后仍真：「现状：`strip_timing` 已把 `dev` 与计时字段一样抹掉，E58 登记为 timing；`research/results/` 下没有 E58 的留存产物…E65、E66 不在复跑登记表」——现查 replay.sh 的 `strip_timing` 含 `dev`（1134 行），`E58｜e58-csum-grain｜…｜timing`（894 行），research/results 下 e58-csum-grain 零个文件，replay.sh 里无 E65 / E66 行 |
| P90003 | .claude/kb/milestone/01-first-txn.md:19 | 要改 | 「四个 crate」已不真：根 `Cargo.toml` 的 members 今天是 singlefs-format、core、replay-prefix、harness、checker、checker-tier、reference-filesystem、unit-graph 八个，`exclude = ["scripts/experiments"]`。改后的句子：**开工那天（2026-09-14）的现状**：根 `Cargo.toml` workspace（排除 `research/`）+ 四个 crate：…（今天 workspace 成员是八个，以根 `Cargo.toml` 的 `members` 为准）。 |
| P90004 | .claude/kb/milestone/01-first-txn.md:43 | 要改 | 末句「镜像准备（journal 环与根环区域全 0）靠新建的稀疏镜像，真设备上要显式写 0 的那一半还没做」已不真：`make_filesystem` 现在显式 `write_zeroes_at` 清 [0, 根环基址)、各根环区域与 journal 环（`crates/singlefs-core/src/make_filesystem.rs` 558、560、562 行）；另「六个模块」后列了七个名字。改后的句子：…镜像准备（journal 环与根环区域全 0）由 `make_filesystem` 显式 `write_zeroes_at` 写 0，稀疏镜像与真设备同一条路。 |
| P90005 | .claude/kb/milestone/01-first-txn.md:76 | 要改 | 「政策函数复算比对计数（新池新建文件恒 0）」已不真：`PoolAllocator::policy_mismatches` 字段已删（02-second-txn.md 356 行「运行时计数…已删除（C515 2026-09-25 已还清）」，crates/ 里 `policy_mismatches` 零命中）。改后的句子：…提交内生块从开放段 bump（容器按 32768 对齐档）、分配记录 20 字节编码（当时还有政策函数复算比对计数，恒 0；2026-09-25 随 C515 已删）；… |
| P90006 | .claude/kb/milestone/01-first-txn.md:103 | 事件句不改 | 这一段记步 3 开工那天落地了什么（「验收四条全过」，各偏移与条目宽），树表条目宽已写 200（含 2026-09-16 改宽）；提到的 `BirthSequenceAllocator`（transaction.rs）、`index_node_view`、`check_index_node_keys`、`packed_unit_view` 现查都在 |
| P90007 | .claude/kb/milestone/01-first-txn.md:134 | 事件句不改 | 记步 4（映射）开工那天的落地与验收（「验收三条全过、一条要覆盖写、释放、回退与复用」）；`mapping_key_for_data` / `mapping_key_for_node` / `mapping_key_sort_key` 现查在 records.rs 里，「没做」指新池新建文件里做不到的两条 |
| P90008 | .claude/kb/milestone/01-first-txn.md:159 | 要改 | 「两个运行时计数（取号序不一致、落点偏离政策函数）都是 0」里的落点偏离政策函数那个计数已删（C515，2026-09-25 已还清；crates/ 里 `policy_mismatches` 零命中，02-second-txn.md 356 行）。改后的句子：两个运行时计数（取号序不一致、落点偏离政策函数）都是 0（落点偏离政策函数那个计数 2026-09-25 随 C515 已删）。 |
| P90009 | .claude/kb/milestone/01-first-txn.md:191 | 事件句不改 | 记步 5（恢复）开工那天的落地与八条探针验收（「验收五条全过」，「丢掉进程内的设备句柄、按路径重开两个镜像走完整挂载路径」）；现查 `JournalPolicy::ConsultWithoutNamedVerification`、`parse_index_node` 等仍在 crates/ 里 |
| P90010 | .claude/kb/milestone/01-first-txn.md:217 | 要改 | 「`crates/singlefs-harness` 的 `crash` 模块」已不真：harness 的 src 下没有 crash 模块，崩溃点重放在 `crates/singlefs-checker-tier/src/crash.rs`（`check_records` 在 37 行），`CrashImage` 在 `crates/singlefs-harness/src/memory_pool.rs:445`。改后的句子：`crates/singlefs-checker-tier` 的 `crash` 模块（`src/crash.rs`；当时住在 `crates/singlefs-harness`）——录制流开内容保留（`SharedStream::retaining_contents`）；…（`CrashImage` 今天在 `crates/singlefs-harness/src/memory_pool.rs`，实现 `PoolReader`）… |
| P90011 | .claude/kb/milestone/02-second-txn.md:68 | 要改 | 「固定脚本做到发布 E…54 段、279 次写、闭式 2104413」与 `crates/singlefs-harness` 的 `crash` 模块都已不真：同文件 227 行写固定脚本今天到正常卸载、78 段、481 次写、全量 1 678 377 204，crash 模块在 `crates/singlefs-checker-tier/src/crash.rs`。改后的句子：**现状（2026-09-17 步 5 落地之后的读数；2026-10-17 注：固定脚本今天做到正常卸载，78 段、481 次写，崩溃点重放的模块在 `crates/singlefs-checker-tier/src/crash.rs`，数以 227 行与门禁 checker-tier-crash-point-replay 当前产物为准）**：固定脚本做到发布 E（…）：54 段、279 次写、闭式 2104413 个状态… |
| P90012 | .claude/kb/milestone/02-second-txn.md:94 | 要改 | 句首「验收第 2 条「搬落点、经映射读对」与第 4 条三条变异没做，归增补 2 收口表第 36、45 行」已不真：第 36 行（线二 2026-09-22 已落地）与第 45 行（步 1 验收第 4 条三条变异实现已落，2026-09-23）都已做；`publish_file_version` 在 crates/ 零命中（cae5092f 删，今天入口是 `publish_first_file` / `publish_overwrite`）。改后的句子：**现状（2026-09-16 开工；验收第 2 条与第 4 条三条变异当时没做，归增补 2 收口表第 36、45 行，之后都已做完，2026-09-22、2026-09-23；其余两条过了）**：…把新池新建文件那条单体函数抽成 `publish_file_version`（今天已无此名，入口是 `publish_first_file` 与 `publish_overwrite`）… |
| P90013 | .claude/kb/milestone/02-second-txn.md:121 | 事件句不改 | 记步 2（释放）开工那天随步 1 落地的东西与验收数（「mkfs 3 + A 10 + B 10 = 23 槽」「记账 15 行」）；`release`、`deferred_slots`、`placements_to_release_via_mapping`、`lowest_user_data_slot` 现查仍在 crates/ 里 |
| P90014 | .claude/kb/milestone/02-second-txn.md:145 | 事件句不改 | 这一段开头已带「（2026-10-11 注：这一段点名的四个 `MountError` 成员今天都不在 `crates/` 里…今天的拒绝形态以 `mount_writable` 的代码为准）」，主体是 2026-09-16 落地与 2026-09-17 用户定案那天做了什么 |
| P90015 | .claude/kb/milestone/02-second-txn.md:198 | 要改 | 同一行的 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 与用例名今天都不在 crates/ 里（见该行 P3 组的判定）；改后的句子：没做过可写挂载的进程抬 F 被拒绝而不 panic（当时的错误成员 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 与用例 `raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking` 今天都不在 `crates/` 里，以 `mount::raise_rollback_floor` 的代码为准）。 |
| P90016 | .claude/kb/milestone/02-second-txn.md:227 | 事件句不改 | 这一段带多处日期注（「2026-09-17 当时新池新建文件流 262165 个状态零违例（2026-10-11 注：后来屏障改动把新池新建文件流改成 11 段、闭式 16 777 240）」「2026-10-17 注：2026-09-21 起 `MODES` 加了…」），当时的读数与今天的数已分开写；主体句是 2026-09-17 部分落地那天的验证记录 |
| P90017 | .claude/kb/milestone/02-second-txn.md:250 | 事件句不改 | 说的是 E152 在 2026-09-17 步 7 那次正式跑的结果（「5 轮一次过、判据全中」，「发布 B 两盘合计 21 次写调用」），是那一次跑的事件 |
| P90018 | .claude/kb/milestone/02-second-txn.md:292 | 事件句不改 | 记增补 1 在 2026-09-17 立账与开工那天的交回状态（「第 1 件已交回」「第 3 件交回，报告入库」「代码三方第一轮打中失败路径那一格」），是事件句；`crates/singlefs-core/src/write_accounting.rs` 现查在 |
| P90019 | .claude/kb/milestone/02-second-txn.md:330 | 事件句不改 | 这一段写的是 2026-09-17 立、09-19 重核时收口表哪几行已关（「2026-09-19 重核时收口表里已关的是第 10、11、15、33、34、37、39、42 行」），并写明「之后各行的去向以收口表「去向」一列为准」，是重核那几次的事件 |
| P90020 | .claude/kb/milestone/02-second-txn.md:427 | 事件句不改 | 记增补 3 在 2026-09-18 立账那天的状态（「工具已装（nightly 工具链、cargo-mutants 27.1.0、cargo-fuzz 0.13.2）」「第 1 件交回」），是事件句；cargo-mutants 与 cargo-fuzz 仍装在 ~/.cargo/bin，版本号没复核（见「要主 agent 现查」） |
| P90021 | .claude/kb/milestone/02-second-txn.md:550 | 要人看 | 「**还剩一样没做**：崩溃点重放每个恢复结果上跑一遍（归提交时的主 agent）」今天是否仍没做我判不准：crash_injection.rs:6923 用到 `mount_read_only`，但没有逐条核到「每个恢复结果上各跑一遍」的覆盖；见「要主 agent 现查」 |
| P90022 | .claude/kb/milestone/02-second-txn.md:574 | 要人看 | 「**还没做的两样**：崩溃点重放小负载（一次发布让容器分裂一次，归提交时的主 agent）；十万个文件那一档」：现有 `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs` 一次发布建 N 个 inode 把叶容器撑过分配记录树叶的末槽，可能已覆盖第一样；十万个文件那一档没找到。是否仍没做要人判，见「要主 agent 现查」 |
| P90023 | .claude/kb/verification-build.md:154 | 事件句不改 | 「2026-09-14 现状：「harness 自己要证明会红的用例」那张表的第一条…已做成用例」是那一天的状态，同一句后文已加「（2026-09-17 起覆盖写、释放、多次挂载、回退都已进崩溃点重放覆盖写固定脚本流，现状见…）」；两个用例名（`removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker`、`named_unit_verification_keeps_a_damaged_transaction_out_of_the_rebuilt_root`）现查都在 |
| P90024 | .claude/kb/verification-build.md:236 | 事件句不改 | 「2026-09-14 现状：里程碑「新池新建文件」步 0–7 做完…在这一天都还没有代码」是那一天的状态，同一句后文已加「（覆盖写事务 2026-09-16 起有代码…现状：第 7 步 crash refinement 的取前缀、择根、扫环三截进了 Verus…）」更新到今天 |

## 要主 agent 现查

- `.claude/kb/milestone/02-second-txn.md:550`（P90021，判要人看）：「还剩一样没做：崩溃点重放每个恢复结果上跑一遍」今天是否仍没做。我只看到 `crates/singlefs-checker-tier/src/crash_injection.rs:6923` 用了 `mount_read_only`，没有逐条核到「每个恢复结果上各跑一遍」。
- `.claude/kb/milestone/02-second-txn.md:574`（P90022，判要人看）：「还没做的两样」里崩溃点重放小负载（容器分裂一次）可能已由 `crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs`（一次发布建 N 个 inode、叶容器撑过分配记录树叶末槽）覆盖，「十万个文件那一档」没找到。
- `.claude/kb/milestone/02-second-txn.md:374`（P40053，判事件句不改）：同一格末尾「待代码三方与崩溃验证」2026-09-23 之后是否做过没核；C378 已于 2026-09-28 还清，该用例有变异靶。
- `.claude/kb/milestone/02-second-txn.md:427`（P90020，判事件句不改）：「cargo-mutants 27.1.0、cargo-fuzz 0.13.2」是立账当天装的版本，今天版本没复核（`cargo mutants --version` 被重型测试闸拒了，没绕）。
- `.claude/kb/checks-owed.md:387`（P70001，判事件句不改）：同一格里「`.claude/gate.d/doc-decisions.sh` 第 1640、1653 行」「第 1644、1679 行」现在对应的是别的代码（1640 行是 `for line in body:`，1653 行是 `bad("清单", …已定项 7 的代码块里一行「第 N 层：」都没读到")`），行号对不上；这一行写着「前提不在」，是否该关账请判。
- `.claude/kb/decisions/22-单元原子性怎么合成.md:24` 已定项 14 表格行写「进度：未实现」，decisions.md:392 是它的投影；P50005 判要改，改的源头是这一行，投影是否由门禁 decision-items-sync 自动带出请主 agent 确认。
- `.claude/kb/experiments/49-反向链宽度32还是64.md:203`（P30230，判要改）：这一段里 base 296..319、303、307 是当时的记录头长上的读数，单测今天扫 440..=520；我只改了单测名与范围的说明，这一段的数字是否要整体重核请判。
- `.claude/kb/milestone/01-first-txn.md:159`（P90008）：「取号序不一致」那个运行时计数是否还在，我在 crates/ 里没找到对应名字，只确认了「落点偏离政策函数」那个已删。
- `.claude/kb/milestone/02-second-txn.md:94` 与 `:592` 都点到 `publish_file_version`，该函数 cae5092f 之后已不在 crates/（今天入口是 `publish_first_file` / `publish_overwrite`）；:94 我判了要改并写进改后句，:592 那一行不在我的候选里，没判。
- P3 探针的范围只含 crates/scripts/gate.d/hooks，所以 E175 装置名等住在 `research/results/` 的名字都被它扫成零命中；这类假阳性可以在探针里加 `research/results` 的 `device-source.rs`。

## 汇总

- 183 行：要改 16、要补 0、事件句不改 58、不相干 107、要人看 2（`awk` 数「## 逐行判定」表第三格得出：P3 63 / 18 / 3 / 0，P4 28 / 26 / 3 / 0，P5 10 / 1 / 1 / 0，P7 4 / 1 / 1 / 0，P9 2 / 12 / 8 / 2，格式「不相干 / 事件句不改 / 要改 / 要人看」）。
- 要改的 16 行（组号）：P30230、P30235、P30242、P40013、P40042、P40052、P50005、P70002、P90003、P90004、P90005、P90008、P90010、P90011、P90012、P90015。
- 数「## 逐行判定」表行数的命令：`grep -c '^| P' judge-S39.md`，得 183。

## 没做什么

- 没改任何仓内文件，没改 `stale=`。没建副本、没起编译、没跑重型测试与门禁。
- P9 里 milestone/01、milestone/02 的「现状（日期 开工 / 落地）」段落是整段步骤快照，我只现查了能 `git grep` 到的名字与几处会变的数（crate 数、`write_zeroes_at`、模块位置、计数字段），没有逐个验收数字重算；判事件句不改的几行在这一点上可能漏。
- P4 里 C 号所在句子是否还写成开着，按前后各约 200 字判；整格很长的行（02-second-txn.md 收口表各行）没有逐字读完。
- 推翻条件：任何一行「不相干」若其被引名字其实是现有代码里的符号（在 crates/ 以外的目录里的 `git grep` 命中不算），或「事件句不改」那一格里没有写明日期 / 「当时」，就该改判要改。

## check-report 输出末行

  ✓ 报告判全了：183 行候选都有逐行判定
