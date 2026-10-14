# m3-batch50-code-r1 云端攻方第二次抽样（s2）报告

轮名 m3-batch50-code-r1（代码轮第一轮，删除线 W3 + 第 50 批），攻方腿第二次抽样。代码行号一律按冻结副本 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree/`；量的数都在我的仓副本 `~/.local/share/singlefs/work/m3-batch50-code-r1-opus-s2/repo/` 上（副本上的数，不算入库装置上的数）。

## 复跑命令与模型目录各文件的 sha256

模型目录 `research/prompts/m3-batch50-code-r1-opus-s2-model/`。复跑按阶段（每阶段一条前台命令、4 分钟内），工作目录给一个空目录；被判的冻结副本默认取 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree`（环境变量 `SINGLEFS_S2_TREE` 可改），只拷不改：

```
M=<模型目录的副本>; WD=<空目录>; export SINGLEFS_REPO=/home/fy5090/code/singlefs
for st in prepare build k5 k3 random random-knobs random-churn random-abandoned control scan \
          k7-m3-left-first k7-m3-floor-116 k7-m3-floor-118 k7-r3-carried-from-new-position \
          k7-checker-i934-no-lineage-filter k7-checker-i932-no-birth-before-death k7-checker-i932-no-death-at-most-version \
          direction crash compare; do bash $M/rerun.sh $WD $st || break; done
```

（上面是次序；我交回前在空目录 `~/.local/share/singlefs/work/m3-batch50-code-r1-opus-s2/rerun-check/` 里逐阶段各起一条命令从头跑通（跑完连同里面的仓副本与编译目录删了，k7 那几份日志拷在 `kept-logs/`），`compare` 那一步 22 份产物全是「同」，原样在草稿目录 `rerun-compare.txt`；跑通之后 `apply.py` 只加了一行 `sys.dont_write_bytecode = True`（不在模型目录里留 `__pycache__`），改法本身没动，我在空副本上重放过 `noret` 那一组、与复跑用的那份逐字节相同。每阶段耗时：k7 每条约 3 分钟、random 那几批约 2～2.5 分钟、crash 约 3 分钟，其余一分钟内。）`compare` 把 `outputs/` 下每份按行排序、去掉 `elapsed_seconds` 与 `finished in` 那一段再比。

```
fee319911a042033295eb1f0d9fe484da5d5bc2462bb0ebd4434da6b3278622b  apply.py
90de0e23e3667bc11631b02976728e8ef94129b2c3d1d96862dfc0de34753647  crash-proto/Cargo.toml
1079a953e9405cd77ba115d2ab5ab5219eb9cbc51fa86acf62ab57bf9322125b  crash-proto/src/main.rs
68ac6f01a0939e9cbe1e7933b1a1a60cd82ead615c5b7808e24e89043176eafa  mutations.py
8306d8e27afb4e5f31f50be95075f473956f1ae4f2bf109853d83ea8b03bd660  opus_s2_probe_direction.rs
4b2ca63ce3049c89082112ae1d5c99c145ce010053df4ec9304037c0e1d9be15  opus_s2_probe_edge_unlink.rs
a058ee95b2bd0313d30c1d90c735a3988eaddea9aefc05fee4a4302081ea6591  opus_s2_probe_first_file_tree_identifier_watermark.rs
0d5f5cfd3dcfe29092e1038bec2c2204fbb346c9d32bc8f2c7bd90f79ffd9e1d  opus_s2_probe_random_leaves_abandoned.rs
1680bd06a3423eb11fe72bc4ff41b95cec07c995ec57d0c739f05ad9f0719831  opus_s2_probe_random_leaves.rs
3b65d6c2abb9e0abec7ae148908c7bb7b7426345b7536d45db8525557334fcc0  outputs/abandoned-no-delete-control.txt
18eae3e382e92cd8930604b2c74953793320c3b329de9eccbdc439c8d3bcf06e  outputs/abandoned-suffix-scan.txt
c5c8f59b937a1e2cc2bb893b829cb0686a64d7e2b6ae731a655f06248c9fc084  outputs/crash-plans-over-budget.txt
f0e953a5ef51463d6e76dbcbdd807b9230cef4cace33a1e4f2ad2a661070101b  outputs/crash-rollback-after-absorb.txt
c0c1f606be2332438a112993a960ead97885c56a20b19c1a9b22520579fff837  outputs/first-file-watermark.txt
59c4059ff870075ac398a8b28f437acafbf3ca314721ce188ad3a9acd0101ed0  outputs/k5-edge-main-new232.txt
b24cb093a9dc3a33803ccc9577e24a5e1a6b149d0e680d6b407161611417e71f  outputs/k5-edge-main-new233.txt
532944d715c75f8be20c026820b17a15d48b448d6380abcfac0981b8d3377ca3  outputs/k5-edge-noret-new232.txt
063c91d5788b9a3949e69aadc3db7a28f751aa1005619c7d8d25037c6dc5d776  outputs/k5-edge-noret-new233.txt
c93ef8110b5411bccff1ddc842ccf7c4a5dc6d590d558bc83177ad76c1fbd40f  outputs/k7-direction-left-first.txt
ede549787585974e2792bb5a449811c1785e65928934a42efc593d8e2ebb05b6  outputs/k7-direction-main.txt
6b081a9edfef337ccfb9c9f3bc42f972d6735df6ff9db97e5da75c252174ac56  outputs/k7-mutations.txt
c2de3b6090dafc76487538560308797123d3bb6b930ad9a97dc0af94108b4bfc  outputs/main-badimages.txt
e5046f1961816158f06a38b0876083e5703f984f6f660d16e8ceb8b9a22af1cd  outputs/random-leaves-abandoned-hist.txt
1a6382051892d7abd7af36e29eb7d5adb98f4ec1fa73862334b3a5b186a50a18  outputs/random-leaves-abandoned-red.txt
338d0e1b6c37f006abac57e797e3abb8cb763aefec0a9d54202178a3513556c9  outputs/random-leaves-hist.txt
cf9bb7c93deb81268995db24292294d26164d969ec7ad29c358d9060721d23b6  outputs/random-leaves-knobs-churn-hist.txt
6db0accc37dcaac8bd9d794b8dbfde9b2075303b36b6795c8a83a6530435d59b  outputs/random-leaves-knobs-churn-retirement-trees.txt
11b8993d0a53eb0c8de81bea1b7b9f4a8fddebd908159ea1ae13d6e29f0d033d  outputs/random-leaves-knobs-hist.txt
4d904fccd4fcdde8bce60ffd076749af7dc3194a962eb91ba3d8956df1002eb0  outputs/random-leaves-knobs-retirement-trees.txt
13038d3157a3ff2276652acab841174585549454101a96bf63e3534f34900cb0  outputs/random-leaves-op-counts.txt
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  outputs/random-leaves-red.txt
2afd455f3274b52bb8c7b1ae33e36974e54670aa671a914ffc52e9b869168218  rerun.sh
```

## 各格判定一览

| 格 | 判定 | 依据（详见各节） |
|---|---|---|
| K1-1 Opt2 滤法 | 没打中 | 读码与 D23 已定项 14 同一件事；1 008 段定向扫 + 70 次抛弃，I-9.29 / I-9.30 0 红（量过） |
| K1-2 五种形状 | 没打中 | 回退目标是 cur、连续两次回退、回退之后崩溃 / 抛弃量过；实例表行不全读码 |
| K2-1 / K2-2 / K2-3 | 没打中 | 四批随机历史与 65 548 个崩溃状态上恢复、I-3.13、I-9.30 0 红（量过）；C769 的代价给了一张推的坏镜像（台阶代段写晚一格 + 同格重发，② 漏报） |
| K2-4 ckpt_cost 台阶跨叶 | 没造 | 我的历史到不了 463 级台阶 |
| **K3-1 登记与旧读法** | **打中** | 补丁没动的 checker 档用例 `crash_points_tree_identifier_watermark_orphans.rs:113` 仍断言第一个文件版本水位 65548；harness 档原型照它的搭法量到 65549、树表 12 条（量过搭法那一步；那份用例本身是 checker 档，没跑） |
| K3-2 / K3-3 / K3-4 | 没打中 | 138 856 次删除类发布上自写的退役完备判据 0 红；容量开关下退役树长到 3 层；回退、抛弃之后再删 0 红（量过） |
| K4-1 / K4-2 | 没打中 | 叶结构判据 0 红；R3 照抄叶序改坏三处都红（量过）；论证一次发布至多一片身份消失（推的，量到最大 1） |
| K4-3 N1 之后的分隔 key | 不中，记射程 | 落在左片最大号与右片最小号之间的错值单张镜像分不出（判别子观测不到） |
| **K4-4 模型与胶水** | **打中（覆盖空格）** | 模型只罩删第一个文件、退役树一片容器；多片叶的 M3 在对拍里没有对照（读码） |
| K5-1 | 没打中 | 每次删除类发布至多一条退役记录（量过），W 的 6 + H_退役 罩得住 |
| **K5-2（必做）** | **乙二 读法下造不出；去掉退役项也不红** | 四条臂贴边量过：被放过的写 0、释放全做成、物理空闲比 ckpt + 挂载期多六百槽以上；甲读法两条臂都负，分不出退役项 |
| K5-3 常驻 3 槽与实占 | 式子读法下中（保守的双扣），乙二 下不中 | 开出第一片之后已分配多 1 槽 / 盘、常驻 3 槽照扣（量过两格的差） |
| K6-1 | 没打中 | 全部合法镜像 checker 0 红（抛弃那一步的 I-3.1 除外，与本轮无关） |
| K6-2 GPU 同判 | 没跑 | 读码没找到 I-9.35 父版门的分歧 |
| K6-3 | 没逐条核 | 退役五条的坏镜像各断言只红自己 |
| **K7 判别力** | **三条照绿** | 「先右后左」改成「先左后右」五份用例全绿，而产品写法走得到的一段历史上退役的是另一片叶（量过）；I-9.32 的「容器出生代 < 死亡 txg」「死亡 txg ≤ 这一版」两句 harness 档照绿（量过） |
| 顺带 | 交主 agent | 崩溃恢复抛弃最新根那一刻 I-3.1 红，不删 inode 的对照也红（量过），与删除线无关 |

## K5　准入（R2 两样都做）

### K5-2（必做）贴边建满后 unlink 不超卖

**先给读法（记作乙二）。** 删除类发布不判式子、只判落点，它们新写的槽由环窗口兜：`.claude/kb/decisions/03-空间分配.md@已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况`：「删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判」。所以删除之后式子里的「可用」是负的不算超卖——那正是环窗口在被用。我把「不超卖」读成三样都成立：
1. 贴边之后被准入放过的写入类发布写得下（不在落点那一判被拒、不在落盘途中报空间不够）；
2. 删除类发布与它之后的释放（unlink 一个大文件）做得成，不被落点拒；
3. 每次发布之后，物理空闲（容量 − 已分配 − 隔离 − 不可回收）不小于 checkpoint 保留池 ÷ 副本数 + 挂载期承诺量（这两项写不出去就发不了布）。

实现员的甲（转过一圈根环之后的稳态上式子「可用 ≥ 0」）我不取，理由是量出来的：在我能造到的「稳态」（正常卸载把 F 抬到现行 txg、再可写挂载）上，式子可用在贴边格里恒为负，退役项在不在都一样负，判别子分不出退役那几项（下表 甲 那一列）。

**造法**（`opus_s2_probe_edge_unlink.rs`，内存稀疏盘，384 槽那一档小盘几何、两块盘）：mkfs → 第一个文件 → 建 `new` 个空 inode → 顺序写第一个文件到 `du` 个数据单元（准入判着，du 步长 1 扫到第一个被拒的格）→ unlink `victim`（会话一次调用 = 意图 + 各批）→ 正常卸载 + 可写挂载（准入判着）→ 建一个 inode（准入判着）→ unlink 第一个文件（贴边之后的释放）→ 池级 checker 判整份镜像。`new=233, victim=2`：234 条记录装成 [1..=233]、[234] 两片，删 2 号左片吸收右片、开出退役树（第一条退役记录）；`new=232` 是对照臂：233 条一片，删 2 号不并、不写退役记录。两条臂各在实现上（主臂）与「去掉退役那几项」的副本上（标签 6 的 2 + H_退役 + 1 与 35 × n_记录、W 的 6 + H_退役、ckpt_cost 的退役项与它的映射几把全置 0，差异在草稿目录 `noret.diff`）各扫一遍。

贴边那一格（主臂、有退役记录）原样（`research/prompts/m3-batch50-code-r1-opus-s2-model/outputs/k5-edge-main-new233.txt` 第 3、4 行）：

```
name=edge new=233 pre=0:none victim=2 du=114 | write: avail=0.915 cap=1391 alloc=267 iso=0 mount=200 window=900 label6=7.085 ckpt=16.000 free=1124 txg=5 F=0 | unlink=ok(versions=2,raises=0) leaves 2->1 retired=1: avail=-26.081 cap=1391 alloc=294 iso=0 mount=200 window=900 label6=7.081 ckpt=16.000 free=1097 txg=7 F=0 | quiesce=[]  remount=mounted retired=1: avail=-23.081 cap=1391 alloc=291 iso=0 mount=200 window=900 label6=7.081 ckpt=16.000 free=1100 txg=13 F=7 | create=refused(NoSpaceAfterRaisingTheFloor(NoSpaceAfter) | release=ok(versions=2,raises=0): avail=-50.076 cap=1391 alloc=318 iso=0 mount=200 window=900 label6=7.076 ckpt=16.000 free=1073 txg=15 F=7 | checker_red=0
name=edge new=233 pre=0 victim=2 du=115 write=refused(NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { la) avail=241.915 cap=1391 alloc=26 iso=0 mount=200 window=900 label6=7.085 ckpt=16.000 free=1365 txg=4 F=0
```

去掉退役那几项的副本上同一条臂的贴边格（`outputs/k5-edge-noret-new233.txt` 第 4、5 行）：

```
name=edge new=233 pre=0:none victim=2 du=209 | write: avail=1.414 cap=1391 alloc=459 iso=0 mount=164 window=750 label6=3.586 ckpt=13.000 free=932 txg=5 F=0 | unlink=ok(versions=2,raises=0) leaves 2->1 retired=1: avail=-25.583 cap=1391 alloc=486 iso=0 mount=164 window=750 label6=3.583 ckpt=13.000 free=905 txg=7 F=0 | quiesce=[]  remount=mounted retired=1: avail=-22.583 cap=1391 alloc=483 iso=0 mount=164 window=750 label6=3.583 ckpt=13.000 free=908 txg=13 F=7 | create=refused(NoSpaceAfterRaisingTheFloor(NoSpaceAfter) | release=ok(versions=3,raises=0): avail=-62.581 cap=1391 alloc=523 iso=0 mount=164 window=750 label6=3.581 ckpt=13.000 free=868 txg=16 F=7 | checker_red=0
name=edge new=233 pre=0 victim=2 du=210 write=refused(NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { la) avail=434.414 cap=1391 alloc=26 iso=0 mount=164 window=750 label6=3.586 ckpt=13.000 free=1365 txg=4 F=0
```

对照臂（不写退役记录）两份的贴边格在 `outputs/k5-edge-main-new232.txt` 第 4、5 行与 `outputs/k5-edge-noret-new232.txt` 第 5、6 行。汇总（每盘、槽；「可用」取式子原值）：

| 臂 | 贴边 du | 写完可用 | 卸载再挂之后可用（甲） | 释放之后物理空闲 | ckpt ÷ 副本 + 挂载期 | 乙二 1/2/3 | checker |
|---|---|---|---|---|---|---|---|
| 主、有退役记录 | 114 | 0.915 | −23.081 | 1073 | 16 + 200 | 过 / 过 / 过 | 0 红 |
| 主、对照 | 115 | 0.919 | −24.076 | 1072 | 16 + 200 | 过 / 过 / 过 | 0 红 |
| 去退役项、有退役记录 | 209 | 1.414 | −22.583 | 868 | 13 + 164 | 过 / 过 / 过 | 0 红 |
| 去退役项、对照 | 210 | 1.417 | −23.581 | 867 | 13 + 164 | 过 / 过 / 过 | 0 红 |

（乙二 第 1 条：建 inode 在四格上都被准入拒〔`create=refused(NoSpaceAfterRaisingTheFloor`〕，放过的写一个都没有，没有「放过而写不下」；第 2 条：四格 `release=ok`；第 3 条：物理空闲比两项之和多六百槽以上。）

**结论：在乙二下造不出超卖；去掉退役那几项也不红**（量过，上表第 3、4 行）。原因是数出来的：384 槽这档盘的环窗口 900 槽 / 盘（去掉退役项 750），一次 unlink 的全部新写在这里是 25～40 槽，环窗口里 6 + H_退役 那 25 × 6 = 150 槽与常驻 3 槽在这块盘面上没有一次被够到。甲那一列在四格上全负、两个臂一样负：卸载再挂那几次空发布本身换下的固定点被候选根钉着（主臂卸载再挂之后已分配比写完多 24 槽，退役记录那一格只多 1 槽，见下），甲在这里量的是固定点的钉住，不是退役项。

**什么现象会推翻它**：在某种几何上（环窗口窄、W 的分配记录树那几层取到 min 的上界、一次发布新写接近 W）一段贴边历史里出现 `release=ERR(...)`、或被放过的写入类发布报落点拒、或物理空闲低于 ckpt ÷ 副本 + 挂载期。我只扫了 384 槽这一档、两块盘、K = 144，见「这条腿自己的限度」。

### K5-3　开出第一片退役容器之后常驻 3 槽与实占同时在账上

量到的是：同一贴边格上，删 2 号开出退役树那条臂比对照臂多占 1 槽 / 盘（主臂 du=114 卸载再挂之后已分配 291，对照臂 du=114 那一格〔`outputs/k5-edge-main-new232.txt` 第 3 行〕290；容器 2 + 根 1 − 被吸收的叶 2 = +1），而常驻 3 槽照扣不还。所以开出第一片之后，式子可用永久少 1 槽（墓碑树第一次开出时同形，是 +3 − 0）。换盘面能让可用变负——在贴边格上它本来就负（甲那一列），但在乙二下这 3 槽是多扣的、不是少扣的，物理上不缺。**判定：式子读法下「可用变负」成立，属于双扣（保守），不是超卖；乙二 下不中。** 这是读码加两格量出的差值，没另扫几何。

### K5-1　H_退役、n_记录取哪一版；一次发布至多几条退役记录

- 标签 6 用「这次之后」那一版的 H_退役 与 n_记录：「retirement_tree_internal_levels_read_from_the_root_node_header(resolved, previous),」（`crates/singlefs-core/src/transaction.rs:4409`）；W 用要接着的那一版：「.map_or(0, TransactionOutput::retirement_tree_internal_levels),」（`crates/singlefs-core/src/admission.rs:1569`）；ckpt_cost 同样取要接着的那一版：「let retirement_tree_internal_levels = file_version.retirement_tree_internal_levels();」（`crates/singlefs-core/src/admission.rs:1081`）。一次发布让退役树从只有根长到一层那一次，W 用的是长之前的 H：W 的 6 已含「根满长高多出的右半 1」，罩得住（读码，没造那一次：要 959 × 135 条退役记录，或装只供测试的容量开关）。
- 一次发布至多写几条退役记录：按 M3 的写法是 1 条（推的，论证在 K4 一节「一次发布至多一片身份消失」）。量的：两批随机历史（K4 一节）共 60 141 次删除类发布，每次「上一版叶身份 − 这一版」的最大值是 1（`outputs/random-leaves-hist.txt` 与 `outputs/random-leaves-abandoned-hist.txt` 每行的 `max_gone_in_one_publish`），没有一次两条。W 的 6 + H_退役 按至多 134 条留的余量远大于实际。

## K3　退役树与退役记录

### K3-1（打中）一份 checker 档用例还按「十二棵树、水位 65548」断言

第 50 批把新池新建文件那次的树表从 11 条加到 12 条、发号多一棵（退役树 65548），新水位变成 65549：「pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;」（`crates/singlefs-format/src/lib.rs:219`）、「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;」（`crates/singlefs-format/src/lib.rs:197`）。补丁没动的 checker 档用例 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs` 在搭流 ① 时照旧断言旧值：「(CheckpointTxg(3), 65548),」（`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:113`），失败说明「"流 ①：第一个文件版本 (1, 3) 从 mkfs 的水位 65536 发 65536..65547、新水位 65548"」（`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:114`）。

量法：那份用例是 checker 档、我不跑。我把它流 ① 的搭法（同一份 `common::parameters()`、内存盘、分配器装法、取号、暖机、第一个文件内容）原样搬进 harness 档原型 `opus_s2_probe_first_file_tree_identifier_watermark.rs`，只做到断言之前、打出那条根。原样输出（`outputs/first-file-watermark.txt`）：

```
name=first-file-watermark txg=3 tree_identifier_watermark=65549 tree_table_entries=12 checker_tier_test_expects=(3, 65548)
```

所以那份用例会在搭流那一步就 panic（推的：同一组调用在我的副本上量到 65549，用例本身没跑）。它只有一条 `#[test]`（文件里第 320 行起），开头就调 `first_file_publish_after_mkfs`，这一份整份红；实现员第 50 批两份报告「受影响的层 0 流」那一节没点它。

四句：分辨臂——不涉及臂（是用例跟不上格式常量）；被判对象看得到——看得到（常量在 `singlefs-format` 里）；满足的分句——K3-1「有没有一处还按 11 条、65548 读」；改法——把那一行改成 65549、头注「发十二棵树」改成十三棵（推的，没在副本上改了跑，因为那是 checker 档）。同一类旧说法我在 `crates/` 里按「65548」「十二棵」「十一棵」「11 条」「2200」搜过一遍，命中的别处都是对的（`highest()` = 65548 是最大号、不是水位；`crash_judge_gpu_agrees…` 文件头的「树表 11 条」说的是第 10 批那一段历史）。

### K3-2　死亡写侧：身份、条数、死亡 txg

没打中。造法：三批随机历史（`opus_s2_probe_random_leaves.rs`：建 1 / 3 / 60 / 116 / 117 / 118 / 232 / 233 / 234 / 300 个、删任意活号、成串删一片叶里的号、挂着回退到最近 30 版里任一版、进程崩溃再挂、抬 F、覆盖写；第三批另装只供测试的容量开关〔退役容器 1 条一片、容器树码 2 节点 2 条〕并加「建 233 个再从大到小全删」的成片生灭），每次删除类发布之后用我自己写的判据（与实现、checker 都不共用代码）比：上一版叶身份 − 这一版 = 死于这一版的新退役记录（集合与条数），新记录死亡 txg = 这一版 txg，退役记录以上一版的为前缀（只追加）。量到的（`outputs/random-leaves-hist.txt`、`outputs/random-leaves-knobs-hist.txt`、`outputs/random-leaves-knobs-churn-hist.txt` 各行求和）：

| 批 | 段 × 步 | 删除类发布 | 叶死亡 | 借 | 回退做成 | 退役树内部层数 | 判据红 | checker 红 |
|---|---|---|---|---|---|---|---|---|
| 产品容量 | 40 × 40 | 31 882 | 48 | 73 | 139 / 184 | 0 | 0 | 0 |
| 装开关 | 40 × 40 | 33 351 | 54 | 56 | 139 / 202 | ≤ 1 | 0 | 0 |
| 装开关 + 成片生灭 | 24 × 25 | 73 623 | 157 | 4 | 34 / 48 | 1 / 2 / 3（各 6 / 12 / 6 段） | 0 | 0 |

同一次发布里「分裂、合并、再分裂」造不出来：分裂只在建 inode 的发布、合并只在 unlink 最后一批，一次发布只做一种。「一批删掉几片、连锁吸收」也造不出：见 K4「一次发布至多一片身份消失」。

### K3-3　回退照抄 cur、崩溃恢复随所选根

没打中。上面三批里回退共做成 139 + 139 + 34 = 312 次；另两样：带崩溃恢复抛弃根的随机历史 40 段 × 40 步（`opus_s2_probe_random_leaves_abandoned.rs`；抛弃做法照 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and`、改到内存稀疏盘：最新那条根的根槽与它那条记录点名的数据与打包单元暂时读回全 0、见证它的系统配置轮换槽写回轮换前那一代，可写挂载择到前一条根，再原样写回），28 259 次删除类发布、53 次叶死亡、70 次抛弃，I-9.33 / I-9.34 / I-9.35 / I-9.29 / I-9.30 一次没红（`outputs/random-leaves-abandoned-red.txt` 里 50 行全是 I-3.1，见「顺带看到的」）；以及定向扫：前缀「建 k1 → 建 k2（最新）→ 抛弃它 → 新实例再建 k3（重发被抛弃那段的号）」固定，之后两步用户动作放开扫（回退到环里第 0..5 新、删重发的号、删前缀里的号、建 1、建 233、崩溃再挂、再抛弃），k1/k2/k3 取七组，7 × 144 = 1 008 段，checker 0 红（`outputs/abandoned-suffix-scan.txt` 的七行 `name=scan-total`）。

### K3-4　容器树一次追加多条、祖先并集

产品路径上一次发布至多一条退役记录（K4），「一次追加多条」那一支只在容量开关下、而且只在一次追加跨两片时走到。装开关那两批里退役树长到 3 层内部节点、墓碑树与谱系树同一份代码照常走，checker 0 红；一次追加两条以上那一支我没造（要一次发布身份消失两片，产品写法给不出）。读码：`previous_root_is_carried_down_unchanged` 那一判只在「长了一层、旧根下面没有容器重写」时成立，单条追加正是这一形，多条追加跨满长层时旧根照样被换下，与文档一致。

## K4　M3 合并、均分借、N1、R3

### K4-1　合并与借的判定

没打中。三批随机历史共 138 856 次删除类发布（K3-2 那张表三行之和），每次之后我的叶结构判据（每片 1..=233 条、号严格升序、号 ≥ 容器号与分隔 key、左片最大号 < 右片分隔 key ≤ 右片最小号、分隔 key 严格升序、身份不重）一次没红；借 133 次、叶死亡 259 次。

**一次发布至多一片身份消失**（推的，加上面的量：`max_gone_in_one_publish` 恒 ≤ 1）。论证：设每次发布之后「相邻两片之和都 > 233」成立（建 inode 只往最右片追加、满 233 才在末尾切出 1 条的新片，和 = 234；回退取的是更早一版，那一版本身满足）。删掉 H 片的一条之后：
- H 没删空：若右邻 R 与 H 之和 ≤ 233，H 吸收 R，幸存片之和 = 原 H + R − 1 ≥ 233，再与左右哪一片都 > 233，停；只有左邻 L 合得进时同理。两边都合得进要 L + H = H + R = 233（删之前两边都是 234），按「先右后左」H 吸收 R。一次都没并、H < 117 才借，借完两片 ≥ 117，借出那片与它右邻之和若 ≤ 233 再吸收一次，之后又回到「相邻之和 > 233」。
- H 删空：删之前 H 只有 1 条，左右邻都是 233，摘掉 H 之后那一对之和 466，不并。
所以每次至多一片身份消失；`crates/singlefs-core/src/inode_tree.rs:383` 那一支（删空之后从左邻起并）在产品写法上走到了也一次都不并。

由此两条推论（读码，没造出反例）：中间一片与最左一片删不空（它们有右邻，低于 117 就借或被并，到不了 0）；删空只发生在最右片。R3 判决第七条点的「删空最左一片」在产品写法上走不到，实现员已在 `m3-batch50-m3b-implementer-report.md` 第五节第 2 条改用「第一片吸收第二片」验 R3。

### K4-2　R3 照抄

没打中。把照抄的片的「上一版叶序」改成「这一版叶序」（变异 `r3-carried-from-new-position`，`mutations.py` 里那一条）：core 的 inode_tree 单测、`inode_leaf_absorb_and_borrow_on_the_unlink_path` 与我的随机历史都红，后者在池级 checker 上红 I-9.2、I-9.12、I-3.9（`outputs/k7-mutations.txt` 第 4 行；红行原样在复跑工作目录的 `k7-r3-carried-from-new-position-probe.log`，我这一次的拷在草稿目录 `kept-logs/` 下）。所以照抄的叶序错了这几样都拦得住；「内容其实变了却照抄」在 138 856 次删除类发布上一次没出（会先红 I-9.2 / I-9.12 或我的判据）。

### K4-3　N1 删 I-9.4 第三句之后

不中，记一条射程。I-9.12 判「第 i 条的分隔 key ≤ 第 i 个孩子的 key 区间下界，且 > 第 i−1 个孩子的 key 区间上界」：分隔 key 写大了（超过右片最小号）、写小了（不大于左片最大号，包括「借了没改、还是容器号」那一形，已有变异行）都红。拦不住的是分隔 key 落在左片最大号与右片最小号之间、却不等于 N1 该写的那个值：单张镜像上它与「借之后右片又删了最小那几条」的合法镜像逐字段相同，判别子看不到（四句里的「判别子观测不到」），而且它不破路由。要拦得跨版本比（分隔 key 只在左邻借走记录的那一版变），今天没有这一判。

### K4-4　理想模型与胶水

打中「不是两份独立算法」那一问的另一半：**模型根本不描述多片叶的 M3**。模型罩的历史只删第一个文件、退役树只罩一片容器：「/// 退役树的开放容器（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件」（`crates/singlefs-harness/src/model.rs:104`），胶水对第二片容器直接 panic：「"模型今天只罩退役树一片容器"」（`crates/singlefs-harness/src/model_comparison.rs:125`）。模型里没有吸收、借、117、先右后左（`grep -n '吸收\|borrow\|absorb' crates/singlefs-harness/src/model.rs` 0 行）。所以 M3 的阈值与方向没有对拍那一道，只有 core 单测与 harness 那四条用例；对拍失去的不是「独立性」，是整段覆盖。这是写法上的空格，不是行为错——K7 那条「先左后右」照绿就是它的后果。改法（被攻过零轮、推的）：模型加一个按叶序记条数的 M3，对拍只比身份集与条数分布；或在 core 单测里钉一格「L = R、删后 L + H = 233」（K7 那一格）。

## K7　判别力（顺带）

在仓副本上逐条改坏、跑 harness 档与 core 单测里相关的那几份（core `--lib inode_tree`、`inode_leaf_absorb_and_borrow_on_the_unlink_path`、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`、`pool_checker_judges_the_deletion_state_of_every_walked_version`、`checker_known_bad_images`）加我的随机历史（种子 100..105、40 步）。变异清单 `mutations.py`、改法 `apply.py` 在模型目录，复跑是 `rerun.sh` 的 `k7-<变异名>` 那几个阶段（每份变异副本各用自己的编译目录）；原样结果 `outputs/k7-mutations.txt`。checker 档（GPU 对拍、层 0）没跑。

| 变异 | 改了什么 | harness 档与 core 单测 | 我的随机历史 | 判定 |
|---|---|---|---|---|
| `m3-left-first` | `absorb_until_no_pair_fits` 先查左邻再查右邻（`crates/singlefs-core/src/inode_tree.rs:466` 与 `:468` 两支对调） | 五份全绿 | 0 红 | **照绿**（量过） |
| `m3-floor-116` | 借的下限 ⌈233 ÷ 2⌉ 改成 ⌊233 ÷ 2⌋ | core 单测、absorb 用例红 | 0 红 | 拦得住 |
| `m3-floor-118` | 下限 + 1 | core 单测、absorb 用例红 | 借的断言 panic | 拦得住 |
| `r3-carried-from-new-position` | 照抄的片带这一版叶序 | core 单测、absorb 用例红 | 9 行红（I-9.2、I-9.12、I-3.9） | 拦得住 |
| `checker-i934-no-lineage-filter` | I-9.34 不按谱系滤 | `checker_known_bad_images` 红（删 → 回退 → 再删那条合法镜像） | 5 行红 | 拦得住 |
| `checker-i932-no-birth-before-death` | I-9.32 去掉这一句：「&& retired.container_birth < record.death_txg」（`crates/singlefs-checker/src/walk.rs:10589`） | 五份全绿 | 0 红 | **照绿**（harness 档量过；checker 档没跑，GPU 对拍里没有一条造这一形，推的照绿） |
| `checker-i932-no-death-at-most-version` | I-9.32 去掉这一句：「&& record.death_txg <= version.checkpoint_txg;」（`crates/singlefs-checker/src/walk.rs:10590`） | 五份全绿 | 0 红 | **harness 档照绿**；checker 档的 GPU 对拍用例 `the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version`（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:8213`）要 CPU 红 I-9.32，推的能拦，没跑 |

四句（三条照绿）：
- `m3-left-first`：分辨臂——方向是实现员按 E176 M3 主取样点自定的（正文第二节），两个方向都满足 D8 已定项 6 的每一句；被判对象看得到——两个方向都只在「L = R 且 L + H = 233」那一格上分得出（K4-1），镜像与 checker 看不出方向，只有单测钉得住；分句——K7「M3 的阈值与方向」；改法（被攻过零轮、推的）：core 单测加一格三片 [L, H, R] = [117, 117, 117]（产品写法走得到：三片都 ≥ 117、相邻和 234）、删 H 的一条，钉「H 吸收 R、R 的身份进退役」，变异表加一行。
- `checker-i932-no-birth-before-death`：这一句只在一种镜像上独自起作用——退役记录的死亡 txg 早于被退役那片的出生、而那个死亡 txg 已不在回退候选集里（I-9.35 不判），被退役的身份今天又不在树里（I-9.34 不判）、谱系上也没有第二条（I-9.33 不判）。坏镜像改法：在一份删过叶的镜像上把那条退役记录的死亡 txg 改成 1、重封那片退役容器。没造（推的）。
- `checker-i932-no-death-at-most-version`：同上，harness 档没有坏镜像钉它，只靠 checker 档 GPU 对拍那一条兜（有卡才跑）。

变异表里已有的几行（两片之和 233 不并、身份留给右片、借不均分、N1 不改、借完不接着看、低于下限也不借、R3 按位置重写、死亡 txg 取前一个、Opt2 全取、ckpt_cost 用之前的行数）我没重复证红。

`m3-left-first` 不是等价变异，在产品写法走得到的一段历史上分得出（`opus_s2_probe_direction.rs`，经会话发：建 698 个空 inode、三片各删到 117 条、再删中间片的 350）。实现上（`outputs/k7-direction-main.txt` 第 3 行）：

```
name=direction after-unlink-350 leaves=[(1, 117), (234, 233)] new_retirement_records=[(467, 702)] checker_red=0
```

先左后右的副本上（`outputs/k7-direction-left-first.txt` 第 3 行）：

```
name=direction after-unlink-350 leaves=[(1, 233), (467, 117)] new_retirement_records=[(234, 702)] checker_red=0
```

两边身份消失的是不同的叶（467 对 234），池级 checker 都 0 红，五份用例都绿。

## K1　W3 回退水位（Opt2）

### K1-1　滤法与条款是不是同一件事

读码：回退取 inode 号水位时环里只交「按 cur 的实例表没被抛弃」的根：「.filter(|root| !abandoned_by_table(root, current_table))」（`crates/singlefs-core/src/mount.rs:7835`），调用处「&ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),」（`crates/singlefs-core/src/mount.rs:8120`）；判抛弃是「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1565`）。树 ID 水位照旧取环里全部根：「.fold(current.root.tree_identifier_watermark, u64::max);」（`crates/singlefs-core/src/mount.rs:8114`）。与条款`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`：「inode 号水位 = max(cur 那一版内存里的水位, 环里读得出、按实例表没被崩溃恢复抛弃的根的水位)」是同一件事。

两个方向都没造出来：
- 被抛弃的根没被滤掉（回退跳到被抛弃那段的水位）：定向扫 1 008 段（K3-3）每段都先抛弃一次建 inode 的发布、再在新实例里重发它的号，之后两步放开扫（含回退到环里第 0..5 新的根），I-9.30 / I-9.29 一次没红；随机历史 70 次抛弃同样没红。
- 没被抛弃的根被滤掉（水位低于某个还会被选的根）：推的造不出——同一条时间线上水位单调（建 inode 加、回退取 max、崩溃恢复回到所选根那一版的值），没被抛弃的根都在 cur 的来路上或被 cur 那一版的回退带过，cur 内存里那一项已不小于它们。反过来说，Opt2 里「环」那一项在今天的写法下取不到比 cur 大的值（推的，被攻过零轮）；它在实例表行写错、或读到别的池的根时才起作用，那两形不在本轮射程。

### K1-2　五种形状

| 形状 | 怎么造 | 结果 |
|---|---|---|
| 环里有读不出的根 | 抛弃那一步让最新根的根槽在挂载时读回全 0（之后写回） | 抛弃那一步之后 I-3.1 红（与删除线无关，见「顺带看到的」），I-9.29 / I-9.30 不红 |
| 实例表行不全 | 没造（D23 已定项 14「水位」一格写着 cur 的实例表读不出时今天的代码先拒整次回退，我没核那一支） | — |
| 回退目标就是 cur | 定向扫的 `rollback0`（环里最新那一版就是 cur）：`rb->8:ok` 那一类（`outputs/abandoned-suffix-scan.txt`） | 做成，checker 0 红 |
| 连续两次回退 | 定向扫 `rollbackX,rollbackY` 36 × 7 段 | 0 红 |
| 回退之后崩溃 | 定向扫 `rollbackX,crash-mount` 与 `rollbackX,abandon`（抛弃的正是回退那次发布，谱系记录随之没了） | 0 红 |

## K2　W3 台阶行与谱系

没打中。K2-1、K2-2：进程崩溃再挂（恢复读回所选那一版、按「代段 = 根 txg 的那一行」要求那一行）在产品容量那一批里有 129 步（按步数出，`outputs/random-leaves-op-counts.txt`；别的几批与定向扫里也有，没逐次数），一次没被拒；空发布（抬 F、卸载、写行、暖机）、回退那次、第一个文件那一版都在其中。K2-3：I-9.30 ①②④ 在全部合法镜像上 0 红（同上各批）。K2-4 没造（台阶行跨叶要 463 级台阶，我的历史一段至多几十次建 inode）。

**C769 的代价（推的，没造镜像）**：③ 不判之后，一张「台阶行代段写晚了」的坏镜像判不出来。造法：合法历史里 txg 6 那次建 inode 把水位从 4 抬到 10（台阶 (6, 10)），txg 7 是一次空发布；把台阶 (6, 10) 删掉、同时在 txg 7 让 9 号重发（对象出生代 7）。按 `inode_number_watermark_steps_of_a_version` 的认法，代段 7 那一行与上一级台阶 (5, 4) 不同、自己被认成台阶 (7, 10)；② 找「H > 9 且出生代 > g」的台阶：(7, 10) 要出生代 > 7，9 号的 7 不满足——不红；正确的台阶 (6, 10) 在时它会红。① ④ 不看代段。这正是 walk.rs 自己写下的那一句：「/// 正推草案的 ③（水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根的 txg）不判：台阶行与每次发布写的那一行同 key 维度、只差代段，」（`crates/singlefs-checker/src/walk.rs:10815`）。代价：台阶的代段错一格、与同一格里的重发合起来时漏报；单独错代段不影响别的判。

## K6　checker 与 GPU 判器

- K6-1（CPU 判法在合法镜像上）：没打中。回退越过叶的出生（建 233 / 234 / 300 之后回退到更早一版）、删 → 回退 → 再删、借后吸收（133 次借里有借完接着吸收的）、候选集里有被抛弃的根（70 次抛弃）、I-9.35 父版取法在空发布 / 回退那次 / 抛弃过的段上，全部 0 红（除抛弃那一步的 I-3.1）。克隆头：`crates/` 没有克隆路径，造不出。
- K6-2（GPU 与 CPU 同判）：没跑（GPU 对拍是 checker 档）。读码对了 I-9.35 两边的父版门：GPU `d_parent_version_to_compare`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 第 3628 行起）与 `d_single_version_of_the_scan_at_txg`（同一文件第 3582 行起）与 walk.rs 的 `parent_txg` 取法、去重键、全零指针不去重、同 txg 多份判不了，在我追过的几形（全零指针的版、同一条根在候选与版本两张清单里各出现一次、谱系读得出而零根）上结论相同；没找到分歧形。
- K6-3（坏镜像各自只红它自己）：没逐条核。只知道 `checker_known_bad_images` 在我的仓副本（没改坏）上「test result: ok. 224 passed; 0 failed」（草稿目录 `main-badimages.log`），退役那五条的坏镜像在那份文件里各自断言只红自己（例：「"退役树根条目的身份与容器头不符：只红 I-9.31：{verdicts:?}"」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:10759`））；I-9.16 ④、I-9.27、I-1.7 那几条我没逐条看。

## 崩溃状态（K1-2「回退之后崩溃」、K2-2、K3-3 的崩溃那一半）

原型 `crash-proto/`（独立小 crate，路径依赖仓副本；二进制 `s2_crash_states`）：自己造五段小流，被枚举的那一段只录那一次用户动作，先用 `closed_form_state_count` 与层 0 口径（原地覆写三态，`layer0_state_count_with_torn_in_place_overwrites`）各算状态数，不超过 10⁶ 的才调 `enumerate_layer0_versions`（每一段都展开），枚举器在每个状态上恢复、跑 oracle 与池级 checker。

| 场景 | 被枚举的那一段 | 状态数（闭式 / 层 0 口径） | 跑没跑 | 结果 |
|---|---|---|---|---|
| `rollback-after-absorb` | 建 233 个、unlink 2（吸收、一条退役记录）之后，回退到建完那一版 | 65 543 / 65 548 | 跑了，8 线程 167 秒 | oracle 0、checker 0；I-3.13、I-9.30–I-9.35 在 65 548 个状态上都评估过 |
| `absorb` | unlink 2（意图 + 最后一批，开出退役树） | 5 242 893 / 5 242 903 | 没跑：一段自己就超过 10⁶ | — |
| `borrow` | unlink 350（借 58 条，N1） | 8 388 621 / 8 388 631 | 没跑 | — |
| `lender-absorb` | unlink 350（借完借出那片吸收右邻） | 71 303 181 / 71 303 191 | 没跑 | — |
| `reunlink-after-rollback` | 回退之后再 unlink 2（同一身份第二次退役） | 5 242 893 / 5 242 903 | 没跑 | — |

原样：`outputs/crash-rollback-after-absorb.txt`、`outputs/crash-plans-over-budget.txt`。跑的那一段的结果行：

```
name=crash scenario=rollback-after-absorb states=65548 violations=0 first_violation=None checker_red=0 checker_first={} failed_states=0 no_file_states=0 evaluated=["I-3.13:65548", "I-9.30:65548", "I-9.31:65548", "I-9.32:65548", "I-9.33:65548", "I-9.34:65548", "I-9.35:65548"]
```

超预算那四段的大头是最后一批那一段：两块盘各写十来个单元（inode 叶、根、退役容器与根、记账、分配记录树、映射树、树表……），一段 22～26 个写，2²² 起。这一条腿只跑了这 65 548 个状态，连复跑那一遍共 131 096 个（预算 10⁷）。四段要不要跑、怎么跑交主 agent（命令在「要主 agent 跑的命令」）。随机历史那几批的崩溃状态没枚举：每段几千次发布，单次发布就过 10⁶，按定义 3b 不跑；它们只覆盖了「发布之间的进程崩溃再挂」与「崩溃恢复抛弃最新根」两种。

## 顺带看到的（不归本轮的格，交主 agent 定要不要追）

**崩溃恢复抛弃最新根的那一刻，池级 checker 红 I-3.1**（「记账的已分配」比「遍历全部有效根」多几十到几百个槽）。只在抛弃那一步之后的那张镜像上红，下一次发布之后不再红；**不删任何 inode 的对照也红**（`S2_NO_DELETE=1`，种子 200..211 共 12 段，6 段里红了 9 行、全是 I-3.1，原样 `outputs/abandoned-no-delete-control.txt`），所以与删除线、退役树无关。我的造法与 `tests/common/mod.rs` 的 `abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and` 有两处不同：用的是会话（影子账开着）、藏的是最新那条根的记录点名的数据与打包单元；那份 helper 的用例在它自己那段历史上断言「抛弃 C 之后那一刻全绿」。我没分清这是我的造法造出了故障模型之外的状态，还是挂载那一刻读坏根槽时的保守隔离没落到盘上、checker 不知道。复现：`S2_TRACE=1 S2_SEED_FROM=215 S2_SEEDS=1 S2_STEPS=7` 跑 `opus_s2_probe_random_leaves_abandoned` 的 `random_histories_over_several_inode_leaves_with_abandoned_roots`，第 6 步红。

## 没打中的形状（试过什么、取样多大）

| 格 | 试过的形状 | 取样 |
|---|---|---|
| K1-1 / K1-2 | 抛弃建 inode 的发布 → 新实例重发号 → 两步用户动作放开扫（回退到第 0..5 新、删重发的号、删前缀的号、建 1 / 233、崩溃再挂、再抛弃）；随机历史里夹抛弃 | 7 组 (k1, k2, k3) × 144 段 = 1 008 段；40 段 × 40 步、70 次抛弃 |
| K2-1 / K2-2 / K2-3 | 每步之后恢复与 checker（I-3.13、I-9.30）；回退那次发布的崩溃状态全枚举 | 四批随机历史；65 548 个崩溃状态 |
| K3-2 / K3-3 / K3-4 | 自写判据比退役完备与只追加；容量开关下退役树长到 3 层内部节点；回退、抛弃之后再删同一片 | 138 856 + 28 259 次删除类发布，叶死亡 259 + 53 次 |
| K4-1 / K4-2 | 自写叶结构判据；R3 照抄叶序改坏证红 | 同上 |
| K5-1 / K5-2 / K5-3 | 贴边建满（du 步长 1 扫到第一个被拒的格）后 unlink，有 / 无退役记录 × 有 / 无退役项两份 | 384 槽小盘、两块盘、K = 144，四条臂各扫到边 |
| K6-1 | 上面全部合法镜像上的 checker | 同上 |

## 这条腿自己的限度

- 只有一种几何（384 槽小盘加环窗口补足、两块盘、批大小 144）。乙二 在这块盘面上离红差六百多槽，结论「造不出」只对这块盘面量过；环窗口窄的几何没扫。
- 崩溃状态只枚举了一段（回退那次，65 548 个，本腿共跑两遍 131 096 个）；最后一批、借、吸收那几段一段就过 10⁶，没跑（交主 agent）。随机历史只有发布之间的进程崩溃与抛弃最新根两种崩法。
- 抛弃最新根是我照 helper 改到内存盘上的造法，那一步之后的 I-3.1 红我没分清是造法的问题还是实现的问题（「顺带看到的」）。
- K6-2 没跑 GPU（checker 档），只读了 WGSL 的父版那一段。
- 退役记录一次两条以上那一支（容器树一次追加多条）在产品写法上造不出，我也没装开关去硬造。
- 我自己提的改法（K4-4 模型加 M3、K7 加 [117, 117, 117] 那一格单测、两条 I-9.32 坏镜像、checker 档那份用例改成 65549）都只在我的推理上，被攻过零轮；表里标了「量过」的只有 K7 方向那一格（`outputs/k7-direction-*.txt`）。

## 要主 agent 跑的命令

1. 超预算的四段崩溃枚举（每段 5.2×10⁶～7.1×10⁷ 个状态，本机 8 线程约 400 状态 / 秒，最小那段约 3.6 小时，推的）：在模型目录副本里先 `SINGLEFS_REPO=/home/fy5090/code/singlefs bash rerun.sh <工作目录> prepare`，再以 `<工作目录>/crash-proto` 为工作目录跑 `CARGO_TARGET_DIR=<工作目录>/target-proto bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 8G bash /home/fy5090/code/singlefs/scripts/hosts/capped.sh <线程> cargo run --release --offline -q -- absorb`（另三个场景名 `borrow`、`lender-absorb`、`reunlink-after-rollback`）。原型里 10⁶ 的闸要先改掉（`src/main.rs` 的 `STATES_AT_MOST`），它超过本腿的预算，要不要跑由主 agent 定。看 `name=crash` 行的 `violations=0`、`checker_red=0`。
2. checker 档那份旧断言（K3-1）：`SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 8 cargo test --release -p singlefs-checker-tier --test crash_points_tree_identifier_watermark_orphans`，推的会在搭流那一步 panic、失败说明是「流 ①：第一个文件版本 (1, 3) 从 mkfs 的水位 65536 发 65536..65547、新水位 65548」。

## 没做什么

- 没跑任何门禁阶段与 checker 档测试（含 GPU 对拍、层 0、变异整表）；K3-1 那份用例只在 harness 档原型里复现了它断言前的那一步。
- 没改冻结副本、没改主工作区；K5-2 的「去掉退役项」与 K7 的变异都在仓副本上。
- 没读禁读清单里的文件。
- 模型目录里我自己这一轮新建的几份文件（原型源码、`mutations.py`、`apply.py`、`rerun.sh`、`outputs/` 下一份）改过几次，是写临时文件再改名换上，不是定点替换。
