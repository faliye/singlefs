# 核对表：里程碑「第二个事务」步 4 / 步 5 代码三方第一轮，本地辩方英文提示逐句核对

方法：先按 kb 原文与 `crates/` 源码逐点核实每一处要辩护的取法，再写英文提示；本表整理成对照，每行是「英文提示里的哪一句 / 对应原文文件:行 / 首稿里发现并改正的偏差 / 定稿状态」。原文引用整段抄自 `research/prompts/_m2-step45-code-r1-appendix.md`（下称附录一）与 `crates/` 源码，未转述。

## 项 1（P1：新实例第一条 jsn = R_old 自己那条 + 1）

- 英文句：`next_counter = own_record.counter + 1` 的机制描述。
- 原文：`crates/singlefs-core/src/mount.rs:570`（`let next_counter = own_record.counter + 1;`）；附录一行 981「实做时定下的（标预想、交用户）：回退之后新实例的第一条 jsn = R_old 那条 + 1（C340 取 P1……）」；附录一行 1433（C340 登记行）。
- 首稿偏差：P2（对照臂）译成「the highest jsn currently readable in the whole journal ring」，与原文「接可读链末尾」不是同一个量（链末尾要求连续，不是环里最大 jsn）。
- 定稿：改成「the end of the continuous chain of readable records, found by following jsn continuity forward」，用 `research/scripts/replace-once.py` 命中 1 次替换、回读确认。
- 记录核对器区分「合法覆盖」与「真洞」：原文附录一行 20（`crates/singlefs-harness/src/crash.rs` `check_records`：「一份记录「丢了」= 不在盘上且没有更晚落在同一位置的写在盘上」）；英文句忠实转述为「distinguish a slot legitimately overwritten by a later valid record from a genuine gap」。
- 验收造不出的一格：附录一行 981「验收里造不出来的一格：……取 P1 之后 R_old 之后那个记录槽被新实例的记录盖掉，所选根 (1, 3) 紧接的 jsn 4 是实例 3 的、回退行在不在都不施加，第五条在这条脚本上只在函数级可达（决策点）」；英文句原样转述这一机制（预置实例 1 的记录、回退行在不在都不影响、只在函数级可达）。

## 项 2（回退行 W 恒 0）

- 原文：附录一行 981「回退行的 W 恒 0」；`crates/singlefs-core/src/mount.rs:616`（`applied_transaction_high_water: 0` in `PreviousInstanceRow` for `mount_rollback`）；前缀第五条原文见附录一行 19-24（D23 已定项 14）。
- 英文句忠实转述前缀第五条的机制（W 为 0 时一条都不施加）与 W 恒 0 的写法，并把它与项 1 的发现（P1 已经让下一格记录物理不可读）连起来问「W 现在是不是死代码」。未发现首稿偏差。

## 项 3（影子账窄读法）

- 原文：附录一行 7（D23 已定项 14 正文：「分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽……2026-09-16 用户定案取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内」）；代码 `crates/singlefs-core/src/mount.rs:587-611`（`own_slots` 取自 `previous.allocation_records`，不区分 `is_released`；隔离循环遍历 `roots` 里 `(txg, instance) > target` 的每一条）；`crates/singlefs-core/src/allocator.rs:543`（`isolate_abandoned`）。
- 首稿偏差：3(a) 问句首稿写成「narrow reading only excludes slots that appear in R_old's records as allocated, not slots R_old released」，这与代码相反——`own_slots` 的构造不区分 `is_released`，已释放的记录同样在 `own_slots` 里、同样被排除出隔离范围。
- 定稿：改写为准确描述——`own_slots` 不检查每条记录的已分配/已释放状态，只按槽号成员判断；因此 R_old 自己曾经释放过、后来被被抛弃根重新分配的槽，现在的代码仍然把它当「R_old 自己的」而不隔离。已用 `replace-once.py` 命中 1 次替换、回读确认。

## 项 4（非空持久有效根的读法）

- 原文：附录一 D16 已定项 1 表格「抬 F 的上限」行（`.claude/kb/decisions/16-发布语义.md:358-416`，附录一行 111-168 区间内，表格第 4 行）；里程碑 `02-second-txn.md` 步 5「现状」段（附录一行 1017 前后：「非空 = 环里有它自己那条记录且事务号非 0（预想）」）；代码 `crates/singlefs-core/src/mount.rs:228-238`（`non_empty` 用 `record.transaction != 0` 过滤）。
- 「三种读法在这条脚本上同答 11」：附录一行 1042（「三种「非空」读法在这条脚本上同答 11」）。
- 英文句忠实转述判据（`record.transaction != 0`）、暖机与抬 F 空发布事务号恒 0 的事实（依据 `crates/singlefs-core/src/mount.rs:380`、`:309` 的 `transaction: 0`）、以及「猜测未被攻过」的状态。未发现偏差。

## 项 5（先回收再推带新 F 的空发布；重开写 F_生效可能更低）

- 原文：D16 已定项 1 表格「抬 F 的上限」「生效」两行（附录一行 127-128）；代码 `crates/singlefs-core/src/mount.rs:261-332`（`raise_rollback_floor`：先 `allocator.reclaim_released_up_to(new_floor)`、再推 `Carry` 空发布，循环上限 `ROOT_RING_REGIONS`）；`crates/singlefs-core/src/recovery.rs:351-371`（`effective_rollback_floor`：各盘最大值取最小）。
- 英文句忠实转述回收先于任何一次带新 F 的发布落盘、循环退出条件（覆盖齐或达到区域数上限，两者先到为准）、以及重开时新实例的根写 `effective_rollback_floor` 可能比曾经宣布的目标更低。追加问句聚焦「循环因为撞上限而不是因为覆盖齐退出时，函数仍返回 Ok」这一点，是读代码后新增的观察，原文没有专门条款讨论这一格，如实标为读代码所得而非引用 kb 结论。

## 项 6（链首无锚点只认 txg + 1）

- 原文：附录一行 19-25（D23 已定项 14「六条口径」第一条，含前缀第五条与⚠️第六条）；代码 `crates/singlefs-core/src/recovery.rs:736-766`（`replay_journal` 的 `root_own_record_counter` / `expected_next` / `chain_start_txg_without_anchor` 逻辑）；上一轮判决 `research/prompts/m2-step3-code-r1-main-verification.md:44`（X1 行）。
- 首稿偏差：描述被拒绝的保守对照臂时写「only ONE journal record (not the root's own record but a different one)」，与原文「所选根自己那条记录读不出（两份都撕了）」这一前提矛盾——那条被撕的记录本身就是所选根的锚点记录，不是「另一条」。
- 定稿：改写为「the chosen root's own anchor record is torn (plus a root slot is corrupted) but a fully intact record landing right after it should still be recovered」，已用 `replace-once.py` 命中 1 次替换、回读确认。
- 一次发布一条记录、txg 每次加一的假设：附录一行 19（六条口径引言）与 `crates/singlefs-core/src/recovery.rs:737`（注释「第一版一次发布一条记录、txg 每次加一」）逐字支持英文句。

## 项 7（第一个文件版本释放 mkfs 树表）

- 原文：D3 已定项 7（附录一行 419-420：「落点释放时条目不删……」）；代码 `crates/singlefs-core/src/allocator.rs:380-392`（`mark_format_time_units` / `format_time_tree_table`）、`:359-378`（`rebuild_from_records`，行 335-336 注释「重开之后上一版从盘上重建、释放经映射与根记录走，这里留空」）；`crates/singlefs-core/src/transaction.rs:571-592`（`format_time_tree_table_to_release`）、`:838-869`（`publish_first_file`，`previous: None`）、`:920-929`（`publish_version` 里 `previous` 为 `None` 时走这条释放路径）。
- 首稿偏差：初稿只问「crash between mkfs and first file publish」这一假设性场景，没有指出 `rebuild_from_records` **永远**不恢复 `format_time_tree_table`（不是「可能丢」，是源码注释明写「留空」）。
- 定稿：改写为陈述这一代码已确认的事实（`rebuild_from_records` 的字段永远是 `None`），并把问题收紧为「mkfs 与第一次文件发布是否保证在同一个不中断的进程会话内完成」。已用 `replace-once.py` 命中 1 次替换、回读确认。

## 项 8（额外问题：第一版可写挂载的射程决策点在回退/抬 F 进来后还成不成立）

- 原文：`research/prompts/m2-step3-code-r1-main-verification.md:45`（X2① 行）、`:62`（决策点表「第一版可写挂载的射程」行）；代码 `crates/singlefs-core/src/recovery.rs:434-459`（`rebuild_version` 的「树表为空」检查）；`crates/singlefs-core/src/mount.rs:479`（`mount_writable` 调用点）、`:572`（`mount_rollback` 调用点，同一个 `map_rebuild_failure`）。
- 英文句忠实转述：两个函数共用同一个 `rebuild_version`、同一个空树表检查、同一个 `NoPublishedVersion` 错误；回退目标若落在第一次文件发布之前（mkfs 根或暖机根），同样会命中这条检查。未发现偏差；这是主 agent 现读代码得出的连接点，原文两份判决材料里都没有专门讨论回退与这条检查的交互，如实按「代码确认、kb 未曾讨论」处理，写进英文提示时未冒充为已有条款。

## 全文格式核对

- 已用 `grep -n '::' 提示文件` 确认零命中（Rust 路径分隔符已用「散文 + 单个函数名 + in file 文件名.rs」代替）。
- 已用 `grep -n '\*\*\|__\|^#'` 确认零处 markdown 强调符号。
- 未使用任何中文；被引用的中文原文只在本核对表里出现，不进英文提示正文。
