# m3-c694-r1 攻方腿（Opus）报告（2026-10-07）

攻击面：正文第三节 K1、K2、K3。立场：给每个候选找一形丢数据、误放或仍拒；用户决定的那几步放开扫，只固定前缀与故障。
代码一律引冻结副本 `~/.local/share/singlefs/work/m3-c694-r1/tree/` 下的全路径:行号；副本上量的数注明「副本」，不是入库装置上的数。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-c694-r1-opus-model/`。复跑（仓根，前台约 3.5 分钟，快档；全量见「要主 agent 跑的命令」）：

```
SINGLEFS_REPO=$HOME/.local/share/singlefs/work/m3-c694-r1/tree SINGLEFS_TOOLS=$PWD THREADS=4 bash research/prompts/m3-c694-r1-opus-model/rerun.sh <空工作目录>
bash research/prompts/m3-c694-r1-opus-model/compare.sh <空工作目录>/summary.txt research/prompts/m3-c694-r1-opus-model/baseline/summary.txt
```

交回前在空目录里从头跑过一次：`RERUN done: …/rerun-quick2/summary.txt（233 行）`，compare 末行 `SAME 233 行`。`out/` 下是前台分批跑的全部判定行（`summarize.py`、`table.py` 从原始日志出表）。各文件 sha256：

```
c0fba79553b79f74b73e3054c16db3f8f5df8e8021a5c0e45e8e972c47acd54d  ./baseline/summary.txt
70935767fe5abe6ba7dc9febee063077d32ae708da294ac98eafe2c157437268  ./compare.sh
5f9297fa517d372655054877f05800e67a130007144becb2f03a730be54e69d9  ./make_patched.py
4757ce5a184086bd792377248a4ed2f0fe95f4b1ddd32d62aaf3610dd0df0df2  ./out/c-isolation-trace-384-5.txt
7d2c159f80de77dde0b23c72c807d53d3ae05f1dcbffe0a46682ebfa9ea77086  ./out/h1-grid-lines.txt
d2610a10d7ff98b99f85a4c6f63e1ab4cd9c2a17725b15bbca23e565ecab6643  ./out/h1-grid-steps.txt
da735097f63814a3a3108914b923417b209dcf06bb78aa797b7d8729645983c6  ./out/h1-grid-table.txt
2f45d597b2aafc6e255a6c2e865326721ee490082f17fc3a71034251779b9838  ./out/h2-transient.txt
b2be84d41e885df7d1b7869a1ed383d9ba6958c2e91112902b5548fec06b2d55  ./out/h5-crashmount-hide-record.txt
104f61a56c1ef5eb53e2f766ba7383bf0e1563c7604b2df6d6ff263257eb674d  ./out/h5-crashmount.txt
fceb98bf7a3e8fd57d8abbe6f5e72ec1fb4303c633fb8d24c7439a7ef161a9da  ./out/k3-c695.txt
2dd1792f66b861c7faf241c1fbc0bf02be8906630095ff0b64faa5a101f3d949  ./probe/Cargo.toml
d5849101446581fb3ef2a1f1642dbc142cd7cc6ed785ace1fc668ff2700dbde7  ./probe/main.rs
aeaafbb559c6b3ab197506be5f978fa7d87159297a0f990ec644c2d77c9bf232  ./rerun.sh
eb2500f9c25c372bbf4bd3200a96bd1dea0cce5cfaa0cac7083750fa3ded13e1  ./summarize.py
2ebca8c65f28ece18a10dd0d54888a6b43e862c49367686e84c807968a70e5b9  ./table.py
```

## 各格判定一览

见下面「判定一览表」。

### 判定一览表

臂的定义（副本里由环境变量选，补丁 `research/prompts/m3-c694-r1-opus-model/make_patched.py`）：

| 名字 | 定义 | 落点 |
|---|---|---|
| today | 冻结副本原样 | — |
| A（U1 裸跳过） | 影子账读一条被抛弃根的账与映射现算、重读仍不成时，不拒、跳过这条根（计数照记），三个入口（挂载、准入抬 F、卸载）同改 | `isolate_slots_referenced_only_by_abandoned_roots` 交错那一支 |
| B（U1 有证据才跳过，我提的） | 同一处，只有当那条根的树表落点在现行账里各有一条覆盖它、代 > 那条根 txg 的记录（新时间线已经在那一槽写过）才跳过，否则照今天拒 | 同上 |
| C（挂载那一刻补隔离，我提的） | 可写挂载重建分配器之后，把环里读得出、txg 高于施加之后那一版、没施加的 journal 记录点名的单元与四条新根指针的落点在内存里隔离（现行账里仍分配着的不隔），这次挂载内不清 | `mount_writable_with_every_test_only_switch` 里 `rebuilt?` 之后 |
| U2（拒得合法、之后某次卸载做成） | 正文点到的；救法我只推了「卸载推空发布到那条根离开根环」，没实现 | — |
| K3-R / K3-S | `open_pool_for_read` 先判树表 0 条再读映射（R）；只在映射根是空指针时先判（S） | `mounted_read.rs` 第 376 行之前 |

| 格 | 判定 | 依据（副本上量的，都是攻方原型，被攻过零轮） |
|---|---|---|
| K1-1 这几份盘面上卸载该不该做成 | 打中 U2 的「拒有东西要护」这半句：拒的那一刻被抛弃的根 R 已经读不出，今天在别的起点上照样让卸载做成 | today 42 个 (宽, k) 起点里 6 个拒；其余 36 个里 33 个在某一步「卸载做成而 R 的可读性探针读不出」（见 K1 节） |
| K1-2 不护会不会让别的一形丢数据或误放 | 举出一形，但它在 A、B、today 上一样中：取新实例那次挂载写行之前再断一次电，恢复落到 R 上读不出（池挂不上） | 层 0 枚举取新实例那次挂载的写流，today / A / B 各 4 个状态 `走读失败：UnitUnreadable { slot: SlotNumber(1542) }`，C 0 个 |
| K1-3 D16 已定项 1 那一行与 D23 已定项 14 乙-配置续各管哪半、哪半没人管 | 没人管的那半：R 的根槽挂载那一刻读回全零、系统配置又没见证到 R 时，R 这次发布**新分配**的单元在新实例眼里是空闲槽，回收门槛管不到 | 见 K1 节；量过：W=384 k=3 起点第一次覆盖写就写掉 R 的 2 个落点 |
| K2 A | 今天拒的 6 个 (宽, k) 起点（每个 4 个根槽落盘的崩溃状态、挂载之后覆盖写 0..14 次每一步）上卸载与再挂全做成；H2 里放过「R 完好、账瞬时读不出」的今天拒得对的盘面 36 行（每行 4 个崩溃状态）；这 36 行里没量到 R 在环里时被写掉（没打中误放，一次抽样）；K1-2 那一形照中 | out/h1-grid-table.txt、out/h2-transient.txt、out/h5-crashmount.txt |
| K2 B | 与 A 在 H1 全部 (宽, k) 上逐行相同；H2 里那 36 行全照今天拒、撤读错之后做成；K1-2 那一形照中；能想到的「仍拒」形：账两槽持续坏（推的，没造） | 同上 |
| K2 C | 6 种拒的起点形全做成、R 一直读得出、checker 不红；K1-2 那一形清零；**打中一形**：R 的 journal 记录也一起读回全零时 C 看不见 R，K1-2 那一形照中（9 个状态）；代价：窄盘上 ENOSPC 有的起点提前（240 槽 k=6 从第 7 次提前到第 2 次） | out/h5-crashmount-hide-record.txt、out/h1-grid-table.txt |
| K2 U2 | 推的：S = 8 时离开根环最多再发 24 次；今天这些起点的会话写在走到之前就被同一个错或 ENOSPC 拒；K1-2 那一形同样中 | 推的 |
| K3 | 恢复（冷启动走读）对树表 0 条那一版先判树表、报 NoFile，只读挂载先读映射、报 UnitUnreadable 槽 0，两条路次序不同；R、S 在 4 种宽上把只读挂载改成 `TreeTableHasNoEntries`，没找到一版读错；但单盘幸存那一形里 txg 3 那次发布的 10 个点名单元在幸存盘上都读得出，「报 0 条」把一个读得出的已确认版本报成空池 | out/k3-c695.txt |
| 顺手 | 丢 1 号盘、剩 0 号盘时 `recover` 报 I-1.1（分配记录树几何按在场的盘算），只读挂载却做成 | out/k3-c695.txt |

## K1　C694：对的世界

### 装置（副本，攻方原型）

起点照调查员草稿的造法自己重写（`research/prompts/m3-c694-r1-opus-model/probe/main.rs` 的 `record_one_overwrite` / `abandoning_mount`）：第一个文件之后会话覆盖写 k 次（k = 0..10，宽 384 / 256 / 240 / 4 GiB），在盘面拷贝上录一次覆盖写（内容全 0x5a），录下的写流交 `singlefs_checker_tier::crash::enumerate_layer0_selecting_versions_observing_each_state` 枚举崩溃状态（它对每个状态恢复、跑 oracle 与池级 checker）；每个状态上做「取新实例那次挂载」，这次挂载期间录的那次发布已落盘的单元写与根槽读回全 0（调查员复现那一形的故障）。之后**用户那几步放开扫**：会话覆盖写 0..14 次，每一步在拷贝上 (a) 卸载、(b) 进程退出再可写挂载接着卸载、(c) 卸载之后把实例代号更高的根槽全写零再恢复（R 的可读性探针：「新实例的根全坏了，恢复落到 R 上读不读得出」）、(d) 卸载之后的盘面跑池级 checker；再数 R 引用的落点（崩溃那一刻 R 的账里仍分配的）被写掉几个。

### K1-1　打中：拒的那一刻已经没有东西可护，today 在别的起点上照样放行同样读不出的 R

today 的 (宽, k) 起点 42 个（240 槽 k=9、256 槽 k=10 前缀上覆盖写就 ENOSPC，造不出），拒的 6 个：384 / 256 / 240 槽 k=5，240 槽 k=10，4 GiB k=4、k=9（后三个调查员的网格没取样到 k；调查员报 4 GiB 0 拒，是 k 只取了 1、2、3、5、8）。拒的那一格原样（`research/prompts/m3-c694-r1-opus-model/out/h1-grid-lines.txt` 第 3933 行）：

```
H1 W=384 k=5 arm=today root_landed=true r_in_ring_at_crash=true step=0 in_ring=true r_placements=22 damaged_before_unmount=14 unmount=Err(StillUnreadable:AccountOfAnAbandonedRoot) remount=Err(StillUnreadable:AccountOfAnAbandonedRoot) probe=- checker=[I-7.4] states=4
```

R 引用的 22 个落点在卸载之前已被写掉 14 个。不拒的起点上，第一次会话覆盖写就写掉 R 的落点、R 已读不出，而卸载照样做成（`out/h1-grid-steps.txt` 第 242–243 行）：

```
  step 0: dmg=0/22 um=Ok(3) re=Ok(unreadable=0)+unmount=Ok(3) probe=onR:read-ok chk=[] n=4
  step 1: dmg=2/22 um=Ok(2) re=Ok(unreadable=0)+unmount=Ok(3) probe=onR:FAILED(MappingStillUnreadable chk=[I-7.4] n=4
```

36 个不拒的起点里 33 个有这样的步（`out/h1-grid-table.txt` 里 today 不拒、探针含 `MappingStillUnreadable` 的行数，命令 `grep 'arm=today ' h1-grid-table.txt | grep -v 'um=Err' | grep -c MappingStillUnreadable` 出 33；剩下 3 个在 R 被写掉之前就 ENOSPC）。所以 today 的拒分辨的是「R 的树表**与**映射根都被写掉」对「只有映射根被写掉」，两边 R 都已经读不出；拒挡不住任何数据损失，只挡卸载。

四句：
- 分不分辨臂：分辨（A、B、C 在这 6 个起点上卸载做成，today 拒）。
- 系统当时看不看得到：看得到它要判的东西——账与映射读不出；看不到「这是被新时间线写掉的、还是一次读错」。B 把后者换成看得到的一样东西（现行账里有一条代更高的记录覆盖那一槽）。
- 满足判据字面哪一句：正文第五节第一条「丢数据或误放」都不是；这是登记行「判正常卸载做得成，或条款写明拒得合法、且之后的某一次卸载做得成」的前一半。按字面，U2 要「之后某一次卸载做得成」：today 在这 6 个起点上会话覆盖写被同一个错或 ENOSPC 拒（`out/h1-grid-table.txt` 末列），产品路径走不到 R 离开根环。
- 改法在这几格中不中：A、B、C 都让卸载做成；U2 不改代码时照拒。

推翻条件：拒的起点上 R 卸载那一刻的可读性探针读得出（我的探针只在卸载做成时跑，拒的那一格 R 的树表与映射根两槽都被写掉，读得出就是我的落点数错了）。

### K1-2　举出的一形：取新实例那次挂载写行根落盘之前断电，恢复落到 R 上读不出——A、B、today 同中，只有 C 不中

R 的根槽只要没被新实例覆盖，R 就是盘上 txg 最大的根。取新实例那次挂载里写行那次发布的单元写在它的根之前落盘，于是写行根落盘之前断电，恢复择到 R、而 R 的单元已被这次挂载写掉。层 0 枚举那次挂载的写流（`out/h5-crashmount.txt`）：

```
H5CLASS W=384 k=5 arm=today row_root_persisted=false onR:FAILED(UnitUnreadable) states=4
L0TALLY m1 W=384 k=5 arm=today states=4128 root_persisted=4116 violations=4 failed=4 no_file=0 file_read=4124 checker_violated=[I-2.1=4,I-4.8=4,I-7.2=4,I-7.4=4120] first_violation=走读失败：UnitUnreadable { slot: SlotNumber(1542) }（持久的写：system_configuration_slot|system_configuration_slot|unit_write|unit_write|unit_write|unit_write|unit_write|u
```

（两行在 `out/h5-crashmount.txt` 里原样；first_violation 是原型自己截到 160 字符的。）256 / 240 槽 k=5 同样各 4 个状态；384 / 240 槽 k=3（那次挂载没碰 R 的单元）0 个。A、B 的数与 today 逐行相同（A、B 只改卸载与抬 F 那一处，这次挂载看不见 R，走不到那一支）；C 的那一行：

```
L0TALLY m1 W=384 k=5 arm=C states=4128 root_persisted=4116 violations=0 failed=0 no_file=0 file_read=4128 checker_violated=[] first_violation=-
```

这一形是「丢数据」：所选根读不出，恢复走读失败、可写与只读挂载都起不来，P（txg 8，最后一次确认过的版本）的数据在盘上完好却够不着；故障只有一次瞬时读错，断电是普通断电。它回答 K1 第二问「不护会不会让别的某一形丢数据」——会，但病根在挂载那一刻、不在卸载那一刻，所以它不分辨 U1 与 U2（A、B、today、U2 同中），只分辨 C。

四句：分不分辨臂——只分 C 与其余；系统当时看得到吗——C 看得到的是读得出的那条 journal 记录（它让 first_txg 跳到 10，`crates/singlefs-core/src/mount.rs` 第 1297 行 `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))`），记录也读不出时谁都看不到（下面 K2 C 那一节）；字面——正文第五节第一条「丢数据」；改法——A、B、U2 都碰不到这几格。

### K1-3　两条条款各管哪半

- D16 已定项 1「根槽这一次读坏」那一行（`.claude/kb/decisions/16-发布语义.md` 第 37 行）管的是**回收门槛**：读回全零的槽「回收门槛三处……钉在它可能住过的最旧 txg」。回收门槛只管「已释放」的槽（`crates/singlefs-core/src/mount.rs` 第 2211、2213 行 `reclaim_released_up_to`）。R 那次发布新分配的单元在所选那一版 P 的账里根本没有记录，是空闲槽，门槛钉在哪都罩不到。
- D23 已定项 14 乙-配置续（`.claude/kb/decisions/23-journal的角色与格式.md` 第 392 行）管的是**系统配置见证过的**更新状态：R 的发布崩在根槽落盘之后、系统配置轮换之前，没见证，不拒（我的枚举里系统配置轮换落了一份的 5 个状态全被拒成 `PublishWitnessedBySystemConfiguration…`，`out/h1-grid-lines.txt` 第 3932 行）。
- 没人管的那半：系统配置没见证到、挂载那一刻根槽读回全零（或读坏）的那条根，它这次发布**新分配**的单元。影子账只在读得出的被抛弃根上算（`crates/singlefs-core/src/mount.rs` 第 1475 行那段注释所在的函数），挂载那一刻读不出就一个都不隔，到这个进程第一次推抬 F 才重算。K1-1 的 W=384 k=3 那一格说明第一次覆盖写（没推抬 F）就写掉了。C554 那一笔（`.claude/kb/checks-owed.md` 第 426 行）记过「同一形在被抛弃的根所在的槽挂载那一刻读不出……照样出现」，这里补的是：不必等新实例的根全坏，取新实例那次挂载里一次普通断电就落到 R 上（K1-2）。

## K2　C694：候选与各自打中的形

### 候选 A（U1 裸跳过）

- 复现的那几个起点：6 个拒的 (宽, k) 起点全做成，再挂也做成（`out/h1-grid-table.txt` 里 `arm=A` 那几行，`各步卸载=['Ok(2)', 'Ok(3)']`）。今天不拒的 36 个起点上 A 与 today 逐行相同（两份表去掉臂名之后 `diff` 只差那 6 行）。
- 放过今天拒得对的盘面：H2（R 在挂载时读得出、账完好，再可写挂载一次让影子账隔离它，之后会话覆盖写 0..8 次，每一步两种瞬时读错：卸载那一刻 R 的树表与映射根两槽读报错；或下一次覆盖写那一次调用期间读报错、之后无故障覆盖写 2 次再卸载）。today 与 B 在这两种读错下都拒、撤读错重试做成；A 放过 36 行（9 行是推抬 F 的那次覆盖写、27 行是卸载）。例（`out/h2-transient.txt` 第 72、110 行）：

```
H2 W=240 k=3 arm=A m2_isolated=28 step=4 (w)faulty_overwrite=Ok(raises=1)[in_ring=true dmg=0 chk=] then=Ok(raises=0)[in_ring=true dmg=0 chk=],Ok(raises=1)[in_ring=true dmg=0 chk=] unmount=Ok(2) damaged_after=0 probe=onR:read-ok states=4
H2 W=240 k=3 arm=today m2_isolated=28 step=4 (w)faulty_overwrite=Err(FloorRaiseFailedWhilePushingForSpace+Account)[in_ring=true dmg=0 chk=] then=Ok(raises=1)[in_ring=true dmg=0 chk=],Ok(raises=0)[in_ring=true dmg=0 chk=] unmount=Ok(2) damaged_after=0 probe=onR:read-ok states=4
```

  放过之后 R 在环里时被写掉的落点：36 行里 0 个（`in_ring=true dmg=` 后面非 0 的，三臂都是 0 处）。R 的落点有 2 个被写掉的那几行都是 R 已离开根环之后（`[in_ring=false dmg=2 …]`），合法复用。这一格对 A 是**没打中误放**，一次抽样、取样只有 240 槽 k=2、3、4 三个起点；推的危险面：抬 F 把候选集缩小，原来被一条低于新 F 的候选根豁免、现行账里已释放的那几个 R 的槽，这一次要补隔离（`crates/singlefs-core/src/mount.rs` 第 3510 行那一段），A 跳过就不补，它们回收之后可发出去。挂载时的隔离位不撤，所以只有这一类槽在险里；我没造出它们被发出去的历史。
- K1-2 那一形：照中（4 个状态，与 today 相同）。
- 四句：分辨 A 与 today / B；系统当时看不到「瞬时读错」与「已被写掉」的差别（A 不看）；字面上是登记行的「做得成」那一半；改法本身就是 A。

### 候选 B（我提的：有证据才跳过）

定义见判定一览表。证据取现行账里的记录，不读盘：R 的树表是 R 那次发布新分配的（每次带文件的发布都重写树表，`crates/singlefs-core/src/transaction.rs` 第 5328 行 `rewritten_roles.push(TransactionUnit::TreeTable);`），R 分配它时它在 R 的父版本账里不是已分配；所以现行账里出现一条代 > R.txg、覆盖那一槽的记录，只能是新时间线在 R 之后写过那一槽。

- 复现的起点：6 个全做成，与 A 在 H1 全部 (宽, k) 上逐行相同（`diff` 0 行）——证据在每一格都成立。
- H2 那 36 行：全照 today 拒，撤读错重试做成（`out/h2-transient.txt` 第 78 行 `H2 W=240 k=3 arm=B m2_isolated=28 damaged_at_m2=0 step=0 damaged_now=0 (u)unmount=Err(StillUnreadable:AccountOfAnAbandonedRoot)->retry:Ok(3) damaged_after=0 probe=onR:read-ok states=4`）。
- K1-2 那一形：照中（与 today 相同，它不碰挂载那一刻）。
- 推的「仍拒」形（没造）：R 的树表与映射根两槽在两块盘上都持续读坏（介质坏，不是被写掉）——证据不成立、照拒，而 R 本来就读不出，U2 的后半句同样走不到；另一形是新时间线在那一槽写过、后来记录被别的记录换掉而代不再 > R.txg——按今天「已释放的记录保留到再分配」（`crates/singlefs-core/src/allocator.rs` 回收只置位不删记录）推不出这一形，没造。
- 推的「误放」形：R 的树表若不是 R 新写的（与父版本共用），代 > R.txg 的「已释放」记录可以是新时间线释放了一个内容完好的槽。今天每次带文件的发布都重写树表，这一形不可达；空发布（抬 F、卸载、暖机）重写不重写树表我没逐个核，被抛弃的根若是一条空发布根，这一句要现核。

### 候选 C（我提的：挂载那一刻按没施加的记录补隔离）

- 复现的起点：在 6 个拒的起点（我跑了三种小盘，4 GiB 没跑 C）上 R 一直读得出、卸载做成、checker 不红：`out/h1-grid-table.txt` 里 `W=384 k=5 arm=C` 那行 `step0 dmg=0/22 um=Ok(2) re=Ok(unreadable=0)+unmount=Ok(3) probe=onR:read-ok chk=[] n=4`。今天不拒、但会话一写就把 R 写坏的那些起点，在 C 下探针一直读得出（三种小盘上 C 的 31 个 (宽, k) 起点里 C 的探针只出 `read-ok`、`-` 与 `NoValidRoot`；`NoValidRoot` 是卸载那几次空发布把 R 的根槽盖掉、R 离开根环之后探针把剩下的根全抹了，推的，从 txg 算术推、没逐格核）。
- K1-2 那一形：清零（上一节 C 那一行）。
- **打中 C 的一形**：R 的 journal 记录也在挂载那一刻读回全 0（故障多藏两个记录写）。first_txg 就退回 R.txg（9），C 没有记录可看，写行那次发布写掉 R 的单元、根落盘之前断电，恢复照样落到 R 上读不出（`out/h5-crashmount-hide-record.txt` 第 5、13 行）：

```
hide-record H5CLASS W=384 k=5 arm=C row_root_persisted=false onR:FAILED(UnitUnreadable) states=9
hide-record L0TALLY m1 W=384 k=5 arm=C states=4133 root_persisted=4116 violations=9 failed=9 no_file=0 file_read=4124 checker_violated=[I-2.1=9,I-4.8=9,I-7.2=9,I-7.4=9] first_violation=走读失败：UnitUnreadable { slot: SlotNumber(1542) }（持久的写：system_configuration_slot|system_configuration_slot|unit_write|unit_write|unit_write|unit_write|unit_write|u
```

  today 在这一形上同样 9 个。四句：分不分辨臂——不分（A、B、C、today、U2 同中），病根是「新实例在自己的写行根落盘之前就往一条它看不见的更新的根引用的槽里写」；系统当时看得到吗——看不到（根槽、记录、单元三样都读回全 0）；字面——第五节第一条「丢数据」；改法——A、B、C、U2 都碰不到。我能想到碰得到的改法是恢复一侧的：所选根走不通、且系统配置没见证到它（它那次发布没返回过）时退到见证到的那一版（推的，零轮，碰的是择根，D22 / C331 那一族，没实现、没量）。
- 代价（量过，副本）：取新实例那次挂载补隔离 24 个落点（12 个单元 × 2 盘，`OPUS_C694_TRACE` 打的 `OPUS_C694_C_ISOLATED placements=24`：W=384 k=5、挂载之后不写的那一趟里 16 次可写挂载（含拷贝上的再挂）7 次打 24、9 次打 0；计数在 `out/c-isolation-trace-384-5.txt`，跑法 `OPUS_C694_TRACE=1 OPUS_C694_ARM=C opus-c694-probe grid 384 5 0`）。窄盘上 ENOSPC 的那一步与 today 比有提前有推后：240 槽 k=0 13→10、k=3 10→7、k=6 7→2、256 槽 k=8 从 14 次内不拒到第 13 次拒；240 槽 k=1 10→14、k=4 7→11（`out/h1-grid-table.txt` 末列逐行比，命令见「复跑」一节的 `table.py`）。我的原型里这份隔离这次挂载内不清；若照影子账「R 离开根环时清」做，提前的那几格会少一些（推的）。

### U2（拒得合法、之后某一次卸载做成）

推的，没实现：今天 S = 8（`singlefs-format` 的 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`），R 离开根环最多再发 24 次；要让卸载自救，卸载撞上这个错时改推空发布到 R 离开根环（不带新 F、不重算影子账）。代价 ≤ 24 次空发布，每次照抬 F 那一串的写量；窄盘上今天会话在第 3、5 次覆盖写就 ENOSPC 或被同一个错拒（240 / 256 槽 k=5），这一串空发布在那里放不放得下没量。它在 K1-1 那几格上做成卸载，在 K1-2、C 被打中的那一形上一样中。

### 几个改法各修哪一格

| 格 | A | B | C | U2（推空发布自救） |
|---|---|---|---|---|
| K1-1 拒的 6 个起点卸载做成 | 修（量过） | 修（量过） | 修（量过，4 GiB 两格没跑，推的） | 修（推的） |
| H2 账瞬时读不出、R 完好 | 放过（量过；没量到误放） | 照拒（量过） | 不碰（推的：C 只在挂载那一刻起作用） | 照拒、推空发布（推的） |
| K1-2 写行根落盘前断电落到 R | 不修（量过） | 不修（量过） | 修（量过） | 不修（推的） |
| C 被打中那一形（记录也读回全 0） | 不修（推的，同 today） | 不修（推的） | 不修（量过） | 不修（推的） |
| 窄盘 ENOSPC 提前 | 不变（量过） | 不变（量过） | 有起点提前、有起点推后（量过） | 推空发布要空间（推的） |

B、C 都是我提的，只在我的模型上量过、被攻过零轮；B 与 C 叠起来（`OPUS_C694_ARM=B+C`）没跑。

## K3　C695：树表 0 条那一版

### 两条读路的次序不同（量过，副本）

- 只读挂载 `open_pool_for_read`：先读映射（`crates/singlefs-core/src/mounted_read.rs` 第 376 行 `let mapping_tree = read_code_two_tree(`），再读树表、判 0 条（第 410 行 `if tree_table.entries.is_empty() {`）。
- 冷启动走读（`recover` 经 `walk_to_file_in_the_unit_area_starting_at`）：先读树表、0 条就交 `Ok(None)`（`crates/singlefs-core/src/recovery.rs` 第 4846 行 `if tree_table.entries.is_empty() {`），映射到第一次用到才读（第 4853 行 `let central_mapping_tree = CentralMappingTreeReadOnFirstUse {`）。
- 影子账的映射现算把「映射根是空指针」明写成读不出（`crates/singlefs-core/src/mount.rs` 第 1581 行那句注释，「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」）。

同一份盘面上两条路的结局不同（`out/k3-c695.txt` 第 71 行）：

```
K3 W=4g arm=today lost=dev0 effective=Some((1, 2)) read_only=Err(Open(Walk(UnitUnreadable { slot: SlotNumber(0) }))) recover=NoFile unapplied_records_above_effective=[(1,3):named=12:survivor_copy_ok=12] checker=[I-2.1,I-4.8,I-7.3,I-7.4,I-7.6]
```

### 候选 K3-R / K3-S 攻不动的地方

四种宽上，两块盘都在时环里每条根直接打开、丢一块盘之后只读挂载：today 报 `UnitUnreadable { slot: SlotNumber(0) }` 的 16 处（(0,0)、(1,1)、(1,2) 三条根直接打开 × 4 种宽 = 12，丢 0 号盘只读挂载 4），R 下全变成 `TreeTableHasNoEntries`（`grep -c 'arm=R .*open=Err(TreeTableHasNoEntries)' k3-c695.txt` 出 12，`grep -c 'arm=R lost=dev0 .*read_only=Err(Open(TreeTableHasNoEntries))' k3-c695.txt` 出 4），带文件的 (1,3) 两臂都是 `Ok(inodes=1)`（各 4 处）。树表 0 条 ⇔ 这一版没有文件：今天带文件的发布都重写树表并登记树（`crates/singlefs-core/src/transaction.rs` 第 5328 行），我造不出「树表 0 条而有文件」或「映射根是空指针而树表有条目」的合法版本，R、S 在这两组上读错的形没打中（取样只有第一个文件之后、覆盖写 0 次这一种历史，四种宽）。

### 打中「报 TreeTableHasNoEntries 就是对的结局」的一形

丢 0 号盘那一份上，恢复落在 (1,2)，而 txg 3 那次发布（第一个文件，fsync 返回过）的记录在幸存盘上读得出、它点名的单元在幸存盘上那一份全部校验和对得上（上面那行 `(1,3):named=12:survivor_copy_ok=12`；三种小盘是 `named=10:survivor_copy_ok=10`）。不施加它是 D23 已定项 14「一个点名单元的两份……都读得出、校验和都对才施加」与 C519 用户知情认下的代价；但 K3-R / S 把这一形报成「这一版还没发布过文件版本」（`TreeTableHasNoEntries` 的文档原意），用户看到的是一个空池，看不到「已确认的那一版因为缺一份没施加、数据在幸存盘上」。四句：分辨 R/S 与「报降级」的那种结局（K3 候选「别的」那一格），不分辨 R 与 S；系统当时看得到（扫环就有那条记录与它的点名单元）；字面上不是丢数据或误放，是结局的读法；R、S 在这一格上都中。

## 没打中的形状

- A 的误放：H2 两种瞬时读错 × 240 槽 k = 2、3、4 × 覆盖写 0..8 次，36 行放过，R 在环里时被写掉 0 个（一次抽样，取样小；推的危险面写在 K2 A 那一节）。
- B 的误放与仍拒：H1 全部 (宽, k) × 覆盖写 0..14 次、H2 全部，没打中；推的两形写在 K2 B 那一节，没造。
- K3-R / S 读错别的版本：上一节，没打中。
- H1 里 A / B / C 在 today 不拒的起点上卸载、再挂的结局与 today 不同的：A、B 0 行；C 只差在 R 的探针与 ENOSPC 那一步。

## 顺手看到、不在这一轮判的

- 丢 1 号盘、剩 0 号盘那一份上，只读挂载读得出 txg 3 那 1 个文件，冷启动走读 `recover` 却报 I-1.1（`out/k3-c695.txt` 第 72 行）：

```
K3 W=4g arm=today lost=dev1 effective=Some((1, 3)) read_only=Ok(inodes=1) recover=Failed(InvariantViolated { invariant: "I-1.1", detail: "分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点" }) unapplied_records_above_effective=[] checker=[I-1.1,I-2.1,I-4.8,I-7.2,I-7.4,I-7.6]
```

  走读读分配记录树时按在场的盘算几何（`crates/singlefs-core/src/recovery.rs` 第 4876 行 `&AllocationRecordTreeGeometry::of_reader(reader),`，`of_reader` 取 `reader.device_identities()`），少一块盘几何就变（推的，与报错那一句 `crates/singlefs-core/src/allocation_record_tree.rs` 第 769 行对得上，没改代码验）。四种宽同样。归 C683 / 单盘幸存那一族。
- `isolate_slots_referenced_only_by_abandoned_roots` 的文档注释还写「树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载」（`crates/singlefs-core/src/mount.rs` 第 1475 行），与第 1544 行的拒不一致。

## 这条腿自己的限度

- **层 0 枚举缩过**：录的那次覆盖写的写流每段历史是 4 段、段长 [20, 2, 1, 2]（小盘）或 [24 / 28, 2, 1, 2]（4 GiB），全量 1,048,583（小盘）到 268,435,463 个状态（`counts` 子命令）。全量跑一段小盘历史前台 235 秒没跑完（`LAYER0_PARALLEL_START states=1048588 …` 之后被 `timeout` 截断，7 线程）。前台只能把多于 12 个写的段（第 0 段，录的那次发布的单元写）不展开，每段历史 13 个状态（根槽落盘的 9 个里 5 个系统配置轮换落了、挂载被乙拒，4 个就是 C694 那一形）；第 0 段展开出来的 2^20 − 1 个状态里根槽都没落盘、没有被抛弃的根，A、B 走不到改动的那一支（推的，从代码推：那一支只在有被抛弃根且账读不出时走到），C 在这些状态上会补隔离（只是代价）。取新实例那次挂载的写流（H5）全量 20,496 个状态，前台同样把 14 个写那一段不展开，跑了 4,128 / 4,133 个。缩前缩后：H1 每段历史 1,048,588 → 13；H5 每段 20,496 → 4,128。全量交主 agent（下一节）。
- 这条腿原型跑的层 0 状态合计 76,552（各日志 `L0TALLY` 行的 states 相加），加上被截断的那一次全量（至多 1,048,588）与被 `timeout` 截断的一次 H5（至多 8,235），合计不超过约 1.14 × 10⁶。
- 线程：开工 7；收到主 agent 两次消息，B 臂小盘跑完后改 5、B 臂 4 GiB 跑完后改 4，之后每条命令照 4（草稿目录 `progress.md` 记了）。
- 估时：小盘一段历史 13 个状态、覆盖写 0..14 步，每臂约 3–4 秒；4 GiB 约 18 秒（前台批跑的挂钟）。
- 4 GiB 没跑 C 臂；B+C 没跑；U2 没实现；H2 只取了 240 槽 k = 2、3、4。
- 副本上的数不是入库装置上的数；原型的探针「把实例代号更高的根槽全写零再恢复」是多重故障下的读法，只用来问 R 还读不读得出。

## 要主 agent 跑的命令（全量，没在前台跑）

工作目录：仓根。环境：`SINGLEFS_REPO=$HOME/.local/share/singlefs/work/m3-c694-r1/tree SINGLEFS_TOOLS=$PWD THREADS=<线程上限> PROFILE=full`。内存：编译 6G、跑 4G（脚本内部经包装）。命令：

```
SINGLEFS_REPO=$HOME/.local/share/singlefs/work/m3-c694-r1/tree SINGLEFS_TOOLS=$PWD THREADS=4 PROFILE=full bash research/prompts/m3-c694-r1-opus-model/rerun.sh <空工作目录>
```

它跑报告里的全部数（仍是缩过的层 0 展开），推的挂钟约 30 分钟（前台分批跑过同一组：每臂小盘约 2 分钟、4 GiB 约 3 分钟）。看 `<工作目录>/summary.txt` 里 `H5CLASS … onR:FAILED` 与 `L0TALLY m1 …` 那几行判 K1-2 与 C 被打中那一形。层 0 不缩的那一份：在上面的命令前加 `OPUS_FULL=1`（H5 每段 20,496 个状态，约 4 分钟一段；H1 每段 1,048,588 个状态再乘每个状态的挂载与 0..14 步，推的约每臂每段数小时，超过 40 分钟，要先由主 agent 定只跑哪几段，推荐 `crashmount` 的 384 / 256 / 240 槽 k=5 today 与 C、加 `OPUS_HIDE_RECORD=1` 两段）。

## 没做什么

- 没判正推那几格（K4 回扫）；没替主 agent 采纳；没跑重型测试、门禁、harness 测试目标（原型是一个工作区外加的二进制，只在副本里编）。
- B、C、U2 的条款没写；B 的「空发布根」那一句、C 的清隔离时点没核。
- 草稿目录 `~/.local/share/singlefs/work/m3-c694-r1-opus/` 里留着仓副本 `repo/`、`rerun-quick/repo/` 与编译目录 `target/`、`rerun-quick/target/`：交回之前删（见交回）。

## 引文对照（正文里点到的行号，逐处 `grep -nF` / `awk` 现查过）

代码在冻结副本 `~/.local/share/singlefs/work/m3-c694-r1/tree/` 下，kb 在仓里。

- `crates/singlefs-core/src/mount.rs:1297`：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」
- `crates/singlefs-core/src/mount.rs:1475`：「树表或分配记录树读不出、解不开的被抛弃根只计数，不拒绝挂载」
- `crates/singlefs-core/src/mount.rs:1544`：「StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot {」
- `crates/singlefs-core/src/mount.rs:1581`：「映射根是空指针的 mkfs 第 0 代、树表 0 条的一版也在内」
- `crates/singlefs-core/src/mount.rs:2211`：「allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);」
- `crates/singlefs-core/src/mount.rs:2213`：「allocator.reclaim_released_up_to(」
- `crates/singlefs-core/src/mount.rs:3510`：「let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {」
- `crates/singlefs-core/src/mounted_read.rs:376`：「let mapping_tree = read_code_two_tree(」
- `crates/singlefs-core/src/mounted_read.rs:410`：「if tree_table.entries.is_empty() {」
- `crates/singlefs-core/src/recovery.rs:4846`：「if tree_table.entries.is_empty() {」
- `crates/singlefs-core/src/recovery.rs:4853`：「let central_mapping_tree = CentralMappingTreeReadOnFirstUse {」
- `crates/singlefs-core/src/recovery.rs:4876`：「&AllocationRecordTreeGeometry::of_reader(reader),」
- `crates/singlefs-core/src/allocation_record_tree.rs:769`：「分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点」
- `crates/singlefs-core/src/transaction.rs:5328`：「rewritten_roles.push(TransactionUnit::TreeTable);」
- `crates/singlefs-format/src/lib.rs:279`：「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」
- `.claude/kb/decisions/16-发布语义.md:37`：「回收门槛三处（挂载时含 m 取不到那一支、进程内、抬 F 那一遍）钉在它可能住过的最旧 txg」
- `.claude/kb/decisions/23-journal的角色与格式.md:390`：「被抛弃根的账（树表或分配记录树）读不出时不跳过」
- `.claude/kb/decisions/23-journal的角色与格式.md:392`：「可写挂载读到的更新状态读不出」
- `.claude/kb/decisions/23-journal的角色与格式.md:362`：「一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加」
- `.claude/kb/checks-owed.md:426`：「同一形在被抛弃的根所在的槽挂载那一刻读不出、可写挂载里重读也读不出时照样出现」
