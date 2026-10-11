# m3-batch10-code-r3 云端攻方（Opus）报告（2026-10-06）

攻击面：正文第三节 K1、K2、K3，重点 K1。只跑 harness 档与自己的原型；数都是冻结副本拷贝（`/tmp/claude-1000/m3-batch10-code-r3-opus/tree/`）上的数，不是入库装置上的数。代码行号一律引冻结副本 `/tmp/claude-1000/m3-batch10-code-r3/tree/`。

## 模型与复跑

模型目录 `research/prompts/m3-batch10-code-r3-opus-model/`（只读导入冻结副本，补丁只打在拷贝上）：

| 文件 | sha256 |
|---|---|
| `main.rs`（原型 `opus-r3-attack`，子命令 grid / cells / l0 / k2young） | 0e7965ba0dd4ba64cc1244bccbc3c127000162f13b646d5e702aff9280896b4e |
| `Cargo.toml` | e914a87979f1a78529550ced127383d13282cf70298853fd891c575fa6bb486d |
| `mount-arms.patch`（拷贝的 mount.rs 上加环境变量 `OPUS_R3_ARM` 臂开关，空 = 冻结副本原样） | fafe4ba34f003422a36ff245aeb368474d507c794610442c6cf3cabbf61dee28 |
| `rollback-instrument.patch`（慢档三条用例每次覆盖写之后打印 I-3.1 两个数，不改断言） | bbcff8a17655739d5d19560520072798c097374b6fb921b44626e1e4a4110db5 |
| `workspace-member.patch` | e4a4bd266e4f3a5100ea889a771326fed291b2a1eba13747a2c6b160bab65848 |
| `rerun.sh` | c7e6224021d96fb31047174037c82c94ecf9a2a7bdaf877c89c2bb8977eb19b6 |
| `agg.py`（从 `out/` 的网格日志按计划号对齐两臂） | b6ac3c362183865acf18344a6eb9499ffe0fdfe1ac356555640190056e6a906b |
| `agg-output.txt`（`agg.py` 这一次的原样输出） | 099557c3ea29dc4f790efcd7148619894558f14e37a3b03057851f311ed45cac |
| `out/*.log`（23 个阶段的原始日志） | 逐份见表下 |

```
9f3a6e8b3461f0ef9538d0066a2cfa1068539f2e90635a09587e2a627916c238  out/cells-first_wins.log
52c81d27f3b8bdd89e3a46a85677967ebe9cc0587935754593c5dd32f13a6726  out/cells-prod.log
1a80eb67720cd79c490c691c502a9e78fd6977e785eb72d6e678da08d644bac7  out/fixed-prod.log
127ee8ea0b9042a9550ac265963823d4ace0162b355aa54a7ff30f0d62b389f1  out/fixed-reread_only.log
6c6917ef0bb44c21b4ae2e3bfa7096f78473c2388f48be69ba397300262f4e9c  out/k2young.log
ce12ffabda460f4eb7f63200182041bcc514cd723534b3dbf2d8e9e5c11c5a04  out/k3.log
05ba5d1507bdd39f1c92011f52c0949a9ce500ce0011f5ada99e8809f0b421e9  out/l0-ab3845-K22-Z.log
7c39b8183b86c448ef58a8ffe393bc3393668e3f944d441ed5e2e5ed39216966  out/l0-ab3845-K22-Z-ro.log
3d1637cc3ef535cfa214ab8f4dc40a43fb85cfab700e7c8fc428cba8444496f8  out/l0-mat384-d0E-d0Z.log
3bc39e2acd3129098fcd951d743e8c45d4e04c432e8306c39d38b29066189ac5  out/l0-mat384-q3new.log
40467182d13538abd24c8b39b32ae5fadd95b11a58b160013dd2a44cc98b6d05  out/r7-prod-a.log
c9d9d99f53df9a1c2544560c4c7424c47f82388a9fb203d35b05b9ff578e99bf  out/r7-prod-b.log
5106836f6ca20e27316b4c1a5cbf2606f2040aea4726d9b02b7549d67cdc2645  out/r7-prod-c.log
49b0ee49f8b742d795ea0725b99658b41fecb9005435275b01fffb55daac21e3  out/r7-prod-d.log
47afbf0351037061cb97f95b5b286769985bf6389a3b026c6aedc2dc6475c842  out/r7-prod-e.log
47050f87ac239aba84a309b00426e2162f76a67bf1a7c604139d97be2391a9aa  out/r7-prod-f.log
af5c344090357f4186e086fc5dfe11e4f6664817023386da649e7a10d6625bd8  out/r7-prod-g.log
5d1a57402c19ea6e303f78bf015d418686dd0d92a8074fb92010c35de2eff3ee  out/r7-reread_only-a.log
d1afb8d152e77a9d8c517dd79bd5d1b8075d3d10232762e9c04b0217e8edf719  out/r7-reread_only-b.log
a329696cb5df1bbe1542a2e8bd1967cab1754e8efd2081fa7f24dd77e9365fd1  out/r7-reread_only-c.log
f0cc66b5cc084f940f5c8236addb213dcc4453409a07c30d9c7c95cbe4523d8a  out/r7-reread_only-g.log
3396f7d693b546d259e59d48248e4bfb56cd54251e5c64bff7e8216fd28ce2ff  out/r7-reread_only-h.log
cbcdefb3587f29c588c3ff8ecd12e1d2f9dfa760dff6f3941d34c93331390172  out/r7-zero_over_bad-n.log
```

复跑（在仓根）：`bash research/prompts/m3-batch10-code-r3-opus-model/rerun.sh setup <空草稿目录>`，`rerun.sh list` 列 23 个阶段，`rerun.sh run <草稿目录> <阶段>` 逐个跑（每个经 `run-with-memory-cap.sh 4G` 与 `capped.sh 31`），`rerun.sh compare <草稿目录>` 比 `OPUS3-SUM`、`OPUS3-L0-TALLY`、`OPUS3-L0-SCAN`、`OPUS3-K2Y`、`OPUS-K3` 与 `test` 行，末行「比了 23 份，N 份不同」。网格各阶段在本机负载下 20–200 秒，四段层 0 各 70–145 秒。`rerun.sh` 自己这一次没有从头跑过一遍（各阶段是在攻方草稿副本里用同样的参数跑的），见末节限度。

## 各格判定一览

| 格 | 判定 | 依据所在小节 |
|---|---|---|
| K1-1 同一槽两遍读到两个自证根 | 没打中：故障模型里造不出；故障模型之外的错位读上「取重读」「取第一遍」各丢一边，不分辨臂 | 第一节 1.1 |
| K1-2 一遍读坏、一遍全零并成读坏 | 没打中（代码三处消费者里读坏的约束包含全零的）；我的网格分不出「全零压过读坏」那条臂，这一格只算推的 | 第一节 1.2 |
| K1-3 并出来的分类与「挂载那一刻读坏」是不是同一类 | 不是同一类，线索（代价形状，checker 不红）：第一遍读出、重读起一直读坏的槽记成「住着根」，进程内抬 F 重读它仍坏就拒抬；窄盘上 31 段从「挂得上」、31 段从「取号之前 Q3 拒」变成「写行暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`」 | 第一节 1.3 |
| K1-4 Q3 收窄之后该拒却挂上、挂上之后 checker 红 | 打中，不分辨臂（C554 那一形）：重读臂拒成 Q3、按槽并挂上之后红 I-7.4 的 139 段，全是被抛弃根 C 那一槽两遍都读坏；同一形在重读臂里挂上照样红（203 段），R1 只换了哪一部分被拒 | 第一节 1.4 |
| K1-5 随机两遍读坏网格与层 0 | 没打中：按槽并臂 4146 段里红的 488 段全是 C554 那一形；重读臂的 310 段非 C554 红在按槽并臂上全绿；四段层 0 共 82044 态 checker 0、oracle 0 | 第一节 1.5 |
| K2-1 按槽归盘与落点公式在什么盘面上不同 | 没打中：只在第 0 代根三份都在时不同（mkfs 后第一次可写挂载崩溃态 66 态里 24 态），都不重读、checker 0；82110 态里公式算不到的非第 0 代根 0 次 | 第二节 2.1 |
| K2-2 卸载那一格照旧按公式 | 没打中（推的）：暖机写 txg 1、2 盖掉区域 1、2 的第 0 代根副本，卸载只接在可写挂载之后；两种归法只在根槽写被吞时不同，两边都只放到 ≤ 0 | 第二节 2.2 |
| K3-1 rollback 三条慢档改过的期望 | 方向没打中（每一步都只红 I-3.1、都在记账多的一边）；线索：差值不是常数，钉住期间每次覆盖写都涨，到 txg 51 涨到 5799936 / 5898240（挂载那一刻钉的 819200 / 917504 的约 7.1 / 6.4 倍），用例只钉挂载那一刻的差、txg 51 只核名字 | 第三节 3.1 |
| K3-2 Q3 判官「按表判被抛弃」等价论证 | 没打中：82110 个崩溃态里最新那条根的表里有它自己实例的行 0 次 | 第三节 3.2 |
| K3-3 丢盘整读的新起点 | 没打中：两条新用例要求丢任一块盘都比到至少一条带活文件的根；拷贝上 15 条全绿 | 第三节 3.3 |

## 第一节　K1 按槽并两遍

装置（只在攻方拷贝上）：原型 `opus-r3-attack` 的读故障设备按「（根环槽，这个槽第几次读）→ 报错 / 全零 / 乱字节」注错，另有「从第 n 次读起一直报错」（坏道从那一刻起）；重建分配器那一遍是每个槽的第 k 次读，k 每个起点现标（整环在第 k 次读报错时钩子恰调一次、调的那一刻每槽恰读过 k 次、读阶段与实例表都第一遍判完），十个起点都是 k=6。挂上之后用户发 30 次覆盖写（窄盘关准入，与第二轮攻方第一次抽样同法），每一步核池级 checker（I-3.1 红在记账多的一边记成「I-3.1+」，用户认的代价，不算红）与「挂载之前环里还在、txg ≥ F 的根丢任一块盘整读与挂载之前相同」。臂（拷贝的 mount.rs 上 `OPUS_R3_ARM`）：空 = 冻结副本今天的按槽并；`reread_only` = R1 之前整份换成重读那一遍；`first_wins` = 两遍都自证过取第一遍；`zero_over_bad` = 读坏与全零并成全零。

起点十个：被抛弃根 C 的盘面（第二轮攻方第二次抽样那一形重写一份）4 GiB 与 384 槽各三形（`ab-<宽>-<C 之前覆盖写>-<之后再挂一次再覆盖写>`），成熟环两个（`mat-*-30-8`，两个实例），窄盘两个（`narrow-264-6-4`、`narrow-384-20-2`）。计划：每个起点 17 或 11 个固定格，加种子 7 的 400 个随机计划（第一遍把一块盘上每个住着有效根的槽读坏、那块盘别的槽一半读坏、另一块盘按 0 / 0.15 / 0.4 读坏；重读那一遍每槽按 0 / 0.25 / 0.5 / 0.85 / 1 读坏；两成计划把重读报错的槽改成从那次读起一直坏）。

### 1.1　同一槽两遍读到两个不同的自证根

- 故障模型里造不出：harness 的读故障只有报错与翻一位（`crates/singlefs-harness/src/fault_injection.rs:142`「ReadReturnsCorruptedBytes { flipped_bit: FlippedBit },」），翻一位过不了整槽校验和；挂载期间没有写者。实现注释的前提就是这一条（`crates/singlefs-core/src/mount.rs:1772`「自证过的根不是读错造得出来的（整槽校验和过、fsid 是本池的）」）。
- 故障模型之外（错位读：读 C 那一槽读回同一块盘上另一个槽里的自证根；根记录不带槽号，`crates/singlefs-core/src/root_record.rs:139`「if &filesystem_identifier != expected_filesystem_identifier {」之后只核字段，自证得过）。`out/cells-prod.log`、`out/cells-first_wins.log` 原样（摘行首到 `classes=`）：

```
OPUS3-RUN sp=ab-384-1-0 arm="" plan=m1-reread-C-reads-Valid(0)@0/0 c554=false desc=C@2/2(d0) outcome=Ok(reread) hooks=1 fired=9 red=true isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] classes={"I-3.1+", "I-7.4", "broken>=F"}
OPUS3-RUN sp=ab-384-1-0 arm="" plan=m2-first-C-reads-Valid(0)@0/0 c554=false desc=C@2/2(d0) outcome=Ok(reread) hooks=1 fired=9 red=false isolated=[(DeviceIdentity(0), 12), (DeviceIdentity(1), 12)] classes={}
OPUS3-RUN sp=ab-384-1-0 arm="first_wins" plan=m1-reread-C-reads-Valid(0)@0/0 c554=false desc=C@2/2(d0) outcome=Ok(reread) hooks=1 fired=9 red=false isolated=[(DeviceIdentity(0), 12), (DeviceIdentity(1), 12)] classes={}
OPUS3-RUN sp=ab-384-1-0 arm="first_wins" plan=m2-first-C-reads-Valid(0)@0/0 c554=false desc=C@2/2(d0) outcome=Ok(reread) hooks=1 fired=9 red=true isolated=[(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)] classes={"I-3.1+", "I-7.4", "broken>=F"}
```

  `ab-384-3-5` 四行同形。四句：不分辨臂（两条臂各在对称的一格上丢 C）；被判系统当时看不到哪一遍是错位的；满足的是 K1「让被抛弃的根掉出影子账」那一分句，但前提在故障模型之外，不按判据算打中；改法里任何一种「两遍取一」都碰不到它，碰得到的是读槽时核「txg 按公式落在这个槽，或是第 0 代根」，把错位读降成读坏（只在我的模型上推的、没实现、被攻过零轮；读到同一槽一圈之前的旧副本那种照样漏）。

### 1.2　一遍读坏不是全零、另一遍全零，并成读坏

- 代码：`crates/singlefs-core/src/mount.rs:1793`「(BadAndNotAllZero, AllZeroNeverWritten) | (AllZeroNeverWritten, BadAndNotAllZero) => {」并成读坏。消费者三处：回收门槛两类都钉在可能住过的最旧 txg（同一个值）；进程内记根时的重读两类一起重读（`crates/singlefs-core/src/allocator.rs:2004`「occupancy.ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held();」），重读仍坏或全零都照旧钉着（`crates/singlefs-core/src/allocator.rs:2027`「| RereadOfARootRingSlotBadAtMount::AllZero => {}」）；准入抬 F 的上限只并读坏那一类。读坏的约束是全零的超集，方向安全（推的，按代码）。
- 量的：固定格 `f-d*-allE-reread-d*-allZ` 与 `f-d*-allZ-reread-d*-allE` 在十个起点上按槽并臂全绿（`out/fixed-prod.log`）。`zero_over_bad` 臂在两个窄盘起点上 822 段也全绿（`agg-output.txt`「TOT ('zero_over_bad', 'green') 822」）：**我的网格分不出这两条臂**，安全方向这一格只算推的；Q2 那一形（F 越过住在全零槽里的有效根）要第一轮攻方第二次抽样的 248 槽盘那种造法，我没造。代价：真全零的空槽第一遍瞬时读错、重读读回全零，并成读坏、并进上限，是 Q1 那一类卡住（用户已认），没量多出几次。

### 1.3　并出来的分类与「挂载那一刻读坏」那一类

不是同一类。第一遍自证、重读读坏的槽并成自证根，根环表记 `ValidRoot` / `AbandonedRoot`，按 D16 已定项 1 那一行它是「这个进程知道住着根的槽」（`.claude/kb/decisions/16-发布语义.md:37`「挂载那一刻读得出、这个进程写过且 FUA 返回过、或进程内记根时重读读出了有效根的槽」），之后抬 F 那一遍重读它（`crates/singlefs-core/src/mount.rs:3288`「let roots = readable_roots_rereading_ring_slots_known_to_hold_a_root_once(」），仍坏就拒这一次抬（同一行「这一次读不出或自证不过就重读一次，仍坏就拒这次抬 F 或这次回退」）。R1 之前（重读臂）同一个槽记成 `BadAtMount`、钉住，抬 F 照走。

量的（只在「从重读那次读起一直坏」的计划上出现，瞬时读坏的 0 段）：`agg-output.txt` 原样：

```
TRANS reread_only->prod ('Ok(reread)', 'Err(FloorRaiseFailedAfterTheMountsPublishes)', 'persist') 31
TRANS reread_only->prod ('Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)', 'Err(FloorRaiseFailedAfterTheMountsPublishes)', 'persist') 31
FRF ('narrow-264-6-4', 'Ok(reread)') 9
FRF ('narrow-384-20-2', 'Ok(reread)') 22
```

例 `out/r7-prod-g.log` 的 `plan=r7-387`：重读臂「`outcome=Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread) … image_unchanged=true`」，按槽并臂「`outcome=Err(FloorRaiseFailedAfterTheMountsPublishes) … image_unchanged=false after_err_classes=[]`」：取了号、写了行与暖机之后才报错，盘面 checker 0 违例、丢盘整读不变。4 GiB 起点挂载不用抬 F，同一类计划照常挂上；384 槽起点挂上之后第 13、14 次覆盖写报 `FloorRaiseFailedWhilePushingForSpace`（重读臂在同一批计划上也报，`agg-output.txt` 的 `PT` 行）。

四句：分辨臂（重读臂挂上或在取号之前拒，按槽并臂写了之后才错）；被判系统看得到（第一遍读出、重读读坏，这一刻两样都在手里）；满足的不是 K1 那几个安全分句（没有越过有效根、没有掉出影子账、checker 不红），是「并出来的分类与挂载那一刻读坏是不是同一个分类」那一问，答「不是」，后果是活性代价；改法（零轮、没实现、推的）：两遍里有一遍读坏的自证根，根环表里另记一类「挂载时读出、重读读坏」，抬 F 重读它仍坏时照 `BadAtMount` 钉住、不拒。压过 R1 那一句「任一遍自证过的根」的字面，交主 agent 定是不是条款要回答的事。

### 1.4　Q3 收窄之后该拒却挂上、挂上之后 checker 红

`agg-output.txt` 原样：

```
TRANS reread_only->prod ('Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)', 'Ok(reread)', 'transient') 464
TRANS reread_only->prod ('Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)', 'Ok(reread)', 'persist') 75
TRANS reread_only->prod ('Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)/c554', 'Ok(reread)/red/c554', 'transient') 112
TRANS reread_only->prod ('Err(NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread)/c554', 'Ok(reread)/red/c554', 'persist') 27
TOT ('reread_only', 'red_c554') 203
```

重读臂拒成 Q3、按槽并臂挂上的 678 段：539 段全绿；139 段红 I-7.4 与「broken>=F」，全是被抛弃根 C 那一槽两遍都读坏（`c554=true`），C 的单元在第 1 次覆盖写被复用，例 `out/fixed-prod.log`「I-7.4: 被抛弃的根（实例 2、txg 8）引用的单元已被重新分配或抹头」。这一形就是欠账 C554 写着的那一种进入法（`.claude/kb/checks-owed.md:427`「同一形在被抛弃的根所在的槽挂载那一刻读不出、可写挂载里重读也读不出时照样出现」）。

四句：不分辨臂——重读臂里 C 两遍都读坏、重读读出别的根时照样挂上、照样红（203 段），R1 只改了「重读整环都坏」那一小块被拒还是挂上；被判系统看不到 C（两遍都没读出）；满足的是 K1「Q3 改成两遍都没有才拒之后……挂上之后 checker 红」那一分句的字面，归 C554 已在欠账的那一族，按判据第二节「已知没做」处理、不算这批的打中；改法：两遍取一的任何一种都碰不到，碰得到的只有「重读过、仍有读坏的槽、最新那条根的实例表里有被抛弃实例的行时拒可写」（零轮，推的，会把 1.3 那批再拒掉一部分，没量）。给 C554 记一笔新入口与数：R1 之后它从「Q3 拒」那一侧漏到「挂上」的有 139 / 4146 段。

### 1.5　网格总数与层 0

`agg-output.txt` 原样（按计划号去重，固定格与随机格合计）：

```
TOT ('prod', 'green') 3658
TOT ('prod', 'red_c554') 488
TOT ('reread_only', 'green') 2833
TOT ('reread_only', 'red_c554') 203
TOT ('reread_only', 'red_not_c554') 310
TRANS reread_only->prod ('Ok(reread)/red', 'Ok(reread)', 'transient') 243
TRANS reread_only->prod ('Ok(reread)/red', 'Ok(reread)', 'persist') 67
```

按槽并臂 4146 段，红的 488 段全是 C554 那一形（1.4），别的 0 段；重读臂红而不是 C554 的 310 段（第二轮 K2-2 那一族：第一遍读出、重读读坏的 C 掉出影子账），在按槽并臂上同一计划全绿——这一组同时是装置会红的自证。按槽并臂里拒成 Q3 的只有 `r7-93` 两段（两个 `-3-5` 起点上每个住着有效根的槽两遍都读坏），盘面逐字节不变。

层 0：每段历史 = 起点 + 计划 + 这次可写挂载（写行与暖机，`OPUS_STEPS=0`），录写流，`enumerate_layer0_selecting_versions_observing_each_state` 每一段都展开，版本表带起点与这次挂载写的每条根的内容。跑前 `closed_form_state_count` 每段 20496。原样：

```
OPUS3-L0-TALLY sp=mat-384-30-8 arm="" plan=f-d1-allE-reread-allE states=20511 oracle_violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 secs=143.5
OPUS3-L0-TALLY sp=ab-384-3-5 arm="" plan=f-d1-allE-reread-A0/3-Z states=20511 oracle_violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 secs=74.4
OPUS3-L0-TALLY sp=mat-384-30-8 arm="" plan=f-d0-allE-reread-d0-allZ states=20511 oracle_violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 secs=131.9
OPUS3-L0-TALLY sp=ab-384-3-5 arm="reread_only" plan=f-d1-allE-reread-A0/3-Z states=20511 oracle_violations=0 first_violation=None checker_violated={} checker_first={} failed_states=0 secs=69.6
```

四段 82044 态，加第二节 66 态，合计 82110 态，远在 10⁷ 之内。限度：只挂载那一串分不出臂（最后一行重读臂同样 0：C 的单元在挂载之后第一次覆盖写才被复用，写行与暖机没碰它）；带一次覆盖写的同一段历史 `closed_form_state_count` 是 16797718–268828700（`OPUS3-L0-PLAN … published=1`，草稿日志），超过每段 10⁶，没跑，交主 agent（末节）。

## 第二节　K2 按槽归盘

### 2.1　两种归法在什么盘面上不同

- 写根槽的只有一处，按公式（`crates/singlefs-core/src/transaction.rs:450`「let target = target_for_publish(」）；公式之外只有 mkfs 三区槽 0 各一份第 0 代根（`crates/singlefs-core/src/make_filesystem.rs:619`「for ring_slot in genesis_root_ring_slots() {」）。所以归法只在区域 1 槽 0 那份第 0 代根还在、盘 1 别的槽没读出有效根时不同。
- mkfs 之后第一次可写挂载的每个崩溃态（`out/k2young.log`，4 GiB 与 384 槽各 33 态）原样：

```
OPUS3-K2Y-SUM width=4g genesis_copies=3 formula_vs_slot_differs=true off=0 own_row=false outcome=Ok(first) hooks=0 checker_after=[] n=12
OPUS3-K2Y-SUM width=4g genesis_copies=2 formula_vs_slot_differs=false off=0 own_row=false outcome=Ok(first) hooks=0 checker_after=[] n=12
OPUS3-K2Y-SUM width=4g genesis_copies=1 formula_vs_slot_differs=false off=0 own_row=false outcome=Ok(first) hooks=0 checker_after=[] n=9
```

  （384 槽三行同数。）两种归法不同的 24 态都不重读、挂载之后 checker 0；第二轮判决的「只剩第 0 代根时重读」不再出现。
- 「公式算不到的槽里住着非第 0 代根」：82110 态逐槽扫（层 0 四段落在 `mat-384-30-8`、`ab-384-3-5` 两个起点上，加 mkfs 之后那 66 态），0 次；十个网格起点本身没单独扫（`OPUS3-L0-SCAN … off_formula_states=0`）。根槽写被吞（`WriteIsSwallowed`）时区域 1 槽 0 可以留着第 0 代根而盘 1 后面的槽有新根，这一形我没造；按代码推：归法把盘 1 算成「最新有效根 txg 0」，m = 0，当场只放到 ≤ 0，比按公式（盘 1 有更新的根时取那一条）保守，只少放开、不越过。
- 其余几处仍按公式归盘：准入上限（`crates/singlefs-core/src/mount.rs:2523`「let device = system_configuration.immutable.region_devices[usize::try_from(」）、抬 F 那一遍判哪几个槽这一遍读出了根（`crates/singlefs-core/src/mount.rs:3157`「.map(|root| target_for_publish(root.checkpoint_txg, slots_per_region))」）、F_生效。三处只在第 0 代根副本上与按槽不同，副本带的 F 与 txg 都是 0：抬 F 那一处把区域 1 槽 0 当「没读出根」、再走一次 E2 重读，读出 txg 0 的有效根，结局相同（推的）。没打中。

### 2.2　卸载那一格照旧按公式

卸载只在一次做成的可写挂载之后（或 mkfs 同一个进程取号暖机之后）；那一次的写行与暖机落 txg 1、2，盖掉区域 1、2 槽 0 的第 0 代根副本（`target_for_publish` 把 txg 1、2 落到区域 1、2 槽 0），所以卸载那一遍读不到副本，两种归法相同（推的，按代码；k2young 里挂载做完的那几态 `genesis_copies=1`）。例外同 2.1 的吞写：卸载那一遍按公式盘 1 没有根 → 当场放开的那一截不放（`crates/singlefs-core/src/mount.rs:3341`「.map(|root| (device_of_txg(root.checkpoint_txg), *root))」之后 `None => Vec::new()`），按槽是放到 ≤ 0：两边都不越过。没打中（推的、没造吞写历史）。

## 第三节　K3 改过期望的用例

### 3.1　rollback 三条慢档

拷贝上打 `rollback-instrument.patch`（钉住那一槽被写掉之前，每次覆盖写之后打印 I-3.1 的两个数，断言一个不改），release 跑三条加对照（`out/k3.log`）：「test result: ok. 4 passed; 0 failed」。被打印的 60 步按用例归（草稿里的一次性脚本，输入是 `out/k3.log`）：

```
恢复之后重开 steps 20 txg 32 .. 51 names {'["I-3.1"]'} diff_first 1146880 diff_last 5898240 min 1146880 max 5898240 monotone True
TornAfterTheFirstHalf：崩溃之后重开 steps 20 txg 32 .. 51 names {'["I-3.1"]'} diff_first 1048576 diff_last 5799936 min 1048576 max 5799936 monotone True
ZeroesWritten：崩溃之后重开 steps 20 txg 32 .. 51 names {'["I-3.1"]'} diff_first 1048576 diff_last 5799936 min 1048576 max 5799936 monotone True
```

- 方向：每一步只红 I-3.1、都在记账多的一边，没有一步红在少的一边，也没有别的不变量红。没打中。
- 大小：差值不是常数。挂载那一刻钉 819200 / 917504（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:1760`「819_200,」），之后每次覆盖写都涨，单调，到写掉那一槽前一次（txg 51）是 5799936 / 5898240。机理（推的）：回收门槛钉在 txg 4，钉住期间每次覆盖写换下的单元释放代都高于 4，一个都回收不了，记账照算「已分配」，遍历只算有效根。用例的 helper 只在 txg 51 核名字（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:244`「violated_before_the_slot_is_written,」那一项比的是 `vec!["I-3.1"]`），中间各步不核方向，注释写的是「写到它前一次为止 I-3.1 仍只红在记账多的一边」（同一份文件第 216 行），代码没钉。
- 四句：不分辨臂（钉最旧 txg 是用户认下的那一种，这里只问代价多大）；系统看得到（钉住的槽在根环表里）；对的是正文第二节「只核代价是不是只有这么大」：用户认的是「红到这个进程写掉那一槽为止」，时长对上了，量是一圈之内全部释放的累积，不是用例钉的那个数；改法：用例每步核方向、在 txg 51 钉差值（不改行为，只把注释那句钉住），或在条款「代价」那一行写明是累积量。只在我的副本上量过，被攻过零轮。

### 3.2　Q3 判官「按表判被抛弃」那半句的等价论证

写行只给比要取的号小的实例写（`instance_rows_to_write` 的区间到 `instance_to_acquire` 为止，不含），最新那条根是它自己实例写的或继承的表，表里不会有它自己实例的行。量的：82110 个崩溃态逐个看「最新那条根的表里有不小于它实例号的行、或它按自己的表被抛弃」，0 次（`OPUS3-L0-SCAN … newest_root_with_own_row_or_abandoned=0`，`OPUS3-K2Y-SUM … own_row=false`）。没打中；取样只覆盖两个起点上的挂载与 mkfs 之后那一次挂载，回退、卸载与带记录重建的恢复没进这份扫描。

### 3.3　丢盘整读的新起点

两条新用例（`on_the_264_slot_pool_with_a_live_file_device_{zero,one}_failing_the_rebuild_pass_…`）要求丢任一块盘时「环里还在、不低于 F 的根」至少比到一条、其中至少一条带活文件，再比整读相同；拷贝上整份 15 条全绿（`out/rr2-tests.log`，不在 `rerun.sh` 的阶段表里：「test result: ok. 15 passed; 0 failed」）。罩住的是第二轮 K2-7 要的那件事。限度：这两条只坏第一遍、重读干净，并出来的就是重读那一遍；「并」本身靠同一份文件里 K2-2 那几条与核心层两条单测罩。没打中。

## 没打中的形状

- 两遍读坏的随机组合：十个起点 × 400 个随机计划 + 每起点 11–17 个固定格，读坏种类报错 / 全零 / 乱字节均匀取，两成计划从重读起一直坏，挂上之后 30 次覆盖写；按槽并臂除 C554 那一形外 0 红（1.5）。
- 一遍读坏一遍全零交错（`f-d*-allE-reread-d*-allZ`、`f-d*-allZ-reread-d*-allE`）：十个起点全绿。
- 两遍都自证过取哪一遍：故障模型里造不出（1.1）。
- Q3 收窄：678 段从拒变挂，除 C554 那一形全绿（1.4）。
- 归盘：mkfs 之后第一次可写挂载 66 态，层 0 四段 82044 态（2.1）。
- 用户动作：挂上之后的那几步只放开了「覆盖写、没文件就建 inode」这一种，次数扫 0..30 每步都判；删除、顺序写、回退没扫。

## 这条腿自己的限度

- 数都在攻方拷贝上量，臂开关是我加的（`mount-arms.patch`），入库装置上没有。
- 层 0 只跑了「只挂载、不接着写」的四段，它们分不出臂（1.5）；带写的历史状态数超过每段 10⁶，没跑。
- 我的网格分不出 `zero_over_bad` 臂（1.2），也没有判「F 越过住在全零槽里的有效根」那一格的判据（I-7.9 那种要专门的盘宽造法）。
- 起点只有一种被抛弃根的造法（藏 C 的根槽与单元、系统配置停在 C 之前）；C718、C719 那两种进入法没造。
- 固定格 `f-*` 与随机格同用 k=6 的读序号；每段注错的计划在钩子那一刻核过每槽读数等于 k（`k_mismatch` 0 段，`agg-output.txt` 没有那一行）。
- `rerun.sh` 是事后整理的：阶段的命令与参数照草稿里实际跑的抄，`setup` 那一步与整套 `compare` 这一次没有从头跑过；`r7-prod-d`、`r7-prod-f` 两份日志是被 `timeout` 截断的批次，只保留了跑完的起点，阶段定义按跑完的那几个起点写。
- 一次抽样；「没打中」各格按规则要再抽一次才记。
- 引 `.claude/kb/checks-owed.md` 的行号取冻结副本里那一份（第 427 行）；仓里那份在这一轮里被别的会话改过，同一句此刻在第 426 行。

## 要主 agent 跑的命令

1. 带一次覆盖写的层 0（分得出臂的那一段，`closed_form_state_count` 16797718，超过子 agent 每段 10⁶ 的上限）：工作目录 `rerun.sh setup` 建的草稿目录，命令 `OPUS_SEED=7 OPUS_PLANS=400 OPUS_STEPS=1 OPUS_L0_MAX_STATES=20000000 OPUS_R3_ARM=<空或 reread_only> bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 31 <草稿目录>/tree/target/release/opus-r3-attack l0 ab-384-3-5 f-d1-allE-reread-A0/3-Z`；按 1.5 的速度约 0.004–0.007 秒一态，估 20–35 小时一臂（推的）。看 `OPUS3-L0-TALLY` 那一行的 `checker_violated`：重读臂该有 I-7.4、按槽并臂该是空。
2. 整套复跑：`rerun.sh setup`、`list` 列的 23 个阶段逐个 `run`、`compare`，末行「比了 23 份，0 份不同」。

## 没做什么

- 不判辩方那几格（K4 与辩方立场的 K1、K3）；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑重型测试，没跑 checker 档包的测试二进制；原型二进制依赖 checker 档库、只调枚举函数跑自己造的流。
- C554 新入口那一笔、1.3 的分类与 3.1 的累积量，都要主 agent 现查再定去向。
- 收到主 agent 的消息：线程上限 15 改 31，在固定格网格跑完、随机网格开跑之前；之后的命令都按 31。
