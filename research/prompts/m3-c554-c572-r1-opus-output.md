# m3-c554-c572-r1 攻方腿（Opus）报告（2026-10-12）

立场：找反例。攻击面 B1–B5（正文「四、分工」云端攻方那一行）。代码行号按冻结副本 `~/.local/share/singlefs/work/m3-c554-c572-r1/repo/`；写报告时主工作区 `crates/singlefs-core/src/` 下 mount.rs、allocator.rs、recovery.rs、transaction.rs 四份的 sha256 与 `research/prompts/m3-c554-c572-r1-snapshot/frozen-copy-sha256.txt` 逐个相同，所以引文写仓内相对路径。副本上量出的数都是副本上的数，不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-c554-c572-r1-opus-model/`。整条经内存包装跑（工作目录随意，`--work` 给一个新的空目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-c554-c572-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m3-c554-c572-r1/repo --work <新的草稿目录>
```

它把冻结副本拷进 `<work>/tree`、打 `patches/core-arms.diff`（core 里 B2 / B4 / B5 的臂，环境变量选臂，空 = 今天）与 `patches/workspace-member.diff`（工作区加探针包）、把 `src/b3_unit_test.rs` 接到 `mount.rs` 末尾、把 `src/probe/` 拷成 `crates/attack-c554-probe`，编、跑，输出写 `<work>/out`，末尾打 `RERUN-CHECK name=… ok=…` 与 `out/` 里的数逐行比。`RERUN_LONG=1` 另跑两段各 270378 态的枚举（本机各约 17–20 分钟）与 k 扫描那几段。编译要现编 librocksdb-sys（本机 5 分钟上下）。`tools/rerun.sh --selftest` 不过：模板的自证在假仓里跑 rerun_steps，这一段要真仓（拷 `.cargo/config.toml`），自证第一格红（原样：「✗ 自检：拷进与仓无关的临时目录、传参应当跑通（退 0、仓根取 --repo、冻结副本拷进草稿目录、target 不拷），实际退 1：cp: cannot stat '/tmp/rerun-template-selftest.AbEnJv/fake-repo/.cargo/config.toml': No such file or directory」）。

每个文件的 sha256：

```
b5b1847e2c8730a5a8a2f96a1ea76f66e8108f260c920099b86a609ba9c9a108  research/prompts/m3-c554-c572-r1-opus-model/out/b1-k2-batch1.log
f17bce91606f9bc624961a716eff7c504e046f5c56172104e06392d1ac4644ac  research/prompts/m3-c554-c572-r1-opus-model/out/b1-rec-k3-4-6.log
2e77ac22a7b36c00367c63d2a2f9f0048af6d20d5be807b2daf88f064d522101  research/prompts/m3-c554-c572-r1-opus-model/out/b1-rec-k5-8-11.log
a1b1ef23f5680207e5186d1dc1e866e38e5ebff2cae7a12d2a3a1668bfdc4dc7  research/prompts/m3-c554-c572-r1-opus-model/out/b1scan.log
80a32fde3662df10b14159d0ef00b894514d526f02ae75095068b02a129d14c4  research/prompts/m3-c554-c572-r1-opus-model/out/b1scan-m2ow.log
998bdeda358c65fdcbd9f2d8a07743f51584d3f46bf97b86f831f21a6311e4c0  research/prompts/m3-c554-c572-r1-opus-model/out/b1scan-widths.log
3b7d5233c7b8bc7fc9fd8847ad06a8d6123eca767a3cc84046d0e3a3c4bb0c3a  research/prompts/m3-c554-c572-r1-opus-model/out/b1-W240-k10.log
983f78430fc6f909d5df945e86e6d3e44870fca2712a80607cdfa03b7b1fbee1  research/prompts/m3-c554-c572-r1-opus-model/out/b1-W240-k10-nofault.log
145b7275ecbc41f6cac7596ce9096b803d9cceb3099f2735ad1902f436fea7bd  research/prompts/m3-c554-c572-r1-opus-model/out/b1-W240-k10-rec-skiponce.log
2425f969becfbe2cb120351b1ed79c3b6f9dbdd9e973f68e6c74d2ba61e9035d  research/prompts/m3-c554-c572-r1-opus-model/out/b1-W240-k10-rec-today.log
dd22111283a6fee3aba3d55c01efe08ead2aaf931c1b2ef6706e2702077470f2  research/prompts/m3-c554-c572-r1-opus-model/out/b2.log
c60fe578826a34f95fd175958760447ee4c6d081c6ef27d6f3c016b9b4386ecc  research/prompts/m3-c554-c572-r1-opus-model/out/b2-pressure.log
dad93e3bfff9437354fec68beeda81a36eefdb2ca808952bd28ad79c05c8973a  research/prompts/m3-c554-c572-r1-opus-model/out/b3-test.log
4bb84697e0c5dea3f554c575402aba97148aba756f133f52286bba595aba4233  research/prompts/m3-c554-c572-r1-opus-model/out/b4.log
524a2431cb1079b172724024dc526b5dd57ed827a9cb5de1dc2e88f8e8dd3f63  research/prompts/m3-c554-c572-r1-opus-model/out/b5-full.log
bb6f35199b35136a9c2379db94a78f378456850db549962e0a4f09659f06c230  research/prompts/m3-c554-c572-r1-opus-model/out/b5-v2.log
68fa5148ae15aba078ec53efbdb69a153ac473d4a380e14acb730853ac504a3c  research/prompts/m3-c554-c572-r1-opus-model/out/trial-b1.log
63b27f7803f06fdf140bc354c0e2b3cb0b0bbbd6492e81a884f812f8e3eac584  research/prompts/m3-c554-c572-r1-opus-model/patches/core-arms.diff
59bdbb82eb01535c6d0dc974d2c43637838fd47e368e4f7c94353d94bedcb0a8  research/prompts/m3-c554-c572-r1-opus-model/patches/workspace-member.diff
8922b2e81444e39129499771f089f6f055a0499e73b496395329dbb254c2eb38  research/prompts/m3-c554-c572-r1-opus-model/src/b3_unit_test.rs
a110e088ad549955715227678a2b0e8321d2e3b04ef2045edc553df41cd3b306  research/prompts/m3-c554-c572-r1-opus-model/src/probe/Cargo.toml
f26ff1fca112adc8c8a6aeb98604902f7bdc92e233e4c5ecb92376bddb218b2c  research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/b1.rs
ed1d404f7a6dfc951375bb0d4e3174099797fe2e32da1bb180cf969c87cfe3a8  research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/b2.rs
f4f4cefba3c788aa9c375d014007689a3f86ed82e73c2456b9c4fa106b772eba  research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/b5.rs
b1ed9121fc7ef466dd32199de4a5c5c03c37908b31572ed42645f337f95c3875  research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/main.rs
6535b6751b2feb9fb4950b7a70dc77760ba10ff24ca8ae2b52e457f6551a3e07  research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/util.rs
0611cb58bf40c9e91d70a26a4bcae3c4124fdf07556732739d95631b260754e4  research/prompts/m3-c554-c572-r1-opus-model/tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 打中了什么（每条标量过 / 推的） |
|---|---|---|
| B1 | 部分：批 17 那一形（P 的根槽与单元读回全零、记录读得出）已被「可写挂载时的预隔离」吃掉；P 的记录也读回全零的那一形还在 | 量过：那一形在一段 20516 态的崩溃枚举里丢盘 0 之后只读挂载打不开 12296 态；候选甲（单盘择根避开）按「看得到新实例的实例表」认，只罩住 12296 里的 3 态；按「系统配置记着更新的实例」认，在同一段历史的无故障对照（270378 态）里 270363 态把没被抛弃的那条根也避开（推的：退到更旧的根、丢两版已确认的）；候选乙（影子账保守集合）判别子观测不到（推的，读码） |
| B2 | 还缺：`None => continue` 那一支今天在我的取样里没在同一个进程里打出丢写，但四个候选都碰不到跨进程那一形 | 量过：R 的根槽一直读回全零、卸载之后再可写挂载一次，今天与四个臂 R 的落点逐字相同地被写掉（继承的 2/2、新分配的 18/20）、只读挂载打不开；窄盘（240 槽）上 released / allocated / allocated_released 三臂比今天早报空间不够（k = 3 早 1–3 次覆盖写）；mapping 臂在映射也读不出时拒可写 |
| B3 | 部分：实现偏离条款（条款按「槽」），偏离不止 ③ 那一处，共五处按起点槽认 | 量过（模型原样调被测函数）：三种形各漏 1 槽；形一的结局随根环读出的次序变 |
| B4 | 部分：射程② 那一句「只剩系统配置没见证到的那一种」不成立；E158 那一机理在两块盘都在时今天被 F2 吃掉；「新实例第一次发布避开读不出的槽对应的 txg」两头都有 | 量过：见证过、已确认的 P 在 P 的根槽、记录与每块盘较新那个系统配置槽一时读回全零时被抛弃（k = 2、3、4、5、10 五个起点全被抛弃）；收严臂 `skip_once` 在 B1 那段历史上把「新实例的根全坏」的只读失败从 0 / 20516 抬到 8219 / 24623，在 B4 那段历史上 5 个起点里 4 个找回已确认的 P、1 个整池打不开 |
| B5 | 还缺（打中） | 量过：同一份坏副本，没隔离时回退被拒，已隔离时回退做成、新根引用它、丢另一块盘后回退出来的那一版读不出；「隔离单元也进逐盘验」修这一格、多读 1 个单元；「直接拒」在隔离而完好的单元上错拒 |

## 取样、分批与估时

- 盘：`common_admission_pool` 那一档内存稀疏盘（两块，单元区默认 384 槽；`UNIT_SLOTS` 改单元区槽数，扫过 240、256、320、384、448、512）。每段历史起点池只建一次，之后拷。
- 崩溃枚举：探针调 `singlefs_checker_tier::layer0_enumeration::enumerate_layer0_with_expansion` + `full_expansion`（每一段都展开、原地覆写三态），每个状态枚举器自己跑恢复与池级 checker，另挂一个观察者。跑之前用 `layer0_state_count_with_torn_in_place_overwrites` 算状态数，超过 10⁶ 不跑。判定库只认 checker 档包 `tests/` 下的候选设点文件，探针借了一份现成的设点文件名（`crash_enumeration_floor_raise_pushed_by_admission.rs`），判定库放在草稿目录、每趟清空。
- 这一条腿枚举过的状态合计 856690（各段的 `OPUS-TALLY … states=` 相加，含一段被后来的输出盖掉的试跑 20511），不到 10⁷。没跑的一段：P 那次覆盖写自己的写流（外层，看崩在 P 的哪一刀会让 P 被抛弃）状态数 1048588 > 10⁶（`OPUS-SKIP name=b1_k2_unit_pb2999_m0_outer states=1048588 closed_form=1048583 reason=over_1e6`），没跑；每段只取「P 的根槽 FUA 落了、系统配置轮换没落」那一刀。会话里再覆盖写的写流同理（一次覆盖写一段 20 次写），没枚举，B2 那几段历史因此不带崩溃。
- 估时：先跑 20511 态一段 24.5 秒（28 线程），按它估 270378 态约 6 分钟；机器上同时有别的会话的变异复跑，实际 17–20 分钟。批没设限时。

## B1　单盘恢复择中被抛弃的根，暖机落满之前（C554 ①）

### 1.1 历史（`MODE=b1`，`src/probe/src/b1.rs`）

1. 新池第一个文件之后在同一个进程里覆盖写 k 次（实例 1），最后一版叫 S；再覆盖写一次 P，崩在 P 的根槽 FUA 之后、系统配置轮换之前（P 没被见证）。
2. 撤到进程外，可写挂载一次（实例 2），挂载期间 `HIDE` 那组落点读回全零（报成功）。择根落到 S，实例 2 写行与暖机；P 被实例 2 的实例表抛弃。
3. 枚举实例 2 那次挂载写流的每个崩溃状态；每个状态撤故障，两块盘都在 / 丢盘 0 / 丢盘 1 各做一次只读挂载、整文件读回，按落到的根比内容；另把实例代号大于 P 的根槽全清零、两块盘都在再做一次（`newbad`，B4 用）。

每一步许可它的那一句：P 没被见证时可写挂载不重读、不拒（`.claude/kb/decisions/23-journal的角色与格式.md:394`「判不出 ⇒ 按真」那一段只在系统配置见证过比所选那一版新的发布时才重读）；C 那一圈只认读得出的记录（`crates/singlefs-core/src/mount.rs:1822`「for record in records.values().filter(|record| {」，`records` 是读阶段扫出、读得出的那几条）；P 的根槽读回全零、不在 `roots` 里，影子账只看 `roots`（`crates/singlefs-core/src/mount.rs:1581`「.filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)」）。

### 1.2 「已被吃掉」那一半：批 17 那一形（量过）

`HIDE=unit`（P 的根槽 + P 的数据单元两份读回全零，记录读得出）：C 那一圈罩住 P 的记录点名的单元，挂载交回 `isolated=[(DeviceIdentity(0), 12), (DeviceIdentity(1), 12)]`。快扫（不枚举，只数实例 2 那次挂载的单元写盖到 P 新分配的几个槽）在 384 槽盘、k = 2..14 上 13 个起点全是 0（`out/b1scan.log` 里 `hide=unit` 的 13 行全是 `overlapped_by_mount2_unit_writes=0`）。枚举两段原样（`out/b1-W240-k10.log`、`out/trial-b1.log`）：

```
OPUS-TALLY name=b1_k10_unit_pb2999_m0_inner states=270378 closed_form=270358 enumerated=270378 observed=270378 oracle_violations=0 CV secs=1041.3 counts=[both_rec_abandoned_read_ok=262155 both_rec_other_read_ok=8223 both_ro_abandoned_read_ok=262155 both_ro_other_read_ok=8223 lost0_rec_abandoned_failed=270369 lost0_rec_abandoned_failed_invariant_violated=270369 lost0_rec_other_failed=9 lost0_rec_other_failed_invariant_violated=9 lost0_ro_abandoned_read_ok=270369 lost0_ro_other_read_ok=9 lost1_rec_other_failed=270378 lost1_rec_other_failed_invariant_violated=270378 lost1_ro_other_read_ok=270378]
```

（`CV` 是我在摘行时把 `checker_violated={…}` 那一长串换掉的，那一串每条不变量都是 0，原文在草稿目录的整份日志里；`*_rec_*` 是 `recover()` 那一路，它在缺一块盘时多数态报 `InvariantViolated`（I-1.1），我不拿它当单盘的结论；只读挂载那一路 `*_ro_*` 才是单盘读的结论，下同。）丢哪块盘都 0 失败。批 17 报的「99 个只读挂载读不出」在今天这一形上是 0（240 槽、k = 10 一段；其余起点见快扫）。

### 1.3 还在的那一半：P 的记录也读回全零（量过）

`HIDE=rec`（P 的根槽 + P 那条记录两份）：C 那一圈是空的（`isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)]`）。快扫找写行与暖机盖到 P 新分配的槽、且 P 与 S 的根不在同一块盘上的起点：单元区 240、256、320、448、512 槽 × k = 2..20 共 95 个起点里 4 个（`out/b1scan-widths.log`：240 槽 k = 10、14、19，320 槽 k = 16）；384 槽、k = 2..14 那 13 个起点里盖到的只有 k = 5、11，而那两个 P 与 S 的根同在盘 0（`out/b1scan.log`）。240 槽 k = 10 那一段枚举原样（`out/b1-W240-k10-rec-today.log`）：

```
OPUS-TALLY name=b1_k10_rec_pb2999_m0_inner states=20516 closed_form=20496 enumerated=20516 observed=20516 oracle_violations=0 CV secs=129.6 counts=[both_rec_abandoned_read_ok=5192 both_rec_other_read_ok=15324 both_ro_abandoned_read_ok=9224 both_ro_other_read_ok=11292 lost0_rec_abandoned_failed=16400 lost0_rec_abandoned_failed_invariant_violated_invariant_i_1_1_detail_key=4104 lost0_rec_abandoned_failed_mapping_still_unreadable_slot_slot_number_1601=2048 lost0_rec_abandoned_failed_unit_unreadable_slot_slot_number_1603=2048 lost0_rec_abandoned_failed_unit_unreadable_slot_slot_number_1604=8200 lost0_rec_other_failed=4116 lost0_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=4116 lost0_ro_abandoned_read_ok=4104 lost0_ro_hit_sc_newer_true_table_false=12293 lost0_ro_hit_sc_newer_true_table_true=3 lost0_ro_other_failed=12296 lost0_ro_other_failed_ro_mount_open_walk_unit_unreadable_slot_slot_number_1603=8200 lost0_ro_other_failed_ro_mount_open_walk_unit_unreadable_slot_slot_number_1604=4096 lost0_ro_other_read_ok=4116 lost1_rec_other_failed=20516 lost1_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=20516 lost1_ro_other_read_ok=20516 newbad_ro_abandoned_read_ok=9224 newbad_ro_other_read_ok=11292]
```

丢盘 0（S 的根 txg 15 在盘 0，P 的根 txg 16 在盘 1）之后，20516 个崩溃状态里 12296 个只读挂载打不开（`Walk(UnitUnreadable)`，槽 1603 / 1604 是写行那次盖到的 P 的单元）。两块盘都在时 0 失败：落到 P、P 走读不通时 F2 退回 S。单盘时 F2 退不了：退的范围是 [见证到的那一版, 所选根)，见证到的 S 那条根在丢掉的盘上，`crates/singlefs-core/src/recovery.rs:5673`「.max_by_key(order_of_the_root_choice)?;」交 `None`。

这一形要的故障：P 那次崩在根槽之后、轮换之前（一次崩溃）；实例 2 挂载时 P 的根槽与 P 那条记录两份一时读回全零（三段、跨两块盘）；实例 2 暖机落满之前再崩（第二次崩溃）；丢掉 S 的根所在那块盘；写行的落点恰好取到 P 的槽、P 与 S 的根又不在同一块盘（108 个快扫起点里 4 个）。

### 1.4 三个候选在打中的那几格上

**甲　择根那一侧单盘时避开被抛弃的根。** 单盘那一刻要认出「P 被抛弃」，盘上能用的只有两样：剩下那块盘上读得出一条实例 2 的记录、按它的新根段读得出实例 2 的实例表、那张表判 P 被抛弃（甲一）；或者只看系统配置里取过号的实例代号比 P 的新（甲二）。观察者在每个只读失败的状态上都数了这两样（`lost0_ro_hit_sc_newer_{真假}_table_{真假}`）：

- 甲一只罩住 12296 个失败态里的 3 个（`lost0_ro_hit_sc_newer_true_table_true=3`）：多数失败态崩在实例 2 写行的单元已落、记录还没落的那一段，剩下那块盘上没有实例 2 的实例表。量过。
- 甲二罩住全部 12296（`sc_newer_true` 两项相加）。同一段历史不注故障的对照（`HIDE=nofault`：实例 2 挂载时 P 读得出、择中 P、P 没被抛弃）原样（`out/b1-W240-k10-nofault.log`）：

```
OPUS-TALLY name=b1_k10_nofault_pb2999_m0_inner states=270378 closed_form=270358 enumerated=270378 observed=270378 oracle_violations=0 CV secs=1202.2 counts=[both_rec_abandoned_read_ok=262155 both_rec_other_read_ok=8223 both_ro_abandoned_read_ok=262155 both_ro_abandoned_read_ok_sc_newer_false_table_false=6 both_ro_abandoned_read_ok_sc_newer_true_table_false=262149 both_ro_other_read_ok=8223 lost0_rec_abandoned_failed=270369 lost0_rec_abandoned_failed_invariant_violated_invariant_i_1_1_detail_key=270369 lost0_rec_other_failed=9 lost0_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=9 lost0_ro_abandoned_read_ok=270369 lost0_ro_abandoned_read_ok_sc_newer_false_table_false=6 lost0_ro_abandoned_read_ok_sc_newer_true_table_false=270363 lost0_ro_other_read_ok=9 lost1_rec_other_failed=270378 lost1_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=270378 lost1_ro_other_read_ok=270378 newbad_ro_abandoned_read_ok=270378]
```

（这里 `abandoned_*` 只是「落到 P 那条根」，这段历史里 P 没被抛弃。）丢盘 0 时 270378 态里 270363 态剩下那块盘上 P 是最新的根、系统配置记着实例 2、看不到实例 2 的实例表——与打中那一段的失败态在系统那一刻看得到的东西逐条相同。甲二在这 270363 态上也避开 P（量过的是条件成立的态数）；避开之后落到盘 1 上下一条根 txg 13，txg 14、15 两版已确认的丢掉（推的：14、15 的根在盘 0，单盘时记录点名单元两份验不了、不施加，按 `.claude/kb/decisions/23-journal的角色与格式.md:365`「两份都验过才施加」）。今天这 270363 态读回 P、内容对。

**乙　影子账在读不出时退到更保守的集合。** 打中那一形里实例 2 挂载那一刻看到的是：txg 16 那个根槽读回全零（环才写到 txg 15，环长 24，这一槽本来就没写过，全零是正常的），P 那条记录的槽读回全零（像 journal 的尾巴）。两样都与「P 从来没写过」逐条相同，判别子观测不到（推的，读码；量过的旁证：`isolated=0`）。要它触发只能无条件保守，而「保守」要罩的是 P 新分配的、在 S 的账里本来就空闲的槽——那就是全部空闲槽。若按回退那一支的退法「现行账里全部已释放的落点」，罩不到 P 新分配的槽（它们在现行账里没有记录，不是已释放），改法碰不到打中的格（推的）。

**丙　认作多重故障下的已知边角写进射程。** 这一形要两次崩溃、三段一时读回全零、丢一块盘，外加落点巧合（1.3 末段）。写进射程时要写清：两块盘都在时今天被 F2 兜住（20516 态里 0 失败）；丢盘之后只读挂载整池打不开，不是读回旧一版。

四句：①分不分辨臂——甲一 / 甲二 / 乙 / 丙在这一格上结局不同，分辨。②系统当时看不看得到——甲一的判别子在 3 / 12296 个失败态上看得到；甲二的判别子在失败态与合法对照上逐条相同（判别子观测不到那一形）；乙看不到。③满足的分句——「只读挂载做不做得成」做不成（`MountReadOnlyFailure::Open(Walk(UnitUnreadable))`）。④改法在打中的格上——甲一对 12293 态不起作用；甲二修全部但在合法对照上错避开 270363 态；乙碰不到。

## B2　水位之上没施加的发布账读不出（C554 ②）

**判定：还缺。** 今天那一支：`crates/singlefs-core/src/mount.rs:1944`「None => continue,」——记录齐（不走 `crates/singlefs-core/src/mount.rs:1932`「None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(」那一支）、账读不出时那次发布的账一个落点都不进 C。同一件事在「较新可读根」那一边（根槽读得出）走丙：`crates/singlefs-core/src/mount.rs:2063`「placements_still_allocated_in_the_account(current_records);」。两边的区别只是那条根的根槽读不读得出。

### 2.1 历史（`MODE=b2`，`src/probe/src/b2.rs`）

起点同 B1（k 次覆盖写，S；R 崩在根槽之后、轮换之前）。可写挂载（实例 2）期间 R 的根槽与 R 的账读回全零：`acct` 是 R 的树表那一片两份，`acct+map` 再加 R 的映射根两份；R 的记录齐、读得出。R 点名的单元里有树表，施加前验不过，R 不施加（前缀第四条）；F1 乙不拒（记录齐）；C 只拿到 R 的记录点名的落点。之后同一次挂载覆盖写 j 次、正常卸载（抬 F、回收）。`MOUNT3=1` 再可写挂载一次（R 的根槽仍读回全零，故障是持续的）、再覆盖写 j 次、卸载。每一段之后：R 从 S 继承的单元（R 的账里仍分配、不是 R 新分配的，`inherited`）与 R 新分配的（`r_new`）各有几个被写掉；再把比 R 新的根槽全清零，只读挂载读回什么。臂在副本的 `mount.rs` 里（`OPUS_B2_ARM`）：空 = 今天；`released` = 退到现行账里全部已释放的落点（正文 B2 第二个候选）；`allocated` = 现行账里仍分配着的落点（丙，较新可读根那一边今天的做法）；`allocated_released` = 两样都并；`mapping` = 记录齐时也走记录不齐那一支（从映射现算、重读一次、看凭据、仍不成拒，即正文的「重读一次再判」加「拒可写」）。

### 2.2 量到的（副本，原样摘行，`out/b2.log`、`out/b2-pressure.log`）

同一个进程里（k = 2..5，j = 3；384 槽 j = 8、14；240 槽 k = 2 j = 8），五个臂都是 `changed_inherited=0 changed_new=0 r_root_slot_still_r=true`，把更新的根清零后读回 R。例：

```
OPUS-B2 k=2 hide=acct arm="" j=3 stage=after_mount2_unmount r_txg=6 inherited=2 r_new=20 changed_inherited=0 changed_new=0 r_root_slot_still_r=true newer_roots_zeroed=7 ro_after_zeroing=Some(6):reads_r eff_txg=5 instance=2 isolated=[(DeviceIdentity(0), 12), (DeviceIdentity(1), 12)] last_txg=13 unmounted=floor_raised_to_11_publishes_2
```

再挂一次（R 的根槽仍读回全零）之后，五个臂逐字相同地全丢：

```
OPUS-B2 k=2 hide=acct arm="" j=3 stage=after_mount3_unmount r_txg=6 inherited=2 r_new=20 changed_inherited=2 changed_new=18 r_root_slot_still_r=true newer_roots_zeroed=16 ro_after_zeroing=None:failed_ro_mount_open_walk_unit_unreadable_slot_slot_number_1511 eff_txg=13 instance=3 isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] last_txg=22 unmounted=floor_raised_to_19_publishes_3
```

（`out/b2.log` 里 k = 2..5、`acct` 五个臂、`acct+map` 四个臂（`mapping` 在那里第二次挂载就拒了）共 36 行 `after_mount3_unmount`，每行都是 `changed_inherited=2` 且只读挂载 `failed`，可用 `grep -c 'after_mount3_unmount.*changed_inherited=2.*failed' out/b2.log` 数。）

窄盘（240 槽，j = 8）上各臂在第几次覆盖写报空间不够（`NoSpaceAfterRaisingTheFloor`，`out/b2-pressure.log`）：（覆盖写从 0 数，`refused=overwrite_N` 的 N）k = 2 今天 8 次全做成，`released` 在 5，`allocated` 在 7，`allocated_released` 在 4，`mapping` 全做成；k = 3 今天在 6，`released` 在 3，`allocated` 在 5，`allocated_released` 在 3，`mapping` 在 6。`acct+map`（映射也读回全零）时 `mapping` 臂 k = 2..5 四个起点全拒可写（`stage=mount2 refused=mount:newer_state_still_unreadable_after_one_reread_account_of_an_unapplied_publish_with_incompl`，行尾被我的计数名截断）。

### 2.3 读法与四句

- 同一个进程里的丢写（C554 登记行 ② 那一形，`checks-owed.md` 记的是 batch10z-no 判决里的旁证）在我的取样里一次都没出现，所以今天这一支在同一进程里的代价我量不出（没打中，一次抽样，见末节）。
- 跨进程那一形五个臂一样丢：卸载之后 F 抬到现行那一版，再挂时 R 的记录已在新水位之下（实例代号为主比），C 不看它；R 的根槽读回全零，影子账也不看它。B2 的每个候选都只改 C 那一份、只护到「这个进程写掉 R 那一槽」或进程结束，碰不到这一格（量过，五臂同数）。这一格就是 C554 登记行「各形的机理」那一句「可写挂载那一刻被抛弃根的根槽读回全零或读错，根环表不记它」。
- 代价（量过，窄盘）：`released` / `allocated` / `allocated_released` 三个臂把 R 的账换成现行账的一大截，在 240 槽盘上比今天早报空间不够（k = 3 时早 1–3 次；k = 2 时今天 8 次全做成而它们在第 5–8 次就报）；`mapping` 在映射读得出时只多护 R 真引用的那几个，一次覆盖写都不少，但映射也读不出时拒可写——拒的那一刻 R 从没被确认过（轮换没落），只为多重故障下退回 R 留着它的单元。
- 四句：①分辨臂——同一进程里五臂同数（不分辨，我的取样里），窄盘上的空间代价与 `acct+map` 的拒分辨。②看得到——R 的记录读得出，系统知道「水位之上有一次发布、账读不出」，看得到；跨进程那一格新挂载看不到 R（在水位之下、根槽全零），看不到。③分句——「只读挂载做不做得成」做不成。④改法——四个候选对跨进程那一格都不起作用；同一进程那一格我没打中，分不出谁修谁不修。

候选 `mapping` 是把记录不齐那一支原样搬过来，`allocated` / `allocated_released` 是把较新可读根那一边的丙原样搬过来；都只在我的模型上量过、被攻过零轮。

## B3　C 那一圈按起点槽去重（C554 ③）

**判定：部分——实现偏离条款，条款本身写的是「槽」；而且偏离不止 ③ 写的那一处。**

条款按槽说：`.claude/kb/decisions/23-journal的角色与格式.md:390`「分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽」（原文 `**` 加粗，比对去掉）。实现按「起点槽」当一个落点的身份，有五处：

- 现行账那一份按起点槽收：`crates/singlefs-core/src/mount.rs:1573`「.map(|record| (record.device.0, record.slot.0))」——现行账 (s, 跨 1) 让被抛弃根 / C 的 (s, 跨 2) 整份豁免。
- 候选根那一份按起点槽收：`crates/singlefs-core/src/mount.rs:1587`「.map(|(device, slot, _)| (device.0, slot.0)),」——同上，换成候选根。
- 被抛弃根那一圈：`crates/singlefs-core/src/mount.rs:1641`「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」——两条被抛弃根同起点、窄的先进集合，宽的整份跳过（③ 那一句）。
- 根槽读坏的被抛弃根那一圈：`crates/singlefs-core/src/mount.rs:1665`「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」——同上。
- C 那一圈：`crates/singlefs-core/src/mount.rs:1686`「if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {」——现行账 (s, 跨 1) 让 C 的 (s, 跨 2) 整份跳过。

### 3.1 模型：拷出被测函数原样调（量过）

在冻结副本拷贝的 `mount.rs` 末尾接一个 `#[cfg(test)]` 模块（`research/prompts/m3-c554-c572-r1-opus-model/src/b3_unit_test.rs`），直接调 `isolate_slots_referenced_only_by_abandoned_roots`，落点从「根环表里记着的」那一支给（不读盘，盘读一次就 panic），一块 384 槽单元区的空闲图。命令与原样输出（`out/b3-test.log`）：

```
OPUS-B3 slot_s_free=false slot_s_plus_1_free=true isolated_slots=0
OPUS-B3-FORM3 got=(false, true)
OPUS-B3 slot_s_free=false slot_s_plus_1_free=true isolated_slots=0
OPUS-B3 slot_s_free=false slot_s_plus_1_free=true isolated_slots=1
OPUS-B3-FORM2 got=(false, true)
test mount::attack_b3_isolation_keyed_by_start_slot::opus_b3_c_ring_placement_with_a_current_record_at_the_same_start_slot_is_skipped_whole ... OPUS-B3 slot_s_free=false slot_s_plus_1_free=false isolated_slots=2
OPUS-B3-FORM1 narrow_first=(false, true) wide_first=(false, false)
ok
test mount::attack_b3_isolation_keyed_by_start_slot::opus_b3_current_account_record_at_the_same_start_slot_exempts_the_whole_wider_placement ... ok
test mount::attack_b3_isolation_keyed_by_start_slot::opus_b3_two_abandoned_roots_narrow_first_leaves_the_second_slot_of_the_wide_one_free ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 248 filtered out; finished in 0.00s
```

（三条用例并行，打印交错；每条的 `FORMn` 行是它自己的结论。）形一：两条被抛弃根 A1 (s, 跨 1)、A2 (s, 跨 2)，A1 在 `roots` 里排前时 s+1 发得出去（`narrow_first=(false, true)`），A2 排前时两槽都罩着——结局随根环里谁先读出来而变。形二：现行账 (s, 跨 1) 仍分配、被抛弃根引用 (s, 跨 2)，s+1 发得出去。形三：C 那一圈 (s, 跨 2)、现行账 (s, 跨 1)，s+1 发得出去。

### 3.2 这是条款没写还是实现偏离

条款写的是「槽」，没写「落点」或「单元」：按字面，s+1 只被被抛弃根引用（形二、形三里现行账与候选根只引用 s），该隔离；实现没隔离。所以是**实现偏离条款**，不是条款缺一句。条款没写的只有一件：同一槽被不同跨度的两份引用、而两份不可能同时活着（一份已被盖掉）时，要不要还护后一份——按「槽」读，照护。

### 3.3 改法与要动的地方（推的，没实现）

把上面五处的集合与判重改成按槽展开：`referenced_by_the_current_account`、`referenced_by_candidates` 收落点覆盖的每一槽；三处 `isolated.insert(key)` 改成「落点覆盖的槽里还有没隔离、不被候选与现行账引用的，就隔离那几槽」。只动 `crates/singlefs-core/src/mount.rs` 这一个函数；分配器那一侧 `isolate_the_slots_not_allocated…` 本来就按跨度逐槽隔离，不用动。代价：多隔离的槽数 = 每处被漏的槽数（形一至形三各 1 槽），推的。

### 3.4 池级 checker 为什么没红、要不要加

池级 checker 判的是镜像，不判内存里的分配器：I-7.4 被抛弃根那一半是沿被抛弃根走读、看单元校验和对不对（`crates/singlefs-checker/src/walk.rs` 里 I-7.4 被抛弃根那一半的文档注释），漏罩的 s+1 只有真被写掉、写落了盘、那条根还在环里时才红。判别力缺的一格是「挂载交回的分配器里，每条被抛弃根落点覆盖的每一槽，除了候选根与现行账引用的，都隔离着」——这是对分配器的判据，该加在 harness 档的挂载用例上（上面三条模型用例就是这样判的），不是加在池级 checker 上；按今天的实现三条都红（上面 `got=(false, true)`），按槽展开之后该转绿（推的）。

## B4　被抛弃实例的根重开时全部暂时读不出（C554 机理那一形）

**判定：部分。** 射程② 的后半句不成立；E158 那一机理（新实例第一次发布取了与被藏根相同的 txg、盖掉它的根槽）今天在两块盘都在时恰好起保护作用；提的收严（新实例第一次发布避开读不出的槽对应的 txg）两头都有，量过。

### 4.1 射程② 那一句（量过）

被判的句子在 `.claude/kb/decisions/23-journal的角色与格式.md:413`「乙-配置续之后这一形只剩被抛弃根是系统配置没见证到的那一种：见证过的在崩溃恢复那次挂载里重读一次仍读不出就拒可写，抛弃不了」。反例（`MODE=b4`，`src/probe/src/b1.rs` 末尾的 `main_b4`）：P 整次发布做完、系统配置轮换也落了（见证过、已确认）；进程退出，下一次可写挂载时 P 的根槽、P 那条记录两份、每块盘世代号较大的那个系统配置槽一时读回全零（报成功、不是设备报错）。原样（`out/b4.log` 今天那几行）：

```
OPUS-B4 k=2 hide=rec+scnew arm="" p=(InstanceGeneration(1), CheckpointTxg(6)) effective=(InstanceGeneration(1), CheckpointTxg(5)) instance2_txgs=[6, 7] p_root_slot_kept=false p_new_units_overwritten=0/20 ro_both=Some(7):other_read_ok ro_after_new_instance_roots_zeroed=Some(5):other_read_ok
OPUS-B4 k=5 hide=rec+scnew arm="" p=(InstanceGeneration(1), CheckpointTxg(9)) effective=(InstanceGeneration(1), CheckpointTxg(8)) instance2_txgs=[9, 10] p_root_slot_kept=false p_new_units_overwritten=14/20 ro_both=Some(10):other_read_ok ro_after_new_instance_roots_zeroed=Some(8):other_read_ok
```

k = 2、3、4、5、10 五个起点（`rec` 与 `rec+unit` 两种藏法）十行全是 `effective` = P 的前一版：见证过的 P 被抛弃，撤故障之后读回的是 S 的内容，P 永久丢。机理：每块盘较新那个系统配置槽读回全零 ⇒ 自证不过、不算「有槽设备报读错」，判不出那一判（R2）不成立，c_见证 取较旧那个槽的 tail，落回 P 之前，P 算没见证过。条款里另有一处认了同一类代价：`.claude/kb/decisions/23-journal的角色与格式.md:421`「已确认的版本在较新的系统配置槽与它的单元同时读回全 0 时，恢复、可写挂载与只读挂载悄悄读回上一版」——那一句写的是「系统配置槽与它的单元」，这里藏的是系统配置槽、根槽与记录（单元读得出），也被抛弃。两处说法不一致：射程② 说见证过的抛弃不了，射程末条（F2 的代价）认了见证过的会被抛弃。

### 4.2 E158 那一机理今天的样子（量过）

上面十行里 `instance2_txgs` 的第一个都等于 P 的 txg、`p_root_slot_kept=false`：实例 2 的写行取了 P 的 txg（`crates/singlefs-core/src/mount.rs:1316`「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」，P 的根与记录都读回全零，最大值落在 S），盖掉 P 的根槽。B1 那段枚举里把实例 2 的根全清零（射程② 写的「撤故障后新实例的根全坏」）再两块盘只读挂载：20516 态 0 失败（`newbad_ro_abandoned_read_ok=9224 newbad_ro_other_read_ok=11292`，见 1.3 原样行）。落到 P 而 P 读不通的，F2 退回 S；P 的根槽已被盖掉的，直接落到 S。所以射程② 的「恢复落在被抛弃的根上读不出」在两块盘都在、S 读得出时今天不出现；要出现得 S 也读不到（单盘，即 B1 那一格）。

### 4.3 收严「新实例第一次发布避开读不出的槽对应的 txg」（量过，臂 `OPUS_B4_ARM=skip_once`）

臂的读法取最窄的一种：第一次发布那个 txg 的根槽这一刻读不出、自证不过或读回全零，就往后挪一个 txg（`patches/core-arms.diff` 里 `first_txg_of_new_instance` 那一段）。B1 那段历史（240 槽、k = 10、`HIDE=rec`）原样（`out/b1-W240-k10-rec-skiponce.log`）：

```
OPUS-B4 skip_once first=16 -> 17
OPUS-TALLY name=b1_k10_rec_pb2999_m0_inner states=24623 closed_form=24598 enumerated=24623 observed=24623 oracle_violations=0 CV secs=149.5 counts=[both_rec_abandoned_read_ok=5192 both_rec_other_read_ok=19431 both_ro_abandoned_read_ok=9224 both_ro_other_read_ok=15399 lost0_rec_abandoned_failed=24614 lost0_rec_abandoned_failed_invariant_violated_invariant_i_1_1_detail_key=4104 lost0_rec_abandoned_failed_mapping_still_unreadable_slot_slot_number_1601=2048 lost0_rec_abandoned_failed_unit_unreadable_slot_slot_number_1603=2048 lost0_rec_abandoned_failed_unit_unreadable_slot_slot_number_1604=16414 lost0_rec_other_failed=9 lost0_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=9 lost0_ro_abandoned_read_ok=4104 lost0_ro_hit_sc_newer_true_table_false=12293 lost0_ro_hit_sc_newer_true_table_true=8217 lost0_ro_other_failed=20510 lost0_ro_other_failed_ro_mount_open_walk_unit_unreadable_slot_slot_number_1603=16414 lost0_ro_other_failed_ro_mount_open_walk_unit_unreadable_slot_slot_number_1604=4096 lost0_ro_other_read_ok=9 lost1_rec_other_failed=24623 lost1_rec_other_failed_invariant_violated_invariant_i_1_1_detail_key=24623 lost1_ro_other_read_ok=24623 newbad_ro_abandoned_read_ok=9224 newbad_ro_other_failed=8219 newbad_ro_other_read_ok=7180]
```

（写流多了一次发布，24623 态，与今天的 20516 态不是同一张状态表；比的是比例。）实例 2 的根全清零、两块盘都在：今天 0 / 20516 失败，收严之后 8219 / 24623 失败——P 的根槽留在环里、P 的单元已被写行盖掉，只读挂载整池打不开（失败原因我只记了「落不到根」这一层；为什么 F2 没退回 S，我推的是实例 2 写行那次轮换把 c_见证 推过了 P 的末条、P 算见证过——推的，没逐态核）。丢盘 0：今天 12296 / 20516，收严之后 20510 / 24623。B4 那段历史（P 见证过）上，收严臂原样（`out/b4.log`）：

```
OPUS-B4 k=2 hide=rec+scnew arm="skip_once" p=(InstanceGeneration(1), CheckpointTxg(6)) effective=(InstanceGeneration(1), CheckpointTxg(5)) instance2_txgs=[7, 8] p_root_slot_kept=true p_new_units_overwritten=0/20 ro_both=Some(8):other_read_ok ro_after_new_instance_roots_zeroed=Some(6):abandoned_read_ok
OPUS-B4 k=5 hide=rec+scnew arm="skip_once" p=(InstanceGeneration(1), CheckpointTxg(9)) effective=(InstanceGeneration(1), CheckpointTxg(8)) instance2_txgs=[10, 11] p_root_slot_kept=true p_new_units_overwritten=14/20 ro_both=Some(11):other_read_ok ro_after_new_instance_roots_zeroed=None:other_failed
```

k = 2、3、4、10 实例 2 的根全坏时读回已确认的 P（今天读回 S、P 丢），k = 5 整池打不开（今天读回 S）。两头取决于写行有没有盖到 P 的单元——而收严不护 P 的单元：实例 2 挂载那一刻看不见 P（B1 乙那一段，判别子观测不到）。

四句：①分辨臂——分辨（同一段历史上两臂结局相反）。②看得到——「这一槽读回全零」看得到，但环才转到 txg 16 时它与「这一槽从没写过」逐条相同，收严那一判在新池上会对每一个从没写过的槽都挪一次（这里只挪一次，所以没炸开）。③分句——「恢复落到的根引用的单元没被复用，或挂载报错」（C554「怎么拦」一格）：收严之后 8219 态两块盘都在也读不出，打中的是「没被复用」那一半。④改法——收严对「P 已确认」那几格有用（4 / 5 个起点），对「P 的单元已被盖」那几格反而造出整池打不开；要它只剩好处，得同时护住 P 的单元，那就回到 B1 乙。

## B5　回退目标引用隔离单元（C572）

**判定：这一形还缺（打中，量过）。** 冻结副本拷贝上造出：同一块盘上同一份读不对的副本，系统**先前没诊断过**时回退被拒，**已经诊断过、隔离着**时回退照做，新根引用这份坏副本；再丢另一块盘，回退出来的那一版读不出。

### 5.1 历史与每一步许可它的那一句

装置 `MODE=b5`（`research/prompts/m3-c554-c572-r1-opus-model/src/probe/src/b5.rs`），384 槽小盘，全在内存稀疏盘上：

1. 第一个文件 V1（数据单元 u1，两块盘各一份）。`persist` 形：把 u1 在盘 1 那一份翻一个字节。
2. 覆盖写成 V2：释放 u1 之前按位置项读盘核，盘 1 那一份重读一次仍对不上，两份记录都留在已分配、分配代不变（`crates/singlefs-core/src/allocator.rs:1865`「if devices_whose_record_stays_allocated.contains(&device.device) {」跳过的就是 `crates/singlefs-core/src/allocator.rs:1882`「record.is_released = true;」那一改）。许可它的条款：`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`「任一份核出对不上，这个单元在每块盘上的分配记录都留在「已分配」、不改成已释放」。
3. 再覆盖写 V3（`EXTRA_OVERWRITES=1` 时再写一版，让回退那次的根落在盘 1）。
4. 挂着时回退到 V1。复活判定逐盘对现行账：u1 在现行账里仍分配、分配代与 V1 那一版账里相同，走 `crates/singlefs-core/src/mount.rs:7743`「Some(_shared_by_both_versions) => {}」，不进 `crates/singlefs-core/src/mount.rs:7746`「if is_resurrected_on_some_device {」那一支，逐盘验（`crates/singlefs-core/src/mount.rs:8107`「every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」）不读它。许可它的条款：`.claude/kb/decisions/23-journal的角色与格式.md:388`「cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」。
5. 回退做成，新根引用 u1；丢盘 0，只读挂载盘 1。

对照两形：`control` 不先坏，V2 正常释放 u1，之后才把盘 1 那一份翻坏（同一份物理坏副本，走复活集那一支）；`transient` 释放核那一刻坏、V2 发布之后翻回去（单元完好，隔离照旧——条款许可：`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`「两次都读不出或都对不上才按对不上处置」，两次读之间不等）。

### 5.2 量到的（副本上的数，原样，`out/b5-v2.log`）

今天（`OPUS_C572_ARM` 空）：

```
OPUS-B5 form=persist arm="" extra=1 rollback_txg=7 checker_before_rollback=["I-2.1", "I-4.8", "I-7.4"] quarantined_copies_at_v2=2 u1_records_still_allocated=2 rollback=ok resurrected_copies=6 released_units=4 quarantined_copies_at_rollback=0 new_root_references_u1=true checker_violated=["I-2.1", "I-4.8", "I-7.4"] ro=[both:Some(7):reads_v1 lost0:Some(7):failed_ro_read_data_unit_checksum_mismatch_everywhere_unit_index_in_file_data_unit_index_in_file_ lost1:Some(6):reads_some_other_version]
OPUS-B5 form=control arm="" extra=1 rollback_txg=6 checker_before_rollback=["I-2.1", "I-4.8", "I-7.4"] quarantined_copies_at_v2=0 u1_records_still_allocated=0 rollback=refused resurrected_unit_copy_unreadable_or_mismatched_unit_data_data_unit_index_in_file_0_device_ new_root_references_u1=false checker_violated=["I-2.1", "I-4.8", "I-7.4"] ro=[both:Some(6):reads_some_other_version lost0:Some(4):reads_some_other_version lost1:Some(6):reads_some_other_version]
OPUS-B5 form=transient arm="" extra=1 rollback_txg=7 checker_before_rollback=["I-3.11"] quarantined_copies_at_v2=2 u1_records_still_allocated=2 rollback=ok resurrected_copies=6 released_units=4 quarantined_copies_at_rollback=0 new_root_references_u1=true checker_violated=[] ro=[both:Some(7):reads_v1 lost0:Some(7):reads_v1 lost1:Some(6):reads_some_other_version]
```

读法：`persist` 与 `control` 盘上那一份坏副本是同一个字节翻坏的；`control` 回退被拒（`ResurrectedUnitCopyUnreadableOrMismatched`），`persist` 回退做成、回退那一版（txg 7，根在盘 1）丢盘 0 之后读文件报 `DataUnitChecksumMismatchEverywhere`。回退已向管理员确认（D16 已定项 7：根持久之后才确认），之后丢一块盘就读不出——与 `.claude/kb/decisions/23-journal的角色与格式.md:445`「这个不能认，这个违背我们数据安全的承诺」那一句用户原话同一个方向。

### 5.3 三个候选在这几格上（臂在副本 `mount.rs` 里，`patches/core-arms.diff`）

| 候选 | persist（坏副本，已隔离） | transient（隔离而完好） | control（坏副本，没隔离） | 多读 | 标 |
|---|---|---|---|---|---|
| 今天（维持现行、改理由句写成边角） | 回退做成，丢盘 0 读不出 | 做成，读对 | 拒 | 0 | 量过 |
| `verify_quarantined`：隔离单元也进逐盘验 | 拒（`ResurrectedUnitCopyUnreadableOrMismatched`） | 做成，读对 | 拒 | 每个「目标引用、现行版本不引用、复活判定当共用」的单元两份；这里 1 个（`units_checked_on_every_device=4`，今天 3） | 量过 |
| `refuse_quarantined`：目标引用隔离单元直接拒 | 拒 | **拒**（错拒：两份都读得出、对得上） | 拒 | 0 | 量过 |
| `verify_all_shared`：目标引用的用户可见单元全部逐盘验 | 拒 | 做成 | 拒 | 目标引用的全部用户可见单元（这里 4 个；真池随文件大小涨） | 量过 |

原样（同一份 `out/b5-v2.log`）：

```
OPUS-B5 form=persist arm="verify_quarantined" extra=1 rollback_txg=6 checker_before_rollback=["I-2.1", "I-4.8", "I-7.4"] quarantined_copies_at_v2=2 u1_records_still_allocated=2 rollback=refused resurrected_unit_copy_unreadable_or_mismatched_unit_data_data_unit_index_in_file_0_device_ new_root_references_u1=false checker_violated=["I-2.1", "I-4.8", "I-7.4"] ro=[both:Some(6):reads_some_other_version lost0:Some(4):reads_some_other_version lost1:Some(6):reads_some_other_version]
OPUS-B5 form=transient arm="verify_quarantined" extra=1 rollback_txg=7 checker_before_rollback=["I-3.11"] quarantined_copies_at_v2=2 u1_records_still_allocated=2 rollback=ok resurrected_copies=6 released_units=4 quarantined_copies_at_rollback=0 new_root_references_u1=true checker_violated=[] ro=[both:Some(7):reads_v1 lost0:Some(7):reads_v1 lost1:Some(6):reads_some_other_version]
OPUS-B5 form=transient arm="refuse_quarantined" extra=1 rollback_txg=6 checker_before_rollback=["I-3.11"] quarantined_copies_at_v2=2 u1_records_still_allocated=2 rollback=refused resurrected_unit_copy_unreadable_or_mismatched_unit_data_data_unit_index_in_file_0_device_ new_root_references_u1=false checker_violated=["I-3.11"] ro=[both:Some(6):reads_some_other_version lost0:Some(4):reads_some_other_version lost1:Some(6):reads_some_other_version]
```

「多读」那一列的数在 `out/b5-full.log` 的 `OPUS-C572 … units_checked_on_every_device=` 行（今天 3，两个验的臂 4）；`verify_all_shared` 那两行也在这一份里（`b5-v2.log` 是没有它的同一趟早一版输出，相同的 18 行逐字相同）。

`refuse_quarantined` 那一臂借用了 `ResurrectedUnitCopyUnreadableOrMismatched` 当拒因（只为看拒不拒），真做要另起错误成员。

四句：①分不分辨臂——分辨：`persist` 一格上今天与 `refuse`/`verify` 结局相反，`transient` 一格把 `refuse` 与 `verify` 分开。②系统当时看不看得到——看得到：隔离单元 = 目标引用、现行账里同槽同分配代仍分配、现行那一版的用户可见单元不引用（臂里就是这样认的，不读盘就认得出），重挂之后也认得出（推的：隔离的记录落盘，D19 已定项 5「落盘即跨重挂」）。③满足的分句——`.claude/kb/decisions/23-journal的角色与格式.md:388` 那一格的理由句「cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」对隔离单元为真（分配器确实不发它），但它推不出「不必逐盘验」：逐盘验防的是「一块盘上那一份不可用」，隔离单元恰是已核出一份不可用的。④改法在打中的格上——`verify_quarantined` 与 `verify_all_shared` 都修 `persist` 格、不错拒 `transient` 格；`refuse_quarantined` 修 `persist` 格、在 `transient` 格上错拒；维持现行对 `persist` 格不起作用。

顺带看到（线索，不在这一格里判）：`transient` 形回退之前池级 checker 判 I-3.11 红（`checker_before_rollback=["I-3.11"]`）。对上 `crates/singlefs-checker/invariants/05-空间记账.md:18` I-3.11 那一行「每一份都读得出且对得上，照旧判违例」：瞬时两次读坏造成的隔离（条款许可的合法状态）在池级 checker 上是红的；`refuse_quarantined` 之后这一红留着，今天与 `verify_quarantined` 回退把 u1 重新引用之后转绿。

## 各格的候选、要动的文件、代价与交用户的岔路

候选都是我提的或照正文的候选建的臂，**只在我的模型上量过、被攻过零轮**。「量过」= 冻结副本拷贝上的原样输出（上面各节）；「推的」= 按代码推、没实现没跑。

| 格 | 候选（一句定义） | 要动的 `crates/` | 代价 | 修哪一格 | 标 |
|---|---|---|---|---|---|
| B1 | 甲一：单盘择根时，剩下那块盘上读得出更新实例的记录、按它的新根段读得出的实例表判所选根被抛弃，就不择它 | `crates/singlefs-core/src/recovery.rs`（择根）、`crates/singlefs-core/src/mounted_read.rs`（只读择根） | 每次择根多扫一遍更新实例的记录、读一张实例表 | 1.3 那一格的 3 / 12296 | 量过（判别子出现的态数）；改法本身推的 |
| B1 | 甲二：单盘择根时，系统配置记着的实例代号比所选根新就不择它 | 同上 | 合法对照 270378 态里 270363 态错避开、丢两版已确认 | 1.3 那一格全部 | 量过（条件成立的态数）；丢几版推的 |
| B1 | 乙：被抛弃根读不出时影子账退到更保守的集合 | `crates/singlefs-core/src/mount.rs`（`isolate_slots_referenced_only_by_abandoned_roots`） | 判别子观测不到，只能无条件保守 | 碰不到 | 推的 |
| B1 | 丙：写进射程当已知边角 | 无 | 丢一块盘之后整池只读也打不开 | 不修 | 量过（发生的态数） |
| B2 | `released` | `crates/singlefs-core/src/mount.rs`（`placements_in_the_accounts_of_unapplied_publishes` 的 `None => continue`） | 240 槽盘上 k = 3 早 3 次、k = 2 第 6 次就报空间不够（今天 8 次全成） | 同进程那一格没打中，分不出；跨进程那一格不修 | 量过 |
| B2 | `allocated`（丙搬过来） | 同上 | k = 3 早 1 次、k = 2 第 8 次报 | 同上 | 量过 |
| B2 | `allocated_released` | 同上 | k = 3 早 3 次、k = 2 第 5 次报 | 同上 | 量过 |
| B2 | `mapping`（记录齐也走映射现算、重读、凭据、拒） | 同上 | 映射读得出时 0 次；映射也读不出时拒可写（4 / 4 起点） | 同上 | 量过 |
| B3 | 五处判重与豁免改成按槽展开 | `crates/singlefs-core/src/mount.rs`（同一个函数） | 每处多隔离被漏的那几槽（三形各 1 槽） | 三形 | 量过（漏）；改后推的 |
| B4 | 射程② 改句：见证过的在见证槽读回全零时同样会被抛弃（与 F2 代价那一条对齐） | 无 | 无 | 文字 | 量过（反例） |
| B4 | 收严 `skip_once`：新实例第一次发布避开这一刻读坏或全零的根槽对应的 txg | `crates/singlefs-core/src/mount.rs`（`first_txg_of_new_instance`） | 两块盘都在、新实例的根全坏时 0 → 8219 / 24623 态打不开 | P 已确认且单元没被盖的 4 / 5 个起点找回 P | 量过 |
| B5 | 隔离单元也进逐盘验 | `crates/singlefs-core/src/mount.rs`（`roll_back_by_a_forward_publish` 里逐盘验那一份） | 多读「目标引用、现行版本不引用、复活判定当共用」的单元两份（这里 1 个） | persist 格；transient 格不错拒 | 量过 |
| B5 | 目标引用隔离单元直接拒 | 同上，另起错误成员 | 0 读；transient 格错拒、池级 checker I-3.11 红留着 | persist 格 | 量过 |
| B5 | 维持现行、改理由句写成已知边角 | 无 | 回退确认之后丢一块盘读不出 | 不修 | 量过 |

交用户的岔路（候选、翻面观测、够判条件；不带倾向）：

| 格 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| B1 | 甲一 / 甲二 / 乙 / 丙 | 打中那一形在真池几何上的频率（写行落点取到被抛弃根的槽、且所选根与它前一版的根分在两块盘的比例）；甲二在合法单盘恢复上错避开的比例 | 两个比例在入库装置上各量一次（同一组起点、全部崩溃状态） |
| B2 | 今天 / released / allocated / allocated_released / mapping | 同一进程里那一形在入库装置上出不出、各臂修不修；窄盘上的空间代价 | 复现 batch10z-no 那一段同一进程的丢写，五臂各跑一遍 |
| B3 | 按槽展开 / 维持 | 无（条款按槽，偏离是实现的） | 已够 |
| B4 | 改句 / 收严 / 两样都不做 | 收严在「P 已确认且单元没被盖」与「P 的单元已被盖」两类状态上的比例 | 入库装置上同一组起点各量一次 |
| B5 | 逐盘验 / 直接拒 / 维持 | 瞬时两次读坏造成的隔离在真机上出不出（出就是直接拒的错拒面） | 已有本报告三形的反例；要不要量瞬时隔离的频率由用户定 |

## 没打中的形状（各一次抽样，不支撑「没有」）

- B1，P 的根槽与单元读回全零、记录读得出（批 17 那一形）：384 槽 k = 2 枚举 20511 态、240 槽 k = 10 枚举 270378 态，单盘与两盘只读挂载 0 失败；384 槽 k = 2..14 快扫 13 个起点写行与暖机一个都没盖到 P 的槽。`acct`（P 的账那一片）、`new`（P 新分配的全部单元）两种藏法在 384 槽 k = 2 各 20511 态 0 失败。
- B1，`HIDE=rec`：384 槽 k = 2、3、4、5、6、8、11 各一段枚举（20516 或 24623 态），只读挂载 0 失败——盖到 P 的那几个起点 P 与 S 的根同盘，单盘时 F2 退得回 S（`out/b1-rec-k5-8-11.log` k = 5 那一段 `lost1_ro_other_read_ok=18460`）。
- B2 同一进程里的丢写：384 槽 k = 2..5、j = 3；384 槽 k = 2、3、j = 8、14；240 槽 k = 2、j = 8 今天那一臂，R 的落点一个都没被写掉。240 槽其余组合在第 3–13 次覆盖写就报空间不够，没走到卸载。
- B4，今天的代码、两块盘都在、新实例的根全坏：B1 那段 20516 态里 0 失败。
- B5 `verify_quarantined` 在 `transient` 与 `control` 两形上没有错拒、没有错放。

## 这条腿自己的限度

- 只抽了一次样；「没打中」各条照规则不算数。
- 单盘结论只用只读挂载那一路判：`recover()` 在缺一块盘时多数态报 `InvariantViolated`（各段的 `*_rec_*_invariant_violated`），我没查为什么，也没拿它当结论。
- 崩溃只枚举了实例 2 那次挂载的写流；P 自己那次发布（1048588 态）与会话里的覆盖写都超过 10⁶，没枚举，各段只取「P 的根槽 FUA 落了、轮换没落」那一刀。B2、B5 两格的历史因此不带崩溃（B5 的回退那次发布也没枚举）。
- 故障只造了「读回全零、报成功」一种（不是设备报错），一次挂载内一直在、交回之后撤。我的故障层按固定字节区间清零，进程自己后来写进那一区间的也照样清零：`out/b2-pressure.log` 里 384 槽 j = 20 那几行卸载被拒（`root_ring_slot_known_to_hold_a_root`）是这一层造出来的，不是发现。
- B4 的收严只建了最窄的一种读法（挪一个 txg、只挪一次），别的读法（挪到下一个读得出的槽、只认读坏不认全零）没建。
- B3 的模型调的是被测函数「根环表里记着」那一支（抬 F 那一串），可写挂载那一支要读盘，没在模型里走；两支共用同一段判重。
- 判定库只认 checker 档包 `tests/` 下的设点文件，探针借了一份现成的名字；判定库在草稿目录、每趟清空，结论不与别处共用，但节点身份与 CPU 结论版本取的是那份文件的。
- 模型源码与输出拷进模型目录时有两次工具调用一次写了 419 行与 158 行（逐份 `cat` 拷贝），超过「每次不超过 150 行」那一条；内容与草稿目录里编、跑的那一份逐字节相同（探针六份、`b3_unit_test.rs`、`core-arms.diff` 交回前逐份 `cmp` 过）。

## 没做什么

- 没在入库装置上跑任何东西；全部数在冻结副本的拷贝上量，不是入库装置上的数。
- 没跑 `tools/rerun.sh` 全程复跑（编译 + 跑超过前台 4 分钟；交主 agent，见下一节）；报告里每个数都来自草稿目录里那一趟跑的输出，模型目录 `out/` 是那一趟输出的原样摘行。
- 没判正推那一边的格，没替主 agent 采纳；B2 同一进程那一形我没复现，那一格的候选我分不出谁修谁不修。
- 没跑重型测试、门禁阶段、崩溃点重放与变异的轻型条目（没改仓内 `crates/`）。

## 要主 agent 跑的命令

- 复跑（工作目录随意；`--work` 给新的空目录；内存上限 8G、线程 16；跑完看末尾 `RERUN-CHECK … ok=` 是否全是 `true`；`RERUN_LONG=1` 时另跑两段 270378 态与 k 扫描，脚本不自动比，要与 `out/` 里同名段的 `OPUS-TALLY` 行人工比）：

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-c554-c572-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-c554-c572-r1/repo --work <新的草稿目录>
```

## 草稿目录

`~/.local/share/singlefs/work/m3-c554-c572-r1-opus/`：`out/`（各趟整份日志，报告里摘行之外的原文）、`logs/`（编译日志）、`report-parts/`、`src/`。仓副本与编译目录（`repo/`、`target-core/`、`crashlib/`）交回前删掉。
