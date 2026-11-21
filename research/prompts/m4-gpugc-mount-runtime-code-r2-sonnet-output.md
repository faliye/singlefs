# m4-gpugc-mount-runtime-code-r2：云端辩方腿报告（2026-11-09）

立场：辩方。复核第一轮判决 `research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md`：撤回的够不够得着、打中的修补有没有打中所有替代方案、有没有放宽。T2 主攻。
读的是冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2/repo/`；本报告引到的十份文件（运行库 `lib.rs`、核心层四份、开卡器两份、`transaction.rs`、两份运行库用例、一份真卡用例）与仓根同路径文件逐字节相同（`cmp` 过，量的），引文行号对仓根也成立。
**腿没跑任何代码、没起 cargo。** 引到的读数是第一轮主 agent 量的两份产物（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/run/readings.out`，sha256 `d42c6e6da0a105361ae73a41bafcd8322e1b9a2ae381b255a6f23dd44cab5498`；`…/s2-run/readings.out`，sha256 `eaf483288abdbc0d2704d6747f74b1c870c6fe4e46bcad4561710cb193ca9634`，`sha256sum` 现取，量的）；本腿自己的新读数一条都没有，写「推的」处都是读码或算术。

模型目录 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-sonnet/model/`，sha256（`sha256sum` 现取，量的）：

```
ec31d95f584d2d13ca4d6ed50f6d45cbb85ed4679cc3db091fd6442ae64a77f7  src/defense_full_pool.rs
2fd7c6f44725887272e817b85f036116d4992ad56231477a2b76fd60634413f6  tools/make_the_small_pool_fixture.py
c66297432aec2b8f71d805b0c9ae5b9ccce4a221c0caef923a48449faa97b7fd  tools/rerun-from-a-clean-work-directory.sh
0a314ef40b1e4153e0d447386125070044bbad874bc60d2046a288ec2c297938  tools/rerun.sh
```

`defense_full_pool.rs` 是第二次抽样攻方探针（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model/src/attack_full_pool.rs`）的改版，**没编过**，`rustfmt --check` 只报了我为查格式放的空桩文件（`fmt-check.txt`，量的）。`rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改了 `rerun_steps`，`bash -n` 过（量的）。第一次跑可能要调。

## 判定一览

| 格 | 辩方判定 | 一句话 |
|---|---|---|
| T1 余量取 S、门与 K | 部分够不着 | 余量取 S 在 S=6、7 比旧值少保一次挂载，没人比过「余量取 min(8, 3S−9)」；判决表里没有「余量只算 K=16」一行（读数 `margin_sweep` 五行都没造出活意图） |
| T1 破例排在让路之前 | 站得住 | 抢锁有上界：每趟一次、趟不留着，线程隔一个周期才再来（读码） |
| T1 每拍先重发 | 站得住，退避一问判决没答 | 重发是逐字节原样再发，不涨分配器；但周期从 30 秒缩到 1 秒，判决没有读数 |
| T1 卸载先重发 | 站得住，没打中替代方案 | 重发失败只交盘、丢会话；同一函数里邻近的分支用 `of_the_session` 把运行库交回 |
| T1 删除走空放锁 | 站得住，但得到的比判决说的少 | 放锁买到的是 fsync 与读；`accept` 在队列不空时被拒，前台单发改动自己把队列走空 |
| T1 开卡器放闸、不持锁建实例 | 站得住 | 见检查单与 T3 |
| T2 撤回「只用 accept 写的走不出来」 | 字面够得着，「运行库救得出来」够不着 | 读数没分辨臂；六组是三条历史；只试了一单元覆盖写 |
| T2 撤回「accept 被插队」 | 站得住（按跑前翻面条件） | 但 accept 多等一个发布，中位等待约为 fsync 的四倍 |
| T2 撤回「反向饿线程」 | 站得住，射程窄 | 那一探针里整理一拍什么都没做 |
| T2 搬根记成状态「照旧」 | 站得住 | 字面是 D16 的口径；崩溃循环的下降主要是挂载自己的发布 |
| T2 崩溃循环「有尽头」 | 站得住 | 第 3 次挂载起整理发布为 0，状态照降 |
| T2 S 扫描用例 | 够不着「产品开关」一句 | 强制选段、水位取 100%、批大小强制为 1；没有 64 单元的最坏一条意图 |
| T2 搬探针成用例有没有放宽 | 有三处放宽 | 底额靠测试开关摆、预算靠直接写计数、没有满盘上的整条路 |
| T3 | 站得住，两处新缺口 | 新抄的删除次序在三种队列里只有 inode 一种被运行库用例走到；开卡器额度改成建之前记，建实例 panic 也占一个额度（共 4 个） |

## T2：产生结论的方法（主攻）

### 2.1 撤回「只用 accept 写的走不出来」（判决 `research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md:24`）

结论：**撤回在字面上够得着，判决里「运行库在满盘上救得出来」这一句够不着。** 依据是五条，每条都核到文件行：

1. 探针每拍的次序是先推钟、先跑一拍整理、之后才试 `accept`：`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model/src/attack_full_pool.rs:259`（`clock.advance_by(elapsed);`）、`:261`（`let pass = runtime.run_one_maintenance_pass()`）、`:263`（`let accepted = runtime.accept(overwrite);`）。删完文件到第一拍整理之间没有试过 `accept`。攻方腿第二次抽样跑前写的撤回条件是「40 拍内过了，且过之前 `total_publishes` 非 0」（`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-s2-output.md:73`），第二个分句在这个次序下**靠构造成立**（第一次试就在整理之后），它分辨不出「整理救的」与「删文件本身就让全空段够了底额」。读数 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-run/readings.out:13`（删 1 个文件，`total_publishes=10 … empty_by_the_records=5 … openable=false accept_went_through=true`）两种解释都容得下。推的，没量。
2. 六组是三条历史。`topped_up=true` 那三组的补写一份都没写成：`…/s2-run/readings.out:39`、`:43`、`:50`（`files_written=0 small_files_written=0 empty_by_the_records=0 … accept_before_unlink_went_through=false`）；它们的 `full_pool_pass` 行（`:41`、`:48`、`:67`）与 `topped_up=false` 的（`:13`、`:19`、`:37`）逐项相同（`total_publishes` 10、6、2，`empty_by_the_records` 5、4、16，`openable` false、false、true，量的）。判决写的「六组」放大了样本数（盘一模一样、过程确定，重复的是同一次运行）。
3. 试的只有一单元、35 字节的覆盖写：`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-model/src/attack_full_pool.rs` 里 `small_content = b"a foreground write after the unlink"`，改动类型 `OverwriteOfAnInode`，入口契约是一单元（`crates/singlefs-core/src/mounted_session.rs:5992`：「WholeWriteContractAtTheAccept::OneDataUnit,」）。攻方腿第二次抽样自己写过「一单元的 `overwrite` 落得下」（`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-s2-output.md:56`）。填盘用的是 32 单元的 `SequentialWriteOfAnInode`（契约 `AnyNumberOfDataUnits`，`crates/singlefs-core/src/mounted_session.rs:5976` 起那一段的第二支）。**同一个前台用同样大小写，没试过。**
4. 收下不等于发成：读数里 `accept=Ok(ChangeAccepted { taken: KeptPending …, publishes: [] })`（`…/s2-run/readings.out:13`）。`accept` 的那一判是「接在上一版后面」的预判，被空间拒时「不推抬 F」（`crates/singlefs-core/src/transaction.rs:10326` 的文档；这一行的 `grep -nF` 命中见下），推抬 F 的是排空那一次。探针收下之后没有调 `publish_pending_now`，所以不知道收下的这一份发不发得成。读数 `openable=false` 在 `:13`、`:19`、`:41`、`:48` 四行（量的）：开不成段，说明可再分配的槽还在 defer 里。
5. 池是 320 MiB、240 段（`…/s2-model/tools/make_the_small_pool_fixture.py`），填满时 173 份文件（`…/s2-run/readings.out:11`，量的）。外推到大盘：这一格的判（底额按「按记录全空的段数」比，`N₀ = 1`）不随盘走，所以**这一条外推我没找到站不住的地方**；会随盘走的是「删一个文件放出多少段」与「环转满一圈要几次发布」，读数没覆盖。

`transaction.rs:10326` 的原文与行号核对：`grep -nF '被空间拒时这里不推抬 F' crates/singlefs-core/src/transaction.rs` 命中 10326（量的，本腿跑过）。

**什么现象会推翻我这条「够不着」**：新探针 `defense_full_pool`（见「要主 agent 跑的命令」）里 `size=thirty_two_units` 的各行 `accept_went_through=true` 且 `pending_published=[ok published_something=true]`，在 `passes_before=0` 的各行也如此。那样撤回连同「救得出来」整句站得住，只是功劳不在整理。反过来，`passes_before=0` 的 `thirty_two_units` 行 `accept_went_through=false` 而 `passes_before=3` 起变 true，说明整理确有功劳，撤回仍站得住但要补一句「限于整理转得完环」；全程 false 才是撤回被打回。

### 2.2 撤回「accept 被插队」（判决 `…main-verification.md:33`）

站得住（按跑前翻面条件「中位数不超过 2」，读数 `…/run/readings.out:31`：`sorted_publishes_overtaking=[0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2]`，量的）。两处限定：

- 同一份产物里 `accept` 的等待时间排序（`waited_microseconds`，`…/run/readings.out:19`–`:30`）是 19003、32999、37992、37999、40991、43987、88986、100010、106084、106314、118362、167981，中位数约 66 ms；`publish_pending_now` 的（`:32`–`:43`）是 376、495、15650、16005、16295、17066、17371、18540、19719、21072、52763、57925，中位数约 17 ms（量的，`sort -n` 现取）。**`accept` 多等的是一个发布，约为 fsync 的四倍**；差别的来由是 `accept` 进门不登记在等（`crates/singlefs-mount-runtime/src/lib.rs:835` 起，登记只在 `publish_pending_now`）。判决不改它是对的（条款字面只说排队），但「撤回」不应读成「两个入口一样」。
- 内存盘上的一次发布 17–168 ms，真盘上只会更长。

### 2.3 撤回「反向饿线程」（判决 `…main-verification.md:34`）

站得住，射程窄。读数 `…/run/readings.out:49`–`:52`：`threads` 0、1、4、8 的 `maintenance_passes` 是 1898、1901、1903、1900（量的）。探针 `foreground_threads_spinning_on_the_lock_hold_back_the_maintenance_thread`（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/opus-model/src/attack_probes.rs` 里，`RuntimeParameters::by_default()` 加 `pool_with_holes_in_the_data_segments`）用的是产品默认水位，这个池上整理一拍什么都不做；`maintenance_passes` 被睡眠周期（`SHORT_PERIOD`）卡在约 950 拍每秒。所以它证明的是「前台打转时维护线程仍能在毫秒内拿到锁走完一个空拍」，没有一条臂是「整理有活、一拍要持锁发一次布」。没有阳性对照（读数里找不到「会红的那一臂」）。**判决说的「撤回」不受影响**（攻方要的是饿死，没饿死），但「维护线程有活时被前台饿不饿」没量。推的，没量。

### 2.4 搬根记成状态「照旧」与崩溃循环「有尽头」（判决 `…main-verification.md:31`、`:32`）

站得住。

- 口径：D16 的定案句是 `.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读` 里的「回退候选集保留最近 4 个可退到的不同状态——只数改过用户可见状态的根」，判法在同一节：「一条有效根算非空 ⟺ 它的树表里用户可见的树」根指针与前一条有效根的不同。字节比较的代码是 `crates/singlefs-core/src/mount.rs:3841`，文档 `:3839` 写着用户定「照字节比，搬根算非空」。门数「状态」就是这个口径，判决没有把门改成另一个口径。攻方自己也写了「字面不违反」（`research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md:23`），并指出这一处「可能是判据写错而不是代码写错」要主 agent 或用户定口径（同一份 `:69`）。
- 崩溃循环的下降主要不是整理造成的：`…/run/readings.out:3`–`:6`（FullestDataSegment 臂第 3 到 6 次挂载）`compaction_publishes=0`，`states_after_the_mount` 却是 6、6、3、1；每次挂载 `txgs_advanced_by_the_mount` 是 3 或 2（量的）。挂载自己的写行与暖机每次占 2–3 槽，环是 3S = 24 槽，8 次挂载才转得满（推的，算术），与整理无关。余量撑的是整理那一份，不是挂载那一份，判决写成「用户接受的边界」的读法对得上这条。
- 判决 `:31` 行末那句「带根臂比对照臂早一次挂载掉（第 4 次 2 个、对照第 5 次 3 个）」逐行对得上：SegmentOfTheInodeRoot 臂第 4 次挂载 `states_after_the_mount=2`（`…/run/readings.out:10`），FullestDataSegment 臂第 5 次是 3（`:5`，量的）。补一条判决没写的：**产品自己选段的臂**（ChosenByTheProduct，`:13`–`:18`）第 1、2 次挂载 `root_relocating_publishes` 各为 1，却没有早掉——第 4、5、6 次挂载是 5、3、1，与对照臂的 6、3、1 只差第 4 次一个状态。带根臂的早掉要强制选 inode 根所在的段才有，产品选段下只差一个状态（量的）。这支持「照旧」，也说明判决可以把「早一次」写成「强制选段下早一次，产品选段下早一个状态」。

### 2.5 S 扫描用例走的是不是产品开关

够不着「走产品开关」一句，射程比用例名说的窄。`crates/singlefs-mount-runtime/tests/mount_runtime_rollback_gate_margin_follows_the_root_ring_slots.rs`：

- 水位取整段数的 100%（`:75` 的 `percent: 100,`），产品是 5%、下限 2（`.claude/kb/decisions/26-后台整理与放置回收.md@已定项 1：整理的触发判据` 里的「全空聚簇段数一条的线取盘上整段数（单元区槽数 ÷ 64）的 5%、下限 2」）。起跑判据走的是 `FollowTheWatermarks` 那一支，只是线被调高；产品默认值下的判法在验证包 `compaction_watermarks_judge_every_branch.rs`（文件头第 4 行自己写了）。
- 选段强制：`:149` 的「switches.forced_region = Some(」，经 `session_for_tests_only()`（`:123` 起 `force_the_fullest_data_segment`）。产品选 greedy（活槽最少的段），强制选「数据单元最多的段」：这是更重的一条意图，对「走得完」是保守的，对「最坏」不是——数据单元一个占 2 槽，一段至多 32 个，`1 + ⌈32÷16⌉ + 删意图` 约 4 次；**文档说的最坏 5 次对应 64 个单槽单元**（`crates/singlefs-core/src/compaction_plan.rs:1532`–`:1533` 的注释），这一族用例一条都没有造它。S=5 时门放 6 次，余 1 次，S=4 放 4 次（<5），所以「S≥5 走得完」对 64 单元的意图靠算术（推的），没有读数。
- 池：「再挂」那一族用 `pool_with_holes_in_the_data_segments_under`（`:325`），最满那一段的单元数没有打出来，推不出它是不是 32。
- 门关那一族的断言把 `slots_per_region` 当成余量写进期望（`:186` 的 `user_states_left_after_the_next_publishes`，调用处 `:284` 的 `slots_per_region + 1`），所以它钉的是「余量取 S」这个决定本身，对「余量该取多少」不构成独立的证据（与 `3S−4−余量` 的算术同一个前提）。判决已经由用户在弹窗里定了 S，这一条只影响「被攻过零轮」的标注，不翻判决。

### 2.6 实现员把探针搬成用例时有没有放宽

三处，都在「覆盖到的情形」上放宽，没有改断言：

1. 底额靠测试开关摆：`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:947`–`:948`（`switches.empty_cluster_segments_floor =` … `EmptyClusterSegmentsFloor::ForcedForTests(SEGMENTS_NO_PUBLISH_MAKES_UP);`），预算靠直接写计数（`:952` 起，`PUBLISHES_SPENDING_THE_BUDGET = 1_000`，`:68`）。攻方探针 `fsync_of_nothing` 也这样摆（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/opus-model/src/attack_probes.rs:700`–`:710`），所以相对探针没放宽；但**满盘上的整条路**（填满、删文件、整理、破例）在修补之后一次都没有重量。
2. fsync 在等靠计数守卫（`a_foreground_fsync_is_waiting()`）而不是真线程：探针量的是 0、1、2、4 个真线程的两秒（`…/run/readings.out:53`–`:56`），用例量的是确定的三趟每趟 1 次（`mount_runtime_compacts_by_itself_and_unmounts_clean.rs:969` 起）。数更紧，并发更弱：破例那次发布持锁时真 fsync 线程排队的情形没有。
3. 开卡器两条（panic 放闸、不持锁建实例）我没有逐条核用例是不是真起了 panic 与真挂住的建实例，只核了判决点名的变异行名存在；写「未核」。

这三处都不翻判决，建议在判决的「被攻过零轮」一栏里带上。

## T1：第一轮的修补本身

### 1.1 余量取 S 槽：打中了 S=5，但没有比过别的取法

代码：`crates/singlefs-core/src/compaction_plan.rs:1537`（「pub fn root_ring_slots_left_by_a_compaction_publish(」）返回每区根槽数；注释 `:1532`–`:1535` 给的数：门放 `2S − 4` 次，一条最坏意图 5 次，一次挂载最多 8 次，「S 更小时一次挂载就可能用完余量」（`:1534`）。判决依据是 `…main-verification.md:25` 的 `geometry_sweep` 与用户弹窗定的「余量取 S 槽」。

我把门放的次数与「一次挂载能保住的余量」按同一个式子（门放 = `3S − 4 − 余量`，注释 `:1532` 与旧值 8 时的 `3S − 12` 对得上，推的）列成一张表，加一个判决里没出现的取法 `min(8, 3S − 9)`（推的，没量过，代入只用文档里的 5 与 8）：

| S | 旧值（余量 8）门放 / 余量 | 取 S：门放 / 余量 | 取 `min(8, 3S−9)`：门放 / 余量 |
|---|---|---|---|
| 4 | 0 / 8 | 4 / 4 | 5 / 3 |
| 5 | 3 / 8 | 6 / 5 | 5 / 6 |
| 6 | 6 / 8 | 8 / 6 | 6 / 8 |
| 7 | 9 / 8 | 10 / 7 | 9 / 8 |
| 8 | 12 / 8 | 12 / 8 | 12 / 8 |

读法（门放要 ≥ 5 才走得完文档里的最坏意图，余量要 ≥ 8 才罩得住一次挂载）：

- 旧值在 S=6、7 已经够走完（6、9 ≥ 5），**取 S 在这两格没有换来走得完，只让「一次挂载不挤掉用户状态」从保得住变成保不住**（余量 6、7 < 8）。需要改的只有 S=4、5。
- 取 `min(8, 3S−9)` 在 S=4 到 8 每一格门放都 ≥ 5，且 S ≥ 6 保住旧值的余量 8；代价是 S=5 门放恰好 5、没有余量，S=4 仍保不住一次挂载。
- 这个替代方案我没有在判决、攻方两份报告、正推腿报告、实现员报告里找到：`grep -n 'min(8'` 在这五份里只命中 `research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-output.md:56` 一次，那是 mkfs 默认 S 的式子，与余量无关（量的）。正推腿在 `research/prompts/m4-gpugc-mount-runtime-code-r1-sonnet-output.md:59` 写过「余量常量 8 的来历……与 S 无关，窗口却随 S 缩，两者不是同一个量」。用户在弹窗里看到的选项我读不到：**如果弹窗给过它，这一条作废**，由主 agent 核问句。

**什么现象会推翻我这一条**：弹窗的选项里有「S ≥ 6 保持 8」；或者主 agent 判一次挂载在 S=6、7 上挤掉用户状态不算事（挂载预算注释 `:1534` 已经写明接受 S 小时用完余量）。

### 1.2 判决表里没有「余量只算 K=16」一行

攻方报告 `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md:25` 列「余量只算 K=16」为打中；判决 `…main-verification.md:16`–`:39` 的逐条表里没有这一行，也没有「不改」的理由。读数 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/run/readings.out:70`–`:74`（`margin_sweep`，批大小 16、8、4、2、1）五行都是 `live_before_the_crash=false … publishes_continuing_at_the_mount=0 … states_after_the_mount=6`（量的）：**探针没有造出崩溃时带活意图的历史，这一问没有读数**。批大小是挂载参数，下界只有 1（`crates/singlefs-core/src/compaction_plan.rs:279`：「pub fn new(units: usize) -> Option<Self> {」，0 才拒），而余量函数只收 S（`:1537`），不收批大小。批大小 1 时一条 64 单元的意图要 `1 + 64 = 65` 次发布（算术，推的），挂载续做只受 `3S − 4` 的预算管，不判门。结论：**修补没打中这一格，读数也没证它不中**，属「没量」。要么判决补一行「批大小小于 16 时余量不够，这一批不修，写进 D26 射程」，要么让探针造出活意图（见「要主 agent 跑的命令」不含此项，需新派）。推的。

### 1.3 破例排在让路之前：抢锁有上界

`crates/singlefs-core/src/mounted_session.rs:4442`（「ForegroundFsync::Waiting if is_this_publish_the_exception => {}」）只放过破例的那一次；`:4443`（「ForegroundFsync::Waiting | ForegroundFsync::CallerIsStoppingItsThread => {」）仍让路，停线程时破例也让。第二次起在同一趟里让路，且低于底额时让路的这一趟不留着（`:3487` 起的 `is_yielding_while_the_foreground_cannot_write`），线程体见 `is_a_trip_under_way` 为假就睡一个 `compaction_period`（`crates/singlefs-mount-runtime/src/lib.rs:605` 起的线程体，默认 30 秒见 `:54`，量的）。所以前台 fsync 高峰期间破例是**每个整理周期一次发布**，不是连着抢。站得住（读码，没量）。会破的前提：调用方把 `compaction_period` 设得很小（挂载参数）；那样破例的速率随它走，不受判决约束，也不挤状态以外的东西——破例不判回退门，每次发布吃一个根槽（`3S` 个槽，S=8 时 24 次就整圈换掉）。这是判决「破例一个周期发一次，不改」那一行已经接受的边界。

### 1.4 每拍先重发：没有退避，害的是日志与次数，不是写放大

重发是「逐字节原样重发」，不涨分配器（`crates/singlefs-core/src/transaction.rs:1611`–`:1613` 的文档；`grep -nF '逐字节原样重发' crates/singlefs-core/src/transaction.rs` 命中 1611，量的）。线程体里没到期那一支也重发（`crates/singlefs-mount-runtime/src/lib.rs:619`：「} else if !self.resend_the_frozen_publish_if_any()? {」），一拍里重发排在 `publish_when_due` 之前（`:522`）。默认 `publish_poll_period` 是 1 秒（`crates/singlefs-mount-runtime/src/lib.rs:56`）。持续写盘报错时：每秒再写一份同样大小的一次发布加一个 `FrozenPublishResendFailed` 事件。**害处有界**（每次只是同一份字节，不累积），判决没给读数也没给退避，实现员报告里提过（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-fixes-writer/result.md` 的「我自己定了的」一节）。站得住；「要不要退避」这一问判决没答，推的：按每秒一次、每次一份发布大小估，盘坏时的写次数是旧写法（30 秒一次）的 30 倍。

### 1.5 卸载先重发：只做了一半的替代

`crates/singlefs-mount-runtime/src/lib.rs:1003`（「if let Err(refusal) = session.resend_the_frozen_publish(&mut devices) {」）重发失败就交回盘、丢会话（`:1004`–`:1008`）。同一个函数里邻近的分支把运行库原样交回（`:1015` 起，`:1016`「runtime: MountedRuntime::of_the_session(」），调用方腾出空间之后可以再卸载一次，卸载先重发。**重发失败时同样可以交回运行库**：会话在 `&mut` 借用下没有被消费，`devices` 也还在，重发失败不改变冻结（`transaction.rs:1616` 的文档：「那次发布照旧冻结着」）。判决选的「交回错误、等价崩溃」把一次瞬时写错加一次重发失败变成必须重挂恢复，用例也是这样钉的（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:1118` 起，盘修好后重挂）。旧写法（卸载直接走核心层，报 `UnmountFailed` 交盘）结局相同，所以修补不比旧写法差，但**没打中「重发失败时也交回运行库」这个替代**。推的，没量。

### 1.6 删除走空放锁：放锁买到的比判决写的少

新函数 `crates/singlefs-core/src/mounted_session.rs:2104`（`publish_at_most_one_step_of_the_queue`）一次只走一步，趟内一次调用至多一步。两次发布之间前台能做的事（读码，没量）：

- `accept` 数据写类：在队列不空时拒（`crates/singlefs-core/src/mounted_session.rs:6423`「if !current_version.queue_of_committed_intents_is_empty() {」，`:6424` 返回 `QueueOfCommittedIntentsNotWalkedBeforeTheAcceptNotSupportedInTheFirstVersion`）。**前台写不是等一步，是被拒一个错**。
- `publish_user_change`：先把队列走空再发这次改动（`:2838` 的文档），前台自己持锁走完剩下的步。
- `publish_pending_now`：待发在趟前已排空，拿到锁是空操作。
- 读：拿得到锁。

所以判决「删除走空整段持锁，不守每次发布之间放锁」打中的是**读与 fsync 的延迟**，不是前台写的延迟。这个队列只在 unlink 自己那次走空被空间拒时才留下（攻方报告 `research/prompts/m4-gpugc-mount-runtime-code-r1-opus-output.md:247`），那时前台写本来就被空间拒。修补站得住，价值比判决写的小。

两份次序：新函数 `:2115`–`:2117` 抄的选下一步的三元组（`deleted_inodes`、销毁中的快照、截断）与 `crates/singlefs-core/src/transaction.rs:12643`–`:12645` 逐项相同（本腿逐行对过，量的）；另一边 `publish_the_queue_until_it_is_empty`（`crates/singlefs-core/src/mounted_session.rs:2083`）只是 `transaction::walk_the_deletion_queue_until_it_is_empty`（`:12630`）的包装。**运行库用例只走 inode 一种队列**（`crates/singlefs-mount-runtime/tests/mount_runtime_compacts_by_itself_and_unmounts_clean.rs:1143` 的 `put_an_inode_into_the_deletion_queue`；用例文件里 `grep -n 'snapshots_being_destroyed\|SnapshotDestruction\|Truncat'` 命中 0 次，量的），销毁中的快照与截断两支在新函数里没有任何用例走过；`history.rs` 里也调用了「至多一次发布」的入口（`crates/singlefs-verification/src/history.rs:10415`，量的），我没核它盖不盖三种队列。推的：把次序下沉成 `transaction.rs` 里的一个函数、两处调同一个，比两份抄的加用例便宜；实现员已提过。

## T3：从未被看过的地方

### 3.1 实现员补的三处

1. **低于底额为 fsync 让路的那一趟不留着**（`crates/singlefs-core/src/mounted_session.rs:3487` 起）：站得住。作用是前台 fsync 一直有时，破例每个整理周期一次（见 1.3）。不留着的代价是这一趟已发的批数计数归零，下一趟从头数；预算计数在会话上（`publishes_since_the_last_user_change`）不受影响。
2. **已起跑的趟接着走删除队列时不受预算挡、不用掉破例**（`:3321` 起的 `is_the_exception_still_open` 与 `deletion_queue_of_the_version_is_empty` 两个条件）：与 D26 的「那一遍删除发的版不占整理一趟的发布预算」一致（`.claude/kb/decisions/26-后台整理与放置回收.md@已定项 13：后台任务的计算放在哪、怎么开、什么节奏` 里的「那一遍删除发的版不占整理一趟的发布预算」）。删除步不判回退门，旧写法（整段走空）同样不判，不是新缺口。实现员自己写了：预算扣完而队列不空时，**新的一趟**仍不起跑（只有已起跑的趟接着走）；我同意这是现状，不在这一批。
3. **开卡器建实例途中丢过的实例不留给下一次开卡**（`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs:198` 的 `instances_forgotten` 比较）：站得住。**判决没说、实现员也没说的一处行为变化**：额度现在在建之前记（`:191`「held.instances_created += 1;」），旧写法在建成之后才记。建实例的函数 panic（不是挂住）时，旧写法不占额度，新写法占一个；额度上限是 4（模块文档），连续 4 次 panic 之后进程内再也开不了卡（`InstanceLimitOfThisProcessReached`，开卡器文档称「进程级不可恢复」）。推的：产品的建实例函数（`VulkanInstanceOfThisProcess::created_for_this_process`）会不会 panic 我没查，所以这是「条件触发的缺口」，不是已知红。判决可以把它记进去：要么 panic 时退额度，要么写进文档。

### 3.2 `run_one_maintenance_pass` 之外有没有绕过放锁的路

运行库 `lib.rs` 里调「至多一次发布」入口的只有一处（`crates/singlefs-mount-runtime/src/lib.rs:534`，`grep -n` 命中，量的）；核心层的整趟入口（`mounted_session.rs:3184`、`:6640`）运行库源码里 0 次调用。绕过放锁的路只剩：调用方自己经 `with_the_session_and_devices`（`lib.rs:927`）借闭包持锁，这是接口本意；挂载时的续做在线程起来之前跑（`lib.rs:660` 的文档），没有线程要放锁；卸载先停线程再走核心层。没找到别的。

### 3.3 `observations()` 持不持会话锁

不持：`crates/singlefs-mount-runtime/src/lib.rs:945`–`:951` 取的是 `self.shared.observations` 那把独立的锁。我只核了 `resend_the_frozen_publish_if_any`（`crates/singlefs-mount-runtime/src/lib.rs:491` 起，更新读数前先 `drop(locked)`）与 `run_one_maintenance_pass` 两处更新读数的位置，没有把全部 `update_the_observations` 调用逐个查是否在会话锁里；写「未全表核」。

### 3.4 `ForegroundFsync::CallerIsStoppingItsThread` 在全部 `match` 上的行为

产品代码里对这个枚举的 `match` 只有一处，穷尽、无通配：`crates/singlefs-core/src/mounted_session.rs:4442`–`:4455`（量的：全仓 `grep -rn ForegroundFsync --include=*.rs crates` 的命中里，别处都是构造、`==` 比较或测试替身；E202 装置 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/command_opening_and_construction_lines.rs:145` 起是 `== ForegroundFsync::Waiting` 的过滤，`crates/singlefs-verification/src/history.rs:10265` 起是返回 `Waiting`/`NoneWaiting` 的替身，都不受新成员影响）。行为：停线程时破例也让，停因照记 `YieldedToAForegroundFsync`（`compaction_plan.rs:622`），这个停因的文档现在写了两种来由（`:620`）。**一个名字两个意思**，见检查单「类型与名字」。

## 检查单

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | **成立** | 9 处逐处对过，每一处都在原文里现取到：运行库 `crates/singlefs-mount-runtime/src/lib.rs:576`（`usize` 到 `u64`）、`:721`、`:987`（两处 `panic!`）；核心层 `crates/singlefs-core/src/compaction_plan.rs:435`（`panic!("16 不是 0")`，在 `const` 上下文 `NonZeroUsize::new(16)` 的 `None` 臂，编不出来）、`:1617`（前面 `.min(整段数)` 在 `:1616`）、`:1801`（只在 `is_above_the_line` 为真时取，`:1791` 起：`空闲段数 × per > above × 已分配`，推出线 < 空闲段数 ⇒ 装得进 `u64`）、`crates/singlefs-core/src/mount.rs:4302`、`:4306`、`:4308`（`usize` 到 `u64`）。运行库 `lib.rs` 与开卡器文件里 `.unwrap()`、`.expect(`、`panic!(`、`unreachable!`、`todo!` 的全部命中就是这三处（`grep -nE` 现取，开卡器文件 0 处，量的）；核心层两份我只扫了改动涉及的行段（`compaction_plan.rs` 1500–1830、`mount.rs` 4190–4330），不说全文件。`:721` 的前提「起不来时线程闭包手里只有接收端」读 `lib.rs:685`–`:731` 对得上（线程拿 `Arc` 要等主线程经通道送，送在 `spawn` 成功之后）；`:987` 的前提「线程 join 过」在 `stop_and_join_the_maintenance_thread` 之后。 |
| 类型与名字 | **不成立（剩两处，低危）** | 对了的：`WatermarkVerdict::Crossed { device, crossing }`（`compaction_plan.rs:1744`–`:1746`，`WatermarkCrossing` 在 `:1716`）三种读数分成 `EmptyClusterSegmentCount`、`FreeRunCount`、`BytesOnOneDevice`，单位真分开；新名字（`publish_at_most_one_step_of_the_queue`、`foreground_cannot_write_under_the_switches`、`deletion_queue_of_the_version_is_empty`、`is_the_exception_of_the_trip_still_open`、`root_ring_slots_left_by_a_compaction_publish`）都说出了内容，没有序数。剩的：① 枚举叫 `ForegroundFsync`，新成员 `CallerIsStoppingItsThread`（`compaction_plan.rs:1517` 起）说的不是 fsync；判决要的是把「兼表停线程」的 `Waiting` 改名，实现员改成拆出新成员（报告写明因为改名要动别的会话在改的 `history.rs`），拆开本身比改名更清楚，但**枚举名没跟着变**。② 停线程引起的让路仍记停因 `YieldedToAForegroundFsync`（`:622`，文档写了两种来由），一个成员名两种意思，没有 fsync 在等时 `last_stop` 也读成「让给前台 fsync」。③ 三个量在 `WatermarkQuantity`（`:1693`，水位强制开关用）与 `WatermarkCrossing` 里各有一份枚举，成员同名；不是错，是两份同形的分类。 |
| 不兼容旧代码与旧格式 | **成立** | `ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH` 在冻结副本 `crates/`、`scripts/` 与仓根的 `.claude/`、`crates/`、`scripts/` 里 `grep -rn` 命中 0 处（量的，只剩 `research/` 的记录，豁免）；补丁里没有版本号、迁移、`legacy` 一类新增（读 `research/prompts/_m4-gpugc-mount-runtime-code-r2-diff.md` 全文，没有）。 |
| 不新建判定库 | **成立** | 对本轮改动涉及的八份文件（运行库 `lib.rs`、开卡器 `gpu_opener_of_this_process.rs`、运行库新用例、真卡新用例、开卡器用例、验证包两份用例、运行库共用搭建）`grep -nE 'rocksdb|sqlite|\.tsv|prune|env::var|/home/|0000:'` 命中 0 处（量的）。`crates/mutations.tsv` 是变异行登记，不是判定库。补丁清单我没有逐份读（读了实现员报告的文件清单）。 |
| 所有任务多机可用 | **成立（新用例本身合规）；wrong-card 不在这一批解决，但要登记并问用户** | 新真卡用例经服务交的卡（`handed_over_gpus_or_report_not_run`），无本机路径与地址（上一格的 `grep` 命中 0 处，量的）。wrong-card：产品开卡函数 `open_background_gpu_with_the_memory_leaked_in_this_process` 的文档写「在本机每一张独立显卡上起设备」「没参与的卡也开（之后的切换可能点它，切换不重开设备）」（`crates/singlefs-background-gpu/src/background_gpu.rs:837`：「/// 没参与的卡也开（之后的切换可能点它，切换不重开设备）」）；新用例走产品构造 `of_this_process`（`crates/singlefs-background-gpu/tests/mount_runtime_thread_on_a_real_gpu_compacts_while_the_foreground_fsyncs.rs:90`：「let opener = GpuOpenerOfThisProcess::of_this_process(OPENING_DEADLINE_BY_DEFAULT);」），已有的真卡用例走 `over_the_vulkan_instance`（`crates/singlefs-background-gpu/tests/mount_runtime_on_a_real_gpu_writes_the_same_block_sequence_as_on_the_cpu.rs:158` 一带），两条路最后都进同一个开卡函数、枚举同一个实例的全部适配器，所以**占用别的卡不是新构造带来的**，背景材料第二节「产品行为不是这一批带来的」我核到了。我没读过已有真卡用例在服务上是否也报过 wrong-card（没有任务记录可读），推不出这一批之前有没有人看到。要不要在这一批解决：不。这是产品语义（开卡时全开、切换不重开），改它是重开 `.claude/kb/decisions/26-后台整理与放置回收.md@已定项 13：后台任务的计算放在哪、怎么开、什么节奏` 第 2 条的实现选择（该条写「点名了就只用点名的卡」，开着没点名的卡是否算「用」条款没逐字答）。出路二选一，交主 agent 或用户：登记一笔欠账（真卡用例会被服务记 wrong-card、同机别的任务多占 202 MiB 量级的显存），或另立一题让挂载只开参与的卡、切换到新卡时再开。 |
| 测试优先多线程 | **成立（按判决点名的范围），一处附注** | 运行库用例文件里起线程的 5 条（`mount_runtime_compacts_by_itself_and_unmounts_clean.rs:739`、`:840`、`:876`、`:1020`、`:1221` 调 `MountedRuntime::mount(`）各有一行「不切片并行」（`:734`、`:836`、`:868`、`:1012`、`:1210`），没起线程的 `:902` 那条也写了（量的，`grep -n` 现取）。真卡用例 `:75` 有。**附注**：开卡器用例文件 `crates/singlefs-background-gpu/tests/gpu_opener_of_this_process_gates_a_hung_opening_and_limits_the_instances.rs` 里起线程的三条（`:47`、`:98`、`:147` 一带，`std::thread::spawn` 在 `:187`）没有这句，`并行` 在文件里命中 0 次（量的）；判决点名的是运行库四条，这份不在其内，推不出是否必须。 |
| 带 --item 的命令与项 | **成立（登记存在）；形式化项名推不出** | 崩溃点重放项登记在 `.claude/gate.d/crash-point-replay-items.tsv:36`–`:40`（`compaction-relocation` 一族与 `deletion-interleaved-with-compaction-across-a-rollback`）、`:73`（`compaction-deadlist-reclamation`）、`:77`（`compaction-with-a-write-while-the-intent-is-live`），名字带 compaction 的项都在（读登记，没跑 `--list-items`）。形式化验证的项名来源我没读，推不出；按实现员报告「项名先 `--list-items`」办。 |
| 新崩溃流多罩了什么 | **成立：不用加流，加一条小用例** | 删除走空放锁与破例前移改的都是「何时发、让不让」，每一步发布本身还是同一批 `publish_unlink_step` / `publish_snapshot_destruction_step` / `publish_truncation_step`（`crates/singlefs-core/src/mounted_session.rs:2119` 起），崩溃枚举罩的是发布产生的盘上状态，不随锁怎么放变。放锁新增的交错里，前台数据写在队列不空时被拒（`:6423`），前台单发改动自己把队列走空，所以不产生「半个队列加前台写」的新盘上形态（读码，推的）。建议的小用例：运行库上先放一个 unlink 进队列、一步之后插一次前台 `publish_user_change`，终态与一次走空的终态逐字节相同（也能钉住 1.6 说的两份次序）。 |

## 要主 agent 跑的命令

一条，探针 `defense_full_pool`（没编过，第一次跑可能要调；编不过或要接着读读数，新派同立场的腿，行首写「承接：`research/prompts/m4-gpugc-mount-runtime-code-r2-sonnet-output.md`」并给任务号）。判决要引的数照三方规则由主 agent 复跑，本腿的读数在跑出之前都是「推的，待主 agent 跑」。

先把模型目录拷进主 agent 自己的草稿目录（`.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算` 核查第 3 条；模型目录此刻在本腿草稿目录 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-sonnet/model/`，入库时放 `research/prompts/m4-gpugc-mount-runtime-code-r2-sonnet-model/`）：

```
cp -r /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-sonnet/model /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model
sha256sum /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model/src/defense_full_pool.rs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model/tools/make_the_small_pool_fixture.py /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model/tools/rerun.sh
```

再交（在仓根起；两台都行，不要卡；脚本只认参数给的路径，服务撞内存上限重排同一条命令时，看到上一次留的记号就清掉 `--work` 再跑，能安全重跑）：

```
task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-run /home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-model
```

跑完看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=`、`exit=`（0 是跑完；1 是编不过或探针 panic，`task-server/task log <号> --all` 找 `panicked at`、`error`、`name=rerun_exit`）；读数在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-main-check-run/readings.out`：应当 1 行 `name=defense_fill`、16 行 `name=defense_arm`（删 1、4 个文件 × 整理 0、1、3、8 拍 × 一单元与 32 单元两种大小）。估时（推的，没量）：填盘照第二次抽样的读数约 100 秒（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r1-main-check/s2-run/readings.out:11` 的 `elapsed_seconds=100.8`，量的），16 个新挂载每个几秒，合计 3 到 5 分钟；盘 320 MiB 两块在内存里，内存估 1 GiB 量级（推的）。

怎么读（跑前写下，读数在后）：

| 读数 | 结论 |
|---|---|
| `size=thirty_two_units`、`passes_before=0` 的行 `accept_went_through=true` 且 `pending_published=[ok published_something=true]` | 删文件本身就让 32 单元的写进得去、发得成：撤回站得住，「整理救的」不成立 |
| 同上 `passes_before=0` 为 false，`passes_before` 1、3 或 8 起变 true | 整理有功劳：撤回站得住，判决补一句「限于整理转得完环」 |
| `thirty_two_units` 全部 false，而 `one_unit` 为 true | 撤回只对一单元的写成立；「运行库在满盘上救得出来」要改成「一单元的写」 |
| `accept_went_through=true` 而 `pending_published` 是 `err …` | 收下发不成：撤回站不住，攻方原判回来 |
| `files_unlinked=1` 与 `4` 的行互相矛盾（4 个比 1 个更难） | 记下，照 `empty_by_the_records` 与 `total_compaction_publishes` 列对照，不当结论 |

没有第二条命令：1.2 的「批大小小于 16 时余量不够」要另造「崩溃时带活意图」的历史，本腿没造，留作下一轮的探针（需新派腿）。

## 没做什么

- 没跑任何代码，没编，没往统一任务服务交任务；本报告里的读数都是第一轮主 agent 量的两份产物与读码，探针的读数一条都没有。
- 没替主 agent 撤销或维持判决，只交复核结果。
- 没读弹窗的问句与选项（读不到），所以「余量取 S 槽」里替代方案是否给过用户，留给主 agent 核。
- 没核开卡器两条新用例是不是真起了 panic 与真挂住的建实例（只核了变异行名与断言的名字）；没逐个核全部 `update_the_observations` 的调用是否在会话锁里；没核形式化验证项名（没跑 `--list-items`）；`history.rs` 里「至多一次发布」入口的调用盖不盖三种队列没核。
- 没读补丁文件清单里的 `crates/mutations.tsv` 全部改动行；核心层 `compaction_plan.rs`、`mount.rs` 的 `expect` / `panic!` 只扫了改动行段，不说全文件。
- 没造「批大小小于 16」的活意图历史（1.2），没量 S=6、7 一次挂载是否真的挤掉用户状态（1.1 的表是算术）。
- 没删文件：草稿目录 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2-sonnet/` 只有模型目录、报告修补用的小脚本与格式检查的空桩目录，没有仓副本与编译目录；交回前 `du -sh` 现取（见交回正文）。
