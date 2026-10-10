# m3-batch10z-b-code-r1 攻方腿报告（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 R C P X1 F1 M6 M7 M11 H1 H2 -->

被判对象：冻结副本 `~/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/`（10z 第二块合入之后）。下面引代码一律写冻结副本的绝对路径与它自己的行号（主工作区的 `mount.rs` 已与冻结副本不同）。所有数都是在冻结副本的拷贝（草稿目录 `~/.local/share/singlefs/work/m3-batch10z-b-code-r1-opus/repo/`）上量的，是副本上的数。重型测试一条没跑；跑的只有 harness 档测试二进制。

## 复跑

```
SINGLEFS_FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs \
  bash research/prompts/m3-batch10z-b-code-r1-opus-model/rerun.sh <新工作目录>
```

末尾应打出「counts.txt 全同」「k3.txt 全同」。`rerun.sh` 本身没有从头空跑过一遍（见「这条腿自己的限度」）；`out/` 里的产物是同一组脚本与规格在草稿目录里分批跑出来的。

模型目录各文件 sha256（`out/*.log` 原样日志 108 份不逐个列，合并起来 `cat out/*.log | sha256sum` = `8aeab9a3a10d1c7fccba4193eb2ac4f971583a0703d817033e84954c33d0968c`）：

```
18ff11e460e31cd7b719cab81928f7c5217d38877c7d5f2c1ed55bdaed20f712  out/counts.txt
d05ff7fad89887275855f204c5a51865070cc0b8f16b6651e04e8f1f44268192  out/f1-summary.txt
4768b54c37ab85792022d61893ff5ae1874b2dc706d70898daa10f2a6983897f  out/k3.txt
45d713d92f02a4ac2440d1aa02f40c281f3c6f7680581fa36993cc154fdc9381  out/x1-summary.txt
d6875832a731c33b72c47b6fee4eebcc52a1b10cfeafb4d6f12c48215eca0641  README.md
f6fb0cffeb538a40598bd285b96bb4c8ee8fd9db4c9be10929e0ac28bd348685  rerun.sh
a42b846dd96aaaa3b290a816397163e496c323f5a919ade4f7c9595e5c50eddd  scripts/agg_f1.py
883a41534b2edc61f66ee14bd26b32e81afcb0bbd7c8d61dd47aa9ebf2942328  scripts/agg_x1b.py
2824bdf011bf9476342e6a72180b4448d3ce584fea9bb038f53cf20863f19939  scripts/batch2.sh
bbd9bcc903d3e4f44b74f8d684ec43b42aa7348589e8027d381aad8b19ede388  scripts/batch.sh
7aa68fe2ff2dc0dd0c50c335dd357a402ca237eb1dbddece93974b7b44dd6228  scripts/count.py
0ff3f30f10b06d9c8f1f524c4c3566550ccaf2593e7a3e2521db082dc0b7aedf  scripts/k3-targets.txt
795dc6af7c45c78b328f51e3fc48c86b50c815b1a79917c073d0d04c37869df6  scripts/spec-b1.tsv
1284a72b25a211b38912b6f47b4db268f4e19faacfcf34110e5b0cdeee045ff4  scripts/spec-b2.tsv
45ce66f8fa1f74b491190f04490b7342d4a72609a7ca51b6e9bc8d5c0384ffe1  scripts/spec-b3.tsv
ed950f9491dbe5a5319348645b373448f5aa678763b44079a51a359f8e3ef0cb  scripts/spec-b4.tsv
b1682ff5f0709e79812b414386c2deff176daae4e5bf0e09635c8bd13a36375f  scripts/spec-b5.tsv
e6dded02bfdb7ca10c43a0b16090b95d0774e8913c0ce40dc627b33393e5b989  scripts/spec-b6.tsv
9a6102a578b5945c836ca8fa90fdf7dda23572325b5ef65d724d6b47b745cdb0  scripts/spec-k3-base.tsv
08517b38b005a47d27c16461d1bce727d7481a29bc03279112ce9ccc5dd4a8b5  scripts/spec-k3-m11.tsv
fd35fe724f06ad2200a319d01ddba5668df286a84daaf4b13089d682df2612ff  scripts/spec-k3-m6.tsv
b5cfb5a6f029b05b6b6a0fc8730bfb298cef55a707b031aae8ed8894bc767a66  scripts/spec-k3-m7.tsv
494d6c1548416415a079688fc5a977022cecda3fe95f7322405ce6a79d731c1e  src/opus10zb_probe_abandoned_slot.rs
ef27a812f044a31167f526c58106f194ec70960f974c8997ea00fcc69c7afa06  src/opus10zb_probe_form1.rs
2442b028f684642fcfb564038acaf3ed3161ecc74bf1b0f9bdd532ac3960b6e6  src/opus-arms-and-mutations.patch
```

## 各格判定一览

| 格 | 判定 | 依据 |
|---|---|---|
| K1-1 抬 F 的路径漏了哪一条 | **打中（白拒，量过）**：H1。四个入口都经 `raise_the_floor_through`，影子账那一处都照记着的落点；但同一串里另两处仍读被抛弃根自己的**根槽**、读坏重读仍坏就拒：抬 F 重算那一遍与准入算上限那一遍。C 记在表里、挂上之后 C 的根槽读报错：今天 90 / 90 段历史以拒收场（卸载拒、写到准入推抬 F 那一次拒），checker 0 红；把被抛弃根的槽从「仍坏就拒」里拿掉、照记着的落点进影子账的臂 0 / 90 段在 C 仍被抛弃时拒，checker 0 红 | 第一节 |
| K1-2 (实例代号, txg) 认错根 | 没打中（读码加入库库单测） | 第一节 |
| K1-3 可写挂载那一道动没动 | 没打中：只多传读盘那一臂（读码） | 第一节 |
| K2-1 记着的落点掉出影子账 | **没打中（一次抽样）**：X1 540 段、F1 2070 段历史（含多次抬 F、挂着时管理员回退再抬 F、写满推抬 F、这个进程写掉那一槽之后）checker 不红 I-7.4 / I-4.8 | 第二节 |
| K2-2 第 1 形两臂 | **打中（白拒，量过）**：H2。R 挂载那一刻按 R 自己的表判有效、记成有效根，写行之后被抛弃；挂上之后 R 的账读报错：今天 285 / 285 段以拒收场，C 那一圈按槽记的落点那一臂 0 / 285，两臂 checker 0 红、R 的单元被写掉 0。R 的账只在挂载那一刻读回全零：两臂各 345 段 0 拒 0 红（没量到掉出影子账） | 第二节 |
| K2-2 第 2–4 形 | 第 2 形今天照拒，是入库用例钉着的处置；第 3、4 形没造（推的） | 第二节 |
| K2-3 `abandoned_roots_unreadable` 的调用方 | 没打中：产品代码里没有靠它做决定的（`grep`） | 第二节 |
| K3 改回去照绿 | **打中两处缺口（量过）**：抬 F 那一遍「表里记着空串就照用」改成照读盘（M7）15 份二进制全绿；抬 F 那一遍从映射现算关掉（M11）默认档全绿，只有一条 `#[ignore]` 慢档用例红。另：挂载记落点只记真隔离的（M6）6 条红，拦得住。LR3 改完照旧钉着「影子账交 Err 时换回分配器」，不钉、改前也没钉「从映射现算」；LR2 的文档句「抬 F 的影子账从映射现算」改完之后不再是实话 | 第三节 |

## 一、K1　定案做到没有

### K1-1　抬 F 的路径：影子账那一处做到了，同一串里读被抛弃根自己根槽的两处没动（H1，打中，白拒）

入口：`raise_rollback_floor` 与 `unmount` 都进同一个函数（「let raised = raise_the_floor_through(」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3208`）、同一句在 `/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3259`），准入推空间那一路经「raise_rollback_floor(」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3418`）。影子账那一处只有一个调用点传记着的那一臂（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3851`），被抛弃根先取表里记着的：「.or_else(|| placements_referenced_by_root(devices, unit_area_start, root))」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:1581`）。这一半做到了，读码与入库新用例都对得上。

漏的是同一串里读被抛弃根**自己根槽**的两处。根环表把被抛弃根的槽算进「知道住着根的槽」：「| RootRingOccupant::AbandonedRoot { .. }」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/allocator.rs:1174`）。抬 F 重算影子账与回收门槛那一遍拿这一份当「重读仍坏就拒」的槽：「ring_slots_known_to_hold_a_root_by(allocator)」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3596`）；准入算上限那一遍同样：「if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:2806`）之后重读仍坏就交「MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:2814`）。于是挂上之后被抛弃根 C 的**根槽**读报错（账读得出），抬 F 那一串照拒，与 K4-2 那一形（C 的账读报错）同一个结局，只是换了一个读不出的地方。

**造法**（`src/opus10zb_probe_abandoned_slot.rs`）：入库新用例同一造法造出被抛弃根 C（覆盖写 1–3 次之后取号 2 写 C、藏 C、崩了再挂取号 3 抛弃 C、写回）；可写挂载（实例 4，准入照公式判、影子账开着），挂载那一刻读得出，根环表把 C 记成被抛弃根、记着 22 个落点；之后故障前覆盖写 k 次（k ∈ {0,1,2,3,5,8}），从这一刻起 C 的根槽那一个物理块读报错（盘上字节不动），再做用户动作：卸载 / 写到准入推出抬 F 再写 3 次后卸载 / 回退一版后卸载 / 回退后写再卸载 / 写后回退再卸载（三种起点 × 6 个 k × 5 种动作 = 90 段）。对照故障：C 的账三个根节点读报错（同入库用例）、不注故障。

原样（`out/x1-today-s1-slot.log`、`out/x1-slot-s1-slot.log`，方括号里是那一步之前 C 那一槽在根环表里的类）：

```
OPUSX1 arm=today start=1 fault=slot k=2 mode=unmount c=(2, 8) kind_at_mount=AbandonedRoot recorded=22 pre=ok unmount[AbandonedRoot]=Err:NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{ring_slot:RootRingSlot{region:2,slot:2},first_reading:Unreadable,rere slot_now=gone faulted_reads=3 checker=-
OPUSX1 arm=today start=1 fault=slot k=2 mode=write+unmount c=(2, 8) kind_at_mount=AbandonedRoot recorded=22 pre=ok write[AbandonedRoot]=Err@11:FloorRaiseFailedWhilePushingForSpace(FloorRaiseFailedWhilePushingForSpace{cause:RollbackFloorCeilingRootRingSlotStillBadAfterOneReread{ring_slot:(n=11,raises=0) unmount[AbandonedRoot]=Err:NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{ring_slot:RootRingSlot{region:2,slot:2},first_reading:Unreadable,rere slot_now=gone faulted_reads=45 checker=-
OPUSX1 arm=slot start=1 fault=slot k=2 mode=unmount c=(2, 8) kind_at_mount=AbandonedRoot recorded=22 pre=ok unmount[AbandonedRoot]=ok(unreadable=0) slot_now=gone faulted_reads=14 checker=-
OPUSX1 arm=slot start=1 fault=slot k=2 mode=write+unmount c=(2, 8) kind_at_mount=AbandonedRoot recorded=22 pre=ok write[AbandonedRoot]=ok(n=14,raises=1) unmount[ValidRoot]=Err:NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot{ring_slot:RootRingSlot{region:2,slot:2},first_reading:Unreadable,rere slot_now=gone faulted_reads=79 checker=-
```

计数（`python3 scripts/count.py <草稿目录>`，`out/counts.txt` 原样）：

```
X1 arm=today fault=slot histories=90 refused=90 refused_while_C_abandoned=90 checker_red=0
X1 arm=today fault=account histories=90 refused=0 refused_while_C_abandoned=0 checker_red=0
X1 arm=today fault=none histories=90 refused=0 refused_while_C_abandoned=0 checker_red=0
X1 arm=slot fault=slot histories=90 refused=18 refused_while_C_abandoned=0 checker_red=0
X1 arm=slot fault=account histories=90 refused=0 refused_while_C_abandoned=0 checker_red=0
X1 arm=slot fault=none histories=90 refused=0 refused_while_C_abandoned=0 checker_red=0
```

slot 臂剩下那 18 段拒，都是这个进程已写掉 C 那一槽（类已换成有效根、故障留在那个物理块上）之后的卸载：那是一条这个进程写过的有效根读坏，照 D16 拒，与被抛弃根无关（第四段原样即一例）。checker 只看 I-7.4、I-4.8 两条（`checker=-` 是两条都不红）。管理员回退在两臂都因同一槽拒（`CandidateJudgementStillUnreadableAfterOneReread`），回退那一处我没改，是 D16 定过的「回退照拒」。

字面：D16 已定项 1「根槽这一次读坏」那一行前半「这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员」（`.claude/kb/decisions/16-发布语义.md:37`）罩着被抛弃根的槽（它在「这个进程知道住着根的槽」里：挂载那一刻读得出）。同一行末句「不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒」（`.claude/kb/decisions/16-发布语义.md:37`）说的「读不出」是它的**账**。所以今天的拒照字面成立，是代价，不是违反；但它与用户 2026-10-07 定案要解的是同一件事（K4-2：正常卸载被一条被抛弃根挡住），被抛弃根的根槽在这一串里只用来取它的 (实例代号, txg) 去查表，上限与回收门槛都不看被抛弃根。交主 agent 判这是条款没写、还是照前半句拒。

### K1-2　按 (实例代号, txg) 认同一条根

没打中（读码，推的）：
- 「对上了却不是同一条根」要两条不同的根同 (实例代号, txg)。txg 在一个实例里只增，新实例取新号；同一槽的内容只有这个进程写得掉，写掉时 `record_root_written_by_this_process` 把那一槽换成有效根。唯一留旧记录的是发布失败退回那一支，那时表里还记着被抛弃根、盘上那一槽可能已是这个进程的新根：按新根的键查不中，交 `None`，落回读盘，新根本身有效、不进影子账，保守。
- 「是同一条根却对不上」要挂载与抬 F 读同一槽读出不同内容：盘上没变，读出的就是同一条，或读坏（那就不进这一遍的根，见 H1）。
- 挂载判被抛弃用最新那条根的表，抬 F 用现行那一版的表（「.filter(|root| abandoned_by_table(root, &table))」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:3829`）。现行那一版的表 = 所选那一版的表加这次挂载写的行，写的行抛弃所选那一版之后的全部，所以挂载那一刻判被抛弃的根，抬 F 时仍判被抛弃；反过来有第 1 形（K2-2）。入库库单测已按根钉住两类槽（变异 3260、3261 行）。

### K1-3　可写挂载那一道

没打中（读码）：`rebuilt_allocator` 只多传「PlacementsOfAbandonedRoots::ReadFromTheirAccounts,」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:2521`），读不出的那一串（映射现算、重读、凭据、拒）照旧走 `placements_of_an_abandoned_root_whose_account_is_unreadable`。入库第三条新用例在副本上绿（第三节基线 `raise_floor_takes_…` 3 passed）。

## 二、K2　I-7.4 与 I-4.8

### K2-1　记着的落点掉出影子账：没打中（一次抽样）

先读码推：挂载那一刻记进表里的是那条根引用的**全部**落点（候选根、现行账豁免的也在内），来自同一次 `isolate_slots_referenced_only_by_abandoned_roots`；账读不出时记的是映射现算的那一份（入库用例钉着它与读得出那一臂同是每盘 14 槽），凭据放行时记空串（那几个根节点槽已被之后的合法写占了）。被抛弃根离开根环之前它的账在盘上不变，所以「挂载时记得不全」只剩「挂载那一刻读得出的部分少于抬 F 那一刻读得出的部分」：账的部分读坏（实例表链、分配记录树中间节点读得出多少认多少）。这一形在 384 槽小盘上造不出（实例表一片、分配记录树一层），推的。

试过的形状（全在副本上，checker 只看 I-7.4、I-4.8；F1 另看 I-7.2、I-2.1）：
- X1：C 记在表里，故障在 C 的根槽 / C 的账 / 无；故障前覆盖写 0–8 次；之后卸载、写到准入推出抬 F（多次抬 F）再写、挂着时管理员回退再卸载、回退后再写、写后回退。三种起点，540 段，0 红。其中 slot 臂有 18 段这个进程已写掉 C 那一槽（「写行之后那一槽被这个进程写掉又换类」），0 红。
- F1：第 1 形 R（挂载那一刻有效根、写行之后被抛弃），覆盖写 3、5、8 次之后录 R（覆盖写一次 / 顺序写三个数据单元），R 写出的每个单元轮流在挂载那一刻读回全零；R 的账挂上之后读报错（A）/ 只在挂载那一刻读回全零（B）/ 无；之后卸载、写 3 次、写 12 次（推抬 F）、回退、写 12 次回退再写 6 次。两臂共 2070 段，0 红，R 写出的单元被写掉 0（第 1 形那几格）。
- T1 甲那一类（两遍里只有一遍自证过的被抛弃那一半）没另造：入库 `rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root` 37 条在副本上绿，且第三节 M6（挂载记的落点只记真隔离的）把其中 3 条打红，说明那几条确实走到「照记着的落点补隔离」。

### K2-2　第 1 形（K4-1 的 R）两臂：打中（H2，白拒；掉出影子账没量到）

造法：入库 `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs` 的 `record_one_publish_after_overwrites` 与 `crash_image_with_the_root_persisted_and_the_rotation_lost`；可写挂载那一刻 R 写出的某一个单元读回全零，择根退到 P，R 的根槽与账读得出。挂载判被抛弃用的是最新那条根的表（「let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:2405`），R 是最新那条根，判有效，记成「(false, RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg }) => {」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/mount.rs:2575`）那一臂、不带落点；写行之后 R 按现行的表被抛弃。两臂：
- today：冻结副本原样（表里没记着，照读盘、照 B）；
- cring：取 C 那一圈在 R 那一槽记着的落点当记着的落点（`placements_named_by_unapplied_journal_records`，「placements_named_by_unapplied_journal_records: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>>,」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-core/src/allocator.rs:1070`）。

原样（`out/f1-today-k5-ow-a.log`、`out/f1-cring-k5-ow-a.log`、`-b.log`）：

```
OPUSF1 arm=today k=5 kind=ow r=(1, 9) p=(1, 8) zero=25116672 var=A mode=unmount selected=(1, 8) r_kind_at_mount=ValidRoot cring_r_slot=48 unmount=Err:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot{abandoned_root:RollbackTarget{instance:InstanceGeneration(1),checkpoint_txg:C r_units_overwritten=0 faulted_reads=8 checker=-
OPUSF1 arm=today k=5 kind=ow r=(1, 9) p=(1, 8) zero=25116672 var=A mode=w12+unmount selected=(1, 8) r_kind_at_mount=ValidRoot cring_r_slot=48 w12=Err@12(raises=0):FloorRaiseFailedWhilePushingForSpace(FloorRaiseFailedWhilePushingForSpace{cause:NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot{ r_units_overwritten=0 faulted_reads=8 checker=-
OPUSF1 arm=cring k=5 kind=ow r=(1, 9) p=(1, 8) zero=25116672 var=A mode=unmount selected=(1, 8) r_kind_at_mount=ValidRoot cring_r_slot=48 unmount=ok(unreadable=0) r_units_overwritten=0 faulted_reads=0 checker=-
OPUSF1 arm=cring k=5 kind=ow r=(1, 9) p=(1, 8) zero=25116672 var=A mode=w12+unmount selected=(1, 8) r_kind_at_mount=ValidRoot cring_r_slot=48 w12=ok(n=12,raises=1) unmount=ok(unreadable=0) r_units_overwritten=0 faulted_reads=0 checker=-
OPUSF1 arm=cring k=5 kind=ow r=(1, 9) p=(1, 8) zero=25116672 var=B mode=w12+rollback+w6+unmount selected=(1, 8) r_kind_at_mount=ValidRoot cring_r_slot=26 w12=ok(n=12,raises=1) rollback=ok w6=ok(n=6,raises=0) unmount=ok(unreadable=0) r_units_overwritten=0 faulted_reads=0 checker=-
```

计数（`out/counts.txt` 原样；form1 = 择根落到 P 且挂载那一刻 R 记成有效根；other = 择根落到 R 自己或别的版本，故障落在现行那一版的账上，不是这一格）：

```
F1 arm=cring form1 var=A histories=285 refused=0 checker_red=0 r_units_overwritten=0
F1 arm=cring form1 var=B histories=345 refused=0 checker_red=0 r_units_overwritten=0
F1 arm=cring form1 var=none histories=285 refused=0 checker_red=0 r_units_overwritten=0
F1 arm=cring other var=A histories=60 refused=20 checker_red=0 r_units_overwritten=0
F1 arm=cring other var=none histories=60 refused=0 checker_red=0 r_units_overwritten=8
F1 arm=today form1 var=A histories=285 refused=285 checker_red=0 r_units_overwritten=0
F1 arm=today form1 var=B histories=345 refused=0 checker_red=0 r_units_overwritten=0
F1 arm=today form1 var=none histories=285 refused=0 checker_red=0 r_units_overwritten=0
F1 arm=today other var=A histories=60 refused=20 checker_red=0 r_units_overwritten=0
F1 arm=today other var=none histories=60 refused=0 checker_red=0 r_units_overwritten=8
```

读法：
- A（挂上之后 R 的账读报错）：today 285 / 285 段以拒收场，拒在卸载或准入推抬 F 那一次写；R 的单元同时由 C 那一圈护着（`cring_r_slot` 48 个落点，C 并上 R 那次发布的账），一个都没被写掉、checker 0 红——拒是白拒。cring 0 拒 0 红。这一格按 D16 末句字面（「不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒」（`.claude/kb/decisions/16-发布语义.md:37`）是违反：R 是抬 F 那一串里的被抛弃根，住在有效根那一类槽，今天照读盘、读不出照拒。
- B（R 的账只在挂载那一刻读回全零）：C 那一圈读不到 R 的账，只剩记录点名的单元与新根段（26 个落点）；today 抬 F 时读盘读得出，cring 照这 26 个。两臂 345 段都 0 拒 0 红：**cring 掉出影子账这一面在这档盘上没量到**。按推：R 引用、又不在 C 那一圈里的只有 R 照抄 P 的那几片（P 的实例表片等），它们在新根段指针里或仍被现行账、候选根罩着；大一点的池（多片实例表、多层分配记录树）上这一面没造过。
- other 里那 8 段「R 的单元被写掉」是择根落到 R 自己、之后回退又写：R 那时是低于 F 的有效根，checker 不红，不是这一格。

### K2-2　第 2–4 形

- 第 2 形（挂载那一刻读坏或读回全零的槽，抬 F 读出被抛弃根）：表里没有落点可用，挂载那一刻也没读过它的账；今天读盘、读不出照映射、重读、凭据、拒。入库 LR3 与慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable` 钉着它。这一形若也要「读不出也不拒」，手里没有一份挂载时的落点，只能拿 C 那一圈（若它在水位之上）或照拒；没造两臂（推的）。
- 第 3 形（E2 重读读出被抛弃根、那一槽已从表里拿掉）：没造（推的）：与第 2 形同一处置。
- 第 4 形（那一槽记着的是另一条根）：见 K1-2，只在发布失败退回那一支出现，那时盘上那一槽是这个进程的根、有效，不进影子账（推的）。

### K2-3　`RaisedFloor::abandoned_roots_unreadable` 的调用方

没打中。冻结副本 `grep -rn '\.abandoned_roots_unreadable' crates --include=*.rs` 去掉 `tests/` 之后，读 `RaisedFloor` 这一项的只有两个实验装置打印它（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 第 608 行、`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs` 第 1304 行）；`e158_root_choice_repair.rs` 读的是 `MountOutput` 那一项（挂载那一遍，口径没变）。产品代码里没有靠它分支的。

## 三、K3　测试判别力

在副本上加三条环境变量开关的变异（`src/opus-arms-and-mutations.patch`，`OPUS_MUT`），每条跑与抬 F、影子账、被抛弃根有关的 15 份 harness 测试二进制（`scripts/k3-targets.txt`，实现员列的 13 份加两份重建分配器重读那一族）与慢档 `raising_the_floor_counts_abandoned_roots_whose_ledger_is_unreadable`（`--include-ignored`）。基线（不设变异）16 件全绿。

| 变异 | 改坏哪一处 | 结果（`out/k3.txt`） |
|---|---|---|
| M6 | 挂载时记进根环表的被抛弃根落点只记这一次真隔离了的，候选根、现行账豁免的不记 | 拦得住：6 条红——`an_abandoned_root_with_a_two_page_instance_table_and_an_unreadable_account_isolates_both_pages_from_the_mapping`、重建分配器重读那一份 3 条（`an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring` 等）、`unreadable_abandoned_root_slot` 2 条。入库那 3 条新用例照绿 |
| M7 | 抬 F 那一遍表里记着的是空串（挂载时凭据放行、或影子账关着）就不用它、照读盘 | **照绿**：16 件全绿。没有用例走到「挂载时凭据放行的被抛弃根、抬 F 时照空串算」 |
| M11 | 抬 F 那一遍表里没记着、账读不出的被抛弃根不从中央映射现算，直接重读、看凭据、拒 | **默认档照绿**，只有慢档那一条红：「抬到 11: NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot { abandoned_root: RollbackTarget { instance: InstanceGeneration(2), checkpoint_txg: CheckpointTxg(8) } })」（`out/k3-m11-reuse-slow.log` 原样，panic 在 `/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs:1116`） |

`out/k3.txt` 里红的那几行原样：

```
k3-m11-reuse-slow.log test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 15 filtered out
k3-m6-an_abandoned_roots_unreadable_account_is.log test result: FAILED. 12 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
k3-m6-rebuilding_the_allocator_rereads_the_roo.log test result: FAILED. 34 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
k3-m6-unreadable_abandoned_root_slot.log test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

读法：
- 这一批的 7 行变异（追加 4、替换 3）各自锚的那一处，实现员证过红；我没重证。上面三条是它们之外、这一批改动造出来的新判别点。
- M11 说明：这一批之后，抬 F 那一遍「从映射现算」只剩第 2–4 形（与 H1、H2 那几形）走得到；入库默认档里没有一条用例走到它成功的那一支，只剩慢档一条。变异表要钉它，得在那一行的参数里带 `--include-ignored`（第 47 行那种写法）。
- LR3 改完之后钉的是「影子账交 Err 时把分配器换回抬 F 之前那一份」（变异 2124 行，实现员重证红，我没重证），它在改前改后都把后一条根的树表与映射根两份抹零、映射现算本来就不成，所以「被抛弃根的账读不出时从中央映射现算」不是 LR3 钉的，改前也不是；M11 下 LR3 那一份 13 条照绿。
- 同一份文件里 LR2 的文档句「C 那两片实例表此刻只被 C 引用、在现行那一版的账里已释放，抬 F 的影子账从映射现算、两片都隔离，回收之后照样不空闲。」（`/home/fy5090/.local/share/singlefs/work/m3-batch10z-b-code-r1/tree/crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs:772`）这一批之后不再是实话：C 在挂载那一刻就从映射现算、记进表里，抬 F 照记着的那一份（M11 下 LR2 照绿即是）。它判别「现算只认实例表第 0 片」那一格照样拦得住，因为挂载那一遍用的是同一个函数。

## 四、打中之后的四句

| | H1（被抛弃根的根槽读坏，抬 F 那一串照拒） | H2（第 1 形 R 的账读坏，抬 F 那一串照拒） |
|---|---|---|
| 分不分辨臂 | 分辨：today 90 / 90 段拒，slot 臂 C 仍被抛弃时 0 / 90 段拒；两臂 checker 都 0 红。不分辨「这一批改前 / 改后」：改前同样拒，不是这一批带进来的 | 分辨：today 285 / 285 段拒，cring 0 / 285；两臂 0 红。B 那一组两臂都 0 拒 0 红，不分辨 |
| 系统当时看不看得到判别它的东西 | 看得到：根环表里那一槽记着 `AbandonedRoot`（实例代号、txg、落点），这个进程没写过那一槽；判被抛弃的那张表只会多抛弃不会少（K1-2） | 看得到：C 那一圈按槽记着 R 那一槽的落点；R 是候选根时挂载那一遍也读过它的账（候选豁免那一圈）。但 C 那一圈是**按槽**记的，分不出同一槽上差一整圈的另一次发布（推的，没造） |
| 满足的是判据字面的哪一分句 | D16 已定项 1「根槽这一次读坏」那一行前半「这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退、报它自己的错误成员」：照字面拒，**不违反**；末句「读不出也不拒」说的是账。打中的是代价（白拒），与用户 2026-10-07 定案要解的那件事同形 | 末句「不论住在哪一类槽，一律照挂载时算出、记在根环表里的落点算影子账，不再从盘上重读它的账，读不出也不拒」：R 住在有效根那一类槽，今天照读盘、照拒，**违反字面**（实现员已自报，留给主 agent） |
| 跑前条款给的改法在打中的格上还中不中 | 跑前没有条款给改法。实现员报告与正文都没点名这一形 | 正文给了两臂（今天 / C 那一圈按槽记）：今天那一臂在 A 那几格上照中；C 那一圈那一臂在 A、B 都不中（只在我的模型上量过、被攻过零轮） |

## 五、我提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 做什么 | H1 那几格 | H2 A 那几格 | H2 B 那几格 | 两条 checker |
|---|---|---|---|---|---|
| slot 臂 | 抬 F 那一串（重算影子账那一遍、准入算上限那一遍）里，根环表记成被抛弃根的槽不算「重读仍坏就拒」的槽；这一遍没读出它的，照表里记着的落点补进影子账 | 量过：0 / 90 段在 C 仍被抛弃时拒（`out/counts.txt` 第 X1 slot 那一行） | 推的：不碰（R 的槽读得出，拒在读账那一步） | 推的：不碰 | 量过：0 红（540 段） |
| cring 臂 | 抬 F 那一串里按现行实例表被抛弃、表里没记着落点的根，取 C 那一圈在它那一槽记着的落点 | 推的：不碰（拒在读根槽那一步，早于影子账） | 量过：0 / 285 拒 | 量过：0 / 345 拒，没量到掉出影子账 | 量过：0 红（2070 段） |
| 挂载那一刻有效根也记落点 | 挂载那一遍为候选豁免已读过每条有效根的账，记进根环表；写行之后它被抛弃时照用 | 推的：不碰 | 推的：不拒（挂载那一刻 R 的账读得出） | 推的：B 那一组 R 是读不出的候选根，记不到，落回读盘，同今天 | 推的，没实现 |
| 补用例 | M7：挂载时凭据放行、抬 F 时照空串算的一条用例；M11：抬 F 那一遍从映射现算成功的一条默认档用例，或在变异行里带 `--include-ignored` | — | — | — | 推的，没实现 |

slot 臂在准入算上限那一遍把被抛弃根的槽当「这一次读坏、不知道住着根」处置，照 D16 那一行前半当作可能是一条非空有效根、F 不越过它；这是我顺手取的，没量它让准入少推了多少。cring 臂的「按槽」在同一槽差一整圈的另一次发布上会认错（K1-2 同一类问题），没造。

## 没打中的形状

- K2-1 掉出影子账：X1 540 段（三种起点、故障三种、故障前覆盖写 0–8 次、五种用户动作含多次抬 F、管理员回退、写掉那一槽之后）与 F1 2070 段（覆盖写 3 / 5 / 8 次之后录 R、两种发布、R 写出的每个单元轮流藏、R 的账三种故障、五种用户动作），checker 0 红。一次抽样，照规则不记站住。
- K1-2 认错根：读码，没造发布失败退回那一支（推的）。
- K1-3 可写挂载那一道：读码加入库第三条用例在副本上绿。
- K2-3：`grep`，产品代码里没有调用方靠这个数。

## 这条腿自己的限度

- 只在 384 槽小盘一档上造；4 GiB 那一档、多片实例表、多层分配记录树都没造。「挂载那一刻部分读得出、抬 F 时读得全」那一形（K2-1 唯一推得出的缺口）在这档盘上造不出，H2 的 B 那一组也因此只量到「没掉出」。
- 用户动作放开扫的只有故障前覆盖写次数与五种动作序列；故障时机只在「故障前覆盖写 k 次之后」那一个点开，没在两次抬 F 之间开、没在回退之后开。
- 历史里没有崩溃点：两份探针只注读故障，没调 `enumerate_layer0` / `enumerate_layer0_versions`，层 0 状态数 0。H1、H2 都是「挂着时」的结局，没量它们崩了之后的下一次可写挂载。
- 每段历史只用 checker 的 I-7.4、I-4.8（F1 另加 I-7.2、I-2.1）判红，别的不变量没看。
- `rerun.sh` 本身没有从头空跑过：`out/` 里的产物是我在草稿目录里用同一组 `scripts/batch.sh`、`batch2.sh` 与规格分批跑的；`spec-b5.tsv`（X1 只扫两种动作的那一批，`out/x1b-*.log`）不在 `rerun.sh` 里，它被 `spec-b6.tsv` 那一批（全量）罩住。
- 臂与变异用环境变量切，同一份二进制里；臂的写法是原型，不是补丁。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳；副本上的数不是入库装置上的数。
- 重型测试一条没跑（checker 档、变异整表、门禁）。只跑了副本里的 harness 档测试二进制。
- 没重证这一批的 7 行变异与 LR3 那一行（实现员证过）。
- H1 的条款读法（前半句「仍坏就拒」管不管被抛弃根的槽）、H2 选哪一臂，交主 agent。

## 草稿目录

`~/.local/share/singlefs/work/m3-batch10z-b-code-r1-opus/`：仓副本 `repo/` 与编译目录 `target/` 交回前删掉；留 `out/`（原样日志）、`rc/`、规格与脚本（小）。
