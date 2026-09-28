# 实审 B1 报告（代码审阅 6c 第 5、6、8 条：checker 盲区）

写于 2026-09-27。实现员，主工作区。

## 一、结论

- 第 5 条（同一根里的交叉链接）：checker 按「版」（根环里的一条根，或由记录施加出来的一版）记一份「这一版跟着指针引用过的落点」，同一版里第二次引用同一个 (设备, 槽) 判 I-5.1；每走一版清空，不同版之间共享照旧去重、不重走。一条指针自己两条位置条目同落点算一个引用（那一格归 I-2.5）。
- 第 6 条（坏镜像 panic）：审阅点名的两处都改了，另外扫出同形三处，都走得到，也一起改了：多层码 2 树节点的 key 宽守卫、反向链计数器 0、按条目往下数节点时 u8 层级溢出、单元区容量减法下溢、「空闲 + 已分配」溢出。**`index_node_view` 拒 `entry_width < key_width + 86` 那一句没照做**，理由见第三节，交主 agent。
- 第 8 条（中央映射条目）：认不得的类标签判 I-1.6、不读；指到的范围进 `references`（参与 I-3.1、I-5.1）；读出来的单元头里的类标签与 key 比（I-1.6），出生身份（诞生代号、写序实例、尾段）与 key 比（I-1.2）。key 里的出生树不比（C289 还开着）。
- 新测试 13 条，都在副本里用变异证过红（见第四节）；变异表追加 15 行（865–879），其中 2 行留给 59 号。
- 已有用例里有一条因判严改判，已改：`checker_known_bad_images.rs` 的 I-4.2 那份坏镜像要连映射 key 一起改，不改就多红 I-1.2（副本里回退这一步，`the_birth_identity_bad_images_redden_only_their_own_invariant` 红，原样见第四节）。
- 登记给我的门禁里 33 号、74 号红，红因都不在这一轮改动里（第五节）。

推翻条件：59 号复跑时追加的 15 行里有一行点名的测试不红；或层 0、崩溃注入在合法状态上因 I-5.1 / I-1.2 / I-1.6 / I-3.1 新红（说明「同一版里一个落点只引用一次」「映射 key 与单元头相符」在实现写出的合法盘面上不成立）。

## 二、这一轮写过的文件

- `crates/singlefs-checker/src/walk.rs`：三条的改动（第三节）。
- `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`：新文件，13 条用例。
- `crates/singlefs-harness/tests/common_tree_split/mod.rs`：把 `second_transaction_supplement_two_tree_split.rs` 里改单元、补校验和那一组辅助函数搬过来并公开（`read_unit_on_device`、`write_unit_to_the_same_slot_on_both_devices`、`reseal_unit_by_its_class`、`root_ring_slot_offsets`、`propagate_the_new_checksum`），新测试与原测试共用，不手抄第二份。
- `crates/singlefs-harness/tests/second_transaction_supplement_two_tree_split.rs`：删掉搬走的那组函数，改用上面那几个。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：I-4.2 那份坏镜像加一步：映射根里码 1 那条 key 的出生 txg 同步改成 4（新函数 `rewrite_the_data_unit_mapping_key_birth_txg`）。
- `crates/mutations.tsv`：末尾追加 15 行（第 865–879 行），变异名都以「实审 B1」开头：
  1. 第 5 条：checker 同一版里第二次引用同一个落点判成立（两个 inode 的 extent 指同一个数据单元照绿）
  2. 第 5 条：checker 每走一版不清空「这一版引用过的落点」（不同根共享的单元被判成第二次引用）
  3. 第 5 条：checker 把一条指针自己的两条同落点位置条目算成两次引用
  4. 第 6 条：多层码 2 树节点头里的 key 宽不是树的 key 宽照样往下切（映射根 key 宽 60 越界 panic）
  5. 第 6 条：同一变异，点名映射叶 key 宽 20 那一条
  6. 第 6 条：反向链拿计数器减一找逻辑前一条（计数器 0 下溢 panic）
  7. 第 6 条：按条目往下数节点时拿孩子层级加一比父层级（层级 255 溢出 panic）
  8. 第 6 条扫出的同形：单元区容量拿盘上槽数直接减单元区起点（起点越过盘末下溢 panic）
  9. 第 6 条扫出的同形：空闲与已分配直接相加去比单元区（溢出 panic）
  10. 第 8 条：映射 key 的类标签不在登记表里也判成立
  11. 第 8 条：映射 key 的类标签认不得照样按 32 KiB 硬读（留给 59 号）
  12. 第 8 条：映射条目指的物理范围不进引用集合
  13. 第 8 条：不拿映射 key 的类标签比被指单元头里的
  14. 第 8 条：不拿映射 key 的出生身份比被指单元头里的
  15. 第 8 条：映射 key 说码 2 也按 32 KiB 读、记两槽引用（留给 59 号）
- 没动：`crates/singlefs-checker/src/lib.rs`、`image.rs`。
- 仓外（草稿）：`/tmp/claude-1000/impl-rev-b1/` 下的日志、证红脚本与 `progress.md`。

## 三、三条各改了什么（`crates/singlefs-checker/src/walk.rs` 现在的行号）

### 第 5 条
- 第 184 行新字段 `placements_referenced_in_this_version`；第 447 行 `begin_walking_a_version` 清空它，`walk_root` 与 `walk_version_applied_only_by_records` 开头各调一次。
- 第 455 行 `note_references_of_a_pointer_followed_by_the_walk`：替换原来四处 `for location in &pointer.locations { self.note_reference(...) }`（`read_index_node`、`walk_inode_root`、`walk_instance_table_chain`、`walk_extent_data_pointer`）。每条位置条目照旧进 `references`，再查这一版有没有引用过这个 (设备, 槽)：引用过就判 I-5.1，说明写「盘 d 槽 s：被同一版引用了两次——先是 …，又是 …」。
- 数据单元那一处的说明文字从「数据单元」改成「inode N 偏移 O 的数据单元」，判红时说得出是哪两个引用。
- `visited_units` 照旧整个走读共用（不同版之间走过就不再走）。
- **留下的缺口（推的，没有会红的镜像）**：更老的一版引用了一棵已经被前一版走过的共享子树 S，同时又在 S 之外另指 S 里面的某个后代单元，这种交叉链接判不出来——S 没重走，它的后代没记进这一版。最新根先走、从空集合起走全，所以最新根上没有这个缺口。要补就得记「走过的单元 → 它的孩子」，遇到共享单元时把整棵子树的落点并进这一版。今天的镜像造不出「两版共享一棵多层子树」的坏样本来证红，这一轮没做，交主 agent 定要不要做。

### 第 6 条
- 第 828 行（`walk_code_two_subtree`）：头里自述的 key 宽 ≠ 这棵树的 key 宽时（`read_index_node` 已按 I-1.1 判红），记走读失败、不往下走。**这一处就是审阅那两个 panic 的根因**：条目宽只按「树的 key 宽 + 86」守，切片却按头里的 key 宽切；key 宽窄了之后，叶头里的 key 交回父节点按 27 字节的形态逐字段读，同样越界（`lib.rs:187` 的 `read_u32`）。
- 第 4206–4209 行（`judge_journal_back_chain`）：`counter.checked_sub(1).and_then(...)`。计数器 0 没有逻辑前一条，与「前一条读不出」一样判不了、跳过。没把计数器 0 判成违例：条款里「计数器从 1 起」没有对应的不变量编号（第六节）。
- 第 2298 行（`note_every_node_below`）：`node.level.checked_sub(1) != Some(child.level)`，原来的 `child.level + 1` 在 255 上溢出。
- 第 5229–5236 行（I-5.2）：单元区容量改成 `checked_sub`，起点越过盘末时容量取 `None`，这一格判红（与盘容量读不出同一个处理）；`free + allocated` 改成 `checked_add`，溢出也判红。
- **没做的：`index_node_view` 拒 `entry_width < key_width + 86`**（规格第 6 条原文）。理由：① 它挡不住 key 宽 20 那一种 panic（叶不带子指针，只能对层级 > 0 的节点这么判），根因是上面那一处 key 宽守卫，而守卫已经把 key 宽 60 那一种也挡住了。所以再加这一判，没有一条测试能单独证它会红。② 它会抢在 `entry_width_holds` 前面：记账树内部节点条目宽窄于 108 时，今天红在 I-1.10，加了之后变成「条目区解不开」的走读失败，I-1.10 不再判。要照原文做，交主 agent 定。

扫同形（切片与减法）的结果在第七节。

### 第 8 条（第 706 行 `walk_central_mapping_entries`，第 283 行 `MappingKeyUnitClass`）
- key 偏移常量：类标签 0、出生 txg 9、实例代号 17、尾段 21（D19 已定项 6 / 10），按字段表另写一份，不用实现的解析。
- 类标签不在 1 / 2 / 3 里：判 I-1.6「映射条目：key 的类标签 X 不在登记表里（1 / 2 / 3），它指的单元不读」，跳过读。
- 跨度按 key 的类取：码 2 一槽、码 1 / 码 3 两槽。两条位置条目都经 `note_reference` 记进 `references` 与 `location_entry_checksums`；不进「同一版第二次引用」那一判。
- 读出来之后：头里的类标签 ≠ key 的类标签判 I-1.6；相等时出生身份与 key 比，不等判 I-1.2。尾段码 1 读 6 字节事务号，码 2 / 码 3 读 4 字节出生序号。
- 码 2 / 码 3 key 末尾那 2 字节补零没判（D19 已定项 10「末尾补零到 27」），出生树没比（C289）。
- `references_of_root` 里那句「主走读的 `note_reference` 也不数它们」已经过时，改成现在的说法。

## 四、新测试与证红（每条先在今天的 checker 上跑、看它红，再改；变异在副本里做）

改之前在主工作区跑新测试二进制（日志 `/tmp/claude-1000/impl-rev-b1/before-fix.log`）：`test result: FAILED. 1 passed; 9 failed`（当时 10 条），panic 落在 `walk.rs:4058:36`、`lib.rs:187:29`、`walk.rs:724:45`、`walk.rs:2152:12`，其余是断言：I-5.1、I-1.2、I-1.6、I-3.1 / I-3.11 那几份镜像上 `left: []`，也就是一条都没判红；类标签 0 那份红的是 `["I-2.1", "I-4.8", "I-7.2", "I-7.4"]`（硬读了）。「共享单元不是交叉链接」那条改之前就绿（它守的是改法本身）。单元区起点、「空闲」溢出、指针两条位置条目同落点这三条是改完才写的，只在副本里证了红。

副本：`rsync -a --exclude target --exclude .git` 拷出、用它自己的 target。基线：新测试二进制 `test result: ok. 13 passed; 0 failed`，`checker_known_bad_images` `test result: ok. 36 passed; 0 failed`，基线红集为空。每条变异照变异表那一行改 walk.rs，跑新测试所在的整个二进制，之后从原件拷回并 `touch`。

| 变异表行（第几行） | 改坏哪一行 | 哪条测试红（红在哪） | 同时红的 |
|---|---|---|---|
| 865 | `judge("I-5.1", false` → `true` | `two_inodes_whose_extents_name_one_data_unit_in_one_root_redden_only_the_disjoint_ranges_invariant`（:219 `left: []`，右边 `["I-5.1"]`） | 无 |
| 866 | `.clear()` → `.retain(\|_, _\| true)` | `a_unit_shared_by_the_roots_of_two_versions_is_not_a_cross_link` | 另外 8 条（干净镜像那句「全绿」先红） |
| 867 | `if is_an_earlier_location_entry_of_this_pointer` 前加 `false &&` | `one_pointer_whose_two_location_entries_name_one_placement_is_one_reference_not_a_cross_link`（:273，I-5.1「盘 0 槽 50180：被同一版引用了两次」） | 无 |
| 868 / 869 | key 宽守卫前加 `false &&` | `a_central_mapping_root_whose_key_width_is_not_the_mapping_key_width_is_judged_without_a_panic`（panic `walk.rs:871:45`）、`a_central_mapping_leaf_whose_key_width_is_narrower_than_the_mapping_key_is_judged_without_a_panic`（panic `lib.rs:187:29`） | 两条互为同时红 |
| 870 | `checked_sub(1)…` → `records.get(&(counter - 1))` | `a_journal_record_whose_counter_is_zero_is_judged_without_a_panic`（panic `walk.rs:4206:36`） | 无 |
| 871 | `node.level.checked_sub(1) != Some(child.level)` → `child.level + 1 != node.level` | `a_central_mapping_leaf_whose_level_is_the_largest_level_byte_is_judged_without_a_panic`（panic `walk.rs:2298:12`） | 无 |
| 872 | `checked_sub(unit_area_start_slot)` → 直接减 | `a_system_configuration_whose_unit_area_starts_past_the_device_end_is_judged_without_a_panic`（panic `walk.rs:5231:22`） | 无 |
| 873 | `free.checked_add(allocated) == Some(capacity)` → `free + allocated == capacity` | `an_accounting_row_whose_free_bytes_overflow_the_sum_is_judged_without_a_panic`（panic `walk.rs:5236:126`） | 无 |
| 874 | 未登记类标签那一判 `false` → `true` | `a_mapping_key_with_an_unregistered_class_reddens_only_the_class_tag_invariant_and_is_not_read`（:700） | 无 |
| 876 | 映射条目的 `note_reference` 换成 `let _ = …` | `a_mapping_entry_naming_an_unallocated_copy_counts_as_a_reference_and_reddens_the_allocated_bytes_invariants`（:736） | 无 |
| 877 | `header_class_tag == key_class_tag` → `true` | `a_mapping_key_whose_class_is_not_the_class_of_the_unit_it_names_reddens_only_the_class_tag_invariant`（:672） | 无 |
| 878 | `judge("I-1.2", matches_the_key` → `true` | `a_mapping_key_whose_birth_identity_is_not_the_one_in_the_unit_header_reddens_only_the_birth_identity_invariant`（:650） | 无 |
| 875、879 | — | 留给门禁 59 号，没在副本里证 | — |

每条变异的日志在 `/tmp/claude-1000/impl-rev-b1/proof/m<行下标>.run.log`，每份末尾都是 `exit=101`。

改已有用例那一处也证了：副本里把 `checker_known_bad_images.rs` I-4.2 坏镜像新加的那一步删掉，`the_birth_identity_bad_images_redden_only_their_own_invariant` 红，原样：
`assertion ``left == right`` failed: 判红的该只有 I-4.2：[… ("I-1.2", Violated("映射条目：key 里的出生身份（诞生代号 3、实例 1、尾段 1）与它指的单元头里的（诞生代号 4、写序实例 1、事务号 1）不符")) …]  left: ["I-1.2", "I-4.2"]  right: ["I-4.2"]`

## 五、交回前的验证（末行原样）

- 动到的测试二进制，整个跑，经内存包装 8G、`capped.sh 5`：
  - `checker_cross_links_malformed_nodes_and_mapping_entries`：`test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s`
  - `checker_known_bad_images`：`test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 171.23s`
  - `second_transaction_supplement_two_tree_split`：`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.55s`
  - `second_transaction_mapping_node_admission`（用了 `common_tree_split`）：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s`
  - `singlefs-checker --lib`：`test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
  - 也用了 `common_tree_split`、这一轮没跑的：`second_transaction_supplement_two_tree_split_layer0`、`second_transaction_position_addressed_trees_layer0`（名字含 layer0；连 `--no-run` 也被重型测试闸拒了）。它们靠下面的 `cargo build --all-targets` 编译通过。
- `cargo build --offline --all-targets`：``    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 28s``（exit=0）。
- `cargo fmt --all --check`：我的文件都格式化过了。格式化用的是对那两个测试文件直接跑 `rustfmt --edition 2021`（它顺带格式化了 `common_tree_split/mod.rs`；`common/mod.rs` 前后 md5 相同），为的是不碰别人的文件。仍有 diff 的都不是我的文件：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/tests/instance_acquisition.rs`、`writable_mount_refuses_a_device_identity_handed_in_twice.rs`、`writable_mount_refuses_caller_parameters_disagreeing_with_the_system_configuration_on_disk.rs`。
- clippy（check.sh 那一套 lint）：
  - `-p singlefs-checker --all-targets`：``    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s``（exit=0）。
  - harness 的 `--lib` 加上面四个测试目标：``    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.30s``（exit=0）。
  - 全仓 `--all-targets` 红，末行是 ``error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts") due to 3 previous errors``；另有 ``error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 4 previous errors``。都是 `shadow_unrelated`，落在 E156 / E158 执行员的 bin 里，不是我的文件。
- 登记给我的门禁阶段（`nice -n 19 bash .claude/gate.d/<文件>`）：
  - 33：exit=1，末行 `    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。` 腐化的是第 93、170 行（`crates/singlefs-harness/src/fault_injection.rs`）和第 301、690 行（`crates/singlefs-core/src/mount.rs`），都不是我的行，也不是我改的文件。开工时同一阶段是绿的：`crates/mutations.tsv 859 条的原文各命中源码一次`，所以是这一轮里别的会话改那两份文件时弄腐化的。
  - 53：exit=0，`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
  - 74：exit=1，末行 `gate74 exit=1`（我追加的记号；阶段本身的末行是 `                签名是 ModelDisagreement 的，…`）。红的是 `second_transaction_supplement_three_random_history` 的 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`，断言原样：`覆盖写、第一个文件一次都没被「每块盘上都没有」拒过（用户数据那一处没跑到）`。单跑同一条（release、经包装）：`历史 32 段：跑完 32、以已知红收尾 {}、新发现 0`。checker 在这一段里一次都没判红、没 panic，红的是核心层落点拒绝的覆盖计数。判断与 checker 改动无关，是推的：checker 只判定、不影响历史怎么走，判红了才会停一段，而这里一次都没判红。更像是核心层（`transaction.rs`、`mount.rs`、`history.rs`，别的会话正在改）改变了拒绝的成员或次数。交主 agent。
  - 89：exit=77（本次未跑），末行 `    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`
  - 92：exit=0，末行 `      第一条纯 SSD 布局线（…）：.claude/kb/layout/02-second-txn.md`
  - 93：exit=0，`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…）`
  - 94：exit=0，末行 `    这一道判不了的：…`
- 负载：开工时 `ps` 看到 7 个 `e158_root_choice_repair` 和 1 个 `e156_allocation_basis_counts`（经包装），不是性能测量，照常 nice 跑；编译时出现过 `Blocking waiting for file lock on build directory`，等锁在秒级。

## 六、不变量清单：交主 agent 写回（我写不了 `.claude/kb/invariants.md`）

1. **I-5.1（物理范围不重叠）**（`.claude/kb/invariants.md:180`）的状态列建议补一句：
   「2026-09-27 实审 B1：同一版（根环里的一条根，或由记录施加出来的一版）里同一个 (设备, 槽) 被两条指针引用判红（`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`；一条指针自己的两条位置条目同落点算一个引用）；中央映射条目指的物理范围算进引用集合；坏镜像在 `crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs`。已知缺口：老一版引用一棵已被前一版走过的共享子树时，子树里面与子树外面之间的交叉链接判不出（共享子树不重走）。」
2. **I-3.1（已分配统计对得上）**（第 131 行）的状态列建议补：「中央映射条目指的物理范围算进遍历得到的引用（实审 B1），与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个。」
3. **映射 key 与被指单元头相符**：今天没有一条不变量的原文罩得住。I-1.6 说的是单元头里三处类标签相等，I-1.2 说的是写序按已发布谓词判，都不说「引用方记的 key 与单元头相等」；C294 欠的正是指针那一半（`.claude/kb/checks-owed.md:256`）。checker 这一轮按它拿指针比单元头的现成读法，把类标签那一半记在 I-1.6、出生身份那一半记在 I-1.2。建议新加一条（编号由主 agent 定），原句：
   「任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。」
   checker 今天没判的有两半：补零那 2 字节、出生树。新条立了之后，I-1.6 / I-1.2 这两处判定要搬到新编号下（变异表第 874、877、878 行的锚点会跟着改）。
4. 计数器 0：条款写了「计数器从 1 起」，但没有对应的不变量编号；checker 这一轮把它当判不了、跳过。要不要判成违例、记在哪一条下，交主 agent。

## 七、「同形的切片与减法」全扫一遍（`crates/singlefs-checker/src/` 全部四份）

扫法：`grep -n '\[[a-z_0-9]*\.\.[a-z_0-9]* *+\|\[[a-z_]*[a-z] *+ *[a-z_0-9]*\.\.'`（命中 21 处）和减法 `grep -n '[A-Za-z0-9_)\]] - [a-zA-Z0-9_(]'`（去掉注释、`saturating_` 与测试段之后命中 16 处），逐处看。

- 走得到、这一轮改了的：`walk.rs:871`（原 724，key 宽守卫）、原 4058（计数器 0）、`walk.rs:2298`（原 2152，u8 层级 +1，是加法，同一族）、`walk.rs:5232`（单元区容量减法）、`walk.rs:5236`（`free + allocated`，是加法，同一族）。
- 看过、有守卫走不到的：`lib.rs:88`、`lib.rs:90`（调用方给的是常量偏移）；`lib.rs:179–190`（读取偏移最大到 52 + 2k + 8，单元至少 16384）；`lib.rs:362`、`lib.rs:364`（先过 `check_unit`，长度等于 16384）；`lib.rs:386`、`lib.rs:539`（切之前判过「起点 + 声明长度 ≤ 单元长」）；`walk.rs:1372`、`walk.rs:3066`（判过「记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136」）；`walk.rs:1579`、`1688`、`1774`（key 宽取形态常量，条目宽先判等于 96 / 110）；`walk.rs:2291`、`2360`、`2449`、`2462`（判过 key 宽与条目宽下界）；`walk.rs:879`、`1588`、`2362`（层级 0 已提前返回）；`walk.rs:906`、`1202`、`1238`（下标或片序号 > 0 的分支里）；`walk.rs:4536`（同一组按计数器升序追加）；`position_addressed.rs:41`（根层级从 1 起）；`position_addressed.rs:145`（先判 `span_slots > 0`，且值域不会溢出）。
- 另一族，这一轮没改（乘法溢出、按盘上值分配内存，都读系统配置的几何，属 C476 那一族）：`image.rs` 的 `root_slot_positions` 里 `base_slot * SLOT_BYTES`、`region * prime_step * chunk_bytes`；`walk.rs` 的 `scanned_journal_records_of_device` 里 `journal_ring_start_slot * SLOT_BYTES + …`，以及 `(0..journal_ring_bytes / 4096).collect()`（按盘上环长分配）。这几处盘上值取得够大就会溢出 panic 或者撑爆内存（推的，没造镜像）。`lib.rs:386` 的 `chunks(entry_width.max(1)).take(entry_count)`（条目宽 0、条目数非 0 照收）是审阅第 38 条，不在这一批。

## 八、判严之后哪些已有用例可能改判

- 已确认并改了：`checker_known_bad_images.rs` 的 I-4.2 坏镜像（第一节）。
- 已跑、没改判：`checker_known_bad_images`（36 条）、`second_transaction_supplement_two_tree_split`（5 条，其中那份多层映射 / 记账树根改坏的语料照旧只红在各自那一格）、`second_transaction_mapping_node_admission`。门禁 74 号跑了随机历史二进制，26 条里 23 条过、2 条忽略，每一步之后都跑 checker；尾部摘要是 `I-5.1：判绿 4155 次、不适用 0 次`，唯一红的那条红在第五节说的覆盖计数上，checker `新发现 0`。
- 推的（没跑，名字含 layer0 或不在我的范围里）：
  1. 层 0 与崩溃注入里走读中途断掉的状态：映射条目指的范围现在算进 I-3.1 的并集，树走读没走到的单元可能经映射条目补进来。原来因为「遍历少算」红在 I-3.1 上的状态，可能转绿，也可能换一个差值；按 I-3.1 机理认的已知红清单（`history::allocation_statistic_mechanism`）有可能认不出、或者对不上段数。
  2. 手工改了单元头（出生身份、类标签）、只沿引用链补校验和、不改映射 key 的坏镜像：会多红 I-1.2 / I-1.6。找到并改了的只有 `checker_known_bad_images.rs` 那一份；其余调 `check_pool_image` 又断言「只红某几条」的测试文件我没逐个跑，按断言数排在前面的是 `second_transaction_step_three_formatted_pool.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_step_four_rollback.rs`、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`、`second_transaction_supplement_two_release_checksum_quarantine.rs`、`second_transaction_supplement_two_fsync_drop_and_devices_without_the_selected_version.rs`、`second_transaction_step_five_reuse.rs`、`second_transaction_supplement_two_instance_table_chain.rs`。
  3. 一条指针的两条位置条目改成同一块盘不同槽、而其中一个槽又被别的指针引用的坏镜像：会多红 I-5.1。
  4. `crash_injection` / `fault_injection` / `bad_disk_input`（harness 的 src）：checker panic 只会少、不会多；`KNOWN_PANIC_SITES` 是空表，不用改。

## 九、停下交主 agent 的设计问题

1. 规格第 6 条「`index_node_view` 拒 `entry_width < key_width + 86`」没照做，理由见第三节。要照原文加，I-1.10 在记账树内部节点上那一格会让位给走读失败。
2. 第 5 条共享子树的缺口（第三节）：要不要为老版本补上「走过的单元 → 孩子」的记录，把共享子树的落点并进每一版。
3. 映射 key 相符那一条：新立编号，还是扩 I-1.2 / I-1.6 的原文（第六节第 3 项）。
4. 计数器 0 要不要判成违例（第六节第 4 项）。
5. 门禁 74 号那条覆盖断言红、33 号四行锚点腐化，都不在我的改动里（第五节），交给在改 `transaction.rs` / `mount.rs` / `fault_injection.rs` / `history.rs` 的那几路。

## 十、没做什么

- 没走三方对抗。层 0、QEMU、herd7 和变异整表归 `crash-verifier`；名字含 layer0 的测试一条没跑，也没编译单跑（被闸拒了），只靠 `cargo build --all-targets` 编译通过。变异表第 875、879 行留给 59 号。全量 `cargo test` 没跑。没提交。
- 没写 kb（不变量原文见第六节）。
- 草稿：`/tmp/claude-1000/impl-rev-b1/` 下的日志、`proof/`（证红脚本、变异行与每条的运行日志）、`progress.md`，留给主 agent 核。仓副本 `/tmp/claude-1000/impl-rev-b1/copy`（1.9G）证红跑完就删了。

## 附：`git diff --stat -- crates litmus` 原样（含别的会话与此前各轮没提交的改动；我这一轮写的文件以第二节为准；新文件是未跟踪的，不在这张表里）
     crates/mutations.tsv                               |  547 +-
     crates/singlefs-checker/src/image.rs               |   77 +-
     crates/singlefs-checker/src/lib.rs                 |  106 +-
     crates/singlefs-checker/src/position_addressed.rs  |    2 +-
     crates/singlefs-checker/src/walk.rs                |  847 ++-
     crates/singlefs-core/src/admission.rs              |  272 +-
     crates/singlefs-core/src/allocation_record_tree.rs |    2 +-
     crates/singlefs-core/src/allocator.rs              |  132 +-
     crates/singlefs-core/src/code_two_tree.rs          |    4 +-
     crates/singlefs-core/src/extent_tree.rs            |    2 +-
     crates/singlefs-core/src/instance_table.rs         |   46 +-
     crates/singlefs-core/src/journal.rs                |   10 +-
     crates/singlefs-core/src/lib.rs                    |    2 +-
     crates/singlefs-core/src/make_filesystem.rs        |   11 +-
     crates/singlefs-core/src/mount.rs                  | 2182 ++++--
     crates/singlefs-core/src/mounted_read.rs           |    7 +-
     crates/singlefs-core/src/recovery.rs               |  413 +-
     crates/singlefs-core/src/rollback_witness.rs       |  314 -
     crates/singlefs-core/src/root_record.rs            |  124 +-
     crates/singlefs-core/src/system_configuration.rs   |  277 +-
     crates/singlefs-core/src/transaction.rs            |  433 +-
     crates/singlefs-core/src/write_request_split.rs    |    2 +-
     crates/singlefs-format/src/lib.rs                  |   56 +-
     .../src/bin/e142_first_transaction_write_dump.rs   |   73 +-
     ...e142_first_transaction_write_dump_one_device.rs |  121 +-
     .../src/bin/e156_allocation_basis_counts.rs        | 7368 ++++++++++++--------
     .../src/bin/e158_root_choice_repair.rs             | 3339 ++++++++-
     .../src/bin/first_transaction_on_device.rs         |  175 +-
     .../src/bin/first_transaction_region_bytes.rs      |    4 +-
     crates/singlefs-harness/src/crash.rs               | 1633 ++++-
     crates/singlefs-harness/src/crash_injection.rs     |  799 ++-
     crates/singlefs-harness/src/device_log.rs          |    2 +-
     crates/singlefs-harness/src/fault_injection.rs     |  749 +-
     .../src/first_transaction_regions.rs               |   92 +-
     crates/singlefs-harness/src/history.rs             | 1083 ++-
     crates/singlefs-harness/src/lib.rs                 |   44 +-
     crates/singlefs-harness/src/model.rs               |  961 ++-
     crates/singlefs-harness/src/model_comparison.rs    |  301 +-
     crates/singlefs-harness/src/on_device_modes.rs     |    4 +-
     crates/singlefs-harness/src/read_tally.rs          |    2 +-
     crates/singlefs-harness/src/segments.rs            |    2 +-
     .../tests/checker_known_bad_images.rs              |  805 ++-
     crates/singlefs-harness/tests/common/mod.rs        |  108 +-
     .../tests/common_tree_split/mod.rs                 |  138 +-
     .../tests/first_transaction_region_bytes.rs        |   78 +-
     .../tests/first_transaction_step_seven_layer0.rs   |  156 +-
     .../singlefs-harness/tests/instance_acquisition.rs |   89 +-
     .../tests/parallel_line_one_sequential_write.rs    |    4 +-
     ...ansaction_parallel_line_one_last_record_flag.rs |    8 +-
     ...ransaction_parallel_line_one_multi_unit_file.rs |    4 +-
     ...ansaction_parallel_line_one_sequential_write.rs |    4 +-
     ..._transaction_parallel_line_three_many_inodes.rs |    2 +-
     ...d_transaction_parallel_line_two_mounted_read.rs |   21 +-
     .../tests/second_transaction_step_five_reuse.rs    |  532 +-
     .../tests/second_transaction_step_four_rollback.rs | 2626 ++++---
     ...action_step_three_acquisition_barrier_layer0.rs |   29 +-
     ...second_transaction_step_three_formatted_pool.rs |  566 +-
     ...econd_transaction_step_three_second_instance.rs |    5 +-
     .../tests/second_transaction_step_zero_layer0.rs   |  563 +-
     ...ond_transaction_step_zero_test_only_switches.rs |    2 +-
     ..._transaction_supplement_three_bad_disk_input.rs |   24 +-
     ...transaction_supplement_three_crash_injection.rs |  196 +-
     ...transaction_supplement_three_fault_injection.rs |  412 +-
     ..._transaction_supplement_three_random_history.rs |  434 +-
     ...transaction_supplement_two_admission_formula.rs |  342 +-
     ...ent_two_c519_whole_device_loss_after_warm_up.rs |   29 +-
     ...two_c533_row_publish_record_without_its_root.rs |    7 +-
     ...ion_supplement_two_commit_generated_fallback.rs |   26 +-
     ...rop_and_devices_without_the_selected_version.rs |  258 +-
     ...nsaction_supplement_two_instance_table_chain.rs |  100 +-
     ...tion_supplement_two_instance_table_page_full.rs |   77 +-
     ...plement_two_instance_table_second_page_write.rs |    4 +-
     ..._two_multi_record_transaction_zero_publishes.rs |    5 +-
     ...action_supplement_two_presumed_clause_checks.rs |  105 +-
     ...plement_two_publish_failure_resent_unchanged.rs |    2 +-
     ...n_supplement_two_release_checksum_quarantine.rs |   88 +-
     ..._transaction_supplement_two_rollback_witness.rs | 1079 ---
     ...ction_supplement_two_rollback_witness_layer0.rs |  457 --
     ...n_supplement_two_root_ring_turn_in_one_mount.rs |    2 +-
     ...nt_two_row_publish_checks_before_acquisition.rs |    2 +-
     ..._two_tree_identifier_watermark_crash_orphans.rs |   72 +-
     ...ement_two_tree_nodes_and_the_central_mapping.rs |   10 +-
     ...second_transaction_supplement_two_tree_split.rs |  147 +-
     ...upplement_two_unreadable_abandoned_root_slot.rs |  314 +-
     .../system_configuration_mutability_classes.rs     |   35 +-
     85 files changed, 22008 insertions(+), 10579 deletions(-)
