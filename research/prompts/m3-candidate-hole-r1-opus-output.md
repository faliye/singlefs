# 回退候选集有洞时的判法：第一轮攻方腿（Opus）报告（2026-10-04）

<!-- doc-lint:not-numbers H1 H2 H3 E1 E1W RC RCW NA K1 P1 -->

轮名 m3-candidate-hole-r1；攻击面 H1、H2（正文第三节）。被判对象按冻结副本 `/tmp/claude-1000/m3-candidate-hole-r1/tree/` 读；原型与反例全在它的拷贝上做（草稿目录 `/tmp/claude-1000/m3-candidate-hole-r1-opus/tree/`），只编 release 原型、只跑原型二进制，没跑任何测试目标与 checker 档。副本上的数不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-candidate-hole-r1-opus-model/`。在一个新草稿目录 `<草>` 里依次跑（每段经 `scripts/run.sh` 走内存包装 2G、`capped.sh 1`；编译经内存包装 5G、`capped.sh 8`）：

```
M=research/prompts/m3-candidate-hole-r1-opus-model
bash $M/scripts/rerun-all.sh <草> build     # 拷冻结副本、打 scripts/apply-walk-patch.py、放进原型、编 release
for p in sb nrb rh skr show ss4 ss5 ss6a ss6b; do bash $M/scripts/rerun-all.sh <草> $p; done   # ss5 约 3.5 分钟，ss6a 约 3 分钟（我分两次跑的：short-ring-sweep.sh <草> 4 6 "4" 与 <草> 6 6 "6"）
python3 $M/scripts/analyze-judges.py <草>/out/ss*.out      # 同法 sb-*、rs1-*、nrb*、skr-*
python3 $M/scripts/pair-writers.py <草>/out ss ; python3 $M/scripts/pair-modes.py <草>/out ss   # 同法前缀 sb-
```

`results/out/` 461 份输出、`results/sum/` 汇总、`results/SHA256SUMS`（sha256 802b8885f9d1fd70e149653ee485ecf85ccca52592c3373081190f8f48dd5e52）是最后一遍按上面命令跑出来的。模型文件 sha256：

| 文件 | sha256 |
|---|---|
| proto/Cargo.toml | 6cbef51d4c08a46a2b6ed487aa386c9a0449696159d69a47ac1e6f9862d41578 |
| proto/src/lib.rs | cedaf51ec179a29b611bb7d582919ca48a1b2af0828f7bba37e66f706ce60965 |
| proto/src/bin/slot_bad.rs | 113f9858622cd6ce8cae11ef4524e4642de411215bb6a331b3c266da7605c15f |
| proto/src/bin/newest_row_bad.rs | 7b5d4a69be55a627fe8bdf386d7756958728059b1b0bc2a8cc09cd0cb742741e |
| proto/src/bin/rollback_hole.rs | a565f5dbc3d5e7788e65e0f23dd727ff26022f32887d769d17fddf204d48f256 |
| proto/src/bin/short_ring.rs | c03acccd0d8b1a054029a4c00824f1ca3c3404f3bcc71e31e514c2e39da669af |
| proto/src/bin/skip_row.rs | 54c0f828554c6aace97870353a997ed79f761ccee9ca3f5f84fe14b5d202e39a |
| walk-patch/hole_proto.rs | 8fc0a132eef7c819c83d2b01d8e8845e0f4b676ea0b7dcf57bfc1450634bb264 |
| scripts/apply-walk-patch.py | 092d285c52ad669c26723dc2e26cf44ea0949c1b99439422ffb18ed552ce57dd |
| scripts/build.sh | 1452233f9496c0b9982cb4f8ecdc74b1e9f70c56cafd3801a99e255208918acf |
| scripts/run.sh | 4e6213aa9e662ebad348791b995506abd87f5a629bc1745af64fe474449e59e0 |
| scripts/rerun-all.sh | db6f8b05f5438676af761749c0a41c961fb922683bd29e934eda0f7d6e7e8d58 |
| scripts/rollback-sweep.sh | 225ba2f729beae83bf308d6a8bda9e3aa44a20c11cd24eefa537b4563773e719 |
| scripts/short-ring-sweep.sh | 9d26209bfcbc970c8b77f7d5bdf492c067206c8d6dcc23a135e0cf08bad8c551 |
| scripts/analyze-judges.py | 188740cf1f93df4557eb78e5019c9309c14b946904a39a8333b0991fd6e581c3 |
| scripts/pair-writers.py | f62d1d5c92ac05f00b0bfdc4d87fc3d4bf983384c2416796d401353d0718f837 |
| scripts/pair-modes.py | 1cf66641f351f1ccfb4f7e31254be2859ed72e0458befd226432108883c2ebf7 |

原型怎么判：`apply-walk-patch.py` 往副本的 `walk.rs` 里 `include!` 一份 `hole_proto.rs`，加三样东西，冻结副本与主仓一个字节没动——① 洞表：窗口 [max(F_生效, N − 3S + 1), N] 里每个 txg 记「根在位 / 被抛弃 / 由记录施加 / 取号跳过 / 存在过却走不到」，取号跳过怎么认由开关 `HOLE_SKIP` 定；② I-3.1 每块盘的判法由开关 `HOLE_I31` 定；③ I-3.5 缺口那一判在 `HOLE_SKIP` 不是 rows 时改按洞表答。副本 core 的回收门槛加一个坏写者开关 `HOLE_BAD_WRITER=m4a`，取变异表那一行的改法（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/mutations.tsv:148`「effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest })」，门槛 = 最旧有效根 + 1，改的是 `/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-core/src/mount.rs:1340`「oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))」这一行，提前回收最旧那条候选根还引用着的槽：记账少，不安全方向）。每张镜像在 12 组开关下各判一次（`proto/src/lib.rs` 的 `COMBOS`）。

候选与开关的对应（攻方提的几样都只在我的模型上量过、被攻过零轮）：

| 名字 | 开关 | 定义 |
|---|---|---|
| H1-A 只认行 | `HOLE_SKIP=rows` | 取号跳过 ⟺ 有一行 (i, Ti)、Ti < t，t 之上第一个锚（读得出的根或由记录施加的版本）的实例 v > i、中间实例的行都是 (j, 0, 0)，t 之下最近的锚不晚于 Ti（R⁻ 在盘上时就是今天的 gap-row；调查员第十节那一格的写法） |
| H1-B 并读 | `HOLE_SKIP=union` | H1-A ∪ 「journal 里有一条带提交标记、txg = t、没被施加的记录」；「施加过」= 被某行 t ≤ Ti 罩住，或这条记录是最新实例自己的 |
| H1-B 朴读 | `HOLE_SKIP=naive` | 同上，但「施加过」只认实例表行（`versions_applied_only_by_records` 那一判的取法） |
| H1-C（攻方，零轮） | `HOLE_SKIP=counter` | journal 里那条记录说 t 存在过（被行罩住或是最新实例的）就压过行证据；否则同 H1-B 并读 |
| H2-NA | `HOLE_I31=na` | 洞表里有「存在过却走不到」的 txg ⇒ I-3.1 整条不适用 |
| E1（字面） | `HOLE_I31=e1` | 丁-defer 门槛改成区间 [θ_low, θ_high]：θ_high 今天那个，θ_low = max(F_生效, min(窗口里最旧有效根, 窗口里那一槽没住着它该住的 txg 的根的位置的最小 txg))；记账落在 [遍历(θ_high), 遍历(θ_low)] 里就算对得上 |
| E1W（攻方宽读法，零轮） | `HOLE_I31=e1w` | 区间上沿换成「今天的遍历和 + 最新根账里释放代高于 θ_low、遍历没走到的已释放槽」；`e1wu` 只在有「存在过却走不到」时放宽 |
| RC（攻方，零轮） | `HOLE_I31=rc` | 记账那一行对最新根账里的记录比：落在 [E(θ_high), E(θ_low)]，E(θ) = 已分配记录的槽 + 释放代高于 θ 的已释放记录的槽；另判遍历走到的每个槽都在 E(θ_high) 那一份里 |
| RCW（攻方，零轮） | `HOLE_I31=rcw` | RC 的下沿与逐槽那一判；上沿取 min(E(θ_low), E1W 的上沿) |

## 各格判定一览

镜像集（数都从 `results/sum/` 现数；「盘面」= 一张被判镜像）：

| 集 | 历史 | 合法性 | 盘面 |
|---|---|---|---|
| ss（短 journal 环） | mkfs（journal 环 1 MiB；12 KiB 到 256 KiB 六档 mkfs 报 `UnitAreaStartOffTheClusterSegmentBoundaryUnsupported` 拒收，那几次探针的输出没留，按它报的 slot_after 推环长要是 1 MiB 的整数倍，1 MiB 是最小一档，推的）→ 实例 1 写到 last ∈ {4,6,8} → 下一次顺序写崩在根槽写之前（记录已落）→ 实例 2 可写挂载（恢复把它由记录施加出来）→ 顺序写 3S 次，每次 1 / 1.5 / 2 MiB（一次发布 33–65 条记录，上限 85 条）；S ∈ {4,5,6} | 合法：无介质故障，崩溃点是层 0 枚举域里的一个状态（根槽写之前的整段前缀） | 正确写者 432（其中洞表里有「存在过却走不到」的 180）；m4a 同数 |
| sb（根槽读坏，同一实例） | 实例 1 写到 L ∈ 12..20，把 t ∈ L−6..L−1 那一槽根记录翻一个字节 | 介质故障镜像（C563 那一类来源） | 正确写者 54 张翻坏的、54 张没翻的；m4a 同数 |
| nrb（最新实例写行那条根读坏） | 实例 1 写到 10（或 4）→ 实例 2 可写挂载写行 → 再写 3（或 3..6 次 2 MiB 顺序写）→ 翻坏写行那一条（或前两条）根槽 | 介质故障镜像 | 翻坏的 6 张 |
| rs1（回退 + 根槽读坏） | 实例 2 挂载写行 11、挂载里另发 12、13，管理员回退到 txg 5..9（发在 14），之后用户动作长度 0..2 放开扫（over / seq / inodes / empty 的全部组合，21 种），把 11..R−1 全翻坏，R 取 14 到最新 | 介质故障镜像 | 285 段，各 1 张干净、1 张翻坏 |
| skr（合法的取号跳过） | 实例 2 的写行崩在根槽写之前、实例 3 取号越过它，之后写 14 次；last ∈ {6,8,10} | 合法 | 正确写者 45，m4a 45 |

| 格 | 候选 | 判定 | 打中的盘面 |
|---|---|---|---|
| H2 短环来源 | H2-NA | **打中（漏红）** | ss 上 m4a 记账少于同一步正确写者的 132 个盘面，132 个 I-3.1 不适用、池级 checker 全绿 |
| H2 短环来源 | E1（字面） | **打中（误红）** | ss 正确写者有走不到那一代的 180 个盘面里 153 个 I-3.1 红，与今天的判法逐个同数 |
| H2 短环来源 | E1W（攻方宽读法） | **打中（漏红）** | 上面那 132 个记账少的盘面里 17 个判成立、池级 checker 全绿 |
| H2 根槽读坏来源 | H2-NA | **打中（漏红）** | sb 上 m4a 记账少的 48 个盘面，48 个不适用、全绿 |
| H2 根槽读坏来源 | E1（字面） | **打中（误红）** | sb 正确写者翻坏的 54 张 54 张 I-3.1 红 |
| H2 根槽读坏来源 | E1W | **打中（漏红）** | sb 上 m4a 记账少的 48 个盘面 48 个成立、全绿；今天的判法在其中 42 个上同样全绿 |
| H2 | RC、RCW（攻方提的，零轮） | 没打中（一次抽样） | 合法与介质故障各集的正确写者盘面 0 误红；m4a 记账少的 180 个盘面全红；它们的代价与没量到的那一半见第三节 |
| H1（经 I-3.1） | H1-A 只认行 | **打中（误红，条件见下）** | 最新实例写行那条根读坏：nrb 6 / 6、rs1 285 / 285 把存在过的那一版认成取号跳过，H2 取 NA 时 I-3.1 照红 |
| H1（经 I-3.1） | H1-B 并读 | **打中（同上）** | 并读包含行证据，同样 6 / 6、285 / 285 |
| H1（经 I-3.1） | H1-B 朴读 | **打中（误红）** | 另把最新实例自己的记录当成没施加：sb 正确写者翻坏的 54 张 54 张红 |
| H1（经 I-3.1） | H1-C（攻方，零轮） | **打中（误红）** | 那条记录被短环盖掉之后回落到行证据：nrb 短环那 4 张里 3 张红 |
| H1（经 I-3.5） | 全部 | 没打中（一次抽样） | `results/out/` 全部 21072 行判定里 I-3.5 红 0 行（每张镜像在 12 组开关下各判一次） |
| 旁及 | 今天的 I-3.1 | 合法镜像上误红 | ss 正确写者 153 / 180，即 `research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 10 行那一形，今天的代码上照红，且不要回退 |
| 旁及 | 今天的 I-3.9 | 介质故障镜像上红 | rs1 翻坏的 285 张里 150 张（回退复活的单元，回退那一版读不出） |

H1 那几行的条件：误红要 H2 取一个按「存在过却走不到」分类的判法（NA、e1wu）；H2 取 E1W、RC、RCW 时这几张 I-3.1 不按分类判、一张都不红（nrb 6 / 6、rs1 285 / 285 成立）。合法历史（ss、skr）上 H1-A 与 H1-B 并读分类全对：短环来源里行 (1, Ti) 的 Ti 罩住被盖掉的那一版，记成「存在过」；skr 的 45 步一步没认错。

## 一、H2：I-3.1 在走不到的那一代上判什么

### 1.1 走不到的那一代从哪来，今天判成什么

短环来源（合法）：`results/out/ss2-s4-l6-c4-ok.out` 是 S = 4、实例 1 写到 6、每次顺序写 2 MiB。崩的那一次是 txg 7：

```
ss2-s4-l6-c4-ok.out:1: CRASHED txg=7 writes=287 cut_before_root_fua=284
```

实例 2 挂载把 7 由记录施加出来（洞表前三步 `7:AppliedByRecords`），写行 8；之后每次顺序写 65 条记录，1 MiB 的环（256 条）转过之后那条记录被盖掉，7 的根槽从没写过、记录也没了，而实例 2 写的那一行 (1, 7) 说它存在过：

```
ss2-s4-l6-c4-ok.out:74:   NOTE k=4 newest=14 skip=rows i31="e1w" HOLE_TABLE evidence="rows" window=[3,14] non_present=["7:ExistedUnreachable"]
ss2-s4-l6-c4-ok.out:67: JUDGE k=4 newest=14 skip=rows i31="" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(19169280)，遍历全部有效根得到 19038208（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、遍历的候选根槽 11 个、并进遍历) I-3.5=Holds
```

记账多 131072 字节（8 槽，安全方向）：7 那一版独有、在 8 被换下的单元释放代 8，高于窗口里最旧有效根 3，分配器不回收，还算已分配；遍历走的候选根与由记录施加的版本里没有 7。这张盘面上池级 checker 别的不变量都不红，是一段合法历史上的误红。由记录施加那一版今天的认法要那条记录还在 journal 里（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:6782`「&& record.checkpoint_txg <= row.published_checkpoint_txg」是 `applied_by_a_recovery` 的最后一条，前面几条都从 journal 里那条记录读）。180 个有走不到那一代的盘面里今天红 153 个，剩下 27 个是 7 换下的单元已经可回收的那几步。

根槽读坏来源（介质故障）：`results/out/sb-l14-t12-ok.out`，实例 1 写到 14、翻坏 12 那一槽：

```
sb-l14-t12-ok.out:25:   NOTE bad12 skip=rows i31="e1w" HOLE_TABLE evidence="rows" window=[3,14] non_present=["12:ExistedUnreachable"]
sb-l14-t12-ok.out:18: JUDGE bad12 skip=rows i31="" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(2392064)，遍历全部有效根得到 2195456（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、遍历的候选根槽 11 个、并进遍历的由) I-3.5=Holds
```

### 1.2 E1（字面）出局：区间只开在丁-defer 上，碰不到走不到那一代

E1 改的是丁-defer 的门槛，而丁-defer 只并「F_生效 之下、没被抛弃的根」的引用（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:5344`「RootStandingInTheNewestRootsTable::Valid => checkpoint_txg < effective_rollback_floor,」，并进来的槽是这些根走读出来的，`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:5309`「walk_of_the_root_below_the_floor.walk_root(&root.record_bytes, false);」）。走不到的那一代没有根，不在任何一条根的引用里；门槛降到多低，遍历(θ_low) 都等于遍历(θ_high)，区间退化成一个点：

```
ss2-s4-l6-c4-ok.out:72: JUDGE k=4 newest=14 skip=rows i31="e1" reds=["I-3.1"] I-3.1=RED(盘 0：（原型区间形）记账的已分配 Some(19169280) 不在 [19038208, 19038208] 里；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0) I-3.5=Holds
sb-l14-t12-ok.out:23: JUDGE bad12 skip=rows i31="e1" reds=["I-3.1"] I-3.1=RED(盘 0：（原型区间形）记账的已分配 Some(2392064) 不在 [2195456, 2195456] 里；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0 个、) I-3.5=Holds
```

E1 红的盘面与今天逐个同数（ss 153 / 180，sb 54 / 54，rs1 285 / 285，nrb 6 / 6）。E1 是代码轮第一轮为挂载那一刻的读错（K2-e）提的：那一形是写者的门槛比 checker 低，不是遍历少一版，两件事不同形。

四句：分辨臂——分（同一批盘面上 NA、E1W、RC、RCW 都不红）；判别子——看得到（行 (1, 7) 与 journal 里那条记录不在，洞表就认得出「存在过却走不到」）；字面分句——跑前判据第一条前半「一个候选在一段合法历史上让池级 checker 误红」（ss 是合法历史；sb 那一半是介质故障镜像，要不要算合法由主 agent 定）；改法——E1 自己的形状在这几格上修不好，要把区间开到「最新根账里那一代换下的已释放槽」上，就是下面的 E1W，它是另一个候选。

### 1.3 H2-NA 出局：洞里的那一代让记账少的坏镜像整条不判

坏写者 m4a 提前回收最旧那条候选根还引用的槽。没有走不到那一代时今天的判法抓得到它（`results/sum/ss.txt` 与 `sb.txt` 的 no_unreachable 格：m4a 记账少于正确写者的那几步 ss 118 / 118、sb 48 / 48 红，`results/sum/pair-modes-ss.txt`「PAIR no_unreachable short>0 today RED 118」）；有了，NA 整条不判：

```
ss2-s4-l6-c4-m4a.out:67: JUDGE k=4 newest=14 skip=rows i31="" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(19005440)，遍历全部有效根得到 19070976（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、遍历的候选根槽 11 个、并进遍历) I-3.5=Holds
ss2-s4-l6-c4-m4a.out:71: JUDGE k=4 newest=14 skip=rows i31="na" reds=[] I-3.1=NA(原型 HOLE_I31=na：洞表里有存在过却走不到的那一代) I-3.5=Holds
```

这一张记账比遍历还少 65536 字节，是明着的不安全方向，NA 照样全绿。配对数（`results/sum/pair-modes-ss.txt`、`pair-modes-sb.txt`，同参数同一步、m4a 盘 0 的已分配少于正确写者算「记账少」）：

```
PAIR unreachable short>0 na NA 132
PAIR unreachable short>0 na NA 48
```

四句：分辨臂——分（同一批盘面上 RC、RCW 132 / 132、48 / 48 红）；判别子——看得到（RC 从最新根账里的记录就算得出记账该是多少）；字面分句——跑前判据第一条后半「或在一张记账少（不安全方向）的坏镜像上漏红」；改法——NA 这一形没有可调的旋钮，「这一代所在的那一对判不适用、整条照判其余」在 I-3.1 上没有「那一对」可切，I-3.1 是逐盘一个标量。

### 1.4 E1W（攻方宽读法，零轮）出局：坏写者少记的不超过那一代的槽数时区间吞掉它

E1W 的上沿把「最新根账里释放代高于 θ_low、遍历没走到的已释放槽」算进来，合法盘面上它们正是走不到那一代换下的单元，所以 ss 180 / 180、sb 54 / 54、rs1、nrb 全成立。漏的是：记账少于真值的量 ≤ 那一代的槽数 D 时，记账落在 [遍历, 遍历 + D] 里。短环上 17 个，例（同参数同一步的正确写者已分配 14860288，m4a 少 1 槽 16384 字节，遍历两边相同）：

```
ss2-s4-l4-c3-m4a.out:108:   NOTE k=6 newest=13 skip=rows i31="e1w" I31 device=0 allocated=Some(14843904) walked=14761984 deferred_high=0 proto=Interval { upper: 14860288 }
ss2-s4-l4-c3-ok.out:108:   NOTE k=6 newest=13 skip=rows i31="e1w" I31 device=0 allocated=Some(14860288) walked=14761984 deferred_high=0 proto=Interval { upper: 14860288 }
ss2-s4-l4-c3-m4a.out:105: JUDGE k=6 newest=13 skip=rows i31="e1w" reds=[] I-3.1=Holds I-3.5=Holds
ss2-s4-l4-c3-m4a.out:101: JUDGE k=6 newest=13 skip=rows i31="rc" reds=["I-3.1"] I-3.1=RED(盘 0：（原型 rc 记录形）记账的已分配 Some(14843904) 不在按记录算的 [14860288, 14860288] 里，或遍历走到的槽里有 0 个不在记账该算的记录里；机理：根环槽数 12、最新根 txg 13、环里自证过的根槽 11 个、最老的自证过的根 txg) I-3.5=Holds
```

根槽读坏那一集上更整齐：m4a 少记的恰好等于 12 那一代的槽数（都是 12 槽），记账与遍历相等，今天的等式、NA、E1W 三个都绿，只有 RC、RCW 红：

```
sb-l14-t12-m4a.out:2: JUDGE clean skip=rows i31="" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(2195456)，遍历全部有效根得到 2392064（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 12 个、最老的自证过的根 txg 3、遍历的候选根槽 12 个、并进遍历的由) I-3.5=Holds
sb-l14-t12-m4a.out:18: JUDGE bad12 skip=rows i31="" reds=[] I-3.1=Holds I-3.5=Holds
sb-l14-t12-m4a.out:24: JUDGE bad12 skip=rows i31="e1w" reds=[] I-3.1=Holds I-3.5=Holds
sb-l14-t12-m4a.out:20: JUDGE bad12 skip=rows i31="rc" reds=["I-3.1"] I-3.1=RED(盘 0：（原型 rc 记录形）记账的已分配 Some(2195456) 不在按记录算的 [2392064, 2392064] 里，或遍历走到的槽里有 0 个不在记账该算的记录里；机理：根环槽数 12、最新根 txg 14、环里自证过的根槽 11 个、最老的自证过的根 txg 3、) I-3.5=Holds
```

四句：分辨臂——分（RC、RCW 在这 17 + 48 个上红）；判别子——看得到（同上，记录在盘上）；字面分句——第一条后半；改法——E1W 的下沿还是遍历，换成按记录算的下沿（RC）就修到这几格。另一个窄的变体 `e1wu`（只在有「存在过却走不到」时放宽）在 ss 上与 E1W 同数，并且继承 H1 的分类错（第二节）。

### 1.5 RC、RCW（攻方提的，零轮）：没打中，代价与没量的那一半

RC 不拿遍历定下沿，拿最新根账里的记录定：记账该算已分配的 = 已分配记录 + 释放代高于门槛的已释放记录，门槛取 [θ_high, θ_low] 两端；另加逐槽一判，遍历走到的每个槽都要在 E(θ_high) 那一份里（`hole_proto.rs` 的 `proto_counted_slots` 与 `proto_i31_per_device` 里 `mode == "rc" || mode == "rcw"` 那一段）。洞里那一代换下的单元在账里是释放代高于门槛的已释放记录，算进 E，所以有洞没洞同一个式子，不用先分类。

量过的（`results/sum/*.txt`、`pair-modes-*.txt`）：正确写者盘面 ss 432、sb 108、rs1 570、nrb 12、skr 45 上 I-3.1 一次不红；m4a 记账少于正确写者的盘面（有洞 132 + 48，没洞 118 + 48）全红。ss 上另有 2 个 m4a 盘面 RC 红而盘 0 的已分配不少于正确写者（`PAIR ... short<=0 rc RED 1` 两行），配对只比了盘 0，盘 1 或遍历落点不同没核，坏写者盘面上的红不算误红。

没量的（推的）：
- RC 丢了泄漏方向那一半：写者忘了把一条已分配记录标成已释放（没有任何版本引用的已分配记录），E 照样数它，记账 = E，RC 绿，今天的等式红（记账多于遍历）。RCW 的上沿再与「遍历 + 释放代高于 θ_low、没走到的已释放槽」取小，这一形照红。我没造这个坏写者。
- RC 认定写者的门槛就是 checker 算的 θ_high（下沿 E(θ_high)）。写者合法地用了更低的门槛、盘上又看不出来时（挂载那一刻的读错，代码轮第一轮 K2-e；今天由 E2 的重读收住），RC 会红在记账多那一边，与今天同。θ_low 只在洞表里有洞的位置时才降。
- 隔离豁免（D19 已定项 5）的记录今天的等式按单元豁免；RC 把所有已分配记录都数进 E，隔离那几条不必另豁免，但「核出坏的才豁免」那一判在 RC 下没有对象了，我没核这一层的口径。
- 代价：每块盘多读一遍最新根的分配记录树（丁-defer 与隔离那两遍今天已经各读一遍同一棵树，冻结副本 `walk.rs` 第 5286、5438 行各有一处 `allocation_record_tree_without_judging`），按已有的缓存是同一批节点，没量耗时。

## 二、H1：「取号跳过」认什么证据

### 2.1 行证据把最新实例自己存在过的那一版认成取号跳过（H1-A、H1-B 并读）

历史（nrb）：实例 1 写到 10 → 实例 2 可写挂载，写行那次发布是 txg 11 → 再写几次 → 11 那一槽读坏。最新根的实例表只有 (1, 10)；最新实例自己没有行，行里没有一个字说 11 是实例 2 写过的。候选集里相邻的是 (实例 1, 10) 与 (实例 2, 12)，行证据与 gap-row 的写法一样认成实例 2 直接接在 10 那一版上（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:475`「row.instance == previous_instance && row.published_checkpoint_txg == previous_txg」，R 的实例大于 R⁻ 的实例那一条在 `/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:470`「if version_instance <= previous_instance {」），于是 11 记成取号跳过：

```
nrb-l10-k3-b1.out:1: BUILT instance=2 row_txg=11 newest_txg=16 bad_writer=None
nrb-l10-k3-b1.out:25:   NOTE bad[11] skip=rows i31="e1w" HOLE_TABLE evidence="rows" window=[5,16] non_present=["11:Skipped"]
nrb-l10-k3-b1.out:22: JUDGE bad[11] skip=rows i31="na" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(2129920)，遍历全部有效根得到 2031616（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 16、环里自证过的根槽 11 个、最老的自证过的根 txg 5、遍历的候选根槽 11 个、并进遍历的由) I-3.5=Holds
nrb-l10-k3-b1.out:30: JUDGE bad[11] skip=union i31="na" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(2129920)，遍历全部有效根得到 2031616（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 16、环里自证过的根槽 11 个、最老的自证过的根 txg 5、遍历的候选根槽 11 个、并进遍历的由) I-3.5=Holds
```

11 那一版存在过（它换下的单元在账里是释放代高于门槛、遍历没走到的已释放记录：E1W 那一组把它们数进上沿，正好对上，见同一份输出 e1w 那一组的 NOTE），I-3.1 却按「没有这一版」判，记账多 98304 字节，红。rs1 那 285 段（回退 + 实例 2 在 R 之前的根全读不出）同一形，285 / 285。H1-B 并读包含行证据，同红；它多认的「journal 里那条记录」在这里说的是「最新实例自己的记录」，按并读不算没施加，不会把分类改回来。

四句：分辨臂——不分 H1 的两个候选（H1-A、H1-B 并读同红；H1-B 朴读更糟，见 2.2），病根是两者共用的前提「从行与 journal 记录能分出存在过与取号跳过」；分得出 H2 的候选（H2 取 E1W、RC、RCW 时这几张都不红）。判别子——行与（被盖掉之后的）journal 都看不到，账里的已释放记录看得到（RC 就是读它）。字面分句——第一条前半，但这几张都是介质故障镜像，算不算合法历史由主 agent 定。改法——H1 的两个候选都修不到这一格；H1-C 只在记录还在时修到（2.3）；把 I-3.1 改成不靠分类的判法（E1W、RC、RCW）就碰不到这一格。I-3.5 那一侧这一形不红（2.4）。

### 2.2 H1-B 朴读：把最新实例自己的记录当成没施加

「施加过」只按实例表行认（与 `versions_applied_only_by_records` 那一判同一个取法，`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:6782` 那一条），最新实例没有自己的行，它写过的每一条记录都成了「已落、没施加」，根槽读坏的那一版全被认成取号跳过：sb 正确写者翻坏的 54 张 54 张 I-3.1 红（`results/sum/sb.txt`「COUNT ok unreachable naive/na i31=red 54」）。H1-B 要取，「没施加」只能对比最新实例旧的实例的记录说。

### 2.3 H1-C（攻方变体，零轮）：记录被短环盖掉之后回落到行证据

H1-C 让「journal 里那条记录说 t 存在过」压过行证据，nrb 前三张转成不适用；把同一形放到短环上（每次顺序写 2 MiB，写行那条记录在挂载之后第 4 次写时被盖掉），回到 2.1：

```
nrbs-l4-k4-b1.out:1: BUILT instance=2 row_txg=5 newest_txg=11 bad_writer=None
nrbs-l4-k4-b1.out:25:   NOTE bad[5] skip=rows i31="e1w" HOLE_TABLE evidence="rows" window=[0,11] non_present=["5:Skipped"]
nrbs-l4-k4-b1.out:32: JUDGE bad[5] skip=counter i31="na" reds=["I-3.1"] I-3.1=RED(盘 0：记账的已分配 Some(12189696)，遍历全部有效根得到 12091392（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 11、环里自证过的根槽 11 个、最老的自证过的根 txg 0、遍历的候选根槽 11 个、并进遍历) I-3.5=Holds
```

k = 4、5、6 三张都红。所以「journal 环已被后来的记录覆盖的那一形」：两个 H1 候选与 H1-C 都退回到行证据，行证据在最新实例的早几条根读坏时认错。

### 2.4 I-3.5 那一侧：误认取号跳过没造出误红，也没造出漏红

误认取号跳过只会让一对从 N3（用户可见树里的单元不判）变成照判，照判更严、不会少判；要误红得让洞右边那条根引用一个诞生代号不大于 R⁻.txg、又不在整棵继承下来的子树里的单元。我让 rs1 在被认错的那几版里放一次管理员回退（目标 txg 5..9），回退之后的用户动作长度 0..2 放开扫（四种动作的全部 21 种组合），洞右边第一条读得出的根 R 取回退那一版到最新那一版之间每一条：285 段、每段在 12 组开关下各判两张，I-3.5 一行没红。原因（按代码推的）：回退那一版整棵带回目标的用户可见树，(b) 放过；之后的顺序写、覆盖写把数据单元整个重写，新单元的诞生代号大于 R⁻.txg，(a) 放过；建 inode 只重写 inode 叶。造得出误红的大概要「回退之后只改大文件的一部分单元」，`publish_sequential_write` 与 `publish_overwrite` 都是整份内容重写，我没找到只改一部分的入口，没造。

反方向（H1-B 多认 journal 证据让一对从 N3 变成照判、多抓一类坏镜像）：合法历史里行证据认不出、journal 认得出的取号跳过我没找到。同一实例里中间跳号已经没有来源：落盘中途失败的发布冻结起来、重发用同一个 txg（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-core/src/allocator.rs:1820`「失败的发布连这张表一起退回、重发用同一个 txg。」），I-3.5 射程里列的「根槽写失败重发跳一格」在今天的代码上不出现（`.claude/kb/invariants.md:146`「另有恢复丢掉尾巴记录、根槽写失败重发跳一格、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）」那一句写着它）。skr（合法的取号跳过，R⁻ 有时已被环盖掉）45 步上 H1-A 一步没认错。

### 2.5 checker 读不读得到那条记录，D13 已定项 5

读得到，而且是自己的一份：池级 checker 只依赖 singlefs-format（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/Cargo.toml:9`「singlefs-format = { path = "../singlefs-format" }」），journal 环它自己扫（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:6167`「fn scanned_journal_records_by_device(」），按恢复的读法取每个计数器第一块盘上那一份（`/tmp/claude-1000/m3-candidate-hole-r1/tree/crates/singlefs-checker/src/walk.rs:6772`「let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);」）。H1-B 要的「已落、没施加」用的就是这几样，不用新读法；D13 已定项 5 的那一句（`.claude/kb/decisions/13-验证路线.md@已定项 5：只共享一份从 kb 生成的常量`「checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块」）不挡它。被盖掉的那一形 checker 读到的是盖它的那条记录，没有别的地方能找回原来那条；能找回「那一版存在过」的只有账里它换下的单元的已释放记录（RC 读的那一份）。

## 三、几个改法各修哪一格

全部只在我的模型上量过、被攻过零轮（E1 是代码轮第一轮攻方提的，同样零轮）。「量过」的格数取自 `results/sum/`；「推的」是按代码推、没实现或没跑。

| 打中的格 | E1 | E1W | H2-NA | RC | RCW | H1-C |
|---|---|---|---|---|---|---|
| ss 合法盘面误红（今天 153 / 180） | 不修（量过：153 红） | 修（量过：0 红） | 修（量过：0 红，整条不适用） | 修（量过：0 红） | 修（量过：0 红） | 不涉及 |
| sb 介质故障盘面误红（今天 54 / 54） | 不修（量过：54 红） | 修（量过） | 修（量过） | 修（量过） | 修（量过） | 不涉及 |
| m4a 记账少、有洞（ss 132、sb 48） | 红（量过，但它连合法盘面一起红） | 漏 17 + 48（量过） | 漏 132 + 48（量过） | 全红（量过） | 全红（量过） | 不涉及 |
| H1 误认取号跳过（nrb 6、rs1 285） | 红（量过，同今天） | 不红（量过） | 红 6、285（量过，按 H1-A） | 不红（量过） | 不红（量过） | 修 3 / 6、285 / 285（量过；记录被盖掉的 3 张照红） |
| 泄漏方向的坏写者（忘了标已释放） | — | 红（推的） | 有洞时不判（推的） | **漏**（推的） | 红（推的） | — |

另试的两个形状：`e1wu`（E1W 只在有「存在过却走不到」时放宽）在 ss、sb 上与 E1W 同数，在 nrb、rs1 上随 H1-A 的分类错一起红（量过：`results/sum/nrb.txt` 6 红）；`e1ws`（E1W 加「遍历走到的槽都要有一条记录」）一张都没多抓：m4a 回收之后记录留在树里（已回收、还没被复用的已释放记录照留），有没有记录分不出回收过没有（量过：`results/out/notes-sb-l14-t12-m4a.out` 第 9 行与第 72 行，坏写者那张没洞的与翻坏的两张都是「  NOTE bad12 skip=rows i31="e1ws" I31_PROTO_SLOTS walked_slots_without_a_record={}」这一形，前一张记账少 12 槽照样一个都没有）。

## 四、旁及（不在攻击面里，交主 agent）

- 今天的 I-3.1 在 ss 这段合法历史上红（153 / 180），即 `research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 10 行那一形；它不要回退、不要介质故障，mkfs 收的最小 journal 环加大文件顺序写就够。H2 不管取哪一个，今天的实现都该先当成「checker 在合法镜像上红」处理。
- I-3.9 在 rs1 翻坏的 285 张里红 150 张，原样（`results/out/rh-i39-t5-over-upto15.out` 第 22 行）：

```
  SHOW badupto:15 I-3.9 Violated("盘 1 槽 1096 的分配记录释放代 15 不在 (5, 6] 里：还引用它的最新一条有效根是 txg 5，最早不再引用它的是 txg 6")
```

  回退把 txg 5 那一版的单元复活，复活它的那一版（14）读不出，checker 算的区间少了中间那一段。I-3.9 也要读同一张洞表（H3 的射程），这一格我没往下查。
- I-3.5 射程里「根槽写失败重发跳一格」那一个来源在今天的代码上不出现（2.4），条款那一句可以回看。

## 五、没打中的形状

- I-3.5 误红：rs1 285 段（回退目标 5 个 × 回退之后的用户动作 0..2 步全组合 21 种 × 洞右边第一条根的位置 1..3 个），12 组开关，0 红。只在 S = 4、1 MiB journal、3000 字节内容、单个文件上试过。
- I-3.5 漏红（H1-B 多认 journal 证据才抓得到的坏镜像）：没找到行证据认不出、journal 认得出的合法取号跳过，没造坏镜像。
- 合法的取号跳过（skr，last ∈ {6,8,10}，挂载之后写 14 次，含 R⁻ 被环盖掉的几步）：H1 的各候选都没认错，各 H2 候选 0 误红。
- RC、RCW 在全部集上 0 误红、坏写者全抓：一次抽样，坏写者只有 m4a 一种。

## 六、这条腿自己的限度

- 「合法历史」只有 ss 与 skr 两集是：sb、nrb、rs1 都靠把根槽翻一个字节（介质故障）。H1 那几格的打中全在介质故障镜像上；它们算不算判据第一条的「合法历史」我没定。`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 3 行当时把根槽读坏的盘面判成「池级 checker 先红，不算合法镜像上的红」，那时先红的就是这里的 I-3.1。
- 坏写者只有 m4a 一种（`crates/mutations.tsv` 第 148 行那一改法，经副本 core 的开关打开）；「记账少」用同参数同一步的正确写者的盘 0 已分配来认，盘 1 没比。泄漏方向的坏写者没造（第三节末行全是推的）。
- 洞表是我照调查员第十节写的一份（`walk-patch/hole_proto.rs` 的 `proto_hole_table_store`），不是正推腿或实现员的；它的「取号跳过」行证据那一判先写成「R⁻ 必须在盘上」，在 skr 上把窗口前沿那一格认成「存在过」（3 步），改成第一节那张表里写的「t 之下最近的锚不晚于 Ti」之后 0 步认错，最后一遍复跑是改过的。
- 只跑了单个文件、一个池两块盘、R = 3；S 只取 4、5、6；ss 每次顺序写 1–2 MiB（一次发布 85 条记录的上限挡住 3 MiB）。
- 崩溃状态：ss、skr 各用一个构造出来的崩溃点（根槽写之前的整段前缀，是层 0 枚举域里的一个状态），没有调 `enumerate_layer0` 把那次发布的崩溃状态逐个展开；这一轮原型跑的层 0 状态数为 0。
- 挂钟：sb 108 件约 21 秒，rs1 285 件约 1 分钟，ss 54 件分五次共约 9.5 分钟（8 件并行），全量没超过 40 分钟，没缩取样。
- RC、RCW、E1W、H1-C 都是我提的，只在我的模型上量过、被攻过零轮；E1W 是我对 E1 的一种宽读法，E1 的提出者未必认它。
- 「没打中」各格都只抽了一次样。

## 七、要主 agent 跑的命令

没有。全部原型都在前台跑完了；没有要放后台的长活，也没有要跑的重型测试。复跑命令在报告开头。

## 八、没做什么

- 没判 H3（三条判法共用一张洞表），只在第四节记了 I-3.9 也要读洞表那一处。
- 没判正推腿那几格；没读禁读清单里的文件。
- 没改冻结副本与主仓的 `crates/`；补丁只打在 `/tmp/claude-1000/m3-candidate-hole-r1-opus/tree/` 上。
- 没跑任何 `cargo test` 目标、checker 档、门禁、变异表。
- 草稿目录 `/tmp/claude-1000/m3-candidate-hole-r1-opus/` 里的仓副本 `tree/` 与编译目录 `target/` 交回前删掉；`out/`、`sum/` 与 `results/` 里的同名文件相同（`results/out/` 是从 `out/` 拷的），一起删。
