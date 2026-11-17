# bggpu-audit-fix-code-r1：判决（2026-11-12）

<!-- doc-lint:not-numbers T1 T2 T3 T4 G01 G02 G03 G04 -->

正文 `research/prompts/_bggpu-audit-fix-code-r1-body.md`，开工快照 `research/prompts/bggpu-audit-fix-code-r1-snapshot/`。这是这一题代码轮的第一轮，前面没有判决。

写判决之前拿快照 `sha256sum -c` 过一遍：`crates-sha256.txt` 39 份、`kb-sha256.txt` 全部对得上，腿跑着的时候没有人改过被判的文件。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（Sonnet） | `research/prompts/bggpu-audit-fix-code-r1-sonnet-output.md` | T3 三段 19 份映射着建的缓冲三处口径一致；改小上限的推导成立、组合步不在 Verus；T4 文字上成立；T1、T2 成立；「冲掉待写的空提交超时算不算那第 4 次」「调用方缺陷按 GPU 出错停任务」两处「条款没写」；检查单八格成立，测试多线程另有一处推不出 |
| 攻方（Opus） | `research/prompts/bggpu-audit-fix-code-r1-opus-output.md`，模型 `research/prompts/bggpu-audit-fix-code-r1-opus-model/` | T1 打中一处（切换时量代价不先等还在跑的提交，`hold` 断言 panic；既有的，这一批没碰）；T2 打中一处（选段打分那一段新成员走不到）；T3 三处口径与推导成立，等不到空提交时重交数多算 1；T4 没打中；检查单「走不到的理由」不成立、「测试多线程」推不出 |

核查命令原样：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1 -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py bggpu-audit-fix-code-r1 --out /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1/bggpu-audit-fix-code-r1-tree --repository /home/fy5090/code/singlefs
python3 scripts/agents/cite-check.py research/prompts/bggpu-audit-fix-code-r1-opus-output.md research/prompts/bggpu-audit-fix-code-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/bggpu-audit-fix-code-r1/bggpu-audit-fix-code-r1-tree --background research/prompts/_bggpu-audit-fix-code-r1-background.md
```

- `snapshot-tree.py`（任务 14631，退 0）：快照里的文件都取「工作区此刻那一份」，sha256 与快照相同；这一批没提交，照 diff 与 HEAD 倒推不出，树就是快照那一刻的工作区。
- `cite-check.py` 末行：「✓ 核了 101 处引文、206 处裸行号，对上 81 处，没判 226 处（其中裸行号没有引文可比 171 处；逐处列在上面）」，没有对不上的。没判的是裸行号、只写文件名的、仓外的 wgpu-core 路径与倒推树里没带的几份（`.claude/gate.d/stage-inputs.tsv`、`crates/singlefs-replay-prefix/src/gpu_gc_capacity_proofs.rs` 这类不在快照清单里的）。
- 主 agent 在冻结副本里逐处读过、判决要靠的行（`crates/singlefs-background-gpu/src/` 下）：`background_gpu.rs:2847`–`:2851`（切换时 `measured_cost().is_none() || is_brought_back_after_being_left_out` 就量代价）、`:1686`–`:1702`（量代价照此刻的界切片、直接交校验和那一段）；`memory_limit_and_chunks.rs:457`–`:460`（`hold` 开头断言没有还在跑的提交）、`:538`–`:540`（冲刷等到了交回建缓冲那一次的错、等不到交回等的那一次的错）；`live_slot_kernels.rs:451`–`:455`（整段清单断言）在 `:471` 的 `reconcile_unless_the_batch_failed` 之前；`waiting_for_the_gpu.rs:176`–`:177`（停卡看 `timeouts`）、`:195`–`:200`（等超过上限算交到了卡上）、`:238`–`:240`（重交数加 1）；`crates/singlefs-core/src/background_compute.rs:259`（重复键不归卡）。基线 `~/.local/share/singlefs/work/bggpu-audit-fix-code-r1/baseline/crates/singlefs-background-gpu/src/background_gpu.rs:1931`、`:1933` 同一处已有 `is_brought_back_after_being_left_out` 与无等待的量代价。
- 攻方模型的复跑：模型目录拷到草稿目录、在拷贝里交（任务 14625，退 0），跑前跑后原目录逐份 sha256 相同。读数原样留在 `research/prompts/bggpu-audit-fix-code-r1-main-checks/attack-model-lines-task-14625.txt`（18 行）：`name=apply_model replaced=3 inserted=2`、`test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 201 filtered out`、`name=prediction_check patterns=18 matched_once=17 lines=18`。没对上的一条是历史乙、`user_switch=both_named_large_limit`：预测要 `prefix_batch` 字段里出现 `UnusableAfterResubmissions`，跑出来那个字段打到 `GpuFailed_{_device:_GpuDeviceIdentifier([48,_48,_48,_48,_58,` 就断了，结局 `outcome=panicked message=这张卡上还有一次等超过上限、没做完的提交：交下一段之前先等它做完` 与预测相同；差在打印字段，不在结局。
- 正推腿两条只读核对脚本的复跑（它草稿目录里的脚本，读冻结副本与基线）：任务 14632 把基线按改名表换名后与快照逐行比，25 份里 19 份「含代码=0」，6 份非 0（`admission.rs` 6 块、`block_device.rs` 2、`pointer.rs` 1、`region_scan.rs` 1、`root_ring.rs` 1、`crates/singlefs-reference-filesystem/src/lib.rs` 1），主 agent 读了明细的全部 12 块，都是同一串记号换了折行；任务 14633：`crates/mutations.tsv` 落在 25 份文件上 417 行「原文锚点恰好命中一次 417；不是一次 0」，`crates/singlefs-core/verus-must-red-worlds.tsv` 438 行同、`crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 47 行同。
- 核不动的：暂存在真卡上落在哪一堆、空提交做完之后显存放没放（要新写上卡探针）；改名之后 Verus 在被验文件上还验不验得过（这一轮没交 formal-verification，归提交时那一趟）。两条都不当核过。

## 二、判决

| 格 | 判决 | 依据 |
|---|---|---|
| T1 开卡量代价出错的泄漏（G01 本身） | 站住 | 两条腿读码一致：「还在跑」的标记只有一处来源，判别子、泄漏账与卸载读同一个字段、同一个算式；量代价之外的开卡出口都在任何提交之前。没有模型罩这一句，第二轮接着攻 |
| T1 甲：切换时量代价撞 `hold` 的断言 | 打中（既有缺陷，这一批没引进，落在这一批的文件与 G01 的同一个判别子上） | 任务 14625：两段历史各 5 种用户动作，给那张卡装得下的上限的 2 种 `outcome=panicked`，另 3 种 `outcome=returned`；对照 `name=attack_model_control history=no_running_submission … outcome=returned … card_at_one_measurements=1`。panic 落在核心层两批之间、会话锁里的 `use_the_settings_from_the_next_batch`；用户照报警给的出路再切换一次就走到 |
| T2 重复条目键，校验和与区域扫描两段 | 站住 | 两条腿一致：先整批查、后动计数，CPU 参照不调；到核心层是不归卡的成员、与别的出错同一种停法；不重交 |
| T2 乙：选段打分那一段新成员走不到 | 打中 | 任务 14625：三种重复摆法乘两份核交成或出错共 6 行，都是 `outcome=panicked message=assertion_…丙段交进来的是单元区的整段、按起点升序、一段一个`；`LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch` 与 `background_gpu_compute.rs` 里它那一臂是死成员。用户定的是重复键交回错误，这一段没做到 |
| T3 三段三处口径、改小上限的推导 | 站住 | 两条腿逐份缓冲对上；任务 14625 `name=attack_model_staging_arithmetic fit_cases=200000 fit_disagreements=0 cut_cases=200000 chunks_over_the_limit=0 pieces_not_largest=0 cut_refusals_wrong=0`（只罩显存那一维，绑定上限那一维没取样） |
| T3 丙：等不到冲掉待写的空提交时重交数多算 1 | 打中（只错在报出去的数） | 读码：那一次交回 `WaitForTheGpuExceededTheLimit`，`the_attempt_reached_the_gpu` 算它交到了卡上，`resubmissions` 加 1，而那一段没交出去；同文件 `:178`–`:179` 的注释写的是「重交那一次建缓冲显存不足没交出去的那几回不算」。停卡读的是 `timeouts`，不读这个数 |
| T3 空提交等超时算不算进「第 4 次等超过上限」 | 照字面算进去，不改 | D26（后台整理与放置回收） 已定项 13 第 4 条写的是「累计到第 4 次等超过上限就报这张卡用不了」；那一次确是一次等超过上限。两条腿都报字面两读，主 agent 取字面这一读，列进交用户的 |
| T3 建到一半显存不足那一支在真设备上的行为 | 没量过 | 只有真设备走得到；读 wgpu-core 30.0.1 源码两条腿一致（空提交带走待写、提交做完才放暂存）。这一格最脆弱，留给第二轮与真卡探针 |
| T4 只改名的 25 份 | 站住 | 两条不共用代码的路一致：主 agent 的记号流量法（`research/prompts/bggpu-audit-fix-code-r1-main-checks/rename-only.out`，25 份「只改名」）与正推腿的换名后逐行比（任务 14632）；登记表锚点任务 14633 都恰好命中一次。丁：四处说明文字还写旧名，不进行为 |
| 检查单：走不到的理由二次查实 | 不成立 | 乙那个死成员走不到而没写理由；`hold` 的断言在甲那条路上走得到 |
| 检查单：类型与名字 | 不成立（存量写法的延续） | `ChunkFootprint::bytes_held_by_a_chunk(&self, input_bytes: u64, entry_count: u64) -> u64`：字节数与条数两种量共用裸 `u64`；这一批新加的几个函数与两个系数常量照旧。本包的切批与记账在这一批之前就整套是裸 `u64`，不是这一批引进的；列进交用户的 |
| 检查单：不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、带 --item 的命令与项、新崩溃流多罩了什么 | 成立 | 两条腿一致；12 行新变异原文各命中一次 |
| 检查单：测试优先多线程 | 成立 | 新用例都是单场景；正推腿点的 `greedy_chunks_match_independent_prefix_search` 是别的会话留下的契约测试，这一批只改了它的参照算式 |

## 三、改法

都是腿提的或主 agent 推的，零轮。A/B 里没有改法的那一臂是任务 14625 的读数；实现员把模型用例收成回归用例，改完之后在同一批输入上读数翻面才算换到东西，读数的任务号由合入后验证补在各条里。动手等管 C996 的那个会话的 `c996-code-r2` 判决也写完，两边合成一批改。

1. 甲：`GpuInUse` 量代价开头先在这一组设置给的等的上限之内等账上还在跑的那次提交、放账（`room_once_the_submission_running_past_its_wait_completed`）；等不到就照量代价出错交回，这张卡不参与、报 `CostNotMeasuredAtTheSwitch`。回归用例取模型两段历史各 5 种用户动作：原来 panic 的 4 行翻成不 panic。零轮。
2. 乙：选段打分那一段在整段清单断言之前先查重复的段，重复就交回 `LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch`，计数不动；不重复而与整段清单对不上的照旧断言。回归用例取模型 6 行：翻成交回出错。零轮。
3. 丙：冲掉待写的空提交等不到时交回一种单独的原因，算进超时数、`the_attempt_reached_the_gpu` 判它没交到卡上；不改停卡的时刻。补一条用例钉重交数。零轮。
4. 丁：不改。现查这四处：`crates/mutations.tsv` 第 4338、4339、4342 行与 `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv` 第 198 行里的旧名都在第一列，那一列是变异行与必红世界的项名（`--item` 点的就是它），点名测试的那一列与 `landing=` 已是新名；换项名要连带换掉按项名引它的记录，换不来行为上的东西。
5. 重复键到了核心层只放弃这一批（用户 2026-11-12 弹窗定）：`BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch` 不走停下任务、报警那条路，这一批作废、计数不动、后台任务不停、不报警，下一批照常算；补用例钉「重复键之后任务没停、下一批照常」，补变异行。零轮。
6. 本包切批与记账里的字节数、条数改成各自的类型（用户 2026-11-12 弹窗定「这一批一起改」）：核心层已有 `GpuMemoryInBytes`；`ChunkFootprint` 的字段与方法、三段的占用、外壳的记账与结账、两个份数常量的用处逐处换，测试与变异行的锚点跟着改。排在改法 1、2、3、5 与 C996 那一批合入之后单开一批（两批都动同一批文件，不同时写）。
7. 写回 kb：D26（后台整理与放置回收） 已定项 13 第 8 条补一句——一批里同一个条目键出现两次，这一批作废、计数不动，核心层收到一种不归哪张卡的出错，后台任务不停、不报警，下一批照常；变更史记一条，原话是用户 2026-11-11 弹窗「改成交回错误」与 2026-11-12 弹窗「只放弃这一批」。

## 交用户的

- 重复键到了核心层是停下后台任务并报警（与 GPU 出错同一条路），还是只放弃这一批、下一批照常：现行代码是前一种，条款没写。用户 2026-11-12 弹窗定「只放弃这一批」，改法 5。
- 字节数与条数共用裸 `u64`：本包切批与记账整套如此，这一批照旧。用户 2026-11-12 弹窗定「这一批一起改」，改法 6。
- 主 agent 取的读法，可以推翻：冲掉待写的空提交等超过上限，算进「第 4 次等超过上限就报这张卡用不了」的累计。
- 知道一下：甲是这一批之前就有的缺陷，用户照报警的出路再切换一次就能让维护线程 panic；改法 1 是零轮，第二轮攻它。

落在别的题里的（两条腿单列的四处，发给管 C996 的那个会话，不计进这一轮）：挂载那一刻开卡线程 panic 没接住（接住的只包切换开卡那一路）；挂载失败后紧接着重挂，前一次量代价线程晚到的泄漏新挂载读不到；开卡线程到点之后挂在驱动里不回，账里没有这张卡；wgpu-core 的 `Queue` 析构对设备丢失之外的错 panic。

## 这一轮被判过的文件

- crates/singlefs-background-gpu/src/background_gpu.rs：判了量代价出错的泄漏处置与记账、三段的记账与真建尺寸断言、建缓冲出错交空提交那一支；甲打中在这一份
- crates/singlefs-background-gpu/src/background_gpu_compute.rs：判了重复键报给核心层的成员与位置；乙的死臂在这一份
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：判了暂存的份数常量、切批判装不下与最大一片的折算、建缓冲出错的结账，对得上
- crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs：判了先整批查重复键、后动计数，位置对得上
- crates/singlefs-background-gpu/src/checksum_kernels.rs：判了校验和一段每份缓冲的份数，三处同一个数
- crates/singlefs-background-gpu/src/region_record_kernels.rs：判了区域扫描头片与映射片每份缓冲的份数、映射三张表连同暂存的字节数，三处同一个数
- crates/singlefs-background-gpu/src/live_slot_kernels.rs：判了选段打分一段每份缓冲的份数与记录表每片条数的除数，对得上；乙打中在这一份
- crates/singlefs-background-gpu/src/gpus_of_this_host.rs：判了泄漏时丢与留的分法、进程泄漏账按次累加差额，对得上
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了开卡线程外面那一层到点之后的账、超时数与重交数的数法；丙打中在这一份
- crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs：判了文件头第 13–14 行「开卡途中量代价出错而提交还在跑的卡在开那张卡的线程上已经泄漏、记进泄漏账」与现行代码对得上
- crates/singlefs-background-gpu/src/lib.rs：判了包文档里显存与切批、两份核比对那几句与这一批的落点对得上
- crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs：判了三处开卡调用跟着签名改、总线地址字面量与基线相同
- crates/singlefs-core/src/background_compute.rs：判了 `BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch` 不归卡、与别的出错同一种处置
- crates/singlefs-mount-runtime/src/lib.rs：判了这一批只动的开卡超时成员那一段文档，期限的形状（枚举期限、枚举完按卡数延）与开卡器的算式对得上
- crates/singlefs-core/src/admission.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 6 块折行
- crates/singlefs-core/src/block_device.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 2 块折行
- crates/singlefs-core/src/bytes.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/checksum.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/code_two_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/container_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/deleted_inodes.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/inode_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/journal.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/lineage_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/logged_operations.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/pointer.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 1 块折行
- crates/singlefs-core/src/records.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/region_scan.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 1 块折行
- crates/singlefs-core/src/retirement_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/root_record.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/root_ring.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 1 块折行
- crates/singlefs-core/src/tombstone_tree.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/truncate.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/unit.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/unlink.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-core/src/write_request_split.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-reference-filesystem/src/lib.rs：判了相对提交 8ed2ea484 只改标识符，换名后只差 1 块折行
- crates/singlefs-replay-prefix/src/compaction_plan_proofs.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同
- crates/singlefs-replay-prefix/src/unlink_proofs.rs：判了相对提交 8ed2ea484 只改标识符，换名后逐行相同

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-12 改了：第 8 条补重复条目键这一批作废、任务不停那一句，随这一轮的改法一批写回 |
| D24（后台重活能不能卸给 GPU） | 不影响 | 2026-11-12 不受影响：这一轮只判泄漏、重复键与显存记账，不碰卸不卸给 GPU 的判据 |
