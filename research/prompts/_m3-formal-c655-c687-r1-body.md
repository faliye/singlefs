# 挂载透明没归类的字段与 I-5.5 判不了的几格：第一轮正文（2026-10-05）

<!-- doc-lint:not-numbers Y1 Y2 Y3 R1 R2 U V F -->

## 一、题面

两笔欠账，用户 2026-10-05 定开三方（弹窗原话记在 `records/2026-10-01-里程碑三形式化接入.md` 第十七节）：

- C655（挂载透明逐字段表有字段没归类）：`.claude/kb/decisions/13-验证路线.md` 已定项 1「挂载透明」那张逐字段表（R2 对 R1）没写这几样各归哪一类——根记录的 `mapping_root`、`allocation_record_tree_root`，墓碑、谱系、deleted_inodes 三根指针，算法、nonce、MAC，系统配置的槽世代号与 journal_tail。还清条件要一个判官：根记录与系统配置的字段清单里有一个字段不在表里就判红，删掉表里一行必须红。第三轮辩方推过一个归法（两根指针并进分配记录那一行，三根指针与算法三样必须相同），推的，没量。
- C687（从 mkfs 水位建树那一版落点判不了的几格）：`.claude/kb/invariants.md` I-5.5 那一行里，这几格今天报「射程里判不了」（没有违例时整条报不适用）：树表读不出、条目宽不到 200 字节；种类 1 条目不止一条；认出来的那一版 extent 树读不全；有数据单元而池里某块盘上一个都没有；种类 3（分配记录树）条目不是恰好一条、或分配记录树读不全；另有开放段在提交内生块回落时的误排（推的）。每一格该判红还是不适用，没有条款。

这一轮要定的是条款，不是代码：每个字段归哪一类、每一格判红还是不适用，各自的依据；代码改法写成对实现的改动，供下一批实现员用。

## 二、实现今天的样子（主 agent 的观测，2026-10-05）

- 挂载透明判官：`crates/singlefs-harness/src/mount_transparency.rs` 的 `MountTransparencyField` 十个成员（第 608 行起，`ALL` 在第 634 行起），与逐字段表十二行里除「根槽位置」「分配记录」之外的十行一一对应；表外那几样代码里一样都没比。「字段清单里有字段不在表里就判红」的检查零命中：`grep -n "字段清单\|不在表里\|every field\|all_fields" crates/singlefs-harness/src/mount_transparency.rs crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 零命中。
- 根记录与系统配置的字段与偏移：`crates/singlefs-format/src/` 的常量、`.claude/kb/layout/01-first-txn.md` 的字段表（以 checker 源码为准，`.claude/rules/verification.md`「checker 即规范」）。可写挂载写什么：`crates/singlefs-core/src/mount.rs`（取号、写行、暖机、续做、抬 F）、`crates/singlefs-core/src/transaction.rs`（发布）。
- I-5.5 判法：池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark`（函数头注释逐格列了判不了的几格）；GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_first_data_unit_of_a_version`（那几格一格都不置位）。坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，合法镜像 `crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`。
- 冻结副本 `/tmp/claude-1000/m3-formal-c655-c687-r1/tree/`（crates 的 sha256 清单在同目录 `crates-sha256.txt`）。代码引文读冻结副本、写那棵树里的路径与行号。方案按 `crates/` 今天的实现来谈。

## 三、格

### Y1　C655：表外字段各归哪一类
对每一样（根记录 `mapping_root`、`allocation_record_tree_root`、墓碑根、谱系根、deleted_inodes 根，算法、nonce、MAC，系统配置槽世代号、journal_tail）：只挂载、挂载加一次发布、挂载途中二次崩溃三种动作下，R2 与 R1 这一样是什么关系——必须相同、模掉（并入哪一行的判法）、还是按一条确定关系判（写出关系式）；依据是实现在哪一步写它（文件:行）与哪条已定条款（整句抄）。量得了的在拷出的副本上量（造崩溃镜像、只读恢复两次逐字段比）。

### Y2　C655：字段清单的完整性与对账判官
根记录与系统配置的全部字段逐个过一遍（以 `crates/singlefs-format/` 与池级 checker 的解析为准），除了题面那十样还有没有表里、表外都没提的字段；对账判官的「字段清单」该从哪一处权威定义取（格式常量、checker 的解析、layout 表），才能做到格式加一个字段而表不跟就红；判别力自证怎么造（删表里一行必须红）。

### Y3　C687：I-5.5 判不了的每一格判红还是不适用
逐格：这一格在合法写侧写出的镜像（含崩溃态、回退、多实例、读故障）上会不会出现——会出现就不能判红；只在坏镜像上出现的，判红是否与别的不变量重复（哪一条已经判红，读 `.claude/kb/invariants.md` 现查整句）；不重复的判红要造一份坏镜像证明改之前报不适用、改之后红。开放段回落误排那一格：今天的准入下够不够得着（读码到行），够不着的写成什么。CPU 与 GPU 两个判器各要怎么改（文件:行）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Y1、Y2、Y3 | 从实现写各字段、各格的那几行代码与已定条款正推每一格的归类与判法，每格给判定、引文与推翻条件 |
| 云端攻方（Opus） | Y1、Y2、Y3，重点 Y1、Y3 | 找反例：一段合法历史让拟定的归类或判红把合法状态判坏，或一个坏实现让它放过；在拷出的副本上造镜像量，checker 档不跑、写推的 |

## 五、跑前写死的判据

- 两腿同判、主 agent 现查坐实 ⇒ 按那一判写回（一次抽样的「没打中」只算这一次）。
- 两腿分歧 ⇒ 主 agent 核分辨两者的那个事实；核不出就开第二轮，换攻击面。
- 腿提的新判法算零轮；照 `.claude/rules/three-way-inference.md`，三轮里多数打穿才算打穿。
- checker 档、GPU 没跑的格，腿的结论只能是「推的」。

交付：腿的报告写 `research/prompts/m3-formal-c655-c687-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-formal-c655-c687-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
