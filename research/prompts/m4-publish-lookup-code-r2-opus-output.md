# m4-publish-lookup-code-r2：攻方腿报告（Opus，接续上一条被停的腿）（2026-10-21）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 公开出去的那一半是不是整理真正用的那一半 | 打中一处；另两问没打中 | 整理发批时核源落点那一步（`relocation_checked_against_the_version`）不经公开函数，按计划里每把 key 在 `mapped_units` 上线性找、再逐 key 查映射；探针在十六文件那一档的历史上命中。抽出前后产出逐项相同（补丁不动函数体，两档镜像摘要改前改后逐行相同）；公开之后没找到绕开整理不变量的路 |
| T2 增长判据数的是不是增长本身 | 打中 | 慢档用例的比较次数是测试在 `units` 上另建一个索引数出来的，解叶数读的是解过几片而不是解了几次；六条让产品退回平方或每 key 重解的变异（MA、MB、MC、MD、MF、MI）点名用例全绿、计数逐字不变（MB 慢档没跑完，我停了） |
| T3 快照判定那一圈换了查法之后的行为 | 没打中 | 首项、panic 文案与条件逐字相同；改前改后快照历史差分 14 行逐行相同；测试自己的留存条目断言能抓到 key 轮换（MH 红 9 条），判别力不只靠产品断言 |
| T4 新加的五行变异与换名的两行 | 打中两处 | 「每把 key 现建一个批量读者」那一行红只因为外层读者没人用、片数读 0，保住计数的同一代价（MF）不红；快照判定那一圈改回线性（MG）、批量读者每 key 重解（MC），没有一行红 |

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-code-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-code-r2/repo --work <新的草稿目录>
```

它把冻结副本拷成改后、变异、改前三份（改前 = `crates.patch` 反向打上），逐条打变异、debug 编两个测试目标、直接跑测试二进制，最后跑改前改后差分；每条 `name=` 行去掉 `finished in` 之后与 `out/rerun-expected.txt`（24 行）逐行比，末行 `name=rerun_compare <对上几行> of <几行>`。MB 不在复跑里（它让慢档用例跑十几分钟以上），只引 `out/runs-logs/` 里它停之前那份日志。预计挂钟约 35 分钟（按我这一次逐条量的挂钟加起来，推的）；这份脚本我没有整份跑过，只跑过 `bash -n`，里面每一步我都单独跑过（见各节原样输出）。

模型目录 `research/prompts/m4-publish-lookup-code-r2-opus-model/` 的文件（`SHA256SUMS` 逐个列了 40 个，它自己的 sha256 是 `ed1f8504f1b54decdc425fc9b18a846389bc2bc10a8b7bb0b4a3fe5797d5230a`）：

```
cd28a65fb0fa58520d41de4664317a1448755529aa03b330f47d285850e51381  ./crates.patch
cff7332e0ab87b20c40902741cebec2f5287e35afbedd0313e11adb38acc11b4  ./mutants.tsv
7ed9785b7f61b6c251fb979e2e96f4e9bbe08caaf3eed7c60f3eda8b9dca76b8  ./snapshot_differential_probe.rs
81aeb7d42b8f0c26a15201abf24d78bdb098f1c3dffb38ab9ed74b45497b482c  ./tools/apply_mutant.py
1332028917df55b7f32ab4cf0f37524ccfe80c7205120918cfbe469464e68e61  ./tools/rerun.sh
554e70ca1a8e80c1f47d844ea97c5474200e1e4d0612e95e2ca515a6bbe9e0de  ./tools/run_mutant.sh
```

`crates.patch` 是 `research/prompts/_m4-publish-lookup-code-r2-diff.md` 第 6–538 行（sha256 与那一段逐字节相同，上一条腿抽的、我核过）。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`；模板自带的 `--selftest` 判的是模板那一段 `rerun_steps`，换了之后它照设计会红，我没改它。

## 接续：上一条腿留下了什么、我用了什么

上一条腿留下的：`crates.patch`（核过与改动全文那一段 sha256 相同）、`mutants.tsv` 的前七行、两个工具脚本、草稿目录里的 `after/`（与冻结副本 `diff -rq` 无差）、`before/`（正向 `git apply --check` 通过，即已是反向打过的样子）、`mut/`（停在打着 MA 的样子）、编译目录、基线与 MA 的编译日志、基线两个目标的运行日志、MA 停在慢档用例的半份日志。都可用，没有重做；MA 的运行我重跑到完。我加的：三行变异（MF、MI、P1）、`run_mutant.sh` 的目标与过滤串选项、失败名排序、差分探针、`rerun.sh`、`out/`。
## T1　公开出去的那一半是不是整理真正用的那一半：打中一处，另两问没打中

### 打中：整理搬单元那一步不经公开函数，按计划里每把 key 另取一份映射

一串合法操作：`publish_lookup_by_identity` 十六个文件那一档的历史（按 inode 号写、整份重写四分之三、抬 F、整理到停），整理一发批就走到。路径：会话推整理「match advance_compaction_by_one_publish(」（`crates/singlefs-core/src/mounted_session.rs:2704`）；读版本那一处经公开函数「let read_of_the_version = version_as_the_compaction_reads_it(&version);」（`crates/singlefs-core/src/mounted_session.rs:3018`）；之后发批走 `publish_compaction_step`，逐项「relocations.push(relocation_checked_against_the_version(」（`crates/singlefs-core/src/transaction.rs:9368`），那里不用读版本时取到的条目，在上一版 `mapped_units` 上按 key 线性找「.find(|(_, key)| *key == mapping_key)」（`crates/singlefs-core/src/transaction.rs:9548`），再逐 key 查「let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {」（`crates/singlefs-core/src/transaction.rs:9551`），而逐 key 查取叶的单元又在 `units` 上线性找「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」（`crates/singlefs-core/src/transaction.rs:4316`）。

公开函数的文档注释写的是「/// 整理读一版时取映射条目的那一步：整理选段与搬单元读版本（`version_as_the_compaction_reads_it`）经它取，别处不调。」（`crates/singlefs-core/src/mounted_session.rs:2791`）。「别处不调」对函数本身成立；但「搬单元」这一步真正用来核源落点的那份映射条目不是它给的。

探针（副本上量的）：把第 9551 行那一次逐 key 查换成 panic（变异表 `P1-probe-relocation-reads-the-mapping-key-by-key`），跑十六文件那一条等价用例：

```
name=run mutant=P1-probe-relocation-reads-the-mapping-key-by-key test=publish_lookup_by_identity rc=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 2.04s failed=[every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units ]
thread 'every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units' (1711555) panicked at crates/singlefs-core/src/transaction.rs:9552:44:
探针：搬单元那一步（relocation_checked_against_the_version）不经 central_mapping_as_the_compaction_reads_it、逐 key 查映射
```

（panic 落在第 9552 行是因为变异把第 9551 行拆成了几行。）

先答四句：

1. 分不分辨臂：这一格问「整理用的映射条目是不是都经公开函数」，打中的是一条另取的路，与第一轮 T2（一遍走叶与逐 key 查是不是同一个答案）不是一件事。这条路用的就是逐 key 查（等价用例拿来当参照的那一个），所以它取到的答案不会与参照不同；它分辨的是覆盖与代价：七·6 的等价用例只核读版本那一份，这一份在 `publish_lookup_by_identity` 里没有任何断言看着；代价上每一批每个计划项是一次 `mapped_units` 上的线性找、一次 `units` 上的线性找、一片叶整片解，一批 K 项就是 K 倍表长，K 由批大小定（`CompactionBatchSize`），推的，没量。
2. 系统当时看不看得到：这是代码路径，不涉及判别子。
3. 字面分句：正文 T1「有没有别的路径另取一份（另一条整理入口、恢复后第一次整理、快照下的整理）」。另两种入口（挂载后续做活意图、快照下整理）走的是同一个 `advance_compaction_by_one_publish` 与同一个发批函数，读码看的，没另跑。
4. 改法：第一轮判决第三节三条都不碰这一处。我提的：发批核源落点时取读版本那一次已有的条目（或在发批里建一次 `mapped_units` 按 key 的索引与一个批量读者），只在我的模型上量过、被攻过零轮，推的（没实现、没跑）。

推翻条件：在第 9551 行换成 panic 之后，十六文件那一档的历史上整理一批都没发而用例仍红在别处，或者有一条整理发批的路径不经 `relocation_checked_against_the_version`。

### 没打中：抽出前后私有函数产出的东西逐项相同

- 读补丁：`mounted_session.rs` 那两段只改了函数签名、加了交回的结构体与新的私有函数头，原函数体（批量读者、`filter_map`、接留存条目）没有一行删改（`research/prompts/_m4-publish-lookup-code-r2-diff.md` 第 9–53 行那两段），次序与「留存条目接在后面」不变。
- 差分（副本上量的）：冻结副本的拷贝与它反向打上 `crates.patch` 的拷贝，两档等价用例打出的镜像摘要逐行相同（改前那一份 `out/differential-before/equivalence.log`，改后 `out/differential-after/baseline-publish_lookup_by_identity.log`）：

```
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
```

`diff` 两边按行排好比，打出 `images-identical`。

### 没打中：公开之后别处能不能拿它绕开整理的不变量

它只收 `&TransactionOutput`、交回自有的条目表与片数，不改版本、不碰分配器与盘；批量读者 `CentralMappingOfAVersionReadLeafByLeaf` 补丁之前已是公开的。读码看的，没有找到一条拿它绕开整理判定的路。
## T2　增长判据数的是不是增长本身：打中

### 打中的是什么

慢档用例数的两样都不在整理与发布真走的路上取：

- 身份比较次数：测试在整理之前，拿 `units` 另建一个自己的索引去查，包装身份（`IdentityCountingItsComparisons`）只包在这个测试自建的索引里。「let comparisons = comparisons_of_looking_up_every_identity(」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1018`），建索引那一句「FirstEntryOfEachIdentity::of_the_table(table, |entry| IdentityCountingItsComparisons {」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:922`）。整理本身在这之后才跑：「let run = session.run_compaction(devices).expect("整理一趟");」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1023`），它的结果只用来判这一次算不算数：「(!run.versions_published.is_empty()).then_some(comparisons)」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1028`）。产品里的三个索引按 `TransactionUnit` 建，测试的包装进不去；所以这个数只由 `FirstEntryOfEachIdentity::first_entry_of` 这一个函数体与表长定，产品调用点（照抄那一圈、批量读者、快照判定那一圈）怎么查都不进这个数。
- 解叶数：测试自己调公开函数「let leaves_decoded = central_mapping_as_the_compaction_reads_it(version).mapping_leaves_decoded;」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:397`），读的是「self.leaves_read.len()」（`crates/singlefs-core/src/transaction.rs:4413`）：解过的不同叶的片数，不是解了几次。文档注释「/// 到此刻解过几片叶：每片叶在第一次有 key 走到它时解一次、之后不再解，所以至多是这一版中央映射树的节点数。」（`crates/singlefs-core/src/transaction.rs:4408`）把「片数」当成「次数」，前提是 `or_insert_with`（`crates/singlefs-core/src/transaction.rs:4402`）只在第一次解；那一处改成每次都解，片数不变。

### 反例：七条改坏查法的变异，点名用例全绿（副本上量的）

变异只打在我的副本上（表 `research/prompts/m4-publish-lookup-code-r2-opus-model/mutants.tsv`），每条都让产品退回按表长线性或更差的查法，答案不变：

| 变异 | 改坏了什么 | `publish_lookup_by_identity`（连慢档） | 两个 growth_in_percent | 挂钟（基线 111.58s） |
|---|---|---|---|---|
| MA-index-unit-linear | `FirstEntryOfEachIdentity::unit` 在 BTreeMap 上逐项找（批量读者取叶、照抄取单元都走它） | ok. 6 passed | 118、110 | 199.26s |
| MB-carry-key-rebuilds-index-per-lookup | 照抄那一圈每查一个角色现建一次上一版 `mapped_units` 的索引（T2 题面举的「每次查都重建索引」） | 慢档之外 5 条 ok；慢档跑了 13 分钟以上没完，我停了它 | 118、（慢档没出数） | 慢档没跑完；快照目标 62.42s（基线 2.78s） |
| MC-leaf-redecoded-per-key | 批量读者每把 key 都把叶整片重解一次（`and_modify` 重解，表里仍一片一条） | ok. 6 passed | 118、110 | 132.58s |
| MD-mapping-key-of-linear | `mapping_key_of` 在 BTreeMap 上逐项找 | ok. 6 passed（快照目标 15 passed） | 118、110 | 215.20s |
| MF-fresh-reader-per-key-keeping-the-count | 每把 key 现建一个批量读者（每次重建 `units` 的索引、重解叶），外层读者照样把同一把 key 查一遍，计数照旧 | ok. 6 passed | 118、110 | 184.22s |
| MI-first-entry-of-walks-every-entry-without-comparing | `first_entry_of` 每查一次先把整张索引走一遍（不比身份），再有序查 | ok. 6 passed | 118、110 | 278.55s |

基线与每条变异的原样汇总行（`runs/<名字>-publish_lookup_by_identity.log` 的 `test result` 与 growth 行，副本上量的）：

```
baseline  growth smaller=ComparisonsOfLookups { lookups: 4852, comparisons: 65036 } larger=ComparisonsOfLookups { lookups: 19036, comparisons: 282512 } growth_in_percent=110
MA/MC/MD/MF/MI  同一行，逐字相同
name=run mutant=MA-index-unit-linear test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 199.26s failed=[] growth_in_percent=118 growth_in_percent=110
name=run mutant=MC-leaf-redecoded-per-key test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 132.58s failed=[] growth_in_percent=118 growth_in_percent=110
name=run mutant=MD-mapping-key-of-linear test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 215.20s failed=[] growth_in_percent=118 growth_in_percent=110
name=run mutant=MF-fresh-reader-per-key-keeping-the-count test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 184.22s failed=[] growth_in_percent=118 growth_in_percent=110
name=run mutant=MI-first-entry-of-walks-every-entry-without-comparing test=publish_lookup_by_identity rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 278.55s failed=[] growth_in_percent=118 growth_in_percent=110
name=run mutant=MB-carry-key-rebuilds-index-per-lookup test=publish_lookup_by_identity rc=143  failed=[] growth_in_percent=118
name=run mutant=MB-carry-key-rebuilds-index-per-lookup test=snapshot_release_and_destroy_through_the_deadlist rc=0 test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 62.42s failed=[]
```

挂钟只当旁证（本机同时有别的会话在编译、跑测试，变异逐条跑、没有交替），判打没打中只看点名用例绿不绿与两个计数逐字不变。MB 的退出码 143 是我用 `proc.py stop` 停了慢档那个测试二进制（跑了约 13 分钟、CPU 11 分 27 秒）；停之前同一份日志里另外五条都已 ok。

### 先答四句

1. 分不分辨臂：这一格的两条臂是「产品的按身份查亚线性」与「产品退回平方」。六条变异都落在平方那一臂，判据两臂都放行，所以它分不出产品的两臂；它分得出的只有 `first_entry_of` 函数体本身被换成比较身份的逐项找（登记的那一行）与公开函数里换了读者（登记的「整理读版本退回逐 key 查映射」）。
2. 被判的系统当时看不看得到判别它的东西：看不到。产品的索引键是 `TransactionUnit`，测试的计数包装进不了产品的三个调用点；批量读者只露出片数，不露出解了几次、建了几次索引。这是「判别子观测不到」那一形（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`「**判别子观测不到**」），要改判据。
3. 满足的是字面哪一个分句：正文 T2「数的那两样在改坏查法时一定涨，在改成别的同样慢的写法（例如每次查都重建索引、索引按插入序线性找）时也涨不涨」——MB 就是「每次查都重建索引」，两样都不涨。
4. 跑前条款给的改法在这几格上还中不中：第一轮判决第三节第 2 条给的是「改成数次数（攻方丙：每批整理解了几片叶、按身份查了几次）……要在产品里加计数的，写明加在哪、只在哪个特性或测试构建下开」。实现取了「测试里数、不在产品里加计数」那一种，于是数的是测试自建索引的比较次数与片数；在 MA、MB、MC、MD、MF、MI 这六格上照样放行（量过，见上表）。「在产品里加计数」那一种没实现，下面另提一个形态，推的。

### 我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪几格 | 标 |
|---|---|---|
| 在产品里数三样，只在测试构建开：`CentralMappingLeafRead::of_the_leaf_bytes` 被调了几次（不是 `leaves_read.len()`）、`FirstEntryOfEachIdentity::of_the_table` 被调了几次、三个查法（`first_entry_of`、`unit`、`mapping_key_of`）每次查走过的索引项数；慢档用例读整理那一趟 `run_compaction` 前后这三样的差，不再在测试里另建索引 | MC、MF（解叶次数涨）；MB、MF（建索引次数涨）；MA、MD、MI（走过的项数涨） | 推的（没实现、没跑） |
| 只把解叶数改成「解了几次」（计数放在 `or_insert_with` 的闭包里） | MC、MF | 推的 |
| 维持现状 | 无 | — |

推翻条件：在产品里加了上面那三样计数之后，MA、MB、MC、MD、MF、MI 任一条点名的慢档用例仍绿，就说明这一节的改法碰不到那一格。

### 门槛会不会被别的因素推过线或压下线

没打中。两样计数在同一份副本上逐次相同（基线日志里小池八次读数是 `lookups: 606, comparisons: 8121` 与 `lookups: 607, comparisons: 8138` 交替，大池是 2379/35306 与 2380/35322 交替），不随负载与构建档变；表长四倍时有序表多一两层，读数 110%、118%，离 200% 远，离逐项找的约 400% 也远。漂移不是这个判据的问题，问题是它量的不是产品。
## T3　快照判定那一圈换了查法之后的行为：没打中（另记两条观察）

### 行为

- 现建还是复用：每次进 `deadlist_changes_of_this_publish` 建一次「let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();」（`crates/singlefs-core/src/transaction.rs:5729`），借着 `previous_version` 的不可变引用，循环里改不了上一版的 `mapped_units`（借用检查守着，读码）。
- 同一角色出现不止一次：索引建表时 `or_insert` 留最先那一项，与旧的 `iter().find` 同一项；`.to_vec()` 与旧的 `key.clone()` 字节相同（读码；第一轮已判过照抄那一圈的同一件事，这里前提没变）。
- 查不到时的 panic：新旧两边都是「.expect("照抄进来的角色在上一版里进映射、有一把 key")」（`crates/singlefs-core/src/transaction.rs:3308`），文案逐字相同；条件都是「上一版 `mapped_units` 里没有这个角色」。只差 panic 落在哪个文件行。
- 差分（副本上量的）：在改前、改后两份拷贝的快照测试文件末尾接同一段探针（`research/prompts/m4-publish-lookup-code-r2-opus-model/snapshot_differential_probe.rs`：建 S1、覆盖写、建 S2、覆盖写两次、删 S1、覆盖写、删 S2、再覆盖写），每一版的内存版本（Debug 全文，含单元字节、deadlist 条目、留存的映射条目）与录制写流各取一个摘要。两边 14 行逐行相同（`diff` 退 0，打出 `probe lines identical: 14`）；节选（改后那一份原样）：

```
attack_probe step=overwrite 37 txg=7 deadlist_entries=8 kept_mapping_entries=8 version_digest=25e350bbe31be99c version_debug_bytes=815267
attack_probe step=destroy S1 batch 0 txg=10 deadlist_entries=4 kept_mapping_entries=4 version_digest=7643b90848b2fefe version_debug_bytes=817356
attack_probe write_stream operations=441 digest=658175b19d1f276d debug_bytes=64614
attack_probe retained_stream digest=05c7d97d622c45ea debug_bytes=15160665
```

### 测试的判别力是不是只靠产品断言：不是

我的变异 `MH-deadlist-key-of-the-next-replaced-role` 让每个换下的角色取下一个换下角色的 key（四把 key 轮换，deadlist 里 key 的集合不变，产品不会撞上同 key 两条）。点名用例红在测试自己的断言上：

```
thread 'an_overwrite_under_a_snapshot_keeps_every_replaced_user_visible_unit_on_the_deadlist_and_releases_the_pool_metadata' (1534959) panicked at crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:242:9:
Data(DataUnitIndexInFile(0))：映射条目照原来的两条位置项留存
```

整个目标 15 条里红 9 条（`out/runs-logs/MH-deadlist-key-of-the-next-replaced-role-snapshot_release_and_destroy_through_the_deadlist.log`）。所以实现员说的「索引键改错那条红在产品断言上」只说明登记的那一种改坏恰好先撞上产品断言；换一种不撞的，测试自己比留存条目的 key 与位置配对照样红。只比 deadlist 的 key 集合那一句（`crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:222`）单独看抓不到轮换。

### 两条观察（不改行为，推的）

- 这一处只换了取 key 那一半；同一个角色进 deadlist 时取位置项仍走「match mapping_locations_of_a_mapped_unit(previous_version, *identity) {」（`crates/singlefs-core/src/transaction.rs:5748`），那是线性找，属正文第二节「已知、报出来不算打中的」那一项。
- 索引在函数里无条件建：这一次没有换下的用户可见角色时，旧代码不走表，新代码也要走一遍 `mapped_units` 建一棵 BTreeMap。每次发布多一次建索引，量级与照抄那一圈建的那一次相同；没量。
## T4　新加的五行变异与换名的两行：打中两处

### 五行追加变异各自凭什么红

| 行（`crates/mutations.tsv`） | 改坏的是 | 点名用例凭什么红 | 标 |
|---|---|---|---|
| 第 5063 行「隔一把 key 才查映射」 | 结果（漏一半条目） | 等价用例「central_mapping_as_the_compaction_reads_it(version).mapping_entries,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:507`）那一处逐项比参照 | 推的（没在副本上跑） |
| 第 5064 行「漏查第一把 key」 | 结果 | 同上 | 推的 |
| 第 5065 行「每把 key 现建一个批量读者」 | 性能（答案不变） | 外层读者不再被用，它的片数是 0，红在解叶数那一句，不是红在答案或代价上（见下） | 量过 |
| 第 5066 行「索引从大到小逐项找」 | 性能 | 改的是 `first_entry_of` 函数体（`crates/singlefs-core/src/transaction.rs:3282`），快档合成表用例自建的索引走的就是它，比较次数按表长涨 | 推的 |
| 第 5067 行「快照判定那一圈的映射 key 一律取 inode 根那一把」 | 结果 | 实现员报告说红在产品断言上；换一种不撞产品断言的改坏，测试自己的断言也红（T3 那一节 MH） | MH 量过，登记那一行推的 |

换名的两行（第 4973、4974 行）点名的都是改名后的慢档用例 `compaction_batches_decode_each_routed_mapping_leaf_once_and_compare_identities_sublinearly_from_512_to_2048_data_units`（`grep -n "^七·6" crates/mutations.tsv` 第 6 列），点得到。两行各自改的是测试直接调到的那两处（`first_entry_of` 函数体、公开函数里的 `filter_map`），所以红；这正是 T2 那一节说的：判据只看得到测试自己调的那两处。

### 打中一：「每把 key 现建一个批量读者」那一行红在计数挂错了读者上，不在它要抓的代价上

登记那一行（我的表里同样的一行叫 `ME-registered-reader-per-key`）在副本上跑十六文件那一条：

```
name=run mutant=ME-registered-reader-per-key test=publish_lookup_by_identity rc=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.07s failed=[every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units ]
thread 'every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_sixteen_files_of_four_data_units' (1876803) panicked at crates/singlefs-harness/tests/publish_lookup_by_identity.rs:399:5:
assertion `left == right` failed: txg 6：整理读这一版解了 0 片映射叶，进映射的 10 把 key 走到 1 片（每片解一次、每把 key 都经批量读者查）
```

同一个代价、只多让外层读者把同一把 key 也查一遍（`MF-fresh-reader-per-key-keeping-the-count`：每把 key 仍现建一个读者、重建 `units` 的索引、重解叶），点名的十六文件用例与慢档用例全绿，见 T2 那一节表里 MF 那一行（`finished in 184.22s`，基线 111.58s）。四句：分辨臂——分不出「每 key 一个读者」与「一个读者」这两种写法，只分得出外层读者有没有被用；判别子——测试看得到的只有外层读者的片数；字面分句——正文 T4「「每把 key 现建一个批量读者」那一行改坏的是性能还是结果，点名用例凭什么红」；改法——T2 那一节的「数解叶次数、数建索引次数」两条都修这一格，推的。

### 打中二：第二批的改法改回去，没有任何一行变异红

| 第二批的改法 | 改回去的样子（我的变异） | 跑了什么 | 结果 | 标 |
|---|---|---|---|---|
| 快照判定那一圈改走索引（`crates/singlefs-core/src/transaction.rs:5729`） | `MG-deadlist-loop-reverted-to-linear`：恢复成在 `mapped_units` 上 `iter().find`、同一句 expect 文案 | 快照目标 15 条 | `test result: ok. 15 passed`；`publish_lookup_by_identity` 里没建快照，这一圈每次发布都走，但每个角色都判进 defer，取哪把 key 都不改结果 | 量过 |
| `leaves_decoded` 的含义「每片只解一次」 | `MC-leaf-redecoded-per-key`：每把 key 都重解叶 | `publish_lookup_by_identity` 连慢档 | `ok. 6 passed`，两个 growth 仍是 118、110 | 量过 |
| 公开函数的两个字段 | 把字段改回不交出（整份反向）会让测试编不过，算红 | 没跑 | — | 推的 |

MG 是等价变异，结果抓不到是应该的；能抓它的只有代价判据，而增长判据看不到这一圈（T2）。第一轮判决第三节第 3 条要的「带一条会红的变异」，实现给的是一条改结果的变异（第 5067 行），它守的是「key 取对」，不守「查法是索引」。

推翻条件：在主工作区上对 MG、MC 两条跑变异轻型，点名用例红。
## 没打中的形状

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 改前改后两档等价用例的镜像摘要（T1 抽出前后、整理读版本） | 两段固定历史（十六个四单元文件、四十八个八单元文件），各 24、44 批整理 | 逐行相同 |
| 快照下的一串操作，改前改后每一版的内存版本与录制写流（T3） | 一段固定历史：两个快照、五次覆盖写、两次删快照（各一个意图加一批），共 11 版、441 次写 | 14 行逐行相同 |
| 快照判定那一圈取错 key（MH，四把 key 轮换） | 快照目标 15 条 | 红 9 条，测试自己的断言抓到 |
| 增长门槛被负载或构建档推过线、压下线 | 慢档用例两档各 8 次读数，基线与五条变异 | 计数逐次相同、只由表长定；不随负载变 |
| 公开函数被别处拿去绕开整理的判定 | 读码：它的签名与 `crates/` 里的调用点 | 没找到 |

没在随机历史上做差分：第一轮攻方做过（223 段随机历史），第二批对行为的改动只有快照判定那一圈，我只在一段快照历史上差分，崩溃状态也没有枚举（见下）。

## 这条腿自己的限度

- 只抽一次样（`.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算`）：T3 的「没打中」与 T1 另两问的「没打中」都只这一次。
- 崩溃状态没有枚举：差分比的是每一版的内存版本与整条录制写流，两边写流逐条相同，崩溃状态集合由写流定，这一步是推的；没调 `enumerate_crash_states`。
- 副本上的数不算入库装置上的数；变异全打在我的副本上，没进 `crates/mutations.tsv`，没跑仓里的变异驱动。
- 挂钟只当旁证：本机同时有别的会话在跑 `cargo test`（开工时 `ps` 看到另一条 `cargo test --release -p singlefs-checker-tier` 与一条多包 `cargo test`），我的变异逐条串行跑、没和基线交替。
- T1 那一条搬单元的路径的代价（每批 K 倍表长）是推的，没量；挂载续做与快照下整理两种入口走同一个发批函数，读码看的，没另跑探针。
- `rerun.sh` 没整份跑过（见「复跑」一节）。
- 登记的五行追加变异里，我只在副本上跑了「每把 key 现建一个批量读者」那一行；另四行「凭什么红」是读码推的。

## 没做什么

- 不判正推那一格的立场，不替主 agent 采纳。
- 没跑门禁阶段、checker 档测试与任何重型测试；没跑变异轻型（MG、MC 两条要在主工作区上判，命令见下）。
- 没起 GPU，没去第二台。

## 要主 agent 跑的命令

1. 复跑本报告的数：见「复跑」一节那一条，内存 16G，线程由脚本里的 `capped.sh 31` 定，跑完看末行 `name=rerun_compare` 是不是 `24 of 24`，`name=differential` 一行是不是 `before_vs_after=identical lines=15`。
2. 若要把 MG、MC、MF 三条（或其中几条）当成入库的会红检查：先按 T2 那一节的改法改判据，再把它们加进 `crates/mutations.tsv` 跑变异轻型（`bash scripts/hosts/run-with-memory-cap.sh <上限> bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <根> --item <项>`，带 `SINGLEFS_RUN_SCOPE` 声明），点名用例要红。改判据之前跑它们，按本报告的读数会是没红。
