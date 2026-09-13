你是 singlefs 三方论证的**反推腿**。立场：**假设候选甲（「第一版不做分配器提示，本项定否」）是错的**，去找它错在哪；写材料的人就是写 E151（用户数据落点的到达序与容器臂） 第四次跑那两条目录臂的人，先攻装置再攻条款。**打不中就明说构造不出，不许造弱反例。**

## 先读

1. 背景材料：`research/prompts/_d14-r3-background.md`（正文 + 小节清单 + 附录）。前两轮：`records/2026-09-07-D14项2第一轮.md`、`records/2026-09-09-D14项2第二轮.md`。
2. 自己去查：
   - `research/e7-index-bench/src/bin/e151_arrival_and_container_arms.rs` 全文——尤其 `allocate_directory_hint`（成员所在段只增不减）、`home_segments_of_directory`（溢出段的分法）、`allocate_directory_home`、`directory_metrics`（三个量怎么算）、`dirty_set`（负载里目录怎么被碰）、`Sim::new`（初始布局 key k 住槽 k 等于每目录一段连续）
   - `research/results/e151-arrival-and-container-arms-2026-09-13-directory.out` 全部 159 行
   - `research/prompts/e151-r4-prereg.md`——跑产物之前补第二条臂那段修订，判它是不是事后建模
   - `research/e7-index-bench/src/bin/e122_directory_locality.rs` 的 `place`——E122 的 grouped 与 E151 的两条臂到底是不是同一件事
   - `.claude/rules/mutation-sampling.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在跑前写死之列」
   - 全仓 grep 目录局部性的消费方（第 4 问）

## 要攻的

1. **攻装置**：目录 = 32 个连续 key 且初始每目录一段连续，是不是把「按目录聚」的收益按构造送给了所有臂又按构造收回；负载里 runs8 = 8 个连续 key 恰好落在同一目录，uniform 每目录 0.25 次 / 轮——这两组负载对提示臂各偏向哪边；`directory_runs_total` 量槽相邻性而 COW 之下谁都守不住，拿它当 H1 是不是判据选错了量；跑产物之前补第二条臂并补判据 H1′，按纪律算不算「先看数再改判据」。
2. **攻「代价」那一句**：家固定的臂把提交内生块回落 99% 归因于溢出段占了尾部全空段——这是装置几何（尾部只有 32 段、没有别的空闲）的产物，还是政策的性质；给一个溢出段来源（例如只留 N₀ 段给提交内生块、其余给家）并推导它守不守得住 ≤ 3 段。
3. **攻 J1**：写材料的人说「段粒度局部性今天没有条款消费」——去全仓找反例：readahead、目录遍历、删目录、快照销毁、整理的搬迁批次、D25 的 smallfile / multistream 那两行，任何一条读到「目录」或「相邻」都算。
4. **攻三个分句那一格**：「不落盘」——父目录关系在 dirent / inode 里本来就落盘，与 `locality_id` 住 key 编码被判「破第一句」是不是同一种落盘；「每次分配都可以重新判」——家固定与它并排写出来是矛盾还是不矛盾；硬要求 5。
5. **攻乙与 D3 已定项 8 的关系**：乙的落点句能不能写成 D3 已定项 8 第 1 条的收严（多要求、不放宽），还是必然放宽。
6. **最强的一条**：若这一项该定成别的东西（例如「否，但把『家固定 + 留 N₀』登记成第一个重开条件」或「维持未定并写死要等哪个数」），说清凭哪些已定条款。

## 交付

- **分段写**：把报告写进 `research/prompts/d14-r3-opus-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（文件已存在就报错停下，不许覆盖），后面的段追加。回复里只写几句要点。
- 可以在 scratchpad（/tmp/claude-1000/-home-fy5090-code-singlefs/89d75d5c-f72d-410c-8998-70768ed46db9/scratchpad/）里拷一份装置改臂实测，不许改仓里的源码与产物；实测要整行抄进报告并写明改了什么。
- 引条款**整行抄**；现查的写明命令与行号。
- 不许用「本条」「上文」这类指代，每一格自足。不许改 kb。
