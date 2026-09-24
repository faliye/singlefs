# m2-refusals-presumed-r1 云端攻方腿（Opus）：R1、R2、R3、R4、Q1

立场：找误拒。2026-09-22 21:56 UTC（2026-09-23 06:56 JST）跑完。
判据照正文第五节第 2 条：**一格算「误拒」，当且仅当给出了一条具体的历史（写序列 + 故障点），而且它在 `crates/` 今天的代码上真的会撞上那个拒绝**。

## 零、复跑

仓副本（`rsync -a --exclude target --exclude .git`，实际连 `target/debug` 一起带过去了，见「这条腿自己的限度」）在
`/tmp/claude-1000/m2-refusals-opus/repo`。**下面的数全是副本上量的，不是入库装置上的数。**

```
# 未打补丁（mount.rs 与仓里一字不差）：五条探针
cp research/prompts/m2-refusals-presumed-r1-opus-model/opus_attack_refusals_r1r2r3q1.rs <副本>/crates/singlefs-harness/tests/
cd <副本> && nice -n 19 cargo test -p singlefs-harness --test opus_attack_refusals_r1r2r3q1 -- --nocapture --test-threads=1
# 打补丁（攻方腿自己提的两个改法，被攻过零轮）：
cd <副本> && patch -p0 crates/singlefs-core/src/mount.rs < .../mount-rs-opus-fixes.patch
cp .../opus_patched_refusals.rs <副本>/crates/singlefs-harness/tests/
nice -n 19 cargo test -p singlefs-harness --test opus_patched_refusals -- --nocapture --test-threads=1
OPUS_SKIP_R1=1 nice -n 19 cargo test -p singlefs-harness --test opus_patched_refusals -- --nocapture --test-threads=1 opus_r1_fix
OPUS_SKIP_R1=1 OPUS_SKIP_R4=1 nice -n 19 cargo test -p singlefs-harness --test opus_patched_refusals -- --nocapture --test-threads=1 opus_r1_fix
```

模型目录 `research/prompts/m2-refusals-presumed-r1-opus-model/`，这一轮新加的七个文件的 `sha256sum`：

```
10953e3d58eba1f87e8892bf65f0471fee71d52a86ad37ed6371aee5e8e1fd99  opus_attack_refusals_r1r2r3q1.rs
e3bfd53032602f82bbdee73441f9068256bd875973637d998ea80e7b24893235  opus_patched_refusals.rs
332ed2bb210a0fbac45c98dd46a18d2568109f1793675a7c2da22d3f4df8ebbb  mount-rs-opus-fixes.patch
7f790255c8a8c810ef108a66eeeccb5e9720d0c82a7c4ba6f68e8e3cce310f3d  baseline-run.log
36ff5b30b797f04d4e6e295bb3bc0c6c2d47eee95b01af19d3158bd3c351ef00  fix-run-plain.log
693dc3ef9990a21f0766827a99dd5abe62098086b06ace87b264faafa018034d  fix-run-skipr1.log
37220cb62368671a3824c65a03f666c7de447da2b7cfb87ab9fe73d88b192c9b  fix-run-skipr1r4.log
```

⚠️ **这个目录里另有七个不是我写的文件**（`opus_m2_refusals_r1.rs`、`raise-reads-table-from-root.patch`、
`two-refusals-off.patch`、`run7-final-unpatched.txt`、`run8-patchA-probe.txt`、`run9-patchB-probe.txt`、
`run10-patchB-stepfive.txt`，时间戳都是 2026-09-22 13:59）。派发提示说上一条腿「产物没落盘」，而这七个在盘上。
我按指示没读它们、也没动它们；主 agent 要注意这个目录今天混着两条腿的产物。

## 一、行号现查（正文的行号是 2026-09-22 上午记的，这一轮全部重取）

| 正文写的 | 现查（2026-09-22 21:56 UTC 工作区） | 差 |
|---|---|---|
| `FileVersionWithoutAnyJournalRecord` 37 / 200 / 1270 | `mount.rs:38`（成员）/ `:226`（`map_rebuild_failure`）/ `:1310`（`mount_rollback`） | 挪了 |
| `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion` 43 / 622 | `mount.rs:44`（成员）/ `:660`（`raise_rollback_floor` 里那一句） | 挪了 |
| `VersionWithoutFileNotWrittenByMakeFilesystem` 71 / 457 | `mount.rs:72`（成员）/ `:487`（`format_time_allocator`，函数头 `:480`） | 挪了 |
| `FormattedPoolMountNotShapedLikeTheFirstTransaction` 118 / 863 | `mount.rs:128`（成员）/ `:901`（判定函数 `:885`） | 挪了 |
| `InstanceRowsOnVersionWithoutFileUnsupported` | `mount.rs:65`（成员）/ `:868`（判定函数 `:861`） | — |
| `RollbackToVersionWithoutFileUnsupported` | `mount.rs:57`（成员）/ `:1300`（判定 `:1299`） | — |
| `mount_rollback` 1255 | `mount.rs:1244` | 挪了 |
| `format_time_allocator` 450 | `mount.rs:480` | 挪了 |
| `FirstFileVersionNotRightAfterTheSecondWarmUp` 1072 / 1392 / 1413 | `transaction.rs:1149`（成员）/ `:1469`（文档）/ `:1490`（返回） | 挪了 |
| `RegionDevicesNotTheFirstVersionLayout` 81 / 171 | `make_filesystem.rs:86` / `:176` | 挪了 |

三条**不是挪行、是内容变了**的观测：

1. **`format_time_allocator` 里那个 `assert_eq!(…, "两盘同槽")` 已经不是 panic 了**：今天是 `mount.rs:82` 的新错误成员
   `FormatTimeUnitLocationsOnDifferentSlots { unit, disagreement }`（`mount.rs:496-500`）。正文列的八个拒绝里没有它；
   `MountError` 今天共 **18** 个成员（`awk '/^pub enum MountError/,/^}/' … | grep -cE "^    [A-Z][A-Za-z]*(\(|,| \{)"`）。
2. **kb 里点名的一个错误成员在 `crates/` 里零命中**：`.claude/kb/milestone/02-second-txn.md:195` 逐字「没做过可写挂载的进程抬 F 报
   `RaiseNeedsWritableMountInThisProcess`（`raising_the_floor_in_a_process_that_never_mounted_writable_is_refused_instead_of_panicking`）」，
   而 `grep -rn "RaiseNeedsWritableMountInThisProcess" crates/` **0 命中**，那个用例名 `grep -rn … crates/` 也 0 命中；
   今天这一格的成员是 `RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`、用例是
   `second_transaction_step_five_reuse.rs:615` 的 `raising_the_floor_on_a_current_version_without_a_rewritten_instance_table_unit_is_refused_instead_of_panicking`。
   ⇒ 判 Q1 用的是**后者**；kb 步 5 现状那一句已经腐了（那份 kb 在工作区里是别的会话未提交的改动，我不碰）。
3. **mkfs 今天把 journal 环与根环三段一起清零**（`make_filesystem.rs:3` 的写序注释、`:10` 逐字「根环连着一起清是同一天的续作
   （C484…，用户 2026-09-22 定案）」、`:260-262` 的 `write_zeroes_at`）。而 `.claude/kb/checks-owed.md:429` 的 C484 行文仍写
   「并行线四 2026-09-22 让 mkfs 整环清零 journal 环……**根环不清**」。我的一条探针（见「没打中的形状」E）实测同 fsid 重做 mkfs 之后
   环里旧记录 **0 条**，与代码一致、与 C484 行文不一致。

## 二、各格判定一览

| 格 | 误拒？ | 拒得早 | 拒得准 | 拒得明 | 结论（我这条腿的） |
|---|---|---|---|---|---|
| **R1** 树表 0 条的一版上要写实例表行 | **是**（两条历史：零故障一条、一次普通崩溃一条，都实测撞上） | **过**（`DiskSnapshot` 逐字节相等、录制流 0 步） | **不过**（被拒的历史零故障可达，且拒完之后这个池**没有任何一条可写的出路**） | 过（名字说的就是它拒的那件事） | **空白，该写条款**：条款要同时答「树表 0 条的一版上行写在哪」与「不写行行不行」，两种选法在盘上分得开 |
| **R2** 回退到树表 0 条的根 | **是**（零故障，实测） | **过**（同上） | **不过**（D23 已定项 14 的候选集条文逐字**允许**这个目标，实现按候选集之外的理由拒） | 过（但「不报候选排除」这一条今天是对的） | **空白，该写条款**：条文与实现今天说反话，要么条文加一句排除、要么实现兑现 |
| **R3** 所选根有文件而环里一条记录都读不出 | **否**（零故障 / 一次普通崩溃都构造不出；要 ≥ 2 次介质故障） | 过（代码路径上在取号与任何写之前） | **存疑**（拒的前提不承重：随便哪条无关记录都放行，而记录全没了时只读挂载照样把文件读回来） | 过 | **登记成第一版不支持**（不该写条款），另记一笔「这道前提可以拆掉」的欠账 |
| **R4** 只做过 mkfs 的池可写挂载只兑现第一次 | 这个**错误成员**零故障够不着（被 R1 那道挡在前面）；但「只兑现第一次」本身是**实现缺口**，不是条款允许的 | 过 | 不适用（成员本身零故障不可达） | 过 | **实现缺口 + 一处条款空白**：`FIRST_TRANSACTION_TXG = 3` 的**射程**没有条款（只管 mkfs 那条流，还是管任何池的第一个文件版本），两种读法在盘上分得开 |
| **Q1** 抬 F 时现行版本里没有实例表单元 | **是**（零故障，实测：mkfs 同一个进程里再覆盖写三次照样被拒） | **过** | **不过**（同一张表从现行根的指针上读得出、解得开；改法之后抬 F 一次成功） | **不过**（名字说「现行版本里没有」，实际条件是「`TransactionOutput::units` 这个内存数组里没有」） | **登记成实现细节 + 一笔欠账**：不立条款，改代码；今天不伤用户只因为产品路径还没有调用点 |

## 三、R1：树表 0 条的一版上要写实例表行（`InstanceRowsOnVersionWithoutFileUnsupported`）

### 3.1 打中的历史

**历史 R1-α（零故障，写序列里一个故障点都没有）**

1. `mkfs`（两块盘，参数同 E142 装置）。
2. 可写挂载一次：取号 1、不写行（区间空）、零单元写行发布 txg 1、暖机 txg 2。**做成了。**
3. **进程正常退出**，镜像关掉。没有崩溃、没有坏盘、没有掉电。
4. 再可写挂载：撞 `InstanceRowsOnVersionWithoutFileUnsupported`。此后每一次都一样。

原样输出（`baseline-run.log`，`opus_r1_second_writable_mount_of_a_formatted_pool_is_refused_and_no_rollback_target_is_an_exit`）：

```
[R1] 第一次可写挂载成功：实例 1，写行那次 txg 1，暖机 1 次
[R1] 第二次可写挂载：Some(InstanceRowsOnVersionWithoutFileUnsupported { chosen_root: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) }, first_row_instance: InstanceGeneration(1), instance_to_acquire: InstanceGeneration(2) })
```

代码上为什么必中：`mount.rs:861-878` 的 `refuse_instance_rows_on_version_without_file` 判
`PreviousVersion::WithoutFile(_) if first_row_instance < instance_to_acquire`；这条历史上
`first_row_instance = max(1, 1) = 1 < 2 = max(系统配置 1, 根环 1) + 1`（`instance_generation_to_acquire`，`mount.rs:1051`）。

**历史 R1-β（一次普通崩溃，且这次崩溃落在「第一次可写挂载还没做完」的窗口里）**：这一格仓里已有用例
（`crates/singlefs-harness/tests/second_transaction_step_three_formatted_pool.rs:115`
`writable_mount_after_a_crash_right_after_acquiring_an_instance_is_refused_before_acquiring_again`），
我复跑确认它今天仍绿、结局与 R1-α 同一个成员。**⇒ 这道拒绝不是「崩溃之后才撞得到」的边角：第一次可写挂载从取号写完那一刻起，
到它的第一条根持久为止，整段窗口里的任何一次掉电都把这个池永久挡在可写挂载之外**（取号之后崩 ⇒ 所选根 (0,0)、要取 2；
写行根落了之后崩 ⇒ 所选根 (1,1)、要取 2；暖机根落了之后崩 ⇒ 所选根 (1,2)、要取 2——三格都满足 `1 < 2`）。

### 3.2 三关

- **拒得早：过**。`DiskSnapshot`（两块盘四个系统配置槽的原样字节 + 环里全部可读根 + 录制流步数）在拒绝前后**逐字节相等**，
  用例里是 `assert_eq!(disk_snapshot(...), before, "[R1] 拒得早：一个写都没发")`，跑绿。判定点 `mount.rs:1054` 在取号
  （`mount.rs:1063`）之前。
- **拒得准：不过**。被拒的那一族里有零故障可达的历史（R1-α），而它挡掉的操作是「挂载我刚建好的文件系统」——没有比这更正当的理由。
- **拒得明：过**。成员名 + 三个字段（`chosen_root`、`first_row_instance`、`instance_to_acquire`）说得出拒的是哪件事，
  与 `RollbackToVersionWithoutFileUnsupported`（回退那条路）分得开。

### 3.3 这条腿新加的一件：拒完之后这个池**没有出路**

同一份镜像上，把根环里**每一条**可读根都当回退目标试一遍（`readable_roots` 给 3 条），全被拒：

```
[R2] 环里可读根 3 条
[R2] 回退到 (0, 0)：Some(RollbackToVersionWithoutFileUnsupported(RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }))
[R2] 回退到 (1, 1)：Some(RollbackToVersionWithoutFileUnsupported(RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(1) }))
[R2] 回退到 (1, 2)：Some(RollbackToVersionWithoutFileUnsupported(RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) }))
[R1] 砖住之后的只读挂载：ok = false，err = Some(Open(Walk(UnitUnreadable { slot: SlotNumber(0) })))
[R1] 对照：刚 mkfs、没挂载过的池只读挂载：ok = false，err = Some(Open(Walk(UnitUnreadable { slot: SlotNumber(0) })))
```

`crates/` 今天写盘的入口只有两个（`mount_writable`、`mount_rollback`，`mount.rs:1165` / `:1244`），两个都关着。
**⇒ R1 与 R2 不是两条各自独立的「第一版不支持」，它们合起来把这个状态的出路清空了。**
只读挂载（`mounted_read.rs:515`）在这个池上也报错，但**对照臂（一次都没挂载过的、刚 mkfs 的池）报同一个错**
⇒ 那一条是「空池本来就没有文件可读」，不是这道拒绝造成的，我不把它算进这道拒绝的账。

代价的量：这个池里**没有用户数据**（树表 0 条），所以「砖住」的代价是「这个 fsid 的池只能重做 mkfs」，不是数据丢失。
我不夸大它；但「正常关掉一次之后再也挂不上」这件事本身与
`.claude/kb/decisions/16-发布语义.md:39`（D16 已定项 1「准入」那一行）以下的整套承诺不是一个量级的问题——它是**可用性**层面的。

### 3.4 打中之后的四句（evidence-discipline「打中之后先判是哪一种」）

1. **分不分辨臂**：分得开。同一份代码、同一条装置，只差「第一次挂载之后退不退出进程」这一步：不退出、同进程里接着发第一个文件版本 ⇒
   一路绿（仓里 `second_transaction_step_three_formatted_pool.rs` 那条流）；退出再进 ⇒ 撞拒绝。**装置没有替我造出这个分辨点**：
   「退出进程」是用户动作，不是我写死的故障。
2. **被判的系统当时看不看得到判别它的东西**：看得到。`instance_to_acquire`、`first_row_instance`、所选根都在它手里
   （拒绝的三个字段就是它们），它是**知道**自己在拒什么的；缺的不是输入，是「这一版上行写在哪」的条款。
3. **满足的是判据字面的哪一个分句**：第五节第 2 条的两个分句都满足——「给出了一条具体的历史（写序列 + 故障点：R1-α 是零故障，
   R1-β 是一次掉电）」且「在 `crates/` 今天的代码上真的撞上那个拒绝」（原样输出在上面）。
4. **跑前条款给的改法在打中的那几格上还中不中**：正文没有给 R1 的改法；里程碑 141 行给的处置是「要开决策」。
   我自己提的 R1-F1（见第七节）把 R1-α 那一格从「拒绝」变成「挂载做成、写 0 行」，但**同一条历史随即撞上 R4 那道拒绝**
   （实测，见第五节）⇒ 只改 R1 这一处不足以把这条历史救出来。

### 3.5 条款侧

- 拒的理由（`mount.rs:65-70` 的文档）是「写行要重写实例表，没有文件版本的一版上它的落点记在哪没有条款」，
  引的是 `.claude/kb/decisions/16-发布语义.md:207` 已定项 9 逐字「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元；
  以后的空发布按 D28（挂载期承诺量） 已定项 4 现算的 c_max 块」。**这句话只说了空发布写零个单元，没说「要写行时怎么办」——
  它支持「这里有空白」，不支持「空白只能用拒绝填」。**
- 第三条出路今天没人提过：**不写行**。理由是可查的：树表 0 条的一版上，每一个更早的实例写出的单元数是 0
  （写行与暖机都是零单元发布，`mount.rs:1098-1119` 的 `PoolVersion::WithoutFile` 那一支），而实例表行的唯一消费者是
  `D18 已定项 11` 的已发布谓词（`.claude/kb/decisions/18-块里携带什么信息.md:309`「无行 ⇒ 已发布」）
  ⇒ **这一版上缺的那几行谓词值恒真、不带信息**。所以「不写行」与「写行」在**已发布谓词上分不开**，
  在**盘上字节**与**可达历史**上分得开（写行要多一个实例表单元、要分配记录与记账树；不写行零字节）
  ⇒ 按正文第五节第 3 条，这一格**该写条款**，不是实现细节。

## 四、R2：回退到树表 0 条的根（`RollbackToVersionWithoutFileUnsupported`）

### 4.1 打中的历史（零故障）

1. `mkfs` → 取号 1 → 暖机 txg 1、2 → 第一个文件版本 A（txg 3）。（`build_pool`，与仓里步 3 同一条流。）
2. 进程退出、重开可写挂载：取号 2、写行 **(1, 3, 0)**、暖机。**做成了。**
3. 管理员带外挑回退目标 (1, 1) 或 (1, 2)——实例 1 的两条暖机根。**被拒。**

```
[R2] 第二次挂载写的行：[(1, 3, 0)]
[R2] 回退到 (1, 1)：Some(RollbackToVersionWithoutFileUnsupported(RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(1) }))
[R2] 回退到 (1, 2)：Some(RollbackToVersionWithoutFileUnsupported(RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(2) }))
[R2] 对照：回退到 (1, 3) 成功 = true
```

**为什么这是「条款允许而实现不允许」而不是「用户想干一件没道理的事」**：
`.claude/kb/decisions/23-journal的角色与格式.md:363`（已定项 14 的显式例外那一段）逐字
「**候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，
或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）**」。这条历史上：(1,1) 与 (1,2) 在环里、F_生效 = 0、
最新根指着的表里实例 1 的行是 (1, 3, 0)、1 ≤ 3 且 2 ≤ 3 ⇒ **按条文逐字它们可选**。实现也同意——
`mount.rs:1260-1295` 三条候选排除一条都没中（所以报的不是 `RollbackTargetNotACandidate`），
`mount.rs:1299` 才用「树表 0 条」这条**条文里没有的**理由把它拒掉。对照臂 (1,3) 成功，说明拒的确实只是「那一版树表 0 条」。

「用户有没有正当理由」这一问我给两条，都不靠想象：① 回退的语义就是「回到这条根那一刻」，而 (1,2) 那一刻正是
「池建好了、还没有文件」——把一个写错了内容的池退回空状态，比重做 mkfs 省一件事（fsid、系统配置里的实例代号谱系、根环历史都留着）；
② 更硬的一条：**条文允许它**，而实现拒它——不需要我论证用户想不想要，条款已经承诺了。

### 4.2 三关

- **拒得早：过**。两次拒绝前后 `DiskSnapshot` 逐字节相等（用例里 `assert_eq!`，绿）。判定点 `mount.rs:1299` 在取号之前。
- **拒得准：不过**。零故障可达，且被拒的动作是条文逐字允许的。
- **拒得明：过**，而且这一格今天做对了一件事：**它不报候选排除**（`mount.rs:136-137` 的文档逐字
  「目标那一版却树表 0 条的，报的是 `MountError::RollbackToVersionWithoutFileUnsupported`（在候选集里、第一版不支持），不在这里」）
  ——名实相符，调用方分得出「你选错了根」与「这一版实现不支持」。

### 4.3 四句

1. **分不分辨臂**：分得开，同一次挂载里目标换成 (1,3) 就成功（上面的对照）。用户动作（选哪条根）在装置里是放开扫的三条，没写死。
2. **系统看不看得到判别它的东西**：看得到——它自己先算完了候选集三条、再读树表。
3. **判据的哪一分句**：第 2 条两个分句都满足（历史零故障，实测撞上）。
4. **跑前条款给的改法还中不中**：仓里这一格的处置是「第一版不支持、要开决策」（里程碑 141 行）。把拒绝去掉不是一个安全的改法
   ——回退要在 R_old 那一版实例表上写回退行，而那一版的实例表是 mkfs 那一片、这一版没有分配记录树与记账树；
   我**没有**在副本上试「去掉这道拒绝会写出什么」（见「这条腿自己的限度」）。

### 4.4 条款侧

两种选法在盘上分得开：① 兑现条文（树表 0 条的根也能回退，回退行写在哪要定）⇒ 盘上多一个实例表单元、多一条回退行；
② 条文加一句排除（候选集再加一条「那一版树表非 0 条」）⇒ 盘上零字节、可达历史里少掉「回退到空池」那一族。
⇒ 按第五节第 3 条这一格**该写条款**。今天两处说反话（D23 已定项 14 的候选集 vs `mount.rs:1299`），这笔账没人记
——`C124` 记的是回退行与重放下界，不是这一条。

## 五、R3：所选根有文件、环里一条记录都读不出（`FileVersionWithoutAnyJournalRecord`）

### 5.1 没打中：这一族在零故障 / 一次普通崩溃下不可达

三步论证，每一步都有落点：

1. **记录先于根持久**。`FileVersionWithoutAnyJournalRecord` 只在所选根的树表非 0 条时才够得着
   （`recovery.rs:658-662`：树表 0 条先 `return Ok(RebuiltVersion::WithoutFile)`，根本走不到那一句）。
   而一条树表非 0 条的根能被择中，说明它那次发布已经持久；D16 已定项 7 的持久顺序（单元 → 记录 → 根）在仓里被
   `.claude/kb/checks-owed.md:322`（C365）逐字记着「最短样本：暖机 txg 1 的记录落盘、根没落」——**反过来「根落了、记录没落」在崩溃模型里不出现**。
2. **记录不会被删，只会被更新的记录盖**，而盖是「整条写成功或整条不变」（层 0 的崩溃模型按写粒度取全或无）。
   所以 `|records| ≥ 1` 在任何崩溃点上都成立。
3. **环大得几乎不回绕**：`JOURNAL_RING_DEFAULT_BYTES = 768 MiB`（`crates/singlefs-format/src/lib.rs:162`），
   `JOURNAL_RECORD_BYTES = 4096`（同 `:139`）⇒ 196608 条槽位；回绕也只是被更新的记录盖。

⇒ 要让 `records` 空，只能把**每一条记录的两份镜像**都毁掉：这是 ≥ 2 次介质故障（本轮预算是零故障或一次普通崩溃），不算误拒。

### 5.2 但这道拒绝的前提不承重（两条实测）

```
[R3] 抹掉的记录槽：32 个（两块盘各 16）
[R3] 环里自证过的记录：0 条
[R3] 冷启动仍读回 (1, 3) 的内容 3000 字节，与写进去的一致 = true
[R3] 可写挂载：Some(FileVersionWithoutAnyJournalRecord)
```

- **拿掉全部记录，文件照样读得回来**：同一份镜像上 `recover(..., JournalPolicy::Consult)` 交回
  `RecoveryOutcome::FileRead`，3000 字节与写进去的逐字节相同。⇒ 这道拒绝挡住的不是「这一版重建不出来」。
- **这条前提随便哪条无关记录都满足**：`mount.rs:1183-1190` 与 `:1303-1309` 两处都是
  「找所选根自己那条；找不到就 `.or_else(|| records.values().max_by_key(|record| record.counter))`」
  ⇒ 环里只要有**任意一条**自证过的记录（哪个实例、哪个 txg 都行）就放行。它要的不是「这条根的记录」，是「有一条记录当占位」。
  `rebuild_version` 拿它只填 `TransactionOutput { record, record_bytes, … }`（`recovery.rs:889-893`），
  而 `recovery.rs:907-910` 的注释自己写着这条记录上的事务号**不会**被拿去接着发布。

### 5.3 三关

- **拒得早：过**（代码路径论证，不是字节量的：`mount.rs:1191` 的 `rebuild_previous_version` 在 `establish_instance`
  （`:1218`）之前，取号在 `:1063`，中间没有写）。⚠️ 我**没有**为 R3 量 `DiskSnapshot`。
- **拒得准：存疑**。被拒的那一族零故障不可达 ⇒ 按本轮判据不算误拒；但它拒的时候手里已经有一条能把整版重建出来的根，
  而它要的那个占位物与这一版毫无关系。**「拒绝优先于猜」在这里没有在猜的东西**。
- **拒得明：过**（名字说的就是「有文件版本、而环里一条记录都没有」，与 `Recovery(NoValidRoot)` 分得开）。

### 5.4 结论

**登记成第一版不支持**（不立条款）：这一族要 ≥ 2 次介质故障才到得了，而到了那里之后正确的动作是走修复、不是猜。
另记一笔欠账：**「要一条记录顶着」这个前提可以拆掉**（`TransactionOutput` 的 `record` 字段对树表非 0 条的重建不承重），
拆掉之后这一族历史从「可写挂载永远失败」变成「照常挂载」；今天没拆，所以一个介质故障族把池钉成只读。

## 六、R4：「只兑现第一次」是条款该允许的，还是实现缺口？

派发明写：R4 那个已知缺口不许当新发现。我答的是那一问本身，答案靠**逐层拆墙**量出来：在副本上把拒绝一道道关掉，看下一道墙是谁。

| 跑法 | 第二次可写挂载的结局（原样） |
|---|---|
| 不打补丁 | `InstanceRowsOnVersionWithoutFileUnsupported { chosen_root: (1, 2), first_row_instance: 1, instance_to_acquire: 2 }` |
| `OPUS_SKIP_R1=1`（树表 0 条时不写行） | `FormattedPoolMountNotShapedLikeTheFirstTransaction { chosen_root: (1, 2), first_txg: CheckpointTxg(3), first_counter: 3 }` |
| `OPUS_SKIP_R1=1 OPUS_SKIP_R4=1` | 挂载**做成了**：`实例 2，写行 0 条，写行那次 txg 3，暖机 1 次（txg [4]）`；紧接着发第一个文件版本 ⇒ `FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Some((CheckpointTxg(4), 4)) }` |

### 6.1 答案

**「只兑现第一次」不是某条条款允许的，是三层实现选择叠出来的，而且最里面那一层与前两层无关**：

1. `mount.rs:861` 那道（R1）——理由是「实例表落点没有条款」。
2. `mount.rs:885` 那道（R4）——理由是「第一个文件版本写死 txg 3 / jsn 3 接不上别的形状」。它是**独立**的一道：
   R1 关掉之后它照样把同一条历史拦住。
3. `transaction.rs:1480` 的 `let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);` 与 `:1487` 的
   `previous_txg.0 + 1 == FIRST_TRANSACTION_TXG && previous_counter + 1 == FIRST_TRANSACTION_TXG`——
   **前两道都关掉之后，池仍然一个文件都写不进去**，因为第一个文件版本的 txg 与 jsn 是写死的 3。

⇒ 2026-09-17 用户定案「允许只做过 mkfs 的池可写挂载」在实现上**只在「这个池此前没有人取过号」这一格兑现**，
而定案的字面没有这个限定。**判 R4 的题面：实现缺口**（缺口的病根在第 3 层，不在第 1、2 层的「落点没有条款」）。

### 6.2 但这里另有一处真的条款空白（这一格要立条款的那一句）

`.claude/kb/decisions/22-单元原子性怎么合成.md:330`（已定项 16 第 5 句）逐字：
「**暖机空发布 2 次、第一个事务 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、
FIRST_TRANSACTION_TXG = 3，登记在 [layout/01-first-txn.md](../layout/01-first-txn.md) 八）。」
**这句话没说它的射程**：它是「mkfs 之后那一条流的常量」，还是「任何池的第一个文件版本的常量」？

- 读法甲（只管 mkfs 那条流）：第二个实例在空池上发第一个文件版本时按**现算**的 txg 走（这条历史上是 txg 5），池能用。
- 读法乙（管任何池）：只做过 mkfs 的池一旦被挂载过一次，就**永远**写不进第一个文件——今天实现取的是乙。

两种读法**在盘上字节与可达历史上都分得开**（第一个文件版本那条根的 txg、jsn、反向链全不同；池能不能有文件更是分得开的）
⇒ 按第五节第 3 条，**该写条款**：定 `FIRST_TRANSACTION_TXG` 的射程。同一句话还连着 `WARM_UP_EMPTY_PUBLISHES = 2`
——我实测这个数今天在两条流上就已经不是 2：只做过 mkfs 的池第一次可写挂载**暖机 1 次**（`[R1] 暖机 1 次`，txg 1 落盘 1、
txg 2 落盘 0，写行那次已经覆盖了一块盘），第二次（跳开两道拒绝之后）也是 1 次。⇒ 那个格式常量在「写行那次发布也算覆盖」的
实现下已经不是常量，而 `mount.rs:979` 的 `warm_up_publish_txgs` 就是按现算走的（P3 那一格归正推腿，我只报观测）。

### 6.3 三关

- **拒得早：过**（`mount.rs:1055` 在取号之前；`DiskSnapshot` 相等由同一条用例的 R1 那一格量过——两道判定挨着，中间没有写）。
- **拒得准：这个成员在零故障历史上够不着**（被 R1 挡在前面；`mount.rs:1054` 先于 `:1055`）。
  它自己那一族要故障才到得了（背景里的 Z3-A：两盘系统配置槽各坏一字节 + 一次崩溃）。
- **拒得明：过**，两个字段（`first_txg`、`first_counter`）说得出是哪一样不对。⚠️ 名字里的「NotShapedLikeTheFirstTransaction」
  没说「为什么非得同形」——理由（`publish_first_file` 写死 3）在别的文件里，调用方按名字读不出「换个实现就没这条限制」。

## 七、Q1：抬 F 时现行版本里没有实例表单元（`RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`）

### 7.1 打中的历史（零故障）

1. `mkfs` → 取号 1 → 暖机 → 第一个文件版本 A（txg 3）。
2. **同一个进程里**覆盖写三次（txg 4、5、6）。零故障、没退出进程。
3. 抬 F 到 3（上限正好是 3）⇒ 被拒。

```
[Q1] 现行版本 txg 6，units 里的角色：["Data", "ExtentRoot", "InodeLeafContainer(InodeLeafContainerIndexInTree(0))", "InodeRoot", "AllocationTree", "AccountingTree", "MappingTree", "TreeTable"]
[Q1] 同一版的根指针指着的实例表读得出、解得开 = true，行数 Some(0)
[Q1] 抬 F 到 3：Some(RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion)
```

**这一格比仓里已有的那条用例宽**：`second_transaction_step_five_reuse.rs:615` 那条只跑「第一个事务刚发完就抬 F」，
读起来像个边角（「还没发布过东西，当然没有表」）。实测是：**mkfs 那个进程里后面发多少次发布都一样**——
`transaction.rs` 的 `units` 只在 `previous` 里有实例表单元时才把它照抄下来
（`transaction.rs:2593-2600` 的 `(None, Some(previous_version)) => if let Some(carried) = …`），
而 `publish_first_file` 那一版的 `previous` 是 `None` ⇒ **整条链上一个都没有**。
换句话说：**mkfs 与第一个文件版本在同一个进程里的那条路径（也就是仓里的固定脚本、E142 那条流），抬 F 永远做不成。**

### 7.2 三关

- **拒得早：过**。`DiskSnapshot` 逐字节相等（用例里 `assert_eq!`，绿）；`mount.rs:660` 是 `raise_rollback_floor` 的第二句，
  在回收、空发布之前。
- **拒得准：不过**。零故障可达，而抬 F 要的那张表**在盘上读得出、解得开**：同一版的 `current.root.instance_table`
  指着 mkfs 那一片，`instance_table_of_root`（`recovery.rs:573`）当场解出 0 行。拒绝拒的不是「判不了」，是「没放在那个数组里」。
- **拒得明：不过**。名字说「现行版本里没有重写过的实例表单元」，字面读起来像「这一版没有实例表」；实际条件是
  「`TransactionOutput::units` 这个**内存**数组里没有」。`mount.rs:41-43` 的文档把射程写清楚了，但**名字自己**把
  「内存里的角色列表」说成了「现行版本」，与 R1/R2 那两个名字（说的是盘上的事）不是一个口径。

### 7.3 改法量过：Q1-F1 让它一次跑通

`mount-rs-opus-fixes.patch` 的第一处（`units` 里没有就从现行根的指针上读那一版表），同一条历史：

```
[Q1-F1] 抬 F 到 3 做成了：上限 3，空发布 2 次（txg [7, 8]），回收 1 个落点
```

上限 3 与条款算得出的一样：有效根 txg 0–6，非空的是 3、4、5、6（四条）⇒ 第 4 新的非空 = 3；
每块盘上最新的有效根 = min(dev0 的 6, dev1 的 4) = 4；min(4, 3) = **3**（`.claude/kb/decisions/16-发布语义.md:36`
那一行「抬 F 的上限 | min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)」）。
读回来的表是 mkfs 那张 0 行表 ⇒ 「按实例表判被抛弃」在这一版上对每条根都是假，与用 `units` 里那份算出来的结果**相同**。

### 7.4 四句

1. **分不分辨臂**：分得开——同一条历史，只改这一处，拒绝变成成功且结果可核（上限 3、空发布落两块盘、回收 1 个落点）。
2. **系统看不看得到判别它的东西**：**看得到**（表就在它手里的根指针后面，一次读就解得开）。这一条是四句里最要紧的：
   Q1 不是「条款空白上的拒绝」，是**实现拿错了输入**。
3. **判据的哪一分句**：第 2 条两个分句都满足。
4. **跑前条款给的改法还中不中**：正文没给 Q1 改法；增补 3 第 2 件的模型把它划进「允许拒」
   （`crates/singlefs-harness/src/model_comparison.rs:232-233` 把它映到 `ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported`）
   ⇒ **模型今天替它背书**，随机历史快档走到 34 次也不会红。改法落地时那一条映射要跟着删，不然模型会把修好的实现判成「少拒了」。

### 7.5 今天为什么不伤用户，以及它什么时候会伤

`grep -rn "raise_rollback_floor" crates/*/src/` 除了 `mount.rs:647` 的定义本身，只有
`allocator.rs:302` 的一句文档注释和 harness（`history.rs:2661`）——**产品路径零调用点**，与
`.claude/kb/checks-owed.md:427`（C482 第 ② 条）逐字「`crates/*/src/` 里零个产品路径调用 `raise_rollback_floor`」一致。
⇒ 今天这道误拒够不到用户。**它会伤的那一天**是 `.claude/kb/decisions/16-发布语义.md:39`（D16 已定项 1「准入」那一行）
逐字「准入不够时先推空发布抬 F……一次准入最多 B = 4 + 2 k_tol 次发布（k_tol = 2 ⇒ 8），做满仍不够才报 ENOSPC」
被实现出来的那一天：在 mkfs 那个进程里，那条「先抬 F」永远走不到第一步，直接掉到 ENOSPC
——撞 `.claude/kb/decisions/03-空间分配.md:175` 已定项 9「不许有假性 ENOSPC，删掉的空间在有界步数内可用」。
**⇒ 结论：不立条款（两种选法在盘上写出同样的字节：表是同一张）、改代码、并把它挂在 C482 ② 与 D16 已定项 1 的产品触发之前。**

## 八、我自己提的两个改法（**只在副本上量过、被攻过零轮**）

补丁 `mount-rs-opus-fixes.patch`（48 行），两处；下表每一格标「量过」（贴的是副本上的原样输出）或「推的」（按代码推、没实现没跑）。

| 改法 | R1-α（空池第二次挂载） | R1-β（第一次挂载崩在取号之后） | R2（回退到暖机根） | R3（记录全没了） | Q1（mkfs 进程里抬 F） |
|---|---|---|---|---|---|
| **Q1-F1**：`raise_rollback_floor` 里 `units` 没有实例表单元时改从 `current.root.instance_table` 读那一版表（`mount.rs:656-662` 换成 `match … None => instance_table_of_root(&*devices, &current.root)`） | 不动（**推的**：不在这条路上） | 不动（**推的**） | 不动（**推的**） | 不动（**推的**） | **修好，量过**：`抬 F 到 3 做成了：上限 3，空发布 2 次（txg [7, 8]），回收 1 个落点` |
| **R1-F1**：树表 0 条的一版上不写行（`rows_written` 清空），`refuse_instance_rows_on_version_without_file` 不再拦 | **半修，量过**：R1 那道过了，同一条历史随即撞 `FormattedPoolMountNotShapedLikeTheFirstTransaction { first_txg: 3, first_counter: 3 }` | 同上（**推的**：三个崩溃点算出来的 `first_txg` 都 ≥ 2，形状判定同样拦） | 不动（**量过**：回退那条路不走 `establish_instance` 的这一句之前的判定——见下一行的限制） | 不动（**推的**） | 不动（**推的**） |
| **R1-F1 + 跳过形状判定**（只为量下一道墙，不是改法提案） | **量过**：挂载做成（实例 2、写行 0 条、txg 3、暖机 txg 4），紧接着第一个文件版本被 `FirstFileVersionNotRightAfterTheSecondWarmUp { previous_record: Some((CheckpointTxg(4), 4)) }` 拒 | — | — | — | — |

两句要紧的：

1. **Q1-F1 是我这条腿唯一敢说「这样改就对」的**：它不改任何盘上字节的语义（读回来的表与本该在 `units` 里的那一份是同一版），
   只把输入从内存数组换成根指针。**没攻过**：我没试「根指着的实例表单元坏了 / 解不开」那一格（那时它落到
   `MountError::InstanceTableMalformed`，比今天的名字更贴切，但我没跑）。
2. **R1-F1 不是一个完整的改法**，它只证明了一件事：R1 与 R4 这两道拒绝**各自独立**地挡着同一条历史，而它们后面还有第三道
   （`FIRST_TRANSACTION_TXG` 写死）。要把「只做过 mkfs 的池可写挂载」兑现到第二次，**三处都要动**，其中第三处要先定
   `FIRST_TRANSACTION_TXG` 的射程（第六节 6.2）。R1-F1 本身还欠一个我没答的问题：不写行之后，
   那几个实例的行什么时候补上——我的答案（「不补，因为那一版上它们的谓词值恒真」）**只在树表 0 条的一版上成立**，
   一旦这一版之后发了文件版本，`[1, n)` 那几行就永远不在表里了，而 D18 已定项 11 的写行规则逐字是「每次可写挂载都写行」
   （`.claude/kb/decisions/18-块里携带什么信息.md:304`）⇒ **这一条要用户定，不是我能定的。**

## 九、没打中的形状（试过哪些、取样范围多大）

- **A. R3 的「一次普通崩溃让记录全没」**：按崩溃模型（写全或无）+ 持久顺序（记录先于根）推出不可达，见 5.1 三步。
  试过的取样：把 16 × 2 = 32 个记录槽清零才到得了那一格（副本上量过）；崩溃点这一路**一个都没试**（层 0 全量枚举归 crash-verifier，
  我没跑）。
- **B. R4 那个成员的零故障历史**：分析上要「要写的行为空 ∧ 形状不对」同时成立 ⇒ 要 `instance_to_acquire == 1`（没人取过号）
  ∧ `first_txg ≠ 1 ∨ jsn ≠ 1`（环里已有 txg ≥ 1 的根或记录）。两者在 2 盘上互斥（mkfs 把两个环都清零、第 0 代根 txg 0）。
  **试过一条**：同一对镜像上用同一个 fsid 再做一次 mkfs（想让旧池的记录顶着）⇒ 实测
  `[R4] 新池环里仍自证得过的旧记录：0 条`、`[R4] 新池第一次可写挂载做成了：实例 1，写行那次 txg 1`——
  **mkfs 今天把 journal 环与根环一起清了**（`make_filesystem.rs:260-262`），这条路堵死。
- **C. 3 块盘上的 R4**：`make_filesystem.rs:176` 那道归属检查只在 `devices.len() == 2` 时判，3 盘时区域归属可以是
  `[0, 1, 1]` ⇒ `device_of(txg 1) == device_of(txg 2)` ⇒ 形状判定当场不过、**零故障**。**没试**：`make_filesystem` 第一句是
  `assert_eq!(devices.len(), 2, "第一版跑 2 块盘（D2 已定项 9）")`（`make_filesystem.rs:188-192`）⇒ 今天 3 盘根本建不出来，
  这条形状要等加盘那条线。**我把它记成一条「以后会中」的提醒，不算打中。**
- **D. R2 的「回退到 mkfs 第 0 代根」在有文件的池上**：没单独试（空池那一格试过、被拒）。推的：`tree_table_has_no_entries`
  对第 0 代根恒真 ⇒ 同样被拒；但第 0 代根在有文件的池上还要过 F_生效 与实例表两关，我没量。
- **E. 「拒得早」对 R3 的字节量**：没量 `DiskSnapshot`，只给了代码路径论证（5.3）。
- **F. 把 R2 那道拒绝去掉会写出什么**：没试。它要在一版没有分配记录树与记账树的池上写实例表单元，
  我判断那会撞 `publish_rows_on_file_version` 的前提（`mount.rs:773`），试它要改的代码比 R1-F1 多得多，本轮没做。

## 十、这条腿自己的限度

1. **全部的数都是副本上的**（`/tmp/claude-1000/m2-refusals-opus/repo`），入库装置上没重做；
   `rsync -a --exclude target --exclude .git` **没能排掉 `target/`**（副本里带了 785 MiB 的 `target/debug`，
   cargo 在它上面做了增量编译）⇒ 这几次跑用的不是全新的构建，判定靠的是用例里的 `assert` 与原样输出，不靠计时。
2. **五条探针全是我自己写的用例**，不是层 0 全量枚举：R1-α / R2 / Q1 三条历史是**手写的固定脚本**，
   「每一个崩溃点都这样」那句话我只对 R1 给了三格的推理（3.1 末），**没有枚举证据**。
3. **用户动作没全放开扫**：R2 那一格我扫了「环里每一条可读根」（3 条，空池那一格）与「(1,1)/(1,2)/(1,3)」（有文件那一格），
   但「回退之后再挂载」「抬 F 之后再回退」这类更长的序列一条都没扫——增补 3 的随机历史装置能扫，我没跑它。
4. **R3、R4 两格是「没打中」，不是「不存在」**：我的不可达论证依赖两条我没有独立验的前提
   （崩溃模型写全或无；D16 已定项 7 的持久顺序在实现里真的是单元 → 记录 → 根）。
   推翻它们的观测：**只要有人给出一条「根落了、它那次发布的记录两份都没落」的崩溃点，R3 立刻变成一次普通崩溃可达。**
5. **判 Q1 的「今天不伤用户」只在 `crates/` 这一层成立**：产品路径零调用点是我 `grep` 出来的
   （`crates/*/src/` 里除定义外 0 个），FUSE / 命令行那一层今天不存在，将来有了要重查。
6. **跑的时候仓在动**：`crates/singlefs-core/src/recovery.rs`（21:56 UTC）与 `transaction.rs`（21:57 UTC）在我这一轮中途被
   别的会话改过（`mount.rs` 没动，13:21 UTC）⇒ 报告里 `transaction.rs` 的行号是 22:01 UTC 重取的，
   **`mount.rs` / `recovery.rs` 的行号取自同一时刻**；再过一会儿可能又不对了，判定请按**错误成员的名字**读。

## 十一、没做什么

- 不判 P1–P7（正推腿的格）与 R5–R8（本地攻方腿的格）。第六节里出现的 `FirstFileVersionNotRightAfterTheSecondWarmUp`（R5）
  只作为「R4 那一问」的观测，不含对 R5 的判定。
- 不读 `research/prompts/m2-refusals-presumed-r1-sonnet-output.md`（禁读），也没读模型目录里上一条腿留下的七个文件。
- 不写 kb、不 `git add`、不提交；`crates/` 里一个字节都没改（改动只在副本上）。
- 没跑层 0 全量、没跑门禁（`.claude/gate.d/stage-owners.tsv` 里没有登记给这条腿的阶段）、没跑变异表。
- 没替主 agent 采纳任何改法；第八节两个改法**被攻过零轮**。
