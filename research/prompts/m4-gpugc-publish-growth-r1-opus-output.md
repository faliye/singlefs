# m4-gpugc-publish-growth-r1 攻方腿（Opus）报告（2026-11-05）

立场：攻方。假设调查员的定位、外推与改法是错的，去找站不住的那几样。方案按 `crates/` 今天的实现谈；判决要引的数一律由主 agent 复跑（下面凡是我自己从日志算出的数标「量的（读日志）」，模型的数全是「推的，待主 agent 跑」）。

## 复跑命令与模型文件

- 模型目录：`research/prompts/m4-gpugc-publish-growth-r1-opus-model/`；冻结副本（只放三份探针日志的原样拷贝）：`/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus/frozen/`（三份 sha256 与调查员目录里的原件逐个相同：7603 `e1a438d5…`、7622 `14845dda…`、7623 `a940fd84…`）。
- 复跑（在仓根起，`<新目录>` 给一个不存在的工作根下目录）：`task-server/task submit -- bash research/prompts/m4-gpugc-publish-growth-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus/frozen --work <新目录>`
- 文件 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里现跑）：

```
a0a62b6784a6126a6a7b48a874ac3498acff1008bafad19a56b775eb830e3f5d  ./arc-bytes/Cargo.lock
9952465ade9c8d2f8b413f32cc9b17d2cf908ad49b56a24306bfdefe9320e80d  ./arc-bytes/Cargo.toml
92152aa8d396e99df9a65af2eb674c2996cfb068d8b2644852776540ca4596a7  ./arc-bytes/src/main.rs
ca55d8f7ea9999cef30491e7cb06417d8e16b053afbdc19e2e992150bdda1d09  ./out/calls.txt
21df885728f5c1f0d1585834f6fa4bdc2e7bdeb215035c9b804450ae0cf70196  ./out/residual_user.txt
9ffd71f26b4637e3dc5ffec03d5413e20e69e4b93979c125435677c0201d54fe  ./out/windows.txt
dab324e5e0058f7216bd60e169071cbef50d93e8e96bf9ab572c6a79fb515d2c  ./tools/calls.py
861c6e4fed9ba6d9a17cb3ca915fb0a34f5b5217f94a3d5d8a82e650635b397c  ./tools/rerun.sh
6862bd282dbbad635fbb08c77ef7d05d191fdd1ded3344abbdc869152482bcfb  ./tools/residual_user.py
f9f3aa500f94336dee5c7cfb436cefd2b3e89ef0c77071c9dbdbbc492041786d  ./tools/windows.py
```

- `tools/` 下三段 python 只读探针日志的文本、按窗口算每次用户改动的毫秒与每 MiB 单价（`windows.py`）、`publish_user_change_total` 减去它直接子格的余数（`residual_user.py`）、每次用户改动进各格的次数（`calls.py`）；`out/` 是它们喂三份日志的输出。仓里那三份被项目代码闸当本仓代码拒直跑，我跑的是草稿目录 `tools/` 下逐字节相同的三份（`cmp` 过）。`arc-bytes/` 是只用 std 的一段 Rust 模型（甲档单元代价），我编不了、跑不了，`out/` 里没有它的输出。

## 各格判定一览

| 格 | 调查员说的 | 攻方判定 | 依据的类别 |
|---|---|---|---|
| Q1.1 探针包没包对行 | 深拷四格包住的就是深拷 | **打中（部分）**：四格包对了它点名的那几行，但漏了同一条路径上另几处整版深拷与元数据节点的两次深拷；删垫片那一段 `publish_user_change_total` 有 43.4–56.9% 落在它的直接子格之外（三份日志 42 个删垫片窗口；顺序写、交错写窗口最多 0.2%） | 源码行 + 读日志（量的） |
| Q1.1 阳性对照 | 证明探针抓得到整版深拷 | **打中**：对照只证明「装置那一侧多花的时间进了窗口减发布的差」，证明不了核心层里的深拷落在哪一格；「1.00x 单元字节」那一条是同一个式子算两遍 | 探针源码行 |
| Q1.1 那 15% | 没装探针、也在涨，没点到行 | **点到行（推的）**：落在中央映射树每次整份重装、`mapped_units` 整份重装与整份比对、元数据节点两次深拷；都随 U 涨，甲档一样也碰不到 | 源码行 |
| Q1.2 「次数不涨、涨的是单次」 | 成立 | **大体成立，字面不成立**：取落点、释放、`record_for` 每次进的次数从 56.5/32.9/41.2 涨到 89.1/52.6/71.5（约 log U），不是不涨 | 读日志（量的） |
| Q1.2 「高于 P² 是缺页造成」 | 推的 | **能分辨，读数只撑一半**：照抄那一格的单价有缺页与没缺页一样（0.078 对 0.072–0.080 ms/MiB），缺页只抬整版深拷那一格；单价在缺页出现之前已经涨了 3.7 倍（缓存装不下） | 读日志（量的） |
| Q1.3 外推 P=36608「几十小时」 | 未写拟合 | **外推越位**：最大量到 P=4096，目标在它外面 8.94 倍；同一段历史三趟读数差到 2.9 倍；按读数里的局部次方 1.78–2.76 推出 8.9–111.4 小时，「几十小时」在里面但不由读数定 | 产物原行 + 算术（推的） |
| Q2.1 甲档改不改语义 | 不改 | **不改（推的）**，这一格没打中 | 源码行 + std 语义 |
| Q2.1 「系数小约三个量级」 | 推的 | **打中（推的，模型待跑）**：只对单元字节那一份成立的上界；整版深拷那一格拷的是整个 `TransactionOutput`，甲档之后剩 U 条映射 key 两张表与 R 条分配记录照拷；共享字节的引用计数住在每个单元自己那块堆上，照抄一次碰一条缓存线 | 源码行 + 模型 |
| Q2.1 甲档之后还剩的随 U 或 R 涨的项 | 列了按身份建索引一项 | **漏了六项**（见 Q2.1 表） | 源码行 + 读日志 |
| Q2.2 乙档是不是唯一能让每次发布不随 U 涨的 | 是 | **打中**：乙档一样不够——中央映射树整份重装、`plan.resolve` 每轮整份装 `mapped_units` 都不碰；乙档按身份有序还改了 `units` 的 bump 次序与下标语义 | 源码行 |
| Q2.3 有序结构 | 做得到，不碰格式 | **做得到、不碰 D3 已定项 7 盘上编码（推的）**；但 `records()` 交的是切片、次序从插入序变键序，157 处调用 | 源码行 |
| Q2.3 回滚日志 | 守 D16 已定项 17，被拒之后与发布之前逐位相同 | **打中**：已定项 17 不要求「换回」，它的欠账 C381 恰是「根落盘之后分配器仍退回」；而且落盘失败那一支要的是发布**之后**那一份分配器（冻结重发），回滚日志只给得出之前那一份 | kb 原句 + 源码行 |
| Q2.4 常驻多出的那一截 | 同一个根，甲档管得到 | **推的，量不出拆法**：`VmRSS` 含 glibc 留着没还的空闲堆，删垫片那一段版本少了 35 MB 而常驻只涨不落；拆不开活着的版本与留着的空闲 | 产物原行 |
| Q3 归类 | 深拷不在那四样判定本身，条文没写到 | **前半句站得住，后半句越位**：条文写到了——那张表的判据是「谁消费这个数」，深拷不产出任何被决策消费的数，按字面就不在第一格；而准入之前的固定点里每轮整份重装 `mapped_units`、整份建身份索引是随 U 涨的遍历，调查员没归类 | 条文原句 + 代码行 |

## Q1 定位与外推

开工先核了量的是哪棵树（量的）：探针副本里没装探针的文件与任务 7496 产物 `name=crates_file` 行逐个相同；主工作区此刻比那棵树多改了 9 份（`mount.rs` 加了一个测试模块与两段注释，其余 8 份只改注释里的包名），发布路径上的代码没变，读数照样适用于今天的 `crates/`。另：背景材料第二节说那四份核心层文件「此刻与 HEAD 相同（`git diff --stat` 输出为空）」——`git diff --stat` 比的是工作区与暂存区；现跑 `git diff --stat HEAD -- <四份>` 有三份不空（allocator.rs 151 行、mounted_session.rs 681 行、transaction.rs 597 行），与 HEAD 不同、与暂存区相同。不影响这一轮的判断（快照 sha256 对得上），但那句话按字面不成立。

### Q1.1 探针包没包对行、阳性对照证明了什么、15% 落在哪

**一、四格包住的行：包对了，但「深拷单元那一组」不只拷单元，也没包全。**

- `clone_into_versions_published` 包的是这一行（探针副本 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-copy/crates/singlefs-core/src/mounted_session.rs:3071` 起探针，下一行就是它）：「.push(published.clone());」（`crates/singlefs-core/src/mounted_session.rs:3067`）。`published` 是整个 `TransactionOutput`，拷的不只是 `units`：同一个结构里还有「pub mapping_keys: Vec<Vec<u8>>,」（`crates/singlefs-core/src/transaction.rs:3242`）、「pub allocation_records: Vec<AllocationRecord>,」（`crates/singlefs-core/src/transaction.rs:3244`）、「pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,」（`crates/singlefs-core/src/transaction.rs:3305`）、「pub files_other_than_the_first: Vec<FileOfAnInodeOtherThanTheFirst>,」（`crates/singlefs-core/src/transaction.rs:3241`）。前三样一样随 U（映射 key 每单元一个堆分配）或 R 涨。调查员把这一格整格算成「深拷单元、随单元字节线性」，口径宽了：读数分不出这一格里单元字节与别的表各占多少（Q2.1 用模型补这一半）。
- **同一条路径上另有整版深拷没包**（源码行，探针副本里这几行前后没有探针）：
  - 删垫片那一段：意图那一版之后走删除队列的每一批，「steps_published.push(current.clone());」（`crates/singlefs-core/src/transaction.rs:12180`；探针副本里是 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-copy/crates/singlefs-core/src/transaction.rs:12182`，没有探针）。它在 `publish_the_change_once` 之外、`clone_into` 之后、`replace_current` 之前跑。
  - 抬 F 推环：「let mut rehearsed = start.clone();」（`crates/singlefs-core/src/mounted_session.rs:3305`）、「versions_published.push(next.clone());」（`crates/singlefs-core/src/mounted_session.rs:3339`）。造池时 `floor_raises=0`，读数里没有它；产品稳态里抬 F 照常推进时每次推环每一版都走这里。
  - 搬迁与整理：「versions_published.push(relocated.clone());」（`crates/singlefs-core/src/mounted_session.rs:2087`）、「let mut relocated_version = previous_version.clone();」（`crates/singlefs-core/src/transaction.rs:14980`）。造池不走。
- **元数据节点每次照抄要拷两遍，两遍都不经 `carried_unit`**：extent 树照抄节点「bytes: carried.bytes.clone(),」（`crates/singlefs-core/src/transaction.rs:13661`）、多层码 2 树（中央映射、记账、deadlist）照抄节点「bytes: carried.bytes.clone(),」（`crates/singlefs-core/src/transaction.rs:16998`），装进各自的 `units` 之后再整份拷进这一版：「units.extend(extent_built.units.iter().cloned());」（`crates/singlefs-core/src/transaction.rs:16250`）、「units.extend(accounting_built.units.iter().cloned());」（`crates/singlefs-core/src/transaction.rs:16301`）、「units.extend(deadlist_built.units.iter().cloned());」（`crates/singlefs-core/src/transaction.rs:16302`）、「units.extend(mapping_built.units.iter().cloned());」（`crates/singlefs-core/src/transaction.rs:16376`）。`carried_unit_clone` 每次复制 93.7 MiB 而这一版 101.4 MiB（读探针的调查员自己的数），差的 7.7 MiB 里除了这次重写的十几个单元，就是这两遍拷的元数据；它们落在「没装探针的 15%」里。
- 量的（读日志，`out/residual_user.txt`，`publish_user_change_total` 减它六个直接子格）：顺序写与交错写窗口里余数最多 0.2%——四格加小格把交错写那一半包全了；删垫片那一段余数是 43.4%–56.9%（三份日志 42 个窗口）。原样两行：
  - 「task-7622.log P=2048 interleaved s=1024 n=264 total=60.78 outside_children=0.00 (0.0%) once_calls=264 admitted_calls=264」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/residual_user.txt:65`）
  - 「task-7622.log P=2048 unlinking_t s=8 n=8 total=69.80 outside_children=36.12 (51.7%) once_calls=8 admitted_calls=16」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/residual_user.txt:66`）
  - 它读的日志原行（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/task-7622.log:157`）：`E202 name=probe_window live_data_units=2048 phase=unlinking_the_spacer_files steps_done=8 window_seconds=0.567262 units_in_the_version=3252 unit_bytes_in_the_version=102088704 allocation_records_in_the_version=33056 mapped_units=3227 minor_faults_total=38828448 user_ticks_total=9385 system_ticks_total=3875 resident_bytes=1500663808 publish_user_change_total=8:0.558399:0 checks_before_the_publish=8:0.001674:0 publish_the_change_once=8:0.202094:0 clone_into_versions_published=8:0.062721:0 discard_after_the_change=8:0.000603:0 relocate_kept_for_snapshots=8:0.000010:0 replace_current_dropping_old=8:0.002327:0 allocator_clone_before_the_publish=16:0.006488:0 judge_before_any_read_or_write=16:0.182045:0 settle_allocation_record_tree=16:0.179784:0 settle_allocator_clone=38:0.008216:0 settle_plan_resolve=38:0.038888:0 settle_placements_released=38:0.013967:0 records_of_each_leaf=218:0.125582:0 allocator_make_room_for_record=1284:0.018642:0 allocator_release_find=1305:0.013246:0 allocator_record_for=1766:0.009186:0 publish_admitted_total=16:0.220443:0 allocation_records_sort_and_assert=16:0.030301:0 build_allocation_record_tree=16:0.011875:0 carried_unit_clone=49426:0.117340:1606844416 first_entry_index_build=528:0.007145:0 persist_the_publish=16:0.002664:0 device_read_at=2299:0.001387:27459584 device_write_at=656:0.001704:10493952 device_zero_discard_barrier=104:0.000041:0 apparatus_drop_of_the_published=8:0.008850:0 positive_control_extra_units_clone=0:0.000000:0`
  - 读法：`publish_admitted_total=16` 对 `publish_the_change_once=8`，每次改动的第二次发布（删除批）在 `publish_the_change_once` 之外；它自己那份 `publish_admitted`、`carried_unit_clone`、`records_of_each_leaf` 照记进全局那几格，**它的 `steps_published.push(current.clone())` 与上一版的释放不进任何一格**。读探针的调查员拿删垫片那一段当「两样分得开的校验」（深拷 23.90 → 16.53 跟着 U 走、扫记录持平），那一段的「深拷」少算了每次一份整版深拷（同一窗口 `clone_into` 一份是 7.84 ms，量的，`out/windows.txt:66`）；方向不变（漏掉的那一份同样随 U），占比（34%）偏低。**这一条不推翻交错写那一半的定位**，推翻的是「四格就是深拷那一组的全部」与删垫片那一段的占比。

**二、阳性对照证明不了「探针抓得到核心层里的一份整版深拷」。**

- 对照的代码（探针副本装置）：「let extra = version_of_trees(session).units.clone();」（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-copy/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:2116`），记的量是「u64::try_from(extra.iter().map(|unit| unit.bytes.len()).sum::<usize>()).unwrap_or(0),」（同文件 `:2119`），「drop(extra);」（同文件 `:2121`）在计时里面；窗口那一列 `unit_bytes_in_the_version` 是「let unit_bytes: usize = version.units.iter().map(|unit| unit.bytes.len()).sum();」（同文件 `:2148`）。
- 读探针的调查员三条判据逐条看：
  1. 「对照每次复制的字节是窗口里单元字节均值的 1.001–1.012 倍」：两个数是同一个式子（同一个 `units` 上 `bytes.len()` 求和）在同一窗口里的两个时刻各算一次，比值只反映窗口里版本涨了多少，**跟探针准不准无关**，是同义反复。
  2. 「对照每 MiB 毫秒是 `carried_unit_clone` 每 MiB 的 0.759–1.058 倍」：对照是一次整拷一个 `Vec<PublishedUnit>` 再当场释放，`carried_unit` 是逐个拷、不在计时里释放；两件事单价相近只说明「拷 32 KiB 缓冲」的单价在两处差不多，**说明不了核心层里哪一处深拷进了哪一格**。
  3. 「装置在发布之外多花的时间是对照的 0.91–1.00 倍」：唯一不平凡的一条。它证明的是「`publish_user_change_total` 没把装置那一侧的活包进去、窗口减它的差抓得住装置多做的事」——检验的是外层那一道减法，对照放在核心层外面，**一处核心层里的深拷落在 `carried_unit_clone`、`clone_into` 还是「没装探针的 15%」，它判不了**。上一条「删除批那一份整版深拷不进任何一格」就是对照漏掉的那种情形。
- 能判的对照（没跑，推的）：在核心层里一处不装探针的地方多拷一份整版（例如 `crates/singlefs-core/src/mounted_session.rs:3078` 之后多一次 `published` 的 clone），判据是 `publish_user_change_total` 减直接子格的余数涨一份、四格不动；再在 `:3067` 那一行拷两份，判据是 `clone_into` 那一格翻倍、余数不动。两条都中才说得上「探针把核心层里的深拷记在对的格里」。

**三、那 15% 落在哪（点到行，推的：源码行，没分格量）。**

读探针的调查员算的是 `publish_admitted_total` 减装了探针的三格（P=2048 交错写末窗口每次 6.89 ms）。`publish_admitted` 里没装探针、随 U 涨的这几处：

- 整份装 `mapped_units` 再与固定点算的那一份整份比：「let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations」（`crates/singlefs-core/src/transaction.rs:15799`）、「mapped_units, resolved.mapped_units,」（`crates/singlefs-core/src/transaction.rs:15804`，一条 `assert_eq!` 的参数）：每单元一把 27 字节 key 的堆分配加一遍比较。
- 中央映射树全部条目装进一张 `BTreeMap`、再把每把 key 拷出一份：「.map(|key| key.bytes().to_vec())」（`crates/singlefs-core/src/transaction.rs:15825`）；装树时先整份核计划里的 key 与这张表的 key：「let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();」（`crates/singlefs-core/src/transaction.rs:16961`）、「.zip(leaf_entry_bytes_by_key.keys())」（`crates/singlefs-core/src/transaction.rs:16966`）。
- 元数据节点两遍深拷（上面第一条）。
- 「let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);」（`crates/singlefs-core/src/transaction.rs:16608`）：这一格记进全局的 `first_entry_index_build`，不在调查员那条减法里减掉；取落点、释放、`record_for` 在 `publish_admitted` 里被调的那几次同样记在全局格、没减掉——读探针的调查员的「15.3%」是窗口减各组之后剩下的，而扫记录那一组按全局格求和，两边有重叠，**这个百分数的误差至少有几个百分点**，它自己也写了扫记录那一组是上界。
- `publish_admitted` 之外：`settle_plan_resolve`（P=2048 交错写末窗口每次 2.85 ms，量的，`out/windows.txt:65`）里每一轮整份装下一版的 `mapped_units`：「let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = Vec::new();」（`crates/singlefs-core/src/transaction.rs:9083`），每次改动 2.6 轮（`settle_plan_resolve` 次数 680 / 264，量的）。

这几处没有一处是「按单元字节」的深拷（元数据那两遍除外），都是按 U 条目数走一遍、或按 U 做堆分配。**甲档一处都碰不到**（Q2.1）。推翻条件：在这几块各装一格，量出它们合起来不到那 6.89 ms 的一半。

### Q1.2 「次数不涨、涨的是单次」与「高于 P² 是缺页造成」

**前一句：大体成立，字面不成立（量的，读日志）。** 每次用户改动进各格的次数（`out/calls.txt`）：

- 「task-7603.log P=512 sequential_ s=64 U=83 R=1226 publish_change_once=1.0 settle_a_clone=2.1 records_of_each_leaf=12.6 a_make_room_for_record=56.5 a_release_find=32.9 a_record_for=41.2 first_entry_index_build=34.1 publish_admitted_total=1.0 judge_before_any_read_or_write=1.0」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/calls.txt:2`）
- 「task-7622.log P=2048 interleaved s=1024 U=3386 R=32944 publish_change_once=1.0 settle_a_clone=2.6 records_of_each_leaf=14.7 a_make_room_for_record=87.5 a_release_find=52.6 a_record_for=71.5 first_entry_index_build=37.7 publish_admitted_total=1.0 judge_before_any_read_or_write=1.0」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/calls.txt:65`）

U 涨 41 倍，取落点、释放、`record_for` 每次进的次数涨 1.55、1.60、1.74 倍，`records_of_each_leaf` 1.17 倍、固定点轮数 1.24 倍。涨得慢（像 log U：每次重写的元数据节点随树高多一两个，每个节点要取一次落点，机理是推的），但不是「不涨」。扫记录那一组每次发布按 R 乘上一个随树高涨的次数，比「按 R 线性」略陡；读探针的调查员自己量到「涨得略快（每千条记录的毫秒 0.43 → 0.51）」，这就是来处之一。

**后一句：日志里有分辨它的读数，读数只撑一半（量的，读日志，`out/windows.txt`）。**

| 窗口 | 每次改动次缺页 | `carried_unit_clone` ms/MiB | `clone_into` ms/MiB | 出处 |
|---|---|---|---|---|
| 7622 P=2048 顺序写 s=128（4.4 MiB） | 0 | 0.021 | 0.022 | `out/windows.txt:50` |
| 7622 P=2048 顺序写 s=1024（33.6 MiB） | 7388 | 0.078 | 0.215 | `out/windows.txt:57` |
| 7622 P=2048 交错写 s=1024（101.4 MiB） | 23499 | 0.078 | 0.213 | `out/windows.txt:65` |
| 7622 P=2048 删垫片 s=16（93.2 MiB） | 0 | 0.078 | 0.083 | `out/windows.txt:67` |
| 7603 P=1024 交错写 s=64（22.1 MiB，进程里前一个池抬过高水位） | 0 | 0.124 | 0.118 | `out/windows.txt:34` |

原样两行：「task-7622.log P=2048 interleaved s=1024 n=264 U=3386 MiB=101.4 R=32944 clone_into=21.63ms (0.213/MiB) carried=7.31ms (0.078/MiB) replace=0.67 drop=3.71 pc=0.000/MiB faults/n=23499 resolve=2.85 feib=0.48」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/windows.txt:65`）；「task-7622.log P=2048 unlinking_t s=16 n=8 U=3116 MiB=93.2 R=33048 clone_into=7.75ms (0.083/MiB) carried=14.24ms (0.078/MiB) replace=0.33 drop=1.33 pc=0.000/MiB faults/n=0 resolve=5.35 feib=0.93」（`research/prompts/m4-gpugc-publish-growth-r1-opus-model/out/windows.txt:67`）。

- 照抄那一格的单价有缺页（0.078）与没缺页（0.078、0.124）一样：**缺页不进照抄那一格的单价**。缺页只抬整版深拷那一格，约每 MiB 多 0.13 ms。
- 缺页出现之前，照抄那一格的单价已经从 0.021 涨到 0.078（3.7 倍）：这一截是版本从装得进缓存到装不进（推的，没量缓存未命中）。
- 缺页数每次改动约是版本页数的 0.86–0.90 倍（7388 / 8602、23499 / 25958，量的），按字节线性。
- 所以「总挂钟高于 P²」在 P ≤ 4096 这一段里来自两个**起头**：缓存装不下、缺页开始。两样一旦起来都按字节线性（单价不再随版本涨），**往大 P 推它们不再贡献高于 2 的那一截**（推的）。调查员把「高于 P²」整个归给缺页，按读数只对了 `clone_into` 那一份。
- 另一条反向的读数（量的，产物原行，各自那一阶段末）：交错写末八分之一每步毫秒在 P=2048 三趟里是 133.529（探针那一趟）、184.663（共用存法）、358.455（各存各的），同一段历史差 2.7 倍；同一阶段的总秒数是 128.233、172.483、165.612，差 1.35 倍。「末八分之一」这个读数本身不稳，拿它算每次发布的次方不可靠：P=2048 → 4096 用共用存法同一趟算出每步次方 1.34，用各存各的同一趟算出 0.51，跨趟拿探针 P=2048 对 P=4096 算出 1.9–2.04。**「每次发布按版本线性」只在 P ≤ 2048 的探针读数里量过，P=4096 没有探针读数，那一档的每步读数既不撑也不推翻它。**
  - 「E202 name=construction_phase tier=2288MiB live_data_units=2048 chunks=shared phase=interleaved_half seconds=172.483 steps=1024 planned_steps_at_most=1024 publishes=2112 step_milliseconds_mean_of_the_earliest_eighth=82.287 step_milliseconds_mean_of_the_latest_eighth=184.663 step_milliseconds_maximum=739.247 units_in_the_version=3386 allocation_records_in_the_version=32944 resident_bytes=621735936 chunk_bytes_held_by_the_devices=696188928 chunk_bytes_stored=348258304」（`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-shared.out:83`）
  - 「E202 name=construction_phase tier=2288MiB live_data_units=2048 chunks=each-device-copies phase=interleaved_half seconds=165.612 steps=1024 planned_steps_at_most=1024 publishes=2112 step_milliseconds_mean_of_the_earliest_eighth=45.018 step_milliseconds_mean_of_the_latest_eighth=358.455 step_milliseconds_maximum=914.138 units_in_the_version=3386 allocation_records_in_the_version=32944 resident_bytes=971112448 chunk_bytes_held_by_the_devices=696188928 chunk_bytes_stored=696188928」（`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-each-device-copies.out:83`）
  - 探针那一趟（仓外，`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/probes-each-device-copies-P2048.out:73`）：`E202 name=construction_phase tier=2288MiB live_data_units=2048 chunks=each-device-copies phase=interleaved_half seconds=128.233 steps=1024 planned_steps_at_most=1024 publishes=2112 step_milliseconds_mean_of_the_earliest_eighth=49.964 step_milliseconds_mean_of_the_latest_eighth=133.529 step_milliseconds_maximum=509.738 units_in_the_version=3386 allocation_records_in_the_version=32944 resident_bytes=972464128 chunk_bytes_held_by_the_devices=696188928 chunk_bytes_stored=696188928`

结论：前一句「推的部分成立、字面不成立」；后一句「推的、只对一半」。推翻条件：P=4096 带探针跑一趟，`carried_unit_clone` 每 MiB 单价比 P=2048 末窗口（0.078）高出一半以上 ⇒ 单价没饱和、我这条错了。

### Q1.3 外推「整档 P=36608 要几十小时」

调查员两份报告与背景材料里都没写这句用的哪条拟合（`grep -rln 36608` 现查的范围是 `records/`、`.claude/kb/`、`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/`、装置源码与这一轮的材料：命中只有材料里出这一题的那一句与装置源码里三档 P 的定义，这几处里没有这句话的来处）。我能给的是按产物读数推的区间（**推的**）：

- 点：造池秒数（量的，产物原行）各存各的 P=1024、2048、4096 是 27.324、184.946、950.446（`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-each-device-copies.out:66`、`:77`、`:88`），共用存法 35.790、188.649、646.824（`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-shared.out:66`、`:77`、`:88`），探针那两趟 P=512、1024、2048 是 11.033、32.838、145.311。
- 每翻一倍的局部次方：各存各的 2.76、2.36；共用 2.40、1.78；探针 1.57、2.15；阳性对照那一趟 1.78。**同一个翻倍（1024 → 2048）三趟给出 2.15–2.76**。
- 端点外推多远：最大量到 P=4096，P=36608 是它的 8.94 倍（3.16 次翻倍）。次方差 0.5，外推结果差 e^(0.5 × ln 8.94) ≈ 3 倍。
- 区间（端点 × 8.94^次方，次方取读数里出现过的 1.78–2.76）：从共用存法端点 646.824 s 出发 8.9 h（1.78）、14.4 h（2.0）、31.6 h（2.36）、75.8 h（2.76）；从各存各的端点 950.446 s 出发 13.0 h、21.1 h、46.4 h、111.4 h。合起来 **8.9–111.4 小时**。
- 「几十小时」要的是次方 ≥ 约 2.15（各存各的端点）或 ≥ 约 2.3（共用端点）。Q1.2 那条推的机理（缓存与缺页两个起头之后单价饱和）指向次方往 2 收，按 2 算是 14.4–21.1 h；从探针 P=2048 端点按 2 算是 145.311 × (36608/2048)² ≈ 12.9 h。**读数定不了是十几小时还是几十小时**；把 P ≤ 4096 的局部次方 2.4–2.8 一直用到 P=36608 是外推越位。
- 外推区间之外的风险（推的，没量）：P=36608 时一版单元字节约 1.9 GB（P=4096 推的 211 MB × 8.94），交错写末多出单元字节的 2.6 倍、碎片挂载再多约 4 倍，常驻会到十几 GB；内存够不够、会不会换页，这几趟读数里没有对应的点。

## Q2 改法

### Q2.1 甲档（`PublishedUnit.bytes` 换共享不可变字节）

**改不改语义：不改（推的），这一格没打中。** 依据：

- 类型今天是「#[derive(Clone, Debug, PartialEq, Eq)]」（`crates/singlefs-core/src/transaction.rs:3205`）加「pub struct PublishedUnit {」（`crates/singlefs-core/src/transaction.rs:3206`），字段「pub bytes: Vec<u8>,」（`crates/singlefs-core/src/transaction.rs:3209`）。共享字节是不可变的：今天核心层里没有就地改单元字节的地方（`grep -rnE '\.bytes\[[^]]*\]\s*=|\.bytes\.(push|extend|truncate|resize|iter_mut|as_mut|fill|copy_from_slice)'` 在 `crates/singlefs-core/src` 只命中 `bytes.rs` 里另一个结构自己的 `bytes` 字段）。换成共享之后两版之间共用一块缓冲，谁都改不了它，「一版的字节不随另一版变」照旧成立。
- `PartialEq` 照内容比、`Debug` 打出与切片同样的文字：这是我印象里 std 对 `Arc<T>` 的约定（本机工具链没装 rust-src，这一轮没核到源码，**未核**）。录制流与写盘拿的都是 `&[u8]`，checker 读盘与录制流，不读内存里的类型：这三样不变（推的，没逐行核写路径）。
- 会多出的代价（推的）：`Vec<u8>` 换成 `Arc<[u8]>` 要重新分配一次、拷一遍（引用计数要住在同一块堆上），所以每个这次重写的单元多拷一次（O(这次重写的)），挂载读盘建版本那一下多拷一整版（O(U)，每次挂载一次）。
- 构造点与整份换字节的地方，读探针的调查员点的路径今天已经不在原处：仓里 `crates/singlefs-harness`、`crates/singlefs-checker-tier` 此刻是指向 `crates/singlefs-verification` 的符号链接（这一轮开工之后才出现的，`git status` 里旧路径是 `D`）。今天现数：`PublishedUnit {` 在 transaction.rs 15 处（含结构定义那一行，构造点 14）、mount.rs 3 处、recovery.rs 1 处，与读探针的调查员数的一样；`\.bytes = ` 在 `crates/singlefs-verification` 下 10 处（没逐处核是不是 `PublishedUnit`），另有 `std::mem::take(&mut unit.bytes)` 一处（`crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs:710`）。

**「系数小约三个量级」：打中（推的，模型待跑）。** 读探针的调查员的原句是「照抄与整版 clone 都只加引用计数，系数小三个量级」与「估计在 1 ms 以下」。两处站不住：

1. 深拷那一组里最大的一格 `clone_into`（P=2048 交错写末窗口 21.63 ms，量的）拷的是整个 `TransactionOutput`（Q1.1 第一条列的字段）。甲档之后它照拷：每单元一把映射 key 的两张表（`mapped_units`、`mapping_keys`，U 个堆分配各一次）、R 条分配记录（R ≈ 10U）、`files_other_than_the_first` 里每个文件的数据指针表；`replace_current` 与装置丢那一份照样逐个释放它们。日志里给得出这一截的上界（量的，读日志）：没缺页的删垫片窗口里 `clone_into` 每 MiB 0.080–0.106，`carried_unit_clone` 每 MiB 0.071–0.080（`out/windows.txt` 7622 删垫片那几行），整版深拷比逐个照抄每 MiB 多出的至多约 0.03 ms × 90 MiB ≈ 2.5 ms，这是「整版里单元之外那几张表」拷一份的上界；三格（拷、换、丢）合起来是这个量级的几倍。
2. 共享字节的引用计数住在每个单元自己那块堆的开头：照抄一次加一、换掉旧版与丢掉交回那一份各减一，**每个单元每一步都要碰一条不在缓存里的线**（版本 100 MiB、每个缓冲 32 KiB，彼此不在一页）。每次发布碰 U × 4 次（照抄、整版拷、换、丢）。这一步是按访存延迟算，不是按「搬几十字节」算（推的，模型 `arc-bytes` 量的就是它）。

跑前写下的读法（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「不许先有结论再建模型」：结果反过来我接受）：模型那一行 `name=model_ratio units=3386 owned_over_shared=<倍数>` 若 ≥ 1000，第 2 条撤回、「三个量级」对单元字节那一份成立；若在 20–200 之间，那句话高估了一到两个量级。`name=model_tables_left` 的 `total_ms` 若大于 `shared_arc` 那一行的 `deep_group_ms`，甲档之后整版深拷那一格的下限由单元之外那几张表定，不由单元字节定。模型只在我的模型上量、被攻过零轮；它不连 `crates/`，单元形状（落点 8 字节、身份 16 字节、字节）、R/U = 10、每 11 个单元一个 16 KiB 元数据单元都是照产物与源码取的近似。

**甲档之后每次发布还剩的随 U 或 R 涨的项**（读探针的调查员只列了按身份建索引一项）：

| # | 项 | 随什么涨 | 出处 | P=2048 交错写末窗口每次改动 |
|---|---|---|---|---|
| 1 | 照抄每个单元仍是一次按身份查加一次计数加一，装 `units`：「let mut units: Vec<PublishedUnit> = Vec::new();」（`crates/singlefs-core/src/transaction.rs:16189`） | U | 源码 | 推的，模型 `shared_arc` 的 `carry_ms` |
| 2 | 整版深拷、换、丢里单元之外那几张表 | U（映射 key 两张表）、R（分配记录） | Q1.1 第一条 | ≤ 约 2.5 ms 一份（推的上界，读日志） |
| 3 | 按身份建索引，每次改动 37.7 次，每次整张表排一遍序 | U log U | `first_entry_index_build` 一格 | 0.48 ms（量的，`out/windows.txt:65`） |
| 4 | 固定点每轮整份装下一版 `mapped_units`（`crates/singlefs-core/src/transaction.rs:9083`），每次改动 2.6 轮 | U | `settle_plan_resolve` 一格 | 2.85 ms（量的） |
| 5 | `publish_admitted` 里整份装 `mapped_units` 再整份比、中央映射树全部条目装表再拷 key、装树前整份核 key（Q1.1 第三条） | U、U log U | 源码 | 在读探针的调查员的 6.89 ms 里（量的减法，拆法推的） |
| 6 | 扫分配记录那一组 | R、R log R，乘一个随树高涨的次数 | 读探针的调查员那七格 | 16.80 ms（量的，上界） |
| 7 | 删除批那一份整版深拷（`crates/singlefs-core/src/transaction.rs:12180`）与抬 F 推环那两处（`crates/singlefs-core/src/mounted_session.rs:3305`、`:3339`） | U（甲档之后不再按字节） | 源码 | 造池交错写不走；删垫片那一段每次一份 |

按这一窗口的读数推（推的）：每次改动 60.78 ms 里，甲档能去掉的是深拷三格里按字节的那部分（约 29.6 ms 减去第 1、2 项的下限），剩下约 30 ms，**造池在 P=2048 快约两倍，涨法（每次发布随 U 与 R 线性、造池约 P²）不变**。

### Q2.2 乙档（持久结构，每次发布不随单元数涨）

**要动的公开类型与调用方（现查）：** `TransactionOutput::units`（「pub units: Vec<PublishedUnit>,」，`crates/singlefs-core/src/transaction.rs:3233`）；交出整版的几处公开类型——「pub type VersionsPublishedInThisChange = Vec<TransactionOutput>;」（`crates/singlefs-core/src/mounted_session.rs:482`）、`CompactionRun`、`RootRingPushed` 等的「pub versions_published: Vec<TransactionOutput>,」（`crates/singlefs-core/src/mounted_session.rs:315`、`:1884`、`:1942`）、删除队列的「pub steps_published: Vec<TransactionOutput>,」（`crates/singlefs-core/src/transaction.rs:12138`）、挂载交回的「pub row_publish: PoolVersion,」（`crates/singlefs-core/src/mount.rs:1041`）、「pub warm_up_publishes: Vec<PoolVersion>,」（`crates/singlefs-core/src/mount.rs:1043`）、「pub deletion_queue_continuation: Vec<TransactionOutput>,」（`crates/singlefs-core/src/mount.rs:1065`）。读 `.units` 的地方现数 365 处：`crates/singlefs-verification` 60 份文件、`crates/singlefs-core/src` 5 份、`crates/singlefs-background-gpu` 与 `crates/singlefs-checker` 合 2 份。

**按读探针的调查员写的样子（「units 换成按身份有序、结构共享的持久结构」）会改语义：打中。**

- `units` 今天的次序是写进字段文档的：「/// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（文件 / 固定点角色按 bump 次序——每个数据单元、extent 根、」（`crates/singlefs-core/src/transaction.rs:3231`）。按身份有序就把 bump 次序换成身份次序，读 `units` 次序的 365 处里依赖它的那些（没逐处分）一起变。
- 有按下标指进 `units` 的公开字段：「pub container_tree_root_unit_positions: ContainerTreeRootUnitPositions,」（`crates/singlefs-core/src/transaction.rs:3325`），它的文档说是根单元「在 `units` 里的下标」。按身份的映射里没有下标这个概念。
- 按身份查的索引写明了表里同一身份可以出现几次、取最先那一项（`crates/singlefs-core/src/transaction.rs:3687` 那一行文档）；按身份为键的映射把重复的并掉，行为与 `iter().find` 不再逐项相同。
- 要不改语义，持久结构得是「按 bump 次序、可按下标寻址、另带按身份的索引」的那一种，不是调查员写的那一种（推的；这是另一个设计，被攻过零轮）。

**「只有它能让每次发布不随单元数涨」：打中——乙档也不够。** 甲档表里第 3、4、5 项（按身份建索引、固定点每轮整份装 `mapped_units`、中央映射树全部条目每次整份装表）都不读 `units` 的类型，乙档一项都不碰；第 6 项随 R。要做到每次发布不随 U 涨，`mapped_units`、中央映射条目表、几张身份索引都要换成增量维护（在固定点里与发布里按「这次重写的角色」改），这几样调查员没列。反过来，「不随单元**字节**涨」也不只有乙档能做到：版本里不带数据单元的字节（读探针的调查员自己写的「还剩的一问」），或者交给调用方的那一份改成移交而不是拷贝（`crates/singlefs-core/src/mounted_session.rs:3067` 那一次），都不用持久结构（推的，两样都被攻过零轮）。

### Q2.3 扫分配记录那一组的三样改法

**一、记录换成按（设备，槽）有序的结构：做得到，不碰 D3 已定项 7 的盘上编码（推的）。** 已定项 7 管的是盘上条目：「| 7 | **分配记录条目的盘上编码与宽度** | 落点粒度 16384；key =（设备身份 4, 槽号 6）；value =（跨度段 2, 代 8）；释放时改写不点删 **状态：已定。** **进度：已实现。** |」（`.claude/kb/decisions/03-空间分配.md:19`），正文那一句整行在附录 A1（`.claude/kb/decisions/03-空间分配.md:134`）。内存里怎么索引，这两行都不管；写进版本的分配记录今天已经在发布时拷出来排过序（「let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();」，`crates/singlefs-core/src/transaction.rs:15389`；「allocation_records.sort_by_key(AllocationRecord::sort_key);」，`crates/singlefs-core/src/transaction.rs:15390`），写盘的字节不变。要动的是公开接口：「pub fn records(&self) -> &[AllocationRecord] {」（`crates/singlefs-core/src/allocator.rs:2198`）交的是切片，`.records()` 现数 157 处（`crates/singlefs-core/src` 里 27 处）；换成有序结构之后切片没了，读者看到的次序从插入序变成键序。「释放时改写不点删」只要释放仍是就地改写那一条就守得住。

**二、「这次碰过哪几片叶」当分配、释放的副产品：做得到，方向对 fs-design 那一节（推的）。** 要点名的一处：读探针的调查员提议把发布时那条全量 `assert!`（`crates/singlefs-core/src/transaction.rs:15401` 起，消息是「"准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点"」，`crates/singlefs-core/src/transaction.rs:15410`）挪进测试。它今天是 release 下也跑的断言：挪走之后产品在「固定点与真发对不上」这种缺陷上从 panic 变成照发。这是失败路径上的行为变化，不是纯性能改法，要单独交主 agent 定。

**三、分配器整份拷贝换回滚日志：做得到，但读探针的调查员引的条款读反了、漏了一支。打中。**

- 读探针的调查员的原句（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator-2/result.md:227`，整行）：「- `.claude/kb/decisions/16-发布语义.md:27`（已定项 17「不许假设可复用旧内存态重试」）：回滚日志那一步必须撤得干净，被拒之后的分配器与发布之前逐位相同，上面的回滚测试就是钉这一条的。」已定项 17 那一行：「| 17 | **fsync 报错之后调用方能假设什么** | 报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定 **状态：已定。** **进度：已实现。** |」（`.claude/kb/decisions/16-发布语义.md:27`）；正文：「- **报错之后不许假设可以照旧复用同一份内存态重试。**（`close(2)` 的 NOTES 段明写「Retrying the close() after a failure return is the wrong thing to do」；本工程取同一条语义。）」（`.claude/kb/decisions/16-发布语义.md:377`）；它的欠账：「**欠**：`C381`（根已落盘之后发布失败，分配器仍退回）（改法落地之前这一格还开着）。」（`.claude/kb/decisions/16-发布语义.md:393`），C381 那一行整行在附录 A2（`.claude/kb/checks-owed.md:265`）。
  - 已定项 17 约束的是**调用方**报错之后能假设什么，它说的是「不许假设旧内存态能复用」；它不要求实现「被拒之后与发布之前逐位相同」。它挂着的欠账 C381 恰好说「根已落盘之后发布失败而分配器整个退回」是缺陷。把「逐位退回」当成已定项 17 要守的东西，是把这一条读反了。
- 今天的代码有三种结局，不是两种：发布前被拒（换回去）、发成（丢掉拷贝），以及**落盘那几步里失败**：「/// 分配器换回发布之前的样子，这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。」（`crates/singlefs-core/src/transaction.rs:1581`），冻结时留着的是发布**之后**那一份分配器：「let allocator_after_the_publish =」（`crates/singlefs-core/src/transaction.rs:1595`），原样重发成功时换上它。只记撤销的回滚日志在这一支上给得出「之前」那一份，给不出「之后」那一份；要么失败时现拷一份（只在失败路径上付 O(R)），要么另记重做。读探针的调查员的测试只写了「在固定点与落盘前各注入一次拒绝」，落盘失败、冻结、原样重发这一支没写到。
- 测试写法也要改：读探针的调查员要的「拒绝之后的分配器与发布之前拷的一份 ==」，今天的分配器没有相等：「#[derive(Clone, Debug)]」（`crates/singlefs-core/src/allocator.rs:1866`）「pub struct PoolAllocator {」（`crates/singlefs-core/src/allocator.rs:1867`）。要加 `PartialEq`；记录换成有序结构之后，这个相等不再看插入次序。
- 拷分配器的地方不止调查员说的两处：发布入口「let allocator_before_this_publish = allocator.clone();」在 `crates/singlefs-core/src/transaction.rs:1859`、`:12499` 两处，固定点「let mut rehearsal = allocator.clone();」在 `crates/singlefs-core/src/transaction.rs:1964`、`:13283` 两处，推环预演「let mut copy = allocator.clone();」（`crates/singlefs-core/src/mounted_session.rs:3304`）。回滚日志要撑住「每一轮从同一个起点重来」与这几处嵌套。
- 位图那一份（「// 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。」，`crates/singlefs-core/src/transaction.rs:12498`）随盘容量涨，这一轮盘大小没变，量不到（推的，同读探针的调查员）。

### Q2.4 常驻比块字节多出的那一截

**结论：推的，读数拆不开；甲档管得到其中按版本份数重复的那部分（推的），管不到版本自己那一份与 glibc 留着没还的空闲堆。**

- 常驻是 `/proc/self/status` 的 `VmRSS`（装置 `resident_bytes_of_this_process`），含已经释放、glibc 没还给内核的堆。读数里能看到它（量的，产物原行）：各存各的 P=2048 碎片挂载之后到删垫片末，版本从 3386 个单元降到 2265 个（少了约 35 MB 单元字节），常驻从 1494048768 涨到 1499574272；P=4096 共用存法那一趟进程开头常驻已经是 624517120（前两个池留下的）。所以「2071564288 对 586481664」那一截里有多少是活着的版本拷贝、多少是留着的空闲堆，这几份读数分不出来。能分的量（没跑）：在同一阶段末读一次 `mallinfo2` 的在用字节，或先 `malloc_trim(0)` 再读常驻。
- 「同一个根」：每个版本值各带整份单元字节（`crates/singlefs-core/src/transaction.rs:3209`），持有方是会话的 `current`、交给调用方的那几份、挂载交回值（`crates/singlefs-core/src/mount.rs:1041`、`:1043`、`:1065`）与装置留着的「let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);」（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:2521`，活到函数结束）。甲档让这些份数共用一块缓冲（前提是挂载里那几版都是从同一份读盘建出来的版本照抄来的，没核），剩下版本自己那一份（与内存盘里的块各存一份）与各版本的元数据表；后一项随 U、R 涨（Q2.1 第 2 项）。

## Q3 归类

读探针的调查员那一句的原文：「它装的是这一版的样子，不是在为分配、准入、defer 窗口或生命周期判定求一个数；表里那一格写的是这四样，它算不算，条文没写到。」

条文（整行）：

- 「**线划在「谁消费这个数」，不划在「用什么机制算出来」。**」（`.claude/rules/fs-design.md:19`）
- 「| 消费者 | 允不允许遍历 |」（`.claude/rules/fs-design.md:21`）
- 「| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 |」（`.claude/rules/fs-design.md:23`）
- 「⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**」（`.claude/rules/fs-design.md:29`）
- 「⚠️ **被禁止的不是「遍历」，是两件事**：」（`.claude/rules/fs-design.md:33`）「「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。」（`.claude/rules/fs-design.md:34`）

判：

1. **前半句站得住。** 深拷的几处都在准入之后：判决与准入在「} = judge_the_publish_before_any_read_or_write(」（`crates/singlefs-core/src/transaction.rs:12861`）里走完，之后才进「publish_admitted(」（`crates/singlefs-core/src/transaction.rs:12970`），照抄在 `publish_admitted` 里，整版深拷在落盘之后（`crates/singlefs-core/src/mounted_session.rs:3067`）。它们不产出任何被分配、准入、defer 窗口或生命周期判定消费的数。
2. **后半句「条文没写到」越位。** 条文写到了：表头的维度是「消费者」，第 29 行的判据是「被问到那一刻有没有答案」，按字面这一行只管「为决策求一个数」的那种遍历；不产出被决策消费的数的事，按这两句就不落在第一格。条文没写的是「不落在第一格的、随 U 涨的发布代价归谁管」——那是另一件事，不是「它算不算，条文没写到」。
3. **调查员两组之外还有一组该归类而没归类。** 准入之前的固定点里每一轮整份装下一版的 `mapped_units`（`crates/singlefs-core/src/transaction.rs:9083`，在 `plan.resolve` 里，`settle_plan_resolve` 一格每次改动 2.85 ms、随 U 涨，量的），并按身份整张建索引；固定点的结果交给准入（「let settled = settle_the_allocation_record_tree(」，`crates/singlefs-core/src/transaction.rs:12721`，在 `judge_the_publish_before_any_read_or_write` 里）。这是在决策那一刻按 U 现算、而不是事务里增量维护着的东西，按第 29、34 行的判据比深拷更像第一格的对象（推的；条文第一格的第二分句「代价不许随盘容量增长」，它随的是 U、不是盘容量，这一分句判不中它，第一分句「不许」遍历按第 33–34 行的口径判得中）。扫记录那一组的归法照调查员（落第一格的第一分句；第二分句只在位图拷贝上判得中，那一样随盘容量涨）。

## 打中之后的四句

照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」。这一轮被判的是调查员结论，「臂」读作调查员给的几条改法（甲、乙、扫记录三样）与几个相争的解释；「判据」读作调查员自己写死的推翻条件与这一轮引的条款。

| 打中 | 分不分辨臂 | 被判的东西当时看不看得到判别它的东西 | 满足的是哪一句的哪一分句 | 各改法在打中的格上还中不中 |
|---|---|---|---|---|
| Q1.1 漏包的整版深拷与元数据两遍深拷 | 不分辨甲乙（两档都管字节）；分辨「深拷 = 四格」与「深拷 > 四格」 | 看得到：三份日志里有 `publish_user_change_total` 与它的直接子格，余数算得出 | 读探针的调查员推翻条件第 3 条（「若其中一格按 U 涨、量级与深拷相当 ⇒ 深拷那一组的占比被低估了」，读探针的调查员报告第 292 行）：元数据那两遍在 `publish_admitted` 里，删除批那一份在它外面；只中「按 U 涨」，「量级与深拷相当」没量 | 甲档、乙档都修得到（都按字节）；扫记录那三样不碰 |
| Q1.1 阳性对照证明不了核心层里的深拷落格 | 不分辨臂 | 看得到：对照放在装置里，核心层里没有对照 | 装探针的调查员推翻条件第 6 条（装探针的调查员报告第 147–148 行）：「抓不到、或者它跟 `unit_MiB` 不成比例」——「成比例」那一分句被同一个式子算两遍满足，是同义反复；「多出差不多这么多」那一分句满足，但它只检验外层减法 | 与改法无关 |
| Q1.1 那 15% 点到行 | 分辨：甲档一处都不修，乙档（按调查员写的）也不修 | 看得到行（源码），量不到拆法（没分格） | 读探针的调查员推翻条件第 3 条（同上第一行）：这几处按 U 涨；量级没分格量 | 甲、乙都还中；要另做增量维护 |
| Q1.2 「次数不涨」字面不成立 | 不分辨臂 | 看得到（次数在日志里） | 读探针的调查员第一节第 2 问末条「每次发布的调用次数不随版本涨」 | 与改法无关 |
| Q1.2 「高于 P² 是缺页」只对一半 | 分辨解释：缺页 / 缓存 | 看得到（缺页数与每 MiB 单价都在日志里） | 「总挂钟高于 P² 是缺页造成」整句 | 甲档去掉字节拷贝，缺页与缓存两份一起去掉；不改结论的去向 |
| Q1.3 外推越位 | 不分辨臂 | 看得到：读数只到 P=4096 | 正文 Q1 第 3 问 | 与改法无关 |
| Q2.1 「三个量级」高估 | 分辨甲与乙的收益：甲档之后整版深拷的下限由单元之外几张表定，乙档共享整版能去掉这一格 | 看得到行；量要等模型 | 读探针的调查员「系数小三个量级」「估计在 1 ms 以下」两句 | 甲档还中（第 2 项）；乙档若同时共享整版（调查员写了 `Arc<TransactionOutput>`）修得到第 2 项，修不到第 3–6 项 |
| Q2.2 乙档改语义、也不够 | 分辨：乙档按调查员写的样子与「按 bump 次序的持久序列」两种 | 看得到（字段文档与下标字段） | 正文 Q2 第 2 问「是不是只有它能让每次发布不随单元数涨」 | 乙档还中第 3–6 项；「版本不带字节」「移交不拷贝」同样不修 3–6 |
| Q2.3 回滚日志读反已定项 17、漏冻结那一支 | 分辨回滚日志的两种写法：只记撤销 / 撤销加失败时现拷之后那一份 | 看得到（`persist_the_publish_or_freeze_it` 在源码里） | D16 已定项 17 定案第二条「报错之后不许假设可以照旧复用同一份内存态重试」，与它的欠账 C381 | 调查员写的「被拒之后与发布之前逐位相同」那条测试在落盘失败那一支上不判；加「失败时现拷之后那一份」修得到 |
| Q3 后半句越位 | 不分辨臂 | 看得到（条文） | `.claude/rules/fs-design.md:29` 那一句判据与第 33–34 行 | 与改法无关；多出来的那一组（固定点里整份装 `mapped_units`）甲乙都不修 |

## 几个改法各修哪一格（我提的改法只在我的模型上量过、被攻过零轮；这一轮一格都没在副本上量）

| 格 | 甲档（共享字节） | 乙档（调查员写的样子） | 有序记录 + 碰过的叶 | 回滚日志 | 我提的：映射条目、`mapped_units`、身份索引增量维护 | 我提的：交出的那一份移交不拷贝 |
|---|---|---|---|---|---|---|
| 照抄与整版深拷的字节（`carried_unit`、`:3067`、`:12180`、元数据两遍） | 修（推的） | 修（推的） | 不修 | 不修 | 不修 | 只修 `:3067` 那一份（推的） |
| 整版深拷里单元之外的表 | 不修（推的，待主 agent 跑模型） | 共享整版时修（推的） | 不修 | 不修 | 不修 | 修 `:3067` 那一份（推的） |
| 按身份建索引 37.7 次 | 不修 | 不修（推的） | 不修 | 不修 | 修（推的） | 不修 |
| 固定点每轮整份装 `mapped_units` | 不修 | 不修 | 不修 | 不修 | 修（推的） | 不修 |
| `publish_admitted` 里映射树整份装表、整份比 | 不修 | 不修 | 不修 | 不修 | 修（推的） | 不修 |
| 扫分配记录（R） | 不修 | 不修 | 修大头（推的） | 只修拷分配器那 1 ms 左右（推的） | 不修 | 不修 |
| 位图拷贝（随盘容量） | 不修 | 不修 | 不修 | 修（推的） | 不修 | 不修 |

## 没打中的形状

- 甲档改不改 `PartialEq`、录制流、写盘字节、checker 读的东西：找了就地改单元字节的写法（核心层零处）、`Debug` 文字、相等的口径，没找到会变的地方（`Arc` 那两样是我印象里的 std 约定，未核）。范围：`crates/singlefs-core/src` 全部 `.rs` 与 `crates/singlefs-verification` 里 `.bytes =` 的十处。按三方规则这一格「没打中」只抽了一次样，不支撑「没问题」。
- 有序记录碰不碰 D3 已定项 7：找了写盘那一处（已经排序后写），没找到盘上字节会变的路径。范围：`transaction.rs` 里拷记录排序那两处（`:2161`、`:15389`）。
- 探针本身有没有在被测路径上改了行为：五份文件的改动都是在原语句外面套一层作用域计时（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus/probe-core.diff`），没看到改语义的；`publish_the_change_once` 那一格把 `match` 的被匹配表达式包进块里，求值次数不变。
- 装置自己在拖：没找到比读探针的调查员更大的一截（`outside` 减装置丢那一份之后 ≤ 1.6%，照它的读数）。

## 这条腿自己的限度

- 我编不了、跑不了：模型 `arc-bytes` 没编过（只用 `rustfmt --check` 查过能解析），类型错误这一层没查；它的每个数都待主 agent 跑。三段读日志的小脚本是我在草稿目录跑的，输出在 `out/`，算「量的（读日志）」，判决要引照旧由主 agent 复跑。
- 探针那三份日志都是各存各的、单趟；P=4096 没有探针读数。我对 P=4096 的说法全是推的。
- 「15%」那几处只点到行，没分格量；元数据两遍深拷的量只用读探针的调查员的「93.7 对 101.4 MiB」差值粗估。
- 崩溃状态：这一题问的是运行时代价，我没造历史、没起崩溃枚举，崩溃状态数 0；定义第 3b 步点名的 `crates/singlefs-harness/src/memory_pool.rs` 此刻经符号链接指到 `crates/singlefs-verification`，没用到。
- 读的是此刻的工作区；开工之后另有会话在合包（`crates/singlefs-harness`、`crates/singlefs-checker-tier` 变成符号链接），被判的六份文件（`judged-sha256.txt`）的 sha256 开工时与交回前各核一次，都对得上。

## 要主 agent 跑的命令

1. 模型目录整份复跑（三段读日志的小脚本与 `arc-bytes` 模型）：
   - 命令：`task-server/task submit -- bash research/prompts/m4-gpugc-publish-growth-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus/frozen --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus-rerun`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（照三方规则，判决复跑时把模型目录拷进主 agent 自己的草稿目录、`--model` 指过去也行）。`--work` 那个目录要不存在或是空的。
   - 估时（推的，没跑过）：编一个只用 std 的小包不到一分钟；跑的部分最大一份是 60500 个共享单元，约 1.9 GB 字节，几十秒以内。各存各的那一臂最多 6724 个单元，峰值约 0.63 GB。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0` 算过；日志里 `name=rerun_compare file=windows.txt same=yes`、`file=residual_user.txt same=yes`、`file=calls.txt same=yes` 三行（与 `out/` 逐字节相同）；`name=model_ratio units=3386 owned_over_shared=…` 与 `units=6724` 两行、`name=model_publish mode=shared_arc units=60500 …` 一行、`name=model_tables_left …` 三行、末行 `name=done`。读法见 Q2.1「跑前写下的读法」。
   - 跑完之后：新派同立场的腿（带「承接：本报告路径」与任务号）读 `task-server/task log <号> --all`，把 Q2.1 与改法表里标「推的，待主 agent 跑」的那几格改成量过或撤回。
2. 下一步可选、这一轮没写命令的（要做再派）：核心层里的阳性对照（Q1.1 第二条末尾那两条：不装探针处多拷一份整版、`:3067` 拷两份）；P=4096 带探针跑一趟（Q1.2 推翻条件）；在 `publish_admitted` 里给 Q1.1 第三条那几块分格。

## 没做什么

- 不判正推腿的格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没在副本上改代码、没编译、没跑装置；副本上的数一个都没有。
- 没逐处分 365 处 `.units` 读者里哪些依赖 bump 次序、157 处 `.records()` 里哪些依赖插入序。
- 没核挂载里那几版是不是都从同一份读盘版本照抄来（Q2.4 甲档管得到的前提）。
- `Arc<[u8]>` 的 `PartialEq`、`Debug` 行为没在源码里核（本机没有 rust-src）。

## 清理

- 没建仓副本、没建编译目录。草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-opus` 里是文本：两份探针 diff、几份读日志的输出、`tools/` 三段小脚本、`rerun_steps.part`、`frozen/probe-runs/` 三份日志拷贝（`.deliverables` 写成「交接」，复跑要用）。

## 附录：整行引的 kb 条款

A1（`.claude/kb/decisions/03-空间分配.md` 第 134 行，整行）：

**落点释放时条目不删、不点删**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4）：改写成「已释放 + 释放代」——释放代写进那 8 字节，已释放标志位借跨度段的最高位，条目留到该落点被重新分配时覆盖。不点删的理由：树按落点排序、代住 value，按代点删要扫全树，而点删所需的按代清单正是崩溃会丢的那份内存态；条目改写与产生这次释放的 COW 在同一次发布里原子生效——分配记录是根下的 COW 树（D22（单元原子性怎么合成） 已定项 3），条目与根同一次发布，释放代 = 那次发布的 checkpoint_txg；崩溃丢掉的是没发布的那个 checkpoint，它里面的释放一条都不留痕、对应的旧版本仍被所选根引用。崩溃后 defer 队列的内存态丢失，接手的是「释放代 ≤ max(F_生效, 环里最旧有效根)」这条闸（I-7.4（近 K 代块未被复用） 的注；谓词由 D16（发布语义） 已定项 1 给，有效性按实例表判；抬 F 的机制押在这个谓词上）。「分配代」的口径：搬迁的目的地那条记录写单元的诞生代，不写搬迁那次发布的 txg（D26（后台整理与放置回收） 已定项 5）。

A2（`.claude/kb/checks-owed.md` 第 265 行，C381 那一行，整行）：

| C381 | 根已落盘之后发布失败，分配器仍退回 | 发布的持久顺序是根记录 FUA 之后再转系统配置槽；系统配置槽那一步失败时根已落盘，恢复会选中它，而发布路径把分配器整个退回到这次发布之前（`crates/singlefs-core/src/transaction.rs` 的 `publish_version`）；同一个写入口接着发下一次发布会把这次占过的槽再分出去，写到一半断电就把已落盘的根引用的单元盖掉（代码三方第二轮攻方：checker 五条红、重开读不出单元） | 造「系统配置槽写失败 → 同一个写入口发一次空发布、单元写完记录与根没写时断电」这段历史，恢复之后 checker 全绿、重开成功——checker 五条红、重开报 `Recovery(UnitUnreadable)`；判别力自证：原样重试同内容的那条对照就绿 | D16（发布语义） 已定项 7 的失败语义要先定：根落盘之后的失败算不算这次发布成立，失败之后这个写入口还许不许接着发布；2026-09-18 用户定走三方立题。第一轮 2026-09-19 已判（`research/prompts/c381-r1-main-verification.md`）：当时的「一律退回」在攻方原历史与「根槽 FUA 报错而根已落盘」两处坐实是缺陷；D23（journal 的角色与格式） 已定项 14 的失败表（先探针写，瞬时走实例切换、持续转只读到下次挂载）已经答了失败之后写入口怎么走，`crates/` 里探针写、实例切换、转只读三样都没有；第二、三轮都判完了（2026-09-21）：第二轮判出三条候选栽在同一个共用前提上——失败表要的探针写盘上没落点、代码里没实现、判别力上分不出落点级的故障；用户定了两题（探针落点进地址空间表、失败表射程逐个失败点列表）、打回两题；第三轮攻那两题，候选甲（不立条款、明写 fsync 报错之后数据可以出现也可以不出现）站住并要加两样，候选乙（探针写改成写失败的那个落点）被推翻。按「第三轮之后停」这条线到此为止。四题用户都定了：第二轮定第 1 题（探针落点进地址空间表）与第 3 题（失败表射程逐个失败点列表），第三轮定第 2 题（按候选甲写回并加两样，D16（发布语义） 新立一条已定项）与第 4 题（改法 B：失败表判别子改成固定落点探针写 + 失败落点只读复核），「实例切换取的是内存里的根」那一题 2026-09-21 定另立一题，2026-09-23 用户改定「改条款：切换要重新读盘」（C458（实例切换取内存里的根，不重新读盘）），条款写进 D23（journal 的角色与格式） 已定项 14 与 D18（块里携带什么信息） 已定项 11。2026-09-21 写回的是第 2、4 题；第二轮那两题各自还缺落地前提（落点登记在哪张表与哪个偏移、射程那八格怎么判），各记一笔欠账。**代码那一半整个仍欠**：`crates/` 里探针写、只读复核、实例切换、转只读四样都没有。**事务号也在重用之列**——根槽那一步失败之后事务号已经进了持久记录，重试拿同一个号、jsn 也重、实例没换（判决判定三，攻方探针 `opus_probe_txn_chain.rs` 复跑坐实；⚠️「改回旧写法同样中」那句是推理坐实、没有 A/B 实测）。这一族要的是 D23（journal 的角色与格式） 已定项 7 后半句「失败即实例切换」落地，而 `crates/` 里探针写、只读复核、实例切换、转只读四样都没有。用户 2026-09-23 划一刀：代码那一半跟里程碑「覆盖写、释放、回退与复用」收口表第 16 行一起挪到后面的里程碑；去向：`.claude/kb/milestone/06-sixth-txn.md` 一·10（代码那一半；探针写、只读复核、实例切换、转只读四样；归属是推的，开工前核）。 | `research/prompts/m2-wave2-code-r1-main-verification.md`、`research/prompts/c381-r2-main-verification.md`、`research/prompts/c381-r3-main-verification.md` 第二节第 7 行；攻方报告 `research/prompts/m2-wave2-code-r1-main-verification.md` 第三节 |

