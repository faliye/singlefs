# m4-gpugc-floor-raise-precheck-code-r2 攻方腿（Opus）报告（2026-11-10）

立场：假设第一轮的修补（槽对那一判的上界加上这次挂载开过的聚簇段里没挡的槽对）是错的，在冻结副本的拷贝上造反例。主攻 T1，T2 与检查单八格逐格回答。
这一腿一条命令都没跑（子 agent 不交任务）：凡是没写「量的」的数都是读码推的；探针的读数等主 agent 跑完「要主 agent 跑的命令」，新派同立场的腿承接。
没读禁读清单里的文件（`research/prompts/m4-gpugc-floor-raise-precheck-code-r2-sonnet-output.md`）。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/`。被判的冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo/`；下面引的代码文件（`mounted_session.rs`、`allocator.rs`、`mount.rs`、`transaction.rs`、验证包 `common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/mutations.tsv`、运行库那一个测试文件）与仓里此刻那一份 sha256 前 12 位逐个比过、相同，所以引文写仓里的相对路径，行号就是冻结副本的行号。

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-opus/rerun-1
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `d85684d0e3766acaed14bda969b6f8725d70f97df535d1f4be7f815e3085eadf` |
| `probe/core_probe_module.rs` | `cfb0b2eb5e096a6532bcdeeeadcecb240f69142469a9affd3afd0d79250d5bcb` |
| `probe/src/device.rs` | `81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3` |
| `probe/src/main.rs` | `fbe1c08506aaffb6ebf26dc498f7eaca34cee880868aa5bde4517b324c95e931` |
| `probe/src/pool.rs` | `4aedb56cacf2951b92b6654c9c98f64567e447d844b873a8ec6c721a5f1bfa6b` |
| `probe/src/shapes.rs` | `e92688e1d846cea869862cb381909790d7b3101b8344488e76a79699bbcfc93d` |
| `probe/src/tally.rs` | `1f1d9c8ee8b2a6dbd4bc1dfa0548f4ec4531774ddeb1513fcf0d922a5814bd9c` |
| `tools/patch_the_copy.py` | `bc66e6b8a755fbf5607f0241899c5dec74a4a0a1feee26e5b42d42ec487c1fd6` |
| `tools/rerun.sh` | `ca38b4f3d1fc298d507ac255a91593d41054e552a4426513242c639581294cdc` |

`out/` 是空的：模型一次没跑过，也没编过（子 agent 不起 cargo；只用 `rustfmt --check` 核过语法，探针五份源文件与拼好探针的 `mounted_session.rs` 都只报排版差、没有解析错误）。复跑的输出写在 `--work` 那个目录的 `out/` 里。

探针做什么（各文件头有一段）：
- `tools/rerun.sh`：从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps`。冻结副本 rsync 进 `$WORK/copy`（不带 target、.git），`tools/patch_the_copy.py` 拼探针，`cargo build --offline --release -p floor-raise-precheck-r2-probe`，跑两条自证（各 62 个配置）与全量（372 个配置），自证有一条不合就退 1、全量的数不算。线程数从 `--repo` 下的 `.claude/heavy-thread-default` 读（服务给了 `SINGLEFS_THREAD_CAP` 就照它）。
- `tools/patch_the_copy.py`：只改拷贝里的 `crates/singlefs-core/src/mounted_session.rs` 两处（原文都要恰好命中一次，在冻结副本上核过各命中 1 次）：先算每进一次记一笔；读一块被点名的盘那一处改调探针的 `judged`，它按开关把聚簇段里那一截照原样、读成 0 或读成 2^20，再调原来的 `device_short_even_with_everything_a_floor_raise_could_release`，判法本身不动，同时记下四样槽数与两种上界。文件末尾拼上 `probe/core_probe_module.rs`；工作区成员加探针那个包。
- 盘：验证包的内存稀疏盘 `singlefs_verification::memory_pool::SparseBlockDevice`，外面包一层剪刀（`probe/src/device.rs`）：上膛之后第一笔落进 journal 环的写照常落盘，之后两块盘的写、清零、discard 一律丢掉（崩在那一次发布的第一条记录之后）。几何取验证包 384 槽小盘那一档（journal 环 6 MiB、单元区 1007 + 384 槽），两块盘各自再加 0 / 64 / 256 槽。
- 一个配置（`probe/src/shapes.rs`）：mkfs、取号、暖机、第一个文件之后，在同一次挂载里走 40 / 120 / 300 步前缀（建文件整份写 1–24 个数据单元、小覆盖写、unlink、整份重写，按种子放开；被拒照常记、推过的串留在盘上）。一半配置接着崩在一次 1–4 个单元的写的第一条记录之后、再可写挂载（C 那一圈罩着那一槽，不带任何读故障），再挂之后 0 / 1 / 3 / 6 次小写。然后在探测点上把一次大写（一半是新建 inode 写 g 个单元，一半是第一个文件整份重写成 g 个单元）的 g 从「段外槽对最少那块盘的数 − 2」扫到「段外 + defer + 隔离 + 空闲而被挡 + 聚簇段里那一截」再加 3，每个 g 在两份拷贝上发：A 照冻结副本；B 把那一截读成 0（第一轮修补之前的上界）。A 被拒、而且白推、或报被宽护卡住、或 C 那一圈还罩着时判出放不下的格（每个探测点至多 10 格），在 A 的拷贝上照出路正常卸载、再可写挂载、再发同一个写（D），记卸载再挂前后段外槽对涨了多少减去 A 算进去的聚簇段那一截；白推的头一格另在拷贝上先做一次小写再重发（用户的另一种动作）。
- 估时：先跑一批（与线程数一样多的配置），按它估全量挂钟，超过 40 分钟就停、退 3、只印那一批。内存（推的，没量）：每个线程同时持池与至多四份拷贝，每份两块盘写过的扇区约几十 MiB，31 线程峰值估 8 GiB 量级。
- 崩溃状态：这一题判的是推不推、报什么、出路走不走得通，探针只用一个崩溃点（第一条记录之后）造 C 那一圈当前提，不当崩溃一致性的样本，不调枚举函数、没有要算的状态数；崩溃一致性不在这一腿的结论里。

## 各格判定一览

| 格 | 判定 | 一句话 | 分量 |
|---|---|---|---|
| T1-1 加的那一截与卸载重挂放得出的口径同不同 | 不同（推的，待探针） | 那一截数的是此刻这次挂载开过的段里没挡的槽对（毛数）；卸载重挂放得出的是净数：卸载那一串先在开着的段里 bump 掉一截，再挂之后头一个提交内生块再开一个最低的全空段（又占掉一整段 32 对） | 读码；探针 `remount pairs-gained-minus-inside-counted` |
| T1-2 多算：聚簇段里的槽对卸载重挂也放不出 | 打中（推的，待探针） | 聚簇段那一截里主要是开着那一段没 bump 到的尾巴，卸载重挂把它还给段外、同时自己再开一段占回去，净放出可以是 0；在这一形上推的那一串与列的卸载重挂都白走 | 读码；探针 `vain-series remount-retry refused` |
| T1-3 白推一串的形与代价 | 打中（推的，待探针） | 不要任何故障：同一次挂载里攒过非空发布（上限高于 F）、需求落在「旧上界」与「旧上界 + 聚簇段那一截」之间的写，先算不再拦，推一串 2 或 3 次空发布、F 抬到上限（回退候选集缩到下限），推完照旧被拒；修补之前同一个写一个字节都不写。C 那一圈不罩着时推完报的是推满仍不够、不列任何出路，这一串换不来修补想保的东西。违 D16 已定项 10 射程「一串都不推」那一句的字面 | 读码；探针 `vain-series`、`vain-series no-outlet-reported` |
| T1-4 被宽护卡住那条出路在这一形上走不走得到 | 打中一半（推的，待探针） | 推一串之后同一次调用里再被拒，先算重读分配器：defer 里每个槽在上界里算一整对，回收出来一个数据单元（两槽）只成一对，上界掉下去；需求落在「推完的上界」与「推前的上界」之间时第二次先算判出放不下，报 `NoSpaceEvenWith…`、带着刚推的串、不报被宽护卡住，出路照旧被吃掉。这一缝与聚簇段那一截无关，修补碰不到 | 读码；探针 `outlet-lost-after-a-series remount-retry published` |
| T2-1 宽护的可达性 | 改判 r1 的前提（读码，验证包已有用例） | 报「被宽护卡住」按 C 那一圈整圈判，不要读坏的被抛弃根：验证包已有一形是不带任何读故障的自然崩溃点（崩在一次写的第一条记录之后），任务 11945 那 11 个目标里它照绿。读坏的被抛弃根这一腿没造 | 读码 |
| T2-2 新那一项读成 0 没有用例红 | 成立（读码） | 新用例直接给字面量，只验算术；`of_the_device` 里那一截读成 0 没有任何用例走得到；变异表里「读分配器时 defer / 隔离 / 扣住的槽读成 0」三行有，这一截没有 | 读码；探针 `arms_differ` 给接到真分配器上的读数 |
| T2-3 条款与代码 | 不一致一处（读码） | D16 已定项 10 射程只写「推抬 F 放得出的全放出来也放不下就一串都不推」，没写槽对那一判的上界另加了推抬 F 放不出的一截 | 读码 |
| 检查单：走不到的理由二次查实 | 成立 | 新 `.expect` 依赖的「每段的数之和等于总数」由 `set_blocking_bit` 一处成对维护，聚簇段集合是 `BTreeSet` 不重 | 读码 |
| 检查单：类型与名字 | 不成立（字面） | 同一个结构体里三个字段是槽、新字段是槽对，都是裸 `u64`，单位只在字段名里；换算在 `most_a_floor_raise_could_make_available` 两行各乘一次数据单元槽数，今天算对 | 读码 |
| 检查单：不兼容旧代码与旧格式 | 成立 | 补丁三个文件里没有 | 读补丁 |
| 检查单：不新建判定库 | 成立 | 补丁里没有新的存放处 | 读补丁 |
| 检查单：所有任务多机可用 | 成立 | 新用例是纯函数，不读环境与文件 | 读补丁 |
| 检查单：测试优先多线程 | 成立 | 两条崩溃循环用例上面各一行「不切片并行」的理由 | 读码 |
| 检查单：带 --item 的命令与项 | 成立 | 新变异行在 `crates/mutations.tsv` 第 7580 行，变异前的原文在 `mounted_session.rs` 只命中一次 | 读登记 |
| 检查单：新崩溃流多罩了什么 | 成立（不要新流） | 白推的那一串走的是已有的抬 F 那一串，`floor-raise-pushed-by-the-session` 那一项罩着它的写形；修补只改推不推，不改推的写什么 | 读码 |

## T1　第一轮的修补本身

被判的那一截：先算读一块盘时，`of_the_device` 用同一个计数算两遍相减（「.expect("段外槽对是全部槽对减去那几段各自的数，各段的数都不为负：段外槽对不大于全部槽对"),」（`crates/singlefs-core/src/mounted_session.rs:5970`）），槽对那一支把它乘一个数据单元的槽数加进上界（「slot_pairs_a_remount_frees_from_the_cluster_segments * slots_of_a_data_unit,」（`crates/singlefs-core/src/mounted_session.rs:6000`））。字段文档写的理由是「卸载重挂时分配器重建、聚簇段清空，它们就算进段外槽对」（`crates/singlefs-core/src/mounted_session.rs:5951`）。

### T1-1　加的那一截是不是「卸载重挂放得出」的那一截

不是同一个口径（推的）。那一截是此刻的毛数：这次挂载开过的每一段里两槽都没挡的槽对（`allocator.rs` 第 576–600 行那个函数：总数减去集合里每段各自的数，「self.unblocked_slot_pairs - pairs_inside_the_cluster_segments」（`crates/singlefs-core/src/allocator.rs:599`））。卸载重挂实际放出来的是净数，中间有三步各自吃掉一截：

1. 正常卸载先推一串空发布（第一版几何 2 或 3 次，`.claude/kb/decisions/16-发布语义.md` 第 43 行「正常卸载」那一行），这一串用的是会话那一个分配器，开着的段还开着，它们的提交内生块先在开着那一段的游标后面 bump（`try_allocate_commit_generated` 先走 `bump_slot_in_open_segment`，`allocator.rs` 第 2584–2588 行）——吃的正是那一截里开着那一段的尾巴。
2. 再挂时分配器从盘上重建，集合清空（「let mut allocator = Self::new(devices);」（`crates/singlefs-core/src/allocator.rs:2270`），`new` 里「cluster_segments: BTreeSet::new(),」（`crates/singlefs-core/src/allocator.rs:1955`））。到这一步，那一截（减去第 1 步吃掉的）才算进段外。
3. 再挂那一刻的写行那次发布与暖机的空发布都写提交内生块。重建之后没有开着的段，头一个提交内生块去开最低的全空段（「Some(segment_start) => {」（`crates/singlefs-core/src/allocator.rs:1074`）那一支，全空的判法是「self.used_per_segment[*segment] == 0」（`crates/singlefs-core/src/allocator.rs:1009`）连同隔离、扣住都是 0），开段那一刻把起点放进集合（「self.cluster_segments.insert(segment_start);」（`crates/singlefs-core/src/allocator.rs:2625`））。这一段一整段 32 对从段外拿走——不管它原来是不是上一次挂载的聚簇段（上一次挂载回收空了的段在那一截里按 32 对算过）。

所以卸载重挂净放出的段内槽对 ≈ 那一截 − 卸载那一串在开着那一段里 bump 掉的 − 再挂开的那一段的 32 对（再加卸载抬 F 回收的 defer、C 那一圈换根之后放掉的隔离，这两样旧上界里已经各算过）。两者相等要「再挂开的那一段落在旧上界之外又不占段外」，这在全空段只按最低起点挑的规则下推不出来。

探针怎么量：D 那一步在卸载之前与再挂之后各读一次每块盘的段外槽对，记「再挂后段外 − 卸载前段外 − A 算进去的那一截」的分布（`remount pairs-gained-minus-inside-counted <桶>`，负数桶是多算）。

翻面条件（跑前写下）：全量里这个分布的负数桶合计为 0（再挂净放出的从不少于算进去的那一截），T1-1 撤回、改「口径不同但不多算」；负数桶占大多数而且集中在 `-32-63` 那一桶，坐实「再挂自己开一段占回去」。

### T1-2　会不会多算：聚簇段里那些槽对卸载重挂也放不出的情形

打中（推的，待探针）。具体的历史（每一步许可它的那一句）：

1. mkfs、第一个文件之后在同一次挂载里发几次改用户可见状态的发布（至少 4 次，准入抬 F 的上限才高于 F = 0：「**准入抬 F**：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」（`.claude/kb/decisions/16-发布语义.md:38`））。每次发布的提交内生块在开着的段里 bump，段里没 bump 到的尾巴两槽都没挡，算在那一截里（5951 行那一项）。
2. 用户写一个 g 个数据单元的文件，g 落在「段外槽对 + defer + 隔离 + 空闲而被挡」与「再加那一截」之间（按段外槽对最少那块盘算）。落得下那一判排在式子那一判前面（「admit_the_units_landing_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13317`）在「admit_on_every_device(」（`crates/singlefs-core/src/transaction.rs:13420`）之前），被拒点名的是槽对那一判。
3. 先算：上界 = 段外 + 每个放得开的槽补一对（「slots.deferred + slots.isolated + slots.free_but_blocked;」（`crates/singlefs-core/src/mounted_session.rs:5992`））+ 那一截 ≥ 需求，不判出放不下（「(upper_bound.0 < i128::from(short.demand.0)).then_some(」（`crates/singlefs-core/src/mounted_session.rs:6015`）），推一串（「match push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:6119`））。
4. 这一串只能把 defer 回收、把扣住放开，段内的槽对在一次挂载里出不了集合（集合只在 2625 行加，「段里的块被回收空了也仍算聚簇段」（`crates/singlefs-core/src/allocator.rs:1890`））；第一轮 T1-1 已论证旧上界是推一串能放出的真上界。所以推完照旧被拒，循环回到先算（「push_one_floor_raise_for_a_refusal_for_space(」（`crates/singlefs-core/src/mounted_session.rs:3795`））。
5. 用户照出路（或自己）正常卸载、再可写挂载、再发同一个写：按 T1-1，那一截里开着那一段的尾巴被卸载那一串吃掉一部分，再挂又开一整段占回去；g 落在「旧上界 + 净放出」之上时照旧被拒。

这一形上推的那一串白写，列的卸载重挂（C 那一圈罩着时才列）也白走。

四句：
1. 分不分辨臂：分辨「那一截按毛数算」（冻结副本）与「那一截按卸载重挂的净数算」或「不算」两种；与修补之前（B）比，B 在第 3 步就判出放不下、一个写都不发。
2. 系统那一刻看得到吗：一半。开着那一段的起点与游标在分配器里（`open_segment`、`bump_cursor`，`allocator.rs` 第 1885–1886 行），尾巴有多少对看得到；再挂之后最低的全空段落在哪、卸载那一串要 bump 多少，要预演才知道，先算不预演。按「至少再挂要开一段」扣 32 对是看得到的下界。
3. 字面分句：字段文档那一句「卸载重挂时分配器重建、聚簇段清空，它们就算进段外槽对」（5951 行）在这一形上不成立；判决第三节第 1 条的条款只写「加上这次挂载开过的聚簇段里没挡的槽对」，照字面实现了，打中的是条款本身的口径。
4. 改法：判决只给了「加上这一截」一个改法，它就是打中的原因，在这几格上照中。

翻面条件：探针全量 `vain_remount_still_refuses` 为 0（白推的每一格卸载重挂都放得下），T1-2 撤回、只剩 T1-3 的代价；大于 0 坐实多算，看 `EXAMPLE FOLLOWED … D=…` 行里的 g、`inside_min` 与 `remount_gained_minus_inside`。

### T1-3　宁大勿小之后白推一串：在什么形上出现、多大

打中（推的，待探针）。形：T1-2 第 1–4 步，不要任何故障、不要崩溃，只要两件事——这次挂载开过的段里还有没挡的槽对（开着那一段的尾巴就够），上限高于 F（上一次抬 F 之后又有过改用户可见状态的发布）。需求落在 (旧上界, 旧上界 + 那一截] 的写都走这一形，宽度就是那一截的对数（按数据单元算；一段 64 槽、32 对，开着那一段的尾巴 0–32 对，加上集合里别的段里的空对）。

一次调用里发生的事（冻结副本，A）与修补之前（B）：

| 步 | A（冻结副本） | B（第一轮修补之前） |
|---|---|---|
| 头一次被拒（槽对那一判） | 上界算上那一截 ≥ 需求，照推一串：2 或 3 次空发布，F 抬到上限 | 上界 < 需求，判出放不下，`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，盘上逐字节不变（「这一次判之前已经推成的串、发成的版在里面，头一次被拒就判到这里时它们是空的，盘上逐字节不变」那一句，`mounted_session.rs` 第 645 行） |
| 推完重发，再被拒 | 先算重读分配器：仍 ≥ 需求就再推，这时上限不高于 F，一个字节不写就停（「if ceiling <= current.root.rollback_floor {」（`crates/singlefs-core/src/mount.rs:4853`）），报 `NoSpaceAfterRaisingTheFloor`；已经掉到需求以下就报 `NoSpaceEvenWith…`、带着刚推的串（T1-4） | — |
| 报不报出路 | C 那一圈一槽都不罩时不报被宽护卡住（「let Some(target) = root_ring_push_target(allocator, current.root.checkpoint_txg) else {」（`crates/singlefs-core/src/mounted_session.rs:1980`）那一支交 `NotWhatStopsTheAdmission`），一条出路都不列 | 不列 |

代价（推的，探针数）：
- 每个落在窗口里的写，一次调用多写一串（2 或 3 次空发布，每次重写树表等固定点单元、写根槽与系统配置），用掉 2–3 个根环槽。
- F 抬到上限，回退候选集从「环里还有的全部状态」缩到条款下限 4 个不同状态（上限就是第 4 新的非空根，kb 第 38 行）。B 那一臂这一步什么都不动。
- 紧接着重发同一个写不再推（上限不高于 F）；但只要中间夹一次改用户可见状态的小写，上限又抬高，再发这个写又推一串。探针在白推的头一格上数这一种（`vain-series retry-after-one-small-write series=…`）。
- 这一串换来的东西：C 那一圈罩着时推到上限报被宽护卡住、列卸载重挂（判决要保的那条出路）；C 那一圈不罩着时什么都换不来——而后者是不带故障的常态。

四句：
1. 分不分辨臂：分辨 A 与 B（B 一个写都不写）；也分辨「那一截只在 C 那一圈罩着隔离槽时才算」的改法（见改法表），它在 C 那一圈不罩着的格上与 B 相同。
2. 系统那一刻看得到吗：看得到。C 那一圈这一刻罩不罩着隔离槽，报被宽护卡住那一判就是只读分配器算的（`root_ring_push_target` 与「let isolated_slots_on_each_device = allocator.isolated_slots_covered_by(&target.placements);」（`crates/singlefs-core/src/mounted_session.rs:1983`）），先算手里就有同一个分配器。
3. 字面分句：D16 已定项 10 射程「抬 F 之前先算（用户 2026-11-09 定）：推抬 F 放得出的全放出来也放不下就一串都不推」（`.claude/kb/decisions/16-发布语义.md:275`）。那一截不是推抬 F 放得出的，这一形上推抬 F 全放出来也放不下、照推了一串：字面违。代码自己的文档也写这一成员的用处是「推了也写不进的写不花根环的槽与写盘」（`crates/singlefs-core/src/mounted_session.rs:673`）。判决第三节第 1 条的理由（保住被宽护卡住那条出路）只罩 C 那一圈罩着的那几格，条款写成了一律加。
4. 改法：判决给的改法就是这一形的来源，在这几格上照中。

翻面条件（跑前写下）：
- 全量 `vain_series_by_the_inside_term` 为 0 而自证二（A 那一截读成大数）大于 0：探针接得上、这一份取样里窗口没撞上，T1-3 改「推的、取样没撞上」，要再抽一份。
- 大于 0：坐实；`vain_without_outlet`（白推而 C 那一圈不罩着、不报出路）占多少就是修补换不来东西的比例；`vain_empty_publishes / vain_series` 是每次白推的空发布数。
- `published_where_the_old_bound_fired` 大于 0：那一截让一个修补之前会被误拒的写写进了，与第一轮 T1-1「旧上界是真上界」相反，T1-3 的「白推」在那几格不成立，回看第一轮 T1-1。

### T1-4　被宽护卡住那条出路在这一形上走不走得到

一半走得到，一半走不到（推的，待探针）。报被宽护卡住要三样同时成立：先算没判出放不下、推满停在「F 已在上限」（「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」（`crates/singlefs-core/src/mounted_session.rs:1974`））、C 那一圈还罩着隔离槽。走得到的那一半：需求不超过推完一串之后重读出来的上界。走不到的那一半，历史如下：

1. 前缀之后发一次写 R，崩在它的第一条记录之后，再可写挂载：择根退到 R 之前那一版，R 的记录在水位之上、没施加，C 那一圈只罩 R 那一槽（验证包已有同一形，「攻方 K1 那一形崩在 R 第一条记录之后的盘面（不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，」（`crates/singlefs-verification/tests/common_images_for_the_given_states_engine/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3590`））。
2. 挂上之后上限高于 F（同一份文件第 3689 行：挂上之后头一次被拒时「这一次调用里会话先推一串抬 F、把 F 抬到上限」）。
3. 用户写 g 个单元，槽对那一判拒。先算：上界里 defer 的每个槽都按「补成一对」算（5992 行）。g 不超过它，推一串。
4. 这一串回收 defer：一个数据单元两槽回收出来只成一对，按上界的算法却算过两对；聚簇段里回收出来的单槽元数据凑对更少；这一串自己的提交内生块又在开着那一段 bump 掉几对。推完重发再被拒，先算重读（3795 行进来，6098 行重算），上界掉了；g 落在「推完的上界」与「推前的上界」之间时这一次判出放不下，报 `NoSpaceEvenWith…`，`floor_raises` 带着刚推的那一串。
5. 停在上限那一步没走到，C 那一圈虽然罩着，被宽护卡住不报，卸载重挂不列。

这一缝的宽度来自「defer 每槽算一对」的松，与聚簇段那一截无关；修补把整个窗口往上平移那一截，没有收窄它。

四句：
1. 分不分辨臂：分辨「每次被拒都重跑先算」（冻结副本）与「推过之后不再先算、照推到停」或「先算判出放不下时 C 那一圈罩着就照样带出路」两种。
2. 系统那一刻看得到吗：看得到（同 T1-3 第 2 句）。
3. 字面分句：D16 已定项 1「根槽这一次读坏」那一行那一句的前提「准入推抬 F 停在上限、C 那一圈这一刻还隔离着槽时（按整圈判，分不出宽护与 C 里别的几份）」（`.claude/kb/decisions/16-发布语义.md:39`）没发生，字面不违；违的是判决第二节那一格的采纳理由「推到上限照旧报被宽护卡住、列卸载重挂」。D16 已定项 10 射程许「推过一串之后…才判出的，盘上有这次调用写的东西」，许的是盘上有写，不管出路。
4. 改法：判决的改法在这几格上照中（它不碰 defer 那一项）。

翻面条件：全量 `outlet_lost_after_a_series`（C 那一圈罩着、推过一串之后判出放不下、卸载重挂之后写得进）为 0 而 `stuck_reports` 大于 0：这一半在取样里没出现，T1-4 改「推的、取样没撞上」；大于 0 坐实。`outlet_lost_first_round` 大于 0 是 C 那一圈罩着时头一次就判出放不下、卸载重挂却写得进——那是修补要保而没保住的格，说明那一截算少了，与 T1-2 方向相反，要回看。

## T2　产生结论的方法与从未被看过的地方

### T2-1　第一轮 T3-4 的可达性：「宽护要读坏的被抛弃根才造得出」

这一句把两样东西混了（读码）。读坏的被抛弃根造出来的是宽护本身（R 那一槽并上现行账里全部已释放的落点）；修补要保的那条出路是「被宽护卡住」那一报，它按 C 那一圈整圈判：「这里分不出 C 那一圈里哪一份是宽护、哪一份是窄的那几份」（`crates/singlefs-core/src/mounted_session.rs:716`），条款同一句写「按整圈判，分不出宽护与 C 里别的几份」（kb 第 39 行，T1-4 引过）。C 那一圈不要任何读故障就有：崩在一次发布的记录之后、根槽之前，再可写挂载时那几条记录在水位之上、没施加（T1-4 第 1 步引的验证包那一形，3590 行「不带任何读故障的自然崩溃点」）。那一族用例（`run_the_scenario_where_a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push` 等）在 `raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table` 那个目标里，正文第二节记任务 11945 那 11 个目标照绿（主 agent 的观测，这一腿没核）。

所以第一轮判决「可达性（宽护要读坏的被抛弃根）没量」那一条的前提不成立：出路这一报在不带故障的崩溃之后就可达，T1-3、T1-4 两形都不要读故障。探针的 `crash-after-the-first-record` 那一族就是这一形（剪刀剪在第一笔 journal 写之后），`c-ring setup cut=true holding-after-mount=true` 的计数是它的阳性对照，`stuck_reports` 是报出来的次数。

读坏的被抛弃根（宽护本身）这一腿没造：验证包里造它的办法（可写挂载那一刻让几段读回全零、挂上就撤，`pool_stuck_behind_the_wide_protection`，同一份文件第 2710 行起）在测试模块里，拷进探针要连带拷读回全零的盘与录制流，这一轮没做；按上面的论证它不是出路可达的必要条件，对 T1 的判定不起作用（推的）。

翻面条件：探针 `c-ring setup … holding-after-mount=true` 计数为 0（剪刀剪了而 C 那一圈不罩），或 `stuck_reports` 为 0：这一族在探针里造不出报出路的那一形，T2-1 只剩读码与验证包那一份证据。

### T2-2　新那一项读成 0 时没有用例红：判法模块的用例是不是只验了算术

成立（读码）。新用例 `fn an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound()`（`crates/singlefs-core/src/mounted_session.rs:8374`）直接给 `SlotsAFloorRaiseCouldRelease` 字面量（改动全文第 145–151 行），走的是 `most_a_floor_raise_could_make_available` 那一行算术；`of_the_device` 在 `crates/` 下只有 6044 行一处调用（`grep -rn 'SlotsAFloorRaiseCouldRelease::of_the_device\|fn of_the_device' crates --include=*.rs` 命中三行：5957 定义行、6044 调用行，另一行是 `compaction_plan.rs` 里不相干的 `of_the_device_map`），判法模块不经它。新变异行（`crates/mutations.tsv:7580`）改的也是 6000 行那一句算术。同一族已有「读分配器时 defer 读成 0」「隔离读成 0」「扣住的槽读成 0」三行（`crates/mutations.tsv` 第 7492–7494 行），聚簇段那一截没有对应的一行。验证包 11 个目标「上界变大之后没有用例改判」（正文第二节），说明没有一个集成场景走到这一截让结局变——把它读成 0，全仓用例照绿（推的，实现员报告同一句）。

探针给的是接到真分配器上的读数：A 与 B 只差「`of_the_device` 里那一截读成 0」，`arms_differ` 大于 0 就是这一截在真池上改了结局的格数，`EXAMPLE` 行带每一格的读数，可以照它写一条集成用例（见改法表）。

### T2-3　条款与代码

不一致一处（读码）：D16 已定项 10 射程那一段写「式子那一判的上界是可用加 defer 加隔离，短在段外槽对那一判同样先算，上界恰好等于需求照推」（`.claude/kb/decisions/16-发布语义.md:275`），没有写槽对那一判的上界另加了一截推抬 F 放不出的量；同一段开头又写「推抬 F 放得出的全放出来也放不下就一串都不推」。代码按判决改了，条款没跟（`grep -rn '聚簇段里没挡\|卸载重挂放得出' .claude/kb/` 零命中）。按 T1-3，两句之中只能留一句：要么条款写明「槽对那一判另加这次挂载开过的聚簇段里没挡的槽对、在这一截上推了也可能写不进」，要么代码去掉这一截。

## 检查单八格

### 走不到的理由二次查实：成立

新加的唯一一处 `.expect`（`crates/singlefs-core/src/mounted_session.rs:5970`）要「传集合算出的段外槽对 ≤ 传空集算出的」。被调的函数返回总数减去集合里每段各自的数（`allocator.rs` 599 行）；传空集时减 0，就是总数。要它不下溢，得「集合里各段的数之和 ≤ 总数」。总数与每段的数只在一处成对改：「self.unblocked_slot_pairs -= 1;」（`crates/singlefs-core/src/allocator.rs:552`）紧挨着同一段的减一，「self.unblocked_slot_pairs += 1;」（`crates/singlefs-core/src/allocator.rs:556`）紧挨着同一段的加一，起步「unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().sum(),」（`crates/singlefs-core/src/allocator.rs:489`）；`grep -n 'unblocked_slot_pairs \?[-+]=' crates/singlefs-core/src/allocator.rs` 只这两处。所以总数恒等于全部段的数之和；集合是 `BTreeSet`，同一段不会减两次。被调函数里那两处断言（段在这块盘的单元区里、在段边界上）准入那一判拿同一个分配器的同一份集合先调过（实现员报告「走不到的理由」那一节），这一腿没另找到第二个调用点。走不到。追调用点：`of_the_device` 只在 6044 行一处调，参数是拒绝点名那块盘的 `device_map` 与 `allocator.cluster_segments()`，与准入判它短的是同一个分配器（第一轮 T2 已核）。

### 类型与名字：不成立（字面）

新名字 `unblocked_slot_pairs_inside_the_cluster_segments` 说出了内容、单位在名字里。但同一个结构体里 `deferred`、`isolated`、`free_but_blocked` 三个字段是槽数，新字段是槽对数，四个都是裸 `u64`（「deferred: u64,」（`crates/singlefs-core/src/mounted_session.rs:5945`）、「unblocked_slot_pairs_inside_the_cluster_segments: u64,」（`crates/singlefs-core/src/mounted_session.rs:5953`））；换成字节时两行各乘一次「TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();」（`crates/singlefs-core/src/mounted_session.rs:5994`）：前一行把槽对数乘成槽数（一对 = 一个数据单元的槽数，对），后一行把「每个放得开的槽补一对」也乘成槽数（同一个乘数、意思不同）。今天两行都算对；槽对与槽共用裸整数，单位只靠名字，正文问的就是这一条，字面答「有」。改法（零轮）：槽对数包一个只能乘成槽数的新类型。

### 不兼容旧代码与旧格式：成立

补丁 `crates.patch` 两个文件（`mounted_session.rs`、运行库测试文件）加 `mutations-append.tsv` 一行，逐段读过：没有认旧版本、旧布局、旧字段的分支，没有只为旧数据放宽的读法。

### 不新建判定库：成立

同一份补丁里没有新的目录、TSV（变异表那一行是已有登记表的追加，不是判定存放处）、sqlite、RocksDB；新用例的断言都在进程内。

### 所有任务多机可用：成立

新用例 `an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound` 是纯函数，不读环境、文件、主机名；运行库那两处只加了注释。

### 测试优先多线程：成立

`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs` 第 1093、1101 行各一行「// 不切片并行：每次挂载接着上一次崩溃留下的盘，次序本身就是被测对象，没有可切的区间；两种填法各是一个测试函数，由测试框架并行。」，就在两条崩溃循环用例上面；理由与第一轮攻方读到的一致（历史前后相依）。新用例是单个纯函数，不涉及。

### 带 --item 的命令与项：成立

新变异行在 `crates/mutations.tsv:7580`（行首「抬 F 先算：槽对那一判的上界不算聚簇段里没挡的槽对」），第三列原文 `slot_pairs_a_remount_frees_from_the_cluster_segments * slots_of_a_data_unit,` 在 `mounted_session.rs` 只命中 6000 行一处（`grep -nF` 量的）。这一批没有新的崩溃点重放项，第一轮那三项照旧。只读登记，没跑 `--list-items`。缺的一行见 T2-2（`of_the_device` 里那一截读成 0）。

### 新崩溃流多罩了什么：成立（不要新流）

修补只改「推不推」，不改推的那一串写什么：白推（T1-3）走的是已有的 `push_one_floor_raise_within_the_admission_budget`，写形与 `floor-raise-pushed-by-the-session` 那一项罩的同一种（推的）。新的是「这一类写也会推」——崩在白推那一串中途，盘面与那一项的同形；头一次就判出放不下的那一形一个写都不发，没有崩溃状态。不要新流。

## 这一腿提的改法各修哪一格

都是这一腿提的，读码推的、没实现没跑（探针也还没跑），被攻过零轮。每一格都是「推的」；探针跑出来之后只能回答「打中的格在不在」，回答不了改法本身，改法要另在副本上实现、交主 agent 跑。

| 改法 | T1-2 多算 | T1-3 白推（C 那一圈不罩） | T1-4 推过一串之后丢出路 | 别的 | 标 |
|---|---|---|---|---|---|
| 去掉那一截；先算判出放不下时，C 那一圈这一刻罩着隔离槽就照 `wide_protection_at_the_ceiling` 的同一个判法（不要求停在上限）给 `NoSpaceEvenWith…` 带上 R 那一槽与卸载重挂那条出路（第一轮攻方 T3-4 提过这一臂） | 修（不推） | 修（不推、不报出路，与 B 同） | 修（头一次与推过之后判出放不下都带出路） | 推环那条出路要不要在这里预演另议；`NoSpaceEvenWith…` 多一个字段 | 推的 |
| 那一截只在 C 那一圈这一刻罩着隔离槽时才算 | 不修 | 修 | 不修 | 改动最小 | 推的 |
| 那一截按净数算：减去开着那一段的尾巴与再挂要开的一段（32 对） | 部分修（只剩卸载那一串 bump 的那几对） | 收窄、不消 | 不修 | 「再挂开的那一段落在哪」读不到，只能按下界扣 | 推的 |
| defer 那一项按「回收出来成几对」算（数据单元两槽一对；聚簇段里的 defer 不算），要按段另维护一个 defer 计数 | 不修 | 收窄 | 收窄（推前推后上界差变小） | 第一轮攻方 T3-3 改法的同一个计数 | 推的 |
| D16 已定项 10 射程补「槽对那一判的上界另加这次挂载开过的聚簇段里没挡的槽对，这一截推抬 F 放不出，落在它上面的写推了也写不进」，或照第一行改法删掉那一截之后不补 | — | 条款与代码对齐，不改行为 | — | T2-3 | 推的 |
| 一条集成用例：照探针 `EXAMPLE FOLLOWED` 那一行的配置造池，钉住「A 白推、B 不推」或改法之后的结局；配变异行「读分配器时聚簇段那一截读成 0」 | — | — | — | T2-2 | 推的 |
| 槽对数包一个新类型 | — | — | — | 检查单「类型与名字」 | 推的 |

## 要主 agent 跑的命令

1. 探针整份复跑（`tools/rerun.sh`，模板拷来、只换了 `rerun_steps`）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2/repo --work /home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-opus/rerun-1
```

   - 在哪起：仓根（命令里的路径都是绝对路径）；`--work` 指到这一腿认领目录下还不存在的 `rerun-1`（认领目录的 `.deliverables` 写成交接）。在新 shell 里起。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完、退非 0 的件数为 0。`task-server/task log <号>` 里依次有 `PATCH_THE_COPY …`、`name=build done …`、`name=exit_codes arm_a_zero=0 arm_a_huge=0 probe=0`、三份的 `ESTIMATE`、`WALL_SECONDS` 与 `PROBE_RESULT` 行、`name=self_checks arms_differ_when_both_zero=0 vain_when_the_inside_term_is_huge=<正数>`、`name=rerun_done …`。退 1 而日志尾巴是 `cargo build` 的报错：这一腿没编过探针，编不过是它的错，交承接的腿修（`$WORK/out/build.log`）。退 1 而写「自证一不合」或「自证二没红」：探针判不出它要判的，全量的数作废。`probe=3`（或自证那两趟是 3）是估时超过 40 分钟，`.out` 里有 `ESTIMATE_EXCEEDS_FORTY_MINUTES` 与先跑那一批的计数，交承接的腿缩配置数。
   - 内存与时间（推的，没量）：编译要编验证包的库（连带判定库客户端那一份的 RocksDB），编译占大头；跑的时候每个线程持一个池与至多四份拷贝，31 线程峰值估 8 GiB 量级；全量 372 个配置、两条自证各 62 个，挂钟由探针先跑一批自己估，超过 40 分钟自己停。
   - 重跑：模板的 `parse_arguments` 遇到非空的 `--work` 就退 2（`rerun_steps` 之前，这一腿按定义只改 `rerun_steps`，改不到那里）。服务撞内存上限重排同一条命令时第二趟会当场退 2；那时换一个新的 `--work`（`rerun-2`）重交。这一条与正文「五、交付要求」那一句「草稿目录非空时先清再跑」对不上，交主 agent 定。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与 `rerun-1/out/` 下三份 `.out`，按本报告 T1-1 到 T1-4、T2-1 写下的翻面条件改判定。

## 没打中的形状

读码试过、判不出毛病的（都是推的）：
- 那一截与 defer、隔离、空闲而被挡三样重算：那一截只数两槽都没挡的对，另三样数的都是挡着的槽，同一个槽不会两头都算。
- 不等大的盘：那一截逐盘按同一份集合算，集合里的段是各盘一致答出来的全空段起点（被调函数的断言），每块盘都是整段在单元区里；逐盘判，与准入逐设备合取同形。
- `accept` 那一路与走空那一路：三处共用 `push_one_floor_raise_for_a_refusal_for_space`（3795、6213、7048 行），修补对三处一样，没有只在一处出现的形。
- 式子那一判：修补不碰它，上界照旧是可用加 defer 加隔离。
- 新 `.expect`：走不到（检查单第一格）。
- 修补之后误拒（推了本来写得进而先算拦了）：上界只变大，不会新添误拒；探针 `published_where_the_old_bound_fired` 数的是反方向（修补之前误拒、修补之后写进），作为第一轮 T1-1 的复核。

探针的取样范围（等主 agent 跑）：372 个配置、种子 1–372；两块盘各在 0 / 64 / 256 槽里挑；同一次挂载里前缀 40 / 120 / 300 步，动作四种按权重放开；一半配置崩在一次 1–4 个单元的写的第一条记录之后再挂、再挂之后 0 / 1 / 3 / 6 次小写；探测点上 g 从段外槽对最少那块盘的数 − 2 扫到全部算上之后 + 3，至多 160 个 g（宽了就按步长取），A、B 都头一次就判出放不下连着 6 格就停；每个探测点至多 10 格走卸载再挂。两条自证各 62 个配置、种子 900001 起。这一份取样只算一次观测，`vain_series_by_the_inside_term` 为 0 也要换种子再抽一次才记没打中。

## 这条腿自己的限度

- 一条命令都没跑：判定全是读码推的；探针连编都没编过（子 agent 不起 cargo），只用 `rustfmt --check` 核过语法，编不过的可能不小。被引的 API（会话、挂载、卸载、mkfs、稀疏盘）照第一轮攻方那一份编过的探针的用法写，`mount.rs`、`transaction.rs` 在两轮之间改过，公开函数的签名逐行 diff 过、没变。
- 只用一个崩溃点（第一笔 journal 写之后）造 C 那一圈，不枚举崩溃状态，不判崩溃一致性；读坏的被抛弃根（宽护本身）没造。
- 大写只用整份写（新建 inode 写 g 个单元，或第一个文件整份重写成 g 个单元）；`accept`、unlink 走空那两路没走。前缀只在同一次挂载里，不带快照、截断。
- 探针在拷贝里改了核心层两处（加记录与开关，判法本身不动）；B 那一臂等于「`of_the_device` 里那一截读成 0」，与第一轮修补之前那一版的差只在这一项（改动全文只动了这一截、文档与用例），没另拿修补之前的源码编一臂。
- T1-1 的「再挂开一段占回 32 对」只在每块盘还有全空段时成立；没有全空段时走回落到最低空槽，那时占回的更少，探针的分布会显出来。
- 没读禁读清单里的文件；辩方腿的结论不知道。

## 草稿与清理

- 认领目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-opus/`：这一腿没建仓副本、没建编译目录。`fmt/` 下是 `rustfmt --check` 用的源文件拷贝与输出（552K，`du -sh` 量的）与 cite-check 的输出（4.0K），交回前都已删。`.deliverables` 写一行：`rerun-1` 交接，给主 agent 的复跑用。
- 模型目录在仓里 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/`，`out/` 是空的。

## 没做什么

- 不判辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 副本上的数：这一腿一个都没有。改法表里每一格都是推的，没在副本上实现。
- 没跑门禁、没跑 `--list-items`、没跑 `--selftest`。


引文核对：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-output.md` 末行「✓ 核了 33 处引文、4 处裸行号，对上 33 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）」。
