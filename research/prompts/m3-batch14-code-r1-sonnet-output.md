# 删除批第 14 批合入的代码：代码轮第一轮，云端正推腿（Sonnet）报告（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 GA GB GC K1 K2 K3 U8 -->

立场：正推。格：K1（条款对实现，N1–N6）、K3（21 份文件各一格）。K2（哪一格会错）归攻方腿，这份不判。
引代码写冻结副本 `/tmp/claude-1000/m3-batch14-code-r1/tree/` 里的相对路径加行号（报告里写 `crates/…:行号`）；引 kb 写 kb 文件自己的行号（`.claude/kb/decisions/…`）。

## 零、做法与证据

| 事 | 命令 / 做法 | 结果 |
|---|---|---|
| 冻结副本与快照一致 | 在冻结副本根上 `sha256sum -c research/prompts/m3-batch14-code-r1-snapshot/crates-sha256.txt` | 21 份全 OK，0 份对不上 |
| kb 与快照一致 | `sha256sum` D16、D8 两份，与 `…/kb-sha256.txt` 比 | 开工时两份逐字相同（`0e43f344…`、`92d97707…`）；交回前 D16 已被根环那条线写回改过（现行 sha256 `eeeb6563…`），D8 没变；D16 的差见 N1 第一段 |
| 两份补丁与冻结副本对得上 | 把背景材料 diff 文件里实现员 P（`_m3-batch14-code-r1-diff.md` 第 8–1611 行）、Q（第 1617–1757 行）两段各存成补丁，在冻结副本的拷贝上 `patch -p1 -R --dry-run` | 两份都能整份反向套上，无一处 FAILED；P 动 14 份（含新建 1 份）、Q 动 6 份，加变异表 = 21 份 |
| 变异表锚点 | 模型目录 `anchors.py`：名字含「第 14 批」「抬 F 回收的槽不扣住」「丙 1」「十二棵树」的行，原文在被改文件里数命中次数 | 15 行（第 14 批新增 10 行：2273–2282；步 5 一行：46；丙 1 四行改名：2174–2177）每行恰好命中 1 次，没有一行原文与替换文相同 |
| 变异逐条复跑 | 在冻结副本的拷贝（草稿目录里的 `w/`，交回前已删）上，`runmut.sh`（现为模型目录的 `run-row.sh`）套 `run-with-memory-cap.sh 6G` 与 `capped.sh`（先 15、协调者通知后改 7），每条：套替换、`cargo test --offline` 点名的目标、还原、核还原后 sha256 | 第 14 批新增 10 行全红在点名的用例上；步 5 一行与丙 1 四行（改名）全红；见第一节各格 |
| harness 档基线 | 同一拷贝上 `cargo test --offline --no-fail-fast -p singlefs-harness --test …` | 见下表 |
| checker 档 | 一条测试不跑；只 `cargo check --offline -p singlefs-checker-tier --all-targets` | 退出码 0（不开 required-features 的目标），日志在模型目录 `logs/check-checker-tier.log` |

harness 档基线（冻结副本拷贝上，没有任何变异，全部原样）：

| 目标 | 结果行 |
|---|---|
| a_lone_surviving_device_reads_back_what_it_read_before_the_unmount | `test result: ok. 2 passed; 0 failed` |
| admission_raises_the_floor_before_refusing | `test result: ok. 25 passed; 0 failed; 1 ignored` |
| admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments | `ok. 5 passed` |
| admission_ring_window_and_tombstone_prepay_on_the_publish_path | `ok. 5 passed` |
| container_trees_grow_a_level_when_the_root_is_full | `ok. 4 passed` |
| mount_transparency_judged_field_by_field_after_a_writable_mount | `ok. 33 passed` |
| overwrite_in_one_instance | `ok. 12 passed` |
| unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over | `ok. 24 passed` |
| unlink_idempotence_criteria_and_rollback_to_the_emptied_version | `ok. 5 passed` |
| unlink_mount_continuation_and_the_emptied_version | `ok. 14 passed` |
| unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches | `ok. 1 passed` |
| unlink_single_file_publishes | `ok. 9 passed` |
| writable_mount_of_a_formatted_pool | `ok. 15 passed` |
| write_accounting_per_structure | `ok. 3 passed` |
| model_answers_unmount_with_the_floor_at_the_current_version | `ok. 5 passed` |
| unlink_in_histories_judged_by_the_model | `ok. 8 passed` |
| random_histories | `ok. 23 passed; 0 failed; 18 ignored`（慢档 18 条没跑） |
| `-p singlefs-core --lib` | `ok. 180 passed` |

日志在模型目录 `research/prompts/m3-batch14-code-r1-sonnet-model/logs/` 的 `gc-base.log`、`base1.log`、`hist.log`、`core-lib.log`。

收到协调者消息（线程上限 15 改 7）的位置：GA 两行（2273、2274）第二次复跑（为取完整的 `MappingStillUnreadable` 槽号）之后；此前的命令用 15，此后的用 7（`capped.sh 7`）。

## 一、K1　条款对实现

### N1　GA：卸载那一格只当场放开一截

**从定案推到条款。** 用户 2026-10-03 弹窗选「甲 GA（推荐）」，岔路单第 5 行的定义是「卸载那一格回收时，释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用，其余照准入那一格扣到生效」，状态「用户已定」（`research/prompts/m3-unlink-code-r3-forks.md:5`）。D16 今天的原文有这一句：「最小者的那一截当场可用（预演与真发都算上），其余与准入那一串一样扣到生效」（`.claude/kb/decisions/16-发布语义.md:42`）。条款里「有效根」的定义是：「盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg」（`.claude/kb/decisions/16-发布语义.md:34`）里「自证合法、按实例表判仍然有效」那一半——被抛弃的根不算有效。读坏的槽：快照时第 38 行写的是挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽「当没有根」；**快照之后**这一行被根环那条线改写（用户 2026-10-04，D16 现行文件的 sha256 已不是快照里的 `0e43f344…`，第 34、42 行与快照逐字相同，第 35、37、38 行不同——我拿背景材料里的行与现行文件逐行比出来的）：算准入抬 F 的上限时这类槽改成「当作可能是一条非空有效根、F 不越过它」，判回退候选与算 F_生效 时才当没有根。两版第 38 行都有的前半句是「挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽」（`.claude/kb/decisions/16-发布语义.md:38`），后面「这个进程知道住着根的槽读坏，重读一次，仍坏就拒这次抬 F 或这次回退」两版都有。GA 的取小不在改写的那两处里，条款没有字面规定读不出的槽在这一步怎么算（见下表），所以改写不影响本格；缺席让取小只会更小的读法也不依赖那一句。

**从条款推到代码行（冻结副本）。**

| 条款的一句 | 代码行 |
|---|---|
| 卸载那一格只放开一截，准入整截扣 | 「RaiseFloorEntry::Admission => ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:2345`）；「RaiseFloorEntry::Unmount => ReuseOfTheSlotsReclaimedByTheSequence::ImmediatelyBelowTheNewestValidRootOnEveryDevice,」（`crates/singlefs-core/src/mount.rs:2346`） |
| 每块盘上最新有效根，被抛弃的不算 | 「!abandoned_by_table(root, table)」（`crates/singlefs-core/src/mount.rs:2380`）与「&& device_of_txg(root.checkpoint_txg) == *device」（`crates/singlefs-core/src/mount.rs:2381`）；`abandoned_by_table` 一处定义转到 `recovery::root_is_abandoned_by_the_instance_table`，判法是表里有这个实例的行且根的 txg 大于行里选定的根：「.any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1540`） |
| 取小；有一块盘一条有效根都没有 | 「.collect::<Option<Vec<CheckpointTxg>>>()?」（`crates/singlefs-core/src/mount.rs:2389`）、「.min()」（`crates/singlefs-core/src/mount.rs:2391`）：任何一块盘 `None` 整个交回 `None` |
| 当场放开的槽预演与真发都算上 | 放开发生在整串预演之前、回收那一步：「let floor_of_the_reclaim = reclaim_floor(new_floor, oldest_valid_root_in_the_window);」（`crates/singlefs-core/src/mount.rs:2947`），然后「floor_of_the_reclaim.min(newest_valid_root_on_every_device),」（`crates/singlefs-core/src/mount.rs:2962`）带 `Immediately`；预演在回收之后（预演的分配器拷贝已经带着放开的槽） |
| 其余扣到生效 | 「reclaimed.extend(allocator.reclaim_released_up_to(」（`crates/singlefs-core/src/mount.rs:2967`）第二次按整个 `floor_of_the_reclaim` 带 `HeldUntilFloorTakesEffect`；第一次已回收的槽不重复回收：「if !self.reclaimed.insert(key) {」（`crates/singlefs-core/src/allocator.rs:1735`），所以第二次只收第一次剩下的；生效之后「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:3082`）放开扣着的 |
| 有一块盘一条有效根都读不出：整截扣 | 「None => Vec::new(),」（`crates/singlefs-core/src/mount.rs:2965`）：第一次回收空，第二次整截扣。**条款没写这一格**（D16 第 42 行只说「最小者」），是代码取的保守读法，写在函数文档里（`crates/singlefs-core/src/mount.rs:2367`）：「有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效」 |
| 根槽读坏的两类 | 读根环那一遍在动分配器之前、读坏拒：「let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(」（`crates/singlefs-core/src/mount.rs:2893`）；挂载时就读不出而没写过的槽在 `roots` 里就是缺席，缺席让取小只会更小（推） |
| 准入那一格不变 | 准入臂的回收调用与改之前同一个：`reclaim_released_up_to(floor_of_the_reclaim, ReclaimedReuse::HeldUntilFloorTakesEffect)`，只是 `floor_of_the_reclaim` 取成局部量；m7（把准入臂也换成 GA）在 `admission_raises_the_floor_before_refusing` 里红 5 条，所以准入那一格有钉 |
| 树表 0 条的一版不推 | 「PoolVersion::WithoutFile(current_version_without_file) => {」（`crates/singlefs-core/src/mount.rs:2474`）在调抬 F 那一串之前就 `return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile …)`；新函数只在 `raise_the_floor_through` 里调，走不到这一支，与 D16 第 42 行最后一句对得上 |

**两条我自己推的（标「推」，没有现成条款）。**
1. 对卸载，`floor_of_the_reclaim` 恒不低于每块盘上最新的根：新 F 就是现行那一版的 txg，「let rollback_floor = current_file_version.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:2483`），会话里现行那一版就是环里最新的一版（推）。所以卸载这一臂里 `floor_of_the_reclaim.min(…)` 的 `min` 恒等于取小的那一项，那个 `min` 是冗余的，不是错（推）。
2. 安全方向：放开的槽 g 满足 g ≤ 各盘最新有效根 txg 的最小者。引用一个槽的根都早于它的释放代，所以任何一块盘上「不早于该盘最新有效根」的根都不引用它。恢复择根照 `(checkpoint_txg, 实例代号)` 取最大、不过滤被抛弃的根：「三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的」（`crates/singlefs-core/src/recovery.rs:1263`）。代码把被抛弃的根排在「最新」之外只会让取小更小、放开的更少；读不出的槽缺席同理。成立的前提是：幸存那块盘上恢复读得出它最新的有效根。幸存盘上最新根同时又读不出（丢盘加坏扇区的双故障）时，恢复会退到更旧的根，这一格不在 D16 第 42 行「丢一块盘」的射程里，我没推、也没量。

**钉它的测试与变异（我在冻结副本拷贝上复跑，原样输出在模型目录 `logs/`）。**

| 变异 | 改了什么 | 红的用例 / 结果 |
|---|---|---|
| 变异表 2273、2274 | 去掉 `.min(newest…)`，等于改回丙 1 | 各红 1 条，红在 `assertion left == right failed: 剩下那块盘走到的根 txg 与每个活文件的内容与卸载之前同一块盘上读出的相同`，`left: Err("只读挂载：Open(Walk(MappingStillUnreadable { slot: SlotNumber(1411) }))")`（h2）与 `slot: SlotNumber(50242)`（h4）；与实现员 P 报的槽号一致 |
| m4 | 取小改取大 | `a_lone…` 两条红 |
| m5 | 去掉「落在这块盘」的过滤（每块盘取成同一个全局最新） | `a_lone…` 两条红 |
| m6 | `min(newest + 1)`（差一） | `a_lone…` 两条红 |
| m8 | 只看第一块盘 | `a_lone…` 两条红 |
| m1 | 去掉第二次回收（其余不扣到生效、也不回收） | `admission_raises…` 红 4 条，`writable_mount_of_a_formatted_pool` 红 1 条 |
| m7 | 把准入臂也换成 GA | `admission_raises…` 红 5 条 |
| 变异表 2174–2177（丙 1 四行改名） | 卸载那一格整截扣到生效 | 四行各自点名的用例都在红里；每行都是同一处替换，整份 `admission_raises…` 红 4 条（`a_mount_whose_continuation_walks_to_empty…`、`a_mount_whose_deletion_continuation_stays_short…`、`an_unmount_losing_power…`、`after_the_unmount_loses_power…`），红在「卸载那一串的预演算上新 F 回收出来的槽，做成」被预演拒 |
| 变异表 46（步 5） | 准入臂的回收改成 `Immediately` | `reuse_after_raising_the_floor` 的 `slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect` 红：`txg 16 的固定点落在这次回收的槽 50240 上：F 还没在两块盘上生效`。实现员 P 自报「点名的是慢档用例，没证」：这一行的命令带 `--include-ignored`，我按表里的命令原样跑，这条红了，证了 |

**没有钉的两处（我自写的变异，在下面 12 个 harness 目标里一条都不红）。**
- m3：去掉被抛弃根的过滤（`!abandoned_by_table(root, table)` 那一半）。
- m2：`None` 那一格（某块盘一条有效根都没有）改成整截当场放开。
- 12 个目标 = 会调 `unmount(` 的 11 个加 `reuse_after_raising_the_floor`，分两批跑：第一批 8 个（`a_lone…`、`admission_raises…`、`model_answers_unmount…`、`writable_mount_of_a_formatted_pool`、`mount_transparency…`、`unmount_marker…`、`rebuilt_root_takes…`、`reuse_after_raising_the_floor`）加第二批 4 个（三个 `entries_after_…` 与 `checker_known_bad_images`），日志 `logs/battery.log`、`logs/battery4.log`。checker 档那几条（`crash_injection…`）没跑，这两处在那里红不红我不知道。
- 我的读法（推）：被抛弃的根在活会话里排不进「最新」——回退之后新实例的 txg 大于环里每一条根，暖机又先把根铺满两块盘，所以每块盘的最新根都是新实例的；这一格在活会话里走不到，只在构造的镜像上有。P 自报「被抛弃的根在单盘上被择中那一形没有用例钉」与此相符。

**这一格的判定：一致**（定案 → D16 第 42 行 → 代码，每一句对得上）；两处保守的读法条款没写（某块盘无有效根 → 整截扣；读不出的槽缺席），**规则没说**，记在这里。

**什么现象会推翻它。** (1) 有一份起点，丢一块盘后剩下那块盘只读挂载读不出或读出的与卸载之前不同，而 GA 下的放开确实写掉了被它择中的根引用的单元（攻方 K2 的主查项）；(2) 有一份起点，GA 下卸载在预演里被拒而条款说该做成；(3) 去掉被抛弃根过滤或 `None` 那一格的变异在会话里造得出红（说明那两处不是死分支）；(4) D16 第 42 行改字。

### N2　GC：单盘幸存读回用例 h2、h4

**从判决推到用例。** 上一轮判决第三节第 2 条要的是：「剩下那块盘只读挂载、整读每个活文件，判与卸载之前同一块盘上读出的相同」（`research/prompts/m3-unlink-code-r3-main-verification.md:65`），取 h2、h4 两形；丙 1 下红，改之前那一版与 GA 下绿。

**用例做的事（冻结副本）。** 读回的定义：「let read_only = mount_read_only(image).map_err(|failure| format!("只读挂载：{failure:?}"))?;」（`crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs:69`）、「for record in read_only.mounted.inode_records() {」（`crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs:72`）逐个活文件打开、读到文件尾；这是用户看得到的读回（只读挂载做得成、每个活文件读出的字节），不是池级 checker 的条名。注错：从「先写系统配置 2 个写 + 第一次空发布的单元写」之后的第一个写起，每块盘的每个写都报错（`…unmount.rs:155` 的 `EveryMatchingCallFromTheNthOnward`），单元写的个数是不注错地在一份拷贝上卸载一次数出来的。丢掉的盘是住着现行那一版根的盘（`…unmount.rs:198` 起）。断言三项：幸存盘上恢复择到的根早于现行那一版，「.is_some_and(|chosen| chosen < survivor.current_txg_before_the_unmount),」（`…unmount.rs:228`）；卸载之前同一块盘读得出，「survivor.before_the_unmount.is_ok(),」（`…unmount.rs:235`）；掉电之后读出的与卸载之前逐项相同，「survivor.after_the_crash, survivor.before_the_unmount,」（`…unmount.rs:240`）。h2 在 `…unmount.rs:250`，h4 在 `…unmount.rs:318`。

**我量到的。**
- 基线两条绿（`logs/gc-base.log`：`test result: ok. 2 passed`）。
- 去掉 `.min(…)`（变异表 2273、2274，等于丙 1）两条红，原样输出里的 `left` 是：`Err("只读挂载：Open(Walk(MappingStillUnreadable { slot: SlotNumber(1411) }))")`（h2）、`Err("只读挂载：Open(Walk(MappingStillUnreadable { slot: SlotNumber(50242) }))")`（h4），与 P 自报的槽号一致。
- 换成「改之前那一版」（变异表 2176 那一行的替换，卸载整截扣到生效）两条仍绿（`logs/fullhold.log`：`test result: ok. 2 passed`），与判决「改之前那一版与 GA 下要绿」相符。
- **整读的活文件集合：h2 是空的。** 探针（`probe-lone-survivor-live-files.sh`，在副本里临时加一行输出，跑完还原并核 sha256）原样输出：`PROBE live_files=0 sizes=[] root=CheckpointTxg(13)`（h2），`PROBE live_files=1 sizes=[(1, 3000)] root=CheckpointTxg(10)`（h4）。h2 里续做已把文件删空，所以 h2 实际判的是「只读挂载走树做得成、走到的根 txg 相同」，没有逐字节的文件内容可比；h4 才读到一个 3000 字节的活文件。丙 1 下 h2 红在挂载那一步（`Open(Walk(MappingStillUnreadable…))`），那一步已经是用户看得到的结局（只读挂载失败），所以 h2 的判别力在，只是「整读每个活文件」在 h2 里是空操作。
- 同在 `admission_raises_the_floor_before_refusing` 里、P 改成「钉 GA 的结局」的那几条（`assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_the_newest_root_of_every_device`，`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs:1615`）：P 自报「改回丙 1 照样绿」，**我证实了**——把变异表 2273 那处替换（等于丙 1）套在整份 `admission_raises_the_floor_before_refusing` 上，`test result: ok. 25 passed; 0 failed; 1 ignored`（命令见下一行）。它们的名字与注释读起来像钉 GA，其实只钉「卸载做得成、没有落在释放代高于 19 的槽上」，不分辨丙 1；能分辨 GA 与丙 1 的只有 `a_lone…` 这两条。判决要的就是这两条，所以不算缺，记下来是为了不要把 `admission_raises…` 里那些用例当 GA 的钉。
  命令：`MUT_W=<副本> CARGO_TARGET_DIR=<编译目录> run-row.sh --adhoc crates/singlefs-core/src/mount.rs mut/ga.orig mut/ga.repl -p singlefs-harness --test admission_raises_the_floor_before_refusing`。

**判定：一致**（读回结局是用户看得到的；丙 1 红、GA 绿、改之前绿，三项都量到）。**注**：h2 整读为空；GA 在入库代码上钉它的只有这两条，共用同一个造法（`start_on_narrow_devices…`／`start_plain` 后卸载）。

**什么现象会推翻它。** 把 h2 的文件删空改成不删空（让整读真读到字节）而结果变了；或者有第三种造法的单盘幸存形在丙 1 下红而这两条绿（攻方 K2 的活）。

### N3　Q-F 乙：意图带时间、三格同改

**从定案推到条款。** 岔路单第 9 行：「乙 删除意图那次由调用方带时间，写进三个时间秒（atime、mtime、ctime 共一个值）；各批照抄意图那次写的值」（`research/prompts/m3-unlink-code-r3-forks.md:9`），状态用户已定。D8 今天的原文：「unlink 的意图那次写调用方随这次 unlink 给的时间，三个时间秒（atime、mtime、ctime）共一个值，各批照抄意图那次写的值」（`.claude/kb/decisions/08-核心索引结构.md:195`），①b：「意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间」（`.claude/kb/decisions/08-核心索引结构.md:470`）。

**从条款推到代码行。**

| 条款的一句 | 代码行 |
|---|---|
| 调用方给时间 | `UserChange::Unlink` 带时间：「Unlink {」（`crates/singlefs-core/src/mounted_session.rs:64`）、「write_time_seconds: u64,」（`crates/singlefs-core/src/mounted_session.rs:66`）；会话把它交给意图：「} => publish_unlink_intent(」（`crates/singlefs-core/src/mounted_session.rs:252`）。`UserChange` 是 `Copy`，被拒后推抬 F 再试的重发用的是同一个值（推：重发循环里 `change` 按值再传） |
| 意图那次三个时间秒是调用方的值 | 意图的参数「write_time_seconds: u64,」（`crates/singlefs-core/src/transaction.rs:5857`），写进记录：「write_time_seconds,」（`crates/singlefs-core/src/transaction.rs:5882`）；记录序列化把同一个字段写三遍：「for _clock in 0..3 {」（`crates/singlefs-core/src/records.rs:77`）、「writer.put_u64(self.write_time_seconds);」（`crates/singlefs-core/src/records.rs:78`），偏移 64 起 atime、mtime、ctime 三格同值 |
| 各批照抄意图那次写的值 | 「write_time_seconds: record.write_time_seconds,」（`crates/singlefs-core/src/transaction.rs:5945`），`record` 是上一版里 nlink = 0 的那条记录；批的函数没有时间参数（`crates/singlefs-core/src/transaction.rs:5907` 起） |
| 续做的几批没有调用方、不另取时间 | 会话走空与挂载续做共用一个走空函数，走空只调批：「match publish_unlink_step(pool, allocator, current, inode, batch_size, instance) {」（`crates/singlefs-core/src/transaction.rs:6010`）；挂载续做从盘上重建的那一版读记录，所以拿到的是意图那次写在盘上的值 |
| 随机历史里的取法 | 「FIXED_WRITE_TIME_SECONDS + u64::try_from(step_index).expect("步号装得进 u64");」（`crates/singlefs-harness/src/history.rs:4663`）与覆盖写、建 inode 同一个取法；「HistoryOperation::Unlink => apply_unlink(pool, write_time_seconds),」（`crates/singlefs-harness/src/history.rs:4686`） |

**十份 harness 测试传的时间。** 除下面点名的一处，传的是该记录上一版的值，盘上字节与改之前相同（推：改之前的实现就是照抄这条记录的上一版时间）。逐份：`admission_refuses_before_units…rs:227`、`:288`（`write_time_seconds_of_the_file`），`admission_ring_window_and_tombstone_prepay…rs:142`、`unlink_idempotence_criteria…rs:87`、`unlink_of_a_file_spanning_two_extent_lower_leaves…rs:108`（都是 `previous.inode_record….write_time_seconds`），`unlink_single_file_publishes.rs:69`（「map_or(FIXED_WRITE_TIME_SECONDS, |record| record.write_time_seconds)」），`mount_transparency…rs:1101`（`*write_time_seconds`，池里现有的），`container_trees_grow_a_level…rs:63` 的 `WRITE_TIME_SECONDS_OF_THE_NEW_INODES`（与建空 inode 同一个值），`unlink_mount_continuation…` 传 `write_time_seconds_of_the_first_file`（记录现有的）。**例外一处**：`admission_raises…` 传 `pool.write_time_seconds`，那是池里最近一次写用的时间，每次写前加一：「self.write_time_seconds += 1;」（`crates/singlefs-harness/tests/common_admission_pool/mod.rs:319`），建 inode 那一支也加（同文件 335、348 行同形）；接在建 inode 之后 unlink 时，它可以大于第一个文件那条记录现有的时间，所以这一份「盘上字节与改之前相同」我不能一概写成立（这些用例不钉时间字节，断言不受影响）。全部在基线上绿（见第零节表）。

**①b 还成不成立。** 用例 `two_copies_of_the_image_between_two_batches_continue_to_byte_identical_disks`（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:687`）绿。但它的意图用的是记录现有的时间，不改变「意图的时间 ≠ 上一版」的条件，所以它不分辨 Q-F 乙；①b 在乙下成立靠的是代码结构：批没有时间参数、只从上一版记录抄。**我另外量了这一格**（探针 `probe-mount-continuation-time.sh`，副本里临时加一条用例，跑完还原并核 sha256）：意图给一个与上一版不同的时间（上一版 1788000060，意图 1788000660），崩在意图之后，可写挂载续做，续做的几批：`PROBE time_before=1788000060 intent=1788000660 continuation_in_memory=[Some(1788000660), Some(1788000660), None] continuation_from_disk=[Some(1788000660), Some(1788000660), None]`，末尾 `None` 是删掉记录的最后一批。续做拿到的是意图那次的值，内存里和从盘上重建的两份都对。这条探针不在仓里。

**钉它的用例与变异。** 新用例 `the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:552`）按盘读这一版 inode 叶容器的单元字节，找第一个文件那条记录，读偏移 64、72、80 三格（读的是盘，不是内存里那份记录，`…continuation_and_the_emptied_version.rs:505` 起），断言两块盘上都是给的时间：「vec![[time_of_the_intent; 3]; 2],」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:564`）。变异表 2275（意图照旧带上一版的时间）红在第一项，`left: [[1788000060, 1788000060, 1788000060], [1788000060, 1788000060, 1788000060]]`；2276（中间一批另取时间，取成这一批的改动计数）红在第二项，`left: [[6, 6, 6], [6, 6, 6]]`。两行都量了、都点名这一条。

**缝（我量到的，不是推的）。** 模型和池级 checker 都不读写入时间：在冻结副本上 `grep -rn 'write_time' crates/singlefs-harness/src/model.rs crates/singlefs-harness/src/model_comparison.rs crates/singlefs-checker/src` 零命中。所以随机历史和层 0 不会因为取法变而红或绿，unlink 的时间字段**只有这一条用例（加两行变异）在守**，而它走的是进程内的批，不走挂载续做；挂载续做那一支只靠「走空函数共用」，我的探针量过一次，仓里没有用例钉。checker 档里用 `HistoryOperation::Unlink` 的几处（`crates/singlefs-checker-tier/src/crash_injection.rs:4208`、`:4265`、`crates/singlefs-checker-tier/tests/crash_injection_writable_mount_after_the_crash.rs:50`）有没有钉了字节而受影响，**复核不了**（checker 档一条不跑）；按上一句，被钉的东西里没有时间，我推没有，没量。

**判定：一致**（定案 → D8 第 195、470 行 → 代码行 → 用例 → 变异，每一环对上）；缝两条记下。

**什么现象会推翻它。** 同一份崩溃镜像各自续做，两份盘不逐字节相同；挂载续做的某一批的记录时间不是意图那次的值；`UserChange::Unlink` 被拒后重发时间变了；另有一条写路径在 unlink 的版本里改了这条记录的时间秒而不经意图。

### N4　U8 一行变异与 N2 四条注错用例

**U8。** 判决第三节第 5 条要的是：「重写根那一支补一行变异，点名会红的用例」（`research/prompts/m3-unlink-code-r3-main-verification.md:68`）。代码里重写根那一支是「if resolved.tombstone_tree.is_the_root_rewritten() {」（`crates/singlefs-core/src/transaction.rs:4189`），它返回这次形状算的层数，不读上一版的码 2 头。变异表 2277 的名字是「重写墓碑树根的那次发布不按这次的形状算 H」（`crates/mutations.tsv:2277`），替换是给这一支加 `&& previous.is_none()`，于是有上一版时落到下面读上一版根节点头的那一支；锚点命中 1 次。点名的用例 `the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place`（`crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs:168`）在这个变异下红：`assertion left == right failed: 墓碑预付按长高之后的 H = 1：inode 1、5、6 三个活对象（2、3 删完，4 删除中）`，`left: 98550`。改坏的就是名字说的那一处（长高那次 H 少 1）。

**N2 四行。** 判决第三节第 6 条要的是：「注错点放到发布第一个写之后的位置」（`research/prompts/m3-unlink-code-r3-main-verification.md:69`），加「续做那一处抬 F 先写系统配置报错」。新用例四条在 `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`：第二个单元写（1332 行）、journal 记录写（1358 行）、根槽写（1376 行）、续做里抬 F 先写系统配置的第二个写（1396 行）。变异表 2278–2280 三行是同一处替换：「(walk, pool.writes_of_failed_publishes().to_vec())」（`crates/singlefs-core/src/mount.rs:4837`）换成 `(walk, Vec::new())`，挂载交回的失败账变空；2281 把抬 F 先写系统配置那一步的错里的写账换成 `NOTHING_WRITTEN`（只命中循环里某块盘的写报错那一支，`crates/singlefs-core/src/transaction.rs:624` 起，不是第 609 行的屏障那一支，也不是第 635 行写后屏障那一支）。我逐行复跑：

| 行 | 红在 | 原样输出里的关键一句 |
|---|---|---|
| 2278 | `a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write` | `assertion left == right failed: 写账合计等于落下的写减去取号那两次（…）`，`left: 388` |
| 2279 | `…_on_its_journal_record_write_…` | 同一句，`left: 388` |
| 2280 | `…_on_its_root_slot_write_…` | 同一句，`left: 388` |
| 2281 | `a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_hands_back_every_landed_write` | `assertion left == right failed: 之前落盘的 11 份（写行、暖机、准入推的两次、续做 7 步）、之前推成的是准入那一串、先写系统配置那一步落了 1 个写`，`left: (11, [2], 0, 11)`（要的第三项是 1） |

改坏的都是名字说的那一处。**新用例比旧用例多管了一块**：把 2278 那处替换套在整份 `admission_raises_the_floor_before_refusing` 上跑（`mut/n2.orig`、`mut/n2.repl`），`22 passed; 3 failed`，红的正是新增的三条，已有的「注错点摆在发布第一个写」的几条（如 `a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`）不红——第一个写失败时失败账里本来就没有已落下的写，旧用例分辨不出这个变异，新用例分辨得出。

**判定：一致**（U8、N2 都点名了会红的用例、改坏的是名字说的那一处、复跑红）。

**什么现象会推翻它。** 这四行变异里有一行套上去命中 0 次或多次（我复跑过，1 次）；或者有一个错的实现（失败账里重复算已落下的写、多算一份）让这些用例仍然过——这些用例只钉了「少算」，「多算」没有变异钉，我没造。

### N5　GB 与文档

- GB：`mount.rs` 写丙 1 的几处改成 GA。冻结副本里 `mount.rs` 只剩一处写「丙1」，是对岔路单那一格的指称：「用户 2026-10-03 定「甲 GA」，`research/prompts/m3-unlink-code-r3-forks.md` 丙1 单盘幸存那一格」（`crates/singlefs-core/src/mount.rs:2335`），指的是那一格的名字，不是旧结论。其余几处都写成 GA：`mount.rs` 的 936、965、2457、2887、4785、4984 行都有「定甲 GA」或「当场放开的那一截」。核法：`grep -rn '丙 \?1' --include=*.rs` 在 `singlefs-core`、`singlefs-harness` 下命中的全是描述旧做法或旧名的注释（`a_lone…` 两条的判别力说明、`admission_raises…` 里解释「原来钉的是丙 1 的说法」），没有一处把丙 1 写成现行做法。
- 「只差这两样」改成「三样」：「两个入口只差三样：」（`crates/singlefs-core/src/mount.rs:2317`）与「两个入口只差 [`RaiseFloorEntry`] 那三样：」（`crates/singlefs-core/src/mount.rs:2789`）；三样是：判不判上限（`match entry` 里准入判、卸载不判，`mount.rs:2850` 起）、根带不带卸载记号（`unmount_marker`，`mount.rs:2328` 起）、回收出来的槽什么时候能发（`reuse_of_the_reclaimed_slots`，`mount.rs:2343`），与代码对得上。冻结副本里「只差这两样」零命中。
- 「记录里没有 ctime」那句改对：`grep -rn '没有 ctime' --include=*.rs` 零命中；改成的话是「写入时间另由 [`the_intent_writes_the_given_time_into_the_three_time_fields_and_a_middle_batch_copies_it`] 判」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:466`，下一行接「三个秒字段共写一个 `write_time_seconds`」），与 `records.rs:77–78` 的三次 `put_u64(self.write_time_seconds)` 对得上。同一处被删掉的断言「写入时间不动」也不在了，换成 N3 的新用例。

**判定：一致。** **什么现象会推翻它。** 冻结副本里还有一处把丙 1 或「ctime 不改」写成现行做法的注释（我 grep 过 `.rs` 与 `.tsv`，零命中；`.md` 与 kb 没查，kb 归写回那一批）。

### N6　回扫旧树值：只改注释、消息、名字

我把 Q 的补丁与 P 补丁里属于回扫的几处逐块看了，加冻结副本里读到的值：

| 文件 | 改了什么 | 行为变没变 |
|---|---|---|
| `allocator.rs` | 两处 `///` 说明：「字节表五：28 条里 m1 / m2 各两条，分配代 0」（`crates/singlefs-core/src/allocator.rs:1356`）、「新池新建文件的前七个，字节表零 t2..t8」（`crates/singlefs-core/src/allocator.rs:2592`） | 没变（只动 `///`）。28 与布局文件对得上：`分配记录 28 条 = 14 个单元 × 2 盘（m1、m2、t1..t12 各一条`（`.claude/kb/layout/01-first-txn.md:85`）。「t2..t8」与那个用例分配的 7 个落点（1 槽、2 槽、5 个 1 槽，游标停在 50249）个数对得上，我没逐格核 t2..t8 与它们的对应 |
| `recovery.rs` | 下限常量与它两个使用处的说明；加一条 TODO（`crates/singlefs-core/src/recovery.rs:4265`） | 常量值没变：「const NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE: usize = 10;」（`crates/singlefs-core/src/recovery.rs:4266`）。TODO 写了缺什么（下限没随分配记录树按位置寻址抬上去）和为什么没做（没有条款），但**没指向欠账表**：`.claude/singlefs-ai-sop/rules/code-discipline.md:183` 要的是「欠着的检查记进项目 kb 的欠账表，代码里指过去」，`checks-owed.md` 里目前没有写这个常量的行（背景材料写「记欠账」，还没落表） |
| `transaction.rs` | 模块头与几处文档的八→十二；`expect` 消息「树表第 1 版起恒有七条」→「树表第 1 版起恒有条目」 | 只改消息文字；全仓没有 `should_panic` 或断言匹配这句（`grep` 零命中）。「十一条同一个数」对得上 `NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11`（`crates/singlefs-format/src/lib.rs:188`） |
| `mount.rs` | `RollbackError` 一个成员的文档：「回退目标那一版的十二棵树的号与现行那一版的不同」（`crates/singlefs-core/src/mount.rs:5538`） | 只改文档。十二对：该比较的是 `FileVersionTreeIdentifiers`，十二个字段（含不进树表的中央映射树），「带文件的一版那十二棵树各自的树 ID」（`crates/singlefs-core/src/transaction.rs:2549`），水位从 65536 到 65548 差 12：`TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65548`（`crates/singlefs-format/src/lib.rs:207`） |
| harness 几份 | 「A 的十二个落点」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:511`）、「字节表零 t1..t12」（`crates/singlefs-harness/tests/write_accounting_per_structure.rs:182`）、`writable_mount_of_a_formatted_pool.rs` 里局部变量改名与断言消息的八→十二 | 只改注释、变量名、消息文字；三份基线绿（12、3、15 条） |
| `unit_area_start_follows_the_ring…rs` | `= 7` 的局部常量换成格式常量：「usize::try_from(NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES)」（`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:590`），值 11 | 用到它的地方只有 `index_of_the_entry_of_kind` 的 `(0..N).find(…)`：它按「种类」找第一个命中的下标，两个调用点找的种类是 livelist、稀疏旁表（`…not_passed_over.rs:689`、`:693`），都在树表前 7 条里（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist），`find` 在第 7 条之前就停，7 换成 11 走不到新增的 4 条；找不到才会不同（旧版 panic、新版也许找到后面的），而这两种都在前 7 条里。基线 24 条绿 |
| `mutations.tsv` | 「C511 第 3 步…八棵树…」一行说明列改名（删旧名追加新名） | 我把旧行与新行逐列比：六列里只有第 0 列（名字）不同，原文、替换、命令、用例四列逐字相同；行 2282 复跑红在 `attempt to add with overflow`（副本里报的行是变异后的 2612 行；冻结副本里就是「TreeIdentifier(tree_identifier_watermark + offset_from_the_watermark)」那一行，`crates/singlefs-core/src/transaction.rs:2615`），与这行变异的意思相同（水位离上限不到十二个号时越界） |

**判定：一致**（行为没变，一处小缺：recovery.rs 的 TODO 没指向欠账表）。

**什么现象会推翻它。** 有一条 `.rs` 断言或 `should_panic` 匹配被改掉的消息文字；`NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES` 与实际树表条数不等（那样 7→11 才会走到新增的下标）。

## 二、K3　被判的 21 份文件，一份一格

`+`/`-` 是补丁里该文件的增删行数（`P.patch`、`Q.patch` 用脚本数的，变异表不在两份补丁里）。「基线」是冻结副本拷贝上该文件所属的 harness 档目标的结果（核心文件看它压着的那几个目标）。判定三档：一致 / 冲突 / 规则没说。本轮没有「冲突」。

| # | 文件 | 谁、改了多少 | 改了什么 | 对上的定案或条款，钉它的 | 判定 |
|---|---|---|---|---|---|
| 1 | `crates/mutations.tsv` | 变异表（不在补丁里） | 追加 10 行（2273–2282）；步 5 一行（46）与丙 1 四行（2174–2177）换成新枚举名；旧 C511 行删旧名追加新名 | 10 行 + 5 行复跑全红在点名的用例上，锚点各命中 1 次（第一、二节）。表里同一段 diff 里另有 C670、C673 ④ 等行（形式化接入那条线），不在本轮，没判 | 一致。小注：没有变异钉「被抛弃根的过滤」与「某块盘无有效根」两处 |
| 2 | `crates/singlefs-core/src/allocator.rs` | Q，+2 −2 | 两处 `///` 说明里的旧数 | 数对上 `.claude/kb/layout/01-first-txn.md:85`；`-p singlefs-core --lib` 180 条绿 | 一致 |
| 3 | `crates/singlefs-core/src/mounted_session.rs` | P，+16 −4 | `UserChange::Unlink` 加 `write_time_seconds`，会话交给意图 | D8 第 195、470 行；`unlink_mount_continuation…` 14 条绿，变异 2275、2276 红 | 一致 |
| 4 | `crates/singlefs-core/src/mount.rs` | P，+97 −24 | GA（新枚举、取小函数、回收分两次）；GB 文档；「三样」；`RollbackError` 文档 | D16 第 42 行；`a_lone…` 两条 + 变异 2273、2274、m1、m4–m8 红；`admission_raises…` 25 条绿 | 一致。规则没说两格：某块盘无有效根整截扣、读不出的槽缺席（保守） |
| 5 | `crates/singlefs-core/src/recovery.rs` | Q，+6 −3 | 下限常量两处说明 + 一条 TODO，值 10 不变 | `-p singlefs-core --lib` 180 条绿；`unit_area_start…` 24 条绿 | 一致。小缺：TODO 没指向欠账表（`code-discipline.md:183`），欠账还没落表 |
| 6 | `crates/singlefs-core/src/transaction.rs` | P，+17 −13 | 意图加 `write_time_seconds` 参数并写进记录；批的注释；模块头与文档八→十二；`expect` 消息 | D8 第 195、470 行；变异 2275、2276 红；`unlink_idempotence…` 5 条、`container_trees…` 4 条绿 | 一致 |
| 7 | `crates/singlefs-harness/src/history.rs` | P，+4 −2 | `apply_unlink` 加时间参数，取法同覆盖写、建 inode | `unlink_in_histories_judged_by_the_model` 8 条绿、`random_histories` 23 条绿（慢档 18 条没跑）；模型与 checker 都不读时间 | 一致。缝：时间在随机历史里没人看 |
| 8 | `crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs` | P，+309 −28 | N2 四条新用例 + 一个通用注错 helper；GA 结局的断言；各处传时间 | 25 条绿；变异 2278–2281 各红在点名的那一条；2174–2177、m1、m7 红 | 一致。注：GA 结局那几条断言不分辨丙 1（GA 换成丙 1 后 25/25 仍绿，我量的） |
| 9 | `crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs` | P，+18 −2 | 两处意图调用加时间参数（取 `pool.write_time_seconds`） | 5 条绿 | 一致 |
| 10 | `crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs` | P，+4 −0 | 意图调用加时间（取上一版记录的） | 5 条绿 | 一致 |
| 11 | `crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` | P，+353 新建 | h2、h4 单盘幸存读回 | 判决第三节第 2 条；基线 2 条绿；2273、2274 红在 `MappingStillUnreadable`；全扣到生效的变异两条仍绿 | 一致。注：h2 整读的活文件集合为空 |
| 12 | `crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs` | P，+5 −1 | 提一个常量，意图调用加时间 | 4 条绿；U8 变异 2277 红在这里的 `the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place` | 一致 |
| 13 | `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` | P，+2 −0 | 意图调用加时间 | 33 条绿 | 一致 |
| 14 | `crates/singlefs-harness/tests/overwrite_in_one_instance.rs` | Q，+1 −1 | 一处 `///` 八→十二 | 12 条绿 | 一致 |
| 15 | `crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs` | Q，+6 −7 | `= 7` 的局部常量换成格式常量（值 11） | 24 条绿；`find` 在前 7 条内停，结果不变（N6） | 一致 |
| 16 | `crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs` | P，+4 −0 | 意图调用加时间 | 5 条绿；①（计划相等）仍成立 | 一致 |
| 17 | `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | P，+132 −19 | 新用例（按盘读三个时间秒）+ 两个 helper；删掉「写入时间不动」断言；文档改对；各处传时间 | 14 条绿；①b 用例（687 行）绿；变异 2275、2276 红 | 一致。缝：挂载续做那一支的时间拷贝仓里没用例钉（我的探针量过） |
| 18 | `crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs` | P，+4 −0 | 意图调用加时间 | 1 条绿 | 一致 |
| 19 | `crates/singlefs-harness/tests/unlink_single_file_publishes.rs` | P，+5 −0 | 意图调用加时间（`map_or`，已删完的一格给固定值，意图在任何写之前被拒，不落盘） | 9 条绿 | 一致 |
| 20 | `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs` | Q，+5 −5 | 局部变量改名、断言消息八→十二 | 15 条绿；变异 2282 红 | 一致 |
| 21 | `crates/singlefs-harness/tests/write_accounting_per_structure.rs` | Q，+1 −1 | 一处 `///` 说明 | 3 条绿 | 一致 |

核对文件数：`cut -d' ' -f3 research/prompts/m3-batch14-code-r1-snapshot/crates-sha256.txt | wc -l` 是 21，上表 21 行，行与名单一一对应。

P 的报告标题写 P 还改了 `model.rs`，两份补丁里没有它的改动，`model.rs` 也不在 21 份里；我没读它的改动，不判。

## 三、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 GA | 一致 | 定案 → D16 第 42 行 → `mount.rs:2346`、2380–2391、2947–2967；GC 两条与变异 2273、2274、m1、m4–m8 钉住；准入臂有钉；树表 0 条一支走不到。条款没写的两格（无有效根整截扣、读不出缺席）记「规则没说」；被抛弃根的过滤与无有效根那一格，12 个会卸载的 harness 目标里没有一条会红 |
| K1 N2 GC | 一致 | 两条都判用户看得到的只读挂载与读回；丙 1 红、GA 绿、改之前绿都量到；h2 整读的活文件为空；`admission_raises…` 里钉 GA 结局的断言不分辨丙 1（量过） |
| K1 N3 Q-F 乙 | 一致 | 意图带调用方的时间、三格同值、各批照抄、续做不另取；变异 2275、2276 红；①b 用例绿；挂载续做拷贝时间一支我用探针量过、仓里没钉；模型与 checker 不读时间 |
| K1 N4 U8 与 N2 注错 | 一致 | 五行变异改坏的是名字说的那一处，复跑全红；新注错用例比旧的多管「失败那次已落下的写」 |
| K1 N5 GB 与文档 | 一致 | 丙 1 只剩对岔路单格的指称；「三样」与代码三处不同对上；「没有 ctime」零命中 |
| K1 N6 回扫 | 一致 | 只改注释、消息、名字；`7`→`11` 不改结果（`find` 在前 7 条内停）；一处小缺：TODO 没指向欠账表 |
| K3 21 份 | 21 份一致，0 冲突 | 逐份见第二节，二处「规则没说」、一处小缺 |

## 四、没做什么

- 不判 K2：GA 的扫、单盘恢复择中被抛弃根的那一形、根环有洞、卸载—再挂循环、Q-F 乙的逐字节、模型不比时间的缝，都归攻方腿；我没跑任何放开扫。
- checker 档一条没跑：`crates/singlefs-checker-tier` 的测试、crash_injection 那几条（7 行相关变异）、层 0 枚举、E158 之类。只做了 `cargo check -p singlefs-checker-tier --all-targets`（退出码 0，不含 required-features 的目标）。所以 checker 档里用 `HistoryOperation::Unlink` 的地方受不受时间取法影响，我复核不了。
- 没读主仓的 `crates/`（只用 `git show HEAD:` 试过取基线，HEAD 落后很多批、不能当基线，没用）；没读禁读清单里的文件；没查 kb 与别的 `.md` 里还有没有写着旧说法（kb 写回归 kb-scribe 那一批）。
- 没有把 harness 档 141 个目标全跑：跑了 24 个目标（21 份里的 14 份测试文件各自的目标、`model_answers_unmount…`、`unlink_in_histories_judged_by_the_model`、`random_histories`、`reuse_after_raising_the_floor`、`unmount_marker…`、`rebuilt_root_takes…`、三个 `entries_after_…`、`checker_known_bad_images`），加 `-p singlefs-core --lib`。变异只跑了与本批有关的 15 行表里的行（2273–2282、46、2174–2177）和自写的 m1–m8 八个；表里其余行不归本轮。
- 自写变异 m1–m8 的结论只对我跑的目标有效：m2、m3「没有一条红」是对 12 个目标的结论，checker 档的目标没跑。
- 不替主 agent 采纳、不出判决。

## 五、复核不了的事实、要主 agent 跑的命令

- checker 档（crash_injection 的 7 行变异、层 0 枚举里 `HistoryOperation::Unlink` 那几处、E158）：归提交时。命令照 `.claude/skills/crash-test/commit-stages.md`，我不起。
- 没有放后台等的命令。
- 模型目录 `research/prompts/m3-batch14-code-r1-sonnet-model/`：`rerun.sh <空工作目录> quick` 在空目录里从头跑通一次（`logs/rerun-quick.log`，1 分 30 秒）；`full` 模式（15 行变异加 m1–m8 加两个探针）要分几次前台跑，每次不超过 240 秒，没有一次性跑过。
- 草稿目录 `/tmp/claude-1000/m3-batch14-code-r1-sonnet/` 里我建的编译目录与仓副本（`w/`、`target/`、`rerun-empty/`、`rev/`、`head/`）交回前已删；留下的只有脚本、补丁、日志（都已拷进模型目录）。要再跑用 `rerun.sh`。

模型目录里每个文件的作用：`anchors.py` 锚点核；`mutrow.py`／`run-row.sh` 套一条变异、跑、还原、核还原；`mut/*.orig|*.repl` 自写变异的原文与替换文（m1–m8、`n2`、`ga`、`fullhold`）；`probe-*.sh` 两个探针；`tests.args` 与 `logs/*`（`battery.log` 第一批 8 个目标、`battery4.log` 第二批 4 个目标、`rows.log`、`rows2.log`、`base1.log`、`gc-base.log`、`hist.log`、`core-lib.log`、`fullhold.log`、`ga-on-admission.log`、`n2-on-admission.log`、`rerun-quick.log`、`check-checker-tier.log`）是跑出来的原样输出；`patches/P.patch`、`patches/Q.patch` 是从背景材料 diff 文件里切出来的两份补丁（反向套用：`patch -p1 -R --dry-run < P.patch`，在冻结副本的拷贝根上）。
