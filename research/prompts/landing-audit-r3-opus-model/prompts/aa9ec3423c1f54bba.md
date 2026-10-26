# 派发：implementation-writer

重型测试：不跑
批次：records/2026-10-13-里程碑四预开发.md#截断#5
线程上限：31
草稿目录：/home/fy5090/.local/share/singlefs/work/m4-trunc-checker-writer
认领：bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m4-trunc-checker-writer --purpose 'm4 截断截乙：池级 checker 与扫描重建'
报告：/home/fy5090/.local/share/singlefs/work/m4-trunc-checker-writer/patch/report.md
条款：判决 /home/fy5090/.local/share/singlefs/work/m4-predev/research/prompts/m4-trunc-r1-main-verification.md 第三节 K6、K7（CPU 一侧），连同 K1、K2、K5 定的格式与中间态、第六节 S1 的标签收严（整节读；正推报告 `research/prompts/m4-trunc-r1-sonnet-output.md` 的 W6、W7 与攻方 `m4-trunc-r1-opus-output.md` 的 W5、W6、W7 是依据，攻方 `research/prompts/m4-trunc-r1-opus-model/prototype/rebuild.patch` 里「Z 加写序」那一段是参照、不照抄）；D18（块里携带什么信息） 已定项 10；I-1.13、I-1.9、I-4.4、I-4.6、I-4.7、I-9.5
要动的 crates 文件：crates/singlefs-checker/src/（walk.rs、rebuild.rs、image.rs 与需要的别的文件）、crates/singlefs-checker/invariants/（01-块自描述.md、06-事务与崩溃一致性.md、09-inode树结构.md 等要改的类文件）、crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs、新建 crates/singlefs-harness/tests/checker_truncation_holes_and_intent_must_red_worlds.rs；crates/mutations.tsv 只追加
预计挂钟：2 小时内（五条判加必红世界，推的）

回复与报告用中文。

基线：工作树 `/home/fy5090/.local/share/singlefs/work/m4-predev`（写段工作树，拷副本改，交补丁）。核心层的截断与洞已经打进工作树（截甲：`crates/singlefs-core/src/truncate.rs`、`UserChange::Truncate`、logged_ops 种类码 2、上段叶标签 3 与 4；报告 `~/.local/share/singlefs/work/m4-trunc-core-writer/patch/report.md` 第十节列了格式常量的派生形态）：合法世界用真写路径造（可以读用 `crates/singlefs-harness/tests/common_truncation/mod.rs` 的搭建，不改它），必红世界在真镜像上按字节改。不碰：`crates/singlefs-core/`、`crates/singlefs-harness/src/`、别的 harness 测试文件与公共模块（截丁在改随机历史、翻译与模型接线）、`crates/singlefs-reference-filesystem/`、`crates/singlefs-checker-tier/`。池级 checker 只依赖 `singlefs-format`，格式常量在 checker 里自己写一份。

## 关的是哪几条

1. K6 读法 Z 加写序（`rebuild.rs`）：认领一个数据单元要三条都满足——对象出生代与重建出的 inode 记录相同；偏移在 ⌈size ÷ 净荷⌉.max(1) 之内（size 之外不认领，`DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause` 不再报）；同一 (inode, 对象出生代) 的当前版本里没有偏移更小、出生 txg 更大的单元。验收：`scan_rebuild_agrees_with_the_walked_index.rs` 里钉「覆盖写变短报不适用」那一条改成钉「成立」；先变短再变长（真写路径）成立；去掉写序那一条它红。
2. K7 T-5（I-1.13 两侧按 size 滤）：遍历一侧（`walk.rs` 的 `IndexTablesWalkedFromTheNewestRoot` 装填处）不收 size 之外的单元与它们的映射 key；两侧各写各的。验收：截断中间态（批大小压成 1、意图之后第一批之前那一版）I-1.13 成立。
3. K7 T-1：I-4.4 白名单加截断种类（种类码 2；value 只装 inode、对象出生代、目标 size，补零 10 全 0；inode 在同一版存在且 nlink ≥ 1、对象出生代相等、目标 = 记录的 size；全池至多一条截断条目）。必红世界：补零塞增量、种类码 3、两条截断条目、目标 ≠ size、出生代不符。
4. K7 T-2：截断中间态有账——每个 inode 存在的单元多于 ⌈size ÷ 净荷⌉.max(1) ⇔ 有一条对得上的截断条目；判哪几版照 I-9.5 的共用判法。必红世界：中间态上删掉截断条目、留着尾部单元。
5. K7 T-3：单元数对 size，nlink = 0 与 ≥ 1 都判（合并 I-4.6 与 C711）：存在的单元从 0 起连号；标签 1、2 的单元数等于 ⌈size ÷ 净荷⌉.max(1) 且不许有洞，标签 3、4 必须至少有一个洞单元（判决第六节 S1），标签 0 是一个单元都没有；有截断条目时 size 之外可以多。必红世界：中间缺一个单元、标签 1 却少单元（尾部单元被错放）、标签 3 而没有洞、nlink ≥ 1 多出单元且没条目；合法世界：有洞文件的 unlink 意图那一版不红。
6. K7 T-4：每个存在而在 size 之内的单元，声明长度 = min(size − 起点, 净荷)。必红世界：只改 size 不重写尾单元（缩到单元中间、单元内变长两张）。

不变量清单同步：I-4.4、I-4.6、I-1.13、I-1.9 四行照改（I-1.9 补「size 之外与被写序判死的单元算被压住」），新立 T-2、T-4 各一行（编号照该类文件已有最大号往后取）；`IMPLEMENTED_INVARIANTS` 跟着加。GPU 判器这一批不改，报告里给每条新判列 GPU 判器要做的（下标、判法要点、要读的表），下一批的 GPU 实现员照它写。每条新判先证会红，每处改法在 `mutations-append.tsv` 留一行。

## 出口

六条都达成，补丁目录 `/home/fy5090/.local/share/singlefs/work/m4-trunc-checker-writer/patch/` 交 `crates.patch`（在 m4-predev 工作树上 `git apply --check` 过）、`mutations-append.tsv`、`report.md`（含「要主 agent 跑的命令」一节）。交回前照定义跑 fmt、clippy、build、自己动到的测试二进制与形式化轻型，经内存包装；不跑崩溃与变异轻型、不跑 checker 档。交回前核一遍：你的补丁打上之后，`crates/mutations.tsv` 里凡锚在你改过的文件上的行，原文都恰好命中一次（被你弄坏的整行替换，交 `mutations-replacements.tsv`）。仓副本与编译目录交回前删。