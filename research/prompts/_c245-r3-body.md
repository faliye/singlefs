# C245（系统配置槽的写频率，两条条款说反话） 第三轮正文（2026-09-28）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 -->

## 一、这一轮要判什么

C245 第一、二轮判决：`research/prompts/c245-r1-main-verification.md`、`research/prompts/c245-r2-main-verification.md`（整份进材料）。第二轮之后的结论与后来的定案：

| 事 | 出处 |
|---|---|
| tail 只当扫描起点时不承重（两轮三条腿一致）；承重的是链首锚与链尾发布边界 | 第二轮判决第四节末段 |
| 链首锚取同 txg 里 jsn 最大那条、读不出时靠本次发布内序号；链尾按记录末条标志判定；都已实现 | `.claude/kb/milestone/02-second-txn.md` 增补 2 收口表第 6 行；D23（journal 的角色与格式） 已定项 14 |
| 回退改成挂着时的一次向前发布，回退见证删掉 | D23（journal 的角色与格式） 已定项 14；用户 2026-09-25、2026-09-26 定 |
| F 另记进系统配置（SysPre），抬 F 那一串先逐盘写系统配置、过一道屏障 | D16（发布语义） 已定项 1、已定项 7 |
| 取号那一写把见证值写进系统配置 tail（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续） | `.claude/kb/checks-owed.md` C554、C577 两行 |
| 发布返回前加屏障：根槽 FUA → 系统配置轮换 → 池屏障，屏障做完才返回，知情接受每次发布四个序点 | D16（发布语义） 已定项 7「屏障口径」，用户 2026-09-27 定；C577（系统配置没见证到的最新根，乙罩不到） |
| 系统配置槽写的频率在条款层已统一成「每次发布写一次」，门禁 20 号 write-frequency 格防回归 | D23（journal 的角色与格式） 已定项 3 改动 2；`.claude/gate.d/write-frequency.tsv` |

**还开着的是 D23（journal 的角色与格式） 已定项 3 射程里那一句**：当初选「tail 住系统配置槽」用的代价表是 E23（journal 几何） 的「两条臂差 21 : 500」，它建在「tail 按 checkpoint 推进、比 fsync 稀疏得多」上（E23 按 20000 次 fsync、每 1000 次一个 checkpoint 算，系统配置那条臂稳态多写 21 块）。第一版每次发布都是 checkpoint（D16（发布语义） 已定项 6「fsync 触发的发布也是发布」），这个前提不成立。**这一轮判：按每次发布一次重算之后，「每次发布轮换一次系统配置槽、并在返回前等它持久」这件事的依据是什么、站不站得住；站不住或有更便宜的放法，候选各自的代价数是多少。** 原题「tail 承不承重」前两轮已答，不再重判；本轮问的是系统配置轮换**今天实际被谁用**，E23 那笔账作废之后顶在这一格上的依据是哪一条。

### 要腿答的四问

| # | 问 | 答到什么程度 |
|---|---|---|
| Q1 | **代价重算**：第一版一次发布里系统配置轮换多出来的写数、字节数、序点（按 [layout/01-first-txn.md](../../.claude/kb/layout/01-first-txn.md) 第八节段序列登记表与 D16 已定项 7 的屏障口径），占一次发布的比例；E23 的 21 : 500 按每次发布一次重算变成多少 | 写出式子与数，数的来源写文件与行号；pbs = 512 与 4096 两档各算 |
| Q2 | **消费者清单**：`crates/` 今天读系统配置里每次发布都会变的量（journal tail、实例代号）的每一处，逐处判：轮换若不再每次发布做（例：只在取号、抬 F、卸载时写），它的判定会变成什么、在哪个崩溃状态或故障下判错 | 逐处给源码落点；判「承重 / 不承重」要给一个具体的状态或反例 |
| Q3 | **候选**：甲 = 今天（每次发布轮换、返回前等它持久）；乙 = 系统配置只在取号、抬 F、卸载时写，见证值与逐盘「落后」改由别处出（你给出最强的一种：根记录、journal 记录头或别的已有盘上量）；丙 = 每次发布轮换但不等它持久就返回；另可提丁。逐个答：Q2 那张清单上哪几处要改、会不会出现新的错判状态、代价 | 候选的定义写全；每个候选的代价按 Q1 的口径给数 |
| Q4 | **射程句写回**：D23（journal 的角色与格式） 已定项 3 那句「按新频率重算之后……是 C245 题面上的三方论证」该改成什么；依据指针指到哪一条 | 给出改后的整句 |

**跑前写死的判决条款**：
- Q2 里至少有一处「承重」且那一处在乙、丙下给得出判错的具体状态，而乙、丙的代价（Q1 口径）没有比甲省下一个数量级 ⇒ 甲站住，依据由「21 : 500」换成 Q2 那几处消费者，写回射程句，C245 这一题关。
- 乙或丙在 Q2 清单上一处都不判错、且代价省下可量的一截 ⇒ 成岔路，写岔路单 `research/prompts/c245-r3-forks.md`，按三方纪律带代价数交用户；它与用户 2026-09-27 的「发布返回前加屏障」定案冲突的，交用户时写明。
- 三条腿对「承重」判法不一致 ⇒ 主 agent 去查能分辨它们的那个状态，不投票。

## 二、实现今天的样子（主 agent 的观测，2026-09-28，HEAD `e5253e8a` 加工作区）

- 发布路径：`crates/singlefs-core/src/transaction.rs` 第 134 行 `CommitStep::RotateSystemConfigurationSlots { journal_tail, … }`；第 1201 行 `persist_the_root_then_rotate_the_system_configuration`（根槽 FUA → 系统配置轮换 → 池屏障，屏障做完才返回）；第 744 行 `acquire_instance`（取号那一写带见证值）。
- 挂载路径读 tail 的三处：`crates/singlefs-core/src/mount.rs` 第 3031 行起 `journal_position_of_the_selected_version` 与 `devices_without_the_selected_version`（到第 3137 行）（可写挂载取号之前逐盘核「落后且缺单元」，D18（块里携带什么信息） 已定项 11）；第 3140 行起 `devices_behind_the_current_version`（挂着之后收盘表的入口的落后支）；第 4134 行 `newer_publish_witness`（判据 N-配置：各盘自证系统配置槽里 journal tail 的最大值当见证计数，与所选那一版的末条记录比）。
- 恢复路径不读 tail：`grep -n "journal_tail" crates/singlefs-core/src/recovery.rs` 零命中；每次挂载一律全环扫描（D23（journal 的角色与格式） 已定项 14「没有干净关闭标记」，C499（干净关闭标记有没有没有条款） 已还清）。
- 字段：`crates/singlefs-core/src/system_configuration.rs` 第 259 行 `journal_tail`。
- 用例：`crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs`（C577，去掉返回前那道屏障 3/3 红）；`crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs`（落后支）；`crates/singlefs-harness/tests/unreadable_abandoned_root_slot.rs`（C554）。

## 三、条款（材料员整段抄进附录）

- D23（journal 的角色与格式） 已定项 3（整节，含射程与三条硬要求）、已定项 14（整节）；
- D16（发布语义） 已定项 1、6、7、8；
- D18（块里携带什么信息） 已定项 11 里「取号之前逐盘核带不带所选那一版」那一段所在的整节；
- D22（单元原子性怎么合成） 已定项 9、16；
- E23（journal 几何） 实验页的「结论」一节；
- `.claude/kb/checks-owed.md` 的 C245、C554、C577、C583 四行；
- `.claude/kb/layout/01-first-txn.md` 第八节（段序列登记表）与第一节系统配置字段表；
- 第一、二轮判决整份。

## 四、分工

本地腿：攻（Q1 的代价算术，攻击面是「逐格算数」，与云端攻方的反例构造不重叠；前两轮判决的复核这一轮不做，题面已换，所以不派辩方）

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | Q1–Q4 全部 | 从 `crates/` 今天的实现出发，把 Q2 的消费者清单逐处列全、逐处判承不承重，给出甲站住或不站住的完整论证与 Q4 的改后整句 |
| 云端攻方（Opus） | Q2、Q3 | 专攻「甲的依据」：先写出乙、丙各自最强的一种，再在 Q2 的每一处上造「乙 / 丙判错」或「甲自己也判错」的状态；每个打中做成一个必须报非 0 的小模型或钉在真代码上的探针。另攻：Q2 清单漏没漏读者（全仓搜 `journal_tail`、`quantities`、`verified_system_configuration_slots` 的调用方） |
| 本地攻方 | Q1 | 按事实表逐格算：pbs = 512 与 4096 两档，一次发布的写数、字节数、序点、系统配置轮换所占比例；E23 按 20000 次 fsync 的口径重算两条臂。事实表每行写来源文件与行号，每张表不超过 6 行 |

## 五、交付

- 腿的报告写 `research/prompts/c245-r3-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建），回复只写短句。
- 模型放 `research/prompts/c245-r3-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 写 kb 文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 方案按 `crates/` 今天的实现来谈。
- **资源**：线程上限 4；编译与跑一律经 `bash research/scripts/run-with-memory-cap.sh 8G <命令…>`；只停自己起的进程；交回之前后台不许留着跑的东西。
- 不跑 `cargo test -p singlefs-checker-tier`，不跑门禁 54、55、57、59 号，不跑 harness 耗时用例。
- 不写时刻，时区名也不写；变体起新名字，不用撇号角标。
