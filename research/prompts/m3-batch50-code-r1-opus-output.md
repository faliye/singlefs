# m3-batch50-code-r1 攻方腿（Opus）报告

## 一、复跑、文件与各格判定一览

**复跑**（每个阶段在前台跑得完；`all` 依次跑全部，本机负载不高时约 7 分钟；交回前在空目录里按阶段从头跑过一遍，`compare` 九份全一致）：

```
SINGLEFS_REPO=/home/fy5090/code/singlefs bash research/prompts/m3-batch50-code-r1-opus-model/rerun.sh <空的工作目录> <阶段>
# 阶段：prepare build edge closed churn-b2f-even churn-b2f-odd churn-b3-even churn-b3-odd m2-build m2-run m3 m4-build m4-run m5-build m5-run compare
```

**模型目录文件的 sha256**（`research/prompts/m3-batch50-code-r1-opus-model/` 下，`find . -type f | sort | xargs sha256sum`）：

```
e9d48bf2bd728b2f2efb331ad1da148bba01b365637d69cb4fab62481f0c11c3  ./apply_mutation.py
1a3526d0aea54d29e5d6a7f8f4c4e769309608fe9dcfb2c8ce44db830552491a  ./expected/churn-totals.out
fd40444d0f4e2c34df79aa61ab5ab10676c45ac429e993c0f760bec8351c734d  ./expected/closed-early.out
62899b8b4d1962a718b92676c4cfb8c1e12e3f2b841688e929c9d062ddfb5bc1  ./expected/edge-mut.out
b7d775d577700cd2f1077958a5763a6fe7bf7052de85af96a25257ad68a45e01  ./expected/edge-orig.out
23c85bf0e0fa1b6926f4270d443fbd176ae4c050930d7fa4b6c234b0d2b7fa22  ./expected/m3-core-lib.out
ca16cbd435604390ca89c21a0bfeac7075428c3827111f355aa741c7acafb9b2  ./expected/m4-absorb.out
bf15e266f4f67cea57978e7b84f5c8ff9787c6c4d0832816e3d1470210f36263  ./expected/m4-core-lib.out
cc9ed6a6f9029505537f45b827eb1c65bf9e981ae60db82a612d5bc9a939dd18  ./expected/m5-random.out
9849392dc30a0c7baa11ecd7cae451c6623755b2155e41fbbed67e1932c0415e  ./expected/m5-rollback-all.out
fd03b79951cdb1751f97254d6e08a0b986aabae8205927d740c5651b54b74195  ./mutations/M2-retire-terms.diff
89fd0cbb5ae06dea9529f4f2f5d49ad5a760cd2ef7507264afda8c84446d7d74  ./mutations/M3-retire-capacity-one.diff
e57d3fa1d21adcc0208f506aa9a06746b12a0cbd86d4cfbf8e66c8141abc8560  ./mutations/M4-left-pair-first.diff
1e44934ce9b148e8291f1067791bacab25a9e2833e9853d3b2f7d109bd9c5028  ./mutations/M5-ring-term-dropped.diff
845dfe91503ae9187c9eb97979a0ec3a20afa2a5335681d68e24e6564b1e89c1  ./probes/opus_probe_churn.rs
6d62fbcca33e6ba91e8e9bd0920589dfa36f8f0969863aa47202e46ff29af404  ./probes/opus_probe_container_closed_early.rs
6c31b5b88aeb607146373b0958a1fa00e0e6a94e235f053adada0969e037a66c  ./probes/opus_probe_unlink_at_the_edge.rs
d022c6bf9688f4d3d86028ab5f756580ace7c6116401cddcbe7381813cda5dca  ./rerun.sh
```

**各格判定一览**（「打中」都只在副本上量过；改法都被攻过零轮）

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K5-2 贴边 unlink 不超卖 | 没打中 | 读法取「每次删除都发成 ∧ 删完之后稳态可用 ≥ 0」；四个起点 104 段、贴边那一格删完稳态最少 18.9 槽；去掉全部退役项（M2）照样绿，候选甲做不成会红的用例 | 二 |
| K5-1 取哪一版、W | 没打中 | 一次删除至多一条退役记录（读码推的）；1 280 步里删除类发布新写最多 19 槽，W 36–39 | 二 |
| K5-3 常驻 3 槽与实占 | 没打中（观测） | 开第一片退役容器让稳态可用永久少 1 槽（墓碑树 3 槽），被 defer 里的槽盖住 | 二 |
| K4-1、K4-2 M3、R3 | 没打中 | churn 1 280 步逐版判叶形状、退役集合、照抄落点，0 次违例 | 三 |
| K4-3 N1 | 没打中 | I-9.12 拦得住「没改」，拦不住区间里别的值，写侧有用例钉 | 三 |
| K4-4 模型与实现 | **打中（判别力）** | 对拍模型里没有合并与借；M3「先右」改成「先左」（M4）core 240 条 + 发布路径 4 条全绿 | 三 |
| K3-1 登记 | 没打中 | 常量集中、checker 每步不红 | 四 |
| K3-2、K3-3 死亡写侧、回退、恢复 | 没打中 | 181 次退役逐次比对，0 次对不上 | 四 |
| K3-4 容器树 | **打中（坏镜像漏报，不分辨批次）** | 退役容器（与谱系容器）不满就关，池级 checker 一条不红；墓碑同形红 I-9.21 | 四 |
| K1-1 Opt2 | **打中（判别力，判别子观测不到）** | 环那一项在走得到的状态上恒不大于 cur（推的）；去掉它（M5）回退整份 25 条与回退密集随机历史全绿 | 五 |
| K1-2、K2 | 没打中 | 回退到 cur、连续回退、回退后崩了再挂；台阶行、自己那一行、恢复都不误判 | 五 |
| K6 | 没打中（GPU 没跑） | 1 280 次 checker 0 误报；GPU 判器在 checker 档，没量 | 六 |
| K7 | 两刀照绿 | M4、M5；M3 被 core 单测红 2 条罩住 | 七 |

## 二、K5 准入（R2 两样都做）

### K5 第 2 条（必做）：贴边建满后 unlink 不超卖

**先给读法。** 我取的读法（记作「兑现读法」）：超卖 = 准入放进来的东西之后兑现不了。落到可量的两样上：
- ① unlink 的每一次删除类发布（意图、各批）都发成，不因空间被拒（`NoSpaceAfterRaisingTheFloor` 或推抬 F 失败）；
- ② 删除走完之后的「稳态可用」不小于 0：稳态可用 = 准入读数的可用(d) + 这块盘此刻 defer 里的槽数 × 16384（defer 里的槽是换下而还没放回的，转过回退候选窗口就回可分配集合）。这就是实现员候选甲「删除走完、defer 放回之后的稳态上判可用 ≥ 0」，只是用 `DeviceFreeMap::deferred_slots` 现算，不靠推空发布把它真放回去（推根环那一入口在这几格上被拒，见下面 `push=refused`）。

不取「每次发布之后可用(d) ≥ 0」：贴边那一格意图之后可用就是负的（下表 −28），那是环窗口要兜的暂态，D3 原文把删除各批的新写交给环窗口：「删除各批的新写由准入式子里的常驻扣项「环窗口 = (根环槽数 3 × S + 1) × 每批上界 W」兜（D28（挂载期承诺量） 已定项 1，S 是系统配置每区槽数），不进记账行」（`.claude/kb/decisions/03-空间分配.md@已定项 2：墓碑与 livelist 的 FREE 条目由准入控制算进最坏情况`）。实现员候选乙（新占未放回 ≤ 环窗口）在这块盘上恒成立（下表新占最多 29 槽，环窗口 900 槽），判不出东西。

**装置**（副本上量，`research/prompts/m3-batch50-code-r1-opus-model/probes/opus_probe_unlink_at_the_edge.rs`）：随机历史 384 槽那一档小盘（内存稀疏盘），经挂着的会话：建 n 个空 inode →（可选）先 unlink 一个 → 顺序写把第一个文件写成 du 个数据单元（准入判着）→ unlink 一个空 inode（会话一次调用 = 意图 + 各批）→ 读准入读数 → 推根环 → 再建一个空 inode（写入类，判式子）。每个起点只建一次池，每个 du 从内存里那一份拷。四个起点：

| 起点 | 建的 | 先删 | 要删的 | 删那一次发生什么 |
|---|---|---|---|---|
| A | 234 个（叶 [1..=233]、[234, 235]） | 2 | 3 | 左 231 + 右 2 = 233，左吸收右，开出退役树；墓碑树已开 |
| B（A 的对照） | 235 个 | 2 | 3 | 231 + 3 = 234，不并；墓碑树已开 |
| C | 233 个（[1..=233]、[234]） | — | 2 | 吸收 + 墓碑树、退役树同一次开出 |
| D（C 的对照） | 234 个 | — | 2 | 不并，只开墓碑树 |

du 扫 95–120（之前先跑过 A 起点 du = 1、60 两格的小样估时，每段 0.04 秒级），每格一段历史；崩溃状态这一件没有（全是挂着的发布，不枚举崩溃点）。

**贴边那一格**（每个起点能写进去的最大 du，写完可用 0.9 槽；原样在 `research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out`）：

`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out:34`：「BEFORE avail=["0.915", "0.915"] settled=["41.915", "41.915"] alloc=[267, 267] deferred=[41, 41]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out:35`：「AFTER  avail=["-28.081", "-28.081"] settled=["40.919", "40.919"] alloc=[296, 296] deferred=[69, 69]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out:89`：「AFTER  avail=["-25.085", "-25.085"] settled=["41.915", "41.915"] alloc=[293, 293] deferred=[67, 67]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out:187`：「AFTER  avail=["-26.081", "-26.081"] settled=["18.919", "18.919"] alloc=[294, 294] deferred=[45, 45]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-orig.out:274`：「AFTER  avail=["-21.085", "-21.085"] settled=["19.915", "19.915"] alloc=[289, 289] deferred=[41, 41]」

| 起点 | 贴边 du | 删之前 可用 / 稳态 | 删之后 可用 / 稳态 | 稳态差（槽/盘） |
|---|---|---|---|---|
| A 吸收 + 开退役树 | 103 | 0.915 / 41.915 | −28.081 / 40.919 | −0.996 |
| B 不并 | 103 | 0.910 / 41.910 | −25.085 / 41.915 | +0.005 |
| C 吸收 + 墓碑树、退役树同开 | 114 | 0.915 / 22.915 | −26.081 / 18.919 | −3.996 |
| D 不并、只开墓碑树 | 114 | 0.910 / 22.910 | −21.085 / 19.915 | −2.995 |

全部 104 段（四个起点 × 26 格）：58 段写得进、写得进的 unlink 都做成（`grep -c 'unlink=ok'` 58，`'unlink=refused'` 0），没有一次推抬 F（`raises=0`）。

**结论：在兑现读法下造不出超卖。** ① 每次删除都发成；② 删完之后最小的稳态可用是 C 那一格 18.919 槽，远大于 0。

**稳态差拆出来是什么**（A 对 B、C 对 D，同一块盘、同一个 du，只差删的那一次并不并）：开出第一片退役容器的那一次 +3 槽（容器 2 + 根 1），被吸收的那片叶放回 2 槽，标签 6 降 152 字节（n_live、n_记录各少 1，两份副本），净 −1 槽；只开墓碑树净 −3 槽；两样都开 −4 槽。也就是说，**常驻那 3 槽在第一片开出来的那一次不释放**（`crates/singlefs-core/src/admission.rs:351`「RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,」照旧加进常驻槽）：实占与预付同时在账上，这一次 unlink 让稳态可用永久少 1 槽（实现员第八节第 1 条附注说的那件事，这里量出来是 1 槽 / 4 槽）。这 1–4 槽被贴边那次写自己换下、还在 defer 里的 22–41 槽盖住，所以 ② 不红。

**去掉退役那几项会不会让它红：不会。** 变异 M2（`research/prompts/m3-batch50-code-r1-opus-model/mutations/M2-retire-terms.diff`：标签 6 的退役常驻 3 + H_退役 与 35 × n_记录、ckpt_cost 的退役项、中央映射树删插数的退役那几把、W 的 6 + H_退役 全部乘 0）在副本上重编同一个装置，du 扫 160–200：
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-mut.out:154`：「BEFORE avail=["1.414", "1.414"] settled=["42.414", "42.414"] alloc=[459, 459] deferred=[41, 41]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/edge-mut.out:155`：「AFTER  avail=["-27.583", "-27.583"] settled=["41.417", "41.417"] alloc=[488, 488] deferred=[69, 69]」
贴边那一格从 du 103 挪到 198（多出的 95 个单元几乎都是环窗口少扣的 25 × 6 = 150 槽），删那一次照样做成、稳态差照样 −0.997。变异那一趟 164 段里 160 段写得进，unlink 都做成、0 次被拒。C、D 两个起点在变异下 du = 200 还写得进（扫描上沿，贴边那一格没量到）。

**四句**（按「打中之后先答四句」答在这条没打中的结论上，因为它决定实现员那条用例能不能写）：
1. 分不分辨臂：不分辨。有退役项与没退役项（M2）在兑现读法下都绿，稳态差都是 −1 槽；退役预付在这块盘、这几段历史上观测不到。
2. 系统看不看得到：看得到稳态可用（`deferred_slots` 与准入读数都在会话里）；看不到「常驻 3 槽是给下一片的」——式子里没有一项分得开「这 3 槽已经被这一片用掉」与「还留着给下一片」。
3. 满足的是判据字面哪一分句：兑现读法的 ①② 都满足（没打中）；若换成「删除不让稳态可用变少」（D3（空间分配） 已定项 17「释放空间这个操作本身不需要申请空间」的定量读法），A、C、D 三格打中，但 D 那一格是墓碑树（第 50 批之前就有的形状），条款射程原文又写着它「不承诺「释放一定立刻成功」」（`.claude/kb/decisions/03-空间分配.md@已定项 17：释放空间这个操作本身不需要申请空间`），不算打中，只记观测。
4. 改法在这几格上还中不中：实现员候选甲（只在稳态判）在这几格上绿，去掉退役项也绿，**做不成会红的用例**；候选乙判不出东西（环窗口 900 槽）；候选丙（不做这条用例，退役预付只由单元层钉值与变异行罩着）是这块盘面上唯一不假装的选择。要让「去掉退役项就红」成立，判据得直接看第一片开出那一刻的落点（那一次发布的落点判要的槽数 ≤ 物理空槽，且在 M2 下不成立），而那需要一块环窗口扣得很少的盘面——小盘为环窗口补足了 1007 槽，环窗口本身就把 3 槽罩住了（推的，没量）。

### K5 第 1 条：各项取哪一版、一次发布至多几条退役记录、W 罩不罩得住

- 取哪一版（读码）：环窗口里的 H_退役 取要接着的那一版（`crates/singlefs-core/src/admission.rs:1568`「retirement_tree_internal_levels: version_to_build_on」），标签 6 取这次发布之后那一版（实现员报告第五节第 3 条，`transaction.rs` 第 4398 行起）；写入类发布不动退役树，前后两版的 H_退役、n_记录 之差只在删除类发布上出现，而删除类发布不判式子，所以「取哪一版」在判式子的那几次发布上没有分别。ckpt_cost 那一项按这次之前的层数、长高那次少 1 块是已知的 C766，不重报。
- 一次发布至多几条退役记录（读码 + 量）：最后一批只删一条 inode 记录（`crates/singlefs-core/src/transaction.rs:5320` 那一处每批调一次 `remove_record_from_leaf_containers`）；照 M3 的写法，中间的叶在任何时候都不少于 117 条（只有最右那片可以低于下限），所以一次删除至多让一片叶身份消失：吸收一次，或借完之后借出那片吸收最右那片一次，或最右那片删空一次。这是读码推的：随机历史里每一次删除类发布都比了「上一版叶身份集 − 这一版 = 这次死亡 txg 的退役记录」、0 次对不上（见 K4 节的 churn），但一次发布写了几条没有单独数。W 里那 6 + H_退役 按「跨两片容器」算，对 1 条是宽的。
- W 罩不罩得住（量）：churn 装置每一次删除类发布都拿「这次真写出的单元按槽取整之和」（`rewritten` 里的单元）比 W（按这块盘的分配器经 `ring_window_on_one_device` 现算再除以 25）。产品容量 640 步（b2f）新写最多 17 槽、W = 36；退役树压到每片 1 条、扇出 2（b3，退役树长到 11 片、内部 3 层）新写最多 19 槽、W = 39：
  `research/prompts/m3-batch50-code-r1-opus-model/expected/churn-totals.out:28`：「batch=b3 seed=307 TOTAL issues=0 stats={"W": 39, "carried": 21099, "checker_runs": 40, "leaf_lost": 2119, "max_deletion_new_slots": 19, "max_retire_containers": 11, "max_retire_internal_levels": 3」
  没有一次超过 W。**没打中。** 限度：只在 384 槽那一档小盘上量（分配记录树每盘只罩 3 片叶，W 里分配记录树那一项只有 6）；4 GiB 盘上 W = 374，实现员量过吸收那次 14 槽，我没再量。

### K5 第 3 条：常驻 3 槽与实占同时在账上会不会让可用变负

量出来的就是上面那张表的稳态差：开出第一片退役容器那一次让稳态可用永久少 1 槽（只开墓碑树少 3 槽，两样同开少 4 槽）。它让「删除之后的可用(d)」变负——但贴边那一格不开退役树（B）也照样变负（−25），那是 C669 ② 已记的「unlink 之后可用量反降 20–24 槽」；能看出这 1 槽的只有稳态读法，而稳态读法下贴边那次写自己留在 defer 里的 22–41 槽把它盖住。换盘面会不会让**稳态**可用变负：要找一种写入类发布换下的槽少于 4 的盘面（推的：每次写入类发布至少换下树表 1 槽、inode 叶 2 槽与分配记录树、记账树、中央映射树各一条路径，下限明显高于 4），没造出来。**没打中**，记成观测：常驻那一份在第一片开出来时不释放，是 D3（空间分配） 已定项 2 的写法（墓碑树同形），代价每棵树一次 3 槽。

## 三、K4 M3 合并、均分借、N1、R3

### 装置：随机历史逐版判（`probes/opus_probe_churn.rs`）

384 槽小盘（内存稀疏盘），每步随机取一样：建 k 个空 inode（k ∈ {1, 5, 40, 117, 233, 300, 466}）、从随机一片叶里连删 m 条（m ∈ {1, 10, 60, 115, 116, 117, 118, 200, 233}，从小号起 / 从大号起 / 随机）、管理员回退到最近 6 条根里的随机一条（含 cur 自己）、崩了再挂、覆盖写。每一次发成的版本（unlink 一次调用里的意图与各批逐版）判：
1. 叶内 key 严格升序、叶间 key 不重叠、每片 1–233 条、分隔 key ≤ 本片最小 key 且 > 左邻最大 key；
2. 非最右的叶不少于 117 条（M3 的不变量；最右那片照 D8 不从左借，`crates/singlefs-core/src/inode_tree.rs:480`「它是最右那片时不借（右从左借会破「记录号 ≥ 容器号」）」）；
3. 退役完备（非回退的发布）：上一版叶身份集 − 这一版 = 这一版死亡 txg 等于自己 txg 的退役记录；退役树只追加；回退那一版退役树与 cur 逐条相同；
4. R3：同一身份的叶内容没变而落点变了记「重写了没变的」，内容变了而落点照抄记违例；
5. 水位：水位 > 树里最大号、不比上一版低、这一版自己那一行恰好一条、台阶行是上一版台阶行的前缀延伸；同一个 inode 号不以另一个出生代再出现；
6. 删除类发布新写槽数 ≤ W（K5 第 1 条）。
每一步之后对整张盘跑池级 checker（`check_pool_image`）。

**取样与批**：每段 40 步；先跑 2 段小样估时（一段约 70 秒，checker 占大头），之后每批 8 个进程并行、每个进程 2 段，一批约 150 秒（复跑脚本里改成每个进程 1 段、一批拆两个阶段，`expected/churn-totals.out` 一行一个种子，两两加起来与原来按进程记的 16 行逐项相同）。正式两批：b2f（产品容量，种子 200–215）、b3（退役树每片 1 条、谱系每片 1 条、码 2 节点扇出 2，种子 300–315）；另有早一版装置跑过的 b1（种子 100–115）与 b2（种子 200–215，与 b2f 逐字相同）不进产物。崩溃状态这一件没枚举（见第十节限度）。

| 批 | 步 | checker 次 | 违例 | 退役 | 借（分隔 key 改过） | 叶照抄（落点没变） | 删除类发布新写最多 / W |
|---|---|---|---|---|---|---|---|
| b2f | 640 | 640 | 0 | 92 | 225 | 138 782 | 17 / 36 |
| b3 | 640 | 640 | 0 | 89 | 163 | 93 619 | 19 / 39 |

「重写了没变的」两批都是 0 次（统计里没有这一键）。**K4 第 1、2 条没打中**：没有留下低于下限而没处理的中间叶，没有两片之和超 233 被吸收，没有分隔 key 与 key 次序破，没有内容变了却照抄落点。

### K4 第 3 条：N1 删 I-9.4 第三句之后，分隔 key 写错拦不拦得住

读码：I-9.12 在 checker 里判三样——分隔 key ≤ 本片最小 key、> 上一条的分隔 key、> 左邻的最大 key（`crates/singlefs-checker/src/walk.rs:3298`「self.judgements.judge("I-9.12", separator <= smallest, || {」、`crates/singlefs-checker/src/walk.rs:3309`「.judge("I-9.12", *separator > previous_largest, || {」）。借之后右片分隔 key 没改（还是容器号，≤ 左片借走之后的最大 key）照红；改成 (左邻最大 key, 本片最小 key] 里别的值（例如左邻最大 key + 1）checker 不红——条款要的是「改成它这一版的最小 key」，镜像上分不出「借那次改成了最小 key、之后最小那条又被删了」与「借那次就写错了」，判不了是镜像判据的本分，不算漏。写侧由 `inode_leaf_absorb_and_borrow_on_the_unlink_path.rs` 钉 525 那一判罩着（变异表第 50 批 N1 那一行）。**没打中。**

### K4 第 4 条：理想模型与胶水对 M3 是不是独立的两份

**打中（判别力）。** 对拍模型里根本没有 M3：模型罩的历史只删第一个文件、删完之后只往右建，叶容器的合并与借一次都走不到（`crates/singlefs-harness/src/model.rs:105` 那一行：「那次最后一批让唯一那片叶容器身份消失、追加一条退役记录，退役树恒只有容器 1 那一片」）；`model.rs`、`model_comparison.rs` 里没有吸收、借、117、233 的合并判法（`grep -n '吸收\|absorb\|borrow\|借' model.rs model_comparison.rs` 零命中）。D8 已定项 6 把合并交给「单测 / 模型对拍」：「合并时机是写路径策略（单测 / 模型对拍），不进格式」（`.claude/kb/decisions/08-核心索引结构.md:171`）——模型对拍那一半今天是空的，M3 的写法只剩 core 单测与发布路径上的 4 条用例罩着。

拿实现员自定、零轮的「先查右邻、再查左邻」量这一空缺：变异 M4（`mutations/M4-left-pair-first.diff`，`crates/singlefs-core/src/inode_tree.rs:466`「if right_pair_fits {」那一支与 `crates/singlefs-core/src/inode_tree.rs:468`「} else if left_pair_fits {」对调，先查左邻）在副本上：
- core 单测 240 条全过：`research/prompts/m3-batch50-code-r1-opus-model/expected/m4-core-lib.out:1`：「test result: ok. 240 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s」
- 发布路径那 4 条（`inode_leaf_absorb_and_borrow_on_the_unlink_path`）全过：`research/prompts/m3-batch50-code-r1-opus-model/expected/m4-absorb.out:5`：「test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.80s」
两种次序在「左右两对都装得下」时结局不同：叶 [117, 117, 117] 删中间一条，先右是中间吸收右（[117, 233]，右片身份消失），先左是左吸收中间（[233, 117]，中间身份消失）——两种都是合法镜像，checker 分不出，所以只有对拍或单测能钉它。

四句：分辨臂——分辨「先右」与「先左」两种写法，但两种都不违任何条款（条款只写「左吸收右」），所以它分辨的是**实现与它自己报告的选择**，不分辨对错；系统看得到——看得到（同一份输入两种结局）；字面分句——D8 已定项 6「合并时机是写路径策略（单测 / 模型对拍）」那一句的「模型对拍」没有对象；改法——见第八节表 F1。

## 四、K3 退役树与退役记录

### K3 第 1 条：登记与新池新建文件

读码：树 ID、水位、树表条目数都收在 `singlefs-format` 的常量里（`crates/singlefs-format/src/lib.rs:216`「pub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;」、`crates/singlefs-format/src/lib.rs:197`「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;」），GPU 那一侧是 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:141`「const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 12u;」。`grep` 全部 `crates/` 的 `.rs` 与 `.wgsl` 没找到还按 11 条、65548 当水位读的地方。churn 每段都从 mkfs 起、每步跑 checker（I-9.16 ④、I-9.27 都在里面）没有一次红。**没打中。**

### K3 第 2、3 条：死亡写侧、回退与崩溃恢复

churn 的第 3 项判法每一次发布都比退役集合（b2f + b3 共 181 次退役、1 280 步、回退与崩了再挂穿插）：0 次对不上。一次发布里「分裂、合并、再分裂」在现行写法里走不到（写入类发布只追加、删除类发布只删一条，见 K5 第 1 条），「一批删掉几片、连锁吸收」在中间叶 ≥ 117 的不变量下至多一片。回退那一版退役树与 cur 逐条相同（churn 第 3 项）；崩了再挂之后 checker 的 I-9.32–I-9.35 不红。**没打中。**限度：崩溃恢复抛弃根那一形（要注入读错）churn 没造，靠现成的 `random_histories` 回退密集档（M5 那一节顺带在副本上跑过，绿）。

### K3 第 4 条：容器树一次追加多条、祖先并集

**打中（坏镜像漏报，不分辨批次）。** 退役容器「满了才关」是格式条款：「有新记录的那次发布才重写开放容器，没有的发布照抄；满了才关；第一版不回收」（`.claude/kb/layout/03-third-txn.md:19`）。墓碑树有一条判它的不变量（I-9.21「产品镜像里非最右墓碑容器装满」，`.claude/kb/invariants.md:309`），退役树没有：I-9.31–I-9.35 判身份、记录、谱系与完备，不判容器装没装满、容器号是不是连号。装置 `probes/opus_probe_container_closed_early.rs`：同一段历史（建 233 个、删 234、建 1 个、删 235、回退两次），只换只供测试的容量开关：
`research/prompts/m3-batch50-code-r1-opus-model/expected/closed-early.out:7`：「retirement containers=2 records=[1, 1] numbers=[1, 2]」
`research/prompts/m3-batch50-code-r1-opus-model/expected/closed-early.out:10`：「checker violations: 0」
`research/prompts/m3-batch50-code-r1-opus-model/expected/closed-early.out:16`：「VIOLATION I-9.21: 根（实例 1、txg 9）：墓碑树（树 65545）里不是最右的容器 1 装了 1 条，不是 815（满了才关）」
容器树长高、跨片那一半没打中：b3 那一批把退役树压到每片 1 条、扇出 2，长到 11 片、内部 3 层，640 步 checker 不红；一次发布追加两条以上在中间叶 ≥ 117 的写法下走不到（K5 第 1 条），多条那一支只有单测罩着。退役容器（与谱系容器）装 1 条就关，镜像上与产品写出的没有区别，池级 checker 一条都不红；墓碑容器同样装 1 条就关，I-9.21 红。

四句：分辨臂——不分辨第 50 批的取舍，谱系树（第 50 批之前就有）同缺，病根是「容器树三棵共用写法、只有墓碑树配了满了才关的判法」；系统看得到——看得到（容器记录数与位置都在镜像里）；字面分句——layout 03 第 19 行「满了才关」，对应墓碑那一条是 I-9.21；改法——第八节 F2。写侧那一半有单测罩着：变异 M3（`mutations/M3-retire-capacity-one.diff`，产品容量取 1）在 core 单测里红 2 条（`research/prompts/m3-batch50-code-r1-opus-model/expected/m3-core-lib.out` 前两行，`retirement_tree::tests` 里开第一片、同一身份两条那两条），所以漏掉的是崩溃枚举与随机历史这一路：层 0 的每个崩溃状态都跑 checker，那一路上退役容器提早关看不见。

## 五、K1 回退水位（Opt2）与 K2 台阶行、谱系

### K1：Opt2 那一项环里的根，在走得到的状态上从不起作用

**打中（判别力，判别子观测不到那一型）。** 实现按条款取 max：环里只取没被崩溃恢复抛弃的根（`crates/singlefs-core/src/mount.rs:8120`「&ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),」），再与 cur 那一版内存里的取大（`crates/singlefs-core/src/mount.rs:8123`「.map_or(current.inode_number_watermark(), |ring| {」）；条款原文见 `.claude/kb/decisions/23-journal的角色与格式.md:385`「inode 号水位 = max(cur 那一版内存里的水位, 环里读得出、按实例表没被崩溃恢复抛弃的根的水位)」。

推的（没穷举证明）：水位沿一条时间线只升不降（每次发布照抄或抬高，回退取 max）；只有崩溃恢复让 cur 退回较旧的根，而那一次被越过的较新根按新实例写下的实例表行全都判成被抛弃、被这一滤滤掉。所以环里留下来的每一条根都是 cur 时间线上更早的一版（或某次回退之前那一段上的一版），水位都不大于 cur 的——max 恒取 cur 那一项，环那一项只是多重建了 3S + 1 版。

量过（副本）：变异 M5（`mutations/M5-ring-term-dropped.diff`，交给 `highest_inode_number_watermark_in_the_ring` 的根集合切成空，水位恒取 cur 那一版内存里的）：
- `rollback_by_a_forward_publish` 整份 25 条（含 22 条慢档）全过：`research/prompts/m3-batch50-code-r1-opus-model/expected/m5-rollback-all.out:1`：「test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.41s」——里面就有变异表「第49批 W3 Y1 Opt2」那一行点名的 `a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root`；
- 随机历史回退密集档（含崩溃恢复抛弃根）：`research/prompts/m3-batch50-code-r1-opus-model/expected/m5-random.out:1`：「test rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms ... ok」。
变异表里 Opt2 那一行只钉「被抛弃的也算」一个方向；「环那一项整个不算」这一刀照绿，而按上面推的，它在走得到的状态上与产品写法逐字相同。

四句：分辨臂——Opt1 对 Opt2 分辨得开（被抛弃的根，现成用例钉着）；Opt2 对「只取 cur」分辨不开，这一格不分辨臂；系统看得到——回退那一刻环与 cur 都看得到，但能让两者不同的输入（环里一条没被抛弃、水位高过 cur 的根）推的走不到；字面分句——D23 已定项 14「水位」一格的 max 里「环里读得出、按实例表没被崩溃恢复抛弃的根的水位」那一项；改法——第八节 F3。用户已定的是 Opt2 这个选择本身，不算打中；这里报的是那一项在实现上没有会红的东西，以及每次回退白重建 3S + 1 版（S = 8 时 25 版）的代价。

### K1 第 2 条与 K2：几种形状

churn（b2f + b3，1 280 步）里回退取的是最近 6 条根里随机一条（含 cur 自己，即「回退目标就是 cur」）、连续两次回退、回退之后崩了再挂都出现过；每一版判水位不降、自己那一行恰好一条、台阶行是上一版台阶行的前缀延伸、同号不以别的出生代重发，每一步 checker（I-9.30 ①②④、I-3.13）——0 次违例，崩了再挂一次都没被拒（恢复要求「代段 = 根 txg 的那一行」在，合法镜像上没误判）。环里读不出的根那一形有现成用例 `with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record`（M5 下也绿，同上）。K2 第 4 条（ckpt_cost 记账那一项）读码：`accounting_tree_nodes_after_the_publish_built_on` 用写路径同一个规划器按「这一版的行换成 txg + 1、再加照抄的台阶行」现算整棵树的节点数，跨叶那次多出的节点算进去了；没量。**K1 第 2 条、K2 第 1、2、4 条没打中；K2 第 3 条的 ③（C769）是已记的欠账，没另造镜像。**

## 六、K6 checker 与 GPU 判器

- 合法镜像误报（K6 第 1 条）：churn 1 280 步、每步一次池级 checker，0 次红；形状里有回退越过叶出生（回退到 6 条以内任一根）、删 → 回退 → 再删、借后吸收、崩了再挂。克隆头 `crates/` 没有路径，没造。被崩溃恢复抛弃的根进候选集那一形 churn 没造（要注入读错）。**没打中。**
- GPU 与 CPU（K6 第 2 条）：GPU 判器在 checker 档包里，这一轮不跑 checker 档测试，没量。读码只看了一处：GPU 走过的树表一张记 44 个字 = 4 + 8 棵用户可见树 × 5（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:809`「const C_APPEARANCE_WALKED_TREE_TABLE_WORDS: u32 = 44u;」），树表 12 条里用户可见的不到 8 棵，没看出容量问题。
- 坏镜像各自只红自己（K6 第 3 条）：只造了第四节那一张（退役容器提早关，checker 一条都不红）；I-9.16 ④、I-9.27、I-1.7 的坏镜像没另造。

## 七、K7 判别力（顺带）

| 变异（副本上改） | 改了什么 | 跑了什么 | 结果 | 判 |
|---|---|---|---|---|
| M2 `mutations/M2-retire-terms.diff` | 准入里退役那几项全乘 0 | K5 贴边装置 | unlink 都做成，稳态差与原版同 | 贴边用例罩不住（K5 第 2 条），各项另有单测行（变异表第 50 批 R2 四行），我没跑 core 单测 |
| M3 `mutations/M3-retire-capacity-one.diff` | 产品退役容器容量取 1 | core 单测 | 红 2 条 | 罩住 |
| M4 `mutations/M4-left-pair-first.diff` | M3 先查左邻 | core 单测 240 条 + 发布路径 4 条 | 全绿 | **照绿**（第三节） |
| M5 `mutations/M5-ring-term-dropped.diff` | Opt2 环那一项不取 | 回退整份 25 条 + 回退密集随机历史 | 全绿 | **照绿**，且推的与产品逐字相同（第五节） |

题面点名的另几样（阈值、R3 照抄、死亡 txg、ckpt_cost 之前之后）变异表里各有行（第 50 批 M3 两片之和恰好 233、R3 两行、R1 死亡 txg 取前一个 txg 等），我没逐行复跑。

## 八、改法（都只在我的模型上量过、被攻过零轮）

| 编号 | 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|---|
| F1 | 给 M3 一个不经 `inode_tree.rs` 的参照：模型里补合并与借（或 harness 里另写一份按条款的叶形状参照，与发布路径逐次比叶的记录数、身份、退役集合），再补一条「左右两对都装得下」的用例钉「先右」 | K4 第 4 条、M4 | 推的（M4 照绿是量过的；补了之后 M4 会不会红没量） |
| F2 | 退役树（顺带谱系树）补一条与 I-9.21 / I-9.22 同形的判法：产品镜像上非最右容器装满 959 条、容器号沿叶序连号 | K3 第 4 条 | 推的 |
| F3 | Opt2 环那一项：或者补一条构造态单测（交一份 cur 水位低于环里一条有效根的输入，直接调取水位那一段）把它钉住，或者在条款里写明它在走得到的状态上冗余、实现可以不重建环里各版 | K1、M5 | 推的 |
| F4 | 「贴边建满后 unlink 不超卖」这条用例按实现员候选丙处理（不做），或判据改成看第一片退役容器开出那一刻的落点 | K5 第 2 条 | 推的（候选甲在原版与 M2 下都绿是量过的） |

## 九、没打中的形状

- K5 第 2 条：四个起点（吸收开退役树 / 不并 / 墓碑树退役树同开 / 只开墓碑树）× du 95–120（原版）与 160–200（M2），贴边那一格删一个空 inode；没试删带数据单元的文件、没试删除走到一半崩了再挂之后再贴边写。
- K5 第 1、3 条：384 槽一档小盘；产品容量与「退役每片 1 条、扇出 2」两种容量；没上 4 GiB 盘与 240 / 256 槽窄盘。
- K4、K3、K6：churn 2 批 × 16 段 × 40 步（种子 200–215、300–315），操作 5 种，随机挑叶连删；没造崩溃恢复抛弃根、没造读错、没造克隆头、没造一次发布追加两条以上退役记录（现行写法走不到）。
- K1：回退目标取最近 6 条根；没造「环里有一条没被抛弃、水位高过 cur 的根」——按第五节推的，它走不到。

## 十、这条腿自己的限度

- 崩溃状态一个都没枚举。churn 与贴边装置都是挂着的发布；层 0 的枚举函数在 checker 档包里，派发写明不跑 checker 档测试，我没把它引进原型。累计跑过的崩溃状态数：0（交接的腿照这个数接着算）。
- 池级 checker 在这块小盘上一次约 0.5–1 秒，churn 每段 40 步就要 70 秒，取样只到 1 280 步；同一份提示再抽一次（`-opus-s2-`）之前，第三、四、六节的「没打中」只算一次抽样。
- 副本上的数（M2–M5、churn、贴边表）都是副本上的，要进 kb 得在入库装置上重做。
- GPU 判器一行没跑。
- 编译目录：我手上第一次跑 M3–M5 时几份变异副本共用一个编译目录，cargo 按相对路径与时间戳判新旧，M5 那一次编出的可能是上一份副本的 core（复跑脚本第一次跑 m4 就因此拿到 M3 的二进制、退 101）。`rerun.sh` 改成每个变异各用自己的编译目录之后，M2–M5 在空目录里各自重编重跑，`compare` 九份全一致；报告里引的 `expected/` 是改之前手跑的那一份，与改之后重跑的除耗时外逐行相同。
