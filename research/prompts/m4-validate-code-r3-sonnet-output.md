# m4-validate-code-r3 辩方腿报告：改法甲（第一个文件一个数据单元都没有时 size 0 也写标签 0）（Sonnet，2026-10-26）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 K5 S1 -->

立场：替被否的一方辩护，复核第二轮判决与改法甲「够不够得着、是不是同样打中被否的乙、丙」。方案按 `crates/` 今天的实现谈；冻结副本 `~/.local/share/singlefs/work/m4-validate-code-r3/repo/`，两份产品文件的 sha256 与开工快照 `research/prompts/m4-validate-code-r3-snapshot/judged-sha256.txt` 逐字相同（extent_tree.rs 5d10267f…、transaction.rs d9405f4f…），也与此刻主工作区相同；报告引的其余源文件（recovery.rs 等九份）冻结副本与主工作区 sha256 相同，只有 `crates/mutations.tsv` 两边不同（别的会话在改），变异表行号一律取主工作区此刻的。

## 0 复跑命令与文件 sha256

复跑（整条经内存包装，工作目录仓根，`--work` 给一个新目录；三段共约 15 分钟，我在草稿目录里三段各跑过一次，输出与模型目录 `out/` 逐行相同，见第 7 节）：

```
cd /home/fy5090/code/singlefs && RERUN_STAGES=sweeps,existing,mutants nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m4-validate-code-r3-sonnet-model/tools/rerun.sh --repo $PWD --frozen /home/fy5090/.local/share/singlefs/work/m4-validate-code-r3/repo --work <新草稿目录> --model $PWD/research/prompts/m4-validate-code-r3-sonnet-model
```

模型目录 `research/prompts/m4-validate-code-r3-sonnet-model/`（`sha256sum -c SHA256SUMS` 全 OK，18 份）：

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | cf56e2a5ecdd4533d76bfdbed903ca43b5c6bb02609d9a5cb45823ac5427dbee |
| tools/mutate.py | c74bb528420a34fe3230ddcd4be97c89b453deeee80733f06d674ca71447b9e7 |
| tools/compare-sweeps.py | 6cfb4b525767a3a6f4ff6f98165cea90f74163ac500225423e1377e3f7d9e5a9 |
| src/defense_sweep_of_follow_ups_after_truncate_to_zero.rs | 0ed205fd4183a0c981986f415c8f7245142461032afff64eb7b5374ab75137e0 |
| src/hunks-of-jia.patch | ca30a3691688f1f0e7bf14cffafaecbb19dc403ee8617e89a70375111824aca0 |
| src/relax-truncation-floor.patch | 7b9389755f8c14584bb4f91a90ce84fb2c0c4b6043bc060d8a53a4b5f9ed747e |
| out/sweepA-patched.txt | 86215805e6563166f7fc09378d878334521c0d92466250341b5298aa9e3b41d8 |
| out/sweepA-old.txt | 5c0ce43f5846c82fb62e31a0e7ea5bd5543a4a1ea43d72a6fa292e6f076fb8d9 |
| out/sweepB-patched.txt | 29f84cd7169e62528209a4def20740a79703e960e1529d185ccd069a3900de06 |
| out/sweepB-old.txt | a579a274ebc7b180d1780731e325c7fd08c2e13b2b4af754c1a497d52efdaa41 |
| out/compare-A.txt | 318ea9db68ab300dae4613963645c95168fa5b21590e3f609afe6ef2cb554b76 |
| out/compare-B.txt | daba20c1cb4f1a16b5e767e905cc51a64f2b962fb98780ac70806c2b85db69b2 |
| out/generate.txt | a4e8db16b8480753408cb189f75df6204c595f228ad613a7ec48b95ae07b164f |
| out/existing-patched.txt | c886dc6dc51317144cbd4fd202509dff21989290f8f5a2e2706f7476c9e7ffab |
| out/existing-old.txt | 6970d53613093f4774fc82dfac6f68134e8c4c1a7ca3081958335a4535e7b558 |
| out/mutants.txt | 6a3cb579ff630a572f449695abb3ecc80b075d4ed7cb2656c35f37ccb7f2964c |
| out/others-patched.txt | e7a6828853fac92506270ed4b969e0722e6fc498c007a69e2318527e00bcbeef |
| out/log-mutant-set-caller-always-first-unlink.txt | 1b6ab2fc8abed5eb89844a2d7ccc234c53cb53c336e3e306c7df1fe4e1cd89af |

写路径做成的起点镜像（草稿目录里生成，不入库，复跑会重新生成）sha256：af28ade771f6f7e0a992548d1136e97c2d0658cd1d31cae828950a92e845fb59（草稿目录里的与复跑出来的相同）。

## 1 判定一览（判决条目 / 辩方判定 / 证据）

判定用语：站得住 / 够不着 / 同样打中别的臂。每一格的推翻条件在第 6 节。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 判决 T1 行：N = 0 打中（`research/prompts/m4-validate-code-r2-main-verification.md:25`「N = 0 打中」） | **站得住** | 冻结副本拷贝上撤掉改法甲五个 hunk（`src/hunks-of-jia.patch`），25 串后续里 12 串读者拒（手造起点；写路径做成的起点 14 串），九条用例里四条新加的红（第 2 节 T4、第 5 节）；打上甲 0 串读者拒 |
| 判决第四节：用户定「甲」，实现的就是甲（`research/prompts/m4-validate-code-r2-main-verification.md:47`「size 0 也写标签 0」、`:71`） | **站得住（有一处字面歧义，见 T1-3）** | 代码读法：记录在就写标签 0、不看 size（`crates/singlefs-core/src/extent_tree.rs:420`「Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),」），记录没了不写（`:421`）；与判决第三条测试「截到 0 再删第一个文件，上段叶不留孤条目」一致（`research/prompts/m4-validate-code-r2-main-verification.md:53`「上段叶不留孤条目」） |
| 判决写回第一条：「一律交 Some(ExtentUpperLeafTarget::NoDataUnit)」（`research/prompts/m4-validate-code-r2-main-verification.md:52`） | **字面够不着代码取的读法** | 字面「一律交 Some」会让记录没了（unlink 最后一批之后）也写条目，孤条目再现；我把这种错法做成变异 write-without-record，只有「截到 0 再删第一个文件」一条用例红（第 5 节）。代码取了与同一节第三条测试一致的读法 |
| 判决第三节第 4 点：乙、丙在打中的格上还中不中（`research/prompts/m4-validate-code-r2-main-verification.md:41`「每个改法在打中的格上还中不中」） | **乙：同样被打中（量到一半）；丙：只挡一条进入路径，不挡已经带这一形的池** | 第 4 节 |
| 判决「回看决策」D8 已定项 14「支撑」 | **站得住，D8 正文有两处没跟上** | 第 2 节 T2 |

## 2 逐格

### T1 今天的写路径真到不了「有记录、0 个单元、size 0」吗

T1-1 **代码做的是什么**：改前 `of_a_file_without_data_units` 只看 size（size > 0 才写标签 0），改后只看记录在不在。新函数的体在 `crates/singlefs-core/src/extent_tree.rs:420`「Some(_) => Some(ExtentUpperLeafTarget::NoDataUnit),」与 `:421`「None => None,」。两个调用方：算上段有条目的号集合 `crates/singlefs-core/src/transaction.rs:4003`「self.inode_record.map(|record| record.size),」，装上段叶 `crates/singlefs-core/src/transaction.rs:12340`「(None, None) => ExtentUpperLeafTarget::of_a_file_without_data_units(」。全仓只有这两处调函数（`grep -rn of_a_file_without_data_units crates --include=*.rs` 在冻结副本里命中 5 行：定义 1、两处调用、两处注释；仓里别处只有 `crates/mutations.tsv` 的变异行提到它）。改前改后差别逐位对照：记录 `Some(n)` 且 n > 0 → 都写；记录 `None` → 都不写；只有记录 `Some(0)` 一格变了（改前不写，改后写）。第三个 `build_extent_tree` 调用方是新池新建文件的孪生装置，传的是一个单元（`crates/singlefs-core/src/transaction.rs:15467`「build_extent_tree(」），走不到 `(None, None)` 那一臂。

T1-2 **写出「有记录、0 个单元、size 0」的来路逐条查**（只有代码读法的标「读」，量过的标「量」）：

| 来路 | 判 | 依据 |
|---|---|---|
| 新池新建文件（里程碑一那条流） | 到不了（读） | 长度 0 的内容也切出一个事务：`crates/singlefs-core/src/write_request_split.rs:61`「content_length_in_bytes.div_ceil(payload_capacity).max(1)」 |
| 建 inode | 到不了（读） | 只建第一个文件之外的号，第一个文件那条记录不碰；第一个文件之外的文件没有单元就不在计划里，`(None, None)` 在那里是 panic（`crates/singlefs-core/src/transaction.rs` 装第一个文件之外的上段条目那一处），不经新函数 |
| 截断意图的最后一批 | 到不了（读） | 截到 0 留一个声明长度 0 的单元：`crates/singlefs-core/src/truncate.rs:238`「let keeps_at_least_one_unit_or_had_none」，只有之前就 0 个的（标签 0 的洞）才 0 个 |
| 删除的一批 | 到不了（读） | 非最后一批恒留 ≥ 1 个：`crates/singlefs-core/src/unlink.rs:123`「（恒 ≥ 1）」，`crates/singlefs-core/src/unlink.rs:192`「(1..data_units_before_the_batch).contains(&data_units_kept),」 |
| 删除的最后一批 | 记录同一次发布删掉，不是这一形（读+量） | `.claude/kb/decisions/08-核心索引结构.md:504`「最后一批释放 N 剩下全部单元、删 N 的 inode 记录」；断言 `crates/singlefs-core/src/transaction.rs:13584`「data_pointers.is_empty(),」；扫描 07、08、19 号串（第 2 节 T4）删完读到无文件 |
| 回退 | 到不了（读） | 回退那次发布的新根取目标版的两棵树整棵：`.claude/kb/decisions/08-核心索引结构.md:188`「管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树」；目标版是同一写者写的，要么有条目要么这一形整套带标签 0 |
| 整理换下上段叶 | 到不了（读，未量） | 整理只搬已有单元的落点，上段叶重写仍经上面那两个调用方；没有单元可搬的第一个文件不产生新的「记录在、0 个单元」 |
| 扫描重建之后的第一次发布 | 要看盘上有什么（量） | 重建出的上一版里 `inode_record` 与 `data_pointers` 来自盘；盘上是标签 0 配 size 0 时 `inodes_with_an_extent_upper_leaf_entry` 现在与盘上条目一致（`crates/singlefs-core/src/transaction.rs:7468`「previous.inodes_with_an_extent_upper_leaf_entry();」）。扫描 16–19 号串是「换一次挂载再发布」：打甲 0 拒，撤甲在第二次挂载处就拒 |
| 唯一能到的来路：读进来的标签 0 文件截到 0 | **到得了（量）** | 扫描 00 号串；`.claude/kb/decisions/08-核心索引结构.md:465`「现有写路径自己写不出来」说的就是读进来的版本 |

T1-3 **「用户定的甲」与判决字面**：用户弹窗选「甲：一律写标签 0」（`research/prompts/m4-validate-code-r2-main-verification.md:71`），候选行写的是「第一个文件没有数据单元时 size 0 也写标签 0」（`research/prompts/m4-validate-code-r2-main-verification.md:47`），写回行写「改成一律交 Some(ExtentUpperLeafTarget::NoDataUnit)」（`research/prompts/m4-validate-code-r2-main-verification.md:52`）。三处里「一律」的范围是 size（size 0 与 size > 0 同一形），不是「记录有没有」：unlink 最后一批要把条目去掉（`.claude/kb/decisions/08-核心索引结构.md:504`「不写标签 0」那一句在同一条里，原句「把 N 的条目从上段去掉（不写标签 0）」），判决自己的第三条测试也要「上段叶不留孤条目」。代码因此保留 `None => None`（`crates/singlefs-core/src/extent_tree.rs:421`「None => None,」）、函数参数从 `u64` 改成 `Option<u64>`。字面读「一律交 Some」会让孤条目回来：变异 write-without-record 只有 `…then_unlinked_leaves_no_entry_in_the_upper_leaf` 一条红（`out/mutants.txt`）。这一处是字面歧义，不是实现偏离；要不要把判决写回行的字面改成「记录在就交 Some」，主 agent 定。

T1-4 **改法甲让写出的字节变了没有**：量。两棵树各自编译（已打甲 / 撤掉五个 hunk），同一批 25 串后续、同一个起点，盘上字节的 FNV 哈希逐串比（`tools/compare-sweeps.py`）：手造起点 26 行里字节不同 14 行，写路径做成的起点 17 行（`out/compare-A.txt`、`out/compare-B.txt` 末行）。字节不同的串都含「第一个文件 size 0 而又重写上段叶或删第一个文件」（写路径做成的起点本身就是 size 0，其中不截的 20、22、23 号串也属这一格）；对照串（不截、截到 100、变长后再写、覆盖写第一个文件、只建 inode、建快照）字节逐位相同。所以改动只落在「记录在、0 个单元、size 0」这一格，与实现员自报的一致。

T1-5 **变了的那一版各读者怎么判**：量（扫描 01–06、14–19 号串）。冷走读、冷恢复、可写重挂、挂载态读者都收，读出的长度是 0、7000、97902（三个净荷）字节且内容全 0（`out/sweepA-patched.txt`、`out/sweepB-patched.txt` 的「全0」标记）；池级 checker 红的集合是起点红的子集，没有新红。撤掉甲时冷走读、冷恢复、可写重挂三处拒成「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」，挂载态收 0 字节（变长那几串也拒），与判决 T1 行逐字一致。

### T2 D8 已定项 14 写回之后与代码对不对得上

T2-1 **写回的那一句**：`.claude/kb/decisions/08-核心索引结构.md:451`「写者重写上段叶照写这一条」，括号里写「size 0，或 size > 0 整个文件都是洞」，后半写「这一形上 size 改大、改成原值、截短（截到 0 也算）都只改 inode 记录、标签留 0、不加单元」。对代码：改大 / 改成原值 / 截短三种都进 `crates/singlefs-core/src/truncate.rs:226`「pub fn plan_the_extent_tree_after_a_size_change(」；0 个单元时 `tail_before`、`tail_after` 都是有洞（`crates/singlefs-core/src/extent_tree.rs:335`「if data_units < size_in_bytes.div_ceil(payload_capacity_in_bytes()).max(1) {」对 0 个单元恒真），上段叶那一串不重写、整棵照抄；变长的尾单元重写对 0 个单元交 `None`（`crates/singlefs-core/src/truncate.rs:141`「let unit = DataUnitIndexInFile(data_units_existing.checked_sub(1)?);」）。九条用例里五条旧的 + 四条新的钉这几格，打甲的树上 9 条全绿（`out/existing-patched.txt`，末行 9 passed）。**对得上**。

T2-2 **标签 0 配 size 0 在读者一侧是不是每一处都收**：

| 读者 | 判 | 依据 |
|---|---|---|
| 冷走读、冷恢复、可写重挂的重建 | 收 | 三处共用 `crates/singlefs-core/src/recovery.rs:6675`「fn data_unit_count_matches_the_inode_size(」，标签 0 的尾部是有洞（`crates/singlefs-core/src/extent_tree.rs:1370`「ExtentsOfAFileReadFromDisk::NoDataUnit => TailOfAFile::HoleAfterTheLastDataUnit,」），交给 `crates/singlefs-core/src/write_request_split.rs:112-116`「if data_units_on_disk < data_units_implied_by_the_file_size」那一臂：size 0 要 1 个单元、盘上 0 个，`0 < 1` 成立 → 读出 0。量：扫描打甲 0 串拒 |
| 挂载态读者 | 收 | `crates/singlefs-core/src/mounted_read.rs:815`「ExtentUpperLeafTarget::NoDataUnit」归到「有洞」那一类；同一个函数；量：25 串的「挂载态」列全收 |
| 池级 checker 的单元数对 size | 收 | `crates/singlefs-checker/src/walk.rs:12759`「DataUnitsDeclaredByTheUpperLeafEntry::NoDataUnit), _) => (」配 `:12760`「data_units_existing == 0,」；量：`checker红` 列起点红的子集 |
| checker 档崩溃判器的 GPU 恢复内核 | 收（只读码，没起 GPU） | `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl:432`「let tail_is_a_hole = is_the_first_file」，`:366`「if (lt64(u64_of(units_on_disk), units_implied_by_the_size)) {」与 CPU 同一条 |

T2-3 **`with_the_tail` 把标签 0 交回标签 0 那一臂在 size 0 上成不成立**：成立。`crates/singlefs-core/src/extent_tree.rs:403`「(ExtentUpperLeafTarget::NoDataUnit, _) => ExtentUpperLeafTarget::NoDataUnit,」不看尾部；装叶时 `crates/singlefs-core/src/transaction.rs:12108`「let tail_of_the_first_file = size_of_the_first_file_after_this_publish」的 `tail_of_the_first_file` 对 0 个单元、size 0 算出「有洞」，与标签 0 声明的尾部同向（`tail()` 把 0、3、4 归有洞），不会出现标签与算出的尾部打架。

T2-4 **标签 1、2、3、4 的口径在 size 0、0 个单元上会不会给出两个答案**：写者一侧不会。判决 K5 / S1 的口径：「标签 0（没有数据单元）配 size > 0 是整个文件都是洞」（`research/prompts/m4-trunc-r1-main-verification.md:39`），S1 把标签 0 定义成「标签 0 是这个 inode 一个单元都没有（size 0 的空 inode，或 size > 0 的整个文件都是洞）」（`research/prompts/m4-trunc-r1-main-verification.md:82`）——size 0 本来就在标签 0 的定义里，甲是把写者补齐到这个定义，不是新口径。size 0 且有 ≥ 1 个单元时写者走标签 1/2（`of_data_units_and_size(n,0)` 对 n ≥ 1 为「无洞」），0 个单元走标签 0，两条各管一种单元数，无重叠。读者一侧有一处**不对称**（不是新引入）：挂载态读者对第一个文件「没有条目、size 0、0 个单元」也收（`crates/singlefs-core/src/write_request_split.rs:103`「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 && data_units_on_disk == 0 => {」），冷读那一路把第一个文件缺条目按标签 1、2 的口径判而拒（`crates/singlefs-core/src/recovery.rs:5968`「TailOfAFile::NoHoleAfterTheLastDataUnit,」，注释在 `:5962`）。这就是 C900 说的「读者之间答案不一」，甲之后写者不再产生它，读者不对称本身没改。

T2-5 **D8 正文有两处没跟上甲**（不影响代码对不对，供写回批）：
- `.claude/kb/decisions/08-核心索引结构.md:465`「现有写路径自己写不出来」射程段仍写这一句「标签 0 配 size > 0 这一形」，比 `:451` 窄（没有写 size 0）；不矛盾，但 `:451` 与它并读会让人以为 size 0 不在这条义务里。
- 同一已定项的「依据」表（`.claude/kb/decisions/08-核心索引结构.md:477`「用户定案 2026-10-22」是 2026-10-22 那一条）里没有 2026-10-24 用户定「甲」的一行；`grep -n` 在这份 D8 里对「2026-10-24」与「m4-validate-code-r2」都命中 0 行（判决写回说变更史记了，变更史在 `.claude/kb/decisions-history.md`，D8 本文没有）。依据表要不要补，主 agent 定。

### T3 C900 与第二轮打中那一形还剩哪些来路

对 C900 末格说的甲之后只剩写者漏写一种来路（`.claude/kb/checks-owed.md:579`「一种来路，才能判它红」）。逐条查甲之后第一个文件有记录而上段叶没它的条目的来路：

| 来路 | 判 | 依据 |
|---|---|---|
| 写者漏写（上段叶重写时丢条目） | 甲之后没有 | 两个调用方同一个判据同一个输入（装叶用 `crates/singlefs-core/src/transaction.rs:13555`「record_of_the_first_file_after_this_publish.map(」，算集合用 `crates/singlefs-core/src/transaction.rs:4003`「self.inode_record.map(」；两处都是这一版 inode 树里 inode 1 那条记录，各取一次，读码判同源，没有造出两者不同的输入）；变异把任一调用方改成丢 size 0，各有用例红（第 5 节） |
| 删到一半崩溃 | 没有 | 意图那次不动上段叶（整棵照抄，扫描 07 号串）；非最后一批恒留 ≥ 1 个单元（`crates/singlefs-core/src/unlink.rs:123`「（恒 ≥ 1）」），最后一批把记录与条目同一次发布去掉（`crates/singlefs-core/src/unlink.rs:303`「if inodes_with_an_entry_after.remove(&inode) {」）；发布是原子的（一次发布一个新根） |
| 回退到截断意图中间那一版 | 没有（读） | 目标版是同一写者写的：意图那次发布记录 size 改成目标、上段叶标签留 0（0 个单元的洞），整棵照抄，条目在；回退取目标版两棵树整棵（`.claude/kb/decisions/08-核心索引结构.md:188`「管理员回退那次发布的新根取 R_old 的 inode 树与 extent 树」） |
| 扫描重建 | 没有新来路（读） | 核心层重建是沿树读（`research/prompts/m4-validate-code-r1-main-verification.md:26`「核心层 `rebuild_version_*` 是沿树重建，不按单元头认领」），条目不会被重建「造」出来或丢掉；池级 checker 的扫描重建比三张表，不重写上段叶 |
| 旧版写者写出的池 | **有，且是 C900 判据要判红的东西** | 撤掉甲的树上 01–06、14–19 串（手造起点）与 01–06、14–20、23 串（写路径做成的起点）写出的镜像：记录在（size 0）、上段叶没条目；冷走读、冷恢复、可写重挂拒，挂载态收（`out/sweepA-old.txt`、`out/sweepB-old.txt`） |
| 外来写者把第一个文件写成「没有条目、size 0」 | 有（读） | 挂载态读者收（`crates/singlefs-core/src/write_request_split.rs:103`「DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0」），冷读拒；与上一行同一类 |

**C900 要的判据（第一个文件有记录 ⇒ 上段叶有它的条目）会不会误判合法镜像**：对今天的写者写出的镜像不会——上表前四行都没有「记录在、条目不在」；判红的只有后两行。注意第 5 行的镜像撤甲树上能由用户动作到达：只要有一个用户在旧版本上做「标签 0 文件截到 0、再写别的 inode」。池格式在第一个外部用户之前是软的（`research/prompts/m4-validate-code-r1-main-verification.md:38`「格式在第一个外部用户之前是软的」），这一类池要不要给迁移，判决没写，我也没量。

### T4 第二轮报出的数能不能核

我没读第二轮两条腿的报告与模型目录（禁读），所以「同一段历史」是照判决写的序列自己重造的 25 串，不是它们的 12 种；数不逐个对。能核的是形状：撤甲有读者拒、打甲 0 拒、自带标签 0 目标照绿。

T4-1 **拒收是不是 0 种、读出的长度与内容对不对**（量；`tools/compare-sweeps.py` 末行，`out/compare-A.txt`、`out/compare-B.txt`）：

| 起点 | 串数（含起点行） | 撤甲：有读者拒 | 打甲：有读者拒 | 字节不同 |
|---|---|---|---|---|
| 手造：写路径做成的 3000 字节文件改成标签 0 配 size 3000（同 `crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs` 的做法，带那一条手造泄漏） | 26 | 12 | 0 | 14 |
| 写路径做成：放宽截断下限的副本里让写者自己截到 0 个单元（`src/relax-truncation-floor.patch`），分配记录与映射条目是写者自己释放的 | 26 | 14 | 0 | 17 |

打甲 0 串拒：读出的长度逐串是 0、0、…、7000（截到 0 后变长）、97902（三个净荷 3×32634）、100（覆盖写第一个文件，内容是写进去的 7，非零是对的）、无文件（删第一个文件的几串，挂载态答「没有这个 inode」是对的）；内容列除覆盖写那一串外全是「全0」。撤甲的拒收文字与判决 T1 行一致：「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」（`research/prompts/m4-validate-code-r2-main-verification.md:25` 同句）。

判决 T1 行的「5 种拒收」我的数是 12（手造）/ 14（写路径做成），序列选得更宽（含换一次挂载、两个 inode、快照、删号 2），不是对不上：判决那一格是「12 种后续里 5 种」，我的 25 串里同类的拒收串数更多，量的是同一件事。

T4-2 **自带标签 0 目标照绿**（量，打甲的树，`out/existing-patched.txt`、`out/others-patched.txt`）：`whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session` 9 条全过（旧 5 + 新 4）；`whole_file_hole_of_upper_leaf_tag_zero_with_a_positive_size_read_by_every_reader` 3 条；`checker_known_bad_images` 276 过、2 忽略；`unlink_of_files_other_than_the_first` 10 条；`writes_and_unlinks_after_the_first_file_was_deleted_in_histories_judged_by_the_model` 3 条；全绿。判决说的「四个」我对不上是哪四个，这五个都绿。撤甲的树上同一个九条目标 5 过 4 红（红的正是四条新加的，`out/existing-old.txt`）。

T4-3 **起点镜像的手造泄漏换成不带泄漏的起点，结论变不变**：不变，但我做不出一份「完全干净」的起点，做到的是两档：
- 手造起点的池级 checker 红是 I-3.1、I-3.11、I-1.13、I-3.15（起点行，`out/sweepA-patched.txt`）；25 串打甲之后红的集合都是它的子集，没有一串多出新红（删第一个文件、覆盖写第一个文件的几串里 I-1.13 不再红，我没追查为什么）。
- 写路径做成的起点（`out/generate.txt`）：同一份写路径、放宽了「截断留至少一个单元」，让 3000 字节的内联文件截到 0 个单元，旧单元由写者自己释放。三项分配/记账红（I-3.1、I-3.11、I-3.15）没有了，剩 I-1.13、I-9.38 两条：扫描重建从盘上认出那个已释放单元的残字节、检查器的模型又认为 size 0 也要留一个单元（`max(1)`）。再把残字节抹零，更早的回退候选根引用它，I-2.1、I-4.8、I-7.4 红（`out/generate.txt` 第三行起）。所以「干净起点」要么带泄漏、要么带残字节，从一段真实历史里造不出第三种；一份从未有过那个单元的外来镜像我没造。
- 两档起点上读者的答案一样（上表）。机理也说明与泄漏无关：拒收落在 `crates/singlefs-core/src/recovery.rs:6675`「fn data_unit_count_matches_the_inode_size(」，只比 extent 条目的标签、单元数与 size，不读分配记录。

## 4 判决的结论是不是同样打中被否的乙、丙

判决第三节第 4 点说乙在「写 inode 2 再变长到 7000」上照中、丙全不中，都是推的（`research/prompts/m4-validate-code-r2-main-verification.md:41`「乙（冷读者放宽）」）。我能量到的和推到的：

- **乙（冷走读、可写重挂、冷恢复放宽成也收「没有条目、size 0」）：同样被打中**。撤甲树上的 05、06 两串（写号 2 之后变长到 7000 / 三个净荷）：冷读那一路对「没有条目」的判法是 size 0 才收（`crates/singlefs-core/src/write_request_split.rs:106`「DataUnitsOfTheInode::NoneWritten => refused,」），乙只放宽 size 0，size 7000 的没有条目照拒；量：这两串撤甲树上挂载态读者拒成 `ExtentRecordCountDoesNotMatchTheFileSize`（`out/sweepA-old.txt` 05、06 行，`out/sweepB-old.txt` 同号两行），三处冷读者也拒。乙能救的是 01–04、14–17、19（最终 size 0）这几串，救不了 05、06，18 串在撤甲树上第二次挂载就拒、没跑到变长（推：放宽后会走到同一处）。乙与「丑」的写者一起留着丢条目的写法，这就是判决说的「照中」。
- **丙（拒截到 0）：对「截到 0」这一条进入路径全不中，对已经带这一形的池不挡**。手造起点的 00–19 号 20 串全以截到 0 开头，丙下这一步被拒，后面的步不存在（推）；20–24 号是不截的对照。但写路径做成的起点本身就是「记录在、0 个单元、size 0、标签 0」，撤甲树上 20 号串（不截、只建 1 个 inode 写号 2）与 23 号串（建 142 个写号 143）在它上面照样被冷读拒（量：`out/sweepB-old.txt` 20、23 行；打甲的树上同两串全收）。丙挡的是怎样进入这一形，不是写者遇到这一形怎么写。池里本来就有这一形的来路：撤甲时代截到 0 的池、外来镜像。
- **甲对这两处都不中**：上面所有串打甲 0 拒。
- 乙、丙都不是「同样打中」到判决的结论本身：判决的打中说的是写者在 size 0 上丢条目，乙丙各自留着这个写法或只挡一条路，被打中的是它们能不能当替代方案，不是 T1 这一格的成立。

## 5 T5 测试与变异钉得住改法甲吗

对象：`crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs` 的四条新用例（`:442`、`:478`、`:525`、`:569`）与 `crates/mutations.tsv:5510`「m4-validate-code-r2 T1 甲」、`:5144`、`:5145`（锚点改过的两行，两行的旧串在今天的 `transaction.rs` 里各恰好命中一次，改成它们的新串后各有用例红，见下表）。

**改回「size 0 取 None」：红**。把甲改回，九条用例里四条新加的全红、五条旧的全绿（变异 size-zero-none，等于变异表 5510 行）；撤掉甲的五个 hunk 同样是这四条红（`out/existing-old.txt`）。

**另一种错法红不红**（量；变异在已打甲的冻结副本拷贝上改一处，`tools/mutate.py`，`out/mutants.txt`；「新四」指 `:442`、`:478`、`:525`、`:569` 这四条）：

| 错法 | 九条里红的 | 新四里红的 | 判 |
|---|---|---|---|
| size-zero-none：size 0 取 None（变异表 5510） | 4 | 4（全部） | 钉住 |
| only-size-zero：只在 size 0 时写、size > 0 不写 | 2（旧的 `…keeps_its_entry_when_the_session_rewrites_its_upper_leaf_for_inode_two`、`…keeps_its_upper_leaf_when_the_session_writes_an_inode_past_that_leaf`） | 0 | 钉住（靠旧的 size > 0 两条） |
| set-caller-drops-size-zero：只有算有条目号集合的那个调用方丢 size 0 | 2 | `:478`、`:569` | 钉住，但只有两条 |
| build-caller-drops-size-zero：只有装上段叶的那个调用方丢 size 0 | 2 | `:442`、`:525` | 钉住，但只有两条 |
| write-without-record：记录没了也写条目（读「一律交 Some」的字面） | 1 | `:569` | 钉住，只有一条 |
| build-caller-always-write：装叶的调用方不看记录一律写 | 1 | `:569` | 钉住，只有一条 |
| row5144-arm-none：变异表 5144 行（装叶那一臂又交 None） | 3 | `:442`、`:525` + 旧 `…keeps_its_entry_when_the_session_rewrites…` | 钉住 |
| row5145-set-filter-zero：变异表 5145 行（集合里只认 size 0） | 1（旧的 `:400` 那一条，size 3000） | 0 | 钉住，但它现在只管 size > 0 |
| **set-caller-always-first：算集合的调用方不看记录一律算第一个文件** | **0（九条全绿）** | 0 | **这一目标钉不住** |

最后一行：九条全绿，变异存活。它在另外两个目标里红：`unlink_of_files_other_than_the_first` 的 `deleting_both_files_in_either_order_ends_in_one_zero_entry_upper_root` 与 `writes_and_unlinks_after_the_first_file_was_deleted_in_histories_judged_by_the_model` 的 `the_fast_inode_write_histories_delete_another_inode_after_the_first_file_was_deleted`（`out/log-mutant-set-caller-always-first-unlink.txt`）。所以整仓有用例逮住它，只是不在变异表那一行点名的目标里。

补哪个取样点（供主 agent 定，我没改仓）：
1. 变异表只有一行新增（5510，改共用函数，两个调用方一起错）。两个调用方单独走偏的两种错法（上表第 3、4 行）没有行，各靠两条用例，其中一条（`:569`、`:525`）同时是别的错法的唯一或主要探测；5145 行在甲之后只剩管 size > 0。可补两行：算集合那一处 `self.inode_record.map(|record| record.size),` → `….filter(|size| *size > 0),`（预期 `:478` 与 `:569` 红）；装叶那一处 `size_of_the_first_file_after_this_publish,` → `size_of_the_first_file_after_this_publish.filter(|size| *size > 0),`（预期 `:442` 与 `:525` 红）。
2. 「记录没了不写」（write-without-record、build-caller-always-write）只有 `:569` 一条探测。可补一格：第一个文件截到 0 之后只剩别的文件时删第一个文件，读上段叶是否只剩那个文件的条目（现 `:569` 的断言是「第一个文件那一条没有」，别的文件那一条在不在没看）；这一格我没写。
3. 九条里没有一条把「记录在、0 个单元、size 0」放进**回退**与**换一次挂载之后第二次发布**两格；我的扫描 16–19 号串覆盖了后者（打甲全收），回退只读了码（第 2 节 T1-2），没有用例也没有量。

## 6 什么现象会推翻这些结论

- 「甲的字节只在记录在、0 个单元、size 0 一格上变」（T1-4）：任何一串后续在打甲 / 撤甲两棵树上字节不同、而它从头到尾没有第一个文件 size 0 的时刻——从 `tools/compare-sweeps.py` 打出的「字节不同」行里挑一串，对比两棵树上同一串的盘。
- 「今天的写路径到不了这一形」（T1-2）：一条写路径（新池新建、建 inode、截断、删除、回退、整理、重建之后的第一次发布）在一个起点不含标签 0 的池上写出记录在、0 个单元、size 0 的一版。我的扫描起点都含标签 0，所以它只证明「到得了的来路是读进来的标签 0」，证明不了「别的来路没有」；别的来路靠上表的读码。
- 「读者每一处都收标签 0 配 size 0」（T2-2）：一个读者对这一形拒收。GPU 内核一格只读了码，没起 GPU。
- 「T3 的前四行没有来路」：一个新池上的历史（不经旧版写者）写出记录在而条目不在的一版，池级 checker 补上 C900 的判据之后判红。
- 「T4 结论不依赖泄漏」：拿一份从未有过那个单元的外来镜像（不是从真实历史截出来的）做起点，打甲的树上有读者拒。我没造这样的镜像。
- 「乙同样被打中」：撤甲树上 05、06 两串里挂载态读者答案变成收。「丙不挡已带这一形的池」：撤甲树上 20、23 号串（写路径做成的起点）冷读者不拒。
- 「set-caller-always-first 由别的目标逮住」：那两个 unlink 目标在这个变异下也全绿。

## 7 复跑核对与出处

- 编译与跑：每棵树各用自己的编译目录（`target`、`target-old`、`target-gen`、`target-mut`，都在草稿目录），编译用 `cargo test … --no-run`（不经内存包装，规则不要求），跑测试二进制的每一条经 `scripts/hosts/run-with-memory-cap.sh`（上限 2G 到 4G）与 `scripts/hosts/capped.sh 31`，同一时刻一条 cargo。第一条 8G 的包装排了 600 多秒没排上（本机内存账被别的会话占满），我停了它，改成编译不包装、跑起来的二进制包装 2G 到 4G；有一次五个 `unlink` 相关目标的 6G 请求同样排队，停了，改 2G 重跑。
- 复跑：`tools/rerun.sh` 我在草稿目录跑过三段（`RERUN_STAGES=existing`、`sweeps`、`mutants`），退 0；`sweeps` 与 `mutants` 两段的 `same_as_model_out` 全 yes，`generate` 一格第一次 no（模型目录里的 `out/generate.txt` 是我加「全0」标记之前存的），我换成复跑输出后重新生成了 `SHA256SUMS`；生成出的起点镜像 sha256 与草稿目录里原来的相同（`af28ade7…`），说明写路径做成的起点是确定的。`existing` 段打出两行「failed=… passed=…」，与 `out/existing-*.txt` 的末行一致。
- 一次全仓 `grep -l` 列出了别轮模型目录里的几个文件名（`research/prompts/m4-validate-code-r2-*-model/…`）；我没打开它们，也没读过 r2 两条腿的报告。判决 r2 里转述的数据我只当判决原文用。
- 开工快照：两份产品文件 sha256 与 `research/prompts/m4-validate-code-r3-snapshot/judged-sha256.txt` 两行逐字相同（上面第 0 节）。

## 8 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；上面「建议补」的变异行与取样点都没改仓。
- 没读第二轮两条腿的报告与模型目录，没读攻方腿第三轮的报告与草稿目录，没读 `records/`；T4 的数是我自己重造序列量的，不是复核它们的数。
- 没起 GPU，没跑 checker 档任何测试，没跑门禁阶段；GPU 恢复内核一格只读了码。
- 整理换下上段叶、回退这两条来路只读码，没造用例、没量。
- 没造「从未有过那个单元的外来镜像」做起点；两档起点各有残留红（第 2 节 T4-3）。
- 没跑全部 harness 测试；只跑了与标签 0、unlink 相关的六个目标（九条目标、with_a_positive_size、checker_known_bad_images、两个 unlink 目标，加扫描目标），都在已打甲的拷贝上；变异树上另跑了五个 unlink 相关目标（第 5 节最后一行的变异）。
- 没追查起点镜像里 I-1.13 在几串里消失的原因。
- 没改冻结副本、没改仓里任何文件；放宽截断下限的树（`src/relax-truncation-floor.patch`）与变异树只存在于草稿目录，不是产品改动的建议。

## 9 要主 agent 跑的命令与清理

无必须主 agent 跑的命令。想复核我的量，用第 0 节的复跑命令（经内存包装，约 15 分钟；`RERUN_STAGES` 可只点 `sweeps`、`existing`、`mutants` 其中几段）。

清理（交回前）：草稿目录 `~/.local/share/singlefs/work/m4-validate-code-r3-sonnet/` 里的仓副本与编译目录已删。删前 `du -sh`：

| 路径 | 大小 |
|---|---|
| repo | 39M |
| repo-old | 39M |
| repo-gen | 39M |
| repo-mut | 38M |
| target | 3.0G |
| target-old | 1.9G |
| target-gen | 1.8G |
| target-mut | 3.1G |
| rerun-sweeps（含它自己的拷贝与镜像） | 7.4M |
| images | 7.3M |
