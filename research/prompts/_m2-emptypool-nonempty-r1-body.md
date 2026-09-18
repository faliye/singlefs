# 背景材料：里程碑「第二个事务」空池可写挂载与「非空」按树表认，代码三方对抗第一轮（2026-09-17）

<!-- doc-lint:not-numbers V1 V2 V3 Z1 Z2 Z3 Z4 Z5 Z6 Z7 -->

## 一、被判的对象

2026-09-17 用户在收尾弹窗里定了两件要改代码的事，实现员落地、主 agent 复核（`check.sh` 绿、命名纪律绿、`crates/mutations.tsv` 66 行段数与锚点都对；门禁 59 号全表与层 0 三条流在跑）：「非空持久有效根」从盘上按树表认（D16（发布语义） 已定项 1 表格后那段 ⚠️）；只做过 mkfs 的池允许可写挂载。被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-harness/tests/common/mod.rs`、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`（新）、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`（新）、`crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`、`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`crates/mutations.tsv`。实现员报告 `research/prompts/m2-emptypool-nonempty-r1-implementer-report.md`（可读，它自己列的「停下交主 agent 的设计判断」在第四节与续做一节）。

**实现今天的样子**（主 agent 读过的 `crates/` 路径与看到的事实，都是观测；方案按 `crates/` 今天的实现来谈）：

- `crates/singlefs-core/src/mount.rs` `rollback_floor_ceiling`（第 449 行起）：不再读 journal；有效根 = 可读 ∧ 按实例表不被抛弃 ∧ txg ≥ 当前 F；按 (txg, 实例) 排，一条有效根算非空 ⟺ `recovery::user_visible_tree_root_pointers`（`recovery.rs` 第 702 行）读出的 inode 树与 extent 树根指针（盘上 86 字节原样）与它前一条有效根的不同，最旧那条与「两棵树都没有」比；任一有效根的树表读不出 ⇒ 返回 `MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`，`raise_rollback_floor` 在重算影子账、回收、发布之前用 `?` 返回。取第 4 新、取 min 在 `ceiling_from_newest_and_non_empty_roots`（第 522 行）。
- `recovery.rs` `rebuild_version` 返回 `RebuiltVersion`（第 464 行），树表 0 条时是没有文件的那一版；`transaction.rs` 的 `PoolVersion`（第 450 行，WithoutFile / WithFile）取代 `Mounted.current` 等处的 `TransactionOutput`；`publish_without_units`（第 392 行）是零单元发布，`warm_up` 调它、字节不变。
- `mount.rs` 的 `PreviousVersion`（第 153 行）两个成员；`format_time_allocator`（第 393 行）给树表 0 条的一版建分配器：只有 mkfs 写在单元区里的实例表（跨 2）与树表（跨 1），实例表或树表指针的诞生 txg 不是 0 ⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`（今天实现写不出这种根，只有坏盘能构造）。
- 只做过 mkfs 的池可写挂载：`transaction::instance_generation_to_acquire`（第 287 行，只读）先算要取的号，`refuse_instance_rows_on_version_without_file`（`mount.rs` 第 744 行）判 [max(所选根的实例, 1), 要取的号) 不为空就在取号之前返回 `InstanceRowsOnVersionWithoutFileUnsupported`（第一个事务暖机两次之后、文件版本之前崩溃再重开就是这一格）；为空（上一个实例是 0）就取号 1、零单元写行（txg 1、jsn 1、落盘 1）、零单元暖机（txg 2、jsn 2、落盘 0），之后同一个进程里 `publish_first_file` 发第一个文件版本（txg 3，它写死 txg 3，只在这一格上恰好对）。回退到树表 0 条的根 ⇒ `RollbackToVersionWithoutFileUnsupported`，在任何写之前（第 1017 行）。所选根有文件而环里一条记录都没有 ⇒ `FileVersionWithoutAnyJournalRecord`。`NoPublishedVersion` 删了。
- 用例：`second_transaction_step_three_formatted_pool.rs`（mkfs 之后可写挂载、第一个文件版本冷读回；mkfs 之后与挂载之后 checker 8 条报不适用、18 条成立，第一个文件版本之后 26 条成立；暖机之后崩溃再重开被拒且超级块逐字节不变）、`second_transaction_step_four_rollback.rs`（回退到 (1, 2) 被拒、盘上不变）、`second_transaction_step_five_reuse.rs`（txg 14 的记录两份都坏时上限仍 11；回退那次发布与前一条有效根 A 比、不与被抛弃的 C 比；有效根树表读不出时抬 F 被拒）；层 0 第三条流 `second_transaction_step_three_formatted_pool_layer0.rs`（段序列 `2+2+1+2+2+1+18+2+1+2`、闭式 262165、快测 22 个状态），门禁 54 号已接。
- 变异：`crates/mutations.tsv` 第 62–71 行十条（非空退回按事务号、前一条取任意可读根、只比 inode 树、分配器不认 mkfs 两个单元、零单元暖机反向链写 0、零单元发布少一道屏障、mkfs 实例表按 1 槽记、拒绝挪到取号之后、回退到树表 0 条的根不拒、树表读不出按空猜）。
- `crates/` 里没有的：树表 0 条的一版上重写实例表（写行、回退）的实现；拒绝之后调用方怎么办（只读挂载、修复）；在任意没有文件的一版上发第一个文件的通用发布（`publish_first_file` 写死 txg 3）；抬 F 在没有文件的现行版本上的入口。

## 二、改法（每条都是可以被攻的推论）

| 编号 | 改法 | 压着的条款（原文在附录） |
|---|---|---|
| V1 | 「非空」按树表里 inode 树与 extent 树的根指针与前一条有效根比；树表读不出就拒绝抬 F | D16（发布语义） 已定项 1 表格「抬 F 的上限」一行与表格后「非空」那段 ⚠️ |
| V2 | 只做过 mkfs 的池（上一个实例是 0）可写挂载：取号 1、零单元写行与暖机、同一进程里发第一个文件版本 | D16（发布语义） 已定项 9（树表 0 条时空发布写零个单元）、已定项 8（暖机）；D18（块里携带什么信息） 已定项 11（实例 0 不写行）；D23（journal 的角色与格式） 已定项 14（切换与回退） |
| V3 | 树表 0 条的一版上要写实例表行、回退到树表 0 条的根、实例表或树表不是 mkfs 写的，都在任何落盘之前拒绝 | 同 V2；用户 2026-09-17 对「读不出」的态度（出现就要走修复，不许跳过） |

## 三、判据（跑前写死；每条写「什么观测会让它触发」）

| 判据 | 问 | 触发的观测 |
|---|---|---|
| Z1 非空判法的漏判与误判 | 一次改过用户可见状态的发布，树表里 inode 树与 extent 树的根指针会不会与前一条有效根逐字节相同（例如同一个对象写回同样的内容、落点复用到同一个槽、校验和相同）；一次没改用户可见状态的发布（写行、暖机、抬 F、回退、切换）会不会让这两棵树的根指针变（重写、照抄时出生代或位置条目改写）；「前一条有效根」在回退、切换、两次回退、F 刚抬过的历史上取到的是不是该比的那一版 | 一段历史让上限多算或少算一格 |
| Z2 拒绝的时机 | 三处拒绝真的在任何落盘之前吗：`instance_generation_to_acquire` 与 `acquire_instance` 之间有没有别的写或能让两次算出不同号的状态（两盘超级块不一致、一盘槽坏）；回退的拒绝之前有没有读路径以外的副作用（分配器、录制流、缓存） | 一段历史让拒绝返回时盘上已经有一次写 |
| Z3 空池挂载之后的第一个文件版本 | `publish_first_file` 写死 txg 3：在只做过 mkfs 的池上挂载时 txg 3 恒成立吗（根环里有 txg > 0 的根的池会不会走进这条路）；第一个文件版本换下 mkfs 的第 0 版树表时，释放、defer、记账与第一个事务那条路逐字节相同吗；两块盘超级块实例代号、根环、journal 在这条路上与 mkfs 同一个进程里跑第一个事务那条流逐字节相同吗 | 一个镜像在两条路上字节不同，或 txg 不是 3 时走进这条路 |
| Z4 层 0 第三条流 | 闭式 262165、段序列与第一个事务相同：新流的枚举是不是真的跑了重开之后的恢复与挂载（还是只重放了与第一个事务相同的写）；零单元发布的崩溃状态（记录写到一半、根写到一半）恢复出来是什么、checker 那 8 条不适用在崩溃状态上会不会掩住违例 | 一个崩溃状态按恢复语义不合法而 26 条全绿或不适用 |
| Z5 代码与条款 | D16 已定项 9「树表 0 条 ⇒ 零单元」的射程罩不罩得住写行那一次（它是新实例的第一次发布、按 D16 已定项 8 与 D18 已定项 11 要写行，行为空时写不写实例表）；「非空」那段 ⚠️ 与代码的「前一条有效根」「(txg, 实例) 排序」「最旧那条与没有比」是不是同一件事；`FileVersionWithoutAnyJournalRecord` 那一格有没有条款 | 一句代码与条款字面不符 |
| Z6 变异与用例 | 第 62–71 行十条变异各自点名的用例是不是真钉住那条改法（改回去它红、别的改法改回去它不红）；三处拒绝的用例断言的「盘上逐字节不变」比的是哪些字节（超级块槽、根环、录制流步数），漏了什么 | 一条变异点名的用例在别的改法上也红，或一处写没被「逐字节不变」罩到 |
| Z7 写回 | `.claude/kb/milestone/02-second-txn.md` 步 3、步 5 现状，`.claude/kb/layout/01-first-txn.md` 八「只做过 mkfs 的池的可写挂载」一行，与代码对不对得上 | 一句现状与代码不符 |

**反向接受条款**：Z1–Z4、Z6 打中 ⇒ 改代码、补会红的用例与变异行；打中的格落在没有条款的地方 ⇒ 记进里程碑对应步的决策点、标预想交用户，代码按最保守的读法改（拒绝优先于猜）；Z5 / Z7 打中 ⇒ 改现状或记决策点。
**失败条款**：一条腿引的条款与附录原文不符 ⇒ 那一格作废；「没问题」一次不算（本地腿抽两次）；腿在副本上量出的数不进 kb。

## 四、腿的分工（三条推论腿，2026-09-17 用户定的新规则：云端攻方、云端正推、本地攻方）

| 腿 | 立场 | 分到的格 | 不许碰的格 |
|---|---|---|---|
| Opus 攻方（`three-way-attack`） | 造可达的历史或镜像打穿 V1–V3 | Z1、Z2、Z3、Z4 | Z6（本地攻方的） |
| 本地攻方（`three-way-local-attack`，英文，抽两次，按表逐格填） | 同上，另一组攻击面 | Z6 | Z1–Z4 |
| Sonnet 正推（`three-way-forward`） | 核 V1–V3 的代码做的是不是条款说的 | Z5、Z7 | 不替攻方找反例 |

## 五、禁读清单（各条腿）

这一轮别的腿的输出（`research/prompts/m2-emptypool-nonempty-r1-*-output*.md`）与主 agent 的核实（`research/prompts/m2-emptypool-nonempty-r1-main-verification.md`）。前几轮判决可读：`m2-step45-code-r3-main-verification.md`、`m2-step6-checker-r1-main-verification.md`。本地腿的提示避开 Rust 路径 `::`、不写汉字变量名。代码行号以腿们开工时的主树为准；主 agent 另存了一份快照 `/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/emptypool-r1-legs-snapshot/`（`crates/` 与 `.claude/kb/`），核查员对快照核。
