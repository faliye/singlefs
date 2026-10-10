# 回收门槛取一圈窗口：第一轮攻方腿（Opus）报告（2026-10-05）

攻击面：W1（今天的 `crates/` 会不会留下窗口之外仍有效的根，那一形上假候选与不可恢复出不出现）、W3（各候选在 W1 那几形与 D23 第 399 行目标形态上判成什么，会不会重开窗口当初要关的那一形）。W2 不归这条腿。
副本上的数都是冻结副本拷贝上量的，不是入库装置上的数；模型（`ring_model.py`）是攻方自己写的计数模型，被攻过零轮。

## 复跑命令与文件

冻结副本：开工时主工作区里这几份文件对 `research/prompts/m3-reclaim-window-r1-snapshot/crates-sha256.txt` 与 `kb-sha256.txt` 跑 `sha256sum -c` 全 OK，之后 `rsync -a --exclude target --exclude .git` 拷到 `/tmp/claude-1000/reclaim-window-attack/G/`。臂 RGall / RGalloc / RGchk 是 G 的拷贝打上模型目录里的 `arm-*.diff`。原型 `probe/` 拷进每份拷贝的 `crates/attack-rw-probe/`，工作区成员加一行 `"crates/attack-rw-probe",`。

```
export ATTACK_DIR=/tmp/claude-1000/reclaim-window-attack
M=research/prompts/m3-reclaim-window-r1-opus-model
bash $M/scripts/build.sh G        # RGall、RGalloc、RGchk 同样
# 批 1（S = 4）：每臂 PRE = 0 / 1 各一件
ATTACK_THREADS=2 S=4 C=24 PRE=0 K=0,1,2,3,4,5,6,7,8,9,10,11,12,13,14 J=0,1,2,3,5,8,13,20 bash $M/scripts/run.sh G out/b1-G-pre0.log
# 批 2（S = 8）
ATTACK_THREADS=3 S=8 C=48 PRE=1 K=0,1,2,8,16,22,23,24,25,26 J=0,1,3,8,26 bash $M/scripts/run.sh G out/b2-G-s8.log
# 逐快照全打（窗口外有效根整读）
VERBOSE=1 S=4 C=24 K=1,6,12 J=20 bash $M/scripts/run.sh G out/v-G.log
# 层 0（每个崩溃状态在不丢盘 / 丢盘 0 / 丢盘 1 下恢复 + 整读）
S=4 C=24 K=13 J=0 LAYER0=1 SINGLEFS_LAYER0_THREADS=15 bash $M/scripts/run.sh G out/l0-k13-j0.log
# 计数模型（纯 Python，几秒）
PYTHONDONTWRITEBYTECODE=1 python3 $M/ring_model.py 4 7 PXYC 2 all
```

`results/` 下每份日志的命令就是日志名对应的那一条（参数见日志里每行开头的 `s= c= k= j=`）。`l0-k11-j0.log`、`l0-k13-j0.log` 是在原型加 `DEV_MIB`、`PRE` 两个开关之前那一版源码上跑的（两个开关取默认时行为不变）。

```
3408fc102a94b32bad359477d136c5618f5015420393faf86ba828897351aa1b  ./arm-RGall.diff
ce5ead6948caea5c50bfa13217c484b3063a5256ea4f67e374f26a89c1118cee  ./arm-RGalloc.diff
3948cdb074240b872509a836056da4a4fa09ca658199d30304de4c636e8166cc  ./arm-RGchk.diff
08482b24650ed5d65b32c668ac346bfcd8797fe505b9b3cbd5ac109a94ca905f  ./probe/Cargo.toml
27ca3277310a7578b13fb22c0c0649ae9704694e8f24fc8da8b866ef1a0018c5  ./probe/main.rs
a65462e387042ffb98c8bf463503359b5bff4e9ab66ce846b44f8ec70e9fe940  ./ring_model.py
2553385fe64bb966fd9d1232d746325fc8a47e75193f9efd7f726b87eec790d8  ./scripts/build.sh
1c4737d2df92af71ed939606460245e80988bb81264566062414ebed0fce77b5  ./scripts/run.sh
```
（`results/` 下 20 份日志的 sha256 在模型目录的 `SHA256SUMS`，`sha256sum -c` 28 行全 OK。）

## 各格判定一览

| 格 | 判 | 依据（量过 / 推的） |
|---|---|---|
| W1-a 今天的实现留不留窗口外仍有效的根 | **留**。K1 一次（k = 1）就有一条；K1 连崩 k ≥ 11（S = 4）时实例 1 的根全在窗口外；挂载途中一块盘上最新的有效根落在窗口外 | 量过（副本） |
| W1-b 那几条根引用不引用已复用单元 | **引用**：k = 6、j = 20 那段历史上窗口外的 1:15–1:18 整读报 `DataUnitChecksumMismatchEverywhere` | 量过（副本） |
| W1-c 今天的实现上出不出假候选、不可恢复 | **一次抽样没造出**：快照 290 段历史、层 0 3 段 65 640 个崩溃状态 × 3 种丢盘，0 | 量过（副本），一次抽样 |
| W1-d 挡住它的是什么 | 门槛本身挡不住，挡住的是落点政策与暖机次序：模型里换成「取编号最小的空闲单元」，G 在 K1 连崩族上丢一块盘之后读坏（S = 4 32 状态、S = 8 56 状态） | 模型量过，实现上推的 |
| W3-G 今天的窗口照留 | 今天实现：没造出；**D23 第 399 行目标形态：模型上丢一块盘之后读坏**（PXYC 全枚举 lone1 1009、lone0 35 / 337 408 状态）⇒ 照跑前判据第一条在目标形态上出局（模型，零轮；判据分句见四句） | 实现量过 / 目标形态只在模型上量过 |
| W3-RGall 退回环里最旧有效根，六处一起改 | 今天实现与模型都 0；不重开 K1 那一形（窗口外的根变成候选、全读得对）；代价：S = 4 挂载之后已分配多到 166 槽（G 同段 26） | 量过（副本 + 模型） |
| W3-RGalloc 只改分配器门槛 | **重开**：checker I-3.1 在合法盘面上误红，S = 4 112 / 120 段、S = 8 45 / 50 段（红的全是 k ≥ 1 的段） | 量过（副本） |
| W3-RGchk 只改 checker 候选过滤 | **重开**：I-2.1 + I-3.1 + I-3.10 + I-4.8 + I-7.4 红（k = 1 / 6 / 12、j = 20 三段全红） | 量过（副本） |
| W3-WD 攻方提：窗口门槛再取不高于每块盘上最新有效根 | 模型上 K1 连崩族、目标形态族、随机族全 0；代价介于 G 与 RG 之间（模型 max_held S = 4 PXYC 22 / RG 56 / WD 22；DP 族 22 / 94 / 66）；没在实现上写 | **只在我的模型上量过、被攻过零轮** |
| W3-WL 窗口长度随跳过的 txg 数加长 | 推的：跳一个 txg 加长一格，正好把那一槽的上一圈的根收回窗口，等价于 RG（持续失败时同样一直加长） | 推的 |

## W1　今天的实现会不会留下窗口之外仍有效的根

### 装置

起点：mkfs（两块内存稀疏盘、区域归属 0 / 1 / 0、S = 4 或 8）→ 实例 1 第一个文件 → 覆盖写到 txg C（S = 4 取 24，S = 8 取 48，根环填满两圈）；`PRE = 1` 时再做一次完整挂载不写（让现行版是一次暖机的根）。历史：在起点上连崩 K 次「写行那次单元与记录已持久、根槽没持久」，再做一次完整可写挂载（录写流），之后用户覆盖写 J 次。**用户动作（J、PRE）放开扫**，只固定前缀（起点）与故障（K 次写行崩）。判：每次根槽写落下那一刻各一份快照，跑池级 checker、列环里窗口外的根（只留它一条整读）、按 D16「回退候选集」那一行的定义算候选（窗口里、≥ F，只留它整读）、丢一块盘之后剩下那块盘只读挂载整读；读出来的 (实例, txg, 哈希) 与那一版落满时整读的比。层 0：对最后那次挂载的整段写流跑 `enumerate_layer0_selecting_versions_observing_each_state`（每一段整写子集都展开），每个崩溃状态在不丢盘、丢盘 0、丢盘 1 三种下恢复 + 整读。

### 机理（代码，冻结副本与主工作区同 sha）

- 挂载时窗口锚取环里读得出的最新有效根：`crates/singlefs-core/src/mount.rs:1828`「let window_anchor = roots」；而装进分配器根环表的进程内锚是新实例第一次发布的 txg 减一（`crates/singlefs-core/src/mount.rs:1948`「.expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),」，first_txg 取环里与记录里的最大 txg 加一，`crates/singlefs-core/src/mount.rs:1247`「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」）。K1 每崩一次，记录里多一个没有根的 txg。
- 进程内每记一条根就把锚挪到这条根（`crates/singlefs-core/src/allocator.rs:1828`「occupancy.latest_recorded_txg = checkpoint_txg;」），按窗口里最旧有效根当场回收（`crates/singlefs-core/src/allocator.rs:1855`「self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);」）。写行那次的根一记，锚就跳过 K 个没有根的 txg：K ≥ 3S − 1 时窗口里只剩写行这一条根，门槛 = 写行的 txg，上一实例全部已释放的单元当场回收。
- 窗口判法只看 txg 距离：`crates/singlefs-core/src/root_ring.rs:123`「pub fn txg_is_inside_the_one_revolution_window(」。被跳过的 txg 那一槽里留着上一圈的根，它自证合法、按实例表有效，txg ≤ N − 3S，就是「窗口之外仍有效的根」。

### 量到的（副本）

1. **K1 一次就有**（`results/v-G.log`）：
   ```
   s=4 c=24 k=1 j=20 root#0 newest=26 window_lower=15 F=0 checker=[] outside_window=[1:13=ok@1:24] candidates=[…] lone=[lost0:ok@1:22 lost1:ok@3:26]
   ```
   （candidates 那一段我用 sed 折掉了，原行在日志里。）
2. **K1 连崩 12 次，挂载途中一块盘上最新的有效根落在窗口外**（同一份日志）：写行 37 落在盘 1，盘 0 上最新的有效根是 1:24（窗口下沿 26），丢盘 1 之后剩下的盘 0 恢复就择它：
   ```
   s=4 c=24 k=12 j=20 root#0 newest=37 window_lower=26 F=0 checker=[] outside_window=[1:14=ok@1:24 1:15=ok@1:24 1:16=ok@1:24 1:17=ok@1:24 1:18=ok@1:24 1:19=ok@1:24 1:20=ok@1:24 1:21=ok@1:24 1:22=ok@1:24 1:23=ok@1:24 1:24=ok@1:24] candidates=[…] lone=[lost0:ok@14:37 lost1:ok@1:24]
   ```
3. **门槛跟着跳**：挂载之后每盘已分配（`results/b1-G-pre0.log` 的 `mount` 行）k = 0 时 136，k = 11 时 20、k = 13 时 26；同几段在 RGall 上 136 / 148 / 166：
   ```
   s=4 c=24 k=0 j=0 mount instance=2 chosen=24 effective=24 row=25 warm_up=[26] current=26 alloc_after_mount=[dev0:alloc=136,free=6968,deferred=122,isolated=0 dev1:alloc=136,free=6968,deferred=122,isolated=0] skipped_row_txgs=[]
   ```
   k = 11 那一行的 `alloc_after_mount=[dev0:alloc=20,free=7084,deferred=6,isolated=0 …]`（整行在日志里，跳过的 txg 列表长，不抄）。
4. **窗口外的根引用已复用单元**（`results/v-G.log`，k = 6、j = 20，共 8 份快照）：
   ```
   s=4 c=24 k=6 j=20 root#7 newest=38 window_lower=27 F=0 checker=[] outside_window=[1:15=ERR[read:DataUnitChecksumMismatchEverywhere { unit_index_in_file: Dat] 1:16=ERR[read:DataUnitChecksumMismatchEverywhere { unit_index_in_file: Dat] 1:17=ERR[read:DataUnitChecksumMismatchEverywhere { unit_index_in_file: Dat] 1:18=ERR[read:DataUnitChecksumMismatchEverywhere { unit_index_in_file: Dat]] candidates=[8:31=ok@8:38 8:32=ok@8:38 8:33=ok@8:38 8:34=ok@8:38 8:35=ok@8:38 8:36=ok@8:38 8:37=ok@8:38 8:38=ok@8:38] lone=[lost0:ok@8:37 lost1:ok@8:38]
   ```
   同一段在 RGall 上 0 份（`results/v-RGall.log`：outside 非空 33 份、读坏 0）。这几条根今天没有消费者：候选与丢盘恢复都不择它们，所以不算假候选。
5. **假候选、不可恢复：一次抽样没造出**。快照：G 臂 S = 4 PRE = 0 / 1 各 120 段、S = 8 50 段，`any_bad` 全 false（`results/b1-G-pre0.log` 等三份的 `SNAPSHOTS` 行）。层 0（j = 0，就是写行与暖机那几次发布，暴露的窗口全在这里）：
   ```
   s=4 c=24 k=11 j=0 LAYER0 states=20496 tally_states=20511 failed=0 violations=0 checker_violated={} by_lost={"lost0": {"ok": 20511}, "lost1": {"ok": 20511}, "none": {"ok": 20511}} elapsed=93.1s
   s=4 c=24 k=13 j=0 LAYER0 states=24598 tally_states=24618 failed=0 violations=0 checker_violated={} by_lost={"lost0": {"ok": 24618}, "lost1": {"ok": 24618}, "none": {"ok": 24618}} elapsed=211.7s
   ```
   k = 12 那一段第一次跑时我传给枚举器的已发布版本表漏了起点那几版，枚举器自己的 oracle 报 `violations=16395`（全是「第 24 代根下面没有文件却读出了内容」，表漏造的）；我自己的观察者在同一批 20 511 个状态 × 3 种丢盘上全 ok（`results/l0-G-k12-j0-oracle-without-start-versions.log`）。修了版本表之后没再跑 G 的 k = 12，RGall 的 k = 12 跑了 `violations=0`。

### 挡住它的是什么（推的 + 模型）

今天没出事，靠的不是门槛：写行那次的落点在记根（门槛跳）之前就取完了，暖机那几次的提交内生块接着在写行打开的聚簇段里 bump（`crates/singlefs-core/src/allocator.rs:1186`「那一段仍装着这次提交的内生块，D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」照样压着它」），用户数据要等挂载做完、暖机让两块盘都落了新实例的根之后才写（D16 已定项 8，`.claude/kb/decisions/16-发布语义.md:204`「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回；做法是连推空发布」）。两块盘根环不齐、门槛已越过一块盘上最新有效根的那几次发布里，没有一个落点会碰到那条根引用的单元——这是落点政策给的，不是窗口给的（推的）。

模型（`ring_model.py`，分配取编号最小的空闲单元，不建聚簇段）把这一层拿掉之后，G 在 K1 连崩族上丢一块盘就读坏（`results/model-a.log`、`results/model-b.log`）：
```
S=4 L=16 alphabet=KC U=2 mode=family arm=G histories=289 states=20244 fake=0 lone0=0 lone1=32 newest=0 states_with_valid_root_outside_window=5854 max_held=24
    lone1_sample=KKKKKKKKKKKC@11C|crash-in-28
S=8 L=28 alphabet=KC U=2 mode=family arm=G histories=841 states=104306 fake=0 lone0=0 lone1=56 newest=0 states_with_valid_root_outside_window=36430 max_held=48
```
同两族 RG、WD 都是 `lone1=0`。推翻条件：在今天的实现上造出一段合法历史，写行记根之后、两块盘都落新根之前，有一次落点取到了某块盘上最新有效根引用的单元（例如写行那一段聚簇段装不下暖机那几次，回落到别处——我没造出来，见「没打中的形状」）。

### 其余几形（W1 点名的）

- **连崩两次写行**：K = 2 在 290 段里，同 K1 一类，窗口外的根多一条，没出事。
- **被抛弃的根**：窗口不管它。分配器的窗口最小值里被抛弃根恒不算（`crates/singlefs-core/src/allocator.rs:1052`「RootRingOccupant::AbandonedRoot { .. } => None,」），影子账对环里每一条被抛弃根都隔离、不看窗口（`crates/singlefs-core/src/mount.rs:1456`「for root in roots.iter().filter(|root| is_abandoned(root)) {」），窗口只滤豁免用的候选。没造历史（这一族里没有被抛弃根），推的。
- **挂载那一刻读不出的槽**：没造（内存盘只能造自证不过，造不出设备读错），见限度。
- **两块盘根环不齐**：上面第 2 条就是（K1 连崩让写行与暖机之前一块盘上的最新根落在窗口外）；进程内的根槽写失败今天原样重发、不推进 txg（`crates/singlefs-core/src/transaction.rs:1250`「pub fn resend_the_frozen_publish<Device: BlockDevice>(」），不推进就不跳槽，进程内造不出不齐。

## W3　回收门槛取什么

### 候选与臂的定义

| 名 | 定义 | 怎么实现的 |
|---|---|---|
| G | 今天的：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)，六处同取窗口（`.claude/kb/decisions/16-发布语义.md:33`「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根) |」） | 冻结副本原样 |
| RGall | 退回环里最旧有效根，六处一起退（E139 的 G2SW_ringgate 那一读法） | `arm-RGall.diff`：核心窗口判法 `txg_is_inside_the_one_revolution_window` 恒真、checker 的 `OneRevolutionWindow::contains` 恒真 |
| RGalloc | 只退分配器的回收门槛（进程内与挂载重建两处），checker 与别的四处照旧取窗口 | `arm-RGalloc.diff` |
| RGchk | 只退 checker 的窗口，分配器照旧取窗口 | `arm-RGchk.diff` |
| WD | 攻方提：门槛 = max(F_生效, min(一圈窗口里最旧有效根, 每块盘上最新有效根取最小))；只动回收门槛那几处（分配器进程内、挂载重建、checker I-3.1 丁-defer），候选过滤照旧取窗口 | **只在模型里**，没写进实现 |
| WL | 窗口长度随跳过的 txg 数加长 | 没实现，推的 |

目标形态（D23 第 399 行，`.claude/kb/decisions/23-journal的角色与格式.md:399`「根槽写失败重发时 checkpoint_txg 推进一格再发」）今天没实现（`.claude/kb/decisions/23-journal的角色与格式.md:402`「一次发布失败之后把它冻结，下一次发布之前先逐字节原样重发它」），只在模型里建：事件 X = 一次根槽写失败、推进一格再发；Y = 同一次处置里连失败两次（k_tol = 2 之内）；D = 盘 1 的根槽持续写不进（txg 落在区 1 的那次失败一次、推进再发）。

### 判（量过 / 推的逐格标）

| 候选 | 今天实现 K1 族（副本） | 目标形态（模型） | 重开窗口要关的那一形 | 同取窗口的六处 | 代价 |
|---|---|---|---|---|---|
| G | 0（量过：290 段快照、层 0 3 段 65 640 状态） | **丢一块盘读坏**（量过，模型）：PXYC 全枚举 L = 7 `lone0=35 lone1=1009` / 337 408 状态；PXYR `lone0=38 lone1=1543` / 356 352；D 持续失败 L = 120 `lone1=224` / 240 | 不适用（就是窗口） | 不动 | 基线：S = 4 挂载之后已分配 20–136（量过）；模型 max_held 22 不随 L 长 |
| RGall | 0（量过：290 段快照、层 0 1 段 20 511 状态 `violations=0`） | 0（量过，模型） | **不重开**：窗口外的根回到候选集，副本上全读得对（量过，`v-RGall.log` 读坏 0）；I-3.1 不红（量过） | 六处一起退（实现一处谓词 + checker 一处，量过）；抬 F 的上限、读不出的槽那两处这一族里没走到（推的） | 挂载之后已分配到 166（G 同段 26，S = 4 k = 13，量过）；模型里盘 1 持续写不进时 max_held 随 L 线性涨（L = 15 / 30 / 60 / 120：50 / 80 / 140 / 260），没有上界 |
| RGalloc | — | — | **重开**：I-3.1 误红，S = 4 每个 PRE 112 / 120 段（全部 k ≥ 1 的段）、S = 8 45 / 50 段（量过） | — | — |
| RGchk | — | — | **重开**：I-2.1、I-3.1、I-3.10、I-4.8、I-7.4 红，3 / 3 段（量过） | — | — |
| WD | 没在实现上跑 | 0（量过，模型：PXYC、PXYR、DP、KC 各族） | 推的：K1 洞那一槽里上一圈的根不是哪块盘上最新的根，照旧离开门槛，与 G 同放；只改分配器、不改 checker I-3.1 时，在 WD 与 G 不同的那几格（K1 连崩挂载途中、目标形态 D 族）I-3.1 误红，与 RGalloc 同理（推的） | 三处（分配器进程内、挂载重建、checker I-3.1 丁-defer）；候选过滤、影子账、回退候选集、抬 F 上限不动（推的） | 模型 max_held：PXYC 22（G 22、RG 56）；D 持续失败 32 / 62 / 122 / 242，也没有上界（量过，模型） |
| WL | — | 推的：每跳一个 txg 窗口多一格，正好把那一槽上一圈的根收回来，判法与 RG 同 | 推的同 RG | 六处 | 推的同 RG |

### 判的出处（整行抄自产物）

目标形态上 G 读坏、RG 与 WD 不读坏（`results/model-a.log`）：
```
S=4 L=7 alphabet=PXYC U=2 mode=all arm=G histories=16384 states=337408 fake=0 lone0=35 lone1=1009 newest=0 states_with_valid_root_outside_window=259401 max_held=22
    lone0_sample=XYYYYPP@5P|crash-in-29
S=4 L=7 alphabet=PXYC U=2 mode=all arm=RG histories=16384 states=337408 fake=0 lone0=0 lone1=0 newest=0 states_with_valid_root_outside_window=259401 max_held=56
S=4 L=7 alphabet=PXYC U=2 mode=all arm=WD histories=16384 states=337408 fake=0 lone0=0 lone1=0 newest=0 states_with_valid_root_outside_window=259401 max_held=22
```
今天的字母表（P 发布、K 写行崩、C 崩了再挂、R 记录落了根没落就崩）全枚举，三臂都 0：
```
S=4 L=7 alphabet=PKCR U=2 mode=all arm=G histories=6528 states=196678 fake=0 lone0=0 lone1=0 newest=0 states_with_valid_root_outside_window=145652 max_held=24
```
模型的判法会红：必红对照臂 ALL（释放了就回收）在同一族上全红（`results/model-a.log`）：
```
S=4 L=7 alphabet=PXYC U=2 mode=all arm=ALL histories=16384 states=337408 fake=337408 lone0=61184 lone1=162048 newest=0 states_with_valid_root_outside_window=259401 max_held=0
```S = 8 随机 2000 段 PXYCR 上 G 也是 0（`results/model-b.log`），S 大时要更多次失败才把一块盘的最新根挤出窗口，L = 24 不够（没扫到更长）。

RGalloc 误红（`results/b1-RGalloc-pre0.log` 里 `checker=` 一列数出来全是 `[I-3.1]`，772 行）、RGchk 误红（`results/v-RGchk.log`）：
```
     26 checker=[I-2.1+I-3.1+I-3.10+I-4.8+I-7.4]
      7 checker=[I-3.1]
```
（这两行是 `grep -o "checker=\[[^]]*\]" out/v-RGchk.log | sort | uniq -c` 的原样输出。）

## 打中之后的四句

### 打中一：目标形态上 G 丢一块盘之后读坏（模型）

1. **分不分辨臂**：分辨。同一批历史 G `lone1=1009`、RG 与 WD 都是 0。
2. **系统当时看不看得到判别它的东西**：看得到。分配器的根环表知道每一槽住着哪条根、区域归哪块盘；挂载重建时已经算每块盘上最新有效根（`crates/singlefs-core/src/mount.rs:1871`「let floor_released_immediately = match newest_valid_root_txg_on_every_device(」），进程内没用它。
3. **满足判据字面的哪一个分句**：「回退之后读不出原来的数据（不可恢复）」，前提是把「回退」读成丢一块盘之后按盘退到剩下那块盘上最新的根（E139 的「按盘回退」，D16 已定项 1「正常卸载」那一行写的就是这一读法：`.claude/kb/decisions/16-发布语义.md:42`「恢复择根不看 F，丢一块盘时择的是剩下那块盘上最新的根」）。若主 agent 只把「回退」读成管理员回退，这一格不落在判据字面里（模型里 `fake` 恒 0），要改判据才能判它——这一句交主 agent 定。另一处要主 agent 定的：模型里 lone 不算 journal 重放；副本上盘 1 单独恢复时没有重放（`v-G.log` 里 `lost0:ok@1:22`，同实例 23、24 的记录在却停在 22），盘 0 单独时有没有重放没分开量，所以 `lone0` 偏保守、`lone1` 与副本相容。
4. **跑前条款给的改法在打中的格上还中不中**：跑前条款没给改法。W3 列的几个：RG 在这几格上 0（模型量过）；WD 0（模型量过）；WL 推的同 RG；G 原样就是被打中的那个。

### 打中二：只改一处重开 I-3.1 误红（副本）

1. 分辨臂：分辨。RGalloc、RGchk 红，G、RGall 同几段不红。
2. 判别子看得到：checker 读的是同一张盘面，红的原因是两边门槛不一。
3. 分句：「只改一处时 checker 的 I-3.1 在合法盘面上误红」逐字成立（RGalloc 只红 I-3.1；RGchk 红里含 I-3.1）。
4. 改法：两边一起改（RGall）这几格全绿，量过。

### 没算打中的：今天实现上 G 的落点政策依赖（W1-d）

模型里 K1 连崩族 G `lone1=32`（S = 4），但模型不是今天的实现：今天的实现在副本上同形（K = 11、12、13）层 0 全 ok。按判据它不让 G 出局；它说明 G 在今天站住靠的是落点政策（推的），落点政策一改（例如写行那一段聚簇段装不下暖机、回落到别处），这一形要重判。

## 我提的改法（WD）

WD：回收门槛 = max(F_生效, min(一圈窗口里最旧有效根, 每块盘上最新有效根取最小))。**只在我的模型上量过、被攻过零轮。** 它补的是窗口没管的那一件：一块盘上最新的有效根落在窗口外时，丢另一块盘之后恢复要择它，窗口却已许可它引用的单元被复用。代价：一块盘持续落不了根时与 RG 一样没有上界（模型 L = 120 时 242），差别在 K1 洞那一槽的上一圈根照旧放（推的）。

几个改法各修哪一格：

| 格 | G | RGall | WD | WL |
|---|---|---|---|---|
| 今天实现 K1 族，副本 | 0（量过） | 0（量过） | 没实现（推的同 G 放，同 RG 安全） | 推的 |
| 目标形态丢一块盘读坏，模型 | 中（量过） | 不中（量过） | 不中（量过） | 推的不中 |
| K1 后洞多扣，副本 | 基线（量过） | 多扣，k = 13 多 140 槽（量过） | 推的：只在挂载途中一块盘最新根落在窗口外时多扣 | 推的同 RG |
| 只改一处 I-3.1 误红 | — | 只改分配器红（量过）、只改 checker 红（量过） | 推的：只改分配器会红 | 推的 |
| 一块盘持续写不进根，空间 | 不涨，但读坏（模型量过） | 线性涨、无上界（模型量过） | 线性涨、无上界（模型量过） | 推的同 RG |

## 没打中的形状

- 今天实现上：K = 0..14（S = 4）、K ∈ {0, 1, 2, 8, 16, 22..26}（S = 8）× J ∈ {0, 1, 2, 3, 5, 8, 13, 20}（S = 8 取 {0, 1, 3, 8, 26}）× PRE ∈ {0, 1}（S = 8 只 PRE = 1）的落满快照，候选集与丢盘恢复 0 读坏；层 0 只跑了 j = 0（K = 11、12、13）；j = 1 一段约 107 万个状态，超过一次约 10⁶ 的上限，没跑（closed_form_state_count 原样：`s=4 c=24 k=11 j=1 LAYER0_SKIPPED states=1069078 cap=1`，草稿里的计数日志）。
- 窄盘（想逼写行那一段聚簇段装不下）：每盘 2–24 MiB 都在 mkfs 或第一次覆盖写就被拒（单元区起点在 17 MiB，准入要预留），32 MiB 能跑但聚簇段装得下，没逼出回落。
- 模型 S = 8 随机 2000 段 PXYCR（L = 24）：G 也 0。

## 这条腿自己的限度

- 副本上的数不是入库装置上的数；模型是我写的、被攻过零轮，F 恒 0（不建抬 F，所以造不出 E139 那类与抬 F 有关的假候选，`fake` 在三臂上恒 0）、每版单元只活一版（最坏情形）、分配取最小空闲、丢盘恢复不算 journal 重放。
- 读坏根槽只能造自证不过（内存盘），挂载那一刻设备读错的槽一格没造；被抛弃的根没造历史，只读了代码。
- 原型的候选集是我按 D16 那一行的字面算的（窗口 + ≥ F，不按实例表滤：这一族没有被抛弃的根），不是调实现里的 `rollback_candidate`；只留一条根整读时恢复会顺着同实例的记录往前施加，读出来的往往是更新的那一版（例如 `1:14=ok@1:24`），所以「候选读得对」只核到了「从这条根出发恢复读得对」。
- 层 0 用量：4 次合计 86 151 个状态（G k = 11 / 12 / 13 与 RGall k = 12），远在约 10⁷ 之内。随机跑批没用。
- 用户动作只放开了覆盖写次数与挂载前多不多一次空挂载；删除、建多个对象、回退之后先做什么，这一族没有。

## 没做什么

- 不判 W2，不判正推腿的格；不替主 agent 采纳；没在入库装置上重做。
- 没把 WD 写进实现、没在副本上量；没跑目标形态的实现（今天的代码断言 txg 逐个加一，`crates/singlefs-core/src/allocator.rs:1817`「pub fn record_root_written_by_this_process(&mut self, checkpoint_txg: CheckpointTxg) {」那一函数里，要改核心才能跑）。
- 草稿目录 `/tmp/claude-1000/reclaim-window-attack/` 里的四份仓副本（G、RGall、RGalloc、RGchk）与各自的 `target-*` 编译目录交回前删掉；日志已拷进模型目录 `results/`。
