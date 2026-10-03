# 根环实现与系统配置 R2：代码轮第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R2 Q1 -->

## 一、题面

第 7 批 7-2 把根环那一簇写进了 `crates/`（`records/2026-09-30-里程碑三开工第一轮.md` 第 7 批），合入后验证又跟了两份修补。这一轮判：代码做的是不是条款或用户定案说的，哪一格会错。

改动全文在 `research/prompts/_m3-root-ring-code-r1-diff.md`（三份补丁，各带实现员报告的路径），冻结副本 `/tmp/claude-1000/m3-root-ring-code-r1/tree/`：

| # | 改动 | 主要落点 | 压着的条款或定案 |
|---|---|---|---|
| A | 新根段加三字段（实例表指针、分配记录树根、4 字节卸载记号），头 311 → 487、新根段 188 → 364、点名项 67 → 64；恢复由记录重建的根七项全取末条记录、只有 fsid 照所选根；卸载记号位 0 之外有位为 1 当记录损坏 | `crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/journal.rs`、`recovery.rs` 的 `replay_journal`、`transaction.rs` 三处装记录 | D23（journal 的角色与格式） 已定项 4、已定项 12、已定项 15；判决 `research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行；用户 2026-10-02「回到甲」；用户 2026-10-03 弹窗定卸载记号其余位当记录损坏（调度记录第 7 批） |
| B | 一圈窗口（txg + 3S > 锚）进分配器（挂载重建、进程内、抬 F）、池级 checker 候选过滤与 I-3.1 丁-defer 门槛、影子账候选过滤；挂载时读不出的槽门槛取最小 txg、根环表记最大 txg，且不算「这个进程知道住着根的槽」；读不出槽那段循环改成头尾两段各至多 3S 轮 | `crates/singlefs-core/src/root_ring.rs` 的 `txg_is_inside_the_one_revolution_window`、`allocator.rs` 的 `RootRingOccupancy`、`mount.rs` 的 `rebuilt_allocator` / `raise_the_floor_through` / `isolate_slots_referenced_only_by_abandoned_roots` / `unreadable_ring_slots_inside_the_window`；`crates/singlefs-checker/src/walk.rs` 的 `OneRevolutionWindow`、`candidate_indexes`、`deferred_threshold` | D16（发布语义） 已定项 1；回退候选集那一处没做，用户 2026-10-03 定进下一批（不在这一轮的被判范围） |
| C | 系统配置见证读改取 R2：判不出 ⟺ 有槽设备报读错 ∧ 没有一块盘两槽都自证过；可写挂载读阶段与取号同用一处定义；取号另留「某块盘一份自证过的都没有就拒」（用户 2026-10-03 定留着），拆成单独成员；改钉两条判拒用例的盘面换成 R2 下要拒的那一形 | `recovery.rs` 的 `system_configuration_slots_of_one_device_read_once`、`system_configuration_witness_is_undecidable`；`mount.rs` 的 `newer_publish_witness`、`WitnessedCounterComparison`；`transaction.rs` 的 `self_verified_system_configurations_of_every_device`、`InstanceAcquisitionFailed` | D22（单元原子性怎么合成） 已定项 8；D23（journal 的角色与格式） 已定项 14；C331（择根倒挂压过已确认的写）、C662；用户 2026-10-03 定「改取 R2」 |
| D | 池级 checker 跟上：由记录施加出来的那一版的实例表链与分配记录树根从新根段读；`records_as_the_recovery_reads_them` 与 `record_can_enter_a_replay_prefix` 按 (实例代号, 计数器) 建键；fsid 偏移 463 | `walk.rs`、`crates/singlefs-checker/src/lib.rs` | I-4.1（重放后树可遍历）、I-3.1（已分配统计对得上）、I-8.6（反向链算法）；I-8.10（记录新根段与同代根相同） 扩到七段是用户 2026-10-03 定、进下一批，不在这一轮 |
| E | 改钉：E142 反向链钉值 2 213 468 031 → 2 660 362 816；默认环整盘 CRC 1 733 593 715 → 4 048 568 891；记录头宽、fsid 偏移、点名项切分的单测钉值 | `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs`、`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`、`checker_known_bad_images.rs`、`checker_narrow_invariants_and_abandoned_roots.rs`、`journal.rs` 与 `transaction.rs` 单测 | 7-2 报告第八节、改钉报告 `/tmp/claude-1000/m3-r1/batch7-repin/patch/report.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 冻结副本取自 2026-10-03 主工作区：三份补丁都打上之后，删除批第 12 批也已合入（别的会话，动 mount.rs 的卸载、抬 F 预演与删除队列，transaction.rs 的删除路径）。被判 31 份文件的 sha256 在 `research/prompts/m3-root-ring-code-r1-snapshot/crates-sha256.txt`，冻结副本逐份对得上。
- 合入后验证第三趟：harness 档 129 个测试二进制全绿，fmt、clippy 过（`/tmp/claude-1000/merge-and-verify/2026-10-03-2028362/`）。checker 档一样没跑。
- 已知没做、不在这一轮被判范围的：回退候选集的窗口；清扫器与清扫条件两处收严（`crates/` 里没有清扫器，条款缺五句，另开一轮三方）；I-8.10 扩到七段；harness 判法 `mount_transparency.rs` 的 `newer_publish_witnessed_on_the_crash_image` 仍按取甲判；checker 档里硬写旧格式的几处（GPU 内核 `crash_judge_*.wgsl` 的 311 / 287 / 196、`crash_enumeration_record_spill_over_stream.rs` 的 `(1, 67, 1)`、e170 的 `BAD_RECORD_NAMED_ITEM_AREA_START = 311`，7-2 报告第九节第 5 条）。这几样腿可以拿来当线索，但不当成这一轮的打中。
- 7-2 自报没造出会红用例的五处：挂载时的窗口、抬 F 那一遍、丁-defer 门槛、影子账候选过滤、挂载时读不出的槽（7-2 报告第四节）。
- 主 agent 现跑的探针（7-2 报告第十二节第 1 条）：inject-6 种子 1 那一态 `AFTER VERDICT I-3.1: Holds`、`AFTER RED I-7.9`（txg 9 把 F 从 0 抬到 5，高于上限 1）；种子 0 那一态在新格式下历史只剩 131 段，旧坐标作废。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　新根段三字段与恢复施加（A）
D23（journal 的角色与格式） 已定项 15 与 m3-ring-r3 判决第二节第 1、2 行要的施加规则，代码是不是逐字照做：七项各自取哪里、三处装记录各写的是不是这次写的那条根上的；头宽、偏移、点名项容量与 kb 的 `format-const` 标记逐个对得上吗；卸载记号「位 0 之外有位为 1 当记录损坏」在读者、恢复、池级 checker 三处是不是同一个口径；首个文件、卸载、实例表指针三条用例钉的是不是条款说的那一形。

### K2　一圈窗口（B）
四处同改各自取的锚 N 是不是 D16（发布语义） 已定项 1 说的那一个；核心层与池级 checker 两份窗口实现在边界（txg = N − 3S、N < 3S、txg 贴着 u64::MAX）上是否逐格相同；挂载时读不出的槽「门槛取最小 txg、根环表记最大 txg、不算知道住着根的槽」有没有一段合法历史给出错结局；头尾两段的循环与逐 txg 走的原写法在所有输入上是否相同（`target_for_publish` 的周期是不是真的 3S）。重点攻 7-2 自报没造出会红用例的那五处：造一个会红的盘面，或说明为什么造不出。

### K3　系统配置 R2（C）
R2 的谓词与用户定案、D22（单元原子性怎么合成） 已定项 8 射程逐字相同吗：「设备报读错」只认 `PoolReader::read` 交空，撕裂（读得出、自证不过）、fsid 不符、全零各落哪一边；可写挂载读阶段、逐盘核、取号三处是否都按同一处定义判，有没有一处还留着取甲的写法；留着的「某块盘一份自证过的都没有就拒」与 R2 合起来，有没有一段合法历史让池永远只读而条款不要；改钉的两条判拒用例换盘面之后判的还是原来那两件事吗。

### K4　池级 checker 跟上（D）
由记录施加出来的那一版从新根段读实例表与分配记录树根之后，I-3.1、I-4.1、I-3.10 的读数在「写行之后第一个文件版本、崩在记录已持久根槽没持久」那一形上与实现是否一致；(实例代号, 计数器) 键在两块盘同计数器坐着不同实例时，checker 与恢复（`recovery.rs` 扫环）是否收下同一批记录；checker 有没有一处与 core 共用了窗口或键的代码（D13（验证路线） 已定项 5 要两边独立）。

### K5　改钉（E）
E142 反向链与默认环 CRC 两个钉值都是在补丁代码上现算的：有没有一条不经 core 的路径能独立算出同一个数（池级 checker 自己的 CRC、手算字段表）；钉的说明里「改之前」「怎么得的」与实情是否相符；格式常量改了之后，`.claude/rules/format-evolution.md`「改一个格式常量」第 1 步那几类派生形态（消息串、下标、倍数、商、测试与门禁里钉的产物值）在被判文件里还有没有漏改的；7-2 留着没改的 `write_request_split.rs` 与几份用例名里的 67，会不会让某条断言静默失去意义。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3、K4、K5 | 逐条从条款或定案原文推到冻结副本里的行，再推到钉它的测试与变异行 |
| 云端攻方（Opus） | K1、K2、K3、K4，重点 K2 与 K3 | 在冻结副本的拷贝上造反例，只跑 harness 档；K2 那五处给出会红的盘面或造不出的理由 |

## 五、跑前写死的判据

- 两腿都没打中 ⇒ 站住（一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记）。
- 打中 ⇒ 主 agent 现查坐实，改法进下一批；第二轮攻改法落地与这一轮站住的。
- 第二节列为已知没做的几样，腿报出来不算打中，判决里照原去向写。
- 腿提的新改法算零轮。

交付：腿的报告写 `research/prompts/m3-root-ring-code-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/m3-root-ring-code-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
