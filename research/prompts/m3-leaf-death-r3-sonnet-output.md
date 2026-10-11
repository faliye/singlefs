# m3-leaf-death-r3 正推腿（Sonnet）报告（2026-10-08）

批次：`records/2026-10-01-删除与并发事务.md#删除线第 47 批`。立场：正推。格：K1（三个候选并排、前两轮的数能不能并排、三处没被看过的地方）、K3（N1 新形对 N2）。
只读冻结树 `~/.local/share/singlefs/work/m3-leaf-death-r1/tree/` 与 kb 快照；没读这一轮攻方腿的提示与产出，没读 m3-w3 两轮的东西。方案按 `crates/` 今天的实现来谈。

本报告分段落盘（每段不超过 150 行）。状态：完成。**结论在第 8 节的「判定一览」**；前面各节先交口径、再交量、最后交判。
引文规矩：kb 条款写 kb 文件自己的行号（`grep -nF` 现取）；代码写冻结树里的全路径:行号；产物写 `research/prompts/m3-leaf-death-r3-sonnet-model/expected/` 里的行或前两轮模型目录里的行。
「量」= 模型或仓副本上跑出来的（副本上的数不进 kb，要引由主 agent 在入库装置上重做）；「推的」= 按条款或代码推、没跑；「复核不了」写原因。

## 0　口径：这份报告里每个词量的是什么

一个候选 = 一种让「叶容器 / 记录死了」留在盘上、让扫盘重建读得出的办法。四个候选在**同一段逻辑历史**上各走一遍，读法各取自己最强的一形：

| 名字 | 是什么 | 读法 |
|---|---|---|
| Today | `crates/` 今天：删空即摘、不写退役记录、意图那次写整对象墓碑（`transaction.rs:5997`）但读侧不用它 | S：每组取最新读得出的一版，同号几条取诞生代号最大的版本供出 |
| RetA / RetB | 候选一：删空摘掉与被吸收的右半各写一条退役记录（带死亡 txg）。改法甲（RetA）：退役树随现行，只按 C113 定案 P2 择版本、记录按死亡 txg 在不在谱系上滤；改法乙（RetB）：管理员回退那次退役树取 R_old，与叶同一套滤法 | S + 退役集合 |
| Persist / PersistX | 候选二：零条目版本一直留在叶序里（Persist：合并只在紧邻的非空叶之间做；PersistX：合并跨过零条目叶，第二轮辩方的写法）；叶里被吸收的右半同样留零条目版本 | 零条目版本是树持有的活单元：记录算死、槽照收 |
| Tsz / Fsz | 候选三：读侧用今天意图那次已写的整对象墓碑，在所有版本里不收，加供出规则（一片叶的版本只有供出了至少一条没压掉的记录才算活单元）与数据单元门。Tsz 墓碑在意图那次生效（今天的写法），Fsz 把墓碑的写挪到最后一批 | 墓碑读法 |

三个名字空间：`inode`（重建出的记录与真值逐条比）、`data`（数据单元按重建出的记录判，对象出生代相等且锚点 < size 折成的单元数）、`units`（分配记录 / 映射按槽号算：被收下的单元的槽与真值引用的槽比，E177 页「读数：逐树状态」那几棵派生树的共同来源）。
r = 重建里有、真值里没有（复活）；m = 真值里有、重建里没有（误判死亡）；m 再分「unlink 途中」（现行那一版队列非空，即意图到最后一批之间）与「途外」。

K1 要的「同一组历史与同一套口径」，在我的模型里这样保证（`research/prompts/m3-leaf-death-r3-sonnet-model/src/model.rs`）：
- 用户动作的随机数按（种子, 动作序号）取，发布内的故障（崩溃、丢根）按（种子, 动作序号, 本动作内第几次发布）取：`research/prompts/m3-leaf-death-r3-sonnet-model/src/model.rs:316` 起的 `begin_action` 与 `publish` 开头；崩溃与不崩的掷法不看候选写出多少单元，所以各候选走的是同一段逻辑历史。
- 每次动作开头把逻辑状态（记录、数据内容、队列、水位，与叶怎么分无关）的哈希滚进一个值；`expected/history-hashes.txt` 逐 (形状, 种子) 核：同一 (形状, 种子) 下所有候选、所有借法、所有几何的哈希只有一个。
- 只记主线的写（崩溃支线只判不记账），所以各候选「相对今天多写」可以直接相减。

## 1　前两轮四套模型的前提（只列前提，判在后面）

| 前提 | 第一轮攻方（A） | 第二轮攻方（B） | 第二轮辩方（C） | 第一轮正推（D） |
|---|---|---|---|---|
| 装置 | E177 装置拷贝加 `leaf_death.rs` | 自己写的，不是 E177 拷贝 | A 的拷贝加改动 | E177 装置拷贝加 python 补丁 |
| 历史 | E177 51 段 + 合并 64 段 | 随机动作 1500 / 600 步 | 同 A，另加有序删 36 格 | 同 E177 51 段，加合并两种 |
| 谁有数据 | 只有 1 号 | 所有 inode | 只有 1 号 | 只有 1 号 |
| 怎么挑删 | E177 的族；合并段随机 / 集中 | 70% 落在盯着的一片叶上，其余随机 | 同 A；sweep 里有序 | E177 的族 |
| 故障 | 没有（干净发布之后扫） | 崩溃 / 丢根 / 管理员回退 | 没有 | 没有 |
| 计数的分母 | 主线发布 | 主线加崩溃支线里的发布 | 同 A | 主线发布 |
| 叶容量 | 233 | 233 与 6 | 233 | 233 |

出处（逐字，行号现取）：
- A：「E177 第一段的 51 段历史 × G-环、G-宽」（`research/prompts/m3-leaf-death-r1-opus-output.md:75`）
- A：「这个模型没有崩溃状态（与 E177 一样只有干净发布之后的扫描）」（`research/prompts/m3-leaf-death-r1-opus-output.md:75`）
- A：「攻方自报 Z 的限度：只有 1 号有数据，合并与借从不挪动它」（`research/prompts/m3-leaf-death-r1-main-verification.md:43`）
- B：「取样：2 种形状（叶容量 233、退役容器 959 条、起始 700 个空 inode」（`research/prompts/m3-leaf-death-r2-opus-output.md:97`）
- B：「计数（`expected/counts.txt`）把崩溃各子集那几支里的发布、结构变化一起加了进去，频度是近似。」（`research/prompts/m3-leaf-death-r2-opus-output.md:299`）
- B：「cap233 no_borrow publishes=368856」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:90`）
- B：「cap233 no_borrow final_txg=285611」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:84`）
- C：「源码 = `research/prompts/m3-leaf-death-r1-opus-model/` 的拷贝」（`research/prompts/m3-leaf-death-r2-sonnet-output.md:29`）
- C：「只有 1 号有数据，住最左那片」（`research/prompts/m3-leaf-death-r2-sonnet-output.md:245`）
- C：「换前提（本腿 `r2-sweep`：有序删）」（`research/prompts/m3-leaf-death-r2-sonnet-output.md:204`）
- D：「E177 原装置没有合并，这是副本加的。」（`research/prompts/m3-leaf-death-r1-sonnet-output.md:79`）

B 的计数把崩溃支线里的发布与退役记录也加进去了（`cap233 no_borrow` 一行：publishes 368 856，主线 final_txg 285 611，比 1.29）；A、C、D 与我的模型只数主线。这是「发布粒度」那个前提的第一处差别，B 的分子（退役记录）也含支线里的，我从 B 的产物里拆不开这一项（3.2 节）。


## 2　K1：三个候选并到同一组历史上

### 2.1　复活与误判死亡（量）

形状（`src/main.rs` 的 `SHAPES`）：cap233 = 叶容量 233、起始 700 个空 inode、1500 步、所有 inode 都可能有数据、70% 的删落在盯着的一片叶上（第二轮攻方腿的形状）；cap6 = 叶容量 6、起始 40 个、600 步；
ord233 = 只有 1 号有数据、从小到大删、起始 1400 个（第一轮攻方腿与第二轮辩方腿那种前提）；rnd233 = 随机删、起始 2400 个（多片叶，合并、借、删空都多）。
每个形状 × 3 个几何（G-宽、G-环 取最低号、G-环 随机取）× 7 个候选 × 3 种借法（不借、N1 新形、N2）× 若干种子（cap233 六个、cap6 二十个、ord233 六个、rnd233 四个）。
崩溃（每次发布 3%，在飞的 k 个单元写逐子集枚举，k ≤ 8，更大的取空集、全集、单点、去掉单点）、丢根（1%，三种）、管理员回退（每步 2.5%）都放开；
用户决定的每一步（建、写、删哪个号、什么时候回退）按种子随机放开，只固定前缀（新池新建文件与起始 inode）。全部配置 3 300 段（`configs` 子命令的输出）。

命令与原样输出（自己的读法在 inode 名字空间上；每行是该形状下各几何、各借法、各种子合起来；ret_a 与 today 行含 N1 旧形的段，所以 72 / 240 段）：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh safety research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  today      S              histories=72   r_max=367   r_hist=72   m_max=0  m_hist=0    m_win=0       m_out=0
cap233  ret_a      own_ret_a      histories=72   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap233  ret_b      own_ret_b      histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap233  persist    own_persist    histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap233  persist_x  own_persist_x  histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap233  tsz        own_tsz        histories=54   r_max=0     r_hist=0    m_max=1  m_hist=54   m_win=88785   m_out=0
cap233  fsz        own_fsz        histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap6    today      S              histories=240  r_max=133   r_hist=240  m_max=0  m_hist=0    m_win=0       m_out=0
cap6    ret_a      own_ret_a      histories=240  r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap6    ret_b      own_ret_b      histories=180  r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap6    persist    own_persist    histories=180  r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap6    persist_x  own_persist_x  histories=180  r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
cap6    tsz        own_tsz        histories=180  r_max=0     r_hist=0    m_max=1  m_hist=180  m_win=111156  m_out=0
cap6    fsz        own_fsz        histories=180  r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
ord233  today      S              histories=54   r_max=1261  r_hist=33   m_max=0  m_hist=0    m_win=0       m_out=0
ord233  ret_a      own_ret_a      histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
ord233  ret_b      own_ret_b      histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
ord233  persist    own_persist    histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
ord233  persist_x  own_persist_x  histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
ord233  tsz        own_tsz        histories=54   r_max=0     r_hist=0    m_max=1  m_hist=54   m_win=96450   m_out=0
ord233  fsz        own_fsz        histories=54   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  today      S              histories=36   r_max=291   r_hist=36   m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  ret_a      own_ret_a      histories=36   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  ret_b      own_ret_b      histories=36   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  persist    own_persist    histories=36   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  persist_x  own_persist_x  histories=36   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
rnd233  tsz        own_tsz        histories=36   r_max=0     r_hist=0    m_max=1  m_hist=36   m_win=97701   m_out=0
rnd233  fsz        own_fsz        histories=36   r_max=0     r_hist=0    m_max=0  m_hist=0    m_win=0       m_out=0
```

- 今天的读法 S 在每个形状上都复活：「cap233 today S inode  … histories=72 … r_peak_max=367 … histories_r_pos=72」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:2`）；「ord233 today S inode  … histories=54 … r_peak_max=1261 … histories_r_pos=33」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:176`）。
  另有 cap6 的 「cap6 today S inode  … histories=240 … r_peak_max=133 … histories_r_pos=240」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:44`） 与 rnd233 的 「rnd233 today S inode  … histories=36 … r_peak_max=291 … histories_r_pos=36」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:470`）。
- 候选一两个改法、候选二两个写法、Fsz：inode、data、units 三个名字空间的 r、m 在四个形状全部为 0。例：「cap233 ret_a own_ret_a inode  … histories=72 … r_peak_max=0 … m_peak_max=0」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:8`）；「cap233 persist own_persist units  … r_peak_max=0 … m_peak_max=0」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:24`）。
  这是「没打中」，一次抽样的量不算数（正文第五节）；它覆盖的是这些随机历史，不是所有历史。
- 候选三 Tsz：r 恒 0；m 只出在 unlink 途中，途外恒 0：「cap233 tsz own_tsz inode  … histories=54 … m_peak_max=1 … histories_m_pos=54 … m_scans_in_window=88785 … m_scans_outside_window=0」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:35`）。
  途中漏的不只一条 inode 记录：units 名字空间 m 峰值 「cap233 tsz own_tsz units  … m_peak_max=11」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:36`） 与 「cap6 tsz own_tsz units  … m_peak_max=17」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:78`）（那个对象还没截掉的数据单元与装它的叶单元；量随文件大小涨——这些历史里文件至多 8 个单元，E177 大文件 300 个，第二轮辩方量到 extent 300、分配记录 301，同一机制）。
- 装置看得见这几种错（消融，每个形状每个候选两段种子）：去掉 P2 或去掉版本谱系滤，候选一与候选三都红：「ablation cap233 ret_a own_ret_a_noP2 inode  … histories=18 … r_peak_max=19 … m_peak_max=6」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:557`）；「ablation cap233 ret_a own_ret_a_noLin inode  … histories=18 … r_peak_max=146 … m_peak_max=187」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:554`）。
  第二轮攻方腿打中的「改法丙」（退役容器与叶同一套滤法择版本）在我的模型上同样复现，只在 G-环 上出：「cap233 ret_a own_ret_a_uniform inode  … histories=18 … r_peak_max=64 … histories_r_pos=1」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:11`）；「cap6 ret_a own_ret_a_uniform inode  … histories=18 … r_peak_max=8 … histories_r_pos=8」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:53`）。两套互不共用代码的模型对上了这一形：「读侧对每类单元统一按版本谱系滤、记录带死亡 txg）：G-环 两种几何 9 格复活，峰值 30。」（`research/prompts/m3-leaf-death-r2-main-verification.md:20`）

什么现象会推翻 2.1：任何一段随机历史上候选一 / 二（Fsz）的 own 读法 r 或 m > 0（`expected/safety.txt` 里 `own_ret_a`、`own_ret_b`、`own_persist`、`own_persist_x`、`own_fsz` 任一行的 `r_peak_max` 或 `m_peak_max` 不为 0）；或 Tsz 的 `m_scans_outside_window` 不为 0。

### 2.2　unlink 途中的漏活（Tsz 的代价，量）

窗口 = 现行那一版队列非空，即意图那次到最后一批之间。我的历史 unlink 很密，窗内版本占全部版本的比例：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh window research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  tsz  versions=54303  in_unlink_window=26019  (47.9%)
cap6    tsz  versions=69522  in_unlink_window=31365  (45.1%)
ord233  tsz  versions=62457  in_unlink_window=28614  (45.8%)
rnd233  tsz  versions=58668  in_unlink_window=28242  (48.1%)
```

占比是历史的性质（这些历史里 unlink 占动作的 60%（cap6）到 87%（ord233）），不是候选的性质。每个窗口里 Tsz 漏的东西是确定的：那一个 nlink = 0 的 inode 记录，加那个对象当时还没截掉的数据单元与装它们的叶单元。
Fsz（墓碑挪到最后一批）窗内 r = m = 0，代价在 I-9.18：队列里的号在意图到最后一批之间没有谱系上的墓碑。窗内版本里这一句为假的比例（`expected/structure.txt`，cap233 的 fsz 行）：
```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh i918 research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  fsz  versions_i918_false=23460  of  26019  versions_in_unlink_window  (90.2%)
cap6    fsz  versions_i918_false=28563  of  31365  versions_in_unlink_window  (91.1%)
ord233  fsz  versions_i918_false=25314  of  28614  versions_in_unlink_window  (88.5%)
rnd233  fsz  versions_i918_false=26535  of  28242  versions_in_unlink_window  (94.0%)
```

### 2.3　每次删空 / 合并多写的单元（量）

「角色」= 一次发布里要重写的一个单元：叶容器、退役 / 墓碑容器、退役 / 墓碑树的根（照 `crates/` 的记账口径，每个角色每块盘一次单元写）。`crates/` 每次发布固定重写的那几样（inode 树根、分配记录树、记账树、映射树、树表）对所有候选相同，不数；
它们占的数在第二轮攻方腿的仓副本上量过，删空最右一片的最后一批 9 个角色：「drop_the_rightmost_leaf_234 publish=last_batch count=9」（`research/prompts/m3-leaf-death-r2-opus-output.md:168`）。
下表是 N1 新形借法下（M3 均分借）各候选相对今天（同一段历史、同一借法）多写的角色数；「events」= 删空 + 吸收 + 借的次数：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh cost research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  ret_a    pub=54303  events=252   extra_roles=414   (0.45%   of  today)  per1000pub=7.62    per_event=1.64   stub=0     ret_units=207
cap233  persist  pub=54303  events=654   extra_roles=447   (0.49%   of  today)  per1000pub=8.23    per_event=0.68   stub=621   ret_units=0
cap233  tsz      pub=54303  events=252   extra_roles=0     (0.00%   of  today)  per1000pub=0.00    per_event=0.00   stub=0     ret_units=0
cap233  fsz      pub=54303  events=252   extra_roles=642   (0.70%   of  today)  per1000pub=11.82   per_event=2.55   stub=0     ret_units=0
cap6    ret_a    pub=69522  events=4821  extra_roles=8910  (8.73%   of  today)  per1000pub=128.16  per_event=1.85   stub=0     ret_units=4455
cap6    persist  pub=69522  events=5364  extra_roles=2742  (2.69%   of  today)  per1000pub=39.44   per_event=0.51   stub=5103  ret_units=0
cap6    tsz      pub=69522  events=4821  extra_roles=0     (0.00%   of  today)  per1000pub=0.00    per_event=0.00   stub=0     ret_units=0
cap6    fsz      pub=69522  events=4821  extra_roles=810   (0.79%   of  today)  per1000pub=11.65   per_event=0.17   stub=0     ret_units=0
ord233  ret_a    pub=62457  events=726   extra_roles=186   (0.17%   of  today)  per1000pub=2.98    per_event=0.26   stub=0     ret_units=93
ord233  persist  pub=62457  events=384   extra_roles=-516  (-0.47%  of  today)  per1000pub=-8.26   per_event=-1.34  stub=78    ret_units=0
ord233  tsz      pub=62457  events=726   extra_roles=0     (0.00%   of  today)  per1000pub=0.00    per_event=0.00   stub=0     ret_units=0
ord233  fsz      pub=62457  events=726   extra_roles=846   (0.77%   of  today)  per1000pub=13.55   per_event=1.17   stub=0     ret_units=0
rnd233  ret_a    pub=58668  events=198   extra_roles=174   (0.17%   of  today)  per1000pub=2.97    per_event=0.88   stub=0     ret_units=87
rnd233  persist  pub=58668  events=204   extra_roles=-204  (-0.20%  of  today)  per1000pub=-3.48   per_event=-1.00  stub=105   ret_units=0
rnd233  tsz      pub=58668  events=198   extra_roles=0     (0.00%   of  today)  per1000pub=0.00    per_event=0.00   stub=0     ret_units=0
rnd233  fsz      pub=58668  events=198   extra_roles=762   (0.76%   of  today)  per1000pub=12.99   per_event=3.85   stub=0     ret_units=0
```

- 候选一：每条退役记录 +2 个角色（退役容器 + 退役树根），一个不多一个不少：「cap233 ret_a n1new  … units_retirement=207 … roles_vs_today=414」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:2`）（207 × 2 = 414）；cap6 是 4 455 × 2 = 8 910（「cap6 ret_a n1new  … units_retirement=4455 … roles_vs_today=8910」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:25`））。
  与第二轮攻方腿在仓副本上数的墓碑那一形对上：「意图那次往墓碑树开放容器追加一条，多的是 `TombstoneLeafContainer`、`TombstoneRoot` 两个角色。」（`research/prompts/m3-leaf-death-r2-opus-output.md:173`）
  占全部角色的 0.45%（cap233）、0.17%（ord233、rnd233）、8.73%（cap6：叶只装 6 条，结构性事件每千次发布 69 次）。**每条退役记录 2 个角色是常数，总量是历史的函数**（第 3 节把它按前提量开）。
  乙（RetB）与甲同数：回退那次取 R_old 不多写东西。
- 候选二：净增不是常数，符号随历史变：cap233 +0.49%、cap6 +2.69%、ord233 −0.47%、rnd233 −0.20%。两股力相反——
  省的一股：今天摘掉一片叶时右边各片左移、全部重写（`crates/singlefs-core/src/inode_tree.rs:308` 的 「containers.remove(index.position());」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:308`）），零条目叶不挪位置，右边各片不重写；
  费的一股：每次吸收都多写一片零条目叶，而且被吸收的右半在最右端时会被后面新建的 inode 复活再被吸收（零条目叶被当最右叶接着追加），吸收次数因此涨：「cap233 today no_borrow  … publishes=54303 … drops=45 … absorbs=102」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:17`） 对 「cap233 persist no_borrow  … publishes=54303 … drops=48 … absorbs=543 … stub_versions=591」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:20`）（M2 档，102 对 543）。
- 候选三 Tsz：0（盘上零新增；意图那次的墓碑每个候选一样已经在写，那 2 个角色在基线里）。Fsz：+0.7%–0.8%，是把同样的写挪到最后一批——最后一批发布在崩溃 / 丢根恢复之后被重做的次数多于意图（推的，没单独计）。

### 2.4　候选二的结构代价（量；和 3 节的并排连着看）

零条目叶一直留，指的是树里永远有一个没有记录的码 3 容器。我的写法（`src/model.rs` 的 `right_of`、`left_of`、`dismiss`、`create`）：吸收与借只在紧邻的非空叶之间做（Persist；PersistX 跨过零条目叶）；零条目叶当最右叶时新建的 inode 接着往里追加；吸收之后给零条目叶的分隔 key 取左邻最大 key + 1。同一批历史上：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh structure research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  today      leaves_avg=2.90   peak=5   I-9.13_false=0.0%   S3_false=29.6%  I-9.12_false=0.0%   misrouted=0.0%   sep_not_from_leaf=0.0%   stub_sep_without_room=0/0
cap233  persist    leaves_avg=3.68   peak=6   I-9.13_false=57.7%  S3_false=73.0%  I-9.12_false=2.5%   misrouted=0.0%   sep_not_from_leaf=57.7%  stub_sep_without_room=3/573
cap233  persist_x  leaves_avg=3.68   peak=6   I-9.13_false=57.7%  S3_false=73.0%  I-9.12_false=7.4%   misrouted=4.9%   sep_not_from_leaf=57.7%  stub_sep_without_room=6/615
cap6    today      leaves_avg=2.17   peak=10  I-9.13_false=0.0%   S3_false=3.0%   I-9.12_false=0.0%   misrouted=0.0%   sep_not_from_leaf=0.0%   stub_sep_without_room=0/0
cap6    persist    leaves_avg=13.29  peak=31  I-9.13_false=96.3%  S3_false=44.6%  I-9.12_false=38.9%  misrouted=0.0%   sep_not_from_leaf=96.2%  stub_sep_without_room=288/1695
cap6    persist_x  leaves_avg=13.03  peak=31  I-9.13_false=96.3%  S3_false=47.0%  I-9.12_false=73.3%  misrouted=23.6%  sep_not_from_leaf=96.2%  stub_sep_without_room=420/2292
ord233  today      leaves_avg=4.37   peak=7   I-9.13_false=0.0%   S3_false=44.5%  I-9.12_false=0.0%   misrouted=0.0%   sep_not_from_leaf=0.0%   stub_sep_without_room=0/0
ord233  persist    leaves_avg=6.87   peak=8   I-9.13_false=79.1%  S3_false=67.2%  I-9.12_false=44.1%  misrouted=0.0%   sep_not_from_leaf=79.1%  stub_sep_without_room=48/48
ord233  persist_x  leaves_avg=6.87   peak=8   I-9.13_false=79.1%  S3_false=86.6%  I-9.12_false=79.1%  misrouted=71.8%  sep_not_from_leaf=79.1%  stub_sep_without_room=93/117
rnd233  today      leaves_avg=9.93   peak=11  I-9.13_false=0.0%   S3_false=33.8%  I-9.12_false=0.0%   misrouted=0.0%   sep_not_from_leaf=0.0%   stub_sep_without_room=0/0
rnd233  persist    leaves_avg=11.11  peak=12  I-9.13_false=31.5%  S3_false=35.9%  I-9.12_false=15.7%  misrouted=0.0%   sep_not_from_leaf=31.5%  stub_sep_without_room=12/105
rnd233  persist_x  leaves_avg=11.11  peak=12  I-9.13_false=31.5%  S3_false=35.9%  I-9.12_false=15.9%  misrouted=1.1%   sep_not_from_leaf=31.5%  stub_sep_without_room=12/105
```

列的意思：`I-9.13_false` = 树里有零条目叶的版本占比（I-9.13 「任一打包记录类型 2 的容器记录数 ≥ 1」（`.claude/kb/invariants.md:301`））；`S3_false` = 按字面把零条目叶的容器号代进 I-9.4 第三句为假的版本占比；
`I-9.12_false` = 分隔 key 不严格递增 / 不落在孩子区间之外的版本占比；`misrouted` = 按分隔 key 查某条记录会走到别的叶的版本占比；`sep_not_from_leaf` = 存着的分隔 key 不是「容器号」也不是「孩子这一版的最小 key」的版本占比（零条目叶没有最小 key）；
`stub_sep_without_room` = 吸收之后给零条目叶放分隔 key 时，(左邻最大 key, 右邻分隔 key) 这个开区间里没有整数的次数 / 吸收次数。

- 零条目叶的版本占 31.5%（rnd233）到 96.3%（cap6）：「cap233 persist n1new  … versions=54303 … versions_i913_false=31353」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:4`）；「cap6 persist n1new  … versions=69522 … versions_i913_false=66960」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:27`）；「ord233 persist n1new  … versions=62457 … versions_i913_false=49425」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:68`）。叶数：平均 2.90 → 3.68（cap233）、2.17 → 13.29（cap6）、4.37 → 6.87（ord233）、9.93 → 11.11（rnd233），「叶只增不减」（第二轮辩方写的 60 / 96 片）在随机历史上同向。
- 吸收之后零条目叶要的分隔 key 常常不存在：编号稠密时右邻的分隔 key 恰好是左邻最大 key + 1，严格递增的 key 里夹不进一个整数。cap233 「cap233 persist n1new  … stub_sep_raised_with_room=570 … stub_sep_without_room=3」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:4`）；cap6 「cap6 persist n1new  … stub_sep_without_room=288」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:27`）（1 695 次吸收里 288 次）；ord233（从小到大删，吸收都在最左端）「ord233 persist n1new  … stub_sep_without_room=48」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:68`），48 次吸收一次都放不下。
  放不下的版本此后一直红 I-9.12（零条目叶永不摘），所以 `I-9.12_false` 的占比 2.5%–44% 是「放不下」那些事件的存活期。
- 零条目叶的分隔 key 不是叶的函数：它没有记录，容器头又不带 key 区间。D8 对内部节点的写法是「派生态，丢了从叶重建」（`.claude/kb/decisions/08-核心索引结构.md:160`），对它不成立；`sep_not_from_leaf` 就是这个比例。
- 真 checker 看不见零条目叶的分隔 key 放错：仓副本上手改的镜像（第 6 节细表），零条目叶留在叶序里、分隔 key 留着 234 而左邻已吸收到 250，今天的 checker 只红 I-9.13；放宽 I-9.13 之后这一形没有任何判据拦（推的，镜像上量到的是「只红 I-9.13」）。
  PersistX（合并跨过零条目叶）更糟：`misrouted` 1.1%–71.8%，因为左邻吸收的是隔着零条目叶的右邻，那片零条目叶的分隔 key 夹在两者之间。

### 2.5　2 节的判（K1 前半）

1. 复活与误判死亡：候选一两个改法、候选二、Fsz 在这批随机历史上 r = m = 0；Tsz r = 0、m 只在 unlink 途中（窗内版本约 45%–48%，每个窗口漏一条 inode 记录加那个对象还没截掉的数据单元）。
2. 多写：候选一每条退役记录 2 个角色，总量 0.17%–0.45%（叶容量 233）或 8.7%（叶容量 6）；候选二净增 −0.5% 到 +2.7%，符号随历史变；Tsz 0。多写不分得出候选，分得出的是 3 与 4。
3. 候选二在用户已定的 M3（吸收 / 借，「用户已定的 M3 照这一形实现」（`research/prompts/m3-leaf-death-r1-main-verification.md:56`））下不是「改 I-9.13 一条」的事：吸收留下的零条目叶在稠密编号下没有合法分隔 key，加上它的分隔 key 不是叶的函数。要么 M3 只借不吸收，要么容器头另存分隔 key（格式冻结层的问题，我没有判是哪一层），要么放宽 I-9.12 对零条目叶的严格递增，要么给内部条目的类型段加一个值标出零条目叶、查找跳过它（类型段今天只许 0 和 2：「2 ⇒ 子节点是码 3 容器（读两个 16 KiB 槽）；其余值判条目损坏」（`.claude/kb/decisions/08-核心索引结构.md:164`），多一个值也是改格式）。第二轮辩方把候选二拉回候选集时列了四项账，没有这一项：「合并规则要加一条例外：`pair_fits_in_one_container` 对「零条目叶 + 任意一片」恒为真」（`research/prompts/m3-leaf-death-r2-sonnet-output.md:79`）
4. 推翻 2.3 / 2.4 的现象：找到一种不改容器头、不放宽 I-9.12 的零条目叶分隔 key 取法，在 `stub_sep_without_room` 为正的那些历史上 `I-9.12_false` 回到 0；或候选一的每条退役记录多于 2 个角色（比如退役树根不随每条记录重写）。


## 3　前两轮三套副本模型的数能不能并排（K1 末句）

做法：把前两轮报出来的同名量逐个摆出来，再用我的模型把「前提」一个一个换开，看差多少是前提的、差多少是别的。我的模型是 B（第二轮攻方腿）的另写版本，加了候选二 / 三与同一段历史的做法，所以它在 B 的前提上能直接对上 B 的数（见第三行 N1 旧形）。

### 3.1　同名量对照（数全是算出来的，出处在表下）

| 量 | A 第一轮攻方 | B 第二轮攻方 | C 第二轮辩方 | 本腿（范围） | 并排？ |
|---|---|---|---|---|---|
| 候选一的复活 / 误判死亡 | 0 / 0（E177 51 段、合并 64 段） | 0 / 0（576 段，含崩溃、丢根、回退） | — | 0 / 0（全部段里候选一的那些） | 能：同向、都是「没打中」 |
| 退役记录每千次发布 | E177 3.15；合并 M3 均分 1.12 | 不借：cap233 3.98（分子分母都含崩溃支线里的）；cap6 51.2 | 同 A | 不借 / N1 新形：cap233 2.65 / 3.81；cap6 64.3 / 64.1；ord233 1.49；rnd233 1.48；前提族 1.40–7.00 | 叶容量 233 档同量级（1–8），叶容量 6 档同量级（50–65）；档内差 ≤ 7 倍来自前提（3.2） |
| N1 旧形（借时抬、之后不跟着变）下，分隔 key 两种重建都还原不出的版本占比 | — | cap233 8.9%；cap6 1.4% | — | cap233 9.7%；cap6 1.0% | 能：两套互不共用代码的模型差 1 个百分点以内 |
| N2 相对不借多的退役记录 | — | cap233 +61%；cap6 +20% | — | cap233 +69%；cap6 +5%；ord233 +645%；rnd233 +110% | 方向能并排（N2 每次借 +1 条）；百分比不能（分母是不借时的退役数，借得多不多是前提） |
| 借完之后 I-9.4 第三句为假的版本占比（M3 均分，N1 新形） | 15.4%（M3 均分）、13.7%（M3 刚好够） | — | — | cap233 29.6%；cap6 3.0%；ord233 44.5%；rnd233 33.8% | 不能：3.0%–44.5%，随删序（从小到大删 44.5%）与叶容量（6 条 3.0%）动 |
| 候选二：零条目版本每千次发布 / 零条目叶在的版本占比 | E177 3.15 / 7.2% | — | 合并 M3 均分 1.15 / 11.0% | 0–11.2 / 0%–96%（前提族与长度族、主表） | 不能：同一个模型内就从 0% 跨到 96%（3.2） |

出处（逐字，行号现取；本腿的行在 `expected/` 里）：
- 「ring M1_dropped runs=51  … publishes=20003 … retirements=63」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-e177.txt:1`）
- 「ring M3half_dropped runs=64  … publishes=110257 … retirements=123 … versions_with_third_false=17000」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-merges.txt:4`）
- 「ring M3just_dropped runs=64  … publishes=110165 … versions_with_third_false=15091」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-merges.txt:5`）
- 「ring M1_zero_persist runs=51  … publishes=20003 … zero_versions=63 … versions_with_empty_container=1439」（`research/prompts/m3-leaf-death-r1-opus-model/expected/counts-e177.txt:3`）
- 「ring M3half_zero_persist runs=64  … publishes=110365 … zero_versions=127 … versions_with_an_empty_container=12161」（`research/prompts/m3-leaf-death-r2-sonnet-model/expected/counts-merges.txt:7`）
- 「cap233 no_borrow retirement_records=1469」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:93`）
- 「cap233 no_borrow publishes=368856」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:90`）
- 「cap233 no_borrow final_txg=285611」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:84`）
- 「cap233 n1 sep_versions_stored_matches_neither_rebuild_rule=32944」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:30`）
- 「cap233 n1 versions_judged=371401」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:36`）
- 「cap233 n2 retirement_records=2369」（`research/prompts/m3-leaf-death-r2-opus-model/expected/counts.txt:56`）
- 「cap233 today n1raise  … versions=54303 … versions_sep_matches_neither_rule=5268」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:8`）
- 「cap6 today n1raise  … versions=69522 … versions_sep_matches_neither_rule=723」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:31`）
- A 的候选一：「ring M1_dropped C1 inode histories=51 r_peak_max=0 r_out_peak_max=0 histories_r_pos=0 m_peak_max=0 histories_m_pos=0」（`research/prompts/m3-leaf-death-r1-opus-model/expected/agg-e177.txt:5`）
- B 的候选一：「carried C1t inode histories=288 cells_r_pos=0 cells_m_pos=0 r_max=0 m_max=0」（`research/prompts/m3-leaf-death-r2-opus-output.md:106`）

### 3.2　把前提一个一个换开（量）

前提族（`src/main.rs` 里 `prem_` 打头的形状）：同一个形状（叶容量 233、起始 1400 个空 inode、2000 步），一次只换一个前提——谁有数据（所有 inode / 只有 1 号）、怎么挑删（盯着一片叶 / 随机 / 从小到大）、有没有故障（崩溃、丢根、管理员回退全开 / 全关，全关就是 A、C 的「干净发布之后扫」）。
每格 6 个种子，ring_lowest 几何，N1 新形借法；「零条目叶在的版本占比」是候选二（Persist）的：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh prem research/prompts/m3-leaf-death-r3-sonnet-model/expected
前提(谁有数据_怎么删_故障) 每千发布: 删空 吸收 借 退役记录 | persist 零条目版本 零条目叶在的版本占比
data_all_del_ascending_faults_off     0.00  1.77  9.55  1.77 |   2.31  87.4%
data_all_del_ascending_faults_on      0.00  1.40  9.24  1.40 |   1.26  79.8%
data_all_del_focus_faults_off         0.08  1.79  1.79  1.87 |   6.19  86.4%
data_all_del_focus_faults_on          4.08  1.78  1.74  5.78 |   9.02  73.9%
data_all_del_random_faults_off        0.08  2.61  2.27  2.69 |   5.25  71.0%
data_all_del_random_faults_on         4.26  1.81  1.73  5.90 |   9.88  60.6%
data_one_del_ascending_faults_off     0.00  1.79 10.18  1.79 |   2.72  86.5%
data_one_del_ascending_faults_on      0.00  1.54 10.09  1.49 |   1.25  79.1%
data_one_del_focus_faults_off         0.27  1.83  1.83  2.10 |   6.07  87.9%
data_one_del_focus_faults_on          5.28  1.66  1.48  6.54 |  11.07  72.8%
data_one_del_random_faults_off        0.23  3.02  2.43  3.25 |   7.60  70.0%
data_one_del_random_faults_on         5.29  2.14  1.84  7.00 |  11.15  59.3%
```

历史长度族（`len_` 打头，E177 式前提：只有 1 号有数据；A 的 E177 每段平均 392 次发布、合并段 1 723 次，我的主表每段 1159（cap6）到 4889（rnd233）次，所以长度也是一个前提）：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh len research/prompts/m3-leaf-death-r3-sonnet-model/expected
删序_动作数            发布/段  退役记录/千  零条目版本/千  零条目叶在的版本占比
del_ascending_100          179        0.00          0.00             0.0%
del_random_100             199        7.54          9.21            49.3%
del_ascending_300          516        0.65          0.65             1.5%
del_random_300             581        7.74         10.61            42.6%
del_ascending_1000        1716        1.46          0.87            65.0%
del_random_1000           1936        6.37          9.55            36.9%
```

读这两张表：
- **有没有故障**：退役记录每千次发布翻 2–3 倍（随机 / 盯着的删：2.1–3.3 → 5.8–7.0），因为回退与丢根之后同一批删空 / 吸收要重做；删空次数从 0.1–0.3 涨到 4–5。A、C 没有故障，B 有，这是 B 的 3.98 对 A 的 1.12–3.15 的主要来源之一。
- **怎么挑删**：从小到大删，借的次数每千次发布约 10，盯着的 / 随机删约 1.5–2.4（4–6 倍）；退役记录几乎不动（1.4–1.8）。第三句为假的占比跟着借走。
- **谁有数据**：对这些频度影响不大（同一删序、同一故障下 data_one 比 data_all 高 0%–21%，方向是分母：只有 1 号有数据时没有数据批，发布少）。所以 A 的「只有 1 号有数据」对退役 / 零条目 / 第三句这几个量不是主要前提；它影响的是 Tsz 在窗内漏多少数据单元（2.1 的 units m 峰值），不影响次数。
- **每段历史多长**：候选二的「零条目叶在的版本占比」，从小到大删时随历史变长涨：0%（100 步，179 次发布）、1.5%（300 步）、65%（1 000 步）、79%（2 000 步，前提族 data_one_del_ascending_faults_on）；随机删没有单调趋势（49%、43%、37%、59%）。同一模型内跨 0%–96%，A、C 的 7.2% / 11.0% 落在这个范围里，但它不是一个能从一个模型搬到另一个模型的量。
- **分母**：B 把崩溃支线里的发布与退役记录都加了进去（分母比主线 final_txg 大 1.29 倍），A / C / D / 本腿只数主线；B 的 3.98 因此不是主线口径，我从 B 的产物里拆不开（支线里的退役数没单列）。

### 3.3　判

1. **能并排的**：候选一 r = m = 0（四套模型同向）、Tsz 的 m 只在窗内、N1 旧形的「还原不出」比例（B 8.9% 对我的 9.7%，cap6 1.4% 对 1.0%）、N2 每次借 +1 条退役记录的方向、每条退役记录 2 个角色（B 在仓副本上数的与我模型的 207 × 2 = 414 对上）。这几项我用的都是比例或常数，不依赖前提。
2. **不能并排的**：任何「每千次发布」或「占版本多少」的绝对数——退役记录（每千次发布 1.1 到 7.0）、借的次数（删序 5 倍）、第三句为假的占比（3.0%–44.5%）、零条目叶在的版本占比（0%–96%）。差在三个前提：**有没有故障**（回退与丢根让结构性事件重做）、**怎么挑删**（从小到大删让借多 5 倍）、**每段历史多长**（零条目叶存活得越久占比越高）；加一个**计数的分母**（B 含崩溃支线）。「谁有数据」不是前提差。
3. 所以交用户的代价表（第 5 节）一律写「每条事件多少角色」这种常数，加上一个按前提族给的范围，不写某一套模型的绝对次数。

什么现象会推翻 3 节：前提族里把「有没有故障」「怎么挑删」「谁有数据」三项换开之后，同一个量的跨度不到 2 倍（目前退役记录 1.40–7.00、借 1.48–10.18、零条目叶占比 59.3%–87.9%）；或 A / C 的 7.2% / 11.0% 在我的模型上用 A 的历史长度能复现到 ±3 个百分点（我没有复现 A 的 51 段 E177 历史，只换了长度，所以这一项「落在范围里」是推的，不是复现）。


## 4　三处没被看过的地方（K1 末条）

### 4.1　克隆头共享叶（量，有界穷举）

kb 已定的规则（C113 定案 P6，`crates/` 里没有克隆，没有写者）：「新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号)——保留容器号、只换出生树与出生代，同时给旧身份写退役记录」（`.claude/kb/decisions/08-核心索引结构.md:171`）。
也就是说**克隆头动一片共享叶，今天已定的规则就要写一条退役记录**，不分候选。

模型（`src/clone.rs`，与主模型互不共用代码）：叶容量 2；origin（头 0）先建 5 个 inode，叶 [1,2] [3,4] [5]；fork 在这之后；然后两个头各自 `create` / `unlink 最小号` / `unlink 最大号`，
长度 1 到 6 的**全部**序列（6 种操作，共 55 986 条，其中能执行的 55 858 条），每走一步对两个头各重建一遍视图、与各自的真值比。规则：每个头自己一份退役记录与墓碑记录；读克隆头时只收 origin 在 fork 之前写的版本，origin 的退役 / 墓碑记录只认死亡 txg ≤ fork 的。
三个消融各去掉一条读侧规则：GlobalRetire（退役记录不分头）、GlobalTomb（墓碑不分头）、NoForkBound（读克隆头时不按 fork 截 origin 的版本）。

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh clone research/prompts/m3-leaf-death-r3-sonnet-model/expected
Persist  GlobalRetire  sequences=55858  r(origin/clone)=0/0          m(origin/clone)=46075/0      seq_with_r=0      seq_with_m_out=46075
Persist  GlobalTomb    sequences=55858  r(origin/clone)=0/0          m(origin/clone)=0/0          seq_with_r=0      seq_with_m_out=0
Persist  NoForkBound   sequences=55858  r(origin/clone)=0/9283       m(origin/clone)=0/21930      seq_with_r=9283   seq_with_m_out=21930
Persist  None          sequences=55858  r(origin/clone)=0/0          m(origin/clone)=0/0          seq_with_r=0      seq_with_m_out=0
Ret      GlobalRetire  sequences=55858  r(origin/clone)=0/0          m(origin/clone)=42624/10835  seq_with_r=0      seq_with_m_out=48072
Ret      GlobalTomb    sequences=55858  r(origin/clone)=0/0          m(origin/clone)=0/0          seq_with_r=0      seq_with_m_out=0
Ret      NoForkBound   sequences=55858  r(origin/clone)=0/25316      m(origin/clone)=0/20938      seq_with_r=25316  seq_with_m_out=20938
Ret      None          sequences=55858  r(origin/clone)=0/0          m(origin/clone)=0/0          seq_with_r=0      seq_with_m_out=0
Today    GlobalRetire  sequences=55858  r(origin/clone)=13359/37180  m(origin/clone)=42624/1397   seq_with_r=43977  seq_with_m_out=43379
Today    GlobalTomb    sequences=55858  r(origin/clone)=37180/37180  m(origin/clone)=0/0          seq_with_r=51238  seq_with_m_out=0
Today    NoForkBound   sequences=55858  r(origin/clone)=37180/48095  m(origin/clone)=0/20938      seq_with_r=52858  seq_with_m_out=20938
Today    None          sequences=55858  r(origin/clone)=37180/37180  m(origin/clone)=0/0          seq_with_r=51238  seq_with_m_out=0
Tsz      GlobalRetire  sequences=55858  r(origin/clone)=0/0          m(origin/clone)=42624/1397   seq_with_r=0      seq_with_m_out=43379
Tsz      GlobalTomb    sequences=55858  r(origin/clone)=0/0          m(origin/clone)=31144/31144  seq_with_r=0      seq_with_m_out=49852
Tsz      NoForkBound   sequences=55858  r(origin/clone)=0/25126      m(origin/clone)=0/20938      seq_with_r=25126  seq_with_m_out=20938
Tsz      None          sequences=55858  r(origin/clone)=0/0          m(origin/clone)=0/0          seq_with_r=0      seq_with_m_out=0
```

- 规则齐全时（ablation=None）：候选一、候选二、候选三 r = m = 0，55 858 条序列一条没打中（模型只在每个操作的意图 + 最后一批都走完之后判，没判窗内，所以 Tsz 的窗内 m 不在这张表里）；今天（不写死亡的读法 S）r 在两个头上各 37 180 次扫描：「R3 name=clone cand=Today ablation=None … sequences=55858 … r_scans_origin=37180 … r_scans_clone=37180 … sequences_with_r=51238」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:12`）。
- GlobalRetire 把**每个**候选都打红，包括今天（它也有 P6 的退役记录）：「R3 name=clone cand=Today ablation=GlobalRetire … m_scans_origin=42624 … m_scans_clone=1397」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:9`）；「R3 name=clone cand=Persist ablation=GlobalRetire … m_scans_origin=46075 … m_scans_clone=0」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:1`）。**退役记录按头分是 P6 已经逼出来的，不是候选一的新账。**
- NoForkBound 同样打红每个候选：「R3 name=clone cand=Persist ablation=NoForkBound … r_scans_clone=9283 … m_scans_clone=21930」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:3`）。按 fork 截版本是克隆头读法的底，三个候选共用。
- GlobalTomb 只打红 Tsz（别的候选不读墓碑）：「R3 name=clone cand=Tsz ablation=GlobalTomb … m_scans_origin=31144 … m_scans_clone=31144」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:14`）。**Tsz 比另两个多一条读侧规则：墓碑只对记它的那个头（加 fork 之前继承来的）生效。**墓碑树本来就按头（I-9.27：「种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID」（`.claude/kb/invariants.md:315`）），所以这条规则有载体，只是读侧要写。
- 候选二在克隆头上并不省退役树：共享叶不能原地写零条目版本（会改 origin 的身份），要先 P6 重生成本头的身份再写零条目，P6 的退役记录照写，而且比别的候选多：「R3 name=clone cand=Persist ablation=None … writes_at_full_depth=[create=93312,intent=185856,last=191480,p6_retire=85100]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:4`） 对 「R3 name=clone cand=Tsz ablation=None … writes_at_full_depth=[create=93312,intent=185856,last=103616,merge_retire=5260,p6_retire=83974]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/clone.txt:16`）（p6_retire 85 100 对 83 974，多出来的 1 126 条推的是给共享叶写零条目时的）。
- 这一格模型没有的：快照、克隆的克隆、崩溃与回退、数据单元、分配记录的跨头合并。读码推的：一旦有克隆头，退役记录的载体就是 P6 欠的，候选一多一棵树的账在那一天摊平；单头的第一版要不要提前建，是另一个问题（不在 K1 的数里）。

什么现象会推翻 4.1：有界穷举里出现一条序列使规则齐全的 Ret / Persist / Tsz 在某个头上 r 或 m > 0（`expected/clone.txt` 里 ablation=None 的前三个候选任一行 `r_scans_*` 或 `m_scans_*` 不为 0）。

### 4.2　副本坏一份（w = 2）

先说我量不到的：我的模型一个单元就是一个槽，没有两份副本，一份坏另一份读得出等于没坏，不单列格——**复核不了**，原因是模型没建副本。
kb 里的依据：扫描重建的副本归并要求同类身份段的多份可读单元载荷相同（「定案 P3：副本归并要求」（`.claude/kb/decisions/18-块里携带什么信息.md:179`）），E104 的全规则臂含 w = 2 一档全部 0（「判据 1：全规则臂在 24 个种子 × 十八个世界（靶子世界全部按第八版纪律造，混合世界含随机回退与 w = 2 一档）全部 0。」（`.claude/kb/experiments/104-扫描重建的现行版本判定.md:88`））。
第二轮攻方腿也把这一格写成「没量」：「单份坏、另一份读得出时选版本不变，模型没建副本（限度）。」（`research/prompts/m3-leaf-death-r2-opus-output.md:161`）
所以「一份坏」这一格我只能写推的：另一份读得出，视图不变，三个候选一样。

我量了更狠的一格——**一个证据单元的两份都坏**（模型里这一单元读不出）。取各候选自己最要紧的那个单元（候选一：记录最多的退役容器；候选二：最新写的零条目叶；候选三：记录最多的墓碑容器），每 5 个版本丢一次、重判；inode 名字空间的 r / m 峰值：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh loss research/prompts/m3-leaf-death-r3-sonnet-model/expected
cap233  loss_ret_a      histories=72   r_max=124  r_hist=72   m_max=0  m_hist=0
cap233  loss_ret_b      histories=54   r_max=127  r_hist=54   m_max=0  m_hist=0
cap233  loss_persist    histories=54   r_max=120  r_hist=54   m_max=0  m_hist=0
cap233  loss_persist_x  histories=54   r_max=84   r_hist=54   m_max=0  m_hist=0
cap233  loss_tsz        histories=54   r_max=43   r_hist=45   m_max=1  m_hist=54
cap233  loss_fsz        histories=54   r_max=41   r_hist=54   m_max=0  m_hist=0
cap6    loss_ret_a      histories=240  r_max=11   r_hist=240  m_max=0  m_hist=0
cap6    loss_ret_b      histories=180  r_max=12   r_hist=180  m_max=0  m_hist=0
cap6    loss_persist    histories=180  r_max=6    r_hist=180  m_max=0  m_hist=0
cap6    loss_persist_x  histories=180  r_max=6    r_hist=180  m_max=0  m_hist=0
cap6    loss_tsz        histories=180  r_max=2    r_hist=180  m_max=1  m_hist=180
cap6    loss_fsz        histories=180  r_max=1    r_hist=180  m_max=0  m_hist=0
ord233  loss_ret_a      histories=45   r_max=233  r_hist=32   m_max=0  m_hist=0
ord233  loss_ret_b      histories=45   r_max=233  r_hist=32   m_max=0  m_hist=0
ord233  loss_persist    histories=45   r_max=233  r_hist=45   m_max=0  m_hist=0
ord233  loss_persist_x  histories=45   r_max=233  r_hist=26   m_max=0  m_hist=0
ord233  loss_tsz        histories=54   r_max=1    r_hist=29   m_max=1  m_hist=54
ord233  loss_fsz        histories=54   r_max=1    r_hist=25   m_max=0  m_hist=0
rnd233  loss_ret_a      histories=36   r_max=71   r_hist=36   m_max=0  m_hist=0
rnd233  loss_ret_b      histories=36   r_max=71   r_hist=36   m_max=0  m_hist=0
rnd233  loss_persist    histories=36   r_max=74   r_hist=36   m_max=0  m_hist=0
rnd233  loss_persist_x  histories=36   r_max=55   r_hist=36   m_max=0  m_hist=0
rnd233  loss_tsz        histories=36   r_max=24   r_hist=5    m_max=1  m_hist=36
rnd233  loss_fsz        histories=36   r_max=11   r_hist=11   m_max=0  m_hist=0
```

- 三个候选都有单点：丢一个证据单元，退回到那一组还读得出的最老版本，装着已死记录的整片叶回来。候选一、二回来的是一片叶里没删完的旧记录（cap233 的 124 / 120，ord233 的 233 = 一片叶装满）；候选三回来的是被墓碑点名的那些对象还留在可读旧版本里的记录（cap233 的 43、ord233 的 1），小 3 倍（cap233）到 200 倍以上（ord233）。
  Fsz 与 Tsz 同一级（41、1）。
- 候选之间的差别不在有没有单点，在一个单元装多少死亡：零条目叶 1 个、退役容器 959 条、墓碑容器 815 条（D18 已定项 24 的容量，主模型的 `ret_cap`、`tomb_cap`）。但装得多不等于丢了更糟：退役 / 墓碑容器的版本是累积的，丢最新版本退回上一版只丢最后几条（推的）；丢了更糟的是退回的那个旧版本里还留着多少已死的记录，这由叶的删除历史定，不由证据容器的容量定。
- 这个比较的限度：只丢一个单元、取的是各候选自己的证据单元，没有丢普通叶（普通叶丢了也退回旧版本，三个候选一样）；没有两份副本里一份是陈旧内容的格（那是 C113 P3 的 `no_replica_merge` 臂，E104 量过）。

### 4.3　清扫删实例表行

代码（冻结树，读码）：清扫的次序是先判、后抹——「一个实例的根全部轮出根环之后，抹掉池里按它那一行判未发布的单元头，再发一次专用的清扫发布」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:2`）；有一次读失败就「这一轮一行都不删、一个头都不抹」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:8`）。
它认单元只看码 1 / 2 / 3 与头校验和、不看打包记录类型（「UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/instance_table_sweep.rs:195`）），所以退役容器（类型 3）、墓碑容器（类型 1）、零条目叶（类型 2）三类证据单元清扫一视同仁，没有哪个候选的单元类逃出它的射程。

模型（`src/model.rs` 的 `sweep_instance_rows`，cap6 形状，每格 24 段）：每个动作开头，对根都已轮出的实例，抹掉按它那一行判未发布的单元再删行。两种：清扫（sweep_perfect）和清扫漏抹证据单元却照样删行（sweep_misses_evidence，故障注入，不是设计里的行为——代码注释写的是读失败就一行都不删）：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh sweepfam research/prompts/m3-leaf-death-r3-sonnet-model/expected
清扫方式                    候选     崩溃点 在飞:退役/墓碑/零条目叶 清扫抹/漏 | inode r_max m_max | units r_max (r>0 的段/24)
sweep_perfect_cap6          today     868        0/232/  0           1233/  0   |   -   -       |   -
sweep_perfect_cap6          ret_a     868       52/232/  0           1275/  0   |   0   0       |   0 (0)
sweep_perfect_cap6          persist   868        0/232/ 54           1270/  0   |   0   0       |   0 (0)
sweep_perfect_cap6          tsz       868        0/232/  0           1233/  0   |   0   1       |   0 (0)
sweep_misses_evidence_cap6  today     868        0/232/  0           1023/210   |   -   -       |   -
sweep_misses_evidence_cap6  ret_a     868       52/232/  0           1015/260   |   0   0       |  10 (16)
sweep_misses_evidence_cap6  persist   868        0/232/ 54           1003/267   |   0   0       |   9 (16)
sweep_misses_evidence_cap6  tsz       868        0/232/  0           1023/210   |   0   3       |   9 (16)
```

- 清扫齐全时：所有候选 r = m = 0（Tsz 仍只有窗内的 m 1）；清扫抹的单元数与今天同量级（1 233 对 1 270–1 275），清扫本身不分辨候选。
- 漏抹证据单元时：**只有 Tsz 在窗外出现误判死亡**：「sweep_misses_evidence_cap6 tsz own_tsz inode  … histories=24 … m_peak_max=3 … m_scans_in_window=14596 … m_scans_outside_window=592」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/safety.txt:530`）；候选一、二的 inode 名字空间仍是 0，只有 units 名字空间多出一个孤儿单元（r 9–10）。
  机制（推的，对着产物行读）：Tsz 的证据（墓碑）在意图那次发布里写，崩在意图那次的发布被抛弃，对象没被 unlink、仍活着，孤儿墓碑却因为行被删而变成「已发布」，永久杀掉一个活对象；候选一、二的证据在最后一批发布里写，那次发布被抛弃时意图已经发布、队列还在，重做会写出同一条证据，孤儿与重做出来的重合。Fsz 同候选一（证据也在最后一批）。
- 崩溃点上带着证据单元在飞的比例：Tsz（意图那次写墓碑）「sweep_perfect_cap6 tsz  … crash_points=868 … tomb_in_flight=232」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/sweep.txt:8`），候选一 「sweep_perfect_cap6 ret_a  … ret_in_flight=52」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/sweep.txt:6`），候选二 「sweep_perfect_cap6 persist  … zero_entry_leaf_in_flight=54」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/sweep.txt:7`）（868 个崩溃点里 232 / 52 / 54，约 27% / 6% / 6%）。今天基线里每个候选都在写意图墓碑，只有 Tsz 读它。
- 所以清扫这一格对 Tsz 的要求最高：P2（已发布谓词）和清扫都不能漏。去掉 P2 的消融对候选一、三一样红（2.1 节），清扫漏抹只红候选三。

什么现象会推翻 4.3：清扫漏抹的变体里候选一或二出现 inode 名字空间的窗外 m（`expected/safety.txt` 里 `sweep_misses_evidence_cap6 ret_a|persist own_* inode` 的 `m_scans_outside_window` 不为 0），或清扫齐全时任一候选 r、m 窗外不为 0。

### 4.4　4 节的判

1. 克隆头共享叶：三个候选在规则齐全时都过（有界穷举 55 858 条序列）；各自多要的读侧规则：候选一 0（P6 已要求退役记录按头分）、候选二 0（但克隆头上写零条目要 P6 重生，比别的候选多写退役记录）、候选三 1（墓碑按头生效）。
2. 副本坏一份：一份坏另一份好 = 没坏（读 kb 与 E104，我的模型没建副本，复核不了）；两份都坏三个候选都有单点，候选一、二回来整片叶，候选三回来被点名的记录。
3. 清扫删行：清扫本身不分辨候选；清扫漏抹证据单元时只有候选三（证据在意图那次写）在窗外误判死亡。


## 5　交用户的代价表（K1：三个候选并排）

数的出处都在 2–4 节；这里只摆结论列。范围 = 四个形状（叶容量 233 的三种历史与叶容量 6）；「前提族」指 3.2 节换前提之后的跨度。

| | 候选一 甲（RetA） | 候选一 乙（RetB） | 候选二（Persist） | 候选三 Tsz | 候选三 Fsz |
|---|---|---|---|---|---|
| 复活 r（随机历史，含崩溃 / 丢根 / 回退） | 0 | 0 | 0 | 0 | 0 |
| 误判死亡 m 途外 | 0 | 0 | 0 | 0（清扫齐全时） | 0 |
| unlink 途中漏活 | 无 | 无 | 无 | **每个窗口漏 1 条 inode 记录 + 该对象未截的数据单元**（窗内版本 45%–48%；units 峰值 11–17） | 无，但队列里的号在窗内没有墓碑（I-9.18 窗内 88%–94% 的版本为假） |
| 每条结构性事件多写的角色 | 2（退役容器 + 退役树根），常数 | 2 | −1.3 … +0.7，符号随历史 | 0 | 结构性事件本身 0；墓碑的写挪到最后一批，恢复重做使总数 +0.7%–0.8%（推的） |
| 占全部角色 | 0.17%–0.45%（叶 233）；8.7%（叶 6） | 同 | −0.5% … +2.7% | 0 | +0.7%–0.8% |
| 盘上新增 | 退役树（新种类码，树表 11 → 12 条）+ 记录加死亡 txg | 同（B 说乙下死亡 txg 可以不要，我没单测） | 零条目叶永不摘（叶数 +12% … ×6） | 零 | 零 |
| 与用户已定的 M3（吸收 / 借）的相容 | 相容 | 相容 | **吸收之后零条目叶无处放分隔 key（从小到大删 100%，别的历史 0.5%–17%，是吸收次数里放不下的占比）；I-9.12 / I-9.4 第三句对它失明** | 相容 | 相容 |
| 克隆头上多要的读侧规则 | 0（P6 已要求退役记录按头分） | 0 | 0，但共享叶上写零条目要先 P6 重生，多写退役记录 | 1：墓碑只对记它的那个头生效 | 同 Tsz |
| 清扫漏抹证据单元时 | 只多孤儿单元，记录对 | 同 | 同 | **窗外误判死亡**（证据在意图那次写） | 同候选一 |
| 单点证据丢失（两份副本都坏）r 峰值 cap233 / ord233 | 124 / 233 | 127 / 233 | 120 / 233 | 43 / 1 | 41 / 1 |
| 管理员回退取 R_old 的树要改 | 不改 D23 已定项 14（退役树随现行，要 C615 读侧只对取 R_old 的树判版本谱系） | **改 D23 已定项 14**：退役树也取 R_old，与墓碑树方向相反 | 不涉及 | 不涉及 | 不涉及 |

读法：候选一与 Fsz 在这些量上不出 r / m；Tsz 的代价集中在「窗内漏活」与「清扫 / P2 不能漏」；候选二在 M3 下的结构冲突是量到的，不是推的。
表里没有的格：数据单元的区间死亡（K2，不归本腿）、快照、克隆的克隆、`crates/` 里还不存在的东西的写账（预付式）、崩溃点重放的状态数。

### 5.1　每个候选要改的不变量与已定项（读码 + grep，命令与行号现取）

通用（所有候选都欠，不是候选的账）：C615 的读侧「扫盘重建按择新键选中被回退掉的那一版」（`.claude/kb/checks-owed.md:475`）；回退取 R_old 的树才判版本谱系，随现行的树只按 P2 择版本（2.1 节「改法丙」复现说明统一滤法会错）。

**候选一**
- D18 已定项 11 登记的类型 3 要有载体、要带死亡 txg：「容器合并时右半退役、重生时旧身份退役」（`.claude/kb/decisions/18-块里携带什么信息.md:300`）（登记里没有任何一棵树收它，`crates/` 没有写者）；
- D8 已定项 6 两句：「类型 2 的记录死亡不写墓碑，死亡由现行叶版本体现」（`.claude/kb/decisions/08-核心索引结构.md:169`）（补：容器身份消失写退役记录）；「记录数为 0 的类型 2 容器不落盘**（发布前必须合并掉」（`.claude/kb/decisions/08-核心索引结构.md:171`）（删空的退化形也写）；`crates/` 里把删空叫左吸收右的退化形：`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:277` 的注释，写者落点 「pub fn remove_record_from_leaf_containers(」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:286`）、「tombstone: TombstoneRecord::of_the_whole_object(inode, record.object_birth, txg),」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/transaction.rs:5997`）（最后一批追加退役记录的对应处）。
- 树表多一种树：I-9.16 ④ 「进树表的十一种（② 那一串）各恰好一条」（`.claude/kb/invariants.md:304`）、I-9.27 「种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目等于这张树表里种类 2 条目的树 ID」（`.claude/kb/invariants.md:315`） 各加一格；新池新建文件的树表条数 「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-format/src/lib.rs:188`）；checker 「const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:4345`）；再立一条退役树自己的判据（与 I-9.17、I-9.23 同形：条目身份与子头相符、记录合法）。
  同类先例（谱系树加进来时碰到的地方，命令原样输出）：

```
$ cd /home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree && grep -rc 'TREE_KIND_LINEAGE' crates --include=*.rs | grep -v ':0$'
crates/singlefs-checker/src/walk.rs:7
crates/singlefs-core/src/recovery.rs:11
crates/singlefs-core/src/records.rs:2
crates/singlefs-core/src/transaction.rs:2
crates/singlefs-harness/tests/checker_known_bad_images.rs:25
crates/singlefs-harness/tests/tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker.rs:3
crates/singlefs-harness/src/mount_transparency.rs:2
$ grep -c 'TREE_KIND_LINEAGE\|PACKED_TYPE_LINEAGE' crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
7
```
  即同类新树碰到 7 个 Rust 文件 52 处加 GPU 判器 7 处（推的：退役树的触点数同量级）。
- 乙才要改的：D23 已定项 14 「墓碑树、谱系树随现行那一版，不取 R_old」（`.claude/kb/decisions/23-journal的角色与格式.md:378`），加一句「退役树取 R_old」。

**候选二**
- I-9.13 「任一打包记录类型 2 的容器记录数 ≥ 1」（`.claude/kb/invariants.md:301`） 要放宽；D8 已定项 6 那一句不落盘要改；C116 的三条写路径纪律之一 「合并保留左半身份、记录数为 0 的类型 2 容器不落盘」（`.claude/kb/checks-owed.md:106`） 与 `crates/mutations.tsv` 里以「三·2 inode 树删一条」打头的五条变异跟着重指；
- 分隔 key：零条目叶没有最小 key，吸收之后给它放的分隔 key 常无整数可放（2.4 节）。出路四选一，都不是「改一条不变量」：M3 只借不吸收；容器头加分隔 key 字段（格式冻结层的问题，我没判是哪一层）；放宽 I-9.12 对零条目叶的严格递增；给内部条目的类型段加一个「零条目叶」的值（2.5 节第 3 条的引文，改格式）。
- 叶只增不减，回收门（D18 已定项 24 第 4 条同一句：「第一版不回收墓碑，统计量 13、14」（`.claude/kb/decisions/18-块里携带什么信息.md:575`））要给零条目叶另写。

**候选三 Tsz**
- D8 已定项 6 那一句整句换掉（记录死亡由墓碑体现），不是补一句；D18 已定项 24 第 2 条用它现成的：「死亡 txg = 意图那次发布的 checkpoint_txg」（`.claude/kb/decisions/18-块里携带什么信息.md:573`）；
- K-W5b 要定读侧：「扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b」（`.claude/kb/decisions/18-块里携带什么信息.md:579`）——Tsz 的读法是：按（号, 对象出生代）在所有版本里不收，墓碑按死亡 txg 在不在谱系上滤，加供出规则与数据单元门，墓碑按头生效；
- 要用户答的一句：E177 的真值是「记录还在树里就算活」，Tsz 窗内把 nlink = 0 的记录判死。I-9.19 的字面是「谱系上的墓碑不指活对象」（`.claude/kb/invariants.md:307`）（指认的 (号, 出生代) 在 inode 树里的记录 nlink = 0 才算合法），按它自己的词，有谱系墓碑的对象不算活——这一句改成真值口径就是 X3「意图那次生效」的答案；
- 不改：I-9.13、I-9.4、I-9.12、I-9.16、I-9.27、D18 已定项 11、树表条数。

**Fsz**：撞 I-9.18 「`deleted_inodes`（D8（核心索引结构） 已定项 16）里的每个号 N，墓碑树（D18（块里携带什么信息） 已定项 24）里都有一条在谱系上的墓碑记录」（`.claude/kb/invariants.md:306`）（窗内版本 88%–94% 为假）与 D18 已定项 24 第 2 条的死亡 txg 口径。

### 5.2　零轮形态（这一轮新冒出来的，不为它再开一轮）

- 零轮 a：吸收时不留零条目叶、也不另立退役树，把被吸收的身份写进左半容器自己（容器头加「吸收过的身份」列表）。读侧无独立载体；代价是容器头变长、列表只增，我没建模。
- 零轮 b：Tsz 配 N2（借时换身份）：N2 的旧身份被新身份的拷贝压住，Tsz 不需要退役记录（`expected/safety.txt` 里 tsz 在 n2 借法下 r = 0，经 2.1 节同一批历史），N2 的账因此只剩「改 D8 的 COW 重写身份不变」一句；没有单独攻过。
- 零轮 c：候选二只用于删空（不吸收的那一支），吸收走退役记录——等于候选一加候选二两份账，没有算。


## 6　K3：N1 新形对 N2

### 6.1　两个形态按 `crates/` 今天的样子是什么

- `crates/` 里没有借：「它右边的每一片位置左移一格、这次全部重写。别的合并（左从右借、两片都不空的左吸收右）」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:17`）（文件头，删除只落「删空即摘」）。命令与输出：

```
$ cd /home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree && grep -rno '别的合并（左从右借[^）]*）' crates/singlefs-core/src --include=*.rs
crates/singlefs-core/src/inode_tree.rs:17:别的合并（左从右借、两片都不空的左吸收右）
```
  所以 N1 新形与 N2 都是**借这件事要新写的代码**，不是改一处现成的。分隔 key 今天就是容器号：「今天的最小 key 可以比诞生时大，而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）。」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:80`）
- N1 新形：分隔 key = 孩子这一版的最小 key，每次写父节点现算；删 I-9.4 第三句。核心改 `separator_key()` 一个函数（今天返回 `identity.container`），四个调用点不动：

```
$ cd /home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree && grep -n 'separator_key()' crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/inode_tree.rs
crates/singlefs-core/src/transaction.rs:7365:        .map_or(0, |container| container.contents.separator_key())
crates/singlefs-core/src/transaction.rs:7369:        .map_or(0, |container| container.contents.separator_key())
crates/singlefs-core/src/transaction.rs:7386:                    container.contents.separator_key(),
crates/singlefs-core/src/transaction.rs:7541:                        container.contents.separator_key(),
```
- N2：借完给右片换身份（容器号 = 借后最小 key，出生代 = 这次发布），旧身份按候选另处理（候选一写一条退役记录；候选二写一版零条目叶；候选三什么都不写）。I-9.4、I-9.12 的判据一个字不动，分隔 key 仍是容器号。
- `crates/` 今天读池时根本不按分隔 key 走：挂载读把内部节点的条目逐条读出来、每个叶都读（「for entry_bytes in &inode_root.entries {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/mounted_read.rs:480`）），分隔 key 只在 checker 的 I-9.12 与恢复读叶时的一道比较里用（「if record.inode < separator_key || record.inode < identity.container {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/recovery.rs:4879`））。所以「路由对不对」目前没有读者会踩，只有 I-9.12 在看。

### 6.2　每次借多写什么（量，同一段历史）

借法 × 候选，同一批历史（节选 cap233 与 ord233；全表在 `expected/cost.txt`）。`extra_roles_vs_today` 是相对今天（同一借法）多写的角色数；retire_records 是退役记录条数；stub_versions 是零条目版本数：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh k3cost research/prompts/m3-leaf-death-r3-sonnet-model/expected | grep -E '^(cap233|ord233) +(n1new|n2) '
cap233  n1new      persist  borrows=33   absorbs=573   drops=48    retire_records=0     stub_versions=621   extra_roles_vs_today=447
cap233  n1new      ret_a    borrows=36   absorbs=171   drops=45    retire_records=207   stub_versions=0     extra_roles_vs_today=414
cap233  n1new      today    borrows=36   absorbs=171   drops=45    retire_records=0     stub_versions=0     extra_roles_vs_today=0
cap233  n1new      tsz      borrows=36   absorbs=171   drops=45    retire_records=0     stub_versions=0     extra_roles_vs_today=0
cap233  n2         persist  borrows=21   absorbs=606   drops=48    retire_records=0     stub_versions=675   extra_roles_vs_today=489
cap233  n2         ret_a    borrows=36   absorbs=171   drops=45    retire_records=243   stub_versions=0     extra_roles_vs_today=486
cap233  n2         today    borrows=36   absorbs=171   drops=45    retire_records=0     stub_versions=0     extra_roles_vs_today=0
cap233  n2         tsz      borrows=36   absorbs=171   drops=45    retire_records=0     stub_versions=0     extra_roles_vs_today=0
ord233  n1new      persist  borrows=306  absorbs=48    drops=30    retire_records=0     stub_versions=78    extra_roles_vs_today=-516
ord233  n1new      ret_a    borrows=630  absorbs=96    drops=0     retire_records=93    stub_versions=0     extra_roles_vs_today=186
ord233  n1new      today    borrows=630  absorbs=96    drops=0     retire_records=0     stub_versions=0     extra_roles_vs_today=0
ord233  n1new      tsz      borrows=630  absorbs=96    drops=0     retire_records=0     stub_versions=0     extra_roles_vs_today=0
ord233  n2         persist  borrows=105  absorbs=0     drops=72    retire_records=0     stub_versions=177   extra_roles_vs_today=-657
ord233  n2         ret_a    borrows=630  absorbs=96    drops=0     retire_records=693   stub_versions=0     extra_roles_vs_today=1386
ord233  n2         today    borrows=630  absorbs=96    drops=0     retire_records=0     stub_versions=0     extra_roles_vs_today=0
ord233  n2         tsz      borrows=630  absorbs=96    drops=0     retire_records=0     stub_versions=0     extra_roles_vs_today=0
```

- 今天与 Tsz：N1 新形与 N2 多写 0（叶重写集合一样；换身份不改哪些叶要重写）。
- 候选一：N1 新形下借不退役（cap233 退役记录 207），N2 下每次借 +1 条（243 = 207 + 36 次借，ord233 693 = 93 + 600）；每条 2 个角色，所以 N2 相对 N1 新形多 「cap233 ret_a n2  … units_retirement=243 … roles_vs_today=486」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:11`）（cap233 +72 角色，约 +0.08%）、「ord233 ret_a n2  … units_retirement=693 … roles_vs_today=1386」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/cost.txt:73`）（ord233 +1 200 角色，约 +1.1%：从小到大删借得多）。
  借的频度是前提（3.2：从小到大删每千次发布约 10 次，盯着的 / 随机删 1.5–2.4 次）。**N2 的账 = 2 角色 × 借的次数，加上候选一本来的 2 角色 × 删空 / 吸收次数。**
- 候选二：N2 要给旧身份写一版零条目叶（cap233 零条目版本 675 对 621），而且旧身份的零条目叶夹在左邻与换了身份的右片之间，要的分隔 key 放不放得下，同 2.4 节：I-9.12 为假的版本占比 cap233 「cap233 persist n2  … versions=54303 … versions_i912_false=20652」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/structure.txt:13`）（N1 新形 2.5%）。
- 第三句为假的版本占比（N1 新形下 I-9.4 第三句按字面为假的，即必须删它的理由）：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh k3third research/prompts/m3-leaf-death-r3-sonnet-model/expected | grep -E '^(cap233|ord233) '
cap233  n1new      today  S3_false=29.6%  sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
cap233  n1raise    today  S3_false=29.6%  sep_not_derivable_from_leaf=29.6%  sep_matches_neither_rule=9.7%  I-9.12_false=0.0%  misrouted=0.0%
cap233  n2         today  S3_false=0.0%   sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
cap233  no_borrow  today  S3_false=0.0%   sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
ord233  n1new      today  S3_false=44.5%  sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
ord233  n2         today  S3_false=0.0%   sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
ord233  no_borrow  today  S3_false=0.0%   sep_not_derivable_from_leaf=0.0%   sep_matches_neither_rule=0.0%  I-9.12_false=0.0%  misrouted=0.0%
```

  N1 新形下借过之后右片的最小 key 超过它的容器号，第三句按字面为假：cap233 29.6%、ord233 44.5% 的版本（cap6 3.0%，rnd233 33.8%）；N2 下 0（每次借完容器号 = 最小 key）。
  N1 旧形（借时抬、之后不跟着变）在 cap233 有 9.7% 的版本分隔 key 两种重建都还原不出，新形 0——这是第二轮攻方腿打中、改成新形的那一格，我的模型独立复现了它。

### 6.3　要改的判据与代码触点（读码，命令 / 行号现取）

| | N1 新形 | N2 |
|---|---|---|
| I-9.4 | 删第三句：CPU 「.judge("I-9.4", left.largest_inode < right.container, || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:2782`）；GPU 「w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2918`）（连带 2915 行的局部变量 `left_largest`） | 不动 |
| I-9.12 | 不动（它已经判 分隔 key ≤ 孩子下界、> 上一个孩子上界：CPU 「self.judgements.judge("I-9.12", separator <= smallest, || {」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker/src/walk.rs:3252`）；GPU 「w_judge(INVARIANT_I_9_12, le64(separator, smallest));」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2901`）） | 不动 |
| kb 条文 | D8 已定项 6 的分隔 key 一句 「= 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目」（`.claude/kb/decisions/08-核心索引结构.md:164`） 改成「这一版的最小 key」；I-9.4 「沿叶序容器号严格递增，且左容器的最大 key < 右容器号（这条逼出合并只许左吸收右 / 左从右借）」（`.claude/kb/invariants.md:292`） 删后半 | D8 已定项 6 加：借时换身份（与「COW 重写身份不变」并存的第三种身份变化）；候选相关的退役 / 零条目条文（5.1） |
| 核心 | `separator_key()` 一个函数；`after_the_split_the_separator_keys_are_the_container_numbers_and_the_ranges_do_not_overlap` 那条单测（`inode_tree.rs:518`）改名改断言 | 借的写路径里换身份（新代码）；C116 记的第三条变异（「右半容器号给错、分裂点不取末尾、COW 重写时身份跟着换出生代」（`.claude/kb/checks-owed.md:106`））对应的单测要给借留例外 |
| 变异 / 坏镜像 | `crates/mutations.tsv` 第 952、953 行（「实审 B2 第 9 条（I-9.4 第三句）：左容器的最大 key 小于右容器号不判」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/mutations.tsv:952`）、「实审 B2 第 9 条（I-9.4 第三句）：判严一格」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/mutations.tsv:953`））与它们指的 `checker_narrow_invariants_and_abandoned_roots.rs:947` 那条坏镜像用例删掉或改成阳性对照 | 不动 |
| GPU 判器 | 删 1 行 + 1 个局部变量（`left_largest`）；叶行里存最大 key 的两个字还有没有别的读者，我没逐个查（I-9.12 用的是 `C_SEPARATORS` 行，另装） | I-9.4 / I-9.12 不动；退役树若进候选一，是另一笔账（5.1：同类新树碰到 GPU 7 处） |
| 内部节点「丢了从叶重建」 | 重建规则 = 取孩子最小 key（纯函数，量到 0% 还原不出）；零条目叶没有最小 key（与候选二冲突） | 重建规则 = 取容器号（不变，0% 还原不出） |


### 6.4　删掉第三句之后，C116 那一族「右半容器号给错」由哪条判据拦（量，真 checker，仓副本）

做法：冻结树的 `crates/` 拷进草稿目录的仓副本，加一个 harness 测试目标 `sonnet_r3_leaf_death_images`（`copy/sonnet_r3_leaf_death_images.rs`）、给 `walk.rs` 的 I-9.4 第三句加一个环境变量开关（`tools/patch_checker.py`，设了就判成恒真，等于删掉这一句）；
用 `pool_with_new_inodes` 建 3 片叶 [1..233] [234..466] [467]，把第二片的容器号改成一族值（叶头那份与父条目身份引用那份一起改），分隔 key 两种约定：T = 分隔 key 跟着容器号（今天，也是 N2、不借的约定）；N = 分隔 key 是孩子这一版的最小 key（N1 新形，第二片恒 234）。
每份镜像用两种 checker 各判一遍。这是 harness 档的 `cargo test`（3 条用例，10 秒），不是 checker 档。完整输出在 `expected/copy.txt`：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh mirrors research/prompts/m3-leaf-death-r3-sonnet-model/expected
N n=0 today=["I-9.4"] third_clause_deleted=["I-9.4"]
N n=1 today=["I-9.4"] third_clause_deleted=["I-9.4"]
N n=2 today=["I-9.4"] third_clause_deleted=[]
N n=17 today=["I-9.4"] third_clause_deleted=[]
N n=100 today=["I-9.4"] third_clause_deleted=[]
N n=233 today=["I-9.4"] third_clause_deleted=[]
N n=234 today=[] third_clause_deleted=[]
N n=235 today=["I-9.4"] third_clause_deleted=["I-9.4"]
N n=300 today=["I-9.4"] third_clause_deleted=["I-9.4"]
N n=466 today=["I-9.4"] third_clause_deleted=["I-9.4"]
N n=467 today=["I-1.8", "I-9.4"] third_clause_deleted=["I-1.8", "I-9.4"]
T n=0 today=["I-1.1", "I-9.4", "I-9.12"] third_clause_deleted=["I-1.1", "I-9.4", "I-9.12"]
T n=1 today=["I-1.1", "I-9.4", "I-9.12"] third_clause_deleted=["I-1.1", "I-9.4", "I-9.12"]
T n=2 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.12"]
T n=17 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.12"]
T n=100 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.12"]
T n=233 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.12"]
T n=234 today=[] third_clause_deleted=[]
T n=235 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.4", "I-9.12"]
T n=300 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.4", "I-9.12"]
T n=466 today=["I-9.4", "I-9.12"] third_clause_deleted=["I-9.4", "I-9.12"]
T n=467 today=["I-1.1", "I-1.8", "I-9.4", "I-9.12"] third_clause_deleted=["I-1.1", "I-1.8", "I-9.4", "I-9.12"]
```

正确值是 234（触发分裂的那条新记录的 inode 号）。读法：
- **T 约定**（分隔 key 还是容器号）：删第三句之后，右半容器号落在 (1, 233]（比左容器号大、不超过左容器最大 key）的镜像仍然红——I-9.12 接住：「SONR3 image=right_half_number_100 separator_convention=T_separator_is_the_container_number leaves=[no=1 sep=1 n=233 1..233][no=100 sep=100 n=233 234..466][no=467 sep=467 n=1 467..467] checker=third_clause_deleted violated=["I-9.12"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:19`）；今天是 I-9.4 与 I-9.12 都红。T 约定下第三句与 I-9.12 在这一族上同真同假，删它不丢拦截。
- **N 约定**（N1 新形）：同一族镜像删第三句之后**全绿**：「SONR3 image=right_half_number_100 separator_convention=N_separator_is_the_childs_smallest_key leaves=[no=1 sep=1 n=233 1..233][no=100 sep=234 n=233 234..466][no=467 sep=467 n=1 467..467] checker=third_clause_deleted violated=[]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:17`）；今天它红 I-9.4：「SONR3 image=right_half_number_100 separator_convention=N_separator_is_the_childs_smallest_key leaves=[no=1 sep=1 n=233 1..233][no=100 sep=234 n=233 234..466][no=467 sep=467 n=1 467..467] checker=today violated=["I-9.4"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:18`）。绿的是容器号落在 (1, 233] 的整个区间（样本 2、17、100、233），因为 N 约定下分隔 key 不看容器号，I-9.12 对容器号没有话说。
- 删第三句之后还拦得住的两头：号 ≤ 左容器号（0、1）被第二句拦（严格递增）：「SONR3 image=right_half_number_0 separator_convention=N_separator_is_the_childs_smallest_key leaves=[no=1 sep=1 n=233 1..233][no=0 sep=234 n=233 234..466][no=467 sep=467 n=1 467..467] checker=third_clause_deleted violated=["I-9.4"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:13`）；号 > 右半最小 key（235、300、466、467）被第一句拦（记录号 ≥ 容器号）：「SONR3 image=right_half_number_300 separator_convention=N_separator_is_the_childs_smallest_key leaves=[no=1 sep=1 n=233 1..233][no=300 sep=234 n=233 234..466][no=467 sep=467 n=1 467..467] checker=third_clause_deleted violated=["I-9.4"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:45`）。C116 变异表里记的那一条具体变异（右半容器号写成 0：「并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/mutations.tsv:194`））是 0 这一形，第二句仍拦，所以这条变异删第三句前后都红。
- **拦不住的一形**（N1 新形下）：右半容器号落在（左容器号, 左最大 key] 里，不小于左容器号 + 1、不大于触发分裂那一刻左容器的最大 key。在叶容量 233 的最右分裂，这个区间有 232 个值。I-9.4 剩下的第一、二句和 I-9.12 都不判它，今天只有第三句判。
- 谁来拦它：kb 一直把这一类放在镜像判据之外——「分裂 / 合并的身份传递（左半身份不变、右半号 = 触发分裂的新记录号、出生树 = 执行分裂的树）与合并时机是操作断言，对着一个镜像判不了」（`.claude/kb/invariants.md:319`）；`inode_tree.rs` 的文件头同样写明「这三条今天没有会失败的检查（C116（叶容器的分裂 / 合并纪律没有会失败的检查））」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:10`），后置条件靠「三条纪律在它上面是后置条件、10 条单测判得了，另一层是恢复加 oracle」（`.claude/kb/checks-owed.md:106`）。
  所以第三句拦住 C116 这一族的一部分，是巧合不是设计；N1 新形删它，是把巧合的那部分也交还给写路径的单测与恢复 oracle。这一形的危害（推的）：容器号只剩身份用途，路由不再看它，号取得不对不会读错记录，破坏的是「右半容器号 = 触发分裂的新记录号」这条契约本身。

零条目叶那一边（候选二）同一份仓副本上的镜像：

```
$ bash research/prompts/m3-leaf-death-r3-sonnet-model/tools/views.sh stubs research/prompts/m3-leaf-death-r3-sonnet-model/expected
prefix_left_100_middle_17
    [no=1 sep=1 n=100 1..100][no=234 sep=234 n=17 234..250][no=467 sep=467 n=1 467..467]
    today=[] third_clause_deleted=[]
drop_leaves_a_zero_entry_leaf
    [no=1 sep=1 n=100 1..100][no=234 sep=234 n=0 empty][no=467 sep=467 n=1 467..467]
    today=["I-9.13"] third_clause_deleted=["I-9.13"]
absorb_leaves_a_zero_entry_leaf_separator_kept_234
    [no=1 sep=1 n=117 1..250][no=234 sep=234 n=0 empty][no=467 sep=467 n=1 467..467]
    today=["I-9.13"] third_clause_deleted=["I-9.13"]
absorb_leaves_a_zero_entry_leaf_separator_raised_to_251_with_room
    [no=1 sep=1 n=117 1..250][no=234 sep=251 n=0 empty][no=467 sep=467 n=1 467..467]
    today=["I-9.13"] third_clause_deleted=["I-9.13"]
absorb_leaves_a_zero_entry_leaf_no_room_for_a_separator
    [no=1 sep=1 n=117 1..250][no=234 sep=251 n=0 empty][no=251 sep=251 n=1 251..251]
    today=["I-1.1", "I-9.13"] third_clause_deleted=["I-1.1", "I-9.13"]
```

零条目叶只红 I-9.13，不红 I-9.12 / I-9.4 第三句（checker 对空容器没有 key 区间可判）：「SONR3 image=absorb_leaves_a_zero_entry_leaf_separator_kept_234  leaves=[no=1 sep=1 n=117 1..250][no=234 sep=234 n=0 empty][no=467 sep=467 n=1 467..467] checker=today violated=["I-9.13"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:4`）——左邻已经吸收到 250，零条目叶的分隔 key 还是 234，按它查 234..250 会走进空叶，今天没有判据看得见；放宽 I-9.13 之后这一形无人拦。
分隔 key 取 251 但右邻也是 251 时：「SONR3 image=absorb_leaves_a_zero_entry_leaf_no_room_for_a_separator  leaves=[no=1 sep=1 n=117 1..250][no=234 sep=251 n=0 empty][no=251 sep=251 n=1 251..251] checker=today violated=["I-1.1", "I-9.13"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:2`），I-1.1（条目 key 严格递增）红。

### 6.5　内部节点丢了从叶重建（量 + 读码）

D8：「派生态，丢了从叶重建」（`.claude/kb/decisions/08-核心索引结构.md:160`）。`crates/` 今天没有扫描重建读侧（挂载读整棵树，上面 6.1），这一格目前只有 checker 的 I-9.12 与第二轮攻方腿、我的模型在看。
- N1 新形：分隔 key = 叶的最小 key，纯函数；模型里「存着的分隔 key ≠ 由叶现算的」的版本 0%（6.2 表 `sep_not_derivable_from_leaf` 在 n1new 行）。零条目叶没有最小 key，重建规则要为它另给一个取值（候选二的冲突）。
- N1 旧形（借时抬、之后不跟着变）：分隔 key 由历史定，cap233 9.7%、cap6 1.0% 的版本两种重建都还原不出，第二轮攻方腿量到 8.9% / 1.4%。
- N2：分隔 key = 容器号，纯函数，也是 0%；换了身份的右片的容器号写在叶头里，重建照读叶头。
- GPU 判器：N1 新形删 1 行 + 1 个局部变量（6.3）；N2 对 I-9.4 / I-9.12 不动。我没能在 GPU 上跑任何东西：GPU 一侧是读码推的，**复核不了**（要卡，且这一轮不跑 checker 档）。

### 6.6　K3 的判

1. **多写**：N1 新形与 N2 在今天的写路径、Tsz 下都是 0；在候选一下 N2 每次借 +1 条退役记录 = +2 角色（cap233 +0.08%，ord233 +1.1%，借得多的历史），候选二下 N2 另写一版零条目叶并撞分隔 key。
2. **要改的判据**：N1 新形删 I-9.4 第三句（CPU 1 处、GPU 1 处）+ `separator_key()` 一个函数 + D8 一句 + 2 条变异 + 1 条坏镜像用例；N2 不改 I-9.4 / I-9.12，但借的写路径要换身份、退役或零条目条文按候选改、「COW 重写身份不变」的单测给借留例外。
3. **C116 那一族**：N2 保住第三句，这一族照旧被拦；N1 新形删掉第三句之后，容器号落在（左容器号, 左最大 key] 里的右半没有任何镜像判据拦（第一、二句拦两头，I-9.12 在 N 约定下对容器号无话），只剩 C116 的写路径后置条件与恢复 oracle——而 kb 本来就把这一类放在镜像判据之外。
4. **内部节点重建**：N1 新形、N2 都是叶的纯函数（0% 还原不出）；N1 旧形不是（9.7%）。
5. **取向**（正推给的，不替主 agent 采纳）：候选一已经要退役树时，N2 的增量是 2 角色 × 借的次数且保住第三句；候选三 Tsz 下 N2 不需要任何新机制（模型里 tsz × n2 的 r = 0），且保住第三句；候选二下 N1、N2 都撞冲突。N1 新形的好处是不多写、不换身份，代价是丢掉第三句顺带拦着的那一部分「右半号给错」。

什么现象会推翻 6 节：N 约定下容器号在 (1, 233] 的镜像被某一条剩下的判据拦住（`expected/copy.txt` 里 `separator_convention=N_…` 且 `checker=third_clause_deleted` 的行，n = 2、17、100、233 的 `violated` 不再是 `[]`）；或模型里 N2 在某个候选下 r、m 在窗外不为 0（`expected/safety.txt` 里该候选 n2 行）。


## 7　冲突的两句并排（整行抄）

**冲突 1　候选二的账（第二轮判决 对 本腿量到的镜像）**
- 「判：**候选二回到候选集**，代价是要改 I-9.13、叶数不减。」（`research/prompts/m3-leaf-death-r2-main-verification.md:28`）
- 「SONR3 image=absorb_leaves_a_zero_entry_leaf_no_room_for_a_separator  leaves=[no=1 sep=1 n=117 1..250][no=234 sep=251 n=0 empty][no=251 sep=251 n=1 251..251] checker=today violated=["I-1.1", "I-9.13"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:2`）

**冲突 2　N1 新形对 D8 与实现**
- D8：「= 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目」（`.claude/kb/decisions/08-核心索引结构.md:164`）
- 实现：「今天的最小 key 可以比诞生时大，而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）。」（`/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree/crates/singlefs-core/src/inode_tree.rs:80`）
- N1 新形的定义（派发正文第三节 K3）是分隔 key = 孩子这一版的最小 key，两处都要改。

**冲突 3　第二轮判决「第三句与 I-9.12 同一个谓词」对 N 约定下的镜像**
- 「只红 I-9.4 的坏镜像造不出来（30 张镜像量过）」（`research/prompts/m3-leaf-death-r2-main-verification.md:39`）
- 「SONR3 image=right_half_number_100 separator_convention=N_separator_is_the_childs_smallest_key leaves=[no=1 sep=1 n=233 1..233][no=100 sep=234 n=233 234..466][no=467 sep=467 n=1 467..467] checker=today violated=["I-9.4"]」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/copy.txt:18`）
  （今天只红 I-9.4；删掉原第三句后同一份镜像全绿，见 6.4。第二轮量的是「改写后的第三句」，与 I-9.12 同谓词没错；这里量的是**原第三句**在分隔 key 脱钩之后拦的那一段，两者不是同一个量。）

**冲突 4　Fsz 对 D18 已定项 24 第 2 条**
- 「死亡 txg = 意图那次发布的 checkpoint_txg」（`.claude/kb/decisions/18-块里携带什么信息.md:573`）
- Fsz 把墓碑的写挪到最后一批：「if self.cfg.cand == Cand::Fsz { self.tomb_append(&mut next, ino, obirth); }」（`research/prompts/m3-leaf-death-r3-sonnet-model/src/model.rs:705`）（死亡 txg = 最后一批那次发布的 txg）。

## 8　判定一览

「一致」= 本腿正推与被判对象（前两轮判决 / 候选的说法）同向；「冲突」= 并排的两句见第 7 节或正文；「规则没说」= kb 没有条款可对。每条后面的「推翻：」是什么现象会推翻它。

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 复活 / 误判死亡（同一组历史） | 一致 | 候选一两改法、候选二、Fsz 的 own 读法 r = m = 0，Tsz r = 0、m 只在 unlink 途中（3 300 段历史，其中 288 段前提族把故障关掉，别的含崩溃 / 丢根 / 回退）。推翻：任一 own 读法行 r 或 m 不为 0，或 Tsz 窗外 m 不为 0 |
| K1 unlink 途中漏活 | 一致 | Tsz 每个窗口漏 1 条 inode 记录加未截数据单元（窗内版本 45%–48%）；Fsz 窗内无漏但 I-9.18 窗内 88%–94% 的版本为假。推翻：Tsz 窗内 m 为 0 |
| K1 每次删空 / 合并多写 | 一致 | 候选一每条退役记录 2 角色（常数，0.17%–0.45% / 叶 6 为 8.7%）；候选二 −0.5% … +2.7%；Tsz 0；与第二轮攻方腿在仓副本上数的「墓碑两个角色」对上。推翻：退役树根不随每条记录重写 |
| K1 候选二与用户已定的 M3 | 冲突 | 见冲突 1：吸收留下的零条目叶在稠密编号下无处放分隔 key（从小到大删 100%，别的历史 0.5%–17%）；I-9.12 / I-9.4 第三句对零条目叶失明。推翻：找到不改格式的取法 |
| K1 要改的不变量与已定项 | 规则没说 | D18 已定项 11 登记了类型 3 却没有任何一棵树收它；K-W5b 读侧没定；清单与同类新树的触点数在 5.1。乙才改 D23 已定项 14，Tsz 才要用户答 E177 真值口径（X3） |
| K1 前两轮三套模型的数能不能并排 | 冲突 | 比例与常数能（候选一 r = m = 0、N1 旧形还原不出 8.9% 对 9.7%、每条退役记录 2 角色），绝对数不能：差在「有没有故障」「怎么挑删」「每段历史多长」和 B 的分母含崩溃支线；**「谁有数据」不是前提差**（与派发正文列的三个前提不同）。推翻：前提族里三项换开后同一个量的跨度 < 2 倍 |
| K1 克隆头共享叶 | 一致 | 规则齐全时三候选在 55 858 条序列上 r = m = 0；退役记录按头分是 P6 已逼出来的；Tsz 多一条（墓碑按头生效）；候选二在共享叶上写零条目要 P6 重生，多写退役记录。推翻：有界穷举里出现反例 |
| K1 副本坏一份（w = 2） | 规则没说 | 一份坏另一份好 = 视图不变（P3 与 E104 w = 2 世界）；模型没建副本，**复核不了**；两份都坏三候选都有单点，候选一二回来整片叶，候选三回来被点名的记录。推翻：一份坏的格上量到差别 |
| K1 清扫删实例表行 | 一致 | 清扫不分辨候选；漏抹证据单元时只有 Tsz 在窗外误判死亡（证据在意图那次写，崩溃点上在飞比例 27% 对 6%）。推翻：漏抹变体里候选一 / 二出现 inode 窗外 m |
| K3 每次借多写 | 一致 | N1 新形 0；N2 在候选一下每次借 +2 角色，Tsz 下 0，候选二下 +1 零条目叶并撞分隔 key。推翻：N2 在 Tsz 下多写 |
| K3 要改的判据 | 冲突 | 见冲突 2：N1 新形删 I-9.4 第三句（CPU 1 处、GPU 1 处）+ `separator_key()` + D8 一句 + 2 条变异 + 1 条坏镜像；N2 不改 I-9.4 / I-9.12，但借要换身份，「COW 重写身份不变」留例外 |
| K3 删第三句后 C116 族由谁拦 | 冲突 | 见冲突 3：T 约定下 I-9.12 接住；N1 新形（N 约定）下容器号落在（左容器号, 左最大 key] 的右半无镜像判据拦，只剩写路径后置条件；两头被第一、二句拦。推翻：N 约定下该区间的镜像被剩下的判据拦住 |
| K3 内部节点丢了从叶重建 | 一致 | N1 新形、N2 都是叶的纯函数（0% 还原不出）；N1 旧形 9.7%（B 8.9%）。零条目叶没有最小 key（候选二冲突） |
| K3 GPU 判器改动量 | 规则没说 | N1 新形删 1 行 + 1 个局部变量；N2 对 I-9.4 / I-9.12 不动；退役树若进候选一，同类新树的先例碰 GPU 7 处。GPU 一侧是读码推的，**复核不了** |

## 9　没做什么

- 不替主 agent 采纳或出判决：第 5、6.6 节的「取向」是正推给出的次序，不是采纳。K2（读法 Z、区间墓碑）、K1 的攻方格不归本腿。
- 数全是副本上的（我的模型、仓副本上的 harness 测试），没有在入库装置上重做，不进 kb；入库重做是主 agent 的事（E178 登记的臂里没有的，要补登记）。
- 模型的限度：主模型是单头；`units` 名字空间是按槽号的代理，没有真的 extent 树、分配记录树、记账、映射；崩溃只枚举在飞单元写的子集（k ≤ 8 逐子集，更大的取空集 / 全集 / 单点 / 去掉单点），没有 journal 记录与根槽两段；环回收按「释放之后 24 次发布」近似；回退目标限在 22 次发布内；读侧是手写的（`crates/` 没有扫描重建的读侧）。
- 候选二的写法是我选的（吸收与借只在紧邻非空叶间做，零条目叶当最右叶时接着追加，吸收后分隔 key 取左邻最大 key + 1）；PersistX 是第二轮辩方的跨过零条目叶的写法。别的写法（比如吸收时另立记号、不复活零条目叶）没算，结论「无处放分隔 key」依赖稠密编号这一前提，随机删时是 0.5%–17%。
- 没复现 A 的 51 段 E177 历史，只换了三个前提加历史长度；A、C 的 7.2% / 11.0% 落在我模型的 0%–96% 范围里是推的，不是复现。
- 克隆头模型叶容量 2、无快照、无克隆的克隆、无崩溃与回退、无数据单元；w = 2 一份坏没建副本；GPU 判器没跑（没卡，也不在这一轮的档里）。
- 没单测 RetB 不带死亡 txg（B 说可以不带，我没验）。
- 没跑 checker 档、层 0、QEMU、变异整表；harness 档只跑了自己的一个测试目标（3 条用例，仓副本里）。
- 没改 kb、`crates/`、门禁、`.claude/`；没读禁读清单里的文件。

## 10　复跑与文件

模型目录 `research/prompts/m3-leaf-death-r3-sonnet-model/`（入库内容见下 sha256）。复跑（仓根为工作目录；线程数读 `R3_THREADS`；仓路径给 `SINGLEFS_REPO`，只用来找 harness 副本要拷的 `crates/`，给冻结树或仓根都行；每阶段整条经内存包装，编译 6G、其余 4G，同一时刻只起一条 cargo）：

```
M=research/prompts/m3-leaf-death-r3-sonnet-model; S=scripts; O=<空目录>
export SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r1/tree
bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh $O build
for k in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O sweep-$k; done
bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O clone determinism
bash $S/run-with-memory-cap.sh 6G bash $M/rerun.sh $O copybuild
bash $S/run-with-memory-cap.sh 4G bash $M/rerun.sh $O copyrun summarize
bash $M/compare.sh $O
```

交回前在空目录 `~/.local/share/singlefs/work/m3-leaf-death-r3-sonnet/rerun3/` 里按上面从头跑通（`R3_THREADS=8`，机器忙，loadavg 30+；每个阶段单独跑都在 4 分钟内，只有一次把 sweep-4、sweep-5 链在一条命令里，超过工具的 240 秒前台限时被转到后台，两个阶段自己都正常跑完）：`compare.sh` 11 份先是 10 份 same、`determinism.txt` differ（345 → 346 段，是我在 rerun1 之后加了历史长度族、挑选的下标变了），更新 `expected/determinism.txt` 之后 11 份全 same、退 0。
`expected/` 里另外 10 份由 `rerun1/` 的 `summary/` 拷来、`rerun3/` 从头独立复出后逐份相同，所以这一次比对不是同义反复。另在 `rerun2/` 里独立复跑 sweep-2、sweep-13、sweep-14、clone 与 copy 两阶段，排序后的 sha256 / 副本输出与期望逐个相同（当时 `R3_THREADS=6`，与 8 的结果相同，印证线程数不影响输出）。
模型确定：「R3 name=determinism histories=346 two_runs_identical=true」（`research/prompts/m3-leaf-death-r3-sonnet-model/expected/determinism.txt:1`）。同一 (形状, 种子) 下所有候选 / 借法 / 几何的逻辑状态哈希只有一个：`expected/history-hashes.txt` 第一行 `segments_by_shape_and_seed=… with_more_than_one_logical_hash=0`。

模型目录每个文件的 sha256（`find . -type f | sort | xargs sha256sum`）：

```
fa72dc7f8cbc19e67083d96d9855c53ce08047f377f1a55ff599fbce4c2bc059  ./Cargo.lock
d61409167374144772ed5da9b6f6924c8e25a07da13e568effc2aab8f41e9db1  ./Cargo.toml
5b55688b756ab538a2547b434c9a4d6b213c8a0ddf6af6bd30d1c78d7b680f17  ./compare.sh
9f01c2eaf8807d21d494c59a325c37a0f386e7c1419a0ee1144c5da9523e55ea  ./copy/sonnet_r3_leaf_death_images.rs
5c3d363d86754610f0d9063204c2673b6b4165e106c635a84a0a749ec4faab2a  ./expected/clone.txt
1a7b03d4d0615841e2cb7214873c174945d339daf3867a3300cf2eb27db76140  ./expected/copy.txt
d0569be2dd11892525b4616687a8feca895c00a4e1589c144c75437ae04ae687  ./expected/cost.txt
dd8a3627a6d4caf0c114e7bbf209af3f50a7a74f167965900b307c0378a20b01  ./expected/determinism.txt
53afe8209316f3cb07109e09d1f9e7e57b9909c7b513dbb4fedd053cddde99d1  ./expected/history-hashes.txt
3a759e1fc56964206261085a9223c6cec9c68b09c5f546a04821ada78ce0aac5  ./expected/loss.txt
0634c8c3f23f66fda1d8b6ab7f175d37dae8b9ea0107df78098dc75f76f8b285  ./expected/premises.txt
df78b6c1f7c5dda8831ab41c4c168bcb76bce4f11bcc2ac1ec7794e8795ef13a  ./expected/safety.txt
a546209d7fdf15a0ee3be3e24013bdf47b9d61e418e7ccf6ad7b5e44057e71ef  ./expected/sorted-output-sha256.txt
acdac1dcbd115bf1095594fd2cd2a96b49f1ec55771689aea52a5fb17a7e9852  ./expected/structure.txt
3263dd350767edfd26cb38be6921204401142b29604884dce04b3edbc94ac030  ./expected/sweep.txt
90e4c9651bbf5ae0700f45cf235f63cda092611e10d4e65d9e06ec85dd8de504  ./README.md
a962062ef0eaabe9f298e3b5225f23364be124c3157f2677ba0568be4a8d0345  ./rerun.sh
8debcbc8678caa3cce86119033e73f5004c08b57477e429a06d5d75acaddbc51  ./src/clone.rs
b6e4620304600936bd2ef4371ead1555c269e3231874dc489ba9a526eba69764  ./src/main.rs
f193f8c5ff58b8bd49378deb515b285a0117757b9187a696bba2f99f4c693fde  ./src/model.rs
9898cda7d066fc082ee8a7df9edf274261be1ddad58e97c1e9b7747eae884dc9  ./tools/patch_checker.py
c95e87f84bd3dc202e6c4be91ca7ff23f7b8038e0efcc644b23e60f7b0b0076a  ./tools/tables.py
7f372e6abbeb974b6b0bfa0199538dd9937132a097fb967c8accb8e4ba10176b  ./tools/views.sh
```

## 11　草稿与副本

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-leaf-death-r3-sonnet/`（现 50M）：`dev/`（开发用源码与小工具；与模型目录逐字节相同的那几份之外，`tools/mk.py`、`tools/cite.sh`、`tools/citerow.py` 是写报告用的，没入库）、`runs/` `runs2/`（开发中的模型输出）、`rerun1/` `rerun2/` `rerun3/`（三次复跑的 `*.out` 与 `summary/`）、`seg/`（报告各段的生成脚本与成品）、`tables/`、`report-backup-broken.md`（第 2 段第一次追加时 heredoc 展开出错的那一份，已用 `replace-once.py` 从报告里剔掉重写，留作记录）。
- 自己建的编译目录与仓副本交回前已用字面绝对路径删掉：`…/m3-leaf-death-r3-sonnet/target-dev`、`…/repo`、`…/repo-target`，以及 `rerun1/`、`rerun2/`、`rerun3/` 下各自的 `target/`、`repo/`、`repo-target/`（单个 `repo-target` 约 1G）；删完 `ls` 核过一个不剩。
- 没入库的：完整模型输出（`sweep-*.out` 与 `clone.out`，三份复跑合计不到 50M），`rerun.sh` 逐字节复出，`expected/sorted-output-sha256.txt` 钉着。

