# m4-publish-lookup-code-r3 攻方腿报告（Opus，2026-10-22）

立场：在冻结副本的拷贝上造反例（正文第四节「云端攻方」那一行）。只攻 T1–T4，前两轮攻过的角度（随机历史差分、快照判定圈差分、整理读版本读漏、增长判据量 CPU 时间、测试另建索引来数）不重复，除非第三批改了它们的前提。所有数都是在草稿目录 `~/.local/share/singlefs/work/m4-publish-lookup-code-r3-opus/` 里冻结副本的拷贝上量的，不是入库装置上的数。

## 复跑命令与模型文件

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-code-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo \
  --work <新的草稿目录> --model research/prompts/m4-publish-lookup-code-r3-opus-model
```

它拷冻结副本成改后、改前（第三批 `crates.patch` 反向打回，补丁取自 `research/prompts/_m4-publish-lookup-code-r3-diff.md`）、变异、特性四棵树，依次跑：改前改后差分 → 覆盖 → 改前两档等价用例的镜像摘要 → 基线与 10 条变异（各跑 `publish_lookup_by_identity` 全部六条连慢档）→ cargo tree 与判法单元键。打出的 `name=` 行与 `out/rerun-expected.txt` 逐行比，末行 `name=rerun_compare <N> of <M> lines …`。挂钟约 45 分钟（单条 cargo、31 线程、16G）。

模型目录 `research/prompts/m4-publish-lookup-code-r3-opus-model/` 每个文件的 sha256：

```
5bb840ab7c0378be9ffe47b088f531c4f395e6048e46cd9923ee209b21c655df  ./mutants/mutants.tsv
e268854c66f644ba288052ea6410781f1b85181f24a5e9c34248ebb7c514f248  ./out/rerun-expected.txt
bd7b6be0abdb07cf9bcc94b964c2e07c5c8707f1e6b45b257f6fb182733aa999  ./tests/r3_opus_coverage.rs
10b60519fb7d138501b43e7b32b5a6c26de3938e47a0f0bdc60fe8c075d9e737  ./tests/r3_opus_differential.rs
30d3e648bce770eb03cd8188171598cf67d1bc0d9bd97f98b64c86992edac299  ./tools/apply_mutant.py
ffe9faaa4c6122a91342342b34e9dea925ac006906aa341134e22ca38e888961  ./tools/make-expected.sh
2ac8da3f6006f257e56e786d99419a911b9c60f900de55733d84aab68a8f58dc  ./tools/rerun.sh
667eb270e3d9fc17dfd85a4f7cecb2b0f325d04816cdbfda78300980b244eeb3  ./tools/run_mutant.sh
```

`out/rerun-expected.txt` 是 `tools/make-expected.sh` 从各次单独跑的草稿日志拼的（与 rerun.sh 同一套取法），不是拿 rerun.sh 的输出回填的；rerun.sh 整条跑过一次的结果在报告末尾「整条复跑」一节。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 新索引下前两轮判成立的还成立吗 | 没打中 | 192 段随机历史（六组比重 × 两种盘宽 × 16 个种子 × 40 步）改前改后每一步镜像摘要、结局、录制流步数逐行相同；两档等价用例改前改后镜像摘要相同；身份类型的 `Ord`/`Eq` 全是派生 |
| T2 并进来的两处查法 | 行为上没打中；判别力打中（并入 T3、T4） | 同上那组差分里 46/48 段走到释放三处与快照判定圈，6/48 段走到整理发批核源落点；两档等价用例 24、44 批镜像相同。查不到时的 panic 文案逐字照抄 |
| T3 计数判的是不是产品调用点上的事 | **打中** | ① 两处查法各「一半改回去」（落点退回逐 key 查、键的索引照用：RA、RB）六条全绿——逐 key 查里那次 `version.unit` 线性找不经计数；② 「每项现建索引」三条（RC、RD、RE）全绿——建索引次数的增长判据只看两档之比，慢档用例里整理每次只搬 1 个单元，K 定长的几处每项重建在两档上同倍涨；③ 释放三处合记一处，只改回其中一处（RF）全绿；④ 第 9986 行整理发批核源落点还有一处按角色线性找（`previous.unit(role)`），不经计数 |
| T4 特性与变异 | **打中** | ① 计数本身可被改坏而不红（RG：走索引不记看过几项），叠上第三批追加表的 MA 之后 MA 不再红（RG+MA 六条全绿，MA 单独 3 红）；② 判法单元键：checker 档构建脚本只读自己包的特性，连 harness 测试一起编时链进来的 `singlefs-core` 开着计数（cargo tree 量的），而键照「不开」算（unit-graph 量的：开与不开两把键不同）——一起编出的二进制顶着单独编的键，属「错误复用」那一侧，不是实现员报的「白白现判」 |

自己提的改法都只在我的模型上推过、没实现，被攻过零轮（第「改法」一节）。

## T1　新索引下前两轮判成立的还成立吗：没打中

试过的形状与读码：

- 取最先一项：`crates/singlefs-core/src/transaction.rs:3445`「pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);」排在稳定排序之后，`dedup_by` 丢的是后一项；身份类型（`TransactionUnit`、`InodeNumber`、`&[u8]`）的 `PartialEq`/`Ord` 在 `singlefs-core`、`singlefs-format` 里没有手写实现（`grep -rn 'impl\(<[^>]*>\)\? \(Partial\)\?\(Ord\|Eq\) for' crates/singlefs-core/src crates/singlefs-format/src` 命中 0 行），派生的 `==` 与 `cmp == Equal` 一致，排序、去重、折半查三处用的判等是同一件事。
- 折半查：`crates/singlefs-core/src/transaction.rs:3456`「let found = loop {」那一圈在空表（`low >= high` 直接出圈）、单项、查不到时都交 `None`，与改前 `BTreeMap::get` 相同；索引里身份严格升序、不重复，不存在「全同身份」的索引。
- 差分（量的）：`tests/r3_opus_differential.rs` 在改前、改后两棵拷贝里各跑同一组随机历史（比重 snapshots、compaction、compaction-rollback、broad-compaction、truncation、inode-writes，盘宽 4 GiB 与单元区 240 槽，种子 0–15，每段 40 步，不跑每步池级 checker），每段打一行结局、结局条数、录制流步数、每一步镜像（两块盘写过的扇区）摘要串起来的 sha256。草稿里两棵各跑一次：

```
before rc=0
before wall=315.30
192
after rc=0
after wall=334.80
192
identical
    192 ending=completed outcomes
```

  （`diff` 两边 `name=` 行 192 行逐行相同，192 段全部 completed。）
- 两档等价用例（`publish_lookup_by_identity` 的十六文件、四十八文件两条，整理批大小 2）改前改后打出的镜像摘要（量的，改后取自基线那一趟，改前另跑）：

```
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
```

  两棵树打出的这两行逐字相同。
- 查不到时的 panic：改前 `mapping_locations_of_a_mapped_unit` 的 `.expect("进映射的单元每个一把 key（…）")` 原样搬进 `crates/singlefs-core/src/transaction.rs:4612`「let (_, key) = mapped_units_by_role」下一行；逐 key 查与批量读者解叶时找叶单元用的是同一个常量 `EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION`。只有 panic 的位置（文件:行）变了。

什么会推翻「没打中」：一段合法历史在两棵树上打出不同的镜像摘要或结局；或一种身份类型的 `==` 与 `Ord` 不一致（今天没有手写实现，加一个就要回来看）。

## T2　并进来的两处查法：行为上没打中

- 覆盖（量的，改后那棵，差分同一组比重与盘宽、种子 0–3 共 48 段，`tests/r3_opus_coverage.rs` 读 `lookup_work` 计数）：

```
CompactionReadsTheCentralMapping builds 180
CompactionReadsTheCentralMapping leaf_decodes 181
CompactionReadsTheCentralMapping lookups 181
MappingEntriesOfReleasedUnits builds 20066
MappingEntriesOfReleasedUnits leaf_decodes 10070
MappingEntriesOfReleasedUnits lookups 102352
RelocationSources builds 86
RelocationSources leaf_decodes 32
RelocationSources lookups 128
SnapshotJudgement builds 6340
SnapshotJudgement leaf_decodes 245
SnapshotJudgement lookups 33906
histories with >0 lookups per site: {'SnapshotJudgement': 46, 'MappingEntriesOfReleasedUnits': 46, 'CompactionReadsTheCentralMapping': 24, 'RelocationSources': 6}
```

  快照判定那一处解了 245 次叶：只有进 deadlist 那一支的 `locations_of_role` 解叶，所以那一支真走到了。整理发批核源落点只在 6/48 段里走到、共 128 次查，薄；两档等价用例补上 24 批与 44 批（批大小 2），镜像同上逐字相同。
- 同一 key 几个角色、同一角色几把 key：`mapped_units_by_key` 与 `mapped_units_by_role` 都走同一个「稳定排序后留最先一项」的建法，与 `iter().find` 取最先那一项同义（T1 第一条）；产品里 `mapped_units` 由发布按角色逐个装、key 由角色定，我没造出同一 key 两个角色的合法历史（差分 192 段里没有一段打出不同）。
- 建了索引之后表会不会被改：`MappedUnitsOfAVersionReadByKey`、`MappedUnitsOfAVersionReadByRole` 都借着 `&'version TransactionOutput`，借用期间那一版改不了（编译器守）。
- 改前改后 panic 与错误：`ReleaseNotInMapping`、`MappingEntryNarrowerThanItsFieldTable` 两处 `return Err` 的条件原样照抄，只换了取落点的那一个调用：`crates/singlefs-core/src/transaction.rs:6071`「let locations = match previous_version_read_by_role.locations_of_role(*identity) {」。整理发批那一处多了一步：第一项计划到来时就建按 key 的索引与批量读者（建索引不会 panic），之后的查与改前同序。

什么会推翻：一段走到整理发批的历史改前改后镜像不同。这一格的取样里整理发批只有 6 段随机历史加两档等价用例，见「这条腿自己的限度」。

## T3　计数判的是不是产品调用点上的事：打中

### 变异（量的，草稿里改后那棵拷贝上逐条单独打，各跑 `publish_lookup_by_identity` 全部六条连慢档）

变异表 `mutants/mutants.tsv`（原文须恰好命中一次，`tools/apply_mutant.py` 打）。每条一行，取自 `tools/run_mutant.sh` 打出的原样（`finished in` 的秒数是后来才从汇总行里去掉的，原始日志里有）：

```
name=mutant mutant=baseline rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RA rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RB rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RC rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RD rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RE rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RF rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
```

| 变异 | 改了什么（都只改 `crates/singlefs-core/src/transaction.rs`） | 结果 |
|---|---|---|
| RA | 整理发批核源落点：按 key 的索引照用，落点退回逐 key 查（`central_mapping_locations_in_the_version(previous, &mapping_key)`） | 六条全绿 |
| RB | 按角色读者 `locations_of_role`：按角色的索引照用，落点退回逐 key 查（释放三处与进 deadlist 那一支都经它） | 六条全绿 |
| RC | 整理发批：`get_or_insert_with` 换成 `insert`，计划里每一项现建按 key 的索引与批量读者 | 六条全绿 |
| RD | 按角色读者：`get_or_insert_with` 换成 `insert`，每查一个角色现建按角色的索引与批量读者 | 六条全绿 |
| RE | 快照判定那一圈：每个换下的角色现建一份 `mapped_units_by_role()` | 六条全绿 |
| RF | 释放三处之一（核位置项指池里两块不同的盘）退回「线性找 key、逐 key 查映射」，另两处照走索引 | 六条全绿 |

RA 与第三批追加表最后一行（「整理发批核源落点退回按 key 线性找角色、逐 key 查映射」，两半一起改回）的差别只在 key 那一半照走索引：那一行红是因为核源落点那一处的查找次数掉到 0，落点那一半改不改它看不见。RB 对追加表倒数第二行同理。

### 为什么不红（读码，各指到许可它的那一句）

1. **逐 key 查里的找叶单元不经计数**。`crates/singlefs-core/src/transaction.rs:4636`「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」里的 `version.unit` 是 `crates/singlefs-core/src/transaction.rs:4056`「.find(|unit| unit.identity == identity)」那一次线性找，不走 `SortedIdentityIndex`，计数一项都不记；逐 key 查只记一次 `mapping_leaf_decodes`。测试只在「整理读一版」那一处判解叶次数，核源落点、释放三处、进 deadlist 那一支的解叶次数没有一条断言。所以 RA、RB 每查一把 key 在 `units` 上走一遍（表长 P），计数里只多出几次解叶，没有一项看过的索引项。
2. **建索引次数的增长判据只看两档之比**。`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1227`「larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);」判的是每一趟整理建索引次数从小池到大池涨多少；失败文案 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1239`「每查一把 key 现建一个索引就按 key 数涨」只对「每一趟查的 key 数随池长」的那几处成立（整理读一版）。核源落点每批查 K 项、释放与快照判定每次查「这次换下的角色」，都不随池长，K 由 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1137`「const COMPACTION_BATCH_UNITS_OF_THE_GROWTH_POOLS: usize = 8;」定死，每项重建在两档上同倍涨，比值不动。
3. **慢档用例里整理几乎不搬**（量的，只改测试的探针 PR：`run_compaction_and_count_its_lookups` 每一趟打出所在线程、核源落点那一处的查找次数、建索引次数、搬的单元数、批数与版数）：

```
     36 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=2 relocation_lookups=3 relocation_builds=2 batches=1 versions=1
     30 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=0 relocation_lookups=0 relocation_builds=0 batches=0 versions=1
      8 probe_relocation thread=growth-pool-thread units_relocated=1 relocation_lookups=2 relocation_builds=2 batches=1 versions=1
      8 probe_relocation thread=growth-pool-thread units_relocated=0 relocation_lookups=0 relocation_builds=0 batches=0 versions=1
```

   慢档用例两档各 8 趟（线程名是 `growth-pool-thread` 的那 16 行）：8 趟每趟只搬 1 个单元，另 8 趟一个都不搬（只发意图那一版）。K = 8 的批从没满过，RC 的「每项现建」在慢档上等于没改；`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:247`「relocation_sources.index_lookups >= units_relocated,」在不搬的那 8 趟是 0 ≥ 0。慢档读数的大头不在核源落点：基线两档合计 15064 + 59033 次查，核源落点那一处两档合起来 8 趟 × 2 次 = 16 次（读数见上，大头推的是照抄全部单元那一圈，没逐处拆）。
4. **三处释放合记一处、断言取「至少」**。`crates/singlefs-core/src/transaction.rs:4302`「let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);」与第 4816、4891 行三处进同一个 `LookupSite`；测试 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:359`「work.index_lookups >= data_units_replaced,」要的是三处合起来 ≥ 换下的单元数，一处就够，RF 改回一处照过。

### 不经 `lookup_work` 的按身份查（读码，没跑）

计数只数经 `SortedIdentityIndex` 的 `get` 与逐项走（`crates/singlefs-core/src/transaction.rs:3496`「count(|work| work.index_entries_examined += 1);」）。发布与整理路径上还有这些按身份线性找，计数看不见：

| 位置（冻结副本） | 在哪一处里 | 每次调用的代价（推的） |
|---|---|---|
| `crates/singlefs-core/src/transaction.rs:9986`「RelocatedUnitHolder::RoleOfThisVersion(role) => previous.unit(role).bytes.clone(),」 | 整理发批核源落点（`RelocationSources`）：计划里每一项取单元字节 | 每批 K 次在 `units` 上线性找，K × P；同一处的批量读者手里已有 `units_by_role` 索引，这一行没用它 |
| `crates/singlefs-core/src/transaction.rs:4636`「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」 | 逐 key 查（`central_mapping_locations_in_the_version`），第三批之后产品里还调它的地方不再在点名几处里，但 RA、RB 这类改回去就走它 | 每把 key 一次 P |
| `crates/singlefs-core/src/transaction.rs:4308`「.filter(|identity| !kept_for_the_snapshots.contains(identity))」（第 4742 行同一句） | 释放判定路径、释放三处挑角色 | 换下的角色数 × 留给快照的角色数 |
| `crates/singlefs-core/src/transaction.rs:3972`「.find(|file| file.inode == inode)」（`file_other_than_the_first`，经 `has_a_placement_in_the_previous_version` 每个换下的角色调一次） | 释放判定路径 | 换下的角色数 × 文件数 |
| `crates/singlefs-core/src/transaction.rs:6116`「.find(|(key, _)| *key == entry.mapping_key)」 | 删快照那一批释放 deadlist 条目 | 释放的条目数 × 留存映射条目数，两者同长时是平方 |
| `crates/singlefs-core/src/transaction.rs:9917`「.find(|(key, _)| *key == mapping_key)」 | 整理发批，计划项是留给快照的映射条目时 | 每项一次留存条目表长 |
| `crates/singlefs-core/src/transaction.rs:11854`「let carried = previous_version.unit(role_in_the_previous_version);」 | 照抄 extent 树节点 | 每个照抄的节点一次 P |

第一行最要紧：它就在用户 2026-10-21 定并进的那一处里，第三批换了 key 与落点两步，第三步取字节没换。改前那一处每批是「找 key 线性 + 逐 key 查里找叶线性 + 取字节线性」三个 K × P，改后是「建两个索引 2 × P log P + 取字节 K × P」：K = 8、P 在 600–2400（慢档读数里 `largest_index_looked_up`）时，改后每批的比较次数不见得比改前少（推的，没量）。七·6 那一行要的是这一处随表长的那部分去掉，这一句没去掉，计数也看不见它。

### 线程与并行

- 产品代码不起线程：`grep -rn 'thread::\|rayon\|spawn(' --include=*.rs crates/singlefs-core/src` 命中 0 行，与 `crates/singlefs-core/src/transaction.rs:3258`「/// 什么都不做，发布构建不多出开销。计数记在本线程上：核心层不起线程，一次发布或一趟整理的查找都在调它的那个线程里做；」相符。
- 用例并行：计数在线程局部变量里，libtest 每条用例一个线程，慢档两档各一个 `scope` 线程，`lookup_work_while` 先挪开此前的计数、量完加回，套着量也各量各的。上面那组差分与覆盖我用 31 个线程并行跑同一个二进制，覆盖读数每段各自成数，没见串数（量的只是「读数非 0 且各段不同」，没有专门造串数的反例）。没打中。

### 门槛 200% 在登记规模上

基线两条增长读数 124%、123%，建索引次数 100%（量的，上面 baseline 那一行）。把查法整个退回逐项找（第三批替换表「索引退回逐项找」那一行）按表长涨，离 200% 远；我造的几条都没碰到门槛，它们不红不是因为离门槛近，是因为计数里根本没有它们做的那份事（前一小节）。没找到被别的因素推过线或压下线的形状。

### 打中之后的四句（照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. 分不分辨臂：这一格没有臂之分，判的是「测试判不判得出第三批改法被改回」；RA–RF 每一条都是第三批某一处改法的一半或一种退化，测试一条都不红，分辨得出「判别力够 / 不够」。
2. 被判的系统当时看不看得到：看得到——RA、RB 多出的是 `units` 上的线性找，RC–RE 多出的是建索引，产品在那一刻都在做这件事，只是计数不记（第 1 条）或判据只看比值（第 2、3 条）。这不是「判别子观测不到」，是计数点与判据没放到那里。
3. 满足的是判据字面的哪一句：第二轮判决第三节第 1 条「增长判据与变异要判到产品调用点」与正文 T3「改回线性、每次重建索引、每把 key 重解叶这几类改坏，计数是不是一定涨」——RA、RB 属「改回线性」，RC–RE 属「每次重建索引」：计数里建索引次数确实涨了（RC 每批 2 → 2K），但测试判的是两档之比，没有一条红。
4. 跑前条款给的改法在打中的格上还中不中：第二轮判决给的改法「在产品里数……测试改成读这些计数判增长」已经实现，在这几格上照样不红；要中，计数要覆盖 `TransactionOutput::unit` 这类线性找，判据要加每处的绝对上界（见「改法」一节）。

## T4　特性与变异：打中

### 计数自身可以改坏而不红（量的）

```
name=mutant mutant=RG rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RG+MA rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=124 index_builds_growth_in_percent=100
name=mutant mutant=MA rc=101 test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out failed=[compaction_batches_decode_each_routed_ma,every_lookup_by_identity_agrees_with_a_l,every_lookup_by_identity_agrees_with_a_l] growth_in_percent=124
```

RG 删掉 `crates/singlefs-core/src/transaction.rs:3496`「count(|work| work.index_entries_examined += 1);」（从前往后走索引那一个 `next`）。产品今天不在索引上逐项走，所以单独删它六条全绿；但第三批追加表里 MA、MD、MI 三行（以及替换表「索引退回逐项找」）红，全靠这一句把逐项走数进去：RG 与 MA 叠在一起，MA 就不再红（上面第二行），MA 单独打照红（第三行，三条红在 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:218` 那条「每次查至多一次折半查找」断言上）。没有一条用例直接判「在索引上走 n 项、计数记 n 项」——计数器没有自己的会红检查。只倒着走的那一句（第 3504 行）RG 没动，「从大到小逐项找」那一行照红（推的，没单独跑）。

### 第三批的改法改回去而没有一行红

上一节 T3 的 RA–RF 六条都是：第三批改了的那一处，改回一半（RA、RB、RF）或退化成每项重建（RC、RD、RE），`crates/mutations.tsv` 末尾七·6 那几行与它们点名的用例没有一行红。追加表八行、替换表三行我没有逐行复跑（变异轻型归主 agent），只复跑了 MA 一行作对照。

### checker 档判法单元键：一起编时键与二进制对不上（量的 + 推的）

链进来的 `singlefs-core` 开不开计数（量的，`cargo tree --offline … -e features -i singlefs-core` 里 `lookup-work-counters` 出现的行数，改后那棵）：

```
name=cargo_tree checker_tier_alone_lookup_work_counters=0 checker_tier_with_harness_lookup_work_counters=1 workspace_lookup_work_counters=1
```

单独编 checker 档（`-p singlefs-checker-tier`）不开；与 harness 一起编（`-p singlefs-checker-tier -p singlefs-harness`，或整个工作区）时开：`crates/singlefs-harness/Cargo.toml:18`「singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }」这一条开发依赖进了同一次构建，cargo 把特性并到唯一那一份 `singlefs-core` 上。

键怎么算（读码）：checker 档构建脚本取特性只看自己这个包——`crates/singlefs-checker-tier/build.rs:73`「let features: Vec<String> = workspace.crates[own_crate.0]」、`crates/singlefs-checker-tier/build.rs:79`「"CARGO_FEATURE_{}",」——交给 unit-graph 解特性时，非根 crate 只顺普通依赖往下传：`crates/singlefs-unit-graph/src/closure.rs:348`「let sections: &[DependencySection] = if crate_index == root_crate {」、`crates/singlefs-unit-graph/src/closure.rs:351`「&[DependencySection::Normal]」。harness 的开发依赖它看不见，所以不管怎么编，构建脚本写进二进制的都是「`singlefs-core` 不开计数」那一把键。

开与不开两把键不同（量的，unit-graph 命令行 `closure`，崩溃镜像判定那一组四个入口，`--target lib --no-default-features --cfg 'feature="default"' --cfg 'feature="verdict-store"'`，三棵树：改前、改后、改后但 checker 档的 `singlefs-core` 依赖加开计数特性以模拟一起编时真正链进来的那一份）：

```
name=unit_key group=judgement_of_the_crash_images tree=before key=3a64ff49abed2a74e898656817df1b5ab796a76df74ab4f33e5af6c51b2dd959 units=2488 lookup_work_units=0
name=unit_key group=judgement_of_the_crash_images tree=after key=128ef400926b03638da82545bee8e288744ff4e366d11473e6aff34b0d440d0f units=2500 lookup_work_units=3
name=unit_key group=judgement_of_the_crash_images tree=featured key=e7b303e7ccd1833e04763be39736105b8a3897330afa6514b1fce34e29e9f0cc units=2506 lookup_work_units=9
```

开计数时闭包里多了 `LookupSite`、`LOOKUP_WORK_ON_THIS_THREAD`、`LookupWorkOfEachSite`、`LookupWorkOnThisThread` 与两条 `use`（`diff` 两份组成清单，6 行全是 `lookup_work` 模块里只在特性下编的条目）。

所以：

- 实现员报的「带不带 harness 测试的两种编法里单元键不同」与构建脚本的做法对不上：两种编法写进 checker 档二进制的是同一把键（128ef4…）。
- 一起编（`cargo test --all` 这一类，`scripts/gate/merge-and-verify.sh:27` 写 check.sh 跑的就是它）出来的 checker 档二进制跑的是开计数的代码（真键该是 e7b303…），却顶着不开计数那一把键；它判过记进库的判定，单独编的二进制按同一把键复用，反过来也一样。这是判定库「键相同 ⇒ 闭包相同」那条承诺被破的那一侧（错误复用），不是「白白现判」。
- 后果（推的）：计数代码只往线程局部变量里加数，不改任何返回值、不改写盘次序，开不开计数判定结论应当相同，所以今天这一次复用不会把红判成绿。但这是「这一份计数碰巧行为中性」撑着，不是键撑着：以后任何只在测试特性下编、会改行为的代码走同一条路，键照样分不出。一起编的跑法会不会真把判定写进长期库，取决于那一趟设没设 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 这类变量，我没查到实例（没量）。

### 打中之后的四句（T4）

1. 分不分辨臂：不涉及臂。RG 判的是计数器自身有没有会红的检查；单元键判的是键与二进制是否一一对应。
2. 被判的系统看不看得到：构建脚本看不到——`CARGO_FEATURE_*` 只给本包的特性，依赖被别的包的开发依赖并上的特性不进环境。这一条属「判别子观测不到」：要改的是取特性的办法（例如读 cargo 的单元图或在 `singlefs-core` 里导出一个编进去的特性标记），改完是新一轮。
3. 满足的是判据字面的哪一句：正文 T4「这会不会改变 checker 档崩溃判定库的单元键、让判定白白现判或错误复用」里的「错误复用」那一分句；RG 满足「造一个改坏判据而测试不红的变异」。
4. 改法碰不碰得到：实现员的处理（把这一格报出来、不当打中）碰不到；第「改法」一节的两条各修哪一格写在那里。

## 改法（只在我的模型上量过或推过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的格上 |
|---|---|---|
| 甲：计数也数产品里剩下的按身份线性找——`TransactionOutput::unit`、`file_other_than_the_first`、留存映射条目的 `iter().find` 走过一项记一项（同 `WalkOfTheSortedIdentityIndex::next`），或者把第 9986 行改走批量读者手里的 `units_by_role` | T3 RA、RB 与「不经 lookup_work」表 | 推的：RA、RB 记数会涨；慢档两档之比过不过 200% 要看它们在读数里的份量（核源落点每趟只查 2 次，见探针），没量 |
| 乙：每处加绝对上界，不只看两档之比——核源落点每批建索引 ≤ 2、释放与快照判定每次发布建索引 ≤ 常数、每处解叶次数 ≤ 那一处 key 走到的不同叶数；并让一条用例真发 K 项满批（今天慢档每趟至多搬 1 个） | T3 RC、RD、RE；RA、RB 里的解叶那一半 | 推的：两档等价用例（批大小 2）里有 36 趟每趟搬 2 个（探针），RC 下核源落点每批建 4 次 > 2，会红；没实现、没跑 |
| 丙：释放三处各一个 `LookupSite`，断言每处各自 ≥ 换下的单元数 | T3 RF | 推的 |
| 丁：计数器自己的会红检查——在合成表上 `first_entries()` 正着走、倒着走各 n 项，断言 `index_entries_examined == n`；`get` 一次断言看过的项数 ≤ 位数且 ≥ 1 | T4 RG（与 RG+MA） | 推的 |
| 戊：键认得依赖被并上的特性——`singlefs-core` 设 `links` 并在自己的构建脚本里按 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS` 打出一行元数据，checker 档构建脚本读 `DEP_<links>_…` 把它并进特性再算键；或 `singlefs-core` 导出 `cfg!(feature = "lookup-work-counters")` 的常量，checker 档在它为真时不读不写长期判定库 | T4 单元键 | 推的；两条都没在副本上试 |

## 没打中的形状

- T1：192 段随机历史（六组比重 × 两种盘宽 × 种子 0–15 × 40 步）改前改后逐段相同；两档等价用例（24、44 批，批大小 2）镜像相同；身份类型判等与排序一致（全派生）。折半查的空表、查不到、单项靠读码。
- T2：同上；整理发批核源落点在随机历史里只走到 6/48 段（覆盖那组 4 个种子），等价用例补 68 批。造同一 key 两个角色的合法历史：没造出来（产品按角色定 key）。查不到时的 panic 与 `Err`：读码逐字相同。
- T3 线程：产品不起线程；用例并行下读数没见串数，没有专门造串数的反例。
- T3 门槛：没找到把读数推过或压下 200% 的非查法因素；基线 124、123、100。
- T4 发布构建零开销：没量（没编 `--release`、没比汇编）；读码看 `count`、`entering` 在不开特性时只剩 `let _ = …`，`InsideALookupSite` 是零大小类型、不开特性时没有 `Drop`。

## 这条腿自己的限度

- 副本上的数：全部在草稿目录里冻结副本的拷贝上量，debug 构建；不是入库装置上的数。
- 取样：差分没有缩历史（16 个种子 × 40 步是一开始就定的规模，先跑 1 个种子估的挂钟：12 段 79 秒，按 192 段 31 线程估 5 分钟，实跑 315、335 秒）；覆盖只用 4 个种子。整理发批核源落点的随机覆盖薄（128 次查），满 K 的批一次都没有。
- 崩溃状态：没有调 `enumerate_crash_states` 逐个枚举。差分比的是每一步之后的整份镜像与录制流的步数，不是录制流逐条内容；两边每一步镜像逐字节相同推不出写流逐条相同（同一镜像可以有不同的写序），崩溃状态集合是否相同是推的。
- 变异：只跑了我自己的 9 条（RA–RG、RG+MA、PR 探针）与 MA 一条对照；第三批追加表与替换表其余十行没复跑，「改回去而没有一行红」只就我造的六条说。
- 判法单元键：只算了崩溃镜像判定那一组（`judgement_of_the_crash_images`）一组，别的组没算；「一起编的跑法会不会真把判定写进长期库」没查到实例。
- 第 9986 行那一处改前改后的比较次数谁多谁少是推的。

## 这一批报出的数能不能核

- 「debug 下 `publish_lookup_by_identity` 连慢档 … 6 passed … finished in 97.94s」：副本上同一命令 `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 89.20s`（量的），对得上（秒数随负载）。
- 慢档读数：副本上 `growth_in_percent=123 index_builds_growth_in_percent=100`，合成表 `growth_in_percent=124`（量的）；实现员报告与背景材料里没有这两个数可比。
- 第二台 267 个测试二进制、2547 条测试，变异轻型 258 条抓到 257：要整轮或变异轻型才核得动，归主 agent，我没核。

## 整条复跑（rerun.sh 跑过一次）

命令（草稿目录里起，工作目录换成我的草稿目录）：

```
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-code-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo --work ~/.local/share/singlefs/work/m4-publish-lookup-code-r3-opus/rerun --model research/prompts/m4-publish-lookup-code-r3-opus-model
```

退出码 0，标准输出 38 行，原样如下：

```
name=trees after_transaction=2d69b817bed2bdfe before_transaction=0a6a104eb416d059
name=differential_compare histories=192 before_vs_after=identical endings=ending=completed:192
name=coverage_sum site=CompactionReadsTheCentralMapping histories_reaching_it=24 lookups=181 builds=180 leaf_decodes=181
name=coverage_sum site=MappingEntriesOfReleasedUnits histories_reaching_it=46 lookups=102352 builds=20066 leaf_decodes=10070
name=coverage_sum site=RelocationSources histories_reaching_it=6 lookups=128 builds=86 leaf_decodes=32
name=coverage_sum site=SnapshotJudgement histories_reaching_it=46 lookups=33906 builds=6340 leaf_decodes=245
name=images tree=before shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
name=images tree=before shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
name=mutant mutant=baseline rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=images tree=after shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
name=images tree=after shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
name=mutant mutant=RA rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RB rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RC rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RD rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RE rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RF rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RG rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
name=mutant mutant=RG+MA rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=124 index_builds_growth_in_percent=100
name=mutant mutant=MA rc=101 test result: FAILED. 3 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out failed=[compaction_batches_decode_each_routed_ma,every_lookup_by_identity_agrees_with_a_l,every_lookup_by_identity_agrees_with_a_l] growth_in_percent=124
name=mutant mutant=PR rc=0 test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out failed=[] growth_in_percent=124 growth_in_percent=123 index_builds_growth_in_percent=100
     36 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=2 relocation_lookups=3 relocation_builds=2 batches=1 versions=1
     30 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=0 relocation_lookups=0 relocation_builds=0 batches=0 versions=1
      8 probe_relocation thread=growth-pool-thread units_relocated=1 relocation_lookups=2 relocation_builds=2 batches=1 versions=1
      8 probe_relocation thread=growth-pool-thread units_relocated=0 relocation_lookups=0 relocation_builds=0 batches=0 versions=1
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=8 relocation_lookups=13 relocation_builds=10 batches=5 versions=7
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=8 relocation_lookups=12 relocation_builds=8 batches=4 versions=5
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=7 relocation_lookups=11 relocation_builds=8 batches=4 versions=5
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=2 relocation_lookups=3 relocation_builds=2 batches=1 versions=2
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=21 relocation_lookups=32 relocation_builds=22 batches=11 versions=13
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=1 relocation_lookups=2 relocation_builds=2 batches=1 versions=1
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=11 relocation_lookups=17 relocation_builds=12 batches=6 versions=7
      1 probe_relocation thread=every_lookup_by_identity_agrees_with_a_l units_relocated=0 relocation_lookups=0 relocation_builds=0 batches=0 versions=0
name=cargo_tree checker_tier_alone_lookup_work_counters=0 checker_tier_with_harness_lookup_work_counters=1 workspace_lookup_work_counters=1
name=unit_key group=judgement_of_the_crash_images tree=before key=3a64ff49abed2a74e898656817df1b5ab796a76df74ab4f33e5af6c51b2dd959 units=2488 lookup_work_units=0
name=unit_key group=judgement_of_the_crash_images tree=after key=128ef400926b03638da82545bee8e288744ff4e366d11473e6aff34b0d440d0f units=2500 lookup_work_units=3
name=unit_key group=judgement_of_the_crash_images tree=featured key=e7b303e7ccd1833e04763be39736105b8a3897330afa6514b1fce34e29e9f0cc units=2506 lookup_work_units=9
name=rerun_compare 37 of 37 lines of out/rerun-expected.txt found
```

`out/rerun-expected.txt` 的 37 行全在（末行）。这一趟的变异、差分与键与草稿里各次单独跑的逐行相同。

## 没做什么

- 没判正推腿的格，没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑门禁阶段、没跑 checker 档测试、没起 GPU；变异轻型与崩溃轻型（含第三批追加、替换那十一行的复跑）归主 agent，没起。
- 没编 `--release`，「发布构建零开销」只读了码。
- 没逐个枚举崩溃状态（见「这条腿自己的限度」）。
- 「改法」一节的五条都没实现、没在副本上跑。

## 要主 agent 跑的命令

- 判决要引 RA–RF、RG+MA 与单元键的数：把模型目录拷进草稿目录，照开头的复跑命令跑（约 45 分钟，单条 cargo，31 线程，16G 上限），看末行 `name=rerun_compare 37 of 37` 与上面那 37 行。
- 第三批追加、替换表那十一行在 RA–RF 下的红绿（变异轻型，主 agent 照 `.claude/rules/verification.md` 的写法起）：本报告没跑。
