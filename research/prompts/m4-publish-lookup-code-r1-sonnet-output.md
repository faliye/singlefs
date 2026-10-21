# m4-publish-lookup-code-r1 云端正推腿报告（Sonnet，2026-10-20）

立场：云端正推。核代码做的是不是题面说的那件事，逐格对补丁原文与冻结副本的行。被判对象是 `.claude/kb/milestone/04-fourth-txn.md`「七　GPU 垃圾回收」作业点表七·6 那一行落地的补丁。
下面引的 `crates/` 行号一律是冻结副本里的行号。冻结副本里被判的四份文件的 sha256 与 `research/prompts/m4-publish-lookup-code-r1-snapshot/sha256sums.txt` 里登记的逐行相同（见 0.1）；我写报告途中主工作区的 `transaction.rs` 被别的会话改过（见 0.1），所以 `cite-check.py` 要用 `--root` 指到冻结副本跑。
只读了背景材料、补丁原文、冻结副本与 `crates/` 源码；没读禁读清单里的任何文件。

## 0 摘要、复跑命令与文件

一句话结论（逐格依据在各节，推翻条件在各节末）：

- T1 成立：五处的索引与原来的 `iter().find` 对重复身份都取表里最先那一项；建索引之后没有一处改表再查（借用检查守住，没有内部可变类型）。
- T2 成立：叶遍历与逐 key 查走同一个路由函数、同一张 `units` 首项；三种叶情形与 `MappingLookup` 每一种结局产出相同；`CentralMappingEntry` 的集合与次序相同。
- T3 成立：三处查不到时的 panic 文案逐字相同、条件相同；resolve 里的索引每次调用现建，settle 每轮调一次 resolve，所以每轮现建、不跨轮复用。
- T4 部分成立：等价测试的参照边是改前的线性查法（不是两边都走新索引）；七行「按身份查」变异我在自己的副本里逐条套上，七条都红；两行「增长」变异：「按身份查的索引退回逐项找」红（609%），「整理读版本退回逐 key 查映射」不红（359%、357%）；而且把补丁的调用点整个还原成改前的写法，慢档用例也不红（379%、377%），400% 门槛在登记的 debug 命令下分不出改前与改后。
- T5 成立：查法之外没有别的行为改动；改前改后同一串操作写出的两块盘镜像 sha256 在两档池上逐字节相同。

我自己量的东西（全是 `singlefs-harness` 档的测试二进制，在草稿目录的副本里编，经 `scripts/hosts/capped.sh 31` 与 `scripts/hosts/run-with-memory-cap.sh` 跑；没跑 checker 档、门禁、GPU）：

- 复跑入口：`bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-publish-lookup-code-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> --model research/prompts/m4-publish-lookup-code-r1-sonnet-model`；
  默认只跑确定性的两步（改后与改前的镜像摘要，约 2 分钟）；增长数带负载噪声，要跑用环境变量点名，例如 `RERUN_STEPS="growth-new growth-old growth-m9 growth-m8"`（每步 100 到 220 秒）。
- 变异逐条跑：`DRAFT=<草稿目录> REPO_ROOT=<仓根> bash research/prompts/m4-publish-lookup-code-r1-sonnet-model/tools/run_mutant.sh <变异名片段> <测试名过滤> <日志前缀>`（草稿目录的布局在脚本文件头）。
- `rerun.sh` 的自证跑过（5 格全过）；`digests-new` 与 `digests-old` 两步在新草稿目录里各完整跑过一次，输出在 `out/rerun-digests-new.out`；增长四步用的是同一份函数，我是用等价的直接命令跑的（日志在 `out/run-growth-*.log`），没有经 `rerun.sh` 整条重跑。
- 复核不了的：合入后验证在第二台的 254 个二进制 2489 过（主 agent 观测）。我只读了它的日志做了计数：`grep -E '^test result: ok'` 254 行、累加 passed=2489、ignored=78、`^test result: FAILED` 0 行；没有在第二台重跑。

模型目录 `research/prompts/m4-publish-lookup-code-r1-sonnet-model/` 每个文件的 sha256：

```
c384e054f07e5541cfe88b3c05d6bdd3e9a3bd76014e1b87e165e01c8c8f418b  diff/callsites.diff
e303e45c491edb96e6a4ce346ec7a7b5d2281cfd8140e588157c80e6637d18a2  diff/mounted_session.diff
2f7aa34d0b9c93e67dfc426ee402b18c82ee2d18ff40d17966f1ea72551cbf6e  diff/transaction.diff
f3d6b24a6c05ffa7d6add59680c27b5c70179fc480202c1547f943fd2fff59f8  out/mut-m1.run.log
7a8217de6b8f87982cee08eb3dbbd0648f12415ee1d5c6c65b269e2383b84460  out/mut-m2.run.log
d11550fe21484c9fce81ba49df966f44110f0a7280e8ff0da1e73a2511852ac2  out/mut-m3.run.log
a4d48943cc467b8fe17ecf30af894d8fbe8410e0c0c8fc2a9f5e2e4c03abeb91  out/mut-m4.run.log
bea21c175b592fd8df85cc562e2816b425f9d8f0ced0c5d6f3491da86c04c72f  out/mut-m5.run.log
f10009ec47f87650c3e912b092e1e22eb31f9929d85407e64aac56434b807d93  out/mut-m6.run.log
512658bc0acd113673d229cd9f01eb7d7ba6f52a934a465a37d646fc2451431c  out/mut-m7.run.log
490cbe542346677e909f4977b53163a759ddd51f16a1b325127576d0505b351a  out/rerun-digests-new.out
750044e32df97f3dc8e2f356f71e6c300aeedeca7d80be0907e031b51faf0011  out/run-16.log
db20e001c4926a87c927d6b99680f8b17e1c99ce4d6ec3193da15ea4924e0e37  out/run-48.log
2b1fe795db57aee36a6d80237a87f0bdeb1d09a5b371a6c27e8b78e462c50194  out/run-growth-base2.log
847922844a6d6ef444fceb10022019f862d9d05abc14880700f6a012c2bc9a6e  out/run-growth-base.log
8e5ef32c39b7ef7e46784273091abfb53197e528e82afceb6316c504767eab82  out/run-growth-m8.log
f5b3732b19d61d201337eba3af8b538060326009d1ffa68ebc58a2cc8f75561f  out/run-growth-m9b.log
b2e86beb162221a396893315b2a73e5ccfbc27e7afdf217b61002b0a20cbafb5  out/run-growth-m9.log
7968a99b61c7ae01ac87f057cf5f4b7b08e3c89b860cfd6ce4ac52fb046beb2e  out/run-growth-old2.log
1fc1c74b52d1d89b1778c0480fac066e939d81391fb4351474966d49b00d9c85  out/run-growth-old.log
0ebdee743a6b41a2e24aa0b568073c42f77240e4f05e59f5b79610ce9c1629eb  out/run-old-digests.log
c573e5628175cfe7bc7a5b5b727ea6d557e9d71fc46e81257b189f67551066fe  out/run-small.log
e7a303d43da8aee85878d3a20b5ae2df4fbcfaedda8b415e073aeb91f22d00eb  tools/apply_mutation.py
63fd7877c1dcf39821563d99f04ca65cc91d3aae805120db22d9d6a41244fb2e  tools/rerun.sh
817ad1304f458ac6efdca25f025d8f2ddab4e1914c4a3846e16cde4d5be330d9  tools/run_mutant.sh
```

说明：`diff/transaction.diff` 与 `diff/mounted_session.diff` 是改前的 git blob（`ad9371e`、`75fd509`，`git cat-file -p` 取出）到冻结副本的 `diff -u`；`diff/callsites.diff` 是前者里只留调用点那 13 块（去掉三块新接口定义），反向打在冻结副本上就得到「新接口留着、调用点退回改前」的树，下面叫「改前发布路径树」。
`out/*.log` 是我跑的原样输出（路径里的家目录换成了 `~`）。

### 0.1 冻结副本与主工作区、补丁与冻结副本

命令与原样输出（在冻结副本根目录跑）：

```
$ sha256sum crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/mounted_session.rs crates/singlefs-harness/tests/publish_lookup_by_identity.rs crates/mutations.tsv
4f27be528003e1bf9abb36b7bf416efd2b0af780495d0a2f02edf0b62bab25dd  crates/singlefs-core/src/transaction.rs
f027e88a3299dd9968dcc3e14cecad03fb4be8110be17f69b1b69349af10bbbd  crates/singlefs-core/src/mounted_session.rs
948e1f96597838b8692dba7e6b97fa9e82cd7a48bebf556ff441b6fe6550bb0d  crates/singlefs-harness/tests/publish_lookup_by_identity.rs
99dc56e6387ada5b7c96ba36f0cd5d0f734eb0fc7e2b528f719e1241c5b0c63a  crates/mutations.tsv
```

这四行与登记的 `research/prompts/m4-publish-lookup-code-r1-snapshot/sha256sums.txt` 里同路径的四行逐字相同（`grep` 取出对过）。我开工时主工作区的这四个文件与它们也相同；写报告途中（本报告完成前）别的会话改了主工作区的 `crates/singlefs-core/src/transaction.rs`，现在的 sha256 是 `178ec464ce49038359de47c9a4e564339baad5c8a713a0dd8dc072f8a9b077ac`，行号已经和冻结副本对不上。这一改不在被判补丁里，我没有看它的内容。
补丁是否就是冻结副本相对改前的全部改动：把改前 blob 与冻结副本做 `diff -u`，取所有 `+`/`-` 行，与补丁原文 `crates.patch` 部分的 `+`/`-` 行逐行比：

```
$ wc -l patch.pm ours.pm        # 两份都是去掉 +++/--- 头之后的 +/- 行
  282 patch.pm
  282 ours.pm
$ diff patch.pm ours.pm && echo IDENTICAL_PLUS_MINUS_LINES
IDENTICAL_PLUS_MINUS_LINES
```

所以下面对「改前」的任何说法，都是 `diff/transaction.diff`、`diff/mounted_session.diff` 里 `-` 行的内容；产品代码的改动不超出这两份文件。

## T1 索引与线性查找是不是同一项

**结论：成立。** 五处每一处，同一身份在表里出现几次，原来的 `iter().find` 与新的 `FirstEntryOfEachIdentity` 取的都是表里次序最先的那一项。

### T1.1 原查法与新查法各取哪一项

通用部分（五处共用，一次说清）：

- 建索引：`crates/singlefs-core/src/transaction.rs:3202`：「for entry in table {」顺着表的次序走；`crates/singlefs-core/src/transaction.rs:3204`：「.entry(identity_of(entry))」后面接 `crates/singlefs-core/src/transaction.rs:3205`：「.or_insert(entry);」，见过的身份不覆盖，所以映射里每个身份留下的是表里最先见到的那一项。
- 查：`crates/singlefs-core/src/transaction.rs:3215`：「self.first_entry_of_each_identity.get(identity).copied()」。
- 原来：`iter().find` 返回按表次序第一个让谓词成立的元素。键相等的口径：索引按 `Ord`、原来按 `==`；两边口径一致才算同一项。身份类型 `TransactionUnit` 与 `InodeNumber` 都是派生的比较：`crates/singlefs-core/src/transaction.rs:2398`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」、`crates/singlefs-core/src/address.rs:44`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]」；它里面嵌的位置类型也是派生，字段全是 `u8`、`u32`、`u64` 与 `DeviceIdentity(pub u32)`（`crates/singlefs-core/src/allocation_record_tree.rs:44`：「pub struct AllocationRecordTreeNodePosition {」是其中字段最多的一个，三个字段 `level: u8`、`device: DeviceIdentity`、`index_in_device: u64`），并且核心层没有手写的 `PartialEq` / `Ord` 实现：

```
$ grep -rnE 'impl[^{;]*\b(PartialEq|Ord|PartialOrd|Eq)\b[^{;]* for ' crates/singlefs-core/src/*.rs crates/singlefs-format/src/*.rs | wc -l
0
```

  `DeviceIdentity` 的定义在 `crates/singlefs-core/src/address.rs:14`：「pub struct DeviceIdentity(pub u32);」，派生行在它上面一行。上面那条命令同时查了 `singlefs-format`，也没有手写实现。

五处逐处（「表」是被查的那张）：

| 处 | 表 | 原查法（改前的 `-` 行） | 新查法（冻结副本） | 同一身份重复时 |
|---|---|---|---|---|
| 照抄单元 `carried_unit` 及其七个调用点 | 上一版 `units` | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:244`：「-    previous.unit(identity).clone()」，其下 `TransactionOutput::unit` 是 `crates/singlefs-core/src/transaction.rs:3716`：「.find(|unit| unit.identity == identity)」 | `crates/singlefs-core/src/transaction.rs:11256`：「units_of_the_previous_version.unit(identity).clone()」，索引由 `crates/singlefs-core/src/transaction.rs:3723`：「FirstEntryOfEachIdentity::of_the_table(&self.units, |unit| unit.identity)」建 | 都取 `units` 里最先那一项 |
| 第一个文件的数据单元 `carried_file_version_units` | 上一版 `units` | 改前 `carried.unit(...)`（同上 `TransactionOutput::unit`） | `crates/singlefs-core/src/transaction.rs:12437`：「let units_of_the_carried_version = carried.units_by_role();」 | 同上 |
| resolve 里照抄角色的映射 key `carried` | 上一版 `mapped_units` | `crates/singlefs-core/src/transaction.rs:7005`：「.find(|(mapped, _)| *mapped == identity)」（`mapping_key_carried_from` 原样留着，改前的 `-` 行是 `research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:227`：「-                previous.expect("照抄的角色只出现在接着上一版的发布里"),」那一处调用） | `crates/singlefs-core/src/transaction.rs:7889`：「previous.map(TransactionOutput::mapped_units_by_role);」与 `crates/singlefs-core/src/transaction.rs:7894`：「.mapping_key_of(identity)」 | 都取 `mapped_units` 里最先那一项的 key |
| 照抄的第一个文件之外的文件 | 上一版 `files_other_than_the_first` | `crates/singlefs-core/src/transaction.rs:3632`：「.find(|file| file.inode == inode)」（`file_other_than_the_first` 原样留着；改前的调用在 `diff/transaction.diff` 里是 `-                    previous` 起的四行） | `crates/singlefs-core/src/transaction.rs:12750`：「previous.map(TransactionOutput::files_other_than_the_first_by_inode);」与 `crates/singlefs-core/src/transaction.rs:12769`：「.and_then(|files| files.first_entry_of(&file_plan.inode))」 | 都取最先那个文件 |
| 整理读版本里的叶单元 | 这一版 `units` | `crates/singlefs-core/src/transaction.rs:4248`：「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」（逐 key 查，原样留着） | `crates/singlefs-core/src/transaction.rs:4335`：「CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes)」，索引由 `crates/singlefs-core/src/transaction.rs:4314`：「units_by_role: version.units_by_role(),」建 | 都取最先那一项 |

重复身份会不会真出现（推的，读代码，不影响上面的结论）：`units` 末尾的实例表各片，重写时每片一个角色（页号不同）、照抄时是上一版 `units` 里的各片原样（`crates/singlefs-core/src/transaction.rs:13812`：「(InstanceTablePlan::Carry(_), Some(previous_version)) => {」与 `crates/singlefs-core/src/transaction.rs:13817`：「.filter(|unit| unit.identity.is_a_page_of_the_instance_table())」），从盘上重建的版本只放第 0 片（`crates/singlefs-core/src/recovery.rs:3960`：「TransactionUnit::InstanceTable,」，没有第 1 片起的页）。代码里没有「角色互不相同」的断言，所以不能说重复不可能；重复时两边取的是同一项，这一条不靠不重复。
推翻 T1.1 的现象：一张表里同一身份出现两次，而索引与 `iter().find` 取到不同的元素（比地址）；或者某个身份类型的 `==` 与 `Ord` 判相等的集合不同。

### T1.2 建了索引之后那张表还会不会被改

**结论：不会；没有一处在建了索引之后改表再查。**

- 索引借着表：`crates/singlefs-core/src/transaction.rs:3193`：「first_entry_of_each_identity: BTreeMap<Identity, &'table Entry>,」存的是对表项的共享引用；`crates/singlefs-core/src/transaction.rs:3199`：「pub fn of_the_table(table: &'table [Entry], identity_of: impl Fn(&Entry) -> Identity) -> Self {」收的是共享借用的切片。身份键在建的时候取一份拷贝（`identity_of(entry)` 返回值本身）。索引活着时对同一张表的 `&mut` 过不了借用检查，所以「建了索引再就地改表」写不出来；这是编译期的守卫，不靠约定。
- 表项里没有绕开借用检查的内部可变：

```
$ grep -nE 'RefCell|Cell<|Mutex|RwLock|Arc<|Rc<|unsafe' crates/singlefs-core/src/transaction.rs
（零行输出）
```

  身份字段都是普通字段：`crates/singlefs-core/src/transaction.rs:3062`：「pub struct PublishedUnit {」里 `identity: TransactionUnit`，`crates/singlefs-core/src/transaction.rs:3248`：「pub struct FileOfAnInodeOtherThanTheFirst {」里 `inode: InodeNumber`，`mapped_units` 的元素是 `(TransactionUnit, Vec<u8>)`。这条 grep 只查了 `transaction.rs`，其他文件里的嵌套类型我没逐个查内部可变；但被当键的字段是上面三个普通 `Copy` 字段，不嵌套别的类型。
- 公开字段 `units` 别处确有就地改，但都在索引建之前：
  - 核心层只有一处：`crates/singlefs-core/src/transaction.rs:3320`：「pub fn take_the_mapping_positions_over_the_location_hints<PositionOfKey>(」里，`crates/singlefs-core/src/transaction.rs:3410`：「.iter_mut()」那一段把 `unit.slot` 换成映射里的新落点（只改 `slot`，不改 `identity`、不改次序、不增删）。它的调用点是 `publish_admitted` 里对上一版的克隆：`crates/singlefs-core/src/transaction.rs:12599`：「let mut relocated_version = previous_version.clone();」与 `crates/singlefs-core/src/transaction.rs:12600`：「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」。时序：克隆与就地改（12599 起）→ 把 `previous` 重绑成这份克隆 `crates/singlefs-core/src/transaction.rs:12614`：「let previous: Option<&TransactionOutput> =」→ 文件索引 12750 → 单元索引 13630。三处索引都建在就地改之后，克隆在它之后不再被改。改前同一份重绑后的 `previous` 也是 `carried_unit` 等的来源（`diff/transaction.diff` 里 `-` 行的 `previous.expect(...)`），所以「照抄的单元带着搬迁后的落点」改前改后一样。
  - 测试里有几处对克隆的 `units` 做 `iter_mut` / `retain`（例如 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:623` 附近的 `rebuilt.units`），都发生在把这份版本交给产品函数之前，产品函数的索引在函数入口才建。
  - 检查器档的探针 `crates/singlefs-checker-tier/src/bin/e180_admission_reading_versus_writable_bytes.rs:708`：「for (index, unit) in output.units.iter_mut().enumerate() {」把数据单元的 `bytes` 取走再还回去，是在活会话的两次操作之间，那一刻没有索引活着。
- 每处索引的生命周期都不长于它借的那一版：`resolve` 的索引 `crates/singlefs-core/src/transaction.rs:7889`：「previous.map(TransactionOutput::mapped_units_by_role);」随函数结束；`CentralMappingOfAVersionReadLeafByLeaf` 带着 `&'version TransactionOutput`（`crates/singlefs-core/src/transaction.rs:4255`：「pub struct CentralMappingOfAVersionReadLeafByLeaf<'version> {」），调用方 `crates/singlefs-core/src/mounted_session.rs:2786`：「let mut central_mapping = CentralMappingOfAVersionReadLeafByLeaf::of_the_version(version);」在 `version_as_the_compaction_reads_it` 内部建、内部用完。

推翻 T1.2 的现象：在 `FirstEntryOfEachIdentity` 或 `CentralMappingOfAVersionReadLeafByLeaf` 存活期间，有代码拿到同一张表的 `&mut`（编不过），或表项里有内部可变的身份字段（grep 到 `Cell` 之类并被当键）。

## T2 整理读版本一遍走叶与逐 key 查是否逐项相同

**结论：成立。** 叶遍历没有自己找叶：每把 key 仍由同一个路由函数送到同一片叶，只是每片叶的解析结果按角色存了一份；三种叶情形与 `MappingLookup` 的每一种结局，两边产出相同。

### T2.1 一把 key 落在哪片叶（路由）

- 逐 key 查（原来，现在仍是公开函数）：`crates/singlefs-core/src/transaction.rs:4240`：「let shape = &version.central_mapping_tree.shape;」、`crates/singlefs-core/src/transaction.rs:4241`：「let Some(leaf) = leaf_position_routed_to(」、`crates/singlefs-core/src/transaction.rs:4243`：「&CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),」。
- 叶遍历：`crates/singlefs-core/src/transaction.rs:4313`：「shape: &version.central_mapping_tree.shape,」（同一个 `shape`）、`crates/singlefs-core/src/transaction.rs:4324`：「let Some(leaf) = leaf_position_routed_to(」、`crates/singlefs-core/src/transaction.rs:4326`：「&CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),」。两边调同一个函数 `crates/singlefs-core/src/code_two_tree.rs:1156`：「pub fn leaf_position_routed_to(」，参数相同，函数只读 `shape` 与 `key`（它沿分隔 key 下降：`crates/singlefs-core/src/code_two_tree.rs:1168`：「.rposition(|child| child.separator_key <= *key)」），所以同一把 key 在两种走法下必然得到同一个叶位置；路由不出叶（`None`）时两边都返回 `NodeMalformedOrNoEntryWithThisKey`（`crates/singlefs-core/src/transaction.rs:4245` 与 `crates/singlefs-core/src/transaction.rs:4328` 同一句 `return MappingLookup::NodeMalformedOrNoEntryWithThisKey;`）。
- 「key 不在它路由到的那片叶里」：两边都只看路由到的那一片，那片里没有就是 `NodeMalformedOrNoEntryWithThisKey`（逐 key 查：`crates/singlefs-core/src/transaction.rs:4717`：「None => MappingLookup::NodeMalformedOrNoEntryWithThisKey,」；叶遍历：`crates/singlefs-core/src/transaction.rs:4299`：「locations_of_each_key.get(key).map_or(」）。别的叶里有同一把 key 与它无关，两边都不碰。
- 「几片叶都有同一把 key」：同上，只有被路由到的那一片算数；叶遍历不会因为缓存里别的叶有这把 key 而取到它，因为缓存按叶的角色存（`crates/singlefs-core/src/transaction.rs:4333`：「.entry(leaf_role)」），而叶的角色由叶位置推出：`crates/singlefs-core/src/transaction.rs:4330`：「let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, self.shape);」，根是 `MappingTree`（`crates/singlefs-core/src/transaction.rs:2513`：「(MultiLevelCodeTwoTree::CentralMapping, true) => TransactionUnit::MappingTree,」）、非根是 `MappingTreeNodeBelowTheRoot(position)`（`crates/singlefs-core/src/transaction.rs:2515`：「TransactionUnit::MappingTreeNodeBelowTheRoot(position)」），同一份 `shape` 里不同位置对应不同角色、同一位置对应同一角色，缓存键与叶一一对应。

### T2.2 叶字节取哪一份

逐 key 查：`crates/singlefs-core/src/transaction.rs:4248`：「mapping_locations_for_key(&version.unit(leaf_role).bytes, key)」，`unit` 是 `units` 上的 `iter().find`（`crates/singlefs-core/src/transaction.rs:3716`：「.find(|unit| unit.identity == identity)」）。叶遍历：`crates/singlefs-core/src/transaction.rs:4335`：「CentralMappingLeafRead::of_the_leaf_bytes(&units_by_role.unit(leaf_role).bytes)」，经索引取最先那一项（T1.1 表里「整理读版本里的叶单元」那一行）。同一片叶的字节在叶遍历对象存活期间不变（T1.2）。

### T2.3 叶解一次：三种叶情形与 `MappingLookup` 每一种结局

逐 key 查的叶内部是 `mapping_locations_for_key`（`crates/singlefs-core/src/transaction.rs:4700` 起），叶遍历的是 `CentralMappingLeafRead::of_the_leaf_bytes` 与 `locations_of_key`（`crates/singlefs-core/src/transaction.rs:4273` 起）。逐结局对：

| 叶的样子 | 逐 key 查 | 叶遍历 | 同不同 |
|---|---|---|---|
| 叶解不开 | `crates/singlefs-core/src/transaction.rs:4701`：「let Ok(node) = parse_index_node(mapping_node_bytes) else {」，其后返回 `NodeMalformedOrNoEntryWithThisKey` | `crates/singlefs-core/src/transaction.rs:4274`：「let Ok(node) = parse_index_node(mapping_node_bytes) else {」→ `crates/singlefs-core/src/transaction.rs:4275`：「return Self::NodeMalformed;」，查时 `crates/singlefs-core/src/transaction.rs:4292`：「Self::NodeMalformed => MappingLookup::NodeMalformedOrNoEntryWithThisKey,」 | 同：走到这片叶的每把 key 都是 `NodeMalformedOrNoEntryWithThisKey`。解析函数 `crates/singlefs-core/src/unit.rs:403`：「pub fn parse_index_node(bytes: &[u8]) -> Result<IndexNodeHeader, UnitError> {」只吃字节，与 key 无关 |
| 有一条条目窄于字段表 | `crates/singlefs-core/src/transaction.rs:4706`：「let Some((candidate, locations)) = parse_mapping_entry(entry) else {」，按条目次序走，遇到第一条切不动的就 `crates/singlefs-core/src/transaction.rs:4707`：「return MappingLookup::EntryNarrowerThanItsFieldTable {」，字节数 `crates/singlefs-core/src/transaction.rs:4708`：「entry_bytes: entry.len(),」；遇到切不动的条目就直接返回、不看已经暂存的 `found`（暂存的被丢掉），所以不论那把 key 的条目排在它前面还是后面都一样 | `crates/singlefs-core/src/transaction.rs:4280`：「let Some((key, locations)) = parse_mapping_entry(entry) else {」，同样按次序走、遇到第一条切不动的返回 `crates/singlefs-core/src/transaction.rs:4281`：「return Self::EntryNarrowerThanItsFieldTable {」带 `crates/singlefs-core/src/transaction.rs:4282`：「entry_bytes: entry.len(),」，整片叶记成这一种，查时每把 key 都回它 | 同。「切得动」只由条目自己的长度决定（`crates/singlefs-core/src/records.rs:850`：「if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {」），与 key 无关；两边找的都是条目次序里第一条切不动的，字节数相同；那把 key 的条目排在它前面还是后面，两边都是这种结局 |
| 同一把 key 有几条 | `crates/singlefs-core/src/transaction.rs:4711`：「if candidate == key {」后 `crates/singlefs-core/src/transaction.rs:4712`：「found = Some(locations);」，后面的覆盖前面的 → 取最后一条 | `crates/singlefs-core/src/transaction.rs:4285`：「locations_of_each_key.insert(key, locations);」，`BTreeMap::insert` 覆盖同键旧值 → 取最后一条 | 同。键相等的口径：映射条目的 key 恒是条目前 27 字节（`crates/singlefs-core/src/records.rs:853`：「let key = bytes[..usize::try_from(MAPPING_KEY_BYTES).expect("27")].to_vec();」）；逐 key 查按字节切片 `==`，叶遍历的 `BTreeMap<Vec<u8>, _>::get(&[u8])` 按 `[u8]` 的全序，两者判相等的集合相同（字节全等） |
| 叶里没有这把 key | `crates/singlefs-core/src/transaction.rs:4717`：「None => MappingLookup::NodeMalformedOrNoEntryWithThisKey,」（在 4716 `Some(locations) => MappingLookup::Found(locations),` 的下一行） | `crates/singlefs-core/src/transaction.rs:4299`：「locations_of_each_key.get(key).map_or(」的缺省臂 `NodeMalformedOrNoEntryWithThisKey`，命中臂 `crates/singlefs-core/src/transaction.rs:4301`：「|locations| MappingLookup::Found(*locations),」 | 同；`[LocationEntry; 2]` 是 `Copy`，`*locations` 与逐 key 查拿到的是同一对 |

四种结局里 `Found`、`NodeMalformedOrNoEntryWithThisKey`、`EntryNarrowerThanItsFieldTable { entry_bytes }` 三个成员全部覆盖；`MappingLookup` 只有这三个成员（`crates/singlefs-core/src/transaction.rs:4688` 的定义）。

### T2.4 `CentralMappingEntry` 的集合与次序

`version_as_the_compaction_reads_it` 对 `version.mapped_units` 按表的次序逐个查（`crates/singlefs-core/src/mounted_session.rs:2788`：「.mapped_units」），`Found` 产出一条 `CentralMappingEntry`，其余两种结局产出 `None`（`crates/singlefs-core/src/mounted_session.rs:2791`：「MappingLookup::Found(locations) => Some(CentralMappingEntry {」；改前同一段在 `research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/mounted_session.diff:37`：「-            |(_, key)| match central_mapping_locations_in_the_version(version, key) {」起，三个臂的写法逐行相同，只有被调的函数不同）。逐 key 的答案相同（T2.1 到 T2.3），迭代次序没变，所以收集出的 `Vec` 的元素与次序相同；后面接留存的映射条目那一段 `crates/singlefs-core/src/mounted_session.rs:2799`：「mapping_entries.extend(version.mapping_entries_kept_for_the_snapshots.iter().map(」不在改动里（补丁的 `-` 行里没有它）。

### T2.5 panic 面

叶的角色不在 `units` 里：逐 key 查在第一把路由到它的 key 上 panic（`TransactionOutput::unit`，文案 `crates/singlefs-core/src/transaction.rs:3717`：「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」）；叶遍历在第一把路由到它的 key 上才解（`crates/singlefs-core/src/transaction.rs:4334`：「.or_insert_with(|| {」，闭包里调 `FirstEntryOfEachIdentity::unit`，文案 `crates/singlefs-core/src/transaction.rs:3227`：「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」同一个常量）。同一把 key 先 panic，文案相同；panic 发生在 `collect` 里，没有半成品被用。

### T2 小结与推翻条件

- 我跑的支撑（自己的副本，未变异的补丁）：解不开 / 窄 / 同键两条三种叶，和一版多片叶的池，批量读与逐 key 查逐项相同，用例都过：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-small.log:4`：「test leaf_by_leaf_mapping_read_agrees_with_the_per_key_lookup_on_a_malformed_leaf_a_narrow_leaf_and_repeated_keys ... ok」、`research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-48.log:8`：「test every_lookup_by_identity_agrees_with_a_linear_find_on_every_version_of_forty_eight_files_of_eight_data_units ... ok」。把批量读改成「同键取最先」「几片叶共用一份解出的条目」两条变异分别让这两个用例红（T4）。这是读代码得出结论之后的印证，不是结论的来源。
- 推翻 T2 的现象：造一版中央映射树，让某把 key 在叶遍历与 `central_mapping_locations_in_the_version` 下得到不同的 `MappingLookup`（或 panic 与否不同）；或 `leaf_position_routed_to` 对同一个 `(shape, key)` 两次调用结果不同（它只读参数，读代码看不出会不同）。

## T3 照抄路径：`carried_unit`、`mapping_key_carried_from`、`file_other_than_the_first`

**结论：成立。** 查不到时三处仍然 panic，文案逐字相同、条件相同；映射 key 的索引在每次 `resolve` 里现建，`settle_the_allocation_record_tree` 每轮调一次 `resolve`，所以每轮现建、不跨轮复用。

### T3.1 查不到时的行为

| 处 | 条件 | 改前 | 现在 | 文案同不同 |
|---|---|---|---|---|
| 照抄单元 `carried_unit` | 上一版 `units` 里没有这个角色 | `TransactionOutput::unit`：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:84`：「-            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")」 | `crates/singlefs-core/src/transaction.rs:3227`：「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」，常量 `crates/singlefs-core/src/transaction.rs:3184`：「"八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";」；`TransactionOutput::unit` 自己也改用同一个常量（`crates/singlefs-core/src/transaction.rs:3717`） | 逐字同 |
| 照抄角色的映射 key | 上一版 `mapped_units` 里没有这个角色 | `mapping_key_carried_from`（原样留着）：`crates/singlefs-core/src/transaction.rs:7007`：「.expect("照抄进来的角色在上一版里进映射、有一把 key")」 | `crates/singlefs-core/src/transaction.rs:3240`：「.expect("照抄进来的角色在上一版里进映射、有一把 key")」 | 逐字同 |
| 照抄的文件 | `previous` 是 `None`，或上一版没有这个文件 | 改前这一句没动（`diff/transaction.diff` 里它只作上下文行出现：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:291`：「                         .expect("照抄的文件在上一版里")」） | `crates/singlefs-core/src/transaction.rs:12770`：「.expect("照抄的文件在上一版里")」，前面接 `crates/singlefs-core/src/transaction.rs:12769`：「.and_then(|files| files.first_entry_of(&file_plan.inode))」 | 逐字同 |

「上一版整个不在」（`previous` 是 `None`）那一类 panic，改前每处调用点各自带一句 `previous.expect("…")`，改后改成 `units_of_the_previous_version.as_ref().expect("…")`，七句文案逐字没动。改前的 `-` 行与改后的 `+` 行对照：

- 映射 key 的 `carried`：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:227`：「-                previous.expect("照抄的角色只出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:7893`：「.expect("照抄的角色只出现在接着上一版的发布里")」。
- 第一个文件之外的文件的单元：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:319`：「-                    previous.expect("照抄的文件只出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:13668`：「.expect("照抄的文件只出现在接着上一版的发布里"),」。
- 没重写的叶容器：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:330`：「-                    previous.expect("没重写的叶容器只会出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:13686`：「.expect("没重写的叶容器只会出现在接着上一版的发布里"),」。
- 没重写的 inode 根：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:341`：「-            previous.expect("没重写 inode 根的发布恒接在上一版之后"),」→ `crates/singlefs-core/src/transaction.rs:13702`：「.expect("没重写 inode 根的发布恒接在上一版之后"),」。
- 照抄的分配记录树节点：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:352`：「-                    previous.expect("照抄的分配记录树节点只出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:13719`：「.expect("照抄的分配记录树节点只出现在接着上一版的发布里"),」。
- 照抄的删除树节点：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:363`：「-                previous.expect("照抄的删除树节点只出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:13733`：「.expect("照抄的删除树节点只出现在接着上一版的发布里"),」。
- 照抄的容器树节点：`research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff:374`：「-                    previous.expect("照抄的容器树节点只出现在接着上一版的发布里"),」→ `crates/singlefs-core/src/transaction.rs:13748`：「.expect("照抄的容器树节点只出现在接着上一版的发布里"),」。

条件也相同：`Option` 是 `previous.map(...)` 得来的（`crates/singlefs-core/src/transaction.rs:13630`：「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」），`previous` 为 `None` 当且仅当索引的 `Option` 为 `None`；建索引本身不会 panic（`of_the_table` 只遍历与插入）。
新多出来的一句是第一个文件的数据单元那一臂：改前 `Some(carried) if …` 直接用 `carried`，现在 `Some(_) if …` 改用 `previous` 的索引（`crates/singlefs-core/src/transaction.rs:13640`：「.expect("照抄第一个文件的发布接在上一版之后：carried_file 就是上一版")」）。走不到这句：`carried_file` 恒等于 `previous` 且为 `Some` 时才进这一臂（`crates/singlefs-core/src/transaction.rs:12650`：「None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),」，`previous` 是 12614 重绑之后的那个，建索引用的也是它，T1.2）。

### T3.2 `settle_the_allocation_record_tree` 里索引是每轮现建还是跨轮复用

- 现在 `resolve` 里不再调 `mapping_key_carried_from`：`crates/singlefs-core/src/transaction.rs:7889`：「previous.map(TransactionOutput::mapped_units_by_role);」在 `resolve` 函数体里建一次（`resolve` 的 `previous` 是共享借用：`crates/singlefs-core/src/transaction.rs:7576`：「previous: Option<&TransactionOutput>,」），`carried` 闭包借着它（`crates/singlefs-core/src/transaction.rs:7890`：「let carried = |identity: TransactionUnit| {」）。
- 固定点循环每一轮调一次 `resolve`：循环 `crates/singlefs-core/src/transaction.rs:11057`：「for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {」，调用 `crates/singlefs-core/src/transaction.rs:11067`：「let resolved = plan.resolve(」。所以索引每轮现建，随该轮的 `resolve` 返回而释放，**没有跨轮复用**；跨轮携带的只是 `rewritten`、`nodes_after`（`crates/singlefs-core/src/transaction.rs:11056`：「跨轮携带的是重写集（只增不减）与这一版的节点（换成上一轮走出来的那一批）。」），它们不是上一版的表。`previous` 在循环里从头到尾是同一个共享借用，中间一轮改不了它。
- 代价（读代码，没量）：产品路径上固定点三四轮就定（`crates/singlefs-core/src/transaction.rs:11142`：「/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。产品路径上三四轮就定（第一轮重写集是空的、」），索引每轮重建，一次发布建三四次、每次 O(上一版进映射单元数 × log)。不影响行为。
- 没改到、仍是线性的相邻一处：`mapping_key_carried_from` 还剩一个调用点 `crates/singlefs-core/src/transaction.rs:5652`：「let mapping_key = mapping_key_carried_from(previous_version, *identity);」，在 `deadlist_changes_of_this_publish` 里对每个被换下的角色各查一次（该函数由 `resolve` 调：`crates/singlefs-core/src/transaction.rs:7663`），其文档说（`crates/singlefs-core/src/transaction.rs:6996`：「（快照判定那一圈只问这次换下的几个）」）。整理的一批换下的是 K 个角色，这句成立；一次覆盖写整份换下一个大文件时换下的角色数约等于文件的单元数（推的，没量过），那一圈仍是「角色数 × 进映射单元数」。它不在补丁点名的五处，也不是补丁声称改的，只记在这里。

### T3 推翻条件

某个查不到的输入上，改前 panic 而现在返回、或两边 panic 的文案不同（构造：上一版 `mapped_units` 缺一个照抄的角色）；或 `settle` 的某一轮里 `previous` 指向的表在两轮之间被改（借用检查下写不出来）。

## T4 测试与变异判得出它要判的

**结论：分三层。** 等价测试的参照边是改前的线性查法，成立（只覆盖查法的 API 层，调用点的集成等价靠镜像摘要）；七行「按身份查」变异我逐条套上，七条都红，成立；两行「增长」变异，一条红、一条不红，而且这个门槛连「把补丁整个退回改前」都分不出，不成立。

### T4.1 等价测试比的是不是改前改后真不同的路径

测试里每种查法的左边是新接口，右边是下面这些，**右边都不经过新索引**：

| 查 | 左（新） | 右（参照） |
|---|---|---|
| `units` 按角色 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:361`：「is_the_same_entry(units_by_role.first_entry_of(&role), found_linearly),」 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:359`：「let found_linearly = version.units.iter().find(|unit| unit.identity == role);」，测试自己写的 `iter().find` |
| `mapped_units` 按角色 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:379`：「is_the_same_entry(mapped_units_by_role.first_entry_of(&role), found_linearly),」 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:377`：「.find(|(mapped, _)| *mapped == role);」 |
| 文件按 inode 号 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:402`：「files_by_inode.first_entry_of(&inode),」 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:403`：「version.file_other_than_the_first(inode)」，产品里原样留着的线性函数 |
| 叶遍历 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:426`：「read_leaf_by_leaf.locations_of_key(key),」 | `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:427`：「central_mapping_locations_in_the_version(version, key),」，产品里原样留着的逐 key 查 |

比的是表项的地址（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:338`：「(Some(indexed_entry), Some(found_entry)) => std::ptr::eq(indexed_entry, found_entry),」），不是值相等，所以「取最先那一项」与「取值相同的另一项」分得开。比的范围：造池、整理到停、再写、unlink 这一串里每一版（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:565`：「compared.versions >= 40 && compared.file_lookups >= 16 && compared.unit_lookups >= 64,」），另加一个哪一版都没有的身份。

它**没有**比的：产品里改了的调用点（`carried_unit` 的七个调用点、`resolve` 的 `carried`、文件指针、`version_as_the_compaction_reads_it`）改前改后写出的东西。测试只断言结尾池级 checker 一条违例都没有（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:542`：「"这一串操作之后池级 checker 一条违例都没有（{shape:?}）：{violations:?}"」），并把镜像摘要打出来（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:545`：「images_sha256={}」）供人在改前的副本里比，没有断言它。这一格我补做了：把冻结副本的调用点退回改前（反向打 `diff/callsites.diff`，新接口留着让测试能编），同一个测试二进制跑同一串操作：

```
改后（out/run-16.log 第 3 行、out/run-48.log 第 7 行）
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=87addacdc948bbbc00b1e0f2894f5ffe45f1e93a24f7645bd4092edcb3f2bf75
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=3344bce43504554dd5e0ca94c286b1a3b70f9ac362a374898760405876761b19
改前发布路径（out/run-old-digests.log 第 3 行与第 5 行）
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=87addacdc948bbbc00b1e0f2894f5ffe45f1e93a24f7645bd4092edcb3f2bf75
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=3344bce43504554dd5e0ca94c286b1a3b70f9ac362a374898760405876761b19
```

两档池上改前改后的镜像摘要逐字相同、整理发的批数相同。这是两条确定性的串；它不覆盖别的操作串（别的串是攻方腿的活）。

### T4.2 七行「按身份查」变异各自让点名的测试红的理由

我对每一行：从冻结副本拷出的副本里还原源文件 → 按 `crates/mutations.tsv` 那一行的原文/替换文替换（原文在源码里恰好命中一次，我也逐行核过：九行都是 1 次）→ 编 → 只跑那一行点名的测试。七条全红：

| 变异（`crates/mutations.tsv` 行名） | 点名的测试 | 结果（我的日志） | 为什么红 |
|---|---|---|---|
| 索引恒查不到 | 取最先一项的小用例 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m1.run.log:4`：「panicked at crates/singlefs-harness/tests/publish_lookup_by_identity.rs:601:9:」 | 重复身份 7 在索引里查不到，`is_the_same_entry` 断言失败 |
| 同一身份取最后一项 | 同上 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m2.run.log:4`：「panicked at crates/singlefs-harness/tests/publish_lookup_by_identity.rs:601:9:」 | 索引留下的是后出现那一项（`research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m2.run.log:5`：「身份 7：索引查到的 Some((7, "seven second")) 与线性找的 Some((7, "seven first")) 不是同一项」） |
| 单元索引漏了 `units` 最前面那一个 | 十六个文件的用例 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m3.run.log:4`：「panicked at crates/singlefs-core/src/transaction.rs:3227:14:」 | 红在产品自己的 `expect`（`FirstEntryOfEachIdentity::unit`）：发布路径要照抄的第一个单元查不到，不是测试的断言 |
| 照抄角色的映射 key 一律取 inode 根那一把 | 同上 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m4.run.log:4`：「panicked at crates/singlefs-core/src/transaction.rs:8207:9:」 | 红在产品自己的断言「进映射的单元与留存的映射条目各一把 key，互不相撞」 |
| 照抄的文件按下一个号查 | 同上 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m5.run.log:4`：「panicked at crates/singlefs-core/src/transaction.rs:12770:26:」 | 红在产品自己的 `expect("照抄的文件在上一版里")` |
| 批量读映射同一把 key 取最先那条 | 三种叶的用例 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m6.run.log:4`：「panicked at crates/singlefs-harness/tests/publish_lookup_by_identity.rs:655:9:」 | 同键两条时批量读与逐 key 查的结果不同 |
| 批量读映射几片叶共用一份解出的条目 | 四十八个文件的用例 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m7.run.log:4`：「panicked at crates/singlefs-harness/tests/publish_lookup_by_identity.rs:425:9:」 | 日志里第一处不同在版本 38（`research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m7.run.log:6`：「left: NodeMalformedOrNoEntryWithThisKey」，同一把 key 逐 key 查得到 `Found`，`research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/mut-m7.run.log:7`：「right: Found([LocationEntry」）；变异让所有叶共用根叶那一份解出的条目，落在别的叶上的 key 在这份条目里找不到（推的：读变异的替换文；没有逐版核过 38 是不是第一个多叶版本） |

要看的一点：第三到五行是红在产品自己的 panic 或断言，不是红在等价断言。这三条变异改的是「查错了项」，在发布路径上马上撞到产品的校验；如果改坏成「查到另一项但内容恰好合法」，要靠等价断言抓，而等价断言只覆盖 API 层（T4.1）。我没有造这样的变异（那是攻方腿的活）。

### T4.3 两行「增长」变异在 400% 门槛上分不分得出

**结论：分不出一半。** 「按身份查的索引退回逐项找」红；「整理读版本退回逐 key 查映射」不红，实现员的估计对。更要紧的是：把补丁的调用点整个还原成改前的写法，慢档用例也不红。

登记的命令不带 `--release`：`crates/mutations.tsv:4974`：「-p singlefs-harness --test publish_lookup_by_identity -- --include-ignored --exact a_compaction_batch_cpu_grows_less_than_the_threshold_from_512_to_2048_data_units」，所以我按 debug 跑（和登记的命令同一档）。门槛来源的注释：`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:806`：「/// 依据（2026-10-20 实现员在仓副本里量，debug、本机忙）：改后 50.0 → 154.2 ms，308%；改前的平方查找那几处在 release 下」，下一行接「512 → 2048 是 430–470%（调查员同一会话轮流量的两次）」。就是说**改前的数是 release 下量的，门槛却拿 debug 命令跑**。门槛常量：`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:808`：「const HIGHEST_GROWTH_OF_THE_BATCH_CPU_IN_PERCENT_FOR_FOUR_TIMES_THE_DATA_UNITS: u64 = 400;」。

我在自己的副本里、同一个测试二进制的路径下依次换五种代码跑慢档用例（每步 100 到 220 秒，一次只起一条 cargo；开始时本机 loadavg 约 35 到 38，线程 CPU 取 `/proc/thread-self/schedstat`，测试自己用它）。「涨了百分之几」是用例打的 `growth_in_percent`，门槛 400：

| 代码 | 第一次 | 第二次 | 小池中位（第一次 / 第二次，毫秒） | 过不过门槛 |
|---|---|---|---|---|
| 补丁（未变异） | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-base.log:8`：「growth_in_percent=310」 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-base2.log:5`：「growth_in_percent=287」 | 39.0 / 39.0 | 绿（应当） |
| 改前发布路径（调用点退回改前，新接口留着，T4.1 那棵树） | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-old.log:5`：「growth_in_percent=379」 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-old2.log:5`：「growth_in_percent=377」 | 115.0 / 117.0 | **绿（应当红）** |
| 「整理读版本退回逐 key 查映射」 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-m9.log:5`：「growth_in_percent=359」 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-m9b.log:5`：「growth_in_percent=357」 | 113.0 / 111.0 | **绿（应当红）** |
| 「按身份查的索引退回逐项找」 | `research/prompts/m4-publish-lookup-code-r1-sonnet-model/out/run-growth-m8.log:5`：「growth_in_percent=609」 | 只跑了一次 | 53.0 | 红（`out/run-growth-m8.log` 第 8 行：门槛 400%） |

读法：

- 绝对时间上补丁确实快：小池一批整理 39 ms，改前发布路径 115 ms（约三倍），和背景材料里实现员在 release 下报的降幅（P = 256、1024、4096 都降到约一半）同方向，debug 下降得更多。快不快与涨幅是两回事：debug 下每把 key 解一片 16 KiB 的叶要做校验和（`crates/singlefs-core/src/unit.rs:403` 起的 `parse_index_node` 在解析前校验净荷），这一项对池大小是线性的、常数很大，把平方项那点比例冲淡了，改前路径只涨 377 到 379%，在 400% 以内。
- 门槛要落在补丁（287 到 310）与改前（377 到 379）之间才算判得出改前改后；现在的 400 比改前最高的 379 还高 21 个百分点，所以「整个退回」和 M9 一起漏过去。
- 「整理读版本退回逐 key 查映射」的 357 到 359% 与「整个退回」的 377 到 379% 都在门槛下：这一格要换判法（例如直接数解叶次数，或把门槛放进 `release` 档量过的区间，或同时比绝对 CPU）才判得出；换成哪一种归主 agent，我只报这三个数。
- 这些数的可靠度：每格两次（「索引退回逐项找」一次），同一台本机，机器很忙，用的是 debug 档、线程 CPU 时间；没在 release 档重量，没有量空闲机器。改前路径与「整理读版本退回逐 key 查映射」两次各差 2 个百分点，补丁两次差 23 个百分点（287 与 310）；补丁与改前之间最近的差是 67 个百分点（310 与 377）。400% 门槛与它们的相对位置是在这台忙机器上量的，换机器可能挪几十个百分点，我没有量过空闲机器。

### T4 推翻条件

七行变异里任何一行在我的复跑里不红（我七条都红）；或「按身份查的索引退回逐项找」之外还有一种退回在 debug 慢档用例下超过 400%（我只试了两种退回加整个退回）；或在空闲机器上把改前发布路径跑出超过 400%（那样 400% 就分得开改前改后，只是余量小）。

## T5 有没有改到「行为不变」之外

**结论：没有。** 补丁在两份产品文件里删掉的全部行都是查法的替换（下面把删掉的行逐行列出）；次序、错误分支、文案、公开签名、盘上格式、写出次序都没动。

### T5.1 被删掉的行就是全部的「改」

补丁加的行（`+`）是新接口、调用点的新写法、文档与测试；会改变行为的只能是被删的行（`-`）。冻结副本相对改前的 `diff -u` 里被删的行，两份产品文件共 38 行（命令与原样输出）：

```
$ grep -E '^-[^-]' research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff | wc -l
22
$ grep -E '^-[^-]' research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/mounted_session.diff | wc -l
16
$ grep -E '^-[^-]' research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/transaction.diff research/prompts/m4-publish-lookup-code-r1-sonnet-model/diff/mounted_session.diff
mounted_session.diff:-    central_mapping_locations_in_the_version, prepare_the_version_publish,
mounted_session.diff:-    publish_compaction_step_in_the_session,
mounted_session.diff:-    CompactionIntentStep, CompactionStepRefused, DeletionAndLineageOfThePublish, DeletionQueueWalk,
mounted_session.diff:-    FirstFile, MappingLookup, PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError,
mounted_session.diff:-    ReadFailedOnTheDisk, ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion,
mounted_session.diff:-    ReleaseChecksumCheck, RelocationBatchRefused, TransactionOutput, TransactionUnit,
mounted_session.diff:-        .filter_map(
mounted_session.diff:-            |(_, key)| match central_mapping_locations_in_the_version(version, key) {
mounted_session.diff:-                MappingLookup::Found(locations) => Some(CentralMappingEntry {
mounted_session.diff:-                    key: key.clone(),
mounted_session.diff:-                    locations,
mounted_session.diff:-                }),
mounted_session.diff:-                MappingLookup::NodeMalformedOrNoEntryWithThisKey
mounted_session.diff:-                | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => None,
mounted_session.diff:-            },
mounted_session.diff:-        )
transaction.diff:-            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")
transaction.diff:-/// 从上一版取一个照抄进来的角色的映射 key。
transaction.diff:-            mapping_key_carried_from(
transaction.diff:-                previous.expect("照抄的角色只出现在接着上一版的发布里"),
transaction.diff:-                identity,
transaction.diff:-            )
transaction.diff:-/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
transaction.diff:-fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
transaction.diff:-    previous.unit(identity).clone()
transaction.diff:-                carried
transaction.diff:-    // （整理的一批之后上一版的拷贝里它们已换成映射位置）。
transaction.diff:-                    previous
transaction.diff:-                        .and_then(|previous_version| {
transaction.diff:-                            previous_version.file_other_than_the_first(file_plan.inode)
transaction.diff:-                        })
transaction.diff:-            Some(carried) if !rewritten.contains(&identity) => carried_unit(carried, identity),
transaction.diff:-                    previous.expect("照抄的文件只出现在接着上一版的发布里"),
transaction.diff:-                    previous.expect("没重写的叶容器只会出现在接着上一版的发布里"),
transaction.diff:-            previous.expect("没重写 inode 根的发布恒接在上一版之后"),
transaction.diff:-                    previous.expect("照抄的分配记录树节点只出现在接着上一版的发布里"),
transaction.diff:-                previous.expect("照抄的删除树节点只出现在接着上一版的发布里"),
transaction.diff:-                    previous.expect("照抄的容器树节点只出现在接着上一版的发布里"),
```

逐行看（文件名前缀省了路径）：

- `transaction.diff` 22 行：一行 `unit` 的文案改成常量（T3.1）；`mapping_key_carried_from` 文档首行（只改文档）；`carried` 闭包里的 `mapping_key_carried_from(previous.expect(…), identity)` 四行（T3.1、T3.2）；`carried_unit` 的文档、签名、体（`previous.unit(identity).clone()`，T1.1）各一行；`carried_file_version_units` 里 `carried` 一行（T1.1）；一行注释的括注（只改注释）；文件查那四行（T1.1、T3.1）；第一个文件数据单元那一臂 `Some(carried) if …` 一行；其余六行是各调用点的 `previous.expect("…")`（T3.1）。没有一行是 `units.push`、`extend`、排序、`Result`/`?` 分支、`assert`。
- `mounted_session.diff` 16 行：`use` 里的 `central_mapping_locations_in_the_version`（不再用，导入换成新类型）六行，`version_as_the_compaction_reads_it` 里逐 key 查的十行（T2.4）。

### T5.2 其余几项

- **次序**：`units` 的 `push` / `extend` 序列没有一行被删或被加（上面的列表里没有、补丁的 `+` 行里也只有 `carried_unit` 调用的实参变化）；`mapped_units` 的 `push` 序列同理（`resolve` 里只替换了 `carried` 闭包的内部）；`CentralMappingEntry` 的次序见 T2.4。盘上写出的次序取决于这些序列和 `PublishWrites` 的组装，补丁没碰。
- **错误分支与文案**：被删的行里没有 `Err`、`?`、`return Err`；panic 文案逐字不变（T3.1）；新常量与原字符串相同（`crates/singlefs-core/src/transaction.rs:3184`：「"八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在";」）。
- **公开 API**：新增的是 `pub struct FirstEntryOfEachIdentity`（`crates/singlefs-core/src/transaction.rs:3192`）、三个 `TransactionOutput` 方法（`crates/singlefs-core/src/transaction.rs:3722`：「pub fn units_by_role(&self)」及其下两个）、`pub struct CentralMappingOfAVersionReadLeafByLeaf`（`crates/singlefs-core/src/transaction.rs:4255`）；已有公开函数 `central_mapping_locations_in_the_version`（`crates/singlefs-core/src/transaction.rs:4236`：「pub fn central_mapping_locations_in_the_version(」）、`TransactionOutput::unit`（`crates/singlefs-core/src/transaction.rs:3713`：「pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit」）、`file_other_than_the_first`（`crates/singlefs-core/src/transaction.rs:3626`：「pub fn file_other_than_the_first(」）、`mapping_locations_for_key`（`crates/singlefs-core/src/transaction.rs:4700`：「pub fn mapping_locations_for_key(mapping_node_bytes: &[u8], key: &[u8]) -> MappingLookup {」）的签名都不在被删行里。签名变了的只有私有函数 `carried_unit`（`crates/singlefs-core/src/transaction.rs:11252`：「fn carried_unit(」，无 `pub`）。
- **盘上格式与写出的字节**：补丁没碰 `records.rs`、`unit.rs`、`singlefs-format` 与 checker；改前改后同一串操作写出的两块盘镜像 sha256 在两档池上逐字节相同（T4.1），整理发的批数也相同。这是两条确定性的串，不是所有串。
- **只多不少的开销**：`publish_admitted` 与 `resolve` 现在每次发布都建索引，即使这一版没有任何照抄（例如 `crates/singlefs-core/src/transaction.rs:13630`：「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」对每次有上一版的发布都建）。这是时间开销，不是行为；我没量它。

### T5 推翻条件

任何一条被删行之外的行为改动被读出（例如 `+` 行里新增了会改写出次序的分支）；或在任何一串合法操作上，改前改后的盘镜像字节不同、或同一处查到不同的项、或改后 panic 而改前不 panic（后三样是攻方腿造反例的活，我这里只有两条确定性的串）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 索引与线性查找是不是同一项 | 成立 | 五处重复身份时两边都取表里最先那一项（`or_insert` 与 `iter().find`），身份类型的 `==` 与 `Ord` 都是派生的结构比较；索引借着表，建了之后改表写不出来，核心层唯一的就地改 `units` 在建索引之前 |
| T2 整理读版本一遍走叶与逐 key 查 | 成立 | 同一个路由函数、同一张 `units` 首项；解不开 / 窄于字段表 / 同键取最后一条三种叶与三种 `MappingLookup` 结局逐一对上，`CentralMappingEntry` 的集合与次序不变 |
| T3 照抄路径 | 成立 | 三处查不到的 panic 文案与条件逐字相同，七句 `previous.expect` 文案不动；映射 key 的索引在每次 `resolve` 里现建，固定点每轮一次，不跨轮复用；相邻的 `mapping_key_carried_from` 还剩一个线性调用点（不在补丁点名内） |
| T4 测试与变异 | 部分不成立 | 等价测试参照边是改前的线性查法，成立；七行「按身份查」变异我套上七条全红（三条红在产品自己的 panic）；两行「增长」：索引退回逐项找红（609%），整理读版本退回逐 key 查不红（357、359%），整个退回改前发布路径也不红（377、379%），400% 在登记的 debug 命令下分不出改前改后 |
| T5 有没有改到行为不变之外 | 成立 | 被删的 38 行全是查法替换；公开签名只增不改；两档池上改前改后的盘镜像 sha256 逐字节相同 |

给主 agent 采纳时要看的几处（都是我量到或读到的，采不采由主 agent 定）：

1. T4.3：增长慢档用例的 400% 门槛在 debug 命令下漏掉「整个退回」。门槛的依据数里改前的数是 release 下量的（测试文件注释自己写了），登记的命令是 debug。要么门槛落在补丁与改前之间（我量到 287 到 310 与 377 到 379），要么命令改成 release 档，要么换判法。
2. T4.1：改了的调用点没有自己的等价断言，集成等价靠既有测试套件（合入后验证 2489 过）与我手比的两档镜像摘要；如果要让补丁自带这一格，测试里把摘要钉成常量。
3. T3.2：`mapping_key_carried_from` 在 `deadlist_changes_of_this_publish` 里仍是逐角色线性找，整份覆盖写大文件时换下角色数大（推的，没量）。

## 没做什么

- 不判别的腿的格：没造反例，没造「改坏查法而测试不红」的变异（T4 要的攻方活）；没替主 agent 采纳或出判决。
- 只有两条确定性的操作串的镜像摘要比对（16 个四单元文件、48 个八单元文件），没有别的串（快照、unlink 之外的删除、回退、截断……）。
- 没跑 checker 档测试、任何门禁阶段、GPU、崩溃点重放、变异整表脚本；变异是把 `crates/mutations.tsv` 的行用 `tools/apply_mutation.py` 在自己的副本里套上，只跑了点名的那一个用例。
- 没在 release 档量增长；没在空闲机器上量（开工时本机 loadavg 约 35 到 38）；「索引退回逐项找」只量了一次。
- 没在第二台复跑合入后验证的 254 个二进制，只数了它的日志（254、2489、78、0 红）。
- 没查 extent 树与多层码 2 树照抄节点时调 `TransactionOutput::unit` 的延后项（背景材料记作延后），也没量它。
- 没有逐个查 `DataPointer` 等嵌套类型有没有内部可变（T1.2 的论证不依赖它：键是三个普通 `Copy` 字段）。
- 没逐版核 M7 日志里的版本 38 是不是第一个多叶版本。
- `rerun.sh` 的增长四步没有经它整条重跑，用的是等价的直接命令（日志在 `out/`）。
- 自建的仓副本和编译目录（草稿目录下的 `repo/`、`repo-old/`、`target/`、`target-old/`、`rerun-check/`）交回前删了，只留日志与脚本；模型目录里有脚本、diff、日志。

## 引文核对

命令（两个根：冻结副本放 `crates/` 的行，仓根放模型目录里的 diff 与日志）与末行原样输出：

```
$ python3 scripts/agents/cite-check.py research/prompts/m4-publish-lookup-code-r1-sonnet-output.md --root <冻结副本> --root <仓根>
  ✓ 核了 158 处引文、11 处裸行号，对上 158 处，没判 11 处（其中裸行号没有引文可比 11 处；逐处列在上面）
```

注意：写报告途中主工作区的 `crates/singlefs-core/src/transaction.rs` 被别的会话改过（0.1），只用默认根（仓根）跑会在这个文件的行号上对不上；用 `--root` 指到冻结副本跑，或带 `--unchanged-since` 让它把改过的文件列进「没判」。
