# 增补 3 第 4 件（故障注入）代码轮第一轮：云端攻方腿（Opus）报告

攻击面 **K1（注入点抽得够不够）、K2（注入之后历史不往下跑罩不到什么）**。前几轮判决：这一件是第一轮，无。
同一个增补第 3 件（崩溃注入）的两轮判决读过（`research/prompts/m2-supp3-item3-code-r1-main-verification.md`、`-r2-main-verification.md`），本报告刻意避开它们已经打过的角度，避开哪几个写在第六节。

## 零、复跑与产物

模型目录 `research/prompts/m2-supp3-item4-code-r1-opus-model/`。复跑：

```
bash research/prompts/m2-supp3-item4-code-r1-opus-model/reproduce.sh \
     /home/fy5090/code/singlefs /tmp/claude-1000/m2s3i4-r1-opus/repo
```

| 文件 | sha256 |
|---|---|
| `baseline-fast.txt` | `99384f67702cf26548057059244bd9edb96903ee3054829fc11402b9b1d431d6` |
| `control-random-history-with-probe.txt` | `315cce1108976b0240dde4c7452652d7b48652588866f028893fda05db9505f5` |
| `opus_probe_one_device_system_configuration.rs` | `b8b753deff8eee4dfb19cc3db009ac04dd26947c8054ae7af4250c6b8a7698c8` |
| `probe-k1-dead-device.txt` | `abba56f0879d7712f0c460a2261ceb74640b8776a8c5208f8b63b20df0ba13c4` |
| `probe-k1-schedule.txt` | `7f83dc83a54ba7c942dc92e5c7659fd5f868aaa94527a485f90638285b0f5b6d` |
| `probe-k2-continue.txt` | `8ccd3f807fbb42d10c6fdf31df03b7578885a2d1208d5963b4b05d278c44f80f` |
| `probe-k2-single-seeds.txt` | `343f25a368340c94aae10ec925be64f0204aaeac07f35c0acb26842ee14f064c` |
| `probe-k2-suspend-model.txt` | `60c3599f5edecccc14ec7aa8bf6259a2014ab31f07cdcb780ed1f55d6442e705` |
| `probes.patch` | `14b0b4ecda4a0c2a1ce34920d35e84969fa775fba08a26126283a7653ee2d482` |
| `reproduce.sh` | `a7f4e63b58d97a3f79e2b4ed07aef85c16ea78ec38edc12ca88bfe637eaf887d` |

⚠️ **下面每一个数都是在副本 `/tmp/claude-1000/m2s3i4-r1-opus/repo` 上量的**（`rsync -a --exclude target --exclude .git`），
按 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」，**副本上的数不进 kb**：主 agent 要在入库装置上重做一次才能引。
工作区一个字都没改；三处探针（`probes.patch`）全部由环境变量开，默认不改行为，副本上不开开关跑出来的快档与工作区逐字相同（`baseline-fast.txt`，新发现 0、没命中 22 个，与背景材料第二节的数对得上）。

## 一、各格判定一览

| 格 | 问什么 | 判定 | 一句话依据 |
|---|---|---|---|
| K1-a | 排期那四维（哪块盘 / 落点 / 整池还是逐盘 / 第几次）进不进抽样 | **打中（结构性，任何规模下概率 0）** | 随机注入这一路只造一种排期：`fault_injection.rs:1278` 恒调 `the_nth_call_across_the_pool`（每块盘、任意落点、整池数、只在第 n 次上注入一次）。四维里三维只有一个取值，`OnlyDevice` / `OffsetBelow` / `OffsetExactly` / `PerDevice` / `EveryMatchingCallFromTheNthOnward` 一次都没被抽过 |
| K1-b | 「同一段历史里两次故障」抽不抽得到 | **打中（结构性）** | 一个 armed 计划只带一个 `FaultSchedule`，`TheNthMatchingCall(n)` 只在第 n 次命中上返回一次（`fault_injection.rs:203`）⇒ 一段历史至多一次调用被动过手脚 |
| K1-c | 只在故障之后才发生的那些调用 | **打中（结构性）** | 候选集从**不注入**的测量跑算出来（`fault_injection.rs:1126` 拿 `marks` 的逐步调用数差），回落读、重读、镜像另一份那些「因为前一次失败才发出」的调用根本不在候选集里；要注入到它们身上得有第二次故障，而 K1-b 说了没有 |
| K1-d | 这三条合起来漏掉了什么真东西 | **打中（可跑构造，指到具体一行）** | 盘 1 的两个系统配置槽都读不出来时 `choose_system_configuration` 当场 `return Err`（`crates/singlefs-core/src/recovery.rs:244`），不看已经从盘 0 读到的那一份 ⇒ **一块盘的前两个固定槽坏掉，整个两块盘的池挂不上、恢复不了**，而 `.claude/kb/checks-owed.md` 的 C79（系统配置与 journal 环的放置没人定） 那一行写着「每盘一份 + journal 镜像 8 格全可挂」。这要两次读失败落在同一块盘上，今天的抽法永远摆不出来 |
| K1-e | 「42 格里命中 20 格」这个分母对不对 | **打中（轻）** | 那张表只有（操作 × 调用种类 × 写落点）三维，K1-a / K1-b / K1-c 三类盲区**一维都不在表里** ⇒ 「命中 20 / 42」量的是一个不含盲区的空间，读起来比实情乐观 |
| K2-a | 注入之后历史为什么不往下跑 | **坐实（不是「设计选择」，是模型对拍的连带）** | `model_comparison.rs:162` 把块设备错映射成 `Unexplained`；`model.rs:1402` 让 `Unexplained` 永远不被接受 ⇒ 模型必报「该成却拒了」⇒ `history.rs:2953` 当场 `return`。这一步之后的每一步都不跑 |
| K2-b | 让它往下跑，会不会现形出今天看不见的东西 | **打中（硬，24 段同样的 95 个注入点：新发现 0 → 38）** | `probe-k2-continue.txt`：只改「不停」，别的一概不动 ⇒ 新发现 0 → 38、重开走到模型都不允许的版本 0 → 14 |
| K2-c | 那 38 条里有没有不依赖模型的 | **打中（有两条，池级 checker 自己判的）** | `probe-k2-suspend-model.txt`：从注入那一步起把模型对拍整个挂起，只留 checker 与 panic ⇒ 仍有 22 条新发现，其中 **I-3.1 记账多算**（种子 …119）与 **I-7.8 树 ID 水位**（种子 …133）两条签名是 O2 单独判出来的 |
| K2-d | 今天罩着这一类的只有那条写死的 C381 用例吗 | **是，而且那条用例用的正是抽样抽不到的三样** | `tests/second_transaction_supplement_three_fault_injection.rs:359-423`：它 `arm` 了两次（两次故障）、用了 `OffsetBelow` + `EVERY_MATCHING_CALL`（落点维 + 持续维）、并且**失败之后拿同一个 `PoolWriter` 接着发下一次**。K1 的三类盲区与 K2 的「不往下跑」在这条用例里同时被绕开 |
| K2-e | 最小的改动是什么 | **两级：先补「不依赖模型」的那一半，再补模型** | 见第四节；两条改法都只在我自己的副本上量过、**被攻过零轮** |

## 二、K1：这个抽法结构性抽不到的那几类

### 2.1 抽样只造一种排期，四维里三维是常量

`FaultSchedule` 有五个字段（`fault_injection.rs:213-219`），随机注入这一路只经过一个构造器：

```
1276	    let plan = SharedFaultPlan::armed(
1277	        geometry,
1278	        FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),
1279	    );
```

`the_nth_call_across_the_pool` 把另外四个字段写死（`fault_injection.rs:224-232`）：

```
224	    pub fn the_nth_call_across_the_pool(fault: InjectedFault, ordinal: u64) -> Self {
225	        Self {
226	            fault,
227	            device: FaultDeviceSelector::EveryDevice,
228	            placement: FaultPlacement::AnyOffset,
229	            counting: FaultCounting::AcrossThePool,
230	            occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
231	        }
232	    }
```

⇒ 下面这些取值在**任何种子、任何规模**下都取不到，不是「抽得少」：

| 维 | 实现里有的取值 | 随机注入抽得到的 | 抽不到的那一类故障 |
|---|---|---|---|
| 哪块盘 | `EveryDevice` / `OnlyDevice(d)`（`:154-157`） | 只有 `EveryDevice` | 一块盘坏、另一块好（两块盘镜像的全部意义所在） |
| 落点 | `AnyOffset` / `OffsetBelow(n)` / `OffsetExactly(o)`（`:161-166`） | 只有 `AnyOffset` | 「某一类结构的写一律失败」「某个固定槽坏了」 |
| 按什么数 | `AcrossThePool` / `PerDevice`（`:181-184`） | 只有 `AcrossThePool` | 逐盘对齐的故障（同一逻辑写的两份镜像同时坏） |
| 第几次 | `TheNthMatchingCall(n)` / `EveryMatchingCallFromTheNthOnward(n)`（`:190-192`） | 只有 `TheNthMatchingCall` | **持续故障**：盘一直坏、屏障一直报错 |

⚠️ 注意这不是「实现没写」：这五种取值 `crates/` 里都有实现、也都有单测（`fault_injection.rs:1879` 那条 `every_call_from_the_second_onward_on_one_device_below_an_offset` 一条把三维都用上了）。**只是 campaign 一次都没用过。**

### 2.2 一段历史至多一次调用被动过手脚

`FaultOccurrence::fires_at`（`fault_injection.rs:201-208`）里 `TheNthMatchingCall(ordinal)` 只在 `matching_call_ordinal == ordinal` 时为真；`SharedFaultPlan` 只带一个 `schedule`（`decide` 在 `:470` 取的是单数的 `state.schedule?`）。
⇒ 第 n 次之后的每一次调用都原样放行。**「两次故障」这个形状在随机注入这一路上不存在。**

### 2.3 候选集出自不注入的那一遍，故障处理路径因此在候选集外

`draw_faults` 的候选步是拿**测量跑**（`inject_faults_into_history` 的第一遍，`per_step_checker: PerStepChecker::Skipped`、`SharedFaultPlan::unarmed`）记下的逐步调用数差算的：

```
1125	        let candidate_steps: Vec<usize> = (1..marks.len())
1126	            .filter(|index| marks[*index].calls.of(kind) > marks[index - 1].calls.of(kind))
1127	            .collect();
```

测量跑里一次故障都没有 ⇒ **只有在「前一次调用失败了」之后才会发出的调用，一次都没出现在 `marks` 里，因此一次都不是候选。**
这一类调用在 `singlefs-core` 里是实打实存在的，例子（各引一行原文）：

- `crates/singlefs-core/src/recovery.rs:79`：`        block_device.read_at(offset, &mut buffer).ok()?;` —— 读错被吞成 `None`，交给上面的回落逻辑；
- `crates/singlefs-core/src/recovery.rs:208`：`            continue;` —— `read_unit_via_locations` 第一份镜像读不出或校验和不对就读第二份；
- `crates/singlefs-core/src/recovery.rs:230`：`            FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,` —— 槽 0 读不出时按最小槽距去试槽 1。

这三处都只在「已经坏了一次」之后才走得到。要注入到它们身上，需要第二次故障 —— 而 2.2 说了没有第二次。
⚠️ 这一格与第 3 件第一轮判决里的 K1-h（双重崩溃，判「不拿它判」）**不是同一件事**：那一格是「两处都 0 覆盖、不分辨臂」；这里下面 2.4 给出的是一条**分得出臂的、可跑的**构造。

### 2.4 打中：一块盘的两个系统配置槽都读不出来 ⇒ 整个池挂不上

`choose_system_configuration` 逐盘取系统配置，某一块盘的两个槽都读不出来时**当场 `return Err`**，不看已经从别的盘读到的那一份：

```
244	            (None, None) => return Err(RecoveryFailure::NoValidSystemConfiguration { device }),
```

这条路要两次读失败落在同一块盘上（槽 0 一次、槽 1 一次）。按 2.2，随机注入这一路摆不出第二次故障 ⇒ **任何种子、任何规模都走不到这一行。**

可跑构造：`opus_probe_one_device_system_configuration.rs`（两条用例，一条打中一条对照）：

```
$ nice -n 19 cargo test --release -p singlefs-harness \
      --test opus_probe_one_device_system_configuration -- --nocapture
running 2 tests
test both_system_configuration_slots_dead_on_one_device_makes_the_whole_pool_unrecoverable ... ok
test one_failing_read_on_one_system_configuration_slot_still_recovers ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

两条用例的差别只有 `occurrence` 一个字段：

| 臂 | 排期 | 结局 |
|---|---|---|
| 对照（随机注入摆得出的那一格） | `ReadFails` + `OnlyDevice(1)` + `OffsetBelow(两个系统配置槽)` + `TheNthMatchingCall(1)` | `choose_system_configuration` **成功**，`recover` 读回文件 |
| 打中（摆不出的那一格） | 同上，只把 `occurrence` 换成 `EVERY_MATCHING_CALL` | `choose_system_configuration` 报 `NoValidSystemConfiguration { device: DeviceIdentity(1) }`，`recover` 报 `Failed`；**而只拿盘 0 做 reader 时 `choose_system_configuration` 照样成功** —— 信息一点没少，是这段代码自己不肯用 |

⚠️ **kb 里有一条现成的条款指着这一格**（写完这一节之后现查补上的）。`.claude/kb/checks-owed.md` 的 C79（系统配置与 journal 环的放置没人定） 那一行逐字写着（整行的一段）：

> **2 盘的账已算清（2026-09-02）**：E87（固定结构的放置）——系统配置单份掉盘 0 全池不可挂；每盘一份 + journal 镜像 8 格全可挂、窗口零丢失（镜像 ×2 写恰是 D2（RAID 条带策略）已定项 6 对任何写的 w≥2 下限）

「系统配置每盘放一份」这条定案（`.claude/kb/decisions/22-单元原子性怎么合成.md:170`，D22（单元原子性怎么合成） 已定项 8 第 1 条：「**系统配置每盘放一份。** 怎么更新它按已定项 21 办……」）买的就是「掉一块盘还挂得上」。
而 `recovery.rs:244` 在盘 0 的两份都好的情况下报 `NoValidSystemConfiguration { device: DeviceIdentity(1) }`，**买到的那一样今天在代码里兑现不了**。

⚠️ 即便如此，**我仍然不把它判成「core 的缺陷」**，理由是我没查清两件事：① C79 那一行说的「可挂」是几何可行性还是挂载路径的行为；② 「掉盘只读挂载」（`.claude/kb/decisions/02-RAID条带策略.md:148` D2（RAID 条带策略） 已定项 9 那一节的射程句里没写这一句，写这一句的是 C120（分叉盘回归的判定与重同步） 那一行的「第一版 2 盘、掉盘只能只读挂载（D2（RAID 条带策略） 已定项 9）」）在 `crates/` 里有没有落点。
我能坐实的是三句：① 这段代码在信息够的时候选择了失败；② kb 里有一条明写「每盘一份 ⇒ 掉盘还可挂」的账；③ **随机注入这一路在任何规模下都报不出这一格**，它今天没有任何 oracle 罩着。

### 2.5 换上今天抽不到的两种排期，campaign 报出来的东西完全变样

把 `:1278` 那一行换成另外两种排期（`probes.patch` 第三处，环境变量 `SINGLEFS_FAULT_PROBE_SCHEDULE`），别的一概不动，同样 24 段、同样 95 个注入点：

| 排期 | 收尾分布 | core panic | 新发现 | 产物 |
|---|---|---|---|---|
| 今天（`TheNthMatchingCall`） | 返回错误 48、被容错跑完 45、停在别的东西上 1、没走到 1 | 0 | **0** | `baseline-fast.txt` |
| `EveryMatchingCallFromTheNthOnward`（持续坏，每块盘） | 返回错误 81、停在别的东西上 13、没走到 1 | 0 | **13** | `probe-k1-schedule.txt` 第一段 |
| 盘 1 从起点段跑完起一直坏（`OnlyDevice(1)` + `PerDevice` + 每一次） | 返回错误 17、停在别的东西上 78 | 0 | **78** | `probe-k1-dead-device.txt` |

⚠️ **这两栏的「新发现」我不当成 core 的缺陷报**，理由与 K2-c 同源：这两种排期下模型对拍必然对不上（模型里没有「设备错」，见 3.1），13 条与 78 条里绝大多数的签名是 `ModelDisagreement`。
它们证明的是一件更小但更硬的事：**这两维一换，campaign 的输出面目全非；今天这一维一个取值都没抽过。**

⚠️ **我自己先踩了一个坑，写出来免得主 agent 重踩**：`persistent-one-device` 的第一版从第 1 次调用就让盘 1 坏，撞出 **49 次 panic**，位置全是 `crates/singlefs-harness/src/history.rs:832` ——
那是执行器自己 `HistoryPool::start` 里 `make_filesystem(...).expect(...)`，**不是 core 的缺口**（`draw_faults` 的注释 `fault_injection.rs:1123-1124` 早写明了起点段不摆注入点就是为了躲这个）。
`probe-k1-schedule.txt` 里 `##### schedule=persistent-one-device` 那一段**是作废的那一版**（panic 49、新发现 138），留在产物里是为了让这个坑可查；改成「起点段跑完之后盘 1 才死」之后 panic 归 0，那一份是 `probe-k1-dead-device.txt`。

### 2.6 「42 格里命中 20 格」这个分母

`InjectionPoint::all()`（`fault_injection.rs:746-765`）= 7 类操作 × （4 类写落点 + 读 + 屏障）= 42。副本上数出来命中 20 格、没命中 22 格（`awk '/故障注入快档/{n++} n==1' baseline-fast.txt | grep -c '^    注入点 '` = 20），与背景材料第二节一致。
这张表的三个维是（操作、调用种类、写落点）。2.1–2.3 的三类盲区（排期四维、故障次数、故障处理路径）**一维都不在这张表里** ⇒ 「20 / 42」既数不到它们，也不会因为它们而变小。
这不是说那 22 格无所谓（其中 `ColdStartRecover/` 那 5 个写与屏障的格子是恒空的：冷启动只读不写），而是说**验收标准「报每个注入点的命中次数，没命中的逐个列名」量的空间，比这一件实际要罩的空间窄**。

## 三、K2：注入之后历史不往下跑，罩不到什么

### 3.1 为什么不往下跑：三行代码串起来的

背景材料写的是「`model_comparison` 明写 I/O 错一律 `Unexplained`，所以执行器在注入那一步就停」。现查，这句话的机制是三行：

```
crates/singlefs-harness/src/model_comparison.rs:162	        | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
crates/singlefs-harness/src/model.rs:1402	                    ObservedRefusalReason::Unexplained => None,
crates/singlefs-harness/src/history.rs:2953	                if !continues_past_the_ring_turn_form {
```

`Unexplained` 在 `judge_and_advance` 里恒不被接受（`model.rs:1404` 起 `let Some(accepted_reason) = accepted else { … return Err(…) }`），
拒绝的那一格因此必然变成 `RefusedWhenModelRequiresSuccess`；执行器拿到 `model_disagreement.is_some()` 就在 `:2954` `return Some(observation)`。
⇒ **注入那一步之后的每一步，一步都不跑。** 快档 95 次注入里 48 次以「返回了错误」收尾（`baseline-fast.txt`），这 48 段历史各自剩下的步全部没跑。

⚠️ 要点：这**不是**「故障注入选择只判一步」，而是**模型对拍这条路把它顺手关掉的**。`fault_injection.rs:1236-1238` 的注释自己写明了这一点（整行抄）：

```
1236	/// 注入那一步的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一步要么返回了 `Err`，
1237	/// 要么是冷启动、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
1238	/// 模型因此必然报「该成却拒了」——那一格不算失败，它就是这一件要的结果。
```

也就是说，**「这一步返回了 Err」被当成了成功收尾，而不是「继续往下看」的起点。**

### 3.2 把「不停」这一条改掉，同样的 95 个注入点从 0 条新发现变成 38 条

`probes.patch` 第一处：在 `history.rs` 那个 `return` 之前加一个由环境变量开的分支，满足下面四条才放过去 —— checker 没判红、没 panic、执行器没判出、模型对不上的那一格是 `RefusedWhenModelRequiresSuccess`、而且这一步的错误成员含 `BlockDeviceError`。**别的一概不动**（模型不前进，抽样、注入、判定全不变）。

对照（证明这个开关不是「把判据放松了」）：开着开关跑随机历史快档，96 段照样全绿、退出码 0（`control-random-history-with-probe.txt`）——两块等大的内存盘上不注入故障就报不出块设备错，开关是死的。

```
$ SINGLEFS_FAULT_PROBE_CONTINUE_PAST_DEVICE_ERRORS=1 cargo test --release -p singlefs-harness \
      --test second_transaction_supplement_three_fault_injection fault_injection_fast_tier -- --nocapture
```

| 量 | 今天 | 开了「往下跑」 |
|---|---|---|
| 注入次数 | 95 | 95（逐个相同：抽样一个字没改） |
| 收尾「返回了错误」 | 48 | 0（这一格不再是终点） |
| 收尾「被容错、历史跑完」 | 45 | 71 |
| 收尾「历史停在别的东西上」 | **1** | **23** |
| core panic | 0 | 0 |
| 重开走到模型都不允许的版本 | **0** | **14** |
| **新发现** | **0** | **38** |

### 3.3 那 38 条里，有两条是池级 checker 自己判的（不依赖模型）

38 条里大部分的签名是 `ModelDisagreement`，而模型里本来就没有「设备错」这个概念 —— 拿它们当缺陷报是不诚实的。
`probes.patch` 第二处因此再加一道：**第一次放过块设备错之后，把模型对拍整段挂起**，只留 checker（O2）与 panic 这两条不依赖模型的判据。结果（`probe-k2-suspend-model.txt`）：

- 收尾「历史停在别的东西上」7 次、新发现仍有 22 条；
- 其中**两条签名是 `CheckerViolations`**，与模型一个字的关系都没有：

```
种子 7463871032432355119 barrier_fails：整池第 19 次barrier，落在第 1 步（PublishOverwrite）：CheckerViolations { invariants: ["I-3.1"] }
  I-3.1：盘 0：记账的已分配 Some(606208)，遍历全部有效根得到 540672；机理：根环槽数 24、最新根 txg 7、环里自证过的根槽 7 个、最老的自证过的根 txg 0、遍历的候选根槽 7 个、被实例表判抛弃的根槽 0 个、回退下界 F 0、低于 F 的根槽 0 个
种子 7463871032432355133 write_fails：整池第 35 次write，落在第 2 步（PublishFirstFile）：CheckerViolations { invariants: ["I-7.8"] }
  I-7.8：根环水位最大 11，盘上出现过的最大树 ID 14
```

两条都单独复现过（`probe-k2-single-seeds.txt`，一次一段历史、单线程）。逐条说它们**是不是**「已知红」：

- **I-3.1 那条不是已知红第 0 条**：第 0 条要求根环转过一整圈，这里 `根环槽数 24、最新根 txg 7`，没转过 —— 所以执行器把它报成新发现，而不是接走。形态是「记账的已分配 606208 > 遍历得到 540672」，多 65536 字节 = 一个 64 KiB 的单元；
- **I-7.8 那条是「一次失败的发布留下的树 ID，被后面的步收进了『出现过的』」**：水位 11、盘上最大树 ID 14。I-7.8 的 checker 读法（用户 2026-09-14 收尾弹窗定甲）只数诞生代号不超过根环里最大 checkpoint_txg 的节点，所以它**要靠后面几步把 txg 推回去**才数得进来 —— 注入那一步当场是看不见的。

⚠️ **这两条我报成「这一类今天零覆盖」，不报成「core 有两个缺陷」。** 分辨它们要回答「失败的那次发布许不许留下这些东西」，那正是 C381 / D16（发布语义） 已定项 7 还开着的题面，不归这一轮。
能坐实的是：**同一个 checker、同一批种子、同一批注入点，只因为历史停在注入那一步，这两条一次都没判出来。**

### 3.4 这一整类今天谁在罩：只有那条写死的 C381 用例，而它用的正是抽样抽不到的三样

`tests/second_transaction_supplement_three_fault_injection.rs` 里那条 `a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites` 干了三件事：

| 它做了什么 | 代码在哪 | 随机注入这一路摆不摆得出 |
|---|---|---|
| 系统配置槽的写**一直**报错（落点维 + 持续维） | `:359-368`（`FaultPlacement::OffsetBelow(...)` + `FaultOccurrence::EVERY_MATCHING_CALL`） | **摆不出**（2.1） |
| 同一段历史里**第二次** arm（第二次故障） | `:399-402` | **摆不出**（2.2） |
| 失败之后**拿同一个 `PoolWriter`、同一个 allocator 接着发下一次**（`:403-412`），第三步才现形（`:426-444`） | `:398` 起 | **跑不到**（3.1） |

⇒ 背景材料那句「只有写死的那条 C381 用例罩着」现查成立，而且比它写的还紧：**这条用例同时绕开了 K1 的三类盲区与 K2 的停机规则，四样里少任何一样都测不出来。**

### 3.5 这一类里今天零覆盖的形态，逐条列

判据：我按「注入之后还会发生什么」把这一类拆成七种形态，逐条现查今天罩不罩得到。「罩得到」要指得出一条会红的检查。

| # | 形态 | 今天罩得到吗 | 依据 |
|---|---|---|---|
| ① 失败之后**同一个写入口接着发布**，分配器退回把上一次占的槽再分出去 | **只有 C381 那条写死用例**（一条固定历史、一个固定落点） | 3.4；随机注入这一路 3.1 跑不到 |
| ② 失败之后**重开、再写**（冷启动恢复之后接着发布） | **零** | 注入之后的重开是 `fault_injection.rs:1353` 的 `recover(&image, JournalPolicy::Consult)`，只读；整段代码里注入之后没有第二次写 |
| ③ 失败之后**换个实例挂载**（取号烧掉了、下一次挂载才看见） | **零**，而且是一笔已登记的欠账 | `.claude/kb/checks-owed.md` 里 C378（取号之后写行发布被拒，已烧掉的实例代号不回卷） 那一行逐字写着「还开着的是取号之后因块设备报错失败的那一路：写行或暖机那几次写报错时代号已经烧了」（按内容定位，不按行号：那份文件这几天在被另一个会话重排）；开了「往下跑」之后这一形当场现形（`probe-k2-single-seeds.txt` 种子 …116 与 …122 各一条「取到的实例代号：模型 取号 5；实现 取号 6」） |
| ④ 失败之后**回退**（失败那一步留在环里的根成了回退候选） | **零** | 同 3.1；开了「往下跑」之后种子 …122 报出「模型说该拒、实现做成了：模型 MountRollback 该拒：["回退目标不在根环里"]；实现 做成了」 |
| ⑤ 失败之后**抬 F**（上界取自环里那条没提交的根） | **零** | 同上；快档里 `RaiseRollbackFloor` 上注入 8 次，全是读，8 次全在注入那一步停住 |
| ⑥ 失败留下的**记账/水位**要靠后面几步才数得进来 | **零** | 3.3 的 I-3.1 与 I-7.8 两条；注入那一步的镜像上 checker 判绿（`baseline-fast.txt`：checker 跑 95 次、判红 0） |
| ⑦ 失败之后**什么都不做、只重开**（今天唯一跑的那一条） | **罩得到** | `fault_injection.rs:1353-1399`，快档 95 次全跑过 |

七种里罩得到的只有 ⑦，①有一条写死用例，②③④⑤⑥ 五种今天是零。

## 四、要让 campaign 也罩得住，最小的改动

⚠️ 下面两条**都是我自己提的，只在我自己的副本上量过、被攻过零轮**（`.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算『没被攻过』」）。
每一格标「量过」（贴副本上的原样输出）或「推的」（按代码推、没实现没跑）。

### 改法甲（小）：注入之后照常往下跑，但从那一步起只留不依赖模型的判据

一句话：`history.rs` 那个 `return` 前面加一个分支 —— 这一步被拒的成员是块设备错、而且 checker 没判红、没 panic ⇒ 不停，接着跑；同时把这一段历史剩下步的**模型对拍整个挂起**，只留 checker 与 panic。
改动面：`history.rs` 两处（`probes.patch` 前两处，合计 +27 行），`fault_injection.rs` 零行，模型零行。

| 这一格 | 甲改完会怎样 | 量过 / 推的 |
|---|---|---|
| 快档新发现 | 0 → **22**（其中两条是 checker 自己判的） | **量过**：`probe-k2-suspend-model.txt`「以「已知红」收尾 {0: 5, 1: 4}、新发现 22」 |
| 快档会不会立刻红 | **会**（`fault_injection_fast_tier_returns_errors_instead_of_panicking` 当场判红，退出码 101） | **量过**：同上 |
| 3.5 的 ⑥（记账 / 水位） | 从零变成两条会红的签名 | **量过**：3.3 |
| 3.5 的 ②③④⑤ | 走得到了，但判据只剩 checker + panic ⇒ 只在「盘面坏了」时才报得出，「模型不该允许这一版」报不出 | **推的** |
| 挂钟 | 快档 2.1 秒 → 2.5 秒（同一台机、单线程那一版 6.6 秒 vs 基线未单独量） | **量过（粗）**：`probe-k2-continue.txt` 末「用时 2.5 秒」、`baseline-fast.txt` 末「用时 2.1 秒」；两次跑的线程数不同，这个比较**不可靠**，主 agent 要重量 |
| 「已知红」清单 | 要加行：甲之下第 0 条从 3 次变 5 次、还多出两条清单外的形态 | **量过**：`probe-k2-suspend-model.txt`「{0: 5, 1: 4}」 vs 基线「{0: 3, 1: 4}」 |

### 改法乙（大）：给模型一条「设备错」的拒绝理由，模型跟着放宽

一句话：`ObservedRefusalReason` 加一条设备错（或让 `ModelRefusalReason` 多一条），`model_comparison.rs:162` 那一支映射过去，`judge_and_advance` 一律接受这条理由并且**不前进模型状态**；
再把「这一步可能已经烧掉的东西」并进模型的允许集 —— 今天 `inject_one_fault` 已经算过一份（`fault_injection.rs:1370-1379` 的 `allowed`：不注入时失败那一步会写出的那几版），乙要把它从「只在重开那一刻用一次」变成「从此之后一直用」。

| 这一格 | 乙改完会怎样 | 量过 / 推的 |
|---|---|---|
| 3.5 的 ②③④⑤ | 模型能答话了，`冷启动读回` / `取到的实例代号` / `该拒却做成` 这些格子才有意义 | **推的** |
| 会不会立刻红 | **会，而且红得比甲多**：不挂起模型时新发现 38（其中 24 条是模型类签名） | **量过**：`probe-k2-continue.txt`「新发现 38」，减去甲那一版的 22 得 16 —— 这个减法**只是两次跑的差**，不是「乙会报 16 条」，主 agent 别直接引 |
| 改动面 | `model.rs` + `model_comparison.rs` + `history.rs` + `fault_injection.rs` 四处；还要定「失败之后模型的 txg / 实例号 / jsn 怎么走」，那是 C381 与 D16（发布语义） 已定项 7 开着的题面 | **推的** |
| 顺序 | 乙**不能**先做：它要先有「失败之后写入口怎么走」的条款，而那条今天在 C381 里还写着「改法待用户定」 | **推的** |

### K1 侧的改法（同样零轮）：把排期那两维放进抽样

`draw_faults` 今天抽三样（哪一种注入、哪一步、那一步的第几次调用）。再抽两样就够了：
① `occurrence` 在「只这一次」与「从这一次起每一次」之间抽；② `device` 在「每块盘」与「只某一块盘」之间抽。
**别**顺手把落点维也放进来 —— 落点维配上持续维就是 C381 那条用例的排期，它今天必红，会把快档变成一条已知红。

| 这一格 | 会怎样 | 量过 / 推的 |
|---|---|---|
| 换成持续（每块盘） | 新发现 0 → 13、panic 0 | **量过**：`probe-k1-schedule.txt` 第一段 |
| 换成「盘 1 起点段之后一直坏」 | 新发现 0 → 78、panic 0、重开走到模型不允许的 0 | **量过**：`probe-k1-dead-device.txt` |
| 2.4 那一格（两个系统配置槽都坏） | 抽样抽得到了，但**要配落点维**才摆得准；不配落点维时要靠运气让两次读都落在同一块盘的前两个槽上 | **推的** |
| 这两维一放开，模型对拍必然对不上 | ⇒ **K1 的改法压在 K2 的改法上**：不先做甲或乙，放开排期只会多出一堆 `ModelDisagreement` | **量过（间接）**：13 条与 78 条里的签名全是 `ModelDisagreement` / `HarnessJudgement` |

## 五、打中之后的四句（每个打中逐条答）

按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」。

| 打中 | ① 分不分辨臂 | ② 被判的系统当时看不看得到判别它的东西 | ③ 满足判据字面的哪一分句 | ④ 前几轮的改法在这几格上还中不中 |
|---|---|---|---|---|
| K1-a/b/c（结构性抽不到） | **分**：换 `occurrence` 一个字段，新发现 0 → 13；换 `device` + `occurrence`，0 → 78 | **看不到**：报告里没有任何一栏是「排期长什么样」；`FaultSchedule::render`（`fault_injection.rs:257-281`）会打，但 campaign 的 tally 一个字段都不记它 | K1 字面「举出一类**结构性抽不到**的注入点（不是规模不够，是这个抽法永远抽不到）」——三类都不是概率小，是构造上取不到 | 不适用：第 1、3 件的改法都不动排期 |
| K1-d（两个系统配置槽都坏 ⇒ 池挂不上） | **分**：同一份代码、同一块盘、只换 `occurrence` 一个字段，一条 `Ok` 一条 `Err`，两条用例都在 `opus_probe_one_device_system_configuration.rs` 里跑得出来 | **看不到**：快档 95 次注入报出的错误成员只有 4 种（`MountError::Acquisition`、`MountError::Publish(PublishError::BlockDevice)`、`PublishError::BlockDevice`、`publish_without_units`），`RecoveryFailure::NoValidSystemConfiguration` 一次都没出现过 | 同上，并且是 K1 要的「能写成可跑的构造」 | 不适用 |
| K1-e（42 格这个分母） | **弱分辨**：它不改任何一次跑的结果，只改读数的人怎么理解「20 / 42」 | 看得到（报告里就印着），但没人对着它问「这张表的维够不够」 | K1 字面「快档 42 格里 22 格没命中」那一句的前提 | 不适用 |
| K2-b/c（往下跑就有 38 / 22 条） | **分得很开**：0 → 38（模型照旧）、0 → 22（模型挂起）；对照跑证明开关本身不判任何东西（随机历史 96 段照样全绿） | **看不到**：`FaultOutcome::SurfacedAsAnError` 这一格今天的文档注释写的是「要的就是这一条」（`fault_injection.rs:790-791`），报告里 48 次都算在这一格里，没有任何一栏区分「这一步之后还剩几步没跑」 | K2 字面「这一整类损伤，随机注入这一路看不见」 | **中**：第 1 件那轮加的「checker 判绿的镜像上冷启动恢复报错 ⇒ 失败」这条判据在我的两个 K1 探针里各红了一次（`probe-k1-dead-device.txt` 种子 …114），说明它还活着、还判得动；第 3 件两轮的改法都在 `crash_injection.rs`，不碰这一件 |
| K2-d（C381 那条用例同时绕开四样） | **分**：四样里去掉任何一样（不 arm 第二次、不接着发、换成 `TheNthMatchingCall`、换成 `AnyOffset`）那条用例就测不出 C381 —— 这是读代码得出的，**没跑**（列在第六节） | 看不到 | K2 字面「只有写死的那条 C381 用例罩着」 | 不适用 |

### 关于「装置把用户动作写死」这一条

`.claude/rules/three-way-inference.md`「攻方腿的装置把用户动作写死时，它报的『分辨臂』可能是装置造出来的」。逐条对照：

- **K1-a/b/c 不是抽样观测，是代码性质**：「`the_nth_call_across_the_pool` 是唯一的构造器」「一个 plan 只带一个 schedule」「候选集来自不注入的那一遍」三句都能用 `grep` 判真假，与跑哪一段历史无关 ⇒ 不受这一条影响。
- **K1-d 的可跑构造确实把后缀写死了**（mkfs → 取号 → 暖机 → 第一个文件，然后 `recover`）。但它要证的不是「这段历史上不中」而是「这一行走得到」，方向相反：写死的前缀只会让结论更弱，不会让它假。
- **K2-b/c 没有写死用户动作**：走的是第 1 件的生成器、`GenerationWeights::BROAD`、24 个种子、每段 20 步、七类操作照权重抽，与今天快档逐字相同（`FaultInjectionCampaign` 一个字段都没改）。我改的只有「停不停」。这也是我特意不去手写一段历史的原因。

## 六、没打中的形状（试过、没成的）

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 注入之后 core panic | 快档 24 段 × 95 个注入点，四种排期各跑一遍（今天的、持续每块盘、盘 1 从起点段后一直坏、盘 1 从第 1 次调用起坏） | **一次都没有**。唯一的 49 次 panic 是我自己的探针把注入摆进起点段撞出来的执行器 `expect`（`history.rs:832`），改掉就归 0。**「返回错误而不是 panic」这条性质，我攻不动** |
| 重开走到「模型都不允许的版本」（今天三条判定的第三条） | 今天的排期 95 次、持续排期 95 次、盘 1 死 95 次 | 今天 0、持续 0、盘 1 死 0。只有开了「往下跑」之后才出现 14 次 ⇒ 这一条判定在**今天的射程内**判不出东西，不是它写错了 |
| 写落点维（`OffsetExactly` / 按结构分层抽） | 读了 `classify` 与 `WRITTEN_STRUCTURES`（`fault_injection.rs:735-740`、`:484-493`） | **没打中**：一步之内 `call_ordinal` 在 `[first, last]` 上均匀抽（`:1131-1137`），那一步里出现过的每一类写落点概率都严格为正 ⇒ **这一维是「规模不够」，不是结构性抽不到**，按 K1 的题面不算 |
| 「测量跑提前停的那几段」 | 快档 24 段 | **不成立**：快档实测「测量跑就提前停的 0 段」（`baseline-fast.txt`），这一形在快档上是空的；大档没跑 |
| 注入落在重开那一段（`recover` 自己的读上） | 读代码 | 结构性为 0（`fault_injection.rs:1344-1353` 的重开用的是重建出来的 `MemoryPool`，不过故障包装），但这**与第 3 件第一轮 K1-f 同形**（冷启动只读不写那一格），按「不重复前几轮」不拿它当这一轮的打中 |
| 四种注入种类之间的比重 | 读 `draw_injected_fault`（`:1091-1106`） | 四种等概率、`below(4)`，快档实测 26 / 28 / 18 / 23，没发现偏斜 |
| `FlippedBit` 翻位落不落得进短缓冲区 | 读 `FlippedBit::within`（`:97-104`） | **按缓冲区长度取模**，多短都翻得到一位；没打中 |
| C381 那条用例「四样里去掉一样就测不出」 | 只读代码推的，**没跑** | 列在第五节 ④ 那一格里标着「没跑」 |

## 七、这条腿自己的限度

1. **全部的数都在副本上**（`/tmp/claude-1000/m2s3i4-r1-opus/repo`），按规则不进 kb。主 agent 要在入库装置上重做；`reproduce.sh` 从 `rsync` 开始，一条命令能重来。
2. **只跑了快档一轮（24 段 / 20 步 / 每段 4 次），大档一次没跑。** 「新发现 0 → 38」这个对比只在这 24 个种子上量过；换种子基会不会还是这个量级，我没验。
3. **我没有判「这是不是 core 的缺陷」。** 2.4（一块盘的系统配置槽坏了池就挂不上）与 3.3（I-3.1 / I-7.8 两条 checker 红）我都只报到「这一类今天零覆盖」为止：前者我找到了 C79（系统配置与 journal 环的放置没人定） 那一行与 D22（单元原子性怎么合成） 已定项 8 第 1 条（见 2.4），但没查清那句「可挂」说的是几何可行性还是挂载路径的行为；后者压在 C381（根已落盘之后发布失败，分配器仍退回） / D16（发布语义） 已定项 7 还开着的题面上。
   ⚠️ **这一条我自己先犯过一次错**：草稿里写的是「`.claude/kb/decisions/` 下按降级那几个词 grep 零命中」，现查**不是零命中**（10 个文件命中，其中 C79 那一行正面答了这一格）。是我先写了结论再去查，改的过程记在这里，免得主 agent 以为我查过。
4. **改法甲、乙与 K1 那条放开排期的改法，都是我自己提的，被攻过零轮。** 甲量过一轮、乙一行没写、K1 那条只把排期换成写死的两种量过，没做成抽样。
5. **挂钟那一格不可靠**：基线 2.1 秒与探针 2.5 秒是在不同线程数下跑的（基线 24 线程、探针那一版也 24 线程，但单种子那几次是 1 线程）。改法甲的真实代价要重量。
6. **K3–K6 不归我**，一个字没判；第 3 件（崩溃注入）那两轮的结论我只用来避开重复角度，没当这一件的结论。
7. **没跑门禁**：`.claude/gate.d/stage-owners.tsv` 里 `three-way-attack` 名下 `awk` 出来是空的（第八节贴了命令与输出）。
8. **副本上加的那个测试文件与三处探针都没入库**：它们在模型目录里（`opus_probe_one_device_system_configuration.rs`、`probes.patch`），要不要落进 `crates/` 由主 agent 定。副本本身在 `/tmp/claude-1000/m2s3i4-r1-opus/`，会被清掉，要留的从模型目录里取。

## 八、门禁

`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段：

```
$ awk -F'\t' -v me=three-way-attack '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
$ grep -c "three-way-attack" .claude/gate.d/stage-owners.tsv
0
```

（第一条命令零输出，第二条输出 `0`。）⇒ 这一轮没有该由我先跑的门禁阶段，一个阶段都没跑。

## 九、没做什么

- **不判 K3、K4、K5、K6**：K4 / K5 归云端正推，K3 / K6 归本地攻方，我一个字没判。
- **不替主 agent 采纳**：第四节三条改法（甲、乙、放开排期）全部**被攻过零轮**，采纳与否、先后次序都归主 agent 与用户。
- **不把副本上的数当入库数**：全部数字都在 `/tmp/claude-1000/m2s3i4-r1-opus/repo` 上量的，`reproduce.sh` 能重来；按 `.claude/rules/three-way-inference.md`，主 agent 要在入库装置上重做一次才能引进 kb。
- **不动工作区的 `crates/`**：`git status` 里 `crates/` 的改动是这一件的实现员留下的，我一个字节没碰；也没有 `git add` / `git commit`。
- **没跑大档**（`#[ignore]` 那条只在单种子上跑过 3 次），没跑 `check.sh`，没跑门禁 59 号变异表。
- **没判第 3 件（崩溃注入）那两轮的结论**：只读来避开重复角度。刻意避开的角度有五个：起点那一段（第 3 件 K1-d，这一件里归 K5/正推腿）、失败那一步不进候选（第 3 件 K1-e）、冷启动结构性零（第 3 件 K1-f）、断言够不够（第 3 件 K1-g / 第二轮 K6）、抽样分布偏在中间（第 3 件第二轮 K1-③）。**双重故障这个角度我没有原样重复**：第 3 件 K1-h 判的是「两处都 0 覆盖、不分辨臂」，这一轮 2.3 / 2.4 给的是「候选集出自不注入那一遍」这条机理加一条分得出臂的可跑构造。
