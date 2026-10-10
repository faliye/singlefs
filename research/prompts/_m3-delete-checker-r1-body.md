# 删除线 checker 五条不变量的读法：设计轮第一轮正文（2026-10-04）

<!-- doc-lint:not-numbers F1 F2 F3 F4 F5 K1 K2 K3 U5 -->

## 一、题面

里程碑三·3（`.claude/kb/milestone/03-third-txn.md` 第三节表「三·3」那一行）要在池级 checker 里实现 I-4.4（意图只描述目标态）、I-4.6（中间态其余不变量成立）、I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-3.4（可用空间扣待删占用），验收是「每条只红自己；`crates/mutations.tsv` 各一行」，出口 1（同一文件「出口」一节）要求这几条实现。

主 agent 派的只读调查（报告 `/tmp/claude-1000/m3-prep-3-3/report.md`，sha256 c3123a765286fd7fbd8f7a9478eb31fcbb8442fb54f6fe4e10fb9fb4f9808053，结论句只当线索）报：I-9.5 两个方向与 I-3.4 能照字面写判法；I-4.4、I-4.6、I-4.7 的定义各有两种以上读法，判法不同、「只红自己」能不能做到也不同。这一轮判：每一处读法该取哪一个，取不下来的交用户时每个候选的代价与翻面观测是什么。

| # | 问的什么 | 候选（调查报告第六节，原样） |
|---|---|---|
| F1 | I-4.7「处于删除中的 inode」指谁 | A nlink = 0 的记录；B 号在 extent 树里有数据单元而 inode 树里没有活记录；丙 A ∪ B |
| F2 | I-4.4 在第一版判什么 | 甲 `deleted_inodes` 条目宽 = 8 且 logged_ops（种类 12）根指针全零；乙 第一版报不适用、改出口 1 措辞（归用户）；丙 落到 nlink = 0 记录的字段上（与 F3 乙重合） |
| F3 | I-4.6 判什么 | 甲 其余各条的合取；乙1 nlink = 0 的记录：数据单元数 = ⌈size ÷ 净荷容量⌉；乙2 每条记录都判这一式；丙 甲 + 乙 |
| F4 | I-9.5、I-4.7、I-4.6 共用的三件 | a 判哪几版（只最新根 / 回退候选集每版加记录施加出来的几版）；b `deleted_inodes` 与 inode 树怎么配对（经树表条目偏移 116 的头 ID，配不上记判不了）；c 读不全报 `part_of_the_range_not_judged` 还是 `partially_judged` |
| F5 | I-3.4 | a 落哪几句（只落「标签 4 恒 0」/ 三句都落）；b 标签 4 那一行缺了判红还是不适用；c 判哪几版 |

**不在这一轮的**：
- `IMPLEMENTED_INVARIANTS` 的追加次序（只许往末尾加，`crates/singlefs-checker/src/image.rs` 文件头）：实现细节，主 agent 照调查报告第六节 F6 定。
- GPU 判器覆盖表（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 第 75–89 行的断言，用户 2026-10-03 定「改回要求全覆盖」）：排期问题，池级 checker 与 GPU 内核同批合入。
- 三·5 里要先定设计的几笔（C536、C116 合并半、C109、C692、C694、C695）：另开。

## 二、实现今天的样子（主 agent 的观测，2026-10-04，取自调查报告第一到第五节，腿要自己在冻结副本里现查）

- 池级 checker 读到 `deleted_inodes` 根、判了条目宽 = 8（`crates/singlefs-checker/src/walk.rs` 第 1891–1920 行），条目读进来之后没人用。
- inode 记录逐条读号、size、blocks、出生代、改动计数（`walk.rs` 第 2303–2400 行），不读 nlink（记录偏移 36）；`crates/singlefs-checker/src` 里 `nlink` 零命中。
- 记账树行收进 `self.accounting` 时键是 (统计量, 设备)，统计量常量只有 1、2、5、12（`walk.rs` 第 149–156 行），标签 4 没人读。
- logged_ops（种类 12）`key_schema_for_tree_kind` 返回 `None`（`crates/singlefs-checker/src/lib.rs` 第 594–604 行），走读直接返回。
- 实现一侧：单文件 unlink 在 `crates/singlefs-core/src/unlink.rs` 与 `crates/singlefs-core/src/transaction.rs`；每次发布记账标签 4 写 0（`transaction.rs` 第 7930–7932 行一带）；意图的载体是 nlink = 0 的记录、`deleted_inodes` 的一条 key、一条墓碑（D8（核心索引结构） 已定项 16 第 1 条）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　逐条推读法（正推）
对 F1–F5 每一处，从不变量定义原文（`.claude/kb/invariants.md` 那一行整行）、它压着的决策分项（D8（核心索引结构） 已定项 5、6、16；D5（快照 / 空间记账机制） 已定项 4、8；三·1 那三轮判决 `research/prompts/m3-delete-r1-main-verification.md`、`m3-delete-r2-main-verification.md`、`m3-delete-r3-main-verification.md` 里定这几条的那几行）推到每个候选：哪个候选是定义字面、哪个是扩读或缩读、哪个让定义的哪一句落空。每个候选写：判法一句、合法镜像上会不会误红、坏镜像能不能只红自己、要改 kb 哪几处。

### K2　每个候选哪一格会错（攻方）
对 F1–F5 每个候选找反例：
- 合法历史上（unlink 的意图、各批、挂载续做、管理员回退到删除之前、回退之后再删）它误红的镜像；
- 一个真坏的删除中间态它放过的镜像（它该管而没管、别的条也不管）；
- 两个候选在同一组镜像上判法相同、实为同一个的；
- 「只红自己」做不到的候选，判别力实际落在哪条别的不变量上。
用 `crates/` 冻结副本拷贝里的 harness 造镜像，只跑 harness 档；放开扫用户决定的那几步（文件单元数、批大小、崩在哪一批、哪一版、回退到哪一版）。

### K3　代价与交用户的形态
每处取不下来的，写岔路单一行的四列：候选（各自定义）、翻面观测、够判条件、代价（实现要补的读法、判哪几版的扫描次数、要不要改出口措辞），代价没量过的写「推的，没量过」。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 从定义原文与决策分项推到每个候选，判字面与扩读 |
| 云端攻方（Opus） | K2、K3 | 对每个候选造误红与漏判的镜像 |

## 五、跑前写死的判据

- 一处读法两条腿都推到同一个候选、攻方对它没打中：主 agent 照它定，进 kb 写回（`invariants.md` 那几行与状态列）；「没打中」是一次抽样，照 `.claude/rules/three-way-inference.md` 记，实现之后的代码轮再抽。
- 两条腿推到不同候选，或攻方打中正推选的那个：主 agent 现查分辨它们的事实；分不出的交用户，带 K3 那一行。
- 改出口 1 措辞的候选（F2 乙）一律交用户。
- 腿提的新候选算零轮。

交付：
- 腿的报告写进 `research/prompts/m3-delete-checker-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句。
- 模型与脚本放 `research/prompts/m3-delete-checker-r1-<腿名>-model/`，有脚本的 `rerun.sh` 交回前在空目录里从头跑通一次。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号，去 kb 文件里现查，不从背景材料里数；引代码写冻结副本里的文件:行号。
- 方案按 `crates/` 今天的实现来谈。
