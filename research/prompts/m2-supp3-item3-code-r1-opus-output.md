# 增补 3 第 3 件（崩溃注入）代码轮第一轮：云端攻方腿（Opus）报告

攻击面只有 K1（抽样罩得住吗）与 K4（截断粒度）。K2/K3 归云端正推、K5/K6 归本地攻方，本报告不判那四格。
时区：下面写的时刻都是 UTC（本机时钟），人在 JST = UTC+9。

## 一、各格判定一览

| # | 格 | 判定 | 一句话 | 依据在第几节 |
|---|---|---|---|---|
| K1-a | 「加一道／减一道屏障」这类变异，抽样判不判得出 | **打中（硬）** | 屏障在前缀截断下不进任何一个崩溃状态 ⇒ 里程碑给这一件写的那条验收标准（`.claude/kb/milestone/02-second-txn.md:424`）判别力恒为 0，加种子、加崩溃点都救不回来 | 三之一 |
| K1-b | 「同一屏障段内后发的写先持久」这一族窗口 | **打中（整类，零概率）** | 前缀截断摆不出任何带洞的持久集；同段的镜像双写「只持久后一份」那一半一个都摆不出。快档 24 段里这样的镜像对有 2782 对 | 三之二 |
| K1-c | 抽样域相对层 0 枚举域有多大 | **打中（量到数）** | 同一批流上层 0 的枚举域 46 090 683 个状态，前缀截断摆得出 7 243 个（0.0157%），实际抽 92 个（0.0002%） | 三之三 |
| K1-d | 起点那一段的窗口 | **打中（整类）** | 结构性排除；快档 7 219 次写里 627 次（8.7%）永远不是候选，大档 200×40 里 5 533/101 820 | 三之四 |
| K1-e | 失败那一步的窗口 | **打中（结构性，今天量到 0）** | 失败即 `return`，观察者不被调用 ⇒ 那一步的写不进候选。快档与 200 段大档实测 0 段没跑完，所以这一格今天是空的 | 三之五 |
| K1-f | 冷启动那一类操作 | **打中（文档说错）** | `apply_cold_start_recover` 一个字节都不写 ⇒ `落在 ColdStartRecover 里` 恒为 0；`crash_injection.rs:7` 写着「冷启动都在流里」，用例文档注释也说「冷启动的写上都截得到」 | 三之六 |
| K1-g | `assert_every_crash_injection_path_was_exercised` 够不够说明没漏整类 | **不够（打中）** | 它是一串存在性断言（≥1 / 相等），证明的是「这条路径被走过至少一次」，不是「这一类窗口摆得出来」；实测七类操作里三类是 0，而它只查两类 | 三之七 |
| K1-h | 崩在恢复自己写出的那几次写中间（双重崩溃） | **两处都 0 覆盖（不分辨臂）** | 这里的 `recover(&image, …)` 是只读的，崩溃镜像上从不再可写挂载一次；层 0 同样没有 ⇒ 共用前提，不拿它判这一件 | 三之八 |
| K4-1 | 截断的单位 | **一样** | 两处都是「录制流里的一次写」，撕裂都并进「没持久」（D13 已定项 4） | 四之一 |
| K4-2 | 持久集合的形状 | **不一样（打中）** | 层 0 = 前若干段全持久 + 当前段任意子集；这一件 = 流的前缀。前缀集 ⊊ 层 0 集 | 四之二 |
| K4-3 | 屏障的地位 | **不一样（打中）** | 层 0 里屏障定枚举域；这一件里屏障被过滤掉、施加时也跳过，对结果零影响 | 四之三 |
| K4-4 | 每个崩溃状态上跑什么 | **不一样（打中）** | 层 0 跑 Consult + Ignore 两遍恢复、checker、**记录核对器**；这一件只跑 Consult 一遍恢复 + checker + 模型，不跑记录核对器 ⇒ `root_without_record` 与 `claimed_state_missing_unit` 这两类在这一件里一次都没判过 | 四之四 |
| K4-5 | 条款与代码的字面不合 | **打中（轻，方向是放宽）** | D13 已定项 4 写「带 FUA 的写也是段边界：它自成一段」，`writes_and_segments` 把 FUA 写放进它前面那些普通写的同一段 | 四之五 |
| 顺带 | 背景材料第二节的一个数 | **对不上** | 背景写「问模型 92 次（比过内容 58 次、树表 0 条对上 31 次）」，58 + 31 = 89 ≠ 92；副本上跑出来是 61 + 31 = 92 | 六 |
| 顺带 | 大档 200 段 | **不绿** | 未改任何代码，200 段 × 40 步 × 8 点上 3 个崩溃状态是「新发现」（都是 I-3.1，种子 80 那一形长得像收口表第 43 行）。这一格按分工归本地攻方的 K6，我只报观测 | 六 |

## 二、复跑命令与 sha256

副本在 `/tmp/claude-1000/m2s3i3-r1-opus/repo`（`rsync -a --exclude target --exclude .git`，开工时与工作区逐字相同）。
两份量具（我写的，只在副本里）：

| 文件 | sha256 |
|---|---|
| `/tmp/claude-1000/m2s3i3-r1-opus/model/zz_opus_k1k4_probe.rs` | `6b7f841b545fa8fac5813d52260f4cdba6129da42c8dd7441108d467e24504b2` |
| `/tmp/claude-1000/m2s3i3-r1-opus/model/zz_opus_k1_reorder.rs` | `afe0d1af7b4d47e34f0816ccaf4162c4c46c9150417d559402e3dfb316987068` |

复跑（副本里，两份量具放 `crates/singlefs-harness/tests/`）：

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ <副本>/
cp <model>/zz_opus_k1k4_probe.rs <model>/zz_opus_k1_reorder.rs <副本>/crates/singlefs-harness/tests/
cd <副本>
# K1-c / K1-d / K1-e 的数（快档那一批）
nice -n 19 cargo test --release -p singlefs-harness --test zz_opus_k1k4_probe -- --nocapture
# 同上，大档那一批
PROBE_SEEDS=200 PROBE_STEPS=40 PROBE_POINTS=8 nice -n 19 cargo test --release \
  -p singlefs-harness --test zz_opus_k1k4_probe -- --nocapture k1k4_probe
# K1-b 的构造与镜像对计数
nice -n 19 cargo test --release -p singlefs-harness --test zz_opus_k1_reorder -- --nocapture
# K1-a：基线
SINGLEFS_CRASH_INJECTION_THREADS=8 nice -n 19 cargo test --release -p singlefs-harness \
  --test second_transaction_supplement_three_crash_injection -- --nocapture --test-threads 1
# K1-a：施加 crates/mutations.tsv 第 67 行那条变异之后再跑一遍，逐字比
```

产物（都在 `/tmp/claude-1000/m2s3i3-r1-opus/`，**没入库**，理由见第七节）：

| 文件 | sha256 |
|---|---|
| `probe1.log` | `0b8cd3d946f4fb92e968a8dfe50159ed15f91a3801506e71d6c8d24d102d9c97` |
| `ci-baseline.log` | `f79c92f00be311b5f23e74a72b50a12c0f54e3803b3298ad0af77da7ddbf581e` |
| `ci-mutated.log` | `3b14d3a9fc6305b36fced29dc8e1b69d52424e09c62dd7e58aa2059d18a6b4b5` |
| `large80-baseline.log` | `5f9cc79eaee9c756b713949b12bd949c735da8d4d23e00bb178b04ccd9132683` |
| `large80-mutated.log` | `b51035aea3e376ed2fe199bef4591dfcf9e26882679c861e0bad2d8618d71fb2` |
| `large200-baseline.log` | `23d6800727881adb8eba696e39337597ab669558f48d5ba2e7b421153bbaf11f` |

跑之前 `ps -o pid,args -u "$(id -u)"` 看到的：只有 VS Code server、vLLM（`Qwen3-Next-80B-A3B-Thinking-AWQ-4bit`，本地腿那台）与它的 ray 进程在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-*`、`fio`；`uptime` 报 `load average: 0.00, 0.05, 1.32`。没等过 cargo 的文件锁。全部 `nice -n 19`。
变异跑完已还原：副本与工作区的 `crates/singlefs-core/src/transaction.rs` sha256 都是 `b5c320f23355e61d0fd5fb2be6e3ae4346737175959cedeb9b5d8ae5d8f03558`。

## 三、K1：抽样罩不住的窗口

### 三之一　K1-a：屏障在这一件里没有任何判别力 ⇒ 里程碑给它写的验收标准恒不成立

里程碑原文（`.claude/kb/milestone/02-second-txn.md:424`，整行抄）：

```
- 崩溃注入：判别力：施加 `crates/mutations.tsv` 里「步 3：零单元发布在记录与根之间少一道屏障」那一条，抽样在写死的种子数之内判红。
```

**机理**：崩溃点的候选把屏障滤掉（`crates/singlefs-harness/src/crash_injection.rs:600`）：

```rust
        .filter(|index| operations[*index].operation.kind != RecordedOperationKind::Barrier)
```

重建镜像时屏障也跳过（`crates/singlefs-harness/src/crash.rs:172`，`MemoryPool::apply`）：

```rust
            if retained.operation.kind == RecordedOperationKind::Barrier {
```

而崩溃状态就是流的一个前缀（`crates/singlefs-harness/src/crash_injection.rs:474`）：

```rust
        let newly_persisted = &operations[applied_operations..=crash_point.stream_index];
```

⇒ 录制流里加一道或减一道屏障，**候选写的集合、每个候选之后的镜像字节、抽到哪几个，全都不变**；变的只有 `stream_index` 这个标号。

**量过的**（副本，2026-09-20 08:4x UTC）：

1. 镜像层面：`zz_opus_k1k4_probe.rs` 里的 `barriers_do_not_change_any_prefix_state`，把种子 0 那段历史的流里全部屏障删掉，逐个前缀重建镜像与带屏障的逐字节比——
   原样输出 `BARRIER_PROBE prefix_states_compared=440 stream_ops=503 writes=440`，440 个前缀状态全部相同。
2. 用例层面：施加 `crates/mutations.tsv` 第 67 行那条变异（`步 3：零单元发布在记录与根之间少一道屏障`），
   `cargo test --release -p singlefs-harness --test second_transaction_supplement_three_crash_injection` **5 条用例全绿**（`test result: ok. 5 passed; 0 failed; 1 ignored`），
   与基线的输出逐字比只差两行里的两个流下标：

```
51c51
< 崩溃状态上的已知红第 0 条（…）：2 个崩溃点；前几个 (种子, 流下标) [(11, 519), (16, 493)]
---
> 崩溃状态上的已知红第 0 条（…）：2 个崩溃点；前几个 (种子, 流下标) [(11, 517), (16, 491)]
```

3. 加规模也一样：大档 80 段 × 40 步 × 8 个崩溃点，基线与变异的报告逐字相同，同样只差两个流下标（`崩溃状态上的已知红第 0 条 … 117 个崩溃点` 两边都是 117，新发现两边都是 0）。

**判决**：这条验收标准不是「还没跑到足够的种子」，是**结构上取不到真**。它要判的东西（段边界摆在哪）在这一件的状态集合里根本不出现。
改法见第五节；在改法落地之前，里程碑那一行应当改成「这条判别力归门禁 54 号」或换一条真能红的变异。

**打中之后的四句**（`evidence-discipline.md`「判据自己也会写错」）：

1. **分不分辨臂**：分辨。它只打中「前缀截断」这一臂；层 0 的「段内任意子集」臂在同一条变异下会多出「根在、记录不在」的状态，`second_transaction_step_zero_layer0.rs:356` 钉着 `record_root_without_record == 0`，那一臂判得出。
2. **被判的系统当时看不看得到判别它的东西**：看不到。崩溃注入这一路在做决定的那一刻手里只有「流的前 k 次写」，而屏障不改变任何一个 k 对应的字节 ⇒ 两种实现（有屏障 / 没屏障）在它眼里逐条相同。这是「判别子观测不到」那一形：要它判出来，得先改状态集合，不是改断言。
3. **满足的是判据字面的哪一个分句**：里程碑那一行「抽样在写死的种子数之内判红」的「判红」。不是「抽样规模不够」（那一句在这里没有内容——任何规模都不红）。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：这一件的跑前条款（里程碑「预想的细节」）给的处置是「发现的问题照主 agent 判阻塞办」，没给具体改法 ⇒ 没有「改法碰不到打中的格」这一形可核。我自己提的两个改法在第五节，各标了修哪一格。

### 三之二　K1-b：同一屏障段内「后发的写先持久」这一族，一个都摆不出

**这一类在真机上会发生**：一个屏障段里的写之间没有任何顺序承诺，设备可以按任意次序落盘。
最要紧的一支是镜像双写。D13 已定项 4 的定案整行（`.claude/kb/decisions/13-验证路线.md:71`）：

```
**定案**：崩溃点重放要枚举的崩溃状态集合定义为：屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。撕裂子集**不枚举**，一次写的任何撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像要真的生成出来喂给 checker，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。
```

而 D23 已定项 14 对这两份镜像的判定（`.claude/kb/decisions/23-journal的角色与格式.md:354`，整行抄）：

```
两份镜像何时算「记录在」：**任一份自证校验和过即在**（崩溃点重放里的检查由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件）。
```

⇒「只有第二块盘那一份在」与「只有第一块盘那一份在」都得读得回来。前缀截断只摆得出前者的反面：截在两份之间，得到的永远是**先写的那一份在**。
一个只认第一块盘那一份的实现，在这一件里任何种子、任何规模都判不出来。

**可跑的构造**（`/tmp/claude-1000/m2s3i3-r1-opus/model/zz_opus_k1_reorder.rs`，原样输出）：

```
MIRROR_PAIR segment=6 segment_len=2 first_copy=write#13(dev DeviceIdentity(0)) second_copy=write#14(dev DeviceIdentity(1))
NOT_A_PREFIX last_persisted=write#14 holes=1 first_hole=write#13
RECOVER_ON_REORDERED_STATE outcome="NoFile" effective_root=Some((InstanceGeneration(0), CheckpointTxg(0))) prefix_applied=0
DOMAINS writes=129 segments=31 layer0_states=789588 prefix_states=130
```

读法：那段写死的历史（用例 `every_crash_point_of_a_written_out_history_recovers_into_a_committed_version` 用的那一段，
mkfs → 可写挂载 → 第一个文件 → 覆盖写 → 可写挂载 → 覆盖写）的第 6 段恰好只有两个写，就是同一条 journal 记录写两块盘。
「段内只持久第 14 次写」是层 0 枚举域里的一个合法状态（前 6 段全持久、第 7 段的子集掩码 = 0b10），
它在第 13 次写上有一个洞 ⇒ 不是任何前缀 ⇒ 这一件永远摆不出。

**规模**：快档那 24 段历史里，同段的镜像对一共 **2782 对**（journal 记录 429 对、单元写 2353 对），原样输出：

```
FAST_TIER_MIRROR_PAIRS total=2782 by_kind={"journal_record": 429, "unit_write": 2353}
```

每一对至少带来一个这一件摆不出的崩溃状态。

**四句**：① 分辨臂（层 0 那一臂摆得出，见上）；② 被判系统看不到（前缀截断的状态集合里没有带洞的持久集，它连这个状态都构造不出来，更谈不上判）；
③ 满足的是「哪一类崩溃窗口因此永远抽不到」这一问的字面；④ 跑前条款没给改法，我的改法在第五节。

### 三之三　K1-c：抽样域相对层 0 枚举域有多大（量到的数）

同一批录制流（快档那 24 段，种子 [0, 24)、每段 24 步、`GenerationWeights::BROAD`、不跑每步 checker、两块 4 GiB 的盘），
拿层 0 自己的 `writes_and_segments` 切段、`closed_form_state_count` 数状态，与「前缀状态数 = 写数 + 1」并排。原样输出末行：

```
TOTALS writes=7219 candidates=6592 excluded_by_start=627 excluded_by_failed_step=0 layer0_states=46090683 prefix_states=7243
```

- 层 0 的枚举域（D13 已定项 4 的定义，同一批流）：**46 090 683** 个崩溃状态。
- 前缀截断摆得出的：**7 243** 个（0.0157%）。前缀状态集是层 0 状态集的**真子集**——前缀截断落在第 i 段里，就是「前 i−1 段全持久 + 第 i 段的一个前缀子集 + 之后全不持久」，正是层 0 那条定义的一个合法取值。
- 实际抽到的：**92** 个（报告原样：`崩溃点 92 个：截在发布中间（根还没落盘）的 77 个、截回到最后一版之前的 85 个`），占层 0 枚举域的 0.0002%。
- 单段最宽的一段有 18 个写（`max_seg` 那一列），它一段就有 2^18 − 1 = 262 143 个子集，前缀截断在那一段里只摆得出 18 个。

大档那一档（200 段 × 40 步 × 8 点）同样量了一遍：

```
EXTRA zero_candidate_histories=0 not_completed=0 just_before_root_candidates=5829 points_per_history=8 expected_just_before_root_draws=0.48
TOTALS writes=101820 candidates=96287 excluded_by_start=5533 excluded_by_failed_step=0 layer0_states=594980354 prefix_states=102020
```

**这个数说明的与不说明的**：它说明「抽样 + 前缀」这一路与层 0 那一路的覆盖差了四个数量级，
不说明「差掉的那些状态里一定有 bug」——差掉的整类是不是重要，看三之二那一条（镜像不对称）与四之四那一条（记录核对器）。

### 三之四　K1-d：起点那一段的写，一个都不是候选

候选区间的下端钉在「起点跑完时流里有几步」（`crates/singlefs-harness/src/crash_injection.rs:598-599`）：

```rust
    let mut candidates: Vec<usize> = (marks.after_the_starting_point
        ..marks.after_the_last_finished_step.min(operations.len()))
```

⇒ mkfs 那一段、以及起点取 `AfterFirstFile` 时的取号 / 暖机 / 第一次发布，那几十次写上的崩溃窗口一个都抽不到。
量到的：快档 7 219 次写里 **627 次**（8.7%）在起点里；大档 200×40 里 **5 533 / 101 820**（5.4%）。
探针里逐段那一列显示起点占 13 步（`AfterMakeFilesystem`）或 52 步（`AfterFirstFile`）。

**它算不算被别处罩住**：第一个事务那条固定流归门禁 54 号，起点的一部分在那里被全量枚举过（`.claude/gate.d/54-layer0-replay.sh:2` 的 `gate-stage` 行点名两条流）。
但**大档可以在小盘上跑**（`SINGLEFS_CRASH_INJECTION_DEVICES=small` ⇒ `HistoryDeviceWidth::UnitAreaOf384Slots`，
`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:314-317`），而层 0 那两条固定流没有小盘的档
⇒ **小盘上的起点窗口两处都没有**。这一格是整类零覆盖，不是抽样稀疏。

### 三之五　K1-e：失败那一步的写，结构上抽不到

候选区间的上端是「最后一个跑完的操作之后流里有几步」，而这个标记是在观察者回调里更新的
（`crates/singlefs-harness/src/crash_injection.rs:427`：`marks.after_the_last_finished_step = operations_so_far;`）。
执行器在一步判失败时**先 `return`、不调观察者**（`crates/singlefs-harness/src/history.rs:2750-2762`：
`if !continues_past_the_ring_turn_form { return Some(observation); }` 在 `observer(&StepObservation { … })` 之前）；
panic 那一路更是直接展开出去。⇒ 一段历史以 panic、执行器判定或模型对不上收尾时，**那一步发出的写一个都不进候选**。

模块文档把这条写成了一个理由（`crates/singlefs-harness/src/crash_injection.rs:10`）：

```
//! 起点（mkfs 与起点那次发布）整个持久，失败那一步的写不抽（模型还没判过它写出的根，目录里没有那一版）。
```

**这个理由只挡得住那一步里最后那次根槽写**：判读那一步要的是「截断之后盘上已持久的最新根在不在模型目录里」
（`crash_injection.rs:511` 的 `crash_recovery_disagreement(&committed_versions, newest_persisted_root, &read_back)`）。
截在失败那一步的**根槽写之前**，最新持久的根还是上一版、目录里有它 ⇒ 判得动。
按今天的写法，这些判得动的窗口被一起扔掉了，而它们恰好是「这一步正要出问题」的那几次写。

**量到的**：快档与 200 段大档都是 `excluded_by_failed_step=0`、`not_completed=0`——
因为崩溃注入这一路把每步的池级 checker 关掉了（用例 `UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES`，
`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:28-31`），
剩下能让一步失败的只有执行器判定、模型对不上与 panic，这两档里一次都没出现。
**⇒ 这一格今天是空的**：结构性排除成立，但眼下还没有被它吃掉的窗口。它在两个时刻会变成实打实的漏：
① 崩溃注入哪天打开每步 checker；② 随机历史撞出新的 panic / 模型对不上。

### 三之六　K1-f：冷启动那一类操作恒零，而两处文档说它有

`apply_cold_start_recover` 只读不写（`crates/singlefs-harness/src/history.rs:2464`）：

```rust
    let report = recover(&pool.devices, JournalPolicy::Consult);
```

⇒ 这一类操作在录制流里一个写都没有，`crash_points_by_operation_kind[ColdStartRecover]` **恒为 0**。两处文档与它不符：

- `crates/singlefs-harness/src/crash_injection.rs:7`：`//! 换来的是任意一段随机历史上的崩溃点：关闭重开、可写挂载、回退、抬 F、零单元发布、冷启动都在流里。`
- `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:246`：`/// 固定脚本那条流只有一次回退与一次抬 F，这一段证明随机历史上的回退、抬 F、零单元发布、冷启动的写上都截得到。`

两档跑出来的原样计数都印着 `落在 ColdStartRecover 里：0 个`（快档、偏向回退那一段各一次）。
**这不是 bug，是一句写错的话**——但它正好是 K1 问的「几条断言够不够说明没漏整类」的反例：文档声称覆盖了一类，
而那一类在这套装置里不可能有崩溃点，断言里也没有任何一条会发现这件事。

### 三之七　K1-g：`assert_every_crash_injection_path_was_exercised` 够不够说明「没把整类窗口漏掉」

不够。逐条看它证的是什么（`crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:76-161`）：

| 断言（第几行） | 它证的 | 它不证的 | 实测值 |
|---|---|---|---|
| `crash_points >= FAST_TIER_SEEDS`（80-84） | 92 ≥ 24 | 每段至少一个——它允许 23 段全 0、1 段 24 个 | 92 个崩溃点，其中 **1 段一个都抽不到**（报告原样：`一个崩溃点都抽不到的 1 段`；探针指到种子 22，`candidates=0`） |
| 四种写落点各 ≥ 1（93-110） | 四个标签各被贴到过一次 | 同一标签下不同形状的窗口（例如镜像对的两半） | `journal_record：20`、`root_record_fua：6`、`superblock_slot：20`、`unit_write：46` |
| 两类操作各 ≥ 1（111-125） | `PublishOverwrite` 与 `CloseAndMountWritable` 各截过 | 另外五类 | 七类里 **三类是 0**：`PublishFirstFile 0`、`RaiseRollbackFloor 0`、`ColdStartRecover 0`（后者恒 0，见三之六） |
| `crash_points_inside_an_unfinished_publish >= 1`（126-129） | 截在根落盘之前过 | 「记录在、根不在」与「单元在、记录不在」分别有多少 | 77 / 92 |
| `crash_points_that_land_before_the_last_committed_version >= 1`（130-133） | 截回到过去过 | 截回了多远 | 85 / 92 |
| `recoveries_failed == 0`、读回 / 没文件各 ≥ 1（134-142） | 恢复三种结局见过两种 | —— | 61 / 31 / 0 |
| `recoveries_that_applied_journal_records >= 1`（143-146） | journal 真承重过 | 14 / 92，前缀施加的长度分布 | 14 |
| `model_contents_compared >= 1`、`model_versions_without_a_file_matched >= 1`（147-154） | 模型两条路都走过 | —— | 61 / 31 |
| `invariant_holds["I-3.1"]`、`["I-5.4"] >= 1`（155-160） | 两条不变量判过绿 | 判红过没有 | 各 92 |

**共同的毛病**：全是「这条路径被走过至少一次」。要说明「没把整类窗口漏掉」，要的是对**状态集合**的陈述
（这一类窗口在集合里 / 不在集合里），而这套断言一条都不是这种形状。最直接的反证在三之一：把一道屏障删掉，
上表每一格逐字不变——一套对「段结构」零判别力的断言，当然说明不了「段内的窗口没漏」。

### 三之八　K1-h：崩在恢复自己写出的那几次写中间（不分辨臂，两处都没有）

崩溃点上的恢复是只读的（`crash_injection.rs:499` 的 `recover(&image, JournalPolicy::Consult)`，`&image` 不可变），
而真机上一次挂载恢复会写盘。D13 已定项 7 的那一行（`.claude/kb/decisions/13-验证路线.md:130`，整行抄）：

```
**需要第二个输入的核对归记录核对器**，入参 `(崩溃前镜像, 记录流, 崩溃后镜像)`；它与自研第 1 项「Merkle 根链差分」（已定项 8）同属二元检查。⚠️ **故意不给它编号**——O2（独立解析器 + checker） 那场混乱就是编号当称呼造成的，再加一个 O4 只会给下一次漂移多一个落脚点。⚠️ **「崩溃后镜像」是两份，不是一份**：harness 的顺序是先跑实现自己的恢复、再跑记录核对器，而恢复本身会改盘（实例切换写行、重发在飞 checkpoint，D23（journal 的角色与格式） 已定项 14）⇒ 择根与前缀判定看**崩溃态镜像**（「记录流」就是它环里的 journal 记录），比对的对象是**实现恢复后的镜像**。
```

⇒ 「崩溃 → 恢复写了一半 → 再崩」这一族状态，这一件 0 覆盖。

**但它不分辨臂**：层 0（门禁 54 号）同样只枚举一次崩溃，同样不在崩溃状态上再可写挂载一次。
按 `evidence-discipline.md`「打中不分辨臂」那一格的处置，我不拿它判这一件——它是两条路共用的前提，
该另立一笔账（今天在 `.claude/kb/checks-owed.md` 里有没有对应的 C 编号我没查，留给主 agent）。

## 四、K4：这套截断模型与层 0 的差别

### 四之一　单位：一样

两处都是「录制流里的一次写」（`RetainedOperation` / `RetainedWrite`），撕裂都不枚举、一律并进「这次写没持久」——
D13 已定项 4 定案那一行（前面三之二已整行抄）里的「撕裂子集**不枚举**」。这一格没有差别。
「没持久的位置放崩溃前的旧字节」两处也等价：层 0 靠 `CrashImage` 在基线镜像上叠持久的写（`crash.rs:245-282`），
崩溃注入靠从 mkfs 起按次序往 `MemoryPool` 上叠（`crash_injection.rs:474-475`），起点是空盘、`SparseDevice` 的洞读 0，
叠到崩溃点为止得到的就是那一刻的旧字节。

### 四之二　持久集合的形状：层 0 是「段内任意子集」，这一件是「流的前缀」

层 0（`crates/singlefs-harness/src/crash.rs:1035-1052`，`Layer0StatePlan::persisted_writes_of_state`）：所在段之前每段整段持久，
所在段按子集掩码 `subset_mask & (1 << bit)` 逐位取。崩溃注入（`crash_injection.rs:474`）：`operations[..=stream_index]` 全部施加。
⇒ **前缀集 ⊊ 层 0 集**，差多少见三之三的数（46 090 683 对 7 243），差掉的整类见三之二。

### 四之三　屏障的地位：层 0 里它定枚举域，这一件里它什么都不是

层 0：`writes_and_segments`（`crash.rs:306-343`）遇到 `Barrier` 就关段，段宽直接决定 `2^|段| − 1` 个状态。
崩溃注入：屏障被滤出候选（`crash_injection.rs:600`）、施加时跳过（`crash.rs:172`）。
实测后果在三之一——`crates/mutations.tsv` 第 67 行那条变异下五条用例全绿、报告逐字相同。

### 四之四　每个崩溃状态上跑什么：这一件少了记录核对器与 Ignore 那一遍

层 0 的 `evaluate_state_for_versions` 在每个状态上做四件事：

- `crash.rs:793-794`：`recover(Consult)` **与** `recover(Ignore)` 各一遍，两者结局不同的状态单独计数；
- `crash.rs:813-844`：两个结局各过一次 `oracle_violation_for_versions`；
- `crash.rs:845-865`：`check_pool_image`；
- `crash.rs:866-872`：`check_records`——D13 已定项 7 那个故意不给编号的**记录核对器**，入参 (崩溃前镜像, 记录流, 崩溃后镜像)。

门禁 54 号自己也是这么报的（`.claude/gate.d/54-layer0-replay.sh:87`，整行抄）：

```
echo "  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；${worker_threads} 个工作线程，SINGLEFS_LAYER0_THREADS=${SINGLEFS_LAYER0_THREADS}（${threads_origin}），本机 ${machine_cores} 核）：${line#LAYER0 }"
```

崩溃注入在每个崩溃点上只做三件：`recover(Consult)` 一遍（`crash_injection.rs:499`）、`check_pool_image`（`crash_injection.rs:521` 经 `checker_violations_on`）、
问模型（`crash_injection.rs:511`）。**没有记录核对器，也没有 Ignore 那一遍。**

**差别让哪一类问题在这一件里看不见**：`RecordCheck` 那两个字段（`crash.rs:438-443`，整段抄）：

```rust
pub struct RecordCheck {
    /// 某次发布的根槽写已在盘上，而那次发布的 journal 记录一份都不在（E77（发布的持久顺序） b_ur 臂的「记录流有洞」）。
    pub root_without_record: bool,
    /// 恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元两份都不在（E77（发布的持久顺序） 的独立审计）。
    pub claimed_state_missing_unit: bool,
}
```

⇒ ①「根在、这次发布的记录一份都不在」与 ②「实际走的根声称已发布的某个单元两份都不在」（E77 点名的静默嫁接），
在这一件的每个崩溃点上**一次都没被判过**。层 0 那边钉着它们等于 0（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:353-356`）。
这两类在前缀截断下并非摆不出来——发布顺序一旦写反（先根后记录、先根后单元），前缀截断照样会走到「根在、记录不在」的状态；
那时恢复会择到那条根、内容多半也对得上、checker 判绿、模型认得那一版 ⇒ **整条路绿灯**，而层 0 那边会红。

### 四之五　条款与代码在 FUA 那一句上字面不合（轻，方向是放宽）

D13 已定项 4 定案里：「带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。」
代码把 FUA 写放在它前面那些普通写的**同一段**里（`crash.rs:333-335`：先 `current.push(writes.len() - 1)` 再关段），
`segments.rs:73` 把这个读法写在注释里：

```
/// FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）；
```

两个读法枚举出的状态集不同：按条款字面，「FUA 已持久而同段里更早的普通写没持久」摆不出来；按代码，摆得出。
代码这一侧更宽（枚举得更多），所以不是漏，但**两处至少有一处该改**——今天引 D13 已定项 4 去说「层 0 的枚举域就是这样」时，
引的那句话与跑的那份代码不是同一件事。这一格与 K4 的主问无关紧要，附在这里供主 agent 一并处置。

## 五、我自己提的改法（**只在我的副本上量过、被攻过零轮**）

四条，各修哪一格写在表里。「量过」= 我在副本上跑出来了、贴了原样输出；「推的」= 按代码推的，没实现没跑。

| 改法 | 修的是哪一格 | K1-a 屏障 | K1-b 段内乱序 | K1-c 覆盖比 | K1-d 起点 | K1-e 失败那一步 | K1-f 冷启动 | K4-4 记录核对器 |
|---|---|---|---|---|---|---|---|---|
| A：每段历史除了「抽 k 个前缀」，再按同一个种子抽 m 个**段内子集**状态（复用 `Layer0StatePlan` 的子集掩码，段落用 `writes_and_segments` 切） | 状态集合的形状 | **修好**（推的：屏障一变段就变，子集状态跟着变 ⇒ 第 67 行那条变异有机会红） | **修好**（推的） | 改善（推的） | 不动 | 不动 | 不动 | 不动 |
| B：在每个崩溃点上补跑 `check_records(&image, report.effective_root)` | 每个状态上跑什么 | 不动（量过：第 67 行那条变异不改任何状态，补 B 也不会红——除非同时上 A） | 不动 | 不动 | 不动 | 不动 | 不动 | **修好**（推的） |
| C：候选区间的上端从「最后一个跑完的操作」改成「最后一次写」，判读时凡「截断之后最新持久的根在模型目录里」就判，不在就只跑 checker 不问模型 | 候选区间 | 不动 | 不动 | 微改善 | 不动 | **修好**（推的） | 不动 | 不动 |
| D：候选区间的下端放到 0，起点那一段里最新持久的根用 `HistoryPool::start` 之后模型答的那一版 | 候选区间 | 不动 | 不动 | 改善（推的：快档多出 627 个候选） | **修好**（推的） | 不动 | 不动 | 不动 |

另外两条不是改法、是改文字：
- `crates/singlefs-harness/src/crash_injection.rs:7` 与 `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:246` 里的「冷启动」删掉（量过：两档都是 0，且结构上恒 0）。
- 里程碑 `.claude/kb/milestone/02-second-txn.md:424` 那条验收标准：在 A 落地之前换一条真能红的变异，或写明这条判别力归门禁 54 号（量过：今天恒不红）。

**这四条都被攻过零轮**，而且 A 的代价我没量：它把每段的崩溃状态数从 k 提到 k + m，挂钟按状态数线性涨，
快档今天 92 个崩溃点、`check.sh` 里几秒钟，m 取多少才不撑爆快档没人算过。要采纳先登记一次计数模型实验。

## 六、顺带核到的两件（不在我的攻击面里，只报观测）

1. **背景材料第二节那一行数字对不上**。背景写「问模型 92 次（比过内容 58 次、树表 0 条对上 31 次）」，58 + 31 = 89 ≠ 92。
   副本上跑出来是（原样）：`崩溃点上 checker 跑了 92 次；问模型 92 次：比过内容 61 次、树表 0 条对上 31 次`，61 + 31 = 92。
   代码上这两个数只在 `disagreement.is_none()` 时按 `read_back` 的成员各加一次（`crash_injection.rs:512-519`），
   而 `recoveries_reading_a_file = 61`、`recoveries_without_a_file = 31`、零违例 ⇒ 只能是 61 / 31。背景里那个 58 该核。
2. **大档 200 段不绿**（未改任何代码）。`SINGLEFS_CRASH_INJECTION_SEEDS=200 OPERATIONS=40 POINTS=8`：
   `崩溃状态：以已知红收尾 {0: 261}、新发现 3`，用例判红。唯一印出来的签名（原样）：

```
崩溃点上的新发现 CheckerViolations { invariants: ["I-3.1"] }：种子 80，截在录制流第 386 步（unit_write，Operation(21)／Some(RaiseRollbackFloor)）
  I-3.1：盘 0：记账的已分配 Some(1310720)，遍历全部有效根得到 1212416
```

差 1310720 − 1212416 = 98304 = 6 × 16384，与收口表第 43 行登记的「差 6 × 16384」、种子 80 逐项对得上；
而报告里 `崩溃状态上的已知红第 1 条（增补 2 收口表第 43 行）：0 个崩溃点`——因为崩溃状态上第 1 条的判别字段被写死成 None
（`crash_injection.rs:676`：`raised_floor_lands_only_on_abandoned_roots: None,`，理由写在 `crash_injection.rs:657-659` 的文档注释里）。
**这一格按分工表归本地攻方的 K6，我不判，只把观测与复现命令交出来**：

```
SINGLEFS_CRASH_INJECTION_FIRST_SEED=80 SINGLEFS_CRASH_INJECTION_SEEDS=1 SINGLEFS_CRASH_INJECTION_OPERATIONS=40 \
SINGLEFS_CRASH_INJECTION_POINTS=8 cargo test --release -p singlefs-harness \
  --test second_transaction_supplement_three_crash_injection -- --ignored --nocapture
```

## 七、没打中的形状

逐条写试过什么、取样多大，免得「没打中」与「这一轮没想到」混在一起（`three-way-inference.md`「一条腿只抽一次样不算一次观测」）。

| 试的形状 | 怎么试的 | 结果 |
|---|---|---|
| 「同一段历史的 4 个崩溃点共用一份往前叠的镜像」会不会串味 | 读 `crash_injection.rs:468-481`：`applied_operations` 单调、崩溃点已排序、每次只叠 `(上一个, 这一个]`，等价于各自从头重建 | 没打中：与逐点从头重建同一个镜像 |
| 「屏障被当成候选会不会多出状态」 | 注释说「截在屏障上与截在它前一个写之后同一个状态」；`MemoryPool::apply` 跳过屏障 ⇒ 成立 | 没打中 |
| `newest_persisted_root` 用 `max` 攒会不会因根环轮转取错 | 读 `crash_injection.rs:476-480`：按 `ModelRootKey` 取 max，根环覆写旧槽不影响「最新」 | 没打中（这一格与 K2/K3 相邻，判定归云端正推） |
| 「没持久的位置放零而不是旧字节」（D13 已定项 4 点名的一条） | 崩溃注入从空盘按次序叠到崩溃点，洞里是同一位置更早写过的字节或 0（那位置本来就没写过） | 没打中：与层 0 的「旧字节」等价 |
| 多设备各自独立丢写（盘 0 丢第 3 段、盘 1 留到第 5 段） | 层 0 的定义也要求「前若干段全部持久」跨设备一起算 ⇒ 两处都摆不出 | 不分辨臂，不记成打中（同三之八的处置） |
| 撕裂（一次写只落一半） | D13 已定项 4 明写知情接受，两处一样；`MemoryPool::flip_byte` 在这一件里没被调用（`grep -n flip_byte crates/singlefs-harness/src/crash_injection.rs` 零命中） | 不分辨臂 |
| 「每段固定 k 个」会不会让某个窄窗口概率低到等于没有 | 量了「候选里下一次写就是根槽 FUA 写」那一族：大档每段期望抽到 0.48 个，200 段期望约 97 个 | 没打中：这一族不稀疏。真正为零的是三之一到三之六那六类 |
| 快档 24 段里有没有哪一段历史步数多到把 4 个点摊薄成无效 | 探针逐段列了候选数：最多 527、最少 0，中位数几百 | 没打中（这是稀疏，不是零）；`92 / 6592 = 1.40%` 的候选被抽到 |

## 八、这条腿自己的限度

- **副本上的数不算入库的数**。所有数出自 `/tmp/claude-1000/m2s3i3-r1-opus/repo`（`rsync` 拷贝，`--exclude target --exclude .git`），
  要引进 kb 得由主 agent 在原装置上重做一次（`three-way-inference.md`「判决由主 agent 做，不由投票做」）。
  我没往 `research/results/` 落任何产物：这一轮的判决还没做、主 agent 还没定留不留，产物清单与 sha256 在第二节，路径都在 `/tmp` 下，拷走的窗口是本机重启之前。
- **两份量具是我写的测试，不是装置**。它们只读 `singlefs-harness` 的公开接口，没改被判的代码；`crates/singlefs-core/src/transaction.rs` 改坏之后已还原，sha256 与工作区一致。
- **我没跑门禁**：`.claude/gate.d/stage-owners.tsv` 里没有登记给这条腿的阶段（`awk` 按 agent 名过滤零行）；
  也没跑 54 号、59 号、`check.sh` 全量——定义没让跑，而且它们要动主工作区。
- **K2、K3、K5、K6 我一格都没判**，包括六之 2 那条明显的已知红遮蔽——按分工表那是本地攻方的 K6。
- **三之五那一格今天量出来是 0**。我把它记成「结构性排除、眼下为空」，没把它算成一次实打实的漏；
  它要变成漏，得先有一段历史以失败收尾。
- **我提的四条改法一条都没实现**，表里标「推的」的格子没有任何实测支撑。
- **「没打中」的那几行只抽了一轮**（`three-way-inference.md` 要求两次）。第七节那张表里的每一行都只跑过一次，
  按那条规则它们还不够记成「没打中」，只够记成「这一轮没看出来」。
