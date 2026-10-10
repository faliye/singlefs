# m3-batch22-code-r1 攻方腿（Opus）报告：删除批第 22 批合入的代码，K2

日期 2026-10-06。立场：攻方，在冻结副本 `/tmp/claude-1000/m3-batch22-code-r1/tree/` 的拷贝上造反例，只跑 harness 档与我自己的原型（GPU、checker 档测试、层 0 全域都没跑）。所有数都是副本上量的。

## 复跑

```
SINGLEFS_REPO=/tmp/claude-1000/m3-batch22-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs \
  bash research/prompts/m3-batch22-code-r1-opus-model/rerun.sh <空目录> all
bash research/prompts/m3-batch22-code-r1-opus-model/rerun.sh compare <空目录>   # 末行 SUMMARY_MATCHES 907 行
```

段名、内存上限（编译 6G、跑 4G）与线程（`OPUS_THREADS`，默认 10）写在 `rerun.sh` 文件头。交回前在空目录 `/tmp/claude-1000/m3-batch22-code-r1-opus/rerun2/` 从头分段跑通一遍（setup 起到 summary，每段单独起），`compare` 报 `SUMMARY_MATCHES 907 行`；`compare` 比之前两边都排序。`logs/` 是那一遍的各段日志，报告里引的输出行都出自它们。

模型文件的 sha256：

```
5af5343c0d59f049ec6ed05e2f207cacdaf1a2da390230533d768932399652d5  research/prompts/m3-batch22-code-r1-opus-model/opus_b22_attack.rs
4573b160c048fc709707564623d5ea08e560f16b39e7058c4a7d85fc3bfffd0e  research/prompts/m3-batch22-code-r1-opus-model/rerun.sh
6f9764fa45b22b8d68a4bb2ebcd7f44d3b03a9b3ee0b340c19068323ab78d8a5  research/prompts/m3-batch22-code-r1-opus-model/expected-summary.txt
4ce66c6baa53c9b326a722e04fc2b47ca43137bf7b32e328abd0d38470f9e740  research/prompts/m3-batch22-code-r1-opus-model/crash/Cargo.toml.in
b08f14d21f5538d0d7ac070940a1153ef3b95b41177ebff3b5c1643bd751038a  research/prompts/m3-batch22-code-r1-opus-model/crash/src/main.rs
ad5d94dfeb15e5809ed46bdfd58c9191bf9a9c3013c1c27e9ee96e4feff4ecf8  research/prompts/m3-batch22-code-r1-opus-model/logs/build.log
206c65f726b233e661095e8b5c92719f92c0750694f0d2b2ff757893e5181eb7  research/prompts/m3-batch22-code-r1-opus-model/logs/crash-boundaries.log
4878c28cf86afef3ba5d97f94901399ac2295123f9fca792daeb4e9dde0d1115  research/prompts/m3-batch22-code-r1-opus-model/logs/crash-build.log
ac60d216d928710e707a6b27cd3db1d936db5c4be6d3191c6656fdfcdd9f20e4  research/prompts/m3-batch22-code-r1-opus-model/logs/crash-count.log
e12638a63465d598bd44c486167f53c47516122c2d58aed1a897bd9883264af5  research/prompts/m3-batch22-code-r1-opus-model/logs/lineage.log
54ea45e70a17cfa35bd4abd681d80d0169cc2168fa14dd28e30bf91bad503b69  research/prompts/m3-batch22-code-r1-opus-model/logs/n4n7.log
2f8ed17fbeebc7dbd9a1cb9680329a199e495ff7e667c5e7ea6cc6c03e2a7c86  research/prompts/m3-batch22-code-r1-opus-model/logs/n6.log
272e9d2225e3979a643c23b08dec8ca4f14a0f314eb8ff7601eefb55ce468e09  research/prompts/m3-batch22-code-r1-opus-model/logs/n7.log
45d70e9054295ad68b416c92e58fcafce9e0ee17eaed158b24e4a9ad14138514  research/prompts/m3-batch22-code-r1-opus-model/logs/scenarios.log
eb88d8964108ad67937de95805d1e2871a5b1922f6410cdeff9a9227acfb59a7  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0-b.log
5ab5f56e184b58543b80ef1196a5f8a079295d3375a28cc5205002f230849d03  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0.log
8de65f30c41e0acc46874d5798002e9101a9f3d592ca267329e463aec848c622  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0-long-b.log
de86db37f1ec2d79bc1edff8849b41de811c029632f5d6d8d43a0c595f16410e  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0-long.log
c1326f5e935105f3b88603ebdb5c7749dfbf0394152707e3b355816f05421682  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0-reissue-b.log
e6256112a9aa7335331af6549b10a3db31fa18be73a0ada3358bf633b0a3b7d9  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k0-reissue.log
286ef664c4fa3c305282085749c6541db3bad667722897d7b9384e7fecdbf5bb  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k1-b.log
821bddffab46109ccb0962eed2d68e971e1292559269b85677dac26981c43c24  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k1.log
23156f89fbfb04d621231d5e2aeb01ed59d5984fc531147a56b8a6113ce2c6bd  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k1-long-b.log
7ad7624a2d167d3c96e9d968feb4f24c87f7c9bdde1a0eb1421c134b953f99f8  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k1-long.log
a46146f22f882e0d40d0eb57da705e0eaffd773d6856416423e99f6cb4fee6c9  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k2-reissue-b.log
9bf969cc52bd47227f09810cf9391014ca82491323f6217230cc17d3942e9ef1  research/prompts/m3-batch22-code-r1-opus-model/logs/sweep-k2-reissue.log
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K2 N4（I-9.17 / I-9.23 只红自己） | **打中**（三形） | 类型段与孩子真实的类不符的条目：类型段 0 指着码 3 容器、类型段 1 / 5 指着码 2 节点。前者在层级 1 的根上记走读失败（I-7.2、I-4.8、I-7.4 连带），后者按 32 KiB 去读 16 KiB 的节点（I-2.1、I-3.1、I-3.11、I-4.8、I-5.1、I-7.2、I-7.4 连带）；身份引用三段清零、只剩「子单元是码 2 / 码 3」那一句不符时 I-9.23 自己报**成立**（漏判） |
| K2 N4 其余几形 | 没打中（一次抽样） | 类型段写成条款外的 7（层级 1、层级 2、根环转过一圈）、互换成对方那棵树的类型段（1↔5）、类型段 0 条目的身份引用非零：都只红自己 |
| K2 N7（队列读不出、头 ID 配不上时 I-9.24） | 头 ID 配不上：判（成立 / 两条 key 时不成立）；队列根读不出：不适用 | 都照条款；**线索（漏判候选，口径交主 agent）**：`deleted_inodes` 根层级写成 1、两条 8 字节条目，I-9.24 报不适用，整张镜像一条不变量都不红 |
| K2 N5（合法历史误红、违反字面漏判） | 没打中（一次抽样） | 8 段写死前缀的场景、12 批共 4 661 次判定的随机历史（回退目标、回退之后先做什么、进程退出再挂、崩溃恢复落到前一条根都放开扫）上 I-9.17–I-9.29、I-9.5、I-9.10、I-9.6 零红；崩溃段界 249 个状态（恢复前、可写挂载后、再建一个号后各判一次）零红 |
| K2 N5 附：谱系记录自己坏了 | 线索（不算打中） | 谱系记录 T_old ≥ T_rb：I-9.19、I-9.20、I-9.29 报不适用，整张镜像不红；条款里没有一行判谱系记录本身 |
| K2 N9（I-9.29） | 没打中（一次抽样） | 「建号那次被崩溃恢复抛弃 → 新线重发同号（出生代 7 → 11）」固定前缀之后 200 段随机历史、S5 / S7 / S8 三段场景：I-9.29 全成立，被抛弃的根按实例表滤掉 |
| K2 N6（I-9.21 产品读法） | 没打中（一次抽样） | 产品容量两片墓碑容器（815 + 1）与回退之后再删：I-9.20、I-9.21 成立 |
| 第二次抽样：m3-batch21-code-r1 N3（删完之后 GPU 分叉，读代码） | 没打中（第二次） | CPU 冷走读答「没有文件」的三支（树表 0 条、没有 1 号记录、1 号 nlink = 0）在 wgsl 里逐支同序；判在它们之前的那几判两边都是走读失败 |
| 第二次抽样：m3-batch21-code-r1 N7（写调用随规模变） | 没打中（第二次）；线索（推的） | 建池的取样点最大 4 608 个单元，到不了 extent 下段长出第二层内部节点（> 21 168 个单元）那一段，只在那一段多写的实现照绿 |
| 崩溃状态全域（层 0 枚举域） | **没跑**，交主 agent | 每段历史的闭式状态数 3 145 747 到 154 140 709，一次发布的单元写那一段就有 20–26 条写（≥ 2²⁰），超过约 10⁶ 的上限 |

## 一、N4：I-9.17 / I-9.23 只红自己——打中

### 1.1 造法（全在内存稀疏盘上，模型 `opus_b22_attack.rs` 的 `n4_container_tree_entries_whose_identity_does_not_match_redden_only_their_own_invariant`）

- 墓碑树（层级 1）：新池新建文件 → 建 2 号 → 发 2 号的 unlink 意图；墓碑树根由最新那次发布写出、直接管一片容器。
- 谱系树（层级 1）：建 2 号 → 删到底 → 回退到意图之前；谱系树根由回退那次发布写出。
- 墓碑树（层级 2）：照 `container_trees_grow_a_level_when_the_root_is_full.rs` 的旋钮（一片 1 条、一个节点 2 条），建 5 个、删 2、3、4 号；池级 checker 走带旋钮值 1 的入口（I-9.21 不判）。「根环转过一圈」那两形再建 30 次，根之下那几个节点只剩现行这棵墓碑树引用。
- 改法：根的第 0 条条目改一处，根重封，新的整单元校验和沿引用链补到每一条根（`common_deletion_in_progress::change_the_unit_and_propagate_its_checksum`）。每份干净镜像先判一次，全是 `clean=[]`。

### 1.2 结果（副本上的原样输出，模型目录 `logs/n4n7.log`）

```
RESULT N4-a 墓碑树根(层级1)条目类型段1->0 clean=[] mutated(root_level=1) violated=["I-4.8", "I-7.2", "I-7.4", "I-9.23"] I-9.23=不成立（ I-9.17=不适用（
RESULT N4-b 墓碑树根(层级1)条目类型段1->5 clean=[] mutated(root_level=1) violated=["I-9.23"] I-9.23=不成立（ I-9.17=不适用（
RESULT N4-c 谱系树根(层级1)条目类型段5->0 clean=[] mutated(root_level=1) violated=["I-4.8", "I-7.2", "I-7.4", "I-9.17"] I-9.23=成立 I-9.17=不成立（
RESULT N4-d 谱系树根(层级1)条目类型段5->1 clean=[] mutated(root_level=1) violated=["I-9.17"] I-9.23=成立 I-9.17=不成立（
RESULT N4-e 墓碑树根(层级2)条目类型段0->7 clean=[] mutated(root_level=2) violated=["I-9.23"] I-9.23=不成立（ I-9.17=不适用（
RESULT N4-f 墓碑树根(层级2)条目类型段0->1 clean=[] mutated(root_level=2) violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"] I-9.23=成立 I-9.17=不适用（
RESULT N4-g 墓碑树根(层级2)类型段0条目身份容器号写1 clean=[] mutated(root_level=2) violated=["I-9.23"] I-9.23=不成立（ I-9.17=不适用（
RESULT N4-i 墓碑树根(层级1)条目类型段1->0且身份三段清零 clean=[] mutated(root_level=1) violated=["I-4.8", "I-7.2", "I-7.4"] I-9.23=成立 I-9.17=不适用（
RESULT N4-h 墓碑树根(层级2)条目类型段0->7(根环转过一圈) clean=[] mutated(root_level=2) violated=["I-9.23"] I-9.23=不成立（ I-9.17=不适用（
```

N4-j（N4-f 那一刀在根环转过一圈之后）与 N4-f 违例集合相同。N4-a、N4-f 的违例原文（`DETAIL` 行，副本上的原样输出，节选两条）：

```
DETAIL N4-a 墓碑树根(层级1)条目类型段1->0 I-7.2: 最新的根走不完：墓碑树层级 1 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状
DETAIL N4-f 墓碑树根(层级2)条目类型段0->1 I-7.2: 最新的根走不完：墓碑树的容器两份都读不到对得上的
```

### 1.3 为什么（代码，冻结副本）

- 类型段 0 那一支先按 16 KiB 读孩子取类标签，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2128`：「let child_class = read_unit_without_judging(self.reader, &child, node_bytes())」；孩子其实是 32 KiB 的码 3 容器，位置条目里的整单元校验和对不上，读回 `None`，于是这一小句放过，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2133`：「&& child_class.is_none_or(|class| class == 2);」。N4-i 身份三段清零之后 I-9.23 因此报成立。
- 接着进下一层，层级 ≤ 1 的节点记走读失败，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2191`：「if node.level <= 1 {」，原文在 `/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2193`：「"{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",」。走读失败让最新根「走不完」，I-7.2、I-4.8、I-7.4 连带。
- 类型段 = 记录类型那一支按 32 KiB 读孩子并判 I-2.1，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2247`：「data_unit_bytes(),」；孩子其实是 16 KiB 的码 2 节点，校验和对不上，I-2.1 红、记走读失败（`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:2251`：「.push(format!("{name}的容器两份都读不到对得上的"));」），身份那一判一次都没跑到，它下面整棵子树不登记，I-3.1、I-3.11、I-5.1 连带。
- 条款字面，`.claude/kb/invariants.md:310`：「0 ⇒ 子单元是码 2（根长高之后，D8（核心索引结构） 已定项 6），身份引用的出生树、容器号、出生代三段为 0；1 ⇒ 子单元是码 3」，同一行：「身份不符单独判定，不再记走读失败」（I-9.17 那一行 `.claude/kb/invariants.md:304` 同形）。正文题面，`research/prompts/_m3-batch22-code-r1-body.md:49`：「N4：身份不符的坏镜像是不是只红 I-9.17 / I-9.23，不再连带 I-7.2、I-4.8、I-7.4、I-3.1。」

### 1.4 打中之后的四句

1. 分不分辨臂：这一轮没有并列的臂；判决 T7 取 a 的那条做法没做全（类型段与孩子真实类不符那一形照旧记走读失败），不是换一条臂能躲开的。
2. 被判的系统当时看不看得到：看得到。孩子就在盘上，读得出、自证得过，只是 checker 按条目自述的类型段猜了读多长；按两种长度各试一次就知道孩子是哪一类。
3. 满足判据字面的哪一分句：N4-i、N4-f、N4-j 满足「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立」（`research/prompts/_m3-batch22-code-r1-body.md:68`：「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立，或报不适用而镜像上有它的对象」），I-9.23 报成立；N4-a、N4-c、N4-i、N4-f 另外答了 K2 N4 那一问的「否」（连带 I-7.2、I-4.8、I-7.4，N4-f 还连带 I-3.1）。连带红本身不是「误红」那一分句（误红要求从入口走得到的已发布状态，这些是坏镜像）。
4. 跑前条款给的改法：正文第五节只写「改法进下一批」，没有并列改法可逐个核。

### 1.5 改法（我提的，只在我的模型上量过结果、改法本身没实现没跑，被攻过零轮）

| 格 | 改法 | 修没修 |
|---|---|---|
| N4-a、N4-c、N4-i（类型段 0 指码 3） | 类型段 0 那一支：孩子先按 16 KiB、读不出再按 32 KiB 不判地读，读出来是码 3 就判 `identity_invariant` 红、按 2 槽登记引用、不往下走、不记走读失败；层级 ≤ 1 而孩子真是码 2 的那一形才记走读失败 | 推的 |
| N4-f、N4-j（类型段 1 / 5 指码 2） | 容器那一支：先不判地按 32 KiB 读，读不出再按 16 KiB 不判地读，是码 2 就判身份红、按 1 槽登记引用并照类型段 0 那一支把它下面的子树走完（或只登记子树引用），这一次不判 I-2.1 | 推的 |
| 同形的 inode 树 I-9.2 | 先已有之、这一轮没测（inode 树内部条目也是按类型段定读多长），列在「没打中的形状」末尾当线索 | 没量 |

## 二、N7：队列读不出、头 ID 配不上时 I-9.24 报什么

### 2.1 结果（模型 `n7_the_queue_bound_when_the_queue_is_unreadable_or_its_head_identifier_names_no_inode_tree`，起点：建 2、3 号，发 2 号意图，队列 {2}；副本上的原样输出，`logs/n7.log`）

```
RESULT N7-0 干净（队列 {2}） I-9.24=成立
RESULT N7-a 头ID写成999 I-9.24=成立 violated=["I-9.27"]
RESULT N7-c 队列加一条key3 I-9.24=不成立（根（实例 1、txg 5）：deleted_inodes（树 65544）里有 2 条 key {2, 3}，多于一条） violated=["I-9.5", "I-9.18", "I-9.24"]
RESULT N7-d 队列加一条key3且头ID写成999 I-9.24=不成立（根（实例 1、txg 5）：deleted_inodes（树 65544）里有 2 条 key {2, 3}，多于一条） violated=["I-9.27", "I-9.24"]
```

N7-b（队列根两份清零）I-9.24 报不适用，理由「走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了」，另红 I-2.1、I-4.8、I-7.2、I-7.4。

判定：头 ID 配不上时照判（读队列在配对之前），两条 key 配不上也红；队列根读不出报不适用、读不出由 I-2.1 等红。这几格与 `.claude/kb/invariants.md:311`：「每条 `deleted_inodes` 条目都判，不管树表条目的头 ID 配不配得上，干净的一版报成立」对得上。**没打中（一次抽样）**。

### 2.2 线索：形状不对的队列根，整张镜像一条都不红（漏判候选，口径交主 agent）

N7-e：在 N7-c 那份上再把 `deleted_inodes` 根的层级字节 0 → 1（条目宽、key 宽仍 8，两条条目 2、3，重封、补校验和）：

```
RESULT N7-e 队列加一条key3且根层级写成1 I-9.24=不适用（走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了） violated=[]
```

- 删除态扫描把层级不是 0 当「读不全」，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:8267`：「if root.level != 0」，I-9.5、I-4.7、I-4.6、I-9.18、I-9.24 都报不适用；这一遍的前提写在 `/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:8122`：「（I-2.1、I-1.1、I-1.7、I-1.10、I-9.1、I-9.27 等），这一遍只如实记「判不了」，不进 `walk_failures`。」——形状不对由主走读判。
- 主走读对 `deleted_inodes` 根只比条目宽（`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:266`：「TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),」那张表，I-1.10），不比层级；这张镜像上没有一条不变量红。
- 归哪一分句：按 `research/prompts/_m3-batch22-code-r1-body.md:68` 的「报不适用而镜像上有它的对象」，队列根在、两条条目在，算漏判；按 I-9.5 共用判法「读不全的版报判不了」，层级字节坏了算不算「读不全」条款没写。四句：不分辨臂；checker 看得到（层级字节就在节点头里）；分句见上，口径未定；没有跑前改法。改法（推的，零轮）：主走读给 `deleted_inodes` 根判「层级 0」（挂在 I-1.1 或 I-9.24 名下由主 agent 定），删除态扫描照旧报判不了。

## 三、N5：谱系链与死亡 txg 界

### 3.1 合法历史：写死前缀的八段场景（`n5_n9_scenarios_named_in_the_body`，`logs/scenarios.log`）

起点（只建一次）：新池新建文件 → 第一个文件顺序写成 3 个单元 → 建 2、3 号 → 建 4 号。

| 场景 | 步骤 | 结果 |
|---|---|---|
| S1 | 删 2 号到底 → 回退到意图之前 → 再回退回删完那一版 → 回退到意图那一版 → 走空 | 每一步 `violated=[]`，I-9.18、I-9.20、I-9.29 成立 |
| S2 | 删 → 回退 → 再删 → 回退到第一次删完 → 回退到第二次意图 | 同上 |
| S3 | 1 号（3 单元）意图 → 两批 → 删完 → 回退到第一批之后 → 续一批 → 回退到意图 → 回退到起点 → 再发意图 | 同上 |
| S4 | 1 号意图 → 一批 → 进程退出再挂（挂载续做到空）→ 回退到意图 → 再挂 | 同上 |
| S5 | 建 5 号 → 那次发布被崩溃恢复抛弃 → 新线再建（5 号出生代 7 → 11）→ 删 5 号 | 同上 |
| S6 | 3 号意图那次被崩溃恢复抛弃 | 造不出（挂载把藏起来的槽拿去写了，见第八节） |
| S7 | 建 5 号 → 回退到建之前 → 再建 | 新建的是 6 号（水位取 max），`violated=[]` |
| S8 | 删 2 号 → 回退到意图之前 → 回退那次被崩溃恢复抛弃 → 建 → 回退到意图那一版 | 每一步 `violated=[]` |

### 3.2 合法历史：随机扫用户动作（`n5_n9_random_legal_histories`，`logs/sweep-*.log`）

- 前缀与故障固定、用户动作放开：每一步从七种里抽——建 1–2 个、对随机一个活号发意图（先走空队列）、删一批（批大小 1）、走空、回退到最近 12 版里随机一版、进程退出再挂（新实例、挂载续做）、崩溃恢复落到前一条根（最新那次发布的根槽与它那条记录点名的单元一时读不出，见证它的系统配置轮换没落盘；挂载之后原样写回，挂载若写到了藏起来的槽，这一步算做不成）。做不成的换回动作之前那一份、记 skip。
- 每做成一步判一次全量池级 checker。旋钮：`OPUS_KNOB=0` 产品容量；1、2 是墓碑与谱系容器一片 1 / 2 条、码 2 节点 2 条、批大小 1（树会长高），checker 走带旋钮值的入口。`OPUS_PREFIX=reissue` 固定前缀「建 5 号 → 那次发布被崩溃恢复抛弃 → 新线重发 5 号」只建一次。
- 取样（复跑那一次，`rerun2`，十二批）：k0 种子 0–99（10 步）、k0+reissue 0–99、k1 1000–1099、k0 长历史 2000–2035（25 步）、k1 长历史 2000–2035、k2+reissue 3000–3099。合计 4 661 次判定；做成的动作：回退 1 431、意图 1 079、建 962、进程退出再挂 490、崩溃恢复落到前一条根 430、删一批 182、走空 83；做不成的：回退目标不是候选 137、挂载写到藏起来的槽 72、队列空 880、没有活号 54。同一个号先后见过两个出生代的历史 20 段（外加 reissue 前缀的两批每段都有）。
- 估时：先跑 3 段（22 次判定 2.8 秒）估一百段十步约 140 秒；正式一批 50 段在机器负载 90 左右时 61–100 秒，不设限时。

盯着的几条在 4 661 次判定上的判定种类（副本上数的，`logs/sweep-*.log` 的 `RESULT sweep verdict` 行相加）：

| 不变量 | 成立 | 不适用 | 不成立 |
|---|---|---|---|
| I-9.17 | 3 501 | 1 160 | 0 |
| I-9.18、I-9.19、I-9.20、I-9.23 | 各 3 296 | 各 1 365 | 0 |
| I-9.22（旋钮那四批） | 1 670 | — | 0 |
| I-9.21 | 0 | 4 661 | 0 |
| I-9.24、I-9.29 | 各 4 661 | 0 | 0 |

红的只有 I-3.1：67 次判定、37 段历史，全在做过「崩溃恢复落到前一条根」的历史里（`RESULT sweep red ["I-3.1"]` 行，例：`seed=2033 reds=["step15:[\"I-3.1\"]"]`）。不在这一轮的格里（被抛弃的根与影子账那一族，C554 那一形或我那一步造法的副作用，没分辨），照实列出，没往下追。

### 3.3 崩溃状态

- 层 0 全域：六段历史（`crash/src/main.rs` 的 `histories()`）录制之后的闭式状态数（`logs/crash-count.log`）：

```
RESULT crash H1 删->回退到意图之前->再删 marks=["i1t6", "i1t5", "i1t7", "i1t9", "i1t8", "i1t4"] writes=137 segments=20 longest_segment=26 closed_form_states=137363487
RESULT crash H4 建->回退到建之前->再建 marks=["i1t5", "i1t7", "i1t6", "i1t4"] writes=75 segments=12 longest_segment=20 closed_form_states=3145747
```

  每次发布四段 `[单元写 20–26, 记录 2, 根 1, 系统配置 2]`，单元写那一段一段就 ≥ 2²⁰。最小的 H4 也有 3 145 747 个，超过约 10⁶，照派发规则**一段都没跑全域**；`enumerate_layer0_selecting_versions_observing_each_state`（`expand` 恒真）调用已写好，状态数超过 10⁶ 时跳过。估时（推的）：段界那一跑每个状态连恢复前判、可写挂载、挂载后判、再建一个号、再判约 1 秒·线程，H4 全域按 10 线程约 90 小时。
- 补「崩在屏障段界上」那一格（不是层 0 全域、不当覆盖）：每个段界一个状态（前几段全持久、这一段一条没持久），长度 ≤ 2 的段（记录、根、系统配置）全展开，末了全持久一个；每个状态判三次：崩溃镜像本身、可写挂载（崩溃恢复）之后、挂载之后再建一个号（落到旧根之后重发）之后。六段共 249 个状态，全部挂得上，三次都零红：

```
RESULT crash H3 删->回退到意图那一版->进程退出再挂续做 boundaries:2 states=60 outcomes={"mounted": 60} reds={}
RESULT crash total_enumerated_states=249
```

  这一条腿全部原型跑的崩溃状态：段界那一跑两次共 498 个；层 0 全域 0 个。

### 3.4 违反字面的镜像：谱系记录自己坏了（线索，不算打中）

回退之后那一版，谱系容器里那条记录的 T_old 改成 T_rb、T_rb + 1（`n5_a_lineage_record_whose_parent_is_not_older_than_its_rollback`，`logs/lineage.log`）：

```
RESULT N5-L1 T_old改成T_rb violated=[] I-9.18=成立 I-9.19=不适用（走读 I-9.20=不适用（走读 I-9.29=不适用（走读
```

谱系链那一步把它当判不了（`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:9101`：「if some_record_is_not_younger_than_its_parent || some_rollback_has_two_records {」），三条报不适用，整张镜像不红。`invariants.md` 里没有一行判谱系记录本身（I-9.17 判的是条目身份），按字面不算漏判；它与 2.2 是同一种口子：「判不了」那一支的前提是别处会判形状，而这一处没有别处。

## 四、N9：I-9.29

- 固定前缀（只建一次）：建 5 号（出生代 7）→ 那次发布被崩溃恢复抛弃（新实例 2，txg 跳到 10）→ 新线再建，重发 5 号（出生代 11）。被抛弃的根（实例 1、txg 7）之后读得出，按最新根的实例表判抛弃、滤出候选集。前缀那一版：`RESULT sweep prefix=reissue records=[(1, 1, 3), (2, 1, 5), (3, 1, 5), (4, 1, 6), (5, 1, 11)] violated=[]`。
- 之后两批（产品容量、k2）各 100 段随机历史，回退到被抛弃那一版（实例 1、txg 7）的 58 次都被拒（`TargetNotACandidate`），I-9.29 每次判定都成立；无前缀的几批里另有 20 段历史自己走出了重发（崩溃恢复落到建号之前、新线再建），同样成立。S5、S7、S8 三段场景见 3.1：管理员回退之后再建拿到的是新号（S7 建出 6 号），I-9.29 成立。
- 读代码：版本清单是候选根加「由记录施加出来的那几版」，后者要求记录的 txg 不超过实例表那一行的 Ti；表与扫描都按最新根的实例表滤，`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker/src/walk.rs:9856`：「fn judge_one_object_birth_per_inode_number(」只在全部版本扫完之后判一次。
- **没打中（一次抽样）**。中间踩到过一次假红（I-9.29 与 I-2.1 等一起红）：我第一版造「崩溃恢复落到前一条根」时，挂载已经把藏起来的槽写了新东西，我又把旧字节写回去，盖掉了新实例的实例表；那段历史不合法。修法是挂载写过藏起来的地方就算这一步做不成（`abandon_the_newest` 里那一判），之后的数都是修过之后跑的。

## 五、N6：I-9.21 产品读法

产品容量，建 820 个空 inode，删到底 816 个：墓碑树两片容器（815 + 1）；再回退到第二片出现之前、再删一个（`n6_a_product_tombstone_tree_of_two_containers`，`logs/n6.log`）：

```
RESULT N6-a 两片墓碑容器 violated=[] I-9.20=成立 I-9.21=成立
RESULT N6-b 回退之后再删 violated=[] I-9.20=成立 I-9.21=成立
```

墓碑树随现行那一版、只追加，回退不缩它；我没找到别的合法路径让非最右那片不满。**没打中（一次抽样）**；随机扫那几批里 I-9.21 一次都没判到（都是零根或一片），它只靠这一条用例。

## 六、第二次抽样

### 6.1 m3-batch21-code-r1 N3（删完之后 GPU 分叉，读代码，GPU 不跑）

CPU 冷走读答「没有文件」的三支：树表 0 条、inode 树里没有 1 号的记录、1 号 nlink = 0（`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-core/src/recovery.rs:5077`：「if inode_record.nlink == NLINK_OF_AN_INODE_BEING_DELETED {」）。wgsl 里同三支同序：`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2972`：「if (tree_table.entry_count == 0u) {」、`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:3287`：「if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {」。排在它们前面的几判逐个对过：extent 上段零条目（只许上段根兼叶，两边同判）、上段恰好一条、映射条目数（两边都把 `deleted_inodes` 根算 1、墓碑树与谱系树算每个码 2 节点加每片容器）、记账条目数 3 + 6 × 盘数、inode 树根层级 1；两边在这些上都是走读失败，结局同为「失败」，不会一边答没有文件、一边不答。**第二次也没打中，记「没打中」**（这一判只是读代码；对拍归提交时）。

### 6.2 m3-batch21-code-r1 N7（写调用随规模变）

`unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 今天钉的是：快档 1、144、145、432、433 个单元每一次发布的写调用绝对值，慢档 2304、4608 个单元最多那次（45、53），外加 W 式子的常数上界。按条款错的「批大小随规模变」那一刀已由 2⁴⁰ 那条纯算用例罩住。写调用这一边：

- 线索（推的，没实现没跑）：extent 下段内部节点扇出 147（`/tmp/claude-1000/m3-batch22-code-r1/tree/crates/singlefs-format/src/lib.rs:168`：「pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;」），一个下段内部节点罩 147 × 144 = 21 168 个单元；建池的取样点最大 4 608 个，下段一直只有一层内部节点。一个只在下段长出第二层之后才多写的实现（例如每批把第二层内部节点的孩子整层重写，多出至多 147 × 2 盘次写调用），七个取样点上与产品逐次相同，W 式子的上界也罩得住它（上一轮判决记的上界 31 + 748），三条用例照绿。
- 这与上一轮记下的「上界几乎拦不住」是同一件事，不是新形状；我没另造这个错实现去跑（要 2 万个单元以上的文件，两块盘各 0.7 GiB 写进内存稀疏盘，压在 4 GiB 上限边上）。**第二次也没打中，记「没打中」**；斜率怎么读仍归 C2 那一问。

## 七、没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| N4 | 类型段写成条款外的 7（墓碑树层级 1、层级 2、层级 2 且根环转过一圈）；墓碑树 1→5、谱系树 5→1（对方那棵树的类型段）；层级 2 根的类型段 0 条目身份引用容器号写 1 | 各一份镜像，都只红自己 |
| N5 合法历史 | 八段场景 S1–S8（含删 → 回退到意图之前 → 再回退回删完、删 → 回退 → 再删、删除中间态回退与续做、进程退出再挂、建号 / 意图 / 回退那次被崩溃恢复抛弃）；十二批随机历史 4 661 次判定（三档容量、两种前缀、10 步与 25 步） | 种子范围见 3.2 |
| N5 崩溃 | 六段历史的段界与短段（≤ 2 条写）全展开，249 个状态，恢复前、恢复后、恢复后再建号各判一次 | 层 0 全域没跑（3.3） |
| N7 | 头 ID 写成没有的树、队列根两份清零、两条 key、两条 key 加头 ID 配不上 | 各一份 |
| N9 | 固定「建号被抛弃 → 重发」前缀之后 200 段随机历史；管理员回退到建号之前再建 | 见第四节 |
| N6 | 产品容量 815 + 1 两片容器、回退之后再删 | 一份 |
| 第二次抽样 | 第 21 批 N3 逐支读；N7 按取样点推 | — |
| 没测的同形（线索） | inode 树内部条目 I-9.2 也按类型段定读多长（`walk_inode_internal_node`），类型段与孩子真实类不符时大概率同 N4-f 那样连带；这一批没改它、我也没造镜像 | 没量 |

## 八、这一条腿自己的限度

- **崩溃状态没跑层 0 全域**：每段历史 ≥ 3.1 × 10⁶，按规则不跑；段界那一跑是我自己挑的子集（每个长段只取段界），不是层 0 的覆盖，不能拿它说「崩溃状态没打中」。要不要放量跑哪一段，交主 agent（下面「要主 agent 跑的命令」第 2 条）。
- 「崩溃恢复落到前一条根」是我模拟出来的（藏根槽与点名单元、改写系统配置轮换，照 `common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 那一形，藏的东西换成了点名单元）：挂载若把藏起来的槽拿去写，这一步算做不成（72 次），这一族（C554 那一形）没进样本。S6（意图那次被抛弃）因此一步都没造出来。
- 随机历史里用户动作的分布是我定的（回退最多回看 12 版、崩溃恢复约 9%），没有按「哪一步最可能打中」加权；批大小只取 1 与产品。
- I-3.1 那 67 次红没往下追，不知道是实现、checker 还是我造法的问题。
- 所有数都是副本上量的，不是入库装置上的数。
- 第 21 批 N3 那一格只读了代码，GPU 没跑。

## 九、没做什么

- 不判正推那几格（K1、K3）；不替主 agent 采纳；改法都只写了、没在副本上实现（零轮）。
- GPU 不跑；checker 档测试不跑；层 0 全域不跑。
- 没读禁读清单里的文件（正推腿的报告与模型、`/tmp/claude-1000/m3-batch22-code-r1-sonnet/`）；上一轮攻方报告也没读（第二次抽样只读了上一轮判决）。

## 要主 agent 跑的命令

1. 复跑模型（工作目录任取一个空目录，`SINGLEFS_REPO` 指冻结副本、`SINGLEFS_TOOLS` 指有 `research/scripts/` 的仓）：
   `SINGLEFS_REPO=/tmp/claude-1000/m3-batch22-code-r1/tree SINGLEFS_TOOLS=/home/fy5090/code/singlefs bash research/prompts/m3-batch22-code-r1-opus-model/rerun.sh <空目录> all`，再 `bash research/prompts/m3-batch22-code-r1-opus-model/rerun.sh compare <空目录>`，看末行 `SUMMARY_MATCHES 907 行`。编译 6G、跑 4G、线程 `OPUS_THREADS`（默认 10）；我分段跑完一遍，各段 8–100 秒（机器负载 90 左右时）。
2. 若要层 0 全域：`crash/src/main.rs` 已接好 `enumerate_layer0_selecting_versions_observing_each_state`（`expand` 恒真），状态数超过 10⁶ 时跳过；放开要改那一个阈值，最小的 H4 一段 3 145 747 个状态，推的约 90 小时（10 线程）。我不建议为这一格放量（推的）：单元写那一段里的子集，根与记录都没持久，镜像上只多了没被引用的单元。
