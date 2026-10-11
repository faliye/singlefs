# m3-c694-r2 攻方腿（Opus）报告（2026-10-07）

轮名 m3-c694-r2；攻击面：正文第三节 K2（C 的具体形态）、K3（B 的凭据、恢复一侧补法）；立场照正文第四节「云端攻方」。
冻结副本 `~/.local/share/singlefs/work/m3-c694-r2/tree/`（`crates/singlefs-core/src/mount.rs`、`recovery.rs` 的 sha256 与快照 `research/prompts/m3-c694-r2-snapshot/crates-sha256.txt` 对上）。
全部数在我的副本上量（冻结副本 + 第一轮攻方补丁 + 本轮开关，由模型目录的 `make_patched.py` 现造；交回前副本与编译目录已删），不是入库装置上的数。

## 复跑与模型文件

模型目录 `research/prompts/m3-c694-r2-opus-model/`：`make_patched.py`（第一轮攻方补丁原样在前，本轮开关接在后面；每处替换要求旧串恰好命中一次）、`probe/`（原型二进制源码）、`rerun.sh`、`compare.sh`、`expected/`（两档的期望文件，都是在空目录里从头跑出来的）、`out/`（开发时的原样输出，报告里 `out/…` 指这里）。

复跑（仓根下；`SINGLEFS_REPO` 指冻结副本，`SINGLEFS_TOOLS` 指仓根、只取两个包装脚本；编译 6G、跑 4G，线程 `THREADS` 缺省 6）：

```
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-c694-r2/tree SINGLEFS_TOOLS=$PWD PROFILE=quick bash research/prompts/m3-c694-r2-opus-model/rerun.sh <空目录>   # 本机 50 秒
PROFILE=quick bash research/prompts/m3-c694-r2-opus-model/compare.sh <空目录>/summary.txt                                                  # 「全同（146 行）」
SINGLEFS_REPO=~/.local/share/singlefs/work/m3-c694-r2/tree SINGLEFS_TOOLS=$PWD PROFILE=full bash research/prompts/m3-c694-r2-opus-model/rerun.sh <空目录>    # 本机约 12 分钟，五次层 0 全量各约 100–115 秒
PROFILE=full bash research/prompts/m3-c694-r2-opus-model/compare.sh <空目录>/summary.txt                                                   # 「全同（932 行）」
```

每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里）：

```
882fe2a5434c008b3586ea0e54b59f531377a582639c67a2c06b672eace10920  ./compare.sh
75bb96d78b139fd819253ac98e25744224674c9a23cb7676be6f5b69344b45c2  ./expected/summary-full.txt
8cd3bcd816ab937c144cf0493ad672650263b47a16589437ef5834929dd6be77  ./expected/summary-quick.txt
44c483b7b83ff647ecc5b8a8ed57e7acfc59c4ccdb29d5b4f34754cc1f7c4b80  ./make_patched.py
1016b2b7a2fb8f829c0493ec948a8b1bbbb394c1e957e0b0b724d783ca14ede3  ./out/bcheck3-256-k16-C+B.log
b2aad223133f4413a409db469872a74da042974c65a4f459aae6fc2d3cf68bc5  ./out/bcheck3-256-k16-C+W.log
a10d6a379d9fef4ca1ea753f1ad0c8f7608d09cfc2c9a5aeb6c55b993e564ae9  ./out/bcheck3-256-k17-C+B.log
9da3a28b02916aab42a90f7d16c9f9a6d5dedbfc81188e29e9cf599b9341ab6c  ./out/bcheck3-256-k17-C+W.log
930cab2d4646da6b45ce45fab226bee9fcb21219f22ca495c42c67230db5063c  ./out/bcheck3-256-k18-C+B.log
59aa27f6af34943cb212061f679988563af07454bbb7b66b9f376d3ef9e7ad6d  ./out/bcheck3-256-k18-C+W.log
b95e44ce496eee5fc38fa73031f6927bde257ab3984328bd5e081ff598d5a0b2  ./out/bcheck3-256-k19-C+B.log
6c8f8c8644d406308c7dc8b1da0dae98c2eb95c04e9ff9a33a6523d6684b0ece  ./out/bcheck3-256-k19-C+W.log
08a98cd2bd6a3531cdfb25f0b6b0053755eca33146ab61b0dd79f127585993bb  ./out/cc-named.txt
98a2de5f980e781b720b640ca12bf8e94f2b07c0e2663eba4540e16cc3f3bc1b  ./out/cm-dev-A1-C-all-urL.log
7a22255957d857dbb66930c38fba239ba4f89fc63266b0054fc41eb3c15e699c  ./out/cm-dev-A2-C-last-urL.log
2fb51c062b3682defa5be0776289f9eecd9a72097e98c73337e7429617be5864  ./out/cm-dev-B1-today-strict-ow-ur.log
b579aaddd83026113c14b64518f406fae625008f64008066be3d9b7d36a1ad66  ./out/cm-dev-B2-today-strict-ow-urJ.log
8c97ec29f6d52e13b8b579957233e0fae7fd0fb005ffec2486e4587fc31536b1  ./out/cm-dev-smoke-cm.log
7e14150f8392ce6b2185cc4a0d74a21e95c79153222d0339323afdbf3a150069  ./out/cm-summary.txt
ee7d2b316090cf0cc0d28c821567dc9a3ac2e1d84ca4b9e2fe6f721fff45aff9  ./out/grid-sw3-ur-C-ring.log
973c858a89217deeb87ae39ce19690b05cf8221f654001c250ada52da2f49259  ./out/grid-sw3-urF-C-ring.log
1856fd46a831c321fe490fb9ad01b23b8a86ac2a4a32fee779011f643a768015  ./out/grid-sw3-urL-C-ring.log
5b5c5ef50ff3c8cfabd466f6b89654d76b7fb91347ce9fc1c81740cdd5b988ac  ./out/grid-sw3-urM-C-ring.log
7c3b73f0adeba28a89ea48e8cfe46c4f752fea7c8c6a997696ee8c5c5c00f2f1  ./out/grid-sw3-urM-C-ring-loose.log
adab59259e20eec58d4d991ab717335742ea5e25fc0e5a434585578a664d2add  ./out/grid-sw3-urM-C-ring-strict.log
fe1d4f7e484bb6d1972dd57990e0bd68c799c76cdb4cfe68f90c2d705c06f78e  ./out/grid-sw3-ur-today-mount.log
5c81eb8d19184aea6a9b89356857bca813bb3a18c2d477908fcfa128277c697a  ./out/narrow2.txt
ec4b2922e7f9e72fd15e4995216dc90016f1c2f1d977eb8921804dbce8c98dca  ./out/narrow.txt
1d0d34f90e26a13ab1f0a5a03a9f1f15d342b0be47f00aeb068d360a02b07ae5  ./out/r1probe-crashmount-384-5-C.log
9471ab80075d2dfb627709a5d312697b4c3082c7cdfa66475a23f273e98f4913  ./out/r1probe-crashmount-384-5-today.log
be9d937b8637679badafaae2895d975074bf168844b955197770d6df3d0d6738  ./out/silent.txt
586d621cccfaaf8caf7f84493c4959ca33ffb8f76fdea006250ec286a968523d  ./out/stalecheck.txt
b43dff9b8394a9a465ab7f3bb6d7f88bbcf227fd9e34f45ec47c28c50ab82c45  ./probe/Cargo.toml
8dc1f96cb3a89dc9fbdce730b8c351d1bcd732aff5aa60816ff07c40da0edd4a  ./probe/main.rs
881a7eb23571c42126ae37661157aa98c3d72b1a06f8e4b9e30c06b7abc22c73  ./rerun.sh
```

## 各格判定一览

| 格 | 形 | 判定 |
|---|---|---|
| K2 什么时候解开 | C 照 D16 已定项 1 读回全零那一行的时机解（进程内重读读出 R 是被抛弃的根就解） | **打中**（放错）：比 R 被挤出根环早一次发布，那次发布写掉 R 的 2 个落点，I-7.4 红、R 读不出（第一节；只在一个整段前缀状态上量，那次发布层 0 全量超过 10⁶ 没跑） |
| K2 什么时候解开 | 只在这个进程写掉 R 那一槽时解（`ring`）／这次挂载内不解（`mount`） | 没打中（第一节、「没打中的形状」） |
| K2 隔离哪些落点 | 只认带末条标志的记录（`last`） | **打中**：R 跨三条记录、末条读不出时整次发布看不见，层 0 全量 14,205 / 20,516 个状态读不出，与今天同形，写行之后再可写挂载又被 C694 那一处拒（2.1） |
| K2 隔离哪些落点 | 水位之上全部（`all`，第一轮 C） | **打中**：R 的任意一条记录读不出时那条记录点名的单元没人护，M1 之后第一次覆盖写就写掉，R 留在环里 24 次发布，I-7.4 红，卸载不拒、B 无从放行（2.2；判别子观测不到） |
| K2 窄盘代价 | 会话写被拒的那一刻 | 没打中「无界」：多隔离 22 + 4N 槽（N = 那次发布的数据单元数），到这次挂载结束或 R 的槽被写掉；再挂之后与今天同额（第三节） |
| K3 B 的凭据 | 第一轮正推甲 W1（读那一槽、诞生身份不同就判占） | **打中**（限定：瞬时故障是「读回写之前的旧内容」）：256 槽 k=16–19 四个起点上把活着的 R 判成已被占，之后 R 的单元被写掉、I-7.4 红（4.1） |
| K3 B 的凭据 | 第一轮攻方 B（看现行账） | 没打中（4.2） |
| K3 恢复补法 | K1-2、藏记录两形上的结局 | 恢复退到上一版读得出（13,471 / 13,476 个状态，层 0 全量 0 违例）；只改 `recover` 时可写挂载与只读挂载在这些状态上照旧失败（5.2） |
| K3 恢复补法 | 会不会悄悄读回旧版 | **打中**：V 已确认，较新的系统配置槽与 V 的单元读回全 0（S1），`strict` / `loose` 都读回上一版；V 的末条与单元读回全 0（S2）只 `loose` 中（5.3） |
| K3 恢复补法 | 2.2 那一形 | 救不了：计数器全池接着走，R 被算成见证过（5.2 末条） |

## 一、K2：C 什么时候解开（与 D16 已定项 1「读回全零的槽」的钉法怎么并）

**打中一形（C 照 D16 的解开时机解开 = 放错）。**

起点：384 槽、第一个文件之后覆盖写 5 次，录一次覆盖写 R =（实例 1，txg 9）；崩在 R 的根槽落盘那一刀（系统配置没轮换）。
取新实例那次挂载 M1 时 R 的单元与根槽读回全 0、R 的记录读得出（第一轮 K1-2 那一形，hide=`ur`）。C 隔离 R 那条记录点名的 24 个落点。
M1 里 R 的根槽 (0,3) 被记成「挂载时读回全零」、钉在 txg 9（跟踪行 `OPUS_MOUNT_PINS all_zero={RootRingSlot { region: 0, slot: 3 }: CheckpointTxg(9)} bad={}`）。
之后用户照常覆盖写。

三种解开时机（`OPUS_C_RELEASE`）：
- `mount`（第一轮 C）：这次挂载内不解。
- `ring`：这个进程写 R 那条记录的 txg 对应的根环槽时解（在 `record_root_written_by_this_process` 里，那次发布的落点已取完之后）。
- `d16`：`ring` 之外，照 D16 已定项 1 那一行给读回全零的槽的解开时机也解——「这个进程写掉那一槽、或进程内记根时重读读出有效根或被抛弃的根才解开，重读读回全零照旧钉着」（`.claude/kb/decisions/16-发布语义.md:37`）。

走法：第 18 次覆盖写记 txg 32 的根时，R 的槽成了环里最小的那个，重读读出 R、按实例表被抛弃
（跟踪行 `OPUS_D16_REREAD ring_slot=(0,3) txg_recorded=32 outcome=AbandonedByTheInstanceTable`）；
今天的代码在这一处把那一槽当没有根：「occupancy.occupants.remove(&ring_slot);」（`/home/fy5090/.local/share/singlefs/work/m3-c694-r2/tree/crates/singlefs-core/src/allocator.rs:2121`）。
`d16` 在这一刻解开 C 的 24 个落点；R 的根槽要到下一次发布（txg 33，同一个槽 (0,3)）的根槽写才被盖掉。第 19 次覆盖写（或顺序写）取落点时就取到 R 的单元，
单元写在根槽写之前，这之间 R 还在环里。那一次发布的写流层 0 全量 16,777,223 个状态（顺序写那一次 268,435,475 个），都超过约 10⁶，按派发规则不跑全量；
只报层 0 枚举域里那一个整段前缀状态（根槽写之前的写全部持久、根槽与之后一个没持久），原样输出：

```
CC W=384 k=5 R=ow hide=ur arm=C/all/d16/none ops=o^18 last=o SKIPPED closed_form=16777223 named_state=before_root_write in_ring=true dmg=2 chk=[I-7.4] probe=onR:FAILED(MappingStillUnreadable)
CC W=384 k=5 R=ow hide=ur arm=C/all/d16/none ops=o^18 last=s SKIPPED closed_form=268435475 named_state=before_root_write in_ring=true dmg=2 chk=[I-7.4] probe=onR:FAILED(MappingStillUnreadable)
CC W=384 k=5 R=ow hide=ur arm=C/all/ring/none ops=o^18 last=o SKIPPED closed_form=16777223 named_state=before_root_write in_ring=true dmg=0 chk=[] probe=onR:read-ok
CC W=384 k=5 R=ow hide=ur arm=C/all/ring/none ops=o^18 last=s SKIPPED closed_form=268435475 named_state=before_root_write in_ring=true dmg=0 chk=[] probe=onR:read-ok
CC W=384 k=5 R=ow hide=ur arm=C/all/mount/none ops=o^18 last=o SKIPPED closed_form=16777223 named_state=before_root_write in_ring=true dmg=0 chk=[] probe=onR:read-ok
CC W=384 k=5 R=ow hide=ur arm=C/all/mount/none ops=o^18 last=s SKIPPED closed_form=268435475 named_state=before_root_write in_ring=true dmg=0 chk=[] probe=onR:read-ok
```

（`dmg` 是 R 的账里仍分配着的 22 个落点在这一刻被改过几个；`probe` 是把实例代号高于 R 的根槽全写零再恢复：落到 R 上读不读得出，第一轮 H1 的同一个探针。）

四句：
1. **分辨臂**：分辨。同一个前缀、同一个故障、同一串用户动作，`d16` 红、`ring` 与 `mount` 不红；它只打「C 照 D16 读回全零那一行的解开时机解」这一形。
2. **系统看不看得到**：看得到。重读那一刻读出的正是 R 这条根（读得出、自证过、按实例表被抛弃），R 的落点就在它的账里；今天的代码读出来之后把它当没有根丢掉，不是看不到。
3. **满足判据哪一句**：D23 已定项 14 主句「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」（`.claude/kb/decisions/23-journal的角色与格式.md:390`；池级 checker 的 I-7.4 就判这一句，上面那一行它红）；正文第五节「一形放错……该钉的放了」。
4. **改法还中不中**：把解开时机限在「这个进程写掉 R 那一槽」（`ring`，在那次发布取完落点之后解）在这一格上不中（上面第 3、4 行）；`mount` 也不中但有代价（第二节）。
   另一个改法（推的，没实现）：重读读出被抛弃的根时不当没有根，改记成被抛弃的根、把它的账并进影子账，C 的隔离再交给影子账；账读不出时又回到 C694 那一处拒，没解决只是挪地方。

同一处对今天的代码的含义（推的，没单独量）：D16 那一行给读回全零的槽的回收门槛钉，在「重读读出被抛弃的根」时同样解开，R 引用、被新实例释放过的那些槽（比如 R 照抄的上一版的实例表）从这一刻起可回收，窗口同样是一次发布。上面 `ring` 那一行 `dmg=0`，这条链上没撞上。

## 二、K2：C 隔离哪些落点（只认带末条标志的记录？水位之上全部？点名项读不出时）

R 换成跨三条记录的一次发布：顺序写 3 个数据单元（`publish_sequential_write`，前两条记录各点名一个数据单元，末条点名第三个数据单元与全部元数据单元，
每条记录都带整次发布的新根段四个指针）。前缀与故障同第一节，另把 R 的某一条记录也读回全 0：`urL` 末条、`urM` 中间一条、`urF` 第一条、`ur` 一条不藏（对照）。
两种选法（`OPUS_C_SELECT`）：`all` = 第一轮 C（水位之上、没施加的全部记录），`last` = 只认「这次发布带末条标志的那一条读得出」的发布。

### 2.1 只认带末条标志的记录：末条读不出时整次发布看不见，K1-2 原样回来（打中 `last`）

M1 的写流层 0 全量枚举（每一段都展开，20,516 个状态），撤故障之后恢复，原样输出：

```
L0TALLY CM W=384 k=5 R=sw3 hide=urL arm=C/all/mount/none states=20516 root_persisted=4116 violations=0 failed=0 file_read=20516 checker_violated=[]
L0TALLY CM W=384 k=5 R=sw3 hide=urL arm=C/last/mount/none states=20516 root_persisted=4116 violations=14205 failed=14205 file_read=6311 checker_violated=[I-2.1=16391,I-3.1=1344,I-3.10=6911,I-3.11=1344,I-4.8=16391,I-7.2=14205,I-7.4=20507]
CMCLASS W=384 k=5 R=sw3 hide=urL arm=C/last/mount/none row_root_persisted=false onR:FAILED(MappingStillUnreadable) states=10101
CMCLASS W=384 k=5 R=sw3 hide=urL arm=C/last/mount/none row_root_persisted=false onR:FAILED(UnitUnreadable) states=4104
CMMOUNT W=384 k=5 R=sw3 hide=urL arm=C/last/mount/none first_state_of[row_root_persisted=true landed=Some((2, 10)) failed=false] writable=Err(StillUnreadable:AccountOfAnAbandonedRoot) read_only=Ok(inodes=1)
```

`last` 在末条读不出时一个落点都不隔离，等于今天那一臂（今天那一臂同前缀、藏中间一条量的是 `violations=14200`，失败类同形，`out/cm-dev-smoke-cm.log`；今天那一臂藏末条没单独跑），写行根落盘之后的状态上再可写挂载又被 C694 那一处拒。
`all` 在同一个藏法上 0 失败：M1 那几次发布只碰得到 R 的提交内生单元，它们的落点在每一条记录的新根段里都有。

### 2.2 水位之上全部（`all`）：R 的一条记录读不出时，那条记录点名的单元没人护，之后的用户写把它写掉（打中 `all` 与 `last`）

M1 之后用户动作放开扫（字母表 `onsu`：覆盖写、建一个 inode、顺序写两单元、删第一个文件；长度 1–3 的每一串，73 个结点），解开时机取 `ring`。每个结点在拷贝上卸载、跑池级 checker、探 R：

```
GTALLY W=384 k=5 R=sw3 hide=urL arm=C/all/ring/none chk_I-7.4=48 nodes=73 op_refused=15 probe_failed=48
GTALLY W=384 k=5 R=sw3 hide=urM arm=C/all/ring/none chk_I-7.4=48 nodes=73 op_refused=15 probe_failed=48
GTALLY W=384 k=5 R=sw3 hide=urF arm=C/all/ring/none chk_I-7.4=48 nodes=73 op_refused=15 probe_failed=48
GTALLY W=384 k=5 R=sw3 hide=ur arm=C/all/ring/none nodes=73 op_refused=15
G W=384 k=5 R=sw3 hide=urM arm=C/all/ring/none seq=o in_ring=true dmg=2/28 iso=30 free=2162 unmount=Ok(2) in_ring_after_unmount=true chk=[I-7.4] probe=onR:FAILED(MappingStillUnreadable)
```

（`seq=o`：M1 之后第一次覆盖写就写掉 R 的 2 个落点——藏起来那条记录点名的数据单元在两块盘上的两份；`op_refused=15` 都是删掉文件之后再覆盖写或顺序写的 `NoFileVersionToChange`。）
被藏那条记录的落点只有那条记录知道：`urM`、`urF` 下 R 的末条读得出，`last` 与 `all` 隔离同一组，同样漏。和第一轮「记录也读回全零」那一形比有三处不同：
- R 不被写行那次发布挤出根环：读得出的记录带着 R 的 txg 9，新实例从 txg 10 起（第一轮那一形新实例取到 txg 9、写行的根槽写盖掉 R），所以 R 带着被写掉的单元在环里一直留到环转一圈（384 槽这一档每区 8 槽、三区，24 次发布）。
- 卸载不拒：R 的账（树表、分配记录树）落在末条与新根段里，读得出，影子账照常算完，C694 那一处一次都不走到——B 没有可放的拒，第一轮给 C 的盲区留的那条凭据放行的路在这一形上用不上。
- 坏的是 R 本身而不是恢复：池级 checker 的 I-7.4 在卸载之后的盘面上红，与有没有崩溃无关。

四句：分辨臂——不分辨 `all` 与 `last`（两臂同红），分辨 C 与「不该隔离」；被判的系统看不看得到——看不到：读不出的那条记录点名哪几个单元，挂载那一刻没有任何别的东西记着（树表、映射在 R 的末条与单元里，单元也读回全 0），属「判别子观测不到」；判据分句——D23 已定项 14 主句（同第一节第 3 句）；改法——`ring` / `mount` 两种解开时机都碰不到这一格（隔离集本身漏了）。
可能的收严（推的，没实现，被攻过零轮）：水位之上有一次发布读得出的记录本次发布内序号不连续或末条读不出、而那几条点名的单元又验不过时，按 C554 乙的样子重读一次，仍这样就拒可写。它对「崩在写记录中途」那种正常崩溃不拒的前提是那时点名的单元验得过（单元在记录之前落盘），这一条我没量。

## 三、K2：窄盘上多隔离的代价——取样到会话写被拒的那一刻（没打中「无界」）

前缀同第一节（`ur`，R 的记录读得出），R 取覆盖写或顺序写 N 个数据单元；M1 之后一直覆盖写到被拒（至多 40 次），被拒之后在拷贝上卸载、再可写挂载、覆盖写一次。
240 槽、256 槽两档 × k ∈ {0, 3, 6} × R ∈ {ow, sw2, sw4, sw8, sw16} × 臂 {today, C/mount, C/ring}，90 行（`out/narrow.txt`）；240 槽 k=0 另加 sw20–sw32（`out/narrow2.txt`）。

240 槽 k=3 这一排（M1 之后隔离的槽数 / 第几次覆盖写被拒，取自 `out/narrow.txt` 各行的 `isolated=` 与 `refused_at=`）：

| R | today | C/mount | C/ring |
|---|---|---|---|
| ow | 0 / 10 | 24 / 7 | 24 / 7 |
| sw2 | 0 / 10 | 30 / 6 | 30 / 6 |
| sw4 | 0 / 10 | 38 / 4 | 38 / 4 |
| sw8 | 0 / 8 | 54 / 4 | 54 / 4 |
| sw16 | 0 / 6 | 86 / 1 | 86 / 1 |

原样（sw16 那三行，带被拒之后的卸载与再挂）：

```
NARROW W=240 k=3 R=sw16 hide=ur arm=today/all/mount/none r_records=16 M1[effective=(1,6) row_txg=8 isolated=0 free=1816] refused_at=Some((6, "NoSpaceAfterRaisingTheFloor")) isolated_after_each=[0,0,0,0,74] unmount=Ok(3) remount=Ok(isolated=90) write=ok
NARROW W=240 k=3 R=sw16 hide=ur arm=C/all/mount/none r_records=16 M1[effective=(1,6) row_txg=8 isolated=86 free=1730] refused_at=Some((1, "NoSpaceAfterRaisingTheFloor")) isolated_after_each=[] unmount=Ok(3) remount=Ok(isolated=90) write=ok
NARROW W=240 k=3 R=sw16 hide=ur arm=C/all/ring/none r_records=16 M1[effective=(1,6) row_txg=8 isolated=86 free=1730] refused_at=Some((1, "NoSpaceAfterRaisingTheFloor")) isolated_after_each=[] unmount=Ok(3) remount=Ok(isolated=90) write=ok
```

读法：
- **有界，随那次没施加的发布的大小线性长**：多隔离的槽数在这几档上是 22 + 4N（N = R 写的数据单元数；覆盖写 24），与 k、盘宽无关（`out/narrow.txt` 里 C 臂的 `isolated=` 只取 24、30、38、54、86 五个值）；上限是一次发布点名得下的落点数（记录条数上限 × 每条 64 项）。
- **持续到哪一刻**：`mount` 到这次挂载结束；`ring` 到 R 那个根槽被这个进程写掉（这一档 24 次发布），之后剩下的 4 个是别的隔离（256 槽那几行 `ring` 的最后一个读数回到 4）。
- **不是 C 自己新造的代价**：被拒之后卸载、再挂，今天那一臂与 C 那两臂再挂之后都隔离 90 个槽（R 这时读得出，按被抛弃的根进影子账），C 只是把这笔隔离从下一次挂载提前到 M1；今天那一臂在 M1 里不隔离，正是 K1-2 丢数据的那段。
- 「假 ENOSPC 无界」没找到：被拒都在这一次挂载之内，卸载、再挂之后写得进（`write=ok`），240 槽 k=0 sw24 起再挂之后今天那一臂也写不进（`out/narrow2.txt`），与 C 无关。
- 盘满得更早的那一格：240 槽 k=3 sw16、k=6 sw4 / sw8，C 两臂 M1 之后第一次覆盖写就被拒，今天那一臂分别第 6、3、3 次。

## 四、K3：B 的凭据——瞬时读不出与已被占分不分得开

两种凭据都在副本里：第一轮攻方 B（`OPUS_C694_ARM` 含 `B`：被抛弃根的树表每个位置条目，在现行账里都有一条罩住它、代大于那条根 txg 的记录）、第一轮正推甲 W1（含 `W`，照抄：树表与映射根两个指针，每个位置条目都读出诞生身份不同的合法码 2 节点）。
装置（`bcheck`）：每个结点先在拷贝上正常卸载，再可写挂载一次，这次挂载里 R 的树表、映射根、分配记录树根所在的槽瞬时坏（两种：`fail` 读报错；`stale` 读回 R 写它之前那一槽的旧内容），
看 B / W 跳没跳过 R；跳了就撤故障覆盖写 3 次、卸载、探 R。`bc_alive` 是无故障盘面上 R 的账读不读得出（读得出 = R 活着，跳过就是误放，记 `B_MISJUDGE`）。

### 4.1 W1（读出来判占）在「读回旧内容」下把活着的 R 判成已被占（打中 W1，记号见下面四句的限定）

先找 R 写之前那两槽都住着别的合法码 2 节点的起点（`stalecheck`，四档盘宽 × k = 0–19，80 个起点里 4 个：256 槽 k = 16–19，`out/stalecheck.txt`）。在这 4 个起点上（C 开着，R 的单元在 M1 没被写掉）：

```
k=16 C+W b_misjudge=2 b_skip=2 chk_I-7.4=2 nodes=2 probe_failed=2
bc_alive=true | bc_fail:skips=0 mount=Err(StillUnreadable:AccountOfAnAbandonedRoot) | bc_stale:skips=1 B_MISJUDGE mount=Ok then=[ok,ok,ok] unmount=Ok(3) probe=onR:FAILED(UnitUnreadable) chk=[I-7.4]
k=16 C+B nodes=2
bc_alive=true | bc_fail:skips=0 mount=Err(StillUnreadable:AccountOfAnAbandonedRoot) | bc_stale:skips=0 mount=Err(StillUnreadable:AccountOfAnAbandonedRoot)
```

（k = 17、18、19 三个起点逐字同形，`out/bcheck3-256-k1*-C+*.log`。）W1 跳过之后这次挂载不隔离 R 引用的单元，三次覆盖写之后 R 还在环里、单元被写掉，I-7.4 红、R 读不出；B 在同一个盘面上照拒。

四句：
1. 分辨臂：分辨 W1 与 B（同一盘面 W1 误放、B 照拒），不分辨 C 开不开（R 的单元是这次挂载之后才被写掉的）。
2. 被判的系统看不看得到：W1 只看那一槽读出来的字节，「读回写之前的旧内容」与「真被后来的单元占了」在那一槽上逐字节相同（旧内容本来就是一个合法的、诞生身份不同的节点）——对 W1 是判别子观测不到；B 看的是现行账，现行这条时间线没分配过那一槽，B 看得到。
3. 判据分句：正文 K3 第一条「造一形把瞬时读不出的盘面判成已被占（误放），就算打中」——限定：这里的瞬时故障是「读回旧内容」，不是读报错也不是读回全零；读报错（`fail`）与读回全零下 W1 都照拒（上面 `bc_fail` 那一格，第一轮正推甲的写法本来就把这两样排除）。这一类故障在不在本项目的故障模型里，交主 agent 判；D16 已定项 1 只写了「写失败的槽按旧内容算」。
4. 改法：B（看账）在这一格上不中。

### 4.2 B（看账）没打中

试过的形状：384 槽 k ∈ {5, 10, 30}（k=20 前缀那次覆盖写自己被准入拒了，没起来）与 256 槽 k = 16–19，`fail` 与 `stale` 两种瞬时故障，臂 B 与 C+B，M1 之后 0–2 步用户动作（`osn`）。
C 开着时 R 活着（`bc_alive=true`），B 一次都没跳（`skips=0`，全部照拒）；C 关着时 R 在 M1 已经被写掉、账读不出（`bc_alive=false`），B 跳过是对的。
不跳的道理（读代码）：B 的凭据来自内存里现行这一版的账，不读 R 那一槽；现行账里有一条代大于 R 的 txg 的记录罩住 R 的树表槽，说明这条时间线在 R 之后真写过那一槽（单元在根槽之前落盘），瞬时故障造不出这条记录。
B 的漏（不是误放）：「藏记录」那一形里新实例从 R 的 txg 起（第一轮），写行那次发布的单元代等于 R 的 txg，B 要的「大于」不成立；那一形里 R 被写行的根槽写挤出根环，不会走到拒，所以没有后果。

## 五、K3：恢复一侧补法（所选根走不通、系统配置没见证它时退回见证到的那一版）

### 5.1 写成什么样、改在哪

副本里只改了 `recover` 一处（`OPUS_RECOVERY_FALLBACK`，`make_patched.py` 末段），接在走读那一步之后：「let outcome = match walk_to_file_in_the_unit_area_starting_at(」（`/home/fy5090/.local/share/singlefs/work/m3-c694-r2/tree/crates/singlefs-core/src/recovery.rs:5218`）。
- 触发：走读失败。c_见证 = 每块盘两槽里自证过的系统配置的 journal tail 最大值（与可写挂载的判据 N-配置 同一个读数）。
- 所选那一版算「见证过」：它那次发布带末条标志的记录读得出、计数器 ≤ c_见证；末条读不出时拿计数器 = c_见证 的那条记录比 (txg, 实例)，不比所选那一版旧就算见证过；两条都读不出时 `strict` 当见证过（不退）、`loose` 当没见证（退）。
- 退到：根环里比所选根旧、(txg, 实例) 不超过 c_见证 那条记录的最新一条根，不重放、直接走读；走不通照报原来的失败。

要真起作用还得改另外两处（推的，没实现）：可写挂载读阶段之后建上一版那一步「let previous = rebuild_previous_version(devices, unit_area_start, &effective_root, own_record)?;」（`/home/fy5090/.local/share/singlefs/work/m3-c694-r2/tree/crates/singlefs-core/src/mount.rs:6068`）一带，与只读挂载的择根（`crates/singlefs-core/src/mounted_read.rs` 的 `mount_read_only`）。只改 `recover` 时，下面「退回」的那些状态上两条挂载路径照旧失败（`CMMOUNT` 行）。

### 5.2 在 K1-2 与「藏掉 journal 记录」两形上的结局（层 0 全量，每一段都展开）

```
L0TALLY CM W=384 k=5 R=ow hide=ur arm=today/all/mount/strict states=20511 root_persisted=4116 violations=0 failed=0 file_read=20511 checker_violated=[I-2.1=16383,I-3.1=1008,I-3.10=5180,I-3.11=1008,I-4.8=16383,I-7.2=13471,I-7.4=20499,I-9.1=6144]
CMCLASS W=384 k=5 R=ow hide=ur arm=today/all/mount/strict row_root_persisted=false landed=Some((1, 8)):read-ok states=13471
CMMOUNT W=384 k=5 R=ow hide=ur arm=today/all/mount/strict first_state_of[row_root_persisted=false landed=Some((1, 8)) failed=false] writable=Err(Recovery) read_only=Err(Open)
L0TALLY CM W=384 k=5 R=ow hide=urJ arm=today/all/mount/strict states=20516 root_persisted=4116 violations=0 failed=0 file_read=20516 checker_violated=[I-2.1=16388,I-3.1=1008,I-3.11=1008,I-4.8=16388,I-7.2=13476,I-7.4=16388,I-9.1=6144]
CMCLASS W=384 k=5 R=ow hide=urJ arm=today/all/mount/strict row_root_persisted=false landed=Some((1, 8)):read-ok states=13476
CMMOUNT W=384 k=5 R=ow hide=urJ arm=today/all/mount/strict first_state_of[row_root_persisted=false landed=Some((1, 8)) failed=false] writable=Err(Recovery) read_only=Err(Open)
```

- K1-2（`ur`）：今天 13,471 个状态走读失败（补法关着时的数：本轮用第一轮攻方原型 `crashmount 384 5`、`OPUS_FULL=1` 全量复跑，模型目录 `out/r1probe-crashmount-384-5-today.log` 第 155 行 L0TALLY 那一行；C 臂同法 `violations=0`），补法开着这 13,471 个全部退到 (1,8) 读出上一版，oracle 0 违例。
- 藏记录（`urJ`，第一轮 C 的盲区）：13,476 个状态同样全部退到 (1,8)；第一轮报的 9 个是缩过的层 0（多于 12 个写的段不展开），全量下是 13,476 个。
- 两形上 `loose` 与 `strict` 不分：这些状态里 R 的末条或 c_见证 那一条总有一条读得出（`strict` 一次都没走到「两条都读不出」那一支）。
- 池级 checker 的红照旧（I-7.4、I-7.2 等）：补法只改恢复读哪一版，盘上 R 的单元已经被写掉；`CMMOUNT` 两行说明只改 `recover` 时可写挂载与只读挂载在这些状态上仍然失败。
- C 盲区里「一条记录读不出」那一形（第二节 2.2）补法也救不了：之后的探针落到 R 上时，新实例后来的发布已把系统配置的 tail 推过 R 那一条，按计数器比 R 算「见证过」，不退（`GTALLY W=384 k=5 R=sw3 hide=urM arm=C/all/ring/strict chk_I-7.4=48 nodes=73 op_refused=15 probe_failed=48`，`loose` 同数）。计数器全池接着走，「c_见证 ≥ R 的计数器」分不开「见证过 R」与「见证过别的实例后来的一次发布」。

### 5.3 让本该报读不出的盘面悄悄读回旧版（打中两种写法；`loose` 多一形）

起点：k 次覆盖写之后经写入口再发一次覆盖写 V（整次发布落盘，系统配置已轮换、末尾屏障已过：V 已确认）。恢复时三样读回全 0 的组合：

```
SILENT W=384 k=5 arm=today/all/mount/none shape=S1 v=(1,9) landed=Some((1, 9)) outcome=FAILED(UnitUnreadable)
SILENT W=384 k=5 arm=today/all/mount/strict shape=S1 v=(1,9) landed=Some((1, 8)) outcome=read-OLDER(P)
SILENT W=384 k=5 arm=today/all/mount/strict shape=S1c v=(1,9) landed=Some((1, 9)) outcome=FAILED(UnitUnreadable)
SILENT W=384 k=5 arm=today/all/mount/strict shape=S2 v=(1,9) landed=Some((1, 9)) outcome=FAILED(UnitUnreadable)
SILENT W=384 k=5 arm=today/all/mount/loose shape=S2 v=(1,9) landed=Some((1, 8)) outcome=read-OLDER(P)
SILENT W=384 k=5 arm=today/all/mount/strict shape=S3 v=(1,9) landed=Some((1, 8)) outcome=read-OLDER(P)
```

（S1：V 的单元与每块盘世代号大的那一个系统配置槽读回全 0；S1c：只藏 V 的单元；S2：V 的单元与 V 的记录；S3：崩在 V 的根槽落盘那一刀、V 没确认，V 的单元读回全 0。256 槽 k=3 逐形同结局，`out/silent.txt`。）

- S1：V 已经返回给调用方，只因为两块盘上较新的那个系统配置槽这一次读回全 0，c_见证 落回 V 之前那一次；V 的单元这一次也读不出。今天报读不出（调用方可以重试、或拒），补法悄悄读回上一版 P。`strict` 与 `loose` 都中。
- S2：只有 `loose` 中（末条与 c_见证 那一条都读不出时当没见证）。
- S3：V 没确认，退回 P 合语义；但同一形放在层 0 的 oracle 下（V 的根槽是被判流里已持久的最新根）就是「根槽已持久而恢复到旧态」，从「走读失败」换成另一类违例，不是修好。

四句（S1）：分辨臂——分辨补法开与关；被判的系统看不看得到——看不到：它只看得到读出来的系统配置槽，「较新那一槽这一次读回全 0」与「较新那一轮根本没轮换」在盘面读数上相同，属判别子观测不到；判据分句——正文 K3 第二条「会不会让一个本该报读不出的盘面悄悄读回旧版（举一形）」；改法——把「两条都读不出」当见证过（`strict`）对 S1 不起作用，S1 要的是「系统配置槽读回全 0」也按判不出处置（今天的 R2 判不出只认设备报读错，`recovery::system_configuration_witness_is_undecidable`），那样 K1-2 的状态里只要有一个系统配置槽读回全 0 就退不回去。补法若照 5.1 再放进可写挂载，S1 就不只是读回旧版：新实例在 P 上取号，V 被抛弃（推的，没量）。

## 六、几个改法各修哪一格（全部只在我的模型上量过、被攻过零轮）

「量过」= 副本上原样输出在上文那一节；「推的」= 按代码推，没实现、没跑。

| 打中的格 | 改法 | 这一格上还中不中 |
|---|---|---|
| 第一节：C 照 D16 读回全零那一行的时机解开，R 还在环里时单元被写掉 | C 只在「这个进程写掉 R 那一槽」时解（`ring`） | 不中，量过（第一节第 3、4 行输出） |
| 同上 | 这次挂载内不解（`mount`） | 不中，量过；代价见第三节 |
| 同上 | 重读读出被抛弃的根时改记成被抛弃的根、账并进影子账 | 推的；账读不出时回到 C694 那一处拒 |
| 2.1：末条读不出时 `last` 整次发布看不见 | 改认水位之上全部（`all`） | 不中，量过（`violations=0`） |
| 2.2：R 的一条记录读不出，那条记录点名的单元没人护 | `all`、`last`、`ring`、`mount` | 都中，量过（三种藏法各 48 / 73 结点 I-7.4 红） |
| 同上 | 水位之上一次发布的记录序号不连续或末条读不出、点名单元又验不过时重读一次仍这样就拒可写 | 推的 |
| 同上 | 恢复一侧补法（`strict` / `loose`） | 都中，量过（5.2 末条） |
| 4.1：W1 在「读回旧内容」下把活着的 R 判成已被占 | 凭据改看现行账（第一轮攻方 B） | 不中，量过（`C+B` 照拒） |
| 5.3 S2：V 已确认，V 的末条与单元读回全 0，悄悄读回旧版 | 两条都读不出当见证过（`strict`） | 不中，量过 |
| 5.3 S1：较新的系统配置槽与 V 的单元读回全 0，悄悄读回旧版 | `strict` | 仍中，量过 |
| 同上 | 系统配置槽读回全 0 也按「见证判不出」处置 | 推的；代价是 K1-2 状态里只要有一个系统配置槽读回全 0 就退不回去 |
| 5.2：只改 `recover`，可写挂载与只读挂载照旧失败 | 同一套退回逻辑放进可写挂载读阶段与只读挂载择根 | 推的；放进可写挂载之后 S1 从「读回旧版」变成「抛弃已确认的 V」 |

## 没打中的形状

- 「假 ENOSPC 无界」：第三节 100 行窄盘配置（240 / 256 槽 × k ∈ {0,3,6} × R ∈ {ow, sw2, sw4, sw8, sw16}，外加 240 槽 k=0 sw20–sw32）× 三臂，被拒都在这一次挂载内，卸载再挂之后与今天那一臂同样隔离 R（90 槽那一组），C 不留跨挂载的代价。只扫了覆盖写一种用户动作。
- `ring` / `mount` 两种解开时机放错：K1-2 那一形上，覆盖写一条链 28 步（`seq` 到 R 被挤出根环之后 10 步）、`onsu` 字母表长度 1–3 全部 73 个结点（`ow` 与 `sw3` 两种 R，记录全读得出），都 0 个 I-7.4、0 次探针失败。第 19 次发布那一刻的层 0 全量没跑（超过 10⁶），只跑了那一个整段前缀状态。
- C 的选法在记录全读得出时漏：`sw3`、`ow` 两种 R、`onsu` 长度 1–3，`all` 0 红。
- B（看账）误放：第四节 4.2 列的全部形状。
- 恢复补法在 K1-2 与藏记录两形上把一个本该退的状态读坏：两形层 0 全量里 `strict` 一次都没读出错内容（`violations=0`）。

## 这条腿自己的限度

- 「读回旧内容」这一类瞬时故障（4.1）不在第一轮用过的「读回全零」「读报错」两类里；它算不算本项目要防的故障，我没有条款依据，交主 agent 判。
- 第一节那一格只有一个整段前缀状态，不是层 0 全量：那次发布的写流 16,777,223 个状态，超过派发给的约 10⁶ 上限，没跑。
- 用户动作放开扫只扫到长度 3（`onsu`，73 个结点）；第一节的窗口在第 19 次发布，那一段只走了「全覆盖写」与「18 次覆盖写 + 1 次顺序写」两条链。`ring` 不中的道理是读代码得的：它在写掉 R 那一槽的那次发布取完落点之后才解，之后任何一次发布开始时 R 的根槽已被盖掉，与动作种类无关。
- 恢复一侧补法只实现在 `recover`；可写挂载与只读挂载两处没实现，5.2、5.3 里关于它们的句子标了「推的」。
- 「见证过」的两种读法（`strict` / `loose`）是我起的，不是第一轮判决给的定义；判决只写了「系统配置没见证它时退回见证到的那一版」。
- 层 0 全量只跑了 384 槽 k=5 一个起点（2.1、5.2）；状态数合计见下一节。

## 量与取样

- 层 0 全量（每一段都展开）跑过的状态：第一轮原型 `crashmount 384 5` 两臂 41,022；本轮 `cm` 开发时五次（今天 `urM` 一次、2.1 两次、5.2 两次），复跑全量档又五次（`cm-today-ow-ur` 与 2.1、5.2 那四次），每次 20,511–20,516 个状态。单次都不超过 10⁶，合计约 25 万，不超过 10⁷。
- 每次 `cm` 6 线程 117–274 秒（看负载；`LAYER0_PARALLEL_FINISHED` 行的 `elapsed_seconds`）。没有设批时限。没缩历史条数或长度以外的东西；崩溃状态没缩。
- 编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`（编译 6G、跑 4G）与 `capped.sh 6`；同一时刻只起一条 cargo。开工时 `ps` 看到别的会话的 `cargo test --release -p singlefs-checker-tier … --no-run` 在编，没有性能测量在跑；没有等锁。

## 没做什么

- 不判 K1（第一轮判决站不站得住）、K4（回扫），那两格归辩方腿。
- 没跑任何重型测试；`crates/singlefs-checker-tier` 只作为原型二进制的依赖被调用（`enumerate_layer0_selecting_versions_observing_each_state`，展开函数恒为真），测试目标一个没跑。
- 副本上的数不是入库装置上的数；要引得在入库装置上重做。
- 没读禁读清单里的文件（本轮正推 / 辩方腿的报告与模型、第一轮两条腿的报告、本轮判决）。
