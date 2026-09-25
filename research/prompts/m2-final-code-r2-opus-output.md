# m2-final-code-r2 云端攻方（Opus）：Z8、Z9、Z12

2026-09-25 JST（2026-09-24 UTC 19:50–20:50）。立场：找反例。代码读冻结副本 `/tmp/claude-1000/m2-final-code-r2/tree/crates/`，kb 读快照 `/tmp/claude-1000/m2-final-code-r2/kb-snapshot/.claude/kb/`；用例全在副本的拷贝 `/tmp/claude-1000/m2-final-code-r2-opus/tree/` 上跑，**副本上的数不是入库装置上的数**。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Z9-A 见证表写满 | **打中** | 根环 24 个槽全读得出、全自证过，连着 24 次回退（每次崩在写行那次的轮换之后、暖机之前）就写满：第 24 次回退在取号之前被拒 `RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 }`。条款「表写不满」与删除规则 ① 并存时为假。两种回退目标取法（一直回退到同一个旧根 / 每次回退到最新根）都中；不崩的对照三种取法各 40 次，最多 9 条 |
| Z9-B 写行根落了、见证轮换没落 | **打中** | 崩在这一窗口之后，见证条目永远不会补写：之后每一次可写挂载都照删除规则抄旧表、不加这一条。同一段历史，崩了的那一边让比被抛弃的 C 新的根全读不出就落回 C（m = 0..3 次后续挂载，3 / 5 / 8 个根故障）；不崩的对照同样的故障（及更多）落在 R_old。窗口本身（m = 0、1 个故障）是 `m2-witness-r1` 判决第 42 行记过的；**窗口过去之后仍留着**这一点没记过 |
| Z9 删除规则删掉仍护着根的条目 | 没打中 | 读代码 + A1–A3、B 的历史里没见到；理由见 Z9 节 |
| Z9 表满之前先因别的原因挂不上 | 已知（实二一） | A3 那条历史（每次回退到最新根、崩在暖机第一次的根之前）第 36 次回退被 `WarmUpAdmissionRefusedBeforeAcquisition … AllocationRecordsExceedOneNode { records: 820, capacity: 812 }` 拒，此时见证表 13 条。分配记录树只有一个节点，归实二一 |
| Z8 原样重发逐字节 | 没打中 | 1 个 / 2 个数据单元的发布，失败点扫全部 25 / 29 个设备操作，外加重发时再失败一次的全部位置：重发那一遍除系统配置轮换之外逐项相同（盘、种类、偏移、长度、内容哈希）；系统配置只在「失败点在轮换本身」时世代号不同 |
| Z8 冻结期间绕过闸 | 没打中 | 会话里的写路径（`publish_*`、`raise_rollback_floor`）都先过闸；零单元发布、取号是挂载与 mkfs 流里的，会话里没有调用方 |
| Z8 读者两格切断合法历史 | 没打中 | 上面那组失败点 × 不重发直接恢复 / 重开可写挂载 / 重发逐前缀崩，恢复结局只有 A 或 B，重开挂载全过，一次都没撞「两个末条标志」或断链错 |
| Z12 预演与真发相同那条断言 | 没打中 | 副本上用环境变量开关把挂载里的写入口也压到小容量（记账树、中央映射树、一条记录的点名项），扫 70 + 24 组：18390 + 672 次挂载成功，0 次 panic；形态含多层树上的写行 / 暖机、回退到环里每条根、上一次发布逐前缀截断之后挂载、树表 0 条的一版写行跨多条记录 |

## 复跑

- 一条命令：`bash research/prompts/m2-final-code-r2-opus-model/rerun.sh <空目录>`（拷冻结副本、打 `copy-only-core.patch`、放三份用例、按表跑）。`copy-only-core.patch` 只加环境变量开关（`SINGLEFS_R2_OPUS_WITNESS_RULE`、`SINGLEFS_R2_OPUS_WITNESS_REPAIR`、`SINGLEFS_R2_OPUS_CAPS`、`SINGLEFS_R2_OPUS_NAMED`），不设变量时逐行走冻结副本的路径。
- Z12 两遍扫描：`run_z12_sweep.sh`、`run_z12_sweep2.sh`（日志目录写死在 `/tmp/claude-1000/m2-final-code-r2-opus/logs/`）。
- 线程：每条 cargo 命令经 `research/scripts/capped.sh 8`；扫描脚本 4 个进程 × 2 线程。跑的时候机器上另有 3 条 E158 的 `cargo run`（`ps` 看到的 pid 2942704、2942866、2943096），不是性能测量，没等。

## 模型目录的文件与 sha256

目录 `research/prompts/m2-final-code-r2-opus-model/`，全表在同目录 `SHA256SUMS`（列 122 个文件，不含它自己；它自己的 sha256 是 25aab877259b4bb0b5f33e7466e7054bf310bafebf42f32ab4ea0a11659e99bf）（其中 96 个是 Z12 两遍扫描每组一份的摘要日志）。承重的几份：

```
acc36c060833f13c5d54ef2366b52e09fffb205361313f9d2899455ea32916fb  ./copy-only-core.patch
1794dc0f6ed0f5f7e3bbd5488022df51866190991a3e055ff67ce2a22ee6dd9a  ./rerun.sh
830ddb29de3e30511a19022765b09887e7b6c2852fd3018689da76c6b3d5dfe7  ./tests/r2_opus_z9.rs
cb33997f8f59c19a8ec3216d8a194f7b3d2ea73402a8c68a2230622eb384a77f  ./tests/r2_opus_z8.rs
fc2fadcd989c6f8402e04e254005173e697e686eb49e9ca39cefc3445bcfe1ba  ./tests/r2_opus_z12.rs
aef14cee75e9f1f6b700abc277c3a4dca5e52da7391b57a940a90042988d214f  ./logs/z9_rule_original_z9_a1.log
3fc1b0c8ffd2e1b37aab0bb136c2ea0bdb297e1b1c55f18db8ec882206483b26  ./logs/z9_rule_original_z9_a2.log
674b7d3fb8c8e5b4b33cc5524c925af597c4da616c7acdaac4c3c0345785b776  ./logs/z9_rule_original_z9_a3.log
b54cd9aacea2fb986f1b74b2e0b24eec94fec0edcc5e363c0d8304bc7977170c  ./logs/z9_a_control.log
47342b6ba273de2c53893b809f346292d6f2309ccac848b2e7dc9830f9ee28cd  ./logs/z9_b.log
cd5888208dee90bde25ea5047ab5f4fa53f65506bd2a1eba5b9aa3c295063147  ./logs/z9_b_repair.log
9c28cda81e951301e9ead534962a9dbd3e863b13bea4f6b761c0bbb434282d48  ./logs/z8_z8_one_unit.log
4920a48e9e11228f2d59af024b31ad2e4ea92bb8617010ab61f4931a29ebcda9  ./logs/z8_z8_two_units.log
```

冻结副本里被引的源文件（`mount.rs` 那一行与 `/tmp/claude-1000/m2-final-code-r2/crates-src-sha256.txt` 相同）：

```
23d9a14473b7f9804e4805080e957aae346a433cceae1770b678377263bec40e  singlefs-core/src/mount.rs
9ea96e79eb03478712fc30c62da814c58dbffecb628ed930489ab61a69d1e955  singlefs-core/src/recovery.rs
af32afb5ac8d724c8fe0589b6a0b6feb26c78a3a60eea77898743a5be215d490  singlefs-core/src/rollback_witness.rs
498a172e56d69c88342748240d548208954581aa75b788c7802896b3a1e7243b  singlefs-core/src/transaction.rs
```

## Z9　回退见证

### 被判的字面

`decisions/23-journal的角色与格式.md` 第 373 行（快照）：

> **回退见证**（C332（回退实例两个根都读不出时回退被撤销） 的修法）：回退在系统配置槽里记一张见证表，放在系统配置字段表之后、越过 512 字节，靠系统配置的整槽校验和与两槽轮换认撕裂（越过 512 撕开的那一槽作废，等于这次写没持久）。一个条目记一次回退：新实例代号 4 字节、R_old 的实例代号 4 字节与 txg 8 字节。根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N；择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新。见证随回退那一次挂载写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表。条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定，表写不满）。见证表读不出就是系统配置槽读不出，挂载报错，不静默择根。字段落点随实现写进 D22（单元原子性怎么合成） 已定项 9 的字段表与 [layout/02-second-txn.md](../layout/02-second-txn.md)；条目什么时候删随实现，交代码三方。

同一文件第 387、388 行：

> - ① **删除规则**：条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、都自证过，且其中没有一条根的实例代号落在 [r_old, N) 里。
> - ② **写满**：有槽持续读不出时条目删不掉，写满时在取号之前拒绝挂载（`RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided`），盘上不变；写满之后怎么办没定，见 C547（回退见证表的删除规则与写满没有条款）。

代码：冻结副本 `mount.rs:1703`（删除规则的判定 `entry.rollback_target_instance <= root.instance`）、`mount.rs:1717`（文档注释「有槽持续读不出时条目删不掉、表就写得满」）、`mount.rs:1733`（只有回退挂载才加这一次的条目）、`mount.rs:1862`（写行之后起带加了条目的表）；上限 `rollback_witness.rs:54`，S 取 `singlefs-format/src/lib.rs:227` 的 8，上限 23。

### Z9-A　根环全读得出时表写满（打中）

**历史**（`tests/r2_opus_z9.rs` 的 `z9_a1`、`z9_a2`，日志 `logs/z9_rule_original_z9_a1.log`、`…_a2.log`）：
1. mkfs → 第一个文件 A（txg 3）→ 覆盖写到 txg 30，实例 1。根环 24 槽此时全是自证过的根。
2. 进程退出，管理员回退（`mount_rollback`）。A1：目标一直是 (1, 29)，它离开根环之后改回退到最新根；A2：每次回退到最新根。
3. 每次回退挂载崩在「写行那次发布的根 FUA 与两盘系统配置轮换都落了、暖机第一次之前」——录制流里第一次 FUA 之后的两次系统配置写落完即止。一次回退只落一条根、一个 txg。
4. 重开，再回退，重复。

每一步许可它的字面：回退目标在候选集里（第 371 行的候选集定义；`mount_rollback` 按最新根的实例表与见证判，这两条历史的目标都过了那两判）；崩溃在任何一步都合法；条目随写行之后第一次轮换落盘（第 373 行，代码 `mount.rs:1862`）；删除规则 ① 留下它们——A1 里第 k 条 (N_k, 1, 29) 抛弃前几次回退的根 (N_1..N_{k−1})，它们都还在环里，实例代号落在 [1, N_k)；A2 里第 k 条 (N_k, N_{k−1}, t_{k−1}) 的目标根自己就在 [r_old, N) 里。

原样输出（A1，`logs/z9_rule_original_z9_a1.log` 末两行）：

```
k=23 ring_all_readable=true witness_entries_before=22 target=(1,29) new_instance=24 roots_written_txg=[53, 54, 55] persisted_roots=1
k=24 ring_all_readable=true witness_entries_before=23 target=(24,53) REFUSED Recovery(RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: 24, capacity: 23 })
```

A2 同样在 k = 24 被拒，同一个错误成员、同样 `ring_all_readable=true`。`ring_all_readable` 是挂载之前用 `every_root_ring_slot_holds_a_root` 现读的，24 步里每一步都是 true。

**对照（不崩）**：`z9_a_control`，三种回退目标取法（一直同一个旧根再改最新 / 每次最新 / 每次环里最旧的候选）各连着回退 40 次、整次挂载写完：

```
=== control sticky (1,29) then newest: mounts=40 max_witness_entries_before=Some(9) stop=None
=== control newest: mounts=40 max_witness_entries_before=Some(9) stop=None
=== control oldest candidate: mounts=40 max_witness_entries_before=Some(9) stop=None
```

写满靠的是「一次回退只落一个 txg」：整次写完的回退挂载写行 + 暖机落 2 到 3 条根，同一个根的寿命（24 个 txg）里装不下 23 次回退。

**四句**：
1. **分不分辨臂**：这一格的臂是删除规则的取法（第 387 行 ① 是实现员取、被攻过零轮）与第 373 行「表写不满」那句的前提。我在副本上量了三种删除规则（下表），打中的格在不同规则上中不中不一样，分辨得出。与 WF3「按根环几何取上界 / 按实测取」那两臂的关系：两臂在这段历史上都写得满（实测取 3 或 4 条更早满），不分辨它们。
2. **系统看得到判别它的东西吗**：看得到。删除时根环 24 槽、journal 环、见证表都在盘上；A1 里前 22 条被最后一条逐条罩住（目标相同、新实例更大），A2 里的条目一条被抛弃的根或记录都没有——这两件都能在挂载时从盘上判出来。
3. **满足的是字面的哪一分句**：打中的是第 373 行「条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定，表写不满）」里的「表写不满」，以及第 388 行 ② 与 `mount.rs:1717` 注释给写满设的前提「有槽持续读不出时」——这两段历史里一个槽都没读不出。C547（`checks-owed.md` 第 481 行）立账时的前提同样是「有槽持续读不出」。不是 ① 本身的字面错：代码照 ① 字面删，是 ① 这一取法让「表写不满」为假。
4. **各改法在打中的格上还中不中**：跑前条款没给改法（② 只说写满时拒、处置没定）。我自己提的三种，见下表。

| 删除规则（副本上的环境变量） | A1 同一旧根 | A2 每次最新根 | A3 最新根、崩在暖机根之前 | 量过 / 推的 |
|---|---|---|---|---|
| 冻结副本的 ① | 第 24 次写满 | 第 24 次写满 | 13 条封顶；第 36 次被分配记录墙拒 | 量过 |
| ① ∧ 去掉被另一条罩住的（目标 ≤、新实例 ≥） | 7 条封顶，30 次不满 | 第 24 次写满 | 13 条 | 量过 |
| ① ∧ 只留还抛弃着环里某条可读根或某条自证记录的 | 第 24 次写满 | 1 条封顶 | 12 条 | 量过 |
| 两样都加（「both」） | 2 条封顶 | 1 条封顶 | 12 条 | 量过 |

原样摘要（`logs/z9_rule_*_z9_a*.log` 各自的 `^k=` 行数、`witness_entries_before` 最大值、`stop` 行）在各日志里；表里每格都是那一次跑出来的。

**我提的改法只在我的副本上量过、被攻过零轮。**「both」的安全性是推的：罩住的那一条去掉之后并集判法逐字不变（这一半是等式，不靠测）；「没抛弃任何东西就删」那一半靠的是「之后新写的根与记录的实例代号都大于 N、不会被它抛弃」，与 ① 自己的理由同一句，但它把「journal 里暂时读不出、之后又读得出的记录」这一格让了出去，没测。「both」下还有没有一段合法历史能写满，我推的上界约 R × S ÷ 2（每条不被罩住的条目要抛弃一条根或记录，而抛弃对象要占掉额外的 txg），没证、没扫。

**什么现象会推翻这一格**：入库装置上照这两段历史重做，第 24 次回退挂上了（或拒的是别的成员），或那一刻 `every_root_ring_slot_holds_a_root` 交回 None。

### Z9-B　写行根落了、见证轮换没落：之后永远不补（打中）

**历史**（`z9_b_crash_between_row_root_and_its_rotation_never_gets_the_witness`，日志 `logs/z9_b.log`）：A（txg 3）→ B（4）→ C（5），实例 1；管理员回退到 (1, 3)，回退挂载崩在写行那次的根 (2, 6) FUA 落了、紧接的系统配置轮换一次都没落；之后用户照常可写挂载 m 次（m = 0..=3 放开扫，由用户定的那一步不写死）；最后把「(txg, 实例) 比 C (1, 5) 大」的每一条根都设成读不出（`PoolReaderWithUnreadableRootRingSlots`），恢复。对照：同一段历史，回退挂载整次写完。

每一步许可它的字面：第 373 行「见证随回退那一次挂载写行之后的第一次系统配置轮换写」——崩在那一次轮换之前，这一次写就没发生；之后的普通挂载 `mount.rs:1733` 只在 `previous_row.is_rollback` 时加条目，普通挂载不是回退，照删除规则抄盘上的表（空表）。条款没有一句要求之后的挂载补这一条。

原样输出（`logs/z9_b.log`，`^m=.*->` 行）：

```
m=0 crash_between_row_root_and_rotation=true witness=[] unreadable_root_txgs=[6] faults=1 -> FileRead root=(1,5) content_is_A=false content_len=2602
m=0 crash_between_row_root_and_rotation=false witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7] faults=2 -> FileRead root=(1,3) content_is_A=true content_len=3000
m=1 crash_between_row_root_and_rotation=true witness=[] unreadable_root_txgs=[6, 7, 8] faults=3 -> FileRead root=(1,5) content_is_A=false content_len=2602
m=1 crash_between_row_root_and_rotation=false witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8, 9, 10] faults=5 -> FileRead root=(1,3) content_is_A=true content_len=3000
m=2 crash_between_row_root_and_rotation=true witness=[] unreadable_root_txgs=[6, 7, 8, 9, 10] faults=5 -> FileRead root=(1,5) content_is_A=false content_len=2602
m=2 crash_between_row_root_and_rotation=false witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13] faults=8 -> FileRead root=(1,3) content_is_A=true content_len=3000
m=3 crash_between_row_root_and_rotation=true witness=[] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13] faults=8 -> FileRead root=(1,5) content_is_A=false content_len=2602
m=3 crash_between_row_root_and_rotation=false witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] faults=11 -> FileRead root=(1,3) content_is_A=true content_len=3000
```

崩了的那一边读回被抛弃的 C（2602 字节，C 的内容），回退被静默撤销——正是 C332 的形态；对照一边故障更多也落在 R_old、读回 A。m = 0 那一行是 `m2-witness-r1-main-verification.md` 第 42 行记过的「post 把 1 个故障的窗口缩到『行根落了、带见证的那次轮换还没落』这一段」；那一轮只量了回退那一次挂载之内的各段。**新的是 m ≥ 1**：窗口里崩一次之后，这个池永远停在「没有见证」的状态，撤销回退所需的故障数只随之后写的根变多而涨，直到被抛弃的根离开根环；同一段历史不崩的一边永远撤销不了。

**四句**：
1. **分不分辨臂**：分辨 WF2 的写序臂（`m2-witness-r1-forks.md`）。pre 把条目写在取号那一次，根落盘之前条目已在，这一格不出现（pre 有自己的 H1）；post 与 late 都中。它也分辨「之后的挂载补不补」这一条条款没写的实现取法（下面的改法）。
2. **系统看得到吗**：看得到。后续挂载所选根的实例表里带着回退行，`logs/z9_b.log` 的原样行：`m=0 crash=true newest_root=(2,6) its_instance_table_rows=[(1,3,0,rollback)]`、`m=1 crash=true newest_root=(3,8) its_instance_table_rows=[(1,3,0,rollback) (2,6,0,-)]`。新实例代号 N 就是实例表里回退行之后第一个有行的实例（没有就是所选根自己的实例），(r_old, T_old) 就是回退行。`recovery.rs:1176` 已经在按「所选根实例的回退行」找东西。
3. **满足的是哪一分句**：第 373 行开头「（C332（回退实例两个根都读不出时回退被撤销） 的修法）」这个目的在崩溃之后不成立；代码对「见证随回退那一次挂载写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表」是照字面做的。归「替没写的条款做了选择」：条款没说那一次轮换没落时怎么办，代码选了「不补」。
4. **改法在打中的格上还中不中**：跑前条款没给改法。我提的一种（副本上 `SINGLEFS_R2_OPUS_WITNESS_REPAIR=1`：每次可写挂载删除规则之前，按所选根实例表里的回退行把缺的条目补进来），`logs/z9_b_repair.log` 的原样行：

```
m=0 crash_between_row_root_and_rotation=true witness=[] unreadable_root_txgs=[6] faults=1 -> FileRead root=(1,5) content_is_A=false content_len=2602
m=1 crash_between_row_root_and_rotation=true witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8] faults=3 -> FileRead root=(1,3) content_is_A=true content_len=3000
m=2 crash_between_row_root_and_rotation=true witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8, 9, 10] faults=5 -> FileRead root=(1,3) content_is_A=true content_len=3000
m=3 crash_between_row_root_and_rotation=true witness=[(2, 1, 3)] unreadable_root_txgs=[6, 7, 8, 9, 10, 11, 12, 13] faults=8 -> FileRead root=(1,3) content_is_A=true content_len=3000
```

m ≥ 1 三格修好（量过），m = 0 那一格（下一次挂载之前的那段）修不到——那是已记的 1 个故障窗口，只有改写序才动得了。A1、A2 在这个改法下照样第 24 次写满（`logs/z9_repair_z9_a1.log`、`…_a2.log`，量过）：它碰不到 Z9-A 那几格。**只在我的副本上量过、被攻过零轮**；从实例表推 N 的那一句在「一条回退行之后又回退过」的表上对不对，没测（推的）。

**什么现象会推翻这一格**：入库装置上照这段历史重做，m ≥ 1 时盘上见证表里有 (2, 1, 3)，或崩了的那一边恢复落在 (1, 3)。

### Z9 没打中的

- **删除规则删掉仍护着根的条目**：① 要求根环每一槽都读得出、且没有一条根的实例代号落在 [r_old, N)；条目抛弃的根的实例代号都落在 (r_old, N) 或等于 r_old，所以删的那一刻环里没有它抛弃的根；之后新写的根实例代号都 > 这次挂载取的号 > N。重放那一路：所选根的实例不可能是 r_old（环里没有），而前缀规则不跨实例边界。影子账那一路读的也是环里的根。A1–A3、B、对照三组历史里没有一次删掉之后择根变了。形状：只看了这几段历史与读代码，没做针对性的随机扫描。
- **崩在写见证之前、之间、之后，各盘各槽一新一旧**：见证读各盘择到的那一槽取并集（`recovery.rs` 的 `rollback_witness_of_the_pool`），轮换写了一块盘就在并集里；撕开的那一槽退回旧槽，旧槽是取号那一次写的、不带这一条——这就是 Z9-B 的同一格（两块盘的轮换都没成等于没写）。没另做逐槽撕裂的枚举：库里有 `second_transaction_supplement_two_rollback_witness_layer0.rs` 在管，我没跑它（不是我写的二进制）。
- **表满之前先因别的原因挂不上**：A3（每次回退到最新根、崩在暖机第一次的根 FUA 之前，那一次的记录已落）第 36 次回退被拒，原样：`k=36 ring_all_readable=true witness_entries_before=13 target=(36,99) REFUSED WarmUpAdmissionRefusedBeforeAcquisition { instance_to_acquire: InstanceGeneration(37), warm_up_publish_index: 1, warm_up_publishes_planned: 2, cause: AllocationRecordsExceedOneNode { records: 820, capacity: 812 } }`。分配记录树只有一个节点，正文第二节「已知」里归实二一，照写「已知」。

## Z8　发布失败原样重发、读者两格（没打中）

被判的字面：`decisions/23-journal的角色与格式.md` 第 383 行「**这一版的失败处置**」那一段（「一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置与字节都不变），重发成功才建下一次发布」）。代码：`transaction.rs:806`（闸）、`transaction.rs:863`（重发拿冻结时装好的同一份写）、`mount.rs:921`（抬 F 同样先查冻结）、`recovery.rs:1783`（末条标志之后同 txg 还有记录那一格）。

**试过的形状**（`tests/r2_opus_z8.rs`，日志 `logs/z8_z8_one_unit.log`、`logs/z8_z8_two_units.log`）：A 之后发 B（覆盖写 1 个数据单元 / 顺序写 2 个数据单元、两条记录）。一个包在两块盘外面的设备让第 i 次设备操作（写或屏障，两盘共一个计数）报错、什么都不写；i 从 0 扫到参照流长度的 2 倍，B 真失败了的都算：
- ① 重发：最后一遍重发与不失败的参照流逐项比（盘、种类、偏移、长度、内容哈希），系统配置轮换那几步只比盘、种类、长度，另记哈希同不同；
- ② 不重发、直接按此刻的盘面恢复，并重开一次可写挂载；
- ③ 重发那一遍在第 j 步再失败一次，再重发（j 扫到参照流长度的 2 倍）；
- ④ 重发那一遍逐前缀截断，各恢复一次，结局分成 A / B（根还是 3、施加了 B 的记录）/ B（根 4）/ 别的。

原样汇总（两单元那一份）：

```
    25  first attempt did not fail
    23  no-resend recover: FileRead(1,3) len=3000
     4  no-resend recover: FileRead(1,3) len=48951
     2  no-resend recover: FileRead(1,4) len=48951
    29  no-resend remount ok
     1  resend second_failure=false identical_except_sysconfig=true sysconfig_bytes_identical=false
    28  resend second_failure=false identical_except_sysconfig=true sysconfig_bytes_identical=true
    57  resend second_failure=true identical_except_sysconfig=true sysconfig_bytes_identical=false
   784  resend second_failure=true identical_except_sysconfig=true sysconfig_bytes_identical=true
   506  resend-prefix recover A
   169  resend-prefix recover B(root 3 + replayed record)
   137  resend-prefix recover B(root 4)
mismatches=0 []
```

一单元那一份：失败点 25 个，重发一遍 25 次、两次失败 625 次全部 `identical_except_sysconfig=true`，前缀恢复 342 A / 141 B(root 3 + replayed record) / 117 B(root 4)，`mismatches=0`。

- 系统配置哈希不同的那 1 格是失败点落在盘 1 的轮换上：盘 0 那一次已经写了，重发时盘 0 按「这块盘自证过的最大世代号 + 1」再写一次。条款点名要不变的五样（checkpoint_txg、计数器、序号、标志、单元位置与字节）不含系统配置，不算打中。
- 「B(root 3 + replayed record)」是根没落、记录落了，恢复照已定项 15 由记录施加 B；这是「失败报给了调用方，重开之后 B 却在」——条款没说失败必须不可见，不算打中，写在这里给主 agent 看。
- **冻结期间绕过闸**：读代码，会话里会发写的公开入口是 `publish_first_file`、`publish_version`（覆盖写、顺序写、建 inode 都经它）、`publish_instance_table_on_version_without_file`、`raise_rollback_floor`、`resend_the_frozen_publish`，前四个第一步查冻结。`publish_without_units`、`warm_up`、`acquire_instance` 不拿分配器、查不了冻结，但仓里调用它们的只有挂载、mkfs 那条流与用例；挂载本身开新分配器、取新号，冻结的那一次按 `allocator.rs` 的注释不带过去，(实例代号, checkpoint_txg) 不会撞。没造出一条合法历史让两条同 (实例, txg) 的末条标志落盘。
- 取样范围：一种文件大小两档、一个池（实例 1、A 之后的第一次发布）、失败只有「整次不写」一种（没做写了一半的撕裂失败）、没做持续性故障。

## Z12　取号之前的预演与真发（没打中）

被判的：冻结副本 `mount.rs:1968` 起那条断言（`mount.rs:1971` 那句消息「取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同……」）。

**怎么把新形态逼出来**：产品路径上挂载里的写入口一律 `PoolWriter::new`，容量取格式算的（`transaction.rs:194`），用例的只供测试开关进不了挂载（`mount.rs:1766` 自己开写入口）。所以在副本上改 `PoolWriter::new`，让它读环境变量 `SINGLEFS_R2_OPUS_CAPS`（记账树叶 / 内部、中央映射树叶 / 内部）与 `SINGLEFS_R2_OPUS_NAMED`（一条记录装几个点名项），整段历史（mkfs、第一个文件、覆盖写、挂载里的写行与暖机）用同一组容量。这是副本上的改动，**量出的数不算入库装置上的数**。

**扫描**（`tests/r2_opus_z12.rs`；两遍脚本 `run_z12_sweep.sh`、`run_z12_sweep2.sh`；逐组摘要 `logs/z12-sweep1/`、`logs/z12-sweep2/`）：
- 带文件的池：A 之后覆盖写到 txg 10（第二遍对第一遍在建池阶段撞分配记录墙的 21 组改成 txg 4），之后 7 轮（第二遍 3 组 14 轮）「可写挂载 → 在同一进程里覆盖写或建 3 个 inode 0..2 次」；每一轮挂载之前，对根环里每一条根各试一次回退挂载（不落盘），对上一轮最后那次发布的录制流逐前缀截断、各试一次可写挂载（不落盘，即「冻结着一次失败的发布、没重发就重开」的盘面）。
- 树表 0 条的池：只做过 mkfs，连着可写挂载 8 次，每次之前同样试回退到每条根、截断上一次挂载的每个前缀再挂载。
- 容量：记账树叶 {1,2,3,5} × 内部 {2,3} × 映射树叶 {1,2,3,4} × 内部 {2,3} 共 64 组；点名项 {1,2,3} 各跑产品容量与 (2,2,2,2) 两组。

原样汇总：第一遍 `lines=119 mounts_ok_total=18390 panics_total=0`，第二遍 `lines=24 mounts_ok_total=672 panics_total=0`。挂载之后那一版的树形（第二遍的 `main-line-shape`）有 `acct_nodes=30 map_nodes=44`、`acct_nodes=15 map_nodes=47` 这类，写行与暖机是在多层树上发的。

- 第一遍 21 组 `rc=101`，全部是建池阶段的覆盖写报 `AllocationRecordsExceedOneNode`（最大 `records: 944, capacity: 812`），不是那条断言；带文件那一路小容量下多数组只走到 8–25 次挂载就被主线挂载的 `WarmUpAdmissionRefusedBeforeAcquisition` 挡住（同一堵分配记录墙，已知、归实二一）。**这让带文件、小容量那一半的取样偏薄**，写在限度里。
- 「判相同但不该相同」那一面：断言在 `some_copy_was_quarantined` 为真时整条跳过（`mount.rs:1968`），那一格是冻结副本注释里自认的缺口（拷贝上不读盘核，C542）；我没造读盘核对不上的盘面，没攻这一面。
- 回退挂载写见证只改系统配置字节、不碰分配器，读代码判它不进落点序列；上面每轮的回退试挂载都经过它，没见分叉。

## 没打中的形状（汇总）

| 格 | 形状 | 取样 |
|---|---|---|
| Z8 | 失败点 × 重发 / 再失败再重发 / 不重发恢复与重开 / 重发逐前缀崩 | 1、2 个数据单元两档；失败点 25 + 29；两次失败 625 + 841 组；前缀恢复 600 + 812 次 |
| Z9 | 删除之后择根变不变；各盘一新一旧 | 只在 A1–A3、B、对照的历史里看，没做专门扫描 |
| Z12 | 小容量多层树上的写行 / 暖机、回退到每条根、截断前缀之后挂载、树表 0 条写行跨记录 | 第一遍 64 + 6 组、第二遍 24 组；19062 次挂载成功，0 次 panic |

## 这条腿自己的限度

- 全部数都是在冻结副本的拷贝上量的；Z9 的两格打中不依赖任何副本改动（`copy-only-core.patch` 的开关不设时走冻结副本的路径），但要引，按规则须在入库装置上重做。
- Z9-A 的历史靠「每次回退都崩在同一个窗口」：合法，但要 24 次崩溃连着落在同一段。它打的是「表写不满」这句断言，不是一个常见故障。
- Z9-B 的故障用的是「根环槽读不出」这一种读故障（`PoolReaderWithUnreadableRootRingSlots`），数的是根槽故障个数；没试系统配置槽读不出、记录读不出这些别的故障形态。
- Z9 的三种删除规则、Z9-B 的补见证，都是我自己提的，只在我的副本、我的几段历史上量过，被攻过零轮；「both」规则的上界是推的。
- Z12 带文件、小容量那一半多数组很快撞上分配记录墙，每组只有 8–25 次挂载；产品容量下挂载里走不到树分裂，我没去算产品容量下要多少条目才分裂（本地攻方在算每节点条目数）。
- Z8 只做了「整次不写」的失败，没做写了一半的撕裂失败、持续失败；只在一个池、实例 1 的第一次发布上扫。

## 没做什么

- 没判 Z7、Z10、Z11，也没碰算术格（见表、R × S − 1、每节点条目数、点名项上限）。
- 没跑任何不是我写的测试二进制（包括库里的见证层 0、故障注入、随机历史）；没跑门禁、没跑重型测试。
- 没把副本上的数做成入库装置的一次跑；副本在 `/tmp/claude-1000/m2-final-code-r2-opus/`（`tree/` 是改过的拷贝、`target/` 是编译产物、`logs/` 是全量原始日志），用例、补丁、脚本与摘要日志已拷进 `research/prompts/m2-final-code-r2-opus-model/`；`logs/z12/`、`logs/z12b/` 的全量日志只拷了摘要行，原件还在 `/tmp` 下。
- 没替主 agent 采纳改法；没改冻结副本本身、没改主工作区的 `crates/` 与 kb。
