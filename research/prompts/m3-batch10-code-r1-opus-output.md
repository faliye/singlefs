# m3-batch10-code-r1 云端攻方腿（Opus）报告（2026-10-05）

被判对象：冻结副本 `/tmp/claude-1000/m3-batch10-code-r1/tree/`（代码行号都是这份副本的）。攻击面：正文第三节 K1、K2、K3、K4，重点 K1、K2。
所有数都在冻结副本（或 10c 之前那份树）的**拷贝**上量，是副本上的数，不是入库装置上的数。

## 复跑

工作目录给一个空目录 W，按次序跑（每阶段前台几分钟以内；`fin`、`k2ms`、`fxw` 三段并行 15 件，各约 3–15 分钟）：

```
M=research/prompts/m3-batch10-code-r1-opus-model
for s in $(bash $M/rerun.sh list W); do bash $M/rerun.sh $s W; done
bash $M/rerun.sh compare W        # 与 $M/out/summary.txt 逐行比
```

原型底子是第 17 批攻方模型 `research/prompts/m3-batch17-code-r1-opus-model/probe/main.rs`（sha256 2235cb3a…，没改原件），`rerun.sh` 在拷贝上打这里 `patches/` 的三刀：`multistage.py`（一次挂载里几段读坏用「+」连），`i79dump.py`（`FX_I79=1` 时从后往前查、打出前三个 I-7.9 判红状态的 checker 原文），`finalchk.py`（`FX_FINALCHK=1` 时对挂载与之后覆盖写全落盘的那个状态跑池级 checker）。三刀对单段计划、不设这两个变量时不改行为；我实际跑 `fxw`、`pass-jia` 两段用的是没打补丁的一臂，其余用打了的一臂，`rerun.sh` 统一用打了的一臂。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

| 文件 | sha256 |
|---|---|
| out/fin-frozen.txt | c7d7864e8f6de5ce789f46624ce2710e9a525442cf0c742db33ee7c9bf8bab82 |
| out/fin-pre-192k9-n8.txt | 969250565fe8fe6388ef0834a1d89dbfcfb1ea23cd6efef60065ed31169d6cec |
| out/k1j-frozen.txt | a0873fa64c6b9d84eb27ebc2a8d6a9408b7d78887761a223a57bc86acdb0cd69 |
| out/k2-192k9-n8-screen.txt | d1b30158b3f766a396900da01f9726396724cf407765ac2e9f11622d03e3492f |
| out/k2-ms-264-n15.txt | 90ab11eb93ef22d5358c84f0396ef82e5dd62f8ad7ad295b7dcfa213b6d87f43 |
| out/k3-pre-l0-5.txt | 9556131899c18442cd7fb3e5d8ce381d6ea2a31a2621d661480181ce97ea9732 |
| out/phases-s24.txt | 0402c281e374de2a6d45834ba1d6920c4a47651ce01f651d84dd6975ae1c8e8a |
| out/seam-s120.txt | 3d1e11f439ad0d4c165c6a11157826fb927acf35284d3fb502eb46c4555691c9 |
| out/summary.txt | cf86411fc3901e3cdba483f421f8e377b140e8265005c5fd429f21d9a951f3b9 |
| patches/finalchk.py | fbad8d44d36d17718a8630175b50de09d82dd41f3d80b662e5fe40b540a32365 |
| patches/i79dump.py | 8b69dccd4ccafc6de8218a16c25a73cdfa042e276ec9ff303379e5563491ca1f |
| patches/multistage.py | 28102f9ad431ecc4b7be68dd749dafc03edb4b38f2aa032af31a2ad59a20439f |
| rerun.sh | d9d57ef837255c51e4e14bc4f1d442c36de159573e0fb78c711de973b45d278b |
| seamprobe/Cargo.toml | 2933ba62824d290ab1c99e020d94d26108191bca318d1be10c5985abc877a65b |
| seamprobe/src/bin/phases.rs | f18692a38c63b10de6582e3bab41ebce29aa93287ae4b48754c05e2ca8aea1ac |
| seamprobe/src/main.rs | f1b829aad816e8f3fcb76a0ad87a0504de40ad9e8c3335aeaf0a9e55366c3ee7 |

`out/k2-ms-264-n15.txt` 是开跑 `k2ms` 之前的一次试跑（同一组计划的一个子集，单一起点），`rerun.sh` 里没有单独的阶段，`k2ms` 那一段罩住它。

## 各格判定一览

| 格 | 这条腿的判定 | 依据（节） |
|---|---|---|
| K1-a 七处里还有没有一处按一圈窗口取 | 没打中 | 一 1：core 只剩算「读坏的槽能住的最大 txg」一处（条款许可）；checker 只剩 I-7.9 |
| K1-b 钉住的三处在合法历史上越过盘上真住着的有效根、回收它的单元 | 没打中（一次抽样，前缀刀） | 一 2：fxw 窄盘 36 种计划 × 813 段、宽盘 36 × 253 段，pass 窗口 80714 段，k2ms 与 k1pass3 141963 段，丢盘读回坏 0 |
| K1-c 全零槽：上限不并、门槛钉住，是不是同一份读数、同一个分类 | 站住（读代码） | 一 3 |
| K1-d 用户认下的 I-3.1 多记账：有没有一形红在记账少的一边 | 没打中（一次抽样） | 一 4：全落盘状态 14950 + 5876 个，I-3.1 一个没红（两边都没红） |
| K1 线索：影子账判抛弃用的那张表来自另一遍读（第 3 遍择根），不与重建那一遍同读 | 线索，推的，没造盘面 | 一 5 |
| K2-a 重读只坏一部分槽、读数与择根那几遍不一致 | 没打中（一次抽样，前缀刀） | 二 1：30 种两段计划 × 7 个形状 × j∈{0,3}，8215 段 0 坏；只有两遍全坏才 `Recovery(NoValidRoot)` |
| K2-b 连坏两遍的写前拒不在 D23 已定项 14 的单子上 | 不是打中；今天的调用者走不到，给条款写法一条（零轮） | 二 2 |
| K2-c `192:9:3:1` n = 8 两段为什么拒、该不该拒 | 拒因量清；该拒；代价只在窄盘上、与读错那一形同数 | 二 3：36 种计划（含不注错一种）里 34 种与 10c 之前同数，变的只有这两种：窄盘 0 拒 / 53、98 段坏 → 165 拒 / 0 坏；宽盘两种都 0 拒 |
| K3-a 10c 之前的树上 `w:all/but1:z` 两盘都在 I-7.9 红 4116：实现越了上限还是 checker 判错 | 实现越了盘上的真上限、checker 判对；越的方式就是用户认下的 Q3 代价 | 三 1：复跑 4116 对上，checker 原文打出 |
| K3-b 那个代价是不是只有这么大 | 这一族里只有这么大 | 三 2：冻结副本上 3835 个全落盘状态 I-7.9 红，全是读回全零的计划，每一个都只越过 1 条非空有效根、越 1–4 个 txg |
| K3-c I-8.5 改报不适用的边界、I-7.9 只留镜像最新根窗口漏红 | 没攻 | 七 |
| K4-a 掉电接缝不在段首的状态 | 没打中 | 四 1：332 道接缝全在段首（43 段写死 + 120 个种子的 289 道） |
| K4-b 孤儿历史上三段各摆得出二次崩溃吗 | 写死的孤儿历史上五段各一个；随机历史里抹头、清扫发布两段 0 个 | 四 2（覆盖观察，不算打中） |
| K4-c `claimed_state_missing_unit` 压着哪条条款 | 不是打中：D13 已定项 7 故意不给编号 | 四 3 |

本轮 0 格打中。没打中的格都只抽了一次样，照规则不支撑「没打中」的结论，要再抽一次。

## 一、K1 回收门槛

### 1. 七处里还有没有一处按一圈窗口取（K1-a）：没打中

现查冻结副本：core 里窗口只剩算「挂载时读坏的槽能住的最大 txg」那一处，`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1781`「let lowest_txg_inside_the_window = window_anchor」，那个量只进准入抬 F 上限的「能住的最大 txg」，可能住过的最旧 txg 走的是 `/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1797`「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」，不看窗口。条款许可这一处：`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`「一圈窗口这个量现在只剩这一处在用」。`root_ring.rs` 的 `txg_is_inside_the_one_revolution_window` 在三个 crate 的 src 里没有调用者。checker 里 `OneRevolutionWindow` / `window_anchored_at_the_newest_root_of_the_image` 12 处全在 I-7.9 那两支（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-checker/src/walk.rs:5402`「&& window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)」、`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-checker/src/walk.rs:5544`「window_anchored_at_the_newest_root_of_the_image.contains(*txg)」）；参考模型 `model.rs`、`model_comparison.rs` 与 `allocator.rs`、`instance_table_sweep.rs` 里没有环窗口（`allocator.rs` 的 22 处 window 是只供测试的 `ReuseWindow` 开关）。

推翻条件：七处里任一处的调用链上找到按 txg > N − 3S 滤根的一步。

### 2. 钉住的三处在合法历史上越过盘上真住着的有效根（K1-b）：没打中

机理（读代码）：挂载时读坏、读回全零的槽都钉在「落到这个槽、小于 first_txg 的最小 txg」，也就是 [0, 3S) 里的一个数（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1797` 那一行的循环）；进程内与抬 F 那一遍重读仍读不出、读回全零时照钉着的 txg 算（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/allocator.rs:1253`「return (Some(pinned_txg), rereads);」）。钉着的槽在环里时，可再分配谓词里环里最旧有效根那一半恒落在 [0, 3S)，门槛实际就是 F_生效，比任何盘上真住着的根都低；挂载那一刻「当场放开」那一截在 m 取不到时取同一个最小值（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:2041`「None => oldest_valid_root_in_the_ring,」），m 取得到时 m 是读得出的每块盘最新有效根取小，只会比盘上真值低。能让门槛越过一条盘上有效根的路只剩「那一槽根本不在根环表里」（年轻的环里 txg ≥ first_txg 才轮到的槽，或重读读出的根按表被判抛弃），前一种要 first_txg 低于盘上真最大 txg，而 first_txg 取读阶段的记录与另一遍读环的最大值，单一读错压不低它（推的）。

量的（冻结副本拷贝，原型前缀刀：挂载写流每个前缀各丢盘 0、丢盘 1，剩下那块盘只读挂载整读每个活文件，与基线比）：

| 取样 | 段数 | 任何写之前拒 | 落了写之后拒 | 丢盘读回坏 |
|---|---|---|---|---|
| fxw 窄盘（20 个形状，35 种整次挂载读坏计划加不注错一种） | 36 × 813 | 见二 3 | 0 | 0 |
| fxw 宽盘（6 个形状，同 36 种） | 36 × 253 | 见二 3 | 0 | 0 |
| pass（9 个形状，单段整遍读坏窗口 all/all，读错与读回全零） | 80714 | 43536 | 1781 | 0 |
| k2ms（7 个形状，30 种「重建那一遍全坏 + 重读部分坏」，j ∈ {0, 3}） | 8215 | 1322 | 0 | 0 |
| k1pass3（5 个形状，单段窗口 × 三种选盘 all/all、all/-、-/all × 读法） | 133748 | 33108 | 3214 | 0 |

同一张 pass 表在第 17 批攻方的树上（10c、10e 之前）是 80714 段里 2264 段坏、落了写之后拒 5273（`research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt` 里 `[pass-jia-…]` 各行求和），这一批把坏的那一截全换成了写前拒（43536 对 33424）。

推翻条件：任一段历史上丢盘之后只读读回与基线不同而基线读得对（原型里记 `BAD`）。只判了前缀刀（每个写之前崩一次），没展开段内子集，见第七节。

### 3. 全零槽：上限不并、门槛钉住，是不是同一份读数、同一个分类（K1-c）：站住（读代码）

重建分配器那一遍（或它的重读）只读一次根环，`ring_slot_readings` 一份读数同时分出读坏、读回全零、自证过三类，三类各进根环表（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:2079`「.chain(ring_slots_reading_back_all_zero_at_mount.iter().map(」），回收门槛与之后进程内、抬 F 那一遍都从这张表取钉着的槽；准入抬 F 的上限只取表里的 `BadAtMount`（`ring_slots_bad_at_mount`），不取 `AllZeroAtMount`。分类定在挂载那一刻，之后那一槽读法变了（全零变读错或反过来）不改分类，两边一直对同一张表。`ring_slots_known_to_hold_a_root` 两类都不算（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/allocator.rs:1074`「RootRingOccupant::BadAtMount(_) | RootRingOccupant::AllZeroAtMount { .. } => false,」），抬 F 那一遍它们读坏不拒。

推翻条件：找到一处在挂载之后重新按当下的读法给钉着的槽改类。

### 4. 用户认下的 I-3.1 多记账：有没有一形红在记账少的一边（K1-d）：没打中

对每段挂载（与之后覆盖写）全部落盘的那一个状态跑池级 checker（`patches/finalchk.py`）：

- fin：fxw 那 72 种计划在冻结副本上，去重之后 14950 个全落盘状态，判红的只有 I-7.9（3835 个，全是读回全零的计划，见三 2），I-3.1 0 个（`out/fin-frozen.txt`）。
- k1j：8 个形状 × 14 种「挂载交回之后仍坏」（`:p`）的计划 × 挂载后覆盖写 j ∈ {1, 3, 6, 13}（宽盘族真做了覆盖写，299 段做满 j 次；窄盘族不做覆盖写），5876 个全落盘状态，I-3.1 0 个（`out/k1j-frozen.txt`）。

记账多的那一边也没在这两族里出现：这两族的起点是卸载半途崩溃，F_生效 已抬到现行那一版，门槛由 F 那一半决定，钉住的槽压不住它。所以这一格只说明「这两族里两边都不红」，没量到用户认下的那一形，也就量不到它的时长。

推翻条件：任一状态上 I-3.1 红、实现记的已分配少于遍历。

### 5. 线索（推的，没造盘面）：影子账判抛弃用的那张表不与重建那一遍同读

`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1878`「// 影子账、回收门槛与分配器那张根环表读的是同一遍根环（每槽读一次）：两遍之间的瞬时读错会让几边看见的根不一样。」这句只罩根环那一遍。判谁被抛弃的那张表另取：`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1865`「let (newest_table, instance_table_of_the_newest_root) =」，取法是 `/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1426`「if let Some(table) = choose_root(reader, system_configuration)」，也就是另一遍整环读（原型认的第 3 遍，下标从 0 数）里读得出的最新那条根指着的表，不核它是不是读阶段所选那一版的、也不核它是不是环里真正最新的；之后 `/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1974`「let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);」同时管根环表的分类与影子账的候选过滤（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1489`「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」）。

推的那一形：环里已有一条被更新的实例那一行抛弃的根 R（崩溃恢复落到旧根造出来的，C314 那一族）；之后某一次可写挂载，只有第 3 遍读环时最新实例的几条根读不出（瞬时）。那一遍择到的最新根属于更早的实例，它的表里没有抛弃 R 的那一行，R 被当成有效根：不进影子账隔离、只算「候选豁免」，R 独占、在现行账里没有记录的单元在分配器里是空的，这次挂载的写行与暖机可以写掉它们——`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」。这一形要先有一条被抛弃的根（今天的随机历史造不出，崩溃恢复抛弃根那一步在 C554 乙之后被拒），这一轮的原型族里没有，没量。

四句：分辨臂——条款没给候选，不适用；系统当时看不看得到——看得到（读阶段已经择定那一版，可以核两者是否同一条根，推的）；满足的分句——上面那一句主句；跑前条款的改法——没有。交主 agent 判要不要立一笔欠账。

推翻条件：造出这一形之后，影子账照样隔离了 R 独占的单元（例如别处另有一道按读阶段所选那一版核表的步骤）。

## 二、K2 重读再拒（D）与 10c 之后改成被拒的两段

### 1. 重读只坏一部分槽、读数与择根那几遍不一致（K2-a）：没打中

原型的读坏计划能写两段（`patches/multistage.py`）：第 5 遍（从 0 数，原型用回溯认过是 `rebuilt_allocator` 那一遍，窄盘族、宽盘族都是）整遍读坏，第 6 遍（不注错时是下一遍，注错之后重读正好落在这一段计数上）按每块盘一种选槽法坏：不坏、全部、除最新那条根之外全部、只最新那条根，两块盘组合 15 种 × 读错 / 读回全零 2 种 = 30 种计划。7 个形状（窄盘 192:4:3:1、264:6:4:0、216:7:5:4、192:9:3:1、248:9:10:1，宽盘 384:3:1:2、256:5:1:2），每个形状的卸载刀全扫，挂载之后用户动作 j ∈ {0, 3}（宽盘覆盖写、窄盘不做），按（起点、卸载刀、计划）去重共 8215 段（两个 j 算一段；原型的 FXTALLY 按 j 分开数，数是这里的两倍）：

| 结局 | 段数 |
|---|---|
| 丢盘读回坏 | 0 |
| `Recovery(NoValidRoot)`（写前） | 530，全是第 6 遍也整遍坏的两种计划（`P5:all/all+P6:all/all`，读错 265、读回全零 265） |
| `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`（写前） | 792：重读里有一块盘整盘读不出有效根，m 取不到、当场放开只到钉着的最旧 txg，窄盘取不到写行的落点（与二 3 同一个机理） |

第 6 遍只坏一部分时，重读读出的有效根集合比择根那几遍小，但重建用的是重读的读数：读不出、读回全零的槽照挂载时读坏钉在最旧 txg，m 只会取小。264:6:4:0 n = 15 那一刀上 `P5:all/all:e+P6:-/all:e` 与 `+P6:all/-:e` 两种写流逐条相同，挂载交回的根、F 与不注错时相同（`out/k2-ms-264-n15.txt` 第 4、5 行，与第 1 行不注错的那一次比），实现员那一条「重读只坏盘 1」与它对称的「只坏盘 0」都在里面。

推翻条件：任一种部分坏的重读计划下丢盘读回坏，或挂载交回的 F、写行的根高于不注错时。

### 2. `Recovery(NoValidRoot)` 不在 D23 已定项 14 的单子上（K2-b）：不是打中

`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-harness/src/mount_transparency.rs:2248`「MountError::Recovery(_)」排在交 `None` 的那一串里，调用者在 `/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2637`「WritableMountRefusalVerdict::ReasonNotOnTheD23List => {」那一支，拒因不是单元区墙就报红。

今天走不走得到：崩溃注入第二、三截那次挂载起在崩溃镜像上、不带故障层（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2567`「ReadFaultsUnderTheRefusedMount::TheRefusedMountReadTheCrashImageWithoutFaults;」）。没有故障层时重建那一遍读到的就是择根那一遍读到的字节；择根择出的那条根按它自己指着的表不会被抛弃（表里的行都是更晚的实例写给更早的实例的），重建那一遍至少读得出它，`newest_valid_root_txg_of_the_readings` 恒取得到，`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:1935`「return Err(MountError::Recovery(RecoveryFailure::NoValidRoot));」走不到（推的；k2ms 与 pass 两表里这一拒只出在第 5、6 两遍都整遍坏的计划上，量过）。所以「连坏两遍那一形挂载透明该不该判违例」在今天的判官上是空问题。

条款怎么写（零轮，只在我这份读代码上推过）：这一拒的判据「重建那一遍与它的重读都读不出一条不被抛弃的自证有效根」在崩溃镜像上复算恒为假（理由同上），和「读阶段判完之后系统配置又读不出」同形。照那一条的写法放进单子：被拒那次挂载带故障层时复算为假不算不合法，不带故障层时复算为假即不合法（照 `judge_writable_mount_refusal_of_a_crash_state_knowing_the_read_faults_under_the_refused_mount` 里改法 G3 那一支）。不放进单子的后果：将来有带故障层的调用者（例如 E171 那类装置换成瞬时读错）时，这一拒在合法历史上报红。

### 3. `192:9:3:1` n = 8 的 `w:but1/all:z`、`w:all/but1:z` 为什么拒、该不该拒（K2-c）

复现：冻结副本上两段（连同读错的两种）都在任何写之前拒，`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { instance_to_acquire: InstanceGeneration(3), publish_index: 0, …`，不注错的那一次照常挂上（`out/k2-192k9-n8-screen.txt` 第 1–5 行）。

为什么拒（读代码 + 量）：两种计划都让一块盘的根环在整次挂载里除最多一条之外读回全零。10c 之前全零当「从没写过」：那块盘上读得出的最新根就是剩下那一条（或一条都没有），当场放开一直到它；10c 之后全零的槽记成 `AllZeroAtMount`、钉在 [0, 3S) 里，`but1/all` 那一形盘 1 一条有效根都读不出，m 取不到，当场放开只到钉着的最旧 txg（`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:2041`「None => oldest_valid_root_in_the_ring,」），其余全扣到暖机落满每块盘；单元区 192 槽的窄盘上，取号之前在分配器拷贝上预演写行那次发布就取不到落点（publish_index 0），在取号之前拒、盘上不动。`all/but1` 那一形盘 0 整盘读不出，同一个机理。拒因的文档自己写明这一处条款没定：`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-core/src/mount.rs:221`「取号之前的准入怎么把这次挂载要写的落点算进去，条款没定」。

这一拒有多大（fxw，冻结副本对第 17 批攻方的树，`out/summary.txt` 前 72 行对 `research/prompts/m3-batch17-code-r1-opus-model/out/summary.txt` 同名行）：

| 计划 | 窄盘 813 段：写前拒 / 有坏的段（10c 之前 → 今天） | 宽盘 253 段：写前拒 / 有坏的段 |
|---|---|---|
| `w:all/but1:z` | 0 / 53 → 165 / 0 | 0 / 0 → 0 / 0 |
| `w:but1/all:z` | 0 / 98 → 165 / 0 | 0 / 0 → 0 / 0 |
| 另外 34 种（含这两种的读错版，各 165 / 0） | 逐行相同 | 逐行相同 |

今天读回全零的 17 种计划与读错的 17 种逐种同数（另一种是不注错的对照）（例：`w:-/all:z` 与 `w:-/all:e` 都是 165 拒，`w:all/only1:z` 与 `:e` 都是 197）。

该不该拒：该。10c 之前同一批窄盘历史里这两种计划丢盘之后读回坏 53 段、98 段（第 17 批判决甲一那一格），今天全部换成写前拒、0 坏；在那块盘一条有效根都读不出的前提下，能让写行落得下的只有放开钉着的那一截，而那一截正是丢掉另一块盘之后幸存盘上择中的根还引用的单元（甲一那一形）。拒的代价只出在窄盘上，数与读错那一形相同（165 / 813），是第 17 批判决第二节已经采为代价数的那一族多了读回全零这一种进入法，没有新的拒法。

四句：不分辨臂（没有候选臂，被问的是这一拒对不对）；系统当时看不看得到判别它的东西——看不到，挂载那一刻读回全零与没写过的槽字节相同（这就是用户定乙的理由）；不是任何判据的打中；不适用。

推翻条件：找到一段窄盘历史，这一拒之后换成「当场放开到读得出的那条根」也没有任何丢盘读回坏。

## 三、K3 I-7.9

### 1. 10c 之前的树上 `w:all/but1:z` 两盘都在 I-7.9 红 4116（K3-a）

复跑（10c 之前那份树 `m3b17-today/tree` 的拷贝，`mount.rs` sha256 77678733…，与第 17 批判决里那棵树同一个值）：层 0 原型枚举 28733 个状态，`checker_violated={"I-7.9": 4116}`，丢盘读回 0 坏（`out/k3-pre-l0-5.txt` 第 6 行），与第 17 批攻方的数对上。从后往前查出的判红原文（同一份文件第 3 行起）：

> 实例 3 txg 24 那条根把回退下界 F 从 21 抬到 22，高于它之前的根算出的抬 F 上限 21：每块盘上最新的有效根 {0: 23, 1: 22}，非空有效根 [21]，空不空判不了的有效根 []，最旧有效根 txg 21

判：实现越过了盘上的真上限，checker 判得对。txg 21 那条根（卸载那一串的最后一条，所选那一版由记录施加到 21）住在盘 0，这次挂载整次读盘 0 根环都读回全零；准入抬 F 算上限时全零的槽不并（用户 2026-10-04 定），实现看见的有效根里没有一条非空根，上限取它看得见的最旧有效根 22，F 抬到 22。镜像上 21 读得出、是唯一一条 F 之上的非空有效根，上限是 21。这就是正文第二节列为用户已认的那一样：`research/prompts/_m3-batch10-code-r1-body.md:27`「读坏成全零的槽 F 可能越过住在里面的有效根」。不算打中。

### 2. 那个代价是不是只有这么大（K3-b）

冻结副本上（10c、10e 都在），fxw 那 72 种计划去重之后 14950 个全落盘状态，I-7.9 红 3835 个，全是读回全零的计划（按写流去重之后各计划的状态数：`w:only1/all:z` 648、`w:only1/only1:z` 529、`w:-/only1:z` 450 ……），读错的 17 种 0 个（`out/fin-frozen.txt`）。从每条判红原文里取「抬到 b」「上限 c」「非空有效根」三样算：

| F 越过上限几个 txg | 越过的非空有效根条数 | 状态数 |
|---|---|---|
| 1 | 1 | 2432 |
| 2 | 1 | 1169 |
| 3 | 1 | 220 |
| 4 | 1 | 14 |

每一个都只越过一条非空有效根，就是 F 之上唯一的那一条；它之后的根都是空发布（同一组用户可见树），越过之后那个状态仍由后面的空根带着，可退到的不同状态一个没少（推的，从判红原文里「非空有效根」只有一条推出来）。这一族的起点都是卸载半途崩溃（F 已抬到现行那一版），F 之上只有一个状态；F 低、环里多个非空状态住在读回全零的槽里的那一形（第三轮 `k2-zeros` 那一族）这一轮没造，越过几条没量，见第七节。

推翻条件：任一判红原文里「非空有效根」有两条以上落在 [上限, 抬到的 F) 里。

## 四、K4 checker 档装置（E）

### 1. 掉电接缝不在崩溃段段首的状态（K4-a）：没打中

接缝只在落在崩溃段第一个写之前时才交给记录核对器：`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-checker-tier/src/crash_injection.rs:1517`「*first_write_after_the_power_loss <= first_write_of_the_segment」。不在段首的接缝要求掉电之前录下的最后一步不关段（不是屏障、不是 FUA 写）。掉电在第一个落进 journal 环的写到达时触发：`/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-harness/src/history.rs:4667`「if self.journal_ring_offsets_in_bytes.contains(&offset.0) {」。

量（原型 `seamprobe`，只跑历史、切段、数接缝，切段用 harness 的 `writes_and_segments_with_stream_indexes`，与崩溃注入同一条规则）：写死 43 段（第 10b 批那段孤儿历史，加掉电那次改动的内容长度 7 种 × 掉电前覆盖写 0 / 1 / 3 次 × 掉电后覆盖写 0 / 2 次），再加偏向留下孤儿那组比重的随机历史 120 个种子 × 30 步：接缝 332 道，段首 332、段内 0（`out/seam-s120.txt` 第 334 行「SEAMTOTAL heads=332 inside=0」）。332 道接缝之前录下的三步全是「Write, Barrier, Barrier」：掉电那次发布的单元写之后每块盘一道屏障，之后才是第一个 journal 记录写，段在屏障处关上。

推翻条件：一段历史里掉电之前最后一步是不关段的写（例如某种发布在单元写与第一个记录写之间不过屏障），接缝落进段内。今天的写序下走不到（量过的范围内）；写序改了这一格要重判。

### 2. 孤儿历史上三段各摆得出二次崩溃吗（K4-b）：观察，不算打中

原型 `seamprobe/src/bin/phases.rs` 调崩溃注入的库函数（不跑 checker 档包的测试），读 `tally.second_crash_points_by_phase`（`out/phases-s24.txt`）：

- 写死的孤儿历史（`inject_a_crash_at_the_start_of_the_last_step_of_the_history_whose_mount_erases_heads_of_orphans`）：一个第一次崩溃，取号、写行、暖机、抹头、清扫发布五段各摆了一个二次崩溃。
- 偏向留下孤儿那组比重的随机历史 24 个种子 × 30 步、每段抽 3 个第一次崩溃：每段都是取号 3、写行 3、暖机 3，抹头与清扫发布 0 个。随机历史走产品阈值（369 行）不扫，抹头那一段只有写死的那一条摆得到；它那一段每个崩溃状态另有专门用例枚举（`crash_points_of_the_instance_table_sweep.rs`，没跑，按文件头）。

### 3. `claimed_state_missing_unit` 压着哪条条款（K4-c）：不是打中

判据原文 `/tmp/claude-1000/m3-batch10-code-r1/tree/crates/singlefs-harness/src/memory_pool.rs:778`「恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元两份都不在」。它是记录核对器的二元判据，按条款就不进不变量清单：`.claude/kb/decisions/13-验证路线.md:190`「需要第二个输入的核对归记录核对器」，验证构建那一份写的是 `.claude/kb/verification-build.md:23`「D13（验证路线） 已定项 7 故意不给编号」；判据的出处是 E77 的独立审计（同一行括号里点名）。`invariants.md` 零命中是条款这样定的。

## 五、没打中的形状与取样范围

| 形状 | 取样 | 结局 |
|---|---|---|
| 整次挂载每块盘一种选槽法（不坏 / 全部 / 除最新一条 / 只最新一条）× 读错 / 读回全零 × 交回之后坏不坏 | 窄盘 20 个形状 813 段、宽盘 6 个形状 253 段，各 36 种计划 | 0 坏 |
| 单段整遍读坏窗口 [a, b)，all/all | 9 个形状，80714 段 | 0 坏 |
| 单段整遍读坏窗口，all/all、all/-、-/all 三种 | 5 个形状，133748 段 | 0 坏 |
| 重建那一遍全坏 + 重读部分坏（30 种）× j ∈ {0, 3} | 7 个形状，8215 段 | 0 坏 |
| 挂载之后仍坏（`:p`，14 种）× 覆盖写 j ∈ {1, 3, 6, 13}，看全落盘状态 | 8 个形状，5876 个状态 | I-3.1 0 红；I-7.9 只在读回全零的计划上红（Q3 代价） |
| 全落盘状态 checker | fxw 72 种计划去重 14950 个 | 只有 I-7.9 红 3835（Q3 代价） |
| 掉电接缝 | 332 道 | 全在段首 |
| 二次崩溃分段 | 写死孤儿历史 1 段、随机 24 段 | 写死那段五段齐；随机里抹头 0 |

由用户决定的那几步放开扫的范围：卸载刀全扫（窄盘族每个形状里落在「系统配置已带新 F、带新 F 的根没落满」那一截的每一刀），挂载之后的覆盖写次数 j 取 0、1、3、6、13（宽盘族；窄盘族原型里不做覆盖写），读坏落在哪几遍读按原型认的整遍边界扫全部区间。没放开的：挂载之后的别的用户动作（unlink、回退、再卸载）。

## 六、这条腿自己的限度

- 「丢盘读回坏」只判了挂载写流的前缀刀（每个写之前崩一次，丢一块盘之后整读），没展开段内整写子集。层 0 枚举只跑了 K3 那一段（10c 之前的树 28733 个状态，跑了两次：第一次的 I-7.9 原文打印写得太慢、被我停掉，枚举那一半已跑完；第二次完整）。冻结副本上 `320:9:3:1` n = 20 `w:-/only1:z` 那一段的层 0（闭式 278600 个状态）跑到 104496 个时同样因为打印那一刀太慢被我停掉，没出数，改用全落盘状态的 checker（三 2）。三次合计枚举 161962 个状态，没超过约 10⁷ 的总量；每次跑之前都看了原型打的 `closed_form_states`，都在 10⁶ 以下。
- 全落盘状态只是每段历史崩溃状态里的一个；I-3.1 / I-7.9「没红」只对那一个状态成立。
- 估时：`fxw` 那一段我先跑一件（最小的形状）5 秒，按它估全量明显偏小，实际 92 件分 7 批、每批 15 件、共 861 秒；`k2ms` 12 件一批 299 秒；`fin` 92 件 7 批 194 秒；`k1j` 8 件一批 163 秒；`seam` 120 个种子 199 秒；`phases` 24 个种子 194 秒。都没设批限时。第一次编原型时外面套过一次 `timeout 235`（编译，不是跑批），之后没再套。
- 原型族只有两个起点家族（卸载半途崩溃之后再挂的窄盘族、宽盘族）与掉电孤儿历史；F 低、环里有多个非空状态的起点没造，K3-b「只有这么大」只对这一族说话。
- K1 线索（一 5）是读代码推的，没造被抛弃的根。
- 所有数在冻结副本（或 10c 之前那份树）的拷贝上量，不是入库装置上的数；腿提的改法（K2-b 的条款写法）被攻过零轮，只在我这份读代码上推过，没实现、没跑。

## 七、没做什么

- K3 的「I-7.9 两支只留锚在镜像最新根上的窗口会不会漏红」与「I-8.5 改报不适用的边界」：没攻。
- K1-d 用户认下的 I-3.1 多记账那一形本身（没有读故障、崩在记录之后根槽之前的全零槽红一版；一直读不出时最长一圈）：这两族起点都走不到，时长没量。
- K2：只读坏根环槽，没读坏 journal 记录、系统配置与实例表；挂载之后的用户动作只放开了覆盖写次数。
- 一 5 那一形的盘面没造（要先造出被抛弃的根）。
- K5、K6、K7 不归这条腿。checker 档包的测试、层 0 用例、门禁一个没跑。

## 八、草稿与副本

草稿目录 `/tmp/claude-1000/m3-batch10-code-r1-opus/`：`w17/out/` 是全部原始日志（`fxw-*`、`pass-*`、`k2ms-*`、`k1pass3-*`、`fin-*`、`k1j-*`、`k3-*`、`seam-*`、`phases-*`），`w17/tables/` 是跑批的件表，`b17model/` 是第 17 批攻方模型脚本的拷贝，`tools/` 是补丁脚本的工作版。三份树拷贝与三个编译目录（`w17/jia`、`w17/ms`、`w17/pre`、`w17/target-jia`、`w17/target-ms`、`w17/target-pre`）交回前删掉；`rerun.sh` 从冻结副本重新拷、重新编。模型目录里的 `out/` 是从这些日志里抽出来的判定行，复跑之后用 `rerun.sh compare` 比 `summary.txt`。

