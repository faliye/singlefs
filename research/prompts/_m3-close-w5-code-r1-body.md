# m3-close-w5-code-r1：按 inode 号写数据与多文件删除的产品代码（代码轮第一轮）正文（2026-10-19）

<!-- doc-lint:not-numbers M1 M2 M3 M4 M5 M6 M7 W1 W2 W3 W4 W5 -->

## 一、题面

里程碑三批 5 往主工作区合进了两段产品代码改动，这一轮攻这两段：
- **按 inode 号写数据**：里程碑三一·1，用户 2026-10-18 定「先给 core 加按 inode 号写数据的入口」。
- **多文件删除**：判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 的改动 M1 到 M7，加上用户 2026-10-19 弹窗定的 K-U5「β 时序级」读法。

被判的是 `crates/singlefs-core/src/` 下 9 份文件：`admission.rs`、`deleted_inodes.rs`、`mounted_session.rs`、`mount.rs`、`recovery.rs`、`transaction.rs`、`write_accounting.rs`、`extent_tree.rs`、`unlink.rs`。三份补丁里产品代码那几份的原样 diff 在 `research/prompts/m3-close-w5-code-r1-snapshot/product-code.diff`，合入之后的文件 sha256 在同目录 `tree-sha256.txt`。

被测条款在材料附录里抄：D8（核心索引结构） 已定项 5、6、14、16；D18（块里携带什么信息） 已定项 24；D16（发布语义） 已定项 1、5；D5（快照 / 空间记账机制） 已定项 19；不变量 I-4.7（删除中 inode 有账）、I-9.5（删除队列里的号仍在树里）、I-1.13（扫描重建与遍历方向的索引相同）。判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 与岔路单 `research/prompts/m3-multi-file-unlink-r1-forks.md` 腿直接读原文。

这一轮要的是两件：代码做的是不是条款说的；哪一格会错。

## 二、实现今天的样子（主 agent 的观测，2026-10-19）

行号都按树副本 `~/.local/share/singlefs/work/m3-close-w5-code-r1-tree` 算。它是主工作区合入批 5 全部补丁之后的 `crates/` 原样拷出，9 份被判文件与主工作区逐份同哈希。

**新入口与新数据结构**
- `transaction.rs` 加了 `publish_sequential_write_of_an_inode` 与 `publish_overwrite_of_an_inode`。
- `UserChange` 加了 `SequentialWriteOfAnInode`、`OverwriteOfAnInode` 两个成员。
- `TransactionOutput.files_other_than_the_first` 按 inode 号记别的文件的数据指针与下段节点指针。
- `TransactionUnit` 加了 `DataOfAFileOtherThanTheFirst`、`ExtentLowerNodeOfAFileOtherThanTheFirst` 两个角色。

**上段规范形与删除**
- 上段的形状由纯函数 `extent_tree::plan_the_extent_upper_segment` 定，判决 M2 那一条规则：
  - 节点取「各有条目的 inode 的根路径之并」；
  - 根层级取罩得住最大号的最低一层；
  - 集合为空时取层级 0 的零条目根。
- 写与删共用这个函数。
- `unlink.rs` 删任一 inode（M1），删第一个文件走同一份计划（M3）。
- `next_unlink_step_after` 按被删 inode 自己的单元数取下一步。

**恢复与 K-U5 的 β 读法**
- 恢复（`recovery.rs`）收第一个文件之外的 extent 条目。
- 新错误成员 `DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion` 只拒对象出生晚于 inode 1 墓碑死亡 txg 的 inode 写数据。

**实现员自己定、主 agent 照收、交这一轮攻的六处**
1. β 只在 inode 1 的记录不在时判；有几条墓碑时取死亡 txg 最大的那条。
2. 回退之后的水位取现行那一版的。
3. 上段节点槽数按上界算。
4. `MAX_UPPER_ENTRIES` 一批 64。
5. 「少一个祖先」改成「少一片叶」。
6. 第一个文件之外的写，截断与变长照旧拒（`SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses`）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### W1　写入口与条款
两个新入口是否走同一个事务状态机与封闭的提交步骤枚举，没有开第二条发布路径（`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。

写别的 inode 时，下面三样是否与第一个文件同一套规则：
- 上段与下段的写、换下的节点进 defer；
- 准入扣的量（`admission.rs`、`write_accounting.rs`）；
- 会话入口（`mounted_session.rs`）。

### W2　上段规范形
写与删两边是否都只经 `plan_the_extent_upper_segment` 定形。对同一个「有条目的 inode 集合」，是否只能写出一种上段形状。

要攻的地方：
- 层级升降；
- 叶变空；
- 只剩第一个文件、只剩别的一个文件、全删空三种终态；
- 一批 64 的分批与槽数按上界算这两处，会不会在大号或稀疏号上写出非规范形，或让准入低估。

### W3　删除各步
意图、分批、最后一批三步在 inode ≠ 1 上，是否照 D8 已定项 16 第 1、2 条套用：
- 批大小；
- size；
- 最后一批删记录与 `deleted_inodes` 那条 key 是否同一次发布；
- 删第一个文件之后别的文件还活着时，版本的成员取哪一个；
- 两种删序的终态是否逐字节相同。

### W4　恢复、续做、墓碑与 β
- 恢复收多文件时，「至少一个单元、单元连号、尾部没有洞」那一判在删除中间态上会不会误拒。
- 挂载续做在多文件上走不走得完。
- 墓碑每个对象一条，出生代怎么写。
- β 的「取死亡 txg 最大」与「只在 inode 1 记录不在时判」两处，有没有让该拒的写放过、或该放的写拒掉的历史；第一题面对的是删完再建同号。
- 回退之后水位取现行那一版，会不会让树 ID 或 inode 号退回去重发。

### W5　测试与变异
新测试是否测的是条款而不是实现。每一处改法有没有变异行；它们点名的测试会不会真红。要找条款里有、而测试与变异都没罩到的格。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | W1–W5 | 逐格核「代码做的是不是条款说的」：每条条款找到对应的代码行与测试，对不上的写出来；引代码写树副本的路径与行号 |
| 云端攻方（Opus） | W1–W5 | 攻「哪一格会错」：造历史（交错写与删、删一半崩溃、续做、快照、回退、上段长高之后删、删到只剩一个），在树副本的拷贝上写用例或小模型量，打中的给复现命令与输出 |
