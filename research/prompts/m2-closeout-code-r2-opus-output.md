# m2-closeout-code-r2 云端攻方腿（Opus）报告

轮名 `m2-closeout-code-r2`，格 Y1、Y2、Y4、Y6、Y7（正文第三节）。代码一律读快照 `refs/sop/m2-closeout-code-r2-snapshot`（提交 `30c084138f7da7488f27874417902caeb5272a15`），用例写在快照副本上跑，没碰主工作区。下文「快照 `文件` 第 N 行」都是快照里的行号（取法 `git show refs/sop/m2-closeout-code-r2-snapshot:crates/<路径>`）。跑在 2026-09-27。线程上限 4（两件并行时每件 2），内存上限 8G（`replay.sh` 的缺省）。

## 复跑命令与文件哈希

模型目录 `research/prompts/m2-closeout-code-r2-opus-model/`。复跑：`bash research/prompts/m2-closeout-code-r2-opus-model/rerun.sh <草稿目录> [线程上限]`。它把快照解到草稿目录、拷进用例，逐条经 `capped.sh` 与 `run-with-memory-cap.sh 8G` 跑，再把原型补丁打在第二份副本上重跑 Y2 与 5 份回归。全程约 75 分钟（单故障扫约 28 分钟，Y6 有文件那一族约 30 分钟）。原样日志在 `logs/`。

| 文件 | sha256 |
|---|---|
| `opus_r2_y1_y7_single_fault_ordinal_sweep.rs` | `9b6995b28a6ab184c7e4749c88a4ea383d9508249daa8868a555c89b4c03b2b6` |
| `opus_r2_y2_session_and_entries_with_stale_or_swapped_devices.rs` | `d3cf6cd3e48581b27e91d10b5da6c5c21f433ebd0d0fadbebe7177aaf3a9c975` |
| `opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs` | `08d569d9f9b50c889b9278e9bd1357bd0e7517cc4a0521b6fbbfa0593709a205` |
| `opus_r2_y6_journal_ring_turned_torn_records.rs` | `c9b9fddefd2fa127657c3092b11bb06cab7c0d2176b9c35ea2386dcb55725502` |
| `opus_r2_y7_same_segment_system_configuration_writes.rs` | `6e6daad54274fb41f455bde8f2399e5c085e18b2ad590e7290f680b0d3e984dc` |
| `batch_sweep.sh` | `5008ed13fd5e9dd784f46d5877abb60ab9cfe2a51fc874f6d99793c9232800bd` |
| `rerun.sh` | `acd0b4af8e0327e4c7af695527aa3d51c8f949fc873d97c4352b9cb7608d587e` |
| `analyze_sweep.py` | `a4611f20f541a14187c03f386136b552050365e16d5e9e179fbe7a0829bcabca` |
| `fix_prototype_swapped_or_behind.patch` | `59f3e99b47effe6b091299af6f3b0b4e29c6c8adf3c6baf7f495b0cfc8162f37` |
| `logs/a2a.log` | `3bb07ec1bd5e8314426148bfd2c221fdf72aaa0e132fa67cd55f04f4434fa974` |
| `logs/b1-1.log` | `daee914264a3db9f8f241fb718fea8f473d18aa4dce08d2bbda55dc09fec7341` |
| `logs/b1-1.rc` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `logs/b1-2.log` | `c586394da48e90d0749ef2e6cab0a88c813819f175d520a5e112cffcd180f8b9` |
| `logs/b1-2.rc` | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| `logs/fix-regression.log` | `15e75b85b161d05f4c273010203ce186ae127d18ca7641a49d70a19708cfe2aa` |
| `logs/snapshot-regression.log` | `5e9eef000adab7c78855deab69243ace34c99cfcedc56188ff4a0f8227ddad79` |
| `logs/sweep-probe.log` | `3469d154c2ecad83895477fb62acab667103295b58c295ed0277bf5cc30f2816` |
| `logs/y2-fix.log` | `7e2186842cf43c84a6758dffe305708b8b94a53c33d20f28a33b29f9bee47e27` |
| `logs/y2.log` | `0d558c64d03e3718457c041351466e0cead6cd7313c43ade527731d601b20dbd` |
| `logs/y4.log` | `f736827662c6b4d738731bed2234b91b57a640174bf892c2573bb06f1bcf3d12` |
| `logs/y6-estimate.log` | `ad14613192ba76be686c7ba1cbdca52133e94c72fddb5cf8a167378562bb1f63` |
| `logs/y6-nofile.log` | `142bcfb2f81b58f4da713f9f27b2193e5acfc843cc14ad68c816198041fe4ff4` |
| `logs/y6-probe.log` | `5d36213131ac7f97c9f7d0021f5b9631efca00769e037c6e83a2e1127703771b` |
| `logs/y6-unmount.log` | `7fe83dc29274f72f1c38e0d1852f80e25433d41df30db11644824c86d7a874d1` |
| `logs/z3a-existing.log` | `46a924b73f0a893445c25ebc4c610105cfae0e1d859cde2273e54c29b4aea6e4` |

## 各格判定一览

| 格 | 打中的 / 试的 | 结论（三种之一） | 已知？ |
|---|---|---|---|
| Y2 | **Y2-a**：同池旧快照的盘换进来，会话发布或正常卸载照常做成、把旧盘的系统配置轮换到「跟得上」；之后可写挂载的落后支不再读单元、放它过去（实例 2 做成），池级 checker 红 1–6 条 I-2.1。对照：同样的盘直接可写挂载被拒。用户动作放开扫 16 格，16 格都这样 | 兑现了条款（D18 已定项 11 第 314 行「不核落后那一支」的字面），条款给放行的理由（「单份坏」）罩不住一整块旧盘 | 否 |
| Y2 | **Y2-b**：会话收下两块盘盘体对调的盘表，照常发布、把错的本盘设备号写进两块盘；之后池可写挂不上，池级 checker 0 条违例。正常卸载同一盘表在任何写之前拒 | 替没写的条款做了选择（会话只比身份与「可见」，不比本盘设备号） | 否 |
| Y2 | 只剩一槽自证的盘、别的池的盘、全零空盘（第一轮三形） | 兑现了条款（没打中） | — |
| Y4 | **Y4-a**：core 不读系统配置的根环起点（371）与 journal 环起点（325），取常量；checker 按字段读。根环起点写 0：checker 报 I-7.13 红、自称「实现整池拒绝挂载」，core 可写挂载做成；写 65：core 照挂、checker 报 I-7.1。反方向：417 改 +64、环长改到段边界之外：core 整池拒，checker 的 I-7.13 不红 | 和条款说反话（D22 已定项 16 第 332 行「基址 = 该字段 × 16384」、第 336 行「同一槽自述的根环起点」；I-7.13 第 64 行「与实现整池拒绝挂载一致」在三格上不成立） | 否 |
| Y4 | mkfs 环长不是物理块整数倍；core 读者 panic 面抽看 | 没打中（mkfs 写之前被块层拒）；panic 面只读了代码 | — |
| Y1 | 单故障逐序号扫五个入口 3024 段（含 C577 那道屏障、零单元发布那一支、取号前后的读）：checker 红 0、重挂败 0、写了再拒 0、冻结后重发败 0；零单元发布不冻结、整挂载报错 | 兑现了条款（没打中）；零单元发布那一支是替没写的条款做了选择（D23 已定项 14 第 397 行只写冻结），今天结局无害 | 零单元那一支：正文第二节「已知」C577 报告第 2 条；回退读错报 NotInRing：已知（Q7，在飞，第三轮） |
| Y7 | A2a 同段同盘两次系统配置写：189 条流 0 次；「写了再拒」见 Y1 | 兑现了条款（没打中） | — |
| Y6 | journal 环转圈之后撕开的记录：进了第三态枚举；1 MiB 环两族全量 196665 + 4155 个状态，oracle 与 checker 0 违例，再挂全做成 | 兑现了条款（D13 已定项 4 第 72 行，没打中） | — |
| Y6 | C577 之后关段：根槽 FUA 自成一段、轮换两写由末尾屏障关段 | 兑现了条款（D13 已定项 4 第 72 行字面） | — |
| Y6 | 续跑与分片；模型的「重读仍读不出」那一格 | 只读了代码与现有用例，没另造输入 | — |

## Y2　挂着之后每个入口逐盘核（Z3-A 乙）

用例 `opus_r2_y2_session_and_entries_with_stale_or_swapped_devices.rs`（7 条，全过，原样输出在 `logs/y2.log`）。起点同第一轮：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。

### Y2-a　同池旧快照的盘，经挂着之后的入口「洗白」过可写挂载的落后支（打中，新）

**条款**。可写挂载取号之前的逐盘核有「落后且缺单元」一支；挂着之后的入口按条款只核「可见」那一支，`.claude/kb/decisions/18-块里携带什么信息.md:314`：「不核落后那一支，规格表第 1 行只要这一支」。同一行给可写挂载放行的第二种写的理由是「系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）」（`.claude/kb/decisions/18-块里携带什么信息.md:314`）。

**代码**（快照行号）。挂着之后的入口共用 `caller_inputs_agreeing_with_the_disk`（快照 `mount.rs` 第 3261 行起），会话在 `publish_user_change`（快照 `mounted_session.rs` 第 216 行起）第 242 行调 `devices_without_a_self_verified_system_configuration`（快照 `mount.rs` 第 3028 行起）：只问两槽里有没有一份本池 fsid 的自证槽。落后支只在可写挂载的 `devices_without_the_selected_version`（快照 `mount.rs` 第 2949 行起）里，判「系统配置不落后就不读单元」在第 2976 行。

**历史**（每一步都是条款许可的）：
1. 第一个文件之后留一份盘 1 的镜像（旧快照）；会话接着覆盖写两次。
2. 盘表交 `[(盘 0, 此刻的盘 0), (盘 1, 旧快照)]`。旧快照两槽自证、fsid 是本池、本盘设备号是 1：「可见」。
3. 会话发布一次覆盖写，或正常卸载。入口照常做成，本池的写（含系统配置轮换）落到旧快照上。
4. 拿这两块盘可写挂载。

**原样输出**（`logs/y2.log`）：
```
Y2 control_stale_direct writable err=WritableMountRefusedByDevicesWithoutTheSelectedVersion { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(5) }, selected_version_journal_position: JournalSequenceNumber { instance_generation: InstanceGeneration(1), counter: 5 }, device
Y2 session_stale ok=true err=None stale_disk_written=true
Y2 session_stale checker_violations_after=4 first=Some("I-2.1: 树表单元 在盘 1 槽 50276 的那一份与位置条目里的校验和对不上")
Y2 session_stale writable_after ok instance=2
Y2 session_stale checker_violations_after_writable_mount=3 first=Some("I-2.1: 树表单元 在盘 1 槽 50276 的那一份与位置条目里的校验和对不上")
Y2 unmount_stale accepted=true err=None stale_disk_written=true
Y2 unmount_stale writable_after ok instance=2
Y2 unmount_stale checker_violations_after_writable_mount=3 first=Some("I-2.1: extent 树（树 11）的根 在盘 1 槽 50265 的那一份与位置条目里的校验和对不上")
```
对照那一行：同样两块盘不经挂着的入口、直接可写挂载，被落后支拒。经过一次会话发布或一次卸载，旧快照的系统配置被轮换到「跟得上」，落后支第 2976 行那一判放行、不再读单元，可写挂载做成（实例 2），池级 checker 红 3–4 条 I-2.1：现行那一版的单元在盘 1 上只有旧字节，这个池的冗余少了一份，没有一个入口报过错。

**用户动作放开扫**（`stale_snapshot_substitution_swept_over_the_user_steps`，照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」那一节的写死用户动作那一条）：旧快照取自第一个文件之后或覆盖写 1 之后 × 换盘 0 或盘 1 × 之后会话发 1/2/3 次或正常卸载一次，16 格。16 格对照都拒（`control_direct_writable_ok=false`），16 格入口都做成、之后可写挂载都做成，checker 红 1–6 条。

**四句**：
1. 分不分辨臂：分。「只核可见」（今天，Z3-A 乙）在 16 格都放过；「可见 + 落后支」（原型，见 Y2 末尾的改法表）在 16 格都拒。
2. 系统当时看得到判别它的东西吗：看得到。入口手里有现行那一版（它末条记录的 (实例, 计数器) 与单元），旧快照上最新那一槽的 (实例, tail) 比它小、单元字节不同；可写挂载的落后支就是拿这两样判的。
3. 满足判据字面的哪一个分句：代码兑现 D18 已定项 11 第 314 行「不核落后那一支」的字面；打中的是这句的后果——入口写系统配置把旧盘改成「跟得上」，同一行给可写挂载放行第二种写的理由（「单份坏」）罩不住一整块旧盘。
4. 跑前条款给的改法在这几格上还中不中：第一轮判决第二节的「Z3-A 乙」只核可见，对这 16 格不起作用。

**结论**：兑现了条款（D18 已定项 11 第 314 行的字面），条款的理由罩不住这一格。

### Y2-b　会话收下两块盘盘体对调的盘表（打中，新）

**代码**。会话只比盘表的身份次序（`is_the_device_table_of_the_mount`，快照 `mount.rs` 第 690 行起，「只比身份，不读盘」）再核可见；别的入口经 `caller_inputs_agreeing_with_the_disk` 还比每块盘自证槽里的本盘设备号（`device_table_disagreeing_with`，快照 `mount.rs` 第 3105 行起）。

**历史**：同一起点，盘表交 `[(盘 0, 盘 1 的盘体), (盘 1, 盘 0 的盘体)]`，会话覆盖写一次；再把盘体放回各自的身份挂载。

**原样输出**（`logs/y2.log`）：
```
Y2 session_swapped ok=true err=None
Y2 session_swapped read_only_after chosen=(1,5) effective=(1,6)
Y2 session_swapped checker_violations_after=0 first=None
Y2 session_swapped writable_after err=CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [OwnDeviceNumberDiffersFromTheIdentityHandedIn { identity_handed_in: DeviceIdentity(0), own_device_number_on_disk: DeviceIdentity(1) }, OwnDeviceNumberDiffersFromTheIdentityHandedIn { identity_handed_in: DeviceIdentity(1), own_device_number_on_disk
Y2 unmount_swapped accepted=false err=Some("CallerParametersDisagreeWithTheSelectedSystemConfiguration { disagreeing_fields: [], disagreeing_device_table: [OwnDeviceNumberDiffersFromTheIdentityHandedIn { identity_handed_in: DeviceIdentity(0), own_device_number_on_disk: DeviceIdentity(1) }, OwnDeviceNumberDiffersFromTheIdentityHandedIn { ident"
```
会话照常发布：系统配置轮换把「本盘设备号 0」写进盘 1 的盘体、「1」写进盘 0 的盘体，txg 6 的根槽写在了不归它的那块盘上（只读择根只到 (1,5)，(1,6) 靠重放）。之后盘体放回原位，这个池可写挂载被拒（两块盘都报本盘设备号不符）；池级 checker 0 条违例——checker 读本盘设备号（快照 `singlefs-checker/src/lib.rs` 第 265 行），没有一条不变量拿它比盘的身份，这一格 checker 看不出（越格线索，归 Y5）。同样的盘表交给正常卸载，在任何写之前被拒。

**四句**：分辨臂（会话那一判与卸载那一判在同一盘表上结局相反）；看得到（会话已经为「可见」读了每块盘两槽，本盘设备号就在里面）；满足的分句：kb 没写会话要不要比本盘设备号，第 314 行只写「可见」；Z3-A 乙对这一格不起作用。

**结论**：替没写的条款做了选择（会话不比本盘设备号，别的入口比），影响：会话把错的本盘设备号写进两块盘的系统配置，池从此可写挂不上；今天没有会红的东西钉着这一格。

### Y2-c　只剩一槽自证的盘、第一轮三形（没打中）

- 盘 1 较新那一槽写坏一个字节：会话照常发布、checker 0、可写挂载做成（`Y2 session_one_slot ok=true`、`checker_violations_after_writable_mount=0`）。
- 第一轮 Z3-A 三形与别的池的盘：快照自带的 `entries_after_a_writable_mount_refuse_a_device_without_a_self_verified_system_configuration.rs` 8 条复跑全过（`logs/z3a-existing.log`：`test result: ok. 8 passed`），都在任何写之前拒。

## Y1、Y7　单故障逐序号扫：屏障报错、写了再拒、零单元发布那一支

用例 `opus_r2_y1_y7_single_fault_ordinal_sweep.rs`（批跑脚本 `batch_sweep.sh`，两件并行，原样日志 `logs/b1-1.log`、`logs/b1-2.log`，两件退出码都是 0）。起点同 Y2（txg 5，会话开着），只建一次，之后每段从内存里那一份拷。五个入口：会话覆盖写、正常卸载、准入抬 F 到上限、管理员回退到 (1, 4)、崩了再可写挂载。每段只注入一次：整池第 n 次读、写或屏障报块设备错（自写的一层 `SweepDevice`，只那一次，之后放行）。每段之后：有冻结的发布就撤掉故障原样重发（`resend_the_frozen_publish`）；盘面跑池级 checker；再在拷贝上可写挂载一次、挂载之后再跑 checker。

**取样**。不注入时各入口的调用数（原样）：
```
SWEEP entry=session_overwrite clean outcome=Ok reads=86 writes=29 barriers=6 seconds=0.00
SWEEP entry=unmount clean outcome=Ok reads=283 writes=44 barriers=16 seconds=0.00
SWEEP entry=raise_floor_to_the_admission_ceiling clean outcome=Ok reads=48 writes=0 barriers=0 seconds=0.00
SWEEP entry=rollback_to_1_4 clean outcome=Ok reads=245 writes=21 barriers=6 seconds=0.00
SWEEP entry=crash_remount clean outcome=Ok reads=393700 writes=46 barriers=16 seconds=0.33
```
前四个入口的每一次读、写、屏障逐个注入（784 段）。可写挂载的读有 393700 次，几乎全是扫 768 MiB journal 环：前 600 次与末 1500 次逐个、中间按步长 4999 取（缩前 393762 段，缩后 2240 段。估时：跑前按不注入那一趟量到的挂载 0.33 秒、会话那一段量到的每段约 0.4 秒（`SWEEP runs=6 … seconds=2.4`）估每段约 0.75 秒，全量约 82 小时、超过 40 分钟，于是缩；缩后实际 1648.3 秒，合每段约 0.74 秒）。缩掉的只有扫环那一截：取到的 2178 个读序号里，环扫最后一次读是第 393248 次，之后的 452 次读（重建、见证、取号前后读系统配置槽 68 次）逐个都注入过。写与屏障逐个。

**结局**（`analyze_sweep.py` 按入口、注入种类、结局成员、冻不冻结分格，原样）：
```
OUT ('rollback_to_1_4', 'barrier', 'Err:Publish', 'true') 6
OUT ('rollback_to_1_4', 'read', 'Err:ResurrectedUnitCopyUnreadableOrMismatched', 'false') 8
OUT ('rollback_to_1_4', 'read', 'Err:TargetNotACandidate', 'false') 1
OUT ('rollback_to_1_4', 'read', 'Ok', 'false') 236
OUT ('rollback_to_1_4', 'write', 'Err:Publish', 'true') 21
OUT ('session_overwrite', 'barrier', 'Err:Publish', 'true') 6
OUT ('session_overwrite', 'read', 'Ok', 'false') 86
OUT ('session_overwrite', 'write', 'Err:Publish', 'true') 29
OUT ('unmount', 'barrier', 'Err:RaiseFloorSequencePublishFailed', 'true') 12
OUT ('unmount', 'barrier', 'Err:RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot', 'false') 4
OUT ('unmount', 'read', 'Ok', 'false') 283
OUT ('unmount', 'write', 'Err:RaiseFloorSequencePublishFailed', 'true') 42
OUT ('unmount', 'write', 'Err:RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot', 'false') 2
OUT ('crash_remount', 'barrier', 'Err:Acquisition', 'false') 4
OUT ('crash_remount', 'barrier', 'Err:Publish', 'false') 12
OUT ('crash_remount', 'read', 'Ok', 'false') 2178
OUT ('crash_remount', 'write', 'Err:Acquisition', 'false') 2
OUT ('crash_remount', 'write', 'Err:Publish', 'false') 44
```
（准入抬 F 那 48 段都是 Ok、一个写都不发：这个起点上 F 已在上限，这个入口在这里只读不写。）

**判**（3024 段逐段）：
- 入口之后盘面 checker 红：0 段；撤掉故障重挂失败：0 段；重挂之后 checker 红：0 段；冻结的发布原样重发失败：0 段。
- 读故障让入口报错、而报错之前已发过写：0 段。报错的读故障只有管理员回退那 9 段，9 段 `writes_before=0`。
- 注入的写 / 屏障报错之后还有写：3 段，都是取号失败的回卷（`rollback: RolledBack`，回卷写之前一道屏障），是 D18（块里携带什么信息） 已定项 11 的全或无回卷，不是「写了再拒」。

**C577 那道屏障**。会话覆盖写第 5、6 次屏障（末尾那道，两块盘各一次）报错：`Err Publish { cause: BlockDevice(…) }`、冻结、原样重发做成、checker 0。这与 `.claude/kb/decisions/23-journal的角色与格式.md:397`「一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它」一致。

**零单元发布那一支**（可写挂载取号之后那一串，快照 `transaction.rs` 的 `publish_without_units`、第 1220 行起）：写行那次发布末尾那道屏障是第 9、10 次，最后一次暖机末尾那道是第 15、16 次。16 次屏障注入里 12 次落在取号之后的发布上，都是整个挂载返回 `Err Publish(PublishSequenceFailed { … })`，不冻结；撤掉故障重挂做成、checker 0。这是快照注释里写的处置（`FrozenPublish` 的文档：零单元发布不冻结，一失败就整个挂载返回错误），D23 已定项 14 第 397 行只写了「冻结、原样重发」，没写零单元发布；这一支今天的结局在上面这 12 段里没有坏后果。

**已知**：管理员回退第 23 次读（盘 1 根环）一次读错，回退报 `TargetNotACandidate { …, exclusion: NotInRing }`——目标明明在环里，一次瞬时读错让它报成「换一条目标」。这是正文第二节「在飞」里 A3b 第三节 Q7 的那一处（`mount.rs` 管理员回退的候选判 `rollback_candidate` 读不出就放过），去向第三轮。

**结论**：Y1 的屏障报错与零单元发布那一支、Y7 的「写了再拒」在这 3024 段上没打中。零单元发布不冻结、整挂载返回错误：替没写的条款做了选择（D23 已定项 14 只写了冻结），今天没有专门钉它的用例，本腿的 12 段是第一批。

### A2a　同一段同一块盘两次系统配置写（Y7，没打中）

用例 `opus_r2_y7_same_segment_system_configuration_writes.rs`（`logs/a2a.log`）。同样五个入口，每个入口「不注入」一趟加逐个写序号、逐个屏障序号各注入一次，注入层外面再包一层录制；入口之后有冻结的就原样重发、再在同一组盘上可写挂载一次，三步录在同一条流里；按 `segments::SegmentClosingRule` 切段，数每段每块盘的系统配置槽写。原样末行：
```
A2A runs=189 runs_with_two_sc_writes_on_one_device_in_a_segment=0 runs_with_fua_after_unreleased_plain=0 worst_per_device=1 seconds=88.8
```
189 条流里，一段一块盘最多一次系统配置写；「FUA 写发出时同一块盘同一段里还有没放行的普通写」0 次。

## Y4　坏盘输入与格式边界：系统配置里几个位置字段，core 取常量、checker 取字段

用例 `opus_r2_y4_system_configuration_fields_core_and_checker_read_differently.rs`（2 条，全过，`logs/y4.log`）。做法：第一个文件之后那份合法镜像，把两块盘四个系统配置槽里某一个字段改掉、重算整槽校验和（坏盘输入「盘上内容可控」那一档），看 core 的只读挂载、可写挂载与池级 checker 各怎么判。

**条款**。`.claude/kb/decisions/22-单元原子性怎么合成.md:332`：「基址（设备内字节偏移）= 该字段 × 16384」（「该字段」是系统配置的根环起点字段）。同一项第 5 句，`.claude/kb/decisions/22-单元原子性怎么合成.md:336`：「读者判固定结构槽距：≥ 4096，且槽 1 整槽落在同一槽自述的根环起点之前」。I-7.13 那一行说 checker 的处置「与实现整池拒绝挂载一致」（`.claude/kb/invariants.md:64`）。

**代码**（快照行号）。
- core 解系统配置槽（`system_configuration.rs` 第 473 行 `parse_slot` 起）不读偏移 325（journal 环起点）与 371（根环起点）：第 497 行 `reader.skip(4 + 8)` 跳过；根环区域起点取编译期常量（`root_ring.rs` 第 94 行 `region_start`：`ROOT_RING_BASE_SLOT * SLOT_BYTES + …`），journal 记录偏移取常量 `JOURNAL_RING_START_SLOT`（`journal.rs` 第 201 行）。core 判槽距的上界也是常量根环起点（`recovery.rs` 第 647–648 行 `region_start(0).0 - SYSTEM_CONFIGURATION_SLOT_BYTES`）。
- core 另判两样 checker 不判的：偏移 417 等于环长现算的起点（`recovery.rs` 第 719 行）、起点在 64 槽段边界上（第 727 行）。
- checker 从槽里读 325、371、417（`singlefs-checker/src/image.rs` 第 241、243、244 行），I-7.13 的界只有格式版本、加密类型、槽距（对「同一槽自述的根环起点」）、根槽宽、环长（`journal_ring_bytes_lie_in_the_supported_range`，第 305 行起：在飞上限 ≥ 1、环末端不越过 417）。

**原样输出**（`logs/y4.log`；baseline 是没改的那份）：
```
Y4 baseline core_writable ok instance=2
Y4 baseline checker_violations=0 list=[]
Y4 A unit_area_start 50176 -> 50240
Y4 A_unit_area_start_plus_64 core_read_only err=Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: UnitAreaStartNotTheSlotAfterTheJournalRing { recorded_unit_area_start_slot: SlotNumber(50240), slot_after_the_journal_ring: SlotNumber(50176) } })
Y4 A_unit_area_start_plus_64 checker_violations=1 list=["I-5.2: 盘 0：空闲 Some(3472605184) + 已分配 Some(278528) ≠ 单元区 Some(3471835136)"]
Y4 B journal_ring_start 1024 -> 1023
Y4 B_journal_ring_start_1023 core_writable ok instance=2
Y4 B_journal_ring_start_1023 checker_violations=0 list=[]
Y4 C root_ring_base 64 -> 65
Y4 C_root_ring_base_65 core_writable ok instance=2
Y4 C_root_ring_base_65 checker_violations=1 list=["I-7.1: 根环里一条自证过的根都没有"]
Y4 E root_ring_base 64 -> 0
Y4 E_root_ring_base_0 core_read_only ok chosen=(1,3)
Y4 E_root_ring_base_0 core_writable ok instance=2
Y4 E_root_ring_base_0 checker_violations=1 list=["I-7.13: 盘 0 偏移 0 的系统配置槽自证过，而池级字段越出这一版读者收的范围（FixedStructureSlotSpacingOutsideTheFormatRange）：实现整池拒绝挂载"]
Y4 D ring 805289984 unit_area_start 50176 -> 50175
Y4 D_ring_off_the_segment_boundary core_read_only err=Recovery(SystemConfigurationValueRefused { device: DeviceIdentity(0), value: UnitAreaStartOffTheClusterSegmentBoundaryUnsupported { unit_area_start_slot: SlotNumber(50175) } })
Y4 D_ring_off_the_segment_boundary checker_violations=1 list=["I-5.2: 盘 0：空闲 Some(3472605184) + 已分配 Some(278528) ≠ 单元区 Some(3472900096)"]
```
（A、D 的 `core_writable` 与 `core_read_only` 同一个错，C、B 的 `core_read_only` 都是 `ok chosen=(1,3)`，见日志。）

**读法**：
- E（根环起点字段写 0）：checker 报 I-7.13 红、原文自称「实现整池拒绝挂载」；core 只读挂载、可写挂载都做成（取号 2）。C（写 65）：core 可写挂载做成、之后照常量往 1 MiB 起的根环写；checker 按字段读根环，一条根都找不到，报 I-7.1。
- A（417 改成 +64）、D（环长改成 768 MiB − 16 KiB、417 跟着改成它现算的 50175）：core 整池拒；checker 的 I-7.13 不红，别的不变量照判（I-5.2 红一条，是单元区起点不同算出来的账）。
- B（journal 环起点字段写 1023）：core 照常量扫环、可写挂载做成；checker 按字段扫一个错位 16 KiB 的环，0 条违例（它读到的环里没有记录，journal 那几条不变量在这份镜像上判的是空的）。
- 单元区起点这一个数：mkfs、恢复、分配器都按环长现算（快照 `make_filesystem.rs` 第 363 行、`recovery.rs` 第 744 行、挂载按择到的系统配置建空闲图），checker 取偏移 417。合法镜像上两者相等（A3b 让读者判 417 = 现算起点）；盘上 417 被改时 core 拒、checker 照 417 走，两边读的不是同一个数。

**四句**：
1. 分不分辨臂：这一格没有候选臂；它分辨「core 按字段」与「core 按常量」两种实现，C、E 两格在两种实现上结局相反。
2. 系统当时看得到判别它的东西吗：看得到，字段就在 core 已经整槽读进来、校验和已核过的那 4096 字节里。
3. 满足判据字面的哪一个分句：D22 已定项 16 第 332 行「基址 = 该字段 × 16384」——core 取常量，C、E 两格上与字面相反；I-7.13 第 64 行「与实现整池拒绝挂载一致」——A、D、E 三格上不一致（A、D 是 core 拒而 checker 不拒，E 是 checker 拒而 core 收）。
4. 跑前条款给的改法：A3b 那一件（读者判 417 = 现算起点、判段边界）只管 417，对 325、371 这几格不起作用。

**结论**：和条款说反话（D22 已定项 16 第 332 行、第 336 行：core 不读根环起点字段，用常量；判槽距的上界也用常量根环起点）。I-7.13 与 core 的「整池拒」各判一套、互不是对方的子集。今天 mkfs 只写默认值（64、1024），合法历史走不到；盘上内容可控时走得到，且 E 格是 checker 自称「实现整池拒绝挂载」而实现照挂的一格。

### mkfs 收不收环长不是物理块整数倍的几何（没打中）

```
Y4 mkfs ring=1048476 ok=false err=Some("BlockDevice(Unaligned { offset: DeviceOffsetInBytes(16777216), length: 1048476, physical_block_size: 512 })") written_sectors=[0, 0]
Y4 mkfs ring=1048064 ok=true err=None written_sectors=[114, 113]
Y4 mkfs ring=1046528 ok=true err=None written_sectors=[114, 113]
```
环长 1 MiB − 100（不是 512 的整数倍）：mkfs 在整环清零那一步被块层拒（报的是块设备错，不是几何错），两块盘一个扇区都没写。mkfs 的几何判定（快照 `make_filesystem.rs` 第 305 行 `check_geometry` 起）没有「环长是物理块整数倍」这一判，靠块层兜住；结局是「写之前拒」。1 MiB − 512、1 MiB − 2048 两种 mkfs 收。

### 读路径 panic 普查（只读代码，没造输入）

按 `.expect(`、`unwrap()`、`panic!`、`assert!` 在 core 的 14 份读者文件里过了一遍（非测试部分）：`recovery.rs` 45 处、`journal.rs` 23、`unit.rs` 35、`system_configuration.rs` 27、`extent_tree.rs` 27、`code_two_tree.rs` 23、`mounted_read.rs` 22 等。抽看的那几处（`recovery.rs` 第 745 行按择过的系统配置取单元区起点、第 3836 行按 inode size 开缓冲——上一步 `data_unit_count_matches_the_inode_size` 已把 size 压在数据单元数 × 净荷以内、`journal.rs` 第 290 行点名项数、第 198 行环槽数取模）都有前置判定挡着，没找到盘上内容走得到的一处。这一条是读代码，没有造输入，不算「没打中」的观测。

## Y6　journal 环转圈之后的撕裂、C577 之后的关段、续跑与分片、模型

### Y6-a　环转圈之后的 journal 记录写进了第三态枚举，枚举出来的状态恢复、checker、再挂都绿（没打中）

**条款**。`.claude/kb/decisions/13-验证路线.md:72`：「系统配置槽写与覆盖旧记录的 journal 写属于这一类」（「这一类」是取第三态「新旧都读不出」的原地覆写）。

**代码**。`crash.rs` 的 `is_tearable_in_place_overwrite`（快照第 2560 行起）三条：不是单元写、长于一个扇区、基镜像那一段有非零字节或写表里更早有同盘带字节的写与它重叠。环转过圈之后的记录写罩住的是基镜像里的旧记录，第三条成立。

**历史**（`opus_r2_y6_journal_ring_turned_torn_records.rs`）：4 GiB 两盘、journal 环改成 1 MiB（256 槽、在飞上限 85；mkfs 收，单元区起点 1088 在段边界上）。
1. 有文件那一族：mkfs、取号、暖机、第一个文件，会话覆盖写 298 次，最后一条记录的计数器 301（环已转过一圈）；以这一刻为基镜像，只录一次正常卸载。
2. 没文件那一族：只 mkfs，之后崩了再可写挂载 101 次把计数器推过 300；以这一刻为基镜像，只录一次崩了再可写挂载。

先探段序列（`probe_segment_sizes_of_the_entries_on_a_turned_ring`，原样）：
```
Y6 probe window=overwrite segments=24+2+1+2 journal_writes=2 tearable_journal_writes=2 tearable_total=4 layer0_states=16777233 counter_after=302
Y6 probe window=unmount segments=2+16+2+1+2+16+2+1+2 journal_writes=4 tearable_journal_writes=4 tearable_total=10 layer0_states=131113 counter_after=304
Y6 probe window=crash_remount segments=2+18+2+1+2+16+2+1+2+16+2+1+2 journal_writes=6 tearable_journal_writes=6 tearable_total=14 layer0_states=393273 counter_after=307
```
覆盖写那一段单元写 24 个、全量 16777233 个状态，超过一段历史 10⁶ 的线，不跑；卸载、崩了再挂都在线内。按层 0 的枚举域（`enumerate_layer0_in_state_slices` + `full_expansion`，原地覆写三态）全量枚举，枚举器自己在每个状态恢复之后跑 oracle 与池级 checker；另把每个状态的盘面拿去可写挂载、挂载之后再跑 checker。

**原样输出**（`logs/y6-nofile.log`、`logs/y6-unmount.log`）：
```
Y6 window=nofile_remount overwrites_to_turn=101 writes=29 segments=11 tearable_journal_writes=6 tearable_total=14 states_this_run=4155 full_states=4155 estimate=false
Y6 window=nofile_remount states=4155 violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 torn_journal_states=15 remount_candidates=4155 seconds_enumerate=48.9
Y6 window=nofile_remount remount mounted=4155 refused=0 checker_red_after_mount=0 examples=[] seconds_total=113.5
Y6 window=unmount overwrites_to_turn=298 writes=65 segments=13 tearable_journal_writes=6 tearable_total=14 states_this_run=196665 full_states=196665 estimate=false
Y6 window=unmount states=196665 violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 torn_journal_states=15 remount_candidates=15 seconds_enumerate=1780.4
Y6 window=unmount remount mounted=15 refused=0 checker_red_after_mount=0 examples=[] seconds_total=1780.8
```
（`overwrites_to_turn` 在没文件那一族是可写挂载次数。有文件那一族的卸载紧接在第 298 次覆盖写之后录，比探段那一趟早一步，段数 13、状态数以这一趟为准。）

**读法**：两族里撕开的 journal 记录都进了枚举（有文件那一族 15 个、没文件那一族 15 个状态里有一条撕开的记录落了盘），恢复之后 oracle 0 违例、池级 checker 0 违例；没文件那一族 4155 个状态逐个可写挂载，全做成、挂载之后 checker 0。有文件那一族的再挂只挂「有一条撕开的记录落盘」的那几个状态（这是本腿另加的观测，按候选取；枚举与恢复之后的判定一个不落）。

**估时与取样**：没文件那一族全量 4155 状态，枚举 48.9 秒（两线程，约 11.8 毫秒一个），按它估有文件那一族 196665 个状态约 39 分钟、在 40 分钟线内，不缩（实际 1780.4 秒）；再挂那一遍全挂约要 51 分钟（按没文件那一族量到的 64.6 秒 / 4155 个、约 15.5 毫秒一个），加上枚举超线，只挂撕开记录落盘的那几个。本腿原型跑的崩溃状态合计 4155 × 2（没文件那一族跑了两趟，第一趟作废，见「这条腿自己的限度」）+ 60（估时那一趟）+ 196665 = 205035，在 10⁷ 以内。

### Y6-b　C577 之后的关段规则（兑现了条款）

`.claude/kb/decisions/13-验证路线.md:72`：「当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上」，又「带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久」。`segments.rs` 的 `SegmentClosingRule::after`（快照第 117 行起）逐字对应：普通写把它的盘记成没放行，屏障或 FUA 把它的盘放行，全都放行且段里有写才关段。上面的段序列里每次发布末尾是 `…+1+2`：根槽 FUA 自成一段（一块盘、前面已被屏障放行），轮换那两写同一段、由 C577 那道屏障关上——正是这条规则的字面后果。

这条规则把「FUA 放行它那块盘上前面同段的普通写」当成真设备的语义（REQ_FUA 不带 PREFLUSH 时只保这一次写，这一差别归 C6（块层语义假设写错））。A2a 那 189 条流里「FUA 写发出时同一块盘同一段里还有没放行的普通写」0 次（见 Y7 节），所以今天的写流上这一差别走不到，结论同第一轮 Z5。

### Y6-c　续跑与分片（只读了代码与现有用例，没另造输入）

快照已有 `crash_enumeration_resumes_from_its_progress_file.rs`（整行截断续跑、换线程数、换版本表、缺字段整份作废、从头跑、判红不留文件）与 `crash_enumeration_sharded_across_processes.rs`（两片三片合并与不分片逐字相同、缺片、换输入指纹、重复片、换切法、换工具链、分片被截断续跑）。本腿没另造输入：没找到这两份用例之外的形状（进度文件整行才算、同一片出现两次计数不同整份作废、合并按文件头核齐）。这一格是读代码，不算「没打中」的观测。

### Y6-d　模型在「重读一次仍读不出就拒可写」那一格（只读了代码）

`model.rs` 的 `answer_mount_writable_with_the_newest_root_unreadable`（快照第 1275 行起）不自己算见证值：执行器只在「最新那条根是这个进程发的」时调它，模型据此直接答拒（注释：「系统配置没见证到最新那条根的那一形……这一步造不出，模型不答」）。实现一侧照盘上的系统配置算。两边在这一格不是各算各的：模型用的是 C577 之后「返回了的发布都被见证」这一前提，不是独立的一份判定；这一前提在 Y1 节 3024 段与 Y6-a 的枚举上没被打穿。

## 这条腿提的改法（只在我的模型上量过、被攻过零轮）

原型补丁 `fix_prototype_swapped_or_behind.patch`（快照 `mount.rs`、`mounted_session.rs` 两份，只打在攻方副本上）：新加 `devices_swapped_or_behind_the_current_version`，挂着之后的抬 F 那一串（正常卸载、抬 F 都走它）与会话每次发布，在「可见」之后再核两样——每块盘自证槽里的本盘设备号等于盘表给它的身份；这块盘最新那份系统配置的 (实例, tail) 不落后于现行那一版末条记录，落后就读现行那一版的单元、有一份字节不同即拒（与可写挂载落后支同一判，「所选那一版」换成现行那一版）。拒时借用已有的成员报（会话借 `DevicesWithoutASelfVerifiedSystemConfiguration`、卸载借 `NoSelfVerifiedSystemConfigurationOnTheDeviceHandedIn`），成员名不对，只为量。管理员回退那一个入口没打。

| 格 | 改法 | 修不修 | 标 |
|---|---|---|---|
| Y2-a 旧快照经会话 / 卸载洗白（16 格） | 原型补丁的落后支 | 修：16 格入口都拒、之后可写挂载照对照一样被拒（`logs/y2-fix.log`：`entries_accepted=[false]`… `writable_after_ok=false`） | 量过 |
| Y2-a 同上，经管理员回退 | 同一判加进 `roll_back_by_a_forward_publish` | 推的：回退手里也有现行那一版 | 推的 |
| Y2-b 会话收盘体对调 | 原型补丁的本盘设备号那一判 | 修：`Y2 session_swapped ok=false err=Some("DevicesWithoutASelfVerifiedSystemConfiguration { devices: [DeviceIdentity(0), DeviceIdentity(1)] }")`，之后可写挂载做成、checker 0 | 量过 |
| Y2-c 只剩一槽自证 | 原型补丁 | 不误拒：`Y2 session_one_slot ok=true`、挂载之后 checker 0 | 量过 |
| Y2 第一轮三形 | 原型补丁 | 不变：快照自带的 8 条照过 | 量过 |
| Y4 core 不读根环起点字段 | core 解槽时读 325、371，与常量不等就整池拒（或按字段走）；判槽距上界改用「同一槽自述的根环起点」 | 推的：没实现 | 推的 |
| Y4 I-7.13 与 core 整池拒各判一套 | checker 的 I-7.13 补「417 = 环长现算」「起点在段边界」、core 补 325/371 那几项，两边收同一张表 | 推的 | 推的 |

代价（推的，没量）：活的会话里两块盘在每次返回之后都已轮换到现行那一版，落后支只在盘被换过时才读单元，平时只多比两个数；本盘设备号在「可见」那一判已经读进来的两槽里，不多读盘。

回归（副本上的数，不算入库装置上的数）：原型补丁下跑快照里 5 份测试（Z3-A 那份 8 条、参数与盘表那份 6 条、准入抬 F 那份 10 条、抬 F 与正常卸载那份 11 条、原样重发那份 1 条）。红 4 条：`root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise` 与 `rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs` 里钉录制流头六步的 3 条。**同样这 4 条在没打补丁的快照上也红**（同一组命令在没打补丁的快照上跑，`logs/snapshot-regression.log`；与副本上的 `logs/fix-regression.log` 比 `FAILED` 行，`diff` 无差）：前一条是正文第二节「在飞」的 P1（挂载那一刻就读不出的根槽照 D16 第 37 行当没有根），后三条是 C577 带来的钉值（抬 F 那一串头上那道屏障紧跟在上一次发布末尾的屏障之后，录制流把它并掉），归「测试钉值的统一修正」，都是已知。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| 单故障（读 / 写 / 屏障各一次）落在会话覆盖写、正常卸载、准入抬 F、管理员回退的每一次调用上 | 784 段，逐个 | checker 红 0、重挂败 0、写了再拒 0、冻结后原样重发败 0 |
| 同上，崩了再可写挂载 | 2240 段：写 46、屏障 16 逐个，读前 600 与末 1500 逐个、中间按 4999 取（缩前 393762） | 同上；零单元发布那道屏障报错 12 段都是整挂载报错、重挂做成 |
| 同一段同一块盘两次系统配置写（A2a）；FUA 前同盘同段有没放行的普通写 | 上面五个入口 × 不注入 + 逐个写、逐个屏障注入，入口之后原样重发与再挂录在同一条流里，189 条流 | 0、0 |
| journal 环转圈之后撕开的记录 | 1 MiB 环；有文件一族正常卸载那一段全量 196665 状态、没文件一族崩了再挂那一段全量 4155 状态 | oracle 0、checker 0（有文件一族见 Y6-a 那几行）；没文件一族 4155 个状态再挂全做成、checker 0 |
| 只剩一槽自证的盘交给会话 | 较新那一槽写坏 1 字节，1 条历史 | 照常发布、checker 0、再挂做成 |
| 别的池的盘、全零空盘（第一轮三形） | 快照自带 8 条 | 都在任何写之前拒 |
| mkfs 收环长不是物理块整数倍 | 1 MiB − 100、− 512、− 2048 | − 100 写之前（块层）拒、一个扇区不写；另两个收 |
| 系统配置见证过的最新根被当成被抛弃（Y1 主问） | 上面 3024 段的读注入（含重建、见证、取号那 452 次读逐个）与 Y6-a 的枚举 | 没造出来：单次读错都被重读一次吸收（崩了再挂 2178 段读注入全部做成） |

## 越格线索（不归这条腿的格，只记）

- 会话收盘体对调之后，池级 checker 0 条违例、可写挂载却因本盘设备号拒：checker 读本盘设备号（快照 `singlefs-checker/src/lib.rs` 第 265 行），没有一条不变量拿它比盘的身份（Y5）。
- Y4 的 B 格：checker 按偏移 325 扫环，环起点被改时它在一个错位的环上判 journal 那几条不变量、判出 0 条违例（Y5）。

## 这条腿自己的限度

- Y2-a、Y2-b 的「可达」与第一轮 Z3-A 同一个前提：管理员交进来的盘表由人定，盘可以被换。单故障（一次读错、一次写错）造不出旧快照与盘体对调。
- 原型补丁只打了抬 F 那一串（正常卸载、抬 F 都经它）与会话；管理员回退没打。补丁拒时借用的错误成员名不对，只为量。
- Y4 四格都是改字段、重算整槽校验和的坏镜像（盘上内容可控那一档），合法历史走不到；只跑了一份起点（第一个文件之后）。
- Y1/Y7 扫的是单次故障、故障只在那一次，没扫双故障、没扫一直失败的盘；读注入在可写挂载那一段对环扫那 39 万次读按步长取样（取样范围见 Y1 节）。「入口之后盘面」是全部写都落了的那一个状态，不是崩溃状态；崩溃状态只在 Y6-a 那两段上枚举。
- Y6-a 有文件那一族只录了一次正常卸载；覆盖写那一段 1677 万个状态、崩了再挂那一段 39 万个状态没跑（前者超过一段历史 10⁶ 的线；后者在线内，但加上前面的量估时超过 40 分钟，没排）。
- Y6-a 没文件那一族第一趟作废：版本表给错了（把没文件的根也登记成带空内容的文件），枚举器当场报 `violations=4155 first_violation=Some("第 301 代根下面有文件却报没有（持久的写：）")`；改成不给版本表重跑，日志只留第二趟。
- Y6-c、Y6-d 与 Y4 的 panic 面只读了代码。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这几格的「没打中」都只是一次抽样。
- 用例钉的是快照今天的行为（打中的样子），不是该有的样子。副本上量出的数（含原型补丁那几行）不算入库装置上的数。

## 没做什么

- 不判 Y3、Y5、Y8（正推腿与本地攻方的格）；Y5 的线索记在「越格线索」一节。
- 没跑任何名字带 layer0 的用例与重型测试；原型里调枚举函数跑的两段流是本腿自己造的，名字不带 layer0，每段全量状态数先按层 0 口径算过、在 10⁶ 以内，合计 205035。
- 没改主工作区，没读主工作区 `crates/`；没做 git 写操作。
- 不替主 agent 采纳；改法都只在我的模型上量过、被攻过零轮。
