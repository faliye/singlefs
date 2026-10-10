# 删除批第 16 批代码轮第一轮：云端正推腿报告（K1、K3）（2026-10-04）

立场：从用户定案、判决行、kb 条款推到冻结副本 `/tmp/claude-1000/m3-batch16-code-r1/tree/` 里的行，再推到钉它的测试与变异行。引代码写冻结副本里的文件:行号；引 kb 写 kb 文件自己的行号（引用前逐句 `grep -nF`）。禁读清单里的东西没读。

冻结副本 6 份被判文件的 sha256 我重算过，与 `research/prompts/m3-batch16-code-r1-snapshot/crates-sha256.txt` 逐行 `diff` 无差（输出「SAME」）。

复核用的拷贝：`/tmp/claude-1000/m3-batch16-code-r1-sonnet/w/`（冻结树拷贝，变异只施加在它上）。模型与脚本在 `research/prompts/m3-batch16-code-r1-sonnet-model/`（`rerun.sh`、`apply.py`、`last-run-summary.txt`），只跑 harness 档与 core 库单测，经 `run-with-memory-cap.sh 8G` 加 `capped.sh 15`。

## 一、N1 甲：可写挂载重建分配器只放开一截

### 1.1 条款与用户定案推到代码

- 用户定案（岔路单 `research/prompts/m3-batch14-code-r1-forks.md` 第 1 行，用户 2026-10-04 选甲）：可写挂载重建分配器时，当场可用的只到 min(F_生效, 每块盘上最新有效根的 txg 最小者)，其余扣到这次挂载的暖机在每块盘上都落了根；某块盘一条有效根都读不出时照今天的门槛。
- 条款（D16 已定项 1「正常卸载」那一行）：「重建分配器时当场可用的只到释放代不高于 min(F_生效, …) 的槽，其余扣到这次挂载的暖机在每块盘上都落了根，某块盘一条有效根都读不出时照…那一行的门槛（用户 2026-10-04 定，零轮）」（`.claude/kb/decisions/16-发布语义.md:42`）。
- 「可再分配」那一行：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根) |」（`.claude/kb/decisions/16-发布语义.md:33`）。

代码三处（冻结副本）：

1. 当场放开：「let floor_released_immediately = match newest_valid_root_txg_on_every_device(」（`crates/singlefs-core/src/mount.rs:1871`）。有 m 时：「Some(newest_valid_root_on_every_device) => reclaim_floor(」（`crates/singlefs-core/src/mount.rs:1877`）、「effective_floor.min(newest_valid_root_on_every_device),」（`crates/singlefs-core/src/mount.rs:1878`）、「oldest_valid_root_in_the_window,」（`crates/singlefs-core/src/mount.rs:1879`）。没有 m 时：「None => floor_of_the_reclaim,」（`crates/singlefs-core/src/mount.rs:1881`）。然后「allocator.reclaim_released_up_to(floor_released_immediately, ReclaimedReuse::Immediately);」（`crates/singlefs-core/src/mount.rs:1883`）。
2. 扣住那一遍：第 1884–1887 行用原门槛「let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_window);」（`crates/singlefs-core/src/mount.rs:1870`）和「ReclaimedReuse::HeldUntilFloorTakesEffect,」（`crates/singlefs-core/src/mount.rs:1886`）。已回收的槽不重复回收，所以这一遍只回收当场放开那一截之外、到原门槛为止的那一截。
3. 放开：「allocator.release_reclaim_holds();」（`crates/singlefs-core/src/mount.rs:4748`），在暖机循环「for planned_txg in &warm_up_publishes_planned {」（`crates/singlefs-core/src/mount.rs:4725`）整个走完之后。`rebuilt_allocator` 只有一个调用点：「let rebuilt = rebuilt_allocator(」（`crates/singlefs-core/src/mount.rs:5620`），之后进 `establish_instance`。暖机每次发布都用 `?` 往外报错，走到 4748 就是暖机全部落盘。

### 1.2 小项判定

**a. 当场放开的是不是 min(F_生效, m) 以下：一致，附一处条款没写的 o 项，和与卸载一格不同形的一处差异。**

- 代码取 a = max(min(F, m), o)，o 是一圈窗口里最旧有效根：「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」（`crates/singlefs-core/src/mount.rs:1351`），重建把 `min(F, m)` 交给它当 `effective_floor`。条款字面只有 min(F, m)，没有 o。
  - 字面读法放进普通重挂载（F = 0，o > 0）会让释放代在 (0, o] 里的槽都被扣到暖机之后，与「可再分配」那一行（kb 第 33 行）矛盾，也与用户那一句「照今天的门槛」不合；所以代码把 o 并进来是把原门槛里的 F 换成 min(F, m)，我读作条款没写的省略，不是错。条款这一句没说 o 怎么并，算「规则没说」，交主 agent 要不要补一个字。
  - F ≤ m 时 min(F, m) = F，当场放开就是原门槛，第二遍回收不出任何东西：普通重挂载（没有卸载半途崩溃）行为不变（推的；代数见下）。
- 与卸载那一格不同形（并排）：
  - 重建：「effective_floor.min(newest_valid_root_on_every_device),」（`crates/singlefs-core/src/mount.rs:1878`），外面再套 `reclaim_floor(…, o)`，即 max(min(F, m), o)。
  - 卸载：「floor_of_the_reclaim.min(newest_valid_root_on_every_device),」（`crates/singlefs-core/src/mount.rs:3084`），即 min(max(F, o), m)。
  - 我对 F、m、o 各 0–7 的 512 个三元组穷举（python，输出「224 288 True True」「224 224」）：两式不等的恰好是 o > m 的 224 个，o ≤ m 的 288 个全相等；不等时重建那一格更大。
- o > m 的前提：那块盘上最新有效根 m 落在一圈窗口之外，且窗口里它的槽没有读不出的（读不出的槽会把 o 拉低）。我没造出这样的历史，也没找到不被「落后」那几道拒掉的入口；交攻方腿的第二次抽样去看。
- 推翻条件：造出一份 o > m 的盘面，丢另一块盘后剩下那块盘（根 m）引用了释放代落在 (m, o] 里的槽，而再挂那次把它写掉——那就是甲在这一域里漏了一截，两式不同形成了真缺口。证明 o > m 在可达状态上不出现，这一条就降成纯文字差异。

**b. m 是不是与 GA 同一处定义：一致（同一个函数），调用处各写各的。**

- 同一个函数：「fn newest_valid_root_txg_on_every_device(」（`crates/singlefs-core/src/mount.rs:2475`），重建调用在 1871，卸载调用在 3077（「ImmediatelyBelowTheNewestValidRootOnEveryDevice => {」，`crates/singlefs-core/src/mount.rs:3076`）。过滤被抛弃的根在函数里：「!abandoned_by_table(root, table)」（`crates/singlefs-core/src/mount.rs:2487`）。
- 不是同一处的三样：`device_of_txg` 闭包各写一份（重建按择到的系统配置的区域归属表，卸载按 `parameters_of_the_pool`）；`roots` 的来源（重建是每槽读一次的整遍根环，卸载是「let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(」，`crates/singlefs-core/src/mount.rs:3015`）；`table`（重建是 `choose_root` 择中那条根指着的表，「let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);」，`crates/singlefs-core/src/mount.rs:1826`）。
- 推翻条件：两处 `device_of_txg` 对同一份盘面给出不同归属（读两处的闭包没看到差异）。

**c. 扣住的那一截在哪一步放开，与「暖机在每块盘上都落了根」对不对得上：一致；没有任何测试或变异钉它。**

- 暖机计划「fn warm_up_publish_txgs(」（`crates/singlefs-core/src/mount.rs:3675`）从写行那次的 txg 起逐个加一，直到「let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish_txg)];」（`crates/singlefs-core/src/mount.rs:3686`）起算的覆盖集盖住全部盘，至多根环区域数次。放开排在循环之后，与条款的「暖机在每块盘上都落了根」同时。
- 扣住位只有两处会置，`release_reclaim_holds` 全部调用只有 3204（抬 F 末尾）与 4748 两处；抬 F 的入口都排在 4748 之后（`push_floor_raises_after_the_row_publish` 的调用在 4798，取号之前的预演在拷贝上做），所以 4748 放开的全是重建那一处置的。
- 实测（`rerun.sh x1`，`last-run-summary.txt` 的 x1 段）：把 4748 那一句换成注释，13 个相关 harness 目标（含三个本批目标、`admission_raises_the_floor_before_refusing`、`writable_mount_of_a_formatted_pool`、`whole_device_loss_after_warm_up`、`admission_formula_on_the_mounted_state`）全部 exit=0。x3（重建里扣住的那一遍整句去掉）与 x2（`None` 那一支改成当场放开到 0）各跑 5 个目标，也全绿。这三处都是有行为的代码，当前只靠读代码。与实现员 P 自报「没有变异钉住暖机之后放开那一句」一致。
- 推翻条件：有一个 harness 目标在 x1 下红（我少跑了一个目标，则 1.2c 的「没有钉」降成「钉在别处」）；或读 4438–4748 行发现取号到暖机之间有一次抬 F 置了扣住位（我读了没有）。

**d. 某块盘没有有效根时的取法：与条款对得上；这一支没有钉。**

- 条款：「某块盘一条有效根都读不出时照…那一行的门槛」（`.claude/kb/decisions/16-发布语义.md:42`）。代码：「None => floor_of_the_reclaim,」（`crates/singlefs-core/src/mount.rs:1881`），当场放开就是原门槛，第二遍什么也不回收。
- 卸载那一格同一情形的取法不同，也与条款对得上（条款写「这一截为空、整截扣到生效」）：「None => Vec::new(),」（`crates/singlefs-core/src/mount.rs:3087`）。
- 钉：卸载那一格有（变异表第 2387 行，`last-run-summary.txt` 的 rows 段里 row2387 红在 `when_no_valid_root_on_device_one_is_read_during_the_unmount_every_reclaimed_slot_is_held_and_device_one_reads_back_its_live_file`）；重建那一格没有测试也没有变异：x2 把它改成 `None => CheckpointTxg(0)`，5 个目标全绿。
- 这一支在重建里什么时候走到：m 取不到 ⇔ 某块盘上读出来的、按择到那条根的实例表判有效的根一条都没有。我没有逐条看可写挂载的入口能不能让这种盘走到重建（`devices_without_the_selected_version` 在重建之后才判），可达性未核；交攻方腿。
- 推翻条件：一个 harness 目标在 x2 下红；或造出重建里 m 取不到、且改回条款字面外的取法会改读回结局的历史。

**e. 准入、卸载两格有没有被碰：卸载一格的代码没动，准入一格的条款没写扣住位对挂载处的影响。**

- 卸载：补丁在 `newest_valid_root_txg_on_every_device` 上只改了文档注释（diff 第 2436 行那一块只有 `///` 行），调用处 3076–3095 原样。
- 准入：代码没动，但重建新置的扣住位让落点能不能取到的读数变少：「slots_for_commit_generated_units: device_map.unblocked_slots(),」（`crates/singlefs-core/src/admission.rs:741`），而「!self.held_until_floor_takes_effect[index]」（`crates/singlefs-core/src/allocator.rs:436`）把扣住位算作挡。所以只在 F_生效 > m 的挂载里（形 R 那一类起点），取号之前的预演与准入读数比放开前更紧。准入那一行（kb 第 43 行）没写这一点，算「规则没说」。F ≤ m 的挂载里没有新增的扣住位（1.2a）。扣住让再挂在预演里被拒的格子数是攻方腿的事，我没量。
- 我没逐行读 `admit_on_every_device` 的空闲字节口径（扣住的槽算不算空闲字节），不判。

**f. 钉 N1 的测试与变异行（对着 1.2a–d 汇总）**

- 变异表第 2386 行（把 `effective_floor.min(…)` 改成 `effective_floor`，当场放开回到原门槛）：`rerun.sh rows` 里 row2386 红，两条形 R 用例都红（表里只点了 h2 那一条，4 GiB 那一条也红）：
  `test after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads ... FAILED` 与
  `test after_a_crashed_unmount_of_a_four_gibibyte_pool_remounted_once_the_next_writable_mount_crashing_after_its_thirty_third_write_keeps_what_device_one_reads ... FAILED`（`last-run-summary.txt` row2386 段）。
- 没钉的：暖机之后放开（x1）、扣住那一遍（x3）、重建的 `None` 支（x2）、o 项（没做对应变异；我没把 `reclaim_floor(…, o)` 的 o 去掉跑一遍，没量）。

## 二、N2 / N6：形 R 两条读回用例、GC 活文件不为空

**N2：判的是用户看得到的读回结局，一致。**

- 两条用例：「fn after_a_crashed_unmount_of_a_narrow_pool_whose_deletion_was_continued_the_next_writable_mount_crashing_in_its_row_publish_keeps_what_device_one_reads(」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:280`）与 4 GiB 那条（同文件第 356 行）。
- 判的是：卸载只落第一个写就崩、两盘再可写挂载、挂载里再崩、丢盘 0 之后，剩下那块盘只读挂载并整读每个活文件，与卸载之前同一块盘上读出的相同：「survivor.after_the_crashes, survivor.before_the_unmount,」（`crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs:269`）。这是判决正文第五节说的结局（只读挂载做不做得成、整读活文件与卸载之前相同），不拿池级 checker 分辨。
- 比的结构里带 `root_txg`，用户并看不到根 txg：两次崩溃之后剩下那块盘若走到一条更新的、内容相同的根，等式会误红。当前两个固定刀点不会（本批跑绿），但这一条断言比「读回结局」略紧一点，且刀点只有两个（卸载落 1 个写；再挂落 6 个或 33 个写；丢盘 0）。放开扫是攻方腿的事。
- 判别力我复跑了：变异 2386 下两条都红（上一节 1.2f），baseline 两条绿（`last-run-summary.txt` 的 baseline 段：2 passed）。
- 推翻条件：形 R 的某个刀点在甲之下读不出而这两条绿（攻方腿的扫描）；或这两条在一个读回结局正确的盘面上红（`root_txg` 那一项造成）。

**N6：判的是读回结局，一致。**

- 窄盘崩了再挂一次之后卸载、掉电在第二次空发布的单元写之后，剩下的盘 1 整读第一个文件：「.is_ok_and(|whole_read| !whole_read.live_files.is_empty()),」（`crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs:541`）先断言卸载之前读得出活文件（这一形要判的就是活文件不为空），再走共用的等式：「survivor.after_the_crash, survivor.before_the_unmount,」（`crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs:313`）。
- 变异表第 2389 行（卸载一格整截当场放开，改回丙1）下：这条与另外两条（h2、4 GiB 第二次卸载）一起红，共 3 条红 1 条绿（row2389 段），点名的那一条在其中。
- 推翻条件：把丙1 改回去而这条绿（没发生，我跑了）；或这条用例的活文件在掉电之后仍读得出但内容被写掉而等式没抓（等式比的是逐字节内容 `Vec<u8>`，不会漏）。

## 三、N3：GA 文档里被抛弃根那句理由

**一致，附一处文档残留。**

- 判决要改的那一行（`research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「单盘恢复择中被抛弃的根」：旧理由「被抛弃的那条时间线接着那一版走，也不引用它」与 C314 那一形对不上，GA 的结论不靠它，改写那句理由）。
- 新理由：「- 被抛弃的根（按 `table`，`abandoned_by_table`）不算。卸载那一格滤不滤结局相同：卸载接在一次可写挂载之后，那次挂载的暖机先在每块盘上」（`crates/singlefs-core/src/mount.rs:2468`）、「落了新实例的根（D16（发布语义） 已定项 8 戊），新实例的 txg 大于环里每一条根（D23（journal 的角色与格式） 已定项 14 第 3 条），」（`crates/singlefs-core/src/mount.rs:2469`）。D23 已定项 14 第 3 条的原文：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:371`）；D16 已定项 8 的定案：「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回」（`.claude/kb/decisions/16-发布语义.md:203`）。理由推得通，而且把「滤」限在卸载那一格；它没说重建那一格：重建那一格的最新根可以是被抛弃的，这时滤才有用（见 N5）。
- 残留：同一段文档第 2474 行「- 有一块盘上一条有效根都没读出来：交回 `None`，这一串回收的槽整截扣到生效（不按 txg 0 去比，说清是哪一格）。」（`crates/singlefs-core/src/mount.rs:2474`）只对卸载一格为真；重建一格 `None` 是照原门槛放开（`crates/singlefs-core/src/mount.rs:1881`）。该函数现在两处共用，这句话读起来是函数的契约，实际是调用方各自的取法。只是注释，不影响行为。
- 推翻条件：卸载时某块盘上最新的根是被抛弃的（即新实例的暖机没在那块盘上落根，又走到了抬 F 那一串），那一句「结局相同」就不成立。

## 四、N4：「某块盘没有有效根时整截扣」补变异与用例

**条款对得上；强制进入的状态在入口走不到；断言还是保留分支，由主 agent 定。**

- 条款：「有一块盘一条有效根都读不出时这一截为空、整截扣到生效」（`.claude/kb/decisions/16-发布语义.md:42`）。代码：「None => Vec::new(),」（`crates/singlefs-core/src/mount.rs:3087`）。变异第 2387 行把它改成「照原门槛当场放开」，`rerun.sh rows` 红在 `when_no_valid_root_on_device_one_is_read_during_the_unmount_every_reclaimed_slot_is_held_and_device_one_reads_back_its_live_file`（row2387 段：3 passed、1 failed）。
- 用例怎么进这一支：改写本分配器「fn allocator_not_knowing_the_root_ring_slots_of(」（`crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs:342`）让进程不知道盘 1 的根环槽，加上两次卸载里盘 1 的根环槽每次读都失败。
- 状态在什么条件下可达（读代码）：卸载读根环走「let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(」（`crates/singlefs-core/src/mount.rs:3015`），进程知道住着根的槽读坏、重读仍坏，就在动分配器之前返回「MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))」（`crates/singlefs-core/src/mount.rs:3023`）。做过可写挂载的会话，暖机在每块盘上都写了本实例的根且被记成已知；mkfs 同一个进程的会话，mkfs 在三个区域各种一条第 0 代根（`crates/singlefs-core/src/make_filesystem.rs:672` 的「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」，区域归属 0 / 1 / 0 覆盖两块盘）并按同一份记进根环表。所以任一块盘都至少有一个已知槽，它读坏就拒，走不到 `None`。该盘的有效根被判成被抛弃也走不到：新实例的根不会被它自己那张表判成被抛弃（表里只有上一个实例的行）。结论：两条入口都走不到，与实现员报告一致。我没有穷举 `install_root_ring_occupancy` 以外别的装根环表的口子（公开方法 `install_root_ring_occupancy` 在 `crates/singlefs-core/src/allocator.rs` 里，测试能直接调）。
- 该不该改成断言：这一支是条款明写的取法（用户 2026-10-03 定），改成 `unreachable!`/断言要连条款那半句一起改；留着分支加强制进入的用例符合 `.claude/rules/fs-design.md` 五条硬要求第 2 条（每条分支必须能被测试强制进入）。我不替主 agent 判，只写：条款要求的是行为，代码里没有不可达的理由写成不变量。如果改成断言，变异第 2387 行与这条用例要一起删。
- 推翻条件：造出一条入口历史（不用改写本、不改 `install_root_ring_occupancy`）让卸载走到 `None`，则「走不到」不成立，用例就不是只守一个死分支。

## 五、N5：被抛弃根的过滤只在函数边界上加单测

**单测不能代表系统结局，也没人声称它能；它只钉函数语义，调用点接线没人钉。**

- 单测：「mod newest_valid_root_on_every_device_tests」（`crates/singlefs-core/src/mount.rs:6786`）里的「fn an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root() {」（`crates/singlefs-core/src/mount.rs:6821`）。手造的实例表和 `RootRecord`，调函数本身。变异第 2388 行（去掉 `!abandoned_by_table(root, table)`）下 `rerun.sh rows` 红（row2388：1 failed，`mount::newest_valid_root_on_every_device_tests::an_abandoned_root_newest_on_a_device_does_not_stand_for_that_devices_newest_valid_root`）。
- 系统层面（`rerun.sh x4`）：施加同一条变异 2388，13 个相关 harness 目标（含三个本批目标）全部 exit=0（`last-run-summary.txt` 的 x4 段 13 行）。没有任何 harness 目标分辨这个过滤，与判决「找不到敏感取样点」一致。
- 过滤在哪一格有结局（推的）：卸载那一格没有（N3 的理由）；重建那一格，某块盘上最新的根可以是被抛弃的——表里有上一个实例之前的行 (i, T_i)，那个实例在 T_i 之后写的根都是被抛弃的；新实例在挂载里写行之后、暖机还没在那块盘上落根就崩，再挂时那块盘最新的根就是被抛弃的。这时不滤的 m 会比滤了的大，当 F_生效 夹在两者之间（F_生效 要大于那块盘上最新有效根，被抛弃根带的 F 也算进 F_生效）才改当场放开的那一截。这是一个待攻方腿去造的候选形，我没造出来（`an_abandoned_roots_rollback_floor_counts_toward_the_effective_floor` 等相邻用例在 x4 下也全绿）。
- 调用点接线：重建传 `&newest_table`（`crates/singlefs-core/src/mount.rs:1826`），卸载传现行版本的表；若把重建一格的表换成空表，等价于关掉过滤，同样没有 harness 目标会红（x4 等价）。
- 推翻条件：造出一份入口历史，使 x4 下某个 harness 目标红（那就找到了敏感取样点，单测之外有了系统钉）；或证明过滤在可达状态上恒空转（那么它就是死代码、单测是等价变异的留档）。

## 六、N7：Q-F 两条新用例

**判的是条款说的三格；变异改坏的是名字说的那一处；我造不出两条都过的错实现。**

- 条款：「| 64 | atime / mtime / ctime 秒，各 8（unlink 的意图那次三个共写调用方给的一个值，各批照抄） | 24 |」（`.claude/kb/decisions/08-核心索引结构.md:188`）；「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同（意图那次与每一批的改动计数是本次发布的 checkpoint_txg；意图那次的三个时间秒由调用方给、共一个值，各批照抄，续做的几批不另取时间）」（`.claude/kb/decisions/08-核心索引结构.md:470`，该行片段）。
- 用例读盘上三格：「field_at(record, 64),」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:539`），同一函数里 72、80 紧随；按盘读单元字节，不是读内存里那份记录。
- 经会话那条：「vec![vec![[time_of_the_unlink; 3]; 2]; 3],」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:607`），意图与还带记录的两批，两块盘，三格；给的时间取 `上一版 + 600`，与池里原值不同。
- 挂载续做那条：「vec![vec![[time_of_the_intent; 3]; 2]; 2],」（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:634`），续做的两批照抄意图写的值。
- 变异（`rerun.sh rows`）：2390（会话丢时间）红 1 条（点名的）；2391（续做取 0）红 3 条（含点名的）；2392（续做取固定时间 1_788_000_000）红 3 条；2393（读回丢时间秒）红 1 条（点名的）。实现员 Q 自报的「续做批取固定时间」「读回时丢掉时间」两行归提交时没证红，我在 harness 档单跑那一个目标，两行都红。四行红的测试都含点名的那一条。
- 想错实现、两条照样过（我找过的）：
  - 只写 atime、mtime 不写 ctime：池里原值与给的时间不同，ctime 会留旧值，等式红。
  - 续做批另取时间：红（2391、2392）。读回丢时间：红（2393）。
  - 三个时间秒对、纳秒字段（偏移 96 起）错：两条都不比纳秒，会漏。条款写纳秒恒 0（`.claude/kb/decisions/08-核心索引结构.md:195` 一行里「今天的参数只带秒数，所以写 0」），这一格没有 Q-F 的用例，不在 N7 点名的三格里，记「没钉」。
  - 两条用例都在整次调用结束之后才读盘（Q 自报）：读的是各版本单元自己的槽，后发的版本都落新槽（回收槽在 defer 窗口里不复用），所以不会读到被后一版盖掉的内容；若实现把某一版的单元写进前一版同槽，会读到后一版的值，这一形不在 N7 内。
- 推翻条件：某个只改时间字段的错实现下两条同时绿；或在 harness 档重跑第 2390–2393 行有一行没红（`rerun.sh rows`，我跑了全红）。

## 七、N8：recovery.rs 那条 TODO 指到 C692

**一致，只改注释。**

- 冻结副本：「欠账在 `.claude/kb/checks-owed.md` C692（新池新建文件每盘落点下限偏松），定下取法之后照那一行改常量与它的单测。」（`crates/singlefs-core/src/recovery.rs:4266`）。补丁只加了这一行并把上一行句号换成分号，常量 `NEW_POOL_FILE_CREATION_PLACEMENTS_PER_DEVICE` 的值（10）没动。
- C692 在 kb 里存在：「| C692 | 新池新建文件每盘落点下限偏松 |」（`.claude/kb/checks-owed.md:548`）。
- 推翻条件：补丁在注释之外动了一个字（diff 全文只有这一块 `///`）。

## 八、K3：被判的 6 份文件，一份一格

sha256 与快照文件逐行相同（开头已记）。mount.rs 在冻结副本里还带着根环线、改钉簇与 9-9 合进来的改动，不属于这一批；这里只判第 16 批的几块（下列行号）。

| 文件 | 判到的块（冻结副本行号） | 判定 |
|---|---|---|
| `crates/singlefs-core/src/mount.rs` | 重建分配器 1855–1887（甲）；`establish_instance` 暖机之后 4741–4748；GA 文档 2465–2474；单测模块 6783 起（`mod newest_valid_root_on_every_device_tests`，6786） | **一致**，附三处：重建一格比条款字面多并进 o 项、与卸载一格不同形（1.2a）；文档第 2474 行只对卸载一格为真（N3）；4748 放开、重建的 `None` 支、扣住那一遍三处没有测试也没有变异（1.2c、1.2d） |
| `crates/singlefs-core/src/recovery.rs` | 4266 一行 `///`（N8） | **一致**，只改注释，C692 存在（`.claude/kb/checks-owed.md:548`） |
| `crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs`（新文件） | 全文 376 行，两条用例在 280、356 | **一致**，判读回结局；两个固定刀点；比的结构带 `root_txg` 略紧于用户看得到的读回（N2） |
| `crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` | 第 16 批补的两条：活文件不为空 527、没有有效根强制进入 555，改写本 342、整根环槽 385 | **一致**，N6 判读回；N4 的强制进入状态入口走不到（读代码），保留分支还是改断言由主 agent 定 |
| `crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs` | 两条新用例 581、617，读盘上三格的函数 507 起 | **一致**，三格 64 / 72 / 80 与条款字段表对上；四行变异在 harness 档都红；纳秒字段没钉 |
| `crates/mutations.tsv` | 第 2386–2393 行（8 行，六段制表符；2386–2389 是 P 的，2390–2393 是 Q 的） | **一致**：我用 python 对 8 行逐行数锚点命中次数，全是 1 次；`rerun.sh rows` 8 行全红（点名的测试都在红的集合里）；2386 表里只点了 h2 一条，4 GiB 那条也红 |

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 N1 a 当场放开到 min(F, m) | 一致；规则没说 o 项；与卸载一格不同形 | 代码取 max(min(F, m), o)，条款只写 min(F, m)；两式 o ≤ m 时相等、o > m 时重建更大（224 / 512 个三元组），o > m 的历史没造出 |
| K1 N1 b m 与 GA 同一处 | 一致 | 同一函数，闭包 / roots / table 三样各写一份 |
| K1 N1 c 扣住那一截放开的时机 | 一致，无钉 | 4748 在暖机循环之后；去掉它 13 个相关 harness 目标全绿 |
| K1 N1 d 某块盘没有有效根的取法 | 一致，重建一格无钉 | 照原门槛对得上条款；卸载一格有变异 2387 钉住，重建一格改成放开到 0 的 5 个目标全绿 |
| K1 N1 e 准入、卸载两格 | 卸载一致；准入规则没说 | 卸载只动注释；扣住位让 F > m 的挂载里落点读数更紧，准入那一行没写 |
| K1 N2 形 R 两条读回 | 一致 | 判读回结局；变异 2386 下两条都红；两个固定刀点 |
| K1 N3 GA 文档理由 | 一致，一处残留 | 新理由推得通；第 2474 行 `None` 整截扣到生效只对卸载一格为真 |
| K1 N4 强制进入没有有效根 | 条款对得上；入口走不到 | 卸载读根环的重读闸与 mkfs 的第 0 代根让该支入口走不到；断言还是保留分支主 agent 定 |
| K1 N5 被抛弃根过滤 | 单测只钉函数语义 | 变异 2388 只有该单测红，13 个 harness 目标全绿；重建一格的候选形留给攻方 |
| K1 N6 GC 活文件不为空 | 一致 | 判读回；变异 2389 下 3 条红含它 |
| K1 N7 Q-F 两条用例 | 一致 | 三格对；变异 2390–2393 全红；纳秒没钉 |
| K1 N8 TODO 指到 C692 | 一致 | 只改注释，C692 存在 |
| K3 mount.rs | 一致，附三处 | 见第八节 |
| K3 recovery.rs | 一致 | 只改注释 |
| K3 the_mount_after…rs | 一致 | 读回结局 |
| K3 a_lone_surviving…rs | 一致 | N4、N6 |
| K3 unlink_mount_continuation…rs | 一致 | N7 |
| K3 mutations.tsv | 一致 | 8 行锚点各命中 1 次，8 行全红 |

## 十、复核过的命令与原样输出

- 6 份文件的哈希：`sha256sum` 六行输出与快照文件 `diff` 无差（「SAME」）。
- 复跑（`research/prompts/m3-batch16-code-r1-sonnet-model/rerun.sh`，在空目录里从头跑，阶段分五次起，每次前台 < 240 秒）：原样输出整份在 `research/prompts/m3-batch16-code-r1-sonnet-model/last-run-summary.txt`（85 行，含 48 个 `exit=`）。baseline 段：三个目标 2 / 4 / 16 passed，`--lib` 1 passed；rows 段 8 行全 `exit=101`（红）；x1 13 行、x2 5 行、x3 5 行、x4 13 行全 `exit=0`（绿）。
- 开跑前 `ps` 看到的：主 agent 的一条 `cargo test --offline --no-fail-fast -p singlefs-format …` 在跑（经内存包装 8G），没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，所以加 `nice -n 19` 照常跑。有一次起 `run-with-memory-cap.sh` 退出码 252（会话里另一条在跑，包装排不上），那一次的输出不算结果；之后按包装说明设了并行理由重起，不是绕开包装。
- 复核不了的：第 16 批合入后 150 个 harness 目标全绿（主 agent 的 `merge-and-verify.sh` 日志），我只重跑了与这一批直接相关的 13 个目标，没复跑 150 个；checker 档（含变异整表的门禁回放）按规则不跑，变异第 2386–2393 行在 `mutate.sh` / 门禁回放下的结果我没有，只有 harness 档按行施加的结果。

## 十一、没做什么

- K2（哪一格会错：形 R 放开扫、第二次抽样、N7 错实现的放开扫）归攻方腿，我没扫；o > m 的盘面、过滤有结局的重建候选形、准入读数的空闲字节口径都标成「交攻方腿 / 没读」。
- 没替主 agent 采纳或出判决：N1a 的 o 项要不要补进条款、N4 的断言 / 分支取舍、N3 的文档残留要不要改，都只写了现象与推翻条件。
- 没判 mount.rs 里不属于这一批的改动（根环线、改钉簇、9-9、形式化接入）。
- 没跑 checker 档、`mutate.sh`、门禁阶段与各类 lint；没起后台任务；没动仓里的源码与 `.claude/` 下任何东西。
- 没有量代价数：「窄盘上扣住让再挂在预演里被拒的格子数」是攻方腿的事。
