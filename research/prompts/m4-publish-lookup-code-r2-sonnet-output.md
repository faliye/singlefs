# m4-publish-lookup-code-r2 云端正推腿报告（接续上一条腿）（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

立场：云端正推（Sonnet），核「代码做的是不是第一轮判决第三节要的那件事」。路径一律是冻结副本 `~/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo/` 里的相对路径（`crates/...`）。第一轮判决 `research/prompts/m4-publish-lookup-code-r1-main-verification.md`。方案按 `crates/` 今天的实现来谈。

## 一、复跑方法与产物

模型目录 `research/prompts/m4-publish-lookup-code-r2-sonnet-model/`。复跑命令（仓根 `/home/fy5090/code/singlefs`，`--work` 给新目录；`RERUN_STEPS` 选步骤，默认 `baseline,append,replacements,variants,t3diff`；`RERUN_TARGET_DIR` 可指已有编译目录省去整仓重编）：

```
RERUN_STEPS=append bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-code-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo --work <新草稿目录> --model research/prompts/m4-publish-lookup-code-r2-sonnet-model
```

我实际核过能跑通的两步：`append`（五行追加变异，3 分 14 秒，判定行与 panic 行同 `out/mutations-append.out`，只差线程号与耗时）、`t3diff`（6 分 38 秒，打出 `name=t3diff case_lines_pre_vs_post=identical cases=8`）。`baseline`、`replacements`、`variants` 三步在草稿目录里直接跑过（`tools/mutate-check.py`、`tools/run-in-tree.sh`），没有经 `rerun.sh` 再跑一遍；`replacements` 每行 4–6 分钟，`variants` 三行各 2–4 分钟，合起来超过一次前台调用的上限，所以 `rerun.sh` 里这三步只写了没整条跑。`RERUN_STEPS` 的拆法是我拆的，不是模板自带的。

「去掉产品断言再跑第五行」（第四节 T3）是手工做的：把 `crates/singlefs-core/src/transaction.rs` 里 `assert!(previous_entry.is_none(), "一个单元只被杀一次…")` 整块换成 `let _ = previous_entry;`，再跑 `mutations-append.tsv` 第 5 行，跑完用冻结副本的原文件覆回。这一步没进 `rerun.sh`。

每个文件的 sha256（模型目录内相对路径）：

```
e86342780dff3939e0a1fb25d33ece8bc5f3bc0cbac9a34fae61c3322566395b  mutations-append.tsv
3ea543a405e60eeb04aab92ddcd9ce3343c51cafa24fb1991aaa5f9f90d8f462  mutations-replacements.tsv
c6ca482e7aa8891e61e8ead49f4e8e5ddeb970de17879c38f54eca244ae22bf6  variants.tsv
3e256090b78e97513c0c65b2963a044e61d34846818ccdef318b14c31af88963  tools/diff_t3.rs
89ae0ed003827322708bdde8021bc5397cd35db2da71a4ff41073516289cfa6f  tools/mutate-check.py
b6bd493de52fa471ec3781fddd29c7d0225c0f474f5b0c373494ba2242e807eb  tools/rerun.sh
c840a72fc6aa2042b9d186548837047f6298efcae80e336e4aafc73f0af40621  tools/run-in-tree.sh
bc370cf4e4e784d54937cd7caca771f7dc6d060d7c90b8620d4d5429a732a9b6  out/baseline-post.log
b973f783f63e3e68225684aade97ad5cbd9a1e3ec501e480e59d74ee7d9ac622  out/mutations-append.out
62ec160f7ce3b53e6edb3b3bab96903fdc58e3d1e510b414d635bdbb3b08ca3e  out/append-logs/row1.log
25032d670e737774ef330273594a807920b2bf3b59ffabc974f564715bfa71bb  out/append-logs/row2.log
b0d00f99aeac384a47a8ac804afbf9e14e5add67255a785d5c99c9ac541dc774  out/append-logs/row3.log
9c053772b716df532ba87a363999670e684226b373519c488eca11ed3753325d  out/append-logs/row4.log
bb4fca66934f524cd60a4cc381e3ffafc952740b0f1a569911900fb05acc6e3e  out/append-logs/row5.log
5efeaecae88c140d4d659bd5cf8dfaad0a57b7aef98bfa00f92b7acdcfd83853  out/mutations-replacements-1.out
38ed03925062513938fce7530900deaa9ab5ad7330615c2b56208d53090db1cf  out/mutations-replacements-2.out
affb4c3d31202af6ddf1697df83ea8e932dfa36ead8db6e00a50cadea5737ac5  out/replacements-logs/row1.log
bf421d50a053c4cd4bce13f63e18d02140cb0354fc2f26ada07a00f2482d04ff  out/replacements-logs/row2.log
8408ea69a6726ee8ca21df0079f892a94cc93e7bb80b4e62e2cf35d9a55da6a3  out/row5-without-product-assert-logs/row1.log
5db1b72b54fa0a9b1990e384dba446917740969da7929852976cf1092f8e88c4  out/row5-without-product-assert.out
aac76cf85242a1f56cb14ed7ddeefe1c2d7fab733dffcd176f2b2cffce2184c0  out/t3-post.log
507256b6ad4cebd0b43d25a8ef0eea5a74deec4256f92c58fccb8837a655f25d  out/t3-post.T3DIFF.txt
c10e9611a6d27e06a6d351be429b519d24b0f44ebad145304a9996c4651b7918  out/t3-pre.log
d0eaece0e39e308a8b6b65fd4bd003de3fc9a2bd2d74f50bf299cf1931316599  out/t3-pre.T3DIFF.txt
5be6dcf2d5ff0654233cc1aacd277f113e25fe594f279567fb05de45db8d014a  out/variants-1.out
8065b86a2f1b53939221a8c55adcf8c90f7d6b1d0e2ad0be1fb2b911ef61f96d  out/variants-2.out
bbf7c4cb72a5acbbced9b278d6cf3443c7ff96fc2880f858aa9b9cffd40606c2  out/variants-3.out
409e357c5209412898bf4f1b2d91a28b2344b736c12274d6a7c9cb7aa2b702fe  out/variants-logs/row1.log
d527c05cb989ea872b09acf68a24ebd981769c483fcd28ce3b197272b15ad906  out/variants-logs/row2.log
936cfcde491326e1c6a4fb7d8033521a097fb25ef5fbd77ce8399d69af7d16b2  out/variants-logs/row3.log
```

## 二、接续现场（上一条腿留下的，核过）

交接摘要 `~/.local/share/singlefs/work/m4-publish-lookup-code-r2-sonnet/handover-1.md` 说上一条腿没写任何文件、没出报告。我核到的：

- 模型目录里的 `mutations-append.tsv`、`mutations-replacements.tsv` 与补丁原文逐字节相同：`cmp mutations-append.tsv <(sed -n 544,548p research/prompts/_m4-publish-lookup-code-r2-diff.md)`、`cmp mutations-replacements.tsv <(sed -n 554,555p …)` 都无输出，并且与冻结副本 `crates/mutations.tsv` 的 5063–5067 行、4973–4974 行逐字节相同。可接着用。
- `out/` 是空的，`tools/mutate-check.py` 能用但没有留日志；我加了 `--log-dir` 后全部重跑，上一条腿的任何读数我都没引。
- 草稿目录里 `post/` 的 `transaction.rs` 留在被变异的状态（`diff` 比冻结副本，第 3282 行是「增长：按身份查的索引退回逐项找」那一行的替换文本，上一条腿的变异脚本被停时没来得及还原）。我用冻结副本的原文件覆回，`diff -rq post/crates 冻结副本/crates` 无输出后才接着用。
- `pre/`（用补丁反向打回的改前副本）的源文件时间戳早于 `post/` 已编出的产物，与 `post/` 共用编译目录时 cargo 判「已是新的」，直接拿 `post/` 的产物当 `pre/` 的。我踩到了：第一次在共用编译目录里跑 `pre/` 的差分，panic 位置读出来是 `crates/singlefs-core/src/transaction.rs:3308`（改后的行号），与改前该有的行号对不上；touch 源文件、`pre/` 换独立编译目录（`target-pre`）重跑，才读到改前的 `7074`（见第四节）。这条已写进 `tools/rerun.sh` 的 `t3diff` 步骤（两侧分开编译目录并 touch）。
- 上一条腿的「去掉产品断言」变体没留下可信的输出（交接摘要里只有被截断的命令与一行 panic），我重做了，见第四节。

## 三、T1　公开出去的那一半是不是整理真正用的那一半

**判定：成立。** 产品里取「整理读一版的映射条目」只有一处；它的上游只有一条链，三个入口都汇进这条链；公开之后没有多出能绕开整理自己不变量的口子。另列两处不经它、但补丁前后都没变的取映射条目的地方（第 3、4 条），不算打中。

### 3.1　取映射条目的每一处（冻结副本）

1. 公开函数 `central_mapping_as_the_compaction_reads_it` 自己：「pub fn central_mapping_as_the_compaction_reads_it(」（`crates/singlefs-core/src/mounted_session.rs:2795`）；批量读者建在「let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);」（`crates/singlefs-core/src/mounted_session.rs:2799`），每把 key 经「.filter_map(|(_, key)| match central_mapping.locations_of_key(key) {」（`crates/singlefs-core/src/mounted_session.rs:2803`）查，留存的条目接在后面（「mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(」，`crates/singlefs-core/src/mounted_session.rs:2812`），解叶数取「mapping_leaves_decoded: central_mapping.leaves_decoded(),」（`crates/singlefs-core/src/mounted_session.rs:2820`）。
2. 它的唯一产品调用者是私有函数：「} = central_mapping_as_the_compaction_reads_it(version);」（`crates/singlefs-core/src/mounted_session.rs:2830`），私有函数头在「fn version_as_the_compaction_reads_it(」（`crates/singlefs-core/src/mounted_session.rs:2824`）。复核命令与输出：`grep -rnF 'central_mapping_as_the_compaction_reads_it(' crates` 在冻结副本里命中 `crates/singlefs-core/src/mounted_session.rs:2830`（调用）、`2795`（定义）和测试 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:397、507` 三处调用；`crates/` 下别的产品文件零命中。
3. 私有函数的唯一调用者：「let read_of_the_version = version_as_the_compaction_reads_it(&version);」（`crates/singlefs-core/src/mounted_session.rs:3018`），在 `advance_compaction_by_one_publish`（「fn advance_compaction_by_one_publish<Device: BlockDevice>(」，`crates/singlefs-core/src/mounted_session.rs:2981`）里。这一份读取同时供两件事：
   - 整理选段：「&read_of_the_version.mapping_entries,」（`crates/singlefs-core/src/mounted_session.rs:3024`）喂给 `scan_of`（闭包定义在 `crates/singlefs-core/src/mounted_session.rs:3020`），`scan_of` 三处调用：强制区域「match (is_disqualified, scan_of(forced_region, &*devices)) {」（`crates/singlefs-core/src/mounted_session.rs:3066`）、按水位选段「match scan_of(region, &*devices) {」（`crates/singlefs-core/src/mounted_session.rs:3099`）、对活意图的区域重扫（同一句，`crates/singlefs-core/src/mounted_session.rs:3186`）。钉住的槽「&read_of_the_version.pinned_slots,」（`crates/singlefs-core/src/mounted_session.rs:3025`）与让一段没资格的并集（`crates/singlefs-core/src/mounted_session.rs:3049-3050`）也取自它。
   - 搬单元的计划：码 1 数据单元落着的槽「&read_of_the_version.slots_holding_user_data,」（`crates/singlefs-core/src/mounted_session.rs:3123`）喂「签发前判洞够不够」；它在私有函数里由同一份 `mapping_entries` 算出（「for entry in &mapping_entries {」，`crates/singlefs-core/src/mounted_session.rs:2872`）。
4. `advance_compaction_by_one_publish` 的唯一调用者：「match advance_compaction_by_one_publish(」（`crates/singlefs-core/src/mounted_session.rs:2704`），在 `run_compaction_on_with_the_region_check_of_the_intent_deletion`（「fn run_compaction_on_with_the_region_check_of_the_intent_deletion<Device: BlockDevice>(」，`crates/singlefs-core/src/mounted_session.rs:2597`）里。整理的三个入口都进这个函数：
   - 用户改动之后的至多一批：「published.compaction_after_the_change = Some(run_compaction_on(」（`crates/singlefs-core/src/mounted_session.rs:1910`），经「pub(crate) fn run_compaction_on<Device: BlockDevice>(」（`crates/singlefs-core/src/mounted_session.rs:2570`）；
   - 显式整理一趟：「let mut run = run_compaction_on_with_the_region_check_of_the_intent_deletion(」（`crates/singlefs-core/src/mounted_session.rs:1984`）；
   - 可写挂载后续做活意图（恢复后第一次整理）：「Some(run_compaction_on(」（`crates/singlefs-core/src/mount.rs:6973`）。
   快照下的整理没有另一条入口：快照钉住的单元以留存映射条目的身份接在第 1 条的 `mapping_entries` 后面（`crates/singlefs-core/src/mounted_session.rs:2812`），搬的时候走同一份。

### 3.2　抽出前后私有函数产出是否逐项相同

用补丁反向打回的改前副本和冻结副本比 `mounted_session.rs`：`diff pre/crates/singlefs-core/src/mounted_session.rs 冻结副本/…` 只出三段，原样输出：

```
2782c2782,2795        （改前 2782 行 `fn version_as_the_compaction_reads_it(` 变成新增的结构体、文档与 `pub fn central_mapping_as_the_compaction_reads_it(` 头）
2784c2797             （返回类型 VersionAsTheCompactionReadsIt 变成 CentralMappingAsTheCompactionReadsIt）
2804a2818,2830        （函数尾新增返回结构体，并新增私有函数头与解构）
```

取映射条目的循环体（`mapped_units` 次序，查不到与条目窄于字段表的不收，`crates/singlefs-core/src/mounted_session.rs:2803`）、留存条目接在后面（`crates/singlefs-core/src/mounted_session.rs:2812`）、`pinned_slots` 与 `slots_holding_user_data` 的算法（`crates/singlefs-core/src/mounted_session.rs:2831` 起、`2871-2875`）三段在改前改后一个字都没变，私有函数只是把原来内联的前半截换成解构公开函数的返回值（「mapping_leaves_decoded: _,」，`crates/singlefs-core/src/mounted_session.rs:2829`，丢掉解叶数）。所以 `VersionAsTheCompactionReadsIt` 的三个字段，次序与留存条目接后面的位置，抽出前后逐项相同。这是读码加 `diff` 的结论；没有另做运行时对拍（整理的盘镜像差分第一轮已做）。

### 3.3　变成公开之后能不能绕开整理自己的不变量

- 公开函数只收 `&TransactionOutput`、交回拥有所有权的值，函数体里没有 `&mut`、没有会话、没有分配器（`crates/singlefs-core/src/mounted_session.rs:2795-2822`）；没有产品函数把 `CentralMappingAsTheCompactionReadsIt` 当输入收（`grep -rnF CentralMappingAsTheCompactionReadsIt crates` 在产品文件里只命中 `mounted_session.rs` 自己的定义、返回、解构）。
- `TransactionOutput` 的 `mapped_units`、`mapping_entries_kept_for_the_snapshots` 本来就是公开字段，`CentralMappingEntry` 与 `scan_region` 早就公开（测试已有「use singlefs_core::region_scan::CentralMappingEntry;」，`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:44`）。公开函数没有新增能写、能改的口子，只多了一个读口。

### 3.4　不经它的两处取映射条目（补丁前后都没变，报出来不算打中）

- 搬单元的发布端对计划里每个单元再对上一版核一遍：「relocations.push(relocation_checked_against_the_version(」（`crates/singlefs-core/src/transaction.rs:9368`），函数里先在 `mapped_units` 上按 key 线性找（「.find(|(_, key)| *key == mapping_key)」，`crates/singlefs-core/src/transaction.rs:9548`），再逐 key 查（「let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {」，`crates/singlefs-core/src/transaction.rs:9551`）。这是补丁说明与背景材料「已知」清单都没点名的第三处线性查找（背景只点了 `mapping_locations_of_a_mapped_unit`）；次数受计划长度约束：计划函数收批大小（「plan_batch_skipping_the_units_this_publish_rewrites_anyway(」，`crates/singlefs-core/src/mounted_session.rs:3283`），我没进 `compaction_plan.rs` 逐行核它严格不超过批大小。它不在第一轮判决第三节三条之内。
- 检查档装置里另有一份拷贝：「/// 一版里整理要读的几样，装置照 `mounted_session.rs` 的私有函数 `version_as_the_compaction_reads_it` 另写一份：每个进映射单元的映射条目」（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2381`），用逐 key 查（`crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:2395`）。这是装置，不在产品路径上。

**什么现象会推翻这一条**：在冻结副本里找到第二个调 `scan_region` 的产品调用点（`grep -rn 'scan_region(' crates/singlefs-core/src` 现在只命中 `crates/singlefs-core/src/mounted_session.rs:3021` 与 `crates/singlefs-core/src/region_scan.rs:491`（后者在 `mod tests` 之内，`crates/singlefs-core/src/region_scan.rs:420`））；或找到一条整理入口不经 `run_compaction_on_with_the_region_check_of_the_intent_deletion`。我没做的一步：把「入口只有三个」当全称命题的穷尽检查是靠 `grep -rn run_compaction_on` 的命中（`crates/singlefs-core/src/mounted_session.rs:1910/1984`、`crates/singlefs-core/src/mount.rs:6973`），没有再用调用图工具核。

### 3.5　测试钉的是公开函数，不是私有函数

等价用例拿公开函数的返回值比参照：「central_mapping_as_the_compaction_reads_it(version).mapping_entries,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:507`）。它保不住「私有函数拿到之后原样用」。变体 V6（`variants.tsv` 第 3 行，私有函数取到条目后 `.skip(1)`）：`publish_lookup_by_identity` 一个目标五条全绿（`5 passed; 0 failed; 1 ignored`，`out/variants-logs/row3.log`），而三个整理用例目标红：`compaction_batch_emptying_the_region_deletes_the_intent_in_the_same_publish` 2 条红、`compaction_batch_relocating_a_user_visible_root_is_non_empty` 1 条红、`compaction_runs_stop_and_discard_after_the_publish` 7 条红。结论：私有函数那一段由别的整理用例守着，七·6 自己的用例不守；不是缺口，是分工。

## 四、T2　增长判据数的是不是增长本身

**判定：部分成立。** 数次数这件事做到了，判据不读挂钟也不读 CPU 时间；两种「退回逐项」的坏法都红。但两个数都不是产品调用点的数，「同样慢的别的写法」里至少有一种（批量读者每把 key 重解叶、又把叶记进缓存表）一个数也不涨，七·6 自己的六条用例全绿。

### 4.1　判据读了什么

- 不读时间：`grep -nE 'Instant|SystemTime|elapsed|cpu_time|clock_gettime|thread_cpu' crates/singlefs-harness/tests/publish_lookup_by_identity.rs` 在冻结副本里零命中。第一轮判决第三节第 2 条要的「判据不读挂钟、不读 CPU 时间」这一半成立。
- 解叶数：产品里是「self.leaves_read.len()」（`crates/singlefs-core/src/transaction.rs:4413`），也就是缓存表里有几片叶，不是解了几次。测试拿它和进映射的 key 走到的叶数比：「assert_eq!(」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399`）。
- 比较次数：测试自己造一份包装身份的索引：「let comparisons = Cell::new(0);」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:920`）、「FirstEntryOfEachIdentity::of_the_table(table, |entry| IdentityCountingItsComparisons {」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:922`），建完就把计数清零（「comparisons.set(0);」，`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:926`），只数 `first_entry_of` 的比较；喂的是整理要读的那一版的单元表：「&version_the_batch_reads.units,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1019`）。

### 4.2　读数（冻结副本拷贝上量，未变异）

`out/baseline-post.log`：合成表 `growth_in_percent=118`，池 `growth_in_percent=110`，门槛「const HIGHEST_GROWTH_OF_THE_COMPARISONS_PER_LOOKUP_IN_PERCENT_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:949`）。六条全绿（`test result: ok. 6 passed`，慢档 128 秒）。

两行「增长」变异与追加的「从大到小逐项找」都把读数推过线：

| 变异 | 读数 | 红在哪 |
|---|---|---|
| 七·6 增长：按身份查的索引退回逐项找（`mutations-replacements.tsv` 第 1 行） | 池 391%（小池 4852 次查 1473788 次比，大池 19036 次查 22657592 次比） | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1107` 的增长断言 |
| 七·6 按身份查：索引从大到小逐项找（`mutations-append.tsv` 第 4 行） | 合成表 398%（小表 131840 次比，大表 2100224 次比） | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:990` 的增长断言 |

门槛两侧的余量：未变异 118%/110%，逐项找 391%/398%，门槛 200%，两侧各隔 80 个百分点以上。数只由表的内容和查法定，不随负载与构建档变（代码里没有时间量；同一份未变异读数我量了两次——`out/baseline-post.log` 与上一条腿留在草稿目录的 `growth-base.log`——都是 110）。

### 4.3　三个问题

**① 改成别的同样慢的写法，两个数涨不涨。**

| 写法 | 解叶数 | 比较次数 | 判 |
|---|---|---|---|
| 索引按插入序线性找（`first_entry_of` 里） | 不变 | 涨到 391% 以上 | 红（上表） |
| 整理读版本退回逐 key 查映射（`crates/singlefs-core/src/mounted_session.rs:2803` 换成 `central_mapping_locations_in_the_version`） | 0，对 6 | 不变 | 红，但红在解叶数断言（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399`），不是增长断言：`out/replacements-logs/row2.log` 的 panic 是「整理读这一版解了 0 片映射叶，进映射的 596 把 key 走到 6 片」，增长那一行没走到 |
| 每把 key 现建一个批量读者（`mutations-append.tsv` 第 3 行） | 0，对 1 | 不变 | 红，同样红在解叶数断言（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399`，`out/append-logs/row3.log`） |
| **批量读者每把 key 重解叶、仍把叶记进缓存表**（变体 V1，不入变异表；`variants.tsv` 第 1 行） | 不变（缓存表长度仍是叶数） | 不变 | **不红**：`publish_lookup_by_identity` 六条含慢档全绿（`test result: ok. 6 passed`，`out/variants-logs/row1.log`；慢档 148.6 秒，原样 128.2 秒，多出来的就是重解叶的时间，测试读不到） |
| 每次查都重建索引 | 不变 | 不变（建索引的比较在 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:926` 清零，不数） | 推出来不红；这一格我没起变体跑，读码结论 |

V1 的替换文本（`crates/singlefs-core/src/transaction.rs:4400-4405`，批量读者 `locations_of_key` 里取叶那一段）：

```
        self.leaves_read.insert(
            leaf_role,
            CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes),
        );
        self.leaves_read[&leaf_role].locations_of_key(key)
```

这是第一轮判决要避开的「每把 key 把一整片叶重解一遍」。「解叶数」断言能把它放过，是因为被数的是缓存表的键数，不是解叶的动作数。代码里的文档注释写「每片叶在第一次有 key 走到它时解一次、之后不再解」（`crates/singlefs-core/src/transaction.rs:4408`），这句话在 V1 下不再真，计数依旧过。

**② 产品代码里有没有不经包装的比较漏数。** 数的不是产品的调用：比较次数取自测试自己建的那份索引（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:922`），产品里 `deadlist_changes_of_this_publish`（`crates/singlefs-core/src/transaction.rs:5729`）、`carried`（`crates/singlefs-core/src/transaction.rs:7957`）、`unit()`（`crates/singlefs-core/src/transaction.rs:3781`）等调用点从不进这份计数。变体 V4（`variants.tsv` 第 2 行，把快照判定那一圈改回在 `mapped_units` 上线性找）：`publish_lookup_by_identity` 5 passed、`snapshot_release_and_destroy_through_the_deadlist` 15 passed，全绿（`out/variants-logs/row2.log`）。包装类型是测试里的，索引泛型要求 `Identity: Ord`（「impl<'table, Identity: Ord, Entry> FirstEntryOfEachIdentity<'table, Identity, Entry> {」，`crates/singlefs-core/src/transaction.rs:3263`），把索引换成 `HashMap` 必须改泛型约束，测试里的包装类型没有 `Hash`，会在编译时就红；这一支不漏。

**③ 门槛会不会被别的因素推过线或压下线。** 比较次数只由表的内容和 `BTreeMap` 的层数定：合成表固定 512/2048 个身份，池里两档的 `lookups` 比是 19036/4852，约 3.92 倍（接近 4 倍），读数 110%。能推过线的只有把 `units` 表的组成改到使两档查次比远离 4 倍；这类改动在池造法（`FILES_OF_THE_SMALLER_GROWTH_POOL` 64、`FILES_OF_THE_LARGER_GROWTH_POOL` 256）不动时不会发生。压下线（逐项找也低于 200%）要求表很短：小池 `units` 约 605 项（每次调用 606 次查，含一次查不到的）、大池约 2379 项，逐项找读数 391%，没有这个问题。

**什么现象会推翻这一条**：V1 在 `out/variants-logs/row1.log` 里换成红，或第一轮判决认为 V1 那种写法不算「同样慢」。

## 五、T3　快照判定那一圈换了查法之后的行为

**判定：成立。** 同一角色重复出现取最先一项、查不到时 panic 文案逐字相同、八种构造的上一版上改前改后写出的结果逐行相同（含一种两边都返回同一个报错、四种两边 panic 文案一致）。有两条附带：圈里还剩一处线性查找（已知，延后）；索引每次发布多建一遍（成本，未量）。

### 5.1　快照判定那一圈取映射 key（与映射位置）的每一处

函数 `deadlist_changes_of_this_publish`（「fn deadlist_changes_of_this_publish(」，`crates/singlefs-core/src/transaction.rs:5704`），唯一调用点「let deadlist = deadlist_changes_of_this_publish(」（`crates/singlefs-core/src/transaction.rs:7730`）。圈里：

| 位置 | 取什么 | 改前 | 改后 |
|---|---|---|---|
| 「let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();」（`crates/singlefs-core/src/transaction.rs:5729`） | 建上一版 `mapped_units` 的索引，一次 | 没有 | 新增，在圈外 |
| 「.mapping_key_of(*identity)」（`crates/singlefs-core/src/transaction.rs:5732`），在「for identity in &user_visible_roles_replaced {」（`crates/singlefs-core/src/transaction.rs:5730`）里 | 换下的每个有落点的角色取映射 key | `mapping_key_carried_from` 在 `mapped_units` 上 `iter().find`（补丁原文：「-        let mapping_key = mapping_key_carried_from(previous_version, *identity);」`research/prompts/_m4-publish-lookup-code-r2-diff.md:90`） | 索引取，`.to_vec()` |
| 「match mapping_locations_of_a_mapped_unit(previous_version, *identity) {」（`crates/singlefs-core/src/transaction.rs:5748`），只在进 deadlist 那一支 | 同一角色的映射位置 | 在 `mapped_units` 上线性找一遍，再逐 key 查 | **没变**：「.find(|(mapped, _)| *mapped == identity)」（`crates/singlefs-core/src/transaction.rs:4295`），再「central_mapping_locations_in_the_version(previous, key)」（`crates/singlefs-core/src/transaction.rs:4297`） |
| 「.verdict_of(&mapping_key, previous_snapshot_txg_of_the_live_head)」（`crates/singlefs-core/src/transaction.rs:5735`）、「mapping_key: mapping_key.clone(),」（`crates/singlefs-core/src/transaction.rs:5766`）、「mapping_entries_kept_this_publish.push((mapping_key, locations));」（`crates/singlefs-core/src/transaction.rs:5768`） | 用上面取到的 key，不再取 | — | — |

圈外同一发布里的另一处取映射 key，补丁前就已是索引：「previous.map(TransactionOutput::mapped_units_by_role);」（`crates/singlefs-core/src/transaction.rs:7956`）与「.mapping_key_of(identity)」（`crates/singlefs-core/src/transaction.rs:7961`）。

所以圈里「换下角色取映射 key」这一处换成了索引，「进 deadlist 的角色取映射位置」那一处（第 5748 行经 4295）还是线性找；后者背景材料已作「记延后」报出，我只列出来，不算打中。

### 5.2　改前改后同一角色重复出现、查不到

**读码**：

- 同一角色重复：索引建法是「.or_insert(entry);」（`crates/singlefs-core/src/transaction.rs:3272`，前一行 `.entry(identity_of(entry))` 在 3271），见过的身份不覆盖，取最先一项；改前是 `iter().find`，也是最先一项（补丁原文「-        .find(|(mapped, _)| *mapped == identity)」，`research/prompts/_m4-publish-lookup-code-r2-diff.md:110`）。两边对「相等」的定义一致的前提是 `TransactionUnit` 的 `==` 与 `Ord` 的相等同义：它派生了全部四个（「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」，`crates/singlefs-core/src/transaction.rs:2465`），枚举成员的载荷共 12 个类型，我逐个查了派生行（`DataUnitIndexInFile`、`ExtentLowerNodePosition`、`ExtentUpperNodePosition`、`InodeLeafContainerIndexInTree`、`AllocationRecordTreeNodePosition`、`CodeTwoTreeNodePosition`、`TombstoneContainerIndexInTree`、`ContainerTreeNodePosition`、`LineageContainerIndexInTree`、`RetirementContainerIndexInTree`、`InstanceTablePageIndex`、`InodeNumber`），都是同一行派生；`grep -rnE 'impl.*\b(Ord|PartialOrd|PartialEq)\b for ' --include=*.rs .`（在冻结副本 `crates/` 下，滤掉 `tests/`）只命中 checker 档的 `crates/singlefs-checker-tier/src/crash_judge_tables.rs:159/165` 与 `crates/singlefs-checker-tier/src/crash.rs:1682/1688` 两个无关类型，核心 crate 里没有手写实现。
- 查不到的 panic 文案：改前「-        .expect("照抄进来的角色在上一版里进映射、有一把 key")」（`research/prompts/_m4-publish-lookup-code-r2-diff.md:112`），改后「.expect("照抄进来的角色在上一版里进映射、有一把 key")」（`crates/singlefs-core/src/transaction.rs:3308`），两句逐字相同（都是 `Option::expect`，消息即全文；位置不同，见下）。
- 建的那一刻之后上一版会不会被改：`previous` 是共享引用（「previous: Option<&TransactionOutput>,」，`crates/singlefs-core/src/transaction.rs:5705`），索引借着那张表，文档注释写明：「索引借着那张表（'table）：索引活着时那一版改不了」（`crates/singlefs-core/src/transaction.rs:3258`），`mapped_units` 是普通 `Vec<(TransactionUnit, Vec<u8>)>`，没有内部可变性（该行 `crates/singlefs-core/src/transaction.rs:3228`）。
- 现建还是复用：每次调用 `deadlist_changes_of_this_publish` 在圈外建一次（第 5729 行），圈里复用；不跨调用、不跨版本留；与圈外 7956 行那份是两份独立的索引，同一次发布建两遍（建的是同一张 `previous.mapped_units`）。一个成本上的观察：第 5729 行的建索引不看有没有换下的角色、不看有没有快照，只要 `previous` 是 `Some` 就建（「let Some(previous_version) = previous else {」在 `crates/singlefs-core/src/transaction.rs:5713`，到第 5729 行之间没有判断快照或换下的角色是否为空的分支）；改前这一圈只在换下的角色上各线性找一次，换下的角色少（一次小覆盖写约四个）而表大时，改后建索引的 `O(P log P)` 可能比改前的 `O(k·P)` 大。我没量，只写读码；因为 7956 那一份已经是同样的建法，增量是常数倍，不是新的量级。

**运行**（`tools/diff_t3.rs`，在改前与改后两份草稿副本里各放一份测试，同一串合法操作：建池、建快照、对上一版的 `mapped_units` 做八种改动后覆盖写一次，打出结果摘要或报错或 panic 文案）。原样输出 `out/t3-pre.T3DIFF.txt` 与 `out/t3-post.T3DIFF.txt`，逐行 `diff` 无输出（`t3diff` 步骤打出 `name=t3diff case_lines_pre_vs_post=identical cases=8`）：

```
untouched                                     Ok digest=990476732496cd8d deadlist_entries=4 kept=4 released=8
duplicate-role-appended-with-another-key      Ok digest=990476732496cd8d deadlist_entries=4 kept=4 released=8
duplicate-role-prepended-with-another-key     Err ReleaseNotInMapping { unit: Data(DataUnitIndexInFile(0)) }
duplicate-role-same-key-twice                 Ok digest=990476732496cd8d deadlist_entries=4 kept=4 released=8
role-data-0-missing / role-inode-root-missing / role-extent-root-missing / mapped-units-empty
                                              Panic message=照抄进来的角色在上一版里进映射、有一把 key
```

（上面是把八行合并排版的；每行的原样文字在两份 `T3DIFF.txt` 里。）追加重复项时第一项仍是原来的，所以与未改动一致；前置重复项（第一项换成另一把 key）时改前改后都报 `ReleaseNotInMapping { unit: Data(…(0)) }`；这个报错在释放判定那一路（`crates/singlefs-core/src/transaction.rs:4055` 起）和圈里进 deadlist 那一支（`crates/singlefs-core/src/transaction.rs:5748` 起）都能报，我没区分是哪一处先报，只确认两边相同。panic 的位置不同：改前 `crates/singlefs-core/src/transaction.rs:7074`（已删的私有函数里），改后 `crates/singlefs-core/src/transaction.rs:3308`（`mapping_key_of` 里），每种 4 次（`out/t3-pre.log`、`out/t3-post.log` 里的 `T3DIFF-PANIC-LOCATION` 行）。文案相同、位置不同。改前一侧 panic 在圈里（7074 是改前私有函数），改后在 3308；我没有把改后的 panic 单独归到圈里还是圈外 7957 行那一处：圈的调用点 7730 在 7957 之前，所以先 panic 的是圈，这是读码，不是运行出来的区分。

### 5.3　「索引键改错」那条变异红在产品断言上，说明什么

`mutations-append.tsv` 第 5 行（圈里的映射 key 一律取 inode 根那一把）：原样 `out/append-logs/row5.log` 红在产品里：「previous_entry.is_none(),」（`crates/singlefs-core/src/transaction.rs:5839`，`assert!` 起于 5838 行，panic 位置 `crates/singlefs-core/src/transaction.rs:5838:9`）。问题是测试自己的 deadlist 比较是不是也能红。我把这块 `assert!` 换成 `let _ = previous_entry;`，再套同一行变异跑点名的用例（`out/row5-without-product-assert-logs/row1.log`）：仍红，红在测试自己的断言「四个被快照引用的单元进 deadlist：头 = 活头 inode 树、纪元 = S.txg、映射 key 照建快照那一版」（`crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:224`，panic 位置 `crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:221:5`），实际集合只剩 1 个元素（四条条目的 key 折成了同一把），期望是 4 个。结论：这一行变异的判别力不只靠产品断言；产品断言只是让它红得更早。边界：这是一条变异的读数，不是「测试判别力不靠产品断言」的全称结论；在其他会让圈里取到的 key 仍然互不相同、但配错角色的坏法下（例如两个角色的 key 对换），我没跑。

**什么现象会推翻这一条**：八种构造里改前改后任何一行不同；或找到一个 `TransactionUnit` 的 `==` 与 `Ord` 不一致的载荷类型（我查的 12 个都是派生）。

## 六、T4　新加的五行变异与换名的两行

**判定：部分成立。** 七行变异在原样的代码上各自红，原样用例绿，红的理由都能说清；换名后两行仍点名到改名后的慢档用例。找到两处第二批的改法改回去而没有任何一行变异红（V1、V4），其中 V4 是纯性能、本来就没有值层面的判法，V1 是第一轮判决要避开的那种写法。

### 6.1　七行逐行核红（冻结副本拷贝，每行一次）

锚点在拷贝里都命中恰好一处（`tools/mutate-check.py` 命中不是一处会打 `anchor_hits=` 行，输出里没有）。原样输出 `out/mutations-append.out`、`out/replacements-logs/`、`out/append-logs/`。

| 行 | 结果 | 红在哪 | 改坏的是结果还是性能 |
|---|---|---|---|
| 整理读版本：隔一把 key 才查映射（append 第 1 行） | 红 | 等价用例的断言消息「整理读这一版取的映射条目与逐 key 查的参照逐项相同（不漏、不错、次序同）」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:509`，断言起于第 506 行，txg 6） | 结果 |
| 整理读版本：漏查第一把 key（append 第 2 行） | 红 | 同上（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:506`，txg 6） | 结果 |
| 整理读版本：每把 key 现建一个批量读者（append 第 3 行） | 红 | 解叶数断言（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399`，运行时代入后的消息是 整理读这一版解了 0 片映射叶，进映射的 10 把 key 走到 1 片，txg 6） | **性能**：每把 key 照样查得到同一个答案，映射条目逐项相同（第 506 行那一比没红）；红是因为被数的批量读者是外层那一个，而查走了另外造的读者，外层的缓存表为空 |
| 按身份查：索引从大到小逐项找（append 第 4 行） | 红 | 合成表增长断言 398%（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:990`） | 性能（合成表里每个身份只出现一次，换成从大到小找，结果不变） |
| 按身份查：快照判定那一圈的映射 key 一律取 inode 根那一把（append 第 5 行） | 红 | 产品断言消息「一个单元只被杀一次」（`crates/singlefs-core/src/transaction.rs:5840`，断言起于第 5838 行，`out/append-logs/row5.log`）；去掉产品断言后红在测试自己的 `crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:221` | 结果 |
| 增长：按身份查的索引退回逐项找（replacements 第 1 行） | 红 | 池增长断言 391%（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1107`） | 性能 |
| 增长：整理读版本退回逐 key 查映射（replacements 第 2 行） | 红 | 解叶数断言（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399`，运行时代入后的消息是 整理读这一版解了 0 片映射叶，进映射的 596 把 key 走到 6 片，txg 133） | 性能；红的是解叶数，不是增长比，「增长」一词在名字里，增长断言没走到 |

原样用例在未变异代码上绿：`test result: ok. 6 passed`（`out/baseline-post.log`）。

### 6.2　换名

旧慢档用例 `a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units`（补丁原文「-fn a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units() {」，`research/prompts/_m4-publish-lookup-code-r2-diff.md:481`）在冻结副本的 `crates/mutations.tsv` 与 `crates/singlefs-harness/tests/` 里零命中（`grep -c` 在这两处都是 0）。新名「fn compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units(」在测试文件里存在（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1038`）；`crates/mutations.tsv` 的 4973、4974 两行第 5 列与第 6 列都点名到它，上面两行变异按这个名字实跑，确实跑到并红了（`out/replacements-logs/row1.log`、`row2.log` 里 `compaction_batches_decode_each_routed_mapping_leaf_once_…` 一条 FAILED）。

### 6.3　第二批的改法改回去而没有一行变异红的地方

第二批的改法共五处：`leaves_decoded`、公开函数的两个字段、`mapping_key_of` 文档注释、快照圈改走索引、删掉 `mapping_key_carried_from`。逐处：

| 改法 | 改回去 | 七行变异红不红 | 别处红不红 |
|---|---|---|---|
| `leaves_decoded`（`crates/singlefs-core/src/transaction.rs:4412`） | 改成别的数（常量等） | 读码：第 3、7 行同一条解叶数断言（第 399 行）会红，没起变体跑 | — |
| `mapping_entries` 字段（`crates/singlefs-core/src/mounted_session.rs:2786`） | 改成漏读、读错 | 第 1、2 行已跑，红 | — |
| `mapping_leaves_decoded` 字段（`crates/singlefs-core/src/mounted_session.rs:2788`） | 改成 `central_mapping.leaves_decoded()` 以外的值 | 读码：同上，没起变体跑 | — |
| 快照圈改走索引（`crates/singlefs-core/src/transaction.rs:5729-5734`） | 改回线性找（V4） | **不红**（行为相同） | `publish_lookup_by_identity` 5 passed、`snapshot_release_and_destroy_through_the_deadlist` 15 passed，全绿（`out/variants-logs/row2.log`）。属于纯性能的改动，本来就没有值层面的判法；能判它的只有产品调用点上的计数，现在没有 |
| `mapping_key_of` 文档注释（`crates/singlefs-core/src/transaction.rs:3299-3303`） | 改回原句 | 不红（文档） | — |
| 批量读者按叶记（`crates/singlefs-core/src/transaction.rs:4400-4405`，第一批的改法，第二批只在它上面加了 `leaves_decoded`） | 每把 key 重解但仍记进缓存表（V1） | **不红** | 六条含慢档全绿（`out/variants-logs/row1.log`） |

V1 与 V4 都不在第二批五行追加里；V1 是第二批 `leaves_decoded` 的计数口径带出来的缺口（见第四节 4.3），V4 是第二批「圈里索引」没有可数的东西。两处都没有任何一行变异红，也没有任何用例红。「找不到」这一条我只试了五处里能构造出有意义变体的两处，别的三处（常量、字段、注释）按读码判。

### 6.4　第一轮判决第三节三条的验收对照

- 第 1 条（等价用例判得出整理读漏、读错，攻方两条变异进表、点名的用例红）：做到。`.step_by(2)` 与 `.skip(1)` 在 `mutations-append.tsv` 第 1、2 行，都红在第 506 行的逐项比。
- 第 2 条（增长判据数次数，两行增长变异都红，未变异的用例绿，判据不读挂钟、不读 CPU 时间）：做到一半。两行都红、未变异的绿、不读时间，三样成立；「按身份查了几次」数的是测试自己造的索引，不是产品里的查次数，见第四节 4.3 ②；「每批整理解了几片叶」数的是缓存表长度，见 V1。
- 第 3 条（`mapping_key_carried_from` 那一处改走同一个索引，带一条会红的变异）：做到。圈里 `mapping_key_of`（`crates/singlefs-core/src/transaction.rs:5732`），私有函数已删（补丁原文「-fn mapping_key_carried_from(」，`research/prompts/_m4-publish-lookup-code-r2-diff.md:106`），变异 append 第 5 行红。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 公开出去的那一半 | 成立 | 产品里取「整理读一版的映射条目」只有 `crates/singlefs-core/src/mounted_session.rs:2795` 一处，唯一产品调用者 2830，三个整理入口（1910、1984、`crates/singlefs-core/src/mount.rs:6973`）都汇到 3018 那一个调用点；抽出前后私有函数的产出逐项相同（`diff` 只出三段新增）；公开后只多一个读口。搬单元的发布端另有一处按 key 逐个查（`crates/singlefs-core/src/transaction.rs:9551`）与一处线性找（9548），补丁前后没变 |
| T2 增长判据数的是不是增长本身 | 部分成立 | 不读时间；两行「退回逐项找」的坏法与追加的「从大到小逐项找」都红（391%、398%，原样 110%、118%，门槛 200%）；两个数都不是产品调用点的数，批量读者每把 key 重解叶仍记进缓存表（V1）一个数也不涨、六条全绿 |
| T3 快照判定那一圈的行为 | 成立 | 重复角色取最先一项、查不到 panic 文案逐字相同；八种构造上一版改前改后 `T3DIFF` 行逐行相同；只差 panic 位置（7074 对 3308）；圈里进 deadlist 那一支的 `mapping_locations_of_a_mapped_unit` 仍线性（已知）；「键改错」那行变异去掉产品断言仍红在测试自己的比较上 |
| T4 五行追加与换名两行 | 部分成立 | 七行各自红，红的理由逐行说清；其中「每把 key 现建读者」是性能坏法，凭解叶数断言（读的是外层读者）而红；换名后两行仍点名到新名；V1（读者重解叶）与 V4（快照圈改回线性）改回去没有任何一行变异红 |

## 八、没有复核的、复核不了的

- 实现员报告 `~/.local/share/singlefs/work/m4-publish-lookup-r2-writer/report.md`：没读（正文只给了路径与 sha256），实现员的说法我只拿代码与跑出来的读数核。
- 背景材料第二节写的「第二台测试全绿 2533 条」「变异轻型点名 37 行全抓到」「崩溃轻型」等，是主 agent 的观测，我没复核，也没起；本腿不跑门禁、不跑 checker 档。
- 第 3.4 节搬单元发布端的计划长度是否严格不超过批大小：没进 `compaction_plan.rs` 逐行核。
- 第 4.3 节「每次查都重建索引」那一格：读码结论，没起变体跑。
- 第 6.3 节里 `leaves_decoded`、两个字段改回去的结果：读码结论，没起变体跑。
- T3 的八种构造是我造的，不穷尽；上一版 `mapped_units` 被人改成什么样我只取了重复（追加、前置、同 key 两遍）、缺失（数据单元、inode 根、extent 根）、全空。
- 只抽一次样：本报告里「没打中」（T1、T3 的一致）各格我只跑了一次；按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」这些格的「一致」不能单独支撑结论，要靠攻方腿的另一条路。
- 接续：上一条腿的读数我一个都没引，全部重量。

## 九、没做什么

- 不判攻方腿的格，不替主 agent 采纳或出判决；上面的「判定」是这条正推腿对每格的读码与复跑结论。
- 不改冻结副本，不改 `crates/` 下任何文件，不起门禁阶段，不跑 checker 档，不起 GPU，不跑重型测试。
- 没起第二台（`multi-host-run.sh --peer`）；每条 cargo 都在本机，经 `scripts/hosts/run-with-memory-cap.sh` 带 16G 上限、线程上限 31。
