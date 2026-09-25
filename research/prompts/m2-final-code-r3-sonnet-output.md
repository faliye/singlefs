# m2-final-code-r3 云端正推（Sonnet）：Z14、Z15、Z18

腿：云端正推。格：Z14（extent 树两段 K2）、Z15（挂载怎么读 K4 与 C483 ②）、Z18（第二轮之后的定义改动）。
逐格判「代码/定义做的是不是条款说的」，落一致 / 冲突 / 规则没说。冻结副本：`/tmp/claude-1000/m2-final-code-r3/tree/crates/`；
kb 快照：`/tmp/claude-1000/m2-final-code-r3/kb-snapshot/`；定义快照：`/tmp/claude-1000/m2-final-code-r3/defs/`。
测试跑在拷贝 `/tmp/claude-1000/m2-final-code-r3-sonnet/`（`CARGO_TARGET_DIR` 单独指到那里），未改动冻结副本任何文件。

## Z14　extent 树两段（K2）

**条款**（kb 快照 `.claude/kb/decisions/08-核心索引结构.md` 第 386 行，已定项 14「两棵派生树的结构」第二条，整行抄）：

> - **extent 树**：两段，都按位置寻址。上段按 inode 号的位置寻址；下段一个文件一棵，按数据单元号的位置寻址（叶罩 144 个单元，内部扇出 147），洞就是缺席。**只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针；条目带一个标签字节区分「没有单元 / 下段根指针 / 内联数据指针」（用户 K2）。

### 问 1：单单元内联 ↔ 两个及以上建下段，来回切换时旧节点是不是都释放了

**判定：一致。**

- 代码：`transaction.rs:3479-3561` 的 `plan_the_extent_tree_after_this_publish`——`replaced_previous_roles`（第 3541-3555 行）把上一版 `lower_nodes` 全部记为要释放的角色（第 3542-3545 行 `.chain(...)`），不论这一版是否还有下段；这一版的 `lower_nodes`（第 3536-3539 行）由 `lower_segment_nodes_of_a_file_without_holes(data_units)` 现算，`data_units <= 1` 时交回空 `Vec`（`extent_tree.rs:210-212`）。
- 复跑证据（真实命令与原样输出）：

```
$ cd /tmp/claude-1000/m2-final-code-r3-sonnet/tree && CARGO_TARGET_DIR=/tmp/claude-1000/m2-final-code-r3-sonnet/target bash /home/fy5090/code/singlefs/research/scripts/capped.sh 8 cargo test -p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- --nocapture
running 8 tests
test the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline ... ok
test shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has ... ok
（其余 6 条同批 ok）
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 41.69s
```

  `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`
  （`second_transaction_parallel_line_one_multi_unit_file.rs:272-340`）四步顺序写 145→144→1→3 个单元，
  第 322-330 行逐个断言「上一版下段的每个节点这次发布都在 `output.released` 里」，四步全部通过。
- **推翻条件**：把 `plan_the_extent_tree_after_this_publish` 的 `replaced_previous_roles` 改成只在 `data_units_of_the_file_written_this_publish` 仍 `>1` 时才收集上一版下段角色，这条测试会在「145→144」之后紧跟「144→1」那一步的释放断言上失败——目前没出现这个失败，一致成立。

### 问 2：下段长到两层再缩回一层，层级与区间对不对

**判定：一致。**

- 代码几何：`extent_tree.rs:180-192` 的 `key_range`——节点头 key 区间 `[(0, inode, 首单元×P), (0, inode, (末单元+1)×P−1)]`，与
  `08-核心索引结构.md:385` 的位置寻址覆盖区间口径一致（该条款把「按位置寻址的树节点头区间＝按位置规定罩的那一段」定为与
  「子树覆盖区间」同一段，见 `18-块里携带什么信息.md:52` 整行：「按位置寻址的树（分配记录树、extent 树，D8（核心索引结构） 已定项 14），节点头的 key 区间写这个节点按位置规定罩的那一段，与「子树覆盖区间」在这类树上是同一段；checker 按位置独立算出每个节点该罩的那一段逐节点核。」
- 复跑证据：同一条测试第 313-321 行断言树高从根节点头现读出来的
  `extent_tree_lower_segment_of_the_file` 在 145 个单元时是 `height_and_leaves.0 - 1 = 2`（两层：叶 + 根），
  144 个单元时是 `1`（根兼叶），1 个单元时是 `0`（无下段），3 个单元时又变回 `1`；四步全部通过（上面命令的原样输出）。
- **推翻条件**：把 `lower_root_level_for` 改成不随 `data_units` 缩小而降层（例如钉住历史最高层级），
  这条测试会在「145→144」那一步断言树高仍是 2 而不是 1 时失败——目前没出现，一致成立。

### 问 3：下段 key 的字节偏移 = 单元号 × 32634，与数据单元净荷容量的口径是不是同一个

**判定：一致。**

- `extent_tree.rs:67-69` 的 `payload_capacity_in_bytes()` 直接调用 `crate::unit::data_unit_payload_capacity()`
  （`unit.rs:217-221`：`32768 − 105 − 29 = 32634`），与 `write_request_split.rs:114`、`mounted_read.rs:137`
  引的是**同一个函数**，不是各自另算一份 32634。
- **推翻条件**：`extent_tree.rs` 里另起一个硬编码 `32634u64` 常量而不经 `data_unit_payload_capacity()`，
  一旦 `DATA_UNIT_HEADER_BYTES` 或预留位改了两处会分叉——现状是同一处，没有分叉。

## Z15　挂载怎么读（K4）与 C483 ②

**条款**：
- `08-核心索引结构.md:387`（已定项 14「挂载怎么读」，整行抄）：
  > - **挂载怎么读**：分配记录树挂载时整棵读进挂载态（分配器本来就要）；extent 树按需，打开文件时按位置走下去（用户 K4）。
- `19-块指针的结构与宽度预算.md:182`（已定项 8，节选一句整句抄）：
  > 豁免三类之外的单元（码 1 数据单元与码 2 树节点——extent 树根、inode 树根、inode 叶容器）位置提示读不出时一律经映射回退；豁免三类（映射树根、树表、根记录那条链）不回退。

### 分配记录树整棵读进挂载态

**判定：一致。**`recovery.rs:1403-1419`（`rebuild_version` 一类的写路径重放函数）调用 `read_allocation_record_tree`，
每个节点用 `read_mapped_tree_node_via_hint_then_central_mapping` 做回退（`recovery.rs:1410-1417`），
注释 `recovery.rs:1402` 整行：「分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。」

### extent 树按需读、打开文件时按位置走下去

**判定：一致。**`mounted_read.rs:348-424` 的 `open_pool_for_read` 只记下 extent 树的树表条目与根指针
（第 422-424 行注释整行：「extent 树打开池时不读（D8（核心索引结构） 已定项 14「挂载怎么读」：按需，打开文件时按位置走下去，用户 K4）：这里只记下树表条目里它的号与根指针。」），
真正的读发生在 `open_file`（`mounted_read.rs:613-736`）：`find_upper_leaf_entry` 走上段、标签 1 时 `read_lower_segment` 走下段（第 644-676 行）。

### 问 1：打开文件时读几个节点的自报数，与块层数到的是不是同一个口径

**判定：一致（无多跳时逐字节相等，有依据；判别力自证在测试断言里）。**

- 条款：`mounted_read.rs:11-12`（doc 注释整行）：「D17（实现分层与第三方管道） 已定项 5：第一版是纯用户态库、只对块设备抽象编程；那条射程 ① 要实现给出「一次 API 调用发了几个设备级操作」的读数交装置比对 ⇒ `ReadPathObservation::device_reads_issued`。」——`open_file` 的 `node_reads` 字段是同一条纪律在树节点这一侧的体现。
- 复跑证据：

```
$ cd /tmp/claude-1000/m2-final-code-r3-sonnet/tree && CARGO_TARGET_DIR=/tmp/claude-1000/m2-final-code-r3-sonnet/target bash /home/fy5090/code/singlefs/research/scripts/capped.sh 8 cargo test -p singlefs-harness --test second_transaction_parallel_line_two_mounted_read
running 12 tests
test a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries ... ok
test a_file_whose_extent_record_count_does_not_match_its_size_is_refused_instead_of_guessing ... ok
test a_read_that_runs_past_the_end_of_the_file_is_refused_before_any_unit_is_dereferenced ... ok
test a_location_hint_pointing_at_another_slot_still_reads_through_the_central_mapping_and_counts_one_extra_hop ... ok
test a_central_mapping_root_whose_entry_width_is_narrower_than_the_field_table_is_refused_instead_of_slicing_past_the_entry ... ok
test a_data_unit_resealed_with_only_its_own_checksums_is_caught_by_the_location_entry_checksum ... ok
test a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state ... ok
test corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone ... ok
test a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset ... ok
test every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units ... ok
test random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring ... ok
test the_same_image_read_cold_and_read_through_the_mount_state_gives_the_same_bytes ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
```

- `second_transaction_parallel_line_two_mounted_read.rs:701-708`：打开一个单单元文件，断言 `(counting.tally().reads, file.extent_tree_reads_at_open().node_reads) == (1, 1)`，
  消息整行：「打开文件读 extent 树上段根兼叶那一个节点，块层数到的与实现自报的相等」。
- 同文件第 762-769 行：四单元文件（有下段）断言 `(2, 2)`，消息整行：「打开文件按需读 extent 树：上段根兼叶一个、下段叶一个」。
- 第 740-743 行另一处对 `read_at`（不是 `open_file`）断言 `while_reading.reads == output.observation.device_reads_issued`，消息整行引了 D17 已定项 5 射程 ①。
- **这个「一致」只在没有位置提示过期（无多跳）的路径上被测试覆盖**——`node_reads` 本质是「访问过几个逻辑树节点」的自报数（`open_file` 里 `read_node` 闭包每被调一次 `node_reads += 1`，`mounted_read.rs:633-643`），而单个逻辑节点在提示过期时经 `read_mapped_tree_node_via_hint_then_central_mapping` 回退可能发生 2-3 次实际设备读（`recovery.rs:357-371`：先按两条位置提示各试一次，`recovery.rs:292-303` 的 `read_unit_via_locations` 内部对 `locations` 数组逐条 `reader.read`，再查映射读一次）。**没有找到一条测试在「有多跳」的场景下同时用 `ReadCountingPoolReader` 数块层读、又读 `node_reads`、断言二者相等**——多跳次数由另一个独立字段 `tree_node_stale_location_hint_hops_at_open` 单独报（`mounted_read.rs:586-590`），不折进 `node_reads`，因此「node_reads 与块层数一致」这条断言目前只在零多跳的取样点上被验证过，多跳取样点上没有对应的块层数验证。
- **推翻条件**：造一份提示过期的多单元（有下段）镜像，用 `ReadCountingPoolReader` 打开文件，若 `counting.tally().reads` 与 `node_reads + stale_hops` 或与 `node_reads` 本身对不上却没有被任何断言捕捉，就坐实「自报数与块层数在多跳路径上口径不同、且没有测试盯着」；目前没有这样的用例，因此这一格只报「一致（零多跳取样点上）+ 多跳取样点未被验证」，不报「冲突」。

### 问 2：位置提示过期、映射回退那一路，在下段多层时走不走得通

**判定：机制上一致，测试覆盖上是缺口（不报「冲突」，因为没有一次跑出与条款矛盾的行为）。**

- 机制：`mounted_read.rs:634-643` 定义的 `read_node` 闭包同时喂给 `find_upper_leaf_entry`（上段）与 `read_lower_segment`（下段，
  `mounted_read.rs:653-661`）；`extent_tree.rs:748-797` 的 `read_lower_node_and_its_subtree` 对下段**每一层**（根、内部、叶）
  都调同一个 `read_node` 回调（第 758-765 行 `reading.read_and_judge_node(pointer, ..., read_node, ...)`），
  没有按层级另开一条不经映射回退的路径——即回退机制按代码结构对任意深度的下段节点一视同仁。
- 覆盖缺口：`second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs:367-439` 的
  `the_relocated_node_reads_back_through_the_central_mapping` 是仅有的「树节点提示过期经映射回退」用例组，
  它的 `MappedUnitToRelocate` 枚举（第 138 行起）只有 `ExtentTreeRoot`／`InodeTreeRoot`／`InodeLeafContainer`／
  `AllocationTreeRoot`／`AccountingTreeRoot`／`DataUnit` 六种，且都建在 `first_transaction_image`（单单元、无下段）上；
  **没有一个变体是「145 个单元、下段两层」镜像里某个下段内部节点或叶的位置提示过期**。C483 ②（`checks-owed.md:423`）
  记的正是「树节点没有经映射的回退」这件事没有会红用例，判别力自证那句（整段抄）：
  > ② 条款 2026-09-23 已定（D19（块指针的结构与宽度预算） 已定项 8 补了动作那一半），还欠实现与那条会红的用例（造一份提示指空槽、映射里是真落点的镜像，断言 extent 树根也能多跳读回、整池挂得上；判别力自证：把回退去掉必须变回 `UnitUnreadable`）。
  这句话本身只提「extent 树根」，也没有点名「下段多层」这一档，与代码今天实现的覆盖面一致（两边都停在「根」）。
- **推翻条件**：造一份 145 个单元、下段长成两层的镜像，把下段内部节点（非根、非叶）的位置提示改成指空槽、
  映射里放真落点，若 `open_file` 报 `UnitUnreadable` 或整池挂不上而不是多跳读回，就是代码没有兑现已定项 8
  在这一档上的要求（冲突）；目前没有这条用例，因此判定停在「机制一致、覆盖缺口」，不报冲突也不报「已验证」。

## Z18　第二轮之后的定义改动

材料：`/tmp/claude-1000/m2-final-code-r3/defs-r2-to-r3.diff`（44 行，改 4 份文件：`agent-common.md`、`main-agent.md`、
`crash-verifier.md`、`implementation-workflow.md`）与冻结副本 `defs/.claude/hooks/bash-command-detector.sh`（793 行）。
diff 全文已在派发提示的背景材料之外单独复核过，下面按文件逐条判。

### 问 1：改过的定义之间、定义与 hook 之间，有没有说反话

**判定：agent-common.md 与 main-agent.md 的新增部分之间一致；main-agent.md 自身有一处遗漏（既非这一轮引入、也未被这一轮补上），判「冲突」偏严格，更准确是「不完整，够得着说反话的门槛但主 agent 需要定」。**

- hook `bash-command-detector.sh:31-70` 自己列的「拒绝四种」（逐条整行抄编号与要点）：
  - 第 33 行起 ①：命令位置上起看门狗（`watch.sh` / `agent-watch.py watch`）而 `run_in_background` 不是 true、或整条命令里有单独的 `&`／`nohup`／`disown`／`setsid`／把输出重定向到 `/dev/null`。
  - 第 42 行起 ②：前台没超时的等待循环。
  - 第 53 行起 ③：把活放出追踪的写法（`disown`／`coproc`／`setsid -f`／`nohup … &`／`tmux … -d`／`screen -dm`／不带 `--wait` 的 `systemd-run`）。
  - 第 61 行起 ④：`run_in_background` 里以单独 `&` 收尾、之后同一层没有 `wait` 的作业。
- `agent-common.md:57`（这一轮改的那一句，整句抄）：
  > 项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对四种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。
  四项顺序是②③④①，内容与 hook 的四类逐条对得上，**一致**，没有遗漏、没有多算。
- `main-agent.md:29`（这一轮改的那一句，整句抄）：
  > 盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。
  这一句把 `bash-command-detector.sh` 的②③④与另外三个不同的 hook（重型测试守卫、撇号角标门禁、写范围/覆盖守卫）混编成一张清单，**唯独没有列 ①（起看门狗的错误写法）**。
  而同一句紧接着就是主 agent 自己要执行的动作——「立刻用 `run_in_background` 起看门狗 `bash research/scripts/watch.sh …`（……调用里不再加 `&`、`nohup`、`disown`）」——这正是 hook ① 专门盯的对象（hook 注释 `bash-command-detector.sh:35` 整行：「2026-09-24 主 agent 同一个会话里第三次这样起，`main-agent.md`「调用里不再加 `&`、`nohup`、`disown`」靠自觉守不住」，点名的就是 main-agent.md 这一处）。
  用 `grep -n "错误写法" defs/.claude/main-agent.md` 核过，全文只有第 29 行这一处提到"拒绝一种危险写法"的清单，别处没有补记 ①。
- 这处遗漏**不是这一轮 diff 引入的**：`m2-final-code-r2` 那份 `main-agent.md` 同一句里就已经没有 ①（r2→r3 只在原有列表里插入了「run_in_background 里后面没有 `wait` 的单独 `&`」这一项，即 ④），所以严格说这一轮的改动本身没有制造新的矛盾，但**这一轮本该顺手把 ① 补进去而没有补**：agent-common.md 在这一轮把「四种」写全了、main-agent.md 在这一轮碰了同一张清单却仍然只有三种（外加另外三个 hook 的三项），两处现在互相对不上——一处说「hook 拒绝的是这四种」，另一处对着几乎同一件事列出的清单里少一种，且少的正好是这一份文件自己最相关的那一种。
- **推翻条件**：若 main-agent.md 其他地方（未改动部分）已经单独交代过「起看门狗的错误写法会被拒绝」，这处遗漏就不成立——已用 `grep -n "错误写法\|拒绝" defs/.claude/main-agent.md` 核过全文，没有第二处，推翻条件不成立。

### 问 2：有没有一句写的是为什么、不是怎么做

**判定：这一轮的改动是在清理这类尾巴，不是新增。**

`main-agent.md` 的第二处改动（`defs-r2-to-r3.diff` 第三个 hunk）：

```
-改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做——它们沿用派发那一刻的定义，看不到后来的改动。
+改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
```

删掉的「——它们沿用派发那一刻的定义，看不到后来的改动」正是一句解释「为什么要发消息」的尾巴，删除后只剩「怎么做」，与 `agent-common.md` 开头「「为什么」……这里不写、也不留解释的尾巴」的纪律同向。**这一轮的四处改动里没有新增反例**：另外三处（`agent-common.md` 加四种写法枚举、`main-agent.md` 加④、`crash-verifier.md`／`implementation-workflow.md` 的标题改名）都是纯描述，没有夹带解释句。
- **推翻条件**：若这次 diff 别处出现「……——因为……」这类新增的因果尾巴，就要另记一条；`defs-r2-to-r3.diff` 全文 44 行已逐行读过，没有第二处这样的新增。

### 问 3：检测器第四种拒绝在 `agent-common.md`、`main-agent.md` 里的说法，与 hook 的判据是不是同一条

**判定：一致。**

hook `bash-command-detector.sh:61-70`（第④类，节选判据句整行）：
> ④ run_in_background 为 true 的命令里，有一个作业以单独的 `&` 收尾（不是 `&&`、`>&`、`&>`、`|&`），之后同一层、同一对圆括号里再没有 `wait`（不带参数的 `wait`，或带进程号的 `wait "$pid"`；只带选项的 `wait -n` 不算）：外层 shell 起完它就退出，完成通知当场发出，真跑完的那个进程叫不醒任何人。

`agent-common.md:57` 对应那句：「run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业」——「同一条命令里」与 hook 的「同一层、同一对圆括号里」用词不同，但 hook 自己也说明了「同一层」怎么划（`bash -c '…'`、heredoc 正文、命令替换各是一层，圆括号是子 shell），这是对同一个判据的简化转述，没有放宽或收紧判据本身（不是「同一个 Bash 工具调用里」这种会漏掉「圆括号子 shell里的 &、wait 在外面」也算拒绝的弱化说法）。`main-agent.md:29` 对应那句「run_in_background 里后面没有 `wait` 的单独 `&`」，同样只是更简的转述，指向同一条判据。
- **推翻条件**：若 hook 的判据后续加了「同一条 Bash 工具调用」以外的层次概念而两份定义没跟上（例如 hook 改成按「同一个子 agent 的多次调用」判），转述就会不同于判据本身；今天两者仍是同一个「同一层」概念的两种说法，一致成立。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z14 问 1（内联↔下段切换释放） | 一致 | `plan_the_extent_tree_after_this_publish` 无条件把上一版全部下段节点记进 `replaced_previous_roles`，测试四步顺序写全部通过 |
| Z14 问 2（两层缩回一层） | 一致 | `lower_root_level_for` 纯按当前 `data_units` 现算、不设历史最高层地板，145→144→1→3 四步树高与形状测试全部通过 |
| Z14 问 3（32634 同一口径） | 一致 | `extent_tree.rs` 与 `write_request_split.rs`／`mounted_read.rs` 共用同一个 `data_unit_payload_capacity()`，没有第二处硬编码 |
| Z15（分配记录树整棵读） | 一致 | `recovery.rs` 的写路径重放函数整棵读、逐节点走「提示 → 映射回退」同一条口子 |
| Z15（extent 树按需读） | 一致 | `open_pool_for_read` 只记根指针，真正的读发生在 `open_file`，注释直接引 D8 已定项 14 |
| Z15 问 1（node_reads 自报数=块层数） | 一致（零多跳取样点），多跳取样点未被测试覆盖 | 两条测试断言 `(块层读数, node_reads)` 相等且通过；没有测试在「有多跳」的取样点上同时核这两个数 |
| Z15 问 2（多跳回退在下段多层时走不走得通） | 机制一致，测试是缺口 | `read_node` 闭包对上段/下段任意层一视同仁，但唯一的多跳回退用例组只建在单单元（无下段）镜像上，C483 ② 自己也只点名到「extent 树根」 |
| Z18 问 1（说反话） | main-agent.md 有一处遗漏（非本轮引入，本轮未补） | agent-common.md 的「四种」写全了，main-agent.md 同一句仍只列三种＋另外三个 hook 的项，唯独没有「起看门狗的错误写法」，而这正是它自己下一句要求主 agent 做的动作 |
| Z18 问 2（为什么尾巴） | 一致（本轮在清理，不是新增） | main-agent.md 删掉「——它们沿用派发那一刻的定义，看不到后来的改动」这句解释尾巴，别处没有新增因果句 |
| Z18 问 3（第四种拒绝的转述与判据） | 一致 | 两份定义的简化说法与 hook 的「同一层」判据指向同一条规则，没有放宽或收紧 |

## 没做什么

- 不判 Z13、Z16、Z17（分给云端攻方 Opus）与本地攻方那一格算术，也不采纳或出判决——判决由主 agent 做。
- Z15 问 1、问 2 与 Z18 问 1 的「测试覆盖缺口」「main-agent.md 遗漏」都只报现象与推翻条件，没有去补测试、没有去改定义——写范围只有报告文件、模型目录（这一轮没建模型目录，判定不需要独立模型）与草稿目录，改代码或定义不归这条腿。
- 没有跑 `--selftest`（`agent-write-scope.sh` 之外的门禁）、没有跑全量 `cargo test`／`gate.sh`：只跑了 Z14、Z15 涉及的两个测试二进制（`second_transaction_parallel_line_one_multi_unit_file`、`second_transaction_parallel_line_two_mounted_read`），按共用约束「子 agent 一律不跑重型测试」与线程上限 8。
- `crash-verifier.md`、`implementation-workflow.md` 的标题改动核对了引用方（crash-verifier.md 的段落）与被引方（implementation-workflow.md 的 `## 重型测试只在提交时跑` 小节）逐字匹配，没有去跑 `.claude/gate.d/` 里检测定义引用是否存在的阶段（不在这条腿分给的阶段表里，登记见 `stage-owners.tsv`，这一轮没有现查）。
- 没有对 `defs/.claude/hooks/bash-command-detector.sh` 跑 `--selftest`：派发提示说「检测器的 `--selftest` 在冻结副本 `defs/.claude/hooks/` 那一份上跑」，但 Z18 的三问都是「定义文字与 hook 注释、代码判据是否说法一致」，不涉及 hook 本身的检出/拒绝逻辑要不要重新验证（hook 逻辑本身不在这一轮 diff 改动范围内，`bash-command-detector.sh` 不在 `defs-r2-to-r3.diff` 里），因此没有跑；如果主 agent 需要坐实 hook today 的行为与注释一致，需要另外要求跑 `--selftest`。
