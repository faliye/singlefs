# 第 32–33 批代码轮第一轮：云端正推腿报告（Sonnet，2026-10-06）

立场：正推。P1–P6 逐格从两份判决（`research/prompts/m3-formal-code-b30-r1-main-verification.md`、`research/prompts/m3-formal-code-b31-r1-main-verification.md`）、kb 条款、调查报告与用户定案，推每处改动做的是不是它们说的。
冻结副本 `/tmp/claude-1000/m3-formal-code-b33-r1/tree/`（sha256 清单 314 行，我拷到 `/tmp/claude-1000/m3-formal-code-b33-r1-sonnet/work/tree/` 之后逐行核过：`sha256sum -c` 314 行全 OK，每次变异撤回后再核一次仍 314 行 OK）。
主工作区 `crates/` 与冻结副本此刻逐字相同：`diff -rq crates /tmp/claude-1000/m3-formal-code-b33-r1/tree/crates` 无输出。
代码引文写 `crates/…` 加冻结副本里的行号；kb 引文写 kb 文件自己的行号（我逐条 `grep -nF` 过，命中行就是写的行）。
模型与脚本在 `research/prompts/m3-formal-code-b33-r1-sonnet-model/`，入口 `tools/rerun.sh`（段名 setup static base p1 p2 p3 p4 reuse sweep sweep-slow）；本报告里的读数都能由它复跑，原始汇总在该目录 `out/`。

## 零、方法与不作数的东西

- 全部是 harness 档：`cargo test -p singlefs-harness`（`--lib`、`--test …`）。每份副本自己的 `CARGO_TARGET_DIR=…/work/target`；每一趟编完核日志里被测包确有 `Compiling`（`tools/lib.sh` 的 `cg` 每段末尾打 `compiled:` 行；变异跑的日志里 `Compiling singlefs-checker`、`singlefs-core` 或 `singlefs-harness` 都在）。没有 `Compiling` 的趟不算：变异趟都有；仅有的无 `Compiling` 的两趟（`probe-p4-base-all`、`base-reuse-one`）是同一份源码上紧接着重跑已编好的二进制，不是变异趟。
- cargo 与编出来的二进制都经 `scripts/hosts/run-with-memory-cap.sh` 加 `scripts/hosts/capped.sh`（10 线程，扫整个套件时 3 个并行 × 3 线程），设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`。包装退 252（排不上）的那一遍扫描（124 个二进制）输出不算结果，没绕开包装，改成 3G 上限重跑到全部有结果。
- checker 档一律没跑：`singlefs-checker-tier` 的测试与装置内联单测、GPU 对拍、E156 装置、层 0、`crates/mutations.tsv` 整表。涉及它们的结论写「推的」。
- 33-b 那条 checker 档用例的造法，我把用例文件拷成 harness 档的探针（`probes/make_probe_rollback_floor.py`：删掉唯一的 `singlefs_checker_tier` 的 use 与最后一条用到它的用例，其余原样），在 `singlefs-harness` 包里编、跑。这不是 checker 档包自己的测试二进制；它回答「新造法在今天的 core 上量到什么」，不回答「checker 档那条用例本身绿不绿」。
- 冻结副本只有 `crates/` 与两份 Cargo 文件，几条用例读 `../../litmus`，我从主工作区拷了一份 `litmus/`（这一轮的改动不碰它）。
- 开工前 `ps` 看到别的会话的 `cargo test`（`-p singlefs-harness -- --ignored`、E171 的 python 驱动）在跑，没有 `qemu-system`、`vm-bench`、`e152`、`fio`；我用 `nice -n 19`，没有等锁。

## 一、先说三件主 agent 要知道的事

1. **冻结副本里抬 F 慢档那条用例今天是红的**（P3 末尾详写）：`raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it` 红在 I-9.18，四支豁免（I-5.5、I-4.7、I-4.6、I-4.4）之外还有 I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23 六条报不适用。与「在主工作区单跑 ok」对不上；`crates/` 两边逐字相同，机理是删除线的新不变量（HEAD 的 `crates/singlefs-checker/src/image.rs` 里 I-9.18 命中 0 次，工作区 6 次）。
2. **I-5.5 那支豁免认的理由是 checker 的缺省理由**，不是只有「没有对象」才会出的话（P3）。
3. **旧变异名被 E171 的坏实现表按名引用**：两份 `bad-implementations-117.tsv` 的第 33 行（P4）。

## P1　32-1 的 B3、B4

**判定：B3 一致；B4 一致（取样点之内），取样点之外有一处已知射程限度。**

题面：判决要的是「改法 B3（胶水映射逐片单测，k = 0..134 互不相同，攻方副本上量过两个胶水变体都红）与 B4（模型回退的结构单测，推的），零轮，并进第 32 批」（`research/prompts/m3-formal-code-b30-r1-main-verification.md:28`）。

**B3：钉的是不是「第 k 片都对」，135 从哪来**

- 135 的出处：kb 「条目 120 字节、扇出 135」（`.claude/kb/decisions/08-核心索引结构.md:155`）；实现是 `crates/singlefs-core/src/inode_tree.rs:110`：「index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))」。
- 用例把两边钉在一起：`crates/singlefs-harness/src/model_comparison.rs:1427`：「const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: usize = 135;」，紧跟着 `crates/singlefs-harness/src/model_comparison.rs:1429`：「singlefs_core::inode_tree::leaf_containers_one_root_node_holds(),」断言相等。上界换一个：实现的扇出变了，先红在这句相等上，不会静默少罩；用例自己改小常量，要连这句相等一起改。
- 胶水只有一处映射：`crates/singlefs-harness/src/model_comparison.rs:80`：「ModelUnitRole::inode_leaf_container_at(u64::from(index.0))」。命令 `grep -rn 'inode_leaf_container_at' crates/singlefs-harness/src crates/singlefs-checker-tier/src` 在冻结副本里的命中：`model_comparison.rs:80`（生产用）、`model.rs:110`（定义）、两处测试里的调用；没有第二个胶水点。
- 用例对 0..135 每片断言三件：映到的角色说的位置等于 k、等于 `ModelUnitRole::inode_leaf_container_at(k)`、互不相同（`crates/singlefs-harness/src/model_comparison.rs:1475`：「roles_seen.insert(role),」；末尾 `crates/singlefs-harness/src/model_comparison.rs:1482`：「"135 片映到 135 个互不相同的角色"」）。

量（经 `tools/rerun.sh p1`；行号是冻结副本 `crates/mutations.tsv` 的行号）：

```
row2774 (入库 B3：封顶第 4 片)   red=1 [model_comparison::tests::every_inode_leaf_container_one_root_holds_maps_to_its_own_model_role_and_no_two_share_one]，同一次跑里 B4 那条过（passed=1）
var-glue-cap-5 / -6 / -20 / -60 / -100 / -133   各 red=1（同一条）
var-glue-swap-77-78（第 77、78 片角色互换）      red=1
var-glue-mod-64（序号对 64 取余）                  red=1
var-glue-cap-134                                   passed=2 red=0
```

`var-glue-cap-134` 不红不是漏：序号最大就是 134，`.min(134)` 与不改等价（等价变体）。取样点之外我换了 8 个非等价的封顶 / 互换 / 取余位置，全红；第 9 个（封顶 134）是等价变体。

推翻条件：glue 封顶或错位在某一片 k（0 ≤ k ≤ 134）上而这条用例照绿；或 `leaf_containers_one_root_node_holds()` 不是 135 而这条用例照绿。上面 8 个非等价变体没有出现。

**B4：36 种组合罩不罩得住回退结构的错法**

- 36 = 6 × 6：`crates/singlefs-harness/src/model.rs:4331`：「const CONTAINER_COUNTS: [u64; 6] = [1, 2, 3, 4, 5, 135];」，现行与目标各取一个；目标一版的片分配代是 1000 + k，现行是 2000 + k，所以取错一版的分配代看得出（`model.rs:4356` 起调 `next_root_with_the_user_visible_trees(… TakenFromTheRollbackTarget(&target))`，`model.rs:4360` 比两边最左之外的叶容器）。
- 被测的两句在 `crates/singlefs-harness/src/model.rs:1720`：「role_written_at.retain(|role, _| {」（丢现行那几片）与 `crates/singlefs-harness/src/model.rs:1728`：「matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))」（取回目标那几片）。

```
row2775 (入库：只取回目标第 2–3 片)   red=1 [model::tests::a_rollback_takes_exactly_the_inode_leaf_containers_…]，B3 那条同次过
row2776 (入库：只丢现行第 3–4 片)     red=1（同一条）
var-b4-take-up-to-5 / -6 / -20 / -100 / -133   各 red=1（只取回目标位置 ≤ c 的片）
var-b4-take-up-to-134                           red=0（等价变体，最大位置就是 134）
var-b4-keep-current-from-5 / -6 / -100          各 red=1（现行位置 ≥ c 的片丢不掉）
var-b4-written-at-wrong-at-3                    red=1（第 3 片的分配代取错一版）
var-b4-count-window-60-118                      red=0
```

**射程限度（读数）**：最后一行是我故意造的「只在目标装 60–118 片时才错」的变体，取样点 {1, 2, 3, 4, 5, 135} 碰不到它，用例照绿。按位置错（封顶、丢、取错分配代）的错法，因为 135 片把 1–134 每个位置都走到了，取样点换到任何位置都红；只依赖「片数落在 6–134 之间」的错法这条用例罩不到。判决写的错法形（封顶、只取回、只丢）都在罩内；这一条限度写进「判定一览」，不改判定。
推翻条件：上面任一「red=1」的变体改成绿，或有一个按位置错的变体照绿。

## P2　F1、33-c 与 F2

**判定：（a）两支「部分没判」在合法历史上没有多报：一致（量过：套件里只在刻意弄坏树表或分配记录树的用例上触发）；（b）同形还有一处整版不进评估（树表指针全零、根记录那一项非零而那棵树读不出），读码坐实、套件里没有用例走到它；（c）F2 与 core 在 core 判指针的每一类读者上是同一组取值同一判法，checker 判的点比 core 多（根记录里的四条指针），GPU 的 `head_ok` 与 CPU 逐格同（读码，没跑 GPU）。**

**（a）合法历史上多报吗：量**

条款与判决：F1 的出处 `research/prompts/m3-formal-code-b30-r1-main-verification.md:40` 写「改法 F1（树表读不出的候选根改报部分没判；攻方副本上量过：三张镜像改报部分没判、入库坏镜像 89 条全过、合法批 0 次），零轮，并进第 32 批」；kb 的同类写法是 I-9.28 那一行的「部分没判，列出这些根（盘号、实例、txg、为什么没判）」（`.claude/kb/invariants.md:315`）。

读码：F1 的条件是 `crates/singlefs-checker/src/walk.rs:5095`：「if tree_table_read_back.is_none() {」，只在树表指针非零（`crates/singlefs-checker/src/walk.rs:5092`：「if !tree_table_pointer.all_zero {」）而读不回时记；33-c 的条件是 `crates/singlefs-checker/src/walk.rs:5103`：「tree_table.entries.is_empty() && !allocation_roots[0].all_zero;」加 `crates/singlefs-checker/src/walk.rs:5164`：「&& !this_version_allocation_record_tree_walked_to_its_leaves」。候选根只有最新根与「不被抛弃、不在 F 之下」的根（`crates/singlefs-checker/src/walk.rs:10287`：「let walked = *index == newest_index || (!abandoned && !below_floor);」），合法历史里这些根引用的单元在谓词下不回收，读得回。

量法：在副本的 `walk.rs` 三个分支各加一行 `eprintln!`（`probes/instrument_i54.py`，探针 B33F1 = F1 那支、B33C = 33-c 那支、B33ZERO = 下面（b）那一处），线程名就是测试名；用 `--nocapture` 把整个 harness 套件的每个二进制各跑一遍（`tools/sweep.sh`，二进制清单 `bins.tsv` 共 162 个，含 `--lib`）。原样输出：

```
$ cat work/sweep/*.rc | sort | uniq -c
    162 0
$ grep -c B33ZERO out/sweep-nonignored-probe-lines.txt
0
```

探针行出现的测试一共 13 条（`out/sweep-nonignored-probe-lines.txt`，落在 4 个二进制里，其余 158 个二进制 0 行）：
- 这一轮新加的 F1 / 33-c 用例：`the_tree_table_of_the_newest_version_…`、`…_previous_version_…`、`the_genesis_tree_table_…`、`the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_…`；
- 别的刻意把树表整单元清零 / 读坏的用例：`an_intermediate_tree_table_that_does_not_read_back_…`、`a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged`、`a_readable_root_whose_tree_table_is_unreadable_while_the_others_agree_…`、`a_birth_txg_disagreement_is_reported_as_violated_even_when_another_roots_tree_table_is_unreadable`、`release_generations_hidden_behind_a_later_incomplete_root_…`、`every_root_carrying_the_raised_floor_unreadable_…`；
- `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`（6 行）：干净镜像那一段按 `crates/singlefs-harness/tests/checker_known_bad_images.rs:3084`：「assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");」逐条核，I-5.4 的预期是成立，这条用例过了，所以探针行来自它后面逐个施加的坏镜像；
- 只供测试的「复用窗口强制为 0」开关：`the_forced_zero_reuse_window_switch_…`、`reuse_window_forced_to_zero_…`（`test_only_switches_force_every_branch`）：这个开关造的是实现把候选根还引用着的单元提前回收的坏历史，用例自己断言 checker 红，不是合法历史。

这个套件里的合法历史有：写行与暖机（`writable_mount_of_a_formatted_pool`）、回退（`rollback_by_a_forward_publish` 非慢档部分）、抬 F 之后（`reuse_after_raising_the_floor` 非慢档 15 条）、多实例与不等盘（`unequal_devices`、`multi_record_publishes_of_transaction_zero`）。它们 0 行。
慢档（`--ignored`，一个测试一个任务，`tools/sweep-ignored.sh`，`out/sweep-ignored-summary.txt`）66 条：55 条过、探针行 0；5 条红（`shrink_one_failing_seed_from_the_environment` 要环境变量；`raising_the_floor_to_the_first_release_generation_…` 见 P3 末尾；`rollback_by_a_forward_publish` 三条 I-3.1，第 31 批判决已记的与这一轮无关的红）；6 条一次调用 225 秒内没跑完（`fault_injection_large_tier_from_the_environment`、`one_fixed_history_injects_twelve_faults_and_none_of_them_panics`、`allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall`、`random_histories_large_tier_seeds_and_length_from_the_environment`、两条 `unit_area_wall_sampling_on_small_devices_…`），这 6 条的合法历史部分写「没跑」，要主 agent 跑（末节命令）。慢档里抬 F 加回退加越过 812 条的 `overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed` 过了（140 秒，10 线程）。

层 0 的影响（推的，没跑）：部分没判在层 0 与崩溃放量里「计进不适用这一格」（`crates/singlefs-checker-tier/src/crash.rs:1492` 的注释），不红；不带读故障的层 0 状态里候选根的树表单元在根槽落盘之前已落盘，不会读不回，F1 / 33-c 不改它们的 I-5.4 结局。钉了计数的 `crash_enumeration_sharded_across_processes.rs` 只在它的流里有读故障或改坏单元的状态时才会变，这一条要提交时跑。

推翻条件：套件里出现一条不是故意弄坏树表的用例上有 B33F1 / B33C 行；或层 0 不带读故障的状态里 I-5.4 由成立变成部分没判。

**（b）坏盘上还有没有同形的版本整版不进按版本那一层**

按 `judge_allocation_records_disjoint` 读码逐形（`walk.rs:5058` 起，每条候选根走一遍），一个版本「静默不计」要同时满足：它的分配记录树没走到叶、又没进任何一个名单。逐形：

| 形 | 去向 |
|---|---|
| 树表指针非零、树表读不出 | F1 名单（`walk.rs:5095`） |
| 树表读得出、0 条、根记录那一项非零、那棵树读不出 | 33-c 名单（`walk.rs:5163`） |
| 树表非空、种类 3 条目缺失或比字段表短 | 根记录那一项全零时进「带树一版没计」名单（`walk.rs:5105` 起），非零时判红（`crates/singlefs-checker/src/walk.rs:5109`：「judgements.judge("I-5.4", false,」，说明里写这一版有两棵分配记录树） |
| 树表非空、种类 3 条目指着的树读不出 | 同上「没计」名单 |
| 分配记录树读到一半断（子节点读不出、层数不对） | `allocation_record_tree_without_judging`（`walk.rs:4188`）整棵全有或全无，交 `None`，等于树读不出，归上面几形 |
| 树表指针**全零**、根记录那一项非零、那棵树读不出 | **不在任何名单**：这一块被 `crates/singlefs-checker/src/walk.rs:5092`：「if !tree_table_pointer.all_zero {」包住，设标记的那一行在块里；块外只剩 `crates/singlefs-checker/src/walk.rs:5148`：「let Some((_, records)) =」，读不出就 `continue` |
| 树表指针全零、根记录那一项也全零 | 第 0 代形，合法 |

表里「树表指针全零、根记录那一项非零、那棵树读不出」那一行，就是主 agent 观测里「已知、已记」的那一处。kb 对这种根的写法是「树表指针全零的根没有树表，不算没判到」（`.claude/kb/invariants.md:315`，I-9.28 那一行），所以这一处与 I-9.28 的口径一致；它在 I-5.4 上还不是「部分没判」，是按口径等价于「没有树」。我的补充：这一形在套件里 0 用例（上面 B33ZERO 0 行），没有测试在盯它，能被盯的坏镜像要把根槽里的树表指针清零并重封，我没造。
推翻条件：造出树表指针全零、根记录那一项非零而那棵树读不出的镜像，I-5.4 报成立以外的结局（那就是我读码错了）；或上面表里任一形造出镜像后 I-5.4 报成立。

**（c）F2：判的三段与 core 读路径拒的是不是同一组取值、同一判法**

读码对读（冻结副本）：

| 取值 | core `PointerHeadAsRead::judged` | checker `judge_pointer_encryption_and_compression_fields_take_the_first_version_values` |
|---|---|---|
| 算法类型（偏移 28） | `crates/singlefs-core/src/pointer.rs:214`：「if self.algorithm_type != 0 {」 | `crates/singlefs-checker/src/image.rs:1402`：「judgements.judge("I-2.4", fields.algorithm_type == 0,」 |
| 压缩算法码（偏移 29） | `crates/singlefs-core/src/pointer.rs:221`：「if self.compression_algorithm_code != 0 {」 | `crates/singlefs-checker/src/image.rs:1408`：「judgements.judge("I-2.4", fields.compression_algorithm_code == 0,」 |
| 压后长度（偏移 30，2 字节） | `crates/singlefs-core/src/pointer.rs:228`：「if self.compressed_length != 0 {」 | `crates/singlefs-checker/src/image.rs:1414`：「judgements.judge("I-2.4", fields.compressed_length == 0,」 |

三段都是「非 0 即坏」，偏移 28 / 29 / 30 一样（core 读法 `crates/singlefs-core/src/pointer.rs:194`：「let compressed_length = reader.get_u16();」在算法类型、压缩码之后；checker 读法 `crates/singlefs-checker/src/image.rs:1326`：「compressed_length: read_u16(pointer_bytes, POINTER_COMPRESSED_LENGTH_OFFSET),」）。差别只在报法：core 在第一个不合格处停、checker 三段各记一格（压缩码非 0 时压后长度非 0 的说明文字「压缩算法码 0 时这 2 字节恒 0」不准确，是措辞，不改结局）。
调用点：`walk.rs` 里 `judge_pointer_mac_and_nonce_are_zero(`、`judge_pointer_encryption_and_compression_fields_take_the_first_version_values(`、`judge_pointer_extent_offset_is_zero(` 各 5 处（`grep -c -F` 输出 `5 5 5`），位置逐个同（`walk.rs:1305`、`2118`、`2365`、`2722`、`2888` 是 F2 那 5 处）。

两处超出 core 的地方（读码）：core 读根记录里的四条指针只判 extent 偏移：`crates/singlefs-core/src/root_record.rs:146`：「let tree_table = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;」，实例表、映射根、分配记录树根同；checker 经 `read_index_node`（`walk.rs:1305` 在它里面）判树表根指针的 MAC / nonce（已有）与算法类型、压缩几段（F2）。也就是说一张根记录里树表指针算法类型写 1 的镜像，core 照挂、checker 红 I-2.4。kb 对 extent 偏移那一半写明「根记录里的指针偏移非 0 按这条根自证不过处置：core 退回上一条根」，「两边各判各的、不矛盾」（`.claude/kb/invariants.md:133`）；MAC / nonce 那一半今天就是这个不对称，F2 沿同一个点延伸，不是新类型的不对称。kb 的 I-2.4 那一行还没把算法类型、压缩码、压后长度写进指针清单（只写 MAC / nonce 与 extent 偏移，写回归写回批，判决第三节 2 已写）。
GPU：`w_judge_pointer_head` 调用 5 处（`crash_judge_checker.wgsl:1319` 定义，调用点在 `w_read_index_node`、实例表链、extent 数据指针、inode 根、容器树根，与 CPU 那 5 处一一对应），`head_ok` 的谓词是 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:513`：「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」，与 CPU 三段加 MAC / nonce 同一个合取；`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1321`：「w_judge(INVARIANT_I_2_4, p.head_ok);」。全零指针 `zero_pointer()` 把 `head_ok` 置真，与 CPU 的全零指针不判一致。GPU 对拍新加那条与 F2 的 GPU 变异没跑，**推的**。

量（`tools/rerun.sh p2`）：入库 F2 的 4 行变异在 harness 档上都红：行 2780（三段不判）三条红，行 2781、2782、2783 各自点名的那一条红；合法镜像上 F2 不误红：上面 162 个二进制全过（其中 `checker_known_bad_images` 的干净镜像逐条断言、大量「checker 一条都不红」的历史用例）。
推翻条件：造一个 core 读者拒、checker 不红的指针头（按上表三段之外），或 checker 红、core 的七类读者接受的指针头。

## P3　32-2

**判定：（a）812 用例每版跑 checker 罩得住判决点名的 R3、B3、d，罩不到 R4（判决已写 R4 靠已有用例）：一致；（b）R4、d 点名的已有用例真在它们下面红、基线绿：一致；（c）U11 那一行红在 I-3.1 那一句：读码一致、机理在 harness 档量过，U11 本身没跑（推的）；（d）四支豁免：I-4.4、I-4.6、I-4.7 各只认一种理由，一致；I-5.5 那支认的是 checker 的缺省理由，分不开「没有对象」与「根本没判」——部分放过的风险；另，豁免之外还有六条同类不适用，用例今天仍红。**

**（a）每版跑 checker**

判决要的：「每一版跑池级 checker、要求 0 违例」（`research/prompts/m3-formal-code-b31-r1-main-verification.md:71`），依据是 N3 那一行「改法：这条用例每一版都跑池级 checker、要求 0 违例——攻方原型甲量过 R3 红 I-3.9、B3 红 I-7.17、基线 201 版 0 违例，正推探针量过 d 在 txg 4 红」（`research/prompts/m3-formal-code-b31-r1-main-verification.md:46`）。
实现：`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:725`：「let mut image = pool.memory_pool();」起，每次覆盖写之后只补施加新发出的那一截、跑 `check_pool_image`，有违例就红（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:746`：「"第 {overwrites} 次覆盖写（txg {}，{} 条分配记录）之后池级 checker 判红：{violated_invariants:?}",」）。基线：`cargo test -p singlefs-harness --test overwrite_in_one_instance` → `test result: ok. 12 passed; 0 failed`（`out/baseline-runs.txt`）。

量（`tools/rerun.sh p3`，行号是冻结副本变异表的行号；红的第一处原样）：

```
row2788 (R3 释放代少记一)  allocated_statistic_equals_…：第 1 次覆盖写（txg 4，52 条分配记录）之后池级 checker 判红：[("I-3.9", Violated("盘 1 槽 50178 的分配记录释放代 2 不在 (2, 3] 里：还引用它的最新一条有效根是 txg 2，最早不再引用它的是 txg 3"))]
row2789 (B3 根环公式少一槽) allocated_statistic_equals_…：第 18 次覆盖写（txg 21，504 条分配记录）之后池级 checker 判红：[("I-7.17", Violated("实例 1、txg 21 的根住在区域 0 槽 0：按 txg 算该在区域 Some(0)（第 0 代不判区域）槽 7"))]
row2791 (d 提前回收)       在 812 那份文件下 8 条红（r2791-on-812），allocated_statistic_equals_… 红在第 1 次覆盖写（txg 4）：I-2.1 与 I-3.1
row2790 (R4 先记根再取落点) 在 812 那份文件下 12 过 0 红（r2790-on-812：test result: ok. 12 passed）
```

R4 不在这条用例的射程内，与判决一致（判决把它交给 `root_ring_turn_in_one_mount` 那条已有用例）。33-a 在末尾那一版再判一次：行 2793（R3）红在 `repeated_overwrites_go_past_the_812_…`，行 2794（B3）红在 `crates/singlefs-harness/tests/overwrite_in_one_instance.rs:660` 那句断言：「58 次覆盖写（txg 61，816 条分配记录）之后池级 checker 判红：[("I-7.17", Violated("实例 1、txg 42 的根住在区域 0 槽 0：按 txg 算该在区域 Some(0)（第 0 代不判区域）槽 6"))]」；这条只在末尾判一次，所以 B3 要到 txg 61 才红，每版都判的是 `allocated_statistic_equals_…`（txg 21 就红）。
推翻条件：R3、B3、d 任一在 812 那份文件里全绿；或 R4 红在这份文件里（那就说明判决「R4 不在射程」写错了）。上面没有出现。

**（b）R4、d 点名的已有用例真红，且基线绿**

基线：`cargo test -p singlefs-harness --test root_ring_turn_in_one_mount` → `test result: ok. 3 passed; 0 failed`。施加 R4（行 2790）与 d（行 2791）之后，点名的 `turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it` 各红在 `crates/singlefs-harness/tests/root_ring_turn_in_one_mount.rs:108`，原样：`盘 DeviceIdentity(0)：txg 26 盖掉 txg 2 之后 mkfs 那片树表按谓词回收，同一次挂载里就回到空闲`，同一次编译日志里有 `Compiling singlefs-core` 与 `Compiling singlefs-harness`。两行红在同一句，这条用例分不开 R4 与 d（行本来就是各点名同一条，判决「锁住，不新写用例」）。
判决要求原文：「改法：R4 与 d 各加一行变异点名这条已有用例（锁住，不新写用例），并进 32-2」（`research/prompts/m3-formal-code-b31-r1-main-verification.md:38`）。推翻条件：基线红，或变异下这条仍绿。都没有出现。

**（c）U11 那一行**

判决：「照第 2735 行的改法（记根时环转了也不回收，不碰断言）加一行点名 U11」（`research/prompts/m3-formal-code-b31-r1-main-verification.md:62` 的互相掩护那一行，另见 `:74` 去向）。行 2792 的原文 / 替换文与 E156 的 M35（行 2746）、harness 档行 2731 是同一处同一改法（`out/p6-rows.txt` 的「同一改法的别的行」一列），点名的测试各不同。
U11 的断言就是 I-3.1 那一句：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12263`：「matches!(allocated_statistic_verdict, InvariantVerdict::Holds),」。变异施加的是 `crates/singlefs-core/src/allocator.rs:1961`：「self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);」，核断言（`record_root_written_by_this_process` 里没装根环表就 `expect`）在它前面，不碰。
U11 是 checker 档的装置内联单测，我不跑。同一处变异我在 harness 档量了机理（`row2792-harness-probe`）：`overwrite_in_one_instance` 与 `root_ring_turn_in_one_mount` 共 3 条红，每版 checker 判红的集合只有 I-3.1：

```
第 23 次覆盖写（txg 26，644 条分配记录）之后池级 checker 判红：[("I-3.1", Violated("盘 0：记账的已分配 Some(6078464)，遍历全部有效根得到 6062080（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 24、最新根 txg 26、…
```

txg 26 起 I-3.1 红，不红在别的条上，也没有 core 断言的 panic（两份文件共 15 条，红的 3 条都是断言失败，不是 `expect`）。U11 的内容与此同形，**推的**：它红在 I-3.1 那一句；它另一句「每次调用都成功」（同文件 `:12224` 一带）在这个变异下该保持成立，我没法在 harness 档量。
推翻条件：U11 在这一行变异下红在「每次调用都成功」或 core 断言（E171 装置里量），或根本不红。

**（d）四支豁免**

四支都在 `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs` 的同一个循环里（`:254`：「for (invariant, verdict) in &verdicts {」），循环末尾把没豁免的都要求 `Holds`（`:331`：「"{invariant} 在 E 之后的镜像上要成立"」）。逐支：

| 支 | 用例里认什么 | 这个理由 checker 里是谁写的 | 一种理由吗 |
|---|---|---|---|
| I-5.5 `:276` | `crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:284`：「InvariantVerdict::NotApplicable(reason) if *reason == "这个镜像上没有它判的对象"」 | `crates/singlefs-checker/src/image.rs:358`：「.unwrap_or("这个镜像上没有它判的对象"),」——**所有没评估过、也没写专门理由的不变量**共用的缺省话；判法 `walk.rs` 里 I-5.5 的函数（`crates/singlefs-checker/src/walk.rs:4466`）没有自己的 `not_applicable` 理由 | 只能认这一句，但这句不分「候选集里没有从 mkfs 水位建树的那一版」与「函数提前返回、一次都没判」 |
| I-4.7 `:290` | `…:298`：「if reason.contains("走读到的每一版里都没有处于删除中的 inode")」 | `crates/singlefs-checker/src/walk.rs:8532`（删除态扫描收尾时才写，函数没跑到尾就拿不到） | 是；读不全时走另一条路：`crates/singlefs-checker/src/image.rs:349` 先取 `part_of_the_range_not_judged` 的理由（`walk.rs:8517` 写 `gap.reason()`），字符串不同，`contains` 不过，用例红 |
| I-4.6 `:304` | `…:312`：「if reason.contains("走读到的每一版 inode 树里都没有 nlink = 0 的记录")」 | `crates/singlefs-checker/src/walk.rs:8536` | 同上 |
| I-4.4 `:318` | `…:323`：「InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),」，`assert_eq` 逐字 | `crates/singlefs-checker/src/image.rs:252` 的常量，只由 `walk.rs:10570` 一处写 | 是，且 kb 就是这么写的 |

kb 依据：I-4.4 的「第一版不适用：意图载体（nlink = 0 的记录、`deleted_inodes` 的 key、墓碑）没有可装增量的字段」（`.claude/kb/invariants.md:179`）；I-4.6、I-4.7 的「I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立） 没有对象时报不适用并带理由」（`.claude/kb/invariants.md:292`）；I-5.5 的「V 没有数据单元时不判；不满足 ①② 的版本不判。」（`.claude/kb/invariants.md:199`）。四支都与条款一致。I-5.5 那支的依据是 `/tmp/claude-1000/i55-reuse-investigation/report.md` 的插桩：七个候选版本都在 `walk.rs` 的「是否从 mkfs 水位建树」那里 `continue`，`versions_not_judged=[]`；把候选集换成全部有效根之后 I-5.5 变成成立。这是调查员的量，我没复现（插桩要改 `walk.rs`），用例本身无法替它作证——**I-5.5 这一支比另三支弱**：以后 I-5.5 的函数若因 bug 提早返回，缺省理由一样过得了这支豁免。出路是 checker 给「没有对象」写一句专门的理由（改 `walk.rs`，不属于这一轮）。
推翻条件（I-4.x 三支）：造出读不全的镜像而 I-4.7 / I-4.6 仍报那句专门理由；（I-5.5）：函数提前返回的变异下这条慢档用例照绿。后者我判「会照绿」是读码，没造。

**（e）豁免之外：这条慢档用例在冻结副本里今天是红的**

```
$ cd work/tree && … cargo test -p singlefs-harness --test reuse_after_raising_the_floor -- --ignored --exact raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
thread 'raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it' (328256) panicked at crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:328:9:
assertion `left == right` failed: I-9.18 在 E 之后的镜像上要成立
  left: NotApplicable("走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判")
 right: Holds
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out; finished in 3.92s
```

我在副本的用例里临时加了三行把每条非成立的判定打出来（跑完用冻结副本原文件覆盖回来，`cmp` 相同），12 条：I-8.8、I-4.1（原有的两支豁免）、I-5.5、I-4.7、I-4.6、I-4.4（这一轮的四支）、**I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23 六条报不适用而没有豁免**（理由原文在 `out/slow-tier-reuse-test-nonholds.log`）。同一份 `checker_known_bad_images.rs` 里干净镜像的预期早已把它们写成不适用（`crates/singlefs-harness/tests/checker_known_bad_images.rs:1868`：「QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE => {」，下一行是 `NotApplicable(NO_QUEUED_INODE_IN_ANY_WALKED_VERSION)`）。
归属（推的）：I-9.18–I-9.23 是删除线第 21、22 批合入的新不变量，HEAD 的 `image.rs` 里 `grep -c 'I-9.18\|I-9.23'` 是 0，工作区是 6；32-2 的四支是照 i55 调查（那时树里还没有它们）写的。「在主工作区单跑 ok」应是更早的状态；`diff -rq` 证明主工作区 `crates/` 今天与冻结副本相同，这条今天红。这一条红由谁收（给用例再加六支豁免，还是改成「除第 9 类删除线那几条之外一律成立」），是主 agent 的事。
推翻条件：主工作区单跑这条今天绿。

## P4　33-a、33-b

**判定：（a）C551 的推翻条件满足：一致；（b）① 的 `Some(1 + 3)` 与第 28 批 G5 是同一个口径：一致（读码，checker 档没跑，推的）；（c）② 的新造法测到的就是「F_生效 取系统配置里的 F」那一判，没有被别的拒因先拦：一致（造法在 harness 档量过，checker 档那条用例本身没跑）；（d）旧第 38 行没有别处按旧名找它这一问：**有**，两份 E171 坏实现表按名引用，它们找不到时走补位，不会静默失效。**

**（a）C551**

条款：C551 的前置写「`build_pool` 装上根环表之后，同一条用例判全部不变量。」（`.claude/kb/checks-owed.md:425`）。
实现：33-a 在 `repeated_overwrites_go_past_the_812_…` 末尾对 `check_pool_image` 的全部判定要求没有 `Violated`（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:660` 那句断言的格式串在「{overwrites} 次覆盖写…之后池级 checker 判红」）；`NotApplicable` 与 `PartiallyJudged` 不红。
量：基线 `overwrite_in_one_instance` 12 过（`out/baseline-runs.txt`）。这条在 R3（行 2793）与 B3（行 2794）下红，在「记根时环转了也不回收」（行 2792 的变异，harness 档）下红在 I-3.1：`30 次覆盖写（txg 33，840 条分配记录）之后池级 checker 判红：[("I-3.1", Violated("盘 0：记账的已分配 Some(7913472)，遍历全部有效根得到 6291456…`。也就是 I-3.1（C551 当初红的那一条）在这条用例的末尾版本上真评估过、成立，不是被判成不适用跳过。所以「转过根环之后 I-3.1 红、用例只判 I-1.1 与 I-3.10」的前置已不成立，推翻条件满足。
我没动 `checks-owed.md` 里 C551 那一行的状态（写回归写回批）。判据只管「末尾那一版」，每一版都判的是 `allocated_statistic_equals_…`。
推翻条件：基线下这条用例红，或上面三个变异里有一个它照绿。

**（b）① 的新期望与 G5 的口径**

判决那一格：「改法 G5：二次崩溃那一截照第一截按标记跳过，与射程句一致」（`research/prompts/m3-formal-code-b26-r1-main-verification.md:43`），射程句是 D13 已定项 1 的「带故障的状态不在射程」（`.claude/kb/decisions/13-验证路线.md:57`）。判决只说「跳过」，没说怎么计；计法按这张表本来的写法——按镜像计：二次崩溃镜像上没崩过的那次可写挂载被拒也是每个二次崩溃加一次（`crates/singlefs-checker-tier/src/crash_injection.rs:3062` 一带用 `MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED`）。
G5 的实现在 `crates/singlefs-checker-tier/src/crash_injection.rs:3451`：「也不判，与第二截同一个理由记一次没判」，下面的 `Injected` 臂（`crates/singlefs-checker-tier/src/crash_injection.rs:3466`：「ReadFaultsInTheCrashState::Injected => {」）每个二次崩溃加一次那个理由；每个二次崩溃数一次（`crates/singlefs-checker-tier/src/crash_injection.rs:3385`：「tally.second_crash_points += 1;」）。
新期望：`crates/singlefs-checker-tier/src/crash_injection.rs:6962`：「(1, 1, 1, 3),」（一个第一次崩溃态、注入过读故障、可写挂载做成、按段摆了三个二次崩溃）与 `crates/singlefs-checker-tier/src/crash_injection.rs:6977`：「(BTreeMap::new(), BTreeMap::new(), Some(1 + 3)),」；用例文档写「同一理由共记四次没判」（`crates/singlefs-checker-tier/src/crash_injection.rs:6927`：「三个二次崩溃也不判（第 28 批 G5），同一理由共记四次没判。」）。所以 `1 + 3` 是第一截一次加三个二次崩溃各一次，与代码口径同一个。
三行变异各抓一种（用例文档把它们列全）：行 2552（第一次崩溃镜像上照判，没判只剩 3 次）、行 2639（二次崩溃那一截不看标记，是 1，但行 2639 自己点名的是另一条用例）、行 2795（跳过而不记，是 1）；2552 与 2795 点名这一条。
没法验的：这条是 checker 档 lib 单测，不跑；`crash_injection.rs:6962` 那个 `3` 要「按段各摆一个」的种子抽样给出三段，我读到的是 `draw_second_crash_points` 的注释「挂载的每一段各按种子摆一个」，没量。主 agent 的 `--item` 抓到与基线绿已在背景里记过。
推翻条件：G5 的条款（判决、D13、记录）里有一句写明按第一次崩溃态计一次；或这条用例在今天的 core 上给出的不是 `(1, 1, 1, 3)` 与 `Some(4)`。

**（c）② 的新造法：测到的是不是「回退下界取系统配置里的 F」**

- 条款：F_生效 取到系统配置里读得出的 F（「F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 被抛弃时间线上的根所带 F 的最大值, 系统配置里读得出的 F 的最大值)」，`.claude/kb/decisions/16-发布语义.md:38`），回退候选集是「按现行那一版的实例表判仍然有效 ∧ txg ≥ F_生效」（`.claude/kb/decisions/16-发布语义.md:39`）。旧造法（抬 F 做成、再改坏 txg 7、8 的根）今天红的原因：这个进程写过的槽是「知道住着根的槽」，重读仍坏就先拒，到不了下界那一判（`.claude/kb/decisions/16-发布语义.md:37`：「仍坏就拒这次抬 F 或这次回退、报它自己的错误成员，不按有根或没根猜」；机理与候选 A/B 的量在 `/tmp/claude-1000/red-two-investigation/report.md`，我没复现旧造法）。
- 新造法：抬 F 的第二次写（盘 1 的系统配置槽）报块设备错，只有盘 0 的系统配置带新 F；一条带新 F 的根都没发；回退到 txg 2。用例断言抬 F 报的是 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot` 且只有盘 0 带新 F（`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs:399`：「Err(MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed))」），回退报的是「目标 (1, 2)、`BelowEffectiveFloor`」（同文件 `:468`：「exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,」），录制流不多一步。这一造法一个根槽都不碰，下界只能从盘 0 的系统配置来。
- 量（harness 档探针 `probe_b33_rollback_floor`，`tools/rerun.sh p4`；它不是 checker 档用例本身）：基线 `10 passed; 0 failed`，新造法那条过；施加下面四行，这一条各红，原样：

```
行 2796 (回退候选的 F 用目标根自己带的 F)    txg 2 低于 F 生效值 3：不是候选：Some(TargetNotACandidate { target: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) }, exclusion: VersionWithoutFile })
行 2797 (F_生效 不与系统配置里读得出的取 max)  同一句，exclusion: VersionWithoutFile
行 720  (低于 F_生效 的目标报成树表 0 条)       同一句，exclusion: VersionWithoutFile
行 1376 (harness 档伪造系统配置那条的同一处)    同一句，exclusion: VersionWithoutFile
```

基线是 `BelowEffectiveFloor`，变异下是 `VersionWithoutFile`：红在下界那一判上，没有被别的拒因先拦（先拦会是 `CandidateJudgementStillUnreadableAfterOneReread` 之类，用例的 `matches!` 会认出来）。
- 行 2796 的替换文用 `target_root.rollback_floor`（目标根自己带的 F，今天编得过）。旧名里的「最新根自己带的 F」在这个函数里今天没有 `newest_root` 变量，编不过（见（d））；这条造法里最新根与目标根带的 F 都是 0，所以换成能编的「最新根」写法这条用例同样会红——这一句是推的，没造。
推翻条件：新造法在基线 core 上不是 `BelowEffectiveFloor`；或上面四行有一行它照绿；或搬进 checker 档之后基线不绿（那一条我量不到）。

**（d）旧第 38 行改名之后，别处按旧名找它吗**

旧行（HEAD 的 `crates/mutations.tsv` 第 38 行）改名为「步 4：回退候选集的 F 用目标根自己带的 F 而不是 F_生效」（今天在行 2796）；冻结副本里旧名一次也不在表里（`grep -nF '用最新根自己带的 F' crates/mutations.tsv` 零命中；`grep -nF '步 4：回退候选集' crates/mutations.tsv` 命中第 28 行「候选集不按实例表判」与第 2796 行新名）。旧行的替换文 `newest_root.rollback_floor` 在今天的 core 上编不过：我施加它后 `cargo build -p singlefs-core` 报
`error[E0425]: cannot find value `newest_root` in this scope` / `--> crates/singlefs-core/src/mount.rs:6086:32`（`variants/old-row38-newest-root.spec`，日志 `work/log/var-old-row38-compile.log`），所以旧行本来就是编不过、等于没跑的（C427 那一形）；改名换替换文是对的。
按旧名找它的地方（`grep -rnF '回退候选集的 F 用最新根自己带的 F' scripts/experiments scripts .claude crates` 原样输出两行）：

```
scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/bad-implementations-117.tsv:33:33	2	crates/singlefs-core/src/mount.rs	7	步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	b093430c0295
scripts/experiments/e171_three_oracle_collusion/bad-implementations-117.tsv:33:33	2	crates/singlefs-core/src/mount.rs	7	步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	b093430c0295
```

消费它的是 E171 的 `locate_by_name`：`scripts/experiments/e171_three_oracle_collusion/bad_implementation_table.py:116`：「hits = [row for row in table if row.name == selected.name]」，找不到返回 `STATUS_NOT_FOUND`（`scripts/experiments/e171_three_oracle_collusion/bad_implementation_table.py:118`：「return Located(STATUS_NOT_FOUND, None, False)」），文档写明「找不到、名字不唯一都不跑（补位）」（`:115`），`substitute_for`（`:158`）换同一文件组里下一个不在表里的行。结局：E171 重放会把这一行补位成别的行（第 33 行落到 `mount.rs` 组里的另一条），不是静默失效；但 E171 的「按名字在今天表里逐行核」（`comparator.py` 的 A4）会多报一行找不到，登记 `research/prompts/e171-preregistration.md:349` 的表与 kb 里「117 行」的口径要跟着解释。另外这次删一行让后面所有行号减一：判决与背景材料里的「第 721 行」「第 1377 行」今天是 720、1376（我在上面（c）按今天的号取的）。
推翻条件：E171 的任一消费者找不到旧名时报的不是补位（例如抛错或静默用别的行当它）。我读了 `bad_implementation_table.py` 两份（`…_after_the_oracle_fixes` 里 `:116` 同写法），没跑 E171（它是研究脚本，且 replay 属重型）。

## P5　E156 装置

**判定：（a）第 6 次的门槛读法（全零槽并进 min、三种解开时点）与 core 的 `RootRingOccupant::AllZeroAtMount` 在装置覆盖的那一族上是同一个谓词：一致；（b）装置不是跟 core 一起错的：一致（control 产物证了：core 去掉钉住，装置的钉住表不动）——但装置与 core 共用三个核心函数，共用部分的错这条路看不见；（c）PC-钉门槛与 PC-全零钉各分得出它们要分的东西，各有不分的两处（已在登记「它答不了的」里写了）；（d）改调用点不改函数体，M36 与 M42 没有互相掩护：读码一致。装置内联单测 19 条与 M35、M44–M47 没跑（checker 档），推的。**

条款：用户 2026-10-05 定乙，原话记在 `records/2026-09-30-里程碑三开工第一轮.md:628`：「乙在没有读故障的盘面上也让 I-3.1 红一版」；E156 第 6 次页内的写法是「core 照用户 2026-10-05 定乙，把挂载那一刻读回全零的根环槽钉在它可能住过的最旧 txg」（`.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:22`）。core 的定义：`crates/singlefs-core/src/allocator.rs:990`：「/// 挂载那一刻读回全零的槽：回收门槛照「自证不过」那一类钉在它可能住过的最旧 txg」，解开时点写在 `crates/singlefs-core/src/allocator.rs:992`：「直到这个进程把那个槽写掉，或进程内记根时重读读出有效根、读出被抛弃的根才解开；」与 `crates/singlefs-core/src/allocator.rs:993`：「重读读回全零不算确认为空，照旧钉着。它不算「这个进程知道住着根的槽」。」

**（a）谓词逐项对读**

| 项 | core | 装置 |
|---|---|---|
| 什么算「读回全零」 | `crates/singlefs-core/src/mount.rs:1757`：「if bytes.iter().all(」（先判全零，再 `parse_slot`，读 `physical_block_size` 字节） | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:2191`：「Some(bytes) if bytes.iter().all(」，臂的结果是 `RootRingSlotContent::AllZero`（自证不过那一支里再看原字节，同一个字节数；一个槽自证过与全零不可能同时成立） |
| 钉在哪个 txg | `crates/singlefs-core/src/mount.rs:1803`：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」第一次碰到的最小 txg | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:2239`：「(0..ring_length.min(first_txg_of_the_new_instance)).find(」 |
| 进门槛的方式 | `mount.rs` 把它与有效根一起取 `min`（`oldest_valid_root_in_the_ring`） | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1881`：「.union(&all_zero_slots_still_pinned)」之后交 `reclaim_threshold_read_from_the_ring_image`（函数体不动，`:1569`） |
| 写掉才解开 | `record_root_written_by_this_process` 用写出的根换掉槽上的记录 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1861`：「let released_by_write = occupant.is_some_and(」，闭包是 `occupant_txg >= first_txg` |
| 重读解开 | 记根时槽成了最小那个才重读；读出有效 / 被抛弃的根才解开 | `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1862`：「let released_by_reread = occupant.is_some_and(」，闭包是 `occupant_txg < first_txg`：槽上有挂载前的根就算 |

最后一行装置比 core 宽：core 只在「记根时该槽成了最小」那一刻才重读，装置每个观测点都看槽上有没有根。这一族没有读故障，槽挂载时读回全零、之后只有这个进程能往里写，所以「槽上出现挂载前的根」永远不会发生；产物 `released_by_reread_only=0`，装置只计数、不证它（登记「它答不了的」③，`.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:92`：「③ 「重读读出根才解开」那条路在这一族上一次没出现」）。所以这一项的一致是「在这一族上不可区分」，不是证出来的一致。
同一行（`:92`）还写了：「② 这一族只造第一圈里的前洞」：那一槽在 [0, 第一个 txg) 里只住得下一个 txg，「钉在最旧 txg」与「钉在能住的最大 txg」同值，这一族分不出 core 钉的是哪一个。读 core 的 `txgs_ring_slots_bad_at_mount_can_hold` 两个循环（`crates/singlefs-core/src/mount.rs:1780`：「fn txgs_ring_slots_bad_at_mount_can_hold(」起）是各算各的，全零槽那一份只取最旧（`mount.rs:2035` 一带），与定乙的话一致。

**（b）会不会与 core 一起错**

装置的钉住表与 core 的钉住是两份独立的实现，control 产物证了它们不是同动的。我核了产物与登记里写的数（原样）：

```
$ sha256sum research/results/e156-alloc-basis-counts-2026-10-05-r2.out
1c8812b19456165b10b6cd01f3e00fb2ddd1e3673f700cedac151f5da778176b  e156-alloc-basis-counts-2026-10-05-r2.out
$ grep -a 'name=verdict stage=second ' <r2.out> | grep -o 's11_a15_real_below_points=[0-9]* s11_k10_violation_points=[0-9]*'
s11_a15_real_below_points=0 s11_k10_violation_points=0
$ grep -a 'name=verdict stage=second ' <…all-zero-pin-removed.out> | grep -o 's11_a15_real_below_points=[0-9]* s11_k10_violation_points=[0-9]*'
s11_a15_real_below_points=16 s11_k10_violation_points=34
$ grep -a 'name=all_zero_pin_summary' <两份> 的前半（原样，两份相同）
E7RESULT name=all_zero_pin_summary cells=38 cells_with_pins=16 pins_installed=64 threshold_lowered_points=34 released_by_reread_only=0 bad_slots_at_mount=0 bad_slots_still_pinned=0
$ sed -n 2,3p research/results/e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt（截到差集）
main: k10_points=0 f24_points=0 …
pc: k10_points=34 f24_points=34 k10_eq_L6=True f24_eq_L6=True k10_minus_L6=[] L6_minus_k10=[] f24_minus_L6=[] L6_minus_f24=[]
```

control 是在副本里把 core 的 `mount.rs` 一行换成「挂载时一个全零槽都不装」（`research/results/e156-alloc-basis-counts-2026-10-05-all-zero-pin-removed.patch`，核过它改的就是 `crates/singlefs-core/src/mount.rs:2028`：「&ring_slots_reading_back_all_zero,」那一行）：装置的 `pins_installed`、`threshold_lowered_points` 一个字不动（它按自己读的槽钉），而 K10 与 F24 在 34 个观测点红，这 34 个点与锚点脚本（Python，与装置、core 都不共用代码）独立推出的 L6 逐点相等（`k10_eq_L6=True`）。三条互相独立的路对上了一件事：core 的钉住、装置的钉住、Python 的钉住。
**共用部分**：装置经 `use singlefs_core::recovery::{… read_root_ring_slot …}` 与 `use singlefs_core::root_ring::{slot_offset, target_for_publish, …}`（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:50`：「slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,」）直接调 core 的读槽与 txg→槽映射；`target_for_publish` 若错，装置的钉住与 core 一起错。锚点模型（`anchor_model`）用「槽号 = txg 对 3S 取余」独立算，并且「逐行相同」断言过，第一圈内一一对应（`:2208` 一带的注释 A2），所以这条共用路在第一圈里被 Python / 锚点模型夹着；第一圈之外这一族不造。
推翻条件：用一份 core 与锚点模型在第一圈内读法一致、而 `target_for_publish` 错位的 core 副本，control 里 K10 仍为 0（那就是装置与 core 共错）。我没造。

**（c）两个对照分不分得出它们要分的东西**

- PC-钉门槛（第 5 次加）：格表里 Hh(1, 后) 六格，洞让 txg t − 3S 的旧根留在 t 的槽里，门槛钉在它上面到 t + 3S；交 k = 0 的 A10 判定、期望 {3S}，判「不符」并且滞后集合含 6S − 1。分的是「真实分配器把回收推迟一整圈」与「没有洞的 3S 时间表」。第 6 次产物里六行与第 5 次逐字节相同：`name=pc_pinned_floor_summary cells=6 seen_real_cells=6 seen_every_cells=6`（上面产物原样）。后洞里挂载时读回全零的槽为空，所以全零钉不碰它。
- PC-全零钉（第 6 次加）：core 去掉钉住的副本，前洞格 K10、F24 红在 34 点，等于 L6。分的是「core 钉着全零槽」与「不钉」。
- 它们不分的两处：钉在最旧还是最大 txg（上面（a））；重读解开那条路（`released_by_reread_only=0`）与读坏的槽（`bad_slots_at_mount=0`），这一族不造。
推翻条件：PC-全零钉的 34 点与 L6 不等；或 PC-钉门槛六格有一格 `judge_*_mismatch=0`。产物里没有。

**（d）改调用点不改函数体，M36 与 M42**

`reclaim_threshold_read_from_the_ring_image`（`e156_allocation_basis_counts.rs:1569`）函数体一字未动，M36（变异表行 802）的原文是它的函数体，静态核锚点命中 1 次。调用点现在传的是有效根并上钉住槽的并集（`:1881`），第二处调用（`threshold_lowered_by_all_zero_pins` 那一比）传的仍是只有有效根的集合。M36 把门槛改成「最新有效根 − 根数 + 1」：并集是连续 txg 时（前洞：钉住的那一槽补上了缺口）这个式子与真正的 min 同值，所以 M36 在前洞格上看不出来；它的点名测试是后洞那条（`hh_back_hole_at_four_slots_per_region_pins_the_threshold_to_txg_four_until_txg_twenty_eight`），后洞没有全零钉，集合不连续、式子偏低，红。M42（行 808）改的是 `crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs:1900`：「let every_publish_floor = ring_threshold;」，点名的是无洞那条（`hh_without_holes_at_four_slots_per_region_reclaims_each_tree_table_one_ring_length_after_release`），无洞时钉住表空，`ring_threshold` 就是函数的纯输出，M42 照红。两行点名的测试不同、各自的射程里另一行的改法碰不到，没有互相掩护。M45（行 2785）、M46（行 2786）、M47（行 2787）三行点名同一条 U20，但改的是三处不同的原文（并集、锚点模型的 fold、写掉的判定），各自单独施加时 U20 里各有一句断言红（读码）。
没跑：装置内联单测 19 条、M35、M44–M47 的红绿（checker 档，归提交时）；p6 静态核（锚点恰好一次、点名的 fn 在）这几行都过，见 P6。
推翻条件：M36 在前洞格上被当成「抓到」（那说明并集不连续，我读错了）；或 M42 在有钉住的格上点名的测试仍绿。我没跑，不能给「没出现」。

## P6　变异行

**判定：（a）锚点各恰好命中一次：一致；（b）点名的测试跑得到：一致（harness 档的都跑了，checker 档的读码）；（c）合入后基线绿：harness 档这一轮新行的基线绿，**但变异表里点名那条抬 F 慢档用例的 11 行基线今天是红的**（冲突，见下）；（d）两行原文相同而点名的测试一起错：有，四组，都是有意的（判决 N5 已认「同改法不同测试，门禁判法允许」），没有两行点名同一条测试。**

**（a）（b）静态核**（`tools/rerun.sh static`，原样）：

```
$ python3 scripts/mutation/crates-mutation-rows.py static-check <冻结副本拷贝>
COPIED	crates Cargo.toml Cargo.lock litmus scripts/hosts/run-with-memory-cap.sh scripts/mutation/crates-mutation-rows.py
CHECKED	2817	299	9
```

没有 MALFORMED、ANCHOR、NAME、ROW、STRAY、LITERAL 行（`out/static-check.txt`）。299 比我拷来的 `.rs` 份数多一份是我加的那份 harness 探针；冻结副本不带 `litmus/` 时有一行 ANCHOR（第 1352 行的 litmus 文件不存在），与这一轮无关，拷上 `litmus/` 之后消失。
`tools/p6_rows.py`（`out/p6-rows.txt`）对改动全文里 23 行追加、E156 的 M35 / M44 / M45 / M46 / M47 五行、1 行删除逐行核：整行在冻结副本表里恰好 1 次、锚点命中 1、目标文件都在、点名的 fn 在目标源码里；旧名在表里零行（删除生效）；表里变异名重复 0 个。过滤串那一栏有两行显示「选中 False」（行 2776、行 2795），是我的脚本只看函数名子串、这两行过滤串带模块路径（`model::tests::…`、`crash_injection::tests::…`）造成的假阴性：行 2776 我跑了，点名的测试被选中并红（见 P1）；行 2795 是 checker 档 lib 单测，路径与 `crates/singlefs-checker-tier/src/crash_injection.rs` 里 `mod tests` 对得上，读码。
baseline（harness 档，`out/baseline-runs.txt`）：`--lib` 86 过 1 忽略、`checker_known_bad_images` 102 过、`overwrite_in_one_instance` 12 过、`root_ring_turn_in_one_mount` 3 过，这一轮新行点名的 harness 档测试在基线上都过。checker 档那几行（2792、2795、2796、2797、E156 五行）的基线我没跑，主 agent 的背景里记过「基线 4 行 ok」。

**（c）基线红的 11 行**

变异表里 `-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first` 这个参数的行有 11 行（行 32、33、34、39、43、44、46、190、192、2016、2017，命令 `awk -F'\t' 'index($5,"reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first")>0{print NR}' crates/mutations.tsv`）。它们点名的测试在今天的冻结副本上不加变异就红（P3（e）的原样输出），所以这 11 行的「必须红」今天什么也证明不了，整表复跑时基线就红。这一条与 i55 调查报告第 4 点（`/tmp/claude-1000/i55-reuse-investigation/report.md`）同一件事，那时红的原因是 I-5.5 等四条；32-2 的四支豁免本来要把基线修绿，但冻结副本里又多了 I-9.18–I-9.23 六条（P3（e）），基线仍红。
并排：主 agent 观测写「抬 F 慢档那条在主工作区单跑 ok」；我在 `crates/` 与主工作区逐字相同的冻结副本上跑同一条，输出 `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out; finished in 3.92s`，断言 `I-9.18 在 E 之后的镜像上要成立`，`left: NotApplicable("走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判")`。
推翻条件：主工作区今天单跑这条绿；或把六条豁免补上之后基线绿而这 11 行在变异下仍红。

**（d）同一处同一改法、点名不同测试的行**（`out/p6-rows.txt` 的最后一列，原文替换文逐字相同）：

| 组 | 行 | 点名的测试 |
|---|---|---|
| F1 池级 checker 不记树表读不出的版本 | 2777、2778、2779 | 各点名新版 / 前版 / 第 0 代那一条；施加其中任一行，三条一起红（我跑行 2777 时三条都红：`passed=5 red=3`，行 2778 只筛第二条所以 `red=1`） |
| R3 释放代少记一 | 2788（32-2）、2793（33-a） | 前者点名 `allocated_statistic_equals_…`，后者点名 `repeated_overwrites_go_past_the_812_…`；我跑行 2793 时 4 条红（含这两条） |
| B3 根环公式少一槽 | 2789、2794 | 同上；行 2794 下 `red=2`（这两条） |
| 「记根时环转了也不回收」 | 2792（E171 U11）、2746（E156 M35）、2731（harness 档） | 三个装置 / 用例各自点名 |
| SysPre：F_生效 不与系统配置取 max | 2797（33-b，checker 档）、1376（harness 档） | 两条各自点名 |

这些都是判决 N5 写的「三组同改法不同测试，门禁判法允许」（`research/prompts/m3-formal-code-b31-r1-main-verification.md:62`）的形态。「一起错」的含义：同一个变异让几条测试一起红，任何一条点名的测试被删或改名，另外几行不会替它作证——每行点名自己的那条，所以行与行之间没有互相掩护，也没有一行靠别的行的测试。我没发现两行点名同一条测试。
推翻条件：有两行原文替换文相同、点名同一条测试（那就是重复行）；`p6_rows.py` 的「同一改法的别的行」那一列里同一个名字出现两次。没有。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1-B3 | 一致 | 135 片逐片钉死、与实现扇出相等；入库行与 8 个非等价的封顶 / 互换 / 取余变体全红，封顶 134 是等价变体 |
| P1-B4 | 一致，取样点之内 | 36 组合罩住按位置错的 9 个变体；只在装 60–118 片时才错的变体存活（取样点碰不到，射程限度） |
| P2-F1、33-c 合法历史 | 一致 | 162 个非慢档二进制全过、探针行只在刻意弄坏树表的用例；慢档 55 条过 0 行；6 条慢档没跑完 |
| P2-同形整版不进评估 | 规则没说 | 树表指针全零、根记录那一项非零而树不读出：读码坐实今天不进评估；kb 对这种根的口径是「没有树表」；套件里 0 用例走到 |
| P2-F2 与 core | 一致，checker 判得更宽 | 三段谓词、5 个调用点逐点同；checker 比 core 多判根记录里的四条指针（MAC / nonce 本来就是这样的先例）；GPU `head_ok` 逐格同（读码，没跑 GPU） |
| P3-每版跑 checker | 一致 | R3 红 I-3.9（txg 4）、B3 红 I-7.17（txg 21）、d 红（txg 4）；R4 不红，与判决一致 |
| P3-R4、d 点名的已有用例 | 一致 | 基线 3 过，变异下红在 `root_ring_turn_in_one_mount.rs:108` |
| P3-U11 | 一致（推的） | 同一处变异在 harness 档只红 I-3.1；U11 本身没跑 |
| P3-四支豁免 | 部分弱 | I-4.4、I-4.6、I-4.7 各一种专门理由；I-5.5 认 checker 缺省理由，分不开「没有对象」与「没判」 |
| P3-慢档用例今天的结局 | 冲突 | 冻结副本里仍红在 I-9.18，I-9.18–I-9.23 六条不适用无豁免；与「单跑 ok」对不上 |
| P4-C551 | 一致 | 末尾版本判全部不变量，I-3.1 真评估过，推翻条件满足 |
| P4-① 期望 `Some(1 + 3)` | 一致（推的） | 与 G5 / 按镜像计的写法同一口径；用例没跑 |
| P4-② 新造法 | 一致 | harness 档探针基线绿，四行相关变异下红在下界那一判（`VersionWithoutFile`） |
| P4-旧名被找 | 有 | 两份 E171 `bad-implementations-117.tsv` 第 33 行按旧名引用，找不到走补位；旧行替换文本来就编不过 |
| P5-谓词同一 | 一致（这一族上） | 重读解开那条路这一族不出现，是「不可区分」不是证出来的 |
| P5-装置与 core 共错 | 不一起错；共用三个核心函数 | control 产物：core 去掉钉住，装置的钉住表不动，K10 红 34 点 = Python 推的 L6 |
| P5-两个对照 | 各分得出 | 分不出钉最旧还是最大 txg、重读解开、读坏的槽（登记已写） |
| P5-M36、M42 | 无互相掩护（读码） | 点名的测试与射程不同；没跑 |
| P6-锚点、点名的测试 | 一致 | 静态核无问题行；harness 档行都跑到 |
| P6-基线 | 部分冲突 | 这一轮新行基线绿；点名抬 F 慢档用例的 11 行基线今天红 |
| P6-同改法点名不同测试 | 有，四组 | 门禁允许，没有两行点名同一测试 |

## 要主 agent 跑的命令

1. 慢档里一次调用 225 秒内没跑完的 6 条合法历史（探针版）：`REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b33-r1/tree WORK=<草稿目录> CAP=10 MEM=8G bash research/prompts/m3-formal-code-b33-r1-sonnet-model/tools/rerun.sh setup sweep sweep-slow`（后台跑，不设超时；`sweep-slow` 的单任务时限 `BUDGET` 默认 3000 秒）。看 `$WORK/sweep-ignored/*.rc` 与 `grep -h '^B33' $WORK/sweep-ignored/*.log`：任何一行 B33F1 / B33C 出现在不是故意弄坏树表的测试上，就是 P2（a）被推翻。没跑完的是 `fault_injection_large_tier_from_the_environment`、`one_fixed_history_injects_twelve_faults_and_none_of_them_panics`、`allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall`、`random_histories_large_tier_seeds_and_length_from_the_environment`、两条 `unit_area_wall_sampling_on_small_devices_…`。
2. checker 档（我不跑）：U11 与 E156 内联单测：`SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 10 cargo test -p singlefs-checker-tier --bin e156_allocation_basis_counts` 看 `19 passed`，另 `--bin e171_three_oracle_collusion_after_the_oracle_fixes -- the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1`；33-a 的 ① 与 33-b 的 ② 两条用例主 agent 已跑过，我没复核。
3. 抬 F 慢档用例今天是否红：`cargo test -p singlefs-harness --test reuse_after_raising_the_floor -- --ignored --exact raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`（在主工作区单跑），看是不是 `I-9.18 在 E 之后的镜像上要成立`。

## 没做什么

- 没跑任何 checker 档的东西：`singlefs-checker-tier` 的测试与装置内联单测、GPU 对拍、E156 装置的 19 条单测与 M35 / M44–M47 的红绿、U11、层 0、`crates/mutations.tsv` 整表；涉及它们的结论都标了「推的」或「读码」。
- 33-b 那条 checker 档用例本身没跑，只在 harness 档量了同一造法（探针，不是它自己）。
- 没造「树表指针全零而根记录那一项非零、那棵树读不出」的坏镜像（P2（b）读码）；没造指针头算法类型在根记录里的镜像（P2（c）读码）。
- 没复现 i55 调查的插桩（I-5.5 的 `versions_not_judged=[]`）与旧造法的红因（红二调查）。
- 没判「抬 F 慢档用例该怎么收」与 E171 补位怎么记：是主 agent 的事。
- 没动主工作区、冻结副本与 `.claude/`；没做 git 写操作。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b33-r1-sonnet/`：交回前我删了自己建的编译目录 `work/target`（25G）、仓副本 `work/tree`、复跑自检的 `rr-test/`；留下 `work/log`、`work/sweep`、`work/sweep-ignored`（只有日志，约 5M）与 `work/bin`、几份小脚本，供主 agent 看原始输出，不是编译目录也不是仓副本。

## cite-check 末行

命令：`python3 scripts/agents/cite-check.py research/prompts/m3-formal-code-b33-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b33-r1/tree --root . --background research/prompts/_m3-formal-code-b33-r1-background.md`

末行：`✓ 核了 79 处引文，对上 79 处，没判 0 处（逐处列在上面）`
