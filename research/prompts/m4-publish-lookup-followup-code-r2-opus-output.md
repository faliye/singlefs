# m4-publish-lookup-followup-code-r2：攻方腿（Opus）报告（2026-10-22）

立场：假设第 11 批的修补与它的判法是错的，在冻结副本的拷贝上造反例。代码行号一律是冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo/` 里的。所有量的数都在副本上量（第 11 批之后的冻结副本拷成 `after`，`transaction.rs` 倒打第 11 批那 23 块拷成 `before`），不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m4-publish-lookup-followup-code-r2-opus-model/`。复跑（整条经内存包装，本机跑；约二十几分钟，推的）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-followup-code-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-publish-lookup-followup-code-r2/repo --work <新的草稿目录>
```

打出的 `name=` 行与 `out/rerun-expected.txt` 逐行比（复跑不打挂钟）。报告里的数是在第二台经 `scripts/hosts/multi-host-run.sh --peer`（本机内存额度被占，主 agent 中途让改走第二台）跑出来的；rerun.sh 在本机直接跑同一份 `tools/peer_job.sh`，没在本机整条跑过一遍。

| 文件 | sha256 |
|---|---|
| `mutations/attack-mutations.tsv` | `196d9ffc36d6ad6572dee95c3fa3c3fc7392e809fc7f2b6dedd49654b51823ae` |
| `out/rerun-expected.txt` | `7066710ccb4f185f064774ae2eb77e17f2e1dfd134727ed92b2f90ad0ccc0945` |
| `tests/attack_r2_differential.rs` | `7f3c9e3ee2bfe884c189ce7ae9553c9315586580740d7e1a4d5361a6e09a9116` |
| `tests/attack_r2_probe.rs` | `f1022a8e168b37ee9abb405d8b218e8d9e1ebc55c18912bc94574e84aeea340b` |
| `tests/attack_r2_remount.rs` | `c09697cc35baea263b14396e22c131dc57a7fee989e22d4fb222fd48e63090df` |
| `tools/apply_mutation.py` | `299304d8c3c59a71c28dbd41917a2830ffc18e0a85a778bc32153b08d3b7825d` |
| `tools/peer_job.sh` | `b74523f460d47c3b230c494abb542d555ff8e847eaad3f85a6af726629eb6fac` |
| `tools/rerun.sh` | `7159e95c7bd7c8990d0d630ad139f6bd608c03fd66493473c7413e64d4bfd228` |
| `tools/t4_rustflags.sh` | `7284424f5b7aab7fe77569536dee90fbfbd812724989b48a6a57ca972ab43185` |

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 修补本身（取不取同一项、写不写同一份字节） | **没打中** | 128 段随机合法历史、每段 48 步（6144 步，连同每段的起止行共 6400 行），改前改后每一步的写流、盘镜像与内存里 `units` 的槽逐行相同；差分自己证过会红 |
| T2 产生结论的方法 | **打中四处** | 四条改坏查法的变异（只照抄一棵树、每项重建建在作用域外、恢复那一侧每项重建、退回线性而索引照查）`publish_lookup_by_identity` 全部 12 条不忽略的用例照绿；其中恢复那一侧每 key 进一次、建一次，`index_builds == times_entered` 照样成立 |
| T3 从未被看过的地方 | **打中一处** | `publish_admitted` 装第一个文件的数据单元时 `!rewritten.contains(&identity)`（`crates/singlefs-core/src/transaction.rs:14342`），每次发布比较次数是第一个文件的数据单元数乘这次重写的角色数，三个规模点量过 |
| T4 构建脚本与判法单元键 | **打中一种编法** | `RUSTFLAGS='--cfg feature="lookup-work-counters"'` 加 `--target`：库按开着编，构建脚本按没开编、交出空串；另报清单 `[features]` 加特性而表没加时构建不报、文档点名的守卫用例不存在 |

## T1　修补本身：没打中

### 读码（正推一侧不归我，这里只列造反例时依赖的前提）

- `SortedIdentityIndex::keeping_the_first_value_of_each_identity` 用 `sort_by`（稳定）再 `dedup_by` 丢后一项（`crates/singlefs-core/src/transaction.rs:3513`、`crates/singlefs-core/src/transaction.rs:3515`），留下的是每个身份最先那一项；`TransactionUnit` 的 `PartialEq`、`Ord` 都是 derive（`crates/singlefs-core/src/transaction.rs:2483`），按 `Ord` 折半与按 `==` 线性找认的是同一个相等。要让索引取到与 `iter().find` 不同的项，得有一对 `Ord` 判相等而 `==` 判不等的身份，derive 之下造不出。
- `take_the_mapping_positions_over_the_location_hints` 先问完位置再改槽，`position_of_key` 拿不到 `version`（`version` 以 `&mut` 借进来），两处调用方的闭包一个读 `destination_of_key`、一个读 `mapping_entries.mapping_position_of_key`（`crates/singlefs-core/src/recovery.rs:3425`），都不读 `units`；新加的早退 `if new_slot_of_each_mapped_role.is_empty() {`（`crates/singlefs-core/src/transaction.rs:3826`）之后函数里没有别的语句。
- `publish_admitted` 里 `units_by_role` 借着 `units`（`crates/singlefs-core/src/transaction.rs:14730`），`named_unit_of` 闭包到最后一次调用之前 `units` 改不了，借用检查守住，索引过期造不出。

### 差分（量的）

`tests/attack_r2_differential.rs` 在两份副本各编一份，按同一串种子跑随机合法操作。每一步之后打一行：操作与结局、这一步录下的写流（`RecordingBlockDevice` 共用一条流，写、FUA、清零、discard、屏障的次序与内容哈希逐条渲染再取 sha256）、两块盘镜像的 sha256、这一版的摘要（留存条目数与重复 key 数、`units` 项数与重复角色数、`units` 里逐项的槽连成串的 sha256）。写流逐条相同，两边的崩溃状态集合就相同，所以没有另起崩溃枚举。比第一轮攻方差分多的操作：干净卸载再挂、不卸载直接重挂（两种都从盘上重建一版，走 `crates/singlefs-core/src/recovery.rs:3425` 那一侧的调用）、截断第一个文件、写第一个文件、一次写 10–40 个单元（记录点名切几条，走 `roles_other_than_the_data_roles_named_before_the_last_record`）。

取样：种子 1–128，每段 48 步，批大小在 1、2、3、8 里按种子取。第二台满载（loadavg 41），前台命令一次至多 240 秒，所以每次至多 48 段；第一轮攻方跑的是 512 段 × 64 步，这一轮缩到 128 段 × 48 步（估时：32 段约 53–99 秒、48 段约 100 秒，按此分成 5 次）。

```
cmp <(cat dfb2.lines df4before.lines) all-after.lines && echo all_identical
all_identical
```

`all-after.lines` 6400 行（种子 1–32 来自第二台 `r2opus-df5-after`，33–128 来自 `r2opus-df6-after-a`、`-b`；改前来自 `r2opus-df-b2`、`r2opus-df4-before-a`、`-b`）。覆盖（改后那一份数出来的）：

```
compaction ok=995 create_snapshot ok=404 destroy_snapshot ok=209 first_file err=136 first_file ok=180 new_inodes ok=610 overwrite ok=290 remount unmount=ok=627 remount unmount=skipped=615 truncate ok=226 unlink ok=168 write ok=1684 
panics=0 kept_relocated_steps=249 remount_clean_ok=627 remount_dirty_ok=615 max_dupunits=0 big_writes=351 seeds_with_end=128
```

`first_file err` 的 136 次都是对第一个文件发多于一个单元的 `Overwrite`（只认一个单元以内，`err:Publish` 两边相同）。`max_dupunits=0`：取样里 `units` 从没有过重复角色，「同一角色几项取哪一项」那一支差分没走到，只靠上面的读码。

### 差分会红（证过）

| 弄坏 | 改了什么 | 结果（种子 1–32、48 步，与改后基线比） |
|---|---|---|
| SENS1 | `roles_other_than_the_data_roles_named_before_the_last_record` 的 `.is_none()` 换成 `.is_some()` | `sens1 lines=1600 differing=1568 panics=0 first_diff_seeds=32` |
| SENS2 | `version.units[*position].slot = slot;`（`crates/singlefs-core/src/transaction.rs:3843`）换成 `let _ = (position, slot);` | `sens2 lines=1600 differing=728 panics=0 first_diff_seeds=32` |

另有一次意外的对照：第二台的共用编译目录按 mtime 判新鲜，SENS2 那一趟之后的一趟基线没重编、跑的是 SENS2 的二进制（`r2opus-df4-after`）。它与改前那 96 段比，写流与镜像两列逐行相同（`diff … | grep -c '^>'` 打 `0`），只有 `slots=` 那一列不同：摘要里的槽串看得见这一步只改内存不改字节的那一类偏差。发现之后 `tools/peer_job.sh` 每趟先 touch 两份可能被改的源文件，基线重跑（上面 `all_identical` 用的就是重跑的），变异那几趟每趟都改写源文件、不受影响，`r2opus-rm-base` 那一趟受影响，重跑成 `r2opus-rm-base2`。

### 什么会推翻「没打中」

一段合法历史在改前改后打出不同的一行（写流、镜像或槽串）；或者造出 `units` 里同一角色两项、两项的槽不同、而改前改后取的不是同一项。崩溃之后从崩溃状态恢复那一支只经「写流相同」推过来，恢复代码本身在崩溃状态上没有逐个跑（见末节限度）。

## T2　产生结论的方法：打中四处

四条变异都在 `after` 副本上打（原文与替换文逐字在 `mutations/attack-mutations.tsv`，`tools/apply_mutation.py` 判原文恰好命中一次），每条编 `singlefs-harness` 的 `publish_lookup_by_identity`、直接执行测试二进制跑全部不忽略的用例（`--test-threads 31 --nocapture`）。这 12 条里含第 11 批 20 行变异点名的五条 harness 用例（整份重写、快照下重写与删快照、整理发三批、deadlist 几片叶、构建脚本那条）；`singlefs-unit-graph` 的两条单测没跑（四条变异都不碰那个包）。原样（第二台，`name=job` 行）：

```
name=job job=base2 mutant=baseline target=publish_lookup_by_identity build_rc=0 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]
name=job job=na1 mutant=NA1 target=publish_lookup_by_identity build_rc=0 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]
name=job job=na2 mutant=NA2 target=publish_lookup_by_identity build_rc=0 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]
name=job job=na3 mutant=NA3 target=publish_lookup_by_identity build_rc=0 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]
name=job job=na4 mutant=NA4 target=publish_lookup_by_identity build_rc=0 rc=0 test result: ok. 12 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]
```

### NA1　只照抄一棵树：记账树退回线性，判法空真

`build_multi_level_tree` 里 `let carried = units_of_the_previous_version`（`crates/singlefs-core/src/transaction.rs:15102`）那一句换成：记账树走 `previous_version.unit(…)`（在上一版 `units` 上 `iter().find`），另两棵照旧走索引。按树分开数的判法在照抄 0 个节点时只判这一处 (0, 0, 0)（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1455` 的 `if carried == 0 {`）。基线输出里记账树在每个取样上都照抄 0 个：

```
deadlist_pool_rewrite data_units=8 carried_accounting_nodes=0 carried_deadlist_nodes=2 carried_mapping_nodes=10
deadlist_pool_rewrite data_units=32 carried_accounting_nodes=0 carried_deadlist_nodes=3 carried_mapping_nodes=10
Some(CarriedNodesOfTheAccountingTree): LookupWork { index_builds: 0, index_lookups: 0
```

（末行是整份重写一个八单元文件那条用例打的 `whole_rewrite lookup_work`。）四句：① 分辨臂：只对「记账树那一处」这一格起作用，三棵树一起退回的 MA4 照红。② 系统当时看不看得到：看得到，问题在取样，没有一个池让记账树长到两个节点。③ 满足的分句：正文 T2「按树分开的三处在只照抄一棵树……时，整数遍判法会不会空真」的「会」。④ 后果是推的：记账树条目数由 `accounting_entry_count_is_three_plus_six_per_device(`（`crates/singlefs-core/src/recovery.rs:4129`）判成 3 加每块盘 6 条，两块盘 15 条，推的是装得进一片叶、永远不照抄，NA1 在今天两盘的池上是等价变异，没有代价；盘多到记账树分裂时才有代价，那时这一格没有用例判。

### NA2　每项重建且建在作用域外：点名那一处之外现建，没有用例判 `None`

`named_unit_of` 闭包开头加一句 `let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);`，遮住 `crates/singlefs-core/src/transaction.rs:14730` 那一份：每点名一个角色现建一份这一版 `units` 的索引，建在 `lookup_work::entering(LookupSite::UnitsOfTheRolesNamedByTheRecords);`（`crates/singlefs-core/src/transaction.rs:14744`）之前，记进 `None`。整份重写八单元那一次发布：

```
base2: None: LookupWork { index_builds: 5, index_lookups: 1286 | Some(UnitsOfTheRolesNamedByTheRecords): LookupWork { index_builds: 0, index_lookups: 24
na2: None: LookupWork { index_builds: 29, index_lookups: 1286 | Some(UnitsOfTheRolesNamedByTheRecords): LookupWork { index_builds: 0, index_lookups: 24
```

点名那一处判的 (建 0、查 = 装的次数乘点名角色数) 原样成立，多出的 24 份建在 `None`。全文件里判每一处的只有 `assert_every_lookup_examines_at_most_a_binary_search`（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:235`），它只判每次查看过几项、不判建几次。四句：① 分辨臂：分得出（基线 5、变异 29），只是没人读这一格。② 看得到。③ 满足正文 T2「有没有一处查找落在任何一个 `LookupSite` 的作用域之外（记进 `None`）」，第 11 批只给照抄 extent 树那一处包了作用域（第一轮 MA5），别处没有。④ 改法（推的，没实现）：发布路径各用例另判 `None` 的建索引次数在两档单元数下不涨。

### NA3　恢复那一侧每 key 重建：`index_builds == times_entered` 照样成立

`crates/singlefs-core/src/recovery.rs:3425` 那一次调用换成「每把有位置的 key 各调一次 `take_the_mapping_positions_over_the_location_hints`，闭包只对那一把交位置」：结果与一次调用相同（每个指针、每个单元槽只被它自己的 key 改一次，推的；上面差分没跑这一条），代价是每把 key 走一遍全部指针、建一份 `units` 索引。`tests/attack_r2_remount.rs` 量不卸载直接重挂那一次在这一处做的（`mount_micros` 是第二台满载下 debug 档的挂钟，噪声大，只作参考）：

```
基线 name=remount files=16 units=159 mapped=156 mount=ok times_entered=1 index_builds=1 index_lookups=156 mount_micros=194769
基线 name=remount files=64 units=599 mapped=592 mount=ok times_entered=1 index_builds=1 index_lookups=592 mount_micros=559280
基线 name=remount files=256 units=2354 mapped=2337 mount=ok times_entered=1 index_builds=1 index_lookups=2337 mount_micros=1976262
NA3 name=remount files=16 units=159 mapped=156 mount=ok times_entered=156 index_builds=156 index_lookups=156 mount_micros=248166
NA3 name=remount files=64 units=599 mapped=592 mount=ok times_entered=592 index_builds=592 index_lookups=592 mount_micros=916688
NA3 name=remount files=256 units=2354 mapped=2337 mount=ok times_entered=2337 index_builds=2337 index_lookups=2337 mount_micros=4140883
```

两件事：一、恢复那一侧没有用例读这一处（`UnitsOfTheMappedRolesTakingTheirMappingPositions` 只在整理发三批那条用例里判），第 11 批注释说「整理发批与恢复重建一版都调它」，受益的那一半没有会红的检查。二、正文 T2 第一问的答案是「会分不开」：每项进一次、建一次时 `work.times_entered >= 1 && work.index_builds == work.times_entered,`（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1478`）照样成立（2337 = 2337），分开两类的是「每遍查的次数」，`assert_one_index_built_per_entry` 本身不判它；第 11 批用它的各处靠旁边另一条查的次数断言（`index_lookups >= passes * data_units_rewritten` 之类）才判得出，哪一处只有这一条，哪一处就分不开。四句：① 分辨臂：只这一侧。② 看得到（计数里有）。③ 满足正文 T2 第一问字面。④ 改法（推的）：恢复那一侧加一条重挂用例，判 (建 1、进 1、查 = 有位置的进映射角色数)，这一条红 NA3。

### NA4　退回线性而索引照查：计数看不见

`roles_other_than_the_data_roles_named_before_the_last_record` 的过滤条件在 `.first_entry_of(identity).is_none()` 后面再接 `&& !data_roles_named_before_the_last_record.contains(identity)`：行为不变，第一轮报的 N² 那一处原样回来，而这一处的建、查、进三个数一个不变（计数只数索引上的动作）。四句：① 分辨臂：任何「只数索引动作」的判法都分不开，第 11 批 20 行与第一轮的变异全是「把索引换成线性找」，索引的查数掉到 0 才红；「加回线性、索引照查」这一类没有一条变异代表。② 被判的系统看不到：计数器从构造上看不见不经索引的找法，属于 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 那张表的「判别子观测不到」。③ 满足正文 T2「换一种同样坏的写法还红不红」：不红。④ 这一条是对抗写法，正常改代码的人不大会这样写；能判它的是不读计数的路子（比较次数探针、挂钟），改法是推的、被攻过零轮。

## T3　从未被看过的地方：打中一处

### 第一个文件照抄单元时的 `.contains`（`crates/singlefs-core/src/transaction.rs:14342`）

`publish_admitted` 装这一版 `units` 时，第一个文件的每个数据单元先判 `Some(_) if !rewritten.contains(&identity) => carried_unit(`：`rewritten` 是这次重写的角色清单（`Vec`），每个单元在上面线性找一遍。第 11 批与七·6 前几批都没改这一行（七·6 改的是它右边的 `carried_unit` 走索引）。每次发布（不重写第一个文件的那几种：写别的文件、整理发批、删除步）的比较次数是「第一个文件的数据单元数 F 乘这次重写的角色数 R」。

量法：探针变异 PROBE 把这一处换成同义的 `iter().position` 并把比较次数记进一个全局计数（只在探针副本里），`tests/attack_r2_probe.rs` 把第一个文件长到 F 个单元、另写四个文件，量整份重写其中一个 N 单元文件那一次发布。原样（第二台，`r2opus-probe1`）：

```
name=probe first_file_units=64 first_file_data_pointers=64 rewrite_units=32 units_in_the_version=211 rewritten_roles=44 outcome=ok first_file_contains_comparisons=2816 publish_micros=59794
name=probe first_file_units=256 first_file_data_pointers=256 rewrite_units=32 units_in_the_version=409 rewritten_roles=49 outcome=ok first_file_contains_comparisons=12544 publish_micros=78233
name=probe first_file_units=1024 first_file_data_pointers=1024 rewrite_units=8 units_in_the_version=1134 rewritten_roles=37 outcome=ok first_file_contains_comparisons=37888 publish_micros=308953
name=probe first_file_units=1024 first_file_data_pointers=1024 rewrite_units=32 units_in_the_version=1231 rewritten_roles=61 outcome=ok first_file_contains_comparisons=211968 publish_micros=697014
name=probe first_file_units=1024 first_file_data_pointers=1024 rewrite_units=128 units_in_the_version=1619 rewritten_roles=162 outcome=ok first_file_contains_comparisons=165888 publish_micros=2343715
```

算术：64 × 44 = 2816，256 × 49 = 12544，1024 × 37 = 37888，1024 × 162 = 165888，与量的逐个相等；1024 × 61 = 62464 而量到 211968 = 1024 × 207，推的是那一次发布算到定点走了不止一遍 `publish_admitted`（每遍的 R 不同），没核。N 固定 32 时随 F 是 2816 → 12544 → 211968（F 每乘 4 涨 4.5 倍、16.9 倍），F 固定 1024 时随 N 是 37888 → 211968 → 165888。第一个文件占池的大头（F 接近 P）、又整份重写一个大文件（R 接近 N）时是 P × N，两样都到 P 量级就是 P 的平方；`publish_micros` 是第二台满载下 debug 档的挂钟，不拿它下结论。

第一轮攻方的探针里有一格 `first_file_rewritten_contains`（第一个文件取默认小内容，F 不变），读数 8–145、判「O(P) 或更小，没打中」；那一轮没让 F 变，这一轮补的是 F 这一维。四句：① 七·6 的题面是随池大小，这一处随 F × R，F 只在第一个文件大时才是池的大头，算不算七·6 的漏由主 agent 判。② 看得到。③ 满足正文 T3「随池大小或随一次写的单元数平方涨的」里的后一半在 F ≈ N 时成立，前一半只在第一个文件占池大头时成立。④ 改法（推的、零轮）：`rewritten` 先收进一个 `BTreeSet`，或与 `units_by_role` 同样建一次。

### 别处查过、不算打中的

- `crates/singlefs-core/src/recovery.rs:3425` 那一侧的新索引（正文 T3 第二问）：每次重挂建 1 份、查的次数等于有位置的进映射角色数（上面 NA3 那一节的基线行，16/64/256 个文件各 1 份、156/592/2337 次），内存是一份「角色、下标」对的数组，推的是每项几十字节，没量内存。
- `FirstEntryOfEachIdentity` 在 1–2 个单元的小发布上的代价（正文 T3 第三问）：没量挂钟（两台都满载，分不出微秒级差别）；只按代码推：每次多建一两份几百项的排序数组。
- 不开 `lookup-work-counters` 时 `InsideALookupSite` 也实现 `Drop`（第四问）：`Drop::drop` 的身体整段在 `#[cfg(feature = "lookup-work-counters")]` 下，不开特性时是空函数，`entering` 里的 `count(…)` 不开特性时只剩 `let _ = update;`；推的是内联之后不剩分支，没看汇编。
- 扫过 `transaction.rs`、`mounted_session.rs`、`recovery.rs`、`compaction_plan.rs`、`region_scan.rs` 里 73 处 `.iter().find(`、`.contains(`、`.position(`、`.unit(`、`file_other_than_the_first(`（清单在草稿目录 `linear-candidates.txt`）：多数在 `BTreeSet`、范围或每次发布至多几次的位置上；`settle_the_allocation_record_tree` 的 `previous_roles_replaced.contains(&role)` 第一轮正推已报（K × R，没起读数），这一轮也没量。

## T4　构建脚本与判法单元键：打中一种编法

### `RUSTFLAGS` 带 `--cfg feature=…` 再加 `--target`

cargo 带 `--target` 时 `RUSTFLAGS` 只给目标上的编译、不给宿主上编的构建脚本。`tools/t4_rustflags.sh` 两格各用新编译目录编 `singlefs-core`（`cargo build -v`），`RUSTFLAGS='--cfg feature="lookup-work-counters"'`、不开特性。第一趟（`r2opus-t4`）原样，读 rustc 那一行有没有那个 cfg：

```
name=t4 cell=rustflags_with_target build_rc=0 library_rustc_cfg_lookup_work_counters=on build_script_rustc_cfg_lookup_work_counters=off build_script_output=[]
name=t4 cell=rustflags_without_target build_rc=0 library_rustc_cfg_lookup_work_counters=on build_script_rustc_cfg_lookup_work_counters=on build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
```

第一趟带 `--target` 那一格没读到构建脚本输出（我的脚本只找 `debug/build/`，带 `--target` 时在 `<三元组>/debug/build/` 下），改了找法再跑一趟（`r2opus-t4b`，没重编，所以两格的 cfg 列都读成 off，只看元数据那一列）：

```
name=t4 cell=rustflags_with_target build_rc=0 library_rustc_cfg_lookup_work_counters=off build_script_rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=]
name=t4 cell=rustflags_without_target build_rc=0 library_rustc_cfg_lookup_work_counters=off build_script_rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
```

两趟合起来：带 `--target` 时库按 cfg 开着编（第一趟），构建脚本按没开编、交出 `enabled_features=`（空串，第二趟）；不带 `--target` 时两边都开着、交出 `LOOKUP_WORK_COUNTERS`，对得上。第二趟两格的 cfg 列都是 off，是因为两格都没重编（脚本按「rustc 那一行里有没有」判，没有那一行就记 off），不是 cfg 关着。

四句：① 分辨臂：只分这一种编法，cargo 真开特性、不开特性两种照旧对。② 被判的系统看不到：`singlefs-core/build.rs` 的 `cfg!(feature = "lookup-work-counters")` 判的是编构建脚本那一次的 cfg，带 `--target` 时与编库那一次不是同一份 cfg，构建脚本在它能看到的东西里分不出（第一轮「外带变量」那一格换了个入口：这一次进来的是 rustc 的 cfg，不是环境变量，所以不算重复第一轮）。③ 满足正文 T4「造一种编法让判法单元键与真编的特性不符」：`singlefs-core` 交出的特性是空、编出来的库带着计数。④ 后果有一半是推的：checker 档的构建脚本另读 `CARGO_ENCODED_RUSTFLAGS` 进键，这种编法的键与不带 `RUSTFLAGS` 的那一份本来就不同，不会被当成同一份复用；不符的是键里「`singlefs-core` 开了哪些特性」那一项，按它算的闭包把只在 `#[cfg(feature = "lookup-work-counters")]` 下的代码当成没编进来，那几行改了键不变（推的，没编 checker 档核）。仓里今天有没有脚本这样编，没全仓搜。改法（推的、零轮）：构建脚本另比 `CARGO_ENCODED_RUSTFLAGS` 里的 `--cfg feature=` 与自己的 cfg，对不上就停下。

### 清单 `[features]` 加了特性而表没加：构建不报，文档点名的守卫不存在

- `FeatureTableDisagreesWithTheManifest` 只在 `features_of_the_crate_being_built` 里报，checker 档构建脚本只拿它判 checker 档自己那张三项表；`singlefs-core` 那张表（`crates/singlefs-core/build.rs:18`，一项）在构建时谁都不比。依赖方读 `DEP_SINGLEFS_CORE_ENABLED_FEATURES` 时（`crates/singlefs-checker-tier/build.rs:96` 那一句 `features_enabled_on_the_dependencies(&workspace, own_crate, &read_environment)`），表里漏写的特性永远不出现在元数据里，等于没开、不报错。
- `crates/singlefs-core/build.rs:14` 的文档说这件事由 `singlefs-unit-graph` 的单测 `the_core_build_script_lists_every_feature_of_its_manifest` 判；`grep -rnF the_core_build_script_lists_every_feature_of_its_manifest` 在冻结副本 `crates/` 下只命中这一行本身（数出来 1 处），没有这条单测。真正判它的只有 harness 档那条构建脚本用例里按行读清单的 `feature_names_of_the_core_manifest`（`.skip_while(|line| line.trim() != "[features]")`，`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:2340`），也就是只在跑那一个测试目标时才红，`cargo build` / checker 档构建都不红。正文 T4 第二问的答案：只在一条编法（编并跑 harness 档 `publish_lookup_by_identity`）上才报。这一条要改源码才出现，不是编法，归「推的」：没在副本里加特性编一遍核。

### 没打中的

- `cell=stale`（构建脚本不重跑、元数据留空）：第 11 批之后元数据只看 cfg，环境变量变了 cfg 不变，留着的那一份就是对的；不算问题。
- checker 档三项（`default`、`gpu`、`verdict-store`）：表与清单不符时 checker 档构建脚本当场停（`unwrap_or_else(|error| panic!(…))`），对照读码没找到让表的 cfg 与 rustc 的 cfg 分开的编法，除了上面 `--target` 加 `RUSTFLAGS` 那一种（同样会让 checker 档自己的三项不符，推的，没编）。
- harness 档 dev 依赖开特性而普通依赖没开时两份构建脚本各报什么：没起 cargo 量（第一轮正推量过「单编与 dev 依赖一起编各交对的那一份」）。

## 没打中的形状

| 格 | 试过什么 | 取样 |
|---|---|---|
| T1 | 改前改后随机历史差分：新建、写（1–40 个单元）、单单元覆盖写、写与覆盖第一个文件、截断第一个文件、建与删快照、删除、整理、干净卸载再挂、不卸载直接重挂；每步比写流、镜像、槽串 | 128 段 × 48 步、批大小 1/2/3/8；两种弄坏都抓到 |
| T1 | 读码：稳定排序留最先一项、`Ord`/`PartialEq` 都 derive、早退之后无语句、索引借着 `units` | `transaction.rs` 第 11 批改到的各处 |
| T2 | 按树分开的另两棵（deadlist、中央映射）只照抄一棵：没造，第一轮 MA4 这一批已钉住；「每遍重建」与「每项重建」在 `written_units`、留存条目、快照判定那一圈：读码判旁边有查的次数断言，没起变异 | 读码 |
| T3 | 73 处线性找的清单逐处看了种类与外层循环，只量了第一个文件那一处 | `linear-candidates.txt` |
| T4 | 上面「没打中的」三条 | 读码，加第一轮的真编结果 |

## 这条腿自己的限度

- 崩溃状态没有逐个枚举、也没在崩溃状态上分别用改前改后的恢复代码重挂：T1 的「两边崩溃状态相同」是从写流逐条相同推的；恢复代码（`take_the_mapping_positions_over_the_location_hints` 的恢复那一侧）只在干净与不干净两种重挂上比过，没在中途崩溃的状态上比过。
- 差分 128 段 × 48 步比第一轮的 512 段 × 64 步少（两台都满载、前台命令 240 秒）；`units` 里重复角色那一支一次都没走到。
- 所有数在第二台的 debug 档上量；挂钟噪声大，结论只用计数。`rerun.sh` 没在本机整条跑过一遍。
- NA1 的「两盘下记账树只有一片叶」、NA3 的「结果与一次调用相同」、T4 的闭包后果都是推的。
- 我提的改法（判 `None`、恢复那一侧的重挂用例、`rewritten` 建集合、构建脚本比 `RUSTFLAGS`）都只在我的模型上推过、被攻过零轮，没有一条实现。

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| 发布路径用例判 `None` 的建索引次数不随单元数涨 | NA2 | 推的 |
| 恢复那一侧加重挂用例，判 (建 1、进 1、查 = 有位置的进映射角色数) | NA3 | 推的（NA3 之下 `times_entered=2337 index_builds=2337` 是量过的，判法本身没写） |
| 加一个记账树长到两个节点以上的池（多盘） | NA1 | 推的 |
| 比较次数探针或挂钟判法 | NA4 | 推的 |
| `rewritten` 收成集合 | T3 | 推的 |
| 构建脚本比 `CARGO_ENCODED_RUSTFLAGS` 的 `--cfg feature=` | T4 | 推的 |

## 没做什么

- 没跑门禁阶段、checker 档测试、重型测试，没起 GPU；`singlefs-unit-graph` 的单测没跑。
- 正推那几格不判，不替主 agent 采纳；副本上的数不算入库装置上的数。
- 第一轮攻过的角度（七处改法取的是不是同一项、8 处单次调用的阶数、MA1–MA5、外带 `CARGO_FEATURE_*`）没重复；T4 的 `RUSTFLAGS` 一格换的是入口（rustc cfg），理由见那一节。

## 末表：每格结论与给主 agent 的复跑

| 格 | 结论 | 复跑看哪几行 |
|---|---|---|
| T1 | 没打中（128 × 48 逐行相同，差分证过会红） | `rerun.sh` 的 `name=differential seeds=32 steps=48 lines=1600 before_vs_after=identical`（复跑只跑种子 1–32） |
| T2 | 打中四处：NA1 空真（今天等价，推的）、NA2 建在 `None` 没人判、NA3 恢复那一侧每项重建且 `index_builds == times_entered` 照样成立、NA4 退回线性计数看不见 | `name=job job=NA1`…`NA4` 四行 `12 passed`；`name=none_site` 两行（5 与 29）；`name=remount` 六行 |
| T3 | 打中一处：`crates/singlefs-core/src/transaction.rs:14342` 每次发布比较 F × R 次 | `name=probe` 五行 |
| T4 | 打中一种编法：`--target` 加 `RUSTFLAGS` cfg，库开着、元数据空；表与清单不符只有 harness 档用例判 | `name=t4` 两行 |
