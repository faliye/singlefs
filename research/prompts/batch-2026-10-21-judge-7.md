# 阶段同步第二段：逐行判（batch-2026-10-21，组 F46-F66）

基准 1bb3a24dc，结束「工作区」。候选表 /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv，开工时 `sha256sum` 为 a2913e391a222ec455f068bb72d7123c94659be2da960181125ecf2bc132e6d9（与派发提示一致）。

## 结论

- 分到的组：F46 F47 F48 F58 F59 F60 F64 F65 F66（候选表里 F49-F57、F61-F63 是新立行，不出候选，所以 F46-F66 区间实际有候选的只有这九组）。行数用 `awk -F'\t' '$1 ~ /^F(46|47|48|58|59|60|64|65|66)$/' candidates.tsv | wc -l` 数，输出 224；逐行判定表用 `grep -c '^| F' ` 数也是 224。
- 判定分布（`awk -F'|' '{gsub(/ /,"",$4); print $4}' table | sort | uniq -c`）：不相干 105、事件句不改 90、要改 26、要补 3、要人看 0。
- 按组：F46 12 行（要改 4、不相干 7、事件句 1）；F47 26 行（要补 3、不相干 17、事件句 6）；F48 11 行（要改 1、不相干 9、事件句 1）；F58 20 行（要改 2、不相干 3、事件句 15）；F59 16 行（要改 12、不相干 4）；F60 27 行（要改 1、不相干 15、事件句 11）；F64 16 行（全不相干）；F65 48 行（要改 3、不相干 13、事件句 32）；F66 48 行（要改 3、不相干 21、事件句 24）。

## 要主 agent 知道的（现查结果）

1. F46：E10 状态没同步。现查 `ls research/results/e10-aging-compaction-2026-10-13` 有 4 个 .out（main-s4、disk_1gib 三档），`scripts/experiment-apparatus/e10_aging_compaction.rs` 与 `e10-aging-compaction-run.sh` 在库，`research/prompts/e10-r1-prereg.md` 在；`.claude/kb/experiments.md` 第 35 行与实验页首行仍写「未跑」「仍欠后台整理」，决策 10 号、里程碑四、里程碑九三处引用「未跑」。后台整理本身也已入库：`crates/singlefs-core/src/compaction_plan.rs` 的 `git log` 只有 0b926bd3f，基准那一版没有。要改的四行：.claude/kb/decisions/10-随机写-碎片化的真答案.md:19、.claude/kb/experiments.md:35、.claude/kb/milestone/04-fourth-txn.md:59（还写「E10 没有跑前登记（research/prompts 下 e10-* 零个）」，同一行里整理未实现的几句是别的事实）、.claude/kb/milestone/09-ninth-txn.md:119。实验页 .claude/kb/experiments/10-老化结构抗老化vs事后整理.md 首行也写「未跑」，它不在候选里（检索词没搜到），主 agent 改索引行时一起改。
2. F59：`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 输出 35（基准 `git show 1bb3a24dc:.claude/gate.d/stage-inputs.tsv | grep -c '^crash-case:'` 输出 14）。里程碑三 .claude/kb/milestone/03-third-txn.md:141 写「登记的流 29 条」，三语都要改成 35；.claude/kb/verification-build.md 第 6、7 行三语写 14 条，且同一句里新池新建文件流的 16 777 260 / 16 777 240 应是 16 777 268 / 16 777 243（`FULL_STATES` 现 16_777_268，闭式 16_777_243）、固定脚本流 1 678 377 204 应是 1 678 377 220（`FULL_STATES_THROUGH_THE_UNMOUNT`）；checks-owed C564 三语写「已登记 14 条」要改 35，标题「另外 6 条」是另一个量，35 条之后还剩几条没人跑要现核（我没核）。
3. F65/F66 里 C446 三语（checks-owed.md:314、en:315、ja:315）写「在用 115 条，池级 checker 判 92 条」：README 现写 126（`crates/singlefs-checker/invariants/README.md` 第 36 行），CLAUDE.md 第 113 行的数法命令现数 103；126 减 103 是 23，「其余 23 条」仍对。译文这一轮豁免（调度记录第 72 行）。
4. F47：C566 三语只写到 E156 第 5、6 次重跑，第 7 次（洞数扫到环满一圈，Q1a 峰值仍在涨、上界落在最大档）没补，判「要补」。
5. F58：E142 页 :22、:310 两处「今天的流……全量 16777260」是装置按取号写一轮量的数，crates 现是取号写两轮的 16777268；layout/01-first-txn 三语已写明。其余 E142/E161/E167/E170/E171/E184 页与记录里的 16777260 是产物原行或当次读数，判事件句。
6. F64：剩下的 `~/kbuild/linux-om`、`~/linux-bug-fix/linux`、`~/code/fs-refs`、`~/.vscode-server` 都在 `.claude/out-of-project-paths` 登记放行（现查前几行），16 行全不相干。
7. F65/F66：门禁这两件事（按语言取文件、认变更史节标题；第 7 段只数中文原文）的记载在 `lib-item-ref-status.py` 文件头、`doc-registries.sh` 第 94 行起的注释和调度记录里，我没另加「要补」行；唯一要改的是 C257 三语的「仍按三级回退解析」（文件头现写六级，基准那一版是三级）。
8. 反向问（定义第 10 步）：F46-F66 这九件事各自都有载体记着（F47 的页与索引、F48 的页、F58 的 layout、F60 的 verus-pins 与里程碑三），没记全的已落在上面「要改」「要补」行里，没有另加 M 行。

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F46 | .claude/kb/checks-owed.md:535 | 不相干 | 「取值进代码前没有模型钉绝对值」这一行只把 E10 跑前登记当作建九格计数模型的地方点名；E10 第一段 21 片只跑了 4 片、读数只当线索（records/2026-10-13-里程碑四预开发.md 第 122 行），模型没有钉出绝对值，「没有计数模型」这句不受 E10 装置入库影响。 |
| F46 | .claude/kb/decisions/10-随机写-碎片化的真答案.md:19 | 要改 | 「而它未跑」「降级判据用 E10 自己写死的那一条，而它只跑了第一段 21 片里的 4 片、读数只当线索，剩 17 片停下」。现查：research/results/e10-aging-compaction-2026-10-13/ 有 4 个 .out（main-s4、disk_1gib 三档），scripts/experiment-apparatus/e10_aging_compaction.rs 已入库。 |
| F46 | .claude/kb/experiments.md:35 | 要改 | 「仍欠后台整理」索引行改成：状态列「第一段跑了 4 片（共 21 片）」；说明列把「文件系统已有，仍欠后台整理」换成「装置 scripts/experiment-apparatus/e10_aging_compaction.rs、分片驱动 e10-aging-compaction-run.sh、21 行变异表（抓到 20、无效 1）已入库；第一段 21 片跑了 4 片，读数只当线索，剩 17 片停下，登记的模型与判据要修订三处（records/2026-10-13-里程碑四预开发.md 第 146 行）」。「仍欠后台整理」也过时：crates/singlefs-core/src/compaction_plan.rs 与 publish_relocation_batch 已在 0b926bd3f 入库。页首标题同样写「未跑」。 |
| F46 | .claude/kb/milestone/04-fourth-txn.md:25 | 不相干 | 「的跑前登记留在二·1，出口编号不改」这一句说长期老化挪出之后 E10 跑前登记仍留在二·1，说的是登记放在哪、出口编号动不动；跑前登记 research/prompts/e10-r1-prereg.md 确实写成了，句子不受 E10 装置入库与跑了 4 片影响。 |
| F46 | .claude/kb/milestone/04-fourth-txn.md:34 | 不相干 | 「含 E10」这一格是作业点列表里二·1 设计轮包含什么，不是 E10 的状态；这一行的状态格「未开」属于里程碑四分组状态那件事实（F31 管），这一行对 E10 本身没有现状断言。 |
| F46 | .claude/kb/milestone/04-fourth-txn.md:59 | 要改 | 「E10（老化：结构抗老化 vs 事后整理） 未跑」把「E10 未跑」改成「E10 第一段 21 片跑了 4 片，读数只当线索」；把「E10 没有跑前登记（research/prompts 下 e10-* 零个），实验页里没有数值门槛」改成「E10 跑前登记是 research/prompts/e10-r1-prereg.md（二·1 写成），数值门槛写在登记里、实验页上没有」。现查 ls research/prompts 有 e10-r1-prereg.md，「零个」已不成立。同一行里「整理三条水位第一版恒 0」「整理未实现」等句也与整理已入库不符，归整理实现那件事实，这里不改写。 |
| F46 | .claude/kb/milestone/04-fourth-txn.md:64 | 不相干 | 「三条水位与 `N₀` 的取值」这是二·1 作业点定义：做什么写「E10 跑前登记」是取值的出处、验收写「E10 跑前登记写成」是交付物；登记已写成，这些句子说的是作业内容，不是 E10 现状。「搬共享块那一问等一·1」「意图形态等里程碑三」属于别的事实。 |
| F46 | .claude/kb/milestone/09-ninth-txn.md:23 | 不相干 | 「落在 E10」这一格说长期老化的门槛落在 E10 跑前登记写死的数里，说的是门槛由谁定，登记已写成，不受 E10 装置入库影响。 |
| F46 | .claude/kb/milestone/09-ninth-txn.md:119 | 要改 | 「实验索引里状态是「未跑」」改成「实验索引里 E10 的状态格现写「未跑」，实际装置已入库、第一段 21 片跑了 4 片、读数只当线索；索引与实验页待同步」，或索引行改完之后写「实验索引里状态是「第一段跑了 4 片」」。现查 .claude/kb/experiments.md 第 35 行仍写「未跑」，与 research/results/e10-aging-compaction-2026-10-13/ 下 4 个产物不符。 |
| F46 | .claude/kb/milestone/09-ninth-txn.md:124 | 不相干 | 「门槛由 E10」这是里程碑九五·1 的作业点：门槛由 E10 跑前登记写死，不在这份文件里拍数；说的是谁定门槛，不受 E10 装置入库与跑了 4 片影响。 |
| F46 | .claude/kb/milestone/09-ninth-txn.md:126 | 不相干 | 「长跑多久算够」这一句要先定的是长跑多久算够、照 E10 跑前登记，登记已写成，句子是开工前的待办，不是 E10 的运行状态。 |
| F46 | records/2026-10-13-里程碑四预开发.md:25 | 事件句不改 | 「两轮的正文写成、腿派出、判决写成」这是 2026-10-13 里程碑四预开发记录里第 1 步当时的计划（正文写成、腿派出、判决写成），说的是那一次安排；把它换成 E10 现状这句也不会变真，不改。 |
| F47 | .claude/kb/checks-owed.en.md:366 | 要补 | 「e156-alloc-basis-counts-2026-10-05-r2.out」E156 第 7 次重跑（2026-10-18，产物 e156-alloc-basis-counts-2026-10-18.out）量了 C566：洞数从 1 档扫到环满一圈再跨圈，Q1a 峰值读法甲、乙下都判「仍在涨」，上界数全落在最大档（至多端点、没量到上界）。这一行（英文版）只写到第 5、6 次重跑，应在 C566 的依据里补一句：「第 7 次重跑（2026-10-18，e156-alloc-basis-counts-2026-10-18.out）洞数扫到环满一圈再跨圈，Q1a 峰值仍在涨、没量到上界」。第 4、5、6 次的读数是那几次的事，不改。 |
| F47 | .claude/kb/checks-owed.ja.md:366 | 要补 | 「e156-alloc-basis-counts-2026-10-05-r2.out」同 1360（日文版）：C566 的依据只写到第 5、6 次重跑；补一句 E156 第 7 次重跑（2026-10-18）洞数扫到环满一圈再跨圈、Q1a 峰值读法甲乙都判仍在涨、上界数落在最大档、没量到上界（页内 ### 第 7 次重跑 一节）。 |
| F47 | .claude/kb/checks-owed.md:365 | 要补 | 「第 5、6 次重跑（2026-10-05，新 `crates/`」C566 的依据补一句：「第 7 次重跑（2026-10-18，e156-alloc-basis-counts-2026-10-18.out）洞数从 1 档扫到环满一圈再跨圈，Q1a 峰值读法甲、乙下都判仍在涨，上界数全落在最大档（至多端点），没量到上界」。现查 .claude/kb/checks-owed.md 第 365 行全行 2641 字节，grep 不到「第 7 次」；E156 页第 20 行起那一节写了这个读数。第 4、5、6 次的读数是那几次的事，不改。 |
| F47 | .claude/kb/experiments.md:180 | 不相干 | 「2026-10-18 第 7 次重跑（问题单 H1）」这一行已经写了第 7 次重跑（2026-10-18）和洞数扫到环满一圈，是改后的样子，不受「replay.sh 指第 6 次产物」那件事实影响。 |
| F47 | .claude/kb/experiments.md:189 | 不相干 | 「第 6 次重跑已跑（2026-10-09：r3 档合法一族）」这一行是 E165 的重跑清单，「第 6 次重跑」是 E165 自己的第 6 次，不是 E156；E156 第 7 次重跑与它无关。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:1 | 不相干 | 「第 5 次重跑：core 断言之后 W7 那一格的阳性对照换成 PC-钉门槛」这一行是 E156 页标题行，现查第 1 行已含「第 7 次」，写的已是改后的样子；摘到的「第 6 次」片段只是标题里按时间罗列的各次重跑。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:254 | 不相干 | 「q1d_growth=not_monotonic」这一段在页内「第 7 次重跑」一节（第 20 行起、到第 269 行止）里，整段已按第 7 次产物写（行号 :16564、:16657 是 16669 行的新产物），第 6 次产物只作对照被点名。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:270 | 事件句不改 | 「第 6 次重跑（2026-10-05）：回收门槛照定乙读之后 38 格 K10、F24 都是 0」这是页内第 6 次重跑那一节的标题，说那一次跑出了什么；换成第 7 次的说法这句就不真，是事件句，不改。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:285 | 事件句不改 | 「5819 行 → 6205 行」这一段在第 6 次重跑一节里，写那一次把装置从 5819 行改到 6205 行、19 条单测；第 7 次的 7580 行 25 单测写在第 7 次一节，这一句说的是第 6 次那一趟。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:287 | 事件句不改 | 「主产物 `e156-alloc-basis-counts-2026-10-05-r2.out`（4063 行」这一段在第 6 次重跑一节里，写第 6 次的主产物 r2.out（4063 行）与它当时怎么跑，是那一次的事；replay.sh 现改指 2026-10-18 产物写在页内「复跑」一节。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:296 | 事件句不改 | 「38 格是 S ∈ {4, 8, 16} × ρ ∈ {1, 1/4} 的 k = 0 六格」这是第 6 次重跑那一节里对 r2.out 的逐格命令与读数（38 格 K10、F24 为 0），说的是第 6 次产物上量到的，产物仍在，不受新产物影响。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:417 | 事件句不改 | 「第 5 次重跑与页内第 4 次各节的关系」这一段在页内第 5 次重跑一节，讲第 5 次产物与第 4 次各节的关系，并点到第 6 次照定乙读之后的读数，说的是那几次重跑之间的事，不是 replay.sh 当前登记哪一份。 |
| F47 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:604 | 不相干 | 「现在指 `e156-alloc-basis-counts-2026-10-18.out`」「复跑」一节这一句已经写 replay.sh 的 E156 行现在指第 7 次产物 2026-10-18.out，是改后的样子。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1 | 不相干 | 「第 6 次重跑已跑（2026-10-09：r3 档合法一族）」这是 E165 页标题，「第 6 次重跑」是 E165 自己的，与 E156 的重跑次数无关。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1210 | 不相干 | 「r3 档合法一族的正式跑，2026-10-09」这是 E165 页的小节标题，说 E165 的第 6 次重跑，不是 E156。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1304 | 不相干 | 「e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out」这是 E165 在 replay.sh 里登记的驱动行，与 E156 的登记行无关。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1325 | 不相干 | 「J6Qn 与 j7 五格对照钉值的独立算法（第 6 次重跑）」这是 E165 第 6 次重跑的路径 16，与 E156 无关。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1341 | 不相干 | 「路径 16 的两条路逐个对上」这是 E165 第 6 次重跑的对照结果，与 E156 无关。 |
| F47 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1345 | 不相干 | 「r3 档合法一族的对照跑了、全红」这是 E165 第 6 次重跑的局限说明，与 E156 无关。 |
| F47 | .claude/kb/milestone/03-third-txn.md:186 | 不相干 | 「E165（恢复重放有界穷举与变异杀伤界） r3 档只跑对抗一族」这一行写 E165 补第 6 次重跑的欠账，不是 E156 的重跑。 |
| F47 | .claude/kb/milestone/03-third-txn.md:187 | 不相干 | 「E165（恢复重放有界穷举与变异杀伤界） 单测单元表 15 处」这一行写 E165 注释里写死 replay.sh 行号的问题，与 E156 第 7 次重跑无关。 |
| F47 | records/2026-09-30-里程碑三开工第一轮.md:914 | 不相干 | 「形式化线交接（2026-10-09」这一条写 E165 第 46 批与形式化线交接，与 E156 的重跑次数无关。 |
| F47 | records/2026-09-30-里程碑三开工第一轮.md:918 | 不相干 | 「形式化线第 46 批（E165 第 6 次重跑）交给根环线」这一条写 E165 第 6 次重跑的交接，与 E156 无关。 |
| F47 | records/2026-10-01-里程碑三形式化接入.md:1152 | 事件句不改 | 「33-2 E156 第 6 次交回」这是 2026-10-01 记录里 E156 第 6 次交回的当时报告（认定追加在 e156-r6-prereg.md 末尾），说那一次发生的事；换成第 7 次的说法就不是真话。 |
| F47 | records/2026-10-01-里程碑三形式化接入.md:1399 | 不相干 | 「第 46 批（2026-10-09）：E165 第 6 次重跑」这是记录里的批次标题，E165 的第 6 次重跑，与 E156 无关。 |
| F47 | records/2026-10-01-里程碑三形式化接入.md:1427 | 不相干 | 「第 46 批（E165 第 6 次重跑，给 r3 补合法一族正式跑）」这一条写 E165 第 46 批的分工，与 E156 无关。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:11 | 要改 | 「装置源码原样留在产物目录的 `device-source.rs`」「装置源码原样放在 `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`（不在产物目录里，research/ 整个可删，装置不放那里）」。现查：scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/ 下有 device-source.rs，产物目录只剩 .out 文件。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:36 | 不相干 | 「cp $PWD/scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs」这条复现命令已经从 scripts/experiment-apparatus/ 下的新路径拷装置，是改后的样子。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:46 | 不相干 | 「放进 device-source.rs」这一行说复跑把 device-source.rs 放进副本，只点文件名不点目录，搬家不影响它的真假。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:56 | 不相干 | 「不在这个产物目录里，搬进了 `scripts/experiment-apparatus/`」这一行已经写「搬进了 scripts/experiment-apparatus/」，是改后的样子；此刻 sha256 cdb5d7be… 与现查 sha256sum 输出相同。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:174 | 不相干 | 「`grep -c '^#\[test\]' device-source.rs` 输出 28」这一行只写文件名 device-source.rs 与单测数；现查 grep -c 在新路径上也是 28，不受装置搬家影响。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:197 | 不相干 | 「scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:713」这一行引的已是 scripts/experiment-apparatus/ 下的新路径，是改后的样子。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:198 | 不相干 | 「scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:723」这一行引的已是 scripts/experiment-apparatus/ 下的新路径，是改后的样子。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:199 | 不相干 | 「scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:1028」这一行引的已是 scripts/experiment-apparatus/ 下的新路径，是改后的样子。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:200 | 不相干 | 「scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs:1158」这一行引的已是 scripts/experiment-apparatus/ 下的新路径，是改后的样子。 |
| F48 | .claude/kb/experiments/175-产品路径上inode号水位退回与重发号.md:202 | 不相干 | 「`device-source.rs` 里的计数 `s3_path_disagreements`」这一行只写文件名与计数，不点目录；两路比较 0 次的读数不受装置搬家影响。 |
| F48 | records/2026-10-18-审查报告核实与修复.md:125 | 事件句不改 | 「E175 装置源码搬到 `scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs`」这是 2026-10-18 记录里丙簇交回的当时报告，写那一次把装置源码搬走（文件系统 mv），说的是那次做了什么，不改。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:1 | 事件句不改 | 「第二十五次跑：装置补上 mkfs 每盘」这是页标题里按次数罗列各次跑出了什么，写的是第二十五次跑当时的读数（mkfs 行、29 个区域逐字节全相同等）；换成取号写两轮的 61 次操作、16777268 这句也不会变真，是事件句，不改。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:22 | 要改 | 「补第三态后全量 16777260，全量由」「崩溃点重放主臂本页装置不枚举（`name=layer0 … states=not_run closed_form=16777240`：这是装置按取号写一轮量的，两态闭式 16777240，补第三态后全量 16777260；取号写两轮之后 crates 里枚举的是 61 次操作、全量 16777268，E142 装置没按两轮重跑，全量由 `crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs` 在门禁 checker-tier-crash-point-replay 里枚举）」。现查 .claude/kb/layout/01-first-txn.md 第 420 行已写「系统配置槽写取三态时是 16777268 … E142 装置没按取号写两轮重跑」。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:202 | 事件句不改 | 「第十九次跑第一段（问题单」这是第十九次跑第一段的问题与结论（三态 16777260 与 crates 当时钉的相同），说那一次量到的，不是今天的流；不改。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:225 | 事件句不改 | 「E7RESULT name=segments_three_state path=post_mkfs_stream」这是产物原行（writes=41、16777260），装置按取号写一轮跑出来的，产物逐字抄录，换新数就不是产物了；不改。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:238 | 事件句不改 | 「Q142.47 三态」这是第二十五次跑的判据行，当时「整条流 16777260 与 crates 钉的相同」是那次比对的结果；crates 现钉 16777268 是之后取号写两轮改的，那次比对仍是真话，是事件句，不改。 |
| F58 | .claude/kb/experiments/142-新池新建文件的干跑.md:310 | 要改 | 「今天的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260」「E142 装置量到的新池新建文件流是 12 单元、两态闭式 16777240、补第三态全量 16777260（取号写一轮）；今天 crates 里的流取号写两轮，61 次操作、全量 16777268，本页装置不枚举它，这两格在它身上是不是还恒为 0 没有读数」。现查 crash_enumeration_new_pool_file_creation_stream.rs 的 FULL_STATES 与 layout 第 420 行都是 16777268。 |
| F58 | .claude/kb/experiments/161-崩溃放量的去重与分段耗时.md:125 | 事件句不改 | 「E7RESULT name=anchor_stream stream=first」这是 E161 产物原行（domain_states=16777260 等），当时按取号写一轮量的，产物原样抄录；不改。 |
| F58 | .claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:24 | 事件句不改 | 「S1 的 `24+2+1+2`、种类串、35 次操作」这是 E167 停机条款当次真跑的结果（57 次操作、16777240 个状态，模型与登记对上），说那一次发生的事；不改。 |
| F58 | .claude/kb/experiments/170-O3规约执行器第一版.md:44 | 事件句不改 | 「E7RESULT name=stream stream=new_pool_file_creation operations=57」这是 E170 产物原行（operations=57、full_states=16777260），跑出来的原样，不改。 |
| F58 | .claude/kb/experiments/171-三oracle合谋测试.md:298 | 事件句不改 | 「E7RESULT name=stream stream=new_pool_file_creation ending=completed」这是 E171 产物原行（operations=57、full_states=16777260），跑出来的原样，不改。 |
| F58 | .claude/kb/experiments/184-常规层与大负载层的代价计数.md:208 | 事件句不改 | 「冒烟层新池新建文件流有 267 块、全量态数 16777260」这是 E184 这一次量到的冒烟层读数（共用 266 块），产物里的数；取号写两轮是之后 crates 的改动，E184 装置没按两轮重跑，这句仍是那次读数的真话，不改。 |
| F58 | .claude/kb/layout/01-first-txn.en.md:419 | 不相干 | 「consecutive barriers on same disk merged into one」这一行（英文版）已经写取号写两轮、系统配置槽写取三态时 16777268，产物那一行是取号写一轮时的 16777260，是改后的样子。 |
| F58 | .claude/kb/layout/01-first-txn.ja.md:416 | 不相干 | 「16777243 状態」这一行（日文版）已经写取号写两轮后的 16777268 与产物是取号写一轮时的数，是改后的样子。 |
| F58 | .claude/kb/layout/01-first-txn.md:420 | 不相干 | 「16777243 个状态」这一行已经写 61 次操作、16777268，产物那一行是取号写一轮时的 16777260，E142 装置没按两轮重跑，是改后的样子。 |
| F58 | records/2026-09-24-里程碑二收尾调度.md:315 | 事件句不改 | 「分片预演：跑、中断、续跑（2026-09-28）」这是 2026-09-28 分片预演的记录，写那一次用 crash-case:new-pool-file-creation-stream 跑了什么，说的是当时；不改。 |
| F58 | records/2026-10-01-删除与并发事务.md:774 | 事件句不改 | 「2026-10-05 五·7 e161 调查交回」这是 2026-10-05 调查交回的当时结论（D1-small 从 37 变 16777260），事件句，不改。 |
| F58 | records/2026-10-11-崩溃放量两段与脚本审计.md:68 | 事件句不改 | 「2026-10-12：续跑 m 的崩溃点重放 步」这是 2026-10-12 续跑的当时读数（states=16777260），事件句，不改。 |
| F58 | records/2026-10-13-复用键与全仓改名.md:939 | 事件句不改 | 「重起后第 1 条 new-pool-file-creation 的复用」这是 2026-10-13 重起后量到的复用读数（states=16777260），事件句，不改。 |
| F58 | records/2026-10-13-欠账还五条.md:44 | 事件句不改 | 「课题 2、3（C788、C793）交回」这是 2026-10-13 课题交回的记录，「态数最小 16777260」是那次登记里四条的读数，事件句，不改。 |
| F58 | records/2026-10-18-里程碑三收尾与近期欠账.md:431 | 事件句不改 | 「批 9 甲交回」这是批 9 甲交回的当时报告，正是它把钉值按取号写两轮从 16777260 重算到 16777268，说那一次做了什么，不改。 |
| F59 | .claude/kb/checks-owed.en.md:365 | 要改 | 「已登记 14 条」改成「gate checker-tier-crash-point-replay 的 `--full` 已登记 35 条crash enumeration case (…`grep -c '^crash-case:'` 现数)」。现查 grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv 输出 35（基准 1bb3a24dc 那一版是 14）。标题「另外 6 条」是另一个量，登记 35 条之后还剩几条没人跑要现核，这里不替它改。 |
| F59 | .claude/kb/checks-owed.ja.md:365 | 要改 | 「已登记 14 条」改成「ゲート checker-tier-crash-point-replay 的 `--full` 已登记 35 条クラッシュ列挙ケース」。现查 grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv 输出 35。标题「另外 6 条」要现核后再定。 |
| F59 | .claude/kb/checks-owed.md:364 | 要改 | 「已登记 14 条崩溃枚举用例」改成「门禁 checker-tier-crash-point-replay 的 `--full` 已登记 35 条崩溃枚举用例（`.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，`grep -c '^crash-case:'` 现数）」。现查该命令输出 35。标题与「仍欠两样」里的「另外 6 条」是另一个量，35 条之后还剩几条没人跑要现核，这里不替它改。 |
| F59 | .claude/kb/milestone/03-third-txn.en.md:141 | 要改 | 「identity and reuse keys in」改成「Registered streams count 35 (`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` count; 14 streams registered at the time of Milestone 3 listed below, …)」。现查该命令输出 35，不是 29；括号里「当时登记的 14 条」是那时的事，不改。 |
| F59 | .claude/kb/milestone/03-third-txn.en.md:146 | 不相干 | 「compared against」这一格只写数登记流的命令，没写条数；命令现数 35 不影响这句话的真假。 |
| F59 | .claude/kb/milestone/03-third-txn.ja.md:141 | 要改 | 「三層分類は D13」改成「登録済みストリーム 35 本（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 実数カウント；以下はマイルストーン三当時に登録された 14 本、…）」。现查该命令输出 35，不是 29；「当時に登録された 14 本」是那时的事，不改。 |
| F59 | .claude/kb/milestone/03-third-txn.ja.md:146 | 不相干 | 「登録済みストリーム（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 実数カウント）」这一格只写数登记流的命令，没写条数；命令现数 35 不影响这句话的真假。 |
| F59 | .claude/kb/milestone/03-third-txn.md:141 | 要改 | 「登记的流 29 条」改成「登记的流 35 条（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 现数；下面列的是里程碑三当时登记的 14 条，其后加的流看该文件里各 `crash-case:` 行）」。现查该命令输出 35，不是 29；「当时登记的 14 条」是那时的事，不改。 |
| F59 | .claude/kb/milestone/03-third-txn.md:146 | 不相干 | 「对着登记流（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 现数）」这一格只写数登记流的命令，没写条数；命令现数 35 不影响这句话的真假。 |
| F59 | .claude/kb/verification-build.en.md:6 | 要改 | 「depending on no other crates」改成「35 registered crash enumeration test cases counted via `grep -c '^crash-case:' …`」；同一句里新池新建文件流的 16,777,260 与闭式 16,777,240 也过时：crash_enumeration_new_pool_file_creation_stream.rs 的 FULL_STATES 现为 16_777_268、每次写只取两态的闭式是 16_777_243（取号写两轮，61 次操作）。现查 grep -c '^crash-case:' 输出 35。 |
| F59 | .claude/kb/verification-build.en.md:7 | 要改 | 「only after that stage runs and passes」改成「(counted via `grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`, currently 35 items)」。现查该命令输出 35。 |
| F59 | .claude/kb/verification-build.ja.md:6 | 要改 | 「呼び出しクロージャの収集」改成「登録されたクラッシュ列挙用例35件」，同一句里新プール新規作成ファイルストリームの「全 16,777,260 クラッシュ状態」「閉形式は 16,777,240」也改成 16,777,268 与 16,777,243（FULL_STATES 现为 16_777_268）。现查 grep -c '^crash-case:' 输出 35。 |
| F59 | .claude/kb/verification-build.ja.md:7 | 要改 | 「現在14件」改成「現在35件」。现查 grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv 输出 35。 |
| F59 | .claude/kb/verification-build.md:6 | 要改 | 「算判器组单元键」改成「登记的崩溃枚举用例 35 条」；同一句里新池新建文件流的「全部 16 777 260 个崩溃状态」「闭式是 16 777 240」也改成 16 777 268 与 16 777 243，固定脚本流的「1 678 377 204」改成 1 678 377 220（crash_enumeration_fixed_script_stream.rs 的 FULL_STATES_THROUGH_THE_UNMOUNT 现为 1_678_377_220）。现查 grep -c '^crash-case:' 输出 35。 |
| F59 | .claude/kb/verification-build.md:7 | 要改 | 「现 14 条」改成「（`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 数出，现 35 条）」。现查该命令输出 35。 |
| F59 | CLAUDE.md:113 | 不相干 | 「`grep -c '^crash-case:'` 数，含那两条流」这一句只写数登记用例的命令，没写条数；命令现数 35 不影响它的真假。同一行里「checker 判 103 条不变量」「崩溃放量的 8 项」是另外的量，不归这件事实。 |
| F60 | .claude/kb/checks-owed.en.md:425 | 不相干 | 「root selection核 choose_root_slot 与取prefix核 replay_core」这一行（英文版）写 C650 里 Verus 一侧用 choose_root_slot 与 replay_core 合成推论，必红世界 7 行；现查 grep -c "^C650" verus-must-red-worlds.tsv 输出 7。说的是合判这一笔，不是 singlefs-replay-prefix 里只有哪几样证明，不受 compaction_plan_proofs 加入影响。 |
| F60 | .claude/kb/checks-owed.en.md:429 | 不相干 | 「必红世界期望条数随求解环境漂」这一行（英文版）写 C654：必红世界每行 proof_errors 条数随求解环境漂，举的是攻方在核上加一个没人用的 spec fn 的例子，是那次现象；表里追加变异行不改变这条欠账的真假。 |
| F60 | .claude/kb/checks-owed.ja.md:425 | 不相干 | 「ルート選択核 choose_root_slot 与取プレフィックス核 replay_core」这一行（日文版）写 C650 的合判，必红世界 7 行（现查仍是 7），不是 singlefs-replay-prefix 里证明的清单，不受 compaction_plan_proofs 加入影响。 |
| F60 | .claude/kb/checks-owed.ja.md:429 | 不相干 | 「必红世界期望条数随求解环境漂」这一行（日文版）写 C654 条数随求解环境漂的现象与判法，追加变异行不影响它。 |
| F60 | .claude/kb/checks-owed.md:422 | 不相干 | 「择根核 `choose_root_slot` 与取前缀核 `replay_core` 合成推论」这一行写 C650 的合判（已有的 Verus 一侧、E165 一侧），必红世界 7 行现查仍是 7；说的是这一笔的证明，不是 singlefs-replay-prefix 只有哪几样证明。 |
| F60 | .claude/kb/checks-owed.md:426 | 不相干 | 「必红世界期望条数随求解环境漂」这一行写 C654 条数随求解环境漂的现象与判法（按落点判、条数只报），表里追加的变异行不改变它。 |
| F60 | .claude/kb/milestone/03-third-txn.en.md:157 | 不相干 | 「registering 78 verified (two kernel files」这一行（英文版）已经写 verus-pins.tsv 登记 78 verified、核两份（lib.rs 与 compaction_plan_proofs.rs）；现查 .claude/gate.d/verus-pins.tsv 第 27-29 行：crate 78、kernel 两份，对得上，是改后的样子。 |
| F60 | .claude/kb/milestone/03-third-txn.ja.md:157 | 不相干 | 「78 verified を登録（二つのカーネルファイル」这一行（日文版）已经写登记 78 verified、核两份，现查 verus-pins.tsv 第 27-29 行对得上，是改后的样子。 |
| F60 | .claude/kb/milestone/03-third-txn.md:157 | 不相干 | 「登记 78 verified（核两份：`lib.rs` 与 `compaction_plan_proofs.rs`）」这一行已经写核两份含 compaction_plan_proofs.rs、78 verified；现查 verus-pins.tsv 第 27-29 行对得上，是改后的样子。 |
| F60 | .claude/kb/milestone/05-fifth-txn.en.md:50 | 不相干 | 「P1 segment selection soundness, P2 plan soundness」这一行（英文版）写里程碑五借用 compaction_plan_proofs.rs 的 P1 至 P4，该文件现查在 crates/singlefs-replay-prefix/src/ 下，与这件事实一致。 |
| F60 | .claude/kb/milestone/05-fifth-txn.en.md:120 | 不相干 | 「P4 of `crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs`」这一行（英文版）把 compaction_plan_proofs.rs 的 P4 当既有依据引用，与这件事实一致（文件现查存在）。 |
| F60 | .claude/kb/milestone/05-fifth-txn.ja.md:50 | 不相干 | 「P1 セグメント選択健全性、P2 計画健全性」这一行（日文版）引 compaction_plan_proofs.rs 的 P1 至 P4，与这件事实一致（文件现查存在）。 |
| F60 | .claude/kb/milestone/05-fifth-txn.ja.md:120 | 不相干 | 「の P4」这一行（日文版）把 compaction_plan_proofs.rs 的 P4 当既有依据，与这件事实一致。 |
| F60 | .claude/kb/milestone/05-fifth-txn.md:50 | 不相干 | 「P1 选段健全、P2 计划健全、P3 进展、P4 抽象恒等引理」这一行写里程碑五借用 compaction_plan_proofs.rs 的 P1 至 P4，与这件事实一致（文件现查存在）。 |
| F60 | .claude/kb/milestone/05-fifth-txn.md:120 | 不相干 | 「把 P4 延伸到容器槽」这一行把 compaction_plan_proofs.rs 的 P4 当既有依据，与这件事实一致。 |
| F60 | .claude/kb/verification-build.md:6 | 要改 | 「恢复取 journal 前缀、择根、扫环建表的纯函数核与它的 Verus 证明」改成「`singlefs-replay-prefix`（恢复取 journal 前缀、择根、扫环建表的纯函数核与它的 Verus 证明，另有 `compaction_plan_proofs.rs` 证整理计划的选段健全、计划健全、进展与抽象恒等，只依赖 `vstd`，`singlefs-core` 依赖它）」。现查 crates/singlefs-replay-prefix/src/ 下有 lib.rs 与 compaction_plan_proofs.rs，.claude/gate.d/verus-pins.tsv 第 28、29 行登记两份核；这一行（所在整段第 6 行）没提 compaction_plan_proofs。 |
| F60 | records/2026-09-30-里程碑三开工第一轮.md:947 | 事件句不改 | 「N4 主 agent 做：`git mv crates/singlefs-replay-prefix/verus-mutations.tsv」这是 2026-09 记录里 N4 改名那一次做了什么，说的是当时的事，不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:186 | 事件句不改 | 「Verus 交回（Opus，28 分，39.0 万 token」这是 2026-10-13 记录里 Verus 写段交回的当时报告（证 P1 至 P4、变异 7 行、56 → 78），正是新增 compaction_plan_proofs 这件事的经过；不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:253 | 事件句不改 | 「丁交回（Opus，35 分，49.9 万 token」这是 2026-10-13 记录里丁交回的当时报告，只顺带提到段登记表摘要；事件句，不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:306 | 事件句不改 | 「截丙交回（Opus，45 分，61.6 万 token」这是 2026-10-13 记录里截丙交回的当时报告，提到形式化轻型两格红与基线副本同形，说的是那一次；事件句，不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:351 | 事件句不改 | 「master 形式化门禁与 `compaction_plan_proofs` 一族」这是 2026-10-13 记录里待办表的一行，状态「下轮开」「合回 master 之前」是那天的安排；之后 2026-10-18 记录（第 59 行）已写 compaction_plan_proofs 一族全部判绿。按日期不回改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:390 | 事件句不改 | 「另报形式化 verus-proof 格在副本上红 20 项」这是 2026-10-13 记录里验证段当时的另报，说那次红 20 项；事件句，不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:409 | 事件句不改 | 「形式化 verus-proof 在 `compaction_plan_proofs.rs` 上红 20 项没查」这是 2026-10-13 记录里当天没做的事的清单，事件句，不改。 |
| F60 | records/2026-10-13-里程碑四预开发.md:434 | 事件句不改 | 「`verus-proof-check.py` 对子模块要用 `--verify-module`」这是 2026-10-13 记录里当时查到的根因，之后 2026-10-18 记录又改成 --verify-only-module；事件句，不改。 |
| F60 | records/2026-10-18-审查报告核实与修复.md:17 | 事件句不改 | 「一·4 verus-proof-check.py 一律 `--verify-root`」这是 2026-10-18 记录里对审查报告一条的核实结果，说那次核到了什么；事件句，不改。 |
| F60 | records/2026-10-18-审查报告核实与修复.md:57 | 事件句不改 | 「verus 工具实现员交回（Opus，23 分，25.3 万 token」这是 2026-10-18 记录里 verus 工具实现员交回的当时报告（29 项全红改后全绿，和 78 等于登记 78），事件句，不改。 |
| F60 | records/2026-10-18-审查报告核实与修复.md:61 | 事件句不改 | 「主工作区形式化轻型（`--quick`，内存包装 32G）」这是 2026-10-18 记录里主工作区形式化轻型当时跑出的结果（成项 69、之和 78、登记 78），事件句，不改。 |
| F64 | .claude/kb/checks-owed.en.md:660 | 不相干 | 「已登记进gate code-tooling 的 research-script-selftests 格复跑」这一行（英文版）写 C44 引用核到的树；其中 `~/kbuild/linux-om` 是 singlefs 的 scripts/kb/verify-citations.sh 读的内核树，登记在 .claude/out-of-project-paths（现查第 4 行），放行的路径不在改泛称之列。 |
| F64 | .claude/kb/checks-owed.ja.md:660 | 不相干 | 「`~/kbuild/linux-om`（现查 7.2.0）」这一行（日文版）的 `~/kbuild/linux-om` 是登记放行的路径（.claude/out-of-project-paths 现查第 4 行），脚本 verify-citations.sh 读它，不属于要改成泛称的仓外目录名。 |
| F64 | .claude/kb/checks-owed.md:661 | 不相干 | 「`~/kbuild/linux-om`（现查 7.2.0）」这一行的 `~/kbuild/linux-om` 是登记放行的路径（.claude/out-of-project-paths 现查第 4 行），脚本 verify-citations.sh 读它，不属于要改成泛称的仓外目录名。 |
| F64 | .claude/kb/prior-art.md:731 | 不相干 | 「`/home/fy5090/linux-bug-fix/linux`（v7.3-rc1，浅克隆）」这一行写的是没拿来引的两棵旧克隆；其中 linux-bug-fix/linux 与 code/fs-refs 都是 .claude/out-of-project-paths 登记放行的路径（现查），脚本 fetch-deps.sh、fetch-refs.sh 读它们，不属于要改成泛称的目录名。 |
| F64 | .claude/kb/tooling.en.md:87 | 不相干 | 「`~/kbuild/linux-om` (1.8 GB)」这一行（英文版）写工具表里的内核树路径，`~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查），改的是用户级可执行文件目录与 shell 启动文件，不是这一行。 |
| F64 | .claude/kb/tooling.en.md:92 | 不相干 | 「Inspect source code using `~/kbuild/linux-om`」这一行（英文版）的 `~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），不属于要改成泛称的目录名。 |
| F64 | .claude/kb/tooling.ja.md:87 | 不相干 | 「`~/kbuild/linux-om`（1.8 GB）」这一行（日文版）的 `~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），不属于要改成泛称的目录名。 |
| F64 | .claude/kb/tooling.ja.md:92 | 不相干 | 「ソースコードの調査には `~/kbuild/linux-om` を使用し」这一行（日文版）的 `~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），不属于要改成泛称的目录名。 |
| F64 | .claude/kb/tooling.md:87 | 不相干 | 「`~/kbuild/linux-om`（1.8 GB）」这一行的 `~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），不属于要改成泛称的目录名。 |
| F64 | .claude/kb/tooling.md:92 | 不相干 | 「查源码用 `~/kbuild/linux-om`，不用 `/usr/src/linux-*-headers-*`」这一行的 `~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），不属于要改成泛称的目录名。 |
| F64 | .claude/kb/verification-build.en.md:159 | 不相干 | 「kernel tree `~/linux-bug-fix/linux`」这一行（英文版）的 `~/linux-bug-fix/linux` 是 .claude/scripts/fetch-deps.sh 与 lkmm.sh 取 LKMM 模型文件的内核树，登记放行（.claude/out-of-project-paths 现查第 5 行）。 |
| F64 | .claude/kb/verification-build.ja.md:159 | 不相干 | 「カーネルツリー `~/linux-bug-fix/linux`」这一行（日文版）的 `~/linux-bug-fix/linux` 登记放行（.claude/out-of-project-paths 现查第 5 行）。 |
| F64 | .claude/kb/verification-build.md:203 | 不相干 | 「内核树 `~/linux-bug-fix/linux`」这一行的 `~/linux-bug-fix/linux` 登记放行（.claude/out-of-project-paths 现查第 5 行）。 |
| F64 | records/2026-08-28-原子性与日志定案.md:68 | 不相干 | 「`~/kbuild/linux-om`（Linux 7.2.0」这一行是 2026-08-28 记录里的内核树路径，`~/kbuild/linux-om` 登记放行（.claude/out-of-project-paths 现查第 4 行），而且是当时记下的版本事实。 |
| F64 | records/2026-09-16-subagent拆分提案.md:943 | 不相干 | 「VSCode 服务端日志 `<VSCode 服务端目录>/data/logs/2026-09-22/remoteagent.log`」这一行里 VSCode 服务端目录已经写成泛称 <VSCode 服务端目录>，是改后的样子；另一处 `~/.vscode-server` 是登记放行的路径（.claude/out-of-project-paths 现查），整行是 2026-09-25 的事故记录。 |
| F64 | records/2026-10-18-审查报告核实与修复.md:101 | 不相干 | 「research/results 产物」这一行是 2026-10-18 普查交回，其中 `~/code/fs-refs` 与 `~/kbuild/linux-om` 都登记放行（.claude/out-of-project-paths 现查第 4、6 行），不属于要改成泛称的目录名。 |
| F65 | .claude/kb/checks-owed.en.md:48 | 不相干 | 「item引用只验状态不验身份」这一行（英文版）写 C51：item-ref-status 格只比状态、不比身份；它说的比对口径不因译文取同语言文件或认变更史节标题而变，仍只比状态。 |
| F65 | .claude/kb/checks-owed.en.md:58 | 不相干 | 「item定案值在 kb 里分叉」这一行（英文版）写 C62 定案值分叉的现象，item-ref-status 只出现在审核实测的旁证里，不受取文件口径与节标题认法影响。 |
| F65 | .claude/kb/checks-owed.en.md:203 | 要改 | 「仍按三级rollback解析」把「仍按三级回退解析，第一级是行内最近的那个编号」改成「仍按逐级回退解析（行内紧挨着的指名、行内最近的 D 记号、文件自身的决策号、变更史日期标题上的 D 号、变更史 ## D<n>（简称） 节标题上的 D 号、之前出现过的最近的 D 记号），第一级之后是行内最近的那个编号」。现查 .claude/gate.d/lib-item-ref-status.py 文件头「归属顺序」写了六级，基准 1bb3a24dc 那一版写三级；本阶段新加的是节标题那一级。 |
| F65 | .claude/kb/checks-owed.en.md:567 | 不相干 | 「item引用状态的扫描范围漏了experiment源码」这一行（英文版）写 candidate_files 扫描范围仍含 research/**/*.rs；现查 lib-item-ref-status.py 第 205 行仍是 'research/**/*.rs', 'crates/**/*.rs', '.claude/rules/*.md'，没加 scripts/experiment-apparatus，这条欠账原样成立。 |
| F65 | .claude/kb/checks-owed.ja.md:48 | 不相干 | 「項目引用只验状态不验身份」这一行（日文版）写 C51：item-ref-status 格只比状态、不比身份，口径不因译文取同语言文件或认变更史节标题而变。 |
| F65 | .claude/kb/checks-owed.ja.md:58 | 不相干 | 「该矛盾」这一行（日文版）写 C62 定案值分叉的现象，item-ref-status 不在这段文字里，与取文件口径无关。 |
| F65 | .claude/kb/checks-owed.ja.md:203 | 要改 | 「仍按三级ロールバック解析」把「仍按三级回退解析」改成「仍按逐级回退解析（行内紧挨着的指名、行内最近的 D 记号、文件自身的决策号、变更史日期标题上的 D 号、变更史 ## D<n>（简称） 节标题上的 D 号、之前出现过的最近的 D 记号）」。现查 lib-item-ref-status.py 文件头「归属顺序」写了六级，基准那一版是三级。 |
| F65 | .claude/kb/checks-owed.ja.md:567 | 不相干 | 「項目引用状态的扫描范围漏了実験源码」这一行（日文版）写 candidate_files 扫描范围仍含 research/**/*.rs；现查 lib-item-ref-status.py 第 205 行仍是 research/**/*.rs，没加 scripts/experiment-apparatus，欠账原样成立。 |
| F65 | .claude/kb/checks-owed.md:48 | 不相干 | 「分项引用只验状态不验身份」这一行写 C51：item-ref-status 格只比状态、不比身份，口径不因译文取同语言文件或认变更史节标题而变。 |
| F65 | .claude/kb/checks-owed.md:58 | 不相干 | 「按那条判据整个矛盾判绿」这一行是 C62 的后半，item-ref-status 只出现在审核实测的旁证里（只比状态不比取值，全绿），与取文件口径和节标题认法无关。 |
| F65 | .claude/kb/checks-owed.md:203 | 要改 | 「仍按三级回退解析」把「仍按三级回退解析，第一级是**行内最近的那个编号**」改成「仍按逐级回退解析（行内紧挨着的指名、行内最近的 D 记号、文件自身的决策号、变更史日期标题上的 D 号、变更史 `## D<n>（简称）` 节标题上的 D 号、之前出现过的最近的 D 记号）」。现查 .claude/gate.d/lib-item-ref-status.py 文件头「归属顺序」写了六级，基准 1bb3a24dc 那一版写「三级都判不出的也判红」。 |
| F65 | .claude/kb/checks-owed.md:562 | 不相干 | 「仍含 `research/**/*.rs`」这一行写 candidate_files 扫描范围仍含 research/**/*.rs；现查 lib-item-ref-status.py 第 205 行仍是这个范围，没加 scripts/experiment-apparatus，欠账原样成立，不受取文件口径变化影响。 |
| F65 | .claude/kb/decisions.md:92 | 不相干 | 「状态与正文不符由门禁 doc-decisions 的 item-ref-status 格」这一句说状态与正文不符由 item-ref-status 格判红，门禁仍这样判；取同语言文件与节标题认法只改了归属与取文件，判红口径没变。 |
| F65 | .claude/kb/tooling.en.md:196 | 不相干 | 「In-line bare "Settled item k" is attributed to nearest preceding」这一行（英文版）写行内裸写的「已定项 k」归属给同一行最近的 D 记号，现查文件头归属顺序里「行内最近的 D 记号」仍在，只是行内没有 D 记号时多了节标题一级，这一句仍是真话。 |
| F65 | .claude/kb/tooling.ja.md:196 | 不相干 | 「行内の単独の「確定事項 k」が同一行内の直近の」这一行（日文版）写行内裸写的项归属给同一行最近的 D 记号，现查归属顺序里这一级仍在，仍是真话。 |
| F65 | .claude/kb/tooling.md:197 | 不相干 | 「行内裸的「已定项 k」归属给同一行最近的 `D<n>（简称）`」这一行写行内裸写的项归属给同一行最近的 D 记号，现查文件头归属顺序里这一级仍在，只是行内没有 D 记号时多了节标题一级，仍是真话。 |
| F65 | records/2026-09-03-验证三件套落地调研.md:100 | 事件句不改 | 「22 号门禁此前扫 `research/prompts/`」这是 2026-09-03 调研记录里当时查到的 22 号门禁扫描范围，说那一次的发现；不改。 |
| F65 | records/2026-09-16-subagent拆分提案.md:280 | 事件句不改 | 「改写循环扫 `.claude/kb`、`records`、`research`（不含 `prompts/`）」这是 2026-09-16 提案记录里当时核 relabel-item.py 写范围的结论，说那一次核到的；不改。 |
| F65 | records/2026-09-28-门禁59号提速与双机分片.md:21 | 事件句不改 | 「用户 2026-09-28：「20 22 24 28同规格的东西」」这是 2026-09-28 记录里的一条用户指示与当时的归并，写的是那次做了什么，不改。 |
| F65 | records/2026-09-28-门禁59号提速与双机分片.md:48 | 事件句不改 | 「16 号调 22 号的 `lib-item-ref-status.py` 判冻结层归属表点名的分项已定」这是 2026-09-28 记录里当时的门禁分组表，说那次怎么分波，事件句，不改。 |
| F65 | records/2026-10-01-删除与并发事务.md:1304 | 事件句不改 | 「门禁现状：item-ref-status 279 处」这是 2026-10-01 记录里当时的门禁红数（279 处），说那一次的现状，是事件句，不改。 |
| F65 | records/2026-10-01-对接上游测试脚本规范.md:261 | 事件句不改 | 「主 agent 带前缀跑 `doc-decisions.sh`：17 格里 kb-shape、item-ref-status、history-loss 三格红」这是 2026-10-01 记录里那一次跑门禁的结果，事件句，不改。 |
| F65 | records/2026-10-01-对接上游测试脚本规范.md:502 | 事件句不改 | 「主 agent 带前缀跑：doc-lint 对本轮改的欠账行与 D13 零命中」这是 2026-10-01 记录里那一次跑的结果（item-ref-status 是早有的红），事件句，不改。 |
| F65 | records/2026-10-11-里程碑三待优化上下游协调.md:84 | 事件句不改 | 「该会话 收尾自检转来两处」这是 2026-10-11 记录里一次收尾自检的转述（item-ref-status 多 2 处），事件句，不改。 |
| F65 | records/2026-10-11-里程碑三知识腐烂治理.md:62 | 事件句不改 | 「第一轮自检（带 `SINGLEFS_HEAVY_TESTS=user-request`」这是 2026-10-11 记录里第一轮自检当时的结果（item-ref-status 101 处在 HEAD 上同样红），事件句，不改。 |
| F65 | records/2026-10-11-里程碑三知识腐烂治理.md:90 | 事件句不改 | 「第二轮自检（同上，带 `SINGLEFS_HEAVY_TESTS=user-request`）」这是 2026-10-11 记录里第二轮自检当时的结果，事件句，不改。 |
| F65 | records/2026-10-11-里程碑三知识腐烂治理.md:164 | 事件句不改 | 「批 13 自检（文档类，带 `SINGLEFS_HEAVY_TESTS=user-request`）」这是 2026-10-11 记录里批 13 自检当时的结果（item-ref-status 99 处），事件句，不改。 |
| F65 | records/2026-10-11-里程碑五打包与决策进度列.md:25 | 事件句不改 | 「改的工具：`.claude/scripts/gen-decision-items.py`」这是 2026-10-11 记录里那一批改了哪些工具与样本，事件句，不改。 |
| F65 | records/2026-10-11-里程碑五打包与决策进度列.md:117 | 事件句不改 | 「item-ref-status 142 处」这是 2026-10-11 记录里当时的红行处置（142 处、本轮修掉后绿），事件句，不改。 |
| F65 | records/2026-10-11-里程碑五打包与决策进度列.md:169 | 事件句不改 | 「红 2 格（kb-shape、item-ref-status）」这是 2026-10-11 记录里验收前后对比的一行，写当时红 2 格、之后全绿，事件句，不改。 |
| F65 | records/2026-10-13-里程碑四预开发.md:432 | 事件句不改 | 「本会话文件里的门禁红（item-ref-status」这是 2026-10-13 记录里列给阶段同步的当时红格清单，事件句，不改。 |
| F65 | records/2026-10-17-全仓知识腐烂治理.md:73 | 事件句不改 | 「批 5 自检（带 `SINGLEFS_HEAVY_TESTS=user-request`」这是 2026-10-17 记录里批 5 自检当时的结果，事件句，不改。 |
| F65 | records/2026-10-17-全仓知识腐烂治理.md:100 | 事件句不改 | 「工具缺口」这是 2026-10-17 记录里工具缺口的当时登记（其中 lib-item-ref-status.py 扫描范围一项），2026-10-18 后写明了定案；按日期不回改，且该项（扫描范围）现查仍未改（第 205 行）。 |
| F65 | records/2026-10-18-审查报告核实与修复.md:23 | 事件句不改 | 「二 doc-decisions item-ref-status」这是 2026-10-18 记录里审查报告一条的核实结果（9 处），说那次核到的，事件句，不改。 |
| F65 | records/2026-10-18-审查报告核实与修复.md:80 | 事件句不改 | 「二、三 其余各格（clause-enums、invariant 两格」这是 2026-10-18 记录里审查报告其余格的处置分派，事件句，不改。 |
| F65 | records/2026-10-18-里程碑三收尾与近期欠账.md:736 | 事件句不改 | 「五个 `read-translations` 开关各只让点名的新样本判错」这是 2026-10-18 记录里那一批自证的结果，事件句，不改。 |
| F65 | records/2026-10-18-里程碑三收尾与近期欠账.md:771 | 事件句不改 | 「工具批补丁只打了一部分」这是 2026-10-18 记录里当时的补丁处置，点名 lib-item-ref-status.py 的 LANGUAGE_SUFFIXED 已在提交 767668191，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:11 | 事件句不改 | 「病根大头是译文：doc-lint 日志里 ✗ 行 2545 条」这是本阶段调度记录开工时的现场描述（译文 1091 条等），写那天查到的，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:59 | 事件句不改 | 「里程碑四预开发会话（里程碑四预开发会话）回」这是调度记录里一次会话回复的转述，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:60 | 事件句不改 | 「第 1 批 tooling-writer 交回」这是调度记录里第 1 批 tooling-writer 交回的当时报告（language_of、numbered_files、heads_for_referrer），正是这件事实的经过；事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:69 | 事件句不改 | 「还开着、这一轮接着做的：第二位 tooling-writer」这是调度记录「经过」里那一刻的待办清单（item-ref-status 认节标题当时还没做）；按日期不回改，做完的经过写在第 105 行的批次 1 交回里。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:80 | 事件句不改 | 「出口：大文件那一格与日志两份各自落地」这是第二轮开工时写下的出口，item-ref-status 认节标题是其中一项，出口是当时的计划；事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:82 | 事件句不改 | 「开工现查（`GATE_BASE=1bb3a24dc`，逐格点名跑）」这是第二轮开工时现查到的红数（item-ref-status 120 处），说那天查到的，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:88 | 事件句不改 | 「item-ref-status 认变更史 `## D<n>（简称）` 节标题」这是第二轮依赖表里批次 1 的任务分派（任务 3），当时的计划；事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:97 | 事件句不改 | 「批次 1 派出：tooling-writer 甲」这是第二轮批次 1 派出的当时记录，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:105 | 事件句不改 | 「tooling-writer 丙交回」这是第二轮里 tooling-writer 丙交回的当时报告（节标题、孤立译文、mountinfo），正是这件事实的经过；事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:106 | 事件句不改 | 「主 agent 暂存甲丙的文件」这是第二轮里主 agent 暂存与跑样本自证的当时结果，事件句，不改。 |
| F65 | records/2026-10-21-doc门禁违规修复.md:109 | 事件句不改 | 「里程碑四预开发会话 回」这是调度记录里一次会话回复的转述（里程碑四预开发记录第 67 行改法、红行里没有这份记录了），事件句，不改。 |
| F66 | .claude/agents/kb-scribe.md:22 | 不相干 | 「门禁 doc-decisions 的 kb-shape 格」这一条是 kb-scribe 的输入清单，kb-shape 格只作为要求的句子形态被点名（状态与进度标记），没有写它数不数译文、数几条；第 7 段改只对中文原文不影响这些句子的真假。 |
| F66 | .claude/agents/kb-scribe.md:30 | 不相干 | 「翻分项状态的，给门禁 doc-decisions 的 kb-shape、blocking-verdict 两格要的句子」这一句只说翻分项状态要给两格要的句子，没写第 7 段的计数口径，不受第 7 段只对中文原文影响。 |
| F66 | .claude/kb/checks-owed.en.md:315 | 要改 | 「declared-counts 格判的是那句」改成「在用 126 条（README「现共 126 条在用」），池级 checker 判 103 条，其余 23 条状态列写着「未实现」」（英文版整句同改）。现查 crates/singlefs-checker/invariants/README.md 第 36 行「现共 126 条在用」；CLAUDE.md 第 113 行写的数法命令（对 invariants/0*.md 里已实现的行计数、排掉 I-8.4）现数输出 103，126 减 103 为 23。这一行的 115 与 92 是旧数；英日译文的旧数这一轮被豁免（records/2026-10-21-doc门禁违规修复.md 第 72 行）。 |
| F66 | .claude/kb/checks-owed.en.md:326 | 不相干 | 「事实表标新立的行没有独立evidence」这一行（英文版）写 C470 事实表新立行的漏召回，declared-counts 只作为并入的旧账 C114 的名字出现，与计数口径无关。 |
| F66 | .claude/kb/checks-owed.en.md:648 | 不相干 | 「declared-counts 格只扫索引页」这一行（英文版）写 C114 的旧格撤掉、判据并进 decision-links，只点 declared-counts 格的名字；说的是撤格去向，与它数不数译文无关。 |
| F66 | .claude/kb/checks-owed.ja.md:315 | 要改 | 「declared-counts 格判的是那句」改成「在用 126 条，プールレベル checker 判 103 条，其余 23 条状态列写着「未実装」」。现查 README.md 第 36 行「现共 126 条在用」；CLAUDE.md 第 113 行写的数法命令（grep 已实现行、排掉 I-8.4）现数输出 103。日文版的旧数这一轮被豁免（records/2026-10-21-doc门禁违规修复.md 第 72 行），但句子内容仍旧。 |
| F66 | .claude/kb/checks-owed.ja.md:326 | 不相干 | 「事实表那 21 行没有任何一道闸要求メイン agent 重读」这一行（日文版）写 C470 的漏召回，declared-counts 只作为并入的旧账名字出现，与计数口径无关。 |
| F66 | .claude/kb/checks-owed.ja.md:648 | 不相干 | 「declared-counts 格只扫索引页」这一行（日文版）写 C114 的旧格撤掉、判据并进 decision-links，只点 declared-counts 格的名字，与它数不数译文无关。 |
| F66 | .claude/kb/checks-owed.md:314 | 要改 | 「declared-counts 格判的是那句」改成「在用 126 条（README「现共 126 条在用」），池级 checker 判 103 条，其余 23 条状态列写着「未实现」」。现查 crates/singlefs-checker/invariants/README.md 第 36 行「现共 126 条在用」；CLAUDE.md 第 113 行写的数法命令（对 invariants/0*.md 里已实现的行计数、排掉 I-8.4）现数输出 103，126 减 103 为 23。 |
| F66 | .claude/kb/checks-owed.md:325 | 不相干 | 「事实表那 21 行没有任何一道闸要求主 agent 重读」这一行写 C470 的漏召回，declared-counts 只作为并入的旧账 C114 的名字出现，与计数口径无关。 |
| F66 | .claude/kb/checks-owed.md:649 | 不相干 | 「declared-counts 格只扫索引页」这一行写 C114 的旧格撤掉、判据并进 decision-links，只点 declared-counts 格的名字；说的是撤格去向，与它数不数译文无关。 |
| F66 | .claude/kb/decisions.en.md:29 | 不相干 | 「doc-decisions kb-shape checks marker presence and validity」这一行（英文版）写 kb-shape 对每条分项索引行判标记在不在、值合不合法并核进度列，没提第 7 段数哪些文件；第 7 段只对中文原文不改变这句话的真假。 |
| F66 | .claude/kb/decisions.en.md:36 | 不相干 | 「kb-shape section 7 **independently counts the body text**」这一行（英文版）写第 7 段自己数正文再比，只对中文原文并不使「自己数正文」不真，说的是为什么需要独立计数。 |
| F66 | .claude/kb/decisions.ja.md:29 | 不相干 | 「門番 doc-decisions の kb-shape スロットがマークの有無と妥当性を検証し」这一行（日文版）写 kb-shape 判标记并核进度列，没提数哪些文件，不受第 7 段只对中文原文影响。 |
| F66 | .claude/kb/decisions.ja.md:36 | 不相干 | 「kb-shape スロット（第7段落）が**本文を独自に数え直して比較**します」这一行（日文版）写第 7 段独立重数正文再比，只对中文原文并不使这句不真。 |
| F66 | .claude/kb/decisions.md:30 | 不相干 | 「门禁 doc-decisions 的 kb-shape 格对每条分项索引行判标记在不在」这一行写 kb-shape 判标记并自己数一遍与进度列比，没提数哪些文件，不受第 7 段只对中文原文影响。 |
| F66 | .claude/kb/decisions.md:37 | 不相干 | 「第 7 段**自己去数一遍正文**再比」这一行写第 7 段自己去数一遍正文再比；只对中文原文是缩窄了数的文件集合，「自己数一遍」仍是真话。 |
| F66 | .claude/kb/decisions.md:38 | 不相干 | 「生成器自己数错时，decision-items-sync 格两边一起错，只有 kb-shape 格会红」这一句说生成器数错时只有 kb-shape 会红，与第 7 段数哪些文件无关，仍是真话。 |
| F66 | .claude/kb/tooling.en.md:68 | 不相干 | 「the item count check in the kb-shape slot」这一行（英文版）写 kb-shape 的条数检查只在标题声明「N 项未定」时启动，与第 7 段只对中文原文无关。 |
| F66 | .claude/kb/tooling.ja.md:68 | 不相干 | 「kb-shape スロットによる件数チェックは、見出しで」这一行（日文版）写 kb-shape 的条数检查只在标题声明「N 项未定」时启动，与第 7 段只对中文原文无关。 |
| F66 | .claude/kb/tooling.md:68 | 不相干 | 「的条数检查只在标题声明了「N 项未定」时启动」这一行写 kb-shape 的条数检查只在标题声明「N 项未定」时启动，与第 7 段只对中文原文无关。 |
| F66 | .claude/rules/format-evolution.md:24 | 不相干 | 「写了括注时门禁 doc-decisions 的 kb-shape 格判条数对不对」这一行是规则：半定决策标题括注时 kb-shape 判条数对不对；规则没变，第 7 段只对中文原文不影响括注条数怎么判。 |
| F66 | .claude/rules/format-evolution.md:26 | 不相干 | 「门禁 doc-decisions 的 kb-shape 格要的规范标记」这一行是规则：行末状态与进度标记是 kb-shape 要的规范标记，不涉及第 7 段数哪些文件。 |
| F66 | .claude/rules/path-moves.md:33 | 不相干 | 「门禁 doc-decisions 的 kb-shape、decision-items-sync 两格」这一行是决策分项清单改动后要重跑的两格对照表，不涉及 kb-shape 数哪些文件。 |
| F66 | records/2026-08-30-三轮攻击轮.md:91 | 事件句不改 | 「第 7 段「**决策索引行的状态 vs 正文标题的状态**」」这是 2026-08-30 三轮攻击轮记录里引的当时第 7 段名字，说的是那一轮，事件句，不改。 |
| F66 | records/2026-10-01-删除与并发事务.md:1304 | 事件句不改 | 「门禁现状：item-ref-status 279 处」这是 2026-10-01 记录里当时的门禁现状（kb-shape 一处自指链接），事件句，不改。 |
| F66 | records/2026-10-01-对接上游测试脚本规范.md:261 | 事件句不改 | 「主 agent 带前缀跑 `doc-decisions.sh`：17 格里 kb-shape、item-ref-status、history-loss 三格红」这是 2026-10-01 记录里那一次跑门禁的结果，事件句，不改。 |
| F66 | records/2026-10-01-对接上游测试脚本规范.md:502 | 事件句不改 | 「主 agent 带前缀跑：doc-lint 对本轮改的欠账行与 D13 零命中」这是 2026-10-01 记录里那一次跑的结果，事件句，不改。 |
| F66 | records/2026-10-11-里程碑三知识腐烂治理.md:62 | 事件句不改 | 「第一轮自检（带 `SINGLEFS_HEAVY_TESTS=user-request`」这是 2026-10-11 记录里第一轮自检的结果，事件句，不改。 |
| F66 | records/2026-10-11-里程碑三知识腐烂治理.md:90 | 事件句不改 | 「第二轮自检（同上，带 `SINGLEFS_HEAVY_TESTS=user-request`）」这是 2026-10-11 记录里第二轮自检的结果，事件句，不改。 |
| F66 | records/2026-10-11-里程碑三知识腐烂治理.md:164 | 事件句不改 | 「批 13 自检（文档类，带 `SINGLEFS_HEAVY_TESTS=user-request`）」这是 2026-10-11 记录里批 13 自检的结果，事件句，不改。 |
| F66 | records/2026-10-11-里程碑五打包与决策进度列.md:25 | 事件句不改 | 「改的工具：`.claude/scripts/gen-decision-items.py`」这是 2026-10-11 记录里那一批改了哪些工具与样本（kb-shape 第 7 段数进度标记），事件句，不改。 |
| F66 | records/2026-10-11-里程碑五打包与决策进度列.md:117 | 事件句不改 | 「用户定「要修复」后本轮修掉」这是 2026-10-11 记录里当时红项的处置，事件句，不改。 |
| F66 | records/2026-10-11-里程碑五打包与决策进度列.md:169 | 事件句不改 | 「红 2 格（kb-shape、item-ref-status）」这是 2026-10-11 记录里验收前后对比的一行，事件句，不改。 |
| F66 | records/2026-10-13-里程碑四预开发.md:64 | 事件句不改 | 「新派一个写回员接着它的规格写」这是 2026-10-13 记录里写回员接力与检查结果的当时记录，事件句，不改。 |
| F66 | records/2026-10-13-里程碑四预开发.md:73 | 事件句不改 | 「两个写回员交回后的检查」这是 2026-10-13 记录里那次检查的结果，事件句，不改。 |
| F66 | records/2026-10-13-里程碑四预开发.md:121 | 事件句不改 | 「写回后检查：`doc-decisions.sh --check kb-shape,entry-added,shape` 三格绿」这是 2026-10-13 记录里那次写回后检查的结果，事件句，不改。 |
| F66 | records/2026-10-13-里程碑四预开发.md:156 | 事件句不改 | 「kb 写回员交回（Opus，8.5 分，17.3 万 token）」这是 2026-10-13 记录里 kb 写回员交回的当时报告，事件句，不改。 |
| F66 | records/2026-10-18-里程碑三收尾与近期欠账.md:186 | 事件句不改 | 「文档普查那个会话转来三处，收下并进这一批收尾的阶段同步回扫」这是 2026-10-18 记录里转来三处的当时记录（含「115 条在用实际 123 条」），说那一次发现什么，事件句，不改。 |
| F66 | records/2026-10-18-里程碑三收尾与近期欠账.md:532 | 事件句不改 | 「declared-counts、invariant-count-elsewhere 红：表里在用 140 条」这是 2026-10-18 记录里当时的门禁结果与处置（表里 140 条含别的会话加的行），事件句，不改。 |
| F66 | records/2026-10-18-里程碑三收尾与近期欠账.md:736 | 事件句不改 | 「五个 `read-translations` 开关各只让点名的新样本判错」这是 2026-10-18 记录里那一批自证的结果，事件句，不改。 |
| F66 | records/2026-10-18-里程碑三收尾与近期欠账.md:771 | 事件句不改 | 「工具批补丁只打了一部分」这是 2026-10-18 记录里当时的补丁处置，事件句，不改。 |
| F66 | records/2026-10-19-文档普查报告核实与修复.md:37 | 事件句不改 | 「改完跑相关门禁格（`SINGLEFS_HEAVY_TESTS=user-request`，用户授权所有操作）」这是 2026-10-19 记录里当时跑门禁格的结果（含「115 条在用实际 123」），事件句，不改。 |
| F66 | records/2026-10-21-doc门禁违规修复.md:11 | 事件句不改 | 「病根大头是译文：doc-lint 日志里 ✗ 行 2545 条」这是本阶段调度记录开工时的现场描述，写那天查到的，事件句，不改。 |
| F66 | records/2026-10-21-doc门禁违规修复.md:61 | 事件句不改 | 「第 3 批第二位 tooling-writer（opus）」这是调度记录里第 3 批第二位 tooling-writer 派出时写的当时要修的毛病（第 7 段取到译文等），事件句，不改。 |
| F66 | records/2026-10-21-doc门禁违规修复.md:69 | 事件句不改 | 「还开着、这一轮接着做的：第二位 tooling-writer」这是调度记录「经过」里那一刻的待办清单；按日期不回改，做完的经过写在下面第 71 行的第二位 tooling-writer 交回里。 |
| F66 | records/2026-10-21-doc门禁违规修复.md:71 | 事件句不改 | 「第二位 tooling-writer 交回」这是调度记录里第二位 tooling-writer 交回的当时报告（第 7 段只对中文原文等），正是这件事实的经过，事件句，不改。 |
| F66 | records/2026-10-21-doc门禁违规修复.md:72 | 事件句不改 | 「主 agent 把两句中文不变量条数改成 126」这是调度记录里主 agent 把中文条数改成 126 的当时记录，写明 checks-owed 英、日译文 4 处豁免，事件句，不改。 |

## --check-report 输出

命令：`python3 scripts/kb/stale-candidates.py --check-report <候选表> <本报告> --groups F46-F48,F58-F60,F64-F66`（`--groups F46-F66` 会因 F49-F57、F61-F63 在候选表里没有行而退 2，所以按有候选的区间写），退出码 0，末行：

  ✓ 报告判全了：224 行候选都有逐行判定

## 没做什么

- 没改仓里任何文件、没改 stale=；没读会话记录。
- 候选表每行的「原文」被截在 500 字，几处我读了仓里的整行再判（verification-build 三语第 6、7 行、C564、C446、03-third-txn 第 141 行）；判定表里引的原话都取自候选表那 500 字之内。
- 判「事件句不改」的 records 行是按「把旧说法换成新的这句还是不是真话」判的，没逐份核那些记录的上下文是否另有现状句。
- C564 标题「另外 6 条」在 35 条登记之后是否仍对，没核。
- 三语译文行与中文原文同判；译文这一轮的豁免（调度记录第 72 行）不影响「句子内容是否过时」的判定。
- 推翻条件：任何一行的判定若被现查推翻（例如 `grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 不再是 35、README 第 36 行不再是 126），对应「要改」的改后句子要按新数重写。
