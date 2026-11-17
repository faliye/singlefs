# m4-close-fixes-code-r2 辩方腿：复核 r1 判决第二节各格与第四节五条修补（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

辩方腿，立场：替现行实现与 r1 判决第三节的用户定案找站得住的理由；找不到的照实写，不替它圆。材料：`research/prompts/m4-close-fixes-code-r1-main-verification.md`、正文 `research/prompts/_m4-close-fixes-code-r2-body.md`、改动全文 `research/prompts/_m4-close-fixes-code-r2-diff.md`。被引代码都在主工作区现读（行号以工作区现行文件为准，改动全文里的行号不用）。禁读清单：无。

## 零、先说结论

- 判决第四节五条，**点名的那几处都补上了**：四处读各接上计数，三类豁免有了主动修复路径，查不到 key 不再改记没坏，T3 改名干净（旧名在 `crates`、`scripts` 里零命中），GPU 打开下界补了流与变异行。
- 但判决第二节第一格（「别的读」）补的是**判决字面点的四处**，同一形的兄弟读还有四处没接（`mount.rs` 的 :4801、:4256、:10728、:10847，会话里 :4256 有两个调用点）（第二节表第 1 行，行号逐条现查）。用户定案是「按字面全接上」，字面没接完。这一格对现行实现仍打中。
- GPU 下界判法同形第二处（`crash_judge_recovery.wgsl` 建上一版那一样）没有变异行，同样只补了字面点的 `:286`。
- 新测试文件里有两处「判别力」引的变异行名，在 `crates/mutations.tsv` 里查无此行（第二节表）。
- 第三节第 1 条（映射 key 被换）：条款够不着，不算这批的缺陷；第 5 条（放回字节再判）：能替代结构类不变量那一半，替代不了「弄坏的盘 × 部分落盘的状态」那一半。
- 第四节八格：成立 6、推不出 2（第 7 格带 --item 的项、第 8 格新流多罩；推不出的算打中，见第四节表）。

判定列口径：**站得住** = r1 判决那一条打中现在仍成立；**够不着** = 那一条打中现在够不着现行实现（已补，或条款没写到）；**同样打中别的臂** = 那一条换成别的方案也同样打中，不能拿来区分方案。

## 一、读过的、没读的

读：agent-common.md、three-way-inference.md 全文；evidence-discipline 的小节标题（「别的项目怎么做，是线索不是证据」一节与本轮无关，没有拿别的项目的做法当证据）；D4 已定项 4（`.claude/kb/decisions/04-校验和位置.md` 第 73 行）；D19 自举豁免一条（`.claude/kb/decisions/19-块指针的结构与宽度预算.md` 第 190 行）；D21 已定项 9、10、11；I-1.11 的登记行；实现员两份报告（`~/.local/share/singlefs/work/m4-close-fixes-writer/result.md`、`result-checker-on-restored-bytes.md`）；两个新测试文件全文。

没读：全量域 GPU 内核其余部分、`crates/singlefs-checker/src/walk.rs` 的实现细节（只读了不变量登记行）。

## 二、判决第二节各格复核（r1 判决第二节表，逐格）

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| D4 已定项 4「可写挂载期间别的读」：用户读文件、推抬 F 读实例表、释放前读盘核、搬留存读源 | **四处点名的：够不着（已补）。同形兄弟读：站得住（仍打中，四处读未接）** | 已补：用户读 `MountedSession::read_the_file`（`crates/singlefs-core/src/mounted_session.rs` 新入口，读者套 `PoolReaderCountingUnitReadsWithOneCopyFailing`）；推抬 F 读实例表 `mount.rs`「let instance_table_chain_read = instance_table_chain_of_root(」（`crates/singlefs-core/src/mount.rs:5321`）；释放前读盘核 `transaction.rs:13365` 起；搬留存读源 `relocations_checked_against_the_version`（`crates/singlefs-core/src/transaction.rs:10886`）。**没接的同形读（现读，都用不计数的读者）**：① 同一串推抬 F 里先于 5321 的那一读「let current_instance_table = instance_table_chain_of_root(devices, &current.root)」（`crates/singlefs-core/src/mount.rs:4801`），由「let ceilings = admission_ceilings_on_caller_inputs_agreeing_with_the_disk(」（`crates/singlefs-core/src/mount.rs:4994`）每推一串都走；上限不高于 F 时不进 5321，这一读读到的只坏一份无人计；② 回退门读根环：「let table = instance_table_chain_of_root(reader, current_root)?.records;」（`crates/singlefs-core/src/mount.rs:4256`）连带读每条有效根的树表，会话里两处调用「let publishes_read_from_the_ring = rollback_states_of_the_ring(」（`crates/singlefs-core/src/mounted_session.rs:3218`）与「match rollback_states_of_the_ring(」（`crates/singlefs-core/src/mounted_session.rs:4098`），读者是 `PoolReaderOfOneReadPass::reading_for_the_first_time(devices)`；③ 管理员回退：「let current_table = instance_table_chain_of_root(&*devices, &current.root)」（`crates/singlefs-core/src/mount.rs:10728`）与「unit, &locations, &*devices,」（`crates/singlefs-core/src/mount.rs:10847`，用户可见单元释放前读盘核），会话包装入口「pub fn roll_back_by_a_forward_publish_in_the_session」（`crates/singlefs-core/src/mount.rs:10988`）。这几处带着分配器，账接得上，只是没接。 |
| D4 第 73 行三类豁免：没有主动修复、三类旧落点不走释放前读盘核 | **够不着（点名的两半都补了）；窄处站得住：只核「读到过只坏一份、记着位置」的旧落点** | 主动修复：映射树叶经 `plan_rewriting_the_paths_to_the_moved_entries` 整条路径重写（`transaction.rs` 的 `repair_of_the_exempt_units_at_these_locations`，`crates/singlefs-core/src/transaction.rs:11928`）、实例表整条链重写、树表与映射树根随那次发布本来就换；旧落点隔离：`copies_of_the_exempt_units_read_with_one_copy_failing`（`crates/singlefs-core/src/transaction.rs:12029`），在 `prepare_the_version_publish_relocating_units` 里接在读盘核后面（「copies_failing.extend(copies_of_the_exempt_units_read_with_one_copy_failing(」，`crates/singlefs-core/src/transaction.rs:13375`）。端到端用例三条（映射树叶、实例表片、树表）在 `crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs`。窄处：函数文档自己写「没有记着的不读：豁免映射的单元不在硬规则 1 的射程里」（`crates/singlefs-core/src/transaction.rs:12028`），与 D19 自举豁免那条「硬规则 1 对这三类不适用」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:190`）一致；但 D4 第 73 行字面是「旧落点走释放之前读盘核那一路」，没限「记着位置的」。这不是实现错，是 kb 两条字面没对齐，写回时要把 D4 那半句限成「读到过只坏一份的」。实现员设计问题 ① 说的就是它。 |
| `transaction.rs:11909` 一判（映射里查不到 key 时把真的一份改记没坏），`recovery.rs:417`、`mounted_read.rs:1123` 同形 | **够不着（已补，同形没有剩余）** | 现行：「查不出落点就判不出提示过期：这一份是真读到的坏副本，照计」（`crates/singlefs-core/src/transaction.rs:12251`）；`recovery.rs` 只有查到而落点不是这一组才不转：「Ok(Some(mapped_locations)) if mapped_locations != *locations => {」（`crates/singlefs-core/src/recovery.rs:415`）、「Err(_) => self」（`crates/singlefs-core/src/recovery.rs:418`）；`mounted_read.rs:1123` 读的是这一层的标记，不用改。同形扫描：`transaction.rs` 里别的 `MappingLookup::NodeMalformedOrNoEntryWithThisKey` 分支（4713、6668 报 `ReleaseNotInMapping`，11193、11882、12147 是认 key 与回落留存条目，12267 报错）没有第二处把「查不出」当「没坏」。单测 `a_note_is_dropped_only_when_the_mapping_names_another_position` 四个分支，端到端两条（部分写读回、坏镜像可写挂载）。取舍一条：`Err(_)`（查映射本身判红）也计，会把一条过期提示且映射读不出的读算成真坏——条款字面「读到只坏一份时计数」站这一边，量不出哪边误差小，不算缺陷。 |
| T3 改名与文档 | **够不着（主名已改）；轻微站得住：只供测试开关名没跟着改** | 旧名 `RelocationOfTheOldDataUnitsKeptForTheSnapshots`、`relocation_of_the_old_data_units_kept_for_the_snapshots` 在 `crates`、`scripts` 里零命中（现 grep）。新名 `RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing` 的文档把三处读到的来源都写了。没改的：「pub enum RelocationKeptForTheSnapshotsRefusedForSpace {」（`crates/singlefs-core/src/compaction_plan.rs:527`）及另两个开关，现在也管挂载记下与别的读记下的那一批，名字仍只说快照。实现员报告说「牵 20 多行变异」所以没改，是已知的未了事。 |
| T4、只试一次 | 不支撑（r1 各抽一次，没打中） | 没有新证据可加；账什么时候清的实测在第三节第 3 条。 |
| 两种计数口径（理由句打架） | **站得住（轻）：改了注释，但口径变成四个数，注释数着三个** | 注释写「三个数分开报」（`crates/singlefs-core/src/mounted_session.rs:278`）列的是写入口、挂载、`read_at` 累计；新加的别的读那个数（`MountedSession::other_unit_reads_with_one_copy_failing_in_this_mount`，去重范围是「一组位置条目等着下一次改动的这一程」）是第四个，在它自己的文档里分开说，这一条没数进去。同一次部分写读回有活快照钉着的单元：写入口那个数计 1，搬留存读源又进别的读那个数 1（`an_instance_table_page_read_…` 断言的 2 里就有这一次），一次物理坏副本在两个数里各算一次——是两次真读，不算重复计同一读，但调用方若把几个数加起来会多算。 |
| C946（只坏一份的读没有计数也没有搬迁修复）kb 过期 | **站得住（还没写回）** | 工作区现状：C946 行仍写「读到一份坏之后没有代码把它修掉」（`.claude/kb/checks-owed.md:589`），D4 第 73 行仍写「豁免映射的三类（映射树节点、树表、实例表片）认不出映射 key，不进计划」（`.claude/kb/decisions/04-校验和位置.md:73`）与「推的，多片实例表链里不被改写的那几片没核」（同一行）。实现已经把这三类放进了另发的那一批、整条实例表链重写，这两句现在与代码相反。r1 判决第五节写明「kb 写回随那一批收尾」，所以只算未了，不算新缺陷。 |
| 检查单：走不到的理由 | **够不着（已有这一节）** | 实现员报告「走不到的理由」一节有；第四节第 1 格二次查实见下。 |
| 检查单：类型与名字 | **够不着（已换）** | 三个裸 u64 换成 `InodeNumber`、`SeparatorKeyOfAnInodeLeaf`、`ContainerNumberOfAnInodeLeaf`，生产调用点两处都换（`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/recovery.rs`），单测调用点同改；`InodeNumber` 本来就在 `address.rs`，容器号与分隔 key 在仓里没有现成的新类型可复用（现 grep `pub struct ContainerNumber`、`SeparatorKey` 零命中）。 |
| 检查单：带 `--item` | **站得住（推不出，见第四节第 7 格）** | 见第四节。 |
| 检查单：新崩溃流（GPU 只读挂载打开下界无流无行） | **打开那一样：够不着（流与行都有）；同形第二处：站得住（仍无行）** | 流：`the_gpu_judge_agrees_with_the_cpu_that_a_selected_version_whose_opening_alone_reaches_a_separator_key_above_the_first_record_of_its_leaf_is_not_fallen_back_from`（`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:21984`），行 `crates/mutations.tsv` 第 7126 行。同一句下界判在 wgsl 里出现两次：「if (lt64(inode, separator)」（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:286`，打开）与同一句（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:663`，建上一版）。`mutations.tsv` 里锚点含 `lt64(inode, separator)` 的只有 7126 一行（现 awk），它的锚点带前一行「let inode = rd64(leaf.base, record_offset);」只罩 286 那一处；r3 的 6875 行改的是下一句 `inode_records_of_the_leaf_ascend_after_the_leaves_before`，不是下界。663 那一处没有任何行罩着。用的就是 r1 判决那个「补流补行只补了字面点的一处」的形状。 |

## 三、正文第三节六条攻击面

### 第 1 条：映射 key 被换、封得对的盘，挂载收下，第一次发布盖回

**判：不算这批的缺陷；条款够不着。**

- D21（权威态与派生态的分界）把映射树归派生态（`.claude/kb/decisions/21-权威态与派生态的分界.md@已定项 9：权威态 = 单元 + 记账 + 根，索引是派生态`：「索引（树）」）；已定项 10 的判据是「重建不出时读者拿到保守答案还是错答案」，已定项 11 的前提是「派生态的损坏必须是可检出的」。这张盘上的映射被换，**检得出**：池级 checker 有一条专门的不变量，I-1.11「映射 key 与单元头相符」，登记行写「不等判损坏」（`crates/singlefs-checker/invariants/01-块自描述.md:17`，判在 `walk_central_mapping_entries`），已有坏镜像用例。所以 D21 已定项 11 要的可检出性有，条款没写「挂载必须拒」。
- 挂载侧的核对是条目数而不是内容：`mapping_entry_count_is_one_per_mapped_unit`（`crates/singlefs-core/src/recovery.rs:6836`）只数。这是既有行为，不是这批引进的；换 key 而不改条目数的任何映射损坏（包括两条 key 互换）挂载都收，同样打中**所有**不核映射内容的方案。
- 只读路径走不走得到错读：走不到。数据单元先按提示读，提示读不出才回落映射；回落查不到是报错（`UnreadableAtTheHintsAndMissingFromTheMapping`，`crates/singlefs-core/src/transaction.rs:12267` 起那一支），回落查到别的单元的位置（两条 key 互换那一形）则被读回后的头自述核对拒（同一函数里：「对象不是这个 inode」（`crates/singlefs-core/src/transaction.rs:12283`）、出生身份与指针不符等）；挂载态读同形（`FileReadFailure::DataUnitDisagreesWithTheLookupPath`，`crates/singlefs-core/src/mounted_read.rs:1065`）。靠山是 `.claude/kb/decisions/21-权威态与派生态的分界.md@已定项 1：反向索引在权威态，主体就在单元里`。
- 这批新代码在这一形上的行为：用例 `a_disk_with_a_key_replaced_in_the_mapping_is_mounted_and_the_mount_puts_the_key_back`（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:1245`）里挂载那一串头一次发布按数据指针重算映射条目，key 回来、认得出要搬的单元（`units_at_these_locations` 按位置条目认，不按 key，`crates/singlefs-core/src/transaction.rs:11858`）。
- **什么现象会推翻**：找到一条条款写「挂载读到映射内容与单元头不符必须拒」（本轮在 D19、D21、D4、I-1.11 登记行里没找到）；或找到一条读路径，在 key 被换时交出别的单元的字节而没被头自述核对拦下（读回路径与挂载态读路径我读到的都拦）。
- 连带：这张盘上池级 checker 会判 I-1.11 红，而第一次发布盖回之后红消失。若要把「挂载收下一张 checker 判损坏的盘」算成缺陷，它是 mount 与 checker 判据范围不一致的老题，不是这批的；把它另立一题，别压在这批上。
- 推不出的一半：用例标题称「挂载收下」，但没有用例证实盖回之后池级 checker 绿（用例结尾没调 `assert_the_pool_checker_is_green_…`）。

### 第 2 条：实现员七条设计问题逐条判

| # | 判 | 理由 |
|---|---|---|
| ① 没被读到的坏副本被换下时不核 | **够不着** | D4 第 73 行的前提是「读到只坏一份时」；没人读到的坏副本不在射程内。核了就是每次发布多读映射树根与树表两份（实现员报告的账），且 D19 自举豁免一条明说硬规则 1 对这三类不适用。要的是 kb 写回时把 D4 那半句限成「读到过只坏一份的」，见第二节 D4 第 73 行一格。 |
| ② 攒批会话读不到待发写 | **够不着** | 条款只管读到只坏一份要不要计数与搬，没写读自己待发写。`MountedSessionWithPendingChanges::session()` 交 `&MountedSession`（`crates/singlefs-core/src/mounted_session.rs:6121`），读已发布那一版。不是缺陷，是读语义没定的新问题；建议另立。 |
| ③ 账的口径第四种 | **站得住（轻）** | 见第二节「两种计数口径」一格：口径本身有理由（实例表链每次推抬 F、discard、预演都会反复读同一份），但注释没数它。需要统一口径要用户定，实现已经把三类去重范围各自写清。 |
| ④ 释放前核读到的单元就是这次换下的、读盘核之后才被拒那一形无用例 | **站得住（缺用例，代码路径在）** | 账里的位置条目只在 `relocate_…_after_the_change` 里被 `forget`（`crates/singlefs-core/src/mounted_session.rs` 的那一个函数），发布在读盘核之后被拒时没经过它，账留到下一次改动，下一次发成后按那一版认；代码上成立，但没有用例证。属于补用例。 |
| ⑤ `prepare_the_version_publish_relocating_units` 里 deadlist 条目核不过提前返回时，已读到的只坏一份没进账 | **站得住（轻，丢的只是计数滞后）** | 账在函数末尾才并进（`crates/singlefs-core/src/transaction.rs:13467` 起的「allocator.count_the_other_unit_reads_with_one_copy_failing(」），中间的 `?` 在读盘核之后（释放判定的各 `map_err(not_releasable)?`）会跳过它。丢的是这一次的计数；单元不在账里就不会被搬，但这条发布本来就被拒了，下一次读到时再进账。 |
| ⑥ `Relocated.mapping_keys` 含本来就重写的节点 | **够不着** | 文档写明了；字段语义是「计划里的」，不是「真搬了的」。改字段要动 API，值不值得是用户的事。 |
| ⑦ 实例表读不出整批拒 | **站得住（取舍，可议）** | 真正触发的条件比实现员报告写的宽：认不出的几组里**包含这次改动已经换下的旧落点**（释放前读盘核那一路最常见），它们进 `locations_left`，于是沿链读实例表（`crates/singlefs-core/src/transaction.rs:11966`「let chain = instance_table_chain_of_root(reader, &version.root)?;」）；链读不出时连带本来能搬的数据单元一起按 `RefusedNotForSpace{InstanceTableOfTheVersionUnreadable}` 交回（`mounted_session.rs` 里 `repair_of_the_exempt_units_at_these_locations` 的 `Err` 分支）。实例表两份都读不出时池早已出了更大的事，所以量级不大；但改成「实例表读不出就只做搬迁、跳过豁免修复」更贴「只试一次、结局随那次改动报出」，且不丢数据单元那一搬。 |

### 第 3 条：主动修复只试一次、被空间拒推抬 F 再试，账什么时候清、崩溃之后会不会搬两次

**判：站得住（现行实现对得上条款），没找到搬两次的路。**

- 账只在内存里：「other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,」（`crates/singlefs-core/src/allocator.rs:1941`），各份分配器拷贝共用，所以发布被拒、分配器换回发布之前那一份时账不退（读是真发到盘上的）。
- 清的时机，四处调 `forget_the_locations_read_with_one_copy_failing`：另发那一批认位置读实例表读不出、整批拒时（`crates/singlefs-core/src/mounted_session.rs:2170`）；两样都空交回 `NotNeeded` 时（`crates/singlefs-core/src/mounted_session.rs:2185`）；批发完不论发成还是被拒（`crates/singlefs-core/src/mounted_session.rs:2202`，清的是发之前取的那份快照，加上计划里源落点的槽号对得上的）；挂载交回之前认成映射 key 的那几组（`crates/singlefs-core/src/mount.rs:7876`）。这一批自己推抬 F 时新读到的不在快照里，留给下一次改动。被拒之后「不再试」有用例（`a_mapping_tree_leaf_rewrite_refused_not_for_space_is_not_tried_again_by_the_next_change`，`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:984`）。
- 空间拒再试那一路：抬 F 的空发布都带「InstanceTablePlan::Carry」，不动实例表链（`empty_publish_plan_raising_the_floor` 与 `empty_publish_plan_after_file_version`，`crates/singlefs-core/src/mount.rs:5687`、`:5831` 的函数体里），所以另发那一批在推抬 F 之前算好的 `replaced_chain` 在再试时仍是现行链，不会按旧指针重复释放。映射树路径经 key 重算，推抬 F 若已把那片叶换掉，再试时 `plan_rewriting_the_paths_to_the_moved_entries` 按现行形状走，多重写一次无害。
- 崩溃点：另发那一批是一次普通发布，一个根槽落定；落定前崩溃，坏副本还在盘上，下一次挂载建上一版重新读到、重新记账；落定后崩溃，旧落点已不被任何父指针引用。账在内存里，下一次挂载是新账，所以不会把同一组搬两次。提示过期时旧落点仍被读到，但「按提示读到只坏一份而映射里查到的落点不是提示」不报（`crates/singlefs-core/src/recovery.rs:415`），不会再进账。
- **什么现象会推翻**：下一次挂载建上一版又在已隔离的旧落点上读到只坏一份并计进「挂载读到只坏一份」；用例 `a_mapping_tree_leaf_read_…` 里「下一次挂载读不到坏副本」那条断言（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs` 里 `session_of_a_writable_mount(&mut pool.devices)` 后断言 0 那一段）就是在查它，只覆盖映射树叶一类。实例表与树表两类没有「下一次挂载读不到」的断言。

### 第 4 条：同一份读重复计入、账与别的读那个数不一致、断言的 2 次对不对

**判：断言的 2 次对；同一物理读不重复计；有一处计入面没接全（第二节第 1 行），有一处丢账（设计问题 ⑤）。**

- 去重两层：每一次套子（`PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero`）里同组位置只计一次（`crates/singlefs-core/src/recovery.rs:213` 起那个类型），账里再按「一组位置条目等着下一次改动」去重（`count_the_other_unit_reads_with_one_copy_failing`，`crates/singlefs-core/src/allocator.rs` 的同名函数）。所以实例表链在推抬 F、discard、预演里反复读，账上同一组只算一次。
- 2 次：用例 `an_instance_table_page_read_with_one_copy_damaged_while_raising_the_floor_is_replaced_after_the_next_change`（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:586`）里，第一次被开关拒、没建写入口，所以搬留存读源（`relocations_checked_against_the_version`）那一次在再试时才读；推抬 F 沿链读实例表一次（`crates/singlefs-core/src/mount.rs:5321`，同一串里 `:4801` 先读但不计）。两组位置不同，2 对。我没有跑，是读码推的，主 agent 观测是用例通过。
- 读失败时：`read_the_file` 先读后记账，报错也记（`crates/singlefs-core/src/mounted_session.rs:2703` 起的函数体），`relocations_checked_against_the_version` 先把结果收成 `Result` 再记账，某一项核不过前面的也进（用例 `reading_the_source_of_a_kept_unit_with_one_copy_damaged_counts_it_even_when_the_batch_is_refused`，`:742`）。一处不接：释放前读盘核函数中间的 `?`（设计问题 ⑤）。
- 一次物理坏副本在两个数里各算一次的路径：有活快照的部分写（写入口数 + 搬留存读源）。已在第二节「两种计数口径」一格写明，不算重复计同一读。

### 第 5 条：放回弄坏的字节再判池级 checker，够不够替代放回之前看不到的东西

**判：替代得了「结构类不变量」那一半；替代不了「弄坏的盘 × 部分落盘状态」，也没有一行变异证明放回之后 checker 还抓得住产品的写错。**

放回前看不到什么：弄坏的那一份还在盘上、还被回退候选集里的旧根引用时，I-2.1、I-4.8、I-7.4 本来就红（`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs` 文件头自己写了），没放回就得不到「其余不变量零违例」这一句。放回后得到的：隔离记号与分配记录（I-3.17）、映射 key 与单元头（I-1.11）、引用区间（I-3.5）这些结构类的判，对每个崩溃状态。主 agent 观测 `states_judged_by_the_gpu_here=182`，这一半在此机器上是真判过的。

放回之后看不到、靠手写断言补的：

1. **现行版本不再引用坏落点**：改成 `version_references_the_location`（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:247`；崩溃点那个文件里另抄了一份，`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:170`）。枚举范围是树表、映射树根、中央映射树各节点、实例表链每片、每个进映射的单元、留存条目，与 I-3.15 的引用口径一致（「豁免三类——映射树各节点、树表、实例表各片——与根记录持有的那几样」，`crates/singlefs-checker/invariants/05-空间记账.md:22`），不含 extent 树叶里的位置提示（提示过期本来就允许，读到只坏一份的过期提示不报）。它不与 checker 共用代码，独立性够；缺陷是两个文件各抄一份，将来改一份漏另一份。**判别力**：数据单元与映射树叶两条有行罩着（`crates/mutations.tsv` 第 7130、7119 行）；实例表片那一条用例注释引的「搬留存单元之后留存条目还记旧落点」查无此行（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:584`，现跑 `check-mutation-citations.py` 判 0 命中），所以实例表测试里对留存条目那半的断言没有变异证过它会红。
2. **树表那条断言**：`!version_references_the_location(&after, …, &table_locations[0])` 没有判别力：树表每次发布都换下，任何发布之后这句恒真，写这条的人在报告里也说偏弱。真正有判别力的是同一用例的「旧落点两份都隔离」（`quarantined_copies_of`，罩它的是第 7122 行），它是树表这一类的靠山；该断言本身可以删掉不损失什么，或者改成对「不碰树表的发布（只读会话）」也钉不引用。判：**偏弱但不空**。
3. **弄坏的盘 × 部分落盘**：崩溃点用例的表是在放回字节的基线上枚举 182 个状态判 checker；真正在弄坏的盘上跑 `recover` 的只有每个**前缀**（`for prefix in operations_before..=operations.len() {`，`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:449`），短段（少于 10 个写）按子集展开的那些状态（`persisted_sets_of_the_segments`，同文件第 225 行）没在弄坏的盘上恢复过。也就是「弄坏一份的盘 + 另发那批只落了一部分写」能不能读回文件，只有前缀形态验过。这一格推不出，算打中；补法是对子集状态也跑 `recover`（代价是状态数×一次恢复，182 级，不大）。
4. **放回之后 checker 还抓得住产品写错吗**：实现员报告里自己列的头号风险（`~/.local/share/singlefs/work/m4-close-fixes-writer/result-checker-on-restored-bytes.md` 新变异行表第 4 行「主动修复崩溃流：实例表那一批丢掉最新一行…」）在 `crates/mutations.tsv` 里没有这一行；工作区里只有第 7131 行「判之前不把弄坏的字节放回基线（池级 checker 真在判，报弄坏的那几条）」，它证的是反方向（不放回就红），不是「放回后仍判得出产品写错」。后者没有变异行证过，**推不出**。
5. **no-GPU 机器**：`pool_checker_violations_of_the_image` 与 `pool_checker_violations_of_the_states` 在表交别的机器判时返回 `None`，调用方 `if let Some(...)` 跳过 checker 断言（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:390`）。这是给定状态引擎的通行做法（仓里另有 5 个测试文件同形，现 grep `PlaceholderOfATableLeftToAnotherHost`），不算本批问题；但意味着放回判法在无卡机器上只剩手写断言那一半。

### 第 6 条：T3 改名后的成员在别的调用点的语义、newtype

**判：够不着，附一处语义漂移。**

- `Relocated` 在仓里只被 `crates/singlefs-verification/tests` 里的测试解构，生产代码里只有构造点（`crates/singlefs-core/src/mounted_session.rs:2220`）；新字段 `units_exempt_from_the_mapping_replaced` 只在 `mounted_session.rs:239` 声明、`:2225` 填。没有第二个生产消费者要跟着改。
- 语义漂移：原来 `Relocated` 等于「搬了快照下的旧单元」，`mapping_keys` 非空；现在可以是 `mapping_keys` 为空、只换了豁免映射的单元（用例 `a_mapping_tree_leaf_read_…` 就断言 `mapping_keys.is_empty()` 而结局仍是 `Relocated`）。凡以「Relocated ⇒ 有 key 搬走」写的调用方会错，现在没有这样的调用方，是 API 变宽，文档（`crates/singlefs-core/src/mounted_session.rs` 的成员文档）已写。
- newtype：见第二节「检查单：类型与名字」，三处位置参数不再可互换；单测里那三组参数也用 newtype 写了，位置对调编译不过。

## 四、正文第四节八格

| # | 检查单格 | 回 | 证据 |
|---|---|---|---|
| 1 | 走不到的理由二次查实 | **成立** | 实现员报告写的那一条原样站得住：唯一调用链是「match publish_the_relocation_batch_once(」（`crates/singlefs-core/src/mounted_session.rs:2286`）← 「let batch = publish_the_relocation_batch_raising_the_floor_for_space(」（`crates/singlefs-core/src/mounted_session.rs:2191`），而这一句之前有「if planned.is_empty() && repair.is_empty() {」（`crates/singlefs-core/src/mounted_session.rs:2184`）交回 `NotNeeded`；被断言守着的那一句在「assert!(」（`crates/singlefs-core/src/transaction.rs:10935`），函数只有这一个调用点（现 grep 全 crates 只命中 `mounted_session.rs:2372` 一处调用）。`repair.is_empty()` 看 `units`，认出的映射树节点、树表、实例表片每认一个就推进一个 `units`（`crates/singlefs-core/src/transaction.rs:11928` 起的函数体），所以 `units` 空即无事可做。改动全文里没有新加 `todo!`、`unreachable!`、`Undecided`、`Unsupported`、`OutsideTheFirstVersion`（现 grep 改动全文零命中）。报告没列、但同样「走不到」的是账锁的五处 `expect("账的锁没有中毒…")`（`crates/singlefs-core/src/allocator.rs:1978`、`:1993`、`:2023` 等）：持锁的只有加法与表操作，条件成立，理由写在每个 `expect` 的消息里。 |
| 2 | 类型与名字 | **成立** | `inode_record_is_not_below_the_separator_key_and_the_container` 三个裸 u64 已换，见第二节。`expect` 依赖的不变量逐处：`a_mapping_key_below_the_node` 里「叶不空、内部节点至少一个孩子、孩子的位置在同一份形状里」——读树一路判了空节点：「if header.entries.is_empty() {」（`crates/singlefs-core/src/code_two_tree.rs:1017`），规划一路不留空节点：「PlanningNode::Internal { children, .. } if children.is_empty() => {」（`crates/singlefs-core/src/code_two_tree.rs:569`）；盘上封得对但节点为空的一张盘走读树那一路被拒，到不了这个 `expect`。`Vec<[LocationEntry; 2]>` 一类的键用位置条目整体比（含校验和），不是位置号。新名字里 `repair`、`account` 这类泛名是否过 `vague-names` 门禁格我没跑（实现员报告列了命令），**这一半推不出**，但正文问的是 newtype 与 `expect`，这两样成立。 |
| 3 | 不兼容旧代码与旧格式 | **成立** | 对改动全文跑 `grep -nE 'legacy\|old_format\|migrat'` 零命中；旧名没有留别名：`RelocationOfTheOldDataUnitsKeptForTheSnapshots` 与旧字段名、旧函数名在 `crates`、`scripts` 零命中（现 grep）。 |
| 4 | 不新建判定库 | **成立** | 改动全文 7 个文件块都在 `crates/singlefs-core/src/`；对改动全文 grep `sqlite\|rocksdb\|\.tsv\|File::create\|fs::write\|std::env\|HOME\|hostname` 零命中。新账是分配器上的内存状态（`Arc<Mutex<…>>`），不是判定，也不落盘、不跨挂载。两个新测试走仓里已有的给定状态引擎，没有另开存判定的东西。 |
| 5 | 所有任务多机可用 | **成立（带一条已知的通行限制）** | 产品代码不读本机状态。两个测试的盘都在进程里造：`SparseBlockDevice::new(IMAGE_BYTES, …)` 内存盘，`pool_holding` 里 `make_filesystem` 在内存盘上跑（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:91`）；崩溃点那条用 `build_pool` 的录制镜像。没有 `std::env`、`/home`、`NFS` 之类（现 grep 两个文件零命中，`start_from_environment` 是引擎的入口、读的是仓里统一的环境）。限制：表交别的机器判时 checker 断言跳过，见第三节第 5 条第 5 点。 |
| 6 | 测试优先多线程 | **成立** | `unit_read_…` 11 条各自单池单场景，默认并行；引擎起点用 `one_test_changes_the_environment_at_a_time()` 串行环境变量，不依赖单线程。`crash_points_…` 只有一条，文件里带「harness-test-granularity:one-scenario」说明理由（`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:313`）：同一条录制流上的两次改动与两批，表里的 182 个状态都是这条流的崩溃点，不是 182 个场景。经 GPU 判带 `--test-threads 1` 照 `.claude/rules/verification.md:57`。 |
| 7 | 带 `--item` 的命令与项 | **推不出，算打中** | ① 实现员第二份报告的 21 项 `--item` 命令，在 `crates/mutations.tsv` 里 4 项查无此名（现跑 Python 对第一列：21 项中 17 项在表里）：「现行版本不再引用弄坏的落点：搬留存单元之后留存条目还记旧落点」、「主动修复崩溃流：映射树节点不进那一批的路径重写（两批发完现行版本还引用弄坏的那片叶）」、「主动修复崩溃流：实例表那一批丢掉最新一行（放回字节之后池级 checker 照样判红）」、「查不到 key 改记没坏：坏镜像可写挂载建上一版映射查不到时不转那一报（挂载读到只坏一份计 0）」。表里实际是别的名字（第 7130、7131 行），主 agent 并过表之后那份命令不能原样用，`--item` 要按 `crates/mutations.tsv` 现行名重列。② 两个新用例的注释引的行名有两处查无此行（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:584`、`:1243`，`check-mutation-citations.py` 报 `MISSING 2`）。③ 正文自己写：带 `--item` 的门禁任务 11017 取消，变异在副本上证红 19 行，余下 5 行证红任务另交、判决写下时无结局；所以「每条新测试都有一行证过会红」目前只到 19 行。④ 第 7126 行罩的是 wgsl:286，663 那一处无行（第二节末行）。这一格的命令与项在主 agent 重新列过之前，判不了。 |
| 8 | 新崩溃流多罩了什么 | **推不出，算打中（崩溃流一半成立，GPU 流一半推不出）** | **崩溃流多罩：成立（按构造）。** 旧代码里释放前读盘核只罩 `mapped_roles_released_by_this_publish` 点名的经映射的角色，映射树节点、树表、实例表片都在 `false` 那一组：「| TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,」（`crates/singlefs-core/src/transaction.rs:5221`），所以旧产品不可能产出「豁免三类的旧落点带隔离记号」，任何旧流都罩不到这个状态；新流枚举的「只换落点、条目一个不变的映射树节点重写」与「行不变的实例表整条链重写」也是旧产品没有的发布。**GPU 流多罩：推不出。** 新流只罩 wgsl:286 那一处（打开），同一句下界判在 663（建上一版）无流无行；GPU 缩域到 17 过、全量域 GPU 没跑（正文第二节），所以「全量域上 GPU 与 CPU 对拍照旧一致」没有观测。 |

## 五、什么现象会推翻上面的结论

| 结论 | 推翻它的现象 |
|---|---|
| 别的读没接全（第二节第 1 行） | 证明那四处读在可写会话里走不到（例如 `rollback_states_of_the_ring` 的读者其实是计数层），或用户在 D4 里把「别的读」收窄到那四处点名的；我读到的 D4 第 73 行字面是「可写挂载期间别的读」，用户定案是「按字面全接上」 |
| 豁免三类只核「记着位置的」是 kb 没对齐，不是实现错 | D4 或 D19 有一条明写豁免三类的旧落点**无论读没读到**都要核（我读到的 D19 第 190 行恰好相反） |
| T3 开关名没改 | 实现员或主 agent 已在别处改了这三个开关（我现 grep `pub enum RelocationKeptForTheSnapshotsRefusedForSpace` 仍在） |
| 计数口径四个数、注释说三个 | 注释在别处已补第四个（我只在 `mounted_session.rs:278` 与 `:2687` 两处找到） |
| C946 与 D4 第 73 行还没写回 | 写回已经在别的会话的暂存区里（我读的是工作区文件） |
| GPU 同形第二处无行 | 现有某条流在 663 被弄坏时会红（需要跑变异：把 663 那一句改成恒假，看哪条红） |
| 两个变异行名查无此行 | 行在表里改了名而注释没改（我用「引文是行名的子串」判，改名会漏判） |
| 放回字节判法覆不到「弄坏的盘 × 部分落盘子集」 | 另一条测试在弄坏的盘上对子集状态跑过 `recover` |
| 第三节第 1 条（key 被换）不算缺陷 | 见第三节第 1 条末尾的两个现象 |
| 设计问题 ⑦ 的触发面比实现员写的宽 | 认不出的几组里并没有「这次改动已换下的旧落点」（也就是 `locations_without_a_mapping_key` 在这一形上为空），则只剩实例表片一类触发 |

## 六、同类先例

- 「读到只坏一份记账层只接字面点的几处」：`research/prompts/m4-close-code-r3-main-verification.md` T2 ①（实例表链一次挂载读两遍，计 2）就是同一类——记账层接在哪几处读上、同一份读被几处各接一次的问题；这一批沿用了它的做法（按位置条目去重）。
- 「挂载只数条目数、不核映射内容」：找不到先例条款。在 D4、D19、D21、I-1.11 登记行与 `recovery.rs` 的 `mapping_entry_count_is_one_per_mapped_unit` 文档里找，只找到既有的挂载侧只数不核的行为，没有一条要求挂载核映射 key。
- 「别的会话在 D4 上的字面与 D19 自举豁免的字面不一致」：找不到之前的判决专门处理过；本轮只记为 kb 写回要对齐的一处。

## 七、要主 agent 跑的命令

我没有往统一任务服务交任务，没有跑 cargo、没有跑变异。读数都是现读文件、现 grep 与一个只读 Python 脚本。下面这条是我的脚本，主 agent 要引它的读数就交一次：

```
task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/m4-close-fixes-code-r2-sonnet/check-mutation-citations.py /home/fy5090/code/singlefs
```

期望：两个新测试文件里引的变异行名，当前打印 `MISSING 2` 并退 1（`unit_read_…rs:584` 的「搬留存单元之后留存条目还记旧落点」与 `:1243` 的「坏镜像可写挂载建上一版」）；主 agent 补行或改注释之后应打印 `MISSING 0` 并退 0。脚本 sha256 `872395fb3237effa8b5ff30fa6c3bfb7848dcf9cbb23f1ebe2885713633b741e`（`sha256sum` 量的）。

不交任务、主 agent 自己一眼可核的一组 grep（都在仓根）：

```
grep -nF 'let current_instance_table = instance_table_chain_of_root(devices, &current.root)' crates/singlefs-core/src/mount.rs
grep -nF 'match rollback_states_of_the_ring(' crates/singlefs-core/src/mounted_session.rs
grep -nF 'if (lt64(inode, separator) || lt64(inode, identity_container)) {' crates/singlefs-verification/src/crash_judge_recovery.wgsl
awk -F'\t' 'index($3,"lt64(inode, separator)") {print NR": "$1}' crates/mutations.tsv
```

期望：前两条各命中 `:4801`、`:4098`；第三条命中 `:286` 与 `:663`；第四条只命中第 7126 行。

建议另交的实验（腿没写代码）：对 `crash_points_of_the_repair_of_the_units_exempt_from_the_mapping` 里短段的每个子集状态，在弄坏的盘上也跑 `recover` 读回文件，看是否都读回原内容；对 `wgsl:663` 做一个恒假变异看现有哪条流红。

## 八、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务；判决要引的数没复跑（模型：本轮没写模型；验证包与 GPU 的读数 182 条、11 条、1 条、17 条、849 条都是正文第二节主 agent 的观测，我没复现）。
- 没编译、没跑任何 cargo、没跑变异、没跑门禁；没碰 `crates/` 下任何文件；没读全量域 GPU 内核与池级 checker 的实现，只读了不变量登记行与相关调用点。
- 没复核 `m4-close-code` r1 到 r3 已判过的改动；没攻 r1 第二节以外的角度。
- 没验证 `check-mutation-citations.py` 以外的变异行是否真能证红。

## 九、引文核对末行

`python3 scripts/agents/cite-check.py research/prompts/m4-close-fixes-code-r2-sonnet-output.md` 的末行（退出码 0，没有判红；裸行号多是我只给行号、没抄原文的位置，已在上面逐条现 grep 核过那一行存在）：

```
✓ 核了 31 处引文、44 处裸行号，对上 31 处，没判 44 处（其中裸行号没有引文可比 44 处；逐处列在上面）
```
