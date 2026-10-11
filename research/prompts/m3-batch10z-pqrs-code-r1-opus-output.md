# m3-batch10z-pqrs-code-r1 攻方腿（opus）报告（2026-10-09）

轮名 m3-batch10z-pqrs-code-r1；攻正文第三节 K1–K5，重点 K1-1、K1-2、K2-1、K2-2、K2-3、K3-1。冻结副本 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree/`（只读）；下文代码行号都是冻结副本的行号，量出的数都在副本拷贝（草稿目录 `~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-opus/repo/`）上，不是入库装置上的数。

## 复跑

```
bash research/prompts/m3-batch10z-pqrs-code-r1-opus-model/rerun.sh [<工作目录>]
```

它从冻结副本拷一份仓，用 `assemble.sh` 把冻结副本里的 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 与 `atk_part*.rs` 拼成新测试目标 `atk_pqrs_opus`（只借原文件的搭建函数，攻方用例都以 `atk_` 开头、`#[ignore]`），release 编，逐件经 `scripts/run-with-memory-cap.sh` 跑（每件 3G、一次一件；编译 12G），输出落 `<工作目录>/runs/`；最后跑四个攻方自己的变异（`mutants.py`，原文恰好命中一次才改，否则退 3）。整份约 15 分钟（推的：各件这一轮实测 3–150 秒相加）。这一轮的原始输出在模型目录 `runs/`。

模型目录文件的 sha256（`sha256sum` 原样）：

```
f0e3da96bdd8b1b8eb85ffd2144624bbad3b945af4887dafdb9ab53244237e3d  assemble.sh
5b12ce3adab95890720471ad24e594da9ab4e63e42a30b3100b1ab325687eb00  atk_part.rs
c91f8d6c8fd7cddba25d912f1c933f5c5a6eedd8f2022d47b4d5001f29e6da64  atk_part2.rs
fd24478d96abd778a78d4c3ba7dc8f1007f73786715b257b60bedff174ee9f03  atk_part3.rs
7a39d4a815abc4dc118ebccf2d760a0aa2c88c1542d2e1eb8ba6eda49165e047  atk_part4.rs
8dc6c79274275ef437538e64fce86a037410a4dd3c53e2d00138999dda2217c1  mutants.py
7768ecf75626f77a79fbc2db894e584bed40ee44c9d7580c27ee07be8965ad4e  rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| K1-1 报「被宽护卡住」的判据 | **打中**（一形） | 判据只看「F 在上限 ∧ C 那一圈罩着的槽里有隔离着的」，不看缺口多大；同一份盘面去掉 C（对照臂）照样被拒的写，也报 `StuckBehindTheWideProtection`（9 形 × 3 个长度 = 27 格，全报 Stuck） | 第一节 |
| K1-1 槽数、R 的 txg、次数 | 没打中 | 报的次数等于推环实发次数、报的隔离数等于 C 罩着的隔离数（单槽 C 的各形）；R 没有根时 `None` 与文档一致 | 第一节、第三节 |
| K1-2 出路二（推环）走不走得通 | **打中** | 9 形 203 个长度格全报 Stuck；推环做成（隔离回 0）之后同一个写再试 3 次，182 格照旧被拒，其中 155 格是对照臂（没有 C）放得下的；实现员自己的三形上长到 60、80 个单元的写同样推完仍拒（6/24 格） | 第一节 |
| K1-2 出路一（卸载重挂） | 没打中（措辞有一处不准） | 上面 203 格、实现员三形 24 格出路一全放得下；R 没有根的那一形出路一的话写「R 按新实例的实例表判成被抛弃」，与实际（再挂时 C 那一圈消失）对不上，不分辨臂 | 第一节末 |
| K2-1 推环真发半路块设备错 | 没打中（一致性）；一处报法不准 | 两形 414 + 154 个注入点：会话 `current`、冻结的发布、盘三样一致，崩了再挂池级 checker 全部判定 0 违例；再推一次被报成 `PublishFailedAfterTheRehearsal`（拒因是冻结着），与「预演与真发只在读盘核换下的那一份对不上时分叉」那句对不上 | 第二节 |
| K2-1 崩溃落在推环中间 | 没打中 | 两形 307 + 171 个写前缀（不是层 0 的整写子集枚举）崩了再挂、再覆盖写三次，池级 checker 0 违例；故障没撤时再挂拒可写（C694 的 B，已知），推环写完 R 那一槽之后的前缀反而挂得上 | 第二节 |
| K2-2 提前盖掉别的被抛弃根、多槽 C | 没打中 | C 两槽（Q、R）15 段历史、环里另有被抛弃根 A 5 段历史：推环逐写前缀 I-7.4 / I-4.8 0 违例，离环次序 Q 先 R 后、A 先 R 后 | 第三节 |
| K2-3 推完之后仍卡 | **打中** | 同 K1-2：推完 C 一槽不剩、隔离 0，而上限退到环里最旧的根（推出来的全是同一状态的空发布根），defer 窗口变成整环，被拒的那个写再试 12 次照旧被拒；同一形不推环、只建 3–4 个 inode 就放得下 | 第一节 |
| K3-1 补隔离漏路 | 没打中 | 两种盘面 × 三种故障 × 80 个种子 × 16 步（o、s、n、f、u）：A 在环里时它记着的落点一槽都没空闲、双扣 0、I-7.4 / I-4.8 0 违例 | 第四节 |
| K3-2 照旧整段隔离那一支 | 没攻 | 那一支就是 C762（已知没做），这一轮没造读盘得来、表里没记着的被抛弃根 | 第六节 |
| K4 判别力 | **打中**（四个改回去的写法拦不住） | 攻方变异 M1（真发失败不挪 `current`）、M2（推环入口不核盘表）、M7（挡着的槽数取全部隔离）、M12（R 那一槽取最先轮到的一槽）对那份 harness 测试文件 35 条全绿；M12 被攻方的两槽 C 用例拦住 | 第五节 |
| K5 | 没攻 | 第十八块只动 checker 档测试，这一轮重型测试不跑 | 第六节 |

## 一、K1-1 / K1-2 / K2-3：被宽护卡住的报与推环这条出路

### 被判的那几句

- 判据：`crates/singlefs-core/src/mounted_session.rs:291`「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」，之后只要 C 那一圈罩着的槽里有一个隔离着就报（`crates/singlefs-core/src/mounted_session.rs:303`「.all(|(_, isolated)| *isolated == 0)」那一判不过才回 `NotWhatStopsTheAdmission`）。缺口多大、推环本身要花多少、推完上限在哪，都不进判据。
- 出路二的话：`crates/singlefs-core/src/mounted_session.rs:221`「宽护随之解开、隔离的 {} 个槽放出来」。
- 类型注释的断言：`crates/singlefs-core/src/mounted_session.rs:194`「两条出路对整圈同样走得通。」
- 上限那一条（推环之后起作用的那一半）：`.claude/kb/decisions/16-发布语义.md:36`「非空有效根不足 4 个时取最旧的有效根」。推环发的全是同一状态的空发布根，D16 已定项 1 不数它们（`.claude/kb/decisions/16-发布语义.md:29`「推空与抬 F 产生的空发布根不算」），推完环里非空状态不足 4 个，上限退到环里最旧的那条根。

### 造法（`atk_part.rs` 的 `atk_k1_ways_out_sweep`，原型里自己造的流）

384 槽那一档小盘、第一个文件之后：覆盖写 `pre` 次（放开：2、4、8）；顺序写每次长 5 个单元、长到 `fill` 个单元（放开：6、11、16、21）；再录一次长 3 个单元的顺序写 R（三条记录），崩在 R 的第一条记录写之后（之后的单元、记录、根都没落）。这是不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，C 那一圈只有 R 那一槽。对照臂是同一个池停在 R 之前的盘面（C 一槽都没有）。每一臂从同一份镜像起，长出去的单元数 g 从 1 逐个加，找第一个被拒的（g_ctrl、g_c）；然后 g 从 g_c 扫到 g_ctrl + 2，每个 g 在 C 臂上各起三份：先试一次报什么；推环之后至多再试 3 次；照出路一正常卸载、再可写挂载、再试。用户在被拒之后做什么不写死：另有 `atk_k1_small_writes_after_push_sweep` 放开「推不推环 × 先做 0–24 次建 inode（或 0–8 次小覆盖写）」再试。

### 原样输出

`runs/k1w-run3.log` 每一形的头一行（`grep -E 'crash_after|r_not'`）：

```
ATK k1w pre=2 fill=6 units=6 crash_after_write=39/56 size_now=6 c_slots=1 c_iso=58 g_ctrl=None g_c=Some(67)
ATK k1w pre=2 fill=11 units=11 crash_after_write=49/76 size_now=11 c_slots=1 c_iso=78 g_ctrl=Some(60) g_c=Some(42)
ATK k1w pre=2 fill=16 units=16 crash_after_write=59/96 size_now=16 c_slots=1 c_iso=98 g_ctrl=Some(39) g_c=Some(17)
ATK k1w pre=2 fill=21 units=21 crash_after_write=73/120 size_now=21 c_slots=1 c_iso=122 g_ctrl=Some(16) g_c=Some(1)
ATK k1w pre=4 fill=6 units=6 crash_after_write=39/56 size_now=6 c_slots=1 c_iso=58 g_ctrl=Some(76) g_c=Some(63)
ATK k1w pre=4 fill=11 units=11 crash_after_write=49/76 size_now=11 c_slots=1 c_iso=78 g_ctrl=Some(62) g_c=Some(37)
ATK k1w pre=4 fill=16 units=16 crash_after_write=63/100 size_now=16 c_slots=1 c_iso=102 g_ctrl=Some(36) g_c=Some(13)
ATK k1w pre=4 fill=21 units=21 r_not_recorded ok(raises=1)
ATK k1w pre=8 fill=6 units=6 crash_after_write=39/56 size_now=6 c_slots=1 c_iso=58 g_ctrl=Some(78) g_c=Some(56)
ATK k1w pre=8 fill=11 units=11 crash_after_write=49/76 size_now=11 c_slots=1 c_iso=78 g_ctrl=Some(55) g_c=Some(40)
ATK k1w pre=8 fill=16 units=16 crash_after_write=63/100 size_now=16 c_slots=1 c_iso=102 g_ctrl=Some(39) g_c=Some(16)
ATK k1w pre=8 fill=21 units=21 r_not_recorded ok(raises=1)
```

数（`grep` 数的，命令与输出原样）：

```
$ L=runs/k1w-run3.log; echo "cells=$(grep -c ' g=' $L) stuck=$(grep ' g=' $L | grep -c 'field=Stuck') notwhat=$(grep ' g=' $L | grep -c 'field=NotWhat') push_ok=$(grep ' g=' $L | grep -c 'push_then=ok') push_fail=$(grep ' g=' $L | grep -c 'push_then=fail') way1_ok=$(grep ' g=' $L | grep -c 'way_one_then=ok$') ctrl_refused=$(grep ' g=' $L | grep -c 'ctrl=refused') ctrl_admitted_push_fail=$(grep ' g=' $L | grep 'ctrl=admitted' | grep -c 'push_then=fail')"
cells=203 stuck=203 notwhat=0 push_ok=21 push_fail=182 way1_ok=203 ctrl_refused=27 ctrl_admitted_push_fail=155
```

主形（pre=4、fill=16）几个 g 的原样行：

```
ATK k1w pre=4 fill=16 units=16 g=13 ctrl=admitted field=Stuck push_then=ok@1 way_one_then=ok
ATK k1w pre=4 fill=16 units=16 g=19 ctrl=admitted field=Stuck push_then=ok@1 way_one_then=ok
ATK k1w pre=4 fill=16 units=16 g=20 ctrl=admitted field=Stuck push_then=fail(pushed(19),iso_after=0) way_one_then=ok
ATK k1w pre=4 fill=16 units=16 g=35 ctrl=admitted field=Stuck push_then=fail(pushed(19),iso_after=0) way_one_then=ok
ATK k1w pre=4 fill=16 units=16 g=36 ctrl=refused field=Stuck push_then=fail(pushed(19),iso_after=0) way_one_then=ok
ATK k1w pre=4 fill=16 units=16 g=38 ctrl=refused field=Stuck push_then=fail(pushed(19),iso_after=0) way_one_then=ok
```

同一形上 g=36（对照臂第一个被拒的长度）推环前后的完整报（`runs/k1-run3.log`）与它报的两条出路原文：

```
ATK k1 pre=4 fill=16 step=5 units=16 arm=crash@63/100 ctrl_grow=36 before=nospace(stop=ceiling(F=7,C=7),last=admission[0:1163223/1916928,1:1163223/1916928],field=Stuck(r_slot=2.3,r_txg=None,n=19,iso=[51, 51]),raises=1) push=pushed(19) iso_after_push=0 after=nospace(stop=budget,last=admission[0:1376215/1916928,1:1376215/1916928],field=NotWhat,raises=2) retries=["nospace(stop=budget,last=admission[0:1310679/1884160,1:1310679/1884160],field=NotWhat,raises=2)", "nospace(stop=budget,last=admission[0:1310679/1916928,1:1310679/1916928],field=NotWhat,raises=2)", "nospace(stop=budget,last=admission[0:1245143/1916928,1:1245143/1916928],field=NotWhat,raises=2)", "nospace(stop=budget,last=admission[0:1245143/1916928,1:1245143/1916928],field=NotWhat,raises=2)"] viol=0
ATK k1 pre=4 fill=16 step=5 units=16 arm=crash@63/100 way_out[0]=出路一：正常卸载（mount::unmount）之后再可写挂载一次。再挂时 R（那一槽没有读得出的有效根）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现；R 的账再挂时仍读不出就照旧拒可写（C694 的 B）。
ATK k1 pre=4 fill=16 step=5 units=16 arm=crash@63/100 way_out[1]=出路二：调 MountedSession::push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states（推环到 R 那一槽，根环槽 2.3）：发 19 次 F 不变的空发布写掉那一槽，宽护随之解开、隔离的 102 个槽放出来；根环里更早的回退候选会被这一串整环写掉，回退候选集掉到少于 4 个不同状态（D16（发布语义） 已定项 1），调用即确认。
```

推环之后逐次再试（`runs/k1-trace.log`，g=20，对照臂放得下的长度）：

```
ATK trace mounted txg=13 F=0 ceiling=Ok(7) iso=102 unblocked=[970, 970] c=(1, 102)
ATK trace before nospace(stop=ceiling(F=7,C=7),last=admission[0:1163223/1392640,1:1163223/1392640],field=Stuck(r_slot=2.3,r_txg=None,n=19,iso=[51, 51]),raises=1) | txg=16 F=7 ceiling=Ok(7) iso=102 unblocked=[1001, 1001] c=(1, 102)
ATK trace push=pushed(19) | txg=35 F=7 ceiling=Ok(12) iso=0 unblocked=[1014, 1014] c=(0, 0)
ATK trace attempt1 nospace(stop=budget,last=admission[0:1376215/1392640,1:1376215/1392640],field=NotWhat,raises=2) | txg=40 F=14 ceiling=Ok(17) iso=0 unblocked=[1014, 1014] c=(0, 0)
ATK trace attempt2 nospace(stop=budget,last=admission[0:1310679/1359872,1:1310679/1359872],field=NotWhat,raises=2) | txg=46 F=20 ceiling=Ok(23) iso=0 unblocked=[1010, 1010] c=(0, 0)
ATK trace attempt3 nospace(stop=budget,last=admission[0:1310679/1392640,1:1310679/1392640],field=NotWhat,raises=2) | txg=52 F=26 ceiling=Ok(29) iso=0 unblocked=[1010, 1010] c=(0, 0)
ATK trace attempt4 nospace(stop=budget,last=admission[0:1245143/1392640,1:1245143/1392640],field=NotWhat,raises=2) | txg=58 F=32 ceiling=Ok(35) iso=0 unblocked=[1006, 1006] c=(0, 0)
ATK trace attempt5 nospace(stop=budget,last=admission[0:1245143/1392640,1:1245143/1392640],field=NotWhat,raises=2) | txg=64 F=38 ceiling=Ok(41) iso=0 unblocked=[1006, 1006] c=(0, 0)
ATK trace attempt6 nospace(stop=budget,last=admission[0:1310679/1392640,1:1310679/1392640],field=NotWhat,raises=2) | txg=70 F=44 ceiling=Ok(47) iso=0 unblocked=[1010, 1010] c=(0, 0)
ATK trace overwrite_after ok(raises=0) | txg=71 F=44 ceiling=Ok(48) iso=0 unblocked=[1002, 1002] c=(0, 0) viol=0
```

用户动作放开（`runs/k1-small-inode.log`：被拒之后推或不推环，再先建 k 个 inode、再试同一个长度）：

```
ATK k1small small=inode pre=4 fill=16 grow=20 push=false k0:no k1:no k2:no k3:ok k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:ok k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
ATK k1small small=inode pre=4 fill=16 grow=20 push=true k0:no k1:no k2:no k3:no k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:ok k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
ATK k1small small=inode pre=4 fill=16 grow=28 push=false k0:no k1:no k2:no k3:no k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:ok k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
ATK k1small small=inode pre=4 fill=16 grow=28 push=true k0:no k1:no k2:no k3:no k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:ok k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
ATK k1small small=inode pre=4 fill=16 grow=35 push=false k0:no k1:no k2:no k3:no k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:no k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
ATK k1small small=inode pre=4 fill=16 grow=35 push=true k0:no k1:no k2:no k3:no k4:ok k5:ok k6:ok k7:ok k8:ok k9:ok k10:ok k11:ok k12:ok k13:ok k14:ok k15:ok k16:ok k17:ok k18:ok k19:ok k20:ok k21:ok k22:ok k23:ok k24:ok
```

实现员自己的三形（`runs/k1-impl.log`，卡住之后用户要写的是长到 g 个单元的顺序写）：

```
ATK k1impl form=RollbackSixVersionsBack { overwrites_before: 20 } grow=40 stuck_iso=440 n=20 first=Stuck push_then=ok@1 way_one=ok
ATK k1impl form=RollbackSixVersionsBack { overwrites_before: 20 } grow=60 stuck_iso=440 n=20 first=Stuck push_then=fail(pushed(20)) way_one=ok
ATK k1impl form=RollbackSixVersionsBack { overwrites_before: 20 } grow=80 stuck_iso=440 n=20 first=Stuck push_then=fail(pushed(20)) way_one=ok
ATK k1impl form=InodeCreation { overwrites_before: 40 } grow=40 stuck_iso=616 n=18 first=Stuck push_then=ok@1 way_one=ok
ATK k1impl form=InodeCreation { overwrites_before: 40 } grow=60 stuck_iso=616 n=18 first=Stuck push_then=fail(pushed(18)) way_one=ok
ATK k1impl form=InodeCreation { overwrites_before: 40 } grow=80 stuck_iso=616 n=18 first=Stuck push_then=fail(pushed(18)) way_one=ok
ATK k1impl form=FirstPublishOfADeletion { overwrites_before: 160 } grow=40 stuck_iso=754 n=19 first=Stuck push_then=ok@1 way_one=ok
ATK k1impl form=FirstPublishOfADeletion { overwrites_before: 160 } grow=60 stuck_iso=754 n=19 first=Stuck push_then=fail(pushed(19)) way_one=ok
ATK k1impl form=FirstPublishOfADeletion { overwrites_before: 160 } grow=80 stuck_iso=754 n=19 first=Stuck push_then=fail(pushed(19)) way_one=ok
```

### 读法

- 主形（pre=4、fill=16）：C 那一圈只有 R 那一槽，每盘隔离 51 槽（`iso=[51, 51]`）。对照臂（同一个池停在 R 之前、没有 C）长到 g=35 都放得下，g=36 起被拒；C 臂 g=13 起被拒，13 ≤ g ≤ 38 每一格都报 `StuckBehindTheWideProtection`（`r_txg=None`、`n=19`）。
- 推环本身做成：19 次空发布、R 那一槽写掉、C 一槽不剩、隔离回 0（`push=pushed(19) | … iso=0 … c=(0, 0)`）。g ≤ 19 推完第一次就放得下；g ≥ 20 推完再试照旧被拒。g=20 那一格的需求是每盘 1392640 字节，推前可用 1163223，推完 1376215，之后每次再试会话推两串抬 F 就碰到一次准入 8 次发布的预算（`stop=budget`），F 每次只跟上 6，上限始终落后现行 txg 23（txg 35 时上限 12，txg 70 时上限 47），可用在 1245143–1376215 之间来回，6 次都不够（`runs/k1-trace.log`）。对照臂同一个长度放得下：这一格挡住准入的确实是宽护，可出路二走不通。
- 机理（读码加上面那几行推的）：推环发的都是同一个用户状态的空发布根，D16 已定项 1 不把它们算成状态；推完环里非空状态不足 4 个，抬 F 的上限按「非空有效根不足 4 个时取最旧的有效根」退到环里最旧的那条根，defer 窗口从「第 4 新的状态」放宽到整个根环。推环每次空发布每盘新占约 6 槽、换下的那几槽进 defer，推完它们要等环再转一圈才回收。C 那一圈小（这里每盘 51 槽）时，放出来的比推环与放宽的窗口吃掉的少。
- 不推环、只建 3–4 个 inode（不动文件）同一个长度就放得下；推环之后反而要 4 个（`runs/k1-small-inode.log`）。这几形里推环比什么都不做还慢一步，还丢了回退候选。
- 出路一（正常卸载再可写挂载）在 203 格全放得下，连对照臂也拒的 g ≥ g_ctrl 那 27 格也放得下：卸载把 F 抬到现行那一版，代价与推环同量级（同样丢回退候选），但它把 defer 窗口一并收掉。
- 实现员自己的三形（每盘隔离 440–754 槽）上，卡住之后写长到 40 个单元的放得下，长到 60、80 个单元的推完照旧被拒，出路一都放得下。用例只试了推完之后覆盖写 5 次与长 2 个单元的顺序写 2 次（`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs:3811`「const SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 2;」）。

### 打中之后的四句

1. **分不分辨臂**：分辨「今天的判据（F 在上限 ∧ C 罩着的槽有隔离着的）」与「把推环之后放不放得下也算进去的判据」：后者在 182 格不会把出路二推给用户。也分辨两条出路：出路一 203/203 放得下，出路二 21/203。不分辨「会话推环入口」与调查报告里的另一形（调公开的 `raise_rollback_floor(new_floor = F)`）：两者发的都是同一状态的空发布，推完上限同样退到环里最旧的根（推的，没量那一形）。
2. **被判的系统当时看不看得到**：看得到。被拒那一刻 `last_refusal` 带着每盘的可用与需求，`root_ring_push_target` 给出要推几次，推环入口已经会在分配器拷贝上预演整串空发布；在那份拷贝上再按推完的根环表算上限、预演一次这次改动的准入（连同它会推的抬 F），就知道推完放不放得下，不用写盘（推的：没实现）。
3. **满足的是哪一个分句**：K1-1「不是宽护卡住却报成它」——对照臂同一个长度照样被拒的 27 格（每形 g_ctrl 到 g_ctrl + 2）；K1-2「照抄出路走，是不是都走得通？」——出路二 182 格走不通；K2-3「有没有一形推完仍卡」——推完仍卡，不是「推不动」也不是「推错槽」（推环本身做成、R 那一槽写掉、隔离回 0）。不属于「用户已定」那一条：用户点头的是回退候选集掉到少于 4 个不同状态，这里是代价付了、写照旧被拒。
4. **跑前条款给的改法还中不中**：正文没给跑前改法。攻方提的三个改法见下表，都只在攻方的模型上推过、没实现、被攻过零轮。

| 改法 | 修哪几格 | 量过还是推的 |
|---|---|---|
| 甲：报 `StuckBehindTheWideProtection` 之前在分配器拷贝上预演推环，再在拷贝上按推完的上限预演一次这次改动的准入；放得下才写出路二，放不下时只写出路一（或报 `NotWhatStopsTheAdmission` 并写明推环也不够） | K1-1 的 27 格、K1-2 的 182 格（不再把走不通的出路推给用户）；不改 K2-3 推完仍卡这件事本身 | 推的 |
| 乙：推环做成之后把 F 抬到推出来的最后一版（与正常卸载同一个抬法，调用即确认已经覆盖丢回退候选这件事） | K2-3 与 K1-2 的 182 格里出路二本身走通（推的：出路一放得下就是因为卸载把 F 抬到了现行那一版） | 推的；越过 D16 已定项 1「抬 F 的上限」那一行，要用户定 |
| 丙：只改出路二那句话，写明推完上限退到环里最旧的根、长写可能照旧被拒 | 只修话；182 格的结局不变 | 推的 |

推翻条件：在入库装置上照第一节造法（自然崩溃点、不带读故障）重做，C 臂报 `StuckBehindTheWideProtection` 而推环之后同一个写放得下的格占多数（比如 182 格里过半转成放得下），或对照臂在 g ≥ g_ctrl 那几格放得下。

### 出路一那句话的一处不准（不分辨臂）

主形里 R 根本没有根（崩在它第一条记录之后），报的是 `r_txg=None`，出路一照模板写成 `crates/singlefs-core/src/mounted_session.rs:218`「再挂时 R（{txg_of_r}）按新实例的实例表判成被抛弃，影子账只护它自己账里引用的落点，宽护不再出现」，代入之后是「R（那一槽没有读得出的有效根）按新实例的实例表判成被抛弃」。实测卸载再挂之后 C 那一圈直接没了（`runs/k1-run3.log` 那一行 `remount_c_slots=0 remount_c_iso=0`），没有哪条根被判成被抛弃。出路一走得通，话不对；只影响报给用户的那一句。

## 二、K2-1：推环真发半路块设备错、崩溃落在推环中间

### 造法（`atk_part2.rs`）

- 池：实现员的两形 `InodeCreation { overwrites_before: 40 }`（no40）与 `RollbackSixVersionsBack { overwrites_before: 20 }`（b6o20），各建一次，卡住那一刻的盘面与会话（分配器 `clone`，会话逐字段拷）存成快照，每段历史从快照拷。
- 块设备错：每块盘外包一层，数整池的改盘调用（写、写零、屏障），第 j 次报 `BlockDeviceError::InputOutput`、不落盘，只这一次。先无故障推一遍数出总数（no40 414 次、b6o20 460 次），j 逐个取（no40 步长 1，b6o20 步长 3）。每个 j：推环交回什么、会话现行那一版的 txg、分配器上冻结着的发布；接着再推一次、覆盖写一次、正常卸载一次各交回什么；最后照那一刻的盘面崩了再可写挂载，池级 checker 全部判定（不只 I-7.4、I-4.8）在挂上那一刻与之后覆盖写三次各判一次。
- 崩溃点：录下整串推环的写流，崩在第 k 个写之后，k 取 0 到全长每一个（no40 306 个写、b6o20 340 个写取步长 2）。这是写前缀，**不是**层 0 的整写子集枚举（这一轮派发不许跑崩溃枚举）。每个 k 两臂：普通盘崩了再挂、checker 全部判定、再覆盖写三次；挂载那一刻 R 那几段照旧读回全零（故障没撤）。

### 原样输出

```
k2err-b6o20.log:ATK k2err form=b6o20 dry=pushed(20) calls=460 start_txg=28
k2err-no40.log:ATK k2err form=no40 dry=pushed(18) calls=414 start_txg=55
ATK k2err form=no40 j=1 fired=write push=failed_after_rehearsal(at=1/18,landed=0,cause=BlockDevice(InputOutput(Custom { kind: Other, error: "攻方注入的写错" }))) current_txg=55 frozen=Some(56) again=failed_after_rehearsal(at=1/18,landed=0,cause=PublishFrozenAfterAWriteFailureIsNotResentYet { checkpoint_txg: CheckpointTxg(56), instanc) overwrite=other(Publish { cause: PublishFrozenAfterAWriteFailureIsNotResentYet { checkpoint_txg: CheckpointTxg(56), instance: InstanceGe) unmount=refused(RaiseFloorSequencePublishFailed(PublishSequenceFailed { caus) crash_remount=remount_ok(sel=2:55 iso=22 c=(0, 0) viol_mount=0 viol_later=0 refused=0)
ATK k2err form=no40 j=401 fired=write push=failed_after_rehearsal(at=18/18,landed=17,cause=BlockDevice(InputOutput(Custom { kind: Other, error: "攻方注入的写错" }))) current_txg=72 frozen=Some(73) again=failed_after_rehearsal(at=1/1,landed=0,cause=PublishFrozenAfterAWriteFailureIsNotResentYet { checkpoint_txg: CheckpointTxg(73), instanc) overwrite=other(Publish { cause: PublishFrozenAfterAWriteFailureIsNotResentYet { checkpoint_txg: CheckpointTxg(73), instance: InstanceGe) unmount=refused(RaiseFloorSequencePublishFailed(PublishSequenceFailed { caus) crash_remount=remount_ok(sel=2:72 iso=0 c=(0, 0) viol_mount=0 viol_later=0 refused=0)
```

两形全部注入点归类（命令原样）：

```
$ for f in k2err-no40 k2err-b6o20; do grep 'j=' runs/$f.log | sed -E 's/.*push=([a-z_]+)\(.*again=([a-z_]+)\(.*cause=([A-Za-z]+).*overwrite=([a-z]+)\(([A-Za-z]+).*unmount=([a-z]+)\(.*crash_remount=([a-z_]+)\(.*viol_mount=([0-9]+) viol_later=([0-9]+) refused=([0-9]+).*/push=\1 again=\2 againcause=\3 ow=\4:\5 unmount=\6 crash=\7 vm=\8 vl=\9/' | sort | uniq -c; done
    414 push=failed_after_rehearsal again=failed_after_rehearsal againcause=PublishFrozenAfterAWriteFailureIsNotResentYet ow=other:Publish unmount=refused crash=remount_ok vm=0 vl=0
    154 push=failed_after_rehearsal again=failed_after_rehearsal againcause=PublishFrozenAfterAWriteFailureIsNotResentYet ow=other:Publish unmount=refused crash=remount_ok vm=0 vl=0
```

崩溃前缀两形的汇总行与头尾几行：

```
ATK k2crash form=b6o20 push=pushed(20) writes=340 segments=80 roots=20
ATK k2crash form=b6o20 summary={"plain_ok_clean with_fault_ok": 2, "plain_ok_clean with_fault_refused": 169}
ATK k2crash form=no40 push=pushed(18) writes=306 segments=72 roots=18
ATK k2crash form=no40 summary={"plain_ok_clean with_fault_ok": 3, "plain_ok_clean with_fault_refused": 304}
ATK k2crash form=no40 k=0 roots_landed=0 plain=remount_ok(sel=2:55 iso=22 c=(0, 0) viol_mount=0 viol_later=0 refused=0) with_fault=refused(NewerStateStillUnreadableAfterOneReread(AccountOfA)
ATK k2crash form=no40 k=303 roots_landed=17 plain=remount_ok(sel=2:73 iso=26 c=(0, 0) viol_mount=0 viol_later=0 refused=0) with_fault=refused(NewerStateStillUnreadableAfterOneReread(AccountOfA)
ATK k2crash form=no40 k=304 roots_landed=18 plain=remount_ok(sel=2:73 iso=0 c=(0, 0) viol_mount=0 viol_later=0 refused=0) with_fault=ok(sel_txg=73)
```

### 读法

- **一致性没打中**：568 个注入点（414 + 154）里，推环都报 `PublishFailedAfterTheRehearsal`，会话的 `current` 停在最后一次落了根的那一版（`at=m/18,landed=m−1`、`current_txg` 与之对上），分配器上冻结着下一个 txg 的那次发布；照盘面崩了再挂全部做得成，挂上那一刻与之后三次覆盖写池级 checker 0 违例。崩溃前缀 478 个状态（307 + 171）普通盘再挂全部做得成、0 违例。I-7.4 / I-4.8 都在全部判定里。
- **故障没撤的那一臂**：推环写到 R 那一槽之前的每个前缀再挂都拒可写（`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot…)`，C694 的 B、调查报告第六节第 7 条已量过，不算打中）；推环最后一条根落了之后的那几个前缀反而挂得上（no40 k=304–306、b6o20 k=338、340）：R 离了环，再挂不再要读它的账。
- **一处报法不准（不分辨臂）**：真发失败之后那次发布冻结在分配器上（`frozen=Some(56)` 一类），会话这一层没有重发它的入口：再推一次、覆盖写、正常卸载全被拒，只能等进程重开。再推一次报的是 `PublishFailedAfterTheRehearsal { failed_publish_in_the_push: 1, cause: PublishFrozenAfterAWriteFailureIsNotResentYet … }`——预演在冻结着的分配器拷贝上照样过了：推环的预演直接调 `crates/singlefs-core/src/mounted_session.rs:729`「let (_, next) = prepare_the_version_publish(」，冻结那一道在 `crates/singlefs-core/src/transaction.rs:6358`「refuse_while_a_publish_is_frozen(allocator)?;」，住在 `publish_version_of_trees` 里、`prepare_the_version_publish` 不判。于是 `crates/singlefs-core/src/mounted_session.rs:348`「预演与真发只在读盘核换下的那一份对不上时分叉」这句在这一形上不成立，第二次的错也不是那一成员注释说的「块设备错这一类」。盘、分配器、会话照旧一致，没有写发出去；会话里冻结之后整条会话都动不了是所有会话发布路径共有的（不是第十六块带来的），这里只记推环入口的报法。改法（推的，零轮）：推环入口第一道先看 `allocator.frozen_publish()`，冻结着就在预演之前拒成一个点名冻结的成员。
- **「预演说推得到、真发因空间不够半路失败」造不出（推的，没量）**：推环里 F 不动，回收门槛 max(F, 环里最旧有效根) 在整串推环里停在推环开始那一版之前（`runs/k1-trace.log` 推完那一刻上限 12、现行 txg 35），推环自己换下的那几份（释放代 ≥ 开始那一版的 txg + 1）在这一串里一份都回收不到；预演与真发唯一的分叉（真发读盘核换下的那一份对不上就留在已分配）因此碰不到这一串取的落点。会话单线程，没有别的分配插进预演与真发之间。

## 三、K2-2：C 有两槽、环里另有被抛弃根时推环

### 造法（`atk_part3.rs`）

- 两槽 C：冻结副本测试文件里「丙并中间」那一形的录法（`record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites`：Q 顺序写三个数据单元、R 建 inode，故障 `cfgq,qroot,qlast,racct`，挂上就撤），前缀覆盖写次数放开（1、3、8），挂上之后、推环之前用户覆盖写次数放开（0、1、3、6、10）：15 段历史。
- 被抛弃根 A：`pool_whose_newest_root_was_abandoned_by_a_crash_recovery` 那份盘面（A 按实例 3 的表被抛弃），实例 4 可写挂载（根环表记着 A 与它的落点），覆盖写 k 次（放开：0、1、2、4、6），再录一次建 inode 得 R、崩在 R 的根落盘之后轮换之前，可写挂载那一刻 R 写的全部单元读回全零（挂上就撤）：C 那一圈与 A 同时在。5 段历史。
- 判法：录推环的整条写流，逐个写前缀（不再挂）判 I-7.4、I-4.8；每条根落了之后看 Q、R、A 各在不在环里；推完之后覆盖写 5 次（两槽）或 8 次（A），每次判。

### 原样输出

```
ATK k2two o=1 pre=0 c_at_mount=(2, 52) c_before_push=(2, 52) pre_refused=0 push=pushed(22) writes=382 iso_before=52 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["Q@root21", "R@root22"] violating_prefixes=0 first=None after5(refused=0,violated=0)
ATK k2two o=8 pre=0 c_at_mount=(2, 52) c_before_push=(2, 52) pre_refused=0 push=pushed(21) writes=373 iso_before=52 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["Q@root20", "R@root21"] violating_prefixes=0 first=None after5(refused=0,violated=0)
ATK k2two o=1 pre=10 c_at_mount=(2, 52) c_before_push=(2, 52) pre_refused=0 push=pushed(12) writes=204 iso_before=52 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["Q@root11", "R@root12"] violating_prefixes=0 first=None after5(refused=0,violated=0)
ATK k2two o=8 pre=10 c_at_mount=(2, 52) c_before_push=(2, 80) pre_refused=0 push=pushed(9) writes=161 iso_before=80 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["Q@root8", "R@root9"] violating_prefixes=0 first=None after5(refused=0,violated=0)
ATK k2ab k=0 abandoned_at_mount=1 abandoned_now=1 a_in_ring=true c=(1, 22) iso_all=46 first=ok(raises=0) push=pushed(21) writes=357 iso_before=46 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["A@root15", "R@root21"] violating_prefixes=0 first=None after8(refused=0,violated=0)
ATK k2ab k=6 abandoned_at_mount=1 abandoned_now=1 a_in_ring=true c=(1, 18) iso_all=42 first=ok(raises=0) push=pushed(21) writes=357 iso_before=42 iso_after=0 c_after=(0, 0) abandoned_after=0 leave_order=["A@root9", "R@root21"] violating_prefixes=0 first=None after8(refused=0,violated=0)
```

20 段历史违例前缀数（命令原样）：

```
$ grep -ohE 'violating_prefixes=[0-9]+' runs/k2two-a.log runs/k2two-b.log runs/k2two-c.log runs/k2ab.log | sort | uniq -c; grep -ohE 'after[58]\(refused=[0-9]+,violated=[0-9]+\)' runs/k2two-a.log runs/k2two-b.log runs/k2two-c.log runs/k2ab.log | sort | uniq -c
     20 violating_prefixes=0
     15 after5(refused=0,violated=0)
      5 after8(refused=0,violated=0)
```

### 读法

没打中。两槽时推环推到较晚轮到的 R 那一槽（Q 在倒数第二条根离环、R 在最后一条根离环），次数与 R 那一槽到现行 txg 的距离对上（挂上之后先写 0 次与 10 次：o=1、3 两形 22 与 12 次，o=8 那形 21 与 9 次）；A 在推环途中离环、隔离随之清掉，那是 A 离开根环的那一次发布，照 D28 已定项 1 第九项清；每个写前缀的盘面上 I-7.4 / I-4.8 都不红。推完 C 一槽不剩、隔离 0。A 那一形里挂上之后第一次覆盖写就放得下（没卡住），所以没量到「C 罩着的隔离与 A 的隔离重叠时报的槽数」那一格（第六节）。

## 四、K3-1：被抛弃根那一圈只隔离分配位没置着的槽，回收时补隔离有没有漏路

### 读码

被抛弃根那一圈跳过的是分配位置着的槽，补隔离要靠每一条回收路回收之后照 `placements_the_isolation_still_protects` 补。core 里回收的调用点（`grep -n 'reclaim_released_records_up_to\|reclaim_released_up_to'`）：挂载内记根（回收之后补）、抬 F 那一串（影子账开着时补）、可写挂载重建分配器那两次（在影子账那一圈之前回收，跳过的槽此时已经回收、分配位不置，照常隔离）、影子账关着（只供测试）与回收窗口强制为 0（只供测试）两处不补。可写挂载那一遍读出的被抛弃根，它们的落点整串记进根环表（`RootReadOnOneRebuildPass::AbandonedRoot { referenced_placements }`），记的是整份，不只没被候选豁免的那一截。读码没找到产品路径上的漏路。

### 造的反例尝试（`atk_part4.rs` 的 `atk_k3_sweep`）

两种盘面（A 那份原样；先在实例 4 里把 F 抬到 A 的 txg，A 引用的 defer 槽不再被候选豁免）× 三种挂上之后的故障（不开、A 的根槽读报错、A 的账读报错）× 80 个种子 × 每段 16 步，动作从 o（覆盖写）、s（长两个单元）、n（建 inode）、f（抬 F 到公开上限，上限不高于 F 时 F 不变、照样走一遍抬 F 那一串的影子账重算）、u（正常卸载再可写挂载）里随机取，空间准入判着（会话自己推的抬 F 也在里面）。每一步之后：A 还在环里时它挂载那一刻记着的落点一槽都不空闲；没有一个槽同时算进已分配与第九项；池级 checker 不红 I-7.4 / I-4.8。四段的汇总行原样：

```
ATK k3 totals={"floor_at_a/account clean": 20, "floor_at_a/account refused_f_refused(RollbackFloorC": 2, "floor_at_a/none clean": 20, "floor_at_a/root_slot clean": 20, "floor_at_a/root_slot refused_f_refused(RollbackFloorC": 27, "floor_at_a/root_slot refused_u_refused(NewerStateStil": 38, "plain/account clean": 20, "plain/account refused_f_refused(RollbackFloorC": 2, "plain/none clean": 20, "plain/root_slot clean": 20, "plain/root_slot refused_f_refused(RollbackFloorC": 17, "plain/root_slot refused_u_refused(NewerStateStil": 21}
ATK k3 totals={"floor_at_a/account clean": 20, "floor_at_a/account refused_f_refused(RollbackFloorC": 3, "floor_at_a/none clean": 20, "floor_at_a/root_slot clean": 20, "floor_at_a/root_slot refused_f_refused(RollbackFloorC": 31, "floor_at_a/root_slot refused_u_refused(NewerStateStil": 36, "plain/account clean": 20, "plain/none clean": 20, "plain/root_slot clean": 20, "plain/root_slot refused_f_refused(RollbackFloorC": 15, "plain/root_slot refused_u_refused(NewerStateStil": 22}
ATK k3 totals={"floor_at_a/account clean": 20, "floor_at_a/account refused_f_refused(RollbackFloorC": 4, "floor_at_a/none clean": 20, "floor_at_a/root_slot clean": 20, "floor_at_a/root_slot refused_f_refused(RollbackFloorC": 52, "floor_at_a/root_slot refused_u_refused(NewerStateStil": 28, "plain/account clean": 20, "plain/account refused_f_refused(RollbackFloorC": 2, "plain/none clean": 20, "plain/root_slot clean": 20, "plain/root_slot refused_f_refused(RollbackFloorC": 28, "plain/root_slot refused_u_refused(NewerStateStil": 13}
ATK k3 totals={"floor_at_a/account clean": 20, "floor_at_a/account refused_f_refused(RollbackFloorC": 6, "floor_at_a/none clean": 20, "floor_at_a/root_slot clean": 20, "floor_at_a/root_slot refused_f_refused(RollbackFloorC": 31, "floor_at_a/root_slot refused_u_refused(NewerStateStil": 40, "plain/account clean": 20, "plain/account refused_f_refused(RollbackFloorC": 2, "plain/none clean": 20, "plain/root_slot clean": 20, "plain/root_slot refused_f_refused(RollbackFloorC": 19, "plain/root_slot refused_u_refused(NewerStateStil": 20}
```

480 段历史全部 `clean`（`grep -h 'problems=' runs/k3-*.log | wc -l` 输出 `0`）。`f_refused(RollbackFloorC…` 与 `u_refused(NewerStateStil…` 是装置的毛病：故障钉在 A 那一槽与 A 的账那几槽的物理位置上，A 离环之后那几槽住进了这个进程写的有效根与现行那一版的单元，读报错照设计拒（有效根的槽读坏、重读仍坏就拒）——推的，没逐条核那几步 A 在不在环里。

## 五、K4：推环入口与报的几处，改回去拦不拦得住

攻方自己的四个变异（`mutants.py`，不是入库变异表；原文恰好命中一次才改），每个在一份新拷的副本上编，对冻结副本那份 harness 测试文件（`raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table`，35 条加 1 条忽略）整份跑：

| 变异 | 改了什么 | 那份测试文件 | 攻方原型 |
|---|---|---|---|
| M1 | 真发失败那一支不把会话的 `current` 挪到最后一次落盘的那一版 | 35 passed（拦不住） | 没跑 |
| M2 | `parameters_of_the_pool_agreeing_with_the_caller_inputs` 不核盘表、直接交会话持着的参数 | 35 passed（拦不住） | 没跑 |
| M7 | 报的挡着的槽数取两块盘全部隔离，不限 C 那一圈罩着的 | 35 passed（拦不住） | 没跑 |
| M12 | `root_ring_push_target` 取最先轮到的那一槽（`.max()?` 改 `.min()?`） | 35 passed（拦不住） | 两槽 C 用例当场 panic（推环末尾那道断言） |

原样（各份 `runs/mut-*-run.log` 的结果行与 M12 对原型那一次）：

```
M1-current-not-moved-on-failure: test result: ok. 35 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.42s
M2-devices-not-checked: test result: ok. 35 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.11s
M7-count-all-isolation: test result: ok. 35 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.12s
M12-r-slot-is-the-first: test result: ok. 35 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.17s
thread 'atk_k2_two_c_slots_push' (2604951) panicked at crates/singlefs-core/src/mounted_session.rs:782:9:
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 45 filtered out; finished in 0.04s
```

M1、M2 是正文第二节自己列的「推环入口「收盘表被拒」「真发时块设备错」两支没有用例」，这里只是坐实改回去照绿（不算打中新东西）。M7、M12 是新的：实现员的三形 C 都只有一槽、根环表里没有被抛弃根，「R 那一槽是最后轮到的一槽」与「挡着的槽数只数 C 罩着的」两件事在它们上面与改坏的写法逐数相同。补法（推的，零轮）：把第三节的两槽 C 那一形（或在三形之一上另加一条被抛弃根）做成推环用例，钉住次数等于较晚那一槽的距离、推完两槽都写掉；报的槽数与「推完隔离降了多少」逐盘比。

## 没打中的形状

| 形状 | 取样范围 | 结局 |
|---|---|---|
| K1-1 反方向：宽护挡着而报成 `NotWhatStopsTheAdmission` | 第一节 203 格的第一次被拒全看 | 0 格；推完之后再试时报 `stop=budget`、`NotWhat`，那时 C 已一槽不剩，报得对 |
| K1-1 次数、R 的 txg、槽数 | 第一节 9 形、实现员三形、第三节 20 段 | 次数与实发相等；单槽 C 的槽数与 C 罩着的隔离相等；R 没有根时 `None` |
| K2-1 块设备错之后盘、分配器、会话不一致 | 两形 568 个改盘调用注入点（每一个写、FUA、屏障都轮到），一次性错 | 一致；崩了再挂 0 违例 |
| K2-1 推环中间崩溃 | 两形 478 个写前缀 | 普通盘再挂 0 违例；故障没撤时拒可写是已知的 C694 的 B |
| K2-1 预演过、真发因空间半路失败 | 读码（第二节末） | 造不出，推的 |
| K2-2 提前盖掉别的被抛弃根或回退候选、I-7.4 红 | 两槽 C 15 段、被抛弃根 A 5 段，逐写前缀 | 0 违例；A 在它那一槽被写掉时离环 |
| K2-3 推不动、推错槽、F 被抬错 | 同上 20 段与第一节 | 推环本身每次都做成、写到较晚那一槽；推环不动 F |
| K3-1 补隔离漏路 | 480 段 × 16 步 | 0 漏、0 双扣、0 违例 |

## 这条腿自己的限度

- 第一节的「推完仍卡」只在 384 槽那一档小盘上量过（单元区 384 槽加环窗口补足 815 槽、S = 8）；根环更长（S 更大）时推环次数与放宽的 defer 窗口都随 3S 变，比例会不同，没量。主形的阈值（g = 20 起推完仍拒）是这一档几何上的数。
- 推完之后用户动作放开得不全：放开了「再试同一个写 3–12 次」「先建 0–24 个 inode」「先小覆盖写 0–8 次」三种；没放开「推完之后先卸载」（那就是出路一）与删除类改动。
- 第一节对照臂用来判「是不是宽护挡着」：对照臂与 C 臂只差那几条没施加的记录，挂载各自推过一次抬 F，两臂的 F、上限逐数相同（`ceiling(F=7,C=7)` 两臂都是），可见的差别只有 C 那一圈；但对照臂「放得下」本身也受「保留 4 个状态」的上限约束，所以 K1-1 那 27 格的判法是「同一个上限下没有 C 照样被拒」，不是「没有任何保护时被拒」。
- 第二节的崩溃点只取写前缀，不是层 0 的整写子集枚举：派发这一轮不许跑崩溃枚举，所以没调 `enumerate_layer0` / `enumerate_layer0_versions`，也没算 `closed_form_state_count`。段内乱序落盘的那些状态一个都没量。
- 块设备错只注入了一次性的错（之后照常写）；盘一直坏下去的那一种没造。
- 第三节没量到「C 罩着的隔离与被抛弃根的隔离重叠、而且卡住」那一格：A 那一形挂上之后不卡。报的槽数在那一格是不是虚高，只是读码推的（`isolated_slots_covered_by` 数的是 C 罩着的槽里隔离着的，不分是谁隔离的）。
- 第四节的故障钉在物理位置上，A 离环之后会打到别的东西，那几步的拒是装置造出来的（推的）；80 个种子 × 16 步之外的长度没扫。
- 攻方提的改法（第一节表里甲、乙、丙，第二节末与第五节末的两条）都只在攻方的模型上推过、没实现、被攻过零轮。

## 取样与估时

正式跑之前先各跑一小段估时（K1 一形 1.7 秒、一形 g 扫 3.6 秒；块设备错 41 个注入点 6.7 秒；K3 6 段 × 6 步 3.0 秒），估出全量都在 40 分钟以内，没有缩。并行时每批 4 件、每件 3G（合计 12G）、线程 3：

| 批 | 件 | 段数 | 估时 | 实际 |
|---|---|---|---|---|
| 1 | 块设备错 no40（414 点）、b6o20（154 点）、崩溃前缀 no40（307 态）、b6o20（171 态） | 4 | 约 70–150 秒一件 | 4 件 rc=0，超过前台 240 秒转后台，完成通知后读的 `.rc`（4 个） |
| 2 | 两槽 C 三件（15 段）、被抛弃根一件（5 段） | 4 | 约 1–3 分钟一件 | 4 件 rc=0 |
| 3 | K3 四件（各 20 种子 × 6 组合 × 16 步） | 4 | 约 2–3 分钟一件 | 4 件 rc=0 |

K1 的几件（203 + 24 格）与变异各单独前台跑，各 4–35 秒。编译：release 编约 7–22 秒一次。开跑前 `ps` 看过：没有性能测量、没有别的 cargo 在跑。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、崩溃枚举、GPU 测试；只编、跑了 harness 档（自己拼的 `atk_pqrs_opus` 与冻结副本那份 harness 测试文件）。
- K3-2（照旧整段隔离那一支）没攻：那一支就是 C762，第二节列在「已知留着的」。
- K5（第十八块的扫源码用例会不会逼出别扭的产品写法）没攻。
- 没判正推、辩方的格；副本上量出的数不是入库装置上的数，要引得在入库装置上重做。
- 草稿目录里的仓副本与编译目录（`~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-opus/repo`、`repo-mut`、`target`、`target-mut`）交回前删掉；原始输出已拷进模型目录 `runs/`，草稿目录只留日志与 `model-src/`。

## 引文核对

`python3 scripts/cite-check.py research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree --root /home/fy5090/code/singlefs` 末行原样：

```
  ✓ 核了 11 处引文，对上 11 处，没判 0 处（逐处列在上面）
```
