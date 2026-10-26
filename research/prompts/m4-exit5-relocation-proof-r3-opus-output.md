# m4-exit5-relocation-proof-r3 攻方腿（Opus）报告

日期 2026-10-26。攻击面：正文第三节 Z1–Z5，第三轮角度（「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核）。禁读清单里辩方腿这一轮的产出没读。攻的是第二轮判决第二节站住的整形：core 直验连胶水、信任清单替 `--no-cheating`、按全路径逐项验、判器键去掉规格与证明块、改后协议（正推那份 `gc_ring_proofs` + `relocation_interval_proofs`）与 G7 改写。不重复第一、二轮判决第二节已攻过的角度（第二轮攻方攻的是判决字面两处修补的最直接写法与它自造的 `patched_slot_is_unreferenced`；这一轮攻的是正推那两份真文件本身）。

代码行号：core 引冻结副本 `~/.local/share/singlefs/work/m4-close-rounds-frozen/`，另注主工作区的；改后协议两份文件引正推第二轮模型目录 `research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/src/`（与我模型目录里 bundle 的那两份逐字节同，sha256 见下）；门禁脚本引主工作区。副本上量的数都标「副本」，不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 分不分辨臂 |
|---|---|---|---|
| Z1 | 打中（变体二，Verus 证）；变体一被排除 | 改后协议是两份不组合的模型：根环版（`gc_ring_proofs`）有回退候选、但按起点槽号护（单槽，无跨度）；区间版 P4（`relocation_interval_proofs`）有跨度、但前提只对现行映射量化、没有回退候选这一维。合不成「护住回退候选的多槽单元的每一槽」——回退候选的两槽单元的第二槽照样可写、照样被搬迁覆写，回退目标读坏。单槽写进回退目标读的槽那一形在根环版被排除 | 分辨：打「单槽根环版 + 区间版 P4、两份各证各的」；根环版改成带跨度的区间版（正推 5.5 自己点名未做）的不中 |
| Z2 | 部分打中（推的，针对采纳的未实现改法） | 采的信任清单、`axiom` 一律不许、cfg 分叉判红、MIR 比对今天一条都没写进 `verus-proof-check.py`（grep 0 处），全是零轮。MIR 比对若照正推 Y1 写，泛化地挡住 cfg 这一类（任何编法差异都让 MIR 不同）；residual：差分测试只在有界域取样、规格错只在没取到的输入上时放过（推的） | 分辨：打「只靠正则 + 必红世界判规格对不对」；配 MIR 比对 + 运行时对拍的收严不中 |
| Z3 | 没打中（射程撑得住，有限度） | 用 `lib_function_units.py closure` 从冻结副本现收三条闭包（发布 836 / 回退 1057 / 崩溃后读 654）。改抽象状态的 core 路径都在这三条里：GPU 垃圾回收（`background_compute.rs`）今天没有 GPU 计算路径、只记设置、0 处写盘，不改抽象状态；区域扫描 `region_scan` 只读；挂载续做（`live_compaction_intent_of_the_entries`、`publish_sequence_failed`）在回退闭包里。没找到三条闭包之外改抽象状态的路径 | — |
| Z4 | 部分打中（推的，针对采纳的未实现改法） | 「算键前剥 ghost」今天没写进 `lib_function_units.py`（r2 opus 量到加 ensures 键就变）。剥 ghost 由 Verus 擦除看是 sound 的，但采的是词法剥（`lib_function_units.py` 按词法单元），词法剥会认错 exec / ghost（宏把 ghost 展成 exec、exec 文本落在证明样构造里）；采的「改执行代码键必变」必红只挡得住直白的，认错那一类绕得过（推的，零轮） | 分辨：打「词法剥 ghost」；Verus 擦除粒度剥的不中 |
| Z5 | 打中（数核得出，但「全验行数」对的是比采纳射程小的那一个） | 1073 与 1071 的差现核清楚：两闭包并集 1073 个 fn 单元里 1071 在 core，差的 2 是 replay-prefix 的 `gc_crash_proofs::released_record_can_be_reclaimed` 与 `records_at`（现收，列出名字）。但正推的「全验约 11 万–12.5 万行」是对**两闭包**并集（1073 fn、24970 行）算的；判决采纳的射程是**三闭包**（发布、回退、崩溃后读），现收 1185 fn、1101 core、29807 行，比 24970 多 19%。倍数又只在 2 个 A 类样本上取过，占行数多数的 C / D 类（411 fn、16392 行）一个样本都没有 | 分辨：打「代价数 = 对两闭包、单点倍数」；对三闭包、每类各取样的不中 |

## 复跑

模型目录 `research/prompts/m4-exit5-relocation-proof-r3-opus-model/`（`tools/rerun.sh` 照 `scripts/agents/three-way-rerun-template.sh`，只改 `rerun_steps` 那一段）。复跑命令（仓根起，草稿目录给一个新的；这一次约 90 秒，看内存包装排不排队）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-exit5-relocation-proof-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

它做三件事：一、把 bundle 的攻击 crate（`verus/lib.rs`，引正推那两份改后协议 + 冻结副本两份基线 + 我的 `attack.rs`）直接起 Verus 0.2026.09.20（`--no-cheating --rlimit 10`）验，A1/A2 两处打中应当全绿；二、两份必红（断言 gap 不存在）各应当红 1；三、从 `--frozen` 用 `scripts/gate/lib_function_units.py closure` 现收三条闭包与并集 fn / 行数。末了与 `out/summary.txt` 逐行比，不同退 1。

`out/summary.txt` 全文（副本，量的）：

```
name=attack 55 verified, 0 errors
name=mustred-a1 13 verified, 1 errors
name=mustred-a2 9 verified, 1 errors
name=closure entry=publish fn=836
name=closure entry=rollback fn=1057
name=closure entry=recover fn=654
name=union-two fn=1073 core=1071 lines=24970
name=union-three fn=1185 core=1101 lines=29807
```

模型目录每个文件的 sha256（相对模型目录）：

```
35f75de1110509bb76a3e8e122e39f54b1b4b682be85faa6465ce3343e560121  ./out/summary.txt
9139f5c613abf49a687d6aec72c334947b0d0e6c3e89cdd6a24537955d8e9764  ./tools/rerun.sh
122b3774582bb82885f5fc981106267d1d45070cf2cada740c94ce353ef03b77  ./verus/attack.rs
274e57685cfe504d53daee9be83c66f081de1a9b40c075d233fcec723d4acdfd  ./verus/compaction_plan_proofs.rs
fe8452c4092ffc5826391312e4575bc1994f4296c4340bb3a29dde41cf9527c1  ./verus/gc_crash_proofs.rs
09a4f07f08a752233e4542c629327efcdaf6c2d7fa57358073c640f459ea9511  ./verus/gc_ring_proofs.rs
d8065509644a1d5ec78f496b76fafc09c0036e2365536d3e98ba85783e5f6257  ./verus/lib.rs
d33281c7d26ed889270b4c4090bc9506032a13818f2c029a765afdb370598ba4  ./verus/mustred/a1_body.rs
c1114273a7af6fe3f8295e342f1b6d2e80c94a50486ef6ec527dd818226d57da  ./verus/mustred/a1.rs
b088dd275a395ac392917cd4d02a78df6a0ab51061ab9636c305785ed3bf23b3  ./verus/mustred/a2_body.rs
ebd12067617445ee3c203cbadf92fd6f4efa85c19b232807ceecd41f2c7634f3  ./verus/mustred/a2.rs
d7a3a32662a63f8c68f0ccacbb9d20a8722bfb283a399dd43492c43251ff8e4d  ./verus/relocation_interval_proofs.rs
```

`gc_ring_proofs.rs`（09a4f07f…）与 `relocation_interval_proofs.rs`（d7a3a326…）与正推第二轮模型目录 `src/` 下同名两份逐字节同（sha256 见那份报告第 68、69 行），一个字没改。

取样与估时：这一腿没造崩溃历史（理由在「这条腿自己的限度」），没有崩溃状态数可算。每份 Verus 几秒到几十秒，三条闭包各约 5 秒。

## Z1　改后协议与区间 P4 撑不撑得住

### 打中（变体二）：改后协议是两份不组合的模型，回退候选的多槽单元第二槽不设防

判决 Y5 的关门条件（`research/prompts/m4-exit5-relocation-proof-r2-main-verification.md:23`）：「第三轮在正推那份协议上复跑攻方两个反例的变体（写进回退目标的槽、两槽单元的第二槽），排除了才算这一格关上」。我把正推那两份真文件（`gc_ring_proofs.rs`、`relocation_interval_proofs.rs`）原样引进一个攻击 crate，复跑两个变体。

**变体一（单槽写进回退目标还在读的槽）：在根环版被排除。** 根环版 `RingStep::Write` / `Reclaim` 的允许条件是 `ring_slot_is_unreferenced`（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/src/gc_ring_proofs.rs:90`「        RingStep::Write { slot, .. } | RingStep::Reclaim { slot } => ring_slot_is_unreferenced(s, slot),」），它要每个回退候选都避开那个槽。`attack.rs::a1_ring_protection_is_start_slot_only` 证出：往被候选引用的起点槽 20 写**不**允许（`!ring_step_enabled(s, Write{slot:20})`）。第二轮攻方那个「搬 key7 到回退目标还护着的槽 20」在正推这一版被挡住——比它自造的 `patched_slot_is_unreferenced` 严。

**变体二（两槽单元的第二槽照样可写）：没被排除。** 病根在协议的避让只按起点槽号：`ring_map_avoids(mapping, slot)` 只比 `mapping[key] != slot`（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/src/gc_ring_proofs.rs:77`「pub open spec fn ring_map_avoids(mapping: Map<u64, u64>, slot: u64) -> bool {」），根环版的状态里一个单元只占一个槽（`disk: Map<u64, Seq<u8>>`，没有跨度字段）。而 core 里一个单元跨多槽：`Placement` 有 `pub span: u64,`（`~/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/allocator.rs:1121`），分配记录有 `pub span_slots: u16,`（同文件 `:70`）。`a1_…` 同一证明里证出：同一个状态下，往第二槽 21 写**允许**（`ring_step_enabled(s, Write{slot:21})`）——因为没有哪个 mapping 值等于 21，而 21 在 core 里是回退候选那个两槽单元的第二槽。

区间版 P4 那一半有跨度、却没有回退候选这一维：它的前提 `destination_runs_are_disjoint_and_free` 只对传进来的 `mapping` 量化（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-model/src/relocation_interval_proofs.rs:92`「        0 <= index < count && #[trigger] mapping.contains_key(key) ==> !runs_overlap(」）。`attack.rs::a2_interval_premise_ignores_rollback_target_runs` 证出：现行映射只有 key7→[10,11)，把 key7 搬到目的区间 [21,22)——它与现行映射里唯一的区间 [10,11) 不重叠，所以 `relocation_plan_is_well_formed` 成立、定理保住现行映射每个 key 的读；但回退目标里 key4→[20,22) 的那个两槽单元，搬完之后第二槽 21 从 [88] 变成 [42]，`unit_read(moved, [20,22)) != unit_read(storage_before, [20,22))`。区间前提里没有回退目标的区间这一维，放过了它。

两份必红（断言 gap 不存在）各红 1，证明这两处打中是吃重的：`mustred-a1`（断言第二槽 21 受护）红，`mustred-a2`（断言回退目标两槽读不变）红。原样：

```
name=attack 55 verified, 0 errors
name=mustred-a1 13 verified, 1 errors
name=mustred-a2 9 verified, 1 errors
```

正推自己点了这个 gap：5.3「但**没有一条引理把 `Placement`（u64）与 `SlotRun`（int）接成一条证明链**」（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md:414`），5.5「没有一条证明说 core 的分配器回收条件蕴涵 `ring_slot_is_unreferenced`」（同文件 `:431`）。冻结副本的基线也写明这一层没证：「// 真实分配器必须另证其回收条件蕴涵这一条件；本模型没有从 floor 推出它。」（`~/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-replay-prefix/src/gc_crash_proofs.rs:70`）。判决 Y5 把这两份当「已采的改后协议」、把关门交给第三轮复跑两个变体——复跑结果是变体一关上、变体二没关上，所以 Z1 这一格按判决自己的条件**没有完全关上**。

**四句**：① 分辨：打「单槽根环版 + 区间版 P4、两份各证各的」那一叠法；根环版的 `ring_map_avoids` 改成按区间、映射值带跨度（正推 5.5 自己点名「推的，没写，义务数没量」）的不中。② 判别子观测不到：单看根环版，它决定 Write 是否允许那一刻手里只有起点槽（mapping 值），一槽单元和两槽单元在它的状态里逐字相同——这是 `evidence-discipline.md`「判别子观测不到」那一类；但补法（给根环加跨度，区间版已有那套机械）够得着，所以打中的是「两份改后协议不组合」这一射程缺口，不是整个改后协议的不可判。③ 满足的字面分句：Z1「这两形在改后协议上是否被排除」——变体一排除、变体二没排除。④ 改法：正推点名的「协议避让按区间、映射值带跨度」实现后 A1 的第二槽那一写会被挡（21 落在跨度里）；但那个改法零轮、没写、义务数没量，交出去标「零轮」。

**什么现象会推翻它**：正推那份 `gc_ring_proofs.rs` 的 `ring_map_avoids` 本来就按区间写、`RootView` 的映射值带跨度（现查是单槽：`disk: Map<u64, Seq<u8>>`、`mapping: Map<u64, u64>`）；或判决另有一条引理把 `Placement` 的 span 接进根环协议（现查两份文件里没有跨度接桥）。

## Z2　信任清单与门禁的判法撑不撑得住

### 部分打中（推的，针对采纳的未实现改法）

判决 Y2、Y4 采的那一套（去 `--no-cheating`、每处 `assume_specification` / `external_body` / `axiom` / `uninterp` 进信任清单、`axiom` 一律不许、cfg 分叉判红、产品与 Verus 的 MIR 逐函数相同进门禁）今天一条都没写进判法：`grep -ni 'mir|信任清单|verus-trusted|strip.*ghost'` 在主工作区 `scripts/gate/verus-proof-check.py` 命中 0 处；`CHEATING_MARKER`（主工作区 `scripts/gate/verus-proof-check.py:473`，正则是 `external_body|external_fn_specification|assume\w*|admit|verifier::external`）里没有 `axiom`。所以这些是零轮的改法，不是实现，我只能推、不能在入库装置上实测。

对正文 Z2 问的那几种绕法，按采的改法（若照正推写）逐个看：

- **cfg 换别的原子、宏展开**：判决 Y1 采「产品构建与 Verus 看到的 MIR 逐函数相同进门禁」。这一道泛化地挡住 cfg 这一类——Verus 编（`verus_keep_ghost` 为真）与产品编（为假）只要有任何一支展开成不同的 MIR，逐函数比就红，换哪个 cfg 原子都一样。residual（推的）：MIR 比对比的是它当场起的那两次构建；一个在那两次里设成同值、却在部署时按运行期特性翻的分岔（例如 build.rs 读环境变量）两次 MIR 相同、门禁看不出。这一道门禁本身零轮、没写。
- **trait 默认方法、泛型单态化**：Verus 验的是泛型 / trait 的那一份规格；Rust stable 没有特化，单态化不改行为，默认方法被覆盖时 MIR 比对按函数项看得出不同的函数体。没造出反例（推的，没起 Verus）。
- **差分测试判不判得出一条规格写错的 std 函数**：正推 2.4 的 `assume-demo` 演示过差分测试抓 `Option::copied` 错规格。residual（正推自己列的限度，不是新打中）：差分只在有界域取样（`u64` 全宽取不满）、含量词 / `Seq` / `Map` 的规格写不成可执行孪生、设备读写与校验和没有 std 对应物没参照——规格错只落在没取到的输入上时放过。

**四句**：① 分辨：打「只靠 `--no-cheating` 的正则 + 必红世界判规格对不对」；配 MIR 比对 + 每条登记规格一个运行时对拍的收严不中。② 判别子观测不到：必红世界只证「这条规格被用到了」，证不出「规格对 std 为真」——Verus 那一侧看不见 std 的函数体，这是判别子观测不到，改判据是新一轮（与第二轮打中同形，不另记）。③ 满足的字面分句：Z2「在这一套下还有没有一种写法……门禁不红」——MIR 比对实现后 cfg 这一类不红；差分域外的规格错仍不红（推的）。④ 改法：MIR 比对 + 运行时对拍都零轮、没写，交出去标「零轮」。

**什么现象会推翻它**：`verus-proof-check.py` 已经写进 MIR 比对且它在门禁的离线环境里跑得起来（现查 0 处）；或差分测试已覆盖 `u64` 全宽（有界域取样推翻不了）。

## Z3　射程的完整性

### 没打中：三条闭包之外没找到改抽象状态的 core 路径（有限度）

判决 Y3 定射程 = 发布、回退、崩溃后读映射三个闭包的并集。用 `scripts/gate/lib_function_units.py closure` 从冻结副本现收三条闭包，fn 数与第一、二轮量的相同（副本）：

```
name=closure entry=publish fn=836
name=closure entry=rollback fn=1057
name=closure entry=recover fn=654
```

正文 Z3 点名的几条疑似「改抽象状态而不在三条闭包里」的路径，逐条现查：

- **GPU 垃圾回收（`background_compute.rs` 那一路 / `BackgroundGpuCompute`）**：不在三条闭包里（并集里 grep `background_compute` 0 处）。但它今天不改抽象状态：「今天没有一种后台任务有 GPU 路径：选了 `gpu` 时每一批……在 CPU 上算」（`~/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/background_compute.rs:5`），整份文件 0 处 `BlockDevice` / `PoolReader` / `persist` / 写盘（`grep -cE` 得 0）。实际搬迁走的是普通发布路径（在发布闭包里），`background_compute.rs` 只做设置准入与每批记账。所以它在三条闭包之外是对的——它不改抽象状态。
- **区域扫描（`region_scan`）**：不在并集里，但它只读（`scan_region` 喂给 `compaction_plan` 选段），不改抽象状态。
- **挂载续做整理意图**：`live_compaction_intent_of_the_entries`、`publish_sequence_failed`、`version_to_publish_the_rollback_after` 都**在**回退闭包里（并集里现查到），续做发成的发布走发布路径，已在射程内。
- **整理开关的运行期切换（`set_reuse_window`）**：第二轮攻方已打中（pub 的只供测试开关在闭包外、拨到 `ForcedToZero` 绕过 F），判决 Y3 已采「进第三轮必攻」并要求挪进默认关的 cargo 特性——这一条不重复攻，只记它仍是零轮采纳的改法。

结论：出口 5 的恒等只管 GC 搬迁这一路，别的改抽象状态的操作（truncate、建删快照、删文件）本就不在出口 5 的射程里、本就该改抽象状态。在 GC 这一路上，没找到三条闭包之外改抽象状态的 core 路径。限度见「这条腿自己的限度」。

## Z4　判器键与重判代价

### 部分打中（推的，针对采纳的未实现改法）

判决 Y4 采「判器闭包键在算指纹时去掉 Verus 的规格与证明块（只留会编进产品的代码），把『只改证明、产品代码不变』做成键不变，配三问自证」。这一条今天没写：主工作区 `scripts/gate/lib_function_units.py` 里没有剥 ghost 的逻辑（第二轮攻方量到加 ensures 键就变），`verus-proof-check.py` 里 `strip.*ghost` grep 0 处。零轮。

- **「只改证明、产品代码不变」是不是键不变**：剥 ghost 由 Verus 自己的擦除看是 sound 的（`proof` 块、`spec` / `proof` fn、ghost 变量都擦掉，擦掉的东西按 Verus 的保证动不了产品行为）。但采的键算在 `lib_function_units.py`——它按**词法单元**算（不是经 Verus 擦除）。词法剥 ghost 要它学会在 `verus!` 块里分出哪几段是 ghost；词法分法会认错：宏展开把 ghost 样的东西展成 exec、或 exec 代码文本上落在证明样构造里，剥错就把执行代码的改动当成 ghost 漏判（第二轮攻方 Y4 打中二④自己点过这个入口）。
- **会不会让一处真改了产品行为的改动被当成只改证明**：Verus 的模式检查不让 ghost 值进 exec 控制流（`ghost` 变量参与 exec 分支是模式错，编不过），所以「ghost 参与 exec 分支」这条路在 Verus 下走不通。剩下的唯一入口是上一条的**词法剥错**。采的「改一处执行代码、键必变」必红挡得住直白的改动；词法剥把一段 exec 错划进剥掉的区域时，那一格必红取样取不到就绕得过（推的，零轮）。
- **做不到时整库重判代价**：第二轮攻方量到并集里 549 个 fn 同在判器闭包里（副本）。这一轮我没重量键（第二轮已量，`lib_function_units.py key` 在 core 函数上加 ensures 键变）。

**四句**：① 分辨：打「词法剥 ghost」那一做法；经 Verus 擦除粒度剥的不中。② 判别子观测不到：词法剥器要分 exec 与 ghost，而宏展开后的 exec 它看不全——这一面是词法工具的观测限度。③ 满足的字面分句：Z4「会不会让一处真改了产品行为的改动被当成只改证明」——词法剥错时会。④ 改法：改成经 Verus 擦除（HIR / MIR 粒度）算键、必红样本覆盖「exec 落在证明样构造里」——零轮、没写。

**什么现象会推翻它**：采的键算法已经经 Verus 擦除（不是词法）剥 ghost（现查 `lib_function_units.py` 是词法）；或必红样本里已有「exec 文本落在证明样构造里、改它键必变」这一格。

## Z5　第二轮报的数能不能核

### 打中：数本身核得出，但「全验行数」对的是比采纳射程小的那一个闭包集

- **1073 与 1071 差在哪（核得出，不是矛盾）**：用 `lib_function_units.py closure` 现收两闭包（发布 + 回退）并集，`union-two fn=1073 core=1071`。差的 2 个是 replay-prefix 里的核，不是 core：`singlefs_replay_prefix::gc_crash_proofs::released_record_can_be_reclaimed` 与 `singlefs_replay_prefix::records_at`（现收，逐个列出）。正推 Y3 的 1073 数的是全部 fn 单元（含这 2 个），攻方 Y4 的 1071 数的是 core_fn——两边都对、相容。
- **分类 A–D 的判法**：正推的 `closure_classes.py` 按「直接调用」归类（185 个 MIR 没配上名字的归了 A，所以 A 是上界、传递没算）。我没复跑它的 MIR 归类（要抽 MIR，这一腿没做，限度里记）；只现核了闭包 fn 与行数这几个数。
- **「11 万–12.5 万行」这个数核不住采纳的射程**：正推 3.5 的「全验（1073 个 fn、24970 行）：被验版本约 11 万到 12.5 万行」（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md:340`）算的是**两闭包**并集。判决采纳的射程是**三闭包**（发布、回退、崩溃后读映射）。现收三闭包并集 `union-three fn=1185 core=1101 lines=29807`——比 24970 行多 4837 行（+19%）、比 1073 fn 多 112 个。用同一个 4.5–5 倍对 29807 行外推是约 13.4 万–14.9 万行，不是 11 万–12.5 万。所以报出来的全验行数对的是比采纳射程小的那个闭包集。
- **倍数只在单点取过**：正推自己写「倍数取自两个原型……都是 A 类」「C、D 类的倍数我没量，只会更高」（`research/prompts/m4-exit5-relocation-proof-r2-sonnet-output.md:338`）。占三闭包行数多数的 C 类（291 fn、10160 行）与 D 类（120 fn、6232 行）合计 411 fn、16392 行一个样本都没取，这违 `evidence-discipline.md`「我是不是只在一个点上取了值」。正推已标这个数是「推的」，判决交用户表也标「零轮」。

**四句**：① 分辨：打「代价数 = 对两闭包、单点倍数」那一算法；对三闭包、每类各取一个样本、落成区间的不中。② 判别子观测不到：行数、fn 数都静态可数、看得到。③ 满足的字面分句：Z5「11 万–12.5 万行的推法……按那几份现数重算一遍」——重算后射程口径对不上（两闭包 vs 三闭包）、倍数单点。④ 改法：对三闭包并集（1185 fn、29807 行）、每类（A/B/C/D）各做一个原型量倍数，再外推——零轮、没做。

**什么现象会推翻它**：判决采纳的射程其实就是两闭包（现查 Y3「发布、回退、崩溃后读映射三个闭包的并集」是三条）；或 C、D 类的倍数做出来不高于 A 类的 5 倍（正推自己说「只会更高」）。

## 改法逐格表（我提的改法，只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 标 |
|---|---|---|---|
| 根环协议 `ring_map_avoids` 改成按区间、`RootView` 的映射值带跨度（正推 5.5 已点名未做） | Z1 变体二 | A1 第二槽 21 那一写应当被挡（21 落在候选单元的跨度里） | 推的（零轮） |
| 补一条引理把 `Placement`（u64，带 span）接进根环协议的回退候选保护 | Z1 | 把区间版与根环版合成一条链，变体二才真关 | 推的（零轮） |
| MIR 比对（产品构建 vs Verus）逐函数进门禁 | Z2 | cfg 这一类任何编法差异都红 | 推的（零轮，判决 Y1 已采） |
| 每条登记规格配运行时对拍，平局 / 空 / 重复显式枚举 | Z2 | 抓有可执行孪生的规格错；抓不到对拍没取到的输入 | 推的（零轮） |
| 键经 Verus 擦除（非词法）剥 ghost，配「exec 落在证明样构造里改它键必变」必红 | Z4 | 词法剥错那一类绕不过 | 推的（零轮） |
| 代价数对三闭包并集（1185 fn、29807 行）、A/B/C/D 各取样外推 | Z5 | 全验行数对上采纳射程 | 推的（零轮） |

## 没打中的形状

| 形状 | 试了什么 | 结果 |
|---|---|---|
| 变体一（单槽写进回退目标读的槽）在正推协议上仍成立 | `attack.rs::a1`，证 `!ring_step_enabled(s, Write{slot:20})` | 被排除：根环版比第二轮攻方自造的 `patched_slot_is_unreferenced` 严，这一形关上 |
| 三闭包之外改抽象状态的 core 路径 | `lib_function_units.py closure` 三条 + grep GPU / region_scan / 挂载续做的成员 | 没找到：GPU 不写盘、区域扫描只读、挂载续做在回退闭包里 |
| 分类 A–D 的判法现量复出 | 没复跑 `closure_classes.py`（要抽 MIR） | 没做，记进限度 |
| Z4 判器键在正推采的剥法下变不变 | 没重量（第二轮已量加 ensures 键变） | 没做：剥 ghost 零轮没实现，量不了采后的键 |

## 这条腿自己的限度

- 崩溃状态没枚举：这一腿的反例都在 Verus 单进程里，不靠崩溃放量；Z1 的两槽单元攻击是协议层的（槽是原子地址），没造真历史、没调 `enumerate_crash_states`、没跑 checker；core 实际的回退保护在崩溃放量里中不中，要主 agent 判要不要另派崩溃那一维。
- Z3 的「没找到三条闭包外改抽象状态的路径」是按闭包成员 + 读模块文档 + grep 写盘依赖得的，不是穷举证明「没有别的路径」；按 `evidence-discipline.md`「没打中一次不算」，这一否定结论要再抽一次样才算数。
- Z2、Z4 的打中是针对判决采纳的、今天还没实现的改法（MIR 比对、信任清单、剥 ghost 都零轮），residual 全是推的，没在入库装置上实测——采的改法实现出来之前核不动。
- Z5 的分类数（A/B/C/D）没复跑 `closure_classes.py`；只现核了闭包 fn 与行数、两 / 三闭包并集。
- Verus 用的是 0.2026.09.20（本机副本），第二台没有 Verus；门禁 `scripts/gate/verus-proof-check.py` 与 checker-tier-formal-verification 没起（照正文第五节）。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量的数都不是入库装置上的数。
- 没跑任何门禁阶段（`.claude/gate.d/` 下一律没起），没起 `scripts/gate/verus-proof-check.py`，没跑 checker 档测试，没起 GPU。
- 判决要引的数照 `.claude/rules/three-way-inference.md` 要在入库装置上重做：Z1 的 Verus verified / errors、Z3/Z5 的闭包 fn 与行数都是本机副本上量的。
- 要主 agent 跑的命令：无（复跑命令在「复跑」一节，前台约 90 秒跑得完）。

## 交回前复跑与清理

- 交回前在新 shell 里整份跑一次 `tools/rerun.sh`（命令同「复跑」一节，`--work` 给新目录）：包装退 0，末行原样「name=rerun-compare out/summary.txt same=yes」。退非 0 的件数为 0。
- 删了：两趟复跑工作目录 `rerun-ref/`、`rerun-verify/` 各 1.2M（复跑脚本在 `$WORK` 里建的 verus 拷贝与闭包 dump，已随删）；没建仓副本与编译目录（Verus 直接编单文件 crate，`lib_function_units.py` 只解析源码）。
- 草稿目录 `~/.local/share/singlefs/work/m4-exit5-relocation-proof-r3-opus/` 现约 1.6M：`verus/`（攻击 crate 与必红，与模型目录同一份）、`out/`（attack-build / mustred / closure 的原样输出）；正文只引模型目录 `out/summary.txt` 与复跑重出的行，草稿不入库。
