# m4-close-fixes-code-r3 辩方腿报告（2026-11-09）

辩护的一方：现行实现、r2 判决第三节用户定案（映射 key 被换的盘挂载时拒）、用户定的 P2a。读的是主工作区 `crates/` 今天的样子，不是副本。

判定用词：「站得住」= 修补真补上了那一形；「够不着」= 残留的攻击够不着现行实现或用户定案；「同样打中别的臂」= 残留的问题替代方案同样有；「打中（辩不住）」= 我替它找不到站得住的理由，照实认。

## 一、结论速览

- r2 判决第二节 16 格：站得住 13 格（其中 5 格带残留：kb 待写回两格、放回字节一格、悬空指针一格、GPU 默认环境基线一格），够不着 1 格，打中（辩不住）2 格。辩不住的两格：「按字面全接上」整体（`transaction.rs:679` 没接，且文档写「都走它」）；测试文档点名的变异行又有两条表里没有。同样打中别的臂的残留一处：测试里的静默跳过是引擎既有写法。
- 第四节八格：成立 7，不成立 1（带 `--item` 的命令与项，后半还有推不出的），推不出 0。类型与名字一格成立，附 `allocator.rs:1867` 悬空的「不接的依据」这一处小瑕疵。逐格见第五节。
- 特别问题：`transaction.rs:679` 对「按字面全接上」够得着，而且是每次发布都走的一读，不是「通常紧接着也读」；`mount.rs:1407` 一带今天的行号在主工作区指的是建上一版那一读（已接计数），穷举表那一串「不接」的读在主工作区是 `mount.rs:1767`、`:1809`、`:1998`、`:2170`、`:2241`、`:2525`（影子账读被抛弃根与更新根的账）与 `:3128`，对「计数」半句够得着、对「搬」半句够不着，条款空白。
- 未定项清单漏的（第 4.3 节，13 行里 10 行无去向或去向没落地，含攒批那一行）：重试尾巴、多片实例表链用例、实现员自定的「一样要发的都没有」归属、GPU 与 CPU 差的一形、坏盘乘部分落盘之后接着可写挂载没判、写入口那一读与挂载影子账那几读、崩溃枚举用例的可写挂载期望、E206 登记字面与页处置相撞。攒批读不到待发写：我查到的范围里条款空白，建议另立题，不进 D4。

## 二、判决条目逐格（r2 判决第二节）

引文行号都是此刻在被引文件里 `grep -nF` 现取的。`crates/mutations.tsv` 的数是 `grep -c`、`awk` 量的；任务读数是 `task-server/task log <号> --all` 读的（任务号是主 agent 在正文里给的）。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| D4（校验和位置） 已定项 4 与用户定案「按字面全接上」：判决点名的七处没接 | 七处站得住；「字面全接上」整体打中（辩不住） | 七处都接了：`reading_and_counting_the_other_unit_reads_with_one_copy_failing` 的非文档调用点量到 14 处（`grep -nF` 本命令数出：`instance_table_sweep.rs:743`、`:921`、`:967`，`mounted_session.rs:3318`、`:4219`，`mount.rs:4693`、`:4818`、`:5356`、`:10745`、`:10750`、`:10766`、`:10795`、`:10878`、`:10907`），认位置那一读走 `transaction.rs:11998` 自己套的数只坏一份的读者。判决七处与它们的对应：`mount.rs:4801`→`:4818`，`:4256`→`:3318`、`:4219`，`:10728`→`:10745`，`:10847`→`:10878`，`instance_table_sweep.rs:356`→`:967`、`:741`→`:743`。没接的残留见第四节。 |
| 映射 key 被换、封得对的盘 | 站得住（推的 + 量的）；只读路径够不着 | 判在 `rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at` 里：「central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots(」（`crates/singlefs-core/src/recovery.rs:3637`）是调用，定义在 `recovery.rs:6874`。两个集合来自两个独立来源：期望集合是树里读出的进映射单元的 key 加留存条目的 key，留存条目取自 deadlist：「if deadlist_mapping_keys.contains(key.as_slice()) {」（`crates/singlefs-core/src/recovery.rs:4570`），映射里的集合取自映射树的叶条目，不是同一个数的两种写法。比起旧的条数判，只多拒「条数相等而集合不等」，合法状态里这一形不会有：每次发布每个进映射的单元都插自己的条目。用例与变异行在表里（`mutations.tsv` 第 7283 至 7286 行），任务 11331 `unit_read_…` 37 条过。只读路径不判：我读 `mounted_read.rs` 的先按提示读再查映射的次序，换 key 不动位置条目与单元字节，读不出别的单元的字节（读码，推的）。 |
| 七条① 没读到的坏副本被换下时不核 | 站得住（代码没打中）；kb 写回未做 | 代码没改，r2 判代码没打中。D4 那半句与实现反着是 kb 的事：「它们在挂载这一串的发布里本来就换落点」（`.claude/kb/decisions/04-校验和位置.md:73`）。这批改法让这半句又多一处对不上：树表那一支删了，实例表片与映射树节点是否「本来就换落点」要写回时重看。写回随批，没做。 |
| 七条② 攒批会话读不到待发写 | 够不着；建议另立题 | `MountedSessionWithPendingChanges` 的文档自己写：「读只经 [`Self::session`] 与几个观测。」（`crates/singlefs-core/src/mounted_session.rs:5832`），`session()` 交已发布那一版。在 `.claude/kb` 下 `grep -rnE '写后读\|read-after-write\|读到刚写'` 0 命中（量的，只扫这三个词）；D4 条款射程管只坏一份的读，不管读语义。不进 D4，也不是实现缺陷。 |
| 七条③ 账的口径不统一 | 站得住 | 挂载交回时不再把认成 key 的组从账上去掉：对应的 `forget` 调用在 diff 里是删行，`mount.rs` 现在只剩 `keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting` 那一调用（`crates/singlefs-core/src/mount.rs:7896`）。数据单元与映射树叶挂载那一个数各计一次，之后再读不加：用例 `the_other_reads_count_of_a_copy_read_while_mounting_is_the_same_for_a_data_unit_and_a_mapping_tree_leaf` 在任务 11331 的 37 条里过，变异行 7282 在任务 11288 抓到。 |
| 七条④ 读盘核之后才被拒，记账 | 站得住（用例只造了一形） | 结构保证：`prepare_the_version_publish_relocating_units` 读盘核之后、`publish_admitted` 之前记账：「let quarantine = quarantine_read?;」（`crates/singlefs-core/src/transaction.rs:13546`）在记账之后；账是 `Arc<Mutex<…>>`：「other_unit_reads_with_one_copy_failing: Arc<Mutex<OtherUnitReadsWithOneCopyFailing>>,」（`crates/singlefs-core/src/allocator.rs:1944`），各份分配器拷贝共用，所以攻方说的「分配器换回」那一形也不撤销计数。用例用写失败那一形造，变异行 7288 抓到（任务 11288）。 |
| 七条⑤ 提前返回丢账 | 站得住 | 读盘核整段挪进 `copies_quarantined_by_the_release_checks`（`transaction.rs:13377`），内部的 `?` 都在它里面；调用方先记账、后 `quarantine_read?`。变异行 7287，用例 `a_release_check_refused_part_way_keeps_what_it_read_before_the_refusal_in_the_count`。 |
| 七条⑥ | 站得住（r2 判没打中，没动） | 无改动。 |
| 七条⑦ 实例表读不出整批拒 | 站得住；「一样要发的都没有」那一形是实现员自定，归未定项 | `repair_of_the_exempt_units_at_these_locations` 不再返回 `Result`，唯一调用点 `crates/singlefs-core/src/mounted_session.rs:2253`；链读不出时：「实例表那一半跳过（读不出也写不了行不变的重写），映射树节点与要搬的单元照发，结局报明实例表没换」（`crates/singlefs-core/src/transaction.rs:11941`），`planned` 照发。变异行 7296 至 7298，任务 11288 抓到。没别的要发时报 `RefusedNotForSpace(InstanceTableOfTheVersionUnreadable)` 是实现员写在报告里的选择（报成不用发会把读不出吞了），条款没写，见第四节。 |
| 只试一次 | 站得住（代码与注释）；kb 那一句要写回才对得上 | 注释收窄成写实的一句：「这一组记下的这一次只试一次，结局报在这里，之后的用户改动不接着搬；同一组之后再被读到就再记、再试一次」（`crates/singlefs-core/src/mounted_session.rs:228`）。用户当时的原话落在 `.claude/kb/decisions-history.md:1558`：「挂载交回之前记下，挂载之后第一次用户改动发成之后并进那次改动之后另发的那一批搬迁，只试这一次」，管的是挂载记下的那一批，没说同一份坏副本被会话再读到之后还试不试；再读到走「可写挂载里的读都计数并搬」那一条，是另一个触发。D4 条款里的句子是「那一批搬没搬成都只试这一次，结局随那次改动报出，之后的用户改动不再接着搬」（`.claude/kb/decisions/04-校验和位置.md:73`），字面读会和「再读到再试」打架：写回 kb 时要写成「每一组记下的只试这一次」。用例 `a_batch_refused_not_for_space_is_tried_again_once_the_unit_is_read_again` 钉计数 3，变异行 7299、7310 抓到（任务 11311）。 |
| 放回字节的判法（树表断言没判别力；放回之后 checker 抓不抓得住产品写错） | 站得住；残留两处，其中一处同样打中别的臂 | 树表断言换成读分配记录的隔离记号；卸载之后不放回判真盘加四臂与没修的对照：对照 `the_disk_after_the_unmount_with_the_data_unit_not_repaired_is_red`（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:2350`）必须报 I-2.1 这一族。搬迁那一批不发的变异行（7300）红在 checker 这一判上：绿用例 `assert_the_disk_after_the_unmount_is_green`（同文件 :2336）只断言违例集为空，没有别的前提断言先红。残留一：映射树叶那一臂的变异行（7301）红在搭建函数里「现行版本不再引用」的断言，不是 checker（实现员自陈）；二：`if let Some(violated) =`（同文件 :2341）在判的是占位表时静默跳过，`PlaceholderOfATableRecordedOnly` 在 14 份测试文件里是同一写法（`grep -l` 量的），所有臂一样，同样打中别的臂，属引擎既有约定，不是这一批新开的。 |
| 坏盘 × 部分落盘 | 站得住（`recover` 那半）；「接着可写挂载」没判，归未定项 | 同一张持久集合表在不放回字节的盘上逐个 `recover`，按线程切：「let report = recover(&image, JournalPolicy::Consult);」（`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:470`），用例在同文件 :545，变异行 7302 红（任务 11288）。这一批写了一半、坏副本还在、接着可写挂载的一形没判（派发提示「不碰」），checker 在不放回的盘上本来就该红（I-2.1 这一族），判不了。 |
| 名字与死链 | 站得住；附一处新的悬空指针 | 旧名在 `crates` 下 `grep -rnE` 三个开关旧名、`relocations_of_the_units_read_with_one_copy_failing_while_mounting`、`mapping_keys_of_the_units_at_these_locations` 全 0 命中（量的；`RelocatedUnitHolder::KeptForTheSnapshots` 是另一个真名，命中不算）。新悬空：「每一处读与不接的依据见」（`crates/singlefs-core/src/allocator.rs:1867`）指向的辅助函数文档（`allocator.rs:1985` 起）只列了接了的读，没有一句不接的依据；同一段文档还写「可写挂载期间经 `reader` 按两条位置条目读单元的读都走它或同形的写法」（`crates/singlefs-core/src/allocator.rs:1986`），对 `transaction.rs:679` 不是真话。 |
| 走不到：`transaction.rs:11953` 一支 | 站得住（推的，结构依据） | 删掉的是「账上组恰是现行树表」那一支。实现员的理由：账上的组都是这次发布之前读到的，每次发布都换树表。我查的结构依据：实例表有 Carry：「instance_table: InstanceTablePlan::Carry(previous.root.instance_table),」（`crates/singlefs-core/src/transaction.rs:9869`），树表没有对应的 Carry 变体（`grep -nE 'TreeTablePlan\|tree_table: *(InstanceTablePlan\|TreeTable)'` 在 `transaction.rs` 0 命中，量的）；`publish_user_change` 里 `published?` 在 `relocate_…` 之前：「let mut published = published?;」（`crates/singlefs-core/src/mounted_session.rs:2949`）。没逐个核 `UserChange` 各臂；推翻它的现象：某一臂发出的版本没换树表。补丁加行里 `todo!`、`unreachable!`、`assert!`、`panic!`、`.expect(`、`.unwrap(` 与 Undecided、Unsupported、OutsideTheFirstVersion 的 `grep -nE '^\+'` 0 命中（量的，对 `research/prompts/_m4-close-fixes-code-r3-diff.md`）。 |
| GPU 下界（`crash_judge_recovery.wgsl:663` 没变异行；GPU 那一行点全量域超时） | 站得住（行与用例在）；默认环境下的基线推不出 | 变异行在表里（`crates/mutations.tsv` 第 7303 行），点名缩域用例（`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:22119` 一带），任务 11307 `抓到 4 条`。但那趟是 `env SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=17` 起的（`task-server/task status 11307` 的命令行），变异行第五段不带这个变量；用例里 `expand_up_to()` 没设变量时取全量域（`…every_state.rs:350` 起的 `expand_up_to_from_the_value_of_the_variable`）。「缩域」是 `states_staying_on_r()` 加这个变量两层，不设变量时多大没人量过：见「要主 agent 跑的命令」第 1 条。 |
| 测试文档引的变异行（`unit_read_…rs:584`、`:1243`） | 打中（辩不住）：同形的缺陷又出现两处 | 原来点名的两处改对了。但同文件新写的两条用例又点名了表里没有的行：「可写挂载别的读：回退做成时同一份坏副本再读又加次数（重建目标那一版）」（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:2149`）与「可写挂载别的读：回退做成时同一份坏副本再读又加次数（读现行实例表）」（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:2229`）。我写的 `check_rows.py`（草稿目录里，读 `mutations.tsv` 第一列做前缀比，只读）对这个文件报 `quoted 36 missing 3`：这两条与第三条 `:2479`（引的是「…（提示过期那一形）」，表里是「…（提示过期那一形，key 盖回隔离的旧落点）」，语义对得上、字面不是前缀）。`grep -c 'rebuilding_the_target_of_a_rollback_counts_the_damaged_copy' crates/mutations.tsv` 与 `grep -c 'a_rollback_reading_the_current_instance_table_counts_the_damaged_page' crates/mutations.tsv` 都是 0（量的）：这两条用例今天没有任何变异行点名。实现员第三份报告说补了三行，补丁里 `mutations-append.tsv` 只剩末一行（`grep -c '同一份坏副本再读又加次数'` 两份表都是 0）。 |

## 三、P2a（攻击面 1）

**登记的前提**自己标了「推的」：「这一句是推的，Q5 量它」（`research/prompts/e206-preregistration.md:316`）。我替它找到的站得住的理由和它够不着的地方：

1. 推导（推的，依赖单调性）。重写集 R 里的节点只要上一版有、这一版换下，它的两份旧落点就被释放，释放改写那条记录，记录所在的叶（槽号定）与叶的祖先内容就变了，所以也在不动点里。产品路径上「释放只改写记录、不在这次发布里空出槽（回收要等根记下之后）」（`crates/singlefs-core/src/transaction.rs:13770`），重写集越大取落点那一串越长、有记录的节点只多不少，变了的节点集合对 R 单调。单调之下：每个不动点都对释放链闭，从空集起只并「在每个不动点里必有」的节点，停下的地方就是最小不动点，也就是旧做法停下的地方。不在上一版的节点没有旧落点，走到它为止（`merge_into_the_rewrite_set_walking_the_release_chain` 里 `pointer_of` 交 `None` 就 `continue`）；在重写集里却不在这一版节点里的，旧落点由 `replaced_previous_nodes` 照样释放，走它也不多。读上一版指针的时机对：「let previous_tree = previous.map(」（`crates/singlefs-core/src/transaction.rs:13817`）在循环外取一次，闭包读的是上一版，不读 `rehearsal`。**推翻它的现象**：某段历史上产品代码闭包停下的重写集与把闭包换回旧做法的停下的不同（多出的就是多写的单元）。复用窗口置 0 的测试开关下单调性推不出来，代码自己承认：「只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性也推不出来」（`crates/singlefs-core/src/transaction.rs:13774`），产品路径不进。
2. 量的，范围窄。产品代码上只有一条历史：新目标跑出来重写 99 个角色、87 个节点，与测量补丁上两臂的数相同（`name=peaks item=files_1000_units_10__p1 …peak_final_roles=99 peak_final_nodes=87` 与 p2a 同值，见 `research/results/e206-fixed-point-cost-report-2026-11-09.out` 里这两行；任务 11332 `1 passed`）。这条历史的断言「重写集恰是 `nodes_whose_contents_changed`」是固定点停下条件本身，不是与旧做法的独立比较；与旧做法比的只有 99、87 两个数和 3 轮的峰值。测量补丁上 Q5 同集比对 15 行（`grep -c` 量的），十六个点里少的那一个是 `files_1000_units_100`：旧做法在那里撞上界——「peak_rounds=64 at=same_mount/W1/1 verdict=hits_the_limit」（`research/results/e206-fixed-point-cost-report-2026-11-09.out:357`），页上写明「那个点上 P2a 没有比对对象」（`.claude/kb/experiments/206-发布固定点轮数的代价.md:16`）。该点 P2a 的读数是「peak_rounds=4 at=remounted/W2/13 verdict=not_many」（同产物 :371），是测量补丁上的数，产品代码上没有用例（要约 17 GiB，实现员报告自陈）。所以「P2a 的不动点就是旧做法的」在十六个点里有十五个有对照，唯独旧做法过不去的那一格靠推导。
3. 登记字面与页的处置有一格相撞。登记写：「相同率 < 1 或索引形态下净多，P2a 翻面，P2 只剩 P2b」（`research/prompts/e206-preregistration.md:364`），产物里五个点按索引形态也是净多（「5 linear=costs_more indexed=costs_more」，`.claude/kb/experiments/206-发布固定点轮数的代价.md:155`）。页的处置是净步数这一量出自 P1 逐轮读数、P1 一侧作废、只作线索（`.claude/kb/experiments/206-发布固定点轮数的代价.md:126`：「净步数是 P2a 与 P1 逐轮读数之差，P1 一侧作废，仅作线索」）。P2a 站住靠的是用户定案加作废条款 1，不是 Q5 那一条过了；这一格该进判决的交用户表。
4. 什么形状 P2a 仍走满 64 拒：我推不出有，也没人量到。代码文档不替它担保：「这一半的上界没有推出来，E206（发布固定点轮数的代价）全部取样点量到至多 5 轮」（`crates/singlefs-core/src/transaction.rs:13773`）。我的推导（推的，没量）：闭包之后剩下的轮数来自取落点碰到新叶，每轮多出来的节点角色把游标往后推不过两槽乘新节点数，而一片叶有 812 槽（`crates/singlefs-format/src/lib.rs:163`），要接连碰到新叶界得有游标恰好贴着界的几轮连着发生，所以我不期望 64。这句话的「全部取样点」只是第一段十六个点，第二段 33 项没跑（页首行写明）。推翻它的现象：任一历史上产品代码交回 `AllocationRecordTreeRewriteSetDidNotSettle`。
5. 写行那一份（树表 0 条那一版）两处并入（`transaction.rs:1987`、`:2010`）没有变异行：改回不走闭包，只影响轮数，不影响结果（那一版的树只有 mkfs 两个单元、实例表链与几个节点，释放链短，离 64 很远）；闭包函数本身两边共用，「只走一跳」「走过了头」两行（7306、7309）管它。盲区是真的：接线写错（传错上一版）没有用例会红。我判风险低，不判等价。

## 四、特别问题

### 4.1 `transaction.rs:668` 一带（主工作区今天是 `transaction.rs:679`）对「按字面全接上」够得着

穷举表里写它「不接，交主 agent」，理由是 `PoolWriter` 手里没有分配器。我替它找不到站得住的理由：

- 它是每次发布都走的一读，不是偶尔的一读。所有发布路径共用一处收尾：「三条发布路径（带单元的、零单元的、树表 0 条上只写实例表的）与重发冻结的那一次共用这一处」（`crates/singlefs-core/src/transaction.rs:1656`），收尾里 `CommitStep::RotateSystemConfigurationSlots` 逐盘调 `write_system_configuration_slot`（`crates/singlefs-core/src/transaction.rs:604`），那里「let pool_effective_floor = effective_rollback_floor_rereading_unreadable_reads_once(」（`crates/singlefs-core/src/transaction.rs:679`）现读根环、择最新那条根、沿链读它的实例表。最新那条根是刚写的这一条；用户改动的发布（覆盖写 `publish_overwrite` 从 `transaction.rs:9840` 起，顺序写、建 inode 同）实例表计划是 Carry（「instance_table: InstanceTablePlan::Carry(previous.root.instance_table),」，`crates/singlefs-core/src/transaction.rs:9869`），所以这类发布收尾读的是没换过的旧实例表各片；只有写行那条路径自己换了实例表。
- 读的是按两条位置条目读单元，读者是 `&*self.devices`，只坏一份的那一报落在默认的空实现上，没人数。
- 穷举表说「通常紧接着的 discard 或推抬 F 也读，但不保证」，这个「通常」不成立：发 discard 的函数在没有可发的段时第一道就交回，根本不读实例表（「.all(\|(_, segment_starts)\| segment_starts.is_empty())」，`crates/singlefs-core/src/mounted_session.rs:3990`；读实例表的 `let rows_of_the_instance_table` 在其后，`mounted_session.rs:3997`）；抬 F 不是每次发布都走（空间被拒时推一串，卸载时抬到现行那一版）。会话期间实例表某一片坏了一份，每次发布收尾都读到它，却是整条路径上唯一读到它的一读，不计数、不记位置、不搬。
- 「账够不着」不是障碍：写入口自己已有两样随写入口走、会话事后取走的账：「old_data_unit_reads_back_with_one_copy_failing: Cell<u64>,」（`crates/singlefs-core/src/transaction.rs:372`）与映射 key 那一份「mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,」（`crates/singlefs-core/src/transaction.rs:376`）。位置那一组同形可记，会话在发布之后并进分配器的账。
- 代码里的理由同样不成立。读这一处的注释说的是另一件事：「这个写入口手里没有分配器上那张根环表」（`crates/singlefs-core/src/transaction.rs:675`）讲的是不拒的缘由，不是不计数的缘由。

判：够得着，辩不住。这一读对用户字面「可写挂载期间别的读」是明确在内的，也是频次最高的一个。推翻我这句话的现象：会话里弄坏现行实例表一片的一份、不读文件、不发 discard、不抬 F，只发用户改动，数 `other_unit_reads_with_one_copy_failing_in_this_mount()` 仍为 0 即证实；若为 1，说明某处我没看到的读已经数了。

### 4.2 `mount.rs:1407` 一带

- 写进穷举表的行号里 `mount.rs:1407` 今天是 `rebuild_previous_version` 里的「let table = instance_table_chain_of_root(reader, root)」（`crates/singlefs-core/src/mount.rs:1407`），两个调用点都交的是套着数只坏一份的读者（`mount.rs:9790`、`:9841` 的实参 `devices_counting_unit_reads_with_one_copy_failing`），这一读已经在计数（进挂载那个数），不是「不接」。穷举表那一行指错了行。
- 表里那一串里剩下的 `mount.rs:1767`、`:1809`、`:1998`、`:2170`、`:2241`、`:2525` 都是 `placements_referenced_by_root` 的调用，`:3128` 是 `effective_rollback_floor_under_the_newest_roots_table`：挂载重建分配器时读被抛弃根与比所选更新的根的账、读最新根的实例表。它们读的是别的版本，读到只坏一份时没东西可搬（`units_at_these_locations` 认不到现行版本的映射 key）。挂载里的注释给了第一层依据：「判退的三样读者不经它：它们读的是建上一版接着要读的同几个单元，」（`crates/singlefs-core/src/mount.rs:9767`）。但这句只覆盖判退的三样读者；影子账读的是另一批单元。
- 会话期间它们还会再走：抬 F 时重算影子账，借已记着的落点不读盘，「两样都没记着的照旧读盘」（`crates/singlefs-core/src/mount.rs:5453`）。
- 判：「搬」那半句够不着（没有可搬的单元）；「计数」那半句够得着，字面「读到只坏一份时计数」（`.claude/kb/decisions/04-校验和位置.md:71`）不分读的是哪个版本，「可写挂载期间别的读」（`.claude/kb/decisions/04-校验和位置.md:73`）也没把挂载里读别的版本摘出去。要接计数得有地方放：这些读的单元不属于现行版本，进「别的读」那个数还是单立一个，条款没写。条款空白，不是实现遗漏，交主 agent 或用户定。

### 4.3 未定项清单有没有漏

我对着 r1、r2 判决第五节、两份 r2 腿报告、实现员三份报告「没做什么」与「交主 agent」逐条找去向。去向 = 判决里有一行说谁、什么时候做：

| 条目 | 来源 | 有没有去向 | 判 |
|---|---|---|---|
| kb 写回：D4 第 71、73 行、C946 | r1 第五节、r2 第五节 | 有（随这一批收尾） | 不漏。但这批改法让第 73 行至少三处要重写：「它们在挂载这一串的发布里本来就换落点」（树表那支删了）、「那一批搬没搬成都只试这一次…之后的用户改动不再接着搬」（与再读到再试字面打架）、「读到只坏一份时计数」没写粒度（账按组去重：「读到只坏一份的次数：一组位置条目等着下一次用户改动的这一程里只算一次。」（`crates/singlefs-core/src/allocator.rs:1873`），条款不分） |
| 攒批会话读不到待发写 | r2 第五节 | 判决写「登 checks-owed」，`.claude/kb/checks-owed.md` 里今天没有这一行（`grep -n '攒批' .claude/kb/checks-owed.md` 只命中 C168、C418、C968，三条都是别的事） | 去向没落地，补登 |
| `transaction.rs:679` 写入口轮换那一读不接 | 实现员「交主 agent」 | 无 | 漏。见 4.1，辩不住 |
| 挂载重建分配器读被抛弃根与更新根的账不接 | 实现员「交主 agent」 | 无 | 漏。见 4.2，条款空白 |
| 「一样要发的都没有而实例表读不出」报 `RefusedNotForSpace` | 实现员自定，写明「主 agent 不同意就改」 | 无 | 漏。名字说「被拒」而这一形什么都没被拒；换成单立成员要动 `RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing` 的封闭集合，调用方 `match` 要跟着改 |
| GPU 与 CPU 差一形：两个单元同一把 key、映射多出一条别的 key | 实现员第一份报告第 2 条；wgsl 注释 | 无 | 漏，零轮。「一版里两个单元同一把 key 而映射多出一条别的 key 的那一形这里判不出（CPU 判集合不等），只有造坏的盘走得到」（`crates/singlefs-verification/src/crash_judge_recovery.wgsl:846`）；它前一行写的是：条数相等、每个进映射的单元的 key 与每条 deadlist 条目的 key 都查得到。崩溃枚举里没有这一形的流，所以对拍永远看不到分歧；要 GPU 也判得有人给映射条目建访问表，代价没量 |
| 重试尾巴：抬 F 的空发布已经换下记着的叶，结局仍报进 `units_exempt_from_the_mapping_replaced` | r2 攻方腿报告攻击面 3（`research/prompts/m4-close-fixes-code-r2-opus-output.md:159`） | 无（r2 判决第二节没列，实现员「没在第四节，没动」） | 漏，轻：结局字段说「同一次发布换下」对不上 |
| 多片实例表链用例 | r2 攻方腿报告（`research/prompts/m4-close-fixes-code-r2-opus-output.md:183`） | 无（实现员「没入库」） | 漏。这一批新改的实例表重写走整条链，只有一片的取样点不分「整条换」与「只换坏的那一片」 |
| 坏盘 × 部分落盘之后接着可写挂载 | 实现员「只判 recover，可写挂载不判」 | 无 | 漏。现有用例的 `recover` 那半是真的（4.2 表里一行），接着挂载的一形没人判 |
| 崩溃枚举观察者与各枚举用例的可写挂载期望没改（派发提示「不碰」） | 实现员第一份报告 | 无 | 漏。可写挂载读盘建上一版会多一道集合判，对读到只坏一份的池可能多发一批；我只看到任务 11334 缩域那一个用例 177 个状态绿，别的枚举用例没跑 |
| 接了线而没有变异行的 6 处 | 实现员第一份报告 | 本轮攻击面第 3 条已含 | 不漏。`grep -c` 量到变异表里这 5 个锚点（`highest_inode_number_watermark_in_the_ring`、`stop_of_the_rollback_gate`、`chain_read = allocator`、`reading_and_counting_…(&**devices`、`allocation_records_under_root_in_the_unit_area_starting_at`）都是 0 命中。实现员说「取样造不出」。测试里已有的盘 `DeviceFailingOnRequest`（`crates/singlefs-verification/tests/unit_read_with_one_copy_failing_during_a_writable_mount_is_repaired.rs:1632`）的读失败开关由用例在两次调用之间拨，一次调用中间（判与发之间）弄坏要让它按「第几次读」失败，现成的没有；我不认「造不出」，认「没造」（推的，没试） |
| 整表变异复跑与 `--item` | 判决「整表归提交时」 | 有 | 不漏。任务 11017 取消，45.6 GiB 等整台腾空 |
| E206 登记字面 Q5「索引形态下净多 → P2a 翻面」与页上「P1 一侧作废、仅作线索」相撞 | 本轮 | 无 | 漏，交用户表。见第三节第 3 点 |

### 4.4 映射 key 集合那一判：合法的盘会不会被拒（攻击面 2）

- 期望集合的两半来源独立：进映射单元的 key 取自树里读出的单元，留存条目的 key 取自 deadlist：「if deadlist_mapping_keys.contains(key.as_slice()) {」（`crates/singlefs-core/src/recovery.rs:4570`）。条数判（旧）把两者相加与映射条数比；新判只在「条数相等而集合不等」时多拒一类。合法历史里每次发布每个进映射的单元插自己的条目、留存的条目在 deadlist 里，我没找到条数相等而集合不等的合法状态。
- 留存条目、搬迁中崩溃、回退之后、快照销毁、整理半途：搬迁与回退换的是条目的位置，key 不变；快照销毁与整理在同一次发布里同改映射与 deadlist。已跑的证据只有两处：任务 11331 `crash_points_…` 182 个状态绿（`states_judged_by_the_gpu_here=182`，量的），任务 11334 整理搬与快照共享的块那一流 177 个状态 `red_states=0`（缩域，`exhaustive=false`，量的）。回退之后的盘与快照销毁半途没有专门跑过这一判，推不出来：我替它辩的是「条数判早就拒了条数不等的，新判只多拒条数相等的」这一层包含关系，不是量的。
- GPU 一侧少判的一形见 4.3 表第六行：对拍看不到，今天没有流走到。
- 只读路径不判留下什么：读不出别的单元的字节（读码，推的）：按提示读，提示里的整单元校验和判对错，提示过期转查映射，查不到报 `MappingMiss`，查到别的单元的落点也读不出能对上这个单元校验和的字节。留下的是：换了 key 的盘可以只读挂载，写不了。用户定案是「挂载时拒」，字面没分读写；我读的是它防的是写盘把 key 盖回隔离的旧落点，只读不写，所以只读不拒。这一读法没问过用户，进交用户表。

### 4.5 数字能不能核（攻击面 6）

量的（`task-server/task log <号> --all` 读）：任务 11330 `TASK_SERVER_REASON … type=succeeded exit=0`；任务 11331 两个测试目标 `4 passed` 与 `37 passed`，日志里有 `states_judged_by_the_gpu_here=182`；任务 11332 `1 passed`；任务 11334 `states=177 closed_form=123338919 exhaustive=false violations=0`。E206 页引的：Q4 `held_down`、16 个点 P2a 最大 5 轮（`files_30000` 的 `peak_rounds=5`，`name=q1` 十六行量的）、`E*` 最大 3.520（`peak_effective_rounds` 量的）、Q5 同集 15 行、Q6 15 行 `ratio=1.0000`，页面的第 126 行与产物对得上。对不上的一处：`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT` 文档与 `transaction.rs:13773` 写「全部取样点」，实际只有第一段的 16 个点。

## 五、第四节八格

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立（树表那支是推的） | 补丁加行里 `todo!`、`unreachable!`、`assert!`、`assert_eq!`、`panic!`、`unimplemented!`、`.expect(`、`.unwrap(` 与 Undecided、Unsupported、OutsideTheFirstVersion：`grep -nE '^\+.*(…)' research/prompts/_m4-close-fixes-code-r3-diff.md` 0 命中；删行里的 `todo!`、`unreachable!`、`assert!`、`panic!`、`.expect(` 也 0 命中（量的）。删掉的树表那一支的理由我给了结构依据（实例表有 Carry，树表没有，见第二节走不到一行），没逐个核 `UserChange` 各臂，所以那一句是推的。P2a 那批「走不到的理由：0 处」同样 0 命中。 |
| 类型与名字 | 成立；两处小瑕疵 | 新名字都说得出内容：`merge_into_the_rewrite_set_walking_the_release_chain`、`rounds_of_the_last_allocation_record_tree_settling`（只读观测）、`CENTRAL_MAPPING_KEY_SET_INVARIANT` 与 `CENTRAL_MAPPING_KEY_SET_DISAGREES_WITH_THE_MAPPED_UNITS`（拒因写 I-1.11，该不变量的定义是「映射 key 与单元头相符」，换 key 正是不相符，`crates/singlefs-checker/invariants/01-块自描述.md:17`）、改名后的三个开关。旧名 0 命中（第二节名字与死链一行）。`expect` 依赖的不变量：补丁没新加 `expect`；既有的「.expect("一次用户改动发成之后现行那一版是有树的一版")」（`crates/singlefs-core/src/mounted_session.rs:2229`）前提由 `published?`（:2949）和没有树的版本在入口被拒（`mounted_session.rs:3566` 起的 `match &*current`）保住。小瑕疵：`allocator.rs:1867` 悬空的「不接的依据」与 `:1986` 的「都走它」；`RefusedNotForSpace` 名实不符的一形（归未定项）。 |
| 不兼容旧代码与旧格式 | 成立 | `grep -nE 'legacy\|old_format\|migrat' research/prompts/_m4-close-fixes-code-r3-diff.md` 0 命中（量的，退出码 1）。P2a 不留开关：闭包函数没有开关参数；补丁加行里也没有历史措辞（`grep -nE '^\+.*(此前\|原先\|起初\|最初\|后来\|改成了\|换成了\|修过\|改前\|改之前\|以前\|原为\|用户 .*定\|第[一二三]轮\|判决)'` 0 命中，量的）。 |
| 不新建判定库 | 成立 | 补丁加行里 `rocksdb`、`sqlite`、`.tsv`、`File::create`、`fs::write`、`OpenOptions`、`std::env`、`/home/` 不区分大小写 0 命中（量的）。新测试目标 `allocation_record_tree_settling_walks_the_release_chain_to_its_closure.rs` 里 `std::fs`、`std::env`、`File::` 0 命中；`crash_points_…rs` 的判定经已有的给定状态引擎，不新开存放处。 |
| 所有任务多机可用 | 成立 | 产品代码加行不读本机状态（同上一格）。四个动到的测试目标里 `std::env::var`、`/home/`、`hostname`、`HOME` 0 命中；`crash_points_…rs:450` 一处 `std::thread::available_parallelism()` 只决定切几段线程，每个持久集合独立判，判出的结局不依赖核数。 |
| 测试优先多线程 | 成立 | P2a 目标一条用例按次序做一段历史：后一次覆盖写读前一次留下的那一版，轮数是被量的次序效应（用例的文档自己写了），拆线程就不是同一段历史。`every_persisted_set_…` 按线程切（`crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs:450` 起）；GPU 目标 `--test-threads 1` 是显卡约束（任务 11331 命令行）。 |
| 带 --item 的命令与项 | 不成立 | 项里有两条文档点名的变异行表里没有（第二节最后一行，`grep -c` 量到两条旧用例各 0 行点名）；改名带的 8 行与整行替换的 17 行没有证红的任务（实现员：「开关改名那 8 行只换名字，静态核加整表复跑归门禁」），本轮我看到的四个证红任务（11220、11288、11307、11311）里没有它们；新 GPU 行只在 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=17` 下证过，不设变量的基线没人跑；`--item` 的整表复跑任务 11017 取消。该格的前半（项存在）已经不成立，后半是推不出。 |
| 新崩溃流多罩了什么 | 成立 | 豁免映射的批在验证包里只有三份测试引到（`grep -rlE 'units_exempt_from_the_mapping_replaced\|repair_of_the_exempt_units\|Relocated \{'` 命中 `crash_points_…`、`data_unit_read_…`、`unit_read_…` 三份），崩溃点重放只有 `crash_points_…` 一份引 `CRASH_POINT_REPLAY_GIVEN_STATES`，所以「头一次枚举这两类重写」站得住（头注释的话在 `crash_points_…rs:11`）；182 个状态量的。卸载之后判真盘是新判法（对照臂红在 I-2.1 一族）。GPU 新流：映射 key 被换那一形在这批之前没有任何流造出过（`replaced_mapping_key` 只在新流里），记录号下界的重建一侧同样新；变异行 7303、7304、7305 在任务 11307 抓到。 |

## 六、要主 agent 跑的命令

1. 新 GPU 流不设展开深度变量的基线（第二节 GPU 下界一行）。在仓根起，看 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`，`task-server/task log <号>` 里 `test result:` 的 `N passed; 0 failed`；`exit=0` 且 N 为 2 算过，超时或显卡不够算「默认环境下基线不可用」，这一行变异行就不能进门禁整表。
```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state 里 from_the_root_of_r 的两条新流; 范围=1 个测试目标里按名字挑的 2 条; 为什么=变异行证红只在缩小的展开深度下做过，要看不设变量的基线跑得完' task-server/task submit -- cargo test -p singlefs-verification --release --features gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_the_states_from_the_root_of_r_that_a_selected_version_whose_rebuild_alone_reaches_a_ --test-threads 1 --nocapture
```
2. 变异表静态核（第二节最后一行：看这一格能不能抓到点名了表里没有的行的文档；抓不到就是门禁缺口）。在仓根起，`exit=0` 过，`exit=1` 的 `✗` 行里有没有点名 `unit_read_…rs:2149` 或 `:2229`：
```
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables 格; 范围=1 格; 为什么=核变异表，并看它抓不抓得到测试文档点名了表里没有的行' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables /home/fy5090/code/singlefs
```
3. 没有能证实或推翻第四节 4.1 的现成命令：要一条用例（会话里弄坏现行实例表一片的一份，只发用户改动，读 `other_unit_reads_with_one_copy_failing_in_this_mount()`），归实现员写。
4. 这份报告自己的变异表前缀核对可以直接 `grep -c`（读，不用经服务）：`grep -c '同一份坏副本再读又加次数' crates/mutations.tsv` 应为 0 才说明第二节最后一行属实。

## 七、没做什么

- 没交任何任务、没编译、没跑测试；任务读数都是读主 agent 给了号的 `task-server/task log <号> --all`，没有读到的结局不写。
- 没判决：不替主 agent维持或撤销 r2 判决，只交复核结果。
- 没核 `UserChange` 各臂是否都换树表（走不到那一支的理由是推的）。
- 没验 P2a 在第二段 33 项上的行为，也没验 `files_1000_units_100` 在产品代码上的轮数（约 17 GiB，产品代码上没有用例）。
- 没逐份核「`if let Some` 静默跳过」在 14 份测试文件里是不是同一写法（`grep -l` 只数了含 `PlaceholderOfATableRecordedOnly` 的文件）。
- 没查 POSIX 读语义相关决策的全集（攒批读不到待发写）：只在 `.claude/kb` 下扫了「写后读」「read-after-write」「读到刚写」三个词。
- 没读 `crash_judge_recovery.wgsl` 全文，只读了 :846 一带。
- 没改仓里任何文件；草稿目录里只有 `check_rows.py`（读 `mutations.tsv` 第一列做前缀比，只读）与 `caught.txt`（四个证红任务日志里「抓到」行的名字）。没建仓副本。

## 八、cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m4-close-fixes-code-r3-sonnet-output.md` 的末行（交回前跑的，退出码 0）：

  ✓ 核了 40 处引文、45 处裸行号，对上 40 处，没判 45 处（其中裸行号没有引文可比 36 处；逐处列在上面）
