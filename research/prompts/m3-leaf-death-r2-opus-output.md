# m3-leaf-death-r2 攻方腿（Opus）报告（2026-10-08）

攻击面：正文第三节 K2、K3、K4。只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb（`research/prompts/m3-leaf-death-r2-snapshot/kb-sha256.txt` 四行开工时 `sha256sum -c` 全 OK）。代码引文写冻结树里的全路径:行号；kb 引文写 kb 文件自己的行号或 `文件@小节`。
不重复第一轮攻过的角度（E177 51 段与合并 64 段上的复活与误判死亡）：这一轮的模型是另写的（不是 E177 装置的拷贝），历史里有崩溃、丢根、管理员回退、带数据的对象被借走与吸收、用户动作全部放开；K3、K4 另在冻结树的仓副本上造镜像跑 checker。

## 各格判定一览

「量过」= 在我的模型或仓副本上跑出来的（不是入库装置上的数，主 agent 要在入库装置上重做才能引）；「推的」= 按条款或代码推、没跑。没打中的都是一次抽样。

| 格 | 形态 | 判定 | 依据 |
|---|---|---|---|
| K2 | C1-完整照 K2.0 原样（退役树随现行 + 读侧对每类单元统一按版本谱系滤 + 记录带死亡 txg） | **打中复活**（量过）：G-环 两种几何 9 格 r > 0、峰值 30；随现行的退役容器回退之后现行版本都在被抛弃那一段，链上那一版的槽被再分配，谱系上的退役记录读不到。反例 `trace 395` | K2.3 |
| K2 | C1-完整收严（随现行的树只按 P2 择版本；记录按死亡 txg 滤） | 没打中（量过）：576 段 inode、Z 全 0；崩溃 33 588 点 / 171 448 状态、丢根 11 806、回退 14 941 | K2.2、K2.4 |
| K2 | 退役树回退那次取 R_old、统一按版本谱系滤 | 没打中（量过），而且死亡 txg 可以不要 | K2.2 |
| K2 | 记录不带死亡 txg（随现行、P2 择） | 打中误判死亡（量过，m 峰值 364）；第一轮附条件打中一在放开的历史与 57 个后缀上每一个都在 | K2.2、K2.4 |
| K2 | 崩溃支线（意图之后、退役之前崩；删空那次崩；丢根跳号；号重发） | 没打中（量过）：全靠 P2 挡，去掉 P2 就中 | K2.4 |
| K2 | 清扫门、跨头重生、副本坏一份 | 没量；推的：清扫按码 3 统一判、不分辨候选；跨头要每头一棵退役树、读 origin 的只认死亡 txg ≤ 分叉；单份坏不改择版本 | K2.4 |
| K2 | 每次删空 / 合并 / 换身份的代价 | +2 个角色（退役容器 + 退役树根）× 2 盘 = 4 刀写（推的，照仓副本上量的意图那次墓碑两角色）；今天删空那次 9 / 12 个角色（量过）；频度叶 233 每千次发布 3.4–6.4 条 | K2.5 |
| K3 | 读法 Z：记录被借走 / 被吸收、号回退之后重发、size 不变换单元 | 没打中（量过）：0；对照 RG 36 格全复活 | K3.1 |
| K3 | 区间墓碑 × I-9.19 | **打中**（仓副本，今天的 checker 量过）：活对象上的区间墓碑红 I-9.19；第一轮只说改 I-9.20 | K3.2 |
| K3 | 区间墓碑 × I-9.20 不重号 | **打中**（镜像形状量过，改写推的）：变短 → 长回来 → 再变短到同一末尾，两条只差死亡 txg、都在谱系上，任何按 (号, 出生代[, 区间]) 的不重号都红 | K3.2 |
| K3 | 区间墓碑 × I-9.18、墓碑树长高 | 没打中（读码）；长高只是更早走到 C619 那一步 | K3.2 |
| K4 | N1 下从叶重建分隔 key | **打中**（量过）：叶 233 那一档 8.9% 的版本从叶还原不出；改成「孩子这一版的最小 key」之后 0 | K4.1 |
| K4 | N1 第三句、GPU 判器 | **打中**（30 张镜像量过）：N1 的第三句与 I-9.12 同一谓词，只红它的坏镜像造不出；CPU `walk.rs:2782`、GPU `crash_judge_checker.wgsl:2918` 该删不该改 | K4.2 |
| K4 | N2 × 退役记录 | 没打中撞号（量过）；退役记录多 61%（叶 233） | K4.3 |

## 复跑与文件

模型目录 `research/prompts/m3-leaf-death-r2-opus-model/`（自己写的，不是 E177 装置的拷贝；`copy/` 是放进仓副本的一个 harness 测试目标）。复跑（仓根；每条经内存包装，编译 6G、跑 4G，6 线程；同一时刻只起一条 cargo）：

```
M=research/prompts/m3-leaf-death-r2-opus-model; S=research/scripts; O=<空目录>
export SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O build
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O demos sweep-0
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-1 sweep-2
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O sweep-3 trace determinism
bash $S/run-with-memory-cap.sh 6G bash $S/capped.sh 6 bash $M/rerun.sh $O copybuild
bash $S/run-with-memory-cap.sh 4G bash $S/capped.sh 6 bash $M/rerun.sh $O copyrun summarize
bash $M/compare.sh $O
```

交回前在空目录 `~/.local/share/singlefs/work/m3-leaf-death-r2-opus/rerun4/` 里按上面从头跑通：四批 sweep 58.0、41.0、36.6、12.9 秒（本机当时另有会话占满 16–32 核），copyrun 那一个测试目标 71.73 秒、3 条用例 ok；`compare.sh` 7 份全 `same`、退 0。模型确定：`R2 name=determinism histories=296 two_runs_identical=true`。完整模型输出（`sweep-*.out` 约 1.2 万行、`trace-395.out`）只留在草稿目录，`expected/sorted-output-sha256.txt` 钉着它们排序后的 sha256。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
41bf45efd0f1e391c4c2894d5845b430418a3f3a2baf53f5f52769f45e04226b  ./Cargo.lock
3584b5e4b0fe2cb4920bc2bbb84d6eb64c93b06f5ccccc61aca8a63ebfa37827  ./Cargo.toml
5b55688b756ab538a2547b434c9a4d6b213c8a0ddf6af6bd30d1c78d7b680f17  ./compare.sh
2585631eab196ffe7f0df6d99c1ba626944ab6ee4111ec1f7695a57535206770  ./copy/opus_r2_leaf_death_images.rs
78172037194e8a3aaf6cc3f3c49bdfc5ee2c87c255445594b2b18b03c4ccaf83  ./expected/agg.txt
597168bc4533c6badc307b123e1589c02b35cf162a3d790694e3cc7057baf19d  ./expected/copy.txt
ea87a27abff58a7abb9fd5b1764c8d968a9392f8045b92f49a56091e931d957f  ./expected/counts.txt
03b444b67eb3a29c76186923d316953d34f6d9e09ab005c0bf75f3af7e29a432  ./expected/demos.txt
0fc59c237c8f78dffc7f9ce36fcc2d1adf9659271a9547b7da409887c259b628  ./expected/determinism.txt
a67256ac337a98c72eb6f50f5180d0601978640906aa68538c61dae98e399722  ./expected/sorted-output-sha256.txt
4b7bdc8c3141892051fd21cdb07976cc852674b710f301c042391a14a1301592  ./expected/trace-395-first.txt
c593fe98a588a850abb8f24c73ad602b1b84c75ea2f309c3115ac75076e2df8a  ./rerun.sh
3dc1fe9e416984e26e0b10ee68eeea7b7a50253ab41517aed385f8e1270b2a6e  ./src/demos.rs
55570d4c220d3d1cf5068a09d1e78654f310147c23d2c64b9abb8cf24234d000  ./src/main.rs
a338bd7b94f7c4652b6329c5666be8dd3db53647809b14c6f07bcf12da6e91da  ./src/model.rs
739293a541a095720917d4b77a198a4cdcd638ace8d57952c7d9fc8159bedd98  ./tools/agg.awk
5d2dd92b3ca90a310ab8fac3f4e0f3cf55d3ccdaf6418ee14a4fa94863a2aed5  ./tools/counts.awk
4800089cec53125e8b340aa455cd5270b26fb30d2c2b8721a7ec82c78412001d  ./tools/demos.awk
```

K2.2 那张合起来的表的命令（在 `expected/` 里）：`awk '{k=$4" "$5" "$6; n[k]+=substr($7,11); if($8!="r_peak_max=0") rp[k]++; if($10!="m_peak_max=0") mp[k]++; split($8,a,"="); if(a[2]+0>R[k]) R[k]=a[2]+0; split($10,b,"="); if(b[2]+0>M[k]) M[k]=b[2]+0} END{for(k in n) print k, "histories="n[k], "cells_r_pos="rp[k]+0, "cells_m_pos="mp[k]+0, "r_max="R[k]+0, "m_max="M[k]+0}' agg.txt | sort`。

## K2　候选一的完整形态

### K2.0　我取的最强定义（下称 C1-完整）

被判的形态是第一轮判决第三节那一行「候选一的形态：退役记录带死亡 txg、按谱系滤；载体待定」。我把它补成下面这一份，后面各节都攻这一份；每一处我能选的都选对候选一最有利的那一边，选不了的写明靠的是哪一条还没定的前提。

| 部件 | 定义 | 依据或理由 |
|---|---|---|
| 记录 | 被退役的身份 (出生树 8, 打包记录类型 2, 容器号 8, 容器出生代 8) + 死亡 txg 8 = 34 字节；死亡 txg = 写下它的那次发布的 checkpoint_txg | 第一轮判决第二节「候选一的形态必须带死亡 txg」；D18 已定项 11 登记的类型 3 是四元组、不带写序，加死亡 txg 是改那一行（类型 3 冻结前零代价，同一行原话） |
| 什么时候写 | 一片类型 2 容器的身份在这一版里消失的那次发布：删空摘掉、合并被吸收的右半、N2 借完换身份的旧身份、跨头首次 COW 的旧身份；与身份消失**同一次发布**，不另起发布 | 同一次发布 ⇒ 没有「身份已消失而记录还没写」的已发布版本；崩溃只能让两样一起没发布 |
| 载体 | 一棵新树「退役树」，种类码 13（D8 已定项 9 登记表只追加），day-1 零根注册进树表，头 ID = 同表 inode 树的树 ID；根恒码 2，条目照 inode 树 120 字节形态，孩子是码 3 容器（打包记录类型 3）；容器号从 1 起、下一个号 = 树里最大号 + 1、容器出生代 = 开它那次发布的 txg | 照墓碑树（D18 已定项 24 第 1 条）同形，不进墓碑树：墓碑树条目类型段只许 {0, 1}（I-9.23），类型 3 孩子在 m3-delete-r3 已被删掉（D18 已定项 24 依据第一条） |
| 开放容器 | 有新记录的那次发布才重写，满了才关；一片装 ⌊(32768 − 136) ÷ 34⌋ = 959 条 | 照墓碑树第 3 条 |
| 回退 | 随现行那一版（与墓碑树同，D23 已定项 14）；被回退抛弃的那一段写的记录留在树里，读侧按谱系滤掉 | 第一轮判决第二节：带死亡 txg 按谱系滤、回退取 R_old 两个改法在手造盘上都不中；取「随现行」与墓碑树同一方向，不动 D23 |
| 回收 | 第一版不回收（与墓碑同，D18 已定项 24 第 4 条） | E104 的回收条件「没有根引用该容器任一版 ∧ 过门」要清扫门，`crates/` 没有那道门（K2.4） |
| 读侧（扫盘重建） | ① 已发布谓词 P2（单元写序 + 实例表行）滤掉崩溃抛弃的单元；② 单元诞生代号要落在最新根的谱系父链上（C615 的读侧，所有候选共用）；③ 同身份取剩下的版本里诞生代号最大的；④ 退役树里死亡 txg 落在谱系父链上的记录，点名的那一组整组不收；⑤ inode 记录同号几条时取诞生代号最大的那一版供出的；数据单元按读法 Z 收 | ①③ 是 C113 定案 P2 / P3；② 是 C615 欠的那一半；④ 与 I-9.18 对墓碑的谱系读法同一读法 |

这份定义要动的已定条款与不变量（推的，没写代码）：D18 已定项 11 类型 3 那一行（加死亡 txg、载体从「住自己的容器」写成退役树）；D8 已定项 9 登记表加码 13；I-9.16 ④「进树表的十一种」变十二种（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:4345` 的 `TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES` 与 GPU 判器那两处「十一种」的循环跟着改），新池新建文件的树表 11 → 12 条、树 ID 水位多发一个；I-9.27 加一行「种类 13 的头 ID = inode 树的树 ID」；新立的退役树判据照墓碑树那几条同形（条目身份与子头相符、记录合法、谱系上的退役记录不点名这一版树里的容器）。

### K2.1　装置与取样（我自己的模型，不是 E177 装置的拷贝）

模型 `research/prompts/m3-leaf-death-r2-opus-model/src/`（`model.rs`、`main.rs`、`demos.rs`，不依赖 `crates/`）。三类单元落在槽里：inode 叶容器（类型 2，身份 (容器号, 出生代)）、数据单元（(对象, 对象出生代, 锚点)）、退役容器（类型 3）；每个单元头带诞生代号与写序实例。

- 写路径：建 inode（最右叶追加、末尾分裂，照 `inode_tree.rs` 的分裂纪律）、覆盖写（其中 30% 是「size 不变、每个单元换新内容」）、顺序写、unlink（意图 → 按 1–3 个单元分批 → 最后一批删记录）；最后一批之后照 M2 / M3 均分借：删空就摘掉、两片之和 ≤ 容量就左吸收右、低于一半就从右边均分借；借法三臂：不借、N1（右片身份不变、分隔 key 抬到借后最小 key）、N2（右片换身份 (借后最小 key, 这次 txg)、旧身份退役）。删空、被吸收、N2 换身份都在同一次发布里写退役记录（死亡 txg = 这次 txg）。删空摘掉照 `crates/` 把右边每片重写（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:308`「containers.remove(index.position());」那一支之后的重写清单）。
- 故障：崩溃（在飞发布的 k 个单元写**逐子集全枚举**，每个子集各开一支：恢复、写行、暖机、续做，每次发布之后都扫；主线取全写出那一支往下走；k > 8 的发布不当崩溃点，见限度）；丢根三种（最新一条根读不出、最新根读得出而树读不出（下一个 txg 取环里最大 + 1，跳号）、连丢三条根）；管理员回退（挂着时的一次向前发布，取 R_old 的 inode 树、数据、队列，退役树两臂：随现行 / 取 R_old；回退之后续做队列）。实例表行照 C113 P2：恢复给 [所选根的实例, 新实例) 每个实例写行。下一个 txg = 环里读得出的根的最大 txg + 1（C616 那一句）。
- 几何：G-宽（不再分配）、G-环 取最低号、G-环 随机取（释放 24 次发布之后可再分配；回退目标限在 22 次发布之内，回退与恢复都核目标那一版每个单元还在原槽，`verify_intact`）。
- 读法（每次发布之后扫一遍盘、重建最新那一版，与逻辑真值比 inode 记录 (号, 出生代, nlink, size) 与数据单元内容）：P2（单元写序实例 + 行）→ 版本谱系滤（诞生代号在最新根的谱系父链上）→ 同身份取 (诞生代号, 实例) 最大 → 退役 → 同号取诞生代号最大的叶版本 → 数据读法 Z / RG（只看记录在不在）/ S（全收）。退役读法：`C1t` = 退役容器只按 P2 择版本、记录按死亡 txg 在不在谱系上滤；`C1t_uniform` = 退役容器与叶同一套滤法（P2 + 版本谱系）；`C1n`、`C1n_uniform` = 同上两种择法但记录不按死亡 txg 滤；`S` = 不读退役；消融 `C1t_noP2`、`C1t_noLin`。
- 用户动作全部放开：每段 1500（叶容量 233）/ 600（叶容量 6）个动作，建 / 覆盖写 / 顺序写 / unlink 按千分比掷，unlink 70% 落在每 150 步换一片的「盯着的」叶上；回退 2.5%、崩溃 3%、丢根 1% 另掷。
- 取样：2 种形状（叶容量 233、退役容器 959 条、起始 700 个空 inode；叶容量 6、退役容器 3 条、起始 40 个）× 3 几何 × 3 借法 × 2 退役树回退臂 × 16 个种子 = 576 段。估时：先各跑 6 段，叶容量 233 那 6 段 5.1 秒、叶容量 6 那 6 段 0.6 秒（6 线程），推全量约 2.5 分钟，不缩；正式分四批（96 / 96 / 96 / 288 段），各批 55.0、39.4、32.1、11.0 秒（草稿目录 `run2/sweep-*.err` 的 `elapsed_seconds`）；批不设限时。两遍逐行比：`R2 name=determinism histories=296 two_runs_identical=true`（`expected/determinism.txt`；叶容量 6 那 288 段全取、叶容量 233 每 37 段取一段）。

走到的形状（`expected/counts.txt` 各组相加，含崩溃各子集那几支里的计数）：崩溃点 33 588 个、枚举崩溃状态 171 448 个、丢根 4 045 + 3 995 + 3 766 次、回退 14 941 次（其中跨过一次身份消失或借的 3 012 次）、删空 16 013、吸收 12 725、借 5 165、退役记录 31 404 条、记录带数据被挪到别的叶 24 481 次、新建对象的 (号, 出生代) 与被崩溃抛弃的对象相同（C143 那一形）76 次。

### K2.2　主取样结果：C1-完整（随现行 + 退役容器只按 P2 择 + 死亡 txg 滤）没打中；两种拼法打中

`expected/agg.txt` 按（退役树回退臂, 读法, 树）合起来（`agg.txt` 每行 16 段；按退役树回退臂、读法、树合起来，每组 18 行 = 288 段；`cells_*_pos` 是这 18 行里 r 或 m 峰值 > 0 的行数）：

```
carried C1t inode histories=288 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0
carried C1t data_Z histories=288 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0
carried C1t_uniform inode histories=288 cells_r_pos=9 cells_m_pos=0 r_max=30 m_max=0
carried C1n inode histories=288 cells_r_pos=0 cells_m_pos=18 r_max=0 m_max=364
r_old C1n_uniform inode histories=288 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0
r_old C1t_uniform inode histories=288 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0
r_old C1n inode histories=288 cells_r_pos=0 cells_m_pos=18 r_max=0 m_max=261
carried C1t_noP2 inode histories=288 cells_r_pos=18 cells_m_pos=18 r_max=701 m_max=222
carried C1t_noLin inode histories=288 cells_r_pos=18 cells_m_pos=18 r_max=716 m_max=60
carried S inode histories=288 cells_r_pos=18 cells_m_pos=0 r_max=483 m_max=0
```

（命令：在 `expected/` 里 `awk '{k=$4" "$5" "$6; …}' agg.txt | sort`，全文见复跑一节；各格原行例：`expected/agg.txt` 第 76 行 `cap233 ring_lowest no_borrow carried C1t inode histories=16 r_peak_max=0 …`、第 82 行 `… carried C1t_uniform inode histories=16 r_peak_max=30 histories_r_pos=2 …`、第 456 行 `cap6 ring_random n2 carried C1t_uniform inode histories=16 r_peak_max=6 histories_r_pos=13 …`。）

- 消融都红（装置看得见这几种错）：去掉 P2 复活 701 / 误判死亡 222，去掉版本谱系滤复活 716，不读退役（S）复活 483，记录不按死亡 txg 滤（C1n）误判死亡 364——第一轮附条件打中一（第一轮攻方 `research/prompts/m3-leaf-death-r1-opus-model/expected/rollback.txt` 的 missed = 2）在放开的历史上复现，量级大得多。
- **C1-完整按 K2.0 写的那样（读侧 ② 对每一类单元都按版本谱系滤）打中复活**：`carried C1t_uniform` 9 格 r > 0，全在 G-环 两种几何上（G-宽 0），见 K2.3。
- 退役树「随现行」时只要退役容器按 P2 择版本（不按版本谱系滤）、记录按死亡 txg 滤（`carried C1t`），576 段里 inode、数据（Z）r = m = 0；退役树「取 R_old」时按版本谱系滤就够、死亡 txg 可以不要（`r_old C1n_uniform` 0）。

### K2.3　打中：随现行的退役树被按版本谱系滤，谱系上的退役记录读不到，被删空的叶复活

可复跑的反例：`m3-leaf-death-r2-opus-model trace 395`（叶容量 6、G-环 取最低号、不借、退役树随现行、种子 12；`expected/trace-395-first.txt` 是第一次复活那一刻的盘面）。历史里由用户决定的步都是随机放开的那一段的原样，固定的只有前缀（新池新建文件、建 40 个空 inode）与故障（回退、崩溃）。逐步：

1. txg 74：unlink 删掉叶 (51, 出生代 47) 的最后一条，那片删空摘掉，同一次发布开退役容器 3（出生代 74），记 `51/47@74`。
2. txg 79、86：又有两片身份消失，追加进容器 3，各写一版（诞生代号 79、86）；79 那次把 74 那一版的槽放出去。
3. txg 98：管理员回退到 txg 76（谱系记录 (98, 76)）。退役树随现行：新根引用容器 3 诞生代号 86 那一版。
4. txg 103 崩溃、恢复（行 {1: 58, 2: 102}）；到 txg 105 时 74 那一版的槽已被环再分配（盘上容器 3 只剩 79、86 两版，见下面诊断行）。
5. txg 105 扫描：

```
FIRST C1t_uniform inode r>0 at seq=107 txg=105 inst=3
  current version txg=105 inst=3 lineage=[(14, 2), (17, 2), (98, 76)] rows={1: 58, 2: 102}
  leaf Ident { num: 51, birth: 47 } version birth=55 inst=1 slot=43 on_chain=true published=true live_in_truth=false records_not_in_truth=[51, 53, 54, 55]
  retirement container Ident { num: 3, birth: 74 } version birth=86 inst=2 slot=19 on_chain=false published=true referenced_by_the_current_root=true records=["51/47@74", "56/75@79(off)", "58/80@86(off)"]
  retirement container Ident { num: 3, birth: 74 } version birth=79 inst=2 slot=25 on_chain=false published=true referenced_by_the_current_root=false records=["51/47@74", "56/75@79(off)"]
```

（`expected/trace-395-first.txt` 第 1、2、3、9、10 行原样。）记录 `51/47@74` 的死亡 txg 74 在谱系父链 105 → … → 98 → 76 → 75 → 74 上，它就该生效；但装着它的那片容器在盘上剩下的两版诞生代号 79、86 都落在被回退抛弃的那一段 (76, 98) 里，按版本谱系滤就一版都不剩，链上那一版（74）的槽已经被再分配。读侧于是没有任何一条记录点名 (51, 47)，那片叶诞生代号 55 的旧版（51、53、54、55 四条都已删）整片收回来：复活 4 条。`C1t`（容器按 P2 择版本）取 86 那一版、按死亡 txg 滤掉两条 `(off)`、留下 `51/47@74`，同一刻 r = 0。

为什么只在 G-环：随现行的树，回退之后现行那一版是在被抛弃那一段里写的；链上那一版在回退之前就被换下、放回分配器，回退不把它收回来（新根不引用它），环转过一圈就可以被盖。G-宽 不再分配，链上那一版一直读得到，所以第一轮手造盘与我 G-宽 的 288 段都看不见这一形。

四句：
- 分不分辨臂：分辨「退役树怎么择版本」这一维，不分辨候选一对二、三、四。凡是住在随现行那几棵树里的死亡记录都同形：今天的墓碑树、谱系树随现行（`.claude/kb/decisions/23-journal的角色与格式.md:378`），候选三放进墓碑树也一样（推的，墓碑树不在模型里）；候选二的零条目版本住 inode 树（取 R_old）不受这一形。
- 判别子看不看得到：看得到。被抛弃那一段写的版本过 P2（同一实例、诞生代号不高于行里的 T），也被现行根引用；区分「该取哪一版」的东西（这棵树随现行）在规则层面，不在单元上，读侧照树的回退方向分两种择法就分得开（`C1t` 量过 0）。
- 满足的是判据哪一句：正文第五节第二条「攻方坐实候选一的完整形态一形复活」。被打中的是我在 K2.0 写的那一份的读侧 ②（对每一类单元都按版本谱系滤）；第一轮判决「带死亡 txg、按谱系滤」只说了记录怎么滤，没说装记录的容器怎么择版本，K2.0 补的那一句选错了。
- 改法在打中的格上还中不中：甲「随现行的树（退役树、墓碑树、谱系树）只按 P2 择版本，不按版本谱系滤；记录按死亡 txg 在不在谱系上滤」——不中（`carried C1t` 288 段 0）；乙「退役树回退那次取 R_old（与 inode 树同），按版本谱系滤」——不中（`r_old C1t_uniform`、`r_old C1n_uniform` 0），而且乙下死亡 txg 可以不要；丙「只加死亡 txg、择版本照统一滤法」——照中（`carried C1t_uniform` r 30）。甲、乙都只在我的模型上量过、被攻过零轮。

收严后的 C1-完整（我下面各节攻的是这一份）：K2.0 表里读侧 ② 改成「诞生代号在谱系父链上，只对回退那次取 R_old 的树（inode 树、extent 树、`deleted_inodes`）判；随现行的树（退役树、墓碑树、谱系树）只按 P2 择版本」，其余不动。这一句也是 C615 读侧要写进去的：C615 的欠账行只说「按择新键选中被回退掉的那一版」（`.claude/kb/checks-owed.md:475`），改法若写成对每一类单元统一按谱系滤，墓碑树会照这一形丢掉链上的墓碑（推的）。

### K2.4　其余几形（没打中，写清扫了什么）

- **回退跨过合并 / 删空 / 借，之后用户动作放开扫**（`demos.rs`，`expected/demos.txt`）：叶容量 6、G-宽，前缀 1 号带数据 + 17 个空 inode、六个文件写数据，从第二片删起，删到那一种结构变化第一次发生，回退到它前一版；之后的用户动作取菜单 7 样（建 3 个、删最小号、删最大号、删第二片最小号、最小号覆盖写 3 个单元、最大号顺序写 2 个、再回退到最老的候选）的全部 0–2 步序列，57 个后缀。四种跨法（删空、吸收、N1 借、N2 借）× 57 个后缀，`C1t`、`C1t_uniform` inode 与 Z 全 0；`C1n` 在删空、吸收、N2 借三种上 57 个后缀全是 m > 0（N1 借不写退役记录，0），即第一轮那一形在每一个后缀上都在，不是某个后缀碰巧。
- **崩溃支线**：意图之后、退役写下之前崩——退役与身份消失在同一次发布，崩在它之前两样都没发布、队列还在、挂载续做重做那一批（模型里每个崩溃点都这样走，33 588 个崩溃点、171 448 个崩溃状态，`C1t` 0）。手造的那一形：删空那次的根读不出，续做重做删空时再崩（k 个单元写逐子集），再恢复续做（`crash_in_the_drop_publish`，57 个后缀，`C1t` 0，`C1t_noP2` 57 个后缀 m = 1：被抛弃的退役记录死亡 txg 落在重发的 txg 上，不按 P2 滤就生效）。丢根读得出而树读不出（txg 跳号，父链经过被抛弃的那个 txg）、连丢三条根：同样只靠 P2 挡（`C1t` 0、`C1t_noP2` 非 0）。
- **号回退之后重发**（C143 那一形）：76 次新对象的 (号, 出生代) 与被抛弃的对象相同，`C1t` inode、Z 都 0（P2 挡住被抛弃那一版的叶与数据单元）。
- **清扫门与实例表抛弃**：`crates/` 的清扫认单元只看码 1 / 2 / 3 与头校验和，不看打包记录类型（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:195`「UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),」），类型 3 容器照别的码 3 一样被判、被抹，删行之前没有类型 3 专属的漏网形（推的，读码；模型里不删行）。删行之后实例 i 的单元一律当已发布，被抹之前漏掉的孤儿退役记录会以重发的 txg 生效——那是清扫门本身对所有单元类的承诺，不分辨候选（推的）。
- **跨头重生**：`crates/` 没有克隆头，没量。推的：退役树必须每头一棵（头 ID = 本头 inode 树），克隆头读 origin 的退役记录只认死亡 txg ≤ 分叉那次的（txg 是全池一条，`死亡 txg > 分叉 txg` 就分得开）；I-9.18 那条「其余 txg 的父是 txg − 1」的父链对两个头给同一条链，多头时不能原样拿来滤退役记录（C616 / C113 P6 那一格，所有候选共用）。
- **副本坏一份（w = 2）**：单份坏、另一份读得出时选版本不变，模型没建副本（限度）。两份都坏才退回更旧的退役容器版本，那与任一叶容器两份都坏退回旧版是同一个暴露（不分辨候选，推的）。

### K2.5　每一次删空、合并、换身份多写几个单元、几刀写

量过的（仓副本，`copy/opus_r2_leaf_death_images.rs` 的 `opus_r2_roles_of_the_deletion_publishes`，`expected/copy.txt` 第 37–40 行，原行里分配记录树根下的节点我缩写成 `Alloc(层,盘,序号)`）：今天删空摘掉那一次发布重写的角色——

```
drop_the_rightmost_leaf_234 publish=last_batch count=9 rewritten=[InodeRoot, Alloc(l0,d0,i61), Alloc(l0,d1,i61), Alloc(l1,d0,i0), Alloc(l1,d1,i0), AllocationTree, AccountingTree, MappingTree, TreeTable]
drop_the_middle_leaf_234_to_466 publish=last_batch count=12 rewritten=[InodeLeafContainer(InodeLeafContainerIndexInTree(1)), InodeRoot, Alloc(l0,d0,i61), Alloc(l0,d0,i62), Alloc(l0,d1,i61), Alloc(l0,d1,i62), Alloc(l1,d0,i0), Alloc(l1,d1,i0), AllocationTree, AccountingTree, MappingTree, TreeTable]
drop_the_rightmost_leaf_234 publish=intent count=13 rewritten=[… DeletedInodesRoot, TombstoneLeafContainer(TombstoneContainerIndexInTree(0)), TombstoneRoot, MappingTree, TreeTable]
```

意图那次往墓碑树开放容器追加一条，多的是 `TombstoneLeafContainer`、`TombstoneRoot` 两个角色。退役树照墓碑树同形（K2.0），推的代价：每次有身份消失的发布多 2 个角色（退役开放容器一版 + 退役树根），删空最右片 9 → 11、删空中间片 12 → 14；`crates/` 每个角色每块盘一次单元写（`deletion_scenario_writes.rs` 文件头那一句「单元写 = 重写角色数 × 2 盘」），两块盘多 4 刀写，同一段、不加屏障；新落点要是落进别的分配记录叶，再多 1–2 个 `Alloc` 角色（推的，没造）。退役容器装满（959 条）那次换一片新容器，代价相同；根满（135 片，约 12.9 万条）那一次长高，多 2 个码 2 节点（照 D8 已定项 6 长高那一句，推的）。

频度（模型，`expected/counts.txt`；发布数含崩溃各子集那几支）：

| 形状 / 借法 | 发布 | 退役记录 | 每千次发布 | 退役容器写 / 叶容器写 |
|---|---|---|---|---|
| 叶 233、不借 | 368 856 | 1 469 | 4.0 | 1 442 / 275 008 |
| 叶 233、N1 | 372 011 | 1 263 | 3.4 | 1 241 / 274 790 |
| 叶 233、N2 | 368 790 | 2 369 | 6.4 | 2 329 / 274 951 |
| 叶 6、不借 | 160 588 | 8 229 | 51.2 | 8 025 / 108 271 |
| 叶 6、N2 | 159 038 | 9 907 | 62.3 | 9 659 / 112 301 |

（`expected/counts.txt` 第 16、19、34、35、53、56、71、72、90、93、108、109、164、167、182、183、201、204、219、220 行。）叶 233 那一档每千次发布 3–7 条，与第一轮 E177 / 合并历史的 63 / 20 003、122–123 / 约 11 万同一量级（第一轮判决第二节）；N2 比不借多 61%（叶 233）。

## K3　读法 Z 与区间墓碑

### K3.1　Z 在三形上：没打中（量过）

Z 的定义照第一轮攻方 K2 第 1 节：重建出的 inode 树里每个对象取一条记录（同号几条取诞生代号最大的那一版供出的），数据单元只在「对象出生代相等且锚点 < size 折成的单元数」时收。我的模型里每次扫描都按 Z 判（`C1t data_Z`），对照 `C1t data_RG`（只看记录在不在、不看 size）与 `C1t data_S`（全收）。

| 形 | 走到的次数（`expected/counts.txt` 合计） | Z | 对照 |
|---|---|---|---|
| 记录被借走或被左吸收的对象，它的数据单元（K3 第一形） | 记录带数据被挪到别的叶 24 481 次（吸收 12 725、借 5 165） | 576 段 r = m = 0 | — |
| 号回退之后重发（E175 / C143 那一族） | (号, 出生代) 与被抛弃对象相同 76 次；号在崩溃恢复之后重发是常态（每次崩溃 / 丢根） | 0 | `C1t_noP2 data_Z` r 30 / m 97 |
| 覆盖写之后 size 不变而单元换了 | 覆盖写里 30% 走这一支 | 0 | — |
| 文件变短（覆盖写变少、unlink 各批截短） | 每次覆盖写都可能 | 0 | `C1t data_RG` 36 格全 r > 0（峰值 22） |

（`expected/agg.txt` 里 `C1t data_Z` 36 行全是 `r_peak_max=0 … m_peak_max=0`；`C1t data_RG` 36 行全 r > 0，例第 277 行 `cap233 wide no_borrow carried C1t data_RG histories=16 r_peak_max=21 histories_r_pos=16 …`。）Z 在挪动上不出错的机制：同号只会在一片叶的现行版本里出现，挪走的那一片同一次发布重写、现行版本里已经没有它；被吸收或换身份的旧片整组退役。Z 管得住的前提是 inode 记录读对了（C1t 那一套），记录读错（`S`、`C1n`）时 Z 跟着错（`S data_Z` r 210）。

Z 自己的前提（推的，模型里成立、`crates/` 今天成立）：文件没有洞——`crates/` 的读者就判 extent 记录条数等于 inode size 按净荷容量除出来的单元数，对不上报「这个文件的 extent 记录条数与 inode size 按净荷容量除出来的单元数不符」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/recovery.rs:4901`），有了 punch / 稀疏写 Z 就不成立（第一轮攻方已点过）。

### K3.2　区间墓碑：打中——不只 I-9.20，I-9.19 也要改，而且「不重号」改成什么都会在合法历史上红

仓副本（冻结树 `crates/` 原样拷进草稿目录，只加一个测试目标），`opus_r2_range_tombstones_on_live_objects`：池里 1 号活着（新池新建文件，对象出生代 3）、3 号删完、根环转过一圈，墓碑树第一片容器换成手写的几条，checker 原样判（`expected/copy.txt` 第 17、33、34、35 行）：

```
OPUSR2 image=clean tombstones=[(3, 5, 0, 18446744073709551615, 6)] violated=[]
OPUSR2 image=one_range_tombstone_on_live_inode_one tombstones=[(3, 5, 0, 18446744073709551615, 6), (1, 3, 1, 18446744073709551615, 6)] violated=["I-9.19", "I-9.20"]
OPUSR2 image=one_whole_object_tombstone_on_live_inode_one tombstones=[(3, 5, 0, 18446744073709551615, 6), (1, 3, 0, 18446744073709551615, 6)] violated=["I-9.19"]
OPUSR2 image=two_equal_range_tombstones_on_live_inode_one tombstones=[(3, 5, 0, 18446744073709551615, 6), (1, 3, 1, 18446744073709551615, 5), (1, 3, 1, 18446744073709551615, 6)] violated=["I-9.19", "I-9.20"]
```

- 打中一（I-9.19）：区间墓碑写给的是一个**活着**的对象（变短之后 nlink 仍是 1），而 I-9.19 判「谱系上的墓碑…指认的 (号, 对象出生代)，inode 树里有号与对象出生代都相同的记录时，那条记录的 nlink = 0」（`.claude/kb/invariants.md:307`），checker 那一遍只按谱系滤、不看区间（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:10780`「.filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))」）。第一轮判决第二节 K2 那一行只写了「要改 I-9.20」；照那样改完 I-9.20，每一次变短的那一版照样红 I-9.19。I-9.19 要收成「谱系上的**整对象**墓碑」，GPU 判器那一处（I-9.19 一行写着第 23 批补）跟着改。
- 打中二（I-9.20 不重号）：变短到末尾 e → 顺序写长回来 → 再变短到同一个 e，是 `crates/` 今天就有的写路径（覆盖写把多单元文件写成一个单元、顺序写追加）。两次各写一条 (号, 出生代, [e, 2⁶⁴ − 1])，只差死亡 txg，都在谱系上（第四行那张镜像就是这个形状）。I-9.20 今天按 (号, 对象出生代) 判不重复（`.claude/kb/invariants.md:308`）；改成按 (号, 出生代, 区间) 也照红。能守住合法历史的写法只有「不重号只判整对象墓碑」或把死亡 txg 也算进键（那等于不判）——两者都把 I-9.20 抓「同一次 unlink 写了两条」的判别力只留给整对象那一半（推的，没给 checker 打补丁）。
- I-9.18 不受影响：它要的是整对象墓碑，代码判 `covers_the_whole_object()`（`walk.rs:10733`），区间墓碑既不满足它、也不遮掉缺整对象墓碑的那一形（读码）。
- 与谱系读法：变短在 T1 写区间墓碑、回退到 T1 之前，那条落在被抛弃那一段、不在谱系上，读侧照 I-9.18 的父链滤掉它，回退回来的长文件不被截（推的，与整对象墓碑同一读法）。
- 墓碑容器满之后长高：区间墓碑只追加、不改 I-9.21 / I-9.22 的形状；它让根满（135 片 × 815 = 110 025 条）不只靠 unlink 数，变短多的负载也会走到 K-R3-3 那一步，而那次发布今天没有崩溃流（C619，推的）。
- 区间墓碑在 Z 之下买到什么：K3.1 里 C1t + Z 在文件变短上已经 0，扫盘重建不需要区间墓碑；它的代价是每次变短多 2 个角色（第一轮正推量）加 I-9.19、I-9.20 两条改写。

四句（对打中一、二）：分辨（只打区间墓碑这一臂，Z 不写墓碑不碰）；判别子看得到（区间、nlink、死亡 txg 都在镜像上，被打中的是判据写法）；满足的是任务「区间墓碑与 I-9.20、I-9.18 谱系读法的交互」那一格；改法「只改 I-9.20」在打中一上照中（量过，第三张镜像只红 I-9.19），「I-9.19、I-9.20 都收成只管整对象」两格都不中（推的，没给 checker 打补丁）。

## K4　N1 与 N2

### K4.1　N1：内部节点丢了从叶重建，分隔 key 还原不了；但可以换一个还原得了的定义

`crates/` 今天每次发布都从叶容器的身份现算分隔 key（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:83`「self.identity.container」，写根时 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/transaction.rs:7386`「container.contents.separator_key(),」），所以它是叶的纯函数、「派生态，丢了从叶重建」（`.claude/kb/decisions/08-核心索引结构.md:160` 那一行）成立。N1 照第一轮的写法「分隔 key 随借抬到右片借完的最小 key」之后，分隔 key 变成历史的函数：借过之后右片最小那条再被删，分隔 key 留在借时的值。

模型里 N1 臂每一版都比三样：存着的分隔 key、按容器号重建、按孩子现在的最小 key 重建（`expected/counts.txt` 第 22–37、133–148 行）：

| 叶 233、N1（371 401 版） | 数 | 叶 6、N1（157 083 版） |
|---|---|---|
| 存着的 ≠ 容器号的版本 | 169 090 | 20 448 |
| 存着的 ≠ 现在最小 key 的版本 | 298 848 | 116 811 |
| 两种重建规则都还原不出的版本 | 32 944 | 2 203 |
| 按容器号重建：I-9.12 红的版本 / 走错片的记录峰值 | 169 090 / 201 | 20 322 / 5 |
| 按现在最小 key 重建：I-9.12 红 / 走错片 | 0 / 0 | 0 / 0 |
| 存着的：I-9.12 红 / 走错片 | 0 / 0 | 0 / 0 |

- 打中（判别子观测不到）：叶 233 那一档 8.9% 的版本（32 944 / 371 401），叶里没有任何东西说出借时那个最小 key 是多少，「从叶重建」给不出原来的分隔 key。按容器号重建（今天代码的写法）在 169 090 / 20 322 版上红 I-9.12、按 D8 的查找规则走错片（峰值 201 / 5 条）。
- 改法：把 N1 的分隔 key 定义成「孩子**这一版**的最小 key」，每次写父节点时现算。孩子一改父节点本来就重写（COW 叶 + 全部祖先），不多写；它是叶的纯函数，重建出来与原样逐字节相同，模型里 I-9.12、路由都 0（上表最后两行，只在我的模型上量过、被攻过零轮）。代价是 D8「分隔 key = 该孩子创建时的最小 key」（`.claude/kb/decisions/08-核心索引结构.md:164`）与 I-9.12 引的那半句都要改，最左片的分隔 key 也随它的最小 key 走（不再恒为 1）；根的 key 区间取首末条目的分隔 key（`transaction.rs` 第 7363–7370 行那一段），跟着变。

### K4.2　N1 改第三句之后，那一句就是 I-9.12 的一个小句；GPU 判器那一处要跟、跟完也没有只红它的坏镜像

第一轮判决给 N1 的第三句是「左最大 key < 右分隔 key」。I-9.12 已经判「第 i 条的分隔 key … > 第 i−1 个孩子的 key 区间上界」（`.claude/kb/invariants.md:300`），CPU 那一处是 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3263`「.judge("I-9.12", *separator > previous_largest, || {」，GPU 那一处是 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2907`「w_judge(INVARIANT_I_9_12, lt64(previous_largest, separator));」——同一个谓词。层级 1 的根上两条逐字相同；层级 > 1 时根的 key 区间取首末条目的分隔 key（I-1.1），孩子子树的下界就是它第一条的分隔 key，逐层推下去仍是 I-9.12 的推论（推的）。第一轮攻方量的另一写法「左最大 key < 右最小 key」同样是 I-9.12 两个小句的推论（分隔 key ≤ 孩子下界 且 > 左孩子上界）。

仓副本上量的一族镜像（`opus_r2_separator_family`，三片叶 [1..233]、[234..466]、[467]；左片删到 100 条、再均分借 17 / 66 条，第二条的分隔 key 取 7 个值；`expected/copy.txt` 里带 `n1b_` 的 30 行）：

```
OPUSR2 image=borrowed_moved66_separator299 leaves=[no=1 sep=1 1..299][no=234 sep=299 300..466][no=467 sep=467 467..467] third_clause_today=false n1a_left_max_lt_right_min=true n1b_left_max_lt_right_separator=false violated=["I-9.4", "I-9.12"]
OPUSR2 image=borrowed_moved66_separator300 leaves=[no=1 sep=1 1..299][no=234 sep=300 300..466][no=467 sep=467 467..467] third_clause_today=false n1a_left_max_lt_right_min=true n1b_left_max_lt_right_separator=true violated=["I-9.4"]
```

（第 11、13 行原样。）N1b 为假的 12 行（几个取值重合，有重复的镜像），checker 的 I-9.12 全红（`grep n1b_left_max_lt_right_separator=false expected/copy.txt | grep -vc I-9.12` 输出 0）；第二行是 N1 下合法的借（分隔 key 抬到 300），今天只红 I-9.4 第三句（容器号 234 ≤ 左最大 299）。

- 打中（改法碰不到它要的判别力）：N1 下第三句在每一张镜像上与 I-9.12 同真同假，「只红 I-9.4」的坏镜像造不出来。今天钉在这一句上的坏镜像用例（`crates/mutations.tsv` 第 952 行那条变异指着的 `a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant`）在 N1 下造不出「只红 I-9.4」的新形状，第 952、953 两条变异的锚点随这一句一起没了，第一轮判决里正推那句「`mutations.tsv` 第 952、953 行要重指」没处可指——第三句该删，不是改写；第一轮已经认的「I-9.4 不再抓分裂时号给小了」（第一轮攻方 `research/prompts/m3-leaf-death-r1-opus-model/expected/k3.txt` 第 27、28 行那一形；第一轮判决第二节写的「第四、五行」在入库那份里是别的两行，主 agent 判决引它时要改行号）从镜像判据里整个出去，只剩 C116 那条写路径纪律（单测、模型对拍、崩溃点重放）。
- GPU 判器要跟：`crash_judge_checker.wgsl:2918`「w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));」与 CPU 的 `walk.rs:2782`「.judge("I-9.4", left.largest_inode < right.container, || {」同一句，N1 下合法的借照样在两边都红（上面第二行）；两边都删掉这一句就够，不用改成比分隔 key（那一比 2907 行已经在做）。核心读者那一道判记录号不小于分隔 key 与容器号、否则报 I-9.4 的检查（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/recovery.rs:4879`「if record.inode < separator_key || record.inode < identity.container {」）在 K4.1 的「现在最小 key」定义下照样成立，不用动（推的）。

四句：分辨（只打 N1；N2 不改判据）；判别子看得到（容器号、分隔 key、记录都在镜像上）；满足的是任务 K4「GPU 判器那一处要不要跟」与第一轮正推「重指坏镜像」那一句；改法「删第三句、GPU 同删」在这一族上不再误红合法的借（推的，没给 checker 打补丁）。

### K4.3　N2 与退役记录：没打中撞号；每次借多一条退役、多 2 个角色

- 撞号：N2 换的新身份是 (借后最小 key, 这次 txg)。同一次发布里能诞生的类型 2 身份只有它自己（一次删除类发布只删一条、至多一次结构变化，建 inode 是另一次发布），崩溃重发同一个 txg 的那一版被 P2 挡掉，管理员回退不重发 txg。模型里 N2 臂 192 段（含回退跨过 N2 借、57 个后缀放开扫，`expected/demos.txt` 的 `rollback_across_n2_borrow`），`C1t` inode r = m = 0。
- 一次借写几条：我的写路径一次删除至多一次结构变化，所以一次一条。若合并写成「合并到再也并不了为止、再借」（E176 的 M2 / M3 那一形），同一次发布可以「吸收 + 借」各退役一个身份，两条同一个死亡 txg，读法不变（推的）。
- 代价：退役记录叶 233 那一档 2 369 对不借 1 469（+61%）、叶 6 那一档 9 907 对 8 229（`expected/counts.txt` 第 56、93、167、204 行）；每多一条就是 K2.5 那 2 个角色。另：N2 下 `C1n`（不带死亡 txg）误判死亡峰值 261 / 364，比不借的 136 / 233 大（`expected/agg.txt` 第 138 行等）——N2 让「退役树随现行而记录不带 txg」那一形更重，C1-完整带 txg 之后归 0。
- 跨头：N2 换容器号，与「跨头首次 COW 保留容器号、只换出生树与出生代」那条规则（D8 已定项 6）是两套换身份；克隆头共享叶时 origin 借完换号，克隆那一侧仍引用旧身份，origin 的退役记录必须只对 origin 一头可见（K2.4 跨头那一条），`crates/` 没有克隆、没量。

## 改法表（我自己提的改法：只在我的模型或仓副本上量过、被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| 甲：随现行的树（退役树、墓碑树、谱系树）只按 P2 择版本，不按版本谱系滤；记录按死亡 txg 在不在谱系上滤 | K2.3 复活 | 不中：`carried C1t` 288 段 inode、Z 全 0（`expected/agg.txt`） | 量过（模型，退役树；墓碑树推的） |
| 乙：退役树在管理员回退那次取 R_old（与 inode 树同），按版本谱系滤 | K2.3 复活 | 不中：`r_old C1t_uniform`、`r_old C1n_uniform` 288 段全 0；乙下死亡 txg 可以不要；与 D23 已定项 14 给墓碑树的方向相反 | 量过（模型） |
| 丙：只加死亡 txg、择版本照统一滤法 | K2.3 复活 | 照中：`carried C1t_uniform` r 30 | 量过（模型） |
| 读侧 ② 只对回退取 R_old 的树判版本谱系（K2.3 末段的收严） | 同甲，写进 C615 读侧 | 同甲 | 量过（模型）；C615 本身没写 |
| I-9.19 收成「谱系上的整对象墓碑不指活对象」 | K3.2 打中一 | 推的不中：区间墓碑那一张只剩它红 I-9.19（`expected/copy.txt` 第 33 行）；没给 checker 打补丁 | 推的 |
| I-9.20 的不重号只判整对象墓碑 | K3.2 打中二 | 推的不中（第 35 行那张只因「不是整对象」那一句红） | 推的 |
| 只改 I-9.20、不改 I-9.19（第一轮判决的写法） | 区间墓碑落地 | 照中：整对象那一张只红 I-9.19（第 34 行），区间那一张照红 I-9.19 | 量过（仓副本，今天的 checker） |
| N1 的分隔 key 定义成「孩子这一版的最小 key」，每次写父节点现算 | K4.1 还原不了 | 不中：模型 N1 臂 I-9.12 / 路由 0、与从叶重建逐个相同 | 量过（模型） |
| N1 照第一轮写法「借时抬到右片最小 key」 | — | 照中：8.9% 的版本从叶还原不出（叶 233） | 量过（模型） |
| N1 下删 I-9.4 第三句（CPU `walk.rs:2782`、GPU `crash_judge_checker.wgsl:2918`），不改写 | K4.2 | 推的不中：这一族 30 张镜像上它与 I-9.12 同真同假 | 一族镜像量过（仓副本），删句推的 |

## 没打中的形状

- C1-完整收严之后（`carried C1t`）：576 段里退役树随现行那 288 段、取 R_old 那 288 段（`r_old C1t`）的 inode 与 Z 全 0。形状：2 形状 × 3 几何 × 3 借法 × 16 种子；每段 1500 / 600 个放开的用户动作；崩溃 33 588 个崩溃点、171 448 个崩溃状态（逐子集全枚举，k ≤ 8）、丢根三种 11 806 次、回退 14 941 次（跨过结构变化 3 012 次）。手造的回退 / 崩溃 / 丢根 7 种 × 57 个后缀（`expected/demos.txt`）`C1t`、`C1t_uniform` 全 0。
- Z（K3.1）：挪动 24 481 次、C143 那一形 76 次、size 不变换单元的覆盖写占覆盖写 30%：Z 0。
- N2 撞号（K4.3）：N2 臂 192 段与回退跨过 N2 借 57 个后缀，`C1t` 0。
- 这些「没打中」都是一次抽样（一份提示、一条腿），照规则不算数，要同一份提示再派一条同立场腿。

## 这条腿自己的限度

- 模型不是 `crates/`：没有 extent 树、分配记录树、记账、映射、墓碑树（墓碑那一半的 K2.3 推论是推的）；单元一个槽一份、没有 w = 2 副本；读侧是手写的 P2 / 版本谱系滤 / 择新，不是 `crates/` 的扫描重建（`crates/` 没有）。
- 崩溃只在「在飞发布的单元写」那一段枚举子集（这一段在 `crates/` 里是屏障之前同一段），journal 记录、根槽两段没建（模型没有 journal；未写根就等于没发布）；k > 8 的发布不当崩溃点（200 次跳过，`crash_points_skipped_more_writes_than_k_max`），那是取样的缩法，不是崩溃状态的缩法。`crates/` 的层 0 枚举没用（`crates/` 没有退役记录的写者，没东西可录）。
- 环回收按「释放之后 24 次发布」近似 `crates/` 的「环里最旧有效根」门槛；回退目标限在 22 次发布之内。K2.3 的复活依赖「回退之后、链上那一版的槽被再分配」，在 `crates/` 的真分配器上多久发生没量。
- 叶容量 6 是压小的旋钮（为了合并、借多）；叶容量 233 那一档 1500 个动作里结构变化少（不借 1 469 条退役 / 约 37 万次发布）。
- 管理员回退只建了「挂着时的一次向前发布、取 R_old 的 inode 树与数据与队列」，回退下界 F、实例表、清扫发布都没建；清扫门那一条是读码推的。
- 仓副本上的镜像是在 `crates/` 写出的一版上手改字节（墓碑、分隔 key、叶记录），不是写路径写出来的（区间墓碑、借、N1 分隔 key 在 `crates/` 都没有写者）；checker 没打补丁，N1 / I-9.19 / I-9.20 的改写都只是谓词或推的。
- 计数（`expected/counts.txt`）把崩溃各子集那几支里的发布、结构变化一起加了进去，频度是近似。

## 没做什么

- 没跑重型测试；harness 档只在仓副本上跑了自己的一个测试目标（3 条用例），没跑别的 harness 用例与 GPU 判器。
- K1（辩方）不归我；没判正推、辩方的格；没替主 agent 采纳。
- 副本与模型上的数都没在入库装置上重做：要引，照 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」。
- 墓碑树择版本那一半（K2.3 推到墓碑树）没进模型；跨头、w = 2 没建；区间墓碑与 N1 的 checker 改写没写代码。

## 草稿与副本

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r2-opus/`（现 18M）：`rerun4/`（交回前从头跑通的那一次：`sweep-*.out`、`demos.out`、`trace-395.out`、`copy.log`、`copy.out`、`summary/`）、`run1/` `run2/`（开发中两次全量；`run1` 是加「退役树取 R_old」那一维之前的，不作依据，`run2/sweep-*.err` 是 K2.1 引的估时）、`rerun1/`–`rerun3/`（前三次复跑：`rerun2` 的 `sorted-output-sha256.txt` 对不上，查出来是消融读法同号平局时按哈希次序取，改成按 (诞生代号, 实例, 槽) 取之后 `rerun3`、`rerun4` 全 same；`agg.txt` 等判定用的几份四次逐字节相同）、`demo*.out`、`trace203.out`、`trial-*`（估时的试跑）、`copy-run*.log`、`model/`（模型源码的开发副本，与模型目录逐字节相同）。
- 交回前删掉的（`du -sh` 现查的大小）：编译目录 `target/` 924K、`repo-target/` 1011M、`rerun1/`–`rerun4/` 下各自的 `target/`（约 930K 一份）与 `repo-target/`（约 999M 一份）；仓副本 `repo/` 21M 与 `rerun1/`–`rerun4/` 下各一份 `repo/`（各 21M）。删完 `ls` 核过一个不剩。
- 层 0 的崩溃状态：`crates/` 的 `enumerate_layer0` 一次都没调（`crates/` 没有退役记录的写者）；模型自己的崩溃状态各次全量合计约 171 448 × 6 次 ≈ 1.0 × 10⁶ 个，没有一段历史自己超过 10⁶。
- 没入库的：完整模型输出（太大，`rerun.sh` 逐字节复出，`expected/sorted-output-sha256.txt` 钉着）。
