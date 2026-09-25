# 实二六 交回：抬 F 那一串整串预演、Z15 两个测试缺口、分配记录树根层的绝对槽数收成一个函数

实现员（implementation-writer），2026-09-25（时刻均为 UTC；JST = UTC + 9）。规格 `/tmp/claude-1000/impl-m2-e26/spec.md`（sha256 82f45c3c…0ae0c3，开工时核过）。
改动直接写在主工作区 `crates/` 里（开工快照 `/tmp/claude-1000/impl-m2-e26/pristine/`，01:23Z 拷）。没有另开补丁，`git apply --check` 那一步没有对象：主工作区现状就是结果。
开工到收工，`crates/` 里别的会话动过的只有 `e156_allocation_basis_counts.rs` 与 `crates/mutations.tsv` 末尾两行（E156 M33 / M34），我没碰。

## 一、三件各到哪

| # | 件 | 结局 | 用例 |
|---|---|---|---|
| 1 | 抬 F 第 n 次（n ≥ 2）被拒也换回分配器、放开扣住的槽 | 做了，做法是**这一串在任何写之前在分配器拷贝上整串预演**（第六节 Q1 说为什么不能「落了盘之后再换回」）：第几次撞墙都在预演里拒、一次都不发、分配器整个换回抬 F 之前那一份。新错误成员 `MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite` | 两个种子各一条（随机历史二进制）；第 1、2、3 次被拒各一条（回落二进制，第 1 次那条改断言）；另加一条落盘途中写错（第 2 次写报错、已落盘 1 份） |
| 2 | Z15 两个测试缺口 | 补了三条用例，实现没改；① 在「提示过期」那个取样点上自报数与块层数**不相等**、差每跳 2 次（第六节 Q4） | 中央映射二进制三条 |
| 3 | 分配记录树根层的绝对槽数 | 选「收成一个函数」（第六节 Q3 写为什么）：盘上绝对槽数只有 `allocator::absolute_slot_count_of_device` 一处定义，写侧 `of_allocator`、读侧 `of_reader` 都走 `AllocationRecordTreeGeometry::of_device_sizes_in_bytes`；造了字节数不是 16384 整数倍、正压在根层级分界上的盘，不分叉，没加挂载成员 | 新测试二进制一条；core 单测一条 |

关键落点（行号是现在的文件）：
- `crates/singlefs-core/src/mount.rs`：新成员 79；`raise_rollback_floor` 967，预演在 1068（回收之后、`PoolWriter` 开了但一个写都没发），预演拒了换回分配器 1077；真发之后「预演取的落点 = 真发取的」断言跟在循环后面（同 `establish_instance` 那一条，有一份被隔离时不比）；`txgs_of_the_publishes_carrying_the_floor_to_every_device` 1156（原来循环里的「推到每块盘」条件挪进来，预演与真发共用）；`empty_publish_plan_raising_the_floor` 1180；`rehearse_the_publishes_raising_the_floor` 1215；`placements_taken_by_a_file_version` 1524（从 `placements_taken_by` 抽出来）。
- `crates/singlefs-core/src/allocator.rs`：`absolute_slot_count_of_device` 227（`unit_area_slots_of_device` 改成由它算）；`DeviceFreeMap` 多一个字段 `device_size_in_bytes` 193 与访问器 294。
- `crates/singlefs-core/src/allocation_record_tree.rs`：`of_device_sizes_in_bytes` 119，`of_allocator` 129、`of_reader` 143 都委托过去；单测 924。
- harness：`history.rs` 与 `model_comparison.rs` 的成员映射、`first_transaction_on_device.rs` 两处穷举列表各加新成员；`model.rs` 抬 F 的答案改成「墙在第一次写之前整串判」（1296，同可写挂载、回退）；`on_device_modes.rs` 只改文档注释。

## 二、这一轮写过的文件（自己列；与开工快照 `diff -rq` 逐个对过，差出来的另两个不是我的）

- core：`crates/singlefs-core/src/mount.rs`、`allocator.rs`、`allocation_record_tree.rs`
- harness src：`crates/singlefs-harness/src/history.rs`、`model.rs`、`model_comparison.rs`、`on_device_modes.rs`、`bin/first_transaction_on_device.rs`
- harness tests：`tests/second_transaction_allocation_record_tree_geometry_of_writer_and_reader.rs`（新建）、`tests/second_transaction_supplement_two_commit_generated_fallback.rs`、`tests/second_transaction_supplement_three_random_history.rs`、`tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs`
- `crates/mutations.tsv`：只在末尾追加（第三节）
- 没碰：`e158_root_choice_repair.rs`、`e156_allocation_basis_counts.rs`、`first_transaction_regions.rs`、`tests/first_transaction_region_bytes.rs`、`crates/singlefs-checker/`、`crates/singlefs-format/`、`litmus/`、名字含 layer0 的文件；`e142_first_transaction_write_dump.rs` 没建。
- 手工编辑之外的写法：全部经 Edit / Write；新建的测试文件里一处用了 `replace_all`（自己新建的文件）。`crates/mutations.tsv` 的追加用 Bash `>>`（第三节）。

## 三、`crates/mutations.tsv` 追加了哪些行（第 696–709 行，14 行；原稿 `/tmp/claude-1000/impl-m2-e26/mutations-append.tsv`，生成脚本 `scripts/make-mutation-rows.py`，逐行核过原文在主工作区恰好命中一次）

变异名都以「实二六（代码三方 m2-final-code-r3 判决…）：」开头，下面只写冒号后面那一段：

| 表行 | 变异 | 点名的测试 |
|---|---|---|
| 696 | 抬 F 那一串预演报错照样往下发（改回逐次发）；越格线索种子 4000000045 | `seed_4000000045_…`（随机历史二进制） |
| 697 | 同上；种子 4000000204 | `seed_4000000204_…` |
| 698 | 同上；第二次取不到落点那一格 | `a_raise_whose_second_empty_publish_finds_no_slot_…`（回落二进制） |
| 699 | 同上；第三次那一格 | `a_raise_whose_third_empty_publish_finds_no_slot_…` |
| 700 | 预演拒了却不把分配器换回抬 F 之前那一份 | 第三次那一条 |
| 701 | 预演报的次序从 0 数 | 第二次那一条 |
| 702 | 抬 F 那一串至多推两次 | 第三次那一条 |
| 703 | 读侧把盘尾半个槽算成一槽（读写两侧分叉） | 几何二进制那一条 |
| 704 | 盘上绝对槽数向上取整（两侧一起过分界） | 几何二进制那一条 |
| 705 | 同上（纯几何） | core 单测 `half_a_slot_at_the_end_of_each_device_…` |
| 706 | 打开文件读一个 extent 节点记两次 | `opening_a_file_whose_lower_extent_segment_has_two_levels_…` |
| 707 | 打开文件读 extent 节点时映射一条都查不到（下段根那一格） | `the_root_of_a_two_level_lower_extent_segment_…` |
| 708 | 同上（下段叶那一格） | `a_leaf_of_a_two_level_lower_extent_segment_…` |
| 709 | 抬 F 真发时第二次落盘途中报错、交回的已落盘份数成了 0（接替第 495 行） | `a_raise_whose_second_empty_publish_fails_on_a_write_…` |

**第 495 行要主 agent 处置**（第六节 Q5）：它点名的 `a_raise_whose_second_empty_publish_is_refused_reports_that_one_publish_of_the_sequence_persisted` 被我删了（那一格——第二次被落点拒、第一次已落盘——预演之后走不到），行本身没动（「不改别人的行」），门禁 59 号跑到它会因点名的测试不存在而红。第 709 行是同一处变异、换到走得到的那一格（写错）。

## 四、证红（每条新测试「改坏哪一行 → 哪条断言红」）

做法：草稿副本 `/tmp/claude-1000/impl-m2-e26/mutants/`（自己的 target `mutants-target`），逐组施加变异表行、跑点名测试所在的**整个**测试二进制、记红集、从 `mutants-originals/` 拷回并 `touch`、`cmp` 与主工作区相同（每组都打了 `restored … (== main)`）。脚本 `scripts/prove-red.sh`、`prove-red-h.sh`，日志 `logs/prove-1/`、`logs/prove-2/`。
基线（副本不改动）：core --lib 119 过、几何 1 过、中央映射 12 过、回落 7 过、随机历史 22 过 2 ignored、步 5 13 过，**基线红集为空**；`first_transaction_on_device` 那个 bin 有 4 条红，**开工快照上同样这 4 条红**（第五节），我点名的测试都不在那个 bin 里。
被测路径上没有 `debug_assert`，红的都是测试自己的断言（逐条看过 panic 位置），没另跑 `--release`。

| 组 | 改坏哪一处（表行） | 哪条断言红（文件:行，消息） | 同一二进制里同时红的 |
|---|---|---|---|
| A | 696–699：预演报错时 `Err(_refusal_ignored) => Vec::new()`（mount.rs 预演那一 `match`） | 随机历史 `:1312` 「种子 … 第 27 / 31 步：这一串的第二次在预演里取不到落点，一次都不发」，实得 `RaiseFloorSequencePublishFailed(publishes_persisted = 1, …)`；回落 `:452` 第 2、3 次那两条，`:616` 第 1 次那条 | 随机历史还红 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（模型改成「抬 F 的墙整串判」之后，半路落了盘的拒绝判对不上） |
| B | 700：删 `*allocator = allocator_before_the_raise;` | 回落 `:477` 「被拒之后分配器与抬 F 之前逐项相同」（第 2、3 次两条）；`:642` 「回收的槽回到 defer」（第 1 次那条，left `[55, 55]`） | — |
| C | 701：`publish_index + 1` → `publish_index` | 回落 `:452`，实得 `refused_publish_in_the_sequence: 1`（期望 2）/ `2`（期望 3）；`:616` 实得 0 | — |
| D | 702：`< ROOT_RING_REGIONS` → `< 2` | 回落 `:452` 第三次那条实得 `None`（两次就做成了）；第二次那条实得 `publishes_in_the_sequence: 2` | 回落 `:551`（做成的那条抬 F 只剩两次）、`:616` |
| E | 703：`of_reader` 的字节数 `+ 8192` | 几何 `:74` 「mkfs 之后：写侧与读侧的几何…逐项相等」，写侧 68208 槽根第 1 层、读侧 68209 槽根第 2 层 | — |
| F | 704–705：`absolute_slot_count_of_device` 改 `div_ceil` | 几何 `:78` 「盘尾那半个槽不算…根在第 1 层」left 2；core `allocation_record_tree.rs:932` 同一句 left 2 | — |
| G | 706：`node_reads += 1` → `+= 2` | 中央映射 `:994` 「上段根兼叶、下段根、两片下段叶各读一次」left `(8, 3, 3)`；`:1038` 两条多跳那条 | — |
| H | 707–708：open_file 那一处查映射的口子换成恒查不到 | 中央映射 `:944` 「打开下段两层的那个文件」`ExtentTreeWalk(MappingMiss { slot: SlotNumber(50254) })`（叶）/ 下段根同一处 | 另红两条 extent 树根搬走的旧用例（`:519`、`:607`） |
| I | 709：已落盘账换成 `Vec::new()` | 回落 `:392` 「这一串在报错之前已经落盘了一次（txg 9）」left 0 | — |
| J | 表里原第 36 行（锚点今天落在 `txgs_of_the_publishes_carrying_the_floor_to_every_device` 里） | 步 5 二进制 6 条红（含它点名的 `raising_the_floor_to_the_first_release_generation_…`） | — |

H 组第一版替换文编不过（未使用的闭包推不出错误类型，`logs/prove-1/H-mapping.log`），改了替换文、只重证 H（`logs/prove-2/`）。14 行全证过，没有留给 59 号只证不跑的。

## 六、停下交主 agent 的设计问题与要知道的事（节的次序按写进文件的先后：第六节在第五节前面）

**Q1 第 1 件为什么是「整串预演」，不是「第 n 次被拒时换回分配器」（实现员的取法，被攻过零轮）。**
- 验收要「第 1、2、3 次被拒之后分配器与抬 F 之前逐项相同」。第 n 次（n ≥ 2）被拒时前 n − 1 次已经落盘：它们取的落点、换下的单元、带新 F 的根都在盘上，分配器不能换回抬 F 之前那一份（下一次发布会把那几次还引用着的槽发出去）；扣住位也不能放（F 没在每块盘上生效，丢一块盘就会让 F 之下的根重新成为候选、而它们的单元已被复用——扣住位本来要挡的那一格）。
- 所以唯一能让「任何一次被拒 ⇒ 与抬 F 之前逐项相同」成立的，是在第一个写之前就知道整串做得成：抬 F 那一串（txg 与次数先按「推到每块盘」算好）在分配器拷贝上逐次走 `prepare_the_version_publish`，与可写挂载取号之前那一串（`dry_run_of_the_publishes_after_acquisition`）同一个做法、同一个不读盘核的口径；预演里哪一次报错就一次都不发，分配器换回。真发之后断言「预演取的落点 = 真发取的」（有一份被隔离时不比，同 `establish_instance`）。代价：每次抬 F 多克隆一次分配器、多走至多 3 次落盘之前那一段，不读盘。
- 走得到新成员的是落点、准入、释放判定这些落盘之前的错；新成员在任何写之前返回（用例钉了盘快照：两盘系统配置槽、根环里的根、录制流步数不变）。
- **预演之后仍留着的一格**：真发时落盘途中写错（`RaiseFloorSequencePublishFailed`，前面几次已落盘、那一次冻结在分配器上），与读盘核出对不上、同预演分叉的那一格。这两格里扣住位照旧留在进程里、下一次抬 F 做成才放；怎么放条款没写（C516 / C546 剩下的那一半，写在 `raise_rollback_floor` 的注释里）。第 709 行变异与写错那条用例钉的就是这一格的账。

**Q2 越格线索那 8 槽的机理，与判决写的不完全是一回事。**
- 判决第三节把它记成 C546 第二次起那一半（扣住的槽不退回）。我在开工快照上加打印复跑种子 4000000045（`logs/explore-seed45-debug2.log`）：抬 F 到 7 时，那 8 槽是盘 0 上 txg 8 那次释放的记录——50176（跨 2）与 50265–50270（各跨 1），释放代 8 > 7、不回收，还在记账的已分配（defer）里；txg 8 是回退到 (1, 5) 之后实例 3 写行那次，换下的是 (1, 5) 那一版的单元。第一次空发布（带 F = 7 的根）落盘之后，checker 按那条根的 F 取候选集——txg 7 上是被抛弃的实例 2，(1, 3)–(1, 5) 落到 F 之下（报文里「低于 F 的根槽 3 个」）——遍历走不到那 8 槽，记账却还算着——这是**收口表第 43 行那一形（F 抬进回退留下的空档）**，扣住位只住内存、checker 读盘，跟它无关。它被判成新发现，是因为那一步不是做成的抬 F，已知红清单第 1 条（要 `RaisedFloor`）不接。
- 预演之后，半路停下的抬 F 一个字节都不写，这一格不再出现（两个种子今天 60 步跑完、每一步 checker 绿）；**抬 F 做成、落进空档的那一形照旧是第 43 行的已知红，没修**。C546 第二次起那一半（扣住位）是同一个改动顺带还上的。
- 两个种子只在空间准入关掉（`SpaceAdmission::SkippedByTheTestOnlySwitch`）时走得到：判着准入时，开工快照上两段历史都跑完、没有红（`logs/explore-seeds.log` 第 70、163 行 `E26 ending=Completed`）。用例因此装了开关。

**Q3 第 3 件为什么「收成一个函数」、没加挂载时的比较与错误成员。**
- 两条路径算的本来是同一个式子：`UNIT_AREA_START_SLOT + (字节数 ÷ 16384 − UNIT_AREA_START_SLOT)` 恒等于 `字节数 ÷ 16384`（`DeviceFreeMap::new` 在字节数不够单元区起点时先因减法溢出 panic）。分叉的来源只有「两处手抄」，收成一处定义就没了。
- 输入那一半：每一处建分配器（`rebuilt_allocator`、`allocator_after_make_filesystem`）都用读者报字节数的同一个 `BlockDevice::size_in_bytes()`，而 `FileBackedBlockDevice`、`DirectInputOutputBlockDevice`、`SparseBlockDevice` 都是打开时记下的字段，包装层原样转发——挂载时「两边比一次」的红支，任何真设备、瞬时读错都走不到，照定义第 6 条不该给它开错误成员；它还罩不到 mkfs 那个进程（那里只写不挂载）。
- 验收那两种几何：字节数不是 16384 整数倍（每盘 68208 个整槽再多半个槽，正压在根层 1 / 2 的分界上）就是今天唯一造得出的「单元区不到盘尾」（尾巴半个槽不在单元区里）；不分叉，读写两侧都在第 1 层，mkfs 进程写的树可写挂载读得回、挂载之后写的树冷走读读得回。改坏读侧那一边（第 703 行）写读两侧当场对不上。
- 我取的读法：几何按「盘上整槽数」算（kb 已定项 14 写的「盘上槽数」），不按单元区末端算——以后单元区若不到盘尾（例如盘尾另留一段），两侧照样同一个数。**盘在两次挂载之间变大或变小**（镜像文件被改了大小）没有条款：根层变了的话，下一次挂载读旧树按 I-1.1「层级不是它的位置规定的那一层」拒绝，不静默；要不要支持改盘大小，交主 agent。

**Q4 Z15 缺口 ① 在「提示过期」那个取样点上，自报数与块层数不相等。**
- 下段两层、提示都没过期：自报 4、块层 4，相等（用例钉 `(4, 3, 3)` 与相等）。
- 下段的根或叶提示过期：自报照旧 4（`ExtentTreeReadsAtOpen::node_reads` 的文档是「每读一个节点算一次，不按位置条目数」），块层数到 6——多跳那个节点两条提示各白试一次。用例钉的是 `块层 = 自报 + 2 × 多跳次数`，不是相等。判决第四节第 2 条写「与块层数到的读次数相等」，按今天的口径在这一格不成立；要不要给 `ExtentTreeReadsAtOpen` 另加一个「发了几次设备读」（像数据单元那一侧的 `device_reads_issued`），交主 agent。
- 缺口 ②：下段根、下段叶各一条，提示指空槽、映射里是真落点 ⇒ 挂载态多跳一次读回全部 145 个单元、冷走读读回同一份；实现没改，今天就通过（改坏回退那一处它们红，第四节 H 组）。

**Q5 第 495 行。** 见第三节。我没改别人的行；要么主 agent 删掉第 495 行，要么授权我删。

**Q6 模型那一改。** `model.rs` 里抬 F 的答案改成「容量墙在第一次写之前整串判」（同可写挂载、回退）：实现今天就是这样，不改的话模型会接受「半路落了盘再拒」。随机历史二进制与门禁 74 号在改后都绿；A 组变异下 `unit_area_wall_sampling…` 那条因此多红一条（第四节）。故障注入、崩溃注入两个二进制也用这个模型与执行器，我没跑（第八节）。

## 五、交回前的验证（末尾原样输出；脚本 `scripts/run-touched-final.sh`，日志 `logs/touched-final/`，线程上限 8，02:5x UTC 起跑）

动到的测试二进制各整跑一次（名字含 layer0 的没有动到）：

```
$ cargo test … (core-lib)
test result: ok. 119 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.61s
$ cargo test … (harness-lib)
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.60s
$ cargo test … (geometry)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
$ cargo test … (central-mapping)
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 126.71s
$ cargo test … (fallback)
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.90s
$ cargo test … (random-history)
test result: ok. 22 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 358.31s
$ cargo test … (on-device-bin)
test result: FAILED. 12 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s
error: test failed, to rerun pass `-p singlefs-harness --bin first_transaction_on_device`
```

`first_transaction_on_device` 那 4 条红是**开工快照上就红的**（`logs/prove-1/pristine-on-device-bin.log`：同 4 个名字、同 4 句断言消息「发布 C 与发布 B 同型」「发布 D 与发布 C 同型」「同一段里失败之前已经落盘的发布…」×2），这一轮只在这个 bin 的两处穷举列表里加了新成员，没带进新的红。

`cargo fmt --check`（退出 1，差异只在别的会话的两个文件）：
```
     10 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
      8 Diff in /home/fy5090/code/singlefs/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```
check.sh 那一套 lint 下的 `cargo clippy --all-targets --all-features`（再用 `--keep-going` 跑一遍，确认只有这一处）：
```
     |
     = help: remove the assertion
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#assertions_on_constants
     = note: `-D clippy::assertions-on-constants` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::assertions_on_constants)]`

error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 1 previous error
clippy exit=101
```
唯一的错在 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:4057`（别的会话在改的文件）。

`cargo build --offline --all-targets`：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
```

门禁（登记给 implementation-writer 的几道，`logs/gate-*.log`）：33 号追加变异行之后跑，退出 0，末行：
```
  ✓ 147 个实验二进制都有成形的变异表，1631 条变异的原文各命中源码一次；crates/mutations.tsv 704 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
```
53 号 0、92 号 0、93 号 0、94 号 0、89 号 77（「本次未跑：收口表第 27 行那几笔的前置一个都没进来」）、74 号 0：
```
  ✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 5 段）：
gate74 exit=0
```
（build 那一行之前是增量编译，`summary.txt` 记的是 `build exit=0`、`fmt exit=1`、`clippy exit=101`。）

## 七、要知道的事（不归我，但这一轮碰得到）

- 两个越格种子只在空间准入关掉时走得到（第六节 Q2）；产品路径（判准入）上开工快照就跑完、不红。
- 抬 F 的写序没变：预演一个写都不发，真发那一串与改之前逐次发的是同一串计划（`empty_publish_plan_raising_the_floor` 就是原来循环里那张计划原样挪出去）。层 0 里带抬 F 的那条流（固定脚本到 E）的录制流按理逐字不变，没跑层 0 核。
- `DeviceFreeMap` 多了一个字段，分配器的 `Debug` 输出多一项；按 `Debug` 整份比的用例两边同多，照样相等（回落二进制绿）。

## 八、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异整表归 `crash-verifier`；没提交。
- 没跑层 0 任何一条流；没跑全量 `cargo test`、`check.sh`、`gate.sh` 整轮、门禁 54 / 55 / 57 / 59 / 87。
- 没跑 `second_transaction_supplement_three_fault_injection`、`second_transaction_supplement_three_crash_injection`：我没改它们的源码，但它们用的历史执行器与模型（`history.rs`、`model.rs`）这一轮改了抬 F 那一格（新成员的映射、「墙整串判」）。故障注入把写错摆进抬 F 那一串第二次时，模型那一侧怎么判没核——交回之后提交前的整轮会跑到。
- 没改 kb：C546、C516、C552 的欠账状态、D8 已定项 14「实现取值」要不要补一句「盘上槽数 = 整槽数」、收口表第 58 / 28 行的现状，交书记员。
- 没删、没改 `crates/mutations.tsv` 第 495 行（第六节 Q5）。
- 盘在两次挂载之间改大小、`ExtentTreeReadsAtOpen` 要不要另报设备读数：条款没写，只写进第六节，没做。
- 收口表第 43 行那一形（抬 F 做成、落进回退留下的空档）没修，已知红清单第 1 条照旧。

## 附：`git diff --stat -- crates litmus` 原样（含别的会话此前留下的未提交改动；我改的是第二节那些。新建的几何测试文件没进 git，不在这张表里）

```
 crates/mutations.tsv                               |  342 +-
 crates/singlefs-checker/src/image.rs               |   79 +-
 crates/singlefs-checker/src/lib.rs                 |  100 +-
 crates/singlefs-checker/src/walk.rs                | 2170 ++++++++-
 crates/singlefs-core/src/admission.rs              |  174 +-
 crates/singlefs-core/src/allocator.rs              |  226 +-
 crates/singlefs-core/src/instance_table.rs         |  135 +-
 crates/singlefs-core/src/journal.rs                |  139 +-
 crates/singlefs-core/src/lib.rs                    |    4 +
 crates/singlefs-core/src/make_filesystem.rs        |    2 +
 crates/singlefs-core/src/mount.rs                  | 1259 +++--
 crates/singlefs-core/src/mounted_read.rs           |  285 +-
 crates/singlefs-core/src/recovery.rs               | 1190 +++--
 crates/singlefs-core/src/system_configuration.rs   |   76 +-
 crates/singlefs-core/src/transaction.rs            | 4934 +++++++++++++++-----
 crates/singlefs-core/src/write_accounting.rs       |   20 +-
 crates/singlefs-format/src/lib.rs                  |  131 +
 crates/singlefs-harness/src/bad_disk_input.rs      |  335 +-
 .../src/bin/e156_allocation_basis_counts.rs        |  744 ++-
 .../src/bin/e158_root_choice_repair.rs             | 2578 +++++++++-
 .../src/bin/first_transaction_device_log_check.rs  |  253 +-
 .../src/bin/first_transaction_on_device.rs         |  959 +++-
 .../src/bin/first_transaction_region_bytes.rs      |    5 +-
 crates/singlefs-harness/src/fault_injection.rs     |    5 +-
 crates/singlefs-harness/src/history.rs             |  224 +-
 crates/singlefs-harness/src/model.rs               |  467 +-
 crates/singlefs-harness/src/model_comparison.rs    |   91 +-
 crates/singlefs-harness/src/on_device_modes.rs     |  132 +-
 crates/singlefs-harness/src/scenario.rs            |    2 -
 .../tests/checker_known_bad_images.rs              | 1298 ++++-
 crates/singlefs-harness/tests/common/mod.rs        |   29 +
 .../tests/first_transaction_step_five_publish.rs   |  313 +-
 .../tests/first_transaction_step_seven_layer0.rs   |   16 +-
 .../tests/first_transaction_step_six_recovery.rs   |    5 +-
 .../tests/first_transaction_step_two_data_unit.rs  |    4 -
 .../tests/parallel_line_one_sequential_write.rs    |   55 +-
 .../second_transaction_mapping_node_admission.rs   |  230 +-
 ...ransaction_parallel_line_one_multi_unit_file.rs |  167 +-
 ...ansaction_parallel_line_one_sequential_write.rs |   52 +-
 ..._transaction_parallel_line_three_many_inodes.rs |  256 +-
 ...d_transaction_parallel_line_two_mounted_read.rs |  343 +-
 .../tests/second_transaction_step_five_reuse.rs    |   29 +-
 .../tests/second_transaction_step_four_rollback.rs |  241 +-
 .../tests/second_transaction_step_one_overwrite.rs |  218 +-
 ...second_transaction_step_three_formatted_pool.rs |  192 +-
 ...transaction_step_three_formatted_pool_layer0.rs |    4 +-
 ...econd_transaction_step_three_second_instance.rs |  103 +-
 .../tests/second_transaction_step_zero_layer0.rs   |   27 +-
 ..._transaction_supplement_one_write_accounting.rs |   97 +-
 ..._transaction_supplement_three_bad_disk_input.rs |   37 +-
 ...transaction_supplement_three_crash_injection.rs |    3 +
 ...transaction_supplement_three_fault_injection.rs |  131 +-
 ..._transaction_supplement_three_random_history.rs |  590 ++-
 ...nsaction_supplement_two_accounting_node_full.rs |  134 +-
 ...transaction_supplement_two_admission_formula.rs |  318 +-
 ...two_c533_row_publish_record_without_its_root.rs |   87 +-
 ...ion_supplement_two_commit_generated_fallback.rs |  496 +-
 ...nsaction_supplement_two_instance_table_chain.rs |   53 +-
 ...tion_supplement_two_instance_table_page_full.rs |  278 +-
 ...n_supplement_two_release_checksum_quarantine.rs |  906 +++-
 ...saction_supplement_two_reused_record_overlap.rs |   19 +-
 ...n_supplement_two_root_ring_turn_in_one_mount.rs |    6 +-
 ...saction_supplement_two_row_publish_admission.rs |  358 +-
 ...ement_two_tree_nodes_and_the_central_mapping.rs |  792 +++-
 ...upplement_two_unreadable_abandoned_root_slot.rs |   44 +-
 .../system_configuration_mutability_classes.rs     |    5 +
 66 files changed, 19780 insertions(+), 5217 deletions(-)
```
