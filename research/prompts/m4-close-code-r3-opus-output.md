# m4-close-code-r3 攻方腿（Opus）报告（2026-11-05）

攻击面：正文第四节，第一轮「过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱」，T1–T4 每格找反例。代码读冻结副本 `~/.local/share/singlefs/work/m4-close-code-r3/repo/`；下文引的 `crates/` 行号取那一份（与主工作区这几份逐字节相同，本腿开工时 `sha256sum` 比过前 12 位）。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-close-code-r3-opus-model/`。复跑（不是重型，harness 档两个测试目标；我没跑，交主 agent）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r3-attack/rerun-1
```

| 文件 | sha256 |
|---|---|
| `tests/inode_readers_judge_separator_keys_against_the_left_leaf.rs` | `ef70fadb187dc66c842f7b1aeaa48189c3d139a086ad9b1c2da5df99e6324cc1` |
| `tests/one_copy_failing_counts_on_the_frozen_tree.rs` | `5f48b250d9f93af1adcb3d6518ecd94e47542e0b4ab926649df4f0a211868d95` |
| `tools/rerun.sh`（照 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`） | `b556690b838854a3403eed6720e9769cbd80bba4395d41cfba0524f645b294c7` |

`rerun.sh` 把冻结副本拷进 `--work`，放进两份测试，各跑一个测试目标。测试断言的是**今天的行为**：两个目标都绿 = 下面报的那几形在冻结树上成立；末行 `name=attack_r3_summary both_targets_as_expected=yes`、退 0 算过。观测行以 `attack-r3:` 开头。**下文所有读数都没跑过，一律是「推的，待主 agent 跑」**；模型源码我没编过（子 agent 不编），编不过也算一种结局，报回来我再改。

## 各格判定一览

| 格 | 判定 | 一句话 | 形态 |
|---|---|---|---|
| T1 | 打中（两处） | ① 四个读者判的「I-9.12」只比记录号，不比分隔 key：右片分隔 key 改成 200（≤ 左片最大号 233），池级 checker 判 I-9.12 红，四个读者全收。② 右片分隔 key 改成 235（> 右片头一条 234）：三个读者拒在 I-9.4，只读挂载 `open_pool_for_read` 丢掉分隔 key、收下——同一张镜像四个读者两种结局 | 模型测试，推的，待主 agent 跑 |
| T2 | 打中（三处，另两处推的） | ① 实例表第 0 片一份坏：一次可写挂载经数「只坏一份」的那一层读它两遍，计 2（树表对照计 1）。② 整理搬走单元后盘上提示过期、旧槽一块盘那一份被盖：映射位置两份都好，可写挂载与只读 `read_at` 照样各计一次「只坏一份」。③ 只读 `read_at` 先读到只坏一份、后一个单元两份都坏而整次报错：这次的计数整份丢掉。另：预演不计、被拒的发布计，两处的理由互相打架（推的）；可写挂载只计不搬，D4 字面要搬（推的、代码注释自认） | ①②③ 模型测试，推的，待主 agent 跑；其余按代码推 |
| T3 | 没打中（一处小出入） | 另发那批经 `publish_the_compaction_step_once`、同一个事务层；预算口径与走空一致（每一次被拒的那次发布各算一次准入）；`current` 换法对。小出入：推满时不报「被宽护卡住」那一项，与走空、用户改动那两处不同（推的） | 按代码推 |
| T4 | 没打中 | 挪进新函数的那段 `match` 没有 hunk，逐字没动；`crates/mutations.tsv` 落在 `mounted_session.rs` 的 177 行锚点逐个恰好命中一次（量的：脚本数的，见第五节）；`allow(clippy::needless_borrow)` 只罩那一种借法，编得过就说明每臂传的是同一个写入口 | 按代码推 + 一次文本数数 |

## 一、T1　四个读者判 inode 叶交叠

### T1-①　读者拒因写 I-9.12，判的不是 I-9.12

I-9.12 的字面有三句：条目按分隔 key 严格递增、第 i 条分隔 key ≤ 第 i 个孩子的下界、且 > 第 i−1 个孩子的上界（「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界」（`crates/singlefs-checker/invariants/09-inode树结构.md:24`））。池级 checker 照这三句判（「.judge("I-9.12", separator > previous_largest, || {」（`crates/singlefs-checker/src/walk.rs:4689`））。

四个读者共用的那一判只拿「这一片头一条」比「前面那几片的最大号」，拒因写 I-9.12（「if largest_of_the_leaves_before.is_some_and(|largest_before| largest_before >= first.inode) {」（`crates/singlefs-core/src/recovery.rs:6017`）、「invariant: "I-9.12",」（`crates/singlefs-core/src/recovery.rs:6019`））。分隔 key 那两句里，「≤ 本孩子下界」三个读者在 I-9.4 那一判里顺带判了（「if record.inode < separator_key || record.inode < identity.container {」（`crates/singlefs-core/src/recovery.rs:6808`）），「> 上一个孩子的上界」与「条目严格递增」哪个读者都不判。

最小反例（镜像）：合法两片叶池（左片 1..=233、右片 234..=236，再建一个快照，快照的 inode 根就是活头那一棵），只把 inode 根第 1 条（右片）的分隔 key 从 234 改成 200，根头 key 区间 [1, 236] 不动，校验和沿引用链补到根槽。
- 期望（照 I-9.12 字面与 r2 判决「四个读者一个结局」的意图）：池级 checker 红 I-9.12，四个读者拒。
- 实际（推的，待主 agent 跑 `a_right_leaf_separator_not_above_the_left_leaf_is_red_on_i_9_12_and_accepted_by_every_reader`）：checker 红 I-9.12；可写挂载、冷走读、只读挂载打开池、快照冷读都收下——记录号按叶序严格升序、都不小于分隔 key 与容器号，读者那几判一判都不中。
- 后果：可写挂载收下之后，写者不改不重写的叶的分隔 key（`inode_tree.rs` 照抄 `separator_key_in_the_parent_entry`），之后每一次发布写出的版本池级 checker 都红 I-9.12；产品的查找按叶序折半（`TransactionOutput::inode_record_found_by_its_leaf`），不按分隔 key 走，所以产品读写本身不错——错的是「读者与 checker 判同一张镜像一个结局」这一句。

### T1-②　同一张镜像四个读者两种结局

只读挂载打开池把分隔 key 解出来就丢掉（「let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry_or_refusal(」（`crates/singlefs-core/src/mounted_read.rs:509`）），不判「记录号 ≥ 分隔 key」；另三个读者判（上引 `recovery.rs:6808`）。两处注释都说四个读者一个结局（「同一张两片交叠的镜像读 inode 树的各个读者一个结局」（`crates/singlefs-core/src/mounted_read.rs:573`）、「同一张两片交叠的镜像四个读者一个结局」（`crates/singlefs-core/src/recovery.rs:7595`））。

最小反例：同一个两片叶池，右片分隔 key 从 234 改成 235。
- 期望：四个读者一个结局。
- 实际（推的，待跑 `a_right_leaf_separator_above_its_first_record_splits_the_four_readers`）：可写挂载、冷走读、快照冷读拒在 I-9.4（「inode 记录号小于分隔 key 或容器号」）；只读挂载打开池收下；checker 红 I-9.12。

### 四句

| 问 | T1-① | T1-② |
|---|---|---|
| 分不分辨臂 | 分：r2 判决取的改法甲（判叶间记录号严格升序）在这张镜像上收、照 I-9.12 字面判分隔 key 的那条臂拒 | 分：只读挂载接没接 I-9.4 的分隔 key 那一判 |
| 被判系统当时看不看得到 | 看得到：三个读者手里有分隔 key（`recovery.rs:3808` 那一行解出 `separator_key`），只读挂载也解出了、丢成 `_separator_key` | 同左 |
| 满足的是字面哪一分句 | I-9.12「且 > 第 i−1 个孩子的 key 区间上界」；读者拒因标的 I-9.12 实际判的是「记录号跨叶严格升序」，那是写者的前提（`inode_tree::leaf_containers_are_well_formed` 前一半），I-9.12 与 I-9.4 字面里都没有这一句：I-9.12 后两句推得出它（第 i 个孩子的下界 ≥ 分隔 key > 第 i−1 个孩子的上界），它推不出 I-9.12——「打中归错了判据」那一形落在代码的拒因标签上 | I-9.4 第一句（记录号 ≥ 容器号）的分隔 key 版本只有三个读者判 |
| 跑前条款的改法还中不中 | r2 判决第三节取的改法甲就是被打中的这一判，它在 T1-① 上照样收 | 改法甲不碰只读挂载丢分隔 key 那一处，照样中 |

能不能做成会红的测试：能。模型两条用例断言今天的行为（绿）；把断言翻过来（读者拒）就是修完之后的必红用例；变异「共用那一判不比分隔 key」在修完之后由这两条杀。

改法（只在我的模型上想过、没实现没量、被攻过零轮）：共用那一判多收这一片的分隔 key，判「分隔 key ≤ 头一条」「分隔 key > 上一片的分隔 key」「分隔 key > 前面那几片的最大号」三句，四个读者都把分隔 key 交进去（只读挂载那一处不再丢）。修 T1-① 与 T1-② 两格（推的）。

## 二、T2　只坏一份的计数

### T2-①　同一份坏副本，一次可写挂载计两次

挂载把建上一版的读经一层数「只坏一份」的读者（「PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);」（`crates/singlefs-core/src/mount.rs:9416`）），并且明说判退的三样读者不经它，因为「判完不留字节，经它就把同一份坏副本多数几遍。」（`crates/singlefs-core/src/mount.rs:9414`）。可建上一版自己就把实例表第 0 片读两遍，两遍都经这一层：
1. `rebuild_version_in_the_unit_area_starting_at` 先读自举单元：「read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;」（`crates/singlefs-core/src/recovery.rs:3598`）；
2. `rebuild_previous_version` 接着沿链读整张表：「let table = instance_table_chain_of_root(reader, root)」（`crates/singlefs-core/src/mount.rs:1341`），链从根记录那条指针起（「let mut page_pointers = vec![root.instance_table];」（`crates/singlefs-core/src/recovery.rs:2889`）），第 0 片再经 `read_unit_via_locations` 读一遍（`crates/singlefs-core/src/recovery.rs:2894`）。

两遍都是头一条对不上、第二条对上，`read_unit_via_locations` 各报一声（`crates/singlefs-core/src/recovery.rs:820` 那一判），套子各加一。

最小反例（历史）：mkfs → 取号 → 暖机 → 新池新建文件 → 顺序写成三个单元（内存稀疏盘），把这一版根记录里实例表那条指针头一条位置条目那块盘上的那一份改一个字节，可写挂载。
- 期望（照挂载自己那句「不多数」、C946 用例「计数各加一」）：`unit_reads_with_one_copy_failing_while_mounting` = 1。
- 实际（推的，待跑 `a_writable_mount_counts_one_damaged_copy_of_the_instance_table_twice`）：≥ 2（按代码数是 2）。对照 `a_writable_mount_counts_one_damaged_copy_of_the_tree_table_once`：树表只在自举那一处读，计 1。

同一个根子还有一形（推的，没造历史）：判退为真、退到的那一版 `rebuild_previous_version(…).ok()` 建不出（`crates/singlefs-core/src/mount.rs:9442`），留在所选那一版再建一遍（`crates/singlefs-core/src/mount.rs:9488`），两遍共用一个套子：两版共有的单元（多数单元两版共有）里只坏一份的，加上没建成那一遍里已读到的，一次挂载各计两次。

### T2-②　提示过期、旧槽一块盘被盖：映射位置两份都好，照计「只坏一份」

整理那一批只改映射，盘上 extent 叶里的提示仍指旧槽（`compaction_relocation_batches_publish_through_the_mapping.rs` 里清零旧槽后冷走读要经映射回退，就是这个前提）。旧槽释放之后，一次没发布成、只落了一块盘的写（崩溃把两块盘的写撕开）盖掉其中一份，另一块盘上还是这个单元的旧字节，校验和与提示里的位置条目相符。三条读路径都先按提示读（「if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {」（`crates/singlefs-core/src/recovery.rs:938`）、「read_unit_by_location_entries(reader, &pointer.locations, data_unit_bytes, observation);」（`crates/singlefs-core/src/mounted_read.rs:1061`）），头一份对不上、第二份对上，就计「只坏一份」（「observation.data_unit_reads_with_one_copy_failing += 1;」（`crates/singlefs-core/src/mounted_read.rs:1211`）），根本不看映射位置上那两份。

最小反例（历史）：同上的三个单元 → `publish_relocation_batch` 搬第 1 个数据单元 → 旧槽头一条位置条目那块盘上那一份改一个字节（模型直接改字节，代替「没发布成、只落一块盘」那次写；崩溃枚举那一形没跑，见第六节）→ 只读挂载从 0 读到 size → 可写挂载。
- 期望：映射位置上两份都好，这个单元没有坏副本，两处都计 0；对照不改旧槽也是 0。
- 实际（推的，待跑 `a_stale_hint_whose_old_slot_lost_one_copy_is_counted_although_both_mapped_copies_are_intact`）：只读 `read_at` 计 1、可写挂载建上一版计 1，读出的内容对。

D4 的射程句自己列了这一类来源（「所以只坏一份只来自设备故障，或者读的是旧版本、它的槽释放之后又让一次没发布成的写盖掉了一块盘上那一份」（`.claude/kb/decisions/04-校验和位置.md:73`）），说的是「读旧版本」；这里读的是最新那一版，只是经了过期的提示。把它也算「只坏一份」，按定案下一句就该「经已有的搬迁路径把这个单元整个搬到新槽」——搬的是映射位置上那两份好的，盘上那份被盖的旧槽谁都不修。哪一读法对，条款没写，交主 agent。

### T2-③　一次读报错，这次读到的只坏一份整份丢

`read_at` 在本地攒一份观测，读完才并进打开之后的累计（「self.mount.add_to_observation_since_open(observation);」（`crates/singlefs-core/src/mounted_read.rs:1043`））；中途任何一个单元报错，`?` 直接交回（「self.dereference_data_unit(reader, record, unit_index_in_file, &mut observation)?;」（`crates/singlefs-core/src/mounted_read.rs:1006`）），前面单元读到的只坏一份不进累计。同一份观测里 `device_reads_issued` 的文档写着读不回也算（「/// 向块层发了几次读（每条位置条目试一次就算一次，读不回字节也算——请求已经发出去了）。」（`crates/singlefs-core/src/mounted_read.rs:81`）），报错那一次的块层读同样不进累计。

最小反例（历史）：三个单元的池，只读挂载、打开第一个文件（盘没坏），之后第 1 个单元头一份坏、第 2 个单元两份都坏（挂着时设备出错），从 0 读到 size，再单读第 1 个单元。
- 期望：打开之后累计的只坏一份在报错那一次之后是 1。
- 实际（推的，待跑 `a_read_failing_on_a_later_unit_drops_the_one_copy_failing_count_of_an_earlier_unit`）：报错之后是 0，单读第 1 个单元之后才是 1。

### T2-④　预演不计、被拒的发布计：两处理由互相打架（推的，没造历史）

被拒的那几次发布读回的照计，理由是读真发到了盘上（「/// 被拒的那几次发布读回的也算：读是真发到盘上的。」（`crates/singlefs-core/src/mounted_session.rs:259`））。预演不计，理由换成「不是真改动」（「// 预演里读回的那几次不并进这次挂载的累计：预演不是一次真改动（会话的分配器与现行那一版都不动）。」（`crates/singlefs-core/src/mounted_session.rs:1742`））；可预演的盘是罩在真盘上的（「(*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))」（`crates/singlefs-core/src/mounted_session.rs:1713`）），读回旧单元同样真发到盘上。按前一条的理由预演该计，按后一条的理由被拒的发布不该计。同一次改动里推 k 串抬 F 再发，同一份坏副本计 k + 1 次；这与挂载那句「不多数」又是第三种口径。会红的测试：要一次准入先被空间拒、再发成的部分写（小盘造得出，没造），断言计数 = 1；推的。

### T2-⑤　可写挂载只计不搬（越位，代码注释自认）

挂载注释自己写着「// 只计数、不搬：挂载这一串里另发一批搬迁条款没写（要不要、放在哪一步、取号之前的预演罩不罩它、被拒怎么办），交主 agent。」（`crates/singlefs-core/src/mount.rs:7658`）。用户 2026-11-04 那三条只给了只读挂载「只计数，不修」、有活快照时另发一批、另发被拒推抬 F；D4 定案「读到只坏一份时计数，并经已有的搬迁路径把这个单元整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层，不开第二条写路径。」（`.claude/kb/decisions/04-校验和位置.md:71`）不分读的地方。可写挂载建上一版读到的只坏一份今天没有任何一步去修（之后的部分写碰巧读回它才修），这一格条款与用户定案都没盖住，代码交了主 agent，判决里要有去向。

### T2 四句

| 问 | T2-① | T2-② | T2-③ |
|---|---|---|---|
| 分不分辨臂 | 分：「按读计」读法下 2 是对的，「按坏副本计、一次挂载不多数」读法下 1 才对；补丁两种都用（挂载那句注释是后者，被拒发布照计是前者） | 分：「前一条读坏就算只坏一份」与「读坏的那一条是这个单元映射位置上的副本才算」 | 不分臂，是实现漏 |
| 被判系统当时看不看得到 | 看得到：套子收到的是那一组位置条目（`recovery.rs:284` 那个函数的参数），两遍是同一组 | 看得到：建上一版与只读挂载手里都有中央映射（回退时就查它），读坏那一条的槽与映射位置不等 | 看得到：观测就在本地变量里 |
| 满足的是字面哪一分句 | D4 定案「读到只坏一份时计数」不说按读还是按单元；C946 用例「计数各加一」与挂载注释「不多数」是按单元那一读 | D4 定案「读到只坏一份时计数，并经已有的搬迁路径把这个单元整个搬到新槽」；射程句的来源列表只列「读的是旧版本」 | D4 定案「读到只坏一份时计数」 |
| 跑前条款的改法还中不中 | 条款没给改法 | 条款没给改法 | 条款没给改法 |

### T2 几个改法各修哪一格

改法都只是想过、没实现，被攻过零轮。

| 改法 | T2-① 实例表两遍 | T2-① 判退后再建 | T2-② 过期提示 | T2-③ 读报错丢计数 | T2-④ 预演与被拒 |
|---|---|---|---|---|---|
| 套子按位置条目去重（一次挂载里同一组只计一次） | 修，推的 | 修，推的 | 不修，推的 | 不碰 | 不碰（会话那一份不经套子） |
| 计「只坏一份」之前比读坏那一条的槽与映射位置，不等就记成提示过期 | 不碰 | 不碰 | 修，推的 | 不碰 | 不碰 |
| `read_at` 报错之前也把这一次的观测并进累计 | 不碰 | 不碰 | 不碰 | 修，推的 | 不碰 |
| 会话按映射 key 去重计数（一次改动里同一单元只计一次） | 不碰 | 不碰 | 不碰 | 不碰 | 修被拒重发那一半，推的；预演那一半要先定口径 |

## 三、T3　有活快照时另发一批搬迁与被拒推抬 F

没打中。攻过的角度：
- 第二条写路径：另发那批走 `publish_the_compaction_step_once` → `publish_compaction_step_with_the_region_check_of_the_intent_deletion`，写入口照 `PoolWriter::new` 新开、意图照抄，与整理同一个事务层；留存条目的字节读两份、取先核上的那一份（`relocation_checked_against_the_version`），头一份坏不会让它 `UnitBytesNotAvailable`。
- 计划用的源落点：`relocations_of_old_data_units_kept_for_the_snapshots` 取留存条目的 `locations[0].slot`（「slot: locations[0].slot,」（`crates/singlefs-core/src/transaction.rs:11696`）），发布路径要两条位置条目同槽才收；分配是两盘同槽，对得上。
- 推抬 F 的预算：另发那批「推的数从 0 起」，与走空那一套「每一次被拒的删除类发布各算一次准入」（`crates/singlefs-core/src/mounted_session.rs:5054`）同一口径；D16 已定项 1「一次准入最多 B = 4 + 2 k_tol 次发布」管的是一次准入，不是一次调用。同一次调用里用户改动那一准入与另发那批各推满时一共可推两份预算，这是走空本来就有的形，不是新的越位。
- `current`：推成的抬 F 先换进 `current`，发成再换成那一批那一版；被拒时停在最后一次落盘的那一版。推到一半落盘出错时，`push_one_floor_raise_within_the_admission_budget` 就地推进 `version`，换进 `current` 的是已落盘那几次之后的那一版，对。
- 三个开关：都住 `CompactionOfThisMount`，`of_a_new_mount` 一律 `NotForced`，挂载交回的那一份照它建，产品路径没有调用处（量的：在冻结副本 `crates/` 下 `grep -rn` 三个 `force_the_…_for_tests` 函数名，按文件数得 `3 crates/singlefs-core/src/compaction_plan.rs`（三处定义）与 `7 crates/singlefs-harness/tests/data_unit_read_with_one_copy_failing_is_counted.rs`，别处 0）。
- 排空被拒那一支：五份补丁的 hunk 都不落在攒批会话那一段（改动全文里没有那一段的 hunk）。

一处小出入（推的，不算打中）：另发那批推满时交回 `NoSpaceAfterRaisingTheFloor { last_refusal, stop, floor_raises }`（`crates/singlefs-core/src/mounted_session.rs:2203`），没有走空与用户改动那两处都报的「被宽护卡住」那一项（「wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(」（`crates/singlefs-core/src/mounted_session.rs:5032`））。说明里写的「与会话走空同一套」（`crates/singlefs-core/src/mounted_session.rs:213`）在这一项上不成立：被宽护卡住时调用方从这一批的结局里看不出要推环才有出路。用户改动本身已发成，这一项只影响诊断。

## 四、T4　`publish_the_change_once` 的挪动

没打中。攻过的角度：
- 每臂走的路：改动全文里 `match change {` 是上下文行，各臂没有 hunk，逐字没动；挪进的新函数收 `mut writer: &mut PoolWriter`，各臂照写 `&mut writer`，得到 `&mut &mut PoolWriter`，在函数实参这个强制转换点上解一层，传进去的仍是 `publish_the_change_once` 新开的那一个写入口；之后 `add_the_reads_of_the_writer(&writer)` 读的也是它。
- `allow` 掩住的东西：`needless_borrow` 只报「多借一层再自动解掉」，编得过就说明各臂的形参都是 `&mut PoolWriter<'_, Device>`（形参是泛型 `W` 的话，`&mut &mut PoolWriter` 没有那个实现，编不过）；它不掩类型或行为问题。
- 变异锚点：用脚本把 `crates/mutations.tsv` 里落在 `crates/singlefs-core/src/mounted_session.rs` 的行逐个按原文在冻结副本那一份里数（`\n`、`\t` 照转义还原），177 行、每行恰好命中 1 次（量的，脚本输出「rows 177 not-exactly-once 0」，命令没留档，判决要用请主 agent 重数）。第 2471 行那条变异插的 `thread_local!` 与块表达式 `{ …; &mut writer }` 在新函数里同样编得过、语义同前。

## 五、没打中的形状

- T1「叶跨盘副本不一致时误拒合法镜像」：叶按位置条目读，只收校验和与那一条相符的那一份；两份内容不同时收的就是指针要的那一份，不会把合法的一版判成交叠。没造出反例。
- T1「空叶、单条叶误拒」：共用那一判空叶交回原来那个最大号、单条叶不进 `windows(2)`，都不误拒；空叶本身由可写挂载重建另拒（「inode 叶容器没有记录」），冷走读、只读挂载、快照冷读不拒——这是四个读者的另一处不一致，但要三片叶才造得出「空的那片不改根头 key 区间」的镜像（两片时根头区间先拒），我没造，算推的、不算打中。
- T1「写者合法写出的镜像被拒」：inode 号在一棵树里单调、分裂只在最右、借只取均分，写者写不出跨叶不升序的一版（D8 已定项 6「插入永远落在最右的叶」那一段）；崩溃恢复重发号只在被抛弃那条时间线上，选中那棵树里的号都小于水位。没造出反例。
- T1「第五个读者」：在冻结副本 `crates/singlefs-core/src/` 里搜 `InodeRecord::parse` 与 `inode_leaf_container_judged_against_its_entry` 的调用处，读 inode 叶记录的只有这四处；`inode_tree.rs` 那一处是文档里的说明串。mount 的读阶段缓存、回退判法那三样读者不读 inode 叶记录（只看了调用处，没逐个跟到底）。
- T2「两份都坏时照计」：三处都是「前一条坏且后一条对上」才计，两条提示都坏转去映射的那一回不计；映射那一组再两份都坏照旧返错。没造出反例。
- T2「计数写盘」：三份账都只在内存（`Cell`、结构体字段），没有一处进发布的字节。没造出反例。
- T2「重开、续做、整理之后」：会话的挂载期状态从 `output.compaction_continuation.state_of_this_mount.clone()` 建，挂载那一处在交回之前记数；`of_a_new_mount` 只在挂载建一次（冻结副本 `crates/singlefs-core/src/` 里 `CompactionOfThisMount::of_a_new_mount()` 除定义外的调用处只有 `mount.rs` 一处，量的，`grep -n` 看的）。整理不读回旧数据单元，不动这两个数。没造出反例。
- T3 见第三节；T4 见第四节。

取样范围：模型三张镜像、六段历史，全部是两块 4 GiB 内存稀疏盘上的单个池，几何只有 `common::parameters()` 那一种；没扫几何、没扫单元数、没扫被弄坏的字节位置（固定第 4000 字节）。由用户决定的动作（写哪个单元、搬哪一个、读哪一段）各只取一种，没放开扫。

## 六、这条腿自己的限度

- 模型源码没编过、没跑过（子 agent 不编、不交任务）。所有「实际」都是按代码推的；编不过或某条断言红，说明我对某个 API 或某一步推错了，那一格回到「没核」。
- T2-② 用直接改一块盘上的字节代替「没发布成、只落一块盘的写」，没经录制写流与崩溃枚举（`enumerate_crash_states`）造那个崩溃态；我推它在崩溃枚举里是一个合法状态（同一段写里两块盘的写是两件、子集里有只落一块的），这一句没量。要坐实得在整理一批之后再发一次会复用旧槽的发布、录写流、枚举崩溃态、每个态可写挂载数这个数；那要先让分配器真复用到旧槽，我没造。
- T2-①「判退后再建」、T2-④ 没造历史，只按代码推。
- T3 的「没打中」只抽了一次，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不能单独支撑结论。
- 正文第一节说 C946 在 `.claude/kb/checks-owed.md` 第 595 行；我现查是 `.claude/kb/checks-owed.md:593`（这份文件与开工快照 `kb-sha256.txt` 那一行 sha256 相同，量的），对不上，按 593 引。
- 引文的行号取冻结副本；主工作区这八份被引文件与冻结副本前 12 位 sha256 逐个相同（量的，开工时比的）。

## 要主 agent 跑的命令

1. 模型复跑（harness 档两个测试目标，不是重型）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-close-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-code-r3/repo --work /home/fy5090/.local/share/singlefs/work/m4-close-code-r3-attack/rerun-1`
   - 在哪起：仓根（`--work` 写绝对路径，是本腿草稿目录下新的子目录）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT … exit=0`；日志里 `name=attack_r3_separators_target rc=0`、`name=attack_r3_counts_target rc=0`、`name=attack_r3_summary both_targets_as_expected=yes`；`attack-r3:` 观测行逐行对第一、二节的「实际」。`rc` 非 0 时看同段打出的 `test … FAILED`、`panicked at` 与 `error[E…]` 行：编不过是模型的错，某条断言红是那一格没打中或我推错。跑完 `rerun.sh` 自己删 `$WORK/frozen-tree`；编译目录归服务，`task-server/task clean <号>`。
2. 接着做：主 agent 跑完，新派同立场的腿（「承接：research/prompts/m4-close-code-r3-opus-output.md」，带任务号），读 `task-server/task log <号> --all`，按读数把第一、二节「推的，待主 agent 跑」改成量过或撤回。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上的读数（还没有）不算入库装置上的数；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没写会红的产品测试进仓、没改产品代码；改法都只想过，被攻过零轮。
- 没跑崩溃枚举（T2-② 见第六节）；没碰 GPU 判器与 clippy 那两处（正文第二节说不为它们开格）。
- 草稿目录里只有我拼 `rerun.sh` 与改测试用的四个文本片段（`old1.txt`、`new1.txt`、`old2.txt`、`new2.txt`）与 `cite-check.py` 的输出 `cite.txt`，没有仓副本、编译目录与库拷贝。
