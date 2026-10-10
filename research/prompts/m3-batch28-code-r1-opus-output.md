# m3-batch28-code-r1 攻方腿（Opus）报告（2026-10-06）

<!-- doc-lint:not-numbers K1 K2 K3 H-B H-V J1 T1 T2 T3 -->

攻击面：正文第三节 K2（重点 H-B、H-V），另做 m3-batch27-code-r1 的 J1、CPU 与 GPU 第二次抽样。冻结副本 `/tmp/claude-1000/m3-batch28-code-r1/tree/`（5 份被判文件 sha256 与快照逐个相同），一切实跑都在拷贝上（`/tmp/claude-1000/m3-batch28-code-r1-opus/work/`），只跑 harness 档与自己的原型 crate，GPU 不跑。

## 复跑与模型文件

模型目录 `research/prompts/m3-batch28-code-r1-opus-model/`（`rerun.sh` 文件头写了各段、变量与内存上限）。复跑（串行，同一时刻一条 cargo；仓副本最多两份：`tree` 轮流换成 new / old / fix，另一份 `tree-probe`；每段前台四分钟以内）：

```
M=research/prompts/m3-batch28-code-r1-opus-model; W=<空目录>
export SINGLEFS_REPO=/tmp/claude-1000/m3-batch28-code-r1/tree SINGLEFS_TOOLS=<仓根>
for s in setup kbi-new kbi-old kbi-fix hvsum crash-build crash-count-it crash-count-legal crash-final-s0 crash-final-sd1 crash-final-sd2 crash-final-sl; do bash $M/rerun.sh $W $s; done
bash $M/rerun.sh compare $W        # 本腿在空目录 rerun1 上从头跑完：SUMMARY_MATCHES 346 行
bash $M/count_hv.sh $W             # 第一节那两组数
```

`all` 依次跑同样这几段。层 0 全量枚举那两段（`crash-s0m`、`crash-sdm`）超过前台四分钟，不在 `all` 里，见第七节。包装另读的变量 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`、`RUN_WITH_MEMORY_CAP_PARALLEL_REASON`、`RUN_WITH_MEMORY_CAP_WAIT_SECONDS` 没设时 `rerun.sh` 给默认值。

```
81cf0c682690bfae544f9f32938baa32db217355f5398eff25179d2bf41ba5db  analyze.py
94df62c551184de886cb2a71d8603ac1a21e43fe40798cf417acf25b86b866ff  batch28-walk.patch
f0b49e8d2f0285b20b6eb62a05186cb038b28d0f9203e8a83336537f1833f710  count_hv.sh
4b6f393c3f251f5c78a9746330eb75164e1b272aad8fcc312ae763748f74cb1f  crash/Cargo.toml.in
61dd8f9d1f7141509a983646a20c72081f3112d962a9b928bd7c22603734b5c9  crash/src/main.rs
ed5c45ae04f535f4d72c715de423680de8e6358afaf3e0c369f26ce73cf82417  expected-summary.txt
26dd17eacd80ad24e0957250b3a104f68445c5c78b61fa562e5073d434966f36  fix-FW.diff
02519bdff1df2dd37afdba1253b84315cde54373b7eb3857e67fcd0dba9fbfa9  opus_b28_append.rs
67ee64a506c89ecaf2eeccd5372f6433806b9dea936b2445830b24bff7e310b9  probe.diff
a1fade72bf7049af4cf5c0d91b4060487e7763e049486ad0a655523dffb036c9  rerun.sh
```

接手说明：前一条腿的草稿与模型目录随重启清空，这一份模型由它会话记录里的命令原样重建（原型 `opus_b28_append.rs`、`make_fix.py` / `make_probe.py` 生成的两份 diff、原型 crate），改了三处：`rerun.sh` 改成单份仓副本轮换、串行；原型 crate 加「终态」一档（第二节）、崩溃恢复之后再挂再建那一步改成要设 `OPUS_AFTER_RECOVERY` 才跑；实例表那一例加 core 读者那一列。

## 各格判定一览

| 格 | 判定 | 一句话 | 节 |
|---|---|---|---|
| K2 H-V：两份都看之后，本来判对的现在判错 | **打中**（漏判，坏镜像） | 另一类那一份自证得过时，本类那一份（读者真用的那一份）的子树整片不走：16 格目标漏判，第 28 批之前 8 格，F-W 0 格 | 一 |
| K2 H-V：合法崩溃状态里一份没落盘、另一份旧单元时误红 | 没打中（一次抽样） | 合法终态 2800 张镜像 0 红、探针「另一类」0 次；层 0 全量枚举交主 agent（第七节） | 二 |
| K2 H-B：读不出孩子时照判那一小句在合法状态上误红 | 没打中（一次抽样） | 写者两段出生树取同一个树号；合法终态 2800 张里 H-B 那一支一次没走到 | 三 |
| K2 CPU 与 GPU：第 28 批改的那几支 | 没打中（读码） | 两支 H-B、认类函数逐句同；第一节那一形 GPU 同样漏（同一处 `continue`） | 四 |
| K2 CPU 与 GPU：实例表链上的 I-1.2 | **打中**（读码 + CPU 期望；先于第 28 批） | CPU 每片判指针与头的出生身份，GPU 链循环不判；改指针三段各一形，CPU 只红 I-1.2，GPU 给全成立 | 四 |
| 第二次抽样：m3-batch27-code-r1 J1 | 没打中（读码） | 成环、恰在 256 片处成环、跨版本共享片：两边都停、GPU 走满记判不了 | 五 |
| 第二次抽样：m3-batch27-code-r1 CPU 与 GPU | **打中** | 同第四节实例表链 I-1.2，落在 J1 改的那个函数里 | 五 |

## 一、K2 H-V：两份都看之后，本类那一份的子树整片不判（打中）

### 1.1 造法

冻结副本拷三份：new（原样）、old（`walk.rs` 反打第 28 批补丁）、fix（new 打攻方 F-W，见 1.5）。三份的 `checker_known_bad_images.rs` 末尾追加同一份原型 `opus_b28_append.rs`，用例 `opus_b28_hv_subtree_of_the_verifying_copy`：

- 三棵树的起点镜像同第 28 批坏镜像（inode：`build_pool`；墓碑：`image_after_the_intent_of_a_three_unit_file`；谱系：`image_after_rolling_back_to_the_first_root`），根层级 1、第一条条目指一片码 3 容器。
- 先在那片容器里放一处违例（两份一起改、重封、校验和沿引用链补到根槽）：inode 叶第 0 条记录偏移 112 写 1（I-9.7）、blocks 加 1（I-9.15）、改动计数写成容器诞生代号 + 1（I-9.26）、inode 号写成容器号 − 1（I-9.4）、记录宽减 1（I-1.7）、记录数写 0（I-9.13）；墓碑容器区间止写 0、死亡 txg 写 10⁶（I-9.20）、记录宽减 1（I-1.7）；谱系容器记录宽减 1（I-1.7）。
- 再做第 28 批 H-V 那一步：父条目第 k 条位置条目的整单元校验和换成这棵树根（码 2）16 KiB 的 CRC、重封到根，盘 k 上孩子的槽写进那 16 KiB；另一块盘那一份就是上一步那片带违例、按自己位置条目自证得过的码 3 容器（本类）。k = 0、1 各一张，另加不做 H-V 的对照；条目不改（keep）与改成类型段 0、身份清零（type0）两种。
- 每张镜像跑池级 checker，另跑 core 的恢复（`recover(…, JournalPolicy::Consult)`），看读者用不用本类那一份。

取样：3 棵树、违例 12 种（inode 6、墓碑 4、谱系 2；墓碑、谱系的「记录数写 0」两种在三份代码上不做 H-V 也报成立，下面不计，计的是 10 种），加不放违例的 3 种、条目 2 种、另一类那一份的位置 3 种（无 / 0 / 1），共 90 张镜像 × 3 份代码（`grep -c 'RESULT HV ' logs/hv-new.log` 得 90）；`hvsum.log` 30 行是按形对齐之后的汇总。

> 接手更正（2026-10-06）：前一条腿这里写的是「违例 10 种（加不放违例 3 种）……共 78 张镜像」，在重建的模型上数出来是 90 张，上面已改；下面 1.2 的五行与 16 / 8 / 0、4 / 12 / 20 两组数在重建的模型上逐字复现（五行只是次序与原报告不同）。

### 1.2 结果（原样摘自 `logs/hvsum.log`，条目不改那几形）

```
tree=inode violation=reserved entry=keep target=I-9.7 new[target none/loc0/loc1=V/NA/NA entry=H/V/V loc0-vs-loc1=same] old[target none/loc0/loc1=V/NA/V entry=H/V/H loc0-vs-loc1=flip] fix[target none/loc0/loc1=V/V/V entry=H/V/V loc0-vs-loc1=same] core_loc0=Failed(UnitMalformed { what: "inode 记录" }) core_loc1=Failed(UnitMalformed { what: "inode 记录" })
tree=inode violation=blocks entry=keep target=I-9.15 new[target none/loc0/loc1=V/NA/NA entry=H/V/V loc0-vs-loc1=same] old[target none/loc0/loc1=V/NA/V entry=H/V/H loc0-vs-loc1=flip] fix[target none/loc0/loc1=V/V/V entry=H/V/V loc0-vs-loc1=same] core_loc0=FileRead(3000) core_loc1=FileRead(3000)
tree=inode violation=width entry=keep target=I-1.7 new[target none/loc0/loc1=V/H/H entry=H/V/V loc0-vs-loc1=same] old[target none/loc0/loc1=V/H/V entry=H/V/H loc0-vs-loc1=flip] fix[target none/loc0/loc1=V/V/V entry=H/V/V loc0-vs-loc1=same] core_loc0=Failed(UnitMalformed { what: "inode 叶容器" }) core_loc1=Failed(UnitMalformed { what: "inode 叶容器" })
tree=tomb violation=width entry=keep target=I-1.7 new[target none/loc0/loc1=V/H/H entry=H/V/V loc0-vs-loc1=same] old[target none/loc0/loc1=V/H/V entry=H/V/H loc0-vs-loc1=flip] fix[target none/loc0/loc1=V/V/V entry=H/V/V loc0-vs-loc1=same] core_loc0=NoFile core_loc1=NoFile
tree=lineage violation=width entry=keep target=I-1.7 new[target none/loc0/loc1=V/H/H entry=H/V/V loc0-vs-loc1=same] old[target none/loc0/loc1=V/H/V entry=H/V/H loc0-vs-loc1=flip] fix[target none/loc0/loc1=V/V/V entry=H/V/V loc0-vs-loc1=same] core_loc0=FileRead(3000) core_loc1=FileRead(3000)
```

（`target` 三格依次是：不做 H-V / 另一类那一份在位置条目 0 / 在位置条目 1；V 红、H 成立、NA 不适用。I-9.26、I-9.4、I-9.13 三形与 I-9.15 那一行同样：new 两种盘序都是 NA，old 只在位置条目 1 那一种红。）

数出来的（命令与原样输出）：

```
$ bash research/prompts/m3-batch28-code-r1-opus-model/count_hv.sh <工作目录>
new: 违例在本类那一份、目标漏判（NA 或 H）的格数 = 16；目标红的格数 = 4
old: 违例在本类那一份、目标漏判（NA 或 H）的格数 = 8；目标红的格数 = 12
fix: 违例在本类那一份、目标漏判（NA 或 H）的格数 = 0；目标红的格数 = 20
```

20 格 = 10 种违例 × 2 种盘序。new 还红着的 4 格是墓碑 I-9.20 两形：删除态扫描另读墓碑容器，不靠走读往下走。

### 1.3 机理

- new 在 `child_unit_class_other_than_the_entry_type_names` 里只要有一份自证成别类就交回它：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:4186`：「class_other_than_the_entry_type_names = Some(class_in_the_header);」；调用方判红之后 `continue`，本类那一份不再读（inode 那一支 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:2534`：「"child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码",」所在那一段，第 2546 行 `continue`；墓碑 / 谱系那一支第 2222 行 `continue`）。
- 本类那一份是读者真会用的那一份：core 按条目那一类的长度逐份读、第一份对得上就用（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/recovery.rs:384`：「if crc32_castagnoli(&bytes) == location.unit_checksum {」）。上表 core 那一列：blocks 那一形照样 `FileRead(3000)`，I-9.7 那一形 core 自己读到那条记录、报 `UnitMalformed { what: "inode 记录" }`。读者读到、交给用户的记录，checker 报不适用或成立。
- old 在第一份自证过的副本处停：另一类那一份在位置条目 1 时，本类那一份先被认、照常往下走，违例判得到（代价是 I-9.2 漏判、随盘序翻面，即第 26 批 H-V）。第 28 批把「随盘序翻面」修成「两种盘序都不往下走」，把 I-9.2 那一格的漏判换成了子树里那几条的漏判。

### 1.4 判据四问

1. 分不分辨臂：分辨。new 两种盘序都漏，old 一种盘序漏，F-W 都不漏（上面数出来的 16 / 8 / 0）。
2. 系统当时看不看得到：看得到。`child_unit_class_other_than_the_entry_type_names` 逐份读的时候，本类那一份按本类长度自证得过这件事它已经算过一遍（同一个循环里 `verifies_at_the_length_of_its_class` 为真、类等于类型段那一类的那一份），只是丢掉了。
3. 满足字面的哪一分句：m3-batch22-code-r1 正文第五节「漏判：镜像违反 `invariants.md` 那一行的字面而 checker 报成立，或报不适用而镜像上有它的对象」——I-9.7 / I-9.15 / I-9.26 / I-9.4 / I-9.13 报不适用（对象是被引用、自证得过、core 读者真用的那片 inode 叶里的记录），I-1.7 报成立（同一份镜像里别的码 3 容器判成立，这一片的记录宽违例没判）。不是误红，也不是 C739 那两处（C739 管取哪一类、槽数按哪一类，不管本类那一份走不走）。
4. 跑前条款给的改法：正文第五节只写「打中：主 agent 现查坐实，改法进下一批」，没有预先给改法；第 26 批判决给的 F-V（就是 new）在这几格上照样中（16 格漏判）。攻方自己的改法 F-W 见 1.5。

### 1.5 攻方改法 F-W（只在我的模型上量过、被攻过零轮）

做法（模型目录 `fix-FW.diff`，只改 `walk.rs`）：两棵树「孩子是另一类」那一支先看有没有一份副本头里就是类型段那一类、按那一类的长度自证得过。有，照判条目那一条红，然后不 `continue`，照类型段那一支往下走（槽数按类型段那一类登记、读孩子、判身份、走子树）；没有，照 new。GPU 那两处（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2640`：「w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);」之后的 `continue`，第 3106 行墓碑 / 谱系那一处同形）要同改，没实现。

| 改法 | 子树里的违例（20 格） | 条目那一条两种盘序同判 | 槽数按哪一类登记 |
|---|---|---|---|
| old（第 28 批之前） | 8 格漏（量过） | 不同判，随盘序翻面（量过，`loc0-vs-loc1=flip`） | 位置条目 0 那一形按别类登记、连带红 I-3.1、I-3.11、I-5.1；位置条目 1 那一形按本类（量过，下面原样） |
| new（第 28 批，F-V） | 16 格漏（量过） | 同判（量过，`same`） | 两种盘序都按别类登记，连带红 I-3.1、I-3.11、I-5.1（量过） |
| F-W | 0 格漏（量过） | 同判（量过） | 按类型段那一类登记，这三条不连带（量过） |
| F-W 的 GPU 同改 | 推的 | 推的 | 推的 |

不放违例那三形的红集合（`logs/hv-{old,new,fix}.log` 里 inode 树那几行原样，去掉 core 列）：

```
code=old tree=inode violation=none entry=keep other_class_copy=loc0 target=-:- entry_invariant=I-9.2:V I-2.1:V reds=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4", "I-9.2"]
code=old tree=inode violation=none entry=keep other_class_copy=loc1 target=-:- entry_invariant=I-9.2:H I-2.1:V reds=["I-2.1", "I-4.8", "I-7.4"]
code=new tree=inode violation=none entry=keep other_class_copy=loc0 target=-:- entry_invariant=I-9.2:V I-2.1:V reds=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4", "I-9.2"]
code=new tree=inode violation=none entry=keep other_class_copy=loc1 target=-:- entry_invariant=I-9.2:V I-2.1:V reds=["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4", "I-9.2"]
code=fix tree=inode violation=none entry=keep other_class_copy=loc0 target=-:- entry_invariant=I-9.2:V I-2.1:V reds=["I-2.1", "I-4.8", "I-7.4", "I-9.2"]
code=fix tree=inode violation=none entry=keep other_class_copy=loc1 target=-:- entry_invariant=I-9.2:V I-2.1:V reds=["I-2.1", "I-4.8", "I-7.4", "I-9.2"]
```

F-W 副本上整份 `checker_known_bad_images`（151 个用例，含第 28 批新加的坏镜像）跑一遍（量过，副本上的数）：`test result: ok. 151 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 614.30s`（命令见第七节第 3 条；超过前台四分钟，没进 `rerun.sh`）。

F-W 碰到了 C739 的后一半（「一份本类一份别类时按哪一类登记槽数」）：它取类型段那一类。C739 这一轮不判，这一点交主 agent 定要不要随改法一起走。

## 二、K2 H-V：合法状态里一份没落盘、另一份旧单元时会不会误红（没打中，一次抽样）

### 2.1 读码（推的）

一份副本被认成「另一类」，要它按自己那一类的长度读回的整单元校验和等于父指针里那条位置条目的校验和。写者两条位置条目的校验和都按同一份字节算（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/transaction.rs:7231`：「locations: location_entries(context.device_identities, slot, &bytes),」）。没落盘那一份的槽里是旧单元，旧字节的 CRC32C 等于新字节的才会被认，这要撞校验和。层 0 的崩溃状态里，根槽写在前面几段之后（屏障切段），根落了盘，它下面那几段整段在盘上；没落盘的副本只会出现在还没发布的那一版里，走读不走它。

### 2.2 合法终态（量过）

原型 crate 的 `final` 档：三个起点（S0 = 新池新建文件后再建 2、3 号；SD = S0 之后删 2 号，墓碑树有容器；SL = SD 之后回退到 S0 那一版，谱系树有容器），每段历史 8 步，用户动作按种子随机抽（建 1 / 2 个号、删第 k 个活号、回退到这段历史里记下的任一版、再挂；只固定起点），每做完一步对那一刻的镜像跑一次池级 checker。checker 用 `tree-probe`（`walk.rs` 只加计数器，`probe.diff`）：`calls` 认类函数调用次数，`copy_same_class` / `copy_other_class` 一份自证成类型段那一类 / 别类，`other_class_returned` 交回别类，`location_checksums_differ` 两条位置条目校验和不同，`copy_unverified` 头的类登记过、按那一类长度自证不过的副本，`hb_inode` / `hb_container` 走到 H-B 那两支，`hb_red` H-B 判红。

```
RESULT final start=S0 seeds=1..151 steps=8 checked_images=1200 acts={'C': 356, 'D': 359, 'M': 239, 'R': 237, 'x': 9} reds={} probe[calls=9927,other_class_returned=0,copy_same_class=19854,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]
RESULT final start=SD seeds=1..61 steps=8 checked_images=480 acts={'C': 148, 'D': 137, 'M': 99, 'R': 86, 'x': 10} reds={} probe[calls=5276,other_class_returned=0,copy_same_class=10552,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]
RESULT final start=SD seeds=61..121 steps=8 checked_images=480 acts={'C': 136, 'D': 130, 'M': 95, 'R': 102, 'x': 17} reds={} probe[calls=5179,other_class_returned=0,copy_same_class=10358,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]
RESULT final start=SL seeds=1..81 steps=8 checked_images=640 acts={'C': 193, 'D': 191, 'M': 131, 'R': 120, 'x': 5} reds={} probe[calls=7828,other_class_returned=0,copy_same_class=15656,copy_other_class=0,hb_inode=0,hb_container=0,hb_red=0,location_checksums_differ=0,copy_unverified=0]
```

2800 张合法终态镜像 0 红；每次调用两份都自证成本类（`copy_same_class` 恰是 `calls` 的两倍），一次「另一类」、一次自证不过都没有。`x` 是做不成的动作（没有活号可删、回退被拒）。批大小：先跑 S0 10 段 × 6 步（60 张，7.9 秒）估每张约 0.13 秒，S0 批 1200 张实跑 177 秒；SD 第一次 120 段一批跑了 242 秒，复跑拆成两批各 60 段。

### 2.3 崩溃状态（没跑，交主 agent）

层 0 每一段都展开，单步历史的状态数（`crash-count-legal`，`closed_form_state_count` 算的）：建号、回退各 1048583，删号 68157453–71303181，再挂 S0 393238、SD 327696、SL 4325398；新池新建文件那条流 16777240（`crash-count-it`）。超过约 10⁶ 的不跑。速度：前一次试跑 SD 再挂那一段，原样一行 `LAYER0_PROGRESS slice=37/160 states=[73764,75813) segments=1..=1 finished_slices=40/160 finished_states=81960/327711 resumed_slices=0 elapsed_seconds=194.8`，约 420 状态 / 秒（10 线程）。那一次外面套了 `timeout 230`（违反「随机跑批不设限时」的写法），退 124，输出只拿来估速度，不算结果。S0、SD 两段再挂合计 720934 个状态，按这个速度约 29 分钟，超过前台四分钟，命令写在第七节。这一条腿实际枚举过的状态只有那 81960 个（前一条腿的原型没编完，0 个）。

## 三、K2 H-B：读不出孩子时照判的那一小句，在合法状态上会不会误红（没打中，一次抽样）

### 3.1 被判的那一句与读得出时的同一句

读不出时照判的：inode 树 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:4211`：「birth_tree_in_the_child_pointer_head == identity_birth_tree,」；墓碑 / 谱系树 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:2385`：「child.birth_tree == identity_birth_tree,」。读得出时这一句一直在合取里：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:2611`：「&& child.birth_tree == identity.0;」（墓碑 / 谱系第 2406 行同一句）。所以合法状态上要误红，得有一条合法写出来的条目两段出生树不等、而它的孩子恰好读不出；两段不等而孩子读得出的话，第 28 批之前读得出那一支就该红过。

### 3.2 写者（读码，推的）

条目由同一个函数写：身份引用出生树 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/records.rs:222`：「writer.put_u64(child_identity.birth_tree.0);」，子指针随后整条写进去；调用处把容器的身份与指针分开传（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/transaction.rs:7387`：「container.contents.identity,」、第 7388 行 `container.pointer,`）。这一版重写的单元：指针头出生树取树号（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/transaction.rs:7320`：「birth_tree: inode_tree_identifier,」，墓碑 / 谱系 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/transaction.rs:7477`：「birth_tree: to_build.tree,」），身份出生树也取树号（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/inode_tree.rs:242`：「birth_tree: inode_tree,」，`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-core/src/container_tree.rs:456`：「birth_tree: tree,」）。照搬上一版的容器，身份与指针一起照搬。这几处是不是同一个变量，我没有逐个调用点核过，这一段算推的。

### 3.3 量过的

第二节 2.2 那 2800 张合法终态：`hb_inode=0,hb_container=0`，H-B 那两支一次都没走到，所以这一批只能说明终态上它没机会误红，不能说明它在走到时判得对。崩溃状态里孩子读不出，要旧根的单元被释放、槽被复用，那要层 0 的枚举才看得到（第七节交主 agent，探针的 `hb_*` 三个数就是看这个）。

### 3.4 坏镜像上的一形（读码，推的，没跑）

第 28 批读不出孩子那几条坏镜像都只改父条目的一段出生树。两段一起改成同一个别的值时，按第 4211 行、第 2385 行那一句只比这两段，H-B 照判成立；比孩子头的那几句孩子读不出、不判。这是第 28 批写明不判的部分（那几小句要看孩子头），不算漏判。这一形我没有造镜像跑，只按代码推。

## 四、K2 CPU 与 GPU（GPU 不跑；读 wgsl，用 CPU 算期望）

### 4.1 第 28 批改的那几支：没打中（读码）

- H-B inode：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2665`：「w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));」，只在类型段 2 读不出时，与 CPU 同一句、同一个位置（认类之后、读不出那一支里）。
- H-B 墓碑 / 谱系：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3114`：「w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));」，条目偏移 8 就是 CPU 的 `identity.0`（`read_u64(entry, 8)`）。
- 认类函数：两份都看、只记第一个别类、类长度只认码 1 / 2 / 3，与 CPU 逐句同（CPU 的 `unit_bytes_of_the_class` 也只给这三类长度）。
- 第一节那一形 GPU 同样漏：认出别类之后同样判红就 `continue`（第 2640 行之后、第 3106 行之后），本类那一份不走。F-W 要两边一起改。

### 4.2 实例表链上的 I-1.2：打中（读码 + CPU 期望；先于第 28 批）

- CPU 链上每一片都判指针与头的出生身份：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:2953`：「self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);」。
- GPU 里 I-1.2 那一位只在一处置：`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1410`：「w_judge(INVARIANT_I_1_2, eq64(birth_txg, p.birth_txg) && instance == p.instance && eq64(tail, p.tail));」（`grep -nw INVARIANT_I_1_2 *.wgsl` 只命中这一行与常量定义），它所在的 `w_judge_birth_identity` 只有四个调用点（第 1501、1809、2683、3129 行），链循环 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1589`：「fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {」里不调。链循环读了头里的诞生代号（第 1623 行 `appearance_note_the_birth_txg_of_an_instance_table_page(…)`）、判了出生树（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1627`：「w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));」），判别它要的两样它当时都有。
- CPU 期望（`opus_b28_instance_table_pointer_birth_identity`，新池新建文件那条根，txg 3，实例表指针头部一段加 1、重封根槽、记录新根段跟着改；new 与 old 逐形相同）：

```
RESULT IT code=new field=none core=FileRead(3000) reds=[] I-1.2=H
RESULT IT code=new field=birth_txg core=FileRead(3000) reds=["I-1.2"] I-1.2=实例表单元：头里的出生身份（诞生代号 0、写序实例 0、出生序号 0）与引用它的指针记的（诞生代号 1、实例 0、尾段 0）不符
RESULT IT code=new field=instance core=FileRead(3000) reds=["I-1.2"] I-1.2=实例表单元：头里的出生身份（诞生代号 0、写序实例 0、出生序号 0）与引用它的指针记的（诞生代号 0、实例 1、尾段 0）不符
RESULT IT code=new field=tail core=FileRead(3000) reds=["I-1.2"] I-1.2=实例表单元：头里的出生身份（诞生代号 0、写序实例 0、出生序号 0）与引用它的指针记的（诞生代号 0、实例 0、尾段 1）不符
```

- GPU 会给什么（推的，没跑）：三形都 `reds=[]`。I-1.2 那一位在别的单元上照判成立，GPU 报「成立」；链循环里另外判的几条（位置条目次序、指针头、I-1.12、I-1.1、I-3.8）这三段都不碰。core 读者三形都 `FileRead(3000)`，恢复那一半两边同。这一形在 GPU 判得了的状态里，CPU 红、GPU 不红。
- 四句：① 分辨的是 CPU 与 GPU 两份判器（第 28 批前后不分，new / old 逐形同）；② GPU 当时看得到，见上；③ 满足正文第五节「GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中」；④ 跑前条款没给改法。攻方的改法 F-IT（零轮，推的，没实现）：链循环里在 `packed.ok` 之后按码 3 的头（偏移 73 诞生代号、93 实例、103 出生序号，与第 1393–1396 行同一套偏移）只判 I-1.2，不判 I-4.2（CPU 对实例表单元也只判这一半，`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-harness/tests/checker_known_bad_images.rs:8603`：「/// I-1.2（块头写序已发布） 状态收窄（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行）：实例表单元按例外只免谓词那一半，」）；对拍补这三形。`.claude/kb/checks-owed.md` 里没有这一笔（`grep -n '实例表' .claude/kb/checks-owed.md` 里带 GPU 或 I-1.2 的那几行都不是它）。

## 五、第二次抽样：m3-batch27-code-r1 的 J1、CPU 与 GPU

### 5.1 J1：没打中（读码，与上一次取样不同的形）

上一次（m3-batch27-code-r1 攻方）跑了 62 形：链长 ≤ 256 两边同判、≥ 257 判不了回落 CPU、中途断开停在同一处。这一次只看上一次没取的「链里有重复片」：

| 形 | CPU | GPU | 判 |
|---|---|---|---|
| 第 0 片的链指针指回自己 | 第二次走到第 0 片时 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker/src/walk.rs:2939`：「if !self.visited_units.insert(page_placement) {」并进、停 | `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1611`：「if (!w_visit(pointer.device0, pointer.slot0)) {」并进、停，`page` = 1 | 同判 |
| 环长 255（第 254 片指回第 0 片） | 同上 | 第 255 轮在 `w_visit` 处停，`page` = 255，不记判不了 | 同判 |
| 环长 256 | 走 256 片后并进、停 | 走满（`/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1601`：「for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u) {」），记 `/tmp/claude-1000/m3-batch28-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1687`：「undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;」 | 回落 CPU，不错判 |
| 两版的链共享后面几片 | 后走的那一版在第一片共享处并进、停 | 同 | 同判 |

两边都先查「走过没有」、再判身份与链记录，所以重复片上两边都不判 I-1.1（容器号对不上第几片）。这是共用的写法，不分 CPU 与 GPU。

### 5.2 CPU 与 GPU：打中

就是第四节 4.2：J1 改的正是 `w_walk_instance_table_chain`，那个函数整个不判 I-1.2，上一次「读码一致」那一格漏了它。

## 六、没打中的形状

| 格 | 试过的形 | 取样范围 |
|---|---|---|
| H-V 合法终态误红 | 三个起点 × 随机用户动作（建 1 / 2、删第 k 个活号、回退到记下的任一版、再挂），每步之后判 | S0 150 段、SD 120 段、SL 80 段，各 8 步，合计 2800 张镜像；探针「另一类」0 次 |
| H-V 合法崩溃状态误红 | 读码（第二节 2.1）；层 0 全量枚举没跑 | 单步历史状态数见 2.3；交主 agent 的两段 720934 个状态 |
| H-B 合法状态误红 | 读写者代码（3.2）；合法终态同上 | 2800 张里 H-B 那两支走到 0 次，所以这一格在终态上是空的 |
| H-B 坏镜像 | 两段出生树一起改成同一个别的值 | 只读码，没造镜像 |
| CPU 与 GPU（第 28 批那几支） | 两支 H-B、认类函数、第一节那一形 | 读码 |
| J1 第二次 | 自环、环长 255 / 256、两版共享后面几片 | 读码 |

## 七、要主 agent 跑的命令

工作目录：仓根。内存上限写在 `rerun.sh` 里（编译 6G、跑 4G，经 `run-with-memory-cap.sh`），线程经 `capped.sh`，数取 `OPUS_THREADS`。同一时刻只起一条。

1. 层 0 全量枚举（第二节 2.3、第三节 3.3 那两格）：

```
M=research/prompts/m3-batch28-code-r1-opus-model; W=<空目录>
export SINGLEFS_REPO=/tmp/claude-1000/m3-batch28-code-r1/tree SINGLEFS_TOOLS=$PWD OPUS_THREADS=<线程上限>
for s in setup crash-build crash-s0m crash-sdm; do bash $M/rerun.sh $W $s; done
```

按 10 线程实测约 420 状态 / 秒估：`crash-s0m` 393238 个状态约 16 分钟，`crash-sdm` 327696 个约 13 分钟（线程多了会快一些，估时按 10 线程算）。看 `$W/logs/crash-s0m.log`、`crash-sdm.log` 里 `RESULT crash S0-s4[M] layer0 states=…` 与 `RESULT crash SD-s4[M] layer0 states=…` 两行：`checker_red_states=[]` 且 `probe[…]` 里 `other_class_returned=0`、`hb_red=0` 就是这两格在崩溃状态上也没打中；`hb_inode`、`hb_container` 不为 0 说明 H-B 那一支在崩溃状态上走到过，那时 `hb_red` 是要看的数。有 `DETAIL crash … checker` 行就是红了，原样交回。`states` 应等于 `LAYER0_PARALLEL_START` 那一行的 `states`。

2. 可选：每个崩溃状态恢复之后再可写挂载、判、建一个号、判，命令同上，前面加 `OPUS_AFTER_RECOVERY=1`。每个状态要多做一次挂载、两次 checker，这一档没估过时间。

3. 可选：F-W 副本上整份 `checker_known_bad_images`（1.5 那一行数）：先跑 `setup`、`kbi-fix` 两段，再跑 `OPUS_CODE=fix bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh <线程上限> $W/bin/kbi-fix`，看 `test result: ok. 151 passed`。本腿在副本上跑过一次，614 秒。

## 八、这条腿自己的限度

- 第一节的打中只在手造的坏镜像上：父指针一条位置条目的校验和被改成另一类单元的 CRC、重封到根。合法写者写不出这样的指针（第二节 2.1）。它是漏判（坏镜像上该红的没红），不是合法状态上的误红。
- 1.1–1.4 是前一条腿写的。我在重建的模型上复跑了两遍（`rerun1` 线程 10、`rerun2` 线程 15），数都相同，只改了 1.1 那一处镜像张数。
- 探针在 `walk.rs` 的副本上，只加计数器、不改判定（`probe.diff`）。这一点按 diff 推，没有拿带探针的副本跑一遍坏镜像对照。
- GPU 一律没跑：第四节的「GPU 会给什么」都是读 wgsl 推的。F-W 的 GPU 同改、F-IT 都没实现。F-W 只在 CPU 副本上量过，被攻过零轮。
- 三次前台命令超过四分钟、被挪到后台：SD 120 段那一批（242 秒）、F-W 副本整份坏镜像（614 秒）、`rerun2` 一次连跑五段。都按进程号等到退出才往下做，结果照收。量速度那一次外面套了 `timeout 230`（2.3）。
- 线程上限：主 agent 的消息是在写完第三到第五节、开始 `rerun2` 之前收到的（10 改成 15）。`rerun2` 起用 15，之前的都是 10；两遍的 `compare` 都是 `SUMMARY_MATCHES 346 行`。
- 「没打中」都只是一次抽样。

## 没做什么

- 层 0 崩溃状态没枚举（交主 agent，第七节）；新池新建文件那条流（16777240 个状态）与删号历史（6.8–7.1 × 10⁷）超过约 10⁶，不跑。
- 没跑 GPU、checker 档测试、门禁。K1、K3 不归这条腿。
- 没造「两段出生树一起改」的坏镜像（3.4 只读码）。
- 没核 C739 该怎么定：F-W 碰到它的后一半，交主 agent。
