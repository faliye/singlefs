# m3-sweep-and-tree-id-r1 攻方腿（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 T1 T2 T3 S1 S2 F1 F2 F3 F4 F5 -->

批次：`records/2026-09-30-里程碑三开工第一轮.md` 第 8 节（8-5 三方）。攻击面 S1（清扫五句 Q1–Q5 的候选）、S2（同一树 ID 两次发号的候选判法）。前几轮判决：无。全部数在冻结副本 `/tmp/claude-1000/m3-sweep-and-tree-id-r1/tree/` 的拷贝上量，是副本上的数，不是入库装置上的数；清扫器今天的 `crates/` 里没有，下面的清扫器是我在副本里按条款拼的原型（零轮）。

## 复跑

```
cd /home/fy5090/code/singlefs
bash research/prompts/m3-sweep-and-tree-id-r1-opus-model/scripts/rebuild-and-run.sh <新草稿目录>
```

脚本把冻结副本 rsync 进 `<新草稿目录>/repo`、装上 `sweep-proto`、经内存包装（编译 3G、每次跑 2G）与 `capped.sh 6` 编译，再跑全部 43 份产物到 `<新草稿目录>/out/`。我这一次的复跑（`/tmp/claude-1000/m3-sweep-and-tree-id-r1-opus/rerun`）挂钟 1 分 44 秒，43 份里 41 份与开发时的输出逐字节相同，另 2 份（`th-c342.out`、`th-c342-sysconf.out`）只差后来加的两行打印。模型目录全部文件的 sha256 在 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/SHA256SUMS`（95 行，`sha256sum -c` 在模型目录里跑）。源码：

| 文件 | sha256 |
|---|---|
| `sweep-proto/Cargo.toml` | 41d6e5acfc0264e567300349c7ab4d10429b201e21eafe9163b647c3c6565e76 |
| `sweep-proto/src/lib.rs` | 9328a8c1b39719b51530573a315b9681ed68f809d59498172a0d2935e577ad29 |
| `sweep-proto/src/bin/fake_header_tree_id.rs` | 290029e3360ea385c33195bc80667051c76fee769b25e4c5d43c0d714a9c4041 |
| `sweep-proto/src/bin/sweep_crash.rs` | 151ebad7f10cb6858fd0f657406bce3c97fd507a6a76b17d2506fedb8017c065 |
| `sweep-proto/src/bin/sweep_fake.rs` | 6b0c21dc15a02e76a46b8cd0e6cf2ed3419ede476aca3e36d0ff679717324819 |
| `sweep-proto/src/bin/sweep_stale.rs` | 0ca382d99179e248fcd94510331a5cc881d366aaaedd30d488a1b6a46fee481f |
| `sweep-proto/src/bin/tree_id_histories.rs` | 6fbdcf130d7552ffc055aa75c2929b10104eb73bbfd5896e3c9472482fbe3234 |
| `scripts/rebuild-and-run.sh` | a3ad66d15126bcfacc80b38a1a465d411bc9a31a2c05849313ce3aadd2fe0135 |

下文「产物 X」都指 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/X`。

## 各格判定一览

候选名是我起的（背景材料第三节只给了题，没给候选）；每个候选的定义写在各节开头。「量过」= 副本原型上跑出来的；「推的」= 按代码推、没造盘面。

| 格 | 候选 | 判定 | 依据 |
|---|---|---|---|
| Q1 | 甲：挂载内同步，写行、暖机、抬 F、删除续做之后，交回之前判、抹、发清扫发布 | 没打中（一次抽样） | 量过：sweep_crash 一段前缀；判与抹之间没有分配 |
| Q1 | 乙：挂着时后台续做，与用户改动交错；判完列单子，之后按单子抹 | **打中** | 量过：单子判完之后再覆盖写 6 次，现行那一版的树表单元被抹（I-2.1 等红）；7 次，可写挂载与只读挂载都挂不上 |
| Q1 | 乙-重读：同乙，抹之前在同一串行段里重读那一槽的头，仍是候选行的未发布单元才抹 | 没打中（一次抽样） | 量过：K = 1、2、4、8 |
| Q1 | 丙：只在正常卸载时跑 | 没打中判据（推的） | 没正常卸载的池永远不清扫；只是「该删的删不掉」 |
| Q1 | 丁：并进写行那次发布 | 与落点不合 | 已定落点是「专用的清扫发布」；另外抹头是单元区的写，取号之前不能写 |
| Q1 | 戊：读阶段（取号之前）判、写行与暖机之后抹 | 机理同乙（推的） | 本前缀上写行与暖机没复用孤儿槽（34 个孤儿挂载之后一个不少），没造出 |
| Q2 | 甲：抹单元头所在的第一个 512 字节扇区（写零、FUA），之后每盘一道屏障 | 没打中（一次抽样） | 量过：6 个手摆崩溃状态，下一次挂载都可写、checker 都绿 |
| Q2 | 乙：整单元写零；丙：写一个可认出的「已抹」标记 | 没造（推的） | 判定与甲相同，多写 16–32 KiB 每落点；丙多一种头类型，没有收益 |
| Q3 | 甲：池中每一个写过的槽都按头判（含已分配、不认领） | **打中** | 量过：用户文件内容里的伪头被判成死实例的未发布单元，抹掉之后现行那一版读不出 |
| Q3 | 乙：跳过当前账里已分配的槽 | **打中** | 量过：伪头所在单元已释放、仍被环里上一版的根引用；抹掉之后回退被拒、I-2.1/I-4.8/I-7.4 红 |
| Q3 | 丙：认领规则扫描（D18 已定项 6 射程那一条） | 没打中（一次抽样） | 量过：live、freed 两形 |
| Q3 | 丁：跳过根环里任一条可读根的账里仍分配的槽 | 没打中（一次抽样） | 量过：live、freed 两形 |
| Q4 | 同实例、事务号 0、整链重写、走发布路径的空发布，一次删满足条件的全部行 | 没打中（手摆状态，没跑层 0） | 量过；清扫臂一段 22 个单元写，层 0 全量 4 194 340 态，超过 10⁶，没跑 |
| Q5 | 抹头之后、清扫发布之前崩；清扫发布崩在记录已持久、根槽没持久 | 没打中（手摆状态，没跑层 0） | 量过：前一形行留着、下次重扫；后一形恢复施加清扫记录，实例表换成删行那一版，下一次可写挂载照常 |
| Q5 | C333 在新根段三字段之后还在不在 | 那一形没复现（手摆，一次抽样） | 量过：「记录已持久、根槽没持久」那一态下一次可写挂载照常、checker 前后都绿 |
| S1 旁及 | 判别力：不抹就删行 | 今天没有观察者 | 量过：24 个按删前那一行判未发布的单元留在盘上，oracle、池级 checker、下一次挂载全都不红 |
| S2 | T2：码 2 单元头里同树 ID、诞生代号跨了某条根的水位（逐根判：r 的水位 ≤ 某节点树 ID 而节点诞生代号 ≤ r 的 txg） | **打中** | 量过：用户文件内容在槽边界上带一个码 2 伪头，T2 红，今天的 I-7.8 扫描方向也红 |
| S2 | T2-认领：同 T2，加认领规则 | 没打中（一次抽样） | 量过：live 形；freed 形 60 次覆盖写里伪头没落到无人认领的槽 |
| S2 | T1-全环：根环任两条可读根的树表里同一树 ID 的诞生 txg 不同 | 合法历史上 0；与 I-9.14 第 ③ 条对撞 | 量过三段合法历史；第 ③ 条与它的正向用例要求跨线不同「不许红」 |
| S2 | T1-同线（即今天的 I-9.14） | C342 那一形里已经红 | 量过：多重故障造的镜像上今天的 checker 已红六条（含 I-9.14），I-7.8 不红 |
| S2 | T3：根环可读根按 txg 排，水位不许下降 | 抓不到 C342 那一形 | 量过：全部历史 0 |
| S2 | C342 那一形能不能在今天的代码上造 | 要连系统配置四槽一起退回 | 量过：只藏根与记录时乙-配置续在取号之前拒可写 |

## S1　清扫五句

### 共用的前缀与原型清扫器

「死实例」前缀（`sweep_proto::dead_instance_prefix`，S = 4、两块 4 GiB 内存稀疏盘、根环 12 槽）：mkfs → 实例 1 第一个文件、覆盖写 2 次 → 实例 1 一次 5 个数据单元的顺序写，切在第一条记录写之前（单元落了、记录与根没落，留下孤儿）→ 可写挂载实例 2（写行 (1, 5, 0)）→ 实例 2 覆盖写到根环里没有实例 1 的根。产物 `sc-probe-sweep.out` 第 1 行：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-probe-sweep.out:1`「PREFIX s=4 n=5 i1=1 i2=2 orphan_unit_writes=34 rows_after_mount2=[(1, 5, 0)] unpublished_of_i1_after_mount2=34 overwrites_until_no_i1_root=10 current=(2,17) every_ring_slot_self_verified=true candidate_rows=[(1, 5)] unpublished_targets=24」

原型清扫器（`sweep_proto::judge_sweep`、`erase_headers`、`sweep_publish`）照条款拼：根环 12 槽都自证过，才把「环里没有该实例任何一条根」的行列为候选；读池里每个写过的槽的头（头校验和过、fsid 是本池），写序实例是候选、按行判未发布的记下；每个落点写 512 字节零（FUA），每盘一道屏障；之后一次同实例、事务号 0 的空发布，实例表整链重写成删掉候选行的那一版（与写行那次同一个计划形态，`crates/singlefs-core/src/mount.rs:3029`「instance_table: InstanceTablePlan::Rewrite(instance_table),」）。它对得上的落点条款是 `.claude/kb/decisions/18-块里携带什么信息.md@已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类`「抹头全部 FUA 返回、过一道屏障之后才发这次清扫发布；读失败、设备不全不删。」

### Q1　谁、什么时候跑

候选：
- 甲：挂载内同步。写行、暖机、准入推的抬 F、删除续做都做完，交回 `Mounted` 之前判、抹、发清扫发布；判与抹之间没有任何分配。
- 乙：挂着时后台续做，与用户改动交错。判完列一张单子，之后（中间夹着用户改动的发布）按单子抹、发清扫发布。
- 乙-重读：同乙，但抹之前在同一个串行段里逐个重读那一槽的头，仍是候选行的未发布单元才抹。
- 丙：只在正常卸载那一串里跑。
- 丁：并进写行那次发布（上一轮判决第四节第 1 条的丁）。
- 戊：读阶段（取号之前，只读）判，写行与暖机之后按判的单子抹、发清扫发布。

**乙打中（量过，`sweep_stale`）。** 前缀之后先判（单子 24 个落点），实例 2 再覆盖写 K 次，再按单子抹、删行。孤儿槽在当前账里是空闲的，分配器会把它们发出去；K = 6 时单子里 12 个落点已在现行那一版的账里：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-6-stale.out:1`「SETUP k=6 arm=stale list_a=24 list_a_now_in_current_ledger=12 list_a_now_referenced_by_a_ring_root=12 targets=24」

抹完之后现行那一版的树表单元对不上（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-6-stale.out:2`「I-2.1:树表单元 在盘 0 槽 50284 的那一份与位置条目里的校验和对不上」）。K = 7 抹掉的是现行那一版 extent 树的根，可写挂载与只读挂载都起不来：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-7-stale.out:3`「REMOUNT err=Recovery(MappingStillUnreadable { slot: SlotNumber(50285) })」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-7-stale.out:4`「READ_ONLY_MOUNT ok=false」

挂载在任何写之前失败、盘上不变，所以每一次挂载都一样失败。K 扫过 1、2、4、5、6、7、8、9、10、12、16：1–5 不中（单子里的槽还没被发出去），6、7 中现行那一版，8–16 中的是环里候选根的单元（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-8-stale.out:1`「list_a_now_in_current_ledger=0 list_a_now_referenced_by_a_ring_root=24」，I-2.1、I-4.8、I-7.4 红）。乙-重读在 K = 1、2、4、8 上都绿，K = 8 时单子 24 个、重读之后一个都不抹：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-8-reverify.out:1`「SETUP k=8 arm=reverify list_a=24 list_a_now_in_current_ledger=0 list_a_now_referenced_by_a_ring_root=24 targets=0」

四句：① 分辨臂：乙中、乙-重读与甲不中。② 系统抹之前看得到判别它的东西：那一槽此刻的头（重读一次就有），也看得到自己的分配器发过它。③ K = 7 满足「让池永远挂不上可写」（连只读也挂不上）；K = 6 满足「误判一条合法镜像」的字面（把现行那一版的单元判成死实例的孤儿抹掉）。④ 跑前条款没给逐格改法，只有「出局」。

**戊（推的）。** 判与抹之间隔着写行、暖机、抬 F 的分配，机理与乙相同；本前缀上挂载 2 之后 34 个孤儿一个不少（第 1 行 `unpublished_of_i1_after_mount2=34`），写行与暖机没复用它们，所以这一段历史上造不出。分配器从同一版账起步时发的是哪些槽，取决于账里的空闲次序，换一段前缀（孤儿在最低的空闲槽上）就可能是写行那次的实例表片被抹——那就是「表不可读时：只读挂载」，池从此只读。没造，推的。

**甲、丙、丁。** 甲：sweep_crash 走的就是甲（判、抹、发之间没有分配），见 Q5 各状态，没打中；代价是每次可写挂载读全池每个槽的头（4 TB 一块盘按 16 KiB 一槽是 2.7×10⁸ 次读，推的）。丙：没正常卸载的池（每次都崩）永远不清扫，行只增，落在「该删的删不掉」，不中判据。丁：与已定落点「专用的清扫发布」不合；抹头是单元区的写，还得排在取号之后，丁要把抹头放在取号之后、写行之前，可行性没量。

「下一次挂载取号之前删不掉」与「写行发生在挂载开头」：本前缀上实例 1 那一行要等实例 2 发满 10 次覆盖写、环里 12 槽都换成实例 2 的根才成为候选（第 1 行 `overwrites_until_no_i1_root=10`）；S = 16 时是 48 槽，推的，按同一比例要四十几次发布。也就是说任何候选在取号之后、同一次挂载里都只删得掉「根已全部轮出环」的那些旧行，上一个实例那一行通常删不掉。

### Q2　抹哪些落点、写什么

候选：甲：每个被判成候选行未发布单元的落点（两份各自判、各自抹）写 512 字节零（单元头全在第一扇区：码 1 头末 105、码 2 头末 86 + 2k、码 3 头末 107；k 大于 213 时码 2 头跨出第一扇区，但魔数在 0..4，零掉第一扇区就认不出是单元），FUA，抹完每盘一道屏障。乙：整单元写零。丙：写一个可认出的「已抹」头。

读法：读出来魔数不是 `SFSU` ⇒「已抹头」（与从没写过同待遇）；头校验和过、写序或诞生代号与原来不同 ⇒「已被覆盖」；仍是候选行的未发布单元 ⇒ 没抹成，这次不删。

甲在 Q5 的 6 个手摆状态上没打中（下一次挂载都可写，checker 前后都绿）。乙、丙没造：判定结果与甲相同，乙每落点多写 16–32 KiB，丙多一种头格式、换来的只是「已抹」与「没写过」分得开，而两者在条款里同待遇，推的。

推的、没造的一格：真盘上一次单元写可以只落头所在的扇区、载荷没落（D13（验证路线） 已定项 4 的枚举域只取整写子集，层 0 造不出这一态）。这样的孤儿头校验和过、载荷校验和不过。清扫若要求载荷校验和也过才认「是单元」，它不被抹；删行之后今天的 I-7.8 扫描方向只看头校验和（`crates/singlefs-checker/src/walk.rs:2755`「if &bytes[..4] != b"SFSU" || bytes[6] != 2 {」），会把它数进「出现过的」。要求：清扫认单元的口径不能比任何一个读盘判定更严。

### Q3　怎么判「该实例的未发布单元」、当前账引用的落点算不算

按行判：写序实例 i 有行 (i, Ti, Wi)，码 1 ⇒ b > Ti 且 n > Wi 才算未发布，码 2 / 3 ⇒ b > Ti（D18 已定项 11 的已发布谓词）。候选是「池中每一个落点」怎么读：

- 甲：每个写过的槽都按头判，含当前账已分配的，不认领。
- 乙：跳过当前账里已分配的槽。
- 丙：认领规则扫描：每块盘按槽升序，解出一个头就按它的跨度（码 1 / 码 3 两槽、码 2 一槽）认领掉后面的槽，那几槽不再当探针。
- 丁：跳过根环里任一条可读根的账里仍分配的槽（候选根与被抛弃根都在内）。

攻法：实例 2 覆盖写一次，内容是用户给的任意字节，其中内容偏移 16250（= 槽宽 16384 − 数据单元载荷偏移 134，`crates/singlefs-format/src/lib.rs:38`「pub const DATA_UNIT_PAYLOAD_OFFSET: u64 = 134;」）处放一个码 1 头：照盘上真头抄、写序实例改成死实例 1、诞生代号与事务号改到行 (1, 5, 0) 之上、头校验和重算。它落在这个数据单元第二个槽的开头。live：就停在这一版；freed：再覆盖写一次，伪头所在单元已释放、仍被环里上一版的根引用（回退候选）。

结果（`sweep_fake`，产物 `sf-<live|freed>-<all|skipalloc|claim|ringref>.out`）：

| 形 | 甲（全部） | 乙（跳过已分配） | 丙（认领） | 丁（跳过环里根引用的） |
|---|---|---|---|---|
| live | 抹了伪头 2 份；现行那一版 I-2.1、I-4.8、I-7.2、I-7.4 红 | 没抹，绿 | 没抹，绿 | 没抹，绿 |
| freed | 抹了；候选根 txg 18 那一支 I-2.1、I-4.8、I-7.4 红，回退被拒 | 抹了；同左 | 没抹，绿，回退成 | 没抹，绿，回退成 |

原样：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sf-live-all.out:1`「fake_slots=[(0, 50177), (1, 50177)] fake_in_current_ledger=2 fake_referenced_by_ring_root=2 fake_claimed_away=2 targets=26 fake_targeted=2」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sf-live-all.out:2`「I-7.2:最新的根走不完：数据单元两份都读不到对得上的」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sf-freed-skipalloc.out:4`「ROLLBACK to=(2,18) err=ResurrectedUnitCopyUnreadableOrMismatched { unit: Data(DataUnitIndexInFile(0)), device: DeviceIdentity(0), slot: SlotNumber(50176) }」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sf-freed-claim.out:4`「ROLLBACK to=(2,18) ok resurrected=8 red=[]」

四句（甲、乙）：① 分辨臂：甲、乙中，丙、丁不中。② 系统看得到判别它的东西：那一槽前一个槽上有个头校验和过的码 1 头、跨度两槽（认领规则），环里那几条根的分配记录也读得到。③ 甲：满足「误判一条合法镜像」（把用户载荷判成死实例的孤儿）。乙：被抹的是回退候选根引用的数据单元，「删了被候选集里的根引用的行」那一分句字面不中（删的是行 1，它没有根），最接近的是「误判一条合法镜像」；它同时破坏了 D16（发布语义） 已定项 1 给候选根的保护（defer 窗口里分配器自己都不复用的槽被清扫写了），这一归法交主 agent 判。④ 跑前条款没给逐格改法。

伪头只要用户写得出任意字节就造得出，不需要攻击者：本池自己的原始镜像存成本池上的文件，fsid 相同，头会落在某些数据单元的槽边界上（按载荷 32634 字节一单元算，第 8192k − 1 个数据单元的第二个槽恰好对齐镜像里的一个 16 KiB 节点，推的）。这一格 D18 已定项 6 射程里写过：`.claude/kb/decisions/18-块里携带什么信息.md:186`「同一个卷的自备份镜像 fsid 挡不住（fsid 相同），那一格靠认领规则挡」。

被抛弃时间线的单元（D23 已定项 14 那句保护，`.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换`「被崩溃恢复抛弃的时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头」）：丁按可读根的账跳过，罩住可读的被抛弃根；乙罩不住（被抛弃根的槽在当前账里是空闲的，影子账只在内存里隔离）。读不出的被抛弃根：清扫条件要求环每槽自证过，那一次不删行；但若候选写成「先抹、后判能不能删」，抹头已经发生在它的槽上——次序必须是先判环、判过了才抹。推的，没造（要造一条被抛弃根要两处瞬时读错或 C554 那一形）。

### Q4　清扫发布的形状

原型的形状：同一实例、接在现行那一版之后、事务号 0（同空发布，不推进 W）、jsn 与反向链接现行那一条、实例表整链重写成删行之后那一版（固定点单元照空发布重写）、走普通发布路径；一次删满足条件的全部行。录下来的流（产物 `sc-probe-*.out` 第 2 行）：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-probe-sweep.out:2`「STREAM arm=sweep writes=51 segments=28 segment_lengths=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 22, 2, 1, 2] closed_form_states=4194335 layer0_states=4194340 swept_root=(2,18) rows_after=[]」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-probe-carry.out:2`「STREAM arm=carry writes=45 segments=28 segment_lengths=[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 16, 2, 1, 2] closed_form_states=65567 layer0_states=65572 swept_root=(2,18) rows_after=[(1, 5)]」

多写的：24 次抹头（每次一段，FUA）+ 清扫发布比照抄实例表的空发布多 6 个单元写（同一段里 22 对 16），层 0 状态数从 65 572 涨到 4 194 340。「一次删一行」与「删全部」：每次都整链重写，删一行就多发一次发布、多一次整链重写；正确性上我没找到分得开它们的历史（推的）。准入：原型过的是普通发布路径的落点分配，没判挂载准入式子；它要不要走切换预留、空间不够时这次不删还是报错，原型没分；若清扫失败让挂载失败，就会撞「永远挂不上可写」（推的，没造）。

### Q5　崩在中间；C333

手摆的 6 个崩溃状态（`sweep_crash 4 5 <臂> states`，不是层 0 枚举：每个状态都跑层 0 同一套判定——两种 journal 政策的恢复、oracle、池级 checker、记录核对器——再做一次可写挂载、再跑 checker），清扫臂原样：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-states-sweep.out:4`「STATE erase_half effective=Some((2, 17)) oracle_violations=0 first_violation=None checker_violated={} red_crash=[] leftover_unpublished_of_candidate_rows=12 NEXT_MOUNT ok chosen=(2,17) effective=(2,17) rows_written=[(2, 17, 0)] table_rows_after=[(1, 5), (2, 17)] red_after=[]」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-states-sweep.out:6`「STATE record_persisted_root_not effective=Some((2, 18)) oracle_violations=0 first_violation=None checker_violated={} red_crash=[] leftover_unpublished_of_candidate_rows=0 NEXT_MOUNT ok chosen=(2,17) effective=(2,18) rows_written=[(2, 18, 0)] table_rows_after=[(2, 18)] red_after=[]」

另 4 个（抹完、发布一个写都没落；根槽落了、系统配置没落；系统配置落了一槽；全落）同样 oracle 0、checker 0、下一次挂载可写、红 0（第 5、7、8、9 行）。抹头之后、清扫发布之前崩：行留着、没抹到的孤儿留着，下一次挂载照常、下一次清扫接着判。清扫发布崩在「记录已持久、根槽没持久」：恢复施加清扫那条记录（effective (2, 18)），新实例写行写在删行之后那一版上（`table_rows_after=[(2, 18)]`），下一次可写挂载照常。新根段带实例表指针与分配记录树根，施加时七项全取记录（`.claude/kb/decisions/23-journal的角色与格式.md@已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根`「施加一条记录 = 把所选根的这七个字段换成记录里的」）。

C333（`.claude/kb/checks-owed.md:272`「崩溃点重放覆盖删行那次发布的全部崩溃点」）：上一轮判决第二节第 1 行那一形（下一次可写挂载在取号之前拒 `ReleaseTargetAlreadyReleased`）在这 6 个状态上没复现。它是手摆状态上的一次抽样：清扫臂的层 0 全量 4 194 340 态超过单次约 10⁶ 的上限，我没跑（见「要主 agent 跑的命令」）。

### S1 旁及：「不抹就删行」今天没有观察者

noerase 臂（判完不抹、直接删行）在同样 6 个状态上：oracle 0、池级 checker 0、下一次挂载可写，24 个按删前那一行判未发布的单元留在盘上：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sc-states-noerase.out:6`「STATE record_persisted_root_not effective=Some((2, 18)) oracle_violations=0 first_violation=None checker_violated={} red_crash=[] leftover_unpublished_of_candidate_rows=24 NEXT_MOUNT ok chosen=(2,17) effective=(2,18) rows_written=[(2, 18, 0)] table_rows_after=[(2, 18)] red_after=[]」

行一删，盘上就再没有东西说那些单元没发布过（已发布谓词「无行 ⇒ 已发布」），所以池级 checker 只看一份镜像判不出（判别子观测不到）。C597 的验收要「把删行那一步拿掉必须红」（`.claude/kb/checks-owed.md:458`「判别力自证：把删行那一步拿掉必须红」），把「抹头」那一步拿掉同样要有东西红；能判它的只有看得到删行之前那一版的装置：层 0 按流判「删行那一版生效的每个崩溃态里，被删实例按删前那一行判未发布的单元一个不剩」（原型里的 `leftover_unpublished_of_candidate_rows`，清扫臂 0、noerase 臂 24，手摆状态上量过）。今天这些孤儿还没有读者会捡起来（扫描重建没实现），后果是潜伏的：扫描重建一落地，它们按「无行 ⇒ 已发布」被当成已发布单元。

（扫描重建今天没有实现：`grep -rln '扫描重建\|scan_rebuild\|rebuild_by_scanning' crates/singlefs-core/src` 在冻结副本上 0 行。）

## S2　同一树 ID 两次发号

### 候选判法的定义

- T1-全环：根环里任两条可读根各自的树表，同一树 ID 的诞生 txg 不同 ⇒ 红。
- T1-同线：同 T1，只比按最新根的实例表判有效的根——就是今天的 I-9.14（树表条目的诞生 txg 跨根不变）。
- T2（背景材料第三节「码 2 单元头里同树 ID 而诞生代号跨了水位发号点」的我的读法）：存在一条可读根 r 与一个码 2 节点 n，n 的诞生代号 ≤ r 的 txg，而 r 的水位 ≤ n 的树 ID（n 在 r 之前就有了，r 的水位却没罩住它，水位退过）⇒ 红。三个变体：noexcl 不排除孤儿；excl 照 I-7.8 扫描方向那两道排除（诞生代号 ≤ 环里最大 txg；最新根的实例表里有行 (i, Ti, ·)、Ti > 0、诞生代号 > Ti 的不数）；claim-excl 再加认领规则。
- T3：可读根按 txg 排，水位不许下降。

判法的输入都在盘上：根环、各根的树表、单元头、最新根的实例表，checker 今天都读（I-7.8、I-9.14 已经在读）；原型为了在副本上量，读盘借了 core 的读者，判法本身要照 D13（验证路线） 已定项 5 另写，原型不是可入库的写法。

### T2 打中：用户文件内容里的码 2 伪头（量过）

合法历史：mkfs → 可写挂载 → 第一个文件，内容是用户给的 20000 字节，在内容偏移 16250 处放一个码 2 头（照盘上真节点抄、树 ID 改成 2⁴⁰、诞生代号 1、写序实例 = 本实例、头校验和重算）。它落在数据单元第二个槽的开头。今天的池级 checker 就红 I-7.8，池照常挂得上：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/fh-high.out:2`「CHECKER red=["I-7.8:根环水位最大 65548，盘上出现过的最大树 ID 1099511627776"]」

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/fh-high.out:4`「REMOUNT ok instance=2 red_after=["I-7.8:根环水位最大 65548，盘上出现过的最大树 ID 1099511627776"]」

伪头 fsid 换成别的池的也红（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/fh-foreign.out:2`「CHECKER red=["I-7.8:根环水位最大 65548，盘上出现过的最大树 ID 1099511627776"]」），因为扫描方向照数外池节点（`crates/singlefs-checker/src/walk.rs:2718`「/// 节点头 fsid 不是本池的读不出写序实例（`unit_write_order_instance` 交 None），照旧算。」，I-7.8 条文 `.claude/kb/invariants.md:62`「节点头 fsid 不是本池的照旧数」）。对照：同长内容不带伪头 0 红（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/fh-control.out:2`「CHECKER red=[]」）；伪头放在内容偏移 0（不在槽边界）0 红（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/fh-inode.out:2`「CHECKER red=[]」）。扫描方向逐槽读、不认领：`crates/singlefs-checker/src/walk.rs:2769`「identifiers.insert(read_u64(&bytes, 42));」。

同一镜像上 T2 两种排除都红、T2 加认领规则不红：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-fake_live.out:2`「LIVE T1_all=0 T1_same_line=0 T2_noexcl=6[node (dev 0, slot 50181) tree 1099511627776 birth 1 inst 1 vs root (1,3) wm 65548] T2_excl=6[node (dev 0, slot 50181) tree 1099511627776 birth 1 inst 1 vs root (1,3) wm 65548] T2_claim_excl=0 T3_monotone=0」

四句：① 分辨臂：T2 / T2-excl 中，T2-认领、T1、T3 不中。② 判别它的东西在盘上：前一个槽上那个数据单元头声明两槽跨度。③ 满足「误判一条合法镜像」。④ 跑前条款没给逐格改法。这一格也是今天已实现的 I-7.8 的误红（不是候选，旁及），认领规则进 checker 是 D18 已定项 6 射程里那一句与 C89 欠着的（`.claude/kb/decisions/18-块里携带什么信息.md:186`「这条规则要做成 checker 里能判红的形式」）。

认领规则留下的缝：释放了的两槽数据单元，第一个槽被一槽的码 2 节点拿走、第二个槽的旧载荷还在，那里的伪头就没人认领。fake_freed 形连覆盖写 60 次，伪头在 txg 18 之前就被整个盖掉，一次都没落到无人认领的槽上（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-fake_freed.out:6`「STEP 15 txg=18 fakes_on_disk=0 visible_under_claim_rule=0 fakes_in_slots_referenced_by_a_ring_root=0」）。一段历史、一次抽样，算「没打中」的一次样，不算排除：分配器若先把单槽节点放进被释放的两槽单元的头一个槽，这条缝就开着（推的）。

### 合法历史上 T1、T2、T3 都不红（量过，三段）

| 历史 | 构造 | 全部判法 |
|---|---|---|
| orphan_ff | 实例 1 第一个文件切在第一条记录之前（孤儿带树 ID 65536–65547）→ 实例 2 落回 txg 2、重发同一批号（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-orphan_ff.out:5`「REISSUE first_file=(2,5) wm=65548」） | 0（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-orphan_ff.out:7`「AFTER T1_all=0 T1_same_line=0 T2_noexcl=0 T2_excl=0 T2_claim_excl=0 T3_monotone=0」） |
| applied_ff | 同上但切在根槽写之前（记录已落）→ 恢复由记录施加第一个文件那一版 | 0（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-applied_ff.out:6`「AFTER T1_all=0 T1_same_line=0 T2_noexcl=0 T2_excl=0 T2_claim_excl=0 T3_monotone=0」） |
| rollback | 第一个文件 → 覆盖写 3 次 → 挂着回退到第一个文件那一版 → 再覆盖写 | 0（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-rollback.out:3`「AFTER T1_all=0 T1_same_line=0 T2_noexcl=0 T2_excl=0 T2_claim_excl=0 T3_monotone=0」） |

orphan_ff 里 T2-noexcl 也是 0：实例 2 从同一版账起步，写行与第一个文件发的落点把实例 1 的孤儿盖掉了（D8 已定项 8 射程「丙-2」那一格判为不是错误的那种孤儿，`.claude/kb/decisions/08-核心索引结构.md@已定项 8：树表条目的字段表`「孤儿与活版共用五元组本来就是常态」）。孤儿活下来的历史（孤儿占的槽多于下一个实例开头发的）我没造，那时 T2-noexcl 会不会误红没量。

### C342 那一形在今天的代码上要多少故障（量过）

`tree_id_histories c342`：实例 1 第一个文件（txg 3 发号、水位 65548）之后覆盖写到 txg 10；把这 8 次发布的根槽写与记录写（24 次）全部抹零；实例 2 可写挂载。只藏根与记录时，乙-配置续的见证在取号之前拒可写：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-c342.err:4`「挂载 2: NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion」

再把这几次发布写的系统配置 16 次槽写也退回去（`C342_ROLL_BACK_SYSCONF=1`，等于整卷回滚那一格，D18 已定项 11「前提」那一条宣布不防：`.claude/kb/decisions/18-块里携带什么信息.md@已定项 11：单元类型标签的值域是一张登记表，打包记录单元是独立的一类`「取号那一刻两路见证不完整可见时不额外拦截，与整卷回滚同格」），实例 2 才落回 txg 2、重发 65536–65547（txg 5）。之后把没被盖掉的根槽与记录放回（故障消失）：

`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-c342-sysconf.out:10`「AFTER ring=[(0, 0, 65536), (1, 1, 65536), (1, 2, 65536), (2, 3, 65536), (2, 4, 65536), (2, 5, 65548), (1, 6, 65548), (1, 7, 65548), (1, 8, 65548), (1, 9, 65548), (1, 10, 65548)]」

最新根是实例 1 的 txg 10（实例 2 的线反成了旧的）；今天的 checker 在这份镜像上红 I-3.1、I-3.9、I-4.2、I-8.6、I-9.10、I-9.14 六条，I-7.8 不红；T1 两种、T2 三种都红（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-c342-sysconf.out:11`「AFTER T1_all=55[tree 65536: (1,6) birth 3 vs (2,5) birth 5]」），T3 不红。实例 2 在放回之前再走 6 次或 9 次（到 txg 11、14），被藏的根槽全被同 txg 的新根盖掉，一份都放不回（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-c342-sysconf-x6.out:9`「RESTORED 0/24」），全部判法 0。

结论（量过的部分与推的部分分开）：根环槽按 txg 取模、新实例的 txg 从环里读得出的最大 + 1 接，被藏的旧线根要么 txg 比新线大（那它就是最新根，镜像整个是 C331 那一类倒挂），要么在新线走到同一 txg 时被同槽盖掉（量过）；「新线是最新、被抛弃的旧线根还在环里、两线同一树 ID 诞生 txg 不同」那一窗要新线在被藏的某个 txg 上留一个洞（记录已持久、根槽没持久），推的，没造。在这一窗里 T1-同线（I-9.14）不比被抛弃的根，T2-excl 按行把旧线节点当未发布排除，只有 T1-全环看得见（推的）。

### T1-全环与 I-9.14 第 ③ 条对撞

I-9.14 的判别力第 ③ 条（`.claude/kb/invariants.md:300`「③ 只有被切掉的那条线记得不同，不许红」）与它的正向用例（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds`，改被抛弃根 C 那片树表里的一个诞生 txg、断言每一条都成立）要求跨线不同不红。T1-全环在那份镜像上必红。今天发树 ID 只在第一个文件那一版上从水位连号发十二个号（`publish_first_file`，与 mkfs 那条流上没有上一版的 `publish_version`；`grep -rn 'issued_from_watermark(' crates/singlefs-core/src` 其余两处在 `#[cfg(test)]` 之后），被抛弃线与现行线上同一树 ID 的诞生 txg 不同，只能是那个号被发了两次——第 ③ 条那份镜像在合法历史里的对应物就是 C342。并进 I-9.14 要删第 ③ 条与那条正向用例；另立一条则那条用例的「每一条都成立」要改。没在副本上跑那条用例（它在 harness 档，原型没接它），推的。

## 我提的改法（只在我的模型上量过、被攻过零轮）

- F1：抹头之前，在同一个串行段里逐个重读那一槽的头，仍是候选行的未发布单元才抹（即 Q1「乙-重读」）。
- F2：判完之后、清扫发布落盘之前，把单子上的槽从分配器拿掉（照影子账那样只在内存里隔离），清扫发布落盘之后放回。
- F3：Q3 取「认领规则 + 跳过根环里任一可读根的账里仍分配的槽」。
- F4：池级 checker 的 I-7.8 扫描方向（与任何 T2 式判法）加认领规则。
- F5：层 0 加一个按流判的观察者：删行那一版生效的每个崩溃态里，被删实例按删前那一行判未发布的单元一个不剩。
- F6：清扫的次序写成「先判环每槽自证过、环里没有该实例的根，判过了才抹」，不许先抹后判。

| 改法 | Q1-乙（过期单子） | Q1-戊 | Q3-甲 live 伪头 | Q3-乙 freed 伪头 | S2 T2 / I-7.8 伪头 | 不抹就删的判别力 | 读不出的被抛弃根 |
|---|---|---|---|---|---|---|---|
| F1 | 修，量过：`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/ss-8-reverify.out:1`「targets=0」，K = 1、2、4、8 绿 | 修，推的 | 不修（伪头仍是候选行的未发布单元），推的 | 不修，推的 | — | — | — |
| F2 | 修，推的（没实现） | 修，推的 | 不修，推的 | 不修，推的 | — | — | — |
| F3 | 不修：被重新分配的槽在丁读法下是环里根引用的，抹之前不重判就照抹，推的 | 同左，推的 | 修，量过：`sf-live-claim.out`、`sf-live-ringref.out` 各自单用都绿；合用推的 | 修，量过：`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/sf-freed-claim.out:4`「ROLLBACK to=(2,18) ok resurrected=8 red=[]」；合用推的 | — | — | 丁罩住可读的被抛弃根，推的 |
| F4 | — | — | — | — | T2 那一半修，量过：`th-fake_live.out:2` 里 `T2_claim_excl=0`；checker 里的 I-7.8 没改，推的；释放后无人认领的缝不修，推的 | — | — |
| F5 | — | — | — | — | — | 修，量过（手摆状态）：清扫臂 `leftover_unpublished_of_candidate_rows=0`、noerase 臂 24 | — |
| F6 | — | — | — | — | — | — | 修，推的 |

F1 与 F2 二选一就盖住 Q1 的乙、戊；F3 盖住 Q3；F1 + F3 合用没在原型上跑过（推的）。

## 没打中的形状与取样

| 形状 | 取样 |
|---|---|
| Q1 甲（判、抹、发之间没有分配） | 1 段前缀（S = 4、N = 5、2 盘），6 个手摆崩溃状态 × 3 臂 |
| Q1 乙-重读 | K = 1、2、4、8，各 1 次 |
| Q1 乙（过期单子）不中的 K | K = 1、2、4、5（单子里的槽还没被发出去） |
| Q2 甲、抹头崩在一半 | erase_half、erase_all_publish_none 两个状态 |
| Q5 / C333 | 清扫臂 6 个手摆状态；层 0 全量 0 态 |
| Q3 丙、丁 | live、freed 两形，伪头一个位置（内容偏移 16250） |
| S2 T1、T2、T3 在合法历史上 | orphan_ff、applied_ff、rollback 三段 |
| S2 认领规则的缝 | fake_freed 一段，覆盖写 60 次 |
| S2 C342 那一形 | 4 种：只藏根与记录（被拒）、再退回系统配置、退回之后实例 2 多走 6 次、9 次 |

没试的形状：S ≠ 4、盘数 ≠ 2；清扫时的读失败与设备不全（内存盘不报错，原型的判不含这一支）；读成功的全零根槽（上一轮判过）；中间实例行 (j, 0, 0) 的删行（我的前缀上只有 (1, 5, 0) 一行）；实例切换（今天没实现）；一次删一行的形状；清扫发布被空间准入拒；码 3 孤儿（实例表片）作被清扫的对象；T2 在孤儿活下来的历史上误不误红。

## 这条腿自己的限度

- 清扫器是我在副本里拼的原型；判、抹、发的写法是我对条款的读法，不是实现员的。
- 层 0 逐状态枚举器一次都没跑完（本腿 0 个状态；接手的腿从 0 起算）。手摆状态不是 D13（验证路线） 已定项 4 的枚举域。
- 偏离定义一处：为估挂钟，照抄臂枚举（65 572 态）那一次我在外面套了 `timeout 230`，跑到 16 416 态被截（产物 `sc-enum-carry-s100.timed-out-probe.out`），那份输出不当结果用，只用它估时：6 线程 190 秒 16 416 态，约 11.6 毫秒一态。
- 内存包装自己的结局各一次：编译上限写 8G 超过峰值两倍没给够理由，退 251；写 auto 取到 123M、编译撞上限，退 250；复跑脚本第一次没给并行理由，同会话已有一条在跑，退 252。三次的输出都没用，改了上限或理由重跑。
- S2 的判法原型借 core 的读者读盘，只为在副本上量；入库要照 D13（验证路线） 已定项 5 另写。
- 每格「没打中」都只抽了一次样。
- 正推腿的报告与模型是禁读，没读。

## 要主 agent 跑的命令

先 `bash research/prompts/m3-sweep-and-tree-id-r1-opus-model/scripts/rebuild-and-run.sh <草稿目录>`（工作目录仓根，约 2 分钟），之后在 `<草稿目录>` 下：

1. 照抄臂（对照）层 0 全量，65 572 态，每态之后再做一次可写挂载：`RUN_WITH_MEMORY_CAP_PARALLEL_REASON=<理由> nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 4G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 6 ./target/release/sweep_crash 4 5 carry enumerate 1 > out/sc-enum-carry.out 2> out/sc-enum-carry.err`。估约 13 分钟（6 线程，上面那次估时）。看 `ENUM states=` 一行（states 应为 65572）与 `NEXT_MOUNT_SAMPLED`、`NEXT_MOUNT` 各行。
2. 清扫臂层 0 全量 4 194 340 态（单段 22 个单元写，超过单次约 10⁶），按定义腿不跑：`… ./target/release/sweep_crash 4 5 sweep enumerate 1000 > out/sc-enum-sweep.out`（每 1000 态做一次下一次挂载）。按同一速率估约 13.5 小时；判 `violations`、`checker_violated_states` 与 `leftover_unpublished_with_row_deleted_states` 都应为 0。
3. noerase 臂同 2（4 194 316 态）：`leftover_unpublished_with_row_deleted_states` 应不为 0（判别力自证）。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑 checker 档、层 0 全量、门禁、harness 档的用例（原型是自己的二进制，没接 harness 测试）。
- 没实现 F2（分配器隔离）、F4 在 checker 里的那一半、F6。
- 没造「新线留洞、被抛弃旧线的根留在环里」那一窗（C342 只有 T1-全环看得见的那一格）。
- 草稿目录 `/tmp/claude-1000/m3-sweep-and-tree-id-r1-opus/` 里的仓副本与编译目录（`repo/`、`target/`、`rerun/repo/`、`rerun/target/`）交回前删掉；留下 `out/`、`rerun/out/`、`build.sh`、`run.sh` 与日志。
