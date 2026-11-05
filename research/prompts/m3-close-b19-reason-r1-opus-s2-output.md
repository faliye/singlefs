# m3-close-b19-reason-r1 攻方腿第二次抽样（opus-s2）：Q1 第 1 问与 Q3 窄方向（2026-10-30）

攻击面只取派发给的两处：Q1 第 1 问（今天的写者上「记录在、根没落、判法认 u 落了盘」造不造得出，含正文 Q1 第 2、3 问里挂在它下面的部分）与 Q3「记录比单元窄」那个方向。Q2、Q3 宽方向不攻。从头独立做，没读禁读清单里的四样。

## 复跑

模型全是 python 小模型（照开工快照代码手抄的槽级模型，推的，不是编出来的二进制），不编 cargo、不读冻结副本。复跑（工作目录随意，`--work` 给一个新目录；`--frozen` 照模板要求给一个目录，这一份不拷它）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-reason-r1-opus-s2-model/tools/rerun.sh --repo <仓根> --frozen <任一目录> --work <新的草稿目录>
```

它跑 14 份输出、逐份与 `out/` 里同名的比，末尾打 `name=rerun_compare item=<名> same=true|false`，有一份 false 退 1。交回前在新 shell 里整份跑过一次（本机，`nice -n 19`，4G 上限）：退 0，`same=true` 14 行、`same=false` 0 行。

| 文件（相对模型目录） | sha256 |
|---|---|
| i31_rules.py | 372a546bfd05baacdfc8b6ce607dc6428cb3e2d0838b43eed823992ac7b5780f |
| i31_mirror.py | 02f226f398944256a392a70ae7925a8c5bd571f42ebe9feeb059c85110c871a2 |
| i31_sweep.py | 3ed70ebbd8b0d4f61bc5a7015da549eb93d57398e161449c02cfb556af346c0b |
| q1_model.py | 42f563fd7597dd276ab87f0342e5fa4128eb62eb8bd96e6ed58d568b71d2c9c4 |
| tools/rerun.sh | 43a330ce6dfd37cf056dd8e9493256da6636a54752f4175fc34c68a4b0443c24 |
| out/i31_mirror_none.out | a4652118e318618de0a7a2664dedacf07cc9899dc06f0e1cda7851d0352852d7 |
| out/i31_mirror_greater.out | b8d7ffb6381447b7764fa27cb24db4687cc46b2d4092f42a9c2f2d05e3814af4 |
| out/i31_mirror_per-slot.out | 54e5d686c3892bd951739a113bb928371ccb2642d797606a351ef93a017701f4 |
| out/i31_sweep_4.out | 2b900d58a30274931ff79d43107a74216d91403d62e39b068fa2a0058189f93e |
| out/i31_sweep_5.out | 56f16c19682ed7411bdf25877c9d8a49f97d29a28aac14a35888bf11fb49d541 |
| out/q1_r1_s3.out | 260dae8030eabbe16e990cfb8adc4dab3b02438f3e7e598f6e2600d81ee43f5c |
| out/q1_r1_s4.out | fded08d0d2880a5e23b0af48903dfbd60bf717dac10870478c6efcc69fc24219 |
| out/q1_r2_s2.out | eb8ac40c4f3de970243ef338899e22067b61c531079192a0c24a16ad25db2791 |
| out/q1_r3_s2.out | 5272adbd51baaab5c590a534208bc1f49becad82f423f3a00baced6d868c64b1 |
| out/q1_r1_s3_four_mounts.out | 411a478e6b0bea77ea410781ef97d20811c20c96ca7bc8f2ced68aa7b087ba17 |
| out/q1_break_no-T-check.out | efe058535034479dd2b41f43f733d7d15391c1fdcdd30f54349d36d04e7fc7a6 |
| out/q1_break_first-txg-as-row.out | f65c65272bb08e4d59cbb295c1ab1ec704815d5132af34ef0ae9b5ac8d1173ac |
| out/q1_break_zero-unit-continues.out | f1b826df76f8efbe744bb2f67bc425de14ab6e1ce4b8089b082d9d5e04256da7 |
| out/q1_break_no-freeze.out | c58a4531e652622ffa8a53ed671be2ae2c2484054d0ae9f56ec9b9d1e6c7703b |

被判对象核过与开工快照一致：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/{transaction,mount,allocator}.rs`、`crates/singlefs-checker/invariants/05-空间记账.md` 此刻的 sha256 与 `research/prompts/m3-close-b19-reason-r1-snapshot/` 两份清单逐个相同（命令 `sha256sum`，本次开工时跑的）。下文代码行号都是这几份文件的。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 第 1 问：根槽写失败（冻结、原样重发）这一支 | 没打中（造不出） | 同一实例下一版之前 u 的根 FUA 必须返回成功；FUA 返回成功之后 u 的根已持久，再往后的写都在它之后 |
| Q1 第 1 问：零单元发布失败不冻结这一支 | 没打中（造不出） | 零单元发布只在挂载的写行与暖机里走，失败整个挂载报错，同一实例没有下一版 |
| Q1 第 1 问：准入推空发布、正常卸载推空发布、整理发布 | 没打中（造不出） | 都经分配器那条发布路径，冻结着时在任何读写之前拒 |
| Q1 第 1 问：「≤ k_tol 次根槽写失败」在写者里对应哪段 | 没有对应的代码 | `crates/singlefs-core/src/` 里 k_tol 只出现在三处注释里，没有一段跳过失败的那一版 |
| Q1 第 2 问挂在第 1 问下的那一句（「u 的记录落了、u 的根没落」之后 T = u + 1 > u） | 打中正文的前提（推的 + 模型量的） | T = u + 1 是新实例的第一个 txg，不是 u 那个实例的行；u 那个实例的行 T 取施加之后那一版的 txg，等于 u 或更小，判法在 u < T 那一判上报认不出，不认作落了盘。这一形造得出，但判法挡住了它，缝为空 |
| Q1 第 3 问（钉槽 p 认不认得出） | 不需要钉槽 | 第 2 问那一形下一次挂载时那一槽读得出（旧根或全零），不在钉的射程里；挡住它的是行 T 那一判 |
| Q3 窄方向 | 打中（槽级模型量的） | 一条窄记录 + 一条罩着全零槽、没人引用的未释放记录（泄漏）：按记录自述 I-3.1 只在 (P) 上不成立，取宽后成立报部分没判；I-3.18 只红窄的那条；泄漏那一槽 I-3.15、I-3.18、I-3.10 都没有对象，I-3.11、I-3.16、I-5.2、I-5.4 都成立。把 `!=` 改成 `>` 对这一形不起作用 |

## Q1 第 1 问：今天的写者上「记录在、根没落、判法认 u 落了盘」

### 判法认 u 落了盘靠的那一句

「let its_root_landed = version.instance == raising_instance」（`crates/singlefs-checker/src/walk.rs:9033`）与「.all(|row| overwritten_txg < row.published_checkpoint_txg));」（`crates/singlefs-checker/src/walk.rs:9037`）。行 (i, T, W) 的 T 由可写挂载写：「selected_root_txg: effective_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:9232`），也就是恢复择根、施加记录之后那一版的 txg，写的是所选根那个实例那一行。

### 写者的落盘次序（逐条路径共用）

记录写完一道屏障：「writer.perform_commit_step(CommitStep::Barrier)?;」（`crates/singlefs-core/src/transaction.rs:1494`）；然后根槽 FUA：「writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {」（`crates/singlefs-core/src/transaction.rs:1637`），设备写带「WriteDurability::ForceUnitAccess,」（`crates/singlefs-core/src/transaction.rs:562`）；轮换之后再一道屏障才返回：「writer.perform_commit_step(CommitStep::Barrier)」（`crates/singlefs-core/src/transaction.rs:1645`）。三条发布路径与重发共用这一处（`persist_publish_writes` 只在 1571、1602、1672 三处被调，本次 `grep -n 'persist_publish_writes(' crates/singlefs-core/src/transaction.rs` 现查）。

### 逐条路径

| 路径 | 结论 | 挡住它的那一行 |
|---|---|---|
| 落盘几步里任一步失败 → 冻结 → 原样重发 | 造不出 | 发布路径第一道「fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator) -> Result<(), PublishError> {」（`crates/singlefs-core/src/transaction.rs:1545`），在 1835、9477、10094、12130 四处被调；重发再失败照旧冻结「allocator.freeze_publish(frozen);」（`crates/singlefs-core/src/transaction.rs:1604`）。同一实例下一版发出之前，u 那一次必须整串走到 1645 那道屏障返回成功，u 的根 FUA 已返回成功 |
| 重发「成功」而字节没落盘 | 造不出（在本仓的设备模型里） | FUA 写返回成功就是持久（1637 那一步带 `ForceUnitAccess`）；再往后的写全在 1645 那道屏障之后。设备谎报 FUA 不在模型里（见「限度」） |
| 坏槽：根槽写一直报错 | 造不出 | 落点由 txg 定，重发写的是同一槽（快照 `crates/singlefs-core/src/transaction.rs` 1588–1593 的文档：位置都不变）；一直报错就一直冻结，实例停在 u，下一个实例按恢复写的行 T ≤ u（见第 2 问） |
| 被钉的槽 | 造不出 | 钉值只压回收门槛，不改落点；写者照写那一槽（I-7.19 的条文 `crates/singlefs-checker/invariants/02-根环.md:26` 说的是写掉之后要解钉，不是绕开那一槽） |
| 零单元发布失败不冻结 | 造不出 | 零单元发布只在可写挂载的写行与暖机里走，失败整个挂载报错：写行那一次在 `crates/singlefs-core/src/mount.rs:8103` 以 `?` 交回（原文见表下），暖机那一串「let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(」（`crates/singlefs-core/src/mount.rs:8120`）后面同样以 `?` 交回。挂载报错就没有会话，同一实例不再发；下一次挂载取新号 = max(系统配置里的号, 根环里的号) + 1：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:1007`），取号写两轮都落盘之后才有第一次发布，号不会重用 |
| 准入推空发布、正常卸载推空发布、整理发布 | 造不出 | 都走经分配器的发布路径（1835 / 12130 那两处冻结闸之后才到 `persist_the_publish_or_freeze_it`），与第一行同 |
| 「一次处置里根槽写失败 ≤ k_tol 次」 | 写者里没有对应的代码 | `grep -rn 'k_tol' crates/singlefs-core/src/` 现查只命中三处注释（`crates/singlefs-core/src/mount.rs:4450` 准入发布次数 B、`crates/singlefs-core/src/root_ring.rs:347` 与 `crates/singlefs-core/src/recovery.rs:501` 的 S 下界）；准入上限是常量「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」（`crates/singlefs-core/src/mount.rs:4453`）。没有一段跳过失败的那一版；D23 已定项 14 把「根槽写失败推进 txg」明写成后面的里程碑 |

写行那一次失败的那一行：「.map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;」（`crates/singlefs-core/src/mount.rs:8103`）。零单元发布的另一个调用方是 mkfs 同一个进程里那条流的 `transaction::warm_up`（不经挂载）：产品路径里没有调它的，`crates/singlefs-core/src/mount.rs` 里唯一一处在 `#[cfg(test)]` 之后（12583），harness 里两处都在失败时交回：「.map_err(|error| (StartingPointStep::WarmUp, format!("{error:?}")))?;」（`crates/singlefs-harness/src/history.rs:2733`），`crates/singlefs-harness/src/scenario.rs:186` 同样 `map_err`。

D23 那一句原文：「**这一版的失败处置**：发布不接受失败。一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布；于是合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录，恢复遇到它就是盘坏了，在任何写之前停下（`crates/singlefs-core/src/recovery.rs` 的 `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）。原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做。原样重发在串行提交下会把一次失败放大成整条发布流阻塞，欠 C541（原样重发把一次失败放大成整条发布流阻塞）。」（`.claude/kb/decisions/23-journal的角色与格式.md:409`）。D16 已定项 1 射程里 k_tol 那一条：「- **最少保留 4 个状态只在一次处置里根槽写失败 ≤ k_tol 次时兑现**：S = 4、c₀ = 5、3 次失败那一格违反 24 次，按字面记一次输、按 k_tol 的射程在外；全枚举下 S = 4、c₀ = 1 也违反 160 次（同为超过容忍的格）。」（`.claude/kb/decisions/16-发布语义.md:60`）。它说的是保留状态数的容忍度，写者里不对应任何「跳过」。

### 第 2 问挂在这里的那一句：T = u + 1 是哪一个实例的

正文写「所以「u 的记录落了、u 的根没落」的崩溃态之后新实例的 T = u + 1 > u」。`first_txg_of_new_instance` 算的是新实例自己的第一个 txg：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1420`）。判法查的是 **u 那个实例**（旧实例 i）在 r 的实例表里的行，那一行由新实例在 9232 那一句写成施加之后那一版的 txg：u 的记录被施加时 T_i = u；u 的记录没被施加（它是实例 i 的第一条、跟在别的实例的根后面，前缀不跨实例；或点名单元验不过）时 T_i < u 或那一行是 (i, 0, 0)。三种情形下「u < T」都不成立，判法走「根槽落没落盘判不了」那一支，整条报部分没判，不认 u。

这一形造得出，第一个崩溃点重放用例的形态（R = 1、S = 3 的模型几何；第一版几何同理，只是要多绕一圈）：mkfs → 第一次可写挂载（实例 1：写行 txg 1、暖机 txg 2）→ 挂着时发 txg 3：记录与屏障落了、根槽 FUA 之前崩（或 FUA 报错、冻结、进程退出）→ 第二次可写挂载：所选根 (2, 实例 1)，施加 (实例 1, txg 3) 那条记录，写行 (1, T = 3)，新实例 2 从 txg 4 起 → 实例 2 发 4、5、6，6 盖掉 (0, 0) 那一槽。抬 F 的根取 4、5 或 6 中带新 F 的那一条时，(0, 0) 那一槽镜像上住着 6，落点式取 u = 3，journal 里只有实例 1 一版，3 < 3 不成立 ⇒ 报认不出；真住户是第 0 代根。模型里这一形记成 `caught_record_without_root`，见下。

### 模型（`q1_model.py`，推的）

把写者、恢复、判法按上面那几行抄成状态机，穷举每次发布（与重发）的八种结局（成功、根槽报错没落、根槽报错实际落了、记录报错没落、记录报错实际落了、记录之前崩、记录之后根槽之前崩、根槽之后崩），冻结之后用户可选重发或退出，挂载时暖机 1 或 2 次、挂载次数与故障次数有上限；每一个节点都当一张崩溃镜像判：镜像上每条根 r、每一槽，判法认出的住户与「r 的根落盘前一刻那一槽的真住户」比。复跑那一份（`out/` 原样）：

- 「name=q1_model R=1 S=3 attempts<=7 faults<=3 mounts<=3 break=none images=223094 roots_judged=620274 slots_recognised=1179240 mismatch=0 caught_record_without_root=129246 caught_new_instance_first_txg_is_u_plus_one=129246 caught_row_of_u_instance_at_most_u=129246」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_r1_s3.out:1`）
- 「name=q1_model R=1 S=4 attempts<=6 faults<=3 mounts<=3 break=none images=100465 roots_judged=304427 slots_recognised=637502 mismatch=0 caught_record_without_root=31164 caught_new_instance_first_txg_is_u_plus_one=31164 caught_row_of_u_instance_at_most_u=31164」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_r1_s4.out:1`）
- 「name=q1_model R=2 S=2 attempts<=6 faults<=3 mounts<=3 break=none images=100465 roots_judged=304427 slots_recognised=637502 mismatch=0 caught_record_without_root=31164 caught_new_instance_first_txg_is_u_plus_one=31164 caught_row_of_u_instance_at_most_u=31164」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_r2_s2.out:1`）
- 「name=q1_model R=3 S=2 attempts<=6 faults<=3 mounts<=3 break=none images=100465 roots_judged=335403 slots_recognised=1217420 mismatch=0 caught_record_without_root=0 caught_new_instance_first_txg_is_u_plus_one=0 caught_row_of_u_instance_at_most_u=0」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_r3_s2.out:1`）——环长 6、6 次发布绕不满一圈，这一格只证不误认，没碰到那一形
- 「name=q1_model R=1 S=3 attempts<=6 faults<=2 mounts<=4 break=none images=48937 roots_judged=135698 slots_recognised=239381 mismatch=0 caught_record_without_root=27384 caught_new_instance_first_txg_is_u_plus_one=27384 caught_row_of_u_instance_at_most_u=27384」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_r1_s3_four_mounts.out:1`）

读法：`mismatch=0` 是判法认出的住户与真住户逐槽相同；`caught_record_without_root` 是「u 有记录、真住户不是 u」而判法报认不出的槽数，这些格上「下一个实例的第一个 txg = u + 1」与「u 那个实例的行 T ≤ u」两个计数与它逐个相等——正文那一句里的 u + 1 是对的数，挂错了实例。

模型会红（弄坏开关，同一份 `q1_model.py`，R = 1、S = 3、6 次发布、2 次故障、3 次挂载）：

- 判法不查行 T（`BREAK=no-T-check`）：「name=q1_model R=1 S=3 attempts<=6 faults<=2 mounts<=3 break=no-T-check images=28273 roots_judged=76022 slots_recognised=163949 mismatch=13584 caught_record_without_root=0 caught_new_instance_first_txg_is_u_plus_one=0 caught_row_of_u_instance_at_most_u=0」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_break_no-T-check.out:1`）
- 旧实例那一行照正文那一读写成新实例的第一个 txg（`BREAK=first-txg-as-row`）：「name=q1_model R=1 S=3 attempts<=6 faults<=2 mounts<=3 break=first-txg-as-row images=28273 roots_judged=76022 slots_recognised=155183 mismatch=4818 caught_record_without_root=8766 caught_new_instance_first_txg_is_u_plus_one=8766 caught_row_of_u_instance_at_most_u=8766」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_break_first-txg-as-row.out:1`），打中的第一段历史：「example (('mount(inst=1,first=1,row=(0,1),W=1)', 'mount-publish(txg=1):ok', 'mount-publish(txg=2):ok', 'publish(txg=3):root_fail_noland', 'quit', 'mount(inst=2,first=4,row=(1,4),W=1)', 'mount-publish(txg=4):ok', 'mount-publish(txg=5):ok', 'publish(txg=6):ok'), (6, 2), (0, 0), (3, 1), (0, 0))」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_break_first-txg-as-row.out:2`）——根 (6, 实例 2) 那一刻 (0, 0) 槽判法认 (3, 实例 1)，真住户是第 0 代根
- 零单元发布失败之后同一实例接着发（`BREAK=zero-unit-continues`）：「example (('mount(inst=1,first=1,row=(0,0),W=1)', 'mount-publish(txg=1):ok', 'mount-publish(txg=2):ok', 'quit', 'mount(inst=2,first=3,row=(1,2),W=1)', 'mount-publish(txg=3):root_fail_noland', 'mount-publish(txg=4):ok', 'publish(txg=5):ok', 'publish(txg=6):ok'), (6, 2), (0, 0), (3, 2), (0, 0))」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_break_zero-unit-continues.out:2`）
- 挂着时失败不冻结、接着发下一版（`BREAK=no-freeze`）：「example (('mount(inst=1,first=1,row=(0,0),W=1)', 'mount-publish(txg=1):ok', 'mount-publish(txg=2):ok', 'publish(txg=3):root_fail_noland', 'publish(txg=4):ok', 'publish(txg=5):ok', 'publish(txg=6):ok'), (6, 1), (0, 0), (3, 1), (0, 0))」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/q1_break_no-freeze.out:2`）

四个开关各拆掉一道（判法的 T 一判、恢复写行的取值、挂载报错、冻结），每一道拆掉都出 mismatch；四道都在时 mismatch 是 0。后两个开关的历史就是「零单元发布不冻结那一支要是不整个挂载报错」「冻结要是能被绕过」时正文要的那一种历史——今天的写者这两道都在，所以造不出。

### 第 2 问的方向（「谁高」）与第 3 问（钉槽 p）

- 方向：缝为空（判法在那一形上不认 u），重建出的上限与真上限谁高这一判不发生，我没再按 `rollback_floor_ceiling_of_the_valid_roots` 逐项核弄坏开关下的方向。
- 钉槽 p：第 2 问那一形之后下一次挂载读那一槽，读回的是上一圈的根（自证过）或全零，不是「读坏」；`crates/singlefs-core/src/mount.rs` 2922 起的文档只把挂载那一刻读坏或自证不过的槽钉在它可能住过的最旧 txg（读回全零的只钉回收门槛）。判 r 的镜像上那一槽住着 ≥ r 的根，自证过，也不走读不出那一支（「"根环槽（区域 {region}、槽 {slot}）读不出：抬 F 那一刻它住的是什么无从认"」（`crates/singlefs-checker/src/walk.rs:9004`））。钉槽认不出这一形，也不需要它认：挡住它的是行 T 那一判。

### 打中之后的四句（打中的是正文 Q1 第 2 问的前提句，不是判法）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 这一格没有臂；它改的是「丁的前提是推的」那一句该怎么写：前提真正靠的是「行 T 取施加之后那一版的 txg」与「冻结 / 挂载报错」，不是新实例的 first_txg |
| 被判的系统当时看不看得到判别它的东西 | 看得到：判法读的是 r 自己的实例表里 u 那个实例的行，那一行在镜像上 |
| 满足的是哪一句的哪一个分句 | 正文 Q1 第 2 问「所以「u 的记录落了、u 的根没落」的崩溃态之后新实例的 T = u + 1 > u」：数对，主语挂错（u + 1 是新实例的第一个 txg，判法比的是旧实例那一行） |
| 跑前条款给的改法在这几格上还中不中 | 正文没给改法；这一形判法已报部分没判，不需要改法 |

## Q3 窄方向：取宽后成立、真病没被说全的镜像

### 取宽在窄方向上放过什么（按代码推，模型量过）

改记那一步今天两个方向都改：「Some(slots_of_the_class) if slots_of_the_class != record.span_slots => {」（`crates/singlefs-checker/src/walk.rs:10372`）。窄方向上改记那一份 E 是记录自述那一份并上窄记录单元后面那几槽（尾槽），于是 (L) 取两份小者 = 记录自述那一份，不放宽；(P) 放过「遍历走到而只落在尾槽里」的槽（`crates/singlefs-checker/src/walk.rs:10408` 起）；(U) 的上沿是「.max(as_the_unit_classes.slots_counted_at_the_root_reclaim_threshold)」（`crates/singlefs-checker/src/walk.rs:10421`）再「.min(as_recorded.walked_and_still_deferred_slots),」（`crates/singlefs-checker/src/walk.rs:10422`）——|Ω ∪ U| 那一半只按记录自述算、而且是个数。病根在这里：(U) 比的是个数，窄记录让 E 少数一槽，同一块盘上任何一处「记录多罩了一槽」都被它抵掉，按记录自述那一份 (U) 本身就成立；剩下的只有 (P) 在尾槽上不成立，而这一格正是取宽放过的那一格。

### 转写校验

`i31_rules.py` 逐行抄 10279、10336、10360、10391 起的四个函数与 21317–21362 的分支；拿快照里那五条单测（walk.rs 25638 起）的输入复跑，五条都与断言相同：

- 「name=calibration test=record_one_slot_short_..._counting_both_slots expected=(False, True) got=(False, True) match=True」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_none.out:1`）
- 「name=calibration test=walked_slot_outside_every_record_..._stays_red expected=(False, False) got=(False, False) match=True」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_none.out:4`）
- 另三条见同一份文件第 2、3、5 行，`match=True`。

### 最小镜像（槽级，一块盘）

| 项 | 镜像上的样子 |
|---|---|
| 单元区 | 槽 [0, 64)，一个全段 |
| 槽 0 起 | 一个码 1 数据单元（两槽，头用得了），最新根引用它、引用按两槽走到槽 0、1 |
| 槽 63 | 全零（头用不了），没有任何版本引用 |
| 最新根的分配记录 | (槽 0, 跨 1, 未释放)；(槽 63, 跨 1, 未释放)。两条之外没有记录 |
| 记账 | 已分配 2 槽、defer 0、不可回收 0、空闲 = 容量 − 2 槽、runs 1、全空聚簇段 0（照记录口径：槽 1 空、槽 63 占，空闲段是 [1, 63) 一段） |

逐判（θ_high = t，没有已释放记录）：记录自述 E = {0, 63}，|E| = 2；Ω = {0, 1}，U = ∅。(L) 2 ≥ 2 成立；(P) 槽 1 走到而不在 E 里，不成立；(U) 2 ≤ min(2, 2) 成立 ⇒ 按记录自述三判不成立。改记那一份 E = {0, 1, 63}，|E| = 3：(L) 2 ≥ min(2, 3)、(P) 槽 1 在改记那一份里、(U) 2 ≤ min(max(2, 3), 2) 都成立 ⇒ 走「if the_misfit_lies_in_units_the_span_invariant_reddens {」（`crates/singlefs-checker/src/walk.rs:21354`）那一支，I-3.1 报部分没判。

模型输出：「name=world label=mirror_narrow_record_plus_headerless_leak fix=none I-3.1=partial I-3.18=['red', 'no-object'] I-3.15=['holds', 'no-object'] I-3.10=['judged', 'no-object'] I-5.4=holds I-3.11=holds I-5.2=holds I-3.16=holds」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_none.out:6`）

### 镜像上每一条相关不变量与它们的说明

| 不变量 | 判定 | 说明或理由 |
|---|---|---|
| I-3.1 | 部分没判 | 说明点名「遍历走到而不在记录里的槽 [1]」与 I-3.18（21358 那一段 format） |
| I-3.18 | 红，只红槽 0 那条 | 说明照「"{}：盘 {} 槽 {} 的分配记录（未释放）跨 {} 槽，而它罩住的单元头里是码 {}，那一类占 {slots_of_the_class} 槽",」（`crates/singlefs-checker/src/walk.rs:19323`）：盘 0 槽 0 跨 1 槽、码 1、占 2 槽。槽 63 那条起点头用不了，没有对象（条文 `crates/singlefs-checker/invariants/05-空间记账.md:25` 末句的射程） |
| I-3.15 | 槽 0 成立，槽 63 没有对象 | 条文说头用不了的那一格没有对象（`crates/singlefs-checker/invariants/05-空间记账.md:22`）；「有记录没人引用」那一判（G9「已死」）要读得出头才判得到 |
| I-3.10 | 槽 0 判，槽 63 没有对象 | 同上，射程 ③（`crates/singlefs-checker/invariants/05-空间记账.md:17`） |
| I-5.4 | 成立 | [0, 1) 与 [63, 64) 不相交 |
| I-3.11 | 成立 | 已分配 2 − defer 0 = 最新根走读 2（槽 0、1）。泄漏多数的一槽与漏数的尾槽在个数上抵掉 |
| I-5.2、I-3.20 (a) | 成立 | 空闲 + 已分配 (+ 不可回收 0) = 容量 |
| I-3.16 | 成立 | 占着的那一份并集 {0, 1} 两槽 × 16 KiB = 记账的已分配，判得到；全扫 runs = 1（[2, 64)）= 记账 1；全空段照记录口径 0 = 记账 0 |
| I-3.17、I-3.9 | 不适用 | 没有带隔离记号的记录、没有已释放记录 |

**真病**：槽 63 被一条未释放记录占着、没有任何版本引用、不在 defer、不带隔离记号——一槽泄漏，分配器永远不再发它，记账的已分配把它数进去了（多算槽 63）、把单元尾槽 1 漏了（漏算槽 1）。I-3.18 的说明只覆盖槽 1 那一半（槽 0 的记录短一槽）；槽 63 那一半在镜像上没有任何一条不变量的说明点到它。I-3.1 本来就是管这一形的：「/// 遍历没走到的记录罩住的槽（还在 defer 里、写者照算已分配）。上沿的另一半：记账多于它就是记着没人引用、也不在 defer 里的槽（泄漏）。」（`crates/singlefs-checker/src/walk.rs:10271`），取宽之后它退成部分没判。

### 对照（同一份模型，`out/i31_mirror_none.out` 第 7–9 行）

| 盘面 | I-3.1 | I-3.11 | I-3.18 | 读法 |
|---|---|---|---|---|
| 只有泄漏（槽 0 记两槽，记账照记录 3 槽） | 违例 | 红 | 成立 | 泄漏单独出现时 I-3.1 与 I-3.11 两处都红 |
| 只有窄记录，记账照记录 1 槽 | 部分没判 | 红 | 红 | 窄记录单独出现、记账照记录数时 I-3.11 还红 |
| 只有窄记录，记账照单元 2 槽 | 部分没判 | 成立 | 红 | 设计上要放过的那一形，真病就是 I-3.18 说的那一条 |
| 镜像（窄记录 + 泄漏，记账照记录 2 槽） | 部分没判 | 成立 | 红（只红槽 0） | 泄漏被窄记录抵掉，只剩 I-3.18 一红 |

### 小世界穷举（`i31_sweep.py`）

一块盘 N 槽，单元与全零槽的每种排法、每个单元引用不引用、每个槽上记不记录与记几槽（只取不比起点单元宽的）、记账 0..N，全是未释放记录、一个候选版本：

- 「name=i31_narrow_sweep N=4 worlds=21870 partial=128 partial_with_leak=31 partial_with_leak_nothing_else_red=13 partial_dead_record_only=20 partial_clean=77 fix_partial_with_leak=0 fix_partial_dead_record_only=0 fix_partial_clean=77」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_sweep_4.out:1`）
- 「name=i31_narrow_sweep N=5 worlds=236196 partial=861 partial_with_leak=274 partial_with_leak_nothing_else_red=95 partial_dead_record_only=160 partial_clean=427 fix_partial_with_leak=0 fix_partial_dead_record_only=0 fix_partial_clean=427」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_sweep_5.out:1`）

读法：今天报部分没判的盘面里，`partial_with_leak` 张另有「起点头用不了、没人引用的未释放记录」罩着的槽，其中 `partial_with_leak_nothing_else_red` 张 I-3.11 成立、I-5.4 成立、I-3.16 的 runs 照记录口径写也对得上、I-3.15 没有对象——除 I-3.18 之外一条都不红。`partial_dead_record_only` 是没人引用的记录起点头用得了（I-3.15 判它「已死」、有红说明）；`partial_clean` 是记录罩的每一槽都被走到，真病只有窄记录那几槽。估时跑（没经内存包装、没加 nice，不进复跑）N = 6 那一格是 `partial=5448 partial_with_leak=2025 partial_with_leak_nothing_else_red=564 partial_dead_record_only=1111 partial_clean=2312`、改法之后 `fix_partial_clean=2312`、另两类 0。

### 只放窄方向（10372 的 `!=` 改 `>`）够不够

不够，而且方向反了：这一形本身就在窄方向上。`--fix greater` 下镜像照旧部分没判：「name=world label=mirror_narrow_record_plus_headerless_leak fix=greater I-3.1=partial I-3.18=['red', 'no-object'] I-3.15=['holds', 'no-object'] I-3.10=['judged', 'no-object'] I-5.4=holds I-3.11=holds I-5.2=holds I-3.16=holds」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_greater.out:6`）；它只改宽方向那条单测的结局：「name=calibration test=record_wider_than_its_two_slot_unit expected=(False, True) got=(False, None) match=False」（`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_greater.out:3`）。宽方向不在我的攻击面里，这一行只说明 `>` 对窄方向的漏判不起作用。

### 改法（攻方提的，只在我的模型上量过、被攻过零轮）

「逐槽上沿」：取宽那一支另要「改记那一份 E(t) 的每一槽都在 Ω ∪ U ∪ 隔离豁免的槽里」，有一槽不在就照判红、不报部分没判。它只在已经进了取宽那一支的盘面上起作用（没有 I-3.18 红的盘面不进这一支），所以只可能把部分没判改成违例，不碰今天判成立的盘面。

| 改法 | 镜像（窄 + 泄漏） | 五条单测 | 小世界 N = 4 / 5：只有窄记录的部分没判（partial_clean） | 小世界：窄 + 头用不了的泄漏 | 小世界：窄 + 起点头用得了的死记录 |
|---|---|---|---|---|---|
| 今天（`!=`） | 部分没判（量过，`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_none.out:6`） | 五条相同（量过） | 77 / 427 部分没判（量过） | 31 / 274 部分没判（量过） | 20 / 160 部分没判（量过） |
| `!=` 改 `>` | 部分没判（量过，`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_greater.out:6`） | 宽方向那条变成 (False, None)（量过） | 没量（推的：窄方向不变） | 没量（推的：不变） | 没量（推的：不变） |
| 逐槽上沿 | 违例（量过，`research/prompts/m3-close-b19-reason-r1-opus-s2-model/out/i31_mirror_per-slot.out:6`） | 五条相同（量过，`out/i31_mirror_per-slot.out` 第 1–5 行） | 77 / 427 仍部分没判（量过） | 0 张仍部分没判（量过） | 0 张仍部分没判（量过；这一类本来就有 I-3.15 红） |

模型里的 U 与隔离豁免都是空的；真实装置上那两样怎么并进逐槽那一比是推的。合法盘面上「未释放记录罩着、没有候选版本走到、不在隔离里」的槽有没有（例如截断、整理、待删那几条路径的中间版本），我没核——有的话这一改法会在「另有 I-3.18 红」的盘面上把部分没判改成违例，那张镜像本来就已红。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分辨 `!=` / `>` 两臂：两臂在这一形上同中（量过）。病根在两臂共用的「取宽那一支的上沿只比个数」前提上；逐槽上沿那一改法在这一格上不中 |
| 被判的系统当时看不看得到判别它的东西 | 看得到：槽 63 在 E 里、不在 Ω ∪ U 里，checker 手里两份集合都有（`walked_slots`、记录），只是 (U) 拿个数比 |
| 满足的是判据字面的哪一个分句 | 正文 Q3「有没有一张镜像，I-3.1 按记录自述三判不成立、取宽后成立、而把 I-3.1 退成部分没判之后镜像上的真病（统计漏了或多了的那几槽）不被 I-3.18 的红说明覆盖」：三个分句逐个成立（按记录自述只 (P) 不成立；取宽成立；多算的槽 63 不在 I-3.18 的说明里）。也满足第二轮 Q9 条件的反面：配不上的那块（个数上）有一半不落在 I-3.18 判红的单元里，而别的不变量不红 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 正文给的改法只有「`!=` 改 `>`」：在打中的格上还中（量过）。逐槽上沿是我提的、零轮 |

### 扫过、不在 Q3 字面条件里的一形（推的，没进模型的单列计数）

窄记录 A（槽 s 跨 1，单元两槽）的尾槽 s + 1 被另一条未释放记录 B（起点 s + 1，跨 1）罩着：B 的起点是 A 单元的数据后半，头用不了，I-3.18、I-3.15、I-3.10 对 B 都没有对象，I-5.4 看 [s, s+1) 与 [s+1, s+2) 不相交。这时按记录自述 I-3.1 三判就成立（尾槽在 E 里），不走取宽；I-3.18 照红 A，而它的条文把窄记录的后果写成尾槽在盘上没有归属、可被重新分配（`crates/singlefs-checker/invariants/05-空间记账.md:25`，转述），与镜像不符——尾槽有一条假归属 B，A 释放之后 B 把尾槽钉成永久泄漏。这一形不满足「按记录自述不成立」，不计进 Q3 的打中，留给主 agent 判要不要另立一格。

## 没打中的形状

Q1 第 1 问（取样都在 `q1_model.py` 里，复跑那一份的范围：环长 3、4、6（R × S = 1 × 3、1 × 4、2 × 2、3 × 2），每段历史发布与重发合计 ≤ 6 或 7 次、故障 ≤ 2 或 3 次、挂载 ≤ 3 或 4 次、暖机 1 或 2 次；估时跑另到过 8 次发布、3 次故障，也是 mismatch 0，没进复跑）：

- 根槽 FUA 报错而没落、报错而实际落了；记录写报错而没落、报错而实际落了（任一份在即算在）；记录之前崩、记录之后根槽之前崩、根槽之后崩；
- 冻结之后重发若干次（每次重发同样取上面八种结局）、冻结着直接退出、退出之后再挂；
- 挂载里写行或暖机任一次失败（挂载报错）之后再挂，连着几次挂载失败让中间实例的行成 (j, 0)；
- 抬 F 的根取镜像上每一条根（每一条都当抬 F 的根判一次，不按 F 的值筛），每一槽逐个比。

每一种都是 `mismatch=0`；「记录在、根没落」那一形都落在 `caught_record_without_root` 里（判法报认不出）。

Q3 窄方向：除了上面打中的「窄 + 头用不了的泄漏」，扫过的另几种都被说全了——只有窄记录（记账照记录或照单元数）；窄 + 起点头用得了的死记录（I-3.15 有红说明）；窄记录的尾槽被另一条记录罩住（按记录自述就成立，不走取宽，见上一节）。

## 这条腿自己的限度

- 两个模型都是照快照代码手抄的槽级 / 状态机模型（推的），不是在 `crates/` 上编出来跑的；Q3 的镜像没有做成字节级镜像、没有过真的 `check_pool_image`。I-3.1 的转写拿五条单测对过（五条相同），别的不变量（I-3.11、I-3.15、I-3.16、I-3.18、I-5.4）只按条文与判法读法手写，没对拍。
- Q1 模型不含读错（瞬时读错、潜在坏扇区、整块盘掉）、journal 回绕（派发里排除了）、多盘之间一份新一份旧的落盘、管理员回退、实例表清扫、钉槽，也不含「设备对 FUA 谎报成功」；恢复的择根只取环里 (txg, 实例) 最大的根、施加只取同实例接着的记录。读错那一族（例如挂载时 u 的根与记录都一时读不出、新实例取到 ≤ u 的 txg）没攻。
- Q3 的穷举只到一块盘 5 槽（复跑）/ 6 槽（估时跑），全是未释放记录、一个候选版本；θ_high ≠ t、已释放记录、多个候选版本、多块盘、隔离记号都没扫。
- 跑的时候本机负载高（`uptime` 报 load average 163.59，32 线程；`ps` 里有别的会话的十来条 `cargo test -p singlefs-harness`，没有性能测量在跑；`free -g` 可用 45G），加 `nice -n 19` 之后小模型慢二十来倍。有两次前台命令跑过 180 秒被转到后台：一次是估时跑 `q1_model.py 1 3 9 3 4`，一次是第一版生成 `out/` 的脚本（跑到 `i31_sweep.py 6`）；两次都按 pid 逐个停掉最底层的 python（`proc.py stop 121928`、`proc.py stop 305851`），再列一遍确认整条链都退了。之后把复跑缩到上面那一套（缩前：每段历史 ≤ 8 次发布、`i31_sweep.py 6`；缩后：≤ 6 或 7 次、`i31_sweep.py 5`；估法：同样的参数不加 nice 各跑一次计时，加 nice 之后按 `i31_sweep.py 5` 实测 54 秒对 2.3 秒的比例放大，复跑整份落在一次前台调用里）。
- 「推翻条件」：Q1——找到一条发布路径不经 `refuse_while_a_publish_is_frozen` 就落盘、或可写挂载接住写行 / 暖机的错继续发、或恢复写行时 T 取的不是施加之后那一版的 txg，就能造出正文要的那一形；Q3——真 checker 上若有别的不变量判「未释放记录的起点读不出可用单元头」（我在 05-空间记账.md、07-分配器.md 与 walk.rs 的 I-3.15、I-3.10、I-3.18 判法里没找到），或 I-3.11 的走读计数与我按槽数的读法不同，镜像上就多出一条红，打中要降级。

## 要主 agent 跑的命令

没有重型或长活要交。要坐实 Q3 的打中，建议主 agent 照「最小镜像」那张表在入库装置上做一张字节级坏镜像（可以从 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 里 I-3.18 那份坏镜像 `an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_reddens_only_the_span_invariant` 起步：在同一块盘上再加一条罩着一个全零槽、没人引用的未释放记录，记账已分配照记录数），看真的 checker 是不是只红 I-3.18、I-3.1 报部分没判。那是 checker 档之外的 harness 测试，但属于新写的坏镜像，按定义不归这条腿。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；Q2 与 Q3 宽方向按派发不攻。
- 没编 cargo、没跑任何重型测试、没碰 GPU。
- Q1 第 2 问「谁高」那一判没逐项核（缝为空，见 Q1 节）。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r1-opus-s2` 里只有输出与小脚本（`gen1/`、`verify1/` 两份复跑输出、`rerun.sh` 与 `gen_out.sh` 草稿、计时与 cite-check 的文本、一个空的 `frozen-placeholder/`），没有仓副本、编译目录或库拷贝。
