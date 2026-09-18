# 里程碑「第二个事务」空池可写挂载与「非空」按树表认的代码三方对抗第一轮：主 agent 核实（2026-09-17）

<!-- doc-lint:not-numbers V1 V2 V3 Z1 Z2 Z3 Z4 Z5 Z6 Z7 F1 F2 F3 F4 F5 -->

材料：正文 `research/prompts/_m2-emptypool-nonempty-r1-body.md`（V1–V3 三处改法、Z1–Z7 判据、三条腿的分工，按 2026-09-17 用户定的新规则一轮三条推论腿）、背景材料 `_m2-emptypool-nonempty-r1-background.md`（正文 + diff + 附录）。
三条腿：云端攻方（Opus，`m2-emptypool-nonempty-r1-opus-output.md` 280 行，模型与日志 `m2-emptypool-nonempty-r1-opus-model/`）、云端正推（Sonnet，`m2-emptypool-nonempty-r1-sonnet-output.md` 213 行）、本地攻方（`m2-emptypool-nonempty-r1-local-attack-output-s1/s2.md` 两份干净样本，两次调用即得）。核查员 `m2-emptypool-nonempty-r1-verifier-output.md`（对腿们开工时拍的快照核）。

被判的文件（门禁 56 号按路径认）：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/records.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-harness/tests/common/mod.rs`、`crates/singlefs-harness/tests/first_transaction_step_one_mkfs.rs`、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs`、`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool_layer0.rs`、`crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs`、`crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`、`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs`、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`crates/mutations.tsv`。

## 一、云端攻方（Opus；Z1、Z2、Z3、Z4）

| 格 | 报告 | 主 agent 核 | 处置 |
|---|---|---|---|
| Z1 | 打中 1 条（问句中、触发句不中）：抬 F 到 11、E 复用 50178、txg 16 的根槽坏一字节 ⇒ 重开之后 F_生效 回落到 0，第 0 代根重新进有效根，它的树表单元已被合法复用、读不出 ⇒ 此后每一次抬 F 都按「有效根树表读不出」被拒，直到根环把引用 50178 的根盖掉；上限没算错，是没算出来。另 6 种形状没打中 | 核实机理（核查员在快照副本上重跑逐字相同）。病根是 F 回落，三种「非空」读法共用；V1 的「读不出就拒绝」在上面又叠了一道，让准入要抬 F 的时候抬不动 | 代码不动：「生效」一行已打回重议，这一格并进那一次重议（D16（发布语义） 已定项 1 表格后那段 ⚠️ 补一句，第四节） |
| Z2 | 打中 1 条：判定与取号各读一遍超级块，两盘第 2 次读槽 0 各报一次瞬时读错 ⇒ 判定算出号 1 放行、取号算出 2 并写进两盘超级块，之后 `assert!` panic | 核实（核查员复跑逐字相同）。代码层的错，条款没问题 | **F1 已改**：号只算一次，取号在写之前重算、对不上返回 `InstanceGenerationChangedBeforeAcquisition`、不写；用例 `transient_superblock_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write`；变异第 72 行 |
| Z3 | 打中 2 条：A——两盘超级块槽 0 各坏一字节加一次崩溃 ⇒ 空池挂载放行、新实例从 txg 2 起、暖机推到 txg 4，`publish_first_file` 写死的 txg 3 盖在暖机槽上返回成功，冷恢复落到 txg 4 读不到文件、checker 零违例；B——mkfs 接受条款写死之外的区域归属 [0, 1, 1]，零故障写出两条不同的 (1, 3) 根 | 核实。A 是代码层：写死常量用在了前提不成立的地方；B 是 mkfs 没核条款（`.claude/kb/decisions/02-RAID条带策略.md` 第 263 行写死 0 / 1 / 0） | **F2、F3 已改**：mkfs 两块盘时区域归属不是 [盘 0, 盘 1, 盘 0] 就在写之前返回 `RegionDevicesNotTheFirstVersionLayout`；空池挂载只放行「第一次发布 txg 1、jsn 1，零单元写行与暖机落在不同的盘上」那一格，其余在取号之前返回 `FormattedPoolMountNotShapedLikeTheFirstTransaction`；`publish_first_file` 写之前核上一版记录是 txg 2、jsn 2，不是就返回 `FirstFileVersionNotRightAfterTheSecondWarmUp`。用例各一条，变异第 73–75 行 |
| Z4 | 问句后一半坐实、触发句不中：第三条流的基线镜像、写表（带内容）、段序列与第一条流逐项相同，枚举时不重开挂载——全量等于把第一条流再跑一遍；在同一批崩溃状态上补跑可写挂载，46 个取样 38 个被拒、8 个放行，按代码推全量 262165 个里 262157 个被拒 | 核实 | **F4 已改**：删掉全量枚举，换成快用例钉住两条流逐项相同（变异第 76 行）；门禁 54 号撤回到两条流；段序列登记表那一行改写。崩溃状态上重开可写挂载被拒的那一大片是第五节的设计缺口 |
| 射程观测 | 零故障、零崩溃：只做过 mkfs 的池可写挂载一次、不写文件、进程正常退出，之后每一次可写挂载都被拒（要给实例 1 写行，而这一版没有分配记录树与记账树）；第一次挂载崩在取号那两写里也进这一格 | 核实（核查员复跑 V2 那几行逐字相同） | **设计缺口，交用户**（第五节 1）：用户选的「允许可写挂载」今天只兑现第一次 |
| 顺带 | `RaiseNeedsWritableMountInThisProcess` 在「空池挂载之后同一进程发了第一个文件版本」时名字与事实不符 | 核实 | **F5 已改名** `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`，行为不变 |
| 顺带 | checker 只看一份镜像，看不到「最后确认的那条根是不是择根序里最新的」——Z3-A 那份镜像 checker 零违例 | 核实 | 记欠账（第四节） |

## 二、云端正推（Sonnet；Z5、Z7）

| 格 | 判 | 主 agent 核 |
|---|---|---|
| Z5-a（D16 已定项 9「树表 0 条 ⇒ 零单元」罩不罩得住写行那一次） | 部分不一致：条款主语是空发布，举例是第一次可写挂载的暖机；给写行定语义的是 D18（块里携带什么信息） 已定项 11 | 核了两条原文：D18 已定项 11 写「实例 0 不写」，上一个实例是 0 时写行区间为空，那一次就是一次空发布，写出的字节与第一个事务同一进程里的第一次暖机逐字节相同（快用例钉住）——落在已定项 9 的字面里；代码把它叫「写行那次发布」只是名字。不改 |
| Z5-b（「非空」那段 ⚠️ 与代码） | 两处是实现员补的：按 (txg, 实例) 排、最旧那条与「两棵树都没有」比 | 核实。txg 全池递增、根环里不会有两条根同 txg（新实例第一次发布取环里最大 + 1），实例那一维不起作用；最旧那条与「没有」比是字面最近的读法。写进里程碑步 5 现状标预想，不另交用户 |
| Z5-c（四个拒绝有没有条款） | 都没有；`FileVersionWithoutAnyJournalRecord` 既没进决策点清单、代码也没点破 | 核实。这四个与续做二新加的三个都是「条款没写、拒绝优先于猜」，全部记进里程碑步 3 的决策点 |
| Z7 写回 | 一致 | 一致；续做二之后的改动由主 agent 再写回一次 |

变异复跑：副本上第 62–71 行十条各红、还原绿（核查员复跑一致）。

## 三、本地攻方（Z6；两份干净样本，按表逐格填）

| 问 | s1 / s2 | 主 agent 核 |
|---|---|---|
| 网格 A / B（每条变异还会连带打红哪些用例、每条用例还会被哪些变异打红） | 两份在十条变异里有七条填得不同 | 按「一条腿只抽一次样不算一次观测」记不稳定、不采。每条变异红在点名的用例上由门禁 59 号全表钉着（71 条各红）；「只红点名那一条」不是判据要的 |
| 网格 C（拒绝之前可能的七种写，`DiskSnapshot` 罩不罩得到） | 两份一致的格：单独一道屏障罩不到（录制流把连续屏障合并）；抬 F 那条用例不比可读根 | 不中：屏障不改盘上字节；抬 F 那条用例比录制流步数，任何根槽写都会让它变 |

## 四、处置（都已落地：`check.sh` 绿、命名检查在这一轮改动的文件上无违例、`crates/mutations.tsv` 71 行六段齐、原文恰好命中一次；门禁 59 / 57 / 55 / 54 号在最终的树上跑，结果写回里程碑）

1. F1–F5 见第一节。实现员报告 `research/prompts/m2-emptypool-nonempty-r1-implementer-report.md`「续做二」一节（每条用例怎么证明会红）。
2. 欠账（今天不改代码）：checker 看不到「最后确认的根不是最新的」（Z3-A 那份镜像零违例）；「非空」的两处补的细节标预想；Z1 那一格写进 D16 已定项 1「生效」重议那段。
3. 另一个会话（singlefs-8e）拿 fe1acd72 / 0d4db4d0 / ad1ea35c 当计数实验基线：F1–F5 没碰「非空」判法与 `rollback_floor_ceiling`（`recovery.rs` 未改），`mount.rs` 与 `transaction.rs` 的哈希变了，改动落在取号、空池挂载的形状判定与 `publish_first_file` 的核对，已通知。

## 五、交用户的决策点（都标预想）

| # | 决策点 | 今天的代码（预想） | 说明 |
|---|---|---|---|
| 1 | 没有文件的一版上怎么写实例表行（设计缺口） | 拒绝：只放行与第一个事务同形的那一次可写挂载 | 零故障可达：挂上不写就退出，之后永远不能可写挂载；第一次挂载崩在取号里也一样。要让「允许可写挂载」真兑现，得定这一版重写实例表时它的落点记在哪、换下的 mkfs 实例表谁护着——可能的方向是空池第一次挂载就把记账树、分配记录树建出来（像第一个事务那样，只是没有文件），要走决策与三方 |
| 2 | 回退到树表 0 条的根 | 拒绝 | 与 1 同一个缺口 |

## 六、核查员（`m2-emptypool-nonempty-r1-verifier-output.md`，305 行；对快照核）

判别力自证过。核了 85 处：✓ 81、✗ 或部分 4、核不动 1。三条腿的复跑都逐字相同（攻方 9 条用例的数据行、正推十条变异、本地样本的两道闸）。四处：正推引 D18 已定项 11 时去掉了原文的加粗标记（字不差）；攻方一处松散指位偏 2 行；本地攻方提示里一个函数的文件标成 `transaction.rs`、实为 `mount.rs`（同一份提示里变异条目已订正，样本推理没受影响）；本地攻方提示漏抄一行文档注释。没有一格判决建在这四处上。

## 七、这一轮改法被攻过几轮

V1–V3 被三条腿各攻或核过一轮。F1–F5 是这一轮打中之后的改法，被攻过零轮；它们都是「在写之前拒绝」或改名，用例与变异钉着。按新规则第三轮之后停：这组改动不再为 F1–F5 单开一轮，交用户的表第五节 1 是真正要定的；那一格定了之后的实现再走三方。
