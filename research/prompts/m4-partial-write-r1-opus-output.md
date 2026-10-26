# m4-partial-write-r1 攻方腿（Opus）：通用部分写的读—改—写

日期：2026-10-26。立场：照正文第四节「云端攻方」那一行，在冻结副本的拷贝上造反例；第一轮角度：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

冻结副本：`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/`（下称「冻结副本」）。代码引文写它的绝对路径；主工作区的 `crates/singlefs-checker/src/rebuild.rs` 此刻与冻结副本不同（同一句往前挪了 13 行），所以不写相对路径。
**全部数都在攻方草稿副本上量的**（冻结副本 + 下面的原型补丁），不是入库装置上的数；原型只在我的模型上量过、被攻过零轮。

## 复跑命令与文件

整条经内存包装跑，约 10 分钟（六次 cargo test，最长一次约 190 秒），主 agent 用 run_in_background 起：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-partial-write-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen \
  --work <一个新的草稿目录>
```

它拷冻结副本、打 `prototype/prototype.patch`、放进 `prototype/partial_write_attack.rs`，在 `<草稿目录>/out/` 写六份输出；比法：每份的「观测」行与模型目录 `out/` 里对应那一份逐行比（挂钟那一段除外）。线程默认 31（`RERUN_THREADS` 可改）。

```
fe561980a57f07157e688069198731162071825e91bf6554c68f404539e65c81  research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt
a3b561c94917ae33e5f3b4b61f998eb62f808ebfb1b570c93e484d0a8abba159  research/prompts/m4-partial-write-r1-opus-model/out/run-discriminator-estimate.txt
4bad279762569e093b29b0a1b3c6ee4b441f4239224628f687fe73c56317df28  research/prompts/m4-partial-write-r1-opus-model/out/run-extra.txt
69bdaa04c8acdb00324f77426f294529906b3731def2aa9f501821907871891f  research/prompts/m4-partial-write-r1-opus-model/out/run-r4-mirror.txt
e26fa08b0e3f61d6a1f2acc57ea772a6b3d24cda005568383fd0b6718d2693f8  research/prompts/m4-partial-write-r1-opus-model/out/run-r4-sweep-estimate-2g.txt
75cd656648cea9ef79f3412cff4ee169f46bba4735e312dba3fe4e756e01d712  research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-carry-first.txt
63b1ca9121836988405cd969f54bcc6ff8dfa2a349dc27bdf108439724c74e5f  research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-carry-second.txt
6ef6d58a6d4722ad1af54831c522fa710e8d06821614aab3ff451d8d4279a69c  research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-clear-first.txt
45a287df56edd66963d9b4c62cfb70fb6d8b0ea3c4e6da39296bd8b9479646ee  research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-clear-second.txt
29f5bef2d157ab63386eb51ca3f23048d7c2ee5c1b2cf7609d6bd0f2eac28637  research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-today.txt
e838c567952b9325ba2d7b870a31f712c8b82dc771c4467b7d12610f3440c205  research/prompts/m4-partial-write-r1-opus-model/prototype/partial_write_attack.rs
dce8421084cc78b4a0b22a4d86883c7f2e213876fec564db0c733e46b29b00c0  research/prompts/m4-partial-write-r1-opus-model/prototype/prototype.patch
c38ffd0978ac931b12f184b6840b67ddca4f01a053038c0f0779901758ccfc30  research/prompts/m4-partial-write-r1-opus-model/tools/rerun.sh
```

`out/` 里各份的来历：`run-all.txt` 是加三条用例之前那一版测试文件跑的六条用例（含 155 段的第一次扫）；`run-extra.txt`、`run-discriminator-estimate.txt`、`run-r4-mirror.txt` 各是点名几条用例的那一次；`run-sweep-fix-clear-second.txt` 是修掉「违例清单在开关复位之后才算」那一处打印错之后重跑的（修之前那一次同样 199 段、I-1.13 违例 1 段，只是那一行的清单打成空）。这几份的用例代码与现在这份测试文件里同名的那几条逐字相同，只差后加的用例。

### 原型做了什么（补丁三处）

1. 核心层加 `publish_partial_write_of_the_first_file_attack_prototype`：第一个文件的单元 k 换成调用方给的新净荷（声明长度不变、size 不变、截断字段照抄），走变长那一条「重写一个单元」的发布路（`SizeChangeWithoutAnIntent` 带一个重写单元）。读旧单元那一步放在测试里，两种读法各一条臂：取内存里上一版的字节（截断尾单元今天的做法），或按两条位置条目读盘、核位置条目里的整单元 CRC-32C，都对不上就失败。
2. 核心层加一个只供原型的开关：整份写那一版的截断字段写成测试给的值（模「放掉单元的整份写照截断写字段」那个改法）。
3. 池级 checker 加一个只供原型的开关：关掉读法 Z 的写序那一条。

原型写不了「加单元」的部分写（洞里、文件末尾之外）：`plan_the_extent_tree_after_a_size_change` 断言「截断留至少一个单元（一个单元都没有的整文件洞照旧 0 个）、变长不加单元」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/truncate.rs:241`）。那几格只量读者侧，写侧是推的。

盘：两块 4 GiB 内存稀疏盘（`SparseBlockDevice`），起点状态（mkfs → 取号 → 暖机 → 新池新建文件 → 顺序写三个单元）每条用例只建一次，之后每段历史从它克隆。没有崩溃状态枚举（见「这条腿自己的限度」）。

## 各格判定一览

| 格 | 判定 | 打中的形状（一句） | 量过 / 推的 |
|---|---|---|---|
| R2 | 打中 3 处 | ① crates 里「空的新文件」有三种形态，第一个文件那两种（截到 0、覆盖写成 0 字节）各有 1 个声明长度 0 的旧单元，只有别的 inode 新建出来是 0 个单元；C385 的「新文件写 4 KiB」用例落在前两种上，判不出「读残留」与「定义为全零不读」。② 「读新物理位置的残留」那一读法：新落点上 40 次里 19 次全零、1 次是索引节点、20 次是同一个文件的旧数据单元（含被截断放掉、自描述的偏移与洞里那个位置相同的单元）；残留没有可比的校验和，「验」这一步无从定义。③ `read_at` 在洞里答 0、不发读，按它读回的 RMW 在洞那一格与「定义为全零」同一个读回次数 | ① ③ 量过；② 残留分布量过，「用了残留会把截掉的数据带回洞里」是推的 |
| R3 | 打中 2 处，另推翻背景一句 | ① 取内存字节的那一读法，在挂载之后改坏旧单元两份，部分写照样发成、块层读 0 次；读盘那一读法失败、读 2 次。② 内存字节是可写挂载时逐个读回、核过的：改坏落在挂载之前时两种读法都失败，所以 C57 的用例写成「改坏 → 重挂 → 写」判不出两种读法。今天的截断尾单元那条路同样吞掉会话内的坏尾单元（量过）。背景二节「从盘上重建出来的版本这份字节是空的」只在挂载时两份都读不出才成立：盘完好时重挂之后截到单元中间照样发成 | 全部量过；同一次发布里第二次部分写各读到哪一版：推的（攒批没有实现） |
| R4 | 打中，四句里第二句不过 | 镜像造出：三个单元的文件对单元 0 或 1 部分写，I-1.13 违例、挂载态读者读回正确；放开用户动作的 399 段历史里 95 段违例，全在含部分写的历史上，照字面预测与判定 155 段逐段一致。判别子观测不到：「覆盖写变短再变长留洞」与「中间单元部分写两次」两段历史造到扫描一侧看得到的输入逐字段相同（三个单元头、落点、记录 size、对象出生代、截断字段），一段成立一段违例。「有洞的盘报不适用」碰不到这一格（镜像上没有洞）。截断两轮判决 R8「那时成立的只有每次放掉都写区间墓碑、去掉写序」是过度外推：改法乙（见 R4 节）在 399 段上 0 违例 | 镜像、扫、判别子、两个改法的扫都量过（副本）；写放大的公式是推的，三个单元那一格量过 |
| R5 | 打中 2 处（推的为主） | ① 洞只能在尾部：标签 3、4 声明「最后一个存在的单元之后到 size 是洞」，读者按单元号下标取记录，写进洞中间（不紧挨最后一个存在的单元）造出的不连号单元集合今天的格式表示不了。② 跨文件末尾的写按 `read_at` 读被写那一段会报 `RangeBeyondEndOfFile`（量过），照 D4 已定项 4 就成了「读不回 ⇒ 写失败」 | ② 量过；① 推的（引代码） |
| R6 | 没造反例 | 只推了改动点：今天改 size 对别的 inode 在任何写之前拒，R4 的写序那一条按 (inode, 对象出生代) 判、对任意 inode 同样打中 | 推的 |
| R7 | 打中 1 处 | 「注入时必失败、没注入时同一个写必成功」成对用例在不发读的格（洞、别的 inode 新建、内存字节那一读法）没有读可注入，那几格空过；两条位置条目只坏一份时读盘那一读法照两份逐个试的写法会成功，「读坏 ⇒ 必失败」要写明是两份都坏 | 空过那几格的读回次数量过；单份坏是推的；崩溃切点与 K4 照搬是推的，没跑崩溃枚举 |

## R4　截断判决 K6「读法 Z 加写序」的前提重开

被判的那一句：截断第一轮判决 K6「前提是活单元的出生 txg 随偏移不降」（`research/prompts/m4-trunc-r1-main-verification.md:40`），代码侧同一句「前提是活单元的出生 txg 随偏移不降」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/rebuild.rs:615`），判死的那一判「.is_some_and(|newest_below| *newest_below > birth_txg_of(version))」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/rebuild.rs:800`）。

### 打中一：镜像（部分写之后只走扫描重建，活单元被判死）

历史：mkfs → 取号 → 暖机 → 新池新建文件（3000 字节）→ 顺序写三个单元（txg 4）→ 对单元 k 在净荷偏移 100 写 4096 字节（只重写单元 k，txg 5，读旧单元取内存字节）。每一步许可它的那一句：前五步是今天的入口；最后一步是这一轮要开的部分写，D4 已定项 3「先把旧单元整个读回来并验 MAC，改完重算整单元的 MAC，再整单元写到新位置」（`.claude/kb/decisions/04-校验和位置.md:52`）只重写被写的那个单元，没有一句要求连带重写别的单元。

命令（副本里）：

```
SINGLEFS_RUN_SCOPE='种类=target; …' nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test --offline -p singlefs-harness --test partial_write_attack -- --nocapture --test-threads 1 r4_mirror
```

原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-r4-mirror.txt` 第 12–16 行）：

```
test r4_mirror_a_partial_write_on_one_unit_then_the_scan_rebuild ... 观测 R4 起点：三个单元的出生 txg [4, 4, 4]，I-1.13 Holds
观测 R4 部分写单元 0：出生 txg [5, 4, 4]；挂载态读者读回整文件与期望相同 = true；写序一条照字面判死某个活单元 = true；I-1.13 Violated("扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）：扫描重建没有这一条、遍历方向 落点 {盘 0 槽 50184、盘 1 槽 50184}；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 1、偏移 32634）：扫描重建 没有这一条、遍历方向 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）")；违例的不变量 ["I-1.13"]
观测 R4 部分写单元 1：出生 txg [4, 5, 4]；挂载态读者读回整文件与期望相同 = true；写序一条照字面判死某个活单元 = true；I-1.13 Violated("扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 1 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 4）：扫描重建没有这一条、遍历方向 落点 {盘 0 槽 50186、盘 1 槽 50186}；「每个 inode 的数据单元集合」差 1 处，第一处 key （inode 1、偏移 65268）：扫描重建 没有这一条、遍历方向 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 4）")；违例的不变量 ["I-1.13"]
观测 R4 部分写单元 2：出生 txg [4, 4, 5]；挂载态读者读回整文件与期望相同 = true；写序一条照字面判死某个活单元 = false；I-1.13 Holds；违例的不变量 []
观测 R4 汇总：三个单元各部分写一次，I-1.13 违例 2 次
```

用户动作放开扫（只固定前缀「三个单元的文件」，之后由用户定的动作逐步取遍）：第一次扫五种动作（部分写单元 0 / 1 / 2、整份覆盖写、变长一个净荷）取长度 1–3 全部组合 155 段；第二次扫七种（再加覆盖写变短成一个单元、截到第二个单元中间）399 段。原样汇总行：

```
观测 R4 扫汇总：历史 155 段，I-1.13 违例 79 段，判定与照字面预测一致 155 段，挂钟 29.228383316s
test sweep_today_rules ... 观测 扫汇总（改法开着 = false）：历史 399 段（含部分写 315 段），I-1.13 违例 95 段（含部分写的 95、不含部分写的 0），I-1.13 之外有违例 0 段，读者读回不同 0 段，挂钟 188.631395122s
```

（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt:188`、`research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-today.txt:5`。）「照字面预测」= 部分写之后各单元的出生 txg 里有没有一个单元、偏移更小的单元出生 txg 比它大；155 段逐段与 checker 一致，说明违例全由写序那一条造成。399 段里不含部分写的 84 段一段都不违例：今天的入口上这一条成立，开了部分写才破。读者在全部 399 段上读回正确，坏的是 checker 的扫描一侧，不是数据。

### 打中二：判别子观测不到

两段历史，都从三个单元的文件（txg 4）起：

- 变长留洞：覆盖写变短成一个单元（txg 5，单元 1、2 放掉、还在盘上）→ 变长回 3 × 净荷 − 7（txg 6，单元 0 补零重写，之后是洞）。
- 中间部分写：对单元 0 部分写两次（txg 5、txg 6）。

原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-discriminator-estimate.txt` 第 5–12 行）：

```
test r4_hole_history_and_partial_write_history_look_alike_to_the_scan ... 观测 R4 判别子 变长留洞：记录 size 97895 对象出生代 3 截断字段 InodeRecordTruncationField { truncation_txg: CheckpointTxg(0), truncated_size_in_bytes: 0 }；遍历方向的单元数 1；I-1.13 Holds
观测 R4 判别子 变长留洞 单元 0（这一版指着的 SlotNumber(50190)）：对象 1 对象出生代 3 锚点偏移 0 出生 txg 6 声明长度 32634 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 6 }
观测 R4 判别子 变长留洞 单元 1（起点那一版的落点（遍历方向已不指） SlotNumber(50184)）：对象 1 对象出生代 3 锚点偏移 32634 出生 txg 4 声明长度 32634 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 3 }
观测 R4 判别子 变长留洞 单元 2（起点那一版的落点（遍历方向已不指） SlotNumber(50186)）：对象 1 对象出生代 3 锚点偏移 65268 出生 txg 4 声明长度 32627 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 4 }
观测 R4 判别子 中间部分写：记录 size 97895 对象出生代 3 截断字段 InodeRecordTruncationField { truncation_txg: CheckpointTxg(0), truncated_size_in_bytes: 0 }；遍历方向的单元数 3；I-1.13 Violated("扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）：扫描重建没有这一条、遍历方向 落点 {盘 0 槽 50184、盘 1 槽 50184}；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 1、偏移 32634）：扫描重建 没有这一条、遍历方向 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）")
观测 R4 判别子 中间部分写 单元 0（这一版指着的 SlotNumber(50190)）：对象 1 对象出生代 3 锚点偏移 0 出生 txg 6 声明长度 32634 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 6 }
观测 R4 判别子 中间部分写 单元 1（这一版指着的 SlotNumber(50184)）：对象 1 对象出生代 3 锚点偏移 32634 出生 txg 4 声明长度 32634 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 3 }
观测 R4 判别子 中间部分写 单元 2（这一版指着的 SlotNumber(50186)）：对象 1 对象出生代 3 锚点偏移 65268 出生 txg 4 声明长度 32627 写序 WriteOrder { instance: InstanceGeneration(1), transaction: 4 }
```

扫描一侧读的是单元头与 inode 叶容器里的记录（不读树节点，背景四·1 题面）。两段历史里三个单元头的对象、对象出生代、锚点偏移、出生 txg、声明长度、写序与落点逐项相同，记录的 size、对象出生代、截断字段相同（记录里另外不同的只有改动计数与写入时间，读法 Z 不读它们；单元 0 的净荷字节不同，读法 Z 也不读）。遍历方向一段 1 个单元、一段 3 个。扫描一侧不管按什么规则判单元 1、2 的死活，两段里必有一段与遍历方向不同。

### 四句

1. **分不分辨臂**：分辨。今天的读法 Z 加写序在镜像上中；「有洞的盘报不适用」在镜像上也中——镜像不是有洞的盘：「观测 R4 候选：镜像上第一个文件 size 97895，存在的单元 3，size 要的单元 3，有洞 = false」（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt:24`）；「写到 o 时把 o 之后的单元一起重写」的仿真不中（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt:25`，分几次发布仿的，不是同一次发布）；改法乙不中（下面）。
2. **系统当时看不看得到判别它的东西**：看不到（打中二）。这是「判别子观测不到」那一形：不改盘上写什么，只改扫描一侧的判法，修不了；要么把判别子写上盘（截断字段、区间墓碑），要么去掉判别的必要（o 之后一起重写，使出生 txg 随偏移不降重新成立）。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 那张表，改判据，改完是新一轮。
3. **满足的是判据字面的哪一个分句**：K6 自己写的重开条款「前提是活单元的出生 txg 随偏移不降」那一句的前提（部分写就是它点名的「写到偏移 o 而不重写 o 之后的单元」的入口）；I-1.13 违例落在「中央映射」与「每个 inode 的数据单元集合」两张表，`inode 记录集合` 一处不差。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：见下面的改法表。

### 各改法在打中的那几格上还中不中

打中的格：镜像（部分写单元 0 或 1）与判别子那一对历史。每一格标「量过」（贴副本上的原样输出）或「推的」。

| 改法 | 镜像那一格 | 判别子那一对 | 代价 | 标 |
|---|---|---|---|---|
| 今天的读法 Z 加写序原样 | 中：399 段里 95 段违例 | 中（中间部分写那一段违例） | 无 | 量过 |
| 有洞的盘报不适用 | 中：镜像上「有洞 = false」，这一条的触发条件不成立，I-1.13 照判 | 变长留洞那一段报不适用、中间部分写那一段照判违例 | 无 | 触发条件量过，结局推的 |
| 写到 o 时把 o 之后的单元一起重写 | 不中：「部分写单元 0 再把单元 1、2 原样重写之后出生 txg [5, 6, 7]，I-1.13 Holds」 | 不中（中间部分写那一段不再有比更小偏移旧的单元） | 一次部分写写 (n − k) 个数据单元、每个两份：1 GiB 文件 n = 32903、k 均匀时平均 16452 个、约 1078198272 字节数据写，换一次 4 KiB 的写；三个单元的文件 k = 0 时数据单元写与整份顺序写相同（196608 字节，量过那一格），另加罩着它们的下段叶 | 不中是仿真量过（分三次发布，不是同一次）；1 GiB 那一格是推的 |
| 区间墓碑（截断两轮判决 R8） | 不中（推的：墓碑把判别子写上盘） | 不中（推的） | 今天的扫描重建遇到非整对象的墓碑区间整张交回没定：「ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/rebuild.rs:700`），要连同墓碑树的写者、扫描重建与遍历方向一起改；每次放掉多一条墓碑记录 | 推的 |
| 先建计数实验 | 不是改法，碰不到格；它量的是上面「o 之后一起重写」那一行的写放大 | — | — | — |
| 改法乙（攻方提的）：放掉单元的整份写照截断意图那一写法写截断字段（这次 txg, N），不放单元的整份写照抄上一版；部分写照抄；去掉写序那一条 | 不中 | 不中（变长留洞那一段的截断字段非 0，第四条判死单元 1、2；中间部分写那一段截断字段 0、写序那一条去掉，单元 1、2 认领） | 不加盘上字段（截断字段已有），只改整份写写不写它；整份写不再「清零」 | 量过：399 段 0 违例、I-1.13 之外 0 违例、读者读回不同 0 段 |
| 改法甲（攻方提的，改法乙的变体）：不放单元的整份写照旧清零 | 不中 | 不中 | 同上 | 量过：399 段 1 段违例，见下 |

改法乙两半的原样汇总行（`research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-carry-first.txt:5`、`research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-carry-second.txt:5`）：

```
test sweep_with_the_candidate_fix ... 观测 扫汇总（改法开着 = true）：历史 200 段（含部分写 198 段），I-1.13 违例 0 段（含部分写的 0、不含部分写的 0），I-1.13 之外有违例 0 段，读者读回不同 0 段，挂钟 69.587438793s
test sweep_with_the_candidate_fix ... 观测 扫汇总（改法开着 = true）：历史 199 段（含部分写 117 段），I-1.13 违例 0 段（含部分写的 0、不含部分写的 0），I-1.13 之外有违例 0 段，读者读回不同 0 段，挂钟 72.7645844s
```

改法甲后一半（`research/prompts/m4-partial-write-r1-opus-model/out/run-sweep-fix-clear-second.txt` 第 9–10 行）：

```
test sweep_with_the_candidate_fix ... 观测 扫（改法开着 = true）：历史 [OverwriteShorterToOneUnit, OverwriteShorterToOneUnit, ExtendByOnePayload] 出生 txg [7] 违例 ["I-1.13"] 读者读回相同 = true
观测 扫汇总（改法开着 = true）：历史 199 段（含部分写 117 段），I-1.13 违例 1 段（含部分写的 0、不含部分写的 1），I-1.13 之外有违例 0 段，读者读回不同 0 段，挂钟 68.596087243s
```

那一段：第一次变短写字段 (txg 5, 一个单元)，第二次「一个单元 → 一个单元」不放单元、照今天的写法清零，字段丢了；变长之后单元 1、2（出生 txg 4）落回 size 之内，写序那一条去掉了，第四条又没有字段可判，被认回来。这一段不含部分写，说明「整份写清零」（截断第二轮判决第六节用户定案「整份写清零（R2 的整份写那一句取清零）」，`research/prompts/m4-trunc-r2-main-verification.md:69`）只在写序那一条在的时候才对；代码侧那句理由「这一版从偏移 0 重写了全部单元，没有比它早的单元要靠截断字段判死。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:8963`）在「整份写变短、之后变长」这一形上不成立——比它早的单元就是这次放掉、还在盘上的那几个，今天是写序那一条替它兜着。

**过度外推**：截断第二轮判决 R8 写「pwrite、punch 进来时 R3 与写序那一条都要重开，那时成立的只有」区间墓碑加去掉写序（`research/prompts/m4-trunc-r2-main-verification.md:52`）。改法乙不写区间墓碑，在我这 399 段上同样成立，「只有」那一半是过度外推（反例只在我的模型上，被攻过零轮）。

改法乙的前提与没扫到的：它靠「存在的单元从 0 起连号」（截断第一轮判决 K7 T-3），一次「整个替换」的字段之所以罩得住更早放掉的单元，是因为后来的放掉要先有更多存在的单元，那些位置已被更新的单元盖过（推的）。R5 若放开写进洞中间（单元不再连号），要重扫。没扫：快照、管理员回退（截断第二轮 R5 字段随记录取 R_old）、整理搬迁、别的 inode、单元数过 144 的文件、历史长度大于 3。改法乙改了一条用户定案（「整份写清零」），交用户时要写明。

写账（副本，三个单元的文件，`research/prompts/m4-partial-write-r1-opus-model/out/run-extra.txt` 第 35–36 行）：部分写单元 0 一次发布「合计 WriteCallsAndBytes { write_calls: 31, written_bytes: 508416 }」，其中数据单元「DataUnit: WriteCallsAndBytes { write_calls: 2, written_bytes: 65536 }」；整份顺序写三个单元「合计 WriteCallsAndBytes { write_calls: 39, written_bytes: 655872 }」。小文件上一次发布的元数据写占大头，「o 之后一起重写」的代价随文件长度涨，小文件上量不出来。

## R2　「首次写」包括哪几格

C385 的检查列：「有部分写路径时造「新文件写 4 KiB」的用例，断言读回次数与定案一致（0 或 1），把读法换成另一种检查必须由绿转红」（`.claude/kb/checks-owed.md:271`）。D4 射程：「新分配的单元没有旧内容可读：」（`.claude/kb/decisions/04-校验和位置.md:56`）两种读法。

### 打中一：「新文件」在 crates 里不是一格，C385 的用例落在有旧单元的那两种上判不出两种读法

原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt` 第 8–13 行）：

```
test r2_forms_of_an_empty_file_and_read_at_over_the_cells ... 观测 R2 空文件形态一（新建 inode 2，size 0）：单元 0 个；read_at(0, 0) 成功，0 字节、全零 = true、块层读 0 次
观测 R2 空文件形态二（第一个文件截到 0，之后走了 1 步）：存在的单元 1，单元 0 声明长度 Some(Ok(0))；单元 1 个；read_at(0, 0) 成功，0 字节、全零 = true、块层读 0 次
观测 R2 空文件形态三（第一个文件整份覆盖写成 0 字节）：发成 = true；存在的单元 1；单元 1 个；read_at(0, 0) 成功，0 字节、全零 = true、块层读 0 次
观测 R5 跨末尾：size 3000 的文件 单元 1 个；read_at(2000, 4096) 失败 RangeBeyondEndOfFile { file_size_in_bytes: 3000, offset: FileOffsetInBytes(2000), length_in_bytes: 4096 }
观测 R5 跨末尾：size 3000 的文件只读 size 之内那一段 单元 1 个；read_at(2000, 1000) 成功，1000 字节、全零 = false、块层读 1 次
观测 R2 洞：变长到三个净荷之后存在的单元 1，单元 1 个；read_at(32734, 4096) 成功，4096 字节、全零 = true、块层读 0 次
```

| 格（size 0 或洞里写 4 KiB） | 有没有旧单元 | 读残留 | 定义为全零不读 | 读盘验整单元（有旧单元时） | 取内存字节 | 按 `read_at` 读被写区间 |
|---|---|---|---|---|---|---|
| 别的 inode 新建（形态一） | 无（量过：0 个单元） | 读一次新落点 | 0 | — | — | 区间越过 size ⇒ 报错；截到 size ⇒ 0 次（量过 `read_at(0, 0)`） |
| 第一个文件截到 0（形态二） | 有，声明长度 0（量过） | 不适用：有旧单元，读的是它 | 不适用 | 1 次块层读（第一份好时） | 0 | 同上 |
| 第一个文件整份覆盖写成 0 字节（形态三） | 有，声明长度 0（量过） | 不适用 | 不适用 | 1 次 | 0 | 同上 |
| 第一个文件尾部的洞（变长之后） | 无（量过：存在的单元 1 个、size 罩 3 个） | 读一次新落点 | 0 | — | — | 0 次（量过） |
| 已有单元的中间 | 有 | — | — | 1 次（量过，R3 节） | 0（量过） | 1 次（量过 `read_at(2000, 1000)`） |

推出来的：第一个文件只有形态二、三两种空法（`publish_first_file` 收整份内容，空内容也照「size 0 时 1 个单元」装，形态三量过这一点），在这两格上「读残留」与「定义为全零」都去读那个声明长度 0 的旧单元、次数一样。C385 的用例按「新文件 = 第一个文件 size 0」造，把读法从一种换成另一种不会由绿转红——它要判的正是这两种读法。分得开这两种读法的格只有「别的 inode 新建」（要 R6 先放开别的 inode）与「第一个文件尾部的洞」。另一处：读回次数数什么没写——`ReadPathObservation` 有两个数，「data_units_read: u64,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:75`）与「device_reads_issued: u64,」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:81`）；R3 节量到第一份坏时读盘那一读法块层读 2 次、读回的单元仍是 1 个，「0 或 1」要写明是哪一个数。

四句：分辨臂（分辨「读残留」与「定义为全零」两臂的用例在形态二、三上分不开，在形态一与洞上分得开）；被判的是用例，系统看得到判别它的东西（计数器在）；满足的是 C385 检查列「新文件写 4 KiB」那一分句的字面；改法：把用例的格写成「第一个文件尾部的洞」或「别的 inode 新建」，并写明数块层读还是数单元——两样都不碰「首次写读不读」本身。

### 打中二：「读新物理位置的残留」读到的是什么

历史：三个单元的文件 → 截到 100（意图与最后一批，单元 1、2 放掉）→ 变长回三个净荷（洞）→ 对单元 0 反复部分写 40 次，每次记新分配给单元 0 的那个落点在这次写之前的内容。原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-extra.txt` 第 5–12 行，余下的见原文件）：

```
test r2_residue_at_the_newly_allocated_slot ... 观测 R2 残留：截断放掉的单元 1、2 的落点 [SlotNumber(50184), SlotNumber(50186)]；变长之后存在的单元 1
观测 R2 残留：第 0 次部分写的新落点 SlotNumber(50192) 写之前是 全零（从没写过）
观测 R2 残留：第 19 次部分写的新落点 SlotNumber(50178) 写之前是 不是数据单元（WrongUnitClass { expected: 1, found: 2 }）
观测 R2 残留：第 20 次部分写的新落点 SlotNumber(50180) 写之前是 码 1 数据单元：对象 1、对象出生代 3、锚点偏移 0、出生 txg 3、声明长度 3000；被截断放掉的落点 = false
观测 R2 残留：第 21 次部分写的新落点 SlotNumber(50182) 写之前是 码 1 数据单元：对象 1、对象出生代 3、锚点偏移 0、出生 txg 4、声明长度 32634；被截断放掉的落点 = false
观测 R2 残留：第 22 次部分写的新落点 SlotNumber(50184) 写之前是 码 1 数据单元：对象 1、对象出生代 3、锚点偏移 32634、出生 txg 4、声明长度 32634；被截断放掉的落点 = true
观测 R2 残留：第 23 次部分写的新落点 SlotNumber(50186) 写之前是 码 1 数据单元：对象 1、对象出生代 3、锚点偏移 65268、出生 txg 4、声明长度 32627；被截断放掉的落点 = true
观测 R2 残留：第 24 次部分写的新落点 SlotNumber(50188) 写之前是 码 1 数据单元：对象 1、对象出生代 3、锚点偏移 0、出生 txg 5、声明长度 100；被截断放掉的落点 = false
```

40 次里 19 次全零（盘上从没写过）、1 次是码 2 索引节点、20 次是同一个文件的旧数据单元；第 22 次拿到的正是截断放掉的单元 1，头里对象、对象出生代、锚点偏移（32634，就是洞里单元 1 的位置）与「洞里第一次写单元 1」要的身份一样。由此（推的，原型写不进洞，没走到那一步）：

- 残留没有可比的整单元校验和：位置条目里的校验和只给被指着的单元；新落点没有指针指着它，「验整单元校验」那一步没有期望值。照 D4 已定项 4「旧单元读不回来、或读回来 MAC 验不过时，那个部分写失败并返错」（`.claude/kb/decisions/04-校验和位置.md:70`）字面去判，全零那 19 次连单元头都解不开，首次写在新盘区上必失败；不判，就只剩「读了、不看」这一步，它唯一的可观测效果是那一读报 EIO 时写失败。
- 残留里同一个文件同一个位置的旧单元能过头部自描述的每一项核对（上面第 22 次）。要是读法写成「读到合法单元就当旧内容用」，截断放掉的数据会回到洞里，违反「读者读洞答 0」（K5）。规则里要明写「残留的内容不进新单元，新单元按全零补」。

四句：分辨臂（分辨「读残留并用它」「读残留不看」「定义为全零」三种）；系统看得到——残留的头读得出，只是它不该被信；满足的是 D4 射程「真的去读那个物理位置的残留（别人的死数据，无语义）」那一分句：量出来残留常常是自己的死数据、而且自描述是对的，「无语义」不等于「看不出语义」；改法：首次写那一格的规则写成「发一次读、只判读不读得回、内容丢掉按全零补」，或回到「不发读」——前者是 C385 用户定的「读」，那一读不验校验和（没有期望值），这一点要交用户知道。

## R3　读回旧单元读哪里

两条臂（原型里各一条，`prototype/partial_write_attack.rs` 的 `OldUnitReading`）：取内存里上一版的字节，今天截断尾单元那一读「let bytes = &previous.unit(TransactionUnit::Data(rewrite.unit)).bytes;」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:13189`）；读盘，按两条位置条目逐份读整单元、核位置条目里的整单元 CRC-32C（与释放前核校验和那一读同一个判法，「fn read_one_copy_and_check_it_against_its_location_entry」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:5189`）），两份都对不上就失败。故障：把单元 1 的两份各翻一个净荷字节。

### 打中一：内存字节吞掉会话内注入的坏旧单元

原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt` 第 15–22 行）：

```
test r3_injected_corruption_of_the_old_unit_under_the_two_readings ... 观测 R3 改坏旧单元 = false 读法 MemoryBytesOfThePreviousVersion：部分写结果 Ok(())，块层读 0 次，之后读者读整文件 读得出，违例的不变量 ["I-1.13"]
观测 R3 改坏旧单元 = false 读法 DiskWithTheWholeUnitChecksum：部分写结果 Ok(())，块层读 1 次，之后读者读整文件 读得出，违例的不变量 ["I-1.13"]
观测 R3 改坏旧单元 = true 读法 MemoryBytesOfThePreviousVersion：部分写结果 Ok(())，块层读 0 次，之后读者读整文件 读得出，违例的不变量 ["I-2.1", "I-4.8", "I-7.4", "I-1.13"]
观测 R3 改坏旧单元 = true 读法 DiskWithTheWholeUnitChecksum：部分写结果 Err("读回旧单元：两份都读不回或整单元校验和对不上 ⇒ 部分写失败（D4 已定项 4）")，块层读 2 次，之后读者读整文件 DataUnitChecksumMismatchEverywhere { unit_index_in_file: DataUnitIndexInFile(1), hinted_slot: SlotNumber(50184) }，违例的不变量 ["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
ok
test r3_the_truncation_tail_path_today ... 观测 R3 截断尾单元：旧尾单元两份都改坏，同一进程里截到 65368：Ok(())；违例的不变量 ["I-2.1", "I-4.8", "I-7.4"]
观测 R3 截断尾单元：盘完好，同一进程里截到 65368：Ok(())
观测 R3 截断尾单元：盘完好，重开可写挂载之后（现行版本里单元 2 的字节为空 = false）截到 65368：Ok("发成")
```

（I-1.13 那几处违例是 R4 那一形：部分写的是单元 1，单元 2 被写序判死。I-2.1、I-4.8、I-7.4 是改坏的旧副本留在根环里更早那几版下面，推的。）没注入时两种读法同一个写都成功、注入时读盘那一读法失败——C57 要的成对用例在读盘那一臂上成立；内存字节那一臂注入时照样成功、一次块层读都不发，C57「判据是那个写必须失败」（`.claude/kb/checks-owed.md:54`）判它红。今天的截断尾单元那条路（未改的代码）同样：旧尾单元两份都坏，同一进程里截到单元中间照样发成。

内存字节那一臂没有零填充：它用的是正确的旧内容，数据没丢。它破的是字面——D4 已定项 3「先把旧单元整个读回来并验 MAC」（`.claude/kb/decisions/04-校验和位置.md:52`）与 C57 的用例——不是 D4 已定项 4 防的那件事（「一段数据丢了」变成「一段合法的零」）。主 agent 判这一格时要分开：按字面它不合格；按已定项 4 的理由它不造假。

### 打中二：内存字节从哪来——可写挂载逐个读回、核过；坏落在挂载前后决定 C57 用例分不分得开两臂

原样输出（`research/prompts/m4-partial-write-r1-opus-model/out/run-extra.txt` 第 30–33 行）：

```
test r3_memory_bytes_come_from_the_mount_and_the_order_of_the_fault_decides ... 观测 R3 改坏在挂载之前（挂载后内存里单元 1 的字节为空 = true）读法 MemoryBytesOfThePreviousVersion：Err("内存里这一版的字节解不开：TooShort")，块层读 0 次
观测 R3 改坏在挂载之前（挂载后内存里单元 1 的字节为空 = true）读法 DiskWithTheWholeUnitChecksum：Err("读回旧单元：两份都读不回或整单元校验和对不上 ⇒ 部分写失败（D4 已定项 4）")，块层读 2 次
观测 R3 改坏在挂载之后 读法 MemoryBytesOfThePreviousVersion：Ok(())，块层读 0 次
观测 R3 改坏在挂载之后 读法 DiskWithTheWholeUnitChecksum：Err("读回旧单元：两份都读不回或整单元校验和对不上 ⇒ 部分写失败（D4 已定项 4）")，块层读 2 次
```

依据：重建上一版时「提示与映射都读不出的**数据单元**不算失败：照抄它的位置项、不读内容，那一项的字节是空的」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/recovery.rs:3194`）；读得出的都读回来、核过放进内存（「fn read_data_unit_or_nothing_when_unreadable(」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/recovery.rs:3631`））。所以：

- 背景二节那一句「从盘上重建出来的版本这份字节是空的」说窄了：只有挂载时两份都读不出的单元是空的；盘完好时重挂之后字节在，截到单元中间照样发成（上面最后一行）。
- 内存字节那一臂的「验」发生在挂载时（或本进程写出它时），不在部分写时。C57 的用例写成「改坏 → 重挂 → 部分写」时两臂都失败，判不出；写成「挂着改坏 → 部分写」时才分得开。用例要写明故障在挂载之后注入。

四句（两处合并）：分辨臂（读盘 / 内存字节）；系统看得到——内存字节那一臂部分写时手里没有盘上此刻的字节，它看不到挂载之后坏掉的那一份，这正是它被打中的原因；满足的是 C57 检查列「判据是那个写必须失败」与 D4 已定项 3「先把旧单元整个读回来并验 MAC」；改法：截断尾单元那条路一起改成读盘，与部分写共用一读（量过：读盘那一臂在四种情形里都按字面判）。代价：每次重写尾单元多一次整单元读（1 次块层读，第一份坏时 2 次，量过）。

### 同一次发布里对同一单元第二次部分写各读到哪一版（推的）

今天每个请求同步发布一次：第一次部分写的新单元在第二次之前已经落盘、内存里的现行版本也指着它，两臂读到的都是第一次写之后那一版（R4 节 399 段的扫里有连着两次部分写同一单元的历史，「读者读回不同 0 段」，取内存字节那一臂量过；读盘那一臂是推的）。攒批（D16 已定项 10）之后，一次发布里装几次改动：读盘那一臂按「现行版本的位置条目」去读，读到的是上一次已发布那一版、丢掉同一批里前一次的改动，除非读的是批里自己的缓冲；内存字节那一臂读批里的版本，不丢。截断第二轮判决把按出生 txg 比新旧那几处的攒批欠账记在 R7（`research/prompts/m4-trunc-r2-main-verification.md:51`），读回读哪一版应当一起记进去。

## R5　写跨文件末尾、写进截断留下的旧区

### 打中一（推的，引代码）：洞只能在尾部，写进洞中间造不出今天的格式表示得了的样子

K5「之后到 N 都是洞、不写单元」（`research/prompts/m4-trunc-r1-main-verification.md:39`），K7 T-3「存在的单元从 0 起连号」（`research/prompts/m4-trunc-r1-main-verification.md:41`）。读者按单元号当下标取记录：「.get(usize::try_from(unit_number).expect("单元序号装得进 usize"))」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:1019`），取不到才当「尾部的洞（标签 3、4、0，判决 m4-trunc-r1 K5）：这一段答 0、不读盘。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:1022`）。

历史：三个单元截到 100 → 变长到 3 × 净荷（洞，存在的单元 1 个，量过）→ 在单元 2 的位置写 4 KiB。单元 2 不紧挨最后一个存在的单元（单元 0）：要么把单元 1 当全零单元一起写出来（一次写放大成「洞头到被写位置」那么多个单元：变长到 2 GiB 的空文件在 1 GiB 处写 4 KiB 要写约 32903 个全零单元，推的），要么改格式让洞能在中间（上段叶标签、下段叶条目、读者、T-3、I-4.6 一起改）。条款里两条都没写。写序那一条在这一格上照 R4 打中：洞里先写单元 2、再写单元 1，单元 2 被判死（推的；原型写不进洞，没造这面镜像）。

### 打中二（量过）：跨文件末尾时按 `read_at` 读被写那一段会失败

`read_at` 先判「if end_exclusive > file_size_in_bytes {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_read.rs:994`），原样输出见 R2 节那一段：「read_at(2000, 4096) 失败 RangeBeyondEndOfFile」（`research/prompts/m4-partial-write-r1-opus-model/out/run-all.txt:11`）。照 D4 已定项 4「读不回 ⇒ 写失败」去接，每一次追加写（跨过末尾）都失败。正推若取「挂载态读者 `read_at` 那条路」作 R3 的读盘候选，要写明读的是整单元（`read_at` 读的是区间，裁到 size、洞答 0 不读），否则这一格与 R2 的洞那一格都不对。

### 截断字段 T、N_w 的认领判法（推的，没打中）

截在单元边界（T ≠ 0）再变长、之后往洞里写单元 j（j 紧挨最后一个存在的单元）：新单元出生 txg 大于 T，第四条「单元头出生 txg < T 的，不认领」（`research/prompts/m4-trunc-r2-main-verification.md:32`）不判死它；同一位置上截断放掉的旧单元与新单元同一个对象身份，按写序择当前版本择新的那一个。第四条在这一格成立；不成立的是写序那一条（R4）。部分写本身不碰截断字段（原型照抄），与「各批、最后一批不碰记录」「变长照抄」同形。

## R6　哪些 inode 先有部分写（没造反例）

推的，只列改动点：今天改 size 对别的 inode 在任何写之前拒（`TruncationRefusal::SizeChangeOfAnInodeOtherThanTheFirstFileIsNotDecidedByTheClauses {`，`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:10488`）；别的文件的整份写走自己的计划（`fn plan_of_a_whole_write_of_a_file_other_than_the_first<'content>(`，`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/transaction.rs:9361`）与 `DataOfAFileOtherThanTheFirst` 角色，我的原型只写第一个文件、没碰这一族。两点与别的格耦合：

- R4 的写序那一条按 (inode, 对象出生代) 判（`newest_birth_txg_below_each_data_unit`），任意 inode 一开部分写就同样打中；R4 的改法要先于或随 R6 一起做，不能只给第一个文件修。
- R2 打中一：分得开「读残留」与「定义为全零」的 0 单元新文件只有别的 inode 才有。只给第一个文件开部分写时，C385 的用例只能落在第一个文件尾部的洞上。

## R7　故障与崩溃的验收面

### 打中（量过 + 推的）：成对用例在不发读的格上空过

验收「注入时必失败、没注入时同一个写必成功」要有一次读可以注入。量过不发块层读的格：洞里（`read_at` 0 次）、取内存字节那一读法（0 次）、size 0 的新 inode（0 个单元）。这几格上注入什么都不改变结局，成对用例恒绿，证明不了会红。「读残留」那一读法在首次写那几格上发一次读，但读哪个槽要等分配器在发布里定下才知道，注入设备的落点只能事先点名偏移（「OffsetExactly(DeviceOffsetInBytes),」，`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/fault_injection.rs:433`），要另加「这次发布里第一次读单元区」那一类落点（推的）。

两份只坏一份：数据单元两条位置条目，读盘那一读法照「逐份试、第一份对得上就用」写（与释放前核校验和同一判法），只坏一份时会读第二份、写成功（推的，原型只量了两份都坏：块层读 2 次、失败）。「读坏 ⇒ 必失败」要写明两份都坏；只坏一份时要求成功，否则对的实现过不了用例。注入的种类要连读报错（`ReadFails`）与读回被翻位（「ReadReturnsCorruptedBytes { flipped_bit: FlippedBit },」，`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-harness/src/fault_injection.rs:150`）各一格；「读回长度不对」在块设备契约里不会发生（读要么填满缓冲、要么报错），落到单元上是声明长度与查找路径对不上，那一判在读者里（推的）。

### 崩溃切点（推的，没跑崩溃枚举）

原型的部分写走的是变长那一条一次发布（写账：「合计 WriteCallsAndBytes { write_calls: 31, written_bytes: 508416 }」，`research/prompts/m4-partial-write-r1-opus-model/out/run-extra.txt:35`），持久次序与变长相同：新单元两份、索引节点、journal 记录、根槽、系统配置轮换。要切的点：新单元已落而根没发（恢复到上一版，旧单元原样）、journal 记录已落而根没落（重放）、根已落。R4 打中的那一形在「恢复到部分写那一版」的每个崩溃状态上照样让 I-1.13 红（推的，由最终状态量过的违例推出）：R4 不修，部分写的崩溃流在崩溃点重放里过不了全域 0 红。

### 快照下被换下的旧单元进 deadlist（K4 照搬，推的）

K4「截断换下的单元（尾部与重写前的单元 k）经 `deadlist_changes_of_this_publish` 判生死」（`research/prompts/m4-trunc-r1-main-verification.md:38`）。原型的部分写走同一条发布路（`publish_deletion_step`），被换下的单元 k 走同一处判生死，照搬在代码路径上成立；没在快照下跑。R4 若取「o 之后一起重写」，快照下每次部分写要把 n − k 个单元钉进 deadlist，空间放大与写放大同量级（推的）。

## 几个改法各修哪一格（全部只在我的模型上量过或推的，被攻过零轮）

| 改法 | 修的格 | 修不到的格 | 标 |
|---|---|---|---|
| 改法乙：放掉单元的整份写照截断意图那一写法写截断字段，不放的照抄；部分写照抄；去掉写序那一条 | R4 镜像、判别子那一对、399 段全部 | R5 洞中间（要单元连号）；快照、回退、整理、别的 inode 没扫 | 量过（副本，399 段 0 违例）；改了截断第二轮的用户定案「整份写清零」 |
| 写到 o 时把 o 之后的单元一起重写 | R4 镜像 | 写放大随文件长度涨（1 GiB 文件平均约 1078198272 字节换一次 4 KiB 写）；快照下钉进 deadlist 的单元同量 | 不中是仿真量过（分三次发布）；放大是推的 |
| 截断尾单元那条路一起改成读盘（与部分写共用一读） | R3 两处：会话内注入的坏旧单元被拦下；C57 用例不再依赖故障落在挂载前后 | 同一次发布里的第二次部分写（攒批之后要读批内缓冲） | 读盘那一臂的四种情形量过；攒批推的 |
| C385 用例的格改成「第一个文件尾部的洞」或「别的 inode 新建」，并写明数块层读还是数单元 | R2 打中一 | 不碰「读不读」本身 | 各格读回次数量过 |
| 首次写那一格写成「发一次读、只判读不读得回、内容丢掉按全零补」 | R2 打中二（残留的自描述对得上也不用它） | 残留没有期望校验和这一事实不变；读那一下只剩 EIO 一种可判的结局 | 残留分布量过，规则效果推的 |
| 故障用例写明「两份都坏才失败、只坏一份必成功」，在不发读的格上不出成对用例或改成断言「0 次读」 | R7 空过那几格 | 首次写读残留那一格的注入落点要新加 | 推的 |

## 没打中的形状

- R4：截在单元边界再变长、往洞里紧挨最后一个单元写（第四条照判，推的）；部分写只写最后一个单元（量过：单元 2 那一格成立）；399 段里不含部分写的 84 段（量过：0 违例）。取样范围：第一个文件、起点三个单元、动作七种、长度 1–3 全部组合、一个起点池；没有快照、回退、整理、重挂、别的 inode、多叶（> 144 单元）。
- R3：读盘那一臂在没注入、挂载前注入、挂载后注入三种情形里都按字面判（量过）；两份只坏一份没跑。
- R5：截断字段与紧挨最后一个单元的洞里写不冲突（推的）。
- R6：没造。
- R7：崩溃切点没枚举；快照下没跑。

## 这条腿自己的限度

- 原型是变长那条一次发布路改出来的，只能「重写一个已存在的单元、不改 size」。洞里写、跨末尾写、加单元都没有写侧原型，那几格的写侧全是推的。
- 改法乙的开关是测试里按「这次放不放单元」现算后塞进核心层的，不是核心层自己判；I-9.39 与 GPU 判器那一侧没改、没跑（扫里「I-1.13 之外有违例 0 段」只说明池级 CPU checker 别的不变量没红）。
- 没跑崩溃状态枚举（`enumerate_crash_states`）：要另建一个依赖 checker 档包的二进制 crate，两小时内没做；崩溃切点那一节全是推的。
- 每段历史只有一个起点池；用户动作的放开扫只到长度 3。
- 「内存字节没有造假、只是破字面」是我的读法，按已定项 4 的理由判它合不合格归主 agent。
- 副本上量的数，不是入库装置上的数；改法都被攻过零轮。

## 没做什么

- 不判正推、辩方那几格，不读正推腿的产出（禁读清单）。
- 没跑任何门禁阶段、checker 档测试、GPU；没起崩溃枚举。
- 没改仓里的任何代码；原型与改法只在攻方草稿副本里，补丁在模型目录。
- 草稿副本（冻结副本拷贝加编译目录，交回前 `du -sh` 记 2.1G）已删；草稿目录里只剩日志、补丁与生成 rerun.sh 的辅助脚本（128K）。

## 要主 agent 跑的命令

- 复跑（判决要引复跑数时）：见开头「复跑命令与文件」，工作目录随意，约 10 分钟，主 agent 用 run_in_background 起；看 `<草稿目录>/out/` 每份的「观测」行与模型目录 `out/` 对应那一份逐行比。
- 原型挂内存包装时我用的是 2G（本机内存包装的 slice 此刻被别的会话占到总上限附近，8G 的一次排了 240 秒没起来、我停了它，见草稿目录 `run-r4-sweep-estimate.txt`）；峰值约 923 MiB（钩子提醒里记的），2G 够。

## cite-check

```
  ✓ 核了 26 处引文、14 处裸行号，对上 26 处，没判 14 处（其中裸行号没有引文可比 14 处；逐处列在上面）
```
