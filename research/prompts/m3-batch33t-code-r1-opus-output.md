# m3-batch33t-code-r1 攻方（Opus）报告：K2 四格（2026-10-08）

立场：攻方，在冻结副本的拷贝上造反例。只跑 harness 档（`cargo test -p singlefs-harness --test <自己的目标>` 与几条已有 harness 目标），另编了一个只调 checker 档两个库函数（层 0 的 oracle、记录核对器）的小原型，没跑 checker 档任何测试。所有数都是攻方副本上量的，不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-batch33t-code-r1-opus-model/`。在一个空目录里从头跑（各阶段前台都在 4 分钟内，全跑串行约 12 分钟；本轮在 `~/.local/share/singlefs/work/m3-batch33t-code-r1-opus/rerun3/` 从空目录跑通一次，`compare.sh` 报「全同（1280 行）」）：

```
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-batch33t-code-r1/tree SINGLEFS_TOOLS=<仓根> THREADS=10 \
  bash research/prompts/m3-batch33t-code-r1-opus-model/rerun.sh <空目录>            # 或逐段：rerun.sh <目录> prep build t5 …
bash research/prompts/m3-batch33t-code-r1-opus-model/compare.sh <空目录>/summary.txt
```

`SINGLEFS_REPO` 是被判的树根（只读，拷进 `<目录>/repo` 再加臂开关），`SINGLEFS_TOOLS` 只取 `scripts/run-with-memory-cap.sh` 与 `scripts/capped.sh`。编 6G、跑 4G，同一时刻只起一条 cargo 或一个测试进程。臂开关（`patch_arms.py`，环境变量不设时与冻结副本行为相同）：`OPUS_F2_OFF`（F2 不退，即 F2 落地前）、`OPUS_F2_EXACT`（攻方收严）、`OPUS_F1_OFF`（F1 不拒）、`OPUS_F1_ANY`（攻方收严）、`OPUS_C_ACCOUNT`（攻方收严）。

| 文件 | sha256 |
|---|---|
| `rerun.sh` | 7ff59febcd80ad2c327abcfe36c782533e0472eb3fe6ed666a6069e3d76d3fee |
| `compare.sh` | e496f34eefc810c51af70d0588e8cc96a0d5cf4041e7213fa61a852dd078941c |
| `patch_arms.py` | 8f4ebcb993ca2859093322acecddcf3c4d0c98c4903861d972d437487f0c2fae |
| `probe/opus_b33t_probe.rs` | 6b37619e45d36af0bcf5ae877a806326acb24393da93a2d9a96175d6e0cfb655 |
| `probe/opus_b33t_sweep.rs` | 4a7be6a36c1191102b459ce523b6a68c33edf78d8d36a730a9ecbab5e3fb0ae7 |
| `oracle-probe/Cargo.toml` | d482e4879a13c6927146493c0ecdbb2f8b4685eca681b51cb798e3993e67fae0 |
| `oracle-probe/src/main.rs` | cc450d36f9ad15c42bef89c999794c03fac3bb03c35b1e6c551dc39733d9708b |
| `expected/summary.txt` | 3a5464f7e1c6834ddfba0739230bfec8314d68d1ca7c1e3dda003f6fb955e888 |

下文贴的判定行都是 `expected/summary.txt` 里的原行（行首是日志名 = 臂）。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| T5 F2 读回旧版（超出已接受那一形） | **打中**（两形） | A：较新的 R 走读失败、已确认的 V 根槽读回全 0 ⇒ 三处都悄悄落到 V 之前那一版（今天报读不出），可写挂载还把 V 抛弃；B：见证计数器那条记录读不出 ⇒ 上限退成所选根，落到被新实例的实例表抛弃了的 R | 一 |
| T5 退回之后 oracle / 记录核对器 | **打中**（库函数上量的，checker 档登记流里推的走不到） | 层 0 oracle 把 F2 设计里的那一退（R 走不通、退回见证到的 V）判成 `RecoveredToAnOlderRoot`，与 A 形同一判；记录核对器两形都不报 | 一.3 |
| T4 F1 误拒 | **打中**（带一处单故障） | 纯正常崩溃（写记录那一段 64 个子集）0 拒；写记录中途断电 + 那次发布一个单元的**一份**副本坏（单故障）⇒ 每次可写挂载都拒、盘一字节不动、没有出口，只读与恢复照常 | 二 |
| T1 C 放错（该钉的放了） | **打中**（盲区比正文写的宽） | 根槽加任一条记录（末条、中间一条、第一条）读回全 0、单元读得出：F1 不拒，C 看不见那条记录点名的单元；M1 之后第一次写数据（o 或 s）就写掉 R 的 2 份单元、池级 checker 红 I-7.4，深度 3 每种藏法 58 个结点里 48 个红 | 三 |
| T1 假 ENOSPC 无界 | 没打中 | 隔离只活到这个进程写掉那一槽，量到的每盘隔离 6–17 槽 | 三.3 |
| T1 两种次序分叉 | 没打中（推的） | 新实例的第一个 txg = 环里与记录里最大的 txg + 1；要造出「新实例 txg 低于旧实例记录」得在同一次挂载里藏掉连续两次发布的根与全部记录、再藏系统配置的较新槽 | 三.4 |
| T7 C692 新下限误红 | 没打中 | 34 段随机历史（384/256/240 槽、两档窄盘、4 GiB 两种环）每步恢复 0 次红，每盘记录数比下限多出的最小值 8 | 四 |

攻方提的三条收严（`OPUS_F2_EXACT`、`OPUS_F1_ANY`、`OPUS_C_ACCOUNT`）**只在我的模型上量过、被攻过零轮**，见各节末的改法表。

## 一、T5：F2 在用户已接受那一形之外读回旧版

### 1. A 形：退过已确认的那一版

**历史**（用例 `opus_t5_fallback_goes_past_the_witnessed_version`）：384 槽小盘，第一个文件之后在会话里覆盖写 k 次（k 扫 1、2、4、6），最后一次覆盖写 V 整次落盘（系统配置轮换也落了，V 已确认）；再录一次发布 R（覆盖写 `ow` 或三个数据单元的顺序写 `sw3`），崩在 R 的根槽落盘之后、它的系统配置轮换之前（系统配置没见证 R）。读的那一刻两处读回全 0：R 写出的单元（两盘两份）、V 的根槽（`Ru+Vr`）。之后没有用户动作：三处都在第一次读时就落定。

**每一步许可它的那一句**：
- R 走读失败、系统配置没见证 R ⇒ 进退回判法（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5317` 那个函数）；V 的末条记录读得出、计数器 = c_见证，上限取它：「let ceiling = record_at_the_witnessed_counter.unwrap_or(chosen);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5389`）。
- 候选是根环里**读得出**、不比上限新的根取最大：「order < chosen && order <= ceiling」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5399`）与「.max_by_key(order_of_the_root_choice)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5401`）。V 的根槽读回全 0 ⇒ V 不在候选里 ⇒ 取到 Vminus。没有一句要求退到的就是见证到的那一版本身。
- 退过去不施加记录：「退过去不重放：所选根与所选那一版都换成它，」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:6338`）。V 的记录与单元在盘上都完好，同一族藏法 `Ru+Rr+Vr`（再把 R 的根槽也藏掉）三处都由记录重建出 V，证明 V 在这一刻读得出。
- 可写挂载调同一份判法（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:6352`），落在 Vminus 上取号写行：V 从此是被抛弃的一版（推的：V 的根槽这一次读回全 0，影子账看不见它是根，只剩 C 按 V 的记录钉它的点名单元，到这个进程写掉 V 那一槽为止）。

**量到的**（`expected/summary.txt` 原行，k=1、ow；四个 k × 两种 R 共 8 组，`Ru+Vr` 在 base 臂 8/8 组都是这一行的形）：

```
t5-base OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru+Vr recover=FileRead(root=Vminus content=Vminus) ro=Ok(chosen=Vminus effective=Vminus) rw=Ok(chosen=Vminus effective=Vminus inst=2)
t5-base OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru+Rr+Vr recover=FileRead(root=Vminus content=V) ro=Ok(chosen=Vminus effective=V) rw=Ok(chosen=Vminus effective=V inst=2)
t5-base OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru recover=FileRead(root=V content=V) ro=Ok(chosen=V effective=V) rw=Ok(chosen=V effective=V inst=2)
t5-F2_OFF OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru+Vr recover=Failed(root=R UnitUnreadable) ro=Err(Open) rw=Err:Recovery
t5-F2_EXACT OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru+Vr recover=Failed(root=R UnitUnreadable) ro=Err(Open) rw=Err:Recovery
t5-F2_EXACT OPUS T5A ow k=1 R=(1,5) V=(1,4) Vminus=(1,3) shape=Ru recover=FileRead(root=V content=V) ro=Ok(chosen=V effective=V) rw=Ok(chosen=V effective=V inst=2)
```

用户已接受的那一形是「已确认的版本在较新系统配置槽与它的单元同时读回全 0 时读回上一版（S1）」（`research/prompts/m3-c694-r2-main-verification.md:46`）。A 形里系统配置两槽都好、V 的单元都好，坏的是 R 的单元与 V 的根槽：不是那一形。今天（`F2_OFF`）同一盘面报读不出；F2 把它变成悄悄读回 V 之前那一版，可写挂载还把已确认的 V 抛弃。

### 2. B 形：退到被新实例抛弃了的根

**历史**（用例 `opus_t5_fallback_lands_on_a_root_the_instance_table_abandoned`）：同上起点（R = 覆盖写，k 扫 1、2、4）。取新实例那次挂载 M1 时 R 的根槽与 R 的单元整次挂载读回全 0（第一轮 K1-2 的形，登记的故障模型 `crates/singlefs-harness/src/history.rs` 那一种），M1 落在 V、取号 2；M1 崩在写行那次发布 Row 的根槽落盘之后、它的轮换之前。下一次读时 Row 写出的单元读回全 0（Row 走读失败），V 的末条记录（见证计数器那一条）读回全 0（`RowU+VlastRec`）。

**每一步许可它的那一句**：Row 的末条记录读得出、计数器大于 c_见证 ⇒ 没见证；c_见证 那条记录读不出 ⇒ 上限退成所选根本身（同上 `/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5389` 的 `unwrap_or(chosen)`）；候选里比 Row 旧的最新一条读得出的根是 R。判法不看实例表：Row 的实例表那一行是 (1, 4, …)，按「表里有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:1538` 那一段注释）R (1,5) 是被抛弃的（实例表这一行的内容是推的，按 M1 交回的 `m1_effective=(1,4)` 与写行规则；Row 的实例表这一刻本身也读不出）。

```
t5-base OPUS T5B k=1 R=(1,5) V=(1,4) Row=(2,6) m1_effective=(1,4) r_units_overwritten_in_state=0 ring=["(0,0)", "(1,1)", "(1,2)", "(1,3)", "(1,4)", "(1,5)", "(2,6)"]
t5-base OPUS T5B k=1 shape=RowU+VlastRec recover=FileRead(root=R content=R) ro=Ok(chosen=R effective=R) rw=Refused:InstanceTableOfTheNewestRootForTheShadowLedger
t5-F2_OFF OPUS T5B k=1 shape=RowU+VlastRec recover=Failed(root=Row UnitUnreadable) ro=Err(Open) rw=Err:Recovery
t5-F2_EXACT OPUS T5B k=1 shape=RowU+VlastRec recover=Failed(root=Row UnitUnreadable) ro=Err(Open) rw=Err:Recovery
```

三个 k 都是这一形。恢复与只读挂载读出的是 M1 已经抛弃、内容从没被确认的 R；今天报读不出。

### 3. 退回之后层 0 的 oracle 与记录核对器怎么判

原型 `oracle-probe` 直接调 `singlefs_checker_tier::crash` 的 `classified_oracle_violation_for_versions` 与 `check_records` 两个库函数（不跑 checker 档测试）。崩溃状态取 R 的写表里「根槽及之前全落、之后没落」那一个；「R 的单元读回全 0」在这里写成那几次单元写不在持久集合里，「V 的根槽读回全 0」写成基线上那一槽写零。

```
oracle OPUS ORACLE ow k=1 shape=Ru R=(1,5) V=(1,4) effective=Some((1, 4)) newest_persisted=Some((1, 5)) oracle=RecoveredToAnOlderRoot root_without_record=false claimed_state_missing_unit=false
oracle OPUS ORACLE ow k=1 shape=Ru+Vr R=(1,5) V=(1,4) effective=Some((1, 3)) newest_persisted=Some((1, 5)) oracle=RecoveredToAnOlderRoot root_without_record=false claimed_state_missing_unit=false
oracle OPUS ORACLE ow k=1 shape=none R=(1,5) V=(1,4) effective=Some((1, 5)) newest_persisted=Some((1, 5)) oracle=none root_without_record=false claimed_state_missing_unit=false
```

- oracle 只认「盘上已持久的最新根槽之下一律不许落」：「if (effective_txg, effective_instance) < (newest_txg, newest_instance) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:1111`）。F2 设计里那一退（`Ru` ⇒ V）与 A 形那一退（⇒ Vminus）同判 `RecoveredToAnOlderRoot`：oracle 分不开「合法退回见证到的那一版」与「退过已确认的那一版」。哪条 checker 档的流一旦让 F2 退了（R 的根槽写在流里且落了），这一判就红，不管退得对不对（推的：今天登记的随机历史不藏记录，C 罩住了 K1-2，F2 在登记流上大概走不到；没跑 checker 档去数）。
- 记录核对器第二条只看 txg 不超过恢复自称那一版的发布：「if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-checker-tier/src/crash.rs:149`）。退到 Vminus 时 V 那次发布不在判的范围里，A 形漏判；两形量到的都是 `claimed_state_missing_unit=false`。

### 4. 打中之后的四句（T5）

| 问 | A 形 | B 形 |
|---|---|---|
| 分不分辨臂 | 分：今天（`F2_OFF`）报读不出、F2 落 Vminus、`F2_EXACT` 报读不出；设计里那一退（`Ru`）三臂里 F2 与 `F2_EXACT` 都落 V | 分：今天与 `F2_EXACT` 报读不出，F2 落 R |
| 系统当时看不看得到判别它的东西 | 看得到：c_见证 那条记录读得出，说出见证到的是 (1,V)；环里没有一条读得出的根是 (1,V) ⇒ 知道「见证到的那一版本身读不出」 | 部分看得到：c_见证 那条记录读不出、Row 的实例表也读不出；但系统配置槽里取号写的 (实例 2, tail = c_见证) 与 Row 记录的反向链都指向 V 而不是 R（推的，没在原型里读这两样） |
| 满足判据字面哪一句 | 正文 K2 T5「F2 让一个本该报读不出的盘面读回旧版的形，超出用户已接受的那一形……之外的」（`research/prompts/_m3-batch33t-code-r1-body.md:48`）：今天报读不出，F2 读回比已确认的 V 还旧的一版；盘面与 S1 不同（系统配置与 V 的单元都好） | 同一句：今天报读不出，F2 读回一版被新实例抛弃的根 |
| 跑前条款给的改法还中不中 | 正文第五节只写「打中：主 agent 现查坐实，改法进下一批；改法算零轮。」（`research/prompts/_m3-batch33t-code-r1-body.md:64`），没有点名的改法；退回 F2 甲（不做）不中；攻方的 `F2_EXACT` 不中 | 同左 |

两形都是两处读回全 0 叠在一起（A：R 的单元 + V 的根槽；B：M1 那一形 + Row 的单元 + V 的末条记录），在 D22（单元原子性怎么合成） 已定项 8 的单故障承诺之外（`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 178 行那一段射程），与用户接受的 S1 同属双故障一族；判它算不算要防的，归主 agent。

### 5. 改法（T5）

| 改法 | A 形 | B 形 | 设计里那一退（`Ru` ⇒ V） | 已有 F2 用例（4 条） |
|---|---|---|---|---|
| `F2_EXACT`：只退到 c_见证 那条记录所属那一版本身（环里 (txg, 实例) 恰好相等、读得出的那条根）；那条记录读不出就不退 | 量过：报读不出（`t5-F2_EXACT` 行） | 量过：报读不出 | 量过：仍落 V | 量过：`exist-OPUS_F2_EXACT-recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed test result: ok. 4 passed` |
| 退到候选之后照所选根同一套前缀施加到 c_见证 为止（A 形能重建出 V） | 推的 | 推的：不罩，B 形没有上限可施加 | 推的 | 没量 |
| 判法里先按所选根的实例表滤掉被抛弃的根 | 推的：不罩 A | 推的：B 形 Row 的实例表这一刻读不出，滤不掉 | 推的 | 没量 |
| oracle 加一条：R 没见证、R 的单元不在持久集合里时，落在见证到的那一版不算 `RecoveredToAnOlderRoot` | 推的：A 形照红（落在见证到的那一版之下） | 推的 | 推的：不再红 | 没量 |

已有 F2 用例在 `F2_EXACT` 臂上的原行：

```
exist-OPUS_F2_EXACT-recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`F2_EXACT` 只在我的模型上量过、被攻过零轮。

## 二、T4：F1 误拒

### 1. 纯正常崩溃：没打中

用例 `opus_t4_crash_inside_the_record_segment`：R = 三个数据单元的顺序写（三条记录），k = 1、4。崩在写记录那一段里：R 的单元与其后的屏障都落了，记录那一段 6 次写（3 条 × 2 盘）取全部 64 个子集，根槽没落。每个状态可写挂载一次。

```
t4-base OPUS T4 k=1 R=(1,5) record_writes=6 arm=none states=64 {"Ok(effective=(1,4))": 37, "Ok(effective=(1,5))": 27}
t4-base OPUS T4 k=1 R=(1,5) record_writes=6 arm=u states=64 {"Ok(effective=(1,4))": 28, "Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify": 36}
```

不藏（`arm=none`）0 拒：单元在记录之前整段落盘（「fn persist_publish_writes<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/transaction.rs:1241`）那个函数先写全部单元、过屏障、再写全部记录），读得出的记录点名的单元都验得过。R 的单元两份整次挂载读回全 0（`arm=u`）时，有记录、但不齐的 36 个状态全拒——这 36 个状态 R 的根一个都没落过。

### 2. 带一处单故障：每次都拒、没有出口

用例 `opus_t4_permanent_unit_loss_under_a_publish_whose_root_never_landed`：同上，写记录那一段只落了第 1 条记录（两份），R 的根槽从没落；然后**盘 0 上 R 的第一次单元写那一份**在盘上持久读回全 0（`copies=one-unit`，只坏一个单元的一份，盘 1 那一份完好）。连做三次可写挂载，每次之后逐字节比盘面；再做一次恢复、一次只读挂载。

```
t4l-base-one-unit OPUS T4L copies=one-unit zeroed_ranges=1 k=1 R=(1,5) rw_three_times=["Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify/disk_unchanged=true", "Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify/disk_unchanged=true", "Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify/disk_unchanged=true"] recover=FileRead(root=(1,4) content=V) ro=Ok(effective=(1,4))
t4l-F1_OFF-one-unit OPUS T4L copies=one-unit zeroed_ranges=1 k=1 R=(1,5) rw_three_times=["Ok(effective=(1,4))/disk_unchanged=false", "Ok(effective=(1,4))/disk_unchanged=false", "Ok(effective=(1,4))/disk_unchanged=false"] recover=FileRead(root=(1,4) content=V) ro=Ok(effective=(1,4))
t4l-F1_ANY-one-unit OPUS T4L copies=one-unit zeroed_ranges=1 k=1 R=(1,5) rw_three_times=["Ok(effective=(1,4))/disk_unchanged=false", "Ok(effective=(1,4))/disk_unchanged=false", "Ok(effective=(1,4))/disk_unchanged=false"] recover=FileRead(root=(1,4) content=V) ro=Ok(effective=(1,4))
```

k = 4 同形；盘 0 上 R 的全部单元那一份都坏（`copies=one`）同形；两盘两份都坏（`copies=both`）在 base 与 `F1_ANY` 两臂都拒、`F1_OFF` 不拒。

**每一步许可它的那一句**：
- 读得出的那几条序号不从 1 连到 N 或没有末条、而点名单元有一个验不过就交回：「!(ordinals_run_from_one_without_a_gap && carries_a_last_record)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:6014`）、「!crate::recovery::every_named_unit_of_the_record_verifies(reader, record)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:6012`）。
- 「验得过」要每一份都验得过：「named.locations.iter().all(|location| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:3534`）。一份坏就算验不过。
- 拒在取号之前，盘一字节不动（上面三行 `disk_unchanged=true`）；盘面不变，下一次挂载读到的还是同一份，再拒。我在 `crates/singlefs-core` 里没找到绕开 `read_stage_settled_at_most_on_the_one_reread` 的可写入口（推的，没穷举管理员一侧的路径）。

### 3. 打中之后的四句（T4）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：F1 乙（实现）每次拒；F1 甲（`F1_OFF`）挂上、落 V；攻方的 `F1_ANY` 单份坏挂上、两份坏照拒 |
| 系统当时看不看得到判别它的东西 | 看得到「另一份验得过」（盘 1 那份）；看不到 R 的根到底落没落（年轻的环里 R 那一槽从没写过、读回全 0，与 K1-2 里藏掉的根槽同一个样子；环转过一圈之后那一槽装着一条更旧的自证过的根，看得到「R 的根没落」，推的、没量） |
| 满足判据字面哪一句 | 正文 K2 T4「正常崩溃（写记录中途断电）……被 F1 拒可写的一形」（`research/prompts/_m3-batch33t-code-r1-body.md:47`）：纯正常崩溃不中；正常崩溃 + 一个单元的一份副本坏（单故障；D22（单元原子性怎么合成） 已定项 8 射程写的容错承诺是单故障，`.claude/kb/decisions/22-单元原子性怎么合成.md` 第 178 行）中，而且拒是永久的 |
| 跑前条款给的改法还中不中 | 正文没有点名改法；F1 甲不中（但也就不管 C 的盲区，见第三节）；`F1_ANY` 对单份坏不中、对两份都坏（双故障）照中 |

### 4. 改法（T4）

| 改法 | 单份坏（`one-unit` / `one`） | 两份都坏 | 纯正常崩溃 64 态 | K1-2 藏单元（第一轮那一形，`urL`） | 已有 F1 用例（4 条） |
|---|---|---|---|---|---|
| `F1_ANY`：「点名单元验不过」改成「某个点名单元两份都验不过」 | 量过：挂上（`t4l-F1_ANY-one-unit` 行） | 量过：照拒 | 量过：0 拒（`t4-F1_ANY` 与 base 同） | 量过：照拒（`t1-F1_ANY OPUS T1D k=1 hide=urL M1=Refused:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify no-row-root`） | 量过：`exist-OPUS_F1_ANY-writable_mount_refuses_a_publish_above_the_water_mark_missing_records_whose_named_units_read_back_zero test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| 再加：R 那一槽装着一条自证过、txg 比 R 小的根 ⇒ R 的根没落过 ⇒ 不拒 | 推的：环转过一圈之后两份都坏也不拒 | 推的 | 推的 | 推的：K1-2 那一槽读回全 0，照拒 | 没量 |

`F1_ANY` 只在我的模型上量过、被攻过零轮。

## 三、T1：C

### 1. 盲区：根槽加任一条记录读回全 0、单元读得出

**历史**（用例 `opus_t1_blind_spot_last_record_hidden`、`opus_t1_blind_spot_after_the_mount_with_user_actions`）：R = 三个数据单元的顺序写（三条记录，末条另点名全部元数据单元），k = 1、4，崩在 R 的根槽之后、轮换之前。取新实例那次挂载 M1 整次挂载读回全 0 的是 R 的根槽加一条记录：末条（`rL`）、中间一条（`rM`）、第一条（`rF`）；对照全部记录（`rJ`）。R 的单元读得出。M1 做成之后用户动作放开扫：字母表 o（覆盖写）、s（顺序写两个数据单元）、n（建 inode）、u（删第一个文件），长度 0..3 的每一串都走（每个结点是那一串整次落盘之后的盘面）。

**每一步许可它的那一句**：
- F1 不拒：读得出的两条序号不齐，但它们点名的单元两份都验得过（第二节引的 `/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:6012` 那一判）。
- C 的集合只有读得出的记录交的东西：点名单元（「let named_units = record.named.iter().flat_map(|named| {」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:1689`））加新根段四条指针（「let new_root_segment_pointers = [」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:1672`））。藏掉的那条记录单独点名的单元（`rM` 是第 2 个数据单元，`rF` 第 1 个，`rL` 是第 3 个数据单元与末条另点名的那几个元数据单元，按 R 的形推的、没逐个列）不在里面；R 的根槽读回全 0，影子账也不知道 R 是一条根。
- M1 之后第一次写数据就把那几个空槽发出去。

```
t1-base OPUS T1D k=1 hide=rM records=3 M1=Ok(effective=(1,4) isolated=[15, 15]) r_unit_copies_overwritten=0 chk={} recover=FileRead(root=R content=R) ro=Ok(chosen=R effective=R) rw=Ok(chosen=R effective=R inst=3)
t1-base OPUS T1G k=1 hide=rM seq=- r_in_ring=true r_unit_copies_overwritten=0 chk={} probe=FileRead(root=(1,5) content=R)
t1-base OPUS T1G k=1 hide=rM seq=o r_in_ring=true r_unit_copies_overwritten=2 chk={"I-7.4"} probe=Failed(root=Some("(1,5)") MappingStillUnreadable)
t1deep-base-k1 OPUS T1GTALLY k=1 hide=rF depth=3 nodes=58 checker_red=48
t1deep-base-k1 OPUS T1GTALLY k=1 hide=rJ depth=3 nodes=58 checker_red=0
t1deep-base-k1 OPUS T1GTALLY k=1 hide=rL depth=3 nodes=58 checker_red=48
t1deep-base-k1 OPUS T1GTALLY k=1 hide=rM depth=3 nodes=58 checker_red=48
t1-C_ACCOUNT OPUS T1GTALLY k=1 hide=rM depth=2 nodes=18 checker_red=0
t1-C_ACCOUNT OPUS T1G k=1 hide=rM seq=o r_in_ring=true r_unit_copies_overwritten=0 chk={} probe=FileRead(root=(1,5) content=R)
```

k = 4 逐格同数（`t1deep-base-k4` 四行同上）。深度 3 每种藏法 58 个结点里绿的 10 个恰好是只含 n、u 的串（`-`、n、nn、nnn、nnu、nu、nun、u、un、unn）：只要有一次写数据（o 或 s），R 的单元就被写掉 2 份、池级 checker 红 I-7.4。`probe` 是把实例代号高于 R 的根槽全写零之后再恢复（与设计轮攻方同一个探针），它是额外的多故障，只用来说 R 已经读不出（F2 此时没退：推的原因是 M1 之后的发布把 c_见证 推过了 R 的末条，R 被算成见证过，没在原型里打出是哪一支）。`rJ` 不红是因为 R 的记录也藏了，M1 的第一个 txg 就等于 R 的 txg、写行把 R 那一槽盖掉，R 离开了环。

**四句**：分辨臂——分：实现的 C 红、攻方 `C_ACCOUNT` 0 红；系统当时看得到判别它的东西——看得到：读得出的那几条记录的新根段说得出 R 的分配记录树根在哪，R 的账（单元都好）读得出，账里是 R 引用的全部落点；判据字面——正文 K2 T1「C 剩下的一形盲区：根槽与末条记录读回全零、单元读得出（F1 不拒），末条单独点名的单元 C 看不见」（`research/prompts/_m3-batch33t-code-r1-body.md:46`），量到的比它宽：中间一条、第一条读回全 0 同样中，F1 同样不拒；跑前改法——正文没点名；用户定的 F1 乙要「点名的单元又验不过」才拒，这几形单元都验得过，碰不到。

这几形都要藏记录，在登记的注入模型之外（设计轮辩方的说法），与设计轮第二节「R 的一条记录读不出」同族；设计轮那几形藏了单元，F1 罩得到，这几形不藏单元，F1 罩不到。

### 2. 改法（T1 盲区）

| 改法 | rL / rM / rF 深度 2 | M1 隔离槽数 | 已有 C 用例（4 条） |
|---|---|---|---|
| `C_ACCOUNT`：水位之上每次没施加的发布，用它任一条读得出的记录的新根段拼一条根，按影子账同一份 `placements_referenced_by_root` 读它的账，账里的落点并进 C 的集合（读不出就不加，那时单元多半也验不过，交 F1） | 量过：三种藏法 0 红（`t1-C_ACCOUNT` 行） | 量过：每盘 17（实现是 6 / 15） | 量过：`exist-OPUS_C_ACCOUNT-writable_mount_isolates_the_units_named_by_unapplied_journal_records_until_their_root_ring_slot_is_overwritten test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| F1 不看单元：序号不齐或缺末条就拒 | 推的：拒 M1，不红；正常崩溃在写记录那一段（第二节 64 态里 36 个缺记录的）全拒 | — | 没量 |

`C_ACCOUNT` 只在我的模型上量过、被攻过零轮。

### 3. 钉住不该钉的、假 ENOSPC 无界：没打中

隔离按记录的 txg 归到它在根环里的那一槽（「.entry(target_for_publish(record.checkpoint_txg, slots_per_region))」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:1715`）），这个进程写掉那一槽时解开（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/allocator.rs:2064` 那一处 `remove`）。新实例从「环里与记录里最大的 txg + 1」起发，任何一槽在一圈（24 次发布）之内都会被写到；卸载之后下一次挂载按新水位重算，旧实例的记录按实例为主比都在水位之下。量到的 M1 隔离：每盘 6（`rL`）、15（`rM`、`rF`）、17（`ur`）。没造出一形让它活过这一次挂载。

### 4. (实例, txg) 与 (txg, 实例) 两种次序分叉：没打中（推的）

C 与 F1 的水位按实例为主比（「(record.instance, record.checkpoint_txg)」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:1669`）），F2 按 txg 为主比（「let order_of_the_root_choice = |root: &RootRecord| (root.checkpoint_txg, root.instance);」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/recovery.rs:5360`））。两序分叉要一条旧实例的记录 txg 高于新实例那一版的 txg。新实例的第一个 txg 取「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`/home/fy5090/.local/share/singlefs/work/m3-batch33t-code-r1/tree/crates/singlefs-core/src/mount.rs:1304`）：只藏一次发布（根槽加全部记录），新实例的 txg 与它相等（`T1D rJ` 那一形，写行盖掉它那一槽），两序同判；要严格低于，得在同一次挂载里把连续两次发布的根槽与全部记录都藏掉，前一次已被系统配置见证，还要再藏系统配置的较新槽（不然 N-配置那一判重读后拒可写）。那是 S1 一族再加四处藏，我没造这一形。

## 四、T7：C692 新下限附近误红——没打中

**做法**（`probe/opus_b33t_sweep.rs`）：随机用户历史，每走一步取会话分配器里每盘的分配记录条数，与 `placements_on_each_device_after_the_new_pool_file_creation` 按这个池的几何算出的下限比，同时在那一刻的盘面上跑一次 `recover`（下限就在走读里判）。动作：覆盖写、顺序写 1–4 个数据单元、建 inode、删第一个文件、崩了再可写挂载。取样：384 / 256 / 240 槽三档各 4 段不带删除、4 段带删除（每段 100 步）；不补环窗口的窄盘 176、264 槽各 4 段（准入关着）；4 GiB 默认环与 1024 MiB 环各 1 段 20 步。共 34 段、3240 步，其中 1977 步被拒（窄盘写满、删除之后只许建 inode），走成的 1263 步每步都恢复过。

```
t7-small-w384 OPUS T7TALLY w384-del seed=101 steps=100 bound=12 min_margin=10 recover_failed=0 refused_ops=53
t7-small-w240 OPUS T7TALLY w240-del seed=102 steps=100 bound=12 min_margin=8 recover_failed=0 refused_ops=67
t7-4g OPUS T7TALLY 4g-default seed=1 steps=20 bound=14 min_margin=12 recover_failed=0 refused_ops=0
t7-4g OPUS T7TALLY 4g-ring1g seed=1 steps=20 bound=16 min_margin=14 recover_failed=0 refused_ops=0
```

34 段 `recover_failed=0`；每盘记录数比下限多出的最小值 8（只出现在带删除的段里），其余 10–14。小盘几何的下限是 12（不是默认几何的 14），4 GiB 两种环 14、16，与实现员的单测一致。

为什么离不近（推的）：新池新建文件之后那一版恰好等于下限；之后每次发布在空槽上新分落点（窗口里换下的那几个不复用），已释放的记录留在账里直到被新落点盖住，所以第一次覆盖写之后就多出约 8–10 条。往下掉只有「两条一槽记录被一个两槽落点合成一条」那一种（`allocator.rs` 的 `reusing_two_reclaimed_one_slot_records_for_a_two_slot_unit_rewrites_the_first_and_deletes_the_second`），一次少一条，要连着合 8 次以上而中间不新分落点，随机历史上没出现。我没有专门造「先在新建文件那一版上合一次」的形：那一版之后的第一次发布已经在空槽上新分了落点。

## 没打中的形状

- T4 纯正常崩溃：写记录那一段 6 次写的 64 个子集 × k ∈ {1, 4}，0 拒（第二节）。合法的多记录发布整次落盘：已有 F1 用例与本轮所有 `Ok` 行都没拒。
- T1 C 钉住不该钉的：M1 的隔离 6–17 槽、到这个进程写掉那一槽为止；没造出跨挂载的形（第三节 3）。
- T1 两序分叉：只给了推理，没造（第三节 4）。
- T5「由记录重建的那一版不退」（主 agent 读法 1）：重建出的那一版走读失败时，它点名的单元施加前两份都验过，走不通只能是与所选根共用的单元坏了，所选根也走不通；退不退结局相同（推的，没造）。读法 2（见证判不出不退）与今天相同；读法 3（C 跟着影子账开关走）只在测试臂上；读法 4 没攻。
- T7：34 段随机历史，见上。

## 这条腿自己的限度

- 全部在 384 槽小盘上造（T7 另扫了几档）；R 只用了覆盖写与三个数据单元的顺序写，没用测试开关把一次发布切成更多条记录（`JournalRecordNamedEntryCapacity::CappedForTests`）。
- 读回全 0 的故障是我按区间手放的（T5 的 `Ru+Vr`、`RowU+VlastRec`，T1 的 `rL/rM/rF`，T4 的单份坏），不是 `history.rs` 登记的注入；T5 两形都是双故障。
- 崩溃状态没走层 0 的枚举：T5、T1 取的是一个固定的崩溃点（R 根槽落、轮换没落；M1 写行根槽落、轮换没落）与之后整次落盘的盘面，T4 枚举了写记录那一段的全部子集（与层 0 那一段的子集同一个域；那一段写的是环里没写过的位置，没有原地覆写的撕裂态）。T1 的红在任何一个让那次写数据落了盘的崩溃状态上都成立，没数个数。
- 「系统当时看不看得到」那几格里标了推的：B 形里取号写的系统配置与 Row 记录反向链指向 V，没在原型里读出来。
- oracle 那一判是直接调库函数造的输入，没在 checker 档的流里量 F2 会不会真退。
- 只抽了一次样：「没打中」的几格按规则还要再抽一次。

## 没做什么

- 没跑 checker 档任何测试、没跑层 0 枚举、没跑门禁；K1、K3 与 T2、T3、T6、T8 不归这条腿。
- 没替主 agent 采纳；三条收严都是零轮。
- 要主 agent 跑的命令：无必须的。若要把第三节的红数成层 0 的状态数，要另写一个调 `enumerate_layer0_versions` 的装置（推的：M1 之后第一次覆盖写那一段单元写约 20 次，单段约 10⁶ 个状态，没数），这一轮没写。

## 草稿

`~/.local/share/singlefs/work/m3-batch33t-code-r1-opus/`：`rerun3/` 是交回前那一遍从空目录跑通的工作目录（`summary.txt` sha256 3a5464f7…，与 `expected/summary.txt` 相同），`run-*.log`、`build*.log` 是开发时的日志，`src/` 是探针的开发拷贝，`report/` 是本报告分段的草稿。`rerun3/` 里的仓副本与两个编译目录、开发用的 `repo/`、`target/`、`rerun2/` 交回前都已删掉；留下的只有日志、`summary.txt` 与探针源码。
