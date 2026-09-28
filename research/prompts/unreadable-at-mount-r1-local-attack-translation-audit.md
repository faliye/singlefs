# unreadable-at-mount-r1 本地攻方：提示转述核对表（2026-09-28）

提示：`research/prompts/unreadable-at-mount-r1-local-attack.md`（149 行，sha256 `24609d7a8de8ce30f2e6d7302e08d262996fde58e825e56acbfc0f6bad2c16b0`）。「英文项」列写提示里的事实行号（T01…）与提示文件自己的行号。

- kb 与调查报告的行号是这一次现查的；kb 与两份调查报告的 sha256 与 `research/prompts/unreadable-at-mount-r1-snapshot/kb-sha256.txt` 逐份对得上（`sha256sum -c` 全 OK）。
- 代码行号取自冻结副本 `crates/`（sha256 清单 `research/prompts/unreadable-at-mount-r1-snapshot/crates-sha256.txt`，`sha256sum -c` 全 OK）；下表写成仓内相对路径，因为 `mount.rs`、`transaction.rs`、`recovery.rs`、`root_ring.rs` 四份今天与主工作区逐字节相同（`cmp`）。
- 缩写：`D23` = `.claude/kb/decisions/23-journal的角色与格式.md`；`D21` = `.claude/kb/decisions/21-权威态与派生态的分界.md`；`INV` = `.claude/kb/invariants.md`；`C331报告` = `research/prompts/closeout-recheck-2026-09-28/c331-investigator-report.md`；`C331用例` = `research/prompts/closeout-recheck-2026-09-28/c331_newer_instance_roots_unreadable_then_readable_again.rs`；`C393报告` = `research/prompts/closeout-recheck-2026-09-28/c393-investigator-report.md`；`正文` = `research/prompts/_unreadable-at-mount-r1-body.md`；`core/` = `crates/singlefs-core/src/`。
- 「首稿」= 草稿目录 `prompt-draft1.md`，「定稿」= 仓里的提示（= `prompt-draft2.md`）。定稿与英文项相同的写「同」。

## 一、规则（kb）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| T01 标题「a writable mount finds a newer state it cannot read」（提示第 26 行） | D23:387「可写挂载读到的更新状态读不出」 | 无 | 同 |
| T01 c_witness 的取法：读阶段（择根、扫 journal、重放）读完之后，池里每块盘两槽全部自证过的系统配置槽 journal tail 取最大 | D23:387 | 无 | 同 |
| T01「If c_witness is 0: not judged」 | D23:387「0 ⇒ 不判」 | 无 | 同 |
| T01 末条标志记录读得出（任一份）⇒ c_witness 大于它的计数器为真 | D23:387 | 无 | 同 |
| T01 读不出而计数器等于 c_witness 的记录读得出 ⇒ 比 (实例代号, checkpoint_txg) | D23:387 | 无 | 同 |
| T01「If both records are unreadable: treat the criterion as true」 | D23:387「两条都读不出按真」 | 无 | 同 |
| T02 为真就在本次挂载内有效的读缓存（只收读成且不是全零的落点）上重做读阶段一遍 | D23:387 | 无 | 同 |
| T02「R = 1, taken from the "reread once" in the "root slot read fails this time" row of the publish-semantics rule; there is no waiting between the two reads」 | D23:387「R = 1，取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」，两次读之间不等」 | 无（「D16 已定项 1」写成「the publish-semantics rule」，模型读不到 D16，编号对它没用） | 同 |
| T02 重做那一遍为假就用它往下走；仍为真就在取号之前拒可写、盘上逐字节不变，只读照常 | D23:387 | 无 | 同 |
| T03 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次，仍读不出拒可写 | D23:387 | 无（括注「代码审阅第 22 条」不译：出处标记，模型用不上） | 同 |
| T04 取号那一写不再写 tail 0，改写取号那一刻按同一取法读到的 c_witness | D23:387「「续」：…不再把 tail 写成 0，改写取号那一刻按同一取法读到的见证值 c_见证」 | 无（`write_acquired_instance` 的函数名与「用户 2026-09-27 定」「实现员交回」出处不译：T15 另有函数名一行） | 同 |
| T04 取法细节：每块盘两槽里全部自证过、fsid 与本池相同的槽 journal_tail 取最大，一份都没有时 0，第一道屏障之后、第一个取号写之前读一次 | D23:387 | 无 | 同 |
| T04 取号失败的回卷写带同一个见证值、不退回 0 | D23:387 | 无（「D18 已定项 11」出处不译） | 同 |
| T04 mkfs 写 tail 0；mkfs 之后第一次取号读到 0、写 0，新池新建文件的字节不变 | D23:387 | 无 | 同 |
| T05 取号那一刻某块盘一份自证过的系统配置槽都没有（见证值那一遍读错，逐盘核那一遍读得出）：拒 | D23:387 | 无 | 同 |
| T05「The rule text says this refusal is being implemented.」 | D23:387「实现在做（实审 A3c）」 | 无（「主 agent 2026-09-27 定」「实审 A3c」不译） | 同 |
| T06 系统配置没见证到的最新根：续落地之后下一次挂载在自己的 c_witness 里看到；记欠 | D23:387「这一形记欠见 C554 … 的 Q1」 | 无（C554、Q1 编号不译，写成「recorded as an open item」） | 同 |
| T07 记录水位按实例代号为主比、与 jsn 同序；择根 txg 为主、实例代号破平局；两处比的不同，各守各的序 | D23:366（第 2 条注的前两句） | 无（该注第三句「按 txg 为主时…会被当成水位之后施加」不抄：讲的是另一种比法的反例，与本题网格无关） | 同 |
| T08 标题「invariant I-7.4, blocks of recent generations not reused」 | INV:55「I-7.4 \| 近 K 代块未被复用」 | 「K 代」译成「recent generations」：kb 里「K 代」已改指回退候选集，那一半本行没抄，留 K 反而要另解释 | 同 |
| T08「Roots of an abandoned timeline are not in the rollback candidate set; that only governs the choice of rollback target.」 | INV:55「被抛弃时间线的根不在候选集里，这只管回退目标的选择」 | 首稿写成「are not rollback candidates」，缺「这只管回退目标的选择」 | 补上 |
| T08「Today abandoned timelines are only created by crash recovery.」 | INV:55「被抛弃时间线今天只由崩溃恢复造出」 | 首稿缺这一句 | 补上 |
| T08 被抛弃根引用的块离开根环之前同样不许重新分配、不许抹头 | INV:55「它们引用的块在离开根环之前同样不许重新分配、不许抹头」 | 无 | 同 |
| T08 checker 对根环里每条被判抛弃的根各用自己的走读走一遍，只看读回来对不对，走出违例或走读失败按这条根判红 | INV:55「根环里每条被判抛弃的根…各用一份自己的走读走一遍，只看读回来对不对——走出 I-2.1 违例或走读失败就按这条根判红」 | 无（「最新根的实例表有行时」这一前提与「引用不进 I-3.1 并集」「根环有读不出的槽时报不适用」三处不抄；I-2.1 写成「a checksum violation」） | 同 |
| T08 不抄的部分 | INV:55 回退候选集的定义、K 代的口径、F_生效、C556、defer 队列那几句 | 不抄：本题网格只判被抛弃根引用的单元复用不复用 | — |
| T09 权威态 = 单元 + 记账 + 根，丢了就丢数据；派生态 = 索引（树），丢了只是慢 | D21:188、D21:189 | 无（「参与格式冻结」「不参与格式冻结」两处不译：与网格无关） | 同 |
| T09「根」含根记录 + 树表单元 + 实例表单元 | D21:191 | 无（「都是 I-7.2 要走过的结构，不是索引」不译） | 同 |
| T10 分配记录树整个可以从权威态重建，不进权威态清单 | D21:106 | 无（「权威态仍是三样，不扩项」不译，T09 已有三样） | 同 |

## 二、代码今天的样子（按函数名写进提示，行号只在这里）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| T11 c_witness = 各盘 `verified_system_configuration_slots` 返回的槽 journal_tail 的最大；读不出或自证不过的槽直接不进 max；一个都不剩时 0，比较一支为 NothingWitnessed | core/mount.rs:4146-4161；C331报告:82 | 无 | 同 |
| T12 NothingWitnessed ⇒ 假；对所选那一版末条 ⇒ c_witness 大于它的计数器；对见证计数器那条记录 ⇒ 比 (实例, txg)；Undecidable ⇒ 真 | core/mount.rs:384-398 | 无 | 同 |
| T13 第一遍假用第一遍；为真先调钩子 `before_rereading`，再跑一遍读阶段（这一遍重读系统配置槽）；重读为真拒 `NewerStateStillUnreadableAfterOneReread`；为假用重读那一遍 | core/mount.rs:4242-4267；C331报告:83 | 无 | 同 |
| T13「the witnesses of the two passes are not merged into a maximum, and the version the reread pass selects is not required to be newer than the first pass's」 | C331报告:83「这里不要求重读那一遍所选的版本比第一遍新，也不拿两遍的见证取大」 | 首稿写过「at least as new」，与原文「比第一遍新」不同，落盘前改成「newer than」 | 同 |
| T14 写的世代号 = 这块盘读得出的自证槽里最大世代号 + 1，落槽 = 世代号 mod 2；最大世代号那一槽读不出时正好落回那一槽 | core/transaction.rs:361-366、core/transaction.rs:407-410；C331报告:84 | 无 | 同 |
| T15 取号写带的 tail = 取号时读到的自证槽里 journal_tail 最大（只算读得出的） | core/transaction.rs:845-846、core/transaction.rs:671-678；C331报告:84 | 无 | 同 |
| T16 取号时某块盘自证槽为 0 份 ⇒ 取号失败；至少一份读得出的盘通过 | core/transaction.rs:654-659 | 无 | 同 |
| T17 新实例首个 txg = max(根环里读得出的根的最大 txg, 自证过的记录最大 checkpoint_txg) + 1；落哪个根槽只由 txg 定 | core/mount.rs:1085-1107、core/root_ring.rs:109-117；C331报告:86 | 无 | 同 |
| T17 被藏的根只有 txg 高过新实例已发的全部发布才会之后压过它；被藏的根少于等于新实例的发布数时根槽被逐个盖掉 | C331报告:86 | 无 | 同 |
| T18 新实例的下一个计数器 = 读得出的记录最大计数器 + 1，不与见证值比 | core/mount.rs:4335-4340；C331报告:85 | 无 | 同 |
| T19 每条被抛弃根调一次 `placements_referenced_by_root`；交 None（树表或分配记录树读不出、解不开）就计数加一、continue：不隔离、不重读、不报错、不拒 | core/mount.rs:1251-1255、core/mount.rs:1215、core/mount.rs:1296、core/mount.rs:1348-1360；C393报告:125-126 | 无 | 同 |
| T20 读阶段的重读管 T01 的判据；影子账唯一的重读是最新那条根的实例表（钩子在它之前调）；两者都不碰被抛弃根自己的树表或分配记录树 | core/mount.rs:4245-4249、core/mount.rs:1179-1184；C393报告:127 | 无 | 同 |
| T21 分配记录树节点：按提示读两份，都读不出就查中央映射、照映射落点再读一次（本段历史里同槽）；树表：两份各读一次、没有映射回退 | core/recovery.rs:404-426、core/recovery.rs:2106-2111；C393报告:138-139 | 无 | 同 |

## 三、历史与实测（调查报告）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| T22 共同历史：实例 1 暖机 txg 1、2，A txg 3；实例 2 写行 txg 4、暖机 5、B 6、C 7，全部确认 | C331报告:38 | 无 | 同 |
| T22 A = (1, 3, jsn 3)、B = (2, 6, jsn 6)、C = (2, 7, jsn 7) | C331报告:97 | 无 | 同 |
| T22「Two devices, two system configuration slots per device.」 | C331报告:78「两块盘」；D23:387「每块盘两槽」 | — | 同（多出来的「每盘两槽」，见第五节） |
| T23 读系统配置槽的次序：第 1 次择系统配置、第 2 次核盘表、第 3 次第一遍见证读、第 4 次重读那一遍的见证读、第 5 次读阶段之后（算生效回退下界） | C331报告:133-137、C331报告:178 | 无 | 同 |
| T23 把 V1 的坏读挪到第 5 次，结局变回拒可写（两遍都读到见证 7） | C331报告:125-129、C331报告:165 | 无 | 同 |
| T24 原形今天：一直读不出 ⇒ 拒可写（见证 7 > A 末条 3，重读仍真）；新实例一个字节都没写；撤故障后择 C | C331报告:59 | 首稿把「新实例一个字节都没写，没有已确认的写可以被压」译成「instance 2 has written nothing on top」，主语错了 | 改成「the new instance has not written a single byte, so there is no acknowledged write that can be overridden」 |
| T24 只藏根、记录读得出也拒：重放不跨实例，所选仍 A | C331报告:59 | 无 | 同 |
| T24 只坏一次或只有重读那次读得出：重读择 C；新实例写 E 确认；撤故障重开择 E | C331报告:60 | 无 | 同 |
| T25 V1 故障与今天结局 | C331报告:68 | 首稿缺最后一列「丢的是谁的已确认写：新实例」 | 补「The loss is the new instance's.」 |
| T26 V2 故障与今天结局 | C331报告:69 | 同上 | 补「the loss is the new instance's」 |
| T27 V3 故障与今天结局，含「较旧实例（C 丢，挂载不拒）」 | C331报告:70 | 无 | 同 |
| T28 V4：第 k 次挂载两块盘见证 C 的那一槽暂时读不出、C 的根读得出、择根择 C；取号写落回那一槽写 tail 6，写行之前崩溃；崩溃后四槽 tail 全是 6；第 k + 1 次 C 的根与记录读不出，见证 6 = B 末条，从 B 可写，C 丢，E 活 | C331报告:71；C331用例:910-912；C331报告:181 | 首稿缺「丢的是较旧实例的」 | 补「The loss is the older instance's.」 |
| T29 V1 最后那次重开也不拒：next_counter 不与见证比；实例 3 的记录从 jsn 4 起写，每次轮换写这次发布的计数器；盘上 tail 只剩 6、5，低于 7 | C331报告:85 | 无 | 同 |
| T30 (i) 系统配置没见证过的最新根 D | C331报告:72 | 首稿缺「D 的根 FUA 之后」「崩溃」「D 没确认过」，也没点名 D 与实例 3 | 按原文补齐 |
| T30 (ii) 只坏盘 1 的两槽 ⇒ 盘 0 仍见证 7，拒可写 | C331报告:73 | 无 | 同 |
| T30 (iii) 根与记录只在重读那次读得出 | C331报告:74 | 首稿缺「之后几处直接读盘的地方看不见实例 2 的根」 | 补上 |
| T31 适用范围：只在两块盘、区域 [0, 1, 0]、每次覆盖写一条记录、实例 1 → 2 → 3 这一组参数上造过；V1、V2 要求四个槽恰好在见证读那一次同时读不出 | C331报告:78 | 无（「这种故障在真盘上有多常见没量过」一句不译） | 同 |
| T32 C393 历史：A、B（实例 1）→ 取号 2 → C (2, 8) → 崩溃恢复抛弃 C（C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽坏掉，恢复落到 (2, 7)，实例 3 写行 txg 9、暖机 txg 10）→ C 写回 → 实例 4 可写重开，C 的一份账两份都读不出（块设备错） | C393报告:8 | 无（「C 写回」译成「C becomes readable again」：原文指撤掉读故障、C 的根槽与单元又读得出，不是再写一次） | 同 |
| T33 各臂重开都在第一遍判完：所选 (3, 10) 就是系统配置见证到的那次（jsn 10），判据为假；最新根 (3, 10) 的实例表读得出；C (2, 8) 比所选旧；钩子各臂 0 次；阳性对照钩子 1 次、隔离 14 / 14 | C393报告:25、C393报告:106、C393报告:196 | 无 | 同 |
| T34 C 独占 14 槽（对照臂两盘各隔离 14）；不隔离则重开后分配器当空闲，重开后第一次覆盖写（txg 14）把新数据单元落在 C 的数据单元槽上；C 的根槽到 txg 32 才被盖掉，所以 txg 14 时 C 还在根环里、I-7.4 红 | C393报告:26、C393报告:141、C393报告:15 | 首稿写成「14 per device」，原文只说「C 独占的 14 槽」，每盘各 14 是对照臂隔离数给的 | 改成「C exclusively holds 14 slots (the readable control isolates 14 on device 0 and 14 on device 1)」 |
| T35 R1–R7 七臂的挂载结局、计数、隔离、复用、I-7.4 | C393报告:15-21（表头 C393报告:13） | 无（表里「产品的「重读一次」钩子」一列只在 R3、R6 两行写出「hook called 0 times」，R2、R4、R5、R7 的「0 次」由 T33「各臂 0 次」给） | 同 |
| T36 R4 与 R7 为什么分叉：分配记录树根是进映射的节点，两份提示都读不出后经中央映射再读同槽一次（(2, 1)），那是映射回退不是产品的重读；树表豁免映射、各读一次 (1, 1)；产品层的「重读一次」对这一读不起作用，钩子 0 次 | C393报告:28-30 | 无（「D19 的映射回退」「C554 的重读」两个编号不译，写成「the mapping fallback」「the product's reread」） | 同 |
| T37 调查员副本改法（只用来定位）：None 时再调一次；R7 翻成隔离 14 / 14、计数 0、到 txg 31 不复用、Holds；R2、R3、R5、R6 不变，只有读次数翻倍 | C393报告:145、C393报告:186-187 | 无 | 同 |
| T37「used to locate the cause, not a proposed fix」 | C393报告:4「只查不修；没判该怎么改」 | — | 同（多出来的，见第五节） |
| T38 没看挂着时抬 F 重算影子账那一路：它会再读一次被抛弃根的账，后来读得出的臂可能在抬 F 之后补上隔离；本段历史 txg 14 的复用在任何抬 F 之前 | C393报告:227 | 无（`mount.rs` 2353–2363 行号与 `space_admission=AdmittedBeforeAcquisition` 不译） | 同 |

## 四、候选与题目（正文）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Jia = 甲 读不出即判不出，含括注「调查员副本上的改法，V1–V3 翻成拒」 | 正文:34 | 无 | 同 |
| Yi = 乙 按盘判 | 正文:35 | 无 | 同 |
| Bing = 丙 两遍合并，含「只修 V2 那一格」 | 正文:36 | 无 | 同 |
| Ding = 丁 维持今天：登记「见证读与根同时读错」为不保的多重故障，写进射程 | 正文:37 | 无 | 同 |
| V4 那一支：各候选分别给出取号写的见证值怎么取；已有「取号那一刻某块盘一份自证过的系统配置槽都没有：拒」，看它罩不罩得住 V4 | 正文:38 | 无（「D23 已定项 14」写成指向 T05） | 同 |
| C393 候选「树表与分配记录树各判一遍」 | 正文:40 | 无 | 同 |
| (a) 拒可写并报告：重读一次仍读不出就拒可写，只读照常 | 正文:41 | 无 | 同 |
| (d) 保守隔离：读不出时把被抛弃根所在实例写过的槽整片按隔离算、不复用，直到那条根离开根环（写明「写过的槽」从哪得） | 正文:44 | 无 | 同 |
| (b)、(c) 不进提示 | 正文:42-43 | 不抄：派发把 C393 的列定成「今天、(a)、(d)」 | — |
| Q1 网格一：行 V1–V4，列今天、甲、乙、丙、丁 | 正文:58；派发提示 | 无 | 同 |
| Q2 网格二：行是 C393报告 的五格与两格对照（R1–R7），列今天、(a)、(d) | 正文:58；C393报告:31（五格 = 隔离 0 / 0 的 R2、R3、R5、R6、R7，两格对照 = R1、R4） | 无 | 同 |
| 格里填「拒可写 / 可写且不丢写 / 可写且丢写 / 可写且复用」 | 正文:58 | 无 | REFUSE / WRITABLE-NO-LOSS / WRITABLE-LOSS / WRITABLE-REUSE，一一对应 |
| 每格写推导依据的事实表行号 | 正文:58 | 无 | 「facts: the fact ids you used」 |
| Q3 V4 各候选的取号见证值与 T05 罩不罩得住 | 正文:38 | 无 | 同 |

## 五、英文比原文多出来的（逐条写为什么加）

| 多出来的 | 在提示哪里 | 为什么加 |
|---|---|---|
| 答复规则七条（英文作答、不用 markdown 强调、不写代码行号与文件行号、每格必答不许只答 yes / no、按编号答、每格写推翻条件） | 提示第 5-11 行 | 定义「做什么」第 1 条的要求 |
| Notation 一段：版本写成 (实例代号, checkpoint_txg)；jsn；acknowledged = fsync 返回；unreadable = 读报块设备错；hidden = 这次挂载读不出的根 | 提示第 13 行 | 模型读不到中文材料，记法要先交代；「块设备错」取自 C393报告:8 与 C331报告:201，「被藏的根」取自 C331报告:86 |
| 四个结局标签的英文定义，与「同一段历史既拒又丢写时怎么判」一条 | 提示第 15-20 行 | 正文:58 只给了四个中文词，没给判法；不写判法，同一格可以两种填法，几份样本就对不齐。「丢写」口径取正文:49「已确认返回的写被压过或丢掉」，「复用」口径取 T08 |
| 分节标题 Part A–E、QUESTIONS | 提示多处 | 排版，不带事实 |
| T13、T19 等代码行开头的函数名 | 提示 T11–T21 | 让模型按函数名指代码（定义要求答复不写行号） |
| T22「two system configuration slots per device」 | 提示第 72 行 | V1–V4 都按「四个槽」说事，先交代每盘两槽 |
| T23「from a call-stack trace」 | 提示第 74 行 | 原文次序出自调用栈（C331报告:131），标明这是实测次序、不是推的 |
| T33「C (2, 8) is older than the selected version.」 | 提示第 96 行 | 原文 C393报告:25 就有，放进来是为了 (a) 的判法不必再推一遍新旧 |
| T35 表头「isolation is device 0 / device 1; counter is the unreadable counter of T19」 | 提示第 100 行 | 原表列名「影子账隔离（盘 0 / 盘 1）」「abandoned_roots_unreadable」的英文说法 |
| T37「used to locate the cause, not a proposed fix」 | 提示第 111 行 | C393报告:4「只查不修」与 C331报告:154「这次改动只用来定位，不是修法提议」同口径；不写，模型会把它当成候选 (a) 的实测 |
| Q2 的「for (a) in R3 and R6 say whether your reread calls the hook before_rereading」 | 提示第 138-143 行 | (a) 的定义没说重读走不走产品的钩子，而 R3、R6 两臂的故障正是在钩子里撤掉；不让模型写明，这两格的结局分不出 |
| Q1、Q2 每格的「numbers:」一行 | 提示 Q1、Q2 | 本地腿只问能落成数的题（`.claude/rules/three-way-inference.md`「各条腿必须互不重复」一节）；让每格带出见证值、隔离数、复用 txg，主 agent 能逐格对 |
| Q4 列出靠假设填的格 | 提示第 147 行 | 候选定义没说死的格要单独露出来，主 agent 判时知道哪几格是模型自己补的定义 |
