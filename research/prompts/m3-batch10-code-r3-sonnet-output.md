# 根环实现第 10 批代码轮第三轮：辩方腿（sonnet）报告（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 K4 R1 Q1 Q2 Q3 Q4 rr2 -->

轮名 m3-batch10-code-r3。格 K1、K3、K4，立场：复核第二轮判决 `research/prompts/m3-batch10-code-r2-main-verification.md`，替被判出局的一方（落点公式归盘、整份换读数、整份换但丢根就拒）找站得住的理由。代码行号一律引冻结副本 `/tmp/claude-1000/m3-batch10-code-r3/tree/`；kb 行号是 kb 文件自己的行号，现用 `grep -nF` 取过。副本上的数（`m3-batch10-code-r3-sonnet-model/`）不进 kb，要引的由主 agent 在入库装置上重做。

## 一、判定一览

| 项 | 判定 |
|---|---|
| 第二轮 K2-1（落点公式归盘） | 站住；公式还留在卸载那一格，那一格归攻方 K2 |
| 第二轮 K2-2（影子账）与 R1 三个候选 | 站住；`第二轮判决没量第三个候选`，这一轮在同一组历史上补量了，并出来的那一份在这五段上 0 拒 0 红，另两个臂不是 |
| 第二轮 K2-7（丢盘整读两个空表） | 站住，第 10h 批的补法够得着 |
| K1 五问 | 都没打中；一处「取重读」的前提是故障集里没有「读到旧内容」 |
| K3 | 一处部分打中：入库断言只在起点钉了方向，之后只比不变量名；实测每个状态都在记账多的一边，缺的是断言强度；另报红的大小涨到起点的 7 倍，kb 与用例都没写这个数 |
| K4 | 数都能从代码或变异行推到；`[9, 7, 10, 8]` 不是入库断言钉的值；第二轮判决里两条去向没有载体（E158 重读计数、第 10 批收拢表），C724 的行号过期 |

## 二、第二轮判决条目复核

### 2.1　K2-1：触发与取 m 按落点公式归盘（判「打中」）

| 问 | 结果 |
|---|---|
| 引的原文现查在不在 | mkfs 对每个第 0 代根槽按区域所在的盘写：「for ring_slot in genesis_root_ring_slots() {」（`crates/singlefs-core/src/make_filesystem.rs:619`）；第 0 代根槽是每个区域的槽 0：「RootRingSlot { region, slot: 0 }」（`crates/singlefs-core/src/make_filesystem.rs:672`）；落点公式把 txg 0 落到区域 0：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:114`）。判决引的 `make_filesystem.rs:619`–`627`、`root_ring.rs:109` 对得上（`target_for_publish` 的签名在 109 行，函数体在 113 行起） |
| 字面满足的是哪一个分句 | 满足的是「同一处定义 `newest_valid_root_txg_on_every_device` 按公式把根归盘」那一分句；公式与盘面只在 mkfs 区域 1、2 的两份第 0 代根上不同。别的写根处都经 `target_for_publish` 取槽，例如「let target = target_for_publish(」（`crates/singlefs-core/src/transaction.rs:450`）；core 里按槽直接写根的只有 mkfs 那一处 |
| 同一形状在别的臂上成立吗 | 公式归盘只剩一处：卸载那一格「device_of_txg(root.checkpoint_txg)」（`crates/singlefs-core/src/mount.rs:3341`）。`recovery.rs` 里 F_生效 根那一半注释写「按根环落点公式」，代码走的是 `readable_roots_with_ring_slots`（带槽，「let roots_with_ring_slots = readable_roots_with_ring_slots(」，`crates/singlefs-core/src/recovery.rs:1567`），按槽，不受影响。卸载那一格是否有一形只剩第 0 代根，归攻方 K2，我只读代码：实现的注释写「卸载接在一次可写挂载之后，暖机已在每块盘上落了根」（`crates/singlefs-core/src/mount.rs:2733`），推的，我没造盘面 |
| 第 10h 批改完还中吗 | 不中。触发与取 m 共用「Some((device_of_the_ring_slot(*ring_slot), *root))」（`crates/singlefs-core/src/mount.rs:2005`），闭包在 `mount.rs:1992`。我在副本上跑变异行 `crates/mutations.tsv:2864`（归盘改回落点公式）：`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable` 8 条里 7 过 1 红，红的就是 `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread`；该用例钉的值已改回「root_ring_of_the_rebuilt_allocator: RootRingOfTheRebuiltAllocatorRead::OnTheFirstRead,」（`crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:522`） |
| 什么会推翻 | 造出一条非第 0 代的根住在公式算不到的槽里（现有写根处没有）；或一块盘在第 0 代之后只剩区域 1、2 的第 0 代根而不重读是错的（那时按槽归盘把「盘上有根」报成真，与「盘上读出了有效根」同义，没有错） |

### 2.2　K2-2：重读比第一遍坏时影子账丢根；R1 三个候选

判决的打中靠攻方第二次抽样「rr2 臂 12 / 12 段红，按槽并两臂 0 / 12」。「同样打中所有替代方案」这一问，判决的 R1 够判条件（「各量过 checker 红、被拒次数、隔离槽数」，`research/prompts/m3-batch10-code-r2-forks.md:9`）要三个候选各量一遍，判决里量的是「rr2 之前 / rr2 / 按槽并」三臂，第三个候选「整份换但丢根就拒」没有量。我在冻结副本的拷贝上补了。同一份入库用例 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs` 里 K2-2 那五段（三段重读只坏 C 的槽、一段反方向、一段重读整环都坏），三个臂各跑一遍：

| 历史 | 臂 W：整份换成重读那一遍 | 臂 X：整份换，第一遍自证过的根重读时不是自证过就拒 | 入库的按槽并 |
|---|---|---|---|
| 重读只坏 C 的槽，报读错 | Ok，隔离 false，池级 checker 红 I-7.4 | 拒（Q3 那个拒因） | Ok，隔离，checker 0 违例 |
| 同上，读回全零 | 同 W 第一行 | 拒 | Ok |
| 同上，读回乱字节 | 同 W 第一行 | 拒 | Ok |
| 反方向：第一遍 C 的槽读坏、重读读得出 | Ok | Ok | Ok |
| 重读整环都坏、第一遍读出盘 0 的根 | 拒（Q3） | 拒（Q3） | Ok |
| 合计 | 3 红、1 拒 | 4 拒、0 红 | 0 红、0 拒 |

原样输出在 `m3-batch10-code-r3-sonnet-model/run-arm-w-whole-swap.txt`、`run-arm-w-left-lines.txt`（W 三段 left 里的「I-7.4: 被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头」）、`run-arm-x-refuses-when-first-pass-root-lost.txt`（X：「test result: FAILED. 11 passed; 4 failed」）、`runM-full-merge-baseline.log`（并：三个二进制 8 / 15 / 6 全过）。臂 X 跑了整个二进制（15 条）：11 过 4 红，另外十条（192、216、264 槽起点）与反方向一条都过；臂 W 只跑了这五条，没跑另外十条。

- 判决「打中 K2-2」够得着：它打中的是臂 W，臂 W 在这三段上红 I-7.4，我独立复现。
- 同样打中别的臂：不是。臂 X 不红 checker，打中它的是另一件事（拒得多：同一组历史上 4 / 5 拒，并出来的那一份 0 / 5）；「rr2 之前」也不红 K2-2 那三段，但判决自己量过反方向 4 / 4 红（「before: n=4 red=4 iso0=4 refused=0」，`research/prompts/m3-batch10-code-r2-main-verification.md:14`）。没有第二个臂在两个方向上都是 0 红 0 拒。
- 臂 X 站得住的理由：它从不丢根，代价是读错一个槽就拒可写挂载。这是用户 2026-10-05 在三个候选里没选的那一个，现在有数了：4 / 5。副本上的数，不进 kb。
- 什么会推翻：入库装置上同一组历史里并出来的那一份有一段红或拒；或有一个臂在这五段上 0 红 0 拒且不丢「第一遍自证过」的根。

### 2.3　K2-7：rr2 用例「丢盘整读」比的是两个空表（判「打中」）

- 引的原文：判决说两个起点（192、216 槽）丢任一块盘后都没有活文件。第 10h 批没有删它们，另加第三个起点 `264:6:4`（有活文件，可写挂载做完就崩）。`assert_the_live_file_starting_point_has_live_files_to_compare` 在跑用例之前断言丢任一块盘后「compared > 0 && compared_with_live_files > 0」（`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:971`）；接着覆盖写之后同一断言在「compared > 0 && compared_with_live_files > 0,」（同文件 `:1045`）。比的内容是挂载之前环里还在、不低于 F 的每条自证根丢盘整读，不只是挂载择到的那一版。
- 够得着：是。判决要的是「换一个丢盘后有活文件的起点」，实现是加，不是换；老的 8 条（192、216 槽四种读坏计划各两个起点）仍比两个空表，新的两条只跑读错（`DEVICE_ZERO_REPORTS_READ_ERRORS`、`DEVICE_ONE_REPORTS_READ_ERRORS`）一种计划，读回全零、乱字节两种计划在有活文件的起点上没跑。
- 同样打中别的臂：这件事只打「用例」，不打实现的任何一臂。
- 判别力现查：变异行「第10h批 rr2 用例丢盘整读：重读之后的读数只剩最新那条根…」（`crates/mutations.tsv:2873`）我在副本上单行复跑，`on_the_264_slot_pool_with_a_live_file_device_zero_…` 红在 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:1048`，left 是 `[9, 7, 10, 8]`、right 是空（`m3-batch10-code-r3-sonnet-model/runM2873-mutation-newest-root-only.log`）。`[9, 7, 10, 8]` 是变异下失败信息里的读数，入库断言钉的是空表（见 4.2）。
- 什么会推翻：哪个读坏计划在有活文件的起点上让回收门槛越过环里更旧的有效根，而入库用例不红（读回全零、乱字节两种计划还没在 264 槽起点上跑，这是个开着的缺口，不是反例）。

## 三、K1：按槽并两遍（B）

只读代码与复跑入库用例，两遍读数「各读出一条不同的自证根」这一形在 harness 故障集里造不出（见 3.1），所以这一格没有反例，判定都按「没打中」记，一次抽样。

- 3.1　两遍各读出一条不同的自证根，取重读那一条对不对：站住，带前提。取重读那一条写在「(SelfVerified(_), SelfVerified(root_of_the_reread)) => SelfVerified(root_of_the_reread),」（`crates/singlefs-core/src/mount.rs:1785`），变异行「第10h批 R1 两遍都自证过取第一遍那一条」（`crates/mutations.tsv:2870`）钉它。同一槽两遍读出两条不同的自证根，要么两遍之间有写者，要么故障是「读到旧内容」。实现自己写明前提：「挂载期间又没有别的写者」（`crates/singlefs-core/src/mount.rs:1772`）；harness 注入的读故障只有三种，读错、读回全零、读回乱字节（`ReadsBackGarbage,` 在 `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:104`），没有「读到旧内容」。推翻它的现象：故障模型加上「读到旧内容」，并且旧内容出现在重读那一遍。到那时取 txg 大的那一条在两种次序下都不比取重读差（推的，没量），与用户 R1 的字面不同，记零轮。
- 3.2　一遍读坏不是全零、另一遍全零取读坏，是不是总在安全方向：站住。两类的差别只在准入抬 F 的上限：读坏并进上限（`txgs_ring_slots_bad_at_mount_can_hold(` 的第一次调用，`crates/singlefs-core/src/mount.rs:2101`），读回全零只取可能住过的最旧 txg（「(ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held())」，`crates/singlefs-core/src/mount.rs:2116`）；两类在回收门槛里都钉在最旧 txg。kb 的口径：D16 已定项 1 第 36 行写读回全零的槽不并进上限（`.claude/kb/decisions/16-发布语义.md:36`：「读回全零的槽不并进上限」），读坏槽「卡住就在这次挂载里报 ENOSPC」（同行）。取读坏在回收门槛上与取全零同样，在上限上约束更多，方向是更保守的一边；代价是 Q1 已认的那一种。反过来全零压过读坏（变异行「第10h批 R1 并时全零压过读坏」，`crates/mutations.tsv:2871`）会让一个读坏过的槽退出上限，是「F 越过住在槽里的有效根」那一类，打中的是替代方案。真从没写过的槽被重读的一次瞬时读错并成读坏：这只发生在能住一个 txg 的槽上，年轻的环里没轮到的槽不交回（「就不交回」，`crates/singlefs-core/src/mount.rs:1840`）。推翻它的现象：一个合法盘面上，读坏并进上限使池卡死、而用户 Q1 没认的那种。
- 3.3　并出来的读坏集与「挂载那一刻读坏」是不是同一个分类：站住。生产代码里 `RootRingOccupant::BadAtMount(` 与 `AllZeroAtMount {` 只在两处造：「(*ring_slot, RootRingOccupant::BadAtMount(*txgs_it_can_hold))」（`crates/singlefs-core/src/mount.rs:2194`）与 `mount.rs:2200`，都从 `ring_slot_readings` 派生，两个集合「let bad_ring_slots_not_all_zero: BTreeSet<RootRingSlot> = ring_slot_readings」（`crates/singlefs-core/src/mount.rs:2075`）、`mount.rs:2083` 也是。重读分支把 `ring_slot_readings` 换成并出来的那一份（`mount.rs:2037`、`2047`），没有第二处分类。`allocator.rs` 里的 1074、1092、1129 等行只是读那个枚举。推翻它的现象：另有一处在并之前分类。
- 3.4　Q3 改成两遍都没有才拒之后，有没有本该拒却挂上、挂上之后 checker 红：没打中。拒只在并出来的那一份里没有一条不被抛弃的自证根（「let Some(window_anchor) =」，`crates/singlefs-core/src/mount.rs:2051`；它的 else 分支返回拒因）。入库的五条（`rebuilding_…one_device_reads_no_valid_root.rs` 里 K2-2 三条、反方向一条、整环都坏一条）我在副本复跑：8 / 15 / 6 个测试全过（`m3-batch10-code-r3-sonnet-model/runM-full-merge-baseline.log`），其中重读整环都坏、第一遍有盘 0 的根那一段：挂上、盘面与「两遍都只坏盘 1」相同、checker 0 违例。两遍都没有的两条旧用例（`rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs`）我没在副本复跑，实现员报 6 过、合入验证 170 个二进制全绿（「harness 档 170 个测试二进制全绿」，`records/2026-09-30-里程碑三开工第一轮.md:679`）。没造的形：第一遍有根、重读整环坏而第一遍那条根是被抛弃的：并出来没有不被抛弃的根，仍拒，与旧行为相同。
- 3.5　影子账、窗口锚、根环表、回收门槛是不是用同一份：站住。窗口锚取并出来的那一份（`mount.rs:2051` 起），后面的 `roots`、两个读坏集、`floor_released_immediately` 的 m（「&roots_on_the_devices_they_were_read_from(&ring_slot_readings),」）都读 `ring_slot_readings`；影子账收 `&roots`。

## 四、K3：改过期望的用例（C、D）

- 4.1　rollback 三条慢档钉住的是不是用户认的代价、只红记账多的一边：起点站住。助手函数断言只许一条违例，「vec!["I-3.1"],」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:196`）、方向「allocated > walked,」（同文件 `:203`），差值「allocated - walked,」（同文件 `:208`）与「819_200,」（同文件 `:1760`）、「917_504,」（同文件 `:2106`）。用户认的口径在 kb 里是方向加时长：「到这个进程写掉那一槽为止」（`.claude/kb/invariants.md:143`），记录里写「只许红在记账多的一边」（`records/2026-09-30-里程碑三开工第一轮.md:624`）。我在冻结副本上复跑四条慢档：「test result: ok. 4 passed; 0 failed」（`m3-batch10-code-r3-sonnet-model/run-rollback-4-slow.txt`）。变异行 `crates/mutations.tsv:2877`（撤掉读坏槽在回收门槛里的钉住）单行复跑：torn 那条红在差值那一项，left 688128、right 819200（`m3-batch10-code-r3-sonnet-model/runM2877-mutation-torn.log`）。注意撤掉挂载时那一处钉住之后差值不是 0 而是 688128：819200 里有一部分来自进程内记根那一处的钉住（实现员报告也这么写），两处同属 D16 已定项 1 第 37 行写的「回收门槛三处」钉最旧 txg，数能分开，但「钉的是挂载时那一处」这句话只对其中 131072。
- 4.2　有没有一形红在记账少的一边被这次改期望一起放过：部分打中，打的是断言强度，不是行为。
  - 现状：「写到 txg 51 之后」那一次检查只比不变量名，「vec!["I-3.1"],」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:245`），不比方向；之前的 19 次覆盖写之间（txg 32 到 50）一次都没查；函数注释写的是「写到它前一次为止 I-3.1 仍只红在记账多的一边」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:216` 起），代码没做到这么强。起点那一次有方向与差值，之后没有。
  - 我在副本里补了每个状态的探针（`probe-rollback-direction.diff`：每写一次就查一遍只许 I-3.1 红、且 allocated > walked）：三条慢档各 20 个状态（txg 32 到 51）共 60 个，全部通过。差值在起点之后不再是 819200 / 917504：torn、zeroed 从 1048576（txg 32）涨到 5799936（txg 51），plain 从 1146880 涨到 5898240（`m3-batch10-code-r3-sonnet-model/probe-direction-per-state.txt`）。所以没有红在记账少一边的实例，现状的缺口只是入库断言没拦它。
  - 红的大小不封顶在起点值：到 txg 51 是起点的 7.1 倍（torn、zeroed）与 6.4 倍（plain）。用户认的口径（方向加时长）没有说大小；kb 与用例里都没有写大小的上界。若主 agent 认为用户认的「代价」包括大小，这里没有钉。
  - 什么会推翻：第 10h 批之后某次改动让这三条慢档在 txg 29 到 50 之间某一步红在记账少的一边，而入库用例仍全绿（我的探针在现树上不红，所以现在不会）。
- 4.3　Q3 判官那半句「按表判被抛弃」判等价变异的论证（最新那条根自己的实例表里没有它那个实例的行）在全部合法盘面上成立吗：站住，推的，没造反例。写行的范围不含要取的号：「(first_row_instance..instance_to_acquire.0)」（`crates/singlefs-core/src/mount.rs:3977`），只有一处调用（「let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);」，`crates/singlefs-core/src/mount.rs:4727`）。带这张表的根：写行那次发布「instance_table: instance_table_pointer,」（`crates/singlefs-core/src/transaction.rs:2001`）、零单元发布与别的发布抄上一版「instance_table: previous_root.instance_table,」（`crates/singlefs-core/src/transaction.rs:1483`）、由记录重建的根「instance_table: record.new_instance_table,」（`crates/singlefs-core/src/recovery.rs:3669`，那张表是那次发布带的新实例表，按写行的范围本应只有早于那次发布所在实例的行，这一路我没逐行核）。`InstanceRow {` 在 core 的其余构造点都在 `#[cfg(test)]` 之后（`instance_table.rs:303`、`instance_table_sweep.rs:860` 起，构造在 322、469、878）。被抛弃的谓词是「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1540`）。留档用例钉前提「.all(|row| row.instance < newest.instance)」（`crates/singlefs-harness/tests/writable_mount_refusal_for_no_valid_root_in_the_rebuilt_allocators_root_ring.rs:267`）。没验的：清扫发布改写实例表的那一路（只会删行，不会加）我只读到注释没逐行读；`.any` 改 `.all` 那一行变异（`crates/mutations.tsv:2874`）我只核了原文在冻结副本里恰好出现一次，没复跑。推翻它的现象：哪条路径写出一条实例为 I、指着的表里有 I 的行的根（写行之后同一实例又写行是最近的一形）。
- 4.4　丢盘整读新起点罩住的是不是条文要的那件事：见 2.3，罩住了「挂载之前环里更旧的有效根不被回收门槛越过」；条文口径是 D16 已定项 1 第 37 行的 rr2 那一句：「按读出的那一槽所在区域归盘」（`.claude/kb/decisions/16-发布语义.md:37`）加不丢根。缺口：有活文件的起点在读错、读回全零、读回乱字节三种读坏里只跑了读错一种。

## 五、K4：数能不能核与清单全不全

### 5.1　第 10h 批报告里的数（逐个）

- 14（追加的变异行）：`grep -c '^第10h批' crates/mutations.tsv` 在冻结副本里输出 14（第 2864 到 2877 行）；四行整行替换的原文加这 14 行，一共 18 个原文在冻结副本里各恰好出现一次（`m3-batch10-code-r3-sonnet-model/check-mutation-anchors.py` 输出 18 行都是「1」）。判决 K5 里的「14 / 12」是第 10g 批那一对数：`research/prompts/m3-batch10-code-r2-main-verification.md` 把它写在两行变异红的读数上；它不在第 10h 批报告里，12 对得上 `shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs` 的「12 槽小环」（`crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs:7`），14 在那份测试源码里 0 次出现，是变异下失败信息里的读数。这一对我没复现（不是第 10h 批的数）。
- 819200、917504：钉在断言里（`rollback_by_a_forward_publish.rs:1760`、`:2106`），基线复跑 4 过；变异行 2877（torn）复跑红在差值（见 4.1），变异行 2875、2876（plain、zeroed）我没复跑，只核了原文恰好出现一次。差值本身是慢档调查员的读数（调度记录 `records/2026-09-30-里程碑三开工第一轮.md:663` 转述），从代码里推不出，只能跑出来。
- 52（写掉钉住那一槽的 txg）：钉在「root_slot_position_of(28),」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:225`），即钉住的槽是 txg 28 的槽，24 槽的环（S = 8，三个区域）一圈之后是 52；基线复跑里写到 52 之后全绿。
- `[9, 7, 10, 8]`：不是入库断言钉的值。入库断言钉的是空表（「assert_eq!(」后面的 `Vec::<u64>::new()`，`crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs:1050`），`[9, 7, 10, 8]` 只在变异行 `crates/mutations.tsv:2873` 下的失败信息里出现。我在副本上复跑出同一个数（`m3-batch10-code-r3-sonnet-model/runM2873-mutation-newest-root-only.log`：「left: [9, 7, 10, 8]」「right: []」）。它是丢盘 0 之后读得不一样的根的 txg，没有代码段「推出」它，取决于 264 槽起点的盘面。
- 九种组合穷举：`better_reading_of_a_root_ring_slot_over_the_two_passes` 的 match 六个分支盖九个有序对（1 + 2 + 2 + 1 + 2 + 1），不写 `_ =>`（`crates/singlefs-core/src/mount.rs:1777` 起）；单测把九对逐个列出（`mount.rs:7124` 起的 `each_pair_of_readings_merges_into_the_self_verified_root_then_the_bad_reading_then_all_zero`）。我在副本里复跑单测 2 过，并对三行变异单行复跑：2870（两遍都自证过取第一遍）红在「两遍都自证过：取重读那一条」；2871（并时全零压过读坏）红在「第一遍读坏、重读全零：读坏压过全零」，另一条单测也红；2872（逐槽并时只取第一遍）红在「逐槽取较好的那一份，次序与槽不变」（`m3-batch10-code-r3-sonnet-model/runM2870-2872-core-lib-mutations.log`）。两遍都读坏、两遍都全零那两对分支没有专门的变异行，只有单测钉，改它们改的是返回值而不是判别，单测红。
- 15（`rebuilding_…one_device_reads_no_valid_root.rs` 的用例数）、8、6、6：`grep -c '^#\[test\]'` 在冻结副本输出 15、`a_mount_…` 8、`…no_valid_root_is_read.rs` 6、`writable_mount_refusal_…` 6，与实现员报告的「15 / 8 / 6 / 6 passed」对得上；前三个二进制我复跑 8 / 15 / 6 过。

### 5.2　第二轮判决第三节、第五节、K6 每一条的去向

| 判决条目 | 去向 | 载体 | 判定 |
|---|---|---|---|
| §三 按槽归盘 | 第 10h 批实现 | 「let device_of_the_ring_slot」（`crates/singlefs-core/src/mount.rs:1992`） | 有 |
| §三 `…does_not_reread` 改回 `OnTheFirstRead` | 第 10h 批 | `crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs:522` | 有 |
| §三 丢盘整读换有活文件的起点 | 第 10h 批 | 264 槽两条（`rebuilding_…one_device_reads_no_valid_root.rs:1064`、`:1083`） | 有（加不是换，见 2.3） |
| §三 Q3 判官补变异或留档 | 第 10h 批 | 留档用例（`writable_mount_refusal_…root_ring.rs:243`）与变异行 `crates/mutations.tsv:2874` | 有 |
| §三 R1 重读读数取法 | 用户已定，第 10h 批 | `mount.rs:1777`、`mount.rs:1802` | 有 |
| §三 rollback 三条慢档 | 第 10h 批 | 4.1、4.2 | 有（断言强度见 4.2） |
| §五 准入抬 F 上限里同名拒因 | 欠账 C724 | `.claude/kb/checks-owed.md:577` | 有，行号过期：行里写「`mount.rs` 2463、2506 行」，冻结副本里那两处（「.ok_or(RecoveryFailure::NoValidRoot)?;」）现在是 `crates/singlefs-core/src/mount.rs:2543` 与 `:2586`（第 10h 批加了 80 行；C718 写的「1432 行一带」也移到 1436 与 1442） |
| §五 重读读回全零的槽进上限的新进入法，随 R1 | 按槽并之后这一形没有 | 并的第一遍自证过的根压过重读全零（`crates/mutations.tsv:2867`），用例「the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_…」（`rebuilding_…one_device_reads_no_valid_root.rs:1530`） | 有 |
| §五 用例名与钉值相反 | 钉值改回 `OnTheFirstRead` 之后一致 | 同上第二行 | 有 |
| §二 K6 rr2 与 Q3 随第 10 批 kb 写回 | kb 写回第二份 | D16 已定项 1 第 37 行「按读出的那一槽所在区域归盘」（`.claude/kb/decisions/16-发布语义.md:37`）；D23 已定项 14 第 392 行「重建分配器读根环两遍并起来一条不被抛弃的自证有效根都没有」（`.claude/kb/decisions/23-journal的角色与格式.md:392`） | 有 |
| §二 K6 E158 重读计数「归提交时的实验复跑」 | 判决里写「提交时」 | `grep -c rr2` 在 `.claude/kb/experiments/158-择根与修复四岔路.md`、`171-三oracle合谋测试.md`、`.claude/kb/checks-owed.md` 都是 0；调度记录里 `grep -n E158` 只命中正推腿报缺口的那一条（`records/2026-09-30-里程碑三开工第一轮.md:662`），没有一行把它列成提交时要做的事 | **没有载体**：没有欠账行，没有一行列成提交时要做的清单 |
| §二 K6 第 10 批收拢表「在第 10 批 kb 写回那一批的调度记录里建」 | kb 写回第二份（`records/2026-09-30-里程碑三开工第一轮.md:675`、`:676`）已落地 | `grep -n 收拢表 records/2026-09-30-里程碑三开工第一轮.md` 命中 147、188、212、428、429、448、538、662 共 8 行，都是第 4 到第 9 批或第二轮复述，没有第 10 批的表 | **没有建**；第一轮判决 K7 里「5 行真无去向、5 行只写进下一轮清单」靠这张表兜底，其中 `DeletedRowsStillPresent` 无用例无变异现在仍是：`grep -c DeletedRowsStillPresent crates/mutations.tsv` 输出 0，`crates/*/tests` 里 0 份，只在 `crates/singlefs-checker-tier/src/crash_injection.rs` 里出现 |

### 5.3　K2-2 那组历史缩了没有

R1 定案写「补 K2-2 那 12 段与反方向 4 段的用例」（`research/prompts/m3-batch10-code-r2-main-verification.md:107`，原文「补 K2-2 那 12 段与反方向 4 段的用例」）。第 10h 批做的是 384 槽、前置覆盖写 1 次那一格：K2-2 三段（读错、读回全零、读回乱字节）、反方向一段，另一条整环都坏。实现员报告的「没做什么」里写了没全搬（正文第一节给的实现员报告，不在仓里；那一条的题目是「攻方原型的整张格」），调度记录第 10h 批交回那一条（`records/2026-09-30-里程碑三开工第一轮.md:677`）只写了「K2-2 三条、反方向一条」，没写 12 缩成 3、4 缩成 1。副本上的观察：被丢的维度是盘宽（4 GiB 与 384 槽）与前置覆盖写次数（1、3）；我没造这两个维度，不知道它们会不会分开三个臂。什么会推翻「缩了也够」：4 GiB 盘或前置覆盖写 3 次的任一格上，入库那一份按槽并红而攻方原型的并不红。

## 六、跑过的命令与原样输出

所有命令在冻结副本的拷贝里跑（`nice -n 19`、`capped.sh 15`、`run-with-memory-cap.sh 8G`；开跑前 `ps` 看过：另有一条别的会话的 `cargo test`，没有性能测量在跑；中途另一条腿占着内存包装的日常槽，退出码 252 那一次没跑，加了「派出去的辩方与攻方两条腿各跑各的，同一会话并行」的理由重起，那一次的输出不算结果、之后的才算）。

```
$ grep -c '^第10h批' crates/mutations.tsv
14
$ python3 check-mutation-anchors.py | awk '{print $2}' | sort | uniq -c
     18 1
```

入库那一版（按槽并）三个二进制的基线（`runM-full-merge-baseline.log`）：

```
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.78s
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.20s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.91s
```

三个臂在 K2-2 五段上（`run-arm-w-whole-swap.txt`、`run-arm-x-refuses-when-first-pass-root-lost.txt`）：

```
臂 W（只跑五条）：test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 10 filtered out; finished in 1.50s
臂 X（整个二进制）：test result: FAILED. 11 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.18s
并（入库）：15 passed; 0 failed
```

四条慢档（`run-rollback-4-slow.txt`，带每个状态的探针）：

```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 46.46s
```

变异单行复跑（`runM2877-…`、`runM2873-…`、`runM2864-…`、`runM2870-2872-…`）：2877 红在差值 left 688128、right 819200；2873 红在 `[9, 7, 10, 8]` 对空表；2864 只红 `the_first_writable_mount_of_a_formatted_pool_has_nothing_witnessed_and_does_not_reread`（「test result: FAILED. 7 passed; 1 failed」）；2870、2871、2872 各红自己那条（见 5.1）。

## 七、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 副本上的数（三个臂的拒与红、60 个状态的方向与差值、变异单行复跑）都不是入库装置上的新一次跑；要引进判决，由主 agent 在入库装置上重做（臂 X 是我自己写的副本臂，不是入库形态）。
- 没复跑：`rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs` 的 6 条、变异行 2874、2875、2876、2865 到 2869（只核了原文在冻结副本里恰好出现一次）、判决 K5 里第 10g 批的「14 / 12」。
- 没造盘面的：卸载那一格只剩第 0 代根、清扫发布改写实例表那一路、「读到旧内容」的故障、有活文件的起点上读回全零与乱字节两种计划、4 GiB 盘与前置覆盖写 3 次那些格；都归攻方 K2 或下一批，不是我的格。
- 「没打中」只抽了一次样（K1 五问）；按 `.claude/rules/three-way-inference.md` 再抽一次才算「没打中」。
- 没做 git 写操作、没改 `crates/`、没跑重型测试与门禁；Rust 只在副本编，只跑 harness 档与 core 的 `--lib`。

## 八、要主 agent 跑的命令

没有必须跑的。想把副本上的数引进判决，要在入库装置上重做：K2-2 五段的臂 X 拒数（副本补丁 `m3-batch10-code-r3-sonnet-model/arm-x-w-mount.diff`，环境变量 `ARM_X_REFUSES=1`）、`rollback_by_a_forward_publish` 三条慢档每个状态的方向（补丁 `m3-batch10-code-r3-sonnet-model/probe-rollback-direction.diff`）。两处都是把补丁里的改动并进冻结副本的拷贝（补丁是在 `/tmp/claude-1000` 下 `diff -u` 出来的）后经 `run-with-memory-cap.sh` 跑，输出与 `runX-full.log`、`run1-full.log` 对。

## 九、副本与草稿

- 模型目录：`research/prompts/m3-batch10-code-r3-sonnet-model/`（补丁、日志、`mut.py`、`check-mutation-anchors.py`、`README.md`）。
- 草稿目录 `/tmp/claude-1000/m3-batch10-code-r3-sonnet/`：留 `chk.py`、`mut.py`、各 `*.log`；编译用的仓副本 `work/`（3.0G，带 `target/`）在交回前删掉，需要时从冻结副本重拷、再并进上面两份补丁。
