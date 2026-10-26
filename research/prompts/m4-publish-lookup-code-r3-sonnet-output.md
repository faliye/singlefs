# m4-publish-lookup-code-r3 云端正推腿报告：七·6 第三批补丁（T1–T4）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

立场：从冻结副本的代码与已定条款推「应该是什么」，再与补丁逐格比。冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo/`；引文行号都是它的行号（主工作区这几份文件与它的 sha256 相同，`sha256sum` 对过：transaction.rs `2d69b817bed2…`、mounted_session.rs `d31503616b7c…`、publish_lookup_by_identity.rs `c7c37804f2c4…`、mutations.tsv `82e557e47705…`）。
压着的条款：`.claude/kb/milestone/04-fourth-txn.md` 第 142 行（七·6 那一行）与 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」。第二轮判决：`research/prompts/m4-publish-lookup-code-r2-main-verification.md`「三、交给下一批的实现员（按文件聚簇）」与「五、交用户」。

## 复跑与模型文件

模型目录：`research/prompts/m4-publish-lookup-code-r3-sonnet-model/`（`SHA256SUMS` 自己的 sha256 前 16 位 `e5069e0f85ac0303`，下面逐个文件）。复跑（整条经内存包装，工作目录随意，`--work` 给一个新的草稿目录；编译与跑用例在两棵树上各一遍，本机忙时要超过 4 分钟，前台 240 秒不够，由主 agent 起）：

```text
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-code-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-code-r3/repo --work <新的草稿目录> --model /home/fy5090/code/singlefs/research/prompts/m4-publish-lookup-code-r3-sonnet-model
```

跑完看这几行：`name=reverse_patch 5 files`、`name=differential before_vs_after=identical lines=9`、`FWD_COVERAGE` 五行、`FWD_EDGE compared=2033`、`name=rerun_compare 9 of 9 expected lines found`。变异与键的步骤不在 `rerun.sh` 里，分别是 `tools/apply_mutation.py` 套 `muts/<名>.old/.new`（流程见「〇」）和 `tools/group_keys.py`（`out/group-keys.txt` 是它的原样输出）。

```text
735ddb559ee403d8cee8480d2727b298c90e0d5d4ae015e42bb8cca667a81be3  ./forward_coverage_test.rs
19f0f31aa4c191325ec6a5bcdc5f8baab12761e42e6ae3600e3676cd4bf16776  ./forward_digest_test.rs
472742b27338ef9f4356d5075bba2aa12a0770ab8976b682080c0ef0581e4107  ./forward_edge_cases_test.rs
57e19d6aaabddeeaac95ce1d1f83ce78095a1b0e7e7bdce28c2d5b5317fbf585  ./muts/m1.fast.log
0b7945d4fc020759780b44b92d09933af6664c8f1bbfbfa75e7f4d9e5678602e  ./muts/m1.new
6ad4deffe11241679ee1d5d84a8a5f213df52b277b6462e691ee7f1d143d007f  ./muts/m1.old
71fa0e93435a9dfe12683a7e71add068ed669624ea99e7c726debe92234ce315  ./muts/m2.fast.log
ebdaf6e01741ff97ad1615c45349c9bac18c884048b67e248a0c5db90032a005  ./muts/m2.new
7c5a47f1e7d84283a7391d7b3c4583120474ff881ddb86be965228401c1a868f  ./muts/m2.old
33f43180cddfb47e1b1101b0c2f08feb96cd3c529cfddc9d953f2755ae6f7888  ./muts/m3.fast.log
5efefd50aefdc9e3fc1830d366ed25e0d1635388178f3a4cbbb4368b9fbae677  ./muts/m3.new
76e3088039c0ae5d941cc2443c4eb6ae691c0549433e23a938ced16dd8e7ddef  ./muts/m3.old
3ac6ac715d468b31f3385b004fe59b50fb96756bb56c0199a111f721a7203130  ./muts/m4.fast.log
1a6b2f7771b47c36b4295e98a97ec517c3e58c3b08a3f6726da5e47313c08c01  ./muts/m4.new
0478a0cac3270371b46b304d2197a4c1a4a1f202b93aabccffa5dfc875ecd9c7  ./muts/m4.old
124982403fefe0eb0d1263d46979b269426787c3617922037e986b3896330bca  ./muts/m7.fast.log
838280b04260995dc506c5cc2f95cd66101a4e7df98901d385e7be34db35ab77  ./muts/m7.new
0a73d67189aae7d95e46b39b24869c4d9600f45fed380a9ddb5540bfd3d3ee3b  ./muts/m7.old
b459d4e8c372acde1e716a708bea81348c5ed57e53c86bc4bbb2f2950f729695  ./muts/m7.slow.log
86e6aee7f5c59e54e66eb27b8c2bb6ceaca2d1ae5df4ae7406df16c60b3425b0  ./muts/m8.fast.log
2702f5220b6833c91a353067dd76949073da2c4f3af2b6c13dacd0d0c4f6bb8f  ./muts/m8.new
cda4abf3b0320c42c15b653adc370b4b964eae08a61e06bacfea5f1959606c4c  ./muts/m8.old
6ad4deffe11241679ee1d5d84a8a5f213df52b277b6462e691ee7f1d143d007f  ./muts/none.new
6ad4deffe11241679ee1d5d84a8a5f213df52b277b6462e691ee7f1d143d007f  ./muts/none.old
58238c286fb3e711f1ae92ce3c2f4dae6028ffbfef493f8e44c590a3ebe51caf  ./out/coverage.txt
df01d29c36693c2edf3fca8f5f1a1bbc7b7bf9cf7e5051c7e55e244bd29fefae  ./out/differential-new.lines
df01d29c36693c2edf3fca8f5f1a1bbc7b7bf9cf7e5051c7e55e244bd29fefae  ./out/differential-old.lines
4b89da3585cc847e80b1c5686c56318390325e87c7283614ed5dff127374713e  ./out/edge-cases.txt
45badbb531327ba1ba2a0f0b57171f419f64f4a146411113c11c964fb3117cae  ./out/expected-lines.txt
6423de921d3266d82939c6aa9546df110238e5e2648c23e925cec4b1d61bb9b7  ./out/group-keys.txt
36113753516236a000f6d290678fe96c8076fd8a9d21b3e6f07da0b37a65a8c5  ./out/mutations.txt
c4429f6c09cac9fca9c0d6397ca41046b8ceacb8abd4dbac471b73f62e3ebf9c  ./patch.diff
7d6dd74456836da0e6e7097fae9770007fee29dedcb157a573e692e9fd109ac2  ./tools/apply_mutation.py
15c16980648c01608c02ddcc9795744e5bc729aed20f6da5d5458f84ee656405  ./tools/group_keys.py
046c0e442097a4acea00ff1adeaf9bbfe602a457b43a2756e4c8d983cbf0add3  ./tools/rerun.sh
```

## 〇、我怎么量的

- 改后树 = 冻结副本的拷贝；改前树 = 同一份拷贝把第三批补丁（`research/prompts/_m4-publish-lookup-code-r3-diff.md` 里 `crates.patch` 一节）反向打回（`patch -R -p1`，五个文件全部 `patching file`，没有偏移与拒绝块）。两棵树各编各的（`--target-dir` 各一个），都经 `capped.sh 31` 与 `run-with-memory-cap.sh 16G`。
- 差分：同一份差分用例（`forward_digest_test.rs`）原样追加在两棵树的 `publish_lookup_by_identity.rs` 末尾，加上该测试文件自带的两条镜像摘要用例；改前改后各取摘要行：9 行逐行相同（`out/differential-old.lines`、`out/differential-new.lines`，`diff` 无输出）。
- 其余量法：覆盖用例 `forward_coverage_test.rs`（按处打印计数）、边界用例 `forward_edge_cases_test.rs`、变异套在改后树上（`muts/`、`tools/apply_mutation.py`）、判法单元键（`tools/group_keys.py`）。输出原样在模型目录 `research/prompts/m4-publish-lookup-code-r3-sonnet-model/out/`。
- 本机负载 loadavg 约 80–90（别的会话在跑），所以 `tools/rerun.sh` 整条在前台 240 秒内编不完；旧树那一半在 `rerun.sh` 里跑通（出了 `old.lines`），新树那一半我用等价的手工命令跑了（输出即上面几份）。整条 `rerun.sh` 的跑法见「要主 agent 跑的命令」。

## T1　前两轮判成立的，在新索引下还成立吗

结论：**成立**。

### T1.1　稳定排序加 `dedup_by` 对每种身份类型是否留下表里最先那一项

- 索引的建法：`mut pairs: Vec<(Identity, Value)>,` 在 `crates/singlefs-core/src/transaction.rs:3441`；`pairs.sort_by(|(left, _), (right, _)| left.cmp(right));` 在 `crates/singlefs-core/src/transaction.rs:3443`（`slice::sort_by` 是稳定排序）；`pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);` 在 `crates/singlefs-core/src/transaction.rs:3445`（`dedup_by` 对相邻相同的一串留第一项、丢后面的）。收进来的次序是表的次序：`identities_with_their_entries.push((identity_of(entry), entry))` 在 `crates/singlefs-core/src/transaction.rs:3534`。三者合起来：同一身份的几项稳定排序后保持表里的先后，`dedup_by` 留的是最先那项。
- 身份类型只有三种进这个索引：角色 `TransactionUnit`（`units` 与 `mapped_units` 按角色）、映射 key 的字节切片 `&[u8]`（`mapped_units` 按 key）、`InodeNumber`（文件按 inode 号）。`TransactionUnit` 的派生在 `crates/singlefs-core/src/transaction.rs:2465`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」；`InodeNumber` 是 `crates/singlefs-core/src/address.rs:44` 的 `#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]`，字段 `pub struct InodeNumber(pub u64);` 在第 45 行；`&[u8]` 用标准库的 `Ord` 与 `==`（字典序，两者一致）。
- `Ord` 与 `==` 一致的证据：`grep -rn 'impl.* \(Ord\|PartialOrd\|PartialEq\|Eq\) for' crates/*/src` 在冻结副本里只有 `singlefs-checker-tier` 的两处（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:161`、`crates/singlefs-checker-tier/src/crash.rs:1682` 起），`singlefs-core`、`singlefs-format` 里零命中，也就是 `TransactionUnit` 里嵌的每个位置类型都是派生的。派生的 `PartialOrd`/`Ord` 与 `PartialEq` 按字段同序比，`cmp == Equal` 当且仅当 `==`。
- 旧索引是 `BTreeMap` 的 `entry().or_insert`（补丁里 `-` 行，`research/prompts/_m4-publish-lookup-code-r3-diff.md`），同样只要 `Ord`；所以对三种身份类型，改前改后「取最先一项」的语义一致。

### T1.2　折半查在空表、单项、全同身份、查不到时与线性找逐项相同

- 折半查本体：`if low >= high {` 在 `crates/singlefs-core/src/transaction.rs:3457`；`let middle = low + (high - low) / 2;` 在 `crates/singlefs-core/src/transaction.rs:3460`；三个分支 `std::cmp::Ordering::Less => low = middle + 1,` 在 `crates/singlefs-core/src/transaction.rs:3464`、`std::cmp::Ordering::Greater => high = middle,` 在 `crates/singlefs-core/src/transaction.rs:3465`、`std::cmp::Ordering::Equal => break Some(value),` 在 `crates/singlefs-core/src/transaction.rs:3466`。空表 `low >= high` 直接 `None`；全同身份的表先被 `dedup_by` 压成一项，折半查只面对每个身份至多一项的表，找到哪一项都是「最先那项」。
- 实测：边界用例 `forward_leg_edge_cases_of_the_sorted_index`（改后树，`test … ok`，`FWD_EDGE compared=2033`）：空表（身份取 u64）、单项、全同身份 4 项、2000 项只有 7 个不同身份（次序被步长打乱）、长短不一与互为前缀的字节串 key（含空串与 27 字节的 `0xff`）、`TransactionUnit` 里 `InodeRoot`/`Data(3)` 各出现两次，每个身份加几个表里没有的身份，逐个对 `iter().find` 比地址（`is_the_same_entry` 比地址，不比值），2033 次比对全同。原样输出在 `out/edge-cases.txt`。
- 测试文件自带的重复身份用例 `first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones`（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:867`）只有 5 项的 u64 表；它能抓「去掉 `dedup_by`」（身份 3 在无 dedup 时折半查落在第二项），抓不到「稳定排序换成不稳定排序」：我套了变异 M1（`sort_by` 换 `sort_unstable_by`），`publish_lookup_by_identity` 快档 5 条全绿（`out/mutations.txt` 的 `## m1`）。这是等价变异在真数据上（身份唯一）的表现，不是行为反例；但「同一身份取最先」的文档承诺在重复很多的大表上没有测试守（见 T4）。

### T1.3　查不到时的 panic 与错误文案、条件与改前逐字相同

- 单元查：`.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)` 在 `crates/singlefs-core/src/transaction.rs:3566`（索引的 `unit`）与第 4057 行（`TransactionOutput::unit`），常量文案不在这一批补丁里（补丁上下文里两行原样）。
- 映射 key 查：`.expect("照抄进来的角色在上一版里进映射、有一把 key")` 在 `crates/singlefs-core/src/transaction.rs:3580`，不在补丁改动行里。
- 新的按角色取条目：`.expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");` 在 `crates/singlefs-core/src/transaction.rs:4614`；被删的 `mapping_locations_of_a_mapped_unit` 里那句在补丁的 `-` 行 `research/prompts/_m4-publish-lookup-code-r3-diff.md:456`。两句用 `grep -nF` 对过，字节逐个相同（改前树第 4296 行、改后树第 4614 行各命中一次）。panic 的文案不变，位置（文件:行）变了，不影响文案比对。
- 条件：改前是 `iter().find` 找不到才 panic；改后是 `first_entry_of` 返回 `None` 才 panic；两边的「找不到」是同一个集合（表里没有这个身份）。

### T1 推翻条件

- 找到一个进索引的身份类型，它的 `==` 与 `cmp == Equal` 不一致（例如手写的 `impl PartialEq`）：`grep` 零命中，推翻要先有这样的新类型。
- 找到一张表，有同一身份的几项且 `dedup_by` 之后留的不是最先那项：边界用例 2033 次比对没有，推翻要给出不同的表。
- 差分里改前改后任一摘要行不同：9 行相同。

## T2　并进来的两处查法换了之后的行为

结论：**成立**（一处没验：错误路径没有在调用点上现跑，只读码；见 T2.5）。

零命中的核对：`grep -rn 'mapping_locations_of_a_mapped_unit' crates --include=*.rs` 在冻结副本里没有任何一行（函数与三个调用点都删净了，只剩补丁里的 `-` 行）。

### T2.1　按角色取映射条目（原 `mapping_locations_of_a_mapped_unit`，三个调用点，实际落在四个函数里）

新的批量形态：`pub struct MappedUnitsOfAVersionReadByRole<'version> {` 在 `crates/singlefs-core/src/transaction.rs:4572`；`.get_or_insert_with(|| IndexesOfAVersionReadByRole {` 在 `crates/singlefs-core/src/transaction.rs:4608`（第一次查才建）；`let (_, key) = mapped_units_by_role` 在 `crates/singlefs-core/src/transaction.rs:4612`、`central_mapping.locations_of_key(key)` 在 `crates/singlefs-core/src/transaction.rs:4615`。改前（补丁 `-` 行）：`-fn mapping_locations_of_a_mapped_unit(` 在 `research/prompts/_m4-publish-lookup-code-r3-diff.md:448`、`-        .find(|(mapped, _)| *mapped == identity)` 在 `research/prompts/_m4-publish-lookup-code-r3-diff.md:455`、`-    central_mapping_locations_in_the_version(previous, key)` 在 `research/prompts/_m4-publish-lookup-code-r3-diff.md:457`。

逐处（调用位置 / 改前取哪一项 / 改后取哪一项）：

| # | 函数（冻结副本行） | 改前 | 改后 |
|---|---|---|---|
| a | `placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots`（`pub fn` 在 `crates/singlefs-core/src/transaction.rs:4295`），释放判定路径；入口调用方 4281、11344 | `-                match mapping_locations_of_a_mapped_unit(previous, identity) {`（`research/prompts/_m4-publish-lookup-code-r3-diff.md:435`）：在 `previous.mapped_units` 上按角色线性找最先那把 key，再逐 key 查中央映射 | 4305 行建 `let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);`，4339 行 `match previous_read_by_role.locations_of_role(identity) {`：按角色索引取最先那把 key，再走批量读者 |
| b | `refuse_mapping_entries_that_do_not_name_two_pool_devices`（4809），核位置项指池里两块不同的盘；调用方 4884、11128 | 补丁 `-    match mapping_locations_of_a_mapped_unit(previous, identity) {`（`research/prompts/_m4-publish-lookup-code-r3-diff.md:557`）、`-        let locations = mapping_locations_of_a_released_unit(previous, identity);`（`research/prompts/_m4-publish-lookup-code-r3-diff.md:572`） | 4818 行建读者，4822 行 `let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);`，帮手函数 `fn mapping_locations_of_a_released_unit(` 在 4784、里面 4788 行 `match previous_read_by_role.locations_of_role(identity) {` |
| c | `copies_failing_the_release_checksum_check`（`pub fn` 在 4876），释放前读盘核校验和；调用方 11118 | 同 b，补丁 588 行 `-        let locations = mapping_locations_of_a_released_unit(previous, identity);` | 4894 行建读者，4898 行同 b 的写法 |
| d | `deadlist_changes_of_this_publish`（6024），快照判定那一圈进 deadlist 的那一支；调用方 8052 | 补丁 `-                    match mapping_locations_of_a_mapped_unit(previous_version, *identity) {`（`research/prompts/_m4-publish-lookup-code-r3-diff.md:616`） | 6050 行 `MappedUnitsOfAVersionReadByRole::of_the_version(previous_version);`，6071 行 `let locations = match previous_version_read_by_role.locations_of_role(*identity) {` |

四处的共同点：
- 每个调用点之前都在同一个 `previous` 上逐角色查，改前每次 `find` 一遍 `mapped_units`、再逐 key 重解叶；改后一个函数一次调用建一份读者（c 里还多一份：它先调 b（4884 行），b 自己建一份，所以同一次发布多建两个索引，只是代价不是正确性）。
- 同一角色几把 key 时：改前 `iter().find` 取表里最先那把，改后索引取最先那把（T1.1），逐个相同。同一把 key 对应几个角色在这一格不出现（这里是角色 → key 的方向）。
- d 里除了这一支，还有同一函数 6053 行 `let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();` 与 6056 行 `.mapping_key_of(*identity)`：这是第二批就有的、对每个换下的角色取 key 的索引，第三批没动；所以 d 里同一个 `previous_version` 上同时有两份按角色的 `mapped_units` 索引（一份给取 key，一份在 `MappedUnitsOfAVersionReadByRole` 里），多建一份，不改行为。

### T2.2　整理发批核源落点（`relocation_checked_against_the_version`）

- 调用位置只有一处：`crates/singlefs-core/src/transaction.rs:9695` 的 `relocations.push(relocation_checked_against_the_version(` 在 `publish_compaction_step_in_the_session`（`pub(crate) fn` 在 `crates/singlefs-core/src/transaction.rs:9623`）里，整理的入口 `publish_relocation_batch`（`pub fn` 在 `crates/singlefs-core/src/transaction.rs:9447`）经 `publish_compaction_step`（`pub fn` 在 `crates/singlefs-core/src/transaction.rs:9520`）走到它；那一圈由 `let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);`（9690 行）圈住，`let mut previous_read_by_key = None;`（9693 行）在循环外，第一项要查时才建：`previous_read_by_key.get_or_insert_with(|| {` 在 9698 行、`MappedUnitsOfAVersionReadByKey::of_the_version(previous)` 在 9699 行。计划为空时一次都不建。
- 改前：补丁 `-    let holder_and_locations = match previous`（`research/prompts/_m4-publish-lookup-code-r3-diff.md:708`）接 `-        .find(|(_, key)| *key == mapping_key)`（711 行）与 `-            let locations = match central_mapping_locations_in_the_version(previous, &mapping_key) {`（717 行）：每一项在 `previous.mapped_units` 上按 key 线性找最先那一个角色，再逐 key 查中央映射。
- 改后：按 key 找角色 `.mapped_units_by_key` 在 9896 行、`.first_entry_of(&mapping_key.as_slice())` 在 9897 行；取条目 `.locations_of_key(&mapping_key)` 在 9902 行；索引由 `FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(_, key)| key.as_slice())`（4080 行）建，同一把 key 取最先那一项。
- 同一 key 对应几个角色时：改前取最先那个角色，改后索引取最先那个角色，相同。同一角色几把 key 的方向不出现。
- 留存的映射条目那一支（`.find(|(key, _)| *key == mapping_key)` 在 9917 行，查 `previous.mapping_entries_kept_for_the_snapshots`）这一批没动，仍是线性（见 T3.3）。

### T2.3　索引与批量读者建了之后表会不会被改

- 读者持的是共享借用：`version: &'version TransactionOutput,` 在 `MappedUnitsOfAVersionReadByRole` 里，各函数的参数也是 `previous: &TransactionOutput,`（`crates/singlefs-core/src/transaction.rs:4296`）。读者活着时那张表改不了，由借用检查守住。
- 内部可变性：`grep -n 'RefCell\|Cell<\|Mutex\|RwLock\|Rc<\|Arc<' crates/singlefs-core/src/transaction.rs` 只在计数模块里命中（`use std::cell::RefCell;` 在 3262 行起、线程局部变量 3354 行），`TransactionOutput`（`pub struct TransactionOutput {` 在 3144 行）自己没有。

### T2.4　改前改后同一串合法操作写出的盘镜像是不是逐字节相同

三串操作，改前树与改后树各跑一次，摘要取「两块稀疏盘写过的扇区」的 sha256（测试文件自带的 `digest_of_the_images`）。改后树的原样输出（改前树逐行相同，`diff` 退出码 0）：

```text
publish_lookup_by_identity shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
FWD_DIGEST name=after_create_snapshot sha256=fbb9782c01c7b7fefa38c5c469fb596d2518ce6619342d493e5afca86a689bd2
FWD_DIGEST name=after_overwrites_under_snapshot sha256=250d331b0c4aa35bbac7bad3a0827552317de883fcdbb83ba3841d1828f385a1
FWD_DIGEST name=after_compaction_with_snapshot sha256=9b745719627211945c30b68027ee6661a783b2c57478d93182f7d61d9ea10513
FWD_DIGEST name=after_unlink sha256=9894a1234b0007401ca2ab1868ec0bdf40c4a51c54e8d3a6589dc26565154d57
FWD_DIGEST name=after_destroy_snapshot sha256=3abe678e5f26818e4f568a8f6ab1923e5e61f04c4e3ca249e7b3bff5b155bbb9
FWD_DIGEST name=after_final_compaction sha256=3abe678e5f26818e4f568a8f6ab1923e5e61f04c4e3ca249e7b3bff5b155bbb9
FWD_DIGEST batches=9
publish_lookup_by_identity shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
```

- 前后两串（16 个 4 单元文件、48 个 8 单元文件）是测试文件自带的历史：造池、整理推到停、整份重写、unlink，没有快照。
- 中间一串是我写的（`forward_digest_test.rs`）：16 个 4 单元文件的池 → 建快照 → 快照下整份重写每第 3 个文件（走快照判定那一圈进 deadlist 的那一支）→ 整理推到停（带快照，走留存的映射条目那一支）→ 再重写 → unlink → 删快照 → 再整理。池级 checker 判绿（用例里 `assert!(violations.is_empty()…`）。`after_destroy_snapshot` 与 `after_final_compaction` 同值，是因为最后那趟整理一版都没发——这两格不区分，别的五格各不相同。
- 覆盖：覆盖用例（`out/coverage.txt`）在同一串历史上按处打印计数，四个计数点都被走到：`SnapshotJudgement` `mapping_leaf_decodes: 18`（只有进 deadlist 那一支在这一处解叶，说明第 6071 行那一支真跑了）、`MappingEntriesOfReleasedUnits` 700 次查、`CompactionReadsTheCentralMapping` 12 次、`RelocationSources` 24 次。
- 没覆盖：`ReleaseNotInMapping` 与 `MappingEntryNarrowerThanItsFieldTable` 两条错误路径没有在这四个函数的调用点上现跑（需要造一版映射叶被改坏的上一版再发布）。

### T2.5　查不到时的错误与 panic 是不是逐字相同

- 错误：a 的 `return Err(PublishError::ReleaseNotInMapping { unit: identity })` 在 `crates/singlefs-core/src/transaction.rs:4342`；d 的 `return Err(PublishError::ReleaseNotInMapping { unit: *identity })` 在 `crates/singlefs-core/src/transaction.rs:6074`；两处的匹配臂在补丁里只有缩进变化（`-` 行 435 起与 616 起，条件与载荷字段一一对应）。b、c 经帮手函数在「查不到或条目窄于字段表」时 `panic!`（帮手函数在 4784 行起，补丁里只换了参数 `previous` → `previous_read_by_role`），因为调用方先走 a 那一路（同一个上一版、同一串角色）已经报过错，走不到这里。
- panic：`.expect(…)` 文案逐字不变（T1.3）；「叶角色不在 `units` 里」的 panic 来自 `units_by_role.unit(leaf_role)`（`CentralMappingOfAVersionReadLeafByLeaf::locations_of_key` 里，`.or_insert_with` 之内），文案是同一个常量 `EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION`（3566 行）；改前是 `version.unit(leaf_role)`（4057 行）也是它。触发条件相同：第一把走到这片缺失叶的 key。
- 判不了：以上是读码；没有现跑一版叶被改坏的上一版去比两边的错误值与 panic 文本。

### T2 推翻条件

- 差分里改前改后任一摘要行不同（现在 9 行相同）；
- 某一版里同一角色在 `mapped_units` 出现多把 key，而 `locations_of_role` 取的不是最先那把（边界用例对 `TransactionUnit` 做过重复，2033 次比对相同）；
- 某个调用点上，读者活着时有代码改了 `previous`（借用检查与「`TransactionOutput` 无内部可变性」排除了它；要推翻得先改类型）；
- 造出一版叶被改坏的上一版，两边的错误或 panic 文本不同。

## T3　计数判的是不是产品调用点上的事

结论：**部分不成立**。计数本身记在产品调用点上，没有别的线程漏数，用例并行不串数，200% 门槛站得住；但「改坏之后计数一定涨、测试一定红」不成立：我套了四条第三批改法改回去的变异，快档全绿（其中一条连慢档也绿）。

### T3.1　逐处列出每个计数点

进处（`lookup_work::entering`，返回的守卫活着的时候本线程的查找记在这一处）：

| # | 位置 | 点名的处 | 圈住什么 |
|---|---|---|---|
| 1 | `crates/singlefs-core/src/mounted_session.rs:2794`（`lookup_work::entering(LookupSite::CompactionReadsTheCentralMapping);`，绑定在 2793 行） | 整理读一版取全部进映射单元的映射条目 | `central_mapping_as_the_compaction_reads_it` 整个函数体 |
| 2 | `crates/singlefs-core/src/transaction.rs:4302`（`let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);`） | 经映射释放的角色取映射条目 | T2.1 的 a 整个函数 |
| 3 | `crates/singlefs-core/src/transaction.rs:4816`（同样的写法） | 同上 | T2.1 的 b 整个函数 |
| 4 | `crates/singlefs-core/src/transaction.rs:4891`（同样的写法） | 同上 | T2.1 的 c，从 `refuse_…` 那一趟之后起 |
| 5 | `crates/singlefs-core/src/transaction.rs:6036`（`let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);`） | 快照判定那一圈 | `deadlist_changes_of_this_publish` 从取上一版之后到函数尾 |
| 6 | `crates/singlefs-core/src/transaction.rs:9690`（`let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);`） | 整理发批核源落点 | T2.2 的循环那一块（只圈 `relocation_checked_against_the_version` 的循环） |

记处（`count`，记在最里层那一处；`let site = state.sites_entered.last().copied();` 在 `crates/singlefs-core/src/transaction.rs:3414`，不在任何一处里就记在 `None` 下）：

| # | 位置 | 记什么 |
|---|---|---|
| 1 | `crates/singlefs-core/src/transaction.rs:3446`（`count(|work| work.index_builds += 1);`） | 建一次索引 |
| 2 | `crates/singlefs-core/src/transaction.rs:3471`（`work.index_lookups += 1;`，在 `get` 里） | 查一次，连同本次折半看过几项与最大的索引 |
| 3 | `crates/singlefs-core/src/transaction.rs:3496` 与 3504（`count(|work| work.index_entries_examined += 1);`） | 逐项走索引时每交出一项 |
| 4 | `crates/singlefs-core/src/transaction.rs:4635`（`lookup_work::count(|work| work.mapping_leaf_decodes += 1);`，在逐 key 查 `central_mapping_locations_in_the_version` 里，该函数在 4623 行） | 逐 key 查每次一次解叶 |
| 5 | `crates/singlefs-core/src/transaction.rs:4663`（同样写法，在批量读者的 `of_the_leaf_bytes` 里，该函数在 4662 行） | 批量读者每解一片叶一次 |

实测每一处在一串真历史里都被走到（`out/coverage.txt`，16 个 4 单元文件 + 快照下重写 + 整理推到停）：`SnapshotJudgement` 查 261 次、解叶 18 次；`MappingEntriesOfReleasedUnits` 查 700、建索引 162、解叶 81；`CompactionReadsTheCentralMapping` 查 12、建 12、解叶 12；`RelocationSources` 查 24、建 18、解叶 7；`None`（不在任何点名处里）查 5419、建 98，解叶 0。

### T3.2　产品里有没有放到别的线程上做的查找；有没有不经 `lookup_work` 的查找路径

- 别的线程：`grep -rn 'thread::\|spawn\|rayon\|scope(' crates/singlefs-core/src crates/singlefs-core/Cargo.toml` 零命中。核心层不起线程，一次发布或一趟整理的查找都在调它的那个线程里做。工作区里别处起线程的（`singlefs-harness/src/history.rs`、`fault_injection.rs`，`singlefs-checker-tier/src/crash_injection.rs`、`crash_state_enumeration.rs`，`singlefs-unit-graph/src/workspace.rs`）是测试脚手架、崩溃枚举与建键工具，没有一处读计数。
- 不经 `lookup_work` 的两类路径，都不是这一批改出来的，也不在 kb 七·6 点名的五处加两处里，但计数看不见它们：
  - 对 `mapping_entries_kept_for_the_snapshots` 的线性找：`.find(|(key, _)| *key == mapping_key)` 在 `crates/singlefs-core/src/transaction.rs:9917`（整理发批核源落点里留存那一支，在 `RelocationSources` 圈内），`.find(|(key, _)| *key == entry.mapping_key)` 在 `crates/singlefs-core/src/transaction.rs:6116`（快照销毁批取留存条目，在 `SnapshotJudgement` 圈内）。它们是原始的 `iter().find`，不经 `SortedIdentityIndex`，计数和任何一条断言都看不到。
  - 回退路径逐 key 查：`crates/singlefs-core/src/mount.rs:8743`（`user_visible_units_through_the_mapping`）与 `crates/singlefs-core/src/mount.rs:9222`（`mapping_keys_whose_target_locations_differ_from_the_current_mapping`）对每把 key 调逐 key 查 `central_mapping_locations_in_the_version`：解叶会记进 `None`（4635 行），但没有断言读它，且每次查里 `version.unit(leaf_role)` 仍是在 `units` 上线性找。
  - 另有 `.filter(|identity| !kept_for_the_snapshots.contains(identity))` 在 `crates/singlefs-core/src/transaction.rs:4308` 与 4742：在留存列表上线性 `contains`，同样不计数。
- 边界：`take_the_mapping_positions_over_the_location_hints`（`crates/singlefs-core/src/transaction.rs:3660`）里 `.find(|unit| unit.identity == *identity)`（3751 行）只在 `position_of_key` 给出位置的 key 上才找，整理发批对上一版拷贝调它（13019 行）时就是被搬的那几个（批大小 K），不是 P 个；从盘上重建一版那次（`recovery.rs`）是每把 key 一次，不在发布路径上。

### T3.3　测试读计数的方式，在用例并行时会不会串数

- 计数是线程局部变量（`thread_local!` 在 `crates/singlefs-core/src/transaction.rs:3353`）。测试里只经 `lookup_work_while` 读（`pub fn lookup_work_while<Output>(` 在 3366 行）：先 `mem::take` 挪开此前的、量完加回去，套着量也各量各的。
- 发布的量只包住发布那一次调用：`lookup_work_while(|| session.publish_user_change(devices, change));` 在 `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:263`，`on_every_version` 里测试自己做的查找在窗口外；整理的量 `let (run, lookup_work) = lookup_work_while(|| session.run_compaction(devices));` 在测试文件 235 行；整理读一版 `lookup_work_while(|| central_mapping_as_the_compaction_reads_it(version));` 在 478 行。
- 慢档的两档并行：`std::thread::scope(|scope| {` 在测试文件 1200 行，`let smaller = scope.spawn(|| {` 在 1201 行；每个线程里调 `lookup_work_of_the_compaction_calls_on_a_pool_of`（1162 行）——造池、重挂、量都在同一个线程里做，量出来的 `Vec<LookupWork>` 按值经 `join` 交回，没有跨线程读线程局部变量。libtest 一条用例一个线程。
- 一个小缺口：`lookup_work_while` 没有在 panic 时恢复挪开的计数（没有守卫）。只在被量的动作 panic、即那条用例本来就红了的时候发生，同一线程上的下一条用例开头的 `mem::take` 会把残留挪走，所以不会串到别的用例的读数。我没有另外跑 `--test-threads=1`。

### T3.4　门槛（200%）会不会被别的因素推过线或压下线

- 度量：`growth_in_percent_of_the_entries_examined_per_lookup` 取两档「看过的索引项数 ÷ 查的次数」之比；门槛 `const HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;` 在测试文件 1084 行；每一趟建索引的次数之比 `larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);` 在 1227 行。
- 读数（变异 M7 的慢档那一趟原样，`out/mutations.txt`；M7 只改建索引的次数，不改看过几项）：小池最大索引 606 项、大池 2379 项（二进制位数 10 与 12，折半查预期约 120%），读到 `growth_in_percent=123 index_builds_growth_in_percent=100`；查的次数 15064 与 59033（约 3.9 倍）。逐项找按表长涨（四倍表约 400%，推的，没有单独量）。读数离门槛上方还有 77 个百分点的余量。
- 推过线的因素：`first_entries` 的逐项走也记看过的项数，但它只有测试用（`pub fn first_entries(&self)` 在 `crates/singlefs-core/src/transaction.rs:3551`，产品里没有调用方，`grep -rn 'first_entries()' crates` 只命中测试文件 521 行），所以不会把产品读数推过线。这个函数是产品 API 面里一个无特性开关的 `pub`，只给测试用。
- 压下线的因素（更要紧）：度量只看经 `SortedIdentityIndex` 的查找。改回原始的 `iter().find`（不经索引）不会涨「看过的项数」，只会让「查的次数」少几次，所以平均值纹丝不动；它只被下面的下界断言抓，没有下界断言的地方抓不到。

### T3.5　改回线性、每次重建索引、每把 key 重解叶：计数是不是一定涨，测试是不是一定红

按改法分三类：

| 改坏的类 | 计数怎么动 | 哪条断言会抓 | 抓不到的地方 |
|---|---|---|---|
| 改回原始线性（不经索引） | 不涨；该处「查的次数」少 | 下界断言：`relocation_sources.index_lookups >= units_relocated,`（测试文件 247 行）、`work.index_lookups >= data_units_replaced,`（359 行，对 `SnapshotJudgement` 与 `MappingEntriesOfReleasedUnits` 两处） | 释放判定路径有 a、b、c 三遍，下界只要一遍的量；快照判定那一圈进 deadlist 的那一支（d）的查找在下界里一点不占份额（下界被同一处的 `mapping_key_of` 满足）。测试文件的池里没有快照，d 那一支根本没走到 |
| 每次（每项）重建索引 | `index_builds` 涨 | `compaction_read.index_builds, 1,`（491 行，仅整理读一版）；慢档对每一趟建索引次数的增长比（1227 行） | 核源落点每一项现建一份：每趟的项数是批大小 K 不随池涨，增长比不动（M7 实测 100%）；也没有「核源落点只建一次」的断言 |
| 每把 key 重解叶 | `mapping_leaf_decodes` 涨 | `compaction_read.mapping_leaf_decodes,`（483、487 行，仅整理读一版：解叶次数等于 key 走到的叶片数） | 其他三处（释放、快照判定、核源落点）的解叶没有断言读它 |

实测（改后树，每条变异套 `muts/<名>.old` → `.new`，`tools/run_mutation.sh` 同款流程；输出摘在 `out/mutations.txt`，全文日志在草稿目录）：

| 名 | 改法 | 结局 |
|---|---|---|
| M8 | 核源落点退回线性找角色、逐 key 查（即 `crates/mutations.tsv` 里「整理发批核源落点退回按 key 线性找角色、逐 key 查映射」那一行） | 快档红（16 文件与 48 文件两条用例）：`整理这一趟搬了 11 个单元，核源落点那一处只经索引查了 0 次`——是下界断言抓的，不是增长门槛 |
| M2 | d（快照判定进 deadlist 那一支，6071 行）退回线性找 key、逐 key 查 | 快档 5 条全绿 |
| M3 | a（释放判定路径，4339 行）退回线性找 key、逐 key 查 | 快档 5 条全绿 |
| M4 | c（释放前读盘核校验和，第二处 4898 行）退回线性找 key、逐 key 查 | 快档 5 条全绿 |
| M7 | 核源落点每一项现建一份索引加批量读者（建了两份丢一份） | 快档 5 条全绿；慢档那一条也绿：`growth_in_percent=123 index_builds_growth_in_percent=100` |

M2 到 M7 的 5 条是「第三批的改法改回去，点名的用例不红」。M2 的原因是池里没有快照；M3、M4 的原因是三遍释放的下界只要其中一遍的量；M7 的原因是每趟项数不随池涨。这几条在 T4 里按「有没有一处改回去没有任何一行红」再列。

### T3 推翻条件

- 本格「计数记在产品调用点上」被推翻：找到产品里一处按身份的查找，是在别的线程上做的、或不经 `SortedIdentityIndex` 也不经 `count` 却被点名五处加两处所覆盖（上面列的 9917、6116、mount.rs 两处不在点名处里）；
- 「用例并行不串数」被推翻：把 `--test-threads` 调成 1 或 8，任意一条的读数变了（我没跑，只读码）；
- 「改回去不红」被推翻：给 M2、M3、M4、M7 之一找到一条现有用例会红（我跑的是 `publish_lookup_by_identity` 的快档 5 条与 M7 的慢档 1 条，没有跑别的测试目标，例如 `snapshot_release_and_destroy_through_the_deadlist`；那类用例判的是结果，不判查了几次，M2/M3/M4 的结果与改前相同，所以不会红，但没有现跑）。

## T4　特性与变异

结论：**部分不成立**。(1) 发布构建与 checker 档构建里计数确实编没了，成立；(2) 工作区里一起编 harness 测试不会让判法单元键错误复用，但这一批本身把 9 组里 5 组的键改了（一次现判），与特性无关；(3) 新加与换点名的 12 行变异在读码上各有红的理由（只有 1 行我现跑），但有 5 处第三批的改法改回去而没有任何一行红。

### T4.1　计数在不带特性的构建里是不是编没了；带特性的构建覆盖到哪

- 特性只在核心层声明、默认关：`lookup-work-counters = []` 在 `crates/singlefs-core/Cargo.toml:18`。只有 harness 的 dev 依赖打开：`singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }` 在 `crates/singlefs-harness/Cargo.toml:18`（`[dev-dependencies]` 在第 16 行）。
- 不开时：`entering` 的不开分支是 `let _ = site;`（`crates/singlefs-core/src/transaction.rs:3392`，上一行 3391 是 `#[cfg(not(feature = "lookup-work-counters"))]`），`count` 的是 `let _ = update;`（3424，3423 行同样的 cfg）；守卫类型 `pub(crate) struct InsideALookupSite {` 在 3383 行，只含一个 `()`，`impl Drop for InsideALookupSite {` 在 3399 行带 cfg，不开时守卫没有析构。线程局部变量、`lookup_work_while`、`LookupWorkOfEachSite` 都带 cfg。
- 实测编得过且没有警告（改后树，不开特性，`-p singlefs-core` 单包，工作区解析器 2 下不带 dev 依赖）：
  `cargo check --offline -p singlefs-core` → `Checking singlefs-core v0.1.0 (…/new/crates/singlefs-core)` 然后 `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 16.10s`，输出里没有 `warning`。
- 编没了的程度：计数的 `count(|work| …)` 闭包在不开时只被丢掉；但 `get` 里 `let index_entries = u64::try_from(self.sorted.len()).expect("索引项数装得进 u64");` 在 `crates/singlefs-core/src/transaction.rs:3469`、循环里数 `entries_examined` 的那个局部变量不带 cfg，不开时仍执行（廉价，优化器多半去掉；我没量，不写成零开销的实测）。行为不变：不开时没有任何状态读写。
- 带特性的覆盖：`cargo tree` 的原样输出（改后树）：

```text
$ cargo tree --offline -p singlefs-checker-tier -i singlefs-core -e features
singlefs-core v0.1.0 (…/new/crates/singlefs-core)
└── singlefs-core feature "default"
    ├── singlefs-checker-tier v0.1.0 (…/new/crates/singlefs-checker-tier)
    │   ├── singlefs-checker-tier feature "default" (command-line)
    │   └── singlefs-checker-tier feature "verdict-store"
    │       └── singlefs-checker-tier feature "default" (command-line)
    └── singlefs-harness v0.1.0 (…/new/crates/singlefs-harness)
        └── singlefs-harness feature "default"
            └── singlefs-checker-tier v0.1.0 (…/new/crates/singlefs-checker-tier) (*)

$ cargo tree --offline --workspace -i singlefs-core -e features,dev
singlefs-core v0.1.0 (…/new/crates/singlefs-core)
├── singlefs-core feature "default" (command-line)
│   [dev-dependencies]
│   └── singlefs-harness v0.1.0 (…/new/crates/singlefs-harness)
│       └── singlefs-harness feature "default" (command-line)
└── singlefs-core feature "lookup-work-counters"
    [dev-dependencies]
    └── singlefs-harness v0.1.0 (…/new/crates/singlefs-harness) (*)
```

  只编 checker 档包时核心层不带特性；带 dev 边的整个工作区（`cargo test --workspace`、`cargo build --all-targets`）里核心层整个带特性，包括 checker 档的二进制与测试所链的那一份。那时每次按身份查多一次线程局部变量访问，行为不变（`FirstEntryOfEachIdentity` 的结果与计数无关）。

### T4.2　会不会改变 checker 档崩溃判定库的单元键，让判定白白现判或错误复用

- 键怎么算：构建脚本逐组算判器键，`features: features.clone(),` 在 `crates/singlefs-checker-tier/build.rs:139`，这里的 `features` 是 `singlefs-checker-tier` 自己开的特性（`CARGO_FEATURE_*`）；闭包里核心层的特性由 `resolve_features` 照 cargo 的统一规则从根 crate 的请求顺依赖边传下去，**非根 crate 只走 `Normal` 边**：`&[DependencySection::Normal]` 在 `crates/singlefs-unit-graph/src/closure.rs:351`（根 crate 才用 `root_sections`，349 行）。`singlefs-checker-tier` 自己没有 `[dev-dependencies]`（`grep -n 'dev-dep' crates/singlefs-checker-tier/Cargo.toml` 零命中），harness 的 dev 依赖那条 `lookup-work-counters` 边从来不会进键的模型。
- 所以：特性开不开、哪种编法，不进判法单元键，**不会因特性错误复用或白白现判**。反面也要写：键里模型的「核心层不带特性」与 `cargo test --workspace` 里实际编的「核心层带特性」不一致，这一批的特性不改行为，所以无害；将来有 dev 依赖打开的特性要改行为，键就盖不住它。
- 但这一批的改动本身改了键：用 `unit-graph key`（与构建脚本同样的特性 default、verdict-store，`--toolchain-of-the-build dev`，`--lock-file`；改前树 = 冻结副本反向打回第三批补丁）对 `judging_units_entries.rs` 里每一组入口各算一次，原样输出在 `out/group-keys.txt`。带 gpu 一段：

```text
group=gpu_judge target=lib old=7399b06859a2abfd new=7399b06859a2abfd changed=False
group=judgement_of_the_crash_images target=lib old=44ac2f762d6f7b1a new=bebc0c50c4da2e1a changed=True
group=judgement_of_the_given_states target=lib old=e4dda94055b4d7fd new=c3ef78cd6c8feda8 changed=True
group=cpu_facts_verifier target=lib old=add06eb39352061e new=541b62c58bf00fea changed=True
group=gpu_facts_verifier target=lib old=45115fffc41e5a90 new=45115fffc41e5a90 changed=False
group=recording target=lib old=6397f65be176953b new=7e549a145dd759e2 changed=True
group=gpu_and_cpu_judge_pairing_comparison target=test:crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state old=6b137aa641c2cdb1 new=6b137aa641c2cdb1 changed=False
group=gpu_and_cpu_facts_verifier_pairing_comparison target=test:crash_facts_gpu_verifier_agrees_with_the_cpu_reference old=926448a6a24eb530 new=926448a6a24eb530 changed=False
group=observer_conclusion_composition target=lib old=3138c0c674c963eb new=8eb4e81a890977c0 changed=True
```

  不带 gpu 时能编进来的三组（`judgement_of_the_crash_images`、`judgement_of_the_given_states`、`cpu_facts_verifier`）同样 `changed=True`。9 组里 5 组的键变了：这些组的入口闭包够到 `transaction.rs`、`mounted_session.rs` 里被改的条目（例如 `judge_crash_image` 的闭包从 2475 个单元涨到 2487 个，`pool_after_the_observer` 的从 472 涨到 476，同一次 `unit-graph key` 的 stderr 汇总行）。判法单元键是按闭包原文算的，闭包里的条目改了字就换键，库里按旧键录的判定因此不复用、需要现判一次——这是键机制对任何一次改动的正常反应，不是特性造成的，也不是错误复用。要不要把这一次现判当成本，交主 agent 判；崩溃轻型的结论主 agent 另报。
- 判不了：库里现存的判定是用哪一把键录的、现判要花多久，我没读库也没跑重型。

### T4.3　新加与换点名的变异行：各自让点名的测试红的理由

12 行都在 `crates/mutations.tsv` 里（与补丁 `mutations-append.tsv` 与 `mutations-replacements.tsv` 的 12 行逐行比对，`grep -qxF` 12 条全命中，`out/mutations.txt` 里没有「MISSING」）。红的理由里「实测」只有核源落点那一行；其余是读测试断言推的，没有逐行现跑（变异轻型归主 agent，主 agent 说 257 条抓到）。

| 行 | 名 | 红在哪条断言（推，M8 实测） |
|---|---|---|
| 4967 | 同一身份取最后一项 | 重复身份用例（测试文件 867 行）：`seven first` 的比地址 |
| 4973 | 增长：按身份查的索引退回逐项找 | 点名慢档用例；退回后不经 `get`，所有处 `index_lookups` 为 0 而看过的项数不为 0，`run_compaction_and_count_its_lookups` 里先过 `assert_every_lookup_examines_at_most_a_binary_search`（217 行）就红，不是增长门槛红 |
| 4974 | 增长：整理读版本退回逐 key 查映射 | 慢档用例里 `assert_the_compaction_read_decodes_exactly_the_routed_leaves`（483 行）：逐 key 查不走批量读者，解叶次数不等于叶片数 |
| 5066 | 索引从大到小逐项找 | 合成表用例：不经 `get`，查的次数为 0、看过的项数不为 0，`assert_every_lookup_examines_at_most_a_binary_search`（217 行）红；增长门槛没起作用 |
| 5093 | 单元索引按角色查退回在索引上逐项找（MA） | 同上（217 行，看过的项数超过 查的次数 × 位数） |
| 5094 | 批量读者每把 key 把叶整片重解（MC） | 483 行：解叶次数大于叶片数 |
| 5095 | 映射 key 按角色查退回在索引上逐项找（MD） | 217 行 |
| 5096 | 每把 key 现建一个批量读者、外层读者照查（MF） | 491 行 `compaction_read.index_builds, 1,`：建索引次数大于 1 |
| 5097 | 快照判定那一圈取映射 key 退回线性找（MG） | 359 行：`SnapshotJudgement` 一处查的次数为 0，小于换下的数据单元数 |
| 5098 | 每次查之前先把整份索引走一遍（MI） | 217 行 |
| 5099 | 经映射释放的角色取映射条目退回线性找 key、逐 key 查 | 359 行：三遍释放全退回，`MappingEntriesOfReleasedUnits` 查的次数为 0 |
| 5100 | 整理发批核源落点退回按 key 线性找、逐 key 查 | **实测红**（M8）：`整理这一趟搬了 11 个单元，核源落点那一处只经索引查了 0 次`（247 行）；16 文件与 48 文件两条用例红 |

另有一行没改但这一批把点名换了的：5065「整理读版本：每把 key 现建一个批量读者」点名 16 文件用例，红在 491 行的建索引次数。

### T4.4　有没有一处第三批的改法改回去而没有任何一行红

有，5 处。每一条都套在改后树上，跑 `publish_lookup_by_identity` 的快档 5 条（跳过慢档与我自己追加的用例），结局原样在 `out/mutations.txt`：

| 名 | 改回去的是什么 | 结局 | 为什么没有一行红 |
|---|---|---|---|
| M1 | 排序 `sort_by` 换 `sort_unstable_by`（`crates/singlefs-core/src/transaction.rs:3443`） | 快档 5 条全绿 | 真数据里每个身份只出现一次；重复身份用例只有 5 项，小切片的不稳定排序碰巧稳定。文档承诺「同一身份取最先那一项」在大表重复身份上没人守 |
| M2 | 快照判定进 deadlist 那一支（6071 行）退回线性找 key、逐 key 查 | 快档 5 条全绿 | 测试文件里没有 `CreateSnapshot`，池里没有快照，这一支不被走到；下界被 6056 行的 `mapping_key_of` 满足。`crates/mutations.tsv` 5097、5099 两行改的是别处。kb 七·6 那一行点名了这一支（「含快照判定那一圈进 deadlist 那一支」） |
| M3 | 释放判定路径（4339 行）退回线性找 key、逐 key 查 | 快档 5 条全绿 | 释放有 a、b、c 三遍，下界（359 行）只要一遍的量，其余两遍补上；5099 那行是三遍一起退回才红 |
| M4 | 释放前读盘核校验和（4898 行）退回 | 快档 5 条全绿 | 同 M3 |
| M7 | 核源落点每一项现建一份索引加批量读者（9695–9702 行） | 快档 5 条全绿；慢档也绿，`growth_in_percent=123 index_builds_growth_in_percent=100` | 核源落点每趟项数是批大小，不随池涨，建索引次数的增长比不动；没有「核源落点只建一次」的断言；解叶在这一处也没有断言 |

kb 七·6 那一行的验收写着「每处改法一条会红的变异」（`.claude/kb/milestone/04-fourth-txn.md:142`）。按「处」算，`mapping_locations_of_a_mapped_unit` 是一处还是四个调用点各一处，我不替主 agent 定；事实是 5099 一行盖三遍（同红才红），deadlist 那一支没有任何一行盖。

### T4 推翻条件

- (1) 推翻：不开特性的构建里能读到任何一个计数值，或 `cargo check -p singlefs-core` 出警告；
- (2) 推翻：在 `crates/singlefs-checker-tier/Cargo.toml` 或键的模型里找到 harness 的 dev 依赖边被走到的路径；或 `unit-graph key` 在同一棵树上、只因是否带 harness 编法而出不同的键；
- (3) 推翻：给 M1、M2、M3、M4、M7 任意一条找到点名某一行变异的现有用例会红（我只跑了 `publish_lookup_by_identity` 的快档，M7 另跑了慢档）。


## 附：正文引到的行（按行号从冻结副本取的原文，供逐行核对；同一行重复引的只列一次）

- `crates/singlefs-checker-tier/build.rs:139`：「features: features.clone(),」
- `crates/singlefs-checker-tier/src/crash.rs:1682`：「impl PartialOrd for StateCount1280Bits {」
- `crates/singlefs-checker-tier/src/crash_judge_tables.rs:161`：「impl Ord for CombinationOfTheWritesOnALocation {」
- `crates/singlefs-core/Cargo.toml:18`：「lookup-work-counters = []」
- `crates/singlefs-core/src/address.rs:44`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]」
- `crates/singlefs-core/src/mount.rs:8743`：「|(identity, key)| match central_mapping_locations_in_the_version(version, key) {」
- `crates/singlefs-core/src/mount.rs:9222`：「central_mapping_locations_in_the_version(current, key)」
- `crates/singlefs-core/src/mounted_session.rs:2793`：「let _inside_the_compaction_read =」
- `crates/singlefs-core/src/mounted_session.rs:2794`：「lookup_work::entering(LookupSite::CompactionReadsTheCentralMapping);」
- `crates/singlefs-core/src/transaction.rs:2465`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」
- `crates/singlefs-core/src/transaction.rs:3144`：「pub struct TransactionOutput {」
- `crates/singlefs-core/src/transaction.rs:3262`：「use std::cell::RefCell;」
- `crates/singlefs-core/src/transaction.rs:3353`：「thread_local! {」
- `crates/singlefs-core/src/transaction.rs:3354`：「static LOOKUP_WORK_ON_THIS_THREAD: RefCell<LookupWorkOnThisThread> = const {」
- `crates/singlefs-core/src/transaction.rs:3366`：「pub fn lookup_work_while<Output>(」
- `crates/singlefs-core/src/transaction.rs:3383`：「pub(crate) struct InsideALookupSite {」
- `crates/singlefs-core/src/transaction.rs:3391`：「#[cfg(not(feature = "lookup-work-counters"))]」
- `crates/singlefs-core/src/transaction.rs:3392`：「let _ = site;」
- `crates/singlefs-core/src/transaction.rs:3399`：「impl Drop for InsideALookupSite {」
- `crates/singlefs-core/src/transaction.rs:3414`：「let site = state.sites_entered.last().copied();」
- `crates/singlefs-core/src/transaction.rs:3423`：「#[cfg(not(feature = "lookup-work-counters"))]」
- `crates/singlefs-core/src/transaction.rs:3424`：「let _ = update;」
- `crates/singlefs-core/src/transaction.rs:3441`：「mut pairs: Vec<(Identity, Value)>,」
- `crates/singlefs-core/src/transaction.rs:3443`：「pairs.sort_by(|(left, _), (right, _)| left.cmp(right));」
- `crates/singlefs-core/src/transaction.rs:3445`：「pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);」
- `crates/singlefs-core/src/transaction.rs:3446`：「count(|work| work.index_builds += 1);」
- `crates/singlefs-core/src/transaction.rs:3457`：「if low >= high {」
- `crates/singlefs-core/src/transaction.rs:3460`：「let middle = low + (high - low) / 2;」
- `crates/singlefs-core/src/transaction.rs:3464`：「std::cmp::Ordering::Less => low = middle + 1,」
- `crates/singlefs-core/src/transaction.rs:3465`：「std::cmp::Ordering::Greater => high = middle,」
- `crates/singlefs-core/src/transaction.rs:3466`：「std::cmp::Ordering::Equal => break Some(value),」
- `crates/singlefs-core/src/transaction.rs:3469`：「let index_entries = u64::try_from(self.sorted.len()).expect("索引项数装得进 u64");」
- `crates/singlefs-core/src/transaction.rs:3471`：「work.index_lookups += 1;」
- `crates/singlefs-core/src/transaction.rs:3496`：「count(|work| work.index_entries_examined += 1);」
- `crates/singlefs-core/src/transaction.rs:3534`：「identities_with_their_entries.push((identity_of(entry), entry));」
- `crates/singlefs-core/src/transaction.rs:3551`：「pub fn first_entries(&self) -> impl DoubleEndedIterator<Item = (&Identity, &'table Entry)> {」
- `crates/singlefs-core/src/transaction.rs:3566`：「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」
- `crates/singlefs-core/src/transaction.rs:3580`：「.expect("照抄进来的角色在上一版里进映射、有一把 key")」
- `crates/singlefs-core/src/transaction.rs:3660`：「pub fn take_the_mapping_positions_over_the_location_hints<PositionOfKey>(」
- `crates/singlefs-core/src/transaction.rs:3750`：「.iter_mut()」
- `crates/singlefs-core/src/transaction.rs:3751`：「.find(|unit| unit.identity == *identity)」
- `crates/singlefs-core/src/transaction.rs:4057`：「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」
- `crates/singlefs-core/src/transaction.rs:4080`：「FirstEntryOfEachIdentity::of_the_table(&self.mapped_units, |(_, key)| key.as_slice())」
- `crates/singlefs-core/src/transaction.rs:4281`：「placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(」
- `crates/singlefs-core/src/transaction.rs:4295`：「pub fn placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(」
- `crates/singlefs-core/src/transaction.rs:4296`：「previous: &TransactionOutput,」
- `crates/singlefs-core/src/transaction.rs:4302`：「let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);」
- `crates/singlefs-core/src/transaction.rs:4305`：「let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);」
- `crates/singlefs-core/src/transaction.rs:4308`：「.filter(|identity| !kept_for_the_snapshots.contains(identity))」
- `crates/singlefs-core/src/transaction.rs:4339`：「match previous_read_by_role.locations_of_role(identity) {」
- `crates/singlefs-core/src/transaction.rs:4342`：「return Err(PublishError::ReleaseNotInMapping { unit: identity })」
- `crates/singlefs-core/src/transaction.rs:4572`：「pub struct MappedUnitsOfAVersionReadByRole<'version> {」
- `crates/singlefs-core/src/transaction.rs:4608`：「.get_or_insert_with(|| IndexesOfAVersionReadByRole {」
- `crates/singlefs-core/src/transaction.rs:4612`：「let (_, key) = mapped_units_by_role」
- `crates/singlefs-core/src/transaction.rs:4614`：「.expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");」
- `crates/singlefs-core/src/transaction.rs:4615`：「central_mapping.locations_of_key(key)」
- `crates/singlefs-core/src/transaction.rs:4635`：「lookup_work::count(|work| work.mapping_leaf_decodes += 1);」
- `crates/singlefs-core/src/transaction.rs:4663`：「lookup_work::count(|work| work.mapping_leaf_decodes += 1);」
- `crates/singlefs-core/src/transaction.rs:4742`：「.filter(|identity| !kept_for_the_snapshots.contains(identity))」
- `crates/singlefs-core/src/transaction.rs:4784`：「fn mapping_locations_of_a_released_unit(」
- `crates/singlefs-core/src/transaction.rs:4788`：「match previous_read_by_role.locations_of_role(identity) {」
- `crates/singlefs-core/src/transaction.rs:4809`：「pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(」
- `crates/singlefs-core/src/transaction.rs:4816`：「let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);」
- `crates/singlefs-core/src/transaction.rs:4818`：「let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);」
- `crates/singlefs-core/src/transaction.rs:4822`：「let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);」
- `crates/singlefs-core/src/transaction.rs:4876`：「pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(」
- `crates/singlefs-core/src/transaction.rs:4891`：「let _inside_the_releases = lookup_work::entering(LookupSite::MappingEntriesOfReleasedUnits);」
- `crates/singlefs-core/src/transaction.rs:4894`：「let mut previous_read_by_role = MappedUnitsOfAVersionReadByRole::of_the_version(previous);」
- `crates/singlefs-core/src/transaction.rs:4898`：「let locations = mapping_locations_of_a_released_unit(&mut previous_read_by_role, identity);」
- `crates/singlefs-core/src/transaction.rs:6024`：「fn deadlist_changes_of_this_publish(」
- `crates/singlefs-core/src/transaction.rs:6036`：「let _inside_the_snapshot_judgement = lookup_work::entering(LookupSite::SnapshotJudgement);」
- `crates/singlefs-core/src/transaction.rs:6050`：「MappedUnitsOfAVersionReadByRole::of_the_version(previous_version);」
- `crates/singlefs-core/src/transaction.rs:6053`：「let mapped_units_of_the_previous_version = previous_version.mapped_units_by_role();」
- `crates/singlefs-core/src/transaction.rs:6056`：「.mapping_key_of(*identity)」
- `crates/singlefs-core/src/transaction.rs:6071`：「let locations = match previous_version_read_by_role.locations_of_role(*identity) {」
- `crates/singlefs-core/src/transaction.rs:6074`：「return Err(PublishError::ReleaseNotInMapping { unit: *identity })」
- `crates/singlefs-core/src/transaction.rs:6116`：「.find(|(key, _)| *key == entry.mapping_key)」
- `crates/singlefs-core/src/transaction.rs:8052`：「let deadlist = deadlist_changes_of_this_publish(」
- `crates/singlefs-core/src/transaction.rs:9447`：「pub fn publish_relocation_batch<Device: BlockDevice>(」
- `crates/singlefs-core/src/transaction.rs:9520`：「pub fn publish_compaction_step<Device: BlockDevice>(」
- `crates/singlefs-core/src/transaction.rs:9623`：「pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(」
- `crates/singlefs-core/src/transaction.rs:9690`：「let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);」
- `crates/singlefs-core/src/transaction.rs:9693`：「let mut previous_read_by_key = None;」
- `crates/singlefs-core/src/transaction.rs:9695`：「relocations.push(relocation_checked_against_the_version(」
- `crates/singlefs-core/src/transaction.rs:9698`：「previous_read_by_key.get_or_insert_with(|| {」
- `crates/singlefs-core/src/transaction.rs:9699`：「MappedUnitsOfAVersionReadByKey::of_the_version(previous)」
- `crates/singlefs-core/src/transaction.rs:9896`：「.mapped_units_by_key」
- `crates/singlefs-core/src/transaction.rs:9897`：「.first_entry_of(&mapping_key.as_slice())」
- `crates/singlefs-core/src/transaction.rs:9902`：「.locations_of_key(&mapping_key)」
- `crates/singlefs-core/src/transaction.rs:9917`：「.find(|(key, _)| *key == mapping_key)」
- `crates/singlefs-core/src/transaction.rs:11118`：「copies_failing_the_release_checksum_check(」
- `crates/singlefs-core/src/transaction.rs:11128`：「refuse_mapping_entries_that_do_not_name_two_pool_devices(」
- `crates/singlefs-core/src/transaction.rs:11344`：「placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(」
- `crates/singlefs-core/src/transaction.rs:13019`：「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」
- `crates/singlefs-harness/Cargo.toml:18`：「singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:217`：「let examined_at_most = work.index_lookups * binary_digits_of(work.largest_index_looked_up);」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:235`：「let (run, lookup_work) = lookup_work_while(|| session.run_compaction(devices));」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:247`：「relocation_sources.index_lookups >= units_relocated,」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:263`：「lookup_work_while(|| session.publish_user_change(devices, change));」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:359`：「work.index_lookups >= data_units_replaced,」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:478`：「lookup_work_while(|| central_mapping_as_the_compaction_reads_it(version));」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:483`：「compaction_read.mapping_leaf_decodes,」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:487`：「compaction_read.mapping_leaf_decodes,」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:491`：「compaction_read.index_builds, 1,」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:521`：「for (identity, entry) in index.first_entries() {」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:867`：「fn first_entry_of_each_identity_keeps_the_first_of_repeated_identities_and_misses_absent_ones() {」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1084`：「const HIGHEST_GROWTH_IN_PERCENT_OF_THE_LOOKUP_WORK_FOR_FOUR_TIMES_THE_TABLE: u64 = 200;」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1162`：「fn lookup_work_of_the_compaction_calls_on_a_pool_of(files: u64) -> Vec<LookupWork> {」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1200`：「let (smaller_readings, larger_readings) = std::thread::scope(|scope| {」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1201`：「let smaller = scope.spawn(|| {」
- `crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1227`：「larger.index_builds * smaller_calls * 100 / (smaller.index_builds * larger_calls);」
- `crates/singlefs-unit-graph/src/closure.rs:349`：「root_sections」
- `crates/singlefs-unit-graph/src/closure.rs:351`：「&[DependencySection::Normal]」

## 要主 agent 跑的命令

- 整条 `rerun.sh`（开头「复跑与模型文件」那一条命令）：工作目录随意，环境变量不用设，内存上限 16G，线程 31（脚本里用 `capped.sh 31`）；本机忙时 240 秒前台跑不完，所以我没整条跑完——旧树那一半在脚本里跑通了（出了 `old.lines`），新树那一半是同样的命令手工跑的。看哪几行判结果：开头那一节列的五样。
- 增长慢档未变异的读数我没拿到（本机负载约 80–90，两次 238 秒前台都被截断），主 agent 说的「debug 下连慢档 `6 passed` 97.94s」是主 agent 的数，我没有复核；我手上只有 M7（只改建索引次数）那一趟的读数 `growth_in_percent=123 index_builds_growth_in_percent=100`。
- 想补 M2–M4 对别的测试目标红不红：`snapshot_release_and_destroy_through_the_deadlist` 等整个 harness 档测试目标，在每条变异下各跑一遍。我没跑。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 稳定排序 + `dedup_by` 留最先一项 | 成立 | 三种身份类型的 `Ord` 与 `==` 都是派生、一致；折半查的边界用例 2033 次比对与线性找相同；panic 文案逐字相同（`crates/singlefs-core/src/transaction.rs:4614` 与补丁删去的那句） |
| T1 折半查 vs 线性找（空表、单项、全同、查不到） | 成立 | 边界用例 `FWD_EDGE compared=2033`；一条没有守住的等价变异：`sort_unstable_by`（M1）快档全绿 |
| T2 两处查法换了之后取到的是不是同一项 | 成立 | 角色→key 与 key→角色都取最先一项；读者持共享借用、`TransactionOutput` 无内部可变性；调用点逐处在 T2.1、T2.2 |
| T2 盘镜像逐字节相同 | 成立（取样） | 三串操作（含快照下重写、整理、删快照）改前改后 9 行摘要相同；错误路径没现跑 |
| T3 计数点是不是产品调用点上的事 | 成立 | 6 处进、5 处记；核心层无线程；每处在真历史里都被走到 |
| T3 用例并行时串数 | 不串（读码） | 线程局部加 `lookup_work_while` 挪开与恢复；没跑 `--test-threads=1` |
| T3 200% 门槛 | 站得住 | 读数 123%、余量 77 个百分点；只看经索引的查找，原始线性找看不见 |
| T3 改坏之后计数一定涨、测试一定红 | 不成立 | M2、M3、M4、M7 快档全绿（M7 慢档也绿）；核源落点退回线性（M8）靠下界断言红；另有 9917、6116、mount.rs 两处线性找不在计数里 |
| T4 计数在不带特性的构建里编没了 | 成立 | 单包 `cargo check` 无警告；`cargo tree` 证实只有带 dev 边的整个工作区才带特性；不开时 `get` 里仍算看过几项，没量开销 |
| T4 判法单元键 | 不会错误复用；这一批改了 9 组里 5 组的键 | 键模型不走 dev 边，特性不进键；键变是因为闭包够到被改的条目 |
| T4 12 行变异各自红的理由 | 成立（读码，仅核源落点那行实测） | 理由逐行在 T4.3 |
| T4 改回去无一行红 | 不成立 | 5 处：M1、M2（deadlist 那一支，池里没有快照）、M3、M4、M7 |

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决（kb 七·6 的「每处改法一条会红的变异」按处怎么数，我只列事实）。
- 没跑任何门禁阶段、checker 档测试、GPU；没读禁读清单里的文件（第三轮攻方的提示与产出）。
- 没有现跑错误路径（`ReleaseNotInMapping`、条目窄于字段表、叶角色缺失的 panic）在四个调用点上改前改后是否一致，只读码。
- 12 行变异里我只现跑了核源落点那一行（M8）；其余行的红法是读测试断言推的。主 agent 的变异轻型已跑过它们。
- 没跑 `--test-threads=1`；没跑变异在 `publish_lookup_by_identity` 之外的测试目标上；没量未变异的慢档增长读数；没量计数不开时的开销。
- 没读判法单元库里现存的判定与它们的键，也没算现判的成本。
- `rerun.sh` 整条没有跑完（负载），见上。
- 交回前删我建的仓副本与编译目录（交回消息里列路径）。
