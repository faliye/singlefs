# m3-close-w13-code-r1 攻方腿（Opus）报告（2026-10-22）

立场：假设实现与条款对得上是错的，找让代码与条款不一致的输入、崩溃点或调用方。被判的 7 份文件读主工作区现行版本，开工时 `sha256sum -c research/prompts/m3-close-w13-code-r1-snapshot/code-sha256.txt` 7 行全 OK；量都在草稿目录的仓副本里做（harness 档），副本上的数注明是副本，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-close-w13-code-r1-opus-model/`。复跑命令（整条经内存包装，`--frozen` 给一棵只含 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo` 的树，7 份文件要对得上开工快照，脚本里先 `sha256sum -c` 一次）：

```
bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-w13-code-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结树> --work <新的草稿目录>
```

跑完看 `name=` 开头的行与两行 `test result: ok`。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`；`--selftest` 经内存包装跑过：「✓ 自检通过（查了 5 格）」。

各文件 sha256（`find … -type f | sort | xargs sha256sum` 原样输出）：

```
44704278c63da6ddf4a3db0a15e06393210d08277c09e64ca433ba486e22cb97  research/prompts/m3-close-w13-code-r1-opus-model/out/run1-2026-10-22.log
224e5658d45aeb68d852540a26fcf0e7c7183d5322fc7ec3a064523876023460  research/prompts/m3-close-w13-code-r1-opus-model/out/run2-2026-10-22.log
db149929404fafa6406395986f0f1eb8ecdb207e3d28cade9a67b3b80a5770fa  research/prompts/m3-close-w13-code-r1-opus-model/tests/attack_w13_mount_after_a_remade_pool_named_by_the_old_identifier.rs
221a04e03daa0c44cf783319dea0c529d1b7f81da06eb07d1a929bbb5348e8db  research/prompts/m3-close-w13-code-r1-opus-model/tests/attack_w13_wider_entries_checker_versus_readers.rs
15822eae672708b464d18d360cb88d15abe31427235e276bf07b4d6f53ddfd7f  research/prompts/m3-close-w13-code-r1-opus-model/tools/rerun.sh
```

两份 `out/*.log` 是副本上那两次跑的全量日志，草稿目录的绝对路径换成了 `<草稿目录>`，别的一字未动。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 代码照条文；打中一处 kb 内部矛盾（不在 7 份代码里） | 写序、分段、屏障逐条对得上；条文说段序列「以 layout 八「mkfs 种根」那一行为准」，那一行还是 `14+1+1+1+4`、25 次操作，钉住它的用例已改成 `2+12+1+1+1+4`，segment-registry 格会红（推的，没跑门禁） |
| K2 | 代码照条文；打中一处条文没管到的缝（量过，副本） | 随机源读不出、读不满都拒、不退回别的来源；内层入口的调用方全是测试、装置、实验二进制。缝：对外入口只在成功时交回 fsid，可写挂载按调用方给的 fsid 认池；中途崩溃后调用方手里只有旧池 fsid，重做出来的空池可写挂载报 `CallerParametersDisagreeWithTheSelectedSystemConfiguration`，不是条文点名的那个拒因 |
| K3 | 没打中（推的；没起崩溃枚举） | 作废持久之后新系统配置只在最后一段、根环已清、读者按新几何读；旧池换几何（S、槽距、journal 环长更大）重做也落不到条文不许的结局 |
| K4 | 打中（量过，副本） | 六处比较两边是同一个量（`entry.len()` 就是头里的条目宽）。同形没改的：池级 checker 的 deadlist 树叶与内部条目只守「不窄于」，实现这一侧（本轮的 `crates/singlefs-core/src/records.rs:640` 与 C780 的 `crates/singlefs-core/src/code_two_tree.rs:1019`）拒宽；core 里另有 3 处 `<`，2 处已登记、1 处按设计不该改 |
| K5 | 打中（量过，副本） | 同一张镜像（deadlist 叶条目宽 43 → 44）：池级 checker 新增非「成立」0 条、冷启动恢复读出文件、可写挂载拒。实现拒掉了一张 checker 判合法的盘；另五处 checker 都报 I-1.10 违例，与实现一致。改名在每个调用方的处置与改名前相同 |

打中的三格都只在我的副本模型上量过；下面提的改法都是「只在我的模型上量过、被攻过零轮」或「推的」，逐格标。

## K1 两趟写序与条文

### 逐句对（没打中的部分）

条文：`.claude/kb/decisions/22-单元原子性怎么合成.md:179`：「每块盘的 [0, 根环基址) 清零之后两块盘各过一道屏障（作废），之后才写别的；其余清零与单元写之间不加屏障。」

| 条文要的 | 代码 | 对不对 |
|---|---|---|
| 写任何元数据之前先清每块盘 [0, 根环基址) | `crates/singlefs-core/src/make_filesystem.rs:625`：「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」，之前只有 `check_geometry`（不写） | 对 |
| 清完两块盘各一道屏障，之后才写别的 | `crates/singlefs-core/src/make_filesystem.rs:629`：「device_whose_system_configuration_area_was_zeroed.barrier()?;」，在下一个写之前的独立循环里 | 对 |
| 根环三个区域、journal 环各一次调用，区域不按归属挑盘 | `crates/singlefs-core/src/make_filesystem.rs:633`：「device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;」与 `crates/singlefs-core/src/make_filesystem.rs:635`：「device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;」，每块盘都清 | 对；每块盘 1 + 3 + 1 = 5 次写零、两块盘 10 次 |
| 其余清零与单元写之间不加屏障 | 633–647 行之间没有 `barrier`；单元写之后才是 `crates/singlefs-core/src/make_filesystem.rs:650`：「device.barrier()?;」 | 对 |
| 根种进三个区域、FUA | `crates/singlefs-core/src/make_filesystem.rs:701`：「device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;」 | 对 |
| 系统配置每盘两槽、之后一道屏障 | `crates/singlefs-core/src/make_filesystem.rs:732`：「device.write_at(offset, &slot, WriteDurability::Plain)?;」，再 `crates/singlefs-core/src/make_filesystem.rs:736`：「device.barrier()?;」 | 对 |

我找过「一条写落在条文没说的段里」：没有。写只有上面 6 处调用点；种类序列 `[zero_fill×2,barrier×2]|[zero_fill×8,unit_write×4,barrier×2]|[root_record_fua]×3|[system_configuration_slot×4,barrier×2]` 由 harness 用例 `recorded_stream_matches_the_registered_mkfs_segment_sequence` 钉住（读码，没跑）。多分段、多清区：没有——清的仍是 [0, 根环基址)、三个区域、journal 环，与条文列的三样相同。

### 打中：条文指定的权威登记行与代码对不上（kb 内部，不在被判的 7 份代码里）

- 条文同一行写段序列「以 layout 八「mkfs 种根」那一行为准」（`.claude/kb/decisions/22-单元原子性怎么合成.md:179`）。
- 那一行今天写的是旧写序：`.claude/kb/layout/01-first-txn.md:410`：「14+1+1+1+4，25 次操作（21 次写，两道池屏障按设备记、每道两块盘各一步）、16402 个崩溃状态」。`git status --short` 对这份文件无输出（工作区没改它）。
- 钉住活代码的用例已改：`crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:339`：「"2+12+1+1+1+4",」，同一函数里自己写着 `crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs:336`：「layout/01-first-txn.md 八 mkfs 那一行要跟着改（写回另派）。」
- 门禁 doc-registries 的 segment-registry 格按出处里的用例函数体逐字找登记串：`scripts/crash/check-segment-registry.py:394`：「if entry.segment_sequence_text and f'"{entry.segment_sequence_text}"' not in body:」。函数体里没有 `"14+1+1+1+4"` ⇒ 这一格红（推的：按脚本读的，没跑门禁）。

四句：
1. 分不分辨臂：分辨。实现与条文一致，坏的是条文指过去的登记行；改登记行就消，改代码不消。
2. 系统看不看得到：不涉及运行时判别，是两份 kb 文字与一条用例之间的事实对不上，一条 grep 看得到。
3. 满足的是哪个分句：K1「条文里的其余清零、根与系统配置槽的次序与分段，代码是不是逐条照做」——代码照做了条文正文，但条文把「段序列与步数」的权威交给了 layout 那一行，按那一行代码不照做；属于题面「内部矛盾」那一面。
4. 改法在这格上中不中：改法只有一个——layout 八那一行改成 `2+12+1+1+1+4`、27 次操作、新的崩溃状态数与种类串（推的；用例注释给的闭式 4117 只是「每次写只取两态」的数，带撕裂的数要另量，不能照抄）。body 第二节「这一轮之外」的四样里没有这一样，主 agent 要另记。

推翻条件：layout 那一行在本批里另有写回（工作区里有一处我没看到的改动），或 segment-registry 格不从这条用例取比对对象。

## K2 fsid：对外入口、随机源、内层入口的调用方

### 逐句对（没打中的部分）

条文：`.claude/kb/decisions/22-单元原子性怎么合成.md:179`：「fsid 由 mkfs 的对外入口生成，前 8 字节取自随机源；内层入口可给 fsid，给测试、装置与崩溃点重放用。」

- 取：`crates/singlefs-core/src/make_filesystem.rs:236`：「.and_then(|mut random_source| random_source.read_exact(&mut filesystem_identifier))」，16 字节整份来自随机源，单元头取前 8 字节（`unit::unit_filesystem_identifier` 取 `[..8]` 小端）。前 8 字节的来源与条文一致。
- 读不出、读不够：打不开与 `read_exact` 读不满（`UnexpectedEof`）都走 `crates/singlefs-core/src/make_filesystem.rs:237`：「.map_err(MakeFilesystemError::RandomSourceUnreadable)?;」，没有退回时钟、主机名、常量或内层默认值的分支；取在任何写之前（`crates/singlefs-core/src/make_filesystem.rs:259`：「let drawn_filesystem_identifier = filesystem_identifier_from_the_random_source()?;」先于 `crates/singlefs-core/src/make_filesystem.rs:265`：「let output = make_filesystem(&parameters, devices)?;」）。缺口：`RandomSourceUnreadable` 这条路没有用例（`grep -rn RandomSourceUnreadable crates/ --include=*.rs` 只命中 `make_filesystem.rs` 自己），路径写死成常量 `/dev/urandom`，用例换不了源；判的是读码，不是跑。
- 内层入口的调用方（命令与命中数原样）：
  - `grep -rn 'make_filesystem_drawing_the_filesystem_identifier(' crates/ --include=*.rs | grep -v 'pub fn' | wc -l` ⇒ `1`（harness 用例 `the_outer_entry_draws_filesystem_identifiers_whose_first_eight_bytes_differ_between_two_runs`）。
  - `grep -rln '[^_]make_filesystem(' crates/*/src --include=*.rs` ⇒ 15 份：`singlefs-core` 的 `make_filesystem.rs`（对外入口自己）与 `mount.rs`（命中在 `#[cfg(test)]` 模块里）、`singlefs-harness` 的 `scenario.rs`（E142 新池新建文件装置，盘由它的调用方给）与 `history.rs`（随机历史装置，盘是新建的稀疏盘）、其余 11 份在 `singlefs-checker-tier`（崩溃枚举、放量、判表与实验二进制）。
  - 工作区里 `fn main` 只在 `singlefs-checker-tier/src/bin/` 与 `singlefs-unit-graph`；没有一个产品入口（命令行 mkfs）调内层入口。条文许的三类用途之外没有调用方。
  - 会不会给出与旧池前 8 字节相同的 fsid：会，而且是有意的——C484、C651 的回归用例与 harness 的 `parameters()` 拿同一个 E142 fsid 在用过的盘上重做（C659 认下的那一形）。条文许内层入口给测试用，不算越位。

### 打中：对外入口的 fsid 只在成功时交回，可写挂载按调用方给的 fsid 认池（量过，副本）

可达的序列：用过的盘（旧池 fsid A，调用方手里有它）→ 调对外入口重做 mkfs，取到 B → 写完作废与之后各段、进程在交回之前没了（崩溃、或 `?` 交回错误之前调用方就丢了返回值）→ 下一次可写挂载。

每一步许它的那一句：
- 对外入口只在内层入口成功之后交回：`crates/singlefs-core/src/make_filesystem.rs:265`：「let output = make_filesystem(&parameters, devices)?;」之后才 `Ok(FilesystemMadeWithADrawnIdentifier { parameters, output })`；B 不在盘外任何地方落下。
- 可写挂载先按调用方给的 fsid 筛槽：`crates/singlefs-core/src/mount.rs:8326`：「&parameters.filesystem_identifier,」；筛不出就不筛再择：`crates/singlefs-core/src/mount.rs:8302`：「return choose_system_configuration(devices);」，择到 B，再由参数核报不一致。

用户动作放开扫（只固定前缀「旧池 A 上重做成 B、27 步都已落盘」，调用方拿哪个 fsid 去挂由用户定）——副本上的原样输出（`out/run2-2026-10-22.log`）：

```
name=remade cold=恢复无文件:(InstanceGeneration(0), CheckpointTxg(0))
name=remade writable caller-holds=old-pool-identifier outcome=拒:CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [FilesystemIdentifier], disagreeing_device_table: [] }
name=remade writable caller-holds=remade-pool-identifier outcome=挂上:实例InstanceGeneration(1)
name=remade writable caller-holds=third-identifier outcome=拒:CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [FilesystemIdentifier], disagreeing_device_table: [] }
```

装置的限度：这一张盘面是「27 步全落盘」，与「作废持久之后、最后一道屏障之前、每一写都已持久」那个崩溃状态逐字节相同（内层入口直接跑完，没有经崩溃枚举摆出别的状态）；B 用的是内层入口给的固定值（前 8 字节 `attack-b`），代表对外入口取到的那一种。

四句：
1. 分不分辨臂：分辨「对外入口不落 fsid」与「对外入口先把 B 交给调用方（或写进一个调用方能读的地方）再写盘」两种做法；冷启动恢复那一侧不分（两种做法都读回第 0 代空池）。
2. 系统看不看得到：可写挂载做决定时看得到盘上系统配置里的 B 与调用方给的 A，两样不同；它看不到「调用方根本没机会知道 B」——这是 API 形状，不是盘面。
3. 满足的是哪个分句：K2「对外入口取随机源……内层入口留着」照做了；打中的是条文射程那一句 `.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「下一次挂载只有两种结局——整池拒（`NoValidSystemConfiguration`），或恢复读回 mkfs 的第 0 代空池」——按「挂载」读，第三种拒因出现了；按「恢复」读（判决 S3 删了挂载那两个分句，checker 档用例按 `RecoveryOutcome` 判），没打中。是不是「打中归错了判据」由主 agent 按条文本意判。
4. 改法在这格上中不中（都是推的，没实现）：改法甲，对外入口先取 fsid、先交给调用方（拆成「取 fsid」「照这个 fsid 做 mkfs」两步，调用方落盘之后再写）——这一格不再中；改法乙，条文把那一句的「挂载」改写成「冷启动恢复」、可写挂载的拒因另列——这一格不再中，代价是对外入口中途崩溃后的空池要靠只读路径先读出 fsid 才挂得上；什么都不改——仍中。

推翻条件：仓里另有一个读盘上系统配置交回 fsid 给调用方的产品入口（我只查了 `mount_read_only` 不收 fsid，没查它交不交回 fsid），或条文本意的「挂载」就是冷启动恢复。

## K3 mkfs 中途崩溃

没打中。下面全是推的（按代码推，没起崩溃枚举；本腿没有调 `enumerate_crash_states`，原因在「这条腿自己的限度」）。checker 档用例 `crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 已在同一份旧池、同一组几何上判过这一段（文件头写的分法 31 195 = 8 + 31 122 + 65，我没复跑），我找的是它没取样到的形状。

| 崩溃点 | 盘上剩什么（按写序推） | 下一次冷启动恢复 | 条文 |
|---|---|---|---|
| 作废持久之前（第一段的两次写零有一块没落） | 旧池的根环、journal 环、单元区全在；[0, 根环基址) 至多一块盘被清、或被撕成部分清 | 读回旧池最后确认的一版；被清的那块盘缺系统配置，可写挂载走补写闸（那块盘的单元、根环、journal 环都还是旧池的，闸四按 fsid 筛单元，`crates/singlefs-core/src/system_configuration_repair.rs:53`：「let filesystem_identifier_in_unit_headers = unit_filesystem_identifier(filesystem_identifier);」） | 许 |
| 作废持久之后、新系统配置一槽都没落 | 两块盘 [0, 根环基址) 全 0 | 没有一槽自证得过 ⇒ `NoValidSystemConfiguration`；逐档找槽 1 的读者读的 2 033 档都在 [0, 1 MiB) 里，全 0 | 许 |
| 新系统配置至少一槽落了 | 它在最后一段，前面的屏障与三道 FUA 保证第二段（清根环、清 journal 环、两个单元）与三条第 0 代根都已落 | 读者按新系统配置的几何读根环与 journal 环，只看得到第 0 代根 ⇒ 第 0 代空池 | 许 |

我额外试过、没打中的形状（推的）：
- 旧池用更大的 S（每区域根槽数）或更大的槽距、更长的 journal 环，新 mkfs 用小的：新 mkfs 只清新几何那么长（`crates/singlefs-core/src/make_filesystem.rs:633`），旧池在新区域末尾之后的根、新 journal 环末尾之后的旧 journal 记录都还在盘上。但三个区域的起点固定、新读者只读新 S 个槽、按新槽距算偏移，全落在被清的那一段里；旧 journal 记录落在新单元区开头，魔数不是单元魔数，单元区扫描不认它。落不到「作废之后读回旧池」「新系统配置与旧池的根并存」。
- 旧池用更小的槽距：旧槽 1 在 [0, 1 MiB) 里，`check_geometry` 判过固定结构两两不重叠，槽 1 碰不到 1 MiB 的根环基址，被第一段清掉。
- 写零失败而不是崩溃：作废那道屏障之后任何一步 `?` 交回错误，盘面与对应的崩溃前缀相同，归到上表第二、三行。
- 屏障只到一块盘：条文明写归 C6，不算。

推翻条件：有一个读者在没有本池系统配置时仍去读根环或 journal 环（例如按编译期默认几何扫），或者 FUA 写在崩溃模型里不切段（那样系统配置可以先于某条第 0 代根落盘）。第二样我只按派发给的「屏障与 FUA 切段」读，没去 `crash_state_enumeration.rs` 里核。

## K4 六处读者与同形读者

### 六处：比较两边是不是同一个量（没打中）

| 读者 | 比较 | 左边从哪来 |
|---|---|---|
| extent 树内部条目 | `crates/singlefs-core/src/extent_tree.rs:908`：「if header.entry_width != internal_width {」 | 头里的条目宽（字节）对 110 |
| 分配记录树内部条目 | `crates/singlefs-core/src/allocation_record_tree.rs:757`：「if header.entry_width != internal_width {」 | 同上，对 96 |
| 分配记录 | `crates/singlefs-core/src/allocator.rs:81`：「if bytes.len() != usize::try_from(ALLOCATION_RECORD_BYTES).expect("20") {」 | 条目切片长 |
| deadlist 条目 | `crates/singlefs-core/src/records.rs:640`：「if bytes.len() != DEADLIST_ENTRY_KEY_BYTES {」 | 条目切片长；`DEADLIST_ENTRY_KEY_BYTES` 是 `8 + 8 + 27` = 43，名字带 key 但就是整条（value 空），不是只比 key |
| inode 树内部条目 | `crates/singlefs-core/src/records.rs:806`：「if bytes.len() != usize::try_from(INODE_INTERNAL_ENTRY).expect("120") {」 | 条目切片长 |
| extent 叶记录 | `crates/singlefs-core/src/records.rs:832`：「if bytes.len() != usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {」 | 条目切片长 |

「条目切片长」与「头里的条目宽」是同一个量：`parse_index_node` 按 `entry_width.max(1)` 切 `entry_count` 条（`crates/singlefs-core/src/unit.rs:445`：「.chunks(entry_width.max(1))」），宽 0 而条数非 0 在那之前已拒，所以每条切片长都等于头里的条目宽。四个按切片判的读者的调用方我逐个查过（`grep -rnE 'DeadlistEntry::parse|AllocationRecord::parse|parse_extent_record|parse_inode_internal_entry' crates/ --include=*.rs | grep -v '/tests/'`，产品代码里的调用点：`extent_tree.rs` 1 处、`recovery.rs` 3 处、`allocation_record_tree.rs` 1 处、`mounted_read.rs` 1 处），喂进去的都是 `header.entries` 里的一条，没有一处拿一段更长的缓冲区（带尾随字节）去调。`recovery.rs`、`mounted_read.rs` 两份文件的改动只是匹配臂改名，比较本身在 `records.rs`。

### 同形没改的读者（读码查全仓，命令与命中数原样）

core 里：

```
$ grep -rnE 'entry_width\s*(<|<=)\s' crates/singlefs-core/src --include=*.rs | grep -v '^\S*:\s*//'
crates/singlefs-core/src/unit.rs:437:    if entry_width < key_width {
$ grep -rnE '^\s*(<|<=)\s*usize::try_from\(' crates/singlefs-core/src --include=*.rs
crates/singlefs-core/src/allocation_record_tree.rs:888:                < usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
$ grep -rnE '\.len\(\)\s*<\s*usize::try_from\([A-Z_]+\)\.expect' crates/singlefs-core/src --include=*.rs
crates/singlefs-core/src/root_record.rs:129:        if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("457")
crates/singlefs-core/src/extent_tree.rs:515:        if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113") {
crates/singlefs-core/src/records.rs:853:    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {
```

（`unit.rs:437` 是通用的「不窄于 key 宽」，`root_record.rs:129` 判的是根槽里装不装得下根记录，都不是条目对字段表；另 `.len() != …BYTES|ENTRY` 形的有 17 处，是已经判等于的。）

| 位置 | 是不是该改 | 依据 |
|---|---|---|
| `crates/singlefs-core/src/extent_tree.rs:515`（extent 上段叶 113） | 该改，已登记 | body 第二节「这一轮之外」的「extent 上段叶第七处」；方向与下面 deadlist 相反：实现宽松、池级 checker 按 I-1.10 判等于（推的，读 `walk.rs` 的 `position_and_entry_width_hold` 调用方，没量） |
| `crates/singlefs-core/src/records.rs:853`（映射条目 55） | 不在 I-1.10 射程 | 注释写明欠在 C307，checker 也只守不窄（`field_table_width_of_tree_kind` 不认种类 5），两边一致 |
| `crates/singlefs-core/src/allocation_record_tree.rs:888`（影子账走分配记录树） | 不该改 | 这是 `node_pointers_as_far_as_readable`，`crates/singlefs-core/src/allocation_record_tree.rs:859`：「从根指针往下，读得出多少就认多少」；它少认一个节点，被抛弃根的单元就不隔离。改成「不等于就拒」是往危险的方向收（推的）。body 没点名它，主 agent 写判决时值得写一句「有意不改」 |

池级 checker（`crates/singlefs-checker`，同形读者的另一半）：
- `grep -nE 'entry_width < ' crates/singlefs-checker/src/walk.rs | wc -l` ⇒ `27`，这批多是「先按 I-1.10 判等于、再守不越界」或二次读里只守不越界，我只核了与本轮六棵树对应的主走读：分配记录树、extent 树上下段都判等于（`position_and_entry_width_hold`）；inode 树判等于；**deadlist 树叶与内部条目只守不窄**——`crates/singlefs-checker/src/walk.rs:322`：「的射程里没有 deadlist 树，这里只守」，配置在 `crates/singlefs-checker/src/walk.rs:2695`：「leaf_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {」，判法是 `crates/singlefs-checker/src/walk.rs:2564`：「if view.entry_width < bytes_the_walk_needs {」。
- 不变量正文同样不罩 deadlist：`crates/singlefs-checker/invariants/01-块自描述.md:16` 列的是「extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108」。

这就是 K4 的打中：实现这一侧 deadlist 叶（本轮 `records.rs:640`）与 deadlist 内部条目（C780 的 `crates/singlefs-core/src/code_two_tree.rs:1019`：「if header.entry_width != internal_width {」）拒宽，checker 与 I-1.10 正文不罩。量出来的后果在 K5。

## K5 改名与拒宽条目

### 改名：每个调用方的处置与改名前相同（没打中）

`PointerCarryingEntryRefusal::WidthOtherThanItsFieldTable` 的匹配臂只有三份文件四处（diff 里逐处可见）：`extent_tree.rs` 叶记录臂与 `recovery.rs` 两处 inode 内部条目臂仍报 `RecoveryFailure::EntryNarrowerThanItsFieldTable`，`mounted_read.rs` 仍报 `OpenPoolForReadFailure::RecordMalformed`，与改名前同一个成员、同一个 `what`。拿 `entry_bytes`、`field_table_bytes` 做减法的地方我找过（`grep -rn 'EntryNarrowerThanItsFieldTable' crates/ --include=*.rs | grep -v '/tests/'` 列出的每处），只有构造、按成员分类（`crash_judge_conclusions.rs`、`crash_judge_gpu.rs` 只看成员不看字段）与格式化，没有 `field_table_bytes - entry_bytes` 这种宽了就下溢的算式。`RecoveryFailure` 的成员名与文档（`crates/singlefs-core/src/recovery.rs:531`：「索引节点头自述的条目宽窄于这棵树的条目字段表」）还写「窄于」，body 已列在这一轮之外。

### 打中：实现拒掉了一张池级 checker 判合法的盘（量过，副本）

输入：新池新建文件 → 建快照 → 快照下覆盖写（deadlist 根兼叶装着换下的单元）→ 把 deadlist 根的每条条目末尾补一个 0、条目宽 43 → 44、重封并把整单元校验和沿引用链补到根槽（造法与本轮新用例 `a_deadlist_leaf_whose_entries_are_wider_than_the_field_table_is_refused_by_the_writable_mount_before_any_write` 相同，我在它之外多跑了池级 checker 与冷启动恢复）。另五处同法各造一张，做对照。

副本上的原样输出（`out/run1-2026-10-22.log`，节选，全文 28 行）：

```
name=case-allocation-internal-96 checker_new_not_holding=1 lines=["I-1.10:违例:分配记录树（树 65538）的根：头里自述的条目宽 97 不等于这一层的条目字段表宽度 96"]
name=case-deadlist-43 checker_new_not_holding=0 lines=[]
name=case-deadlist-43 implementation_before cold=恢复读出文件:(InstanceGeneration(1), CheckpointTxg(5)) writable=可写挂载挂上:实例InstanceGeneration(2)
name=case-deadlist-43 implementation_after cold=恢复读出文件:(InstanceGeneration(1), CheckpointTxg(5)) writable=可写挂载拒:Recovery(EntryNarrowerThanItsFieldTable { what: "deadlist 条目", entry_bytes: 44, field_table_bytes: 43 })
name=summary I-1.10-red extent-internal=true extent-leaf=true inode-internal=true allocation-record=true allocation-internal=true deadlist=false
name=summary deadlist-checker-new-not-holding=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 52.98s
```

同一张盘，三个读者三种说法：池级 checker 改前改后的非「成立」判定逐条相同（新增 0 条）；冷启动恢复照常读出文件（它不读 deadlist）；可写挂载在任何写之前拒（之后有没有出路，例如回退到快照之前的一版，我没查）。另五处 checker 都新报 1 条 I-1.10 违例（inode 那一张另有 9 条「不适用」，是读不下去之后的连带），与实现一致。

可达性：今天的写者写 deadlist 叶用 `leaf_entry_width_in_bytes`（deadlist 取 `DEADLIST_ENTRY_KEY_BYTES`），崩溃只丢整写或撕裂、撕裂过不了校验和，所以崩溃点重放摆不出这张盘；要靠一段校验和自洽的外来字节（坏介质恰好、别的写者、以后改宽的版本）。所以这不是「今天会拒掉用户的盘」，是「判合法与否的两个 oracle 在 deadlist 上不一致」：崩溃判法或放量里一旦有一种状态让 deadlist 条目宽变了，checker 判绿、可写挂载拒，判不出谁错。

四句：
1. 分不分辨臂：分辨。实现「拒宽」与 checker「只守不窄」两条臂在这一格上结局不同；另五处两边一致，不出局。
2. 系统看不看得到：看得到。条目宽在节点头里，三个读者读的是同一个字段。
3. 满足的是哪个分句：K5「拒宽条目会不会把一个合法的盘拒掉」——「合法」按池级 checker 与 I-1.10 正文认（两者都不罩 deadlist）；同时满足 K4「有没有第七处、第八处同形的读者没改」——同形的读者在 checker 里。不是条款本身的错：C867 欠账行把 deadlist 列进六处、用户定「宽于字段表也拒」，实现照做了；对不上的是 I-1.10 正文与 checker 没跟上（C780 改 deadlist 内部条目时就已如此，推的，没量内部条目那一形）。
4. 改法（都没被攻过）：

| 改法 | 修哪一格 | 在 deadlist 这一格上 |
|---|---|---|
| 甲：I-1.10 正文加 deadlist 叶 43、内部 key 宽 + 86，checker 把 deadlist 树两处 `GuardedForTheWalkOnly` 改成 `JudgedAsInvariantOneTen` | checker 与实现一致 | 推的：checker 会新报 I-1.10 违例；改不变量正文要走 kb 那条路，不归这一轮 |
| 乙：实现 deadlist 退回只拒窄 | 实现与 checker 一致 | 推的：与用户 2026-10-18 定的「宽于字段表也拒」相反，不建议 |
| 丙：什么都不改，登记成已知分歧 | 无 | 照样中 |

推翻条件：checker 在别处（例如 `judge_snapshots_and_deadlists_of_the_versions`）对 deadlist 条目宽另有判定而我这张镜像没走到——我的输出里改前改后逐条相同，说明至少在这张镜像上没有。

### 没打中：拒宽会不会拒掉今天写者写出的合法盘

我查了六种条目的写者：分配记录树叶 20、内部 96（`allocation_record_tree.rs` 的 `build` 那一处）、码 2 多层树叶按 `leaf_entry_width_in_bytes`（记账 34、映射 55、deadlist 43）、内部 key 宽 + 86、extent 叶 112、内部 110、上段叶 113（`extent_tree.rs` 四处 `build_index_node` 的条目宽参数），都写等于字段表；零条目的节点只有 extent 上段根兼叶，那一层不经这六处判。拿「将来某类条目合法地带尾随字段」去攻：那要改格式版本或 incompat 位，读者按今天的格式拒是 I-1.10 字面要的「等于」；没有一张今天写得出的盘被拒（推的，读写者代码；没有跑全部 harness 用例，`cargo test -p singlefs-harness` 整包是 whole，子 agent 不起）。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| K1 | 多分段、多清区、写落在条文没说的段、屏障少一道或多一道 | 读 `make_filesystem` 全部写调用点（6 处）与录制流种类串；没量 |
| K2 | 随机源打不开、读不满、退回别的来源；内层入口有没有产品调用方；前 8 字节是不是随机源给的 | 读码；调用方用 grep 数（1 处对外、15 份文件调内层）；没量随机源失败那条路（换不了源） |
| K3 | 作废前后各崩；旧池几何（S、槽距、journal 环长）大于或小于新几何；写零出错代替崩溃 | 推的，表在 K3；没起崩溃枚举 |
| K4 | 六处比较两边的量；core 里 `<` 形读者；checker 主走读里对应六棵树的判法 | core 三条 grep（命中 1 + 1 + 3 行）、checker `walk.rs` 的 `entry_width < ` 27 行里只核了与六棵树对应的那几处 |
| K5 | 改名后每个匹配臂的成员与 `what`；拿字段做减法会不会下溢；今天写者写不写宽条目；六处宽条目各一张镜像过 checker | 副本上 6 张镜像各跑一次 checker、冷启动恢复与可写挂载（`out/run1-2026-10-22.log`） |

## 这条腿自己的限度

- 没调 `enumerate_crash_states`、没起任何 checker 档的东西：K3 全是推的。本可以在原型里录「用过的盘上换几何重做 mkfs」那条流、全量枚举（每段整写子集），但旧池要用 harness `tests/common` 里的 `build_pool` 建，原型二进制要把它另写一份；按读码推出的结论是落不到条文不许的结局，挂钟留给了 K4、K5 的量。主 agent 要坐实 K3 的换几何形状，可另派一条腿做这条流（状态数估在 10⁵ 以内，推的：同几何是 31 195）。
- K2、K5 的两份模型都在 harness 档的仓副本里跑，是副本上的数；K2 那张盘是「27 步全落盘」的完成态，不是从崩溃枚举里摆出来的。
- K4 的 checker 同形读者只核了与六棵树对应的主走读；`walk.rs` 里 27 处 `entry_width <` 的其余二次读（例如 `walk.rs` 6168、6178 行读分配记录树那一路，6294–6360 行读 extent 树那一路）只守不窄，它们是不判 I-1.10 的辅助读，我没逐处判它们该不该改。
- deadlist 内部条目宽于 key 宽 + 86 那一形（C780 改的那一处）同样会是「实现拒、checker 不罩」，推的，没造多层 deadlist 去量。
- 读到 `RandomSourceUnreadable` 没有用例：路径写死 `/dev/urandom`，用例没法造读不满。
- 没跑全部 harness 用例：整包是 whole，子 agent 不起；本轮新用例与改过的旧用例在副本里是否全绿我没验。

## 没做什么

- 不判正推那几格，不替主 agent 采纳；副本上的数不是入库装置上的数。
- 没跑门禁（segment-registry 格会不会红是按脚本推的）。
- 没碰 checker 档包的测试，没跑重型。
- 负载：开跑前 `ps` 看到别的会话在跑 `singlefs-checker-tier` 的 `cargo test` 与链接器（本机 loadavg 一度 395），没有性能测量在跑；两次 harness 跑在内存包装里排队 320 秒、3 秒。
- `tools/rerun.sh` 端到端没跑完：经 `run-with-memory-cap.sh 8G` 起，在内存包装的排队那一步等了 966 秒还没轮到（slice 已占 35–40 GiB、总上限 40.1 GiB，别的会话在跑 checker 档），我停了自己起的那条排队进程，包装报「排队那一步退 143……命令一行都没跑」。两份模型在副本里各跑通过一次（日志即 `out/` 两份）；`rerun.sh --selftest` 绿。要坐实复跑的，主 agent 照「复跑」一节那条命令起。
- 交回前删掉的：仓副本 `~/.local/share/singlefs/work/m3-close-w13-code-r1-opus/repo-copy`（35M）、编译目录同目录下 `target`（1.8G）；草稿目录里留着几份日志（共 52K），不含仓副本与编译目录。
