# m4-validate-code-r2 辩方腿（Sonnet）报告（2026-10-24）

立场：复核 m4-validate-code-r1 判决定下的改法（T5 甲用户定「丑」）与 E187 用户定的落点甲，落到代码之后够不够得着、是不是同样打中替代方案。引的代码行号都是冻结副本 `~/.local/share/singlefs/work/m4-validate-code-r2/repo/` 里的；引到的十四份产品代码文件与主工作区此刻逐字节相同（`cmp`，下面第 0 节），所以报告里的 `crates/…:行号` 在两处都对。

## 0. 复跑与哈希

模型目录 `research/prompts/m4-validate-code-r2-sonnet-model/`（探针源码 `probes/`，复跑脚本 `tools/rerun.sh`，输出 `out/`）。复跑一条命令（整条经内存包装；脚本自己再逐个测试目标经 `capped.sh 31`；编译目录默认在 `--work` 里；`RERUN_TARGET_DIR` 给了就用它，我分趟热编译时用，主 agent 复跑不必设；热编译后本机四分钟内跑完，冷编译（依赖要重编）会更久，超过前台时限就用后台起）：

```
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=辩方腿模型目录 rerun.sh 里 singlefs-harness 与 singlefs-core 的测试目标; 范围=17 个测试目标; 为什么=复核代码轮第二轮标签 0 写者与隔离记号并量探针与对照副本' \
  bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-validate-code-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-validate-code-r2/repo \
  --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/m4-validate-code-r2-sonnet-model
```

跑什么：把冻结副本的 `crates/` 拷进 `--work`，探针装进 harness 的 `tests/`（标签 0 探针 = 冻结副本里 `whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs` 前 287 行辅助函数逐字 + `probes/tag_zero_probe_tail.rs`；隔离记号探针 = 冻结副本里 `release_checksum_quarantine_mark_written_to_disk_and_cleared_by_the_rollback.rs` 前 115 行 + `probes/quarantine_probe_tail.rs`），跑它们与冻结副本里八个相关测试目标和核心层五组单测；再从 `git archive HEAD` 导出不认记号的旧代码读新写者倒出的镜像；再在草稿副本里改一处（T1.4）跑对照。另有两个小脚本：`tools/mutate_guard.sh`（T6，弄坏守卫跑两条新用例）与 `tools/guard_leftovers.sh`（T6，数测试跑完在自设 TMPDIR 里剩几份），都吃 `rerun.sh` 拷出来的 `<work>/tree`。**一个坑**：cargo 的编译单元哈希不含绝对路径、按源文件时间判新旧，几棵内容不同的树共用一个编译目录时，后一棵会拿前一棵编的二进制；三个脚本因此每棵树开跑前把 `crates/` 下的源文件时间碰新。我第一趟没碰，旧读者那一步的池级 checker 用了新代码的 checker（`checker_checked=108`），那一趟的输出作废，换成现在 `out/` 里的（`checker_checked=103`）。`tools/rerun.sh` 的模板自证（`--selftest`）没跑：`rerun_steps` 换成了这一腿的步骤，模板自证断言的是模板自带那一段的输出。

```
$ cd research/prompts/m4-validate-code-r2-sonnet-model && sha256sum probes/* tools/* out/*
11bd979c1858182d02671bab8a69c40f0f52553eeb3dd7094f5529b2f6dde3cf  probes/old_reader_probe.rs
294fa6d06392d5ed2109b44d30b7aaeb40610f10e0cd6bda8dba7f2af966f21e  probes/quarantine_probe_tail.rs
089495764b3bc749c71b634ab74a9b085e914f6ac0cc464275db4c846d9cf491  probes/tag_zero_probe_tail.rs
45db3a6f8a7c7b8662ae70ca0f010237740dd98c068382b76b904e7f5bdabbff  tools/guard_leftovers.sh
51a3cf9824c1635a4544ef5a814f5b08ef3bf445613717fc3e371417e1a5dfdb  tools/mutate_guard.sh
9f064212d9d305fa00d41729780297f82f5689a8a17277b6d21e401ca5de0312  tools/rerun.sh
16a5ad38f804fa1cc780253789e28cc1135a3f1ed62a52005a86df95eaf5ce60  out/guard-leftovers-mutant-files.txt
319ccdf06876027bd01f270cc638624425831127212667a55f0a8600803ae803  out/guard-leftovers-stdout.txt
f58e11e9e821d7de54206e742e450d05967fae8334de977f3e1af6cf0c439140  out/mutate-guard-stdout.txt
4bb020e93611b193c4c367a1cd096e850c9faa7cd4a54e1a1117c5ea67380785  out/old-reader-probe-lines.txt
2b0a44fe05f5ef760b7b0a428138f9f4be2e2597f9455453f932c9d2adb03c45  out/rerun-stdout.txt
0a58ef2dc94e2f301f4ce6774f5a4e167dd48e03cdae0d6435da5da3b2981922  out/test-results.txt
```

`out/rerun-stdout.txt` 是 `tools/rerun.sh` 这一次（`--work` 为草稿目录下的 `run7`）打到标准输出的全部 79 行（探针行 59 行、目标退出码行、`old_reader_code head=841b3beabb9812ceaafcd636888b1201dd6e5adf`）；`out/test-results.txt` 是每个测试目标的 `test result` 行（17 个目标全 ok、0 failed，含对照副本里的三个）。

冻结副本与主工作区的比对（命令与原样输出）：

```
$ for f in truncate extent_tree transaction allocator allocation_record_tree mount recovery region_scan compaction_plan block_device mounted_read write_request_split; do cmp -s <冻结副本>/crates/singlefs-core/src/$f.rs crates/singlefs-core/src/$f.rs && echo same $f; done   # 另加 singlefs-checker/src/walk.rs、singlefs-reference-filesystem/src/lib.rs
（十四行全是 same）
```

## 判定一览（细节在 T1–T6 各节，末尾有表）

- T1：N > 0 的三种改 size 站得住；**N = 0（标签 0 的文件截到 0）这一格够不着用户定的「丑」**：打中，且只打中「丑」这一种实现，不同样打中子、寅。一处判法造成，草稿副本里改一处就都读得回。
- T2：站得住。T3：站得住（清记号）；「释放隔离记录」那一句没有活路径走到，只有单测钉着。
- T4：站得住；两处没动的现状（区域扫描不认记号、常驻隔离集不从记号起种）是主 agent 已记的欠账，这一批没修，原因见 T4。
- T5：「丑」改法的完成度见 T1；第一轮判决打中的两处各自状态：甲（panic）修了（N = 0 一格除外）、乙与 T10（大 size 整份分配）没修、没变难也没变易——不是「岔路还开着」，判决里用户已定这一版不改。不带位的旧读者：实测与 E187 / D19 的写法有一处对不上（只读挂载没拒）。
- T6：站得住。

## T1　标签 0 配 size > 0：写者接得住这一形

判定：**N > 0 的三种改 size 站得住；N = 0（标签 0 的文件截到 0）一格够不着用户定的「丑」，打中。** 毛病在「丑」的这一版实现里，不在「丑」本身，也不同样打中子、寅。

### T1.1 代码做的是不是用户定的那一种

用户定的原文，判决第四节 T5 甲「丑」那一格，`research/prompts/m4-validate-code-r1-main-verification.md:49`：「变长与 N = S 只改记录、标签留 0，截短只改记录」。落进 D8 的条款，`.claude/kb/decisions/08-核心索引结构.md:451`：「这一形上 size 改大、改成原值、截短都只改 inode 记录、标签留 0、不加单元」；用户定案在 `.claude/kb/decisions/08-核心索引结构.md:477`：「标签 0 配 size > 0 的第一个文件，写者接得住这一形」。K5 的「变长」一句在判决 `research/prompts/m4-trunc-r1-main-verification.md:39`，没有改（历史判决不改），改后的口径活在 D8 条款里。

判决要的三处，对到冻结副本：

| 判决要的 | 代码 | 状态 |
|---|---|---|
| `truncate.rs` 变长那条断言的前提 | 「let unit = DataUnitIndexInFile(data_units_existing.checked_sub(1)?);」（`crates/singlefs-core/src/truncate.rs:141`）：一个单元都没有时交 `None`，不重写、不加单元 | 做到 |
| `truncate.rs` 截断计划那条断言的前提 | 「data_units_after <= data_units_before && keeps_at_least_one_unit_or_had_none,」（`crates/singlefs-core/src/truncate.rs:240`），前提一行在 `crates/singlefs-core/src/truncate.rs:238` | 做到 |
| 重写上段叶不丢第一个文件那一条 | 「(None, None) => size_of_the_first_file_after_this_publish」（`crates/singlefs-core/src/transaction.rs:12339`）接下一行的 `.and_then(ExtentUpperLeafTarget::of_a_file_without_data_units)`；上段形状的号集合经同一个函数（`crates/singlefs-core/src/transaction.rs:4001-4004`） | 做到，但那个函数在 size 0 上取 `None`，见 T1.3 |

不碰 extent 树、不 panic：三种 N > 0 的改 size 在手造的标签 0 镜像（3000 字节）上做完，extent 树根的槽号前后相同、条目仍是标签 0（rerun-stdout 第 51–54 行，原样如下；第四行是截到 0）：

```
PROBE name=extent_root_extend_to_three_payloads extent_root_slot_before=50240 extent_root_slot_after=50240 entry_after=tag0
PROBE name=extent_root_same_size_3000 extent_root_slot_before=50240 extent_root_slot_after=50240 entry_after=tag0
PROBE name=extent_root_shrink_to_100 extent_root_slot_before=50240 extent_root_slot_after=50240 entry_after=tag0
PROBE name=extent_root_truncate_to_zero extent_root_slot_before=50240 extent_root_slot_after=50240 entry_after=tag0
```

原因：上段是否重写看「有单元重写、单元数变了、标签变了」三者任一（`crates/singlefs-core/src/truncate.rs:271-273` 起），标签 0 的文件三样都不变，整棵照抄。冻结副本自带的五条会话用例与三条读者用例全绿（`out/test-results.txt`）。

### T1.2 截到 0 与从 0 变长的边界；之后第一次写数据单元

- 截到 0 本身：条目仍是标签 0、size 0，三处读者读回（rerun-stdout 第 71 行 `entry=tag0 cold_walk=ok(len=0,all_zero=true)`）；再变长到三个净荷（第 72 行）：读回 97902 个 0，条目仍是标签 0。
- 第一次写数据单元：覆盖写 100 字节 → 标签 2、1 个单元；顺序写三个净荷又 7 字节 → 标签 1、4 个单元；先缩到 100、变长到五个净荷、再覆盖写 50 字节 → 标签 2；截到 0 之后再覆盖写 → 标签 2。三处读者都读回；池级 checker 的红只有手造镜像本来就有的三条（`name=overwrite_inode_one_with_100_bytes` 等行的 `checker_violated=["I-3.1", "I-3.11", "I-3.15"]`，手造时拿掉映射条目所致；基线行 `name=baseline` 另多一条 I-1.13）。

### T1.3 打中：截到 0 之后只要重写上段叶，写出的那一版冷读与可写重挂都拒

序列：标签 0 镜像 → `Truncate` 到 0 → `NewInodes` 1 个 → `OverwriteOfAnInode` 号 2（上段叶要重写）。原样输出（rerun-stdout 第 73 行）：

```
PROBE name=truncate_to_zero_then_new_inode_and_write_inode_two session=ok entry=no_entry cold_walk=ERR(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" }) writable_remount=ERR(Recovery(InvariantViolated { invariant: "E142 走读同款", detail: "这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符" })) mounted_read=ok(size=0,units=0,read_len=0,all_zero=true) checker_checked=108 checker_violated=["I-3.1", "I-3.11", "I-1.13", "I-3.15"]
```

同类的第二条（第 75 行）：建到第一片上段叶之外的号 143 再写它，上段长高，同样 `entry=no_entry`、冷走读与可写重挂 ERR。拆开比：只截到 0（第 71 行）、截到 0 再 `NewInodes`（叶不重写）、截到 0 两次、截到 0 再覆盖写号 1，都读得回；先重写上段叶（号 2）再截到 0（`name=split_d_new_inode_write_inode_two_then_truncate_to_zero`，第 66 行）也读得回、条目仍是标签 0。只有「先截到 0、后重写叶」这个次序出事。

病根（读代码）：截到 0 那一版叶整棵照抄，条目仍是标签 0、size 0，所有读者收。之后某次发布重写这片叶时，第一个文件那一条经 `of_a_file_without_data_units(0)` 取 `None`（`crates/singlefs-core/src/extent_tree.rs:413`：「if size_in_bytes > 0 {」；文档行 `crates/singlefs-core/src/extent_tree.rs:409`：「size > 0 写标签 0，size 0 不写条目」），条目被丢掉。丢掉之后「第一个文件有 inode 记录、size 0、上段没有它的条目」这一形，三处读者答案不一：

- 冷走读与可写重挂（`recovery.rs`）：没有第一个文件那条上段条目时按标签 1、2 收（`crates/singlefs-core/src/recovery.rs:5968`：「TailOfAFile::NoHoleAfterTheLastDataUnit,」），单元数 0 少于 `max(⌈0 ÷ 净荷⌉, 1) = 1`，拒（`crates/singlefs-core/src/recovery.rs:6681`：「TailOfAFile::NoHoleAfterTheLastDataUnit => DataUnitsOfTheInode::SomeWritten,」，判在 `fn data_unit_count_matches_the_inode_size`）；
- 挂载态读者：没有条目收成 NoneWritten（`crates/singlefs-core/src/mounted_read.rs:809`：「None => DataUnitsOfTheInode::NoneWritten,」），size 0 且 0 个单元收（`crates/singlefs-core/src/write_request_split.rs:103`：「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {」）；
- 池级 checker：标签 0 或没有条目都算一个单元都没有（`crates/singlefs-checker/src/walk.rs:12425`：「(None | Some(DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (」），不红：上面输出的 `checker_violated` 与基线行相同。

`extent_tree.rs` 那段文档（第 409–410 行）说它与 `publish_new_inodes` 建出来的空 inode 同一形、读者按没有条目收：对号 ≥ 2 的空 inode 成立，对第一个文件不成立（冷读那一路只给「第一个文件删完」留了没有条目的口子，`crates/singlefs-core/src/recovery.rs:5964`：「let Some(extents) = extents_of_the_first_file_on_disk else {」）。

第二处表现，同一病根：标签 0 镜像 → 截到 0 → `Unlink` 号 1，删完之后上段叶里第一个文件那一条还在；不截到 0 直接删则没有。原样（rerun-stdout 第 67、68 行）：

```
PROBE name=split_e_truncate_to_zero_then_unlink_read_content session=ok entry=tag0 cold_walk=ok(no_file) writable_remount=ok(not_with_file) mounted_read=open_ERR(NoSuchInode { inode: InodeNumber(1) }) checker_checked=108 checker_violated=["I-3.1", "I-3.11", "I-3.15"]
PROBE name=split_e_unlink_without_truncate_read_content session=ok entry=no_entry cold_walk=ok(no_file) writable_remount=ok(not_with_file) mounted_read=open_ERR(NoSuchInode { inode: InodeNumber(1) }) checker_checked=108 checker_violated=["I-3.1", "I-3.11", "I-3.15"]
```

删除计划按 `crates/singlefs-core/src/transaction.rs:4000` 的 `pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {` 给的号集合（size 0 时不含号 1）算要不要换下叶，而叶里实有那一条。读者不拒、checker 也不红（两行 `checker_violated` 相同）：是一条遗留的、没人判的条目。没造出它会让后续发布出错的序列（删完之后写号 2 被另一条规则拒，探针 `name=split_f_…` 为证），所以只记「遗留条目」，不记「会坏」。

### T1.4 对照：改一处就都读得回

草稿副本里只改 `of_a_file_without_data_units` 一处（size 0 也返回 `Some(NoDataUnit)`；`tools/rerun.sh` 第 5 步，冻结副本不动），同样的探针：两条读不回的序列变成三处读者都读回、条目仍是标签 0（rerun-stdout 第 42、44 行）；删除序列里遗留条目没有了（第 36 行 `entry=no_entry`）；冻结副本自带的八条标签 0 用例与探针全绿（`variant_*` 三个目标 ok，`out/test-results.txt`）。原样（第 42 行）：

```
PROBE variant=size_zero_also_tag0 name=truncate_to_zero_then_new_inode_and_write_inode_two session=ok entry=tag0 cold_walk=ok(len=0,all_zero=true) writable_remount=ok(size=Some(0),units=0) mounted_read=ok(size=0,units=0,read_len=0,all_zero=true) checker_checked=108 checker_violated=["I-3.1", "I-3.11", "I-1.13", "I-3.15"]
```

这是一个证据，不是我替主 agent 定改法。条款字面也是这么读：`.claude/kb/decisions/08-核心索引结构.md:451` 写的是「标签留 0」；S1 的判法 `research/prompts/m4-trunc-r1-main-verification.md:82`：「标签 0 是这个 inode 一个单元都没有（size 0 的空 inode，或 size > 0 的整个文件都是洞）」，也把标签 0 配 size 0 算合法。

### T1.5 替被判出局的候选辩护：同样打中吗

- 子（`publish_size_change` 在取 txg 之前拒）：这一格打不中——截到 0 在改任何东西之前被拒，没有「截到 0 的标签 0」这个中间态。代价（推的，没量）：标签 0 的文件只能覆盖写，不能缩、不能变长。子「不动条款」的话，T1.3 之外的另一个病（调查员报的：重写上段叶丢掉标签 0 条目）仍在，要改 `(None, None)` 那一臂（`crates/singlefs-core/src/transaction.rs:12339`）；那个病不是丑特有的，三个候选里只有寅绕得开。
- 寅（读者不再收标签 0 配 size > 0）：这一格打不中；这样的池读不开，改的是读者与 K5、S1 的合法性口径，比丑大一圈（没量）。
- 结论：N = 0 这一格是「丑」当前实现的缺口，不同样打中子、寅，也不使「丑」不成立：一处判法改了就读得回（T1.4）。

推翻这一节的现象：① 在冻结副本原样跑出「标签 0 → 截到 0 → 重写上段叶」后冷读与可写重挂都读得回（rerun-stdout 第 73 行变成 ok）；② 主 agent 在入库装置上重做同一序列得到不同结果；③ 用户把「第一个文件 size 0 配没有条目」定为合法并改了冷读。

## T2　隔离那次发布置记号

判定：**站得住。**

条款（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:117`）：「写者隔离那次发布，把两块盘上那条分配记录的跨度段置 0x4000」，后接「分配代不变，与这次发布的别的改动一起落盘」。逐句对到代码：

| 条款的一句 | 代码 | 实测 |
|---|---|---|
| 跨度段置 0x4000 | 「pub const ALLOCATION_RECORD_QUARANTINE_MARK: u16 = 0x4000;」（`crates/singlefs-core/src/allocator.rs:37`）；落盘 `to_bytes` 把记号或进跨度段（`crates/singlefs-core/src/allocator.rs:94-104`） | 盘上读回跨度段 0x4002（rerun-stdout 第 76 行，下） |
| 两块盘都置 | 发布路径逐个释放的落点，把「核出对不上的那几份」所在的盘点名给 `release_leaving_the_record_allocated_on`（`crates/singlefs-core/src/transaction.rs:13264`：「for placement in release {」；`:13270` 调用）；这一串盘由 `copies_failing_the_release_checksum_check` 给出，口径是任一份对不上就交回两份（`crates/singlefs-core/src/transaction.rs:5118`：「任一份没对上交回两份」） | 只有盘 1 那一份改坏的覆盖写，两块盘的记录都带记号（`release_checksum_quarantine_mark_written_to_disk_and_cleared_by_the_rollback` 的第一条用例 ok） |
| 分配代不变、仍分配 | 点名的盘走 `if devices_whose_record_stays_allocated.contains(&device.device) {` 那一支：只改 `quarantine_mark`，随即 `continue`（`crates/singlefs-core/src/allocator.rs:2227`），没有碰 `generation`、`is_released`、位图 | 读回 `released=false,gen=3,span=2,mark=0x4000`（两盘各一条） |
| 与这次发布的别的改动一起落盘 | 分配记录树重写集的固定点用「照常释放」的拷贝试算（`crates/singlefs-core/src/transaction.rs:11773`：「rehearsal.release(*placement, plan.txg);」）；真发时同一片叶里同一条记录变成「带记号」而不是「已释放」，变的叶是同一批 | 同一次发布里两个单元（数据单元与 extent 树根）都被隔离：四份记录都落盘带记号（下） |

「只有一块盘读坏」：覆盖写 A 的数据单元，盘 1 那一份改坏、盘 0 完好，B 发布成功；盘 0 与盘 1 两条记录都读回 0x4000（上表第二行；用例 `quarantine_writes_the_mark_into_the_span_field_on_both_devices_and_keeps_the_generation` 钉的就是这一形，冻结副本里 ok）。

「同一次发布里隔离两个槽」（探针 `probe_two_units_quarantined_in_one_publish_then_rolled_back`：盘 1 上数据单元和 extent 树根节点两份都改坏，再覆盖写）。原样（rerun-stdout 第 78、76 行）：

```
PROBE name=two_units quarantined_copies=4 [(0, 50180), (1, 50180), (0, 50240), (1, 50240)]
PROBE name=two_units_after_quarantine data(slot 50180): [dev0:released=false,gen=3,span=2,mark=0x4000 dev1:released=false,gen=3,span=2,mark=0x4000] extent_root(slot 50240): [dev0:released=false,gen=3,span=1,mark=0x4000 dev1:released=false,gen=3,span=1,mark=0x4000] checker_checked=108 checker_violated=["I-2.1", "I-4.8", "I-7.4"] I-3.17=Holds
```

池级 checker 的三条红（I-2.1、I-4.8、I-7.4）是盘 1 上那两份被我改坏的拷贝本身所致（一致性校验和对不上），I-3.17（隔离记号与引用相符）成立；回退并写回原样之后 `checker_violated=[]`（第 77 行）。

「记号与已释放标志的组合只写得出允许的几种」：
- 允许的三种（未释放不带记号、未释放带记号、已释放不带记号）都写得出；已释放又带记号：落盘 `to_bytes` 断言拒（`crates/singlefs-core/src/allocator.rs:98`：「!self.is_released,」），读盘 `parse` 返回 `QuarantineMarkOnAReleasedRecord`（`crates/singlefs-core/src/allocator.rs:143`：「return Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord);」）。
- 内存里「已释放又带记号」能构造（`AllocationRecord` 的字段是 pub），只靠上面那条断言在落盘时拦；主 agent 已把这一条记为欠账（背景材料第二节）。生产路径里我枚举了每一处把 `is_released` 置真的地方：只有 `crates/singlefs-core/src/allocator.rs:2232`（`record.is_released = true;`，下一行 `:2233` 同时把记号清成 NotQuarantined），另外两处在测试夹具里（`crates/singlefs-core/src/region_scan.rs:477`、`crates/singlefs-core/src/allocation_record_tree.rs:1044`）。
- `AllocationRecord::parse` 对 0xC000 判坏，在核心层读者一侧没有条款逐字写它：D19 逐字写的是池级 checker 的「带记号的记录必须未释放」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:117`），核心层拒是实现员取的安全一侧，与 checker 一致，我记为「条款没写、实现取了保守的」。

替被判出局的候选辩护（E187 的另一个落点，乙 = 独立的隔离树，`.claude/kb/experiments/187-隔离落盘代价.md:39` 一节读数）：这一格的问题（两盘都置、分配代不变、一起落盘）换成乙是另一套：乙不改分配记录，隔离树自己要写（E187 的写量读数里乙第一次建树多 2 次写）。我没有乙的代码，找不到可比的实现；这一格的结论只对甲。

推翻这一节的现象：任一历史上出现「隔离那次发布之后，盘上读回的记录少一块盘带记号」、或「分配代变了」、或 `checker_violated` 里出现 I-3.17。

## T3　清记号：回退复活与释放

判定：**清记号（回退复活）站得住；「释放隔离记录时改写成已释放」这一句站得住，但今天没有活路径走到它，只有单测钉着，也没有被别的路径意外走到。**

条款（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:117`）：「回退复活那一次，同一次发布清掉两块盘上的 0x4000」与「释放隔离记录时改写成已释放，0x4000 随之清掉」。

### T3.1 回退清记号

代码：回退复活那一判里，现行账中「仍分配、同分配代、带记号」的记录，且回退目标引用它的，逐盘记进 `quarantine_marks_to_clear`（`crates/singlefs-core/src/mount.rs:9153`：「let mut quarantine_marks_to_clear = Vec::new();」，进清单的臂在 `:9211` 起）；这一臂排在「分配代不同就拒」之后（`crates/singlefs-core/src/mount.rs:9199`：「Some(record) if record.generation != target_generation => {」），所以带记号而分配代对不上的先被拒。真正的清发生在分配器上（`crates/singlefs-core/src/mount.rs:9780`：「for (device, placement) in &quarantine_marks_to_clear {」）。它在复活、释放之前，和它们同在「发布被拒时整个换回」的保护里：分配器先克隆（`crates/singlefs-core/src/mount.rs:9772`：「let allocator_before_the_rollback = allocator.clone();」），发布在落盘之前被拒就换回（`crates/singlefs-core/src/mount.rs:9854`：「*allocator = allocator_before_the_rollback;」）。

回退目标里本来没有那条隔离的：不清，对。探针 `probe_rollback_to_a_version_that_does_not_reference_the_quarantined_unit`（A → 盘 1 数据单元那份改坏 → B 隔离它 → C 再覆盖写；回退到 B，B 不引用被隔离的单元；再把盘 1 写回原样回退到 A，A 引用它）。原样（rerun-stdout 第 46、48、47 行）：

```
PROBE name=abc_after_c A_data(slot 50180): [dev0:released=false,gen=3,span=2,mark=0x4000 dev1:released=false,gen=3,span=2,mark=0x4000] checker_checked=108 checker_violated=["I-2.1", "I-4.8", "I-7.4"] I-3.17=Holds
PROBE name=abc_after_rollback_to_B rollback=ok A_data(slot 50180): [dev0:released=false,gen=3,span=2,mark=0x4000 dev1:released=false,gen=3,span=2,mark=0x4000] checker_checked=108 checker_violated=["I-2.1", "I-4.8", "I-7.4"] I-3.17=Holds
PROBE name=abc_after_rollback_to_A rollback=ok A_data(slot 50180): [dev0:released=false,gen=3,span=2,mark=0x0000 dev1:released=false,gen=3,span=2,mark=0x0000] checker_checked=108 checker_violated=[] I-3.17=NotApplicable("最新根的分配记录树里没有带隔离记号的记录")
```

回退到 B 记号留着、I-3.17 成立（没清错）；回退到 A 两块盘一起清、checker 全绿。单个回退同时清两个槽（数据单元与 extent 树根）：rerun-stdout 第 77 行，两个槽四条记录都 `mark=0x0000`、`checker_violated=[]`。

清错的后果与被拒：回退到 A 而盘 1 那份没写回原样，逐盘验拒（`IsolatedUnitCopyUnreadableOrMismatched`），分配器与盘上记号都不变（第 59 行）：

```
PROBE name=refused_rollback rollback=REFUSED IsolatedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(1), slot: SlotNumber(50180) } allocator_after=["dev0:Some((false, QuarantinedAfterTheReleaseChecksumCheck))", "dev1:Some((false, QuarantinedAfterTheReleaseChecksumCheck))"] A_data(slot 50180): [dev0:released=false,gen=3,span=2,mark=0x4000 dev1:released=false,gen=3,span=2,mark=0x4000] checker_checked=108 checker_violated=["I-2.1", "I-4.8", "I-7.4"] I-3.17=Holds
```

「清」若落在没有记号的记录上，`clear_the_quarantine_mark_of_a_record_the_rollback_target_references` 的断言拦（`crates/singlefs-core/src/allocator.rs:2308-2311`，调用方已逐盘判过，这条断言是第二道）。

### T3.2 释放隔离记录

释放一律走 `release_leaving_the_record_allocated_on`：不在点名盘里的盘上，记录改已释放、记号清成 NotQuarantined（`crates/singlefs-core/src/allocator.rs:2232`：「record.is_released = true;」，`:2233` 清记号）。谁会对一个带记号的记录调用它？我把核心层所有 `release(` 与 `release_leaving_the_record_allocated_on(` 的调用点列出来（命令与原样输出，工作目录 `crates/singlefs-core/src`）：

```
$ grep -n '\.release(\|release_leaving_the_record_allocated_on(' *.rs | grep -v 'allocator.rs:3'
allocator.rs:2197:        self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
allocator.rs:2206:    pub fn release_leaving_the_record_allocated_on(
mount.rs:9794:        allocator.release_leaving_the_record_allocated_on(
admission.rs:2249:        allocator.release(data, CheckpointTxg(2));
transaction.rs:1907:            rehearsal.release(*placement, txg);
transaction.rs:2059:        allocator.release(released, txg);
transaction.rs:11773:            rehearsal.release(*placement, plan.txg);
transaction.rs:13270:        allocator.release_leaving_the_record_allocated_on(
region_scan.rs:649:        pool.release(slot_in_region(12));
```

`admission.rs:2249` 与 `region_scan.rs:649` 在测试里；`transaction.rs:1907`、`:11773` 是固定点试算的克隆；`transaction.rs:2059` 释放的是树表 0 条那一版的实例表链与分配记录树节点（不是映射里的用户单元，不会被隔离）；真正放出去的两处是发布路径 `transaction.rs:13270`（`release` 列表来自「经映射释放的角色」，被隔离的单元映射条目已去掉，之后不会再出现在这张表里）和回退 `mount.rs:9794`（现行版引用着的单元）。整理的搬迁源来自映射里有条目的 Movable 单元，隔离的单元映射里没有条目，进不了搬迁。所以这一句今天没有活路径；单测钉着它（`crates/singlefs-core/src/allocator.rs` 的 `releasing_a_quarantined_record_rewrites_it_released_and_clears_the_mark`，核心层 allocator 单测 30 条全 ok，`out/test-results.txt`）。

整理 / 重挂之后的状态（欠账，不是这一批新增）：分配器重建时记录整份照抄，记号保留（`crates/singlefs-core/src/allocator.rs:2093`：「allocator.records = records;」），可写重挂之后那两条仍带记号（用例 `quarantine_writes_the_mark_…` 的后半钉着）。但整理一侧的「常驻隔离集」是一次挂载内的内存集合，起步为空（`crates/singlefs-core/src/compaction_plan.rs:630`：「quarantined_slots: BTreeSet::new(),」），加进去的唯一入口是发布路径当次记下的隔离（`crates/singlefs-core/src/compaction_plan.rs:649`：「self.quarantined_slots.insert(SlotNumber(covered));」）。重挂之后区域扫描把一条带记号、没有映射指着的记录归成 Dead（`crates/singlefs-core/src/region_scan.rs:313`：「return AllocationInRegion::Dead;」），Dead 在第一版只报告，不释放、不搬（`crates/singlefs-core/src/region_scan.rs:104-109`）。所以「释放隔离记录」没有被整理意外走到，只是整理不认记号；主 agent 的那五条已判里「区域扫描不认记号」就是它，这一批没修（记号现在盘上有了，起种在技术上可行，但不在这一批的范围里）。

替被判出局的候选辩护（E187 乙，隔离树）：回退清记号对应「回退时删隔离树里的条目」，同一个回退入口、同一处「在任何写之前逐盘判」的位置，要补的代码形状一样；常驻隔离集不起种这一条，乙同样中（乙要从树里读，甲从已经读进来的分配记录里取，乙还多一次读树）——这条是推的，没有乙的代码可量。

推翻这一节的现象：回退到引用被隔离单元的版本之后，盘上读回仍有 0x4000（探针第 47 行变样）；或回退到不引用它的版本之后记号被清（第 48 行变样）；或在核心层找到第三处把带记号的记录改写成已释放而不清记号的调用点。

## T4　读者认记号

判定：**站得住**（核心层只有一处解码，且它去掉了记号）；两处现状没动（区域扫描不认记号、旧读者不带位）是已记欠账与 E187 已写的取舍，见下与 T5。

条款：D3 的跨度段那一行（`.claude/kb/decisions/03-空间分配.md:131`）：「次高位 0x4000 借作隔离记号」；`.claude/kb/layout/01-first-txn.md:299`：「次高位 0x4000 是隔离记号」。

核心层读跨度的每一处：核心层里把盘上 20 字节分配记录解成字段的只有一处，`AllocationRecord::parse`（`crates/singlefs-core/src/allocator.rs:126`：「pub fn parse(bytes: &[u8]) -> Result<Self, AllocationRecordEntryRefusal> {」），去掉两个标志位用的掩码在 `crates/singlefs-core/src/allocator.rs:40`：「const ALLOCATION_RECORD_SPAN_VALUE_BITS: u16 =」；它的唯一生产调用点是读分配记录树叶（`crates/singlefs-core/src/allocation_record_tree.rs:731`：「let record = AllocationRecord::parse(entry).map_err(|refusal| match refusal {」），之后的落叶判（`crates/singlefs-core/src/allocation_record_tree.rs:275`：「pub fn record_fits_in_its_leaf(&self, record: &AllocationRecord) -> bool {」）用的是去掉记号的 `span_slots`。我对全仓核心层、checker、harness 的 `src/` 里 `0x8000` / `ALLOCATION_RECORD_RELEASED_FLAG` / `AllocationRecord::parse` 都扫过（命令 `grep -rn 'ALLOCATION_RECORD_RELEASED_FLAG\|0x8000\|AllocationRecord::parse' --include=*.rs`，排除 `tests/`）：核心层除上面一处没有第二个解码点；池级 checker 自己有一份（`crates/singlefs-checker/src/walk.rs:5517` 一带掩码同样同时去掉两位，不在这一批的 diff 里）；checker 档的 E171 两个装置里还有改分配记录跨度的辅助，只保留 0x8000 一位、不认 0x4000（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:3669`：「(span_field & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots,」），是实验装置里的改坏工具、不是读者，也不在这一批，只当对照记下。

「读大 16384 倍」的反证：冻结副本自带的单测钉着落叶判按低位跨度：`records_with_the_quarantine_mark_fit_in_their_leaf_by_the_span_in_the_low_bits`（核心层 `allocation_record_tree::tests` 7 条全 ok）与 `records_read_with_the_quarantine_mark_are_judged_by_the_span_in_the_low_bits`（`allocation_record_pool_geometry_tests` 6 条全 ok）。0xC000：`parse` 返回 `QuarantineMarkOnAReleasedRecord`，树读者映射成 `UnitMalformed`（`crates/singlefs-core/src/allocation_record_tree.rs:739`：「AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord => {」），整棵树读不回。

区域扫描、整理计划、恢复三处对带记号的记录：
- 恢复：解码时认记号、跨度取低位；重建分配器时整份保留（上节）。
- 区域扫描：只看 `!is_released`（`crates/singlefs-core/src/region_scan.rs:219`：「.filter(|record| record.device == device && !record.is_released)」），带记号的记录与不带的一样算「未释放」；分类不看记号，只看运行时隔离集、映射、读头（见 T3.2），带记号而隔离集里没有的（重挂之后）归 Dead，只报告。
- 整理计划：同样只按 `!is_released` 过滤（`crates/singlefs-core/src/compaction_plan.rs:117`）；`Dead` 只落到「R 里还有东西搬不动」（`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`）。
- 与「安全一侧」的判法：三处都把带记号的记录当「已分配、不动」，不会放出隔离着的槽，不会把它搬走；唯一放过的是整理对「已隔离」的认知在重挂后丢了，只会让整理多报几次「搬不动」，不会让它动带记号的槽。这是安全一侧。没修的原因：主 agent 已判「区域扫描不认记号」记欠账（背景材料第二节），要用记号起种常驻隔离集，是新功能，不是这一批的约。

推翻这一节的现象：找到核心层另一处读 20 字节分配记录而不经 `AllocationRecord::parse` 的解码；或一条带 0x4000 的记录在核心层读出来的 `span_slots` 不是低位值（单测与上面两条探针会红）。

## T5　第一轮的改法本身与从没被看过的地方

判定：「丑」的完成度见 T1（N > 0 做到，N = 0 打中）；第一轮判决打中的几处在这一批之后：甲 修了（N = 0 一格换了形态）；乙与 T10 没修，不是因为岔路还开着，而是用户已定这一版不改；不带位的旧读者：拒的层级与 D19 / E187 写的有一处对不上。

### T5.1 第一轮打中的几处，这一批之后各是什么状态

| r1 打中的 | 这一批之后 | 依据 |
|---|---|---|
| T5 甲：标签 0 配 size > 0，三种 N 在 `truncate.rs` 两处 panic | 修了：两处断言的前提改了（`crates/singlefs-core/src/truncate.rs:141`、`:240`），三种 N > 0 做成。**N = 0 一格换了形态**：不再 panic，但「截到 0、再重写上段叶」写出冷读与可写重挂都拒的版本（T1.3） | rerun-stdout 第 73、75 行 |
| T5 甲附带：重写上段叶丢掉标签 0 条目，写出的那一版三处读者都拒 | 修了（size > 0）：`(None, None)` 一臂照写标签 0（`crates/singlefs-core/src/transaction.rs:12339`）；size 0 仍丢（同上） | 冻结副本里 `a_whole_file_hole_of_tag_zero_keeps_its_entry_when_the_session_rewrites_its_upper_leaf_for_inode_two` 等两条 ok（`out/test-results.txt`） |
| T5 乙：合法的大 size 让冷读按 size 整份分配 | **没修**。`crates/singlefs-core/src/recovery.rs:6701`：「content.resize(」仍在 | 用户已定：判决 `research/prompts/m4-validate-code-r1-main-verification.md:73` 写「用户 2026-10-22 定这一版不改，照现在的方式读，放到后面的优化里程碑」；欠账 `.claude/kb/checks-owed.md:557` 的 C878 题目是「冷读与 RefFS 按 size 整份分配内存」 |
| T10：RefFS 把截断施加成整份 `resize` | **没修**，同一条用户已定（判决第四节第二行同时管 T5 乙与 T10）。`crates/singlefs-reference-filesystem/src/lib.rs:642` 的 `truncated.resize(` 仍在，文件与主工作区逐字节相同，不在这一批的 diff 里 | 判决 `research/prompts/m4-validate-code-r1-main-verification.md:29`：「RefFS 把截断施加成整份 resize，截到 1 TiB 中止」 |

派发提示与背景材料把第四节第二条岔路写成「还开着」；判决文件自己「用户定案」一节写的是用户 2026-10-22 已定这一版不改、放到优化里程碑。没修的原因是这个，不是「岔路还开着」。

### T5.2 这一批让「大 size 整份分配」更容易打中吗

没有。变长到 2⁴⁰ 字节的路径在这一批之前对普通文件就通：普通 3000 字节文件变长到 2⁴⁰，会话做成、可写重挂与挂载态读者都读回 size（rerun-stdout 第 56 行，第一行）；标签 0 的文件现在也做成（第二行，第 70 行；这一批之前这一格 panic）：

```
PROBE name=ordinary_3000_byte_file_extend_to_2_pow_40 session=ok entry=tag4 cold_walk=skipped writable_remount=ok(size=Some(1099511627776),units=1) mounted_read=ok(size=1099511627776,units=1) checker_checked=108 checker_violated=[]
PROBE name=tag_zero_extend_to_2_pow_40 session=ok entry=tag0 cold_walk=skipped writable_remount=ok(size=Some(1099511627776),units=0) mounted_read=ok(size=1099511627776,units=0) checker_checked=108 checker_violated=["I-3.1", "I-3.11", "I-1.13", "I-3.15"]
```

新增的只是「标签 0 的文件也能被写者变长」，而标签 0 的文件今天写路径写不出（约 5800 次发布里 0 次，判决第四节用户定案一段引的调查员数），要先有一个外来镜像；所以「让冷读多一条按 size 整份分配的入口」的量没变。冷走读我没跑（跑就是分配 1 TiB，探针 `cold_walk=skipped`）。

### T5.3 不带位的旧读者遇到带记号的池，在哪一层拒、拒得对不对

先说写者这一侧：这一批没有给记号配 incompat 位。`crates/singlefs-core/src/system_configuration.rs:27` 起登记的位只有 0x01（退役）、0x02、0x04（截断字段），没有隔离记号的位；这一批的 diff 也不含 `system_configuration.rs`（`research/prompts/_m4-validate-code-r2-diff.md` 的文件清单）。落点甲「不带位」是 E187 与用户定的取舍（D19 条款 `.claude/kb/decisions/19-块指针的结构与宽度预算.md:119`：「不认记号的旧读者读到 0x4001、0x4002 都过不去——分配记录树读者拒整棵树（挂载与恢复失败）」）。

实测：用 `git archive HEAD` 导出的旧代码（不认 0x4000；`out/rerun-stdout.txt` 里 `name=old_reader_code head=841b3beabb9812ceaafcd636888b1201dd6e5adf`），读新写者倒出来的两份镜像——带记号的，与没隔离的对照。原样（`out/old-reader-probe-lines.txt`，两行）：

```
PROBE name=old_reader image=control mount_read_only=ok mount_writable=REFUSED NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi cold_walk=ok(file_bytes=Some(2700)) allocation_record_tree=ok(52 records) checker_checked=103 checker_violated=["I-2.4"]
PROBE name=old_reader image=marked mount_read_only=ok mount_writable=REFUSED NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(4) }, wi cold_walk=REFUSED AllocationRecordOutsideThePoolGeometry { what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽" } allocation_record_tree=REFUSED AllocationRecordOutsideThePoolGeometry { what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽" } checker_checked=103 checker_violated=["I-1.1", "I-2.1", "I-2.4", "I-3.11", "I-4.8", "I-5.4", "I-7.4", "I-3.16"]
```

读法：
- 冷走读与分配记录树读者：在分配记录树读叶那一步拒，原因 `AllocationRecordOutsideThePoolGeometry`（带记号的记录按 16385 / 16386 算越过叶），不是「不认识的特性」；对照镜像读得回。对得上 D19 与 E187 写的「分配记录树读者拒整棵树」。
- 旧池级 checker：对照只有 I-2.4（对照与带记号共有，不归记号）；带记号的多出 I-1.1、I-2.1、I-3.11、I-4.8、I-5.4、I-7.4、I-3.16，含 E187 写的 I-1.1（`.claude/kb/experiments/187-隔离落盘代价.md:16`：「池级 checker 判 I-1.1（块头自述逻辑地址）红」）。
- **只读挂载没拒**：`mount_read_only=ok`，带记号的与对照一样。D19 写的「（挂载与恢复失败）」对只读挂载这一条路径不成立；E187 的旧读者表（`.claude/kb/experiments/187-隔离落盘代价.md` 的「落点甲的旧读者」一节）五处读码里没有只读挂载这一条路径，这一格 E187 没量过。旧读者的只读挂载不读分配记录树，就不看这些记录；它读用户可见的树，而被隔离的单元已经不在树里，读到的文件内容不受影响。这是「安全但没拒」，不是「放过了坏东西」；我没读文件内容（没造这一步），所以不写成「读对了」。
- 可写挂载一列不可用：对照镜像也被拒（同一个 `NewerStateStillUnreadableAfterOneReread`），原因不明，多半是我把镜像按扇区倒出再读的人为产物（倒出的镜像缺了什么见证），不能归因于记号，不引。

「拒得对」：拒在读分配记录树这一层，是封闭式失败（不会按错的跨度往下用），但用的是几何失败的通用错误，不是「遇到不认识的盘上记号」；没有 feature 位把它说清楚。这是用户选「不带位」的后果，条款里已经写了。我不替主 agent 判它够不够。

替被判出局的候选辩护（E187 乙，隔离树）：乙要在树表里多一条种类，旧读者遇到不认识的树种类会怎样——E187 只量了甲的旧读者（五处读码都是甲），乙的旧读者表在 E187 里找不到；我也没有乙的代码，量不了。结论只对甲。

### T5.4 第二轮分工：产生结论的方法 / 从未被看过的地方

- 方法：第一轮判「标签 0 能不能改 size」靠的是单张手造镜像上的三条改动（3000 → 100、3000 → 3000、3000 → 三个净荷）和「重写上段叶」两条；全是 size > 0。没有一条走过 size 0 的边，也没有一条走过「先改 size、再重写叶」的次序，所以 T1.3 没被看到。
- 从没被看过的地方：① 旧读者的只读挂载（上节）；② 池级 checker 对「标签 0 配 size 0 的条目有没有、读者之间答不答得一样」没有一格能红（T1.3 的两条输出里 `checker_violated` 与基线相同）：同一形态上冷读拒、挂载态读者收、checker 不红，三者不一致没人判。这两处都不是这一批引入的，但这一批让写者第一次能走到第二处。

推翻这一节的现象：旧代码读带记号的镜像时 `mount_read_only` 拒（上节第三条变样）；或在冻结副本原样（不加我的探针）下标签 0 → 截到 0 → 重写叶读得回（T1.3 变样）。

## T6　测试临时目录守卫

判定：**站得住。**

守卫（冻结副本 `crates/singlefs-core/src/block_device.rs`，`#[cfg(test)]` 模块）：`temporary_image_path` 现在交回认领着路径的守卫（`crates/singlefs-core/src/block_device.rs:1025`：「fn temporary_image_path(tag: &str) -> TemporaryPathRemovedOnDrop {」，`:1036`：「struct TemporaryPathRemovedOnDrop(PathBuf);」）；Drop 里先 `symlink_metadata` 判类型：不在就算了、目录 `remove_dir_all`、其余 `remove_file`（`crates/singlefs-core/src/block_device.rs:1057`：「Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(&self.0),」）；符号链接不跟（`symlink_metadata` 对链接给的是链接本身，`is_dir()` 假，走 `remove_file` 删链接），删不掉只打一行、不在 Drop 里 panic。

只核守卫、不改被测路径、测试照旧判同一件事：
- 被改的地方：每个用到镜像的用例删掉末尾的 `std::fs::remove_file(&path).expect("清理镜像")`（或手工 `remove_dir_all`）并让 `path` 成为守卫；断言一行没动。守卫实现 `Deref<Target = Path>` 与 `AsRef<Path>`，传给被测函数的仍是路径，被测函数的签名没动；改动里被测函数（`FileBackedBlockDevice::create_image_file_exclusively`、`open_existing_image_file`、`DirectInputOutputBlockDevice::open_existing` 等）一行没碰。有意留着的一处显式删除（先删再重开看报什么，`crates/singlefs-core/src/block_device.rs:1776`：「std::fs::remove_file(&path).expect("清理镜像");」）守卫 Drop 遇到 `NotFound` 当没事。
- 25 条 `block_device::tests` 全 ok（`out/test-results.txt`）。

守卫真的清掉：核心层 block_device 测试在调用方自己设的 `TMPDIR` 下跑完剩几份（`tools/guard_leftovers.sh`；注意内存包装在调用方没设 `TMPDIR` 时给命令一个它自己的临时目录、退出时删，所以必须自己设，否则「没剩」是包装擦的，不是守卫）。原样（`out/guard-leftovers-stdout.txt`）：

```
name=leftovers tree=intact rc=0 test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 320 filtered out; finished in 4.77s files_left_in_TMPDIR=0
name=leftovers tree=mutant rc=101 test result: FAILED. 23 passed; 2 failed; 0 ignored; 0 measured; 320 filtered out; finished in 2.02s files_left_in_TMPDIR=25
```

没弄坏的树 0 份；把守卫里「删文件」一支弄成不删（mutant）剩 25 份镜像（名单在 `out/guard-leftovers-mutant-files.txt`）。同一棵树上这个测量能测出差别，说明「0」不是测量本身的假象。

两条新用例的判别力（`tools/mutate_guard.sh`，原样 `out/mutate-guard-stdout.txt`）：弄坏「目录那一支」→ `a_temporary_image_and_directory_are_gone_once_the_scope_ends_normally` 红、另一条 ok；弄坏「删文件那一支」→ 两条都红。冻结副本的 `crates/mutations.tsv` 里这一批的判别力行还没有（行在补丁目录 `~/.local/share/singlefs/work/m4-tests-delete-temp-dirs/patch/mutations-append.tsv`，用例文档里写的「判别力（`crates/mutations.tsv`）」指的是它落地之后），主 agent 落补丁时要连 `mutations-append.tsv` 一起落。

说明：之前本机 `/tmp` 里有 24 份 2026-10-22 日期的 `singlefs-core-*.img` 是这一批之前的泄漏（日期早于今天，我这一趟没有新增）。不归这一批，不清。

推翻这一节的现象：弄坏守卫之后上面两条用例不红；或没弄坏的树在自设 `TMPDIR` 下剩非零份。

## 逐格判定表

| 格 | 辩方判定 | 证据 |
|---|---|---|
| T1 三种改 size（N > 0）、之后第一次写数据单元 | 站得住 | `crates/singlefs-core/src/truncate.rs:141`、`:240`、`crates/singlefs-core/src/transaction.rs:12339`；extent 树根前后不变 rerun-stdout 第 51–54 行；写数据单元换标签 第 71–72 行与 T1.2 |
| T1 N = 0（标签 0 的文件截到 0）之后重写上段叶 | **够不着用户定的「丑」**（打中）；**不同样打中**子、寅 | rerun-stdout 第 73、75 行（冷读与可写重挂拒）；第 67 行（遗留条目）；病根 `crates/singlefs-core/src/extent_tree.rs:413`；草稿副本改一处即读回，第 42、44、36 行（T1.4） |
| T2 隔离那次发布置记号 | 站得住 | `crates/singlefs-core/src/allocator.rs:2227`；rerun-stdout 第 76、78 行（四条记录落盘带记号）；E187 乙没有可比的代码，只对甲 |
| T3 回退清记号 | 站得住 | rerun-stdout 第 46–48、59、77 行；`crates/singlefs-core/src/mount.rs:9780`、`:9854` |
| T3 释放隔离记录时改写成已释放 | 站得住，但今天没有活路径走到，只有单测钉着；没有被整理 / 扫描重建意外走到 | 调用点清单（T3.2）；`crates/singlefs-core/src/allocator.rs:2232` |
| T4 读者认记号 | 站得住（核心层唯一解码点去掉了记号）；区域扫描 / 常驻隔离集不起种是已记欠账、这一批没修 | `crates/singlefs-core/src/allocator.rs:126`、`crates/singlefs-core/src/allocation_record_tree.rs:731`；`crates/singlefs-core/src/compaction_plan.rs:630`、`crates/singlefs-core/src/region_scan.rs:313` |
| T5 甲（标签 0 改 size panic） | 修了；N = 0 一格换成「写出读不回的版本」 | T5.1、T1.3 |
| T5 乙 / T10（大 size 整份分配） | 没修；用户已定这一版不改；这一批没让它更容易打中 | `research/prompts/m4-validate-code-r1-main-verification.md:73`；rerun-stdout 第 56、70 行；`crates/singlefs-core/src/recovery.rs:6701` |
| T5 不带位的旧读者 | 分配记录树读者 / 冷走读拒（对得上条款），**只读挂载没拒**（条款写「挂载失败」对这一路不成立）；可写挂载一列不可用 | `out/old-reader-probe-lines.txt` |
| T6 测试临时目录守卫 | 站得住 | `crates/singlefs-core/src/block_device.rs:1057`；`out/guard-leftovers-stdout.txt`、`out/mutate-guard-stdout.txt` |

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- 没读禁读清单里的攻方腿报告与模型目录。
- 没动冻结副本、主工作区的任何文件，没做 git 写操作（`git archive HEAD` 与 `git rev-parse` 是读）。
- 没跑门禁阶段、checker 档测试、GPU、全量 `cargo test`；没跑 `tools/rerun.sh` 的模板 `--selftest`（步骤换了，模板自证断言的是模板自带那一段）。
- 腿的数都是草稿副本上量的（`rerun.sh` 拷出来的树、对照副本、弄坏守卫的树）；T1.3 的两条序列、T5.3 的旧读者读数，要在判决里引，按规则要在入库装置上重做一次（`.claude/rules/three-way-inference.md` 的「一条腿在副本装置上量出来的数，要在入库装置上重做一次才能引」），我没替它做。
- 没跑冷走读读 2⁴⁰ 字节的文件（会分配 1 TiB，探针 `cold_walk=skipped`）；没读旧读者读到的文件内容；旧代码的可写挂载一列因对照也拒而不可用。
- 子、寅、E187 乙的辩护部分是读代码与读 E187 产物推的，没有这几个候选的代码可量（报告里标了「推的」「没量」的几处）。
- 池级 checker 之外，GPU 判器与 checker 档的 E171 装置对记号的读法不在这一批的 diff 里，没核。
- 没改 `crates/mutations.tsv`、没落任何补丁。
- 重型测试：没跑。

## 要主 agent 跑的命令

无必须的。若要引 T1.3：把 `probes/tag_zero_probe_tail.rs` 里 `probe_truncate_to_zero_then_rewrite_the_upper_leaf_for_inode_two` 那一条序列（`Truncate` 到 0 → `NewInodes` 1 个 → `OverwriteOfAnInode` 号 2）做成入库装置的一次跑，核对 `entry=no_entry`、冷走读与可写重挂拒。

## 交回前删掉的草稿（删前 `du -sh`）

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-validate-code-r2-sonnet/` 下：`target`（共享编译目录）6.5G、`run7`（`rerun.sh` 拷出的树与输出）113M、`mutg`（弄坏守卫的两棵树）76M、`leftovers`（自设 TMPDIR 的两次跑）28M，加几个几 KB 的文本，合计 6.7G，全部删掉，不留给以后复用；报告里需要的输出都已存进 `research/prompts/m4-validate-code-r2-sonnet-model/out/`。
