# 里程碑二收尾：代码轮第二轮正文（2026-09-27）

<!-- doc-lint:not-numbers Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 Z1 Z3 Z1Q-A Z3-A P1 Q4 Q7 Q8 -->

## 一、这一轮要判什么

第一轮判决 `research/prompts/m2-closeout-code-r1-main-verification.md`（快照 `refs/sop/m2-closeout-code-r1-snapshot`，2026-09-27）第三节写明第二轮只攻：Z3-A 乙的改后代码；第一轮没罩到的（A2a 同一段同盘两次系统配置写、journal 环转圈后的撕裂、续跑与分片）；第一轮快照之后打进主工作区的那几批，以及 A3。这一轮攻的仍是**写好的代码与它的测试**，设计不重判；第一轮与设计轮的判决当已定的前提用。

第一轮快照之后打进来的批次，报告都在 `research/prompts/` 下：

| 批 | 做了什么 | 实现员报告 |
|---|---|---|
| C11b | format 里带算术的 8 个 `pub const` 改字面量；`journal_in_flight_record_limit` 搬出 format，core 与 E158 各算一份、交叉断言 | `m2-rev-c11b-implementer-report.md` |
| B3a-3c、B3a-3d | 四份层 0 文件的流重新布置、position_addressed 打印溢出；降高那条流真走降高、两份层 0 全量登记成崩溃枚举用例 | `m2-rev-b3a3c-implementer-report.md`、`m2-rev-b3a3d-implementer-report.md` |
| A4c、A4d、A4e | 准入按最坏情况计：每块盘改几片叶、落得下的槽、压小容量；分配记录树取 K1 整棵树；C545 那一格改成准入先拒；清单外 3 条用例改期望 | `m2-rev-a4c-implementer-report.md`、`m2-rev-a4d-implementer-report.md`、`m2-rev-a4e-implementer-report.md` |
| B3c-2、B3c-3 | 崩溃注入第二、三截交记录核对器；树表 0 条的一版上写行那一版的对拍比实例表与分配代 | `m2-rev-b3c2-implementer-report.md`、`m2-rev-b3c3-implementer-report.md` |
| C554 乙、乙-配置续、乙-模型（含 B3c-4） | 挂载时有更新的根读不出：重读一次，还读不出就拒可写；取号那一写把 journal tail 写进系统配置当见证值；模型跟上、零单元发布按指针往下带着比 | `m2-impl-c554-yi-implementer-report.md`、`m2-impl-c554-yi-carry-implementer-report.md`、`m2-impl-c554-yi-model-implementer-report.md` |
| Z3-A 乙 | 挂着之后的每个入口都逐盘核，含会话发布 | `m2-impl-z3a-yi-implementer-report.md` |
| 55 号装置 | 两处写死的 23 改成量出来的边界；包装次序改回 Recording 在外、FaultInjecting 在里 | `m2-gate55-steps-implementer-report.md`、`m2-gate55-wrapping-implementer-report.md` |
| 层 0 发现日志 | 层 0 放量时把发现写进日志、按签名去重、续跑不丢 | `m2-impl-layer0-findings-implementer-report.md` |
| A3a、A3d、A3c、A3b | 坏盘输入的 panic 面（core 读者）；两份测试的坏法与期望跟上；取号时读不出见证槽就拒、`transaction.rs` 与 `inode_tree.rs` 剩下的 panic 面；单元区起点按环长现算（写进系统配置偏移 417、读者判 417 = 现算起点）、mkfs 大环在写之前拒、读不出无声放过的两处改成重读一次仍读不出就拒 | `m2-rev-a3a-implementer-report.md`、`m2-rev-a3d-implementer-report.md`、`m2-rev-a3c-implementer-report.md`、`m2-rev-a3b-implementer-report.md` |
| A3b 善后一 | 8 份测试跟上单元区起点随环长（小盘三档环 6 MiB；记录条数上限在 1 MiB 环的在飞上限 85 上测） | `m2-rev-a3b-fallout-1-implementer-report.md` |
| A3-checker、A3-checker-2、I-7.13 坏镜像、坏镜像改钉拒绝 | checker 与 core 同漏的几处；池级走读补齐 I-2.4（单元头与指针头部）、宽 0 报 I-1.10、系统配置越界报 I-7.13；I-7.13 的坏镜像；A3a 让恢复拒掉的那份坏镜像改钉「可写挂载被拒、盘上不变」 | `m2-rev-a3-checker-implementer-report.md`、`m2-rev-a3-checker-2-implementer-report.md`、`m2-rev-i713-bad-image-implementer-report.md`、`m2-rev-bad-image-refused-mount-implementer-report.md` |
| C577 | 发布返回之前、系统配置轮换之后再一道池屏障：`transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration` 现在是根槽 FUA → 系统配置轮换 → 屏障，屏障做完才返回；三条发布路径、原样重发、抬 F 与卸载那一串都走它 | `m2-impl-c577-barrier-implementer-report.md` |
| 树表排序 I-9.16 | `rebuild_version` 与 `walk_to_file` 在读任何一棵树之前判树表条目按树 ID 严格升序且合发号次序（两道合成 I-9.16）；checker `walk.rs` 另写一份、清单 47 → 48；坏镜像「互换种类」 | `m2-rev-tree-table-ordering-implementer-report.md` |

共用问句与三种结论照第一轮正文（`research/prompts/_m2-closeout-code-r1-body.md`「一、这一轮要判什么」），这里不抄第二份：

> **这一处代码的行为，是从哪条已定分项推出来的？推不出的那些，它是在替一条没写的条款做选择吗？如果是，这个选择今天有没有会红的东西钉着？**

| 结论 | 什么样 | 要交出什么 |
|---|---|---|
| **兑现了条款** | 行为是某条已定分项的字面后果 | 那条分项的**整段原文**，加从原文到这段代码的那一步 |
| **替没写的条款做了选择** | 不同实现会做出不同的、都说得通的选择 | 那个选择是什么、它影响哪些字节或哪条可达历史、今天有没有会红的东西钉着 |
| **和条款说反话** | 代码做的与某条已定分项的字面相反 | 两边各自的原文，以及这次差异在哪个字节 / 哪条历史上看得出来 |

## 二、实现今天的样子（主 agent 的观测，2026-09-27 现查）

- **腿读代码一律读快照**：提交 `30c084138f7da7488f27874417902caeb5272a15`（ref `refs/sop/m2-closeout-code-r2-snapshot`，树 `98cbcaf83b58cb6a2ee6dc8dee589eb0f1cbf81e`）。它是现查那一刻主工作区 `crates/` 的原样，含未跟踪的新测试，不在任何分支上。取法：`git archive refs/sop/m2-closeout-code-r2-snapshot crates | tar -x -C <草稿目录>`。主工作区在腿跑着的时候还会被主 agent 改（下面「在飞」那几件），别读主工作区。
- 快照里 `crates/*/src/**/*.rs` 57 份的 sha256 在 `research/prompts/m2-closeout-code-r2-snapshot/crates-src-sha256.txt`。
- 被判的范围：`git diff refs/sop/m2-closeout-code-r1-snapshot refs/sop/m2-closeout-code-r2-snapshot -- crates/`，71 个文件，+38121 / −2073 行。其中 src 29 份：
  - checker：`image.rs`、`lib.rs`、`walk.rs`；
  - core：`admission.rs`、`allocator.rs`、`code_two_tree.rs`、`inode_tree.rs`、`journal.rs`、`make_filesystem.rs`、`mount.rs`、`mounted_session.rs`、`pointer.rs`、`recovery.rs`、`system_configuration.rs`、`transaction.rs`、`unit.rs`；
  - format：`lib.rs`；
  - harness：`bad_disk_input.rs`、`crash.rs`、`crash_injection.rs`、`fault_injection.rs`、`history.rs`、`layer0_progress.rs`、`model.rs`、`model_comparison.rs`、`bin/first_transaction_on_device.rs`；
  - 实验装置 bin：`bin/e156_allocation_basis_counts.rs`、`bin/e158_root_choice_repair.rs`（对不对由各自实验页的变异表与单测判，判决里只按路径点名）；`bin/e161_crash_state_dedup_and_time_split.rs` 是别的会话（singlefs-e1）的 E161 装置，不在这一轮里，判决里按路径点名、写明归属。
- **在飞、不在快照里、归第三轮的**（用户 2026-09-27 定第二轮与它们并行开）：测试钉值的统一修正（C577 与 A3b 带来的非层 0 钉值，C577 报告「第 1 条盘点」一节的算法；A3b 善后二：随机历史两条撞墙步数、小盘几份、故障注入第 595 行、E158 装置自拼的分配器 Q8）；A3b 第三节 Q7 剩下两处「读不出就放过」（`transaction.rs:368` 写系统配置前算生效 F 调原 `effective_rollback_floor`、`mount.rs` 管理员回退的候选判 `rollback_candidate`）与 Q4 新错误成员归到与乙同一级；善后一报告第五节 P1（抬 F 时挂载那一刻就读不出的根槽照 D16 第 37 行当没有根，只对挂载时读得出、这一次读不出的槽重读一次仍读不出才拒，A3b 今天对每个读不出的槽都拒）；层 0 文件、`.claude/gate.d/stage-inputs.tsv` 注释里的数与 litmus 的静态改。腿打中这几格时写「已知」并点名去向。
- **已知、不算打中**（腿打中这几格时写「已知」并点名去向）：
  - C572（回退目标引用隔离单元不进逐盘验）：第一轮 Z1Q-A 立的账。
  - C573–C576（准入固定点上界、落得下那一判、错误成员、三条用例连带红）、C578（乙的读缓存代价没量）、C579（判据在单故障合法状态上也拒可写）：`.claude/kb/checks-owed.md` 各行。
  - 双故障形「每块盘各坏一槽系统配置，同时最新根一时读不出」：用户 2026-09-27 定不在容错射程、记欠账；`second_transaction_supplement_three_random_history.rs` 与 `second_transaction_step_five_reuse.rs` 那两条同形用例改钉「红在 I-7.4 是已知」归统一修正。
  - 宽 0 而条数非 0 在树表、中央映射树上只记走读失败（C307、C476）；指针头部 MAC / nonce 非 0 只判不断走读（A3-checker-2 第四节第 4 条，主 agent 接受，被攻过零轮——**这一格要攻**）。
  - 树表排序报告第八节三条（② 要不要连中央映射树的号一起比；② 只比树表里有的种类；② 在 `rebuild_version` 里排在水位与缺树判定之前，同时违反几条的镜像报 I-9.16）：主 agent 接受它的取法，被攻过零轮——**这一格要攻**。
  - C577 报告「停下交主 agent 的设计问题」第 2 条：屏障报错照 D23 已定项 14「这一版的失败处置」（冻结、原样重发；零单元发布整个挂载返回错误、那一支没专门用例）：主 agent 接受，被攻过零轮——**这一格要攻**。

## 三、八格

### Y1　最新根读不出、见证与屏障（C554 乙、乙-配置续、A3c 取号拒、C577 屏障、A3b 第 4 件）

`mount.rs` 的重读一次再拒可写、`recovery.rs` 的 `effective_rollback_floor_rereading_the_newest_instance_table_once` 与 `readable_roots_rereading_unreadable_root_ring_slots_once`、`transaction.rs` 的 `highest_system_configuration_journal_tail` 与 `persist_the_root_then_rotate_the_system_configuration`（C577 加屏障之后）。压着 D16（发布语义） 已定项 1、已定项 7（序点四个）与第 37 行「根槽这一次读坏」那一格，D23（journal 的角色与格式） 已定项 14，D18（块里携带什么信息） 已定项 11；岔路单 `research/prompts/c554-fix-forks.md` 第 2、4 行的用户定案。问：有没有一条单故障可达历史，系统配置见证过的最新根照样被当成被抛弃、它的单元被再发出去；有没有哪个入口在任何写之后才拒；屏障报错之后调用方看到的与 D23 已定项 14 那一格一致吗；零单元发布那一支屏障报错怎么走。

### Y2　挂着之后每个入口逐盘核（Z3-A 乙）

`mount.rs` 的 `caller_inputs_agreeing_with_the_disk`、会话发布前读两槽、`mounted_session.rs`。压着 D18（块里携带什么信息） 已定项 11（第一轮之后扩到挂着之后每个收盘表的入口）。问：第一轮 Z3-A 那三形（正常卸载、回退到现行那一版、会话发布时盘 1 换成全零空盘）今天各在哪一步拒、拒之前有没有写；换成别的池的盘、换成同池旧快照的盘、换成只剩一槽自证的盘，各自结局。

### Y3　准入按最坏情况（A4c、A4d、A4e）

`admission.rs`、`allocator.rs`、`mount.rs` 抬 F 的准入、`transaction.rs`。压着 D28（挂载期承诺量） 已定项 1、3、4，D2（RAID 条带策略） 已定项 13，D16（发布语义） 已定项 1「准入」。问：K1 整棵树与逐设备合取在两块盘答不同落点时是不是仍是上界；C545 准入先拒之后有没有一条合法写被拒（假性 ENOSPC），有的话是不是已登记在 C571 / C576 里。

### Y4　坏盘输入与格式边界（A3a、A3d、A3c、A3b、C11b）

`recovery.rs`、`journal.rs`、`pointer.rs`、`unit.rs`、`code_two_tree.rs`、`inode_tree.rs`、`make_filesystem.rs`、`system_configuration.rs`。压着 D22（单元原子性怎么合成） 已定项 2、9、16，D18（块里携带什么信息） 已定项 17，D3（空间分配） 已定项 10 ④，D15（格式冻结政策） 已定项 4，C476（盘上内容可控时走得到的 panic 有 21 处）。问：盘上内容可控时还剩哪条读路径走得到 panic、越界下标或 `as` 截断；单元区起点按环长现算之后，默认环以外的环长上 mkfs、恢复、分配器、checker 读的起点是不是同一个数；读者判「417 = 现算起点」与 mkfs 拒「起点不在段边界」之间有没有一块盘两边都收、或两边都拒。

### Y5　checker 新判定与树表排序（A3-checker、A3-checker-2、I-7.13 坏镜像、I-9.16）

`walk.rs`、`image.rs`、`lib.rs`，core `recovery.rs` 的 `rebuild_version` 与 `walk_to_file` 判 I-9.16 那一处。压着 `.claude/kb/invariants.md` I-2.4、I-1.10、I-7.13 三行（整行抄）、D8（核心索引结构） 已定项 8、D13（验证路线） 已定项 7。问：每条新判定是不是只在它该有对象的状态上有对象（不适用与判绿分得开）；I-7.13「一槽越界、其余全报不适用」会不会把别的不变量本该报的违例一起盖掉；I-9.16 两道合一、只比树表里有的种类、排在水位与缺树判定之前，各是条款的字面后果还是选择；checker 与 core 除常量模块之外有没有共用判定（门禁 94 号）。

### Y6　验证装置（B3a-3c、B3a-3d、B3c-2、B3c-3、乙-模型、55 号两处、层 0 发现日志）与第一轮没罩到的撕裂与续跑

`crash.rs`、`crash_injection.rs`、`layer0_progress.rs`、`history.rs`、`model.rs`、`model_comparison.rs`、`bin/first_transaction_on_device.rs`、`lib.rs`。压着 D13（验证路线） 已定项 4、7。问：journal 环转圈之后的原地覆写撕裂有没有进状态枚举；续跑与分片在被杀、重起、两片合并时有没有状态被跳过或重复记绿；模型与实现在「重读一次仍读不出就拒可写」那一格上是不是各算各的；C577 之后录制流每次发布多两步、轮换那 2 写自成一段，`segments.rs` 的关段规则是不是 D13 已定项 4 的字面后果。**这一格要造输入**（第一轮只读了代码）。

### Y7　合并点

第一轮快照之后，`mount.rs` 被 A4c、C554 乙、Z3-A 乙、55 号包装、A3a、乙-配置续、A3b 各改过，`transaction.rs` 被 C11b、A4c、A4d、A4e、B3c-2、B3c-3、C554 乙、Z3-A 乙、55 号包装、A3a、乙-配置续、A3c、C577 各改过，`recovery.rs` 被 C11b、B3c-3、A3a、A3b、I-9.16 各改过；都是主 agent 用 `research/scripts/apply-writer-patch.py` 从各自副本的补丁打进来的。问：同一条发布路径上各批加的判定、重读与屏障，次序与各自报告里写的一样吗；有没有哪一批加的拒绝在另一批加的写之后才判（写了再拒）；A2a 同一段同一块盘两次系统配置写（第一轮没量）。

### Y8　算术（本地攻方）

按事实表逐格算数、与仓里钉的数比：C577 之后第一条流的段序列 `2+2+1+2+2+1+2+24+2+1+2` 与两态闭式 16777240（C577 报告「钉值怎么变」一段）、隔离看第一个事务 33 → 35 步、σ 段闭式 2^18 → 2^16；单元区起点在 768 MiB 环（50176）、6 MiB 环（1408）、159 MiB 环（11200）上的值与「在 64 槽段边界上」；I-7.13 各字段的界（槽距 ∈ [4096, 1 MiB − 4096]、`physical_block_size` ∈ [457, 槽距]）；1 MiB 环的在飞上限 85。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端攻方（Opus） | Y1、Y2、Y4、Y6、Y7 | 找反例：在快照副本上写用例跑出来。与第一轮不重叠的攻击面：第一轮修补本身（Z3-A 乙、C554 乙一族）、产生结论的方法（第一轮「没打中」的几格是不是只读了代码）、从未被看过的地方（撕裂、续跑、分片、合并点） |
| 云端正推（Sonnet） | Y3、Y5 | 逐格核「代码做的是不是条款说的」，三种结论每格落一种 |
| 本地攻方 | Y8 | 只算数，不造历史；与云端攻方不重叠 |

第三轮派辩方腿复核第一、二轮的判决（`.claude/rules/three-way-inference.md`「多轮」那一节），并攻这一轮「在飞」那几件的改后代码。

## 五、交付

每条腿的报告按格写：结论（三种之一）、原文、代码位置（快照里的 `文件:行号`）、打中的给复现（命令、种子、原样输出）。攻方写的用例与模型放各自的模型目录，在快照副本上跑，不改主工作区。

## 六、回看决策

判决里写。
