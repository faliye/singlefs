# m3-batch26-code-r1 攻方腿报告（Opus，K2：重点 H1、H3，另做 m3-batch25-code-r1 K2 末条那几格的第二次抽样）（2026-10-06）

立场：攻方，造反例。只跑 harness 档，GPU 不跑（GPU 那一侧读 wgsl、用 CPU 算期望）。所有数都在冻结副本的拷贝上量（副本上的数，不算入库装置上的数）。
冻结副本 `/tmp/claude-1000/m3-batch26-code-r1/tree/` 的 5 份被判文件与 `research/prompts/m3-batch26-code-r1-snapshot/crates-sha256.txt` 开工时 `sha256sum -c` 逐个 OK。
引代码的行号都是冻结副本里的行号（下文写 `crates/…` 时，根是 `/tmp/claude-1000/m3-batch26-code-r1/tree/`）。

## 复跑

模型在 `research/prompts/m3-batch26-code-r1-opus-model/`。仓路径取 `SINGLEFS_REPO`；冻结副本只有 `crates/` 与 `Cargo.*`，包装脚本（`run-with-memory-cap.sh`、`capped.sh`）另从变量 `SINGLEFS_TOOLS=<仓根>` 取（`rerun.sh` 文件头写明）。编译 6G、跑 4G，线程经 `capped.sh`（`OPUS_THREADS`，默认 10），用自己的编译目录（工作目录下 `target-new/`、`target-old/`、`target-fix/`）；每段前台四分钟以内。

```
SINGLEFS_REPO=/tmp/claude-1000/m3-batch26-code-r1/tree SINGLEFS_TOOLS=<仓根> bash research/prompts/m3-batch26-code-r1-opus-model/rerun.sh <空工作目录> <段|all>
bash research/prompts/m3-batch26-code-r1-opus-model/rerun.sh compare <工作目录>
```

段：`setup build-new build-old build-fix h-new h-old h-fix u-new u-old u-fix v p a2 k b1 b2 kbi`，之后 `summary`（`compare` 自己先做 summary，不读 `SINGLEFS_REPO`，比之前两边各自排序）。
三份源码树：`new` = 冻结副本原样；`old` = 冻结副本反打 `batch26-walk.patch`（第 26 批 `walk.rs` 那两处改动之前，反打之后的 `walk.rs` 的 git blob 是 `535c799ae2dab02a06ade9ea681523d3b97e6d4c`，与改动原样里 `index 535c799..e7562dd` 的前者相同）；`fix` = `new` 打 `fix-FB-FV.diff`（攻方改法，零轮）。只有 `walk.rs` 三份不同；GPU 那一侧没有 old / fix。

交回前用仓里这份模型在两个空目录（`/tmp/claude-1000/m3-batch26-code-r1-opus/rc1`、`rc2`）逐段从头各跑一遍：`expected-summary.txt` 取自 rc1，`compare` 在 `/tmp` 下、不带 `SINGLEFS_REPO` 与 `SINGLEFS_TOOLS` 对 rc2 跑，原样输出 `SUMMARY_MATCHES 8193 行`。

| 文件 | sha256 |
|---|---|
| `batch26-walk.patch` | `4f78af196a5aa22b59fce2db0cbde3404823269317d567af4ecb6196c9d73514` |
| `expected-summary.txt` | `632502a92e2d48284348b9354c8c44dc5a04f3ba95942e980dd4b7e0ce5b803f` |
| `fix-FB-FV.diff` | `72b341fdd0aacefd71d6abb6cc61547c97b68cd556a127aa48a6a3cfaf188998` |
| `opus_b26_attack.rs` | `42ce58d7e56b1a4d1f588b6d9b88647ab1ab6f076ac36bbdbdd082843e601e53` |
| `rerun.sh` | `28dc5d72caed20548a4faf08da03a5edaeebd1d2ee9c13e824a8acb8cbe096ca` |

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K2 H1：新写法让结论随陈旧副本在哪块盘翻面而原写法不翻 | 没打中（一次抽样） | 8 张起点、31 条条目 × 3 种条目改法 × 16 种副本状态对（1 488 形）：新写法盘序翻面 0 次（原写法 31 次）；能让新写法翻面的只有下一行那一形，原写法同样翻 | 一 |
| K2 H1：两份不同类、各自按位置条目自证得过 | **打中 H-V**（两处故障的坏镜像；新旧写法同样翻） | 父条目里一条位置条目的整单元校验和配上另一类单元：目标那条随那一份在盘 0 还是盘 1 翻面，类型段改坏时在一种盘序上漏判 | 1.3 |
| K2 H1：陈旧副本那一份的 I-2.1 | 没打中（坏镜像上只在手造、中央映射树没收的单元上缺） | 走「孩子是另一类」那一支时主走读不读两份、不判 I-2.1；池造出来的镜像上中央映射树那一遍照判 | 1.4 |
| K2 H3：读不出孩子时 I-9.2 误红 | 没打中（一次抽样） | 坏镜像 142 形 + 合法历史 148 段 1 883 次判定：读不出孩子那一支没有一次在合法历史上被走到，坏镜像上没有误红 | 二、四 |
| K2 H3：读不出孩子时该红不红 | **打中 H-B**（坏镜像；I-9.17 / I-9.23 那一侧本批之前就这样、CPU 与 GPU 同） | 容器类型段的条目，孩子两份都读不出时「子指针头部的出生树 == 身份引用的出生树」不看孩子就判得了，却不判；`walk.rs` 新函数注释说那几小句「都要比孩子的头」 | 2.2 |
| K2：CPU 与 GPU 判得不一样 | 没打中（读码，一次抽样） | H1 新写法、H3 三支、H-V、H-B 在 GPU 上逐支与 CPU 同；GPU 只走 inode 根那一层，读得出的类型段 0 孩子记判不了 | 三 |
| 第二次抽样：m3-batch25 K2「走到末片而没滤全」 | 第二次也没打中 → 记「没打中」 | 换 13 形（三片、同一实例两行、链指针记录在片中间、第 1 片是第 0 片的拷贝、位置条目盘序倒置）：门开的每一形都按整张表滤 | 五 |
| 第二次抽样：m3-batch25 K2 N2 GPU 其余各支 | 第二次也没打中 → 记「没打中」 | 重读走读内核的链走读：读、走过、容器、身份、链指针记录、置位、行，逐支与 CPU 同（256 片上限那一支已归第 27 批） | 五 |
| 第二次抽样：m3-batch25 K2 合法历史 | 第二次也没打中 → 记「没打中」 | 换种子、加 inode 两片叶的起点：148 段、1 883 次判定，I-9.29 全部成立、旗 0 | 四、五 |
| 改法 F-B、F-V | 只在我的模型上量过、被攻过零轮 | F-B 修 H-B 的 32 + 90 格，F-V 修 H-V 的 36 格；`checker_known_bad_images` 130 条照绿 | 六 |
| 崩溃状态 | 没跑 | 最小的一段合法流（新建一个号）就有 1 048 583 个状态，过约 10⁶ | 四 |

## 一、H1：孩子两份副本的状态 × 条目改法（H 段、V 段）

### 1.1 起点与装置

起点都由合法历史在内存稀疏盘上造（`opus_b26_attack.rs` 的 `clean_image_named`），一个字节都不改：T1 / T2 / T3 是墓碑树层级 1 / 2 / 3（旋钮一片 1 条、节点 2 条，建 4 / 5 / 7 个号、删 2 / 3 / 5 个），L1 / L2 是谱系树层级 1 / 2（回退 2 / 3 次），I0 是 inode 树根层级 1、两片叶（建 150 再建 150 个号），I0r 是 I0 之后删 5 号再回退到删之前（回退之后的那一版）。I2 是手造的层级 2 inode 树：合法流里 inode 树不长高（容器数超过一个根装得下的 135 片就拒），我把 I0 的层级 1 根原样搬到槽 180 000、原根槽写一个层级 2 的码 2 根（唯一一条条目类型段 0、身份三段 0），整单元校验和沿引用链补到根槽；这张干净镜像只红 I-3.1、I-3.11（槽 180 000 不在记账里，手造的痕迹），I-9.2 成立。

每棵树每个节点取第 0 条与最后一条条目（共 31 条），条目改法三种：不改（Keep）；类型段换成另一个登记过的值（Flip：容器类型段 → 0 并清身份三段，0 → 容器类型段）；类型段写 7（Unreg）。孩子两份副本各取四种状态之一：R 原样；S2 前 16 KiB 换成这棵树根（码 2）的原样字节；S3 前 16 KiB 换成最新根实例表第 0 片（码 3）的前 16 KiB；Z 前 16 KiB 写零。陈旧的那几份都不补校验和，按哪一类的长度都自证不过。每形判一次池级 checker（墓碑、谱系走旋钮入口），新旧两种写法各一遍。三项比对（`h_copy_states_by_entry_changes`）：
- 盘序：(c0, c1) 与 (c1, c0) 两张镜像只差哪一份在盘 0，目标那条的判定与违例集合要相同；
- 目标那条照字面：有一份 R 时条目改了就红、没改不红；两份都不是 R 时只看不看孩子就判得了的那几小句（类型段不认得 ⇒ 红；类型段 0 ⇒ 身份三段非零才红；容器类型段 ⇒ 子指针头部出生树 ≠ 身份出生树才红）；
- 违例集合 = 「条目不改、副本状态相同」那张对照的违例集合 ∪ 目标（多的、少的都列）。

### 1.2 结果：新写法上没有一形随盘序翻面

`expected-summary.txt` 里 `RESULT H-tally` 各行按写法相加（命令：`grep -h "RESULT H-tally <写法> " expected-summary.txt | awk '{k=$6" "$7; s[k]+=$8} END {for (k in s) print k, s[k]}'`）：

| 写法 | 盘序相同 / 翻面 | 目标照字面 / 不照 | 违例集合相同 / 不同 |
|---|---|---|---|
| old（第 26 批之前） | 527 / **31** | 1 304 / 184 | 743 / 249 |
| new（冻结副本） | 558 / **0** | 1 398 / 90 | 782 / 210 |
| fix（new + F-B + F-V） | 558 / 0 | 1 488 / 0 | 791 / 201 |

old 的 31 次翻面都是 Flip 加一份陈旧、陈旧那份头里恰是改后类型段那一类（第 24 批攻方 H1 那一族），新写法上同形两种盘序都红、违例集合相同，原样输出：

```
[h-old-I0] RESULT H-order old FLIP I0 d0 L1 e0/2 t2 Flip R/S2 target=不成立/成立 red={"I-2.1", "I-4.8", "I-7.4", "I-9.2"} swapped={"I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"}
[h-new-I0] RESULT H new I0 d0 L1 e0/2 t2 real3 Flip c0=S2 c1=R target=不成立 red={"I-2.1", "I-4.8", "I-7.4", "I-9.2"}
[h-new-I0] RESULT H new I0 d0 L1 e0/2 t2 real3 Flip c0=R c1=S2 target=不成立 red={"I-2.1", "I-4.8", "I-7.4", "I-9.2"}
```

new 的 90 格「目标不照字面」全是同一族：层级 ≥ 2 的根（T2、T3、L2、I2）里类型段 0 的条目被 Flip 成容器类型段、孩子两份都不是 R（`grep -h 'H-target new MISS' expected-summary.txt` 逐行都是 `t0` 且两份都不是 R）——那是第二节的 H-B，不是 H1。new 的 210 格「违例集合不同」分三类（逐行在 `H-set new DIFF`）：孩子两份都读不出时多红 I-3.1 / I-3.11 / I-5.1（类型段改坏之后按类型段登记的槽数与孩子真占的不符，读不出就认不出；old 同样有，不是这一批改的）；上面 90 格少红目标；I2 上 12 格少红 I-2.1 / I-4.8 / I-7.4（见 1.4）。

读码对照（推的，与上表相符）：新写法 `child_unit_class_other_than_the_entry_type_names` 的结论只在「两份都按各自位置条目自证得过、而类不同」时依赖次序——第一份自证得过的那一份决定交回什么（`crates/singlefs-checker/src/walk.rs:4113`「for location in &child.locations {」，`crates/singlefs-checker/src/walk.rs:4129`「if Some(class_in_the_header) == class_the_entry_type_names {」）。同一个位置条目对两份写同一个整单元校验和时，两份不可能都自证得过又不同类（除非 CRC 碰撞），所以单处陈旧、单处坏都翻不了面。

### 1.3 打中 H-V（两处故障的坏镜像）：两份按各自位置条目都自证得过、却是两类

造法（`v_both_copies_verify_as_different_classes`）：孩子是码 3 容器的条目，把它的一条位置条目的整单元校验和改成这棵树根（码 2）那 16 KiB 的 CRC、重封到根，再把那块盘上孩子的槽写成那 16 KiB；另一块盘那份不动。条目不改 / 改成类型段 0（清身份）两种，那一份在盘 0 / 盘 1 两种。四张起点 9 条条目 × 4 = 36 形，新旧写法输出逐行相同（`[v-new]` 与 `[v-old]` 去掉写法名之后一样）。I0 第 0 条的原样输出：

```
[v-new] RESULT V new I0 d0 L1 e0/2 t2 Keep verifying-node-on-dev0 target=不成立 red={"I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4", "I-9.2"}
[v-new] RESULT V new I0 d0 L1 e0/2 t2 Keep verifying-node-on-dev1 target=成立 red={"I-2.1", "I-4.8", "I-7.4"}
[v-new] RESULT V new I0 d0 L1 e0/2 t2 Flip verifying-node-on-dev0 target=成立 red={"I-1.1", "I-1.2", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4"}
[v-new] RESULT V new I0 d0 L1 e0/2 t2 Flip verifying-node-on-dev1 target=不成立 red={"I-2.1", "I-4.8", "I-7.4", "I-9.2"}
```

同一张镜像只差那一份在哪块盘，I-9.2 翻面；条目改坏（Flip）时在盘 0 那种上 I-9.2 报成立（漏判）、另连带 I-7.2 等——与第 24 批 H1 判决那一行「结论随陈旧副本在哪块盘翻面」同形，只是这里两份都自证得过。墓碑（I-9.23）、谱系（I-9.17）同样 36 形里每对都翻（`grep -h 'RESULT V new' expected-summary.txt` 逐对看）。

四句：
- 分不分辨臂：不分辨「第 26 批前 / 后」——新旧写法逐行相同；分辨「只看第一份自证得过的」与「两份都看」（F-V 上 36 形全红、不翻面，第六节）。
- 系统当时看不看得到判别它的东西：看得到。两条位置条目、两份内容都在盘上，按各自校验和都对得上；checker 看第二份就知道有一份是另一类。
- 满足判据的哪一个分句：本轮正文 K2 第一条「两份各是不同类、各自自证过」；结论随盘序翻面，同第 24 批判决 K2 N1 两故障 H1 的判法。不是「新写法翻面而原写法不翻」那一句——原写法同样翻。
- 改法在这几格上还中不中：第 24 批判决给的 F1（即本批实现）对这几格不起作用（本批就是 F1，照样翻）；F-V 修。

分量（推的）：要两处相关的故障——父指针两条位置条目的校验和不同、且其中一块盘上恰是与那条校验和对得上的另一类单元。写路径对两份写同一个校验和（合法流碰不到）；正常的陈旧、撕裂、坏扇区都造不出「另一份也自证得过」。交主 agent 定要不要管。

### 1.4 没打中：「孩子是另一类」那一支不判陈旧那份的 I-2.1

新写法认出孩子是另一类就判目标、登记引用、`continue`，不再走 `read_referenced_unit`，陈旧那一份在这一处不判 I-2.1。I2 上（孩子是手造的槽 180 000，只有那一条条目引用它）12 格因此比对照少红 I-2.1 / I-4.8 / I-7.4，原样一行：

```
[h-new-I2] RESULT H-set new DIFF I2 d0 L2 e0/1 t0 Flip c0=S3 c1=R missing=["I-2.1", "I-4.8", "I-7.4"] extra=[]
```

但池造出来的镜像上一格都没缺：中央映射树那一遍按映射条目读每个单元的两份、照判 I-2.1（`p-1` 那一形的细节：`[p-1] RESULT P new I0 d0 L1 e1/2 t2 child_slot=50268 Flip c0=S2 c1=R I-2.1: 映射条目指的单元 在盘 0 槽 50268 的那一份与位置条目里的校验和对不上`）。手造的槽不在映射树里才缺，判为装置造出来的，不算打中。

## 二、H3：inode 树读不出孩子（U 段，H 段里两份都不是 R 的那几格）

### 2.1 起点与装置

U 段（`u_unreadable_child_against_the_literal_clauses`）：七张起点（T1 T2 L1 L2 I0 I0r I2）每个节点第 0 条与最后一条，七种条目改法：不改、类型段 0 且清身份（T0Z）、类型段 0 不清身份（T0N，原来就是 0 的写容器号 1）、类型段 7（U7）、子指针头部出生树 + 1（BT）、身份出生树 + 1（IDB）、身份容器号 + 1（IDC）；改了等于没改的跳过。每形孩子 (R, R)、(Z, R)、(R, Z)、(Z, Z) 四种，比「读得出」与「两份都读不出」时目标那条，两份都读不出时再与条款字面里不看孩子就判得了的那几小句比（同 1.1）。层级覆盖：inode 根层级 1（I0、I0r，含回退之后那一版）、手造层级 2 的根与它下面的层级 1 节点（I2，即「根长高之后」）；墓碑、谱系层级 1、2 作对照。

### 2.2 打中 H-B（坏镜像）：容器类型段的条目、孩子两份都读不出，「子指针头部的出生树 == 身份引用的出生树」不判

`I-9.2` 字面：「子指针头部的出生树 == 身份引用的出生树，且子容器的打包记录类型 == 2」（`.claude/kb/invariants.md:289`），这一小句只比父条目里的两段，不看孩子。新函数注释写「类型段 2 那几小句都要比孩子的头，读不出不判」（`crates/singlefs-checker/src/walk.rs:4140`），代码在容器类型段那一支什么都不判（`crates/singlefs-checker/src/walk.rs:4148`「InodeInternalEntryType::LeafContainer => {}」）。读得出孩子时同一小句是判的（`crates/singlefs-checker/src/walk.rs:2555`「&& child.birth_tree == identity.0;」）。

U 段计数（`grep -h "RESULT U-tally <写法> "` 相加）：new 142 形里 32 形「HIT-漏判」、110 形对得上；old 48 / 94；fix 0 / 142。new 的 32 形全是 BT 与 IDB 两种改法、容器类型段的条目、两份都读不出——读得出、或只有一份读不出时都红，两份都读不出就报成立。原样（各树取一条）：

```
[u-new-I0] RESULT U-cmp new HIT-漏判 I0 d0 L1 e0/2 t2 BT readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
[u-new-I0] RESULT U-cmp new HIT-漏判 I0 d0 L1 e0/2 t2 IDB readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
[u-new-I2] RESULT U-cmp new HIT-漏判 I2 d1 L1 e0/2 t2 BT readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
[u-new-T1] RESULT U-cmp new HIT-漏判 T1 d0 L1 e0/2 t1 BT readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
[u-new-L1] RESULT U-cmp new HIT-漏判 L1 d0 L1 e0/2 t5 IDB readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
```

H 段那 90 格是同一小句的另一种走法：层级 ≥ 2 的根里类型段 0 的条目被改成容器类型段（身份三段仍是 0，子指针出生树是树 ID），孩子两份都不是 R，目标报成立：

```
[h-new-T2] RESULT H-target new MISS T2 d0 L2 e0/2 t0 Flip c0=Z c1=Z target=成立 expected_red=true
```

墓碑树、谱系树那一侧（I-9.23「子指针头部出生树 == 身份引用出生树、子容器打包记录类型 == 1」，`.claude/kb/invariants.md:310`）本批之前就这样：容器读不出时只记走读失败（`crates/singlefs-checker/src/walk.rs:2363`「.push(format!("{name}的容器两份都读不到对得上的"));」）；本批把 inode 树对齐到它，连这一缺口一起对齐了。GPU 同：inode 根读不出孩子那一支类型段 2 不判（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2634`「类型段 0 判身份引用三段为 0，类型段不认得的判该条目损坏，类型段 2 不判」），容器树读不出容器只加走读失败（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3085`「let s = w_read_referenced_unit(child, DATA_UNIT_BYTES);」之后那一支）。

old 比 new 多的 16 形都在 inode 树上：类型段 0 而身份非零（T0N 7 形，I2 根那条类型段 0 条目的 IDB、IDC 各 1 形）与类型段不认得（U7 7 形），本批 H3 修好的那一族，new 上都红了，例：

```
[u-old-I0] RESULT U-cmp old HIT-漏判 I0 d0 L1 e0/2 t2 T0N readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=成立 literal-without-child=红
[u-new-I0] RESULT U-cmp new ok I0 d0 L1 e0/2 t2 T0N readable=不成立 one-copy-unreadable=不成立/不成立 both-unreadable=不成立 literal-without-child=红
```

四句：
- 分不分辨臂：不分辨新旧写法（inode 树上 old 一句都不判，new 判了类型段 0 与不认得两支、漏了这一小句）；分辨「不看孩子就判得了的小句全判」与「容器类型段那一支整支不判」（F-B 上 0 形）。
- 系统当时看不看得到：看得到。两段都在父条目里（身份出生树是条目偏移 8，子指针出生树是条目里子指针头部那 8 字节），读不读得出孩子不影响。
- 满足判据的哪一个分句：本轮正文 K2 H3 那条「有没有一形 I-9.2 误红或该红不红」的「该红不红」；口径同 m3-batch22-code-r1 正文第五节的漏判（坏镜像上该红的不红）。另外它与新函数自己的注释「不看孩子就判得了的那几小句照判」相抵。
- 改法在这几格上还中不中：本批的实现（judge_an_inode_entry_whose_child_does_not_read_back）在这几格上不起作用；F-B 修（第六节）。

分量（推的）：要条目里出生树那两段之一坏、且孩子两份都读不出；孩子两份都读不出时 I-2.1、I-7.2 等已经红，镜像不会被当成好的，漏的是「哪一条不变量红」的归因。

### 2.3 没打中：读不出孩子时误红

U 段 new 上没有一形「HIT-误红」（142 形里 0），I2 根那条类型段 0 的条目（根长高之后）T0Z 不红、T0N / IDB / IDC 红，字面相符。合法历史（第四节）上读不出孩子那一支一次都没被走到：1 883 次判定里 I-4.8、I-7.4、I-7.2、I-2.1、I-9.2 一次都没红（这几条有一条红才可能有孩子读不出）。

## 三、CPU 与 GPU：读 wgsl 逐支对 CPU，用 CPU 算期望（GPU 不跑）

| 支 | CPU（`walk.rs`） | GPU（`crash_judge_checker.wgsl`） | 同不同 |
|---|---|---|---|
| 认孩子的类：读头的长度、自证的长度、次序 | 两条位置条目依次，头读 16 KiB，按头里那一类的长度验各自的整单元校验和，第一份自证过的定结论 | 同：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2964`「if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {」，再看第二份；码 1 / 码 3 按 32 KiB 的 CRC 且要落点够 32 KiB（位 4 是「够 32 KiB」，`crash_judge_tables.rs` 现查） | 同 |
| 两份都自证不过 | 交回 None，按类型段读，读不出归 I-2.1 | 交回 NONE，同 | 同 |
| H-V（两份都自证得过、不同类） | 第一份定结论（翻面） | 同（翻面同向） | 同 |
| inode 读不出孩子：类型段 0 / 不认得 / 2 | 身份三段 / 恒红 / 不判 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2637`「} else if (record_type != 2u) {」一支恒红，类型段 0 判三段，类型段 2 不判 | 同（H-B 两边都漏） |
| inode 类型段 0、孩子读得出且是码 2 | 往下走（层级 1 的根下面走不下去记走读失败） | 记判不了（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2753`「undecidable = UNDECIDABLE_TOO_DEEP;」），回落 CPU | GPU 不判，不算分叉 |
| inode 根层级 ≥ 2、类型段 0 的孩子全读不出 | 每条判身份三段、记走读失败 | 同，且这一版 GPU 判得了（没有读得出的类型段 0 孩子才走不到判不了） | 同（推的） |
| 墓碑 / 谱系容器读不出 | 只记走读失败 | 同 | 同（H-B 两边都漏） |

没找到一形 CPU 与 GPU 判得不一样（一次抽样，读码）。GPU 走读只看 inode 根那一层的条目，第 26 批对拍那条流（叶容器盘 0 那份陈旧）CPU 红 I-9.2、不红 I-7.2，GPU 按上表同判；H-B、H-V 两族在 GPU 上照 CPU 同样漏、同样翻（推的）。

## 四、合法历史与崩溃状态

### 4.1 合法历史：用户动作放开扫（B 段，没打中，一次抽样）

造法照抄 m3-batch25-code-r1 攻方原型 B 段（`run_history_b`）：每一步从九种用户动作里随机抽——建 1 / 2 个号、对一个活号发意图、删一批、走空、回退到前 1–12 新的一版（被抛弃的那几版也抽，该被拒）、再挂、崩溃恢复落到前一条根、先建一个号再崩溃恢复抛弃它、连着再挂 5 次；被拒的换回动作之前那一份，做成的判一次。只固定起点：S0（新池新建文件、建 2、3 号）、SX（被崩溃恢复抛弃的那一版与现行线同号不同出生代）、SK（S0 装删除开关，判定走旋钮入口）、SI（新加：inode 树两片叶，建 300 个号）。旗在上一轮那几面（I-9.29 不成立或部分没判、九条红、缺省理由、不适用说假话、I-9.24 不适用）之外加了 I-9.2、I-4.8、I-7.4、I-7.2、I-2.1 红（`extra_flags`）。种子 9101–9425、9501–9812，与 m3-batch24 / 25 攻方用过的不重。

| 批 | 起点 | 段 × 步 | 判定次数 | 暴露 | I-3.1 红 | 旗 |
|---|---|---|---|---|---|---|
| b1 | S0 / SX / SK / SI | 25 × 12 各 | 231 / 242 / 251 / 244 | 42 / 207 / 32 / 33 | 21 / 8 / 15 / 16 | 0 |
| b2 | S0 / SX / SK / SI | 12 × 25 各 | 232 / 229 / 227 / 227 | 41 / 140 / 60 / 49 | 41 / 41 / 37 / 27 | 0 |

合计（`grep -hE '^\[b[12]-S.\] RESULT B tally \S+ checks='` 各行相加）：148 段、1 883 次判定（含 8 次起点），I-9.29「成立」1 883 次；旗 0 条（`grep -hE '^\[b[12]-S.\] RESULT B tally \S+ flag' expected-summary.txt | wc -l` 为 0），148 段每段 `flags=[]`。「暴露」604 次：判的那一刻根环里还有一版被抛弃的、上面有号与现行那一版同号不同出生代，门开着、按实例表滤掉了它。九条之外红的只有 I-3.1，206 次判定（同 m3-batch22 / 24 / 25 判决「附带」那一族，已转根环线，这里没逐段归因）。

估时与分批：探针四个起点各 2 段 × 8 步，一步约 0.4–0.6 秒；每批四件并行、每件 `capped.sh 2`，按探针估一件 1.5–2 分钟；实测 b1 墙钟 92 秒、b2 101 秒。SR 起点（实例表两片，要先再挂 370 次）这一轮没跑，上一轮跑过 47 次判定。

### 4.2 崩溃状态：没跑

`k_count_crash_states_of_small_flows` 只录、只数（`closed_form_state_count`），原样：

```
[k] RESULT K SI 上建 1 个号 writes=25 segments=4 segment_lengths=[20, 2, 1, 2] closed_form_states=1048583
[k] RESULT K SI 上对 5 号发意图 writes=31 segments=4 segment_lengths=[26, 2, 1, 2] closed_form_states=67108871
[k] RESULT K S0 上建 1 个号 writes=25 segments=4 segment_lengths=[20, 2, 1, 2] closed_form_states=1048583
```

最小的一段 1 048 583 个状态，过约 10⁶，照派发规则一段都不跑（全量 0 个状态）。读码推的：合法发布先写齐单元、过屏障、再写根，落了盘的最新根的孩子都在；候选集里更早的根引用的单元在离开根环之前不复用（I-7.4），所以合法流的崩溃状态里「孩子两份都读不出」碰不到，读不出孩子那一支与 H1 两份副本那一支在层 0 上都走不到——推的，没跑。

## 五、上一轮「没打中（一次抽样）」的第二次抽样（m3-batch25-code-r1 K2）

### 5.1 「走到末片而没滤全」：第二次也没打中

起点同上一轮的 X（`pool_x`：被崩溃恢复抛弃的 C 是实例 2、txg 6，号 2 在 C 上出生代 6、现行线上 9；最新根的实例表行 (1, 3)、(2, 5)）。上一轮的 41 形一形不重，换成下面 13 形（`a2_second_sample_of_the_gate_shapes`），原样：

```
[a2] RESULT A A2 S1 三片、被抛弃实例那一行在第2片 完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a2] RESULT A A2 S9 三片、第1片无行、其余行在第2片 完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.11"]
[a2] RESULT A A2 S5 被抛弃实例那一行在第0片txg低一份、第2片txg抬到被抛弃那一版一份 完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.8", "I-3.11"]
[a2] RESULT A A2 S5b 被抛弃实例那一行在第0片txg抬到被抛弃那一版、第2片原样一份 完好 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.8", "I-3.11"]
[a2] RESULT A A2 S1 三片、被抛弃实例那一行在第2片 第1片盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a2] RESULT A A2 S1 三片、被抛弃实例那一行在第2片 第2片盘1那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a2] RESULT A A2 S1 三片、被抛弃实例那一行在第2片 第2片盘0那份坏 I-9.29=成立 core读得出 violated=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-7.4"]
[a2] RESULT A A2 S4 同一片两行txg低的在前 I-9.29=成立 core读得出 violated=["I-3.8"]
[a2] RESULT A A2 S4 同一片两行txg低的在后 I-9.29=成立 core读得出 violated=["I-3.8"]
[a2] RESULT A A2 S6 链指针记录在片中间、被抛弃实例那一行夹在两条链指针记录之间 I-9.29=成立 core读不出 violated=[]
[a2] RESULT A A2 S8 被抛弃实例那一行在链指针记录之后（片末） I-9.29=部分没判[门] core读不出 violated=["I-3.8"]
[a2] RESULT A A2 S7 两片、第1片是第0片行的整份拷贝 I-9.29=成立 core读得出 violated=["I-3.1", "I-3.8", "I-3.11"]
[a2] RESULT A A2 S10 两片、第1片指针的两条位置条目盘序倒过来 I-9.29=成立 core读得出 violated=["I-2.5", "I-3.1", "I-3.11"]
```

门开的 12 形里 I-9.29 都成立：被抛弃实例那一行不论在第 2 片、跨片两份、同片两份，候选集都按整张表滤掉了 C（CPU 的滤法是「有一行使这条根被抛弃」就滤，`any`，两行 txg 不同也滤）；门关的 1 形（S8）是末一条不是链指针记录、该关。S6 是 core 读不出整张表而 checker 一条都不红，与上一轮 A4「插 kind 2」那几形同族（I-3.8 字面只管行，不管链指针记录在片中间），不是「走到末片而没滤全」，这一格不归它。

### 5.2 N2 GPU 其余各支：第二次也没打中（读码）

换一个切入点重读走读内核的实例表链（`w_walk_instance_table_chain`）与 CPU 的 `walk_instance_table_chain`：上一轮比的是置位时机、合法性判法、提前出口、清零与交接；这一轮逐步对次序与每一步的副作用——登记引用在读之前、读两份都判 I-2.1、「走过」查在读之后、容器不合法提前出口、I-1.12 出生树、身份四元组不对记走读失败、末一条不是合法链指针记录（GPU `chain = 2`，CPU `Malformed`，记录数 0 时两边都当不合法）、`judge_rows` 只在挂载根那一遍为真时才收行与置位、收行时片中间的 kind 1 记录不当行。逐步相同；不同的只有上一轮已坐实的 256 片上限（第 27 批）与 GPU 行表满时记判不了（回落 CPU，不是分叉）。上表 5.1 那 13 形按读码推 GPU 同判（推的，GPU 不跑）。

### 5.3 合法历史：第二次也没打中

即 4.1：换种子、加 SI 起点、旗加五条，148 段 1 883 次判定，I-9.29 全部成立、门一次都没关、旗 0。

## 六、几个改法各修哪一格（只在我的模型上量过、被攻过零轮）

`fix-FB-FV.diff` 只改 `crates/singlefs-checker/src/walk.rs` 三处（GPU 那一半没写）：
- F-B：inode 树读不出孩子、类型段 2 那一支判「子指针头部的出生树 == 身份引用的出生树」；墓碑、谱系容器读不出那一支同判。
- F-V：`child_unit_class_other_than_the_entry_type_names` 两份都看，按各自位置条目自证得过的类里有一类不是类型段那一类就交回它（不在第一份自证过的那份处停）。另一种同样不翻面的写法是「有一份自证得过的是类型段那一类就交回 None」，在 H-V 的条目改坏那几形上照样漏判，我没取它。

| 格 | 本批实现（new） | F-B | F-V | 标 |
|---|---|---|---|---|
| H-B：U 段 32 形（BT / IDB、容器类型段、两份都读不出） | 中 | 修（`u-fix` 142 形全对得上） | 碰不到 | 量过 |
| H-B：H 段 90 形（层级 ≥ 2 根里类型段 0 → 容器类型段、两份都不是 R） | 中 | 修（`h-fix` 目标不照字面 0） | 碰不到 | 量过 |
| H-V：36 形随盘序翻面 | 中 | 碰不到 | 修（`v-fix` 36 形全红、两种盘序相同） | 量过 |
| 1.4 手造单元上缺 I-2.1 | 中（只在映射树外的单元上） | 碰不到 | 碰不到 | 量过 |
| 两份都读不出时多红 I-3.1 / I-3.11 / I-5.1（类型段改坏，按类型段登记槽数） | 中（old 同） | 碰不到 | 碰不到 | 量过（`h-fix` 违例集合不同 201 格就是这一族加 1.4 那 12 格） |
| GPU 上 H-B、H-V | 中（推的） | GPU 没改 | GPU 没改 | 推的 |

`fix` 上的 `checker_known_bad_images` 原样：`[kbi-fix] test result: ok. 130 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`（new 上同样 130 条全过）。F-V 每条条目都多读、多验一份整单元校验和，层 0 每个状态都要付（推的，没量时间）。

## 各格判定一览之外：没打中的形状

| 形状 | 取样范围 |
|---|---|
| H1 新写法随陈旧副本所在的盘翻面而原写法不翻 | 8 张起点（墓碑 1–3 层、谱系 1–2 层、inode 根层级 1、回退之后、手造层级 2）31 条条目 × 3 种条目改法 × 4 × 4 种副本状态，1 488 形；两份同类一份自证不过、两份不同类都自证不过、陈旧在盘 0 / 盘 1、两份都坏，新写法 0 翻面 |
| H3 读不出孩子时误红 | U 段 142 形（七张起点、七种改法、四种读不出组合）；合法历史 1 883 次判定 |
| H3 回退之后、根长高之后 | I0r（回退那一版与之前几版共享叶）、I2（根层级 2，类型段 0 条目与其下层级 1 节点）上 H、U 两段都与 I0 同判 |
| CPU 与 GPU 判得不一样 | 第三节 7 支、5.2 链走读逐步 |
| m3-batch25 K2 三格的第二次抽样 | 第五节 |

## 这条腿自己的限度

- 一次抽样。H-B、H-V 是线索，要主 agent 现查；「没打中」的几格照规则还要再抽一次才记（第五节那三格是第二次）。
- 崩溃状态一个都没跑（4.2）；「合法流碰不到读不出孩子」是推的。
- GPU 不跑，第三节与 5.2 都是读码；F-B、F-V 的 GPU 那一半没写。
- I2 是手造的（合法流里 inode 树不长高）：槽 180 000 不在记账、不在中央映射树里，干净那张就红 I-3.1、I-3.11，H 段 I2 那几格的违例集合比对以它为准。
- 副本状态只取了四种（原样、码 2 陈旧、码 3 陈旧、全零）；没造「陈旧那份是同一个单元的旧版本」（同类、同身份、只是内容旧）——它按位置条目自证不过，归到 S2 / S3 同一支（推的）。
- 只取了每个节点的第 0 条与最后一条条目；节点宽到 135 条的情形没造。
- 合法历史只判每一步做完之后的镜像；SR 起点（实例表两片）这一轮没跑。

## 没做什么

- 不判正推那几格（K1、K3）；不替主 agent 采纳；F-B、F-V 只在我的副本上量过、被攻过零轮，没写进 `crates/`；副本上的数不算入库装置上的数。
- 没跑 checker 档、GPU、变异表、门禁；`checker_known_bad_images` 只在 new 与 fix 两份副本上各跑一次（harness 档）。
- 中途 `u-fix` 六张起点与 `v-fix` 有两次经内存包装退 252（同会话另有一条在跑、我漏设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`），那两次输出不算，补设之后原样参数重跑；正式结果全部出自 `rerun.sh` 在空目录里的那一遍（`rerun.sh` 默认设了这个变量与 150 秒等待）。
- 收到主 agent 两条消息：读代码阶段线程上限改 10、正式复跑 `h-new` 之后改 31；之后的段按 `OPUS_THREADS=31` 跑，各测试本身单线程，不影响输出。
