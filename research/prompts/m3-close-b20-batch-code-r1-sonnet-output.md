# m3-close-b20-batch-code-r1 正推腿报告（2026-11-10）

立场：正推。从已定条款与 `crates/` 今天的实现推「应该是什么」，再与第 20 批七份产品补丁逐格比。没写模型，没起脚本，没往任务服务交任务。
引 kb 条款写 kb 文件名加那份文件自己的行号；每句引文写进来之前在被引文件里 `grep -nF` 过一次，命中行号就是写的行号（核法：草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-sonnet/quotes-check.py`，最后一行 `MISSING 0`）。

## 〇、开工核对

1. 工作树里被判的四份源码与两份 kb 的 sha256，与开工快照 `research/prompts/m3-close-b20-batch-code-r1-snapshot/` 里 `crates-sha256.txt`、`kb-sha256.txt` 的记录逐份相同（量的）：
   - `crates/singlefs-core/src/mount.rs` 655bb6f5…，`recovery.rs` 6d006f68…，`mounted_read.rs` 02b07348…，`system_configuration_repair.rs` 87290b5b…
   - `.claude/kb/decisions/22-单元原子性怎么合成.md` 17944f04…，`23-journal的角色与格式.md` 04835490…，`.claude/kb/checks-owed.md` 898dde2c…
   所以下面引的 kb 行号是快照版的行号；快照版 D22 第 185 行已含 H3、H6 两句，D23 第 395、397、468 行已含 Q 缺口与实例表两句。
2. 背景材料的 diff 文件 `research/prompts/_m3-close-b20-batch-code-r1-diff.md` 第 541–545 行，第五份补丁（`m3-close-b20-read-stage-roots`）那一段是空的 ```diff 围栏。这一份是 Q 缺口与 `choose_root_keeping_every_self_verified_root_read` 的补丁，是本轮最大的一份。我改读补丁文件本身 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-read-stage-roots/patch/crates.patch`（515 行，`mount.rs`、`recovery.rs` 与一个测试文件），再对着工作树今天的代码核。下面所有「代码做的」都是工作树今天的行号，不是 diff 里的行号。
3. 背景材料「压着的条款」写「第 2678–2683 行」，主 agent 定「判退那一形用 R」那一句今天在 `records/2026-10-18-里程碑三收尾与近期欠账.md` 第 2682 行（下面引）。

## 一、问题 1 第一件：H1 收 `chosen_root`，不收 `effective_root`

代码做的：
- `crates/singlefs-core/src/mount.rs:1478` `first_txg_of_new_instance` 多收第四参 `chosen_root: &RootRecord`；`mount.rs:1496` `.max(chosen_root.checkpoint_txg);` 接在第二遍读环的 `highest_root_txg(…)`（`mount.rs:1489`）之后，结果再与 `records` 的最大 txg 取 max、加一。
- 调用点 `mount.rs:10023` `let first_txg = first_txg_of_new_instance(`，第四个实参是 `&root_chosen_by_the_read_stage`（`mount.rs:10027`）。这个变量在 `mount.rs:9922` `let root_chosen_by_the_read_stage = chosen_root;` 绑定，`chosen_root` 此刻是读阶段交回的（`mount.rs:9888` 起的 `ReadStage { chosen_root, … }` 解构），不是 `effective_root`。

条款原文：
- `.claude/kb/decisions/23-journal的角色与格式.md:373`：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」
- 同一行：「内存里的 txg 已不小于这次挂载见过的一切」

判：对得上。条款的下界是「全部根记录」与「全部自证通过的记录」两个 txg 的 max 加一；代码的下界是（第二遍读环的最大根 txg、读阶段择到的根 txg、读阶段全部记录的 txg）三者的 max 加一，是条款的超集一侧、不低于它。

为什么收 `chosen_root` 够、`effective_root` 不必（我自己读码核，不照实现员的话）：
- 读阶段择到的根是读阶段读到的根里 txg 最大的：`recovery.rs:2044` 文档「三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的」。所以读阶段读到、这一遍读环没读到的根，txg 都不超过 `chosen_root.checkpoint_txg`；收它一条就盖住读阶段见过的全部根。
- 没判退时 `effective_root` 是 `chosen_root` 加 `records` 里的记录施加出来的（`mount.rs:9365` 起 `replay_journal(cache, &chosen_root, …, &records, …)`），它的 txg 来自 `records`，已在记录那一半；判退时它是退到的那一版，txg 小于 `chosen_root`。两形都不需要另收。

什么现象会推翻：一个盘面上读阶段读到的某条自证过的根 txg 大于读阶段择到的根（即 `choose_root_slot` 没取 txg 最大者）；或没判退时 `effective_root.checkpoint_txg` 大于 `records` 的最大 txg 与 `chosen_root.checkpoint_txg`。前者要推翻 Verus 验的择根核，后者要 `replay_journal` 施加一条不在 `records` 里的记录，读码都没有这条路（推的，没造盘面）。变异行 `crates/mutations.tsv:7455`（「D23 已定项 14 第 3 条：新实例第一个 txg 收判退之后的根…」）原文在 `mount.rs` 里恰好命中一次（我拿 python 读 TSV 数过）。

## 二、问题 1 第二件：判退之后下界收读阶段择到的根

代码做的：
- `mount.rs:9922` 的绑定在 `fall_back_judged_on_the_three_readers(` 调用（`mount.rs:9923`）之前，判退之后 `chosen_root` 在 `mount.rs:9959` 被重新绑定成退到的那一条，绑定出来的 `root_chosen_by_the_read_stage` 不变；`mount.rs:10027` 传给 `first_txg_of_new_instance` 的是前者。
- `mount.rs:1496` 同一行 `.max(chosen_root.checkpoint_txg);` 在这一形里收的就是退掉的 R。

条款原文：只有 `23-journal的角色与格式.md:373` 那句「根环里全部根记录的 txg」。条款原文没有点名判退这一形（我在 D23 第 373、395、397、468 行里都找了，点名判退的只有 Q 缺口与实例表两句，不是下界）。

判：对得上，但是「由『全部根记录』推出」，不是条款逐字说的。R 是根环里的一条自证过的根记录，判退之后写行 txg 若不超过 R，就违背 373 行的下界；判决 `research/prompts/m3-close-b20-repairs-code-r1-main-verification.md` 第二节 H1 一行也把违反的条款认到「第 373 行」。

什么现象会推翻：判退一形里，写行 txg 不大于 R 的 txg 而池级 checker 不红 I-3.12。入库用例 `root_fallen_back_from_and_read_bad_after_the_read_stage_still_numbers_…`（变异行 7455 的必红测试）靶这一形，用例在补丁里，没在本机跑（要主 agent 跑，见「要主 agent 跑的命令」）。

## 三、问题 1 第三件：H3 只读挂载与恢复收本池 fsid

代码做的：
- `crates/singlefs-core/src/recovery.rs:1823` 新 `choose_system_configuration_of_the_pool_naming_devices_without_a_mountable_slot`，走 `choose_system_configuration_among_the_slots_of`（`recovery.rs:1835`），筛法是 `recovery.rs:1829` `SlotsOfWhichPool::OnlyThePool(*filesystem_identifier_of_the_pool)`，与可写挂载的 `choose_system_configuration_of_the_pool`（`recovery.rs:1807`，现在也是调它再 `.map`）同一个筛。
- `recovery.rs:8377` `recover_of_the_pool(reader, policy, filesystem_identifier_of_the_pool)`、`crates/singlefs-core/src/mounted_read.rs:665` `mount_read_only_of_the_pool(reader, filesystem_identifier_of_the_pool)`，都只把「系统配置怎么择」换掉，后面共用 `recovery.rs:8393` `recover_from_the_system_configuration_choice` 与 `mounted_read.rs:680` `mount_read_only_from_the_system_configuration_choice`。
- 新入口不走「筛完一槽没有就不筛再择一次」那一步（该步只在可写挂载的 `mount.rs` 里，因为那里有后面的参数核）。

条款原文（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`）：
- 「带本池 fsid 的只读挂载与恢复（`mounted_read::mount_read_only_of_the_pool`、`recovery::recover_of_the_pool`）与可写挂载同一个筛法，只从本池的槽里择，这两形都读到本池那一版；交进来的 fsid 盘上一槽都没有时报 `NoValidSystemConfiguration`，不读别的池」
- 旧入口同一行：「不带 fsid 的只读挂载与恢复（`mounted_read::mount_read_only`、`recovery::recover`）择到的是别的池那一份」

判：对得上。两个入口名、「同一个筛法」、不读别的池三样都与条款一致；盘上没有这个 fsid 的槽时，`recovery.rs:1835` 起的择法在 `(None, Some(device))` 一支报 `NoValidSystemConfiguration`（我读了那一支的返回），`recover_from_the_system_configuration_choice` 把失败原样装进报告、`mount_read_only_of_the_pool` 装进 `MountReadOnlyFailure::Recovery`。

读码核了两件「新入口之后还有没有走不筛的择法」：`mounted_read.rs` 里 `choose_system_configuration*` 只在 `mounted_read.rs:47`（import）与 `mounted_read.rs:654`（旧入口）出现；`recovery.rs` 里不筛的 `choose_system_configuration(` 出现在 `recovery.rs:1601`（`unit_area_start_of_the_pool`）、`:5112`、`:7962`、`:7981`、`:8000`，它们是不带起点的公开入口（`walk_to_file`、`rebuild_version` 等）和 `chosen_system_configuration_slot_bytes`，`recover`/`mount_read_only` 两条路径不调它们（grep 在 `mounted_read.rs` 与 `recovery.rs` 的 `recover*` 体内零命中）。

什么现象会推翻：一块盘上别池槽世代号更大，`recover_of_the_pool` 或 `mount_read_only_of_the_pool` 仍报 `NoValidRoot` 或 `SystemConfigurationsDisagree`；或交错 fsid 时它们读到了别的池。入库用例 `newest_generation_of_another_pool_on_every_device_reads_this_pool_read_only_given_the_pool_fsid`（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:2056`）与 `…_on_only_one_device_…`（同文件 `:1981`）靶这两形，没在本机跑。

## 四、问题 1 第四件：H6 离线出路断言第 0 代

代码做的（`crates/singlefs-core/src/system_configuration_repair.rs`）：
- `:33` `const JOURNAL_TAIL_OF_THE_GENESIS_VERSION: u64 = 0;`；`:274` 起判断言的实例代号与 txg 是不是 `MKFS_INSTANCE_GENERATION`、`GENESIS_CHECKPOINT_TXG`（转述；这两个常量定义在 `crates/singlefs-core/src/make_filesystem.rs` 的第 50、53 行，值是 0、0）。
- `:281` `(true, None) if asserted_version_is_the_genesis_version => {` 取 `JOURNAL_TAIL_OF_THE_GENESIS_VERSION`，条件是断言那一版的根在根环里读得出（`(true, None)`）；根读不出又没记录走 `(false, None)` 一支，拒 `AssertedVersionReadableNeitherInTheRootRingNorInTheJournalRing`。
- `:326` `let something_newer_than_the_genesis_is_readable_in_the_rings = !records.is_empty()` 或 `highest_root_txg(…) != Some(GENESIS_CHECKPOINT_TXG)`；`:333` 起，断言第 0 代且这个值为真就返回 `TailOfTheAssertedVersionUndecidedWithoutItsLastRecordUnsupportedInTheFirstVersion`。这一返回在 `:399` `let mut pool = PoolWriter::new(…)` 之前，是任何写之前。
- mkfs 写系统配置槽的 tail 是 `crates/singlefs-core/src/make_filesystem.rs:789` `journal_tail: 0,`，与 `:33` 同值，但两处各写一个 0，没共用常量。
- 别的版本：`:281` 之后的 `(true, None)` 一支照旧拒（`:286` 那个成员）。

条款原文（`22-单元原子性怎么合成.md:185`）：
- 「断言第 0 代（mkfs 种的根，实例 0、txg 0）时那一版没有记录，tail 取 mkfs 写的值，根环与 journal 环里读得到比第 0 代新的根或记录就一个字节不写（`birth_txg` 闸数不到读不出的单元）」
- 「别的版本根读得出而末条记录读不出时 tail 取什么条款没写，一个字节不写」

判：对得上。三样都在：第 0 代 tail 取 mkfs 值（0）、根环或 journal 环里读得到更新的东西就不写、别的版本照旧拒。两处细节：
1. 条款说「比第 0 代新的根或记录」，代码的 `!records.is_empty()` 拒的是「读得到任何记录」，比条款多拒「实例 0、txg 0 的记录」；mkfs 不写记录（条款同句「那一版没有记录」），所以这一多拒在合法盘面上不会触发。
2. 新返回点复用已有成员 `TailOfTheAssertedVersionUndecided…`（定义 `:154`，成员文档已改成两形）。成员名字讲的是「没有末条记录」，新用法是「环里读得到更新的东西」，名字对不上第二形；文档串补了两形。没新加成员，所以条款没定的成员为零。

什么现象会推翻：暖机根（txg ≥ 1）在根环里读得出，断言第 0 代却写了系统配置；或新格式化池上取号撕裂一轮，断言第 0 代却被拒。入库用例 `torn_acquisition_round_on_a_formatted_pool_is_repaired_offline_and_publishes_the_first_file_version`（`writable_mount_counts_….rs:2289`）靶后者，同文件还有四条断言第 0 代被拒的用例：`:2415`（单元已写，birth_txg 闸拒）、`:2526`（更新的根读得到、之后的单元与记录读回全零）、`:2536`（更新的记录读得到、之后的单元与根读回全零）、`:2548`（第一次挂载的零单元发布之后），都没在本机跑。

## 五、问题 1 第五件：Q 缺口，并读阶段全部根，按 (实例代号, checkpoint_txg) 去重

代码做的：
- `crates/singlefs-core/src/recovery.rs:2067` `choose_root_keeping_every_self_verified_root_read`：每个根环槽读一次（`read_root_ring_slot`），自证过的留、读不出和自证不过的变 `None`，择根核吃这一份摊开的入参；交回「择到的根」与 `recovery.rs:2108` `.flatten()` 之后的全部自证过的根（根环次序，同一条根住两槽出现两次）。`choose_root`（`recovery.rs:2048`）改成调它再取 `.chosen_root`，择法与读法没变。
- `mount.rs:9362` 读阶段拿它择根，全部根进 `ReadStage.self_verified_roots_read`（`mount.rs:9157` 起的结构），`mount.rs:9890` 解构成 `roots_read_by_the_read_stage`，`mount.rs:10044` 传进 `rebuilt_allocator`。
- `mount.rs:3541` `roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage`：拷重建分配器读根环那一遍的根，再逐条看读阶段的根，`mount.rs:3551` `if !roots.iter().any(|root| {` 里按 `(实例代号, checkpoint_txg)` 比，没有才 `mount.rs:3554` `roots.push(*root_read_by_the_read_stage);`。判重对象是不断变长的 `roots`，所以读阶段里同一条根住两槽也只并一次。
- 用法：`mount.rs:3140` 算出 `roots_for_the_newer_readable_roots`，只在 `mount.rs:3152` 交给 C 并的较新可读根那一个函数；影子账 `isolate_slots_referenced_only_by_abandoned_roots` 收的是 `&sorted.roots`（`mount.rs:3164`），回收门槛 `reclaim_the_released_slots_of_the_rebuilt_allocator` 与根环表 `root_ring_occupants_of_the_rebuild` 收的是 `&sorted`（`mount.rs:3129`、`mount.rs:3196`）。
- 并进来的根还要过较新可读根那个函数自己的筛：`mount.rs:2224`–`2225` `!is_abandoned(root) && (root.checkpoint_txg, root.instance) > (所选版本的 txg, 实例)`。

条款原文（`23-journal的角色与格式.md:395`）：
- 「这几条根从重建分配器读根环那一遍读出的根，并上读阶段择根那一遍读出、自证过的全部根里挑，按 (实例代号, checkpoint_txg) 去重」
- 「按读阶段择到的那条根的实例表有效、读得出的根自己的账里的落点」
- 「影子账、根环表与回收门槛不用这一份，照重建分配器那一遍根环槽里读到的算」

判：对得上，三句各对一处：读阶段那一遍的全部自证根、判重键、影子账与根环表与回收门槛不用这一份。条款没说重复时留哪一份（重建分配器那一遍的还是读阶段的），代码留前者；两份是同一 (实例代号, txg) 的根，除非同键两个不同内容的根（一个 txg 一个实例只发一条根），选哪一份不影响账（推的，没造盘面）。

什么现象会推翻：影子账、根环表或回收门槛的输入里出现读阶段并进来的根（读 `mount.rs:3164`、`3129`、`3196` 三处的实参就能推翻，我读了，都是 `sorted`）；或判退一形里 R 与退到的版本之间读阶段读得出的 Q 在重建那一遍读坏时，可写挂载没拒在 Q 那一份账上。后者的入库用例 `root_read_by_the_read_stage_between_the_version_fallen_back_to_and_the_root_fallen_back_from_whose_units_and_record_read_back_zero_is_refused_on_its_account`（`recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs`，变异行 `crates/mutations.tsv:7581`、`:7578`）靶它，没在本机跑。

## 六、问题 1 第六件：实例表照读阶段择到的根，判退那一形用 R

代码做的：
- `mount.rs:1638` `instance_table_of_the_newest_root_read_at_most_twice` 不再收系统配置，收 `root_chosen_by_the_read_stage: &RootRecord`；第一遍 `mount.rs:1643` `if let Some(table) = instance_table_of_root(reader, root_chosen_by_the_read_stage) {`，读不出就 `mount.rs:1646` 调「重读一次」之前的钩子，第二遍 `mount.rs:1647` `match instance_table_of_root(reader, root_chosen_by_the_read_stage) {`，两遍读的是同一条根。读阶段之后不再择根：`mount.rs` 非测试代码里没有 `choose_root(`（grep 命中的只有 `mount.rs:13858`，在 `mount.rs:13774` 起的 `#[cfg(test)]` 里）。
- 调用：`mount.rs:3099`–`3101` 在 `rebuilt_allocator` 里把 `root_chosen_by_the_read_stage` 交进去；调用点 `mount.rs:10043` 交的是 `&root_chosen_by_the_read_stage`，即 `mount.rs:9922` 判退之前绑定的那一条。得到的表 `newest_table` 被 `mount.rs:3103` 的生效回退下界、`mount.rs:3126` 的 `is_abandoned`（影子账、回收门槛、根环表、较新可读根）同用。
- 判退本身：`fall_back_judged_on_the_three_readers(`（`mount.rs:9923`）收的 `&chosen_root`（`mount.rs:9926`）是判退之前那条，`recovery.rs:8293` `let instance_table_of_the_chosen_root = instance_table_chain_of_root(reader, chosen_root)` 按它的表滤候选。所以判退与判抛弃读的是同一条根的表。
- 拒因：`mount.rs:416` 的字段 `root_chosen_by_the_read_stage: RollbackTarget` 不再是 `Option`，`mount.rs:1652` 构造时直接取 R 的 (实例代号, txg)。

条款原文：
- `23-journal的角色与格式.md:397`：「重建分配器判抛弃用读阶段择到的那条根的实例表，判退那一形（读阶段择到的根走读失败、退到更旧的一版）也用它，读阶段之后不另择根」
- 同一行：「这张表读不出，同样重读一次、仍读不出拒可写（代码审阅第 22 条）」
- `records/2026-10-18-里程碑三收尾与近期欠账.md:2665`：「用户定（弹窗）：Q 缺口「这一批补」；判抛弃用的实例表「照读阶段择到的根的表」」
- 同一份记录 `:2682`：「主 agent 定：用 R 的表」
- `23-journal的角色与格式.md:468`：「判退那一形也用读阶段择到的那条根的表，是主 agent 照那一问的原文判的」

判：对得上。注意判退那一形用 R 的表不是用户原话，是主 agent 定的（D23 第 468 行自己写明，记录第 2682 行同），条款把这一点标成了主 agent 的判读。

什么现象会推翻：读阶段择到的根 R 的槽在读阶段之后读坏、现择落到更旧的根时，抛弃判定仍按旧根的表（实现是不再现择，读码没有这条路）；入库用例 `abandonment_at_a_writable_mount_is_judged_by_the_instance_table_of_the_root_the_read_stage_chose_even_when_its_slot_reads_bad_afterwards`（`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`，变异行 `crates/mutations.tsv:7582`）靶它，没在本机跑。

## 七、问题 3：检查单八格

### 1. 走不到的理由二次查实：成立，附一处报告里的说法不实（轻）
- 七份报告逐份看了：`m3-close-b20-repairs-fixes/result.md:73`、`-fallback-i74/result.md:136`、`-read-stage-roots/result.md:165`、`-instance-table-read-stage/result.md:143`、`-shadow-ledger-refusal-field/result.md:133` 都有「走不到的理由」一节写无；`-repairs-fixes-2/result.md:64` 一节列了两处；`-repairs-fixes-3/result.md` 没有这一节，只在 `:128` 一行写「走不到的理由：无」。背景材料说「七份都写无」，其中 -2 列了两处、-3 没有这一节。
- 我自己的核：七份 `crates.patch` 里 `crates/singlefs-core/src/` 下文件的 `+` 行，搜 `todo!`、`unreachable!`、`assert!`、`assert_eq!`、`assert_ne!`、`.expect(`、`panic!`、`unwrap()`、`debug_assert`、`_ =>`，零命中（量的，草稿目录里的 python 一次过，输出七段全空）。`recovery.rs:2102` 那句 `.expect("择根核只交回自证过的槽…")` 是搬进新函数的原句，在补丁里是上下文行不是 `+` 行。没新加错误成员：全部拒用已有成员（`AccountOfAnUnappliedPublishWithIncompleteReadableRecords`、`InstanceTableOfTheNewestRootForTheShadowLedger`、`TailOfTheAssertedVersionUndecided…`）。
- 不实的一句：`m3-close-b20-repairs-fixes-2/result.md:69` 说新返回点（今天在 `system_configuration_repair.rs:336`–`340`）「崩溃态上闸先拦、走不到这里，只有单元两份都读不出的盘面到得了」。读码：到得了它的条件是断言第 0 代、环里读得到更新的根或记录、`birth_txg` 闸没拦；mkfs 之后第一次可写挂载的写行与暖机是零单元发布，盘上没有任何单元，闸就数不到，这一形不需要「单元读不出」。同文件用例 `writable_mount_counts_….rs:2548` 的注释（`:2544`–`:2546`）自己写了这一点。所以这一处返回是活路径，也有用例走它；代码行为对，说明里「单元都读不出」的限定（`system_configuration_repair.rs:153`、`:325` 两处注释）不全。

### 2. 类型与名字：成立
- 实例代号、txg 是 newtype：`crates/singlefs-core/src/address.rs:33` `CheckpointTxg(pub u64)`、`:37` `InstanceGeneration(pub u32)`；`RollbackTarget`（`mount.rs:907`）的两个字段是这两个类型。fsid 是裸 `[u8; 16]`：新入口的参数 `filesystem_identifier_of_the_pool: &[u8; 16]` 与既有的 `choose_system_configuration_of_the_pool`、`RootRecord.filesystem_identifier`（`root_record.rs:180`）、`SystemImmutableConfiguration`（`system_configuration.rs:300`）同型，全仓没有 fsid 的 newtype（`grep -rn 'struct FilesystemIdentifier' crates` 零命中），新代码没有引入新的裸量。tail 是裸 `u64`，与 `journal_tail: 0`（`make_filesystem.rs:789`）同型。
- `_ =>` 零（见第 1 格的 grep）；新名字没有缩写（读了补丁里全部新函数、新变量、新字段名）；新加的 `expect` 为零。
- 一处小的：`system_configuration_repair.rs:33` 与 `make_filesystem.rs:789` 各写一个 0，没共用常量。两处现在相等；mkfs 若改了 tail，离线出路不会跟着改，只有用例 `:2289` 一条守着（它走完整的「修复后可写挂载做成、发得出第一个文件版本」）。

### 3. 不兼容旧代码与旧格式：成立
- 补丁里没有认旧版本的分支、旧布局号与版本号常量、迁移路径、只为旧数据放宽的读法。H6 的第 0 代出路不是兼容旧格式：它支持的是今天的 `make_filesystem` 加今天的可写挂载真写流撕裂出来的盘（判决 `m3-close-b20-repairs-code-r1-main-verification.md` 第三节用户原话「如果是我们自己代码的问题，那么就要支持修复」）。
- 旧入口 `recover`（`recovery.rs:8365`）、`mount_read_only`（`mounted_read.rs:651`）签名没改。留着的理由读码：产品 crate 里没有别的调用方（`grep -rnE '\b(mount_read_only|recover)\(' crates` 去掉 `singlefs-verification` 与这两个源文件，零命中；`singlefs-mount-runtime` 等其余八个 crate 零命中）；`singlefs-verification/src` 里判官、实验、崩溃装置 63 处（去掉注释行），`tests/` 里 327 处（量的，grep 计数）。D22 第 185 行把不带 fsid 的两个入口当现存入口写进了条款（「不带 fsid 的只读挂载与恢复（`mounted_read::mount_read_only`、`recovery::recover`）择到的是别的池那一份」），所以留着不是只为兼容旧调用方，是条款描述的另一种用法。该不该删是主 agent 的判断；删的话要迁约 390 处调用，条款第 185 行那句要改。

### 4. 不新建判定库：成立
七份补丁改的文件清单（每份 `crates.patch` 的 `+++` 行）只有 `crates/singlefs-core/src/*.rs`、`crates/singlefs-verification/src|tests/*.rs`，加上各自 `patch/` 里追加或换行的 `mutations-append.tsv`、`mutations-replacements.tsv`，目标是已有的 `crates/mutations.tsv`；没有新的目录、TSV、sqlite、RocksDB。

### 5. 所有任务多机可用：成立
七份补丁 `+` 行里搜 `std::fs`、`std::env`、`env::var`、`hostname`、`/dev/`、`/home/`、`/tmp`、`Command::new`、`available_parallelism`、`thread::spawn`、`File::open`、`nvidia`，零命中（量的，同一个 python 脚本）。新用例的盘面是 `MemoryPool`、`SparseBlockDevice`（读补丁 `m3-close-b20-read-stage-roots/patch/crates.patch` 里新用例的造盘函数），不读本机状态。GPU 库单测卡住那一件不在这七份补丁的改动行里，没判。

### 6. 测试优先多线程：成立（有一处推的）
补丁里没有新加线程；新用例是单个盘面的确定场景，带循环的最大一条是 `FIRST_TXG_CELL` 那条，32 格各是一次内存盘上的可写挂载（`m3-close-b20-repairs-fixes/result.md:151` 写的 32 行）。该并行而没并行：没看到。32 格总共多久我没量，「不重」是推的。

### 7. 带 --item 的命令与项：不成立（轻，命令缺项），项本身没问题
- 缺项：三条命令的 `--item` 后面是占位 `<项>`，不是能直接交的整条：`m3-close-b20-read-stage-roots/result.md:160`（四个 `--item <项>`）、`m3-close-b20-instance-table-read-stage/result.md:132`（四个）、`m3-close-b20-fallback-i74/result.md:120`（`--item <项> [--item <项>…]`）。项名在上一行的正文里写了前半截加「…」，要靠先交 `--list-items` 再手填。别的四份（`repairs-fixes/result.md:177`、`-2/result.md:147`、`-3/result.md:105`）命令是整条。
- 项本身：这三份点的八行我在 `crates/mutations.tsv` 里都找到了，行号 `:7455`、`:7458`、`:7578`、`:7581`（read-stage-roots 与 fallback-i74 的）和 `:1017`、`:1018`、`:7526`、`:7582`（instance-table 的）；每行第三列「原文」在 `mount.rs` 里恰好命中一次（python 数的，八行都是 1），第六列点名的测试函数在 `crates/singlefs-verification/tests/` 里都找得到（grep 每个函数名，八个都命中）。「新项列不列得出」要跑 `--list-items` 才知道，待主 agent 跑（第一条命令）。
- 整条八项命令我替主 agent 拼好了，见「要主 agent 跑的命令」第 2 条。
- 形式化验证的 `--item`：实现员写的「不在形式化项里」是推的；我量了一步：`grep -c 'verus!'` 在 `mount.rs`、`recovery.rs`、`mounted_read.rs`、`system_configuration_repair.rs` 四份都是 0，所以这四份源码里没有 Verus 块，这一句从推的变成量的（指到文件级；Verus 验的择根核在 `singlefs-replay-prefix`，本轮没动它）。
- 崩溃点重放的 `--item random-histories-and-fault-injection-given-states` 已登记（`.claude/gate.d/crash-point-replay-items.tsv:64`）。

### 8. 新崩溃流多罩了什么：成立
七份补丁只改 `crates/`，没碰 `.claude/gate.d/crash-point-replay-items.tsv` 与 `stage-inputs.tsv`；补丁里新加的 `#[test]` 函数名（六十来条，含 `m3-close-b20-*` 其余补丁的），在 `.claude/gate.d/` 与 `scripts/crash/` 里 `grep -rlF` 逐个找，一条都没登记成崩溃枚举用例。读次数常量改了（实现员报告 `m3-close-b20-instance-table-read-stage/result.md` 第 3 节），按读次数摆注入点的 `random-histories-and-fault-injection-given-states` 结局变不变，读码推不出，要跑（第 3 条命令）。

### 八格各自什么现象会推翻
1. 走不到的理由：补丁 `+` 行里出现 `todo!`、`unreachable!`、`assert!`、`.expect(` 或新错误成员，或 `system_configuration_repair.rs:336` 那处返回被证明任何盘面都到不了（用例 `:2548` 走到了它，所以后者已被推翻）。
2. 类型与名字：新参数或字段里出现裸整数装实例代号或 txg；新代码里有 `_ =>`。
3. 不兼容旧代码：补丁里出现认旧布局号或旧版本号的分支。
4. 不新建判定库：补丁清单里出现新的目录、TSV、数据库文件。
5. 多机可用：新用例读本机状态（`std::env`、`/dev/`、主机名）。
6. 多线程：新增一条跑数分钟的单线程用例（我没量时间）。
7. `--item`：`--list-items` 的输出里列不出第 7 格点的八行里任何一行。
8. 新崩溃流：补丁里新加的 `#[test]` 名出现在 `crash-point-replay-items.tsv` 或 `stage-inputs.tsv` 里。

## 八、顺带看到的（不在六件与八格的判定里）

1. 条款没说、代码取了读阶段那一遍：`first_txg_of_new_instance` 的记录一半用读阶段的 `records`（`mount.rs:10023` 起第三个实参 `&records`），判退之后重扫的那一份（`mount.rs:9965` 起 `records_rescanned_after_the_trigger`，用在 `:9973`）只给 F1 再判用。`23-journal的角色与格式.md:373` 只写「环里全部自证通过的记录」，没写取哪一遍。读阶段扫环时一条高 txg 的记录两份副本都瞬时读错、它的根又没落，下界收不到它。这是 H1 根那一半的同型（记录那一半没人补第二遍），条款射程里算不算双故障我判不了：推不出，没造盘面，交攻方腿的角度。
2. 不筛 fsid 的择系统配置在挂载之后的入口里还有两处：`mount.rs:7711`（`caller_inputs_agreeing_with_the_disk`，抬 F、准入抬 F、正常卸载、管理员回退都先调它）和 `mount.rs:968`（`ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk`，产品代码里只有测试与验证包调它，`grep -rn 'of_a_pool_on_disk'` 在 `crates/singlefs-core` 外只有 `singlefs-verification`）。D22 第 185 行写「只有一块盘带这样一槽……可写挂载照常做成」，之后的入口若还看得到那一块盘上的别池槽，会报 `SystemConfigurationsDisagree`。读条款推：取号写两轮把每块盘的两个槽都写成本池的（`22-单元原子性怎么合成.md:185`：「取号写两轮（用户 2026-10-19 定，原话在变更史；去留见 C974（取号写两轮的去留没定））：每轮逐盘一写、轮后一道池屏障，每块盘两槽都带取号时的新号」），所以做成之后盘上没有别池槽留下。这是读条款推的，没造盘面，没找到覆盖「别池单槽盘 + 可写挂载做成 + 之后调一个入口」的用例；交攻方腿的角度。
3. 新返回点的说明串不全，见第七节第 1 格；成员名字不贴第二形，见第四件。
4. `mounted_read.rs:647` 与 `recovery.rs:8363` 两个旧入口的文档串新加了一句「系统配置不筛 fsid；……用 [`…_of_the_pool`]」，与条款第 185 行一致。

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 H1 收 `chosen_root` | 一致 | `mount.rs:1496` 收读阶段择到的根；条款 `23-…:373` 的下界是全部根与全部记录的 max 加一，代码不低于它；`effective_root` 不必收（读码推，未造盘面） |
| 2 判退之后下界收读阶段择到的根 | 一致（由「全部根记录」推出） | 绑定在判退之前（`mount.rs:9922`）；条款原文没点名判退，推的 |
| 3 H3 只读与恢复收本池 fsid | 一致 | 两个新入口走 `OnlyThePool` 筛法，不走不筛再择；条款 `22-…:185` 逐句对上 |
| 4 H6 离线断言第 0 代 | 一致 | tail 取 0 与 mkfs 同值，环里读得到更新的根或记录就不写；成员名字不贴第二形、说明串「单元都读不出」不全（轻） |
| 5 Q 缺口 | 一致 | 并读阶段全部自证根，按 (实例代号, txg) 去重，只给较新可读根用；影子账、根环表、回收门槛仍用 `sorted` |
| 6 实例表与判退 | 一致 | 实例表读 `root_chosen_by_the_read_stage`，非测试代码不再现择；判退与判抛弃读同一条根的表；判退用 R 是主 agent 定，不是用户原话 |
| 检查单 1 走不到的理由 | 成立（附一处说明不实，轻） | 补丁零新 `todo!`/`unreachable!`/`assert!`/`expect`/新成员；-2 报告 `:69` 说只有单元读不出才到得了新返回点，不实 |
| 检查单 2 类型与名字 | 成立 | 实例代号、txg 是 newtype；fsid 全仓裸 `[u8; 16]`；`_ =>` 零；tail 的 0 与 mkfs 各写一份 |
| 检查单 3 不兼容旧代码 | 成立 | 没有旧版本分支；旧入口产品 crate 无别的调用方，验证包 63 处、测试 327 处，条款写着它们 |
| 检查单 4 不新建判定库 | 成立 | 只追加已有的 `crates/mutations.tsv` |
| 检查单 5 多机可用 | 成立 | `+` 行零本机状态；用例用内存盘 |
| 检查单 6 测试优先多线程 | 成立（时间是推的） | 无新线程，最大一条 32 格循环，没量时间 |
| 检查单 7 带 --item | 不成立（轻） | 三份报告命令里是 `<项>` 占位（见第七节）；八行变异行与点名测试都找得到，`--list-items` 待主 agent 跑 |
| 检查单 8 新崩溃流 | 成立 | 没有新登记的崩溃枚举用例；读次数常量改动的影响要跑才知道 |

## 十、要主 agent 跑的命令

没写模型，所以没有 `rerun.sh`。下面每一条都是「待主 agent 跑」；跑完看 `TASK_SERVER_RESULT` 末行 `exit=`。

1. 变异项能不能列出（检查单 7）：
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/crates-mutation-replay.sh --list-items /home/fy5090/code/singlefs`
   看什么：输出里有 `crates/mutations.tsv` 第 1017、1018、7455、7458、7526、7578、7581、7582 行的名字（第一列）各一行。
2. 八项变异一次判（我拼好的整条，名字逐字取自 TSV 第一列，存在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b20-batch-code-r1-sonnet/items-command.txt`）：
   ```
   SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 带 --item; 范围=点名的 8 项; 为什么=Q 缺口、实例表、首 txg、水位四处的变异行，三份实现员报告的 --item 命令里是 <项> 占位' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/crates-mutation-replay.sh --item 'D23 已定项 14 第 3 条：新实例第一个 txg 收判退之后的根（读阶段读到 X、退到 Y、取第一个 txg 那一遍 X 读坏时写行 txg 不大于 X）' --item 'D8 已定项 8 ②（号永不重发）：写行那次发布的树 ID 水位收判退之后的根（读阶段读到 X、退到 Y、取水位那一遍 X 读坏时写行水位落回 Y 带的）' --item 'C_NEWER_ROOTS 不并读阶段择到的根（判退退掉的 R 读根环那一遍读坏、账与记录读不出时可写挂载照做成，写行那次发布写掉 R 的单元）' --item 'C_NEWER_ROOTS 只并读阶段择到的根、不并读阶段读到的全部根（P 与 R 之间读阶段读得出的 Q 之后几遍都读坏、账与记录读不出时可写挂载不拒在 Q 那一份账上，写行那次发布写掉 Q 的单元）' --item '代码审阅第 22 条：重建分配器时最新那条根的实例表重读仍读不出，照改之前按空表往下走（无声关掉隔离）' --item '代码审阅第 22 条：实例表重读之前不调「重读一次」之前的钩子' --item '可写挂载别的读：重建分配器读最新那条根的实例表不进账' --item '判抛弃用的实例表取读阶段之后现择的根、不取读阶段择到的根（读阶段之后实例 3 的两条根槽各读坏一次时现择落到被抛弃的 C，按 C 自己的表 C 不算被抛弃，它独占的槽不隔离）' /home/fy5090/code/singlefs
   ```
   看什么：`exit=0`，八行都判「抓到」。
3. 本报告引的三个测试目标今天在工作树上绿不绿（背景材料说合入后验证 12052 全绿，那一趟的读数我没看到）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed; 范围=1 个测试目标; 为什么=核正推腿引的判退下界与 Q 缺口用例' task-server/task submit -- cargo test -p singlefs-verification --test recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed`
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable; 范围=1 个测试目标; 为什么=核正推腿引的实例表照读阶段的根用例' task-server/task submit -- cargo test -p singlefs-verification --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable`
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors; 范围=1 个测试目标; 为什么=核正推腿引的 H3、H6 用例' task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors`
   看什么：每条 `test result: ok` 且 `failed` 为 0。
4. 读次数改动对崩溃枚举的影响（检查单 8）：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃点重放入口带 --item; 范围=点名的 1 项; 为什么=读次数常量改动，按读次数摆注入点的用例结局变不变' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/crash/crash-point-replay.sh --item random-histories-and-fault-injection-given-states`
   看什么：`exit=0`，log 里该项计数行没有 `✗`。

## 十一、没做什么

- 不判攻方腿那几面（读阶段全部根在影子账与回收门槛里的待遇、实例表在读阶段之后被写盖的取法、测试里 `None` 改真值后断言还判不判原来的事、H6 出路在暖机根与跨实例盘面上的边界、H3 旧入口的产品调用方）：第八节只列了我读码时碰到的指针，没有造盘面验证。
- 没替主 agent 采纳或出判决；上面每个「一致」「成立」都是读码和 grep 的读数，没有任何一个测试或变异我自己跑过，没往统一任务服务交任务，判决要引的数没复跑，命令在第十节。
- 没写模型，没起脚本；落盘的只有草稿目录里的 `quotes-check.py`（核引文的）与 `items-command.txt`（八项命令的文本）；其余数命中次数、读 TSV 的 python 与 grep 都是命令行一次性的，没存成文件，要复核照报告里写的条件重做。
- 没读禁读清单里的 `research/prompts/m3-close-b20-batch-code-r1-opus-output.md`。
