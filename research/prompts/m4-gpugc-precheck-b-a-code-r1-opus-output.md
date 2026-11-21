# m4-gpugc-precheck-b-a-code-r1 攻方腿（Opus）报告（2026-11-12）

批次 `records/2026-10-19-里程碑四GPU垃圾回收开工.md#127`。立场：假设「乙的上界不算小、乙不丢走得通的出路、甲不误拒也不漏判、报的成员读得对」是错的，造历史去打。被判的是冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1/repo/` 里的 `crates/singlefs-core/src/allocator.rs` 与 `crates/singlefs-core/src/mounted_session.rs`，下文代码行号都是冻结副本的（这两份与主工作区逐字节相同，背景材料第二节核过）。禁读清单里的文件与目录没读。

## 复跑命令与模型文件

探针写好了，没编、没跑（腿不交任务）。复跑在仓根起，整条交服务：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model
```

模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/` 每个文件的 sha256（`sha256sum` 现算）：

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `879caa27846b1df02d9b5d96b28db0deb03c1a47f99e518be4a7a0a754bd1bb5` |
| `probe/core_probe_module.rs` | `c3e7bae91bac2ac40586c75b972b46d5c333959b6878dfd08836ea7d0b9c5eb0` |
| `probe/src/device.rs` | `81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3` |
| `probe/src/main.rs` | `358477d4da8f03a6d977eeb816cd96c8373be51bb97fd92d36304f422b6c3e74` |
| `probe/src/pool.rs` | `e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac` |
| `probe/src/shapes.rs` | `7be771881660956ce662886cd11d1eb94aa0c4f5263dd6d31f4de31f3adf25ab` |
| `probe/src/tally.rs` | `a0c9e155b0d4f10b05927b16b62e8804df6c877dae35817407ef54d24184a943` |
| `tools/patch_the_copy.py` | `90474fd8d710ad6a99ef76a92a83431b566acd3704a3d475c4e1383bc0b37bb1` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `89f4b906ec759eb15dae1a762295241291996459b4d8c69648630e8b2de701db` |

来历：`probe/src/pool.rs`、`probe/src/device.rs` 逐字节拷自 A/B 探针 `~/.local/share/singlefs/work/m4-gpugc-precheck-ab/model/probe/src/`（sha256 与源相同）；`tools/rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps`；`tools/rerun-from-a-clean-work-directory.sh` 逐字节拷自 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/`。其余是这条腿写的。

探针做什么：`rerun.sh` 把冻结副本 rsync 进运行目录的 `copy/`，`patch_the_copy.py` 在拷贝的 `mounted_session.rs` 里加三处钩子（每处原文在冻结副本里各命中 1 次，量的：用 python 对冻结副本数过 `conjunct 1 devices 1 series 1`、工作区成员那一行 `member 1`）：

1. 推串之前合取 7 那一判（甲，`mounted_session.rs:6175`）算完记一笔判没判出，按线程开关关掉时交 `None`；
2. 放不下的盘那一判（乙与式子，`mounted_session.rs:6113`）开关关掉时交空表；
3. 推一串抬 F（`mounted_session.rs:6322`）之前记每块盘的全空聚簇段数，推成之后再记一次。

驱动（`probe/src/shapes.rs`）：两块一样大的内存稀疏盘（A/B 那一档：单元区 1007 + 384 槽之外再多 0 / 64 / 256 / 1024 槽，按种子取），整理开关照产品值（`N₀` 是第一版的 1），同一次挂载里走 400 步随机历史（建文件整份写 1–32 单元、小覆盖写、整份重写 1–64 单元、大写 32–256 单元、unlink），每一步都经 `MountedSession::publish_user_change` 发。每一步之前拷一份池；产品那一臂报 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 时，在拷贝上发同一个改动两次：O 那一臂（先算整个关掉、被拒就推，写进了就是先算误拒）与重挂那一臂（正常卸载、再可写挂载、先算照冻结副本，写进了就是「卸载重挂」走得通）。按判出它的那一判分：`slot-pairs`、`formula`、`mixed`、`conjunct-before-any-series`（甲）、两个成员同时空 `both-empty`、同时有 `both-set`。误拒的底另数：产品那一臂推过一串之后写进的格（`base_published_after_a_series`）。310 个配置，种子 1 起。

内存与时间（推的，没量过）：编译照 A/B 探针那一份约 6 分钟、约 2 GiB（A/B 第二段报告引的 13514 读数，那是主 agent 的数，没复跑）；跑的那一段每线程同时活着至多三份池（产品、拷贝、O 或重挂那一份），两块盘每份几十 MB 的写过的扇区，31 线程估 4–10 GiB；挂钟先跑一批（31 个配置）估全量，估出来超过 40 分钟退 3、不跑余下的，日志里 `oracle ESTIMATE …` 一行是估算，超了就按那一行缩 `--configurations` 或 `--steps` 再交。每个配置跑完往标准错误打一行 `PROGRESS`。崩溃状态这一题不涉及（探针不枚举崩溃点）。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| T2-1 甲：推一串会不会改全空段数 | **打中（读码推的，探针待跑）** | 甲与 `refusal_is_short_of_space` 的注释都说推了这次写也被合取 7 拒，理由只讲了回收 defer；推的那一串空发布自己换下上一版的固定点单元，释放让一段的未释放计数归零时全空段数加一。产品 `N₀`=1 下甲只在全空段为 0 时判出，那时提交内生块走开放段或回落，回落落点散在数据段里，换下就可能腾空一段 | `mounted_session.rs:6174`、`:2120`；`transaction.rs:15728`；`allocator.rs:902`、`:857`、`:2828`；探针 `misrefused_conjunct_before_any_series`、`series_*_crossed_the_floor` |
| T2-2 甲不误拒的证据 | **打中（量的）** | A/B 两族都量不出甲误拒：drawn 族 O 那一臂推过一串之后写进 0 格（误拒的底是 0），series-admits 族甲一次都没判出；「`fired_by_conjunct7=315 published_eaten_vs_as_coded=0`」不说明甲不误拒 | A/B 产物 `drawn.out` 第 18868、18872 行，`series-admits.out` 第 12400 行 |
| T2-3 甲的射程宽一形 | 成立（读码），A/B 没罩 | 判「还一串没推」用的是这次调用推过几串（`mounted_session.rs:6242`），开头走空发成的几步不算；这一形判出时盘上有这次调用写的东西，与 C1003 记的「开头走空」同形、甲多给了它一条入口 | `mounted_session.rs:6193`、`:6242`、`:3845` |
| T2-4 改 size、建快照不先判 | 成立：漏判是有意的、取照推那一边 | `Truncate` 变长与只改记录、建快照带普通分配时合取 7 判（`transaction.rs:13335` 起），甲把它们归「不先判」，结局是照推、推完由合取 7 拒，不误拒 | `mounted_session.rs:6153`–`6165` |
| T2-5 先乙后甲与报的成员 | 部分打中（推的）：两样同时成立时合取 7 那一份丢掉；产品里没有读这两个字段的调用方 | 「先整理」的信号现在有三种样子（`Publish` 带合取 7 成员、新字段有值、两样同时成立时只报 `devices_short`），注释说调用方分得出两者，第三种分不出 | `mounted_session.rs:2120`、`:6228`–`6250`；冻结副本 `crates/` 里 `grep` 读字段的只有用例 |
| T2-6 判的是不是合取 7 写那一刻读的数 | 成立（同一个函数、同一个 `N₀`）；推了之后的那一刻不是同一个数（见 T2-1） | 甲只能接在落得下那一判或固定点取不到落点之后判出：合取 7 排在它们之后、式子之前，式子拒的时候合取 7 已过 | `transaction.rs:13432`、`:13485`、`:13538` |
| T3-1 `publish_user_change` 与走空传的类型 | 成立（读码）；探针在 `publish_user_change` 上补了真池取样（待跑） | 两处传的都对得上被拒的那一次发布 | `mounted_session.rs:3845`、`:6424` |
| T3-2 甲那条真池用例的 `N₀` | **打中（推的）** | 用例在全空段多的池上把 `N₀` 抬到最大值加一，那里还有全空段可开，提交内生块走开段与 bump；产品 `N₀`=1 下甲判出时一个全空段都没有，提交内生块走回落——T2-1 那一形只在后一种下有，用例罩不到；用例也没断言「推了照旧被拒」 | 改动全文第 1554–1647 行；`allocator.rs:2828` |
| T3-3 新位图与「记账是事务的副产品」 | 不成立（没违） | 每改一槽增量维护，读一次 O(这次挂载的聚簇段数)，不扫单元区；位图内存随单元区槽数线性涨，与已有的逐槽位图同阶 | `allocator.rs:783`、`:873` |
| T1-a 增量计数每条路都跟上 | 成立（读码） | 置、清分配位的只有 `mark_allocated` 与 `mark_reclaimed`，改未释放记录的三处（分配、释放、复活）都过 `note_slots_under_an_unreleased_record`；隔离、扣住只动位不动记录 | `allocator.rs:1174`、`:1080`、`:903`–`933` |
| T1-b 乙少报出路 4801 格 | **打中一半（推的）**：丢的那一条不一定走不通 | 卸载重挂把这次挂载的聚簇段集清空（`allocator.rs:2121`），聚簇段里空着、defer 里、扣住的槽对重挂之后给用户数据；乙的上界不算聚簇段里的槽对。4801 格里走不走得通看聚簇段里放得开的槽对够不够补上差额，A/B 照出路跟了 295 次写进 0 | `allocator.rs:2121`、`:783`；探针 `remount_published_*` |
| T1-c 乙在槽对那一判上误不误拒 | 没量到（A/B 两族都没底）；探针 `misrefused_slot_pairs` 与底待跑 | 读码没找到上界小于推串真放得出的路（推串放不出聚簇段与 `R`，上界只会多算） | `allocator.rs:1200`–`1224` |
| T1-d 同一次准入里后一串回收前一串换下的固定点单元 | 推不出可达（读码） | 回收要释放代不高于 max(F, 环里最旧有效根)：F 不越过空发布，环里有效根齐全时一次准入至多 8 次发布追不上 12 槽的环；环里有不作数的槽时可能，没造 | D16 已定项 1「准入」那一行（`.claude/kb/decisions/16-发布语义.md`），背景材料附录 |
| 检查单：走不到的理由 | 成立 | 产品里两个调用者只传 `allocator.cluster_segments()`，段起点按单元区起点加段号乘 64 算 | `allocator.rs:2910`、`:1234`–`1252` |
| 检查单：类型与名字 | 不成立（两处小的） | 槽数、槽对数同是裸 `u64`；「二选一」类型没挡，产品里两条构造路各填一样，用例辅助函数造出了两样同时空的值 | `mounted_session.rs:6231`、`:6244`、`:8938` |
| 检查单：不兼容旧代码与旧格式 | 成立 | 两份补丁没碰盘上字节、没加版本分支 | 改动全文 |
| 检查单：不新建判定库 | 成立 | 没有新目录、TSV、sqlite、RocksDB；探针也不写判定 | 改动全文 |
| 检查单：多机可用 | 成立 | 新用例与探针都在内存里现造，路径从参数取 | 改动全文；`tools/rerun.sh` |
| 检查单：测试优先多线程 | 不成立（轻） | 两条 600 步伪随机历史各一个种子、单线程，注释里没写为什么不并行；真池两条是单场景 | 改动全文第 341–381 行 |
| 检查单：带 --item 的命令与项 | 成立 | 三项在 `.claude/gate.d/crash-point-replay-items.tsv` 第 18、70、76 行 | `grep` |
| 检查单：新崩溃流多罩了什么 | 成立（不要新流） | 头一次就判出时一个写都不发，崩溃状态与上一次发布之后相同；走空那一形的写由已有的 unlink 流罩着（推的） | `mounted_session.rs:6222` 起 |

## T2　甲的射程与次序（主攻）

### T2-1　推一串会改全空段数：「推了这次写也是被合取 7 拒」不由代码推出（读码推的，探针待跑）

甲不推的理由写在它自己的文档里：

「/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用），推了这次写也是被合取 7 拒。只读分配器（内存）。」（`crates/singlefs-core/src/mounted_session.rs:6174`）。理由只讲了回收 defer。推的那一串是几次空发布，每一次都照发布路径「释放先于分配」：

「// 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。」（`crates/singlefs-core/src/transaction.rs:15728`）。空发布重写固定点单元（树表、分配记录树改到的那几个节点、记账树；`crates/singlefs-core/src/mount.rs:5880` 的计划文档，抬 F 那一串的计划在 `crates/singlefs-core/src/mount.rs:5738`），上一版的这几个单元被换下，经 `release` → `mark_released`，记录改成已释放：

「/// 记录改写成已释放，按记录口径它的槽从此不罩在未释放的记录下（[`Self::empty_segments`]）。」（`crates/singlefs-core/src/allocator.rs:902`）。`note_slots_under_an_unreleased_record` 逐槽减一段的未释放计数，减到 0、又是整段时全空段数加一（`crates/singlefs-core/src/allocator.rs:857`）。所以推一串可以让全空段数变多；变多到不少于 `N₀`，重判时合取 7 就放行。

造法（推的历史，每一步的许可）：

1. 整理开关照产品值，`N₀` 是第一版的 1（`crates/singlefs-core/src/transaction.rs:13459` 那一段注释）。甲只在某块盘全空段为 0 时判出。全空段按记录口径数，「被分配过、只剩 defer 槽」的段也算全空；所以全空段为 0 时每个整段都至少罩着一条未释放记录，也就没有一个整段的分配位全清（开段要它，`allocator.rs:1234`–`1252`），提交内生块只能在开放段里 bump，开放段装不下就回落（`allocator.rs:2828` 的文档：没有全空段就回落，先取这次重写集里的叶、再取空槽最多的叶，取最低的空槽）。回落落点在数据段里（验证包 `commit_generated_units_fall_back_when_the_segment_runs_out.rs:167` 那一段手算的落点就在数据单元旁边）。
2. 一个文件 X 被 unlink、走空的各批把它的数据单元放进 defer：X 的数据所在的数据段 D 只剩 defer 槽与几个空槽，按记录口径全空，全空段数 ≥ 1。
3. 之后一次用户写 W：写之前合取 7 判的是 ≥ 1，放行；W 的提交内生块回落，最后几个（树表、分配记录树的根与改到的节点、记账树——先叶后根、再是记账树、映射树、树表，同一份用例文档那一行）落进 D 的空槽，把 D 的空槽用完。D 重新罩着未释放记录，全空段数回到 0。
4. X 释放之后又有过 ≥ 4 次改用户可见状态的发布，抬 F 的上限越得过 X 的释放代（D16 已定项 1「抬 F 的上限」那一行）。
5. 用户写 U 要的数据单元多于段外此刻没挡的槽对：落得下那一判先拒（`transaction.rs:13432`，排在合取 7 之前）。乙读的上界算上 D 里 X 那几对 defer，够需求，不判出；甲读到全空段 0 < 1，判出，一串不推。
6. 不先算的一臂：推一串，第一次空发布换下 W 留在 D 里的树表、根、记账树（都是空发布要重写的）；D 里没有空槽了，新的落点落到别的叶里；D 又只剩 defer 与已释放的槽，全空段数 0 → 1；这一串回收 X 的 defer（扣到生效、生效后放开）。重判 U：落得下那一判放行，合取 7 读到 1 ≥ 1 放行，式子放行——写进。产品那一臂在第 5 步拒了：**推了写得进而不推**。

不确定的那一环：第 3 步要 W 留在 D 里的只有空发布会重写的单元。W 的映射树、inode 树、extent 树节点空发布不碰，它们要是也回落进 D，D 推完照旧罩着未释放记录、这一形不成。这一环靠探针的取样回答，读码定不了。

打中之后四问：

- 分不分辨臂：分。只有甲那一臂在第 5 步不推；现行（甲之前）那一臂推了、写进。
- 系统当时看不看得到判别它的东西：看得到。推串本来就先在分配器的拷贝上整串预演（D16 已定项 1「抬 F 那一串」那一行），预演之后的全空段数就是重判时合取 7 读的数；甲选了不读它。
- 满足的是判据字面的哪一个分句：改法甲的字面（「写之前那一刻全空聚簇段已少于 N₀，也一串不推」）照写了，没违；违的是它的前提，即 D16 已定项 10 射程「推抬 F 放得出的全放出来也放不下就一串都不推」那一句（`.claude/kb/decisions/16-发布语义.md:276`）——甲判出的那一格推了放得下。
- 改法在打中的格上中不中：跑前没有条款给改法。这条腿提的两个，都没实现、没量（推的，被攻过零轮）：甲改读预演之后的全空段数（在拷贝上整串预演之后再判合取 7，与推串那一段同一个预演），这一格不再误拒；或撤掉甲、只留乙，这一格照推写进，丢掉甲省下的白推（A/B 产物里甲多挡的白推 `precheck_saved_a_vain_series` 是 1799 对 1520，量的，drawn 族）。

同一个前提还压着 `refusal_is_short_of_space` 把合取 7 被拒当「不是空间不够」、原样交回不推（`mounted_session.rs:2120` 那一行注释，不在这一批改动里）。那一处被拒的写在 T2-1 这一形下同样推了写得进；它与改法甲、现行两臂一起出局，属「打中不分辨臂」，另立一笔账，不拿它判甲。

探针读法：`misrefused_conjunct_before_any_series` 大于 0 就是坐实；`series_product_raised`、`series_oracle_raised` 大于 0 说明推串会把全空段数推高（前提不成立），`series_oracle_crossed_the_floor` 大于 0 说明推高到越过 `N₀`。三个都是 0、而 `fired_by_conjunct_before_any_series` 不小（几十格以上）时，这一形在这一族里没出现，照「没打中」记、要再抽一次。

### T2-2　「甲误拒 0」的底是 0（量的，A/B 产物）

A/B 第二段的产物在 `~/.local/share/singlefs/work/m4-gpugc-precheck-ab-2-run/out/`（主 agent 交的任务 13802 跑出；我读的是产物文件，没复跑）。逐字段读（量的）：

- `drawn.out` 第 18868 行（`arm=conjunct7-first`）：`fired_by_conjunct7=315`、`conjunct7_short_at_first_refusal=315`、`published_eaten_vs_as_coded=0`，同一行 `oracle_published_after_a_series=0`、`c_published=0`；第 18872 行（`arm=never`，O 那一臂）`c_published=0`。drawn 族的探测点上连 O 那一臂都一格没写进：拿它数误拒，底是 0。
- `series-admits.out` 第 12400 行（`arm=conjunct7-first`）：`oracle_published_after_a_series=5465`，但 `conjunct7_short_at_first_refusal=0`、`fired_by_conjunct7=0`：有底的那一族里甲一次都没判出。

所以记录第 1019 行那句「乙、甲误拒 0」对甲同样是没底的读数，背景材料只给乙写了「没量到」。另：A/B 的甲那一臂只在「这一次调用里头一次被拒的是用户那一次写」时判（A/B 探针 `core_probe_module.rs` 里 `set_first_refusal_is_the_user_write` 那一个开关），产品代码的甲另判走空之后那一形（T2-3），那 315 格不罩它。

四问：分辨臂（只问甲那一臂的读数）；看得到（O 那一臂就是判别子）；满足的是「误拒没量到」不是「误拒了」；改法是给甲一个有底的族，探针 `base_published_after_a_series` 与 `misrefused_conjunct_before_any_series` 两个数一起读。

### T2-3　甲的射程宽一形：走空发成几步之后头一次被拒也判（读码）

判「这次调用还一串没推」读的是 `published_in_this_change.floor_raises.len()`（`mounted_session.rs:6242`，判在 `:6193`）。`publish_user_change` 的本体开头队列不空时先走空（`mounted_session.rs:3727`），走空发成的每一步进 `versions_published`、不进 `floor_raises`；走空没推串而这次改动头一次被拒，甲照判。这一形读的全空段数是走空之后的，判法本身没错；两处要记：

- 判出时盘上有这次调用写的东西。D16 已定项 10 射程「头一次被拒就判出时盘上逐字节不变」那一句（`.claude/kb/decisions/16-发布语义.md:276`）在这一形下字面不成立，与 C1003 已记的「开头走空」同形（`.claude/kb/checks-owed.md:642`）；甲给这一形多了一条入口，C1003 那一条的射程要连甲一起写。
- 甲还经推环的预演走：被宽护卡住要出路时，`rehearse_the_push_then_the_remainder_of_the_change` 预演推环之后照 `MountedSession::publish_the_change_raising_the_floor_for_space` 重发这次改动（`mounted_session.rs:1890` 起，函数头在 `:1844`），那里面就是 `:3845` 那一处甲。背景材料说调用处三处，这是第四个入口：甲在预演里判出，推环那一条出路就不列；T2-1 那一形成立的话，这一条也会错丢。

### T2-4　改 size 与建快照不先判（成立）

`of_the_user_change`（`mounted_session.rs:6153`–`6165`）把 `Truncate`、`CreateSnapshot` 归「不先判」，没有 `_ =>`。发布路径上 `Truncate` 变长与只改记录（`SizeChangeWithoutAnIntent`、`PartialWrite`）、建快照都算前台写入，合取 7 判（`transaction.rs:13335` 起的 `reach_of_the_empty_cluster_segments_conjunct`）。甲不先判它们，结局是照推、推完由合取 7 拒，多花一串、不误拒。反方向（甲归前台写入、合取 7 却免）没找到：七个前台成员在发布路径上都带 `plan.file` 或新 inode 记录，被空间拒的又一定带普通分配（落得下那一判只对带普通分配的判，`transaction.rs:13411` 一段）。

### T2-5　先判乙再判甲：两样同时成立时合取 7 那一份丢掉（推的）

`judge_before_pushing_a_floor_raise_for_a_refusal`（`mounted_session.rs:6222`）`devices_short` 非空就交回、`empty_cluster_segments_below_the_floor` 填 `None`（`:6231`）。冻结副本的 `crates/` 里读这两个字段的只有用例（`grep -rn 'devices_short\|empty_cluster_segments_below_the_floor' crates` 命中验证包与运行时包的测试，产品代码没有）；验证包 `history.rs` 只按 `last_refusal` 的成员名记。所以今天没有调用方读错。

要记的是信号的样子：`refusal_is_short_of_space` 那一行注释说原样交回合取 7 的成员是为了让「调用方分得出「先整理」与「真没空间」」（`mounted_session.rs:2120`）。甲之后「全空段少于 `N₀`」有三种报法：落得下那一判过了、合取 7 自己拒，交回 `Publish` 带 `EmptyClusterSegmentsBelowTheFloor`（验证包 `compaction_runs_stop_and_discard_after_the_publish.rs:690` 钉的那一形）；落得下那一判先拒、乙不判出，交回新字段有值；落得下那一判先拒、乙也判出，只报 `devices_short`，「先整理」那一半丢掉。第三种给调用方看的是「真没空间」，而整理有可能补上段外槽对（推的，没量）。改法（推的，零轮）：两样都判、两个字段都填，类型上不再二选一。

### T2-6　判的是不是合取 7 写那一刻读的同一个数（成立，推之前）

同一个读法（`device_map.empty_segments()`）与同一个 `N₀` 来源（整理开关），甲在 `mounted_session.rs:6200` 起，合取 7 在 `transaction.rs:13472` 起。次序上合取 7 排在落得下那一判（`transaction.rs:13432`）与固定点取落点之后、式子（`transaction.rs:13538`）之前：式子拒的格合取 7 已经放行，甲读同一个分配器必判不出。所以甲只接在落得下那一判或固定点取不到落点之后判出。推之前没有别的东西改这个数；推了之后改不改见 T2-1。

## T3　覆盖与报（主攻）

### T3-1　`publish_user_change` 与走空传哪一种：传得对，缺的是真池取样

- `publish_user_change` 本体里，被拒的是 `publish_the_change_once(…, change, …)` 那一次（`mounted_session.rs:3777` 起的循环），推串时传 `of_the_user_change(change)`（`:3845`），对得上。
- 本体开头走空（`:3727`）与改动发成之后走空（`:3793`）都经 `publish_the_queue_until_it_is_empty_raising_the_floor_for_space`，那里写死「不先判」（`:6424`）；走空被拒的是删除类的一步，发布路径上免合取 7（`transaction.rs:13335` 起），对得上。
- 推环预演（`:1890`）复用同一个本体，同一处传法。

所以不会传错类型（读码）。没有真池用例的后果在 T2-1：那一形只在 `publish_user_change` 与 `accept` 共用的「落得下那一判先拒、全空段为 0」上出现，`accept` 那条真池用例在另一种池上（T3-2），两处都罩不到。探针整段走 `publish_user_change`，开头走空那一形与「走空之后甲判出」的格另数（`conjunct_fired_with_versions_published`）。

### T3-2　甲那条真池用例的 `N₀` 罩不到产品 `N₀` 下的形（推的）

用例 `an_accept_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series`（改动全文第 1559 行）的文档写着这个池「单元区大半空着，全空段多」（改动全文第 1556 行），`N₀` 抬到每块盘全空段数最大值加一。两处与产品不同：

1. 这个池上有全空段，提交内生块开新段、在开放段里 bump；产品 `N₀`=1 下甲判出时一个全空段都没有（T2-1 第 1 步），提交内生块回落进数据段。T2-1 那一形靠回落落点，只在后一种池上有。
2. 用例文档的判别力一句写「推串之前不判合取 7 时推一串抬 F，推完照旧被拒」（改动全文第 1557 行），用例断言的是一串没推、盘上不变、读得出旧内容，没断言推了之后照旧被拒；在用例那种池上推完被拒成立（推串在开放段里 bump，换下的固定点单元和新的在同一段），换到产品那种池上不一定。

改法（推的，零轮）：用例改成产品 `N₀`、造到全空段为 0 而写之前有一段只罩着当前这一版的回落固定点单元，加一臂「先算关掉」照推、断言写不写进；或者干脆用探针的 O 那一臂做对照。

### T3-3　新位图与「记账是事务的副产品」：没违

`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 管的是运行时决策路径「被问到那一刻有没有答案」。新数随每改一槽增量维护（`allocator.rs:873` 起），读的时候是总数减去这次挂载的聚簇段那几段（`allocator.rs:783`），O(聚簇段数)、不扫单元区，与已有的 `unblocked_slot_pairs_outside_the_cluster_segments` 同一个读法。逐槽一位随单元区槽数线性涨，与已有的分配位、隔离位、扣住位三份逐槽表同阶。

## T1　乙的上界（顺带）

### T1-a　增量计数每条路都跟上（读码，成立）

在冻结副本 `allocator.rs` 里置分配位的只有 `mark_allocated`（`:1174`），清分配位的只有 `mark_reclaimed`（`:1080`）；改未释放记录的三处——分配（`mark_allocated` 末尾）、释放（`mark_released`，`:903`）、复活（`mark_resurrected_out_of_the_defer_queue`，`:933` 那一段）——都经 `note_slots_under_an_unreleased_record`，后者逐槽调 `note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts`（`:842`）。回收（defer → 空闲）不改记录，扣住、隔离只动位。释放之前核出校验和对不上的那一份记录留在已分配、不调 `mark_released`，位留着「罩着」，对：它放不开。减法只在「这一对原来算着」时做，不会下溢。

### T1-b　乙少报出路 4801 格：丢的那一条不一定走不通（读码推的）

4801 格是现行推一串后报被宽护卡住、出路列「正常卸载再可写挂载」，乙先算就判放不下、报的成员里没有出路字段。卸载重挂比推抬 F 多放出一样东西：

> /// 这次挂载开过的聚簇段的起点，开过就一直算数（只在内存，重开后按 D3（空间分配） 已定项 10 ①「挂载后新开一段」重来）。

（`crates/singlefs-core/src/allocator.rs:2121`）。用户数据的落点排除这次挂载的全部聚簇段（`allocator.rs:1209`–`1211`），乙的上界同样排除它们（`allocator.rs:783`）；重挂之后这一组清空，聚簇段里空着、defer 里、扣住的槽对都给用户数据。C 那一圈的隔离、defer（卸载把 F 抬到现行那一版）乙的上界本来就算上了。所以乙判放不下，只说明推抬 F 放不下，不说明卸载重挂放不下；差额正好是这次挂载的聚簇段里放得开的槽对（`R` 之外）。这一项在早先「聚簇段那一项」那一轮撤回过（那一项会让推串白推，`research/prompts/m4-gpugc-floor-raise-precheck-code-r3-main-verification.md` 第 27 行），乙把现行那一份松的上界顺带挡掉的出路里就有它。

4801 格里走不走得通：A/B 照出路跟了乙 295 次、现行 285 次，都写进 0（记录第 1019 行，主 agent 的读数，我没复跑）。那一族的盘上这次挂载开的聚簇段少，补不上差额；读码推不出 4801 格全走不通。探针在每一格乙判出的格上都跟一次重挂（`remount_published_slot_pairs`），不限宽护那一形。

### T1-c　乙在槽对那一判上误不误拒（没打中，读码；探针待跑）

推一串放得出的只有 defer、扣住与随推串写掉根环槽解开的隔离，三样都不罩在未释放记录下、都在上界里；推串只会多开聚簇段、不会少，用户数据还另排除 `R`（`allocator.rs:1212`–`1213`），上界只会比真放得出的多。读码没找到上界小于推串真放得出的路。探针 `misrefused_slot_pairs` 连同底一起读。

### T1-d　同一次准入里后一串回收前一串换下的固定点单元（推不出可达）

这些单元在判的那一刻罩在未释放记录下，不在上界里。推串换下它们之后，回收要释放代不高于 max(F_生效, 环里最旧有效根)；抬 F 的上限按非空根数（D16 已定项 1「抬 F 的上限」那一行），空发布不算，F 追不上它们；环里最旧有效根要再推满一环才越过它们，一次准入至多 8 次发布、环长 12。环里有不作数的槽（读坏、被抛弃）时有效根少、最旧的那一条更新，可能追得上；没造。

## 派发「关的是哪几条」逐条回

1. 乙在槽对那一判上误不误拒：读码造不出（T1-c，试过的形在「没打中的形状」）；探针给出分母——`fired_by_slot_pairs` 是乙在槽对那一判上判出的格数，`misrefused_slot_pairs` 是其中 O 那一臂写进的格数，`base_published_after_a_series` 是这一族误拒的底。三个数待主 agent 跑。
2. 乙少报出路 4801 格：丢的是「卸载重挂」，读码它在聚簇段里放得开的槽对补得上差额时走得通（T1-b）；A/B 那一族跟了 295 次写进 0。探针 `remount_published_slot_pairs`、`remount_published_formula` 待跑。
3. 甲射程宽一形、先乙后甲、改 size 与建快照不先判：宽一形判法不误拒但让 D16 射程那一句与 C1003 多一条入口，另经推环预演走（T2-3）；先乙后甲不误拒、丢合取 7 那一份报（T2-5）；改 size 与建快照漏判是有意的、不误拒（T2-4）。另打中一处背景材料没列的：甲的前提「推了也被合取 7 拒」不由代码推出（T2-1），A/B 量甲误拒的底是 0（T2-2）。
4. `publish_user_change` 与走空没有真池用例、甲用例的 `N₀`：两处不会传错类型（T3-1）；测试 `N₀` 那种池罩不到产品 `N₀` 下甲判出时的回落形（T3-2），T2-1 恰在那一形里。
5. 检查单八格：见下一节。

## 检查单八格

- **走不到的理由二次查实：成立。** 断言在 `pairs_inside_the_cluster_segments` 里（`allocator.rs:514`）。产品里两个段外计数的调用者只传 `allocator.cluster_segments()`：准入落得下那一判（`crates/singlefs-core/src/admission.rs:996`）与先算（`mounted_session.rs:6063`）。这一组只在开段那一处插（`allocator.rs:2910`），插的是 `OpenEmptySegment` 的答案，来自 `lowest_empty_segment_outside_the_region`：单元区起点加段号乘 64（`allocator.rs:1249`–`1252`），各盘一致才开。验证包与实验装置的调用者传空集或分配器自己那一组（`crates/singlefs-implementation-experiments/src/bin/e181_landing_check_fixed_point_and_mount_lower_bound/aligned_pair_shortage_walks.rs:270` 一类，`grep` 列过）。减法：段外数 = 总数减一组不重复的段的分段数之和，不下溢；增量那两处只在一对的状态真翻过去时加减（`allocator.rs:842` 起），不下溢。
- **类型与名字：不成立（两处小的）。** 槽数与槽对数同是裸 `u64`（`SlotsAFloorRaiseCouldRelease` 里 `deferred`、`isolated` 是槽、`slot_pairs_once_everything_is_released` 是对，换算靠乘一个数据单元的槽数）。`devices_short` 与 `empty_cluster_segments_below_the_floor` 二选一只靠构造路：产品里两条（`mounted_session.rs:6231` 填前者、`:6244` 填后者）守住了，用例辅助函数（`:8938`）造的值两样同时空——类型允许、产品路走不到。`of_the_user_change` 没有 `_ =>`。新 `expect("单元区槽数")`（改动全文第 73 行）只写了是什么数，没写为什么装得下，与旁边已有的 `expect("段数")` 同一写法。
- **不兼容旧代码与旧格式：成立。** 两份补丁不动盘上字节、没有版本分支（读改动全文）。
- **不新建判定库：成立。** 没有新目录、TSV、sqlite、RocksDB；探针只往运行目录写输出，不写判定。
- **所有任务多机可用：成立。** 新用例在内存里现造；探针的路径都从参数取，运行目录在工作根下。
- **测试优先多线程：不成立（轻）。** `.claude/rules/implementation-workflow.md@测试与崩溃检测优先多线程` 要「不能并行的写明为什么」。`allocator::tests` 两条各走一个种子的 600 步历史（改动全文第 247–381 行），注释只说做了什么，没写为什么不按种子切片并行；一条历史本身次序就是被测对象，可以并行的是多个种子。真池两条是单场景。
- **带 --item 的命令与项：成立。** 三项在 `.claude/gate.d/crash-point-replay-items.tsv` 第 18、70、76 行（`grep -n` 现查）；这一批没有新项。
- **新崩溃流多罩了什么：成立（不要新流）。** 乙、甲头一次就判出时一个写都不发（`judge_before_pushing_a_floor_raise_for_a_refusal` 的文档，`mounted_session.rs:6222` 那一段），崩溃状态与上一次发布落盘之后相同，已有的流罩着；走空之后甲判出那一形，盘上的写是走空那几步，unlink 那几条流罩着（推的，没逐流比）。

## 这一腿提的改法各修哪一格

都是这条腿提的，没实现、没在副本上跑，被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 甲改读整串预演之后的全空段数（推串本来就先在拷贝上预演） | T2-1；T2-3 推环预演那一条随之 | 推的，待主 agent 跑探针定 T2-1 有没有 |
| 撤掉甲、只留乙 | T2-1（不误拒）；T2-5 不再有第三种报法 | 推的；代价是甲多挡的白推（A/B 产物 drawn 族 1799 对 1520，量的） |
| 两样都判、两个字段都填 | T2-5 | 推的 |
| 甲那条真池用例换产品 `N₀`、造全空段为 0 的回落形，加先算关掉的对照臂断言写不写进 | T3-2 | 推的 |
| 判放不下时另算「这次挂载的聚簇段里放得开的槽对」，补得上差额就在报里列卸载重挂 | T1-b | 推的；早先「聚簇段那一项」是加进上界、让推串白推，这一条只进报、不进推不推的判 |
| C1003 与 D16 已定项 10 射程那一句连甲那一形一起写 | T2-3 | 推的（文字） |

## 要主 agent 跑的命令

一条（在仓根 `/home/fy5090/code/singlefs` 起，不给 `--cwd`）：

task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model

- 包装那一层先删掉运行目录 `…/m4-gpugc-precheck-b-a-code-r1-opus-probe-run`（只认工作根下、名字以 `-run` 收尾的，别的退 2），再调同一个模型目录里的 `tools/rerun.sh`；服务撞内存上限重排同一条命令时照样能重跑。
- 判过：末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`。日志里依次要有 `PATCH_THE_COPY … probe_sources=5`、`name=build done binary=…`、`name=exit_code oracle=0`、`oracle ESTIMATE …`、`oracle WALL_SECONDS …`、`oracle PROBE_RESULT configurations=310 …`、四行 `name=self_check ok …`、`name=self_checks bad=0`、末行 `name=rerun_done …`。有 `✗` 就是没过。
- 退 1、日志是 `build.log` 的末 80 行：编不过（探针没让编译器核过），新派同立场的腿照报错改 `probe/`。退 1、`name=exit_code oracle=1`：自证不合，看 `✗ name=self_check` 那一行。`name=exit_code oracle=3`：估时超过 40 分钟，日志里 `oracle ESTIMATE …` 一行给了先行一批的秒数，照它把 `rerun.sh` 里的 `--configurations 310` 或 `--steps 400` 缩下来（新派腿改）。
- 要读的：`oracle PROBE_RESULT` 一行的 `fired_by_slot_pairs`、`misrefused_slot_pairs`、`fired_by_conjunct_before_any_series`、`misrefused_conjunct_before_any_series`、`conjunct_fired_with_versions_published`、`remount_published_slot_pairs`、`remount_published_formula`、`base_published_after_a_series`、`series_product_raised`、`series_oracle_raised`、`series_oracle_crossed_the_floor`、`fired_both_members_empty`、`fired_both_members_set`；有 `oracle EXAMPLE MISREFUSED …` 与 `oracle EXAMPLE REMOUNT_PUBLISHED …` 的逐行读（种子、步号、写之前每块盘的全空段数、O 那一臂每一串前后的全空段数）。整份输出在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-opus-probe-run/out/oracle.out`。
- 跑完新派同立场的腿，带「承接：research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-output.md」与任务号，读 `task-server/task log <号> --all` 与上面那份输出，接着判 T2-1、T1-b、T1-c。运行目录里的 `copy/`（冻结副本的拷贝，服务没给绝对的编译目录时连编译目录）用完由主 agent 删。

## 没打中的形状

- 乙的上界算小（T1-c）：逐样核了推串放得出什么（defer、扣住、写掉根环槽解开的隔离）、放不出什么（聚簇段、`R`、罩在未释放记录下的），以及推串会不会让聚簇段变少（不会，那一组只增）；多块盘时上界按盘算、真分配要各盘一致，只会更少。造不出上界小于真放得出的。读码，分母是这几条路，没有样本数。
- 同一次准入里回收前一串换下的固定点单元（T1-d）：环里有效根齐全时追不上；环里有不作数的槽那一形没造。
- 甲归前台写入、合取 7 却免（T2-4 反方向）：七个前台成员逐个对发布路径的 `reach_of_the_empty_cluster_segments_conjunct`，没有不判的；被空间拒的又一定带普通分配。
- 甲读的数与合取 7 读的数不是同一个（T2-6 推之前那一段）：同一个函数、同一个 `N₀` 来源，中间没有别的写。
- `publish_user_change`、走空、`accept`、推环预演四处传错类型（T3-1）：四处逐个对了被拒的那一次发布。
- 两个字段同时有值：产品里两条构造路各填一样，造不出；同时空只在用例辅助函数里。
- 探针那一族（310 个配置、每个 400 步、四档盘宽）是取样，跑出来某一格是 0 只算这一族一次抽样没撞上；照规则要再抽一次才记「没打中」。

## 这条腿自己的限度

- T2-1 是读码推的一条链，第 3 步（W 留在 D 里的只有空发布会重写的单元）读码定不了，全靠探针取样；探针没让编译器核过。
- 探针是随机历史，不是照 T2-1 的六步造的：撞不上那一形不等于那一形不存在。要定的话下一条腿照六步写一个确定的造法（产品 `N₀`、回落、unlink 一个文件、用完它那一段的空槽）。
- 探针只走 `publish_user_change`，不走 `accept`；重挂那一臂是正常卸载再挂，不是崩了再挂；没造读故障的宽护（C1003 那一形）。
- 探针每一步拷一份池，内存与挂钟都是推的；O 那一臂是「先算整个关掉」，它比「只关甲」多推几串，误拒按 O 写进算，与 A/B 同一个定义。
- A/B 产物里的数（T2-2、改法表那一行）是主 agent 任务 13802 的产物，我读了文件没复跑；记录第 1019 行的跟出路读数是主 agent 写的，我没在产物里逐个找到出处行。

## 没做什么

- 没往统一任务服务交任何任务、没起 cargo、没跑门禁与自证；判决要引的数都等主 agent 跑上面那一条命令（腿报告里没有一个探针读数）。
- 不判正推那几格、不替主 agent 采纳；副本上的数（跑出来之后）不算入库装置上的数。
- 没改仓里任何别的文件：只写了这份报告与模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/`。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-opus` 里没建仓副本与编译目录（只有认领留下的 `.owner`、写者锁与 `cite-check.py` 的输出 `cite-check.txt`），没有要删的。
