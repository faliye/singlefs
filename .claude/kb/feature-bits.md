# feature bit 记账表

三张位图（incompat / compat_ro / compat）各 256 位，住系统配置。每分出去一位就在这里写一行，说清它叫什么、哪个格式版本哪次提交引进来、现在什么状态、语义是什么。

**位号的唯一登记位不在这个文件里，在 D15（格式冻结政策） 已定项 4 的登记表**（`.claude/kb/decisions/15-格式冻结政策.md`）。「第几位归谁」以那张表为准；两处对不上时改 `.claude/kb/feature-bits.md`，不改那张表。门禁 code-source-discipline 的 feature-bits 格（`.claude/gate.d/code-source-discipline.sh`）逐位比对两处，对不上判红。

分配权是「谁先合并谁先占」，不是审批制（D15（格式冻结政策） 已定项 10）。**位一旦用过不许回收**：回收之后重新分配给新特性，旧镜像那一位会被新代码读成「新特性已启用」，文件系统会真的做错事（例如把不存在的加密特征当存在，去找不存在的密钥）。退役的位状态写「退役」，语义永久锁定为退役前最后一次使用的那一句，不许改写。

<!-- feature-bits:table -->
一行一位，七列照 D15（格式冻结政策） 已定项 10 定下的列序。「语义一句话」要逐字包含 D15（格式冻结政策） 已定项 4 登记表里同一位的「含义」，门禁 code-source-discipline 的 feature-bits 格按这一条比对。「名称」写代码里那个常量名，代码侧没有常量的写 `—`。

| 位号 | 类别 | 名称 | 引入版本 | 引入 commit | 状态 | 语义一句话 |
|---|---|---|---|---|---|---|
| 0 | incompat | INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT | 格式版本 1 | 53c9f38 | 退役 | 第一条纯 SSD 布局线；mkfs 起就置上，不认识这一位的读者挂不上（D12（目标介质） 已定项 1「每套布局一个 incompat 位」） |
| 1 | incompat | INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT | 格式版本 1 | 待提交 | 在用 | 第一条纯 SSD 布局线（系统配置带回退下界 F、根记录带卸载记号、没有回退见证与回退行）；mkfs 起就置上，不认识这一位的读者挂不上；位 0 退役之后读者见到位 0 一律拒挂 |

**compat_ro 与 compat 两张位图一位都没分，incompat 分了位 0（退役）与位 1，别的位一位都没分**（出处：D15（格式冻结政策） 已定项 4 登记表）。

**代码侧引用了哪几位**：`crates/singlefs-core/src/system_configuration.rs` 的 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`，位 1；同一份文件的 `INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT = 0x01` 是退役的位 0，只在测试与注释里用，留着让「用过的位不回收」写进代码；`SUPPORTED_INCOMPAT_BITS` 由位 1 合成，不是位掩码字面量，门禁 code-source-discipline 的 feature-bits 格解不出位号，把它列进成功那句的「没判位号的」名单。`crates/singlefs-checker/src/lib.rs` 判 incompat 用它自己的具名常量 `INCOMPAT_FIRST_SSD_LINE_WITH_ROLLBACK_FLOOR_AND_UNMOUNT_MARKER_BIT = 0x02`（第 219 行）。

**位图的字节序**：位图小端，位 n 住第 n div 8 个字节的第 n mod 8 低位（D22（单元原子性怎么合成） 已定项 13）。
