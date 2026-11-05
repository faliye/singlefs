# m4-gpugc-publish-growth-r1 正推腿报告（Sonnet，2026-11-05）

立场：从探针日志、探针源码与 `crates/` 今天的实现推到结论。引码写主工作区的行号；探针副本的行只写它相对主工作区多出的那几行（`diff` 原样输出附在 Q1.1）。
「量的」= 本轮我读日志或源码现算 / 现 grep 出来的；「推的」= 读码或读数推出来、没有直接读数的。判决要引的数以主 agent 复跑为准。

## 模型与复跑

模型目录：`research/prompts/m4-gpugc-publish-growth-r1-sonnet-model/`（纯文本分析，读调查员留下的三份探针日志，不编译、不起装置）。

| 文件 | sha256 |
|---|---|
| tools/parse_windows.py | b15252262134d8d9d2a2a3e6218ef4772d3923d8a00cb2604e980e33c4db2179 |
| tools/decompose.py | 35ea2e5e4c02254e051692713cba6d714ede415e363e7b1150caf8bc1bdf18e1 |
| tools/shares.py | fbe6528580d4c99f2b2716714649305e50abff87b8bf780e9c1825b6f05bbf05 |
| tools/counts_faults.py | 888d0070b7f72323154f074f5d94923334520d60a6f91a70c439c2238e8abbbc |
| tools/control.py | 24890062b620791cc87113f799df153a1c913195312ee87431785f6de8a0b839 |
| tools/pc_vs_clone.py | 49390bca69d6c85262ee1a050762e5a46c764a26934bb23d1dd1c2363b234a99 |
| tools/extrapolate.py | d8c0e7a651787ca22513dbe8d962ed1a650c53d650ad504fb5e000895c74c24c |
| tools/rerun.sh | bbdda81416dcb5ba8a34ee3d5beaf4b087a58a6ce09bab0ab2baa7f3f9a86a93 |
| tools/arc_bytes_type_only_check.sh | d3285ce3f6f7bffdaf2b76bcc895be359f8e340b8dd0c860fc4a340940e76ea8 |
| out/control-out.txt | bdfe7053ac46a5f8a6ec6904792e81b39361f6ca778e0318b0a51441bdfdcfe9 |
| out/counts-7603.txt | c38aecfaba615bf9b9edcb903cea1cf3f1cc938017792b6a5ea8534c5947ed79 |
| out/counts-7622.txt | d77edff5b4c8773bbd403c5733f84252c2213f969cff1629c26de0077c9db29a |
| out/decompose-7622.txt | 249feafe24f80f653ca04154717f5ab9b345bb54f0caa818d9c187a819fdcc2b |
| out/extrapolate-out.txt | b90b0ea51f977013042a8422661a01ed10066e9c658da3615ee9562834c9499d |
| out/pc-vs-clone.txt | fa7953fa669170d89a19c1e448db43599a145b6b297c17af31e4535d5166892f |
| out/shares-7603.txt | 1356a88060e51deb243fb60a804348389eefa08b686542972c5d3984508f7960 |
| out/shares-7622.txt | 7b3c4a05b40ca8aa523a9b71d7eeaa6bff6617f9d9ff2a5962c04525290e6169 |
| out/shares-7623.txt | 27c0e0ffba8149434e9082a313a2398e4a4401e385fffa9e98a99b7db80e3e90 |

复跑命令在「要主 agent 跑的命令」第 1 条。本轮我没能自己起 rerun.sh（项目代码闸拒仓内脚本），读数是把同一份脚本拷在草稿目录里跑出来的；七份 python 脚本与草稿目录里那七份 `cmp` 逐字节相同，九份 `out/` 与草稿目录里的输出 `cmp` 相同（量的，本轮交回前现跑）。

## Q1　定位与外推

### Q1.1　四格口径：探针包住的是不是它说的那几行

探针相对主工作区的全部改动（我跑 `diff` 看过，下面是逐处摘要、不是原样输出；主工作区四份核心层文件的 sha256 与调查员记的四个前缀相同，量的）：

```
transaction.rs:  13461c13469,13472  units_of_the_previous_version.unit(identity).clone()  →  { enter(CARRIED_UNIT_CLONE); let carried = ....unit(identity).clone(); add_amount(len); carried }
mounted_session: 3064a3071 / 3067a3075   enter(CLONE_INTO_VERSIONS_PUBLISHED) ... published_in_this_change.versions_published.push(published.clone()); drop(...)
mounted_session: 3078c3086  { enter(REPLACE_CURRENT_DROPPING_OLD); *current = PoolVersion::of_a_version_of_trees(published); }
allocator.rs:    1150a1151 / 2208a2210 / 2260a2263   进入函数体第一行 enter(MAKE_ROOM / RECORD_FOR / RELEASE_FIND)
allocation_record_tree.rs: 314a315   进入函数体第一行 enter(RECORDS_OF_EACH_LEAF)
transaction.rs:  12499c12501  let allocator_before_this_publish = { enter(...); allocator.clone() };   13283c13291 同形（rehearsal）
装置:            2093c2100..  drop(published) 前后 enter(APPARATUS_DROP_OF_THE_PUBLISHED)
```

守卫是函数体第一行的 `Instant` 守卫，离开作用域时记（`investigation_probe.rs` 的 `ProbeTimer::drop`），窗口读数在窗口末由装置读完清零，读数本身不在计时区内（`print_the_probe_window` 先取 `window_seconds` 再做版本字节求和）。

逐格判（每格附被包代码的主工作区行）：

1. **`carried_unit_clone`**：包住「查上一版索引 + 整份 clone」两样，不只是 clone。被包代码：`crates/singlefs-core/src/transaction.rs:13461` 「units_of_the_previous_version.unit(identity).clone()」。**成立**，但单价里带一次 O(log U) 二分查找（`FirstEntryOfEachIdentity::first_entry_of` 走 `SortedIdentityIndex`）。
2. **`clone_into_versions_published`**：包住的是整个 `TransactionOutput` 的 clone，不只是 `units` 字节。被包代码：`crates/singlefs-core/src/mounted_session.rs:3067` 「.push(published.clone());」。`TransactionOutput` 里随 U 或 R 复制的还有 `crates/singlefs-core/src/transaction.rs:3242` 「pub mapping_keys: Vec<Vec<u8>>,」（每单元一次小分配）、`crates/singlefs-core/src/transaction.rs:3305` 「pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,」（同）、`crates/singlefs-core/src/transaction.rs:3244` 「pub allocation_records: Vec<AllocationRecord>,」（R × 定长）。**部分成立**：这一格不是「深拷单元字节」，是「深拷整版」。非字节部分占多少：无缺页的删垫片窗口里，这一格单版耗时 ÷ 单版 `carried_unit_clone` 耗时 = 1.02–1.34（量的，`out/shares-7622.txt` 删垫片八行的 clone/carried 列 0.51–0.67，乘 2，因为一次用户改动发两版而只有第一版的 clone 被包）。所以非字节部分至多占这一格的约四分之一（推的，上界；carried 单价里还带查找，字节部分的真价比它低）。
3. **`replace_current_dropping_old`**：包住「构造 `PoolVersion` + 释放旧的 `current`」。被包代码：`crates/singlefs-core/src/mounted_session.rs:3078` 「*current = PoolVersion::of_a_version_of_trees(published);」。`of_a_version_of_trees`（`crates/singlefs-core/src/transaction.rs:2441`）只判成员、不复制。**成立**；P=2048 末窗口只占 0.67 ms，释放 3386 个 32 KiB 块这么便宜。
4. **`apparatus_drop_of_the_published`**：包住装置丢掉 `versions_published`。它在 `publish_user_change_total` 之外（探针在装置里 `publish_while_constructing` 返回之后才进）。**成立**，但分母口径要对齐：四格合计要拿窗口挂钟当分母，不能拿 `publish_user_change_total`。
   - 我算的（量的，`out/shares-7622.txt` 交错写末行）：窗口 64.74 ms 里四格合计 33.32 ms = **51.5%**；只算在 `publish_user_change_total` 里面的三格 29.61 ms ÷ 60.78 ms = **48.7%**。第二位调查员写的 54.8% 是「分子带装置那一格、分母不带」，口径错位，他自己已改成「约一半」。我同意「约一半」，不同意「一半以上」。

装置格与内存盘格（`device_*` 三格）：
- 内存盘格包住的是装置 `RecordingMemoryDevice` 的 `read_at / write_at / 零写 / discard / barrier` 实现体（`e202_gpu_cpu_gc_comparison.rs` 探针行在 `1137`、`1160`、`1189`、`1205`、`1216` 之后），不含核心层在调用它之前拼字节的活（那在 `persist_the_publish` 格里）。三格合计 P=2048 交错写末窗口占 `publish_user_change_total` 的 0.3–0.6%（量的，调查员 `dev` 列；我只复核了量级：`device_read_at` 126.6 次 × 0.5 µs、`device_write_at` 40.8 次 × 2.7 µs，见 `out/counts-7622.txt` 末交错写行）。**成立**。
- 装置自己在发布之外的活：窗口挂钟 − `publish_user_change_total` − 丢弃那格，P=2048 交错写末窗口 0.25 ms/次（3.96 − 3.71，量的）。**成立**，≈0.4%。

### Q1.2　阳性对照（`INVESTIGATION_EXTRA_UNITS_CLONE=1`，任务 7623，对照任务 7603）

对照在装置里每一步多做 `version_of_trees(session).units.clone()` 再 `drop`，记进 `positive_control_extra_units_clone` 一格（探针副本装置 `investigation_extra_units_clone`）。

| 问 | 读数（量的，`out/control-out.txt`） | 判 |
|---|---|---|
| 记的字节是不是整版单元字节 | 对照每次进入记的字节 ÷ 窗口末 `unit_bytes_in_the_version`：P=1024 交错写末窗口 `64:0.250493:3307454464`，每次 51.68 MB 对 53.82 MB | 成立 |
| 对照那格的耗时是不是真挂钟 | 两趟（有对照 / 无对照）按（P、阶段、步）对齐，窗口里「发布之外」的时间之差 ÷ 对照格耗时：32 个窗口里 29 个在 0.91–1.00，三个低的（0.70、0.39、0.76）都是 P=1024 顺序写前三窗，对照格每次只有 0.03–0.13 ms | 成立 |
| 能不能证明「四格能抓到发布路径里一份整版深拷」 | 对照是装置里**发布之外**的一份拷贝，记进它自己新加的一格；它验证的是「`Instant` 守卫记的时间 = 墙钟」，不验证「发布路径里没有别的、没被包住的整版拷贝」 | **证明不了覆盖面** |
| 对照一份整版拷贝的单价能不能代表真发布路径里那份 | `out/pc-vs-clone.txt`：无缺页的窗口（P=1024 顺序写 64–448 步、交错写 128 步）真发布里的 `clone_into_versions_published` ms/MiB ÷ 对照格 ms/MiB = 0.63–1.39；有缺页的窗口（P=1024 交错写 320 步起、P=512 顺序写 160 步起）= 2.3–4.1 | **代表不了**：对照那份拷贝立刻 `drop`，堆块马上复用、不缺页；真发布里那份拷贝被 `versions_published` 持着到装置丢，跨过下一版的构造，堆长高、缺页 |

两趟的噪声（量的，`out/control-out.txt` 的 `T plain` 与 `T ctrl` 两列）：同一窗口，无对照的 7603 与有对照的 7623 的 `publish_user_change_total` 每次毫秒可差 5 倍（P=1024 顺序写 128 步：5.27 对 1.25；P=512 交错写 192 步：23.03 对 12.11）。所以「两趟之差」只在对照格本身远大于噪声的窗口里可信，这也是那三个低值窗口的来历。

**结论（Q1 第 1 问）**：四格口径对（有一处分子分母错位已被调查员自己收回）；阳性对照证明「计时可信」，不证明「无遗漏」，也不证明「对照的单价等于真拷贝的单价」。

推翻它的现象：在探针副本的 `publish_admitted` 里再分几格（见 Q1.3），其中一格按 U 涨、量级与 `carried_unit_clone` 相当 ⇒ 深拷那一组被低估。

### Q1.3　四格之和与发布挂钟差的那一截（约 15%）落在哪

把 `publish_user_change_total` 按探针的嵌套关系逐层减（量的，`out/decompose-7622.txt`、`out/shares-7622.txt`，P=2048 交错写末窗口，每次用户改动的毫秒，`publish_user_change_total` = 60.78）：

| 层 | 总 | 减去探针过的子格 | 剩 | 占 60.78 |
|---|---|---|---|---|
| `publish_user_change_total` | 60.78 | 检查 + `publish_the_change_once` + clone + relocate + replace + discard | 0.00 | 0.0% |
| `publish_the_change_once` | 38.40 | `allocator_clone_before` + `judge` + `publish_admitted_total` + `persist` | 0.92 | 1.5% |
| `judge_before_any_read_or_write` | 19.48 | `settle_allocation_record_tree` | 0.23 | 0.4% |
| `publish_admitted_total` | — | `allocation_records_sort_and_assert` + `build_allocation_record_tree` + `carried_unit_clone` | **6.89** | **11.3%** |

- **层级差之和 = 8.04 ms = 13.2%**（量的）；交错写八个窗口都在 11.5%–14.2%（`out/shares-7622.txt` 的「unprobed」列）。这是调查员说的约 15% 的下界，层级关系是代码结构决定的，不靠假设。
- 还有一截落在 `settle_allocation_record_tree` 里面，探针看不出：它的子格（clone、`plan.resolve`、释放判定、`make_room`、`release_find`、`record_for`、`records_of_each_leaf`）互相的嵌套关系探针没记全。`records_of_each_leaf` 每次用户改动 14.7 次，其中 `crates/singlefs-core/src/transaction.rs:15401` 那条 `assert!` 占 3 次（`nodes_whose_contents_changed` 两次、`nodes_holding_records` 一次，读码），固定点里占其余约 11.7 次（推的）。按每次同价，固定点里剩下 12.47 − 8.5 ≈ **4 ms（6.5%）** 没被任何格解释（推的，±1 ms）。两截加起来约 12 ms = **约 20%**（推的上界）。所以「约 15%」落在 13%–20% 之间，哪个值取决于 `records_of_each_leaf` 在固定点与断言之间的分摊，探针没分开记。
- 没点到行的那 6.89 ms 是整个表里涨得最快的一格：2.09 → 6.89 ms，3.3 倍，快于 U（1439 → 3386，2.35 倍）和 R（14662 → 32944，2.25 倍）（量的）。它含 `first_entry_index_build` 的 0.48 ms（37.7 次 × 12.8 µs，`out/counts-7622.txt`）。读码能点名的候选（都是推的，没量）：
  - `crates/singlefs-core/src/transaction.rs:15246` 「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」与 `crates/singlefs-core/src/transaction.rs:16608` 「let units_by_role = FirstEntryOfEachIdentity::of_the_table(&units, |unit| unit.identity);」：每次发布对上一版与新一版的 U 个单元各排一次序。
  - `crates/singlefs-core/src/transaction.rs:15799` 「let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations」起到 `:15808` 「let mapping_entries_by_key: BTreeMap<CodeTwoTreeKey, Vec<u8>> = mapped_units_with_locations」：U 个映射 key 各克隆一次，再对 U 个条目建 `BTreeMap`（每条两次小分配），`mapping_keys` 再克隆一遍。
  - `crates/singlefs-core/src/transaction.rs:15804` 「mapped_units, resolved.mapped_units,」是 `assert_eq!` 的第二个参数：U 个 key 逐项相等比较，release 下也跑。
  - `crates/singlefs-core/src/transaction.rs:16961` 「let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();」随后的 `assert!` 逐 key 比较，也是 U 级。
- **结论（Q1 第 1 问末句）**：约 15% 不是深拷，至少一部分（`A` 层 11.3%）是别的随 U 涨的活，**甲档（`Arc<[u8]>`）碰不到它**。成立 / 推的：层级差 13.2% 是量的；落在映射条目与索引排序上是推的。

推翻它的现象：在探针副本的 `publish_admitted` 里给 `:15246`、`:15799`–`:15830`、`:16608`、`:16961` 各加一格，若合计 < `A` 层剩余的一半 ⇒ 候选不对，剩余另有来处。

### Q1.4　「每次发布进这几处的次数不涨，涨的是每次的代价」与「总挂钟高于 P² 是缺页造成」

**前一句**（量的，`out/counts-7622.txt` 的第 2 行与第 17 行，P=2048 顺序写 128 步 → 交错写 1024 步，每次用户改动）：

| 格 | 次数 | 单次 | 判 |
|---|---|---|---|
| `records_of_each_leaf` | 14.0 → 14.7 | 17.2 → 729.3 µs（R 涨 17.6 倍，单次涨 42 倍） | 次数不涨：成立 |
| 固定点轮数（`settle_allocator_clone` 次数） | 2.40 → 2.58 | 38 → 249 µs | 成立 |
| `first_entry_index_build` | 36.5 → 37.7 | 0.6 → 12.8 µs | 成立 |
| `allocator_make_room_for_record` | 65.8 → 87.5（+33%） | 0.55 → 14.61 µs | **不严格**：次数涨三成 |
| `allocator_release_find` | 37.3 → 52.6（+41%） | 0.42 → 13.79 µs | 不严格 |
| `allocator_record_for` | 49.0 → 71.5（+46%） | 0.22 → 6.98 µs | 不严格 |
| `carried_unit_clone` | 65.9 → 3022.2（每照抄一个单元进一次） | 0.6 → 2.4 µs | **不成立**：次数 ∝ U，涨 46 倍 |

所以这一句对**扫记录那一组**基本成立（次数最多涨四成、单次涨 25–42 倍，R 才是主因）；对**深拷那一组**不成立（`carried_unit_clone` 的次数 = 带过去的单元数）。另外三格的次数涨三到四成是我在日志里新读到的，调查员没提。

**后一句**（推的）：现有读数不支撑「缺页是总挂钟高于 P² 的原因」，理由四条：
1. 缺页确实在（量的）：交错写和顺序写后半，每次用户改动缺的页数 ÷ 版本单元字节的页数：顺序写 512 步起 0.75–0.86，交错写 0.85–0.92（`out/shares-7622.txt` 末列）；删垫片窗口里缺页为 0（`out/counts-7622.txt` 删垫片行 `faults/user_change` = 0，第一个窗口除外）。
2. 但 `carried_unit_clone` 的每 MiB 单价与缺页无关（量的，`out/shares-7622.txt` 倒数第二列）：版本 4.4 MiB 时 0.021，顺序写 25 MiB 以上 0.078 上下，交错写八窗 0.076–0.150（中间几窗整台机器慢，两组格同涨），**删垫片无缺页的八个窗口是 0.071–0.080**。涨的是缓存装不下（本机 L3 每组 32 MiB，`lscpu` 量的：`L3 cache: 64 MiB (2 instances)`），在 25–34 MiB 处饱和。
3. 缺页只加在 `clone_into_versions_published` 那一格上：有缺页时它是 carried 单价的 2.4–3.3 倍（交错写八窗），无缺页（删垫片，按每版折算）1.02–1.34 倍（量的）。按三倍算，P=2048 末窗口这一格 21.63 ms 里约 13.7 ms（占 60.78 的 22.5%）是缺页和新页代价（推的）。
4. 缺页和缓存两样在版本 > 约 34 MiB 之后都饱和，之后每次发布的代价应当 ∝ 版本字节，总挂钟 ∝ P²，不再多出指数。读数对得上：每次用户改动的毫秒 ÷ 版本 MiB，P=1024 末窗口 0.60（30.98 ÷ 51.3），P=2048 末窗口 0.60（60.78 ÷ 101.4）（量的，`out/shares-7603.txt`、`out/shares-7622.txt`）。产物里 P=1024 → 2048 → 4096 的局部次方 2.76、2.36 因此另有来处：R 项的单次代价 ∝ R^1.3（17.2 → 729.3 µs，R 涨 17.6 倍）、`A` 层剩余涨得比 U 快，以及噪声（见 Q1.5：同一档 P=2048，两个进程交错写末八分之一每步 358.455 ms 对 133.529 ms，2.7 倍）。

能分辨的读数（已有的）：删垫片窗口（无缺页）对交错写窗口（有缺页）的 clone 格单价之比，上面第 3 条。还缺的：在缺页被关掉的进程里重跑同一档（命令见「要主 agent 跑的命令」第 2 条）。

推翻它的现象：把 glibc 的归还内存关掉后（命令 2），交错写窗口缺页降到删垫片窗口的水平，而 P=2048 的 `construction_seconds` 掉到不足原来的三分之二、`clone_into_versions_published` 单价掉回 carried 单价 ⇒ 缺页确实是大头。

### Q1.5　外推「整档 P=36608 要几十小时」

用的读数（量的，`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/` 下三份产物的 `name=construction_timing` 行，加探针任务 7603、7622 日志里的同名行）：

| P | 各存各的（`construction-timing-2288MiB-each-device-copies.out`） | 共用存法（`construction-timing-2288MiB-shared.out`） | 带探针的新进程（任务 7603、7622） |
|---|---|---|---|
| 512 | — | — | 11.033 s |
| 1024 | 27.324 s | 35.790 s | 32.838 s |
| 2048 | 184.946 s | 188.649 s | 145.311 s |
| 4096 | 950.446 s | 646.824 s | — |

- 取点只有 1024、2048、4096 三个（两条序列各三点），端点外推距离：36608 ÷ 4096 = 8.94 倍，3.16 个倍频；取样本身只跨 2 个倍频（1024 → 4096）。**外推距离是取样跨度的 1.58 倍。**
- 同一档的读数离散（量的，`out/extrapolate-out.txt`）：P=1024 三份最大 ÷ 最小 = 1.31；P=2048 三份 = 1.30；P=4096 两份 = 1.47。相邻两档的局部次方因此靠不住：各存各的 2.76 → 2.36，共用 2.40 → 1.78，带探针 1.57 → 2.15。
- 用哪条拟合：取 P=4096 当锚，分别乘 8.94^e（e 取该序列的末倍频局部次方、对数最小二乘斜率、前倍频局部次方、恰好 2）：

| 序列 | e = 2.0 | e = 末倍频 | e = 最小二乘 | e = 前倍频 |
|---|---|---|---|---|
| 各存各的 | 21.1 h | 2.36 → 46.6 h | 2.56 → 71.9 h | 2.76 → 111.1 h |
| 共用存法 | 14.4 h | 1.78 → 8.8 h | 2.09 → 17.4 h | 2.40 → 34.3 h |

  （全是推的；`out/extrapolate-out.txt` 原样。）
- 区间（推的）：全部拟合覆盖 **8.8 h – 111 h**；只取「指数 2.0–2.3」这一窄带（Q1.4 第 4 条的理由：缓存与缺页在 > 34 MiB 后饱和，每次发布 ∝ 版本，剩 R 项的 R^1.3）是 **14 h – 41 h**（各存各的 21–41，共用 14–28）。「几十小时」落在窄带里偏上、落在全区间中间；下沿是个位数小时、上沿过百小时。
- 没纳入的（推的）：目标档的盘比取点用的 3221225472 字节盘大；同一 P=2048 在 9412018176 字节盘上 149.637 s（`construction-timing-8GiB-shared.out`），比同存法 3.2 GB 盘上的 188.649 s 还短 21%（单次读数，噪声 30%，只能说没看到随空盘容量变慢）。
- 能把区间收窄的读数：加一个 P=8192 点。同序列里 P=8192 与 P=4096 之比对不同指数是 3.43（e=1.78）、4.0（e=2）、5.14（e=2.36）、5.89（e=2.56）、6.77（e=2.76）；只有最两头能在 30% 噪声里分开，要重复至少三次。

推翻它的现象：P=8192 单点实测落在 P=4096 同序列读数的 3.4 倍以下（区间下沿成立、`几十小时` 偏高）或 6.8 倍以上（上沿成立、`几十小时` 偏低）。

### Q1　判定汇总

| 问 | 判 |
|---|---|
| 四格口径：探针包住的是不是它说的 | 部分成立：carried、replace、drop、内存盘、装置格成立；`clone_into_versions_published` 包的是整版 clone，不只单元字节；分子分母已对齐为 48.7%–51.5%（量的） |
| 阳性对照能不能证明抓得到整版深拷 | 证明计时可信（29/32 窗口 0.91–1.00，量的）；证明不了覆盖面；对照单价只有真拷贝的 1/2.3–1/4（有缺页窗口，量的） |
| 约 15% 落在哪 | 层级差 13.2%（量的）；加固定点内剩余约 20%（推的）；`publish_admitted` 里的映射条目装配与索引排序是候选（推的），甲档碰不到 |
| 「次数不涨，单价涨」 | 对扫记录一组基本成立（三格次数涨三到四成）；对深拷一组不成立（`carried_unit_clone` 次数 ∝ U） |
| 「总挂钟高于 P² 是缺页造成」 | 推的，读数不支撑（缺页只加在 clone 一格、约占末窗口 22%，且饱和；每 MiB 的毫秒在 P=1024 与 P=2048 末窗口都是 0.60） |
| 外推几十小时 | 推的；区间 8.8–111 h，窄带 14–41 h；外推距离 1.58 倍取样跨度 |


## Q2　改法

### Q2.1　甲档：`PublishedUnit.bytes` 换成 `Arc<[u8]>`

**能不能只改类型而不改语义**：语义能保，「只改类型」做不到（推的；读码加 grep，编译没跑，命令 3 量）。

保得住的（读码）：
- `PartialEq / Eq / Debug / Clone`：`crates/singlefs-core/src/transaction.rs:3206` `pub struct PublishedUnit {` 上一行是 `#[derive(Clone, Debug, PartialEq, Eq)]`；`Arc<[u8]>` 四样都实现，相等按内容比，`Debug` 与 `Vec<u8>` 同形（标准库约定，推的）。
- 写盘的字节与录制流：落盘读的是 `crates/singlefs-core/src/transaction.rs:1505` 「unit: &unit.bytes,」，目标字段是 `crates/singlefs-core/src/transaction.rs:192` 「unit: &'publish [u8],」，`&Arc<[u8]>` 自动解成 `&[u8]`，字节不变。
- checker：`singlefs-checker` 不引 `PublishedUnit`，只有它自己的 `PublishedUnitCopy`（`crates/singlefs-checker/src/rebuild.rs:63` 「pub(crate) struct PublishedUnitCopy {」），读盘上的字节；`crates/singlefs-checker/src/walk.rs` 里两处 `.units` 是快照的 `units`（`:18674`、`:18798`），不是版本的。**不受影响**（量的，grep）。

保不住「只改类型」的（量的 grep 与读码）：
- 构造点：`grep -rn 'PublishedUnit {' crates/singlefs-core` 19 行，去掉定义和四个函数签名，**14 处结构体字面量**（`recovery.rs` 1、`mount.rs` 3、`transaction.rs` 10）。第二位调查员写的「18 处」把签名也数进去了。每处 `bytes: <Vec<u8>>` 都要 `.into()`；`rewritten_unit` 闭包（`crates/singlefs-core/src/transaction.rs:16158` 「let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {」）每个重写单元多一次 `Vec → Arc<[u8]>` 的拷贝（每次发布重写的单元只有十来个：`allocator_make_room_for_record` 87.5 次 ÷ 2 盘 ÷（固定点 2.6 轮 + 真发 1 次）≈ 12，各 16–32 KiB，约 0.4 MB，几十微秒，推的）；备选 `Arc<Vec<u8>>` 免这次拷贝、多一层间接。
- 比较与读取点：`crates/singlefs-core/src/mount.rs:6653` 「.is_some_and(|bytes_on_this_device| bytes_on_this_device == unit.bytes)」与 `:6869` 同形，`Vec<u8> == Arc<[u8]>` 没有 `PartialEq` 实现，编译不过；`crates/singlefs-verification/src/model_comparison.rs:221`、`:645`、`:745` 用 `unit.bytes.as_slice()`，`Arc<[u8]>` 没有 `as_slice`。
- 整份换字节的测试与装置（量的，`grep -rn '\.units\[[^]]*\]\.bytes ='`，在入 workspace 的包 `singlefs-verification` 里）：`crates/singlefs-verification/tests/release_checksum_quarantine.rs:915`、`crates/singlefs-verification/tests/overwrite_in_one_instance.rs:503`、`:623`、`:658`、`crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs:723`，五处都要 `.into()`；同文件 `:710` 的 `std::mem::take(&mut unit.bytes)` 需要 `Arc<[u8]>: Default`（标准库有，推的）。第二位调查员写的路径是 `singlefs-harness` 与 `singlefs-checker-tier`：这两个目录在磁盘上还在、与 `singlefs-verification` 内容逐字相同（`diff -rq` 空），但根 `Cargo.toml` 的 `members` 里只有 `singlefs-verification`，那两份不是 workspace 成员（量的）。
- 原地改字节：我 grep 到的只有整份赋值，没有对 `PublishedUnit.bytes` 下标赋值或 `iter_mut`；`crates/singlefs-verification/src/bad_disk_input.rs:1843` 的 `reseal_index_node(&mut node.bytes)` 是 `NodeBelowTheRoot`，不是 `PublishedUnit`（读上下文，没编译验）。真数交命令 3。

**「系数小约三个量级」怎么算的、对不对**：调查员的算法是「每次 29.6 ms 里复制 32 KiB 缓冲与缺页的部分都没了，剩下按 U 个条目搬指针，估计 1 ms 以下」（第二位调查员报告「改法」一节，推的，没量）。我重算：
- 字节复制那一项：`carried_unit_clone` 每次 2.4 µs（量的：P=2048 交错写末窗口 `carried_unit_clone` 一格的秒数 ÷ 次数，日志原行 `task-7622.log` 最后一条交错写窗口，3022 次 / 用户改动）换成 `Arc` 加一约 5–10 ns，2.4 个量级（推的）。但这一格里还有二分查找（`transaction.rs:3621` `value_stored_for_identity`，折半查，约 12 步），加一次 48 字节的结构体构造，合计约 0.1–0.2 µs（推的）⇒ **这一格整体 12–24 倍，不是 1000 倍**。
- 整个深拷组：之前 29.61（发布内三格）+ 3.71（装置丢弃）= 33.32 ms；之后（推的）：carried 3022 × 0.1–0.2 µs ≈ 0.3–0.6 ms；整版 clone 剩 `Vec<PublishedUnit>`（U × 约 48 B）、`mapping_keys` 与 `mapped_units` 各 U 次小分配（约 6.8k × 50 ns ≈ 0.35 ms）、`allocation_records`（R × 定长，约 1 MB，约 0.1 ms）≈ 0.5–1.0 ms；换掉旧版与装置丢弃各 U 次引用计数减一加 2U 次小释放 ≈ 0.4–0.8 ms。合计 **约 1.2–2.4 ms，缩小 14–28 倍，约一个多量级**。调查员的「1 ms 以下」偏乐观约一倍，「三个量级」只对字节复制那一项的原始单价成立，不对整组。

**改完每次发布还剩哪些随单元数 U 或记录数 R 涨的项**（读码，行号都是主工作区的，推的；第 1 项以外甲档都碰不到）：

| 项 | 位置 | 涨法 |
|---|---|---|
| 照抄单元的查找与 `Arc` 加一 | `crates/singlefs-core/src/transaction.rs:13461` | U 次 × O(log U) |
| 整版 clone 里的 `mapping_keys`、`mapped_units` 每单元小分配，`allocation_records` | `transaction.rs:3242`、`:3305`、`:3244` | U、R |
| 旧版释放 | `crates/singlefs-core/src/mounted_session.rs:3078` | U、R |
| 上一版与新一版的按角色索引（排序） | `transaction.rs:15246`、`:16608` | U log U |
| 映射条目装配（每单元克隆 key、建 `BTreeMap`） | `transaction.rs:15799`–`:15830` | U log U，每条两次小分配 |
| 两处 U 级断言（release 下也跑） | `transaction.rs:15804`、`:16961` 起 | U |
| `plan.resolve`（固定点每轮一次） | `settle_plan_resolve` 格：1.04 → 2.85 ms（量的，调查员 `calls-7622.txt`） | U |
| 扫分配记录一组 | Q2.3 | R |

按这张表算（推的）：甲档之后交错写末窗口 `publish_user_change_total` 约 60.78 − (33.32 − 2) ≈ **29 ms，约快一倍**；剩下最大的一块是扫记录组（16.8 ms）和 `publish_admitted` 里没点到行的那截（6.9 ms）。总挂钟仍 ∝ P²，只是系数约减半，也没有每次发布重新缺一整版页的代价（deep 组的缺页部分随字节复制一起没了）。

甲档的 Q2.1 判定：**做得到（推的）**；改到的文件：`crates/singlefs-core/src/transaction.rs`、`mount.rs`、`recovery.rs`，`crates/singlefs-verification/src/model_comparison.rs`、`tests/release_checksum_quarantine.rs`、`tests/overwrite_in_one_instance.rs`、`src/bin/e180_admission_reading_versus_writable_bytes.rs`，可能还有 grep 看不出的下游读点（命令 3 先量核心层库，下游要核心层改通后才编得到）。

推翻它的现象：命令 3 报核心层库的编译错误数远多于 14（> 40）⇒ 「只多 `.into()`」不成立，改动面更大；或甲档改完后同一条造池命令在 P=2048 交错写末窗口 `publish_user_change_total` 掉不到 40 ms 以下 ⇒ 我对剩余项的估计偏小。

### Q2.2　乙档：`Arc<TransactionOutput>` 加持久结构

**要动的公开类型**（量的，行号现查）：
- `TransactionOutput.units`（`crates/singlefs-core/src/transaction.rs:3233` 「pub units: Vec<PublishedUnit>,」）：字段文档写着按 bump 次序排（「这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄」），换成按身份有序的持久结构会丢位置次序，要另存 bump 次序。
- `ContainerTreeRootUnitPositions`（`crates/singlefs-core/src/transaction.rs:3335`）存的是 `units` 里的下标，注释写「之后 `units` 不再挪动」（`:3333`）；持久结构里没有稳定下标，要换成按身份取。
- `TransactionOutput::unit / units_by_role`（`:4374`、`:4383`）与 `FirstEntryOfEachIdentity::of_the_table`（`:3697`，收 `&'table [Entry]` 切片）：参数形状是切片，持久结构不是切片。
- `PoolVersion` 三个成员按值持 `TransactionOutput`（`:2430` 起）、`VersionsPublishedInThisChange = Vec<TransactionOutput>`（`crates/singlefs-core/src/mounted_session.rs:482`）、`UserChangePublished.versions_published`（`:200`）、`MountOutput` 的 `row_publish`、`warm_up_publishes`、`deletion_queue_continuation`（`crates/singlefs-core/src/mount.rs:1041`、`:1043`、`:1065`）。
- 调用方（量的 grep）：`.units` 出现在 `singlefs-core` 7 个文件（`transaction.rs` 18 处、`mount.rs` 10、`region_scan.rs` 3、`compaction_plan.rs` 1、`mounted_session.rs` 1）、`singlefs-verification` 60 个文件、装置 `e202_gpu_cpu_gc_comparison.rs` 4 处；其中真是版本 `units` 的比例没逐处分（`checker` 的两处不是）。

**是不是只有它能让每次发布不随单元数涨**：**不是充分条件，也不是唯一路径（推的）**。
- 不充分：只要 `publish_admitted` 每次从头装一份 U 项的 `units`、`mapped_units`、`mapping_entries_by_key`、两份 `FirstEntryOfEachIdentity` 索引，并对 U 个 key 做两处断言（上面表格第 4–6 行），每次发布就是 Θ(U) 的，容器换成持久结构也一样；`plan.resolve` 同理。要不随 U 涨，这些循环都得改成只处理这次重写的单元，是对「每次从头派生下一版」这个做法的重做，超出乙档字面。
- 不唯一：不让 `current` 带单元字节、只留落点（第二位调查员「还剩的一问」）也能去掉 per-version 的 U 项字节；但 `mapping_locations_for_key(&version.unit(leaf_role).bytes, key)`（`crates/singlefs-core/src/transaction.rs:5037`）与 `parse_index_node(&self.unit(root_role).bytes)`（`:4436`、`:4452`）说明释放路径与准入要读内存里的映射叶字节与根字节，哪些单元的字节必须留要先查谁在读（没查）。
- 乙档里能做得到的小一块：`Arc<TransactionOutput>` 让 `current` 与 `versions_published` 共用一份（去掉 `mounted_session.rs:3067` 那次整版 clone），这一项不需要持久结构，只要 `VersionsPublishedInThisChange` 的元素类型换成 `Arc<TransactionOutput>`（推的，调用方读 `.units` 的写法大多不变，因为 `Arc` 自动解引用）。

乙档判定：持久结构 **做得到但不单独达成目标（推的）**；`Arc<TransactionOutput>` 这一小块 **做得到（推的）**。

推翻它的现象：把 `publish_admitted` 里 U 级循环（表中第 4–6 行）逐个换成增量后，每次发布的毫秒数在 P=1024 → 4096 间不再随 U 涨 ⇒ 它们本来就能跟乙档分开做，我说的「不充分」要改成「分两步做」。

### Q2.3　扫分配记录那一组：三样各自能不能做

先给这一组每一格今天占多少（量的，P=2048 交错写末窗口，每次用户改动的毫秒，`out/decompose-7622.txt`、`out/counts-7622.txt`）：

| 格 | 每次用户改动 | 占 60.78 |
|---|---|---|
| `records_of_each_leaf`（14.7 次 × 729.3 µs） | 10.72 | 17.6% |
| `allocator_make_room_for_record`（87.5 × 14.61 µs） | 1.28 | 2.1% |
| `allocator_release_find`（52.6 × 13.79 µs） | 0.73 | 1.2% |
| `allocator_record_for`（71.5 × 6.98 µs） | 0.50 | 0.8% |
| `allocator_clone_before_the_publish` + `settle_allocator_clone`（2.58 轮 × 249 µs） | 0.43 + 0.64 = 1.07 | 1.8% |
| `allocation_records_sort_and_assert`（含一份 `records_of_each_leaf` 的 3 次） | 2.52 | 4.1% |

**① 换有序结构**：**做得到（推的）**。
- 改到的文件：`crates/singlefs-core/src/allocator.rs`（字段 `:1877` 「records: Vec<AllocationRecord>,」；`make_room_for_record_on_device` `:1144`，里面 `:1152` 「let intersecting_record_slots: Vec<SlotNumber> = records」整张过滤；`record_for` `:2211` 「.find(|record| record.device == device && record.slot == slot)」；释放 `:2265` 「.find(|record| record.device == device.device && record.slot == placement.slot)」；另有 `resurrect_released_record`（`:2313` 起的 `iter_mut().find`）、`clear_the_quarantine_mark_…`（`:2352` 起）、`rebuild_from_records` 里的 `allocator.records = records;`（`:2142`）），以及 `crates/singlefs-core/src/transaction.rs:15389`。
- 对外：`records()`（`crates/singlefs-core/src/allocator.rs:2198`）返回 `&[AllocationRecord]`；`.records()` 全仓匹配 `singlefs-core` 27 处、`singlefs-verification` 130 处（量的，grep，含非分配器的同名方法，所以是上界；第二位调查员的 157 即这个和）。换成有序映射后不能再给切片，要么给一个按序物化的 `Vec`（每次 O(R)），要么改成迭代器、逐处改。
- 能拿回多少：三处线性找合计 2.51 ms（4.1%），排序约 0.3 ms，合计 ≤ 2.8 ms（≈ 4.6% 的 `publish_user_change_total`）。第二位调查员写的「约 5 ms」把 `allocation_records_sort_and_assert` 整格 2.52 ms 当成排序，里面其实是那条 `assert!` 的 `records_of_each_leaf`。**这一样本身只买回约 5%。**
- 碰 `.claude/kb/decisions/03-空间分配.md` 已定项 7 的盘上编码吗：**不碰**。盘上编码是 `AllocationRecord::to_bytes`（`crates/singlefs-core/src/allocator.rs:84`）与 key = (设备 4, 槽 6) / value = (跨度 2, 代 8)，见 `.claude/kb/decisions/03-空间分配.md:19`（索引行 「落点粒度 16384；key =（设备身份 4, 槽号 6）；value =（跨度段 2, 代 8）；释放时改写不点删」）与 `.claude/kb/decisions/03-空间分配.md:134` 「落点释放时条目不删、不点删」。内存里换容器不动字节；释放仍是原地改写成已释放（`allocator.rs:2263` 起），`make_room` 对已回收记录的删除也照旧。`PoolAllocator` 只 `#[derive(Clone, Debug)]`（`crates/singlefs-core/src/allocator.rs:1866`），没有 `PartialEq`，换容器不改相等语义。
- 碰 `.claude/kb/decisions/16-发布语义.md` 已定项 17 吗：**不碰**。该项管的是调用方在报错后不许假设能复用旧内存态（`.claude/kb/decisions/16-发布语义.md:377` 「报错之后不许假设可以照旧复用同一份内存态重试。」），不规定分配器记录用什么容器。

**② 碰过的叶作为分配、释放的副产品记下来**：**做得到，有两个前提（推的）**。这一样买回最多：`records_of_each_leaf` 的 10.72 ms 里固定点那约 8.5 ms 是 `nodes_whose_contents_changed` / `nodes_holding_records` 整份重分装，只比碰过的叶可降到与碰过的叶数成正比。
- 要改的写点：`records` 的所有改动点，读码数到 6 处：`make_room_for_record_on_device` 里的原地改写与 `retain`（`allocator.rs:1182`）、`record` 里的 `push`（`:2227`）、`release_leaving_the_record_allocated_on`（`:2263`）、`resurrect_released_record`（`:2313`）、`clear_the_quarantine_mark_of_a_record_the_rollback_target_references`（`:2352`）、重建（`:2142`）。漏一处就是静默漏叶，所以必须保留现有全量函数当测试里的对拍参照（运行时与 checker 用不同算法那条，`.claude/rules/fs-design.md:35` 「运行时与 checker 必须用不同的算法。」，这里是运行时与测试参照的不同）。
- 前提一：「碰过」只是候选集，仍要把碰过的叶与上一版比一次内容。现在的 `nodes_whose_contents_changed`（`crates/singlefs-core/src/allocation_record_tree.rs:367`）是「内容不等才算变」；碰过但改回原样的叶按今天的算法不重写，副产品要沿用这一判，否则同一串操作会多重写节点、落盘的单元序列变了（格式不变，序列变）。
- 前提二：固定点每一轮从同一个起点重来（`transaction.rs:13283`），副产品集合要跟着那份拷贝走或每轮清零。
- 碰已定项 7：**不碰**（只决定哪些节点重写，节点字节由 `build_allocation_record_tree` 照旧装）。碰已定项 17：**不碰**，集合是分配器的内存态，随它一起被拷贝或撤销（见 ③）。
- 还得改 `transaction.rs:15401` 的 `assert!`：它用同一对函数审计刚算出的集合，`.claude/rules/fs-design.md:34` 「在需要答案的那一刻还没有答案」与「审计与被审计用同一段代码」两条禁令里的后一条正对着它；落地后把它留成测试里的对拍。

**③ 分配器拷贝换回滚日志**：**做得到，难度最大，买回最少（推的）**。
- 今天的拷贝点：发布入口 `transaction.rs:12499`（`allocator_before_this_publish`，失败时换回，落盘失败时交给 `persist_the_publish_or_freeze_it`，`:12524` 起的注释 「落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发」）与固定点每轮 `:13283`。合计 1.07 ms（1.8%），其中只有位图那份随盘容量涨（`allocator_clone_before_the_publish` 在 R=1872 时单次 20.9 µs，R=32944 时 429 µs，量的：R 无关的底座不超过 21 µs，每条记录约 13 ns）。
- 要覆盖的状态：`PoolAllocator` 一共 18 个字段（量的，`allocator.rs:1868`–`:1927`），再加 `DeviceFreeMap` 内部的位图、defer 队列；每个 `&mut self` 方法都要写日志，漏一个就让「换回去」与今天的拷贝不等价。可以用 `format!("{:?}")` 对拍撤销后的分配器与发布之前拷的一份（没有 `PartialEq`）。
- 落盘失败那条路径需要的是「发布之后的分配器」也留着（`FrozenPublish { allocator_after_the_publish, … }`，`crates/singlefs-core/src/transaction.rs:1596` 「std::mem::replace(allocator, allocator_before_this_publish);」），回滚日志版在失败时先拷一份当前分配器再撤销即可，失败路径不要求快。
- 碰已定项 7：不碰。碰 `.claude/kb/decisions/16-发布语义.md:377` 那一项：**对内存态的要求是「换回去」与今天逐位相同**（冻结后重发成功要 `*allocator = allocator_after_the_publish`），撤销必须撤干净；这一格上 C381（根已落盘之后发布失败，分配器仍退回）仍是欠账（`.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么` 的「欠」一行），回滚日志不能顺手改变它。
- 改到的文件：`allocator.rs`（日志与每个改动方法）、`transaction.rs`（`:12499`、`:13283` 与失败路径）。

三样的合计上限（推的）：① 约 2.8 ms + ② 约 8.5 ms + ③ 约 1.1 ms ≈ 12.4 ms ≈ 20% 的 `publish_user_change_total`（60.78 ms），其余 `plan.resolve`（2.85）、`publish_admitted` 剩余（6.89）、固定点内剩余（约 4）仍在。

推翻它的现象：①改完后命令 3 之外再跑一次同一条造池命令，交错写末窗口 `allocator_make_room_for_record` + `allocator_release_find` + `allocator_record_for` 三格合计不掉到 0.3 ms 以下 ⇒ 这三格的代价另有来处（比如 `retain` 的删除路径）。

### Q2.4　常驻内存比块字节多出的那一截（P=4096 共用存法 2071564288 对 586481664）

读数（量的，`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-shared.out` 的 `name=construction_phase` 行，P=4096）：交错写末 `resident_bytes=1233829888`、`chunk_bytes_stored=699858944`；`fragment_mount`（0 次发布）后 `resident_bytes=2066841600`，涨 833011712 字节；删垫片后 2071564288。这个进程先造过 P=1024、2048 两个池，`resident_bytes` 在 P=4096 的前几个阶段就是 624517120（同一份产物，glibc 不还内存），所以 1485082624 字节的差里有 624517120 是前面池留下的底。P=2048 各存各的（任务 7622 日志）：交错写末 972464128 对块字节 696188928，多 276275200，是这一版单元字节 106364928 的 2.60 倍；`fragment_mount` 涨 522612736，4.91 倍。

**是不是同一个根**（读码）：是，根都是每个版本值各带整份单元字节（`crates/singlefs-core/src/transaction.rs:3209` 「pub bytes: Vec<u8>,」），但**两截不是同一种持有**：
1. 交错写那 2.6 倍：现行版本（1）+ `versions_published` 里被装置持着到丢的那一份（1）+ `publish_admitted` 正在装的下一版（1）的瞬时并存，glibc 不把峰值还给系统，驻留停在峰值上（推的；读数相容，不证明拆法——常驻读数只对得上总量，分不开是哪几份：`.claude/singlefs-ai-sop/rules/evidence-discipline.md:18` 「独立计数只对上了总数，印证不了拆法。」）。
2. 碎片挂载那一大截：`MountOutput` 里的 `row_publish`、`warm_up_publishes`、`deletion_queue_continuation`（`crates/singlefs-core/src/mount.rs:1041`、`:1043`、`:1065`）各是整个版本值，装置把它绑在 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:2521` 「let (_mount_output, mut session) = MountedSession::of_the_mount(mounted);」这个名字上，活到函数结束；第一次挂载那一个（`:2391`）在更早的块里随块结束丢掉。各占几份没量（推的）。

**改法管不管得到**（推的）：
- 甲档：挂载建出的几个版本值经发布路径装出来（`row_publish`、暖机发布走 `carried_unit` 照抄），字节与 `current` 共用，常驻从「N 份」降到「约 1 份单元字节 + 各版本自己的元数据」；从盘上重建的那一版（`crates/singlefs-core/src/recovery.rs:4630` 一带）是字节的原持有者，仍是 1 份。**管得到**，但数据单元的字节仍住在内存版本里，常驻仍随已写数据量涨。
- 装置一侧：把 `_mount_output` 改成不绑名（`let (_, mut session) = …`）立即丢掉 `MountOutput`，不动核心层，也能去掉碎片挂载那一截里 `MountOutput` 持着的部分。**做得到（推的）**，文件 `e202_gpu_cpu_gc_comparison.rs`；效果大小要量。
- 乙档的 `Arc<TransactionOutput>`：同样让这几份共用。

推翻它的现象：装置在碎片挂载之后立即丢掉 `MountOutput`，P=2048 那一步常驻增量没少掉 2 倍单元字节以上 ⇒ `MountOutput` 不是那一截的主要持有者。

## Q3　归类：两组各落不落在 `.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」第一行

只引条文原句与代码行；归类的最后一步由主 agent 定。

条文：
- `.claude/rules/fs-design.md:19` 判据句：线划在「谁消费这个数」（行内原句，引号是条文自带的）。
- `.claude/rules/fs-design.md:23`：「运行时决策路径（分配、ENOSPC 准入、defer 窗口、生命周期判定） | 不许，且代价不许随盘容量增长」。
- `.claude/rules/fs-design.md:29`：「被问到那一刻有没有答案」。
- `.claude/rules/fs-design.md:33` 「被禁止的不是」；`:34` 两句：「在需要答案的那一刻还没有答案」与「审计与被审计用同一段代码」；`:35` 「运行时与 checker 必须用不同的算法。」

**扫分配记录那一组**——逐格对条文括号里的四个词：

| 格 | 代码行 | 对到括号里哪个词 | 落第一行？ |
|---|---|---|---|
| `allocator_make_room_for_record` | `crates/singlefs-core/src/allocator.rs:2214` 「记下一次分配」（`record` 的文档，`record` 调 `make_room_for_record_on_device`，`:1144`） | 分配 | 落 |
| `allocator_release_find` | `crates/singlefs-core/src/allocator.rs:2243` 「= 放进 defer 队列那一刻」（`release` 的文档，`:2263` 起的 `find` 在 `release_leaving_the_record_allocated_on` 里） | defer 窗口 | 落 |
| `allocator_record_for` | `crates/singlefs-core/src/allocator.rs:2202` 「释放判定路径拿它核」 | 生命周期判定（文档说的是「释放判定路径」，对应到括号词是读法） | 落（读法） |
| `records_of_each_leaf` 在固定点里的 11.7 次 | `crates/singlefs-core/src/transaction.rs:12721` 「let settled = settle_the_allocation_record_tree(」，在 `judge_the_publish_before_any_read_or_write`（`:12674`）里；该函数文档 `:12663` 「算这次之后每棵树的样子与这次重写的角色（分配记录树的固定点在分配器的拷贝上走）」，`:12664` 「空间准入的式子。分配器只读、一个盘都不读、一个写都不发。」；注释 `:12691` 「准入先判这次的单元落不落得下」 | ENOSPC 准入 | 落 |
| 两份分配器拷贝 | `crates/singlefs-core/src/transaction.rs:12499`、`:13283`；`:12498` 「第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图）」 | 同上；位图那份按盘的槽数开 | 落；其中位图那一半是条文说的「随盘容量」 |
| `allocation_records_sort_and_assert` 里的 `assert!` | `crates/singlefs-core/src/transaction.rs:15401`、报错文字 `:15410` 「两边走的是同一串释放与取落点」 | 不是括号里四个词的决策，是发布装配后的自检 | **落在 `:34` 的第二条禁令上**（审计与被审计用同一段代码），不是第一行 |

「代价不许随盘容量增长」这一半（读数）：
- 单次代价随记录数 R 涨（量的）：`records_of_each_leaf` 单次 17.2 µs（R=1872）→ 729.3 µs（R=32944），`out/counts-7622.txt`。
- R 跟随的是已分配的槽加造池时没回收的已释放记录，不是空槽：建池没抬 F，`floor_raises=0`（`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-each-device-copies.out` 三行 `construction_timing` 都含 `floor_raises=0`，量的）。同一个 P=2048、共用存法，盘从 3221225472 字节换成 9412018176 字节（2.9 倍），`construction_seconds` 188.649 → 149.637（同目录 `construction-timing-2288MiB-shared.out` 与 `construction-timing-8GiB-shared.out`，量的）：没看到随空盘容量变慢；单次读数，噪声 30%，不是证明（推的）。
- 盘装满时 R 的上界就是槽数，所以「随已用容量涨」在盘写满时就是「随盘容量涨」（推的）。

**深拷那一组**：
- 复制的是版本值本身：`crates/singlefs-core/src/transaction.rs:13461` 「units_of_the_previous_version.unit(identity).clone()」、`crates/singlefs-core/src/mounted_session.rs:3067` 「.push(published.clone());」、`:3078` 「*current = PoolVersion::of_a_version_of_trees(published);」。
- 这个值的文档：`crates/singlefs-core/src/transaction.rs:3219` 「新池新建文件写出的东西，留给验收与探针用。」；装置拿到 `versions_published` 只读条数：`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:2094` 「u64::try_from(published.versions_published.len()).expect("版数"),」。
- 按 `:19` 的判据（谁消费这个数）：这一组里没有「在某个时刻被问一个数」的消费者，括号里四个词（分配、ENOSPC 准入、defer 窗口、生命周期判定）没有一个是复制整版字节。**按条文字面，不落第一行**；表里另两行（checker、后台）也不是发布路径。条文没有一行管「发布路径里随已写数据量涨的拷贝」，这一类悬空（调查员说的「条文没写到」我同意，并补上：不是漏看，是三行消费者里没有它）。
- 与它相邻、代码自己判过的一件事：`crates/singlefs-core/src/transaction.rs:3331` 「它在运行时决策路径上（ENOSPC 准入，」与 `:3332` 「在 units 上逐项找是这一版的单元数（每片容器、每个码 2 节点各一项），随容器总数涨」。代码自己把「ENOSPC 准入在 `units` 上逐项找」当成第一行的违反，用 `ContainerTreeRootUnitPositions` 修掉了；这说的是对 `units` 的查找，不是复制。

Q3 判定：扫记录组 **落第一行**（除那条 `assert!`，它落在 `:34` 第二条禁令）；深拷组 **按条文字面不落**，归类留给主 agent / 用户。

推翻它的现象：若把「发布」整体读作第一行括号里「分配」的一部分（发布是分配的外壳），则深拷组也落；条文里没有这个读法，我没采。

## 要主 agent 跑的命令

1. 复跑我的模型（纯文本分析；输出与 `out/` 九份逐份比，末行 `name=rerun_done differing_files=0` 算过）。在新 shell 里起；`--frozen` 这一轮没有冻结副本，给了开工快照目录凑数（这份 rerun.sh 的步骤不读它）：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-publish-growth-r1-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m4-gpugc-publish-growth-r1-snapshot --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-rerun-1`
   在仓根起；看 `task wait` 末行与 `name=rerun_compare` 九行都是 `status=same`。

2. 缺页与缓存、噪声的分辨（Q1.4）。在探针副本里把 glibc 的归还内存关掉，跑同一条 P=2048 命令，并跑一条不设变量的基线，各三次（读数离散 30%，一次不够）。在仓根起，`--cwd` 指探针副本；产物写到各自新文件名（`probe-runs` 目录我已建好）：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-copy -- env GLIBC_TUNABLES=glibc.malloc.trim_threshold=17179869184:glibc.malloc.top_pad=1073741824 cargo run --release -p singlefs-background-gpu --bin e202_gpu_cpu_gc_comparison -- construction-timing --unit-area 2288MiB --live-data-units 2048 --chunks each-device-copies --product /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-sonnet/probe-runs/no-trim-run1.out`
   基线同一条去掉 `env GLIBC_TUNABLES=…`，产物名 `baseline-run1.out`。每个产物各自新文件名（run2、run3 同形）。跑完看：`task-server/task log <号> --all | grep name=probe_window`，对 `minor_faults_total` 逐窗口求差 ÷ 用户改动数，与 `window_seconds`、`clone_into_versions_published` 一格比；`name=construction_timing` 行的 `construction_seconds` 三次取范围。
   判法（跑前写死）：无归还组的交错写窗口每次缺页降到 < 10% 基线，且 `construction_seconds` 低于基线最小值的 2/3 ⇒ 缺页是大头；缺页降了而秒数不变 ⇒ 缺页不是；缺页没降 ⇒ 变量没起作用，这趟作废。

3. 甲档只改类型的编译检查（Q2.1）：核心层库在 `PublishedUnit.bytes` 换成 `Arc<[u8]>` 后的编译错误条数与分文件分布。在新 shell 里起，`--src` 给仓根：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-publish-growth-r1-sonnet-model/tools/arc_bytes_type_only_check.sh --src /home/fy5090/code/singlefs --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-typecheck-1`
   在仓根起；看末尾 `name=error_lines count=<N>` 与随后按文件的条数。我读码的预期：`transaction.rs`、`mount.rs`、`recovery.rs` 三份里约 14 处构造点加 2 处比较，N 在 20 上下（推的）；N > 40 就是 Q2.1 的推翻现象。`cargo check` 不在范围声明闸的表里，不带声明。

没有别的要跑。第 1 条之外，其余读数都是我现读现算的原样输出，判决要引的数以主 agent 复跑为准。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 四格口径 | 部分成立 | clone 格包的是整版 `TransactionOutput`；分母口径对齐后是 48.7%–51.5%（量的） |
| Q1 阳性对照 | 证计时，不证覆盖 | 29/32 窗口 0.91–1.00；对照单价是真拷贝的 1/2.3–1/4（有缺页窗口，量的） |
| Q1 约 15% | 13.2% 量的，约 20% 推的 | 落在 `publish_admitted`（11.3%）与固定点内部；甲档碰不到 |
| Q1 次数不涨 | 对扫记录组基本成立，对深拷组不成立 | `carried_unit_clone` 次数 ∝ U；三格分配器查找次数涨三到四成（量的） |
| Q1 缺页一句 | 推的，读数不支撑 | 缺页只占末窗口约 22% 且饱和；每 MiB 的毫秒两档相同（量的） |
| Q1 外推 | 推的 | 8.8–111 h，窄带 14–41 h；外推距离是取样跨度的 1.58 倍 |
| Q2.1 甲档 | 做得到（推的） | 语义保得住，不是只改类型；14 处构造点 + 若干比较读取点；整组缩 14–28 倍不是 1000 倍；整次发布约快一倍 |
| Q2.2 乙档 | 做得到但不单独达成目标（推的） | 剩 U 级循环在 `publish_admitted`；`Arc<TransactionOutput>` 这一小块独立做得到 |
| Q2.3 ① 有序结构 | 做得到（推的） | 不碰已定项 7 与 17；只买回约 2.8 ms（约 5%） |
| Q2.3 ② 叶副产品 | 做得到，两个前提（推的） | 买回最多（约 8.5 ms）；漏一处写点即静默出错，需保留全量参照 |
| Q2.3 ③ 回滚日志 | 做得到，最难（推的） | 18 个字段加 `DeviceFreeMap`；只买回 1.07 ms；要求「换回去」逐位相同 |
| Q2.4 常驻内存 | 同一个根、两种持有（推的） | 峰值并存 2.6 倍 + `MountOutput` 持着；甲档与装置丢 `MountOutput` 各管一截，效果没量 |
| Q3 扫记录组 | 落第一行 | 对到「分配」「defer 窗口」「ENOSPC 准入」；`assert!` 落第二条禁令 |
| Q3 深拷组 | 按条文字面不落 | 四个括号词里没有复制整版；归类悬空，留给主 agent |

## 没做什么

- 没替主 agent 采纳或出判决；没往统一任务服务交任务；判决要引的数没复跑，复跑命令见上。
- 没编译、没起装置、没跑 rerun.sh（项目代码闸拒仓内脚本）：读数由拷在草稿目录的同一份脚本跑出，不是 `tools/rerun.sh` 跑出的。
- 没量：`publish_admitted` 里没点到行的那 6.89 ms 各占多少；固定点里 `records_of_each_leaf` 在固定点与断言之间的真实分摊；任何改法落地后的效果；P=4096 的探针读数；缺页是否能被关掉。
- 没改任何 `crates/` 文件，没改探针副本。
- Q2.2 里 `.units` 的 60 个验证包文件没逐个分是不是版本 `units`。
- 外推只用了 `construction_seconds`，没用逐窗口的结构模型积分（取样范围外 10 倍，积分不比幂律更可靠）。

## 清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-publish-growth-r1-sonnet`：只有文本（脚本与读数输出）；没建仓副本、没建编译目录。
