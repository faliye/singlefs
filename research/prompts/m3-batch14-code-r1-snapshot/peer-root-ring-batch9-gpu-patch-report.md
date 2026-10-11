# 第 9 批 9-3 GPU 簇：实现员报告（2026-10-04）

交补丁。补丁目录 `/tmp/claude-1000/m3-r1/batch9-gpu/patch/`：`crates.patch`（6 个文件）、`mutations-append.tsv`（12 行）、`mutations-replacements.tsv`（1 行）、本报告。
补丁对主工作区 `git apply --check` 退 0（打补丁前一刻现查；主工作区里对拍测试文件与扫描内核已被 19-1 改过，补丁照样打得上）。

## 一、结论一览

| 条 | 做了什么 | 验到哪一步 |
|---|---|---|
| 1 I-3.5 | 删「内核还没判」那张表与断言里认它的一支，覆盖表回到全部 68 条、编译期断言「第 k 项是 k」；走读内核实现出现规则（TF、β、gap、gap-row、N3、整对不判、整条不适用），连带把 CPU 第 8 批的两处走读修补里 GPU 缺的那一处（树表别的版本先走过时映射根照样往下走）补上 | naga 解析与校验三个内核都过；`cargo build --all-targets --all-features` 过；对拍两条流（红、成立）写了，没跑 |
| 2 RS5 | `parse_record` 判新根段卸载记号位 0 之外为 0、判实例表指针与分配记录树根两条指针的头，四条指针的头都加判 extent 偏移；施加末条时卸载记号、实例表指针、分配记录树根从记录取 | 同上；对拍三条流写了，没跑 |
| 3 RS6 | `w_walk_one` 的 WALK_KIND_VERSION 四条指针都从记录新根段取，删掉「它施加在其上的那条根」（`applied_on`）的找法与参数 | 同上；对拍一条流写了，没跑 |
| 4 | I-8.10 后三段与 RR1 各一条变异行；I-8.10 第七段补一条流 | 变异行写了，没跑 |

全部新测试都在 checker 档包 `singlefs-checker-tier` 里（对拍测试文件），按定义不跑、证红也不跑，12 条变异行全部留给提交时的 checker-tier-crates-mutation-replay。我能做的只到：每条变异施加之后的内核仍过 naga 解析与校验（12 条全过，见第五节），变异的原文在补丁打上之后的主工作区里恰好命中一次（第五节）。

**什么现象会推翻这份结论**：主 agent 跑对拍（第七节命令）时，① 新加的 7 条用例里任一条红在「CPU 要红某条」那句断言上（说明我对那条流上 CPU 判什么推错了）；② 原有的 `the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows` 或别的对拍用例在 I-3.5 那一位上报 GPU 与 CPU 不同（说明出现规则的移植与 CPU 不同口径）；③ 有状态判不了、站点号是 40–46（新加的四张表容量不够）；④ checker-tier-crates-mutation-replay 上这 12 行里有没红的。

## 二、写过的文件

补丁里（`git apply --stat crates.patch` 原样）：

```
 .../src/crash_judge_checker.wgsl                   |  564 ++++++++++++++++++--
 .../src/crash_judge_checker_common.wgsl            |   46 ++
 .../src/crash_judge_common.wgsl                    |   30 +
 .../src/crash_judge_dispatch.rs                    |   11 
 .../singlefs-checker-tier/src/crash_judge_gpu.rs   |   72 ---
 ...gpu_agrees_with_the_cpu_judge_on_every_state.rs |  307 +++++++++++
 6 files changed, 910 insertions(+), 120 deletions(-)
```

- `crates/singlefs-checker-tier/src/crash_judge_gpu.rs`：覆盖表与编译期断言；`SCRATCH_WORDS` 255 504 → 290 064；模块文档。
- `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`：`INVARIANT_I_3_5 = 65u`；四张新表的草稿区布局（落点表、版本表、每一版的落点、走过的树表）；`SCRATCH_END` 290 064。
- `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`：I-3.5 的记录（`w_start` / `w_begin_version` / `w_note_pointer_references` / 实例表一片的诞生代号 / 树表那一段）与判法（`judge_the_appearance_rule_of_adjacent_candidate_versions`）；`w_walk_tree_table_and_mapping` 共享树表那一支；`w_walk_one` 与 `walk_and_hand_off` 去掉 `applied_on`。
- `crates/singlefs-checker-tier/src/crash_judge_common.wgsl`：新根段后三段的段内偏移常量、`record_pointer_head_is_the_first_version`、`parse_record` 两处判、`replay_journal` 拼新根取七段。
- `crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`：两处单测跟着改（见第四节 S6）。
- `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`：比较结果多收一样「CPU 判过而成立的不变量」；6 个造流函数、7 条用例；文件头与种假位那条用例的注释。
- `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`、`crash_judge_tables.rs`：在派发清单里，没改。

变异表：`mutations-append.tsv` 追加 12 行（名字见第五节表）；`mutations-replacements.tsv` 换 1 行「第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表」（原文 `255_504` 改成 `290_064`，替换文照旧 `247_312`；不换它的原文在补丁之后命中 0 次）。

## 三、做法与依据

### 第 1 条 I-3.5（`crash_judge_checker.wgsl` 补丁后的行号）

逐要素对 CPU `crates/singlefs-checker/src/walk.rs`：

| CPU | GPU |
|---|---|
| `unit_of_each_placement` / `user_visible_tree_of_each_placement`（`note_references_of_a_pointer_followed_by_the_walk`，walk.rs:1072，两张表都是 `or_insert`） | 落点表 `C_APPEARANCE_PLACEMENTS`，`appearance_note_placement`（832），只在 `w_note_pointer_references` 里、上下文 0 记 |
| `user_visible_tree_being_walked`：树表条目宽 ≥ 200、种类在 extent / inode / `deleted_inodes`、根指针非全零时打标签 | `appearance_begin_tree_table_entry` 在树表每一条之前设，之后清 |
| `user_visible_tree_roots_of_this_version`、共享树表照先走那一版记下的取（walk.rs:1985） | 版本表那一行的根 + 走过的树表那张表 |
| `birth_txg_of_walked_units` 与 `birth_txg_of_walked_instance_table_pages`（walk.rs:2575） | 沿用 `C_VISITED_BIRTHS0`：被引用单元标记 1（原样），实例表一片标记 2；I-4.9 只认 1，I-3.5 两种都认 |
| `note_the_version_just_walked_for_the_appearance_rule`：最新根、候选根、由记录施加出来的版本，各记引用的单元、实例、txg、走读这一版有没有记失败 | `appearance_note_the_version_just_walked`（968），`walk_and_hand_off` 唯一调用点之后、这三个阶段记（3770）；落点存成落点表下标 |
| `judge_the_appearance_rule_of_adjacent_candidate_versions`（walk.rs:509）：按 txg 升序同 txg 并组、相邻两组逐对、R 侧逐条判；整对不判；(a) / (b)；N3 | 同名函数（3066），版本走完、转被抛弃根之前调一次（3535）。R⁻ 组 = txg 比 R 小的里最大的那一组。BTreeMap 折落点成单元时同一单元后收的标签盖先收的：GPU 每个单元留 (盘, 槽) 最大的那个落点、标签取它的 |
| `hole_in_the_candidate_set_between`（walk.rs:459） | `appearance_hole_between`（2992）：同一实例多行只算一个、`v − i − 1` 比较、每一行都 T = W = 0 |
| 一个单元都没判到时整条报不适用 | 违例位上不适用与成立都是 0，不另记 |

CPU 第 8 批报告第三节「两处走读修补」的第 2 处（树表别的版本先走过时映射根照样往下走，walk.rs:1413）GPU 原来没有：`w_walk_tree_table_and_mapping`（2818）树表读不出时直接返回。不补的话零单元发布、卸载与抬 F 那几版在 GPU 上少了映射树的单元，下一版里映射树的旧单元会被当成新出现、误红 I-3.5（推的）。补法照 CPU：先判「这片树表别的版本走过没有」，读得出照旧走条目，走过的照先走那一版记下的根取、映射根照样往下走，别的情况照旧返回。第 1 处（墓碑树、谱系树的容器记在根节点下面）GPU 不涉及：GPU 走读今天不走种类 9、10、11（见第四节 S2）。

草稿区：新加四张表 34 560 个字，每个状态 255 504 → 290 064 个字（每个状态约 1.11 MiB）。NVIDIA 绑定上限下一条缓冲装的状态从 2101 降到 1850，同样显存一次派活带的状态少约 12%（推的，没量吞吐）。表满了一律置「判不了」交 CPU，站点号 40–46。

### 第 2 条 RS5（`crash_judge_common.wgsl`）

- `parse_record`：新根段四条指针各过 `record_pointer_head_is_the_first_version`（961）：原来的 `head_ok`（MAC / nonce、算法类型、压缩码、压缩长度）加 extent 偏移为 0，照 core `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from` 的 `judged()`（`crates/singlefs-core/src/pointer.rs` 第 210 行起，五样都判）。原来只判树表与映射根两条、也不判 extent 偏移。
- `parse_record`：卸载记号位 0 之外有位为 1 当记录损坏（998），照 `crates/singlefs-core/src/root_record.rs:43` 与 `journal.rs:346`。
- `replay_journal` 拼新根：卸载记号（1304）、实例表指针（1310）、分配记录树根（1312）从记录新根段取，照 `crates/singlefs-core/src/recovery.rs:3601`、`:3607`、`:3609`；fsid 照所选根（`RootFields` 里没有 fsid，不用动）。
- 恢复内核里 `RootFields.unmount` 与 `.allocation_root` 今天没有读者（走到文件只读实例表与树表）：这两处取法对结论没有影响，改它们的变异是等价变异，没写变异行。

### 第 3 条 RS6（`crash_judge_checker.wgsl`）

`w_walk_one`（2864）：WALK_KIND_VERSION 的树表、映射根、实例表、分配记录树根四条指针都从记录新根段取（与 walk.rs:1348 `walk_version_applied_only_by_records` 同一取法），去掉 `applied_on_base` 参数；`walk_and_hand_off` 里由记录施加出来的版本那一阶段找「同实例、txg 比它小的最大那条根」的循环与重放那一阶段的 `walk_applied_on = newest_root.base` 一并删掉（再没有读者）。I-4.1 那一遍（上下文 1）同样从记录取，与 CPU `judge_trees_of_the_replayed_version_are_traversable` 同。

### 第 4 条

I-8.10 第五、六、七段与 RR1（`checker_record_valid`，294 起那个函数里卸载记号那一判）各一行变异，分别点名第五节表里的流。第七段原来没有一条会让它单独红的流，补了「记录卸载记号 1、根 flags 0」那一条。

## 四、停下交主 agent、写进报告的事

- **S1 清单外两处说法跟着变旧**（文件不在我的清单里，没改）：`crates/singlefs-checker/src/image.rs:72`「第 66 条（下标 65）是 I-3.5（里程碑三第 8 批）：GPU 判器内核还没判它」；`.claude/kb/invariants.md:16`「下标 65 的 I-3.5（引用区间的精确性） GPU 内核还没判、那一位恒 0，点名在 `POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET`……」（kb 归 kb-scribe）。前者改的是 `crates/singlefs-checker/src/` 的注释，打补丁闸会要「受影响的层 0 流」一节，只改注释时那一节写「无」。
- **S2 GPU 走读不走种类 9、10、11**（`w_walk_tree_table_entry` 只认种类 1–5，「别的种类不走」）：CPU 走 `deleted_inodes`（9，用户可见树）、墓碑树（10）、谱系树（11）。有这三种树的镜像上 GPU 的 I-5.1、I-3.1、I-3.5 等都会与 CPU 不同；今天的对拍流里没有这三种树（推的，没逐条流查）。不在这次条款里，没补。
- **S3 共享树表那一支 GPU 不判 I-9.25**：CPU 照先走那一版记下的条目、拿这一版自己的 txg 再判一遍（walk.rs:1985，主 agent 2026-10-03 定改回 kb 字面）；GPU 只在第一个走到那片树表的版本判。差别只在「inode 树那条的 previous_snapshot_txg 大于共享它的较早一版的 txg」这种坏镜像上出现（推的）。我这次给共享树表补的那张表只存 I-3.5 读的根，没扩成 I-9.25 要的条目，没补。
- **S4 记录新根段指针头的 extent 偏移**：第 2 条只点名卸载记号与施加取七段；`parse_record` 原来对树表、映射根两条也不判 extent 偏移、对后两条什么头都不判（第 8 批报告 S7 同一处）。我照 core 把四条都补成判全，算在「与 core 同口径」里；嫌越界就整段退回原样（两行变异「不判实例表指针的头」「不看 extent 偏移」与那条流一起删）。
- **S5 第 3 条的变异写法**：派发要「改回取那条根」，但改完之后内核里已经没有「它施加在其上的那条根」（找法与参数都删了，CPU 那一侧也没有这个概念）。两行变异写成「版本那一支不走记录里的分配记录树根 / 实例表」，对那几条流的效果与改回取那条根相同（那条根的分配记录树根是全零、实例表读得出）。
- **S6 `crash_judge_dispatch.rs` 单测里钉的状态数本来就过时了**：主工作区今天 `SCRATCH_WORDS` 是 255 504，而 `states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit` 钉的 2170 与 13 024 是 247 312 那一版的数（现算：255 504 时应为 2101 与 12 607）。我按 290 064 改成 1850 与 11 105（`python3` 现算）。`the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused` 里找唯一调用点的那一串跟着去掉 `walk_applied_on`。
- **S7 变异行没写的改法**：共享树表时映射根照样往下走那一支，我想不出一条手上这几条流一定会让它红的（推的），没写变异行；I-3.5 的缺口（gap-row、N3）今天的对拍流里没有 txg 不相接的相邻两版（回退流 (1,4)→(2,5) 是相接的），没有流，也没写变异行。要补得造一条带洞的回退候选集的流。
- **S8 `RootFields.unmount`、`.allocation_root` 在恢复内核里没有读者**：见第三节第 2 条末尾。

## 五、用例、变异与验证

### 新用例（全在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`；行号是副本里的，打在主工作区上要再加 19-1 在第 1185 行一带新加的那几十行）

| 行 | 用例 | 流 | 断言 |
|---|---|---|---|
| 2984 | `the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version` | 新池，数据单元诞生代号 3 → 2 重封到根 | CPU 红 I-3.5；每个状态逐位相同 |
| 2999 | `the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule` | 固定脚本到回退 D | CPU 在至少一个状态上判过 I-3.5 而成立、一个状态都不红它；逐位相同 |
| 3029 | `the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied` | 新池，发布那几条记录卸载记号 0b10 | 逐位相同（含落到的根） |
| 3052 | `the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root` | 新池，记录里实例表指针两条校验和取反 | CPU 红 I-8.10；逐位相同 |
| 3066 | `the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied` | 新池，记录里实例表指针 extent 偏移 1 | CPU 红 I-8.10；逐位相同 |
| 3080 | `the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on` | 新池，记录里分配记录树根换成实例表指针、校验和取反 | CPU 红 I-4.1；逐位相同 |
| 3093 | `the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root` | 新池，记录卸载记号 1 | CPU 红 I-8.10；逐位相同 |

「CPU 红某条」都是我按代码推的（第三节与第四节的依据），没跑。都不是崩溃枚举用例（不调 `enumerate_layer0` 一族、名字不带 `every_crash`，与同文件已有的对拍用例同一个形态），不要 `crash-case:` 登记。

### 变异行（`mutations-append.tsv`，12 行，参数都是 `-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- <点名的用例>`）

| # | 变异名 | 点名的用例（行号见上表） | 我推的「为什么红」 |
|---|---|---|---|
| 1 | 第 9 批 GPU 判器：I-3.5 出现规则那一判恒成立 | 2984 | CPU 红 I-3.5、GPU 不红 |
| 2 | 第 9 批 GPU 判器：I-3.5 不认整棵继承下来的用户可见树（(b) 那一支丢了） | 2999 | 回退那一版 (2,9) 引用 txg 3 的单元只能靠 (b) 成立，GPU 误红 |
| 3 | 第 9 批 GPU 判器 RS5：恢复收新根段卸载记号位 0 之外有位为 1 的记录 | 3029 | 记录落了根没落的状态上 GPU 重放到 (1,3)、CPU 停在暖机那条根 |
| 4 | 第 9 批 GPU 判器 RS5：恢复施加记录时实例表照抄所选根 | 3052 | 同样那几个状态 CPU 走到文件之前读不出、GPU 读出文件，结局类别不同 |
| 5 | 第 9 批 GPU 判器：恢复不判记录新根段实例表指针的头 | 3066 | GPU 收了 core 不收的记录，落到的根不同 |
| 6 | 第 9 批 GPU 判器：恢复判记录新根段指针头时不看 extent 偏移 | 3066 | 同 5 |
| 7 | 第 9 批 GPU 判器 RS6：走施加出来的那一版不走记录里的分配记录树根 | 3080 | CPU I-4.1 红（那一版走到分配记录树根读不出），GPU 不走、不红 |
| 8 | 第 9 批 GPU 判器 RS6：走施加出来的那一版不走记录里的实例表 | 3052 | CPU I-4.1 红（实例表读不出），GPU 不走、不红 |
| 9 | 第 9 批 GPU 判器：I-8.10 第五段（实例表指针）不比 | 3052 | 根与记录都落了的状态上 CPU 红 I-8.10、GPU 七段里只差这一段而不比 |
| 10 | 第 9 批 GPU 判器：I-8.10 第六段（分配记录树根指针）不比 | 3080 | 同 9，差在第六段 |
| 11 | 第 9 批 GPU 判器：I-8.10 第七段（卸载记号）不比 | 3093 | 同 9，差在第七段 |
| 12 | 第 9 批 GPU 判器：池级 checker 收新根段卸载记号位 0 之外有位为 1 的记录（RR1 那一判丢了） | 3029 | GPU 收了这条记录、根与记录都落了时红 I-8.10，CPU 不收、不红 |

证过红的：0 行（新测试都在 checker 档包里，按定义第 3 步的证红不跑）。留给 checker-tier-crates-mutation-replay：12 行全部，加换掉的那 1 行。

我能做的替代核对（都在草稿目录里跑，命令与原样输出在下面）：
1. 补丁后的三个内核过 naga 30.0.1 的 WGSL 解析与校验（与 `crash_judge_gpu.rs` 单测 `parse_and_validate` 同一组调用，自己编了一个只链 naga 的小程序，`/tmp/claude-1000/m3-r1/batch9-gpu/work/naga-check/src/main.rs`，结论字号常量照 `verdict_layout_wgsl_constants` 抄成 14 / 8 / 4 / 12 / 13），并按 `kernel_inlined_size` 的算法量了展开之后的表达式数（预算 400 000）。
2. 打在主工作区当前内容上的结果（主仓 `crates/` 拷一份、`patch -p1` 打上本补丁）同样过，扫描内核那份带着 19-1 的改动。
3. 12 条变异逐条施加之后三个内核仍过解析与校验（变异不会因为编不过而「假红」）。
4. 补丁打上之后，主仓变异表（换上替换行、加上追加行）里文件在 `crates/singlefs-checker-tier/` 下的每一行原文在那份拷贝里恰好命中一次：`checked 655 bad 0 dupnames 0`。

naga（补丁后的副本，与打在主工作区上的那一份），原样：

```
recovery OK functions=121 inlined_expressions=290085
walk OK functions=236 inlined_expressions=268444
scan OK functions=178 inlined_expressions=318420
copy/crates exit=0
recovery OK functions=121 inlined_expressions=290085
walk OK functions=236 inlined_expressions=268444
scan OK functions=181 inlined_expressions=335546
work/merged/crates exit=0
```

改前（主工作区开工时那一份）是 recovery 271 283、walk 256 662、scan 318 420。变异逐条（第一列变异序号、第二列退出码）：`1 0 2 0 3 0 4 0 5 0 6 0 7 0 8 0 9 0 10 0 11 0 12 0`。

### 第 4 步那几样（在副本 `/tmp/claude-1000/m3-r1/batch9-gpu/copy` 里，一条 Bash 串起来跑，各段退出码各写一个文件）

```
secs=53
fmt exit=0
clippy exit=101
build exit=0
```

- `cargo fmt --all -- --check`：输出 0 行。
- `cargo clippy --offline --keep-going --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 那七条 `-D`（照 `.claude/singlefs-ai-sop/scripts/check.sh` 第 72–80 行现抄）：退 101，红的全在补丁之外的三个文件（都是 `shadow_unrelated`），我改的六个文件 0 处：

```
error: could not compile `singlefs-harness` (test "an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e165_replay_bounded_enumeration") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e171_three_oracle_collusion") due to 2 previous errors
error: could not compile `singlefs-checker-tier` (bin "e165_replay_bounded_enumeration" test) due to 7 previous errors
error: could not compile `singlefs-checker-tier` (bin "e171_three_oracle_collusion" test) due to 9 previous errors
```

  按文件数（`-->` 行）：`e165_replay_bounded_enumeration.rs` 14、`e171_three_oracle_collusion.rs` 18、`an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 4。三个文件在主工作区里都是别的会话的改动（`git status` 里 `A` / `M`）。
- `cargo build --offline --all-targets`：`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 44.47s`。另一趟 `cargo build --offline --all-targets --all-features`（带 `verdict-store,gpu`，编进对拍测试文件）也退 0、0 条 warning。
- 动到的 harness 档测试二进制：无（只动了 checker 档包）。
- `scripts/crash/crash-case-check.py` 对副本：只红 modules 一格、点名的是 `crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs`（补丁之外，主工作区原样就是这样），我的测试文件不在里面。

编译类 cargo 调用合计 6 次：只链 naga 的小程序 build 2 次、副本 build 2 次、clippy 2 次（第一次没带 `--keep-going`，停在别人那个 harness 测试上）；另有 fmt 2 次。ps 看到的负载：开工时有别的会话的 `cargo test -p singlefs-harness` 与 `cargo build --release -p singlefs-checker-tier --bins`（`/tmp/claude-1000/m3-formal-19-2/copy`）在跑，没有 qemu / fio / e152；各用自己的 target，没有等锁。内存包装两次拒（同会话里已有 8 条在跑、上限超过峰值表 2 倍），照它的出路加 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`、改 `auto` 重跑，那两次输出不算结果。

## 六、要主 agent 跑的命令

工作目录都是主仓根 `/home/fy5090/code/singlefs`，补丁打上之后跑；线程上限 15；内存上限我没量过这两趟的峰值，`auto` 被拒时照 `run-with-memory-cap.sh` 的出路给上限（`replay.sh` 的默认 `REPLAY_MEMORY_CAP` 是 8G，推的不够：每张卡的草稿区在显存里，主机这边是 CPU 判器的线程）。

1. 对拍与判器单测（重型，带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：
   ```
   SINGLEFS_HEAVY_TESTS=commit BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/lib/gcc/x86_64-linux-gnu/14/include" \
     bash scripts/hosts/run-with-memory-cap.sh auto bash scripts/hosts/capped.sh 15 \
     cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --lib --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state
   ```
   看：`test result:` 两行都 ok、跑到的条数不为 0；`CRASH_JUDGE_GPU flow=… mismatches=0` 每条流一行；打 `NOT_RUN_NO_GPU_ADAPTER` 的是本次未跑、不算过。`--lib` 那一趟里要过的是 `crash_judge_gpu::tests` 三条 `*_parses_and_validates`、`scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels`，`crash_judge_dispatch::tests` 的 `states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit`、`every_kernel_stays_within_the_inlined_size_budget`、`the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused`。红了只重跑红的：`python3 scripts/gate/rerun-failed-tests.py <日志> -p singlefs-checker-tier`。
2. 新变异行与换掉的那一行（重型）：
   ```
   SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/checker-tier-crates-mutation-replay.sh \
     --item '第 9 批 GPU 判器：I-3.5 出现规则那一判恒成立' … （mutations-append.tsv 第一列 12 个名字逐个 --item）\
     --item '第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表'
   ```
   看：13 行每行判红。
3. 判法摘要变了（五份内核源码都进 `gpu_judge_digest`）：崩溃放量库里 GPU 判的旧行留作历史、新一版从头判；这一趟不归我起。

## 七、受影响的层 0 流与崩溃枚举用例

`crates/singlefs-checker/src/` 一个字没改：池级 checker（CPU）的判定集合不变，层 0 流与崩溃枚举用例的钉值不跟着变。变的只有 GPU 判器：I-3.5 那一位从恒 0 变成照判，崩溃放量由 GPU 判的那几趟从此判得到 I-3.5（`crash_amplification` 按违例位取不变量）。

## 八、没做什么

- 没走三方对抗；checker 档（对拍、`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异表、Verus 形式化验证）一样都没跑，归提交时的主 agent；没提交。
- 新测试一条都没证红（全在 checker 档包里），12 行变异全部留给门禁。
- 没在真卡上编过内核（只过了 naga）；驱动编译用时与显存没量。
- 第四节 S1（`image.rs:72` 注释、`invariants.md:16`）、S2（种类 9–11 不走）、S3（共享树表的 I-9.25）没改；I-3.5 的缺口那一支（gap-row、N3）与共享树表那一支没有会红的流、没写变异行（S7）。
- 恢复内核里新取的卸载记号与分配记录树根没有读者，没写变异行（S8）。

## 九、草稿目录

`/tmp/claude-1000/m3-r1/batch9-gpu/`：交回前删掉了仓副本 `copy/`（连同它的 `target/`）、开工时的原样 `base/`、`work/merged/`、`work/mutsrc/` 与 `work/naga-check/target/`；留着 `patch/`（补丁目录）与 `work/` 里的日志（`final-*.log`、`clippy1.log`、`build1.log`、`naga1.log`）、naga 小程序的源码、三份草稿片段与 `validate_mutants.sh`，都不是编译目录或仓副本。
