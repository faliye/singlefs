# feature bit 记账表

**中文** · [English](feature-bits.en.md) · [日本語](feature-bits.ja.md)

三张位图（incompat / compat_ro / compat）各 256 位，住系统配置。每分出去一位就在这里写一行，说清它叫什么、哪个格式版本哪次提交引进来、什么状态、语义是什么。

**位号的唯一登记位不在这个文件里，在 D15（格式冻结政策） 已定项 4 的登记表**（`.claude/kb/decisions/15-格式冻结政策.md`）。「第几位归谁」以那张表为准；两处对不上时改 `.claude/kb/feature-bits.md`，不改那张表。门禁 code-source-discipline 的 feature-bits 格（`.claude/gate.d/code-source-discipline.sh`）逐位比对两处，对不上判红。

分配权是「谁先合并谁先占」，不是审批制（D15（格式冻结政策） 已定项 10）。**里程碑 17 收尾之前不存在格式锁定；格式锁定之前没有旧盘，位号可以回收、可以重排**，改位号时 `crates/` 与 checker 在同一次改动里跟着改；锁定之后位一旦用过不许回收，理由在 D15（格式冻结政策） 已定项 10。

<!-- feature-bits:table -->
一行一位，七列照 D15（格式冻结政策） 已定项 10 定下的列序。「语义一句话」要逐字包含 D15（格式冻结政策） 已定项 4 登记表里同一位的「含义」，门禁 code-source-discipline 的 feature-bits 格按这一条比对。「名称」写代码里那个常量名，代码侧没有常量的写 `—`。

| 位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话 |
|---|---|---|---|---|---|---|
| 0 | incompat | — | 格式版本 1 | 53c9f38 | 空出 | 空出，可再分配；没有布局用它，读者见到它按不认识的位拒 |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 格式版本 1 | 45d49aaf | 在用 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）；mkfs 起就置上，不认识这一位的读者挂不上 |
| 2 | incompat | INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT | 格式版本 1 | 0b926bd3f | 在用 | inode 记录带截断字段（偏移 120 起 T 与 N_w），截断字段无条件带上；位 2 是布局身份的一部分，mkfs 起就置上，读者只收位 1、位 2 都带的那一种（第一字节 0x06） |

**compat_ro 与 compat 两张位图一位都没分，incompat 分了位 1 与位 2，位 0 空出，别的位一位都没分**（出处：D15（格式冻结政策） 已定项 4 登记表）。

**代码侧引用了哪几位**：`crates/singlefs-core/src/system_configuration.rs` 的 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`，位 1，与 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELDS_BIT = 0x04`，位 2；空出的位 0 在代码侧没有常量；同一份文件的 `INCOMPAT_FIRST_BYTE_OF_THIS_LAYOUT` 由位 1 与位 2 合成（第一个字节 0x06），不是位掩码字面量、声明跨行，门禁 code-source-discipline 的 feature-bits 格解不出位号，把它列进成功那句的「没判位号的」名单。`crates/singlefs-checker/src/lib.rs` 判 incompat 用它自己的具名常量 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（第 292 行）与 `INCOMPAT_INODE_RECORD_TRUNCATION_FIELD_BIT = 0x04`（第 295 行，常量名比核心层的少一个 S）。

**位图的字节序**：位图小端，位 n 住第 n div 8 个字节的第 n mod 8 低位（D22（单元原子性怎么合成） 已定项 13）。
