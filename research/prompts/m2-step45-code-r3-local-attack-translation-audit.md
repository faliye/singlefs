# 转述核对表：m2-step45-code-r3-local-attack（2026-09-17）

逐句核对 `research/prompts/m2-step45-code-r3-local-attack.md` 里每一句转述与中文原文（现查行号，crates 源码与 kb 决策文件为准，不是背景材料的行号）。格式：英文项 / 原文文件:行 / 首稿缺的 / 定稿。

| 英文项（提示文件里的句子，节选） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| "must not be reallocated and must not have their headers erased" | `.claude/kb/decisions/23-journal的角色与格式.md:1209`（「它们引用的单元不许重新分配也不许抹头」） | 首稿只写了 not be reallocated，漏掉「抹头」（erase headers）那一半 | 两个并列动作都写出来：must not be reallocated and must not have their headers erased |
| candidate set = readable AND valid per instance table AND txg >= floor | `crates/singlefs-core/src/mount.rs:202`（「候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F」）与 `.claude/kb/decisions/16-发布语义.md:376` | 首稿只写了后两个合取项（valid、txg），漏掉「可读」这一项 | 三个合取项都列出，且用 "all three of the following hold" 显式点出是三项合取，第一项是 it is readable |
| the ninth term "is maintained by the rollback code path" | `.claude/kb/decisions/28-挂载期承诺量.md:30`（「由回退路径维护」） | 首稿只抄了公式与计算时机，漏掉「由回退路径维护」这句 | 补一句 "The same source states that this quantity is maintained by the rollback code path." |
| "computed separately per device" | `.claude/kb/decisions/28-挂载期承诺量.md:30`（「按设备算（已分配统计量带设备维）」） | 首稿只写了「只住内存」「回退选中那一刻算出」，漏掉「按设备算」这个维度 | 补入 "computed separately per device" |
| "without walking the whole tree" | `.claude/kb/decisions/28-挂载期承诺量.md:30`（「不遍历树」） | 首稿写了「不加盘上字段」，漏掉「不遍历树」 | 两条限定都写出："without walking the whole tree and without adding any new on disk field" |
| 生词表在 kb 里被显式标注为"两处对同一个量给了两个口径" | `.claude/kb/checks-owed.md:328`（C356：「两处对同一个量给了两个口径」） | 首稿把这句处理成自己的转述判断（"these two descriptions seem inconsistent"），没有标出这是项目自己的记录里写明的事实 | 改写为 "explicitly flagged, in the project's own records, as two different specifications for what is supposed to be the same underlying number"，点明这是原文自陈，不是转述者自己的判断 |
| "amount allocated by the abandoned timeline since R old" 这句本身也是按统计量之差算 | `.claude/kb/checks-owed.md:296`（C318「怎么拦」列：「被抛弃时间线自 R_old 起的分配量（按根记录携带的已分配统计量之差算，不遍历树）」） | 首稿按字面把这句处理成一个独立的、听起来像是"累计增量扫描"的计算方法，没有把括注里"按统计量之差算"这一句带上——这会让读的人误以为它是一种不同于第一种表述的计算方式 | 补上括注："and adds in parentheses that this is computed the same way, as a difference of the allocated statistics carried by root records, without walking the tree" |
| allocation record tree reader "does not attempt to interpret that node's contents as allocation records" | `crates/singlefs-core/src/recovery.rs:407-409`（「读到层 > 0 的根就报格式错，不把内部节点的指针当分配记录解」） | 首稿只写了"returns a format error"，没写清楚不这样做会是什么（把内部节点的指针条目误当分配记录解析） | 补上 "and does not attempt to interpret that node's contents as allocation records" |
| single level guard 的欠账"tracked as unfinished work for a later milestone step" | `crates/singlefs-core/src/recovery.rs:405`（「多层的树这条路还不会走（里程碑「第二个事务」步 6 的欠账）」） | 首稿只写"has not been exercised yet"，读起来像是疏忽，没有传达这是项目正式记账的欠账（不是没想到，是已知且延后） | 补上 "and that this is tracked as unfinished work for a later milestone step" |
| G5 recompute happens "before the corresponding reclaim of freed space happens" | `crates/singlefs-core/src/mount.rs:433,447`（「回收在写第一条带新 F 的根之前」「回收之前先按新 F 重算影子账」） | 首稿把这句写成了单纯的时间先后叙述（"then G5 recomputes"），没有点出这是一条时序上的硬约束（回收必须晚于重算） | 改写为 "Before the corresponding reclaim of freed space happens, G5 recomputes and isolates 2 more slots per device"，把"之前"放在句首当作硬性时序，不当成顺带的时间状语 |

## 历史版本

（暂无历史）
