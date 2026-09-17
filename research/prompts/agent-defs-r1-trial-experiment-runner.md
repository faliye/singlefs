# experiment-runner 试跑的交回原文（主 agent 从转录里原样取出；该 agent 没写报告文件）

E153「树表单元每层棵数随条目宽的整数边界」试跑已完成（仓副本 `/tmp/claude-1000/agents/trials/experiment-designer/repo/`，真仓一个字未动）。

## 开工前 ps 检查
`ps aux` 见本机有别的会话在跑 `cargo test --release --bin e142-first-txn-dry-run`、`second_transaction_step_zero_layer0`。按派发提示给的例外，照常编译（先在 `CARGO_TARGET_DIR=/tmp/claude-1000/agents/trials/experiment-runner/draft/target` 下迭代，最后为让 `replay.sh` 用相对路径 `./target/release/$bin` 找到二进制，又在仓副本自己的 `research/target/` 下正式 build 了一次——这两个 target 目录都在仓副本内部，不碰真仓）。

## 跑前登记「修订」
按主 agent 的两项决定写进了 `research/prompts/e153-preregistration.md` 第十二节：①「200 离最近边界差几字节」采读法乙（Q6），读法甲（Q5）不进报告，只在单测里核读法乙与它的恒等式（原 A10）；②简称沿用登记标题。原判据一字未改。

## 单测数
20 个（`cargo test --release --bin e153_tree_table_entries_per_level_boundary 2>&1 | grep -c '^test tests::'` 数出来），全绿，零 warning。

## 变异三个数与分类
`research/mutations/e153_tree_table_entries_per_level_boundary.tsv` 11 条，`nice -n 19 bash research/scripts/mutate.sh e153_tree_table_entries_per_level_boundary research/e7-index-bench/src/bin/e153_tree_table_entries_per_level_boundary.rs research/mutations/e153_tree_table_entries_per_level_boundary.tsv` 跑完整张表，收尾「已还原，基线仍全绿」。**11 条全部被抓、0 无效、0 未抓**，不需要按 `mutation-sampling.md` 三分。其中头宽 ±1 两条（M1/M2）值得记一笔：本机验证 16253 是素数，`⌊16253/w⌋` 对分母 ±1 在 180–240 区间几乎处处不敏感，真正抓住它们的不是绝对值断言 A2/A4，而是「分母字面量比对」那条断言——已写进源码注释和实验页,供以后参考。

## 产物路径与完成标记
`research/results/e153-tree-table-entries-per-level-boundary-2026-09-17.out`，120 行，末行 `E7RESULT name=done emitted=120`（跑前已删旧文件，这份是本轮才有的产物；此前没有同名旧产物）。

⚠️ **踩了一个坑，已修复**：第一次生成产物用的是 `CARGO_TARGET_DIR` 指向草稿目录的二进制，而那份二进制是**在按 naming-lint 结果重命名 `width_min`/`width_max` 等标识符之前**编译的（那次重命名的 sed 无意中也把输出字段标签 `width_min=`/`width_max=` 改成了 `width_minimum=`/`width_maximum=`，属于良性但改变了输出文本）。用旧二进制生成产物导致 `replay.sh E153` 第一次判「对不上（50 行不同）」。用重命名后重新 build 的二进制重新生成产物后，`replay.sh E153` 字节一致、退出码 0。

## replay.sh 结果
已在 `research/scripts/replay.sh` 里加一行 `E153|e153_tree_table_entries_per_level_boundary||e153-tree-table-entries-per-level-boundary-2026-09-17.out|exact`（二进制无显式 `[[bin]]`，靠 autobins 按文件名 stem 发现，与 E44 同型，没有碰 `Cargo.toml`——那不在我的写范围里）。`bash research/scripts/replay.sh E153` → **字节一致，退出码 0**。

## 实验页路径
`.claude/kb/experiments/153-树表单元每层棵数随条目宽的整数边界.md`，标题状态「已跑（试跑，仓副本，2026-09-17，确定性模型，20 单测 / 11 条变异全抓）」；索引行加进 `.claude/kb/experiments.md`。结果一节整段抄自产物（带文件名），写了复跑命令、口径（纯算术不需要多轮、装置与 `crates/` 不共用代码但逐项同形）、它答不了的（第二层真实容量、发布时机、条目宽以外的旋钮）。

## 阶段归属表登记的门禁阶段（experiment-runner 名下 11 个，逐个单独 nice -n 19 跑过）
`15-research-build.sh`、`27-format-constants.sh`、`33-mutation-tables.sh`、`34-experiment-index-sync.sh`、`40-results-cited.sh`、`52-segment-registry.sh`、`80-absolute-assertions.sh`、`85-repro-command.sh`、`86-experiment-orphans.sh`、`87-replay.sh`、`88-quoted-result-lines.sh`——**全部 exit=0**。87 号默认档跳过 7 个慢/计时实验（E9/E44/E58/E140/E107/E128/E144，与 E153 无关），其余 120 个（含 E153）全部字节一致。33 号确认变异表锚点唯一命中源码（139 个二进制、1408 条变异，含 E153 的 11 条）。88 号确认实验页里整行抄的产物行在产物里逐字找得到（556 行，含 E153 那一段）。这批门禁跑得比较久（gate 87 复跑全部 120 个实验二进制，本机同时有别的会话在跑 cargo，总耗时约 10–15 分钟），中途按 pid 等待、没有靠轮询短 sleep。

另外顺手跑了一次 `bash .claude/scripts/naming-lint.sh`（不在阶段归属表里，但发现我新文件带 13 处缩写违规——`WIDTH_MIN`/`WIDTH_MAX`/`width_min`/`width_max`/`max_drop_and_counts`/两个含 `kb` 的测试名，均已改写全称，改完确认整体门禁从 13 处降到 0，其余 168 个文件不受影响）。

## 试跑观察（定义里哪些不清楚、做不下去、与实际不符；每步墙钟耗时，粗略估计）

1. **「跑前 ps 看负载」这一步的判据只写了看到就停**，没写「主 agent 已给例外时怎么记录」——我按常识在报告里补了这一段，但定义本身没要求「记下当时 ps 看到了什么」（这是 `.claude/agent-common.md` 里的要求，`experiment-runner.md` 自己没写），建议以后统一写进 `experiment-runner.md` 本体，别只靠共用约束兜底。耗时 <1 分钟。
2. **写 bin 源文件**：设计员的跑前登记把判据、断言、变异写得很细（第五、七、九节），照抄基本能落地，但**主 agent 裁定「Q5 不进报告」之后，怎么处理 A10（一个同时依赖 Q5 与 Q6 的恒等式）没有现成指引**——我自己判断把它挪成单测（不进 E7RESULT 报告流），这个判断合不合理需要看的人再确认一遍，登记第十二节我也如实写了这个改动，但它是我在装置写作时自己做的解读，不是登记原文直接给的。耗时约 15–20 分钟（含反复读登记、独立 python 验算、设计每条变异的定位字符串）。
3. **naming-lint 不在定义列出的步骤里，但它是共享门禁的一部分**，写完代码直接编译通过、单测通过，**看起来就能收尾了，但如果不额外主动跑 naming-lint，这 13 处违规会一直带着**（33/80/87/88 这些门禁阶段都不查命名）。建议 `experiment-runner.md` 第 2 步明确点名要跑一次 naming-lint。耗时约 5 分钟（跑 + 改 + 复跑确认）。
4. **产物生成这一步最容易踩坑**：定义第 1 步给的例外允许用独立 `CARGO_TARGET_DIR` 编译，但没提醒「用于生成最终产物 / 给 replay.sh 用的二进制，必须和最后一次源码改动对齐，且要在 `research/target/`（`replay.sh` 硬编码的相对路径）里」。我第一次图省事直接用草稿 target 目录里的旧二进制生成产物，被 `replay.sh` 当场打回，返工一次。耗时因此多花约 5 分钟；若定义里提前点破这条，可以省掉。
5. **门禁阶段耗时远超预期**：`experiment-runner.md` 第 6 步说「其中要编译或复跑整张表的，开跑前照第 1 步看负载」，但没提示 87 号（复跑全部已入库实验）与 15 号（`cargo test --release` 整个 research 工作区）这两个阶段本身就很重——不是因为我这个新实验重，是因为它们要把**全仓所有实验**都跑一遍。这一步单独耗时约 10–15 分钟（后台跑，期间机器上还有别的会话在跑 cargo，互相不冲突但都在排队）。建议以后派发时提前告知这批阶段耗时量级，免得试跑的人以为卡住了。

没做的（按定义本来就不归我）：没判这次结果能不能推翻或确立任何决策（三方论证没走）；没跑门禁全量（只跑了登记给 experiment-runner 的 11 个阶段）；没提交任何东西。
