# m4-gpugc-precheck-b-a-code-r2 攻方腿（Opus）报告（2026-11-12）

批次 `records/2026-10-19-里程碑四GPU垃圾回收开工.md#130`。立场：假设「第一轮的修补改得干净、乙甲在没看过的几处都不出错」是错的，去造历史与输入打它。主攻 T1、T3，检查单八格逐格回。被判的是冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo/` 的 `crates/singlefs-core/src/allocator.rs` 与 `crates/singlefs-core/src/mounted_session.rs`。下文引代码写仓内相对路径：这两份与 `transaction.rs`、`admission.rs`、`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs`、`crates/mutations.tsv` 在主工作区与冻结副本里逐字节相同（`sha256sum` 现算，两边各六份对上），行号两边一样。禁读清单里的文件与目录没读；同目录下另一条腿的模型目录也没读。

## 复跑命令与模型文件

探针写好了，没编、没跑（腿不交任务）。在仓根起，整条交服务：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model
```

模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/` 每个文件的 sha256（`sha256sum` 现算）：

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `f45aed5da6126bd3b7e8f220e0c29bc2736d9a319f65d6dcb99a55a6204327f6` |
| `probe/core_probe_module.rs` | `86928a59d589e645ac9401f453b1f687749ee7bf45e0018af88866dd51caa4de` |
| `probe/src/device.rs` | `81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3` |
| `probe/src/main.rs` | `b9fe4bdc7608acca52b0a1bc27f1597a75fc6d24572e595d6f0695b78d04032d` |
| `probe/src/pool.rs` | `e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac` |
| `probe/src/shapes.rs` | `923647b20430b1851fee5b08c43609574a165e9812dad805f34694cce167fd5d` |
| `probe/src/tally.rs` | `f2b6e02be3598a75c598cbacc48402cd8741697093bafbfb796b53f3ee3dca0f` |
| `tools/patch_the_copy.py` | `05ddc176be772be661ac151064ae8642bbc94e4c2014b3f886aaf55b12b6b3f8` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |
| `tools/rerun.sh` | `24408f26d6c532a26d5aa59d05818a54fdd2b814838ec592e1900227813aed76` |

来历：从第一轮攻方模型 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/` 拷来改。`device.rs`、`pool.rs`、`tools/rerun-from-a-clean-work-directory.sh` 与第一轮逐字节相同（sha256 同上表，第一轮报告里也是这三个值）；`patch_the_copy.py`、`Cargo.toml`、`core_probe_module.rs` 只改了说明里的轮次；`tools/rerun.sh` 拷自 `scripts/agents/three-way-rerun-template.sh`，只换了 `rerun_steps`；`shapes.rs` 换了历史族，`tally.rs`、`main.rs` 加了计数键与一行没有分母的提示。`patch_the_copy.py` 的四处原文在这一轮冻结副本里各命中 1 次（用它的常量对冻结副本数过，量的）。几份 `.rs` 只过了 `rustfmt` 的语法解析，没让编译器核过。

探针做什么（这一族与第一轮不同，是「把全空段压到 0 的写法」）：
- 起点状态 8 个：两块内存稀疏盘在 1007 + 384 槽之外各多 [0, 0]、[256, 256]、[1024, 1024]、[256, 1024] 槽（最后一档两块不等大），乘文件大小两档（1–2 个数据单元、1–8 个）。每个起点状态只建、只填一次：头一个要它的线程 mkfs、写第一个文件，再一轮建 8 个 inode、逐个整份写，到第一次被拒为止，留进一张带锁的表；之后各配置拷那一份（`Pool::copy`）。填满之后每段都留着活数据，全空聚簇段常是 0（`N₀` 照产品值 1）。
- 每个配置在拷来的池上走 250 步打碎的历史：35% 整份重写一个小文件、10% 小覆盖写、15% 大写（8–64 个数据单元）、20% unlink、20% 建一个小文件。每一步经会话的 `publish_user_change` 发。判出放不下时照第一轮的定义跑 O 那一臂（先算整个关掉）与重挂那一臂（正常卸载再挂），按判出它的那一判分；甲另按判出时每块盘全空段数的最小值分 0 与大于 0。
- 每一次改动之后另核一遍重建：拿分配器此刻的记录 `PoolAllocator::rebuild_from_records` 重建一份，比每块盘「两槽都不罩在未释放记录下」的段外槽对数（不排聚簇段、排这次挂载的聚簇段两种）与全空聚簇段数，与增量维护的要相同（T3 第四问）。
- 推成的每一串记推前推后每块盘的全空段数：变多、变少、不变分开数，汇总行加了 `series_product_lowered`、`series_oracle_lowered`（T1 第一问用）。
- 155 个配置（种子 1 起，8 个起点状态轮着取）。先跑一批（31 个配置）估全量挂钟，估出来超过 40 分钟退 3、不跑余下的。

内存与时间（推的，没量过）：编译照第一轮那一份（release，约 6 分钟，第一轮任务 13926 跑通过，内存没读到）。跑的那一段：8 份填满的池常驻（每份两块盘写过的扇区约 90 MB，合计约 0.7 GiB），每个线程同时活着至多三份池（产品、拷贝、O 或重挂那一份），31 线程估 10–14 GiB。挂钟按第一轮读数推：第一轮 145155 次改动在 31 线程上 605.7 秒，判出放不下的 13425 次占了大头；这一族甲会判出得多，每个配置估 2–3 分钟，全量估 10–20 分钟；估错时先行一批会按 40 分钟门槛退 3，日志里 `oracle ESTIMATE …` 一行给了先行一批的秒数。每个配置跑完、每填完 25 轮往标准错误打一行 `PROGRESS`。崩溃状态这一题不涉及（探针不枚举崩溃点）。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| T1-1 改过的两处文档 | 成立，但只写了一边（不判打中） | 两处与代码对得上，「可能推高」「越不越过不另估」都不过头；同一份产物里推串把全空段数压低的是推高的 285 倍，文档只写推高 | `mounted_session.rs:2120`、`:6175`；任务 13926 的 `oracle.out` 计数行（下文贴原样） |
| T1-2 没改的第三处文档 | **打中（读码，文字）** | 成员 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 的结构文档仍把甲那一形归进「推抬 F 放得出的全放出来也放不下」，理由写「推了也写不进的写不花根环的槽与写盘」，正是第一轮判「前提不成立」的那一句 | `mounted_session.rs:686`–`688`；同源较轻的三处 `:658`、`:2937`、`:6828` |
| T1-3 `SlotPairCount` 换得干不干净 | 产品里没有混比混加（成立）；留 `u64` 那一处是同一个量两种表示 | `.0` 只在增量加减、两个段外读数与 `:6090` 一处乘一个数据单元的槽数；`unblocked_slot_pairs_outside_the_cluster_segments` 读的字段已是 `SlotPairCount`、交出去又成裸 `u64`，落得下那一判的结构里槽对与槽是两个并排的裸 `u64` | `allocator.rs:417`、`:783`、`:805`；`admission.rs:980`、`:983`、`:1077` |
| T1-4 两条真池用例钉的形 | 成立（读码） | 两条各被它点名的那一行变异打红，红在用例自己的 `let … else` 那一句；别的改坏里能打红它们的都是同一件事的变体；走空那条只断言成员与「空间不够」，弱一点 | 改动全文；`accept_raises_the_floor_for_space.rs:1525`、`:1588` |
| T3-1 `accept` 并批时取哪一样、分类会不会错 | 成立（造不出，8 形） | 单这一次与并进待发两支都交前台写入那一类，发布路径对这 8 形都判合取 7 | `mounted_session.rs:7233`、`:7235`、`:6162`；`transaction.rs:13363` |
| T3-2 甲判出时 `devices_short` 为空的读者 | 不打中 | 产品代码里没有读这个字段的；只有运行库包里一条测试帮手按「非空」断言，甲在那条历史上判出时会报一句不对题的失败，不是产品走错支 | `crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1026` |
| T3-3 `Sum` 与算术的溢出、下溢 | 成立（没有新路） | `Sum` 拆成 `u64` 求和，与换类型之前同一个式子；减法都是原来那几处换成 `.0`；工作区开着 `overflow-checks` | `allocator.rs:360`、`:783`、`:805`；`Cargo.toml:25` |
| T3-4 克隆与重建 | 成立（读码）；探针逐步核（待跑） | `DeviceFreeMap` 派生 `Clone`，新位图与两个计数随整个结构一起克隆；重建只经 `mark_allocated`、`mark_released` 两个事件，从记录重算、不沿用旧值 | `allocator.rs:365`、`:2534`、`:2536` |
| 欠账再取一份样（乙槽对那一判、甲） | 探针写好，待主 agent 跑 | 换一族：填满再打碎、全空段常是 0 | 上一节 |
| 检查单：走不到的理由 | 成立 | 两批加号行里产品代码没有新的 `assert!`、`expect`、`unreachable!`；减法是原有的几处换成 `.0` | 改动全文 |
| 检查单：类型与名字 | **不成立** | 槽对数在 `allocator.rs` 的一个公开读数与 `admission.rs` 的一个公开字段里仍是裸 `u64`，与槽数并排；照检查单「有一处不是就是打中」 | `allocator.rs:782`；`admission.rs:980`、`:983` |
| 检查单：不兼容旧代码与旧格式 | 成立 | 没碰盘上字节，没有版本分支 | 改动全文 |
| 检查单：不新建判定库 | 成立 | 没有新的判定存放处 | 改动全文 |
| 检查单：所有任务多机可用 | 成立 | 两条新用例在内存池上现造；探针路径从参数取 | 改动全文；`tools/rerun.sh` |
| 检查单：测试优先多线程 | 成立 | 两条伪随机历史各一个种子，理由说的是步与步之间有次序，对一个种子成立；真池两条是单场景 | `allocator.rs:4033`、`:4053` |
| 检查单：带 --item 的命令与项 | 成立 | 9 行变异的原文在冻结副本里各命中 1 次、点的测试函数各有 1 处定义；改写的 7 行旧名在变异表里一个不剩，仓里别处没有登记点名旧名 | 下文贴命令与输出 |
| 检查单：新崩溃流多罩了什么 | 成立（不要新流） | 这两批只改注释、测试与类型，不改任何写盘的路 | 改动全文 |

## T1　第一轮的修补本身（主攻）

### T1-1　改过的两处：对得上，只写了推高一边

改后的两处原文：「推串里的空发布会换下固定点单元，可能把全空段数推高，判的是被拒那一刻的数，」（`crates/singlefs-core/src/mounted_session.rs:2120`）、「推高之后越不越过 `N₀` 不另估，不为它推串。」（`crates/singlefs-core/src/mounted_session.rs:2121`），甲那一处同一个意思（`crates/singlefs-core/src/mounted_session.rs:6175`）。逐句对代码：甲读写之前每块盘的 `empty_segments()`，判法与合取 7 同一个函数，代码不估推完之后（`mounted_session.rs:6202`–`6215`）；回收不改记录，所以「回收不改这个数」对（`allocator.rs` 的 `mark_reclaimed` 不碰 `unreleased_per_segment`，读码）。没有说过头的句子：「可能推高」有任务 13926 的 24 串撑着，「不另估」是代码不估，不是量到越不过。

只写了一边。推串的空发布不只换下旧的固定点单元，也要给新的固定点单元取落点：开放段用完、开最低的全空段时，那一段从此罩在未释放记录下，全空段数减一（读码推的）。同一份产物里压低远多于推高。第一轮那份输出在主 agent 的复跑目录里，sha256 前缀与判决第一节第 2 条写的 `82a71415e825c942` 相同，计数行原样（我读了文件、没复跑，数是主 agent 任务 13926 的，量的）：

```
$ sha256sum …/m4-gpugc-precheck-b-a-code-r1-main-check-opus-run/out/oracle.out
82a71415e825c9425d0f4486ff744a93d04a918ffa7d6e3070573e7a016f234c
$ grep 'COUNT series' …/oracle.out
oracle COUNT series arm=oracle 7300
oracle COUNT series arm=oracle empty-segments=lowered 1543
oracle COUNT series arm=oracle empty-segments=raised 7
oracle COUNT series arm=oracle empty-segments=same 5750
oracle COUNT series arm=product 31219
oracle COUNT series arm=product empty-segments=lowered 6846
oracle COUNT series arm=product empty-segments=raised 24
oracle COUNT series arm=product empty-segments=same 24349
```

产品那一臂 31219 串里压低 6846、推高 24、不变 24349（相加 31219，量的）。判决只引了推高那一个数（判决第三节第 1 条「24/31219 串推高、0 串越过」），汇总行 `PROBE_RESULT` 本来就没有压低那一项，是计数行里有。文档照着判决写，于是只写了推高。这不让文档错：甲不推的理由是写之前已少于 `N₀`，推了多半更少，正好支持不推。所以不判打中，记一句改法（改法表第 2 行）。什么现象推翻这一条：这一轮探针 `series_product_lowered` 远小于 `series_product_raised`，或计数行显示压低的串只出现在 O 那一臂。

### T1-2　第三处文档没跟着改：仍说甲那一形「推了也写不进」（打中，读码）

成员 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 的结构文档第一句：「推抬 F 放得出的全放出来也放不下：`devices_short` 里每块盘的上界都小于需求，或这次调用推串之前全空聚簇段已少于 `N₀`、被拒的又是合取 7 判的前台写入」（`crates/singlefs-core/src/mounted_session.rs:686`），接着给理由：「推了也写不进的写不花根环的槽与写盘」（`crates/singlefs-core/src/mounted_session.rs:688`）。两种判出共用这一个理由。乙那一种有上界撑着（上界小于需求，推了也写不进，第一轮两腿读码一致）；甲那一种代码不估推完之后（T1-1），第一轮判决第二节已判「注释写的前提不成立」，第三节第 1 条改了 `:2120` 与 `:6175` 两处，这一处没在改法里点名、没改。

同源、较轻的三处：枚举成员的文档「空间不够（报给用户就是 ENOSPC），而且推抬 F 放得出的全放出来也放不下：这一次不再推」（`crates/singlefs-core/src/mounted_session.rs:658`）、`publish_user_change` 的文档「推抬 F 放得出的全放出来也放不下时不推，交回」（`crates/singlefs-core/src/mounted_session.rs:2937`）与 `AcceptRefused` 那一行（`mounted_session.rs:6828`）。这三处把成员名照字面复述，不另给理由，读的人会以为甲判出也是「全放出来也放不下」；它们比 `:688` 轻，因为没写「推了也写不进」。模块文档那一句把两种分开写了（`mounted_session.rs:6`–`7`），对。

四问：
- 分不分辨臂：分。乙、甲两条臂共用这一句；乙那一半有上界撑着，甲那一半没有。
- 系统当时看不看得到判别它的东西：看不到。甲只读写之前的数，「推了写不写得进」要整串预演之后的数，代码不算（`:6176` 自己写了「不另估」）。所以这一句写的不是代码判出的事。
- 满足的是哪一个分句：T1「两处文档改完有没有说过头或与代码对不上」里「与代码对不上」那一半，落在没改的第三处；与第一轮判决第二节「推了这次写也是被合取 7 拒」同一个前提，改了两处、漏了一处。
- 改法在这一格上中不中：第一轮改法第 1 条只点名两处，不罩这一处。补一处文字即可（改法表第 1 行）。

造法：不用造历史，读码即可（文字与代码对不上）。行为上有没有反例：第一轮取样甲判出 4 格，O 那一臂 2 格被合取 7 拒、2 格推满仍放不下，没有写进的（下文贴的计数行），所以这一句在取样里没撞上反例，只是没有代码撑着；这一轮探针的 `misrefused_conjunct_before_any_series` 大于 0 就是行为上的反例。什么现象推翻这一条：主 agent 读 `:686`–`688` 认为那一段说的是「这一次不推」的成员语义、不是甲的理由（那一句理由句只挂在乙上），那这一格降为措辞。

第一轮甲判出那 4 格的计数行原样（同一份文件，量的）：

```
oracle COUNT fired by=conjunct-before-any-series 4
oracle COUNT fired by=conjunct-before-any-series oracle=NoSpaceAfterRaisingTheFloor 2
oracle COUNT fired by=conjunct-before-any-series oracle=Publish:EmptyClusterSegmentsBelowTheFloor 2
oracle COUNT fired by=conjunct-before-any-series remount=NoSpaceAfterRaisingTheFloor 2
oracle COUNT fired by=conjunct-before-any-series remount=published 2
```

### T1-3　`SlotPairCount` 换得干不干净

产品代码里取 `.0` 的每一处（`grep -n` 冻结副本两份产品文件，`allocator.rs` 只看 3800 行之前的非测试段）：
- 增量加减：「self.unblocked_slot_pairs.0 -= 1;」（`crates/singlefs-core/src/allocator.rs:696`）与它的加一、段内那一份，`:910`–`915` 另一组同形。都是槽对加减一，不与槽数混。
- 两个段外读数：「self.unblocked_slot_pairs.0」（`crates/singlefs-core/src/allocator.rs:783`）减聚簇段里的那几段之和，交出裸 `u64`；`:805`–`811` 同一个式子，结果再包回 `SlotPairCount`。
- `Sum`：「SlotPairCount(counts.map(|count| count.0).sum())」（`crates/singlefs-core/src/allocator.rs:360`）。
- 上界换字节：「slots.slot_pairs_once_everything_is_released.0 * slots_of_a_data_unit,」（`crates/singlefs-core/src/mounted_session.rs:6090`）。乘的是一个数据单元的槽数，换成字节后与 `short.demand` 比；落得下那一判算需求用的是同一个乘数（`admission.rs:1065`–`1088`：槽对数与数据单元数各乘 `slots_of_a_data_unit`），两边口径相同。同一结构里的 `deferred`、`isolated` 仍是槽数的裸 `u64`（`mounted_session.rs:6050`、`:6052`），只在式子那一支相加，槽对那一支不碰它们。

所以产品代码里没有一处把槽对数与槽数相加、相比（读码，分母是上面这几处，全列了）。

留 `u64` 的那一处：读数 `unblocked_slot_pairs_outside_the_cluster_segments` 读的字段已经是 `SlotPairCount`（「unblocked_slot_pairs: SlotPairCount,」，`crates/singlefs-core/src/allocator.rs:417`），交出去时拆成裸数（「) -> u64 {」，`crates/singlefs-core/src/allocator.rs:782`）；它的兄弟读数交 `SlotPairCount`。同一个量（段外槽对）在这一个文件里两种表示。它交到落得下那一判的结构里，与提交内生块的槽数并排：「pub slot_pairs_for_data_units: u64,」（`crates/singlefs-core/src/admission.rs:980`）、「pub slots_for_commit_generated_units: u64,」（`crates/singlefs-core/src/admission.rs:983`），比较时直接拿槽对数比数据单元数：「if room_on_the_device.slot_pairs_for_data_units < units.data_units {」（`crates/singlefs-core/src/admission.rs:1077`）。今天这几处语义上对（一个数据单元一对），没有混错；但两个裸 `u64` 字段换个位置、或拿槽数去比，编译器不拦。混着比的地方在测试与实验装置里有：`allocator.rs:3931` 把 `u64` 读数包回 `SlotPairCount` 去比上界；E180 装置拿槽数除以 2 与这个读数取小（`crates/singlefs-implementation-experiments/src/bin/e180_admission_reading_versus_writable_bytes/writable_units_search.rs:58`–`1711`），都对，但都是手对单位。主 agent 定这一批不换，理由写在背景材料；我按检查单的字面判，见「检查单八格」。

`pub` 字段：仓里同类的 newtype（`SlotNumber`、`CheckpointTxg`、`BytesOnOneDevice`、`MetadataBlocks`）都是 `pub` 字段，`SlotPairCount` 照惯例。它挡不住拿槽数去包：变异表第 7707、7713 行就是 `SlotPairCount(段外槽对 + defer 槽 + 隔离槽 + …)`，照样编得过。私有字段加构造函数也挡不住这一种（构造函数照样收任何 `u64`），所以不判它该私有；能挡的是把读数与字段都换成带类型的，让「槽对比槽数」那一步要显式换算。

### T1-4　两条真池用例钉的是不是名字说的那一形（成立，读码）

会话单发那条（`accept_raises_the_floor_for_space.rs:1493`）：池摆成落得下那一判先拒（段外没挡的槽对比需求少一对）、乙的上界够、`N₀` 抬到每块盘全空段数加一。照冻结副本：乙不判出，甲读到少于 `N₀`、被拒的是写文件内容、还没推过一串，判出，交 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`。变异第 7700 行把会话单发那一处换成「免」：甲不判，推一串；推完落得下那一判放行（扣住的槽生效后放开），合取 7 照判、`N₀` 仍高，交 `Publish` 带 `EmptyClusterSegmentsBelowTheFloor`，或这一串推不成交 `NoSpaceAfterRaisingTheFloor`；两样都落进用例的 `let … else`：「panic!("写文件内容归合取 7 管：推串之前判出、不推，实际 {refused:?}");」（`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1525`）。红在名字说的那一件事上。同样打红它的别的改坏：甲整个不判（变异第 7692 行）、`of_the_user_change` 把写归错类——都是「这一处不先判甲」的变体，不是别的事。我另试了两种错实现：乙在这一形上误判出（会填 `devices_short`，用例断言它为空，红）；甲推过一串之后也判（变异第 7693 行，这一形一串都没推，与正确实现同一结局，绿），后一种不是这条用例名字说的事，由别的用例管。

走空那条（`accept_raises_the_floor_for_space.rs:1546`）：留一个没走完的 unlink、把空闲槽都记成已分配（「mark_every_free_slot_allocated(session.session_for_tests_only());」，`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1560`）、`N₀` 取最大值加一。发 `NewInodes`：先走空，走空的删除批被落得下那一判的提交内生块那一样拒（数据单元为 0，槽对那一样过；读码推的，也可能是固定点取不到落点的 `PlacementRefused`，两样走同一条路），乙那一样交 `None`、不判出；走空那一处传「免」，甲不判；推一串，推不成，交 `NoSpaceAfterRaisingTheFloor`。变异第 7701 行把那一处换成前台写入：甲判出，交 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，落进用例的 `let … else`，红。甲不看被拒的是哪一类（变异第 7694 行）同样红，是同一件事的变体。

弱处（不判打中）：走空那条只断言成员名与「被拒的是走空那一步、空间不够」，实际只核了 `refusal_is_short_of_space`（「refusal_is_short_of_space(&no_space.last_refusal),」，`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1588`）。它绿就推得出被拒的是走空那一步：要是走空走完、`NewInodes` 自己被拒，那是前台写入，甲会判出、用例红。这一条推理靠「`N₀` 高于每块盘的全空段数」（用例取最大值加一）这一个前提，用例没把「被拒的是走空的第一步」直接写成断言；加一句 `versions_published` 为空、`floor_raises` 为空能把它钉死，不再靠推理（改法表第 4 行，推的）。另：`mark_every_free_slot_allocated` 只置位图、不写分配记录，用例里的分配器与记录不一致（只为造满，读码）；不影响它钉的那一件事。

## T3　第一轮没看过的地方（主攻）

### T3-1　`accept` 并批时 `change_of_the_refused_batch` 取哪一样、分类会不会错（成立，造不出）

取法：单这一次被拒取进门的那一个改动（「BatchJudgedAtTheAccept::ThisChangeAlone => change,」，`crates/singlefs-core/src/mounted_session.rs:7233`）；并进待发合起来被拒取合起来的那一批（「UserChange::DataChangesPublishedTogether(DataChangesPublishedTogether {」，`crates/singlefs-core/src/mounted_session.rs:7235`），交给 `of_the_user_change`（`:7265`）。这一批归前台写入那一类（「| UserChange::DataChangesPublishedTogether(_) => Self::ForegroundWriteTheConjunctJudges,」，`crates/singlefs-core/src/mounted_session.rs:6162`）。发布路径上判不判合取 7 看的是计划里有没有文件或新 inode 记录（「let writes_in_the_foreground = plan.file.is_some()」，`crates/singlefs-core/src/transaction.rs:13363`；「|| !plan.new_inode_records.is_empty()」，`crates/singlefs-core/src/transaction.rs:13364`），再要有普通分配。

逐形对（分母 8 形）：进门的改动只有五种进得到这里（`ChangeAtTheAccept::of`，`mounted_session.rs:6613`–`6679`：整份写第一个文件、覆盖写第一个文件、按 inode 整份写、按 inode 覆盖写、建 inode；unlink、改 size、建删快照走屏障，按偏移写与并好的一批在进门就拒）。单这一次被拒是这五种，各归前台写入，发布路径上五种都带文件或新 inode 记录、都有普通分配（数据单元或 inode 叶），判合取 7；并批被拒有三形（只有整份写、只有建 inode、两样都有；至多一个文件，第二个 inode 的写进门就拒），都带文件或新 inode 记录，判合取 7。8 形两边一致，造不出分错。「并了几样改动取的是哪一样」这一问的答案：取整批，不取其中一样，所以不会因为批里有一样「免」的改动而分错——批里根本进不来免的那几种。

还有一处与分类无关、顺带记下（推的，不判）：单这一次被拒时 `remainder` 只记这一次改动，不带待发（`:7241`）；被宽护卡住、要出路时照它预演的是这一次单发，真收下之后触发的发布是整批。这两批改动没碰 `remainder` 的取法（读改动全文），射程不在这一轮。

### T3-2　甲判出时 `devices_short` 为空，读它的调用方会不会走错支（不打中）

文档写明二选一：「/// 上界小于需求的盘；先判它们，判出了 `empty_cluster_segments_below_the_floor` 就是 `None`。」（`crates/singlefs-core/src/mounted_session.rs:692`）。读这个字段的地方（`grep -rn 'devices_short' crates --include=*.rs`，冻结副本）：产品代码只有构造它的那两处（`mounted_session.rs:6230`–`6250`），没有读它分支的；运行库包 `src/` 里没有命中。读成员的产品与装置代码（`history.rs`、`model_comparison.rs`、E158、E180、`common_admission_pool`）只取 `floor_raises`、`versions_published` 与 `last_refusal`；`last_refusal` 在两种判出里都是空间不够那两种，模型那一侧都映成单元区墙（「PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),」，`crates/singlefs-verification/src/model_comparison.rs:1209`），合取 7 自己拒的那一种也映成同一条（`:1211`），所以模型不分「先整理」与「真没空间」，空表不改它的判。

唯一按「非空」断言的是运行库包的测试帮手：「!no_space.devices_short.is_empty()」（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1026`），在 `assert_refused_with_nothing_pushed` 里（`:1013`）。那条历史要是在某一步由甲判出，这句会红，失败文字说「点名放不下的盘」，与真实原因不对题。这是测试的措辞问题，不是产品走错支；那条历史的池是不是会让全空段到 0，我没造（推的）。

### T3-3　`SlotPairCount` 的求和与算术：没有新的溢出、下溢路（成立）

- 求和：`Sum` 把每一项拆成 `u64` 再求和（`allocator.rs:360`），与换类型之前 `.iter().sum()` 是同一个式子；用在三处：聚簇段里那几段之和（`:533`）、建空闲图时两个总数（`:613`、`:620`–`623`）。每段至多 32 对，总数至多单元区槽数的一半，装得进 `u64`。工作区 `[profile.release]` 开着 `overflow-checks = true`（`Cargo.toml:25`，仓根那一份），真溢出会 panic 而不是绕回。
- 减法：段外数 = 总数 − 聚簇段里那几段之和（`:783`、`:805`）；聚簇段是 `BTreeSet`，各段不重复，所以减数是总数里一组不重复的项之和，不大于总数，不下溢。增量那两组（`:695`–`700`、`:910`–`915`）只在一对的状态真翻过去时加减一，减之前这一对是算着的，不下溢。这三样都是换类型之前就有的式子，只是写成了 `.0`。
- 新加的算术只有 `:6090` 那一处乘法（槽对数乘一个数据单元的槽数），乘积不超过单元区槽数；再换字节（乘 16384）不超过盘的字节数。

分母：`SlotPairCount` 在两份产品文件里的全部用处（`grep -n 'SlotPairCount\|\.0 [-+]= 1'`，上面逐处列了）。

### T3-4　克隆与重建（成立，读码；探针逐步核）

- 克隆：`DeviceFreeMap` 是「#[derive(Clone, Debug)]」（`crates/singlefs-core/src/allocator.rs:365`），新位图 `is_under_an_unreleased_record` 与两个计数（每段一份、总数）是同一个结构的字段，随整个结构一起克隆，没有手写的 `Clone`、没有放在结构外的影子计数。推串的预演、发布被拒时换回、推环的预演都克隆整个 `PoolAllocator`（它也是派生的 `Clone`，`:2135`）。
- 重建：重开时「device_map.mark_allocated(record.slot, span);」（`crates/singlefs-core/src/allocator.rs:2534`），已释放的再「device_map.mark_released(record.slot, span);」（`crates/singlefs-core/src/allocator.rs:2536`），从全新的空闲图起步，两个事件各经 `note_slots_under_an_unreleased_record` 改位图与计数，不沿用任何旧值。位图只在 `note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts` 里写（「self.is_under_an_unreleased_record[index] = match transition {」，`crates/singlefs-core/src/allocator.rs:901`），`DeviceFreeMap` 的字段构造只有 `with_unit_area_start` 里那一处 `Self {`（`allocator.rs:590`；`grep -n 'DeviceFreeMap {'` 只命中结构定义 `:366` 与 `impl` 头 `:536`）。管理员回退复活走 `mark_resurrected_out_of_the_defer_queue`，同一个入口。已回收而记录还写着已释放的槽，重建时先标占再释放，位图上算「不罩在未释放记录下」，与进程里一直跑过来的那一份相同。
- 库单测只拿增量数比「按记录逐槽扫」（`allocator.rs:4031`，那一条用例的文档），没比重建那条路；探针补一格：每一次改动之后用 `rebuild_from_records` 重建一份，比两种段外数与全空段数（`rebuild_checked`、`rebuild_differ`、`rebuild_panicked`）。`rebuild_differ` 大于 0 就是打中，样例行 `REBUILD_DIFFERS` 带种子、步号与哪块盘。

## 欠账：换一族再取一份样（派发第 3 条）

第一轮判决留的两条：乙在槽对那一判上的误拒、甲的误拒各只有一份取样，甲的分母是 4。这一族照派发的提示「把全空段压到 0」：先把池填到第一次被拒，再打碎（见开头「探针做什么」）。与第一轮那一族的差别：第一轮池大半时间有全空段，甲 56681 次判里判出 4 次；这一族填满之后全空段多半是 0，落得下那一判先拒的写一来，乙的上界又够（defer 里成对的槽多），甲就判出。

跑之前写下怎么读（结果出来前写死）：
- 甲：`misrefused_conjunct_before_any_series` 大于 0 就是打中——甲在 O 那一臂推了写得进的格上不推；样例行 `EXAMPLE MISREFUSED … by=conjunct-before-any-series` 给种子、步号、写之前每块盘的全空段数与 O 那一臂每一串前后的数。分母是 `fired_by_conjunct_before_any_series`，其中产品 `N₀` 下的那一种是 `conjunct_fired_with_zero_empty_segments`。分母不到 50 说明这一族没压到位，不记一份样本，交回改形；分母几百而误拒 0，记「第二份取样没撞上」，与第一轮那一份合起来按规则算两次没打中，但第一轮那一份分母只有 4，两份不对称，判决里要写明。
- 乙：`misrefused_slot_pairs` 大于 0 就是打中，分母 `fired_by_slot_pairs`；`base_published_after_a_series` 是这一族误拒的底，0 就是这一族测不出误拒（日志会打 `name=misrefusal_base_zero`）。
- T1-1 那一句：`series_product_lowered` 与 `series_product_raised` 一起读；`series_product_crossed_the_floor` 大于 0 说明推串把全空段从少于 `N₀` 推到不少于，那是甲误拒的前提。
- T3-4：`rebuild_differ` 大于 0 就是增量数与按记录重建的不同，看 `REBUILD_DIFFERS` 行；`rebuild_panicked` 大于 0 是重建那一步自己 panic（探针会停掉这个配置里之后的核对），也要看。
- 重挂那一臂：`remount_published_conjunct_before_any_series` 与 `remount_published_slot_pairs`，与第一轮一起交 C1003（第一轮甲 4 格里重挂写进 2 格）。

结果反过来我接不接受：接受。甲误拒为 0 而分母几百，我撤回「T1-2 文字以外甲还有行为上的问题」这一猜测，只留文字那一格；甲误拒大于 0，T1-2 从文字升成行为上的打中，`refusal_is_short_of_space` 那一处（合取 7 自己拒、不推）同一个前提一起出局，属「打中不分辨臂」，要另记一笔账。

## T2　产生结论的方法（不是主攻，只记两条线索）

- 判决引任务 13926 只引了推高那一项（第三节第 1 条），压低那一项在同一份输出的计数行里、不在汇总行里（T1-1 贴的原样）。探针数的是「任一块盘变多」，与判决说的「把全空段数推高」是同一件事，口径对；漏的是另一个方向。
- 同一份输出里甲判出的 4 格，O 那一臂 2 格推满仍放不下、2 格推完被合取 7 拒，重挂那一臂 2 格写进（T1-2 贴的原样）。判决第二节「乙、甲先拒之后卸载重挂写得进」那一行的「甲 4 格里 2 格」与这几行对得上。

## 检查单八格

- **走不到的理由二次查实：成立。** 在改动全文里现查两批加号行：第 128 批产品文件只改了 `mounted_session.rs` 的两段注释与一条库单测的 `panic!` 文字（实现员报告说「一行没动」不准，但不是新的断言）；第 129 批加号行里产品代码没有新的 `assert!`、`expect`、`unreachable!`、`todo!`，减法是原有的几处写成 `.0`（T3-3 逐处列了），`Sum` 是新写的加法、上界见 T3-3。测试代码里新加的 `expect("有树")`、`expect("池里有盘")` 不在射程。
- **类型与名字：不成立。** 槽对数还有两处是裸 `u64`、与槽数并排：读数 `unblocked_slot_pairs_outside_the_cluster_segments` 的返回值（`allocator.rs:782`，读的字段 `:417` 已是 `SlotPairCount`）与 `PlacementRoomOnOneDevice` 的 `slot_pairs_for_data_units`（`admission.rs:980`，旁边 `:983` 是槽数）。检查单写「同一个量不同语义不共用裸整数……有一处不是就是打中」，这两处按字面打中。今天没有混错（T1-3）；主 agent 定这一批不换的理由（要动公开字段与 E180、E181 两份装置）是代价，不改这一格的字面判定。名字：`SlotPairCount` 与两条用例名没有缩写；`of_the_user_change` 没有 `_ =>`；第 129 批没有新的 `expect`。
- **不兼容旧代码与旧格式：成立。** 两批不碰盘上字节、没有版本号与认旧形态的分支（读改动全文）。
- **不新建判定库：成立。** 没有新目录、TSV、sqlite、RocksDB；探针只往运行目录写输出。
- **所有任务多机可用：成立。** 两条新用例经 `pool_with_the_floor_below_its_ceiling` 在内存盘上现造，不读本机状态；探针的路径都从参数取，运行目录在工作根下。
- **测试优先多线程：成立。** 两行理由「/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。」（`crates/singlefs-core/src/allocator.rs:4033`，`:4053` 同一句）。两条用例各只走一个固定种子（`allocator.rs:3944` 起的那一个状态），一条历史内部确实不能切；能并行的只有多个种子，而用例只有一个，所以「该并行而没并行」不成立。理由只说了步与步，没说为什么只取一个种子，这是取样宽度的事，不是这一格。两条真池用例各是一个场景。
- **带 --item 的命令与项：成立。** 9 行变异逐行核（在冻结副本里跑，原样输出）：

```
7700 | 抬 F 先算：会话单发的写交成合取 7 免的那一种（经会话单发真池上推一串） | 原文命中=1 | 测试函数定义=1
7701 | 抬 F 先算：走空被拒的一步交成合取 7 判的前台写入（走空真池上不推就拒） | 原文命中=1 | 测试函数定义=1
7707 | 抬 F 先算：槽对那一判的上界改回每个放得开的槽各补一对（槽对数类型） | 原文命中=1 | 测试函数定义=1
7708 | 抬 F 先算：槽对那一判的上界只数此刻没挡的槽对（槽对数类型） | 原文命中=1 | 测试函数定义=1
7709 | 抬 F 先算：槽对那一判的上界按槽数不按槽对的字节换（槽对数类型） | 原文命中=1 | 测试函数定义=1
7710 | 抬 F 先算：槽对那一判的上界再加一遍此刻的可用（槽对数类型） | 原文命中=1 | 测试函数定义=1
7711 | 抬 F 先算：式子那一判的上界多算段外放开之后的槽对（槽对数类型） | 原文命中=1 | 测试函数定义=1
7712 | 抬 F 先算：一对翻成都放得开时总数不加（全放开之后的槽对少算）（槽对数类型） | 原文命中=1 | 测试函数定义=1
7713 | 抬 F 先算：槽对那一判的上界改回每个放得开的槽各补一对（accept 真池上推一串）（槽对数类型） | 原文命中=1 | 测试函数定义=1
```

  改写的 7 行旧名：拿第一轮冻结副本变异表第 7692–7706 行的名字（15 个）对这一轮整张表的名字列逐个精确比，不在这一轮表里的恰好是这 7 个（`grep -vxFf`，退 0）：

```
抬 F 先算：槽对那一判的上界改回每个放得开的槽各补一对
抬 F 先算：槽对那一判的上界只数此刻没挡的槽对
抬 F 先算：槽对那一判的上界按槽数不按槽对的字节换
抬 F 先算：槽对那一判的上界再加一遍此刻的可用
抬 F 先算：式子那一判的上界多算段外放开之后的槽对
抬 F 先算：一对翻成都放得开时总数不加（全放开之后的槽对少算）
抬 F 先算：槽对那一判的上界改回每个放得开的槽各补一对（accept 真池上推一串）
```

  别的登记点名旧名：在主工作区 `.claude`、`scripts`、`crates`（变异表本身除外）里按六段旧名的去后缀字串 `grep -rnF`，命中 0 行（命令退 0、`wc -l` 得 0）。只查了这三个目录，`records/` 与 `research/` 是当时的记录，不算登记。
- **新崩溃流多罩了什么：成立（不要新流）。** 两批只改注释、测试与类型，没有一条写盘的路变了；新用例在已有的写盘路上断言，不新增崩溃状态。

## 这一腿提的改法各修哪一格

都是这条腿提的，只在读码上推过、没实现、没在副本上跑，被攻过零轮。

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| `mounted_session.rs:686`–`688` 把两种判出的理由分开写：乙「上界小于需求，推了也写不进」；甲「写之前全空段已少于 `N₀`，推串可能推高也可能压低，越不越过不另估」 | T1-2 | 推的（文字） |
| `:2120`、`:6175` 两处在「可能推高」后补「也可能压低（开新段时）」，引任务 13926 的 24 与 6846 | T1-1 | 推的（文字）；两个数是量的（任务 13926 的计数行） |
| `:658`、`:2937`、`:6828` 三处成员文档写成「推抬 F 放得出的全放出来也放不下，或推串之前全空段已少于 `N₀`」 | T1-2 同源三处 | 推的（文字） |
| 走空那条真池用例补断言 `floor_raises`、`versions_published` 都空、`stop` 是哪一种 | T1-4 弱处 | 推的，待主 agent 跑（只改测试代码） |
| `unblocked_slot_pairs_outside_the_cluster_segments` 交 `SlotPairCount`，`PlacementRoomOnOneDevice::slot_pairs_for_data_units` 换成 `SlotPairCount`，比较处显式换算 | 检查单「类型与名字」 | 推的；代价是动 `admission.rs` 公开字段与 E180、E181 两份装置（背景材料第二节） |
| 运行库包测试帮手 `assert_refused_with_nothing_pushed` 的失败文字写明是乙那一种，或先断言 `empty_cluster_segments_below_the_floor` 为 `None` 再断言 `devices_short` 非空 | T3-2 | 推的（测试代码） |

## 要主 agent 跑的命令

一条，在仓根 `/home/fy5090/code/singlefs` 起（不给 `--cwd`）：

task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-opus-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model

- 包装那一层先删掉运行目录 `…/m4-gpugc-precheck-b-a-code-r2-opus-probe-run`（只认工作根下、名字以 `-run` 收尾的，别的退 2），再调同一个模型目录里的 `tools/rerun.sh`；服务撞内存上限重排同一条命令时照样能重跑。
- 判过：末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`（在新 shell 里起的整份 `rerun.sh` 退 0）。日志里依次要有 `PATCH_THE_COPY … probe_sources=5`、`name=build done binary=…`、`name=exit_code oracle=0`、`oracle ESTIMATE …`、`oracle WALL_SECONDS …`、`oracle PROBE_RESULT configurations=155 …`、四行 `name=self_check ok …`、`name=self_checks bad=0`、末行 `name=rerun_done …`。有 `✗` 就是没过。
- 退 1、日志是 `build.log` 的末 80 行：编不过（探针没让编译器核过；最可能的一处是 `Options` 里那张 `Mutex<Option<Pool>>` 要求 `Pool` 能跨线程，读码判它能），新派同立场的腿照报错改 `probe/`。退 1、`name=exit_code oracle=1`：自证不合，看 `✗ name=self_check` 那一行。`name=exit_code oracle=3`：估时超过 40 分钟，照 `oracle ESTIMATE …` 一行把 `rerun.sh` 里的 `--configurations 155` 或 `--steps 250` 缩下来（新派腿改）。
- 要读的：`oracle PROBE_RESULT` 一行的 `fired_by_conjunct_before_any_series`、`conjunct_fired_with_zero_empty_segments`、`misrefused_conjunct_before_any_series`、`fired_by_slot_pairs`、`misrefused_slot_pairs`、`base_published_after_a_series`、`series_product_raised`、`series_product_lowered`、`series_product_crossed_the_floor`、`series_oracle_crossed_the_floor`、`remount_published_conjunct_before_any_series`、`remount_published_slot_pairs`、`rebuild_checked`、`rebuild_differ`、`rebuild_panicked`、`fills`、`steps_with_zero_empty_segments`；日志里另抓了 `COUNT fill …`（每个起点状态填了几个文件、填完每块盘的全空段数）与 `COUNT fired by=conjunct …`、`COUNT series arm …` 计数行。有 `EXAMPLE MISREFUSED`、`EXAMPLE REBUILD_DIFFERS`、`EXAMPLE REBUILD_PANICKED` 的逐行读。整份输出在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-opus-probe-run/out/oracle.out`。读法在「欠账：换一族再取一份样」一节，跑之前写死。
- 跑完新派同立场的腿，带「承接：research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-output.md」与任务号，读 `task-server/task log <号> --all` 与上面那份输出，接着判甲、乙的第二份取样与 T3-4。运行目录里的 `copy/`（冻结副本的拷贝，服务没给绝对的编译目录时连编译目录）用完由主 agent 删。

## 没打中的形状

- `accept` 并批分错类（T3-1）：8 形逐个对，两边都判合取 7；批里进不来免的那几种。
- `devices_short` 为空让产品走错支（T3-2）：产品代码没有读这个字段分支的地方；读成员的装置与模型只取别的字段。分母是 `grep -rn 'devices_short' crates --include=*.rs` 的全部命中。
- `SlotPairCount` 混比混加、溢出下溢（T1-3、T3-3）：两份产品文件里它的每一处用法逐个对过，没有与槽数相加相比的，算术都是换类型之前就有的式子。
- 克隆与重建沿用旧值（T3-4）：派生 `Clone`、重建只经两个事件；读码造不出，探针那一格待跑。
- 两条真池用例钉错了形（T1-4）：两条各在点名的那一行变异下红在自己的 `let … else`；能让它们照样绿的错实现我只找到「甲推过一串之后也判」一种，不是它们名字说的事。
- 读码没造任何历史去打 T1-2：那是文字与代码对不上，不靠历史。

## 这条腿自己的限度

- 探针没让编译器核过；`Options` 那张带锁的表要 `Pool` 能跨线程送，读码（核心层非测试代码里没有 `Rc`、结构字段里没有不能送的东西）判它能，编不过就是这一处，见「要主 agent 跑的命令」。
- 探针只走 `publish_user_change`，不走 `accept`；甲经 `accept` 的那一支（T3-1）只读码。
- 「填满再打碎」这一族由我定的权重（35/10/15/20/20）与四档盘宽、两档文件大小取样，起点历史对每个起点状态是确定的；撞不上的形不等于没有。
- T1-1 与 T2 里的数都是主 agent 任务 13926 的输出文件里的，我读了文件、没复跑；sha256 与判决写的前缀对上。
- 挂钟与内存是按第一轮读数推的，没量；估错时先行一批会按 40 分钟门槛退 3。

## 没做什么

- 没往统一任务服务交任务、没起 cargo、没跑门禁与自证；探针读数一个都没有，判决要引的数都等主 agent 跑上面那一条。
- 不判辩方那几格（T2 只记了两条线索）、不替主 agent 采纳；副本上跑出来的数不算入库装置上的数。
- 写了：这份报告、模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-model/`、草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-opus/` 里的几份名字清单与核对输出（模型的草稿目录 `draft/` 与模型目录逐字节相同，交回前已删）。草稿目录里没建仓副本与编译目录。
- 写范围之外动过一处：核 `patch_the_copy.py` 的原文命中数时用 python 载入了第一轮模型目录里那一份，python 在 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/` 下留了一个 `__pycache__/`（被 `.gitignore` 忽略）；是我当次建的，已删掉（目录与里面那一个 `.pyc`），第一轮模型的源文件没动。
- 交回前跑过 `python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-precheck-b-a-code-r2-opus-output.md`（仓根，退 0），末行原样：`✓ 核了 21 处引文、45 处裸行号，对上 21 处，没判 45 处（其中裸行号没有引文可比 42 处；逐处列在上面）`。没判里不是「没有引文可比」的三处都是 `Cargo.toml:25`（仓里同名文件多，认不出），我说的是仓根那一份。
