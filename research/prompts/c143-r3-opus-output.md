# C143（inode 号水位在回退后会退回去重发） 第三轮：攻方腿（Opus）报告

立场：反推 / 攻。假设主 agent 的倾向（丙 + CJ2）是错的，也假设丙原样、丙 + CJ、G 各有死角，换前两轮没攻过的面造可达历史去打穿它们。

读过的材料：任务书 `research/prompts/c143-r3-opus.md`；背景材料 `research/prompts/_c143-r3-background.md`（正文、小节清单、附录全文）；前两轮判决 `research/prompts/c143-r1-main-verification.md`、`c143-r2-main-verification.md`；第二轮攻方腿报告 `research/prompts/c143-r2-opus-output.md` 与模型 `research/prompts/c143-r2-opus-model/c143r2_model.py`（只读，按字节拷了一份进这一轮的目录，见下）。附录不够处现查了 kb 原文；下文引 kb 一律写 kb 文件名加那份文件自己的行号（`grep -n` 现查），当论证支点的行在报告末尾附录里整行抄（`sed -n` 机械抽取）。没有读这一轮别的腿的产出，没有改仓里任何别的文件；模型只用 `nice -n 19` 跑，没有编译 Rust、没有跑门禁。

模型：`research/prompts/c143-r3-opus-model/c143r3_model.py`（Python 3.12 标准库，确定性），它 import 同目录的 `c143r2_base.py`——那是第二轮模型的逐字节拷贝（sha256 与原件相同，见第九节），这一轮一个字没改它。原始输出 `c143r3_model.out`。

## 一、模型：沿用第二轮的哪些、新加了什么、两种故障计数

沿用第二轮模型的全部机制（根环三区域、归属 0 / 1 / 0、S = 8，journal 两盘镜像与覆写，严格暖机，影子账只查可读根，孤儿单元当场复用，回退的前缀末两种读法 P1 / P2，切换按 keep 读法），照的 kb 行见第二轮报告第一节的表，这里不重抄。新加的：

| 新加什么 | 为什么 |
|---|---|
| 扫描时数出「根环里读不出可读根的槽数」k_bad（被故障藏的，加上从没写过的——实现分不出从没写过的槽与撕裂的槽） | 给我这一轮提的线索 CK / CK2 用（第 2.6 节） |
| 臂 CK（丙 + 回退时第一个新根 txg = 可读根最大 + 1 + k_bad）、CK2（CK 推广到每次普通挂载：新实例从 max(重放之后那一根, 可读根最大 + k_bad) 往上数）；看到 CK 的毛病之后再加 CR / CR2（跳号固定为 R·S = 24） | 我这一轮提的收严，只在我的模型上量过；这一轮就被我自己的模型打穿、撤回（第 2.6 节） |
| 成对计数加上「x 中而 y 不中的那些脚本里最少几个故障」，和按前缀（同一段历史、只换故障）逐一比 | 背景材料第五节反向接受条款第二条要的就是这个比较 |
| 每族同时出两套数：第二轮的口径，和「只按槽计」——扫描时不许整块盘读不出 | 见下一段 |
| 族 f1（第二轮 s2 的空间）、f5（第二轮 s5）、f11（克隆头、销毁头、树 ID 重发与回退交织）、f12（回退尝试崩了再重做 × 两种前缀末读法、连续三次回退、背靠背回退、回退—崩溃—挂载—回退）；f1x / f1y（f1 的空间按前缀逐一比，故障空间放宽到 10 个根槽、10 条记录） | 任务书要攻的面 |

**第二种计数为什么要有**：第二轮的最少故障数里，丙的 2、CJ 的 5 / 3、G 的 2 都用了「整块盘 d0 在回退那次读扫描里读不出，算 1 个故障」。可是回退是一次可写的恢复（它取号、写行、写根），而 D18（块里携带什么信息） 第 879 行规定可写挂载先要「可写设备数够 w 的下限 ∧ 独占打开池中过半的设备」，同一行写着「第一版 2 盘、掉一块只能只读挂载」；journal 两盘镜像、w ≥ 2 对 journal 记录写同样生效（D22（单元原子性怎么合成） 第 229 行）。⇒ 第一版里「一块盘整块读不出」和「在这块盘上做回退」不能同时成立，除非那块盘读全失败、写却照样成功（一种很特别的瞬时故障）。所以我把每一族再按「扫描时不许整块盘读不出、只能一个槽一个槽地坏」重数一遍，记作 `_slot_only`。两种数都照实摆，哪个是第一版的真口径由主 agent 判。

臂一览（判据：A / AJ / AJ2 / G 按 (树, 号)，C / CJ / CJ2 / CK / CK2 / CR / CR2 按 (树, 号, 出生代)；违例 = 一个身份先后被两个不同对象带进写成了的根，至少一个是在某次回退之后建的，与第二轮同）：

| 臂 | 回退 / 挂载时怎么取 |
|---|---|
| A（甲 / 乙） | 水位 = max(R_old 那一格, 每个可读根里该头那一格) |
| AJ / AJ2 | A 再取 journal 里读得出、单元还在的记录里该头的水位（回退时 / 每次挂载） |
| G（第二轮线索） | 每个头的水位抬到读得出的根与记录头里全池号水位的最大值 |
| C（丙） | 水位从 R_old 重载；第一个新根 txg = 可读根最大 + 1 |
| CJ / CJ2 | 第一个新根 txg = max(可读根, 读得出的记录的 checkpoint_txg) + 1（回退时 / 每个新实例） |
| CK / CK2、CR / CR2（我的线索，已撤回） | 第一个新根 txg = 可读根最大 + 1 + k_bad（CK）或 + 1 + 24（CR）；CK2 / CR2 每次挂载也这么起步 |

脚本记号与第二轮相同（`rb(k,u,sw)`、`rbm`、`mount(u)`、`p(n)`、`f(n)`、`u(n)`、`clone`、`ucx(n)`、`destroy`；`u` 里 `d0` 整块盘、`rN` 最新 N 个根槽、`jN` 最新 N 条记录的副本）；新加 `rbxk(k,u)` = 回退到第 k 新的候选、那次发布崩在记录之后。

## 二、U1：各形最短的「已发布身份被重发」历史

### 2.0 丙的余量从哪来（推理；模型逐条核过，数在 2.1）

1. 回退的第一个新根 T_first = 可读根里 txg 的最大值 + 1（`23-journal的角色与格式.md` 第 1206 行，实现只能对读得出的取）。R_old 是从读得出的候选里选的，所以 T_first ≥ T_old + 1。
2. 严格暖机（`16-发布语义.md` 第 207 行）在 0 / 1 / 0 归属（`22-单元原子性怎么合成.md` 第 1013 行）下：回退根落在 T_first，之后连推空发布到本实例的根覆盖两块盘，第一个用户对象的出生代 L(T_first) = T_first + 2（T_first mod 3 ∈ {0, 1}）或 T_first + 3（T_first mod 3 = 2）。顺带：L 永远 ≢ 1 (mod 3)，回退之后的第一个用户对象从不落在盘 1 的区域上。模型里逐条日志与这个式子相符（例：第二轮 C 那一段 T_first = 5 → 8；本轮 f1 的 CJ 那一段 T_first = 7 → 9）。
3. 丙中 ⟺ 被抛弃时间线里有一个已发布对象，(树, 号, 出生代) 与某个新对象相同。新对象的出生代 ≥ L(T_first) ≥ T_first + 2，所以那个被抛弃对象的出生代 b ≥ T_first + 2，而 txg ∈ [T_first, b] 的根必须全部读不出（否则 T_first 会更大）⇒ **至少连续 3 个 txg 的根读不出**。
4. 0 / 1 / 0 下任何连续 3 个 txg 都是「盘 0、盘 1、盘 0」的某个轮换 ⇒ 按第二轮口径最少 2 个故障（盘 0 整块 + 盘 1 那一个根槽），按 `_slot_only` 最少 3 个。第二轮 s8 的几何表是同一件事的解析版。
5. 丙中的每一段甲 / 乙也中：读得出的根 txg 都 < T_first ≤ b，它们带的水位都 ≤ 那个被抛弃对象的号 ⇒ 甲 / 乙发的号追得上它。f1 与第二轮各族 `hit=C not_hit=A` 都是 0。
6. CJ 中还要求 txg ∈ [T^CJ, 最大] 的记录两份都读不出；T^CJ ≥ T^C。所以「CJ 中而丙不中」的脚本要的故障 ≥ CJ 自己的最少故障 5 / 3（`_slot_only` 9 / 5），而丙在同一类里是 2（`_slot_only` 3）——按「同一类 = 同一种攻击形状」读，跑前第二条反向接受条款的第二个合取项在结构上不可能满足。按「同一段历史」读要另算，那是 2.2 的 f1x。

### 2.1 总表

格子里是这一族里最少的故障数：按槽计 `fs`，与按区间计 `fr` 不同时写成 `fs / fr`；「—」= 这一族里没有违例；「无效 n」= 有 n 条脚本对这条臂无效（挂载找不到可读根，或回退选中的候选里还没有头 12）而对丙有效。出处是 `c143r3_model.out` 里各族的 `arm=` 行与 `invalid_where_C_valid=` 行。

第二轮口径（扫描时整块盘读不出记 1）：

| 族 | A | AJ | G | C 丙 | CJ | CJ2 | CK | CK2 | CR | CR2 |
|---|---|---|---|---|---|---|---|---|---|---|
| f1 单次回退、藏最新的根 / 记录 / 一块盘（第 9–15 行） | 1 | 2 | 2 | 2 | 5 / 3 | 没跑 | — | 没跑 | — | 没跑 |
| f5 回退、崩溃、挂载时读故障（第 344–354 行） | 2 | 2 | 2 | 2 | 2 | 5 / 3 | —（无效 1219） | —（无效 1219） | 6 / 4 | 2 |
| f11 克隆头、销毁头与回退交织（第 870–879 行） | 1 | 1 | 2 | — | — | — | — | — | — | — |
| f12a 回退尝试崩了再重做，前缀末 P1（第 1114–1123 行） | 1 | 1 | 3 | — | — | —（无效 210） | — | —（无效 1575） | — | — |
| f12a 同上，前缀末 P2（第 1358–1367 行） | 1 | 1 | 3 | — | — | —（无效 210） | — | —（无效 1623） | — | — |
| f12b 连续三次回退（第 1602–1611 行） | 1 | 2 | 2 | 2 | 5 / 3 | 5 / 3 | 3（无效 210） | 3（无效 210） | 2 | 2 |
| f12c 背靠背两次回退（第 2188–2197 行） | 2 | 4 / 3 | 4 / 3 | — | — | — | —（无效 150） | —（无效 150） | — | — |
| f12d 回退—崩溃—挂载—回退（第 2426–2435 行） | 2 | 2 | 4 / 2 | 2 | 5 / 3 | 5 / 3 | —（无效 1192） | —（无效 1148） | 2 | 2 |

`_slot_only`（扫描时不许整块盘读不出，逐槽计）：

| 族 | A | AJ | G | C 丙 | CJ | CJ2 | CK | CK2 | CR | CR2 |
|---|---|---|---|---|---|---|---|---|---|---|
| f1（第 163–169 行） | 1 | 3 | 3 | 3 | 9 / 5 | 没跑 | — | 没跑 | — | 没跑 |
| f5（第 606–616 行） | 3 | 3 | 3 | 3 | 3 | 9 / 5 | —（无效 11） | —（无效 11） | 9 / 5 | 3 |
| f11（第 994–1003 行） | 2 | 2 | 4 | — | — | — | — | — | — | — |
| f12a P1 / P2（第 1239–1248、1483–1492 行） | 3 | 3 | 3 | — | — | — | — | — | — | — |
| f12b（第 1903–1912 行） | 1 | 3 | 3 | 3 | 9 / 5 | 9 / 5 | 3 | 3 | 3 | 3 |
| f12c（第 2307–2316 行） | 3 | 7 / 5 | 7 / 5 | — | — | — | — | — | — | — |
| f12d（第 2692–2701 行） | 7 / 3 | 7 / 3 | 7 / 3 | — | — | — | — | — | — | — |

f5 另有 AJ2 一列（第二轮口径 2、`_slot_only` 3，第 346、608 行）。

跨臂的几行（整行抄，`c143r3_model.out`）：

- `hit=C not_hit=A` 在全部 16 个格（8 族 × 两种口径）都是 `scripts=0`，例如第 134 行：
  `name=f1_cj_shift pair hit=C not_hit=A scripts=0 min_fs=None min_fr=None arm_C_min_fs=2 arm_A_min_fs=1`
- CJ / CJ2 中而丙不中的，只在下面几行不是 0（整行）：

```
name=f1_cj_shift pair hit=CJ not_hit=C scripts=1158 min_fs=6 min_fr=4 arm_CJ_min_fs=5 arm_C_min_fs=2
name=f1_cj_shift_slot_only pair hit=CJ not_hit=C scripts=68 min_fs=10 min_fr=6 arm_CJ_min_fs=9 arm_C_min_fs=3
name=f5_rb_crash_mount pair hit=CJ2 not_hit=C scripts=75 min_fs=5 min_fr=3 arm_CJ2_min_fs=5 arm_C_min_fs=2
name=f5_rb_crash_mount_slot_only pair hit=CJ2 not_hit=C scripts=2 min_fs=10 min_fr=6 arm_CJ2_min_fs=9 arm_C_min_fs=3
name=f12b_three_rollbacks pair hit=CJ not_hit=C scripts=32 min_fs=6 min_fr=4 arm_CJ_min_fs=5 arm_C_min_fs=2
name=f12b_three_rollbacks pair hit=CJ2 not_hit=C scripts=32 min_fs=6 min_fr=4 arm_CJ2_min_fs=5 arm_C_min_fs=2
name=f12d_rb_crash_mount_rb pair hit=CJ2 not_hit=C scripts=2 min_fs=7 min_fr=5 arm_CJ2_min_fs=5 arm_C_min_fs=2
```

- 我的线索 CR2 中而丙不中（整行，第 2689 行）——CR2 把新实例的 txg 往后推了 24 格之后，落点撞上了丙在同一段历史上避开的那个出生代，是 CR2 自己比丙差的地方（第 2.6 节）：

```
name=f12d_rb_crash_mount_rb pair hit=CR2 not_hit=C scripts=8 min_fs=5 min_fr=5 arm_CR2_min_fs=2 arm_C_min_fs=2
```

f1 的七条 `arm=` 行（第 9–15 行，整行抄）：

```
name=f1_cj_shift arm=A runs=169428 violating=78732 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=78732 by_fs=0:0/125;1:118/871;2:667/3060;3:2019/7061;4:4196/12174;5:6777/16850;6:9074/20177;7:10440/21746;8:10714/21264;9:9893/18666;10:8237/14806;11:6098/10964;12:4022/7878;13:2557/5612;14:1748/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=f1_cj_shift arm=AJ runs=169428 violating=51493 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=51493 by_fs=0:0/125;1:0/871;2:55/3060;3:390/7061;4:1242/12174;5:2626/16850;6:4243/20177;7:5876/21746;8:7215/21264;9:7678/18666;10:6907/14806;11:5315/10964;12:3627/7878;13:2415/5612;14:1732/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=f1_cj_shift arm=G runs=169428 violating=46000 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=46000 by_fs=0:0/125;1:0/871;2:55/3060;3:390/7061;4:1242/12174;5:2546/16850;6:3951/20177;7:5129/21746;8:5993/21264;9:6413/18666;10:6034/14806;11:4805/10964;12:3295/7878;13:2275/5612;14:1700/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=f1_cj_shift arm=C runs=169428 violating=9144 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=9144 by_fs=0:0/125;1:0/871;2:2/3060;3:33/7061;4:274/12174;5:699/16850;6:1075/20177;7:1298/21746;8:1393/21264;9:1376/18666;10:1187/14806;11:804/10964;12:454/7878;13:221/5612;14:134/3926;15:110/2504;16:68/1264;17:16/416;18:0/64
name=f1_cj_shift arm=CJ runs=169428 violating=3075 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=3075 by_fs=0:0/125;1:0/871;2:0/3060;3:0/7061;4:0/12174;5:2/16850;6:18/20177;7:113/21746;8:356/21264;9:606/18666;10:659/14806;11:496/10964;12:315/7878;13:182/5612;14:134/3926;15:110/2504;16:68/1264;17:16/416;18:0/64
name=f1_cj_shift arm=CK runs=169428 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/125;1:0/871;2:0/3060;3:0/7061;4:0/12174;5:0/16850;6:0/20177;7:0/21746;8:0/21264;9:0/18666;10:0/14806;11:0/10964;12:0/7878;13:0/5612;14:0/3926;15:0/2504;16:0/1264;17:0/416;18:0/64
name=f1_cj_shift arm=CR runs=169428 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/125;1:0/871;2:0/3060;3:0/7061;4:0/12174;5:0/16850;6:0/20177;7:0/21746;8:0/21264;9:0/18666;10:0/14806;11:0/10964;12:0/7878;13:0/5612;14:0/3926;15:0/2504;16:0/1264;17:0/416;18:0/64
```

### 2.2 攻击面一：CJ / CJ2 把第一个新根往后推，新对象落在被抛弃对象的出生代上

**f1（第二轮 s2 的空间，169428 条脚本）**：CJ 中而丙不中 1158 条，其中最少 6 / 4 个故障（第 116 行）；CJ 自己最少 5 / 3，丙最少 2。`_slot_only` 下 68 条，最少 10 / 6，丙 3（第 265 行）。故障最少的那一段（第 117–133 行，整行抄）：

```
example name=f1_cj_shift pair hit=CJ not_hit=C fs=6 fr=4 script=[p(0) f(0) p(0) p(1) rb(1,d0+r1+j3,0) u(2)]
  violations=[((12, 2, 9), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 wm=12:2
    txg5 i1: record c5; root write to disk0 FAILS
      switch -> i2
    txg6 i2 c6 root->disk0 wm=12:2
    txg7 i2 c7 root->disk1 wm=12:2
    txg8 i2 c8 root->disk0 wm=12:2
    txg9 i2 c9 root->disk0 creates (12,2,bg9) wm=12:3
    ROLLBACK u=d0+r1+j3: R_old=txg4 i1; readable ring max txg4; k_bad=22; first new root txg7 i3; records from c5; wm=12:2
    txg7 i3 c5 root->disk1 wm=12:2
    txg8 i3 c6 root->disk0 wm=12:2
    txg9 i3 c7 root->disk0 creates (12,2,bg9) wm=12:3
    txg10 i3 c8 root->disk1 creates (12,3,bg10) wm=12:4
```

读法：txg 5 那次发布根槽写失败（记录 c5 已经落了），切换到 i2 重发，被抛弃时间线在 txg 9 建了 inode 2（盘 0）。回退时盘 0 整块、盘 1 上 txg 7 的根、最新 3 条记录在盘 1 上的副本读不出，外加那次根槽写失败：6 个故障。CJ 读到 c6（txg 6，盘 1 那份）⇒ 第一个新根 7 ⇒ 暖机后新对象落在 9，(12, 2, 9) 被重发。丙只看根：可读根最大 txg 4 ⇒ 第一个新根 5 ⇒ 落在 8，不撞。

**同一段历史上比（f1x：每个前缀单独扫，盘 × 最新 0–10 个根槽 × 最新 0–10 条记录，比 f1 宽，丙在一个前缀上的最少故障不会被截在 3 个根槽）**，第 313–314、327–328 行（整行抄）：

```
name=f1x_same_history prefixes_with_CJ_hit_C_not=1104 C_never_hit_on_prefix=158 CJnotC_fewer_fs_than_C_on_prefix=6 CJnotC_fewer_fr_than_C_on_prefix=66 min_CJnotC_fs=6 min_CJnotC_fr=4 CR_hits=0
name=f1x_same_history gap(CJnotC - C on same prefix) fs_min=-1 fr_min=-3 fs_hist={-1: 6, 0: 3, 2: 33, 3: 14, 4: 123, 5: 143, 6: 142, 7: 157, 8: 80, 9: 72, 10: 60, 11: 67, 12: 15, 13: 4, 14: 6, 15: 12, 16: 5, 18: 1, 19: 3}
name=f1x_same_history_slot_only prefixes_with_CJ_hit_C_not=1104 C_never_hit_on_prefix=172 CJnotC_fewer_fs_than_C_on_prefix=0 CJnotC_fewer_fr_than_C_on_prefix=28 min_CJnotC_fs=10 min_CJnotC_fr=6 CR_hits=0
name=f1x_same_history_slot_only gap(CJnotC - C on same prefix) fs_min=1 fr_min=-3 fs_hist={1: 8, 2: 3, 5: 32, 6: 12, 7: 102, 8: 5, 9: 86, 10: 35, 11: 144, 12: 26, 13: 109, 14: 29, 15: 87, 16: 14, 17: 89, 18: 22, 19: 67, 20: 9, 21: 25, 22: 14, 23: 5, 24: 2, 25: 3, 26: 4}
```

- 有 CJ 中而丙不中脚本的前缀 1104 个。其中 **158 个前缀上丙在整个扫描空间里一次都不中**（`_slot_only` 172 个），而 CJ 中；**6 个前缀上 CJ 中而丙不中的那一段比丙在同一个前缀上最少要的故障还少 1 个**（按槽计；`_slot_only` 0 个）；按区间计少的有 66 个（`_slot_only` 28 个）。
- 丙一次都不中的那种前缀，第 315 行是第一个（整行抄）：

```
name=f1x_same_history row pre=[p(0) p(0) f(0) p(1)] kk=1 C_min=[None, None] CJ_min=[6, 4] CJnotC_min=(6, 4) script=[p(0) p(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
```

  机制（推理，按上面那段日志与 f1y 的计数核，f1y 在 2.7）：txg 6 的根槽写失败，全仓没有 txg 6 的根（「写失败的槽按旧内容算」，`16-发布语义.md` 第 372 行），而那次发布的记录先于根落了盘（持久顺序，D16（发布语义） 已定项 7）。CJ 取的是「可读根与读得出的记录」的最大值，能把第一个新根放在 7，L(7) = 9 正是被抛弃对象的出生代；丙的第一个新根只能是「某个真实存在的根的 txg + 1」，这段历史上没有根 6，丙永远起不了 7，它的落点集合里没有 9。⇒ **CJ 的落点集合严格比丙大**（多出「只有记录、没有根的 txg + 1」这些点），多出来的点可以正好落在被抛弃对象的出生代上。

**回退之后的挂载（f5，CJ2）**：CJ2 中而丙不中 75 条，最少 5 / 3（第 565 行）；`_slot_only` 2 条、10 / 6（第 827 行）。f12b（三次回退）CJ / CJ2 中而丙不中各 32 条、最少 6 / 4（第 1886、1887 行）；f12d CJ2 中而丙不中 2 条、7 / 5（第 2676 行）。f5 那一段（第 566–582 行，整行抄）：

```
example name=f5_rb_crash_mount pair hit=CJ2 not_hit=C fs=5 fr=3 script=[p(1) rb(2,none,0) u(2) crash mount(d0+r1+j3) u(1)]
  violations=[((12, 3, 9), [4, 5], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 creates (12,2,bg4) wm=12:3
    ROLLBACK u=none: R_old=txg3 i1; readable ring max txg4; k_bad=19; first new root txg5 i2; records from c4; wm=12:2
    txg5 i2 c4 root->disk0 wm=12:2
    txg6 i2 c5 root->disk0 wm=12:2
    txg7 i2 c6 root->disk1 wm=12:2
    txg8 i2 c7 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i2 c8 root->disk0 creates (12,3,bg9) wm=12:4
    CRASH (nothing in flight)
    MOUNT u=d0+r1+j3: selects txg4 i1, replays to txg4 (prefix end c4), new i3 from txg7, wm=12:3 [selected root is on an abandoned timeline]
    txg7 i3 c5 root->disk1 wm=12:3
    txg8 i3 c6 root->disk0 wm=12:3
    txg9 i3 c7 root->disk0 creates (12,3,bg9) wm=12:4
```

**按跑前第二条反向接受条款判**（「丙 + CJ / CJ2 在 U1 被一段『CJ 中而丙不中、而且要的故障比丙在同一类里少』的历史打中 ⇒ CJ / CJ2 出局，丙原样」），「同一类」有两种读法，两种都照实摆：

| 读法 | CJ 中而丙不中的最少故障 | 丙在「同一类」里的最少故障 | 判 |
|---|---|---|---|
| 同一类 = 同一种攻击形状（最新的根与记录读不出） | 6 / 4（`_slot_only` 10 / 6） | 2（`_slot_only` 3） | 不触发——2.0 第 6 条说明这一读法下结构上不可能触发 |
| 同一类 = 同一段历史（同一个前缀，只换故障） | 158 个前缀上 CJ 中、丙在扫描空间里一次都不中；6 个前缀上 CJ 少 1 个（按槽）；66 个前缀上按区间计 CJ 少 | 同一前缀上丙的最少 | **触发** |

我的判断（交主 agent 评判）：条款要答的是「CJ 把丙在某些历史上变差了没有」，第二种读法才问到这件事；第一种读法把「CJ 中」与「丙中」放在两段不同的历史上比，比出来的是两条臂各自的最弱点，而丙的最弱点那段历史 CJ 本来也挡得住（f1 第 135 行 `pair hit=C not_hit=CJ scripts=7227`）。按第二种读法，CJ / CJ2 按字面出局。但它要的故障（6 / 4 起、含一次根槽写失败）远多于丙自己的 2，交用户时这一点要一起摆。

### 2.3 攻击面二：被抛弃时间线里的克隆头、销毁的头、树 ID 重发与回退交织

f11 的脚本空间：克隆头建在 R_old 之前或之后（之后那一格就是第一轮 H6 的形状：树 ID 可能被重发）；之前 0–1 次空发布；被抛弃时间线里在克隆头里建 1–2 个对象；销毁或不销毁克隆头；再 0–2 次空发布；回退到 R_old，故障取 F60（盘 × 最新 0–3 个根槽 × 最新 0–4 条记录）；回退后再从头 12 克一个新头、在里面建 2 个对象。有效脚本 1495 条。各臂那一行（第 870–879 行，整行抄）：

```
name=f11_clones arm=A runs=1495 violating=155 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=155 by_fs=0:0/48;1:1/96;2:14/185;3:26/197;4:25/213;5:29/205;6:24/165;7:17/114;8:5/102;9:6/86;10:2/46;11:6/38
name=f11_clones arm=AJ runs=1495 violating=155 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=155 by_fs=0:0/48;1:1/96;2:14/185;3:26/197;4:25/213;5:29/205;6:24/165;7:17/114;8:5/102;9:6/86;10:2/46;11:6/38
name=f11_clones arm=G runs=1495 violating=87 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=87 by_fs=0:0/48;1:0/96;2:1/185;3:3/197;4:10/213;5:18/205;6:20/165;7:16/114;8:5/102;9:6/86;10:2/46;11:6/38
name=f11_clones arm=C runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CJ runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CJ2 runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CK runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CK2 runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CR runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
name=f11_clones arm=CR2 runs=1495 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/48;1:0/96;2:0/185;3:0/197;4:0/213;5:0/205;6:0/165;7:0/114;8:0/102;9:0/86;10:0/46;11:0/38
```

树 ID 重发（第 960、963 行，整行抄；十条臂都是 155 条、最少 1 个故障，`_slot_only` 40 条、最少 2 个，第 1078–1091 行）：

```
name=f11_clones arm=A tree_id_reissued_runs=155 tree_id_reissued_min_fs=1 invalid_where_C_valid=0
name=f11_clones arm=C tree_id_reissued_runs=155 tree_id_reissued_min_fs=1 invalid_where_C_valid=0
```

G 中而丙不中（第 978 行，整行抄）：

```
name=f11_clones pair hit=G not_hit=C scripts=87 min_fs=2 min_fr=2 arm_G_min_fs=2 arm_C_min_fs=None
```

读法：

- 丙、CJ、CJ2 与我的三条线索在这一族一次都不中。这是 2.0 的直接推论，不是这一族的发现：被抛弃时间线里的克隆对象都建在 R_old 之后 1–3 个 txg 内，而丙要撞上，被抛弃对象的出生代至少是 T_old + 3，新克隆头的对象更晚（回退根、暖机、建克隆那次发布之后）。**这一族的射程**：离 R_old 更远的克隆对象我没有单独取样；按 2.0，那一格与 f1 同形（引理里不出现树 ID），丙要的仍是连续 3 个 txg 的根读不出。
- 树 ID 重发（第一轮 H6）十条臂一样多、一样少——它们都用今天那条「树 ID 水位 = 可读根里的最大值」（`08-核心索引结构.md` 已定项 8 ②），这一格不分辨臂，病根照第一轮判决另记。重发的树 ID 不给丙新添一条路：丙的身份里带出生代，新克隆头里的对象出生代比每个可读根都大，要撞上被抛弃克隆头里的对象，要藏的根与 2.0 完全一样。
- 销毁的头（第二轮 H9 的形状，被抛弃时间线里销毁克隆头）：丙、CJ 仍是 0，与第二轮 s1 一致；G 2 个故障（被抛弃对象就在最新那次发布里，与 2.5 同形）。

### 2.4 攻击面三：回退的「前缀末」两种读法、回退尝试崩了再重做、连续多次回退

f12 的四个子族：(a) 回退尝试崩在记录之后（`rbxk`）、一次普通挂载、0–1 次空发布、再回退，故障在尝试、挂载、再回退三处各取（FA × FM × F36），前缀末两种读法各跑一遍（P1 = R_old 覆盖的最后一条 + 1，`rbp=rold`；P2 = 从 R_old 往后整条可读链的末尾 + 1，`rbp=scan`），孤儿单元当场复用（对 AJ 最坏）；(b) 连续三次回退，前两次干净，第三次带 F60；(c) 背靠背两次回退（中间没有用户发布）；(d) 回退、用户发布、崩溃、挂载（F18）、再回退（F18）。各臂那一行（整行抄；P1 第 1114–1123 行、三次回退第 1602–1611 行、回退—崩溃—挂载—回退第 2426–2435 行）：

```
name=f12a_attempt_redo rbp=rold arm=A runs=9459 violating=5223 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=5223 by_fs=0:0/24;1:32/228;2:100/424;3:225/651;4:573/1035;5:768/1294;6:805/1401;7:820/1272;8:589/947;9:442/721;10:333/574;11:293/487;12:206/326;13:36/68;14:1/7
name=f12a_attempt_redo rbp=rold arm=AJ runs=9459 violating=4740 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=4740 by_fs=0:0/24;1:32/228;2:84/424;3:166/651;4:490/1035;5:685/1294;6:714/1401;7:763/1272;8:539/947;9:413/721;10:322/574;11:290/487;12:205/326;13:36/68;14:1/7
name=f12a_attempt_redo rbp=rold arm=G runs=9459 violating=3162 min_fs=3 min_fr=3 viol_mount_on_abandoned=0 viol_other=3162 by_fs=0:0/24;1:0/228;2:0/424;3:36/651;4:284/1035;5:452/1294;6:475/1401;7:549/1272;8:388/947;9:313/721;10:234/574;11:227/487;12:173/326;13:30/68;14:1/7
name=f12a_attempt_redo rbp=rold arm=C runs=9459 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/424;3:0/651;4:0/1035;5:0/1294;6:0/1401;7:0/1272;8:0/947;9:0/721;10:0/574;11:0/487;12:0/326;13:0/68;14:0/7
name=f12a_attempt_redo rbp=rold arm=CJ runs=9459 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/424;3:0/651;4:0/1035;5:0/1294;6:0/1401;7:0/1272;8:0/947;9:0/721;10:0/574;11:0/487;12:0/326;13:0/68;14:0/7
name=f12a_attempt_redo rbp=rold arm=CJ2 runs=9381 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/421;3:0/622;4:0/1011;5:0/1280;6:0/1393;7:0/1260;8:0/930;9:0/748;10:0/586;11:0/478;12:0/320;13:0/75;14:0/5
name=f12a_attempt_redo rbp=rold arm=CK runs=9459 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/424;3:0/651;4:0/1035;5:0/1294;6:0/1401;7:0/1272;8:0/947;9:0/721;10:0/574;11:0/487;12:0/326;13:0/68;14:0/7
name=f12a_attempt_redo rbp=rold arm=CK2 runs=8316 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/210;2:0/375;3:0/533;4:0/803;5:0/1122;6:0/1274;7:0/1130;8:0/876;9:0/627;10:0/538;11:0/437;12:0/266;13:0/88;14:0/12;15:0/1
name=f12a_attempt_redo rbp=rold arm=CR runs=9459 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/424;3:0/651;4:0/1035;5:0/1294;6:0/1401;7:0/1272;8:0/947;9:0/721;10:0/574;11:0/487;12:0/326;13:0/68;14:0/7
name=f12a_attempt_redo rbp=rold arm=CR2 runs=9459 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/24;1:0/228;2:0/424;3:0/651;4:0/1035;5:0/1294;6:0/1401;7:0/1272;8:0/947;9:0/721;10:0/574;11:0/487;12:0/326;13:0/68;14:0/7
name=f12b_three_rollbacks arm=A runs=3830 violating=3050 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=3050 by_fs=0:0/54;1:102/180;2:232/364;3:396/498;4:450/582;5:508/610;6:430/508;7:354/378;8:214/268;9:160/184;10:102/102;11:78/78;12:24/24
name=f12b_three_rollbacks arm=AJ runs=3830 violating=2440 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=2440 by_fs=0:0/54;1:0/180;2:48/364;3:208/498;4:342/582;5:480/610;6:430/508;7:354/378;8:214/268;9:160/184;10:102/102;11:78/78;12:24/24
name=f12b_three_rollbacks arm=G runs=3830 violating=2440 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=2440 by_fs=0:0/54;1:0/180;2:48/364;3:208/498;4:342/582;5:480/610;6:430/508;7:354/378;8:214/268;9:160/184;10:102/102;11:78/78;12:24/24
name=f12b_three_rollbacks arm=C runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CJ runs=3830 violating=231 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=231 by_fs=0:0/54;1:0/180;2:0/364;3:0/498;4:0/582;5:12/610;6:50/508;7:65/378;8:26/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CJ2 runs=3830 violating=231 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=231 by_fs=0:0/54;1:0/180;2:0/364;3:0/498;4:0/582;5:12/610;6:50/508;7:65/378;8:26/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CK runs=3620 violating=120 min_fs=3 min_fr=3 viol_mount_on_abandoned=0 viol_other=120 by_fs=0:0/54;1:0/178;2:0/356;3:12/478;4:12/546;5:12/568;6:12/470;7:12/344;8:12/246;9:12/178;10:12/102;11:12/76;12:12/24
name=f12b_three_rollbacks arm=CK2 runs=3620 violating=120 min_fs=3 min_fr=3 viol_mount_on_abandoned=0 viol_other=120 by_fs=0:0/54;1:0/178;2:0/356;3:12/478;4:12/546;5:12/568;6:12/470;7:12/344;8:12/246;9:12/178;10:12/102;11:12/76;12:12/24
name=f12b_three_rollbacks arm=CR runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CR2 runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12d_rb_crash_mount_rb arm=A runs=7472 violating=4240 min_fs=2 min_fr=2 viol_mount_on_abandoned=272 viol_other=3968 by_fs=0:0/18;1:0/152;2:55/371;3:235/615;4:299/621;5:404/714;6:676/1076;7:545/891;8:482/782;9:604/922;10:489/707;11:282/384;12:121/163;13:40/48;14:8/8
name=f12d_rb_crash_mount_rb arm=AJ runs=7472 violating=3124 min_fs=2 min_fr=2 viol_mount_on_abandoned=228 viol_other=2896 by_fs=0:0/18;1:0/152;2:9/371;3:37/615;4:67/621;5:270/714;6:512/1076;7:392/891;8:392/782;9:537/922;10:463/707;11:280/384;12:117/163;13:40/48;14:8/8
name=f12d_rb_crash_mount_rb arm=G runs=7472 violating=2733 min_fs=4 min_fr=2 viol_mount_on_abandoned=125 viol_other=2608 by_fs=0:0/18;1:0/152;2:0/371;3:0/615;4:36/621;5:237/714;6:463/1076;7:346/891;8:358/782;9:467/922;10:397/707;11:264/384;12:117/163;13:40/48;14:8/8
name=f12d_rb_crash_mount_rb arm=C runs=7472 violating=532 min_fs=2 min_fr=2 viol_mount_on_abandoned=40 viol_other=492 by_fs=0:0/18;1:0/152;2:4/371;3:21/615;4:22/621;5:39/714;6:104/1076;7:82/891;8:52/782;9:64/922;10:59/707;11:45/384;12:30/163;13:10/48;14:0/8
name=f12d_rb_crash_mount_rb arm=CJ runs=7472 violating=174 min_fs=5 min_fr=3 viol_mount_on_abandoned=18 viol_other=156 by_fs=0:0/18;1:0/152;2:0/371;3:0/615;4:0/621;5:8/714;6:40/1076;7:20/891;8:10/782;9:22/922;10:22/707;11:24/384;12:22/163;13:6/48;14:0/8
name=f12d_rb_crash_mount_rb arm=CJ2 runs=7540 violating=102 min_fs=5 min_fr=3 viol_mount_on_abandoned=14 viol_other=88 by_fs=0:0/18;1:0/152;2:0/363;3:0/575;4:0/654;5:2/751;6:23/1072;7:20/921;8:0/798;9:6/898;10:16/703;11:23/412;12:12/167;13:0/48;14:0/8
name=f12d_rb_crash_mount_rb arm=CK runs=6448 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/18;1:0/152;2:0/364;3:0/546;4:0/514;5:0/646;6:0/893;7:0/736;8:0/698;9:0/789;10:0/572;11:0/349;12:0/135;13:0/36
name=f12d_rb_crash_mount_rb arm=CK2 runs=6396 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/18;1:0/148;2:0/341;3:0/480;4:0/538;5:0/570;6:0/732;7:0/786;8:0/717;9:0/768;10:0/741;11:0/386;12:0/141;13:0/28;14:0/2
name=f12d_rb_crash_mount_rb arm=CR runs=7472 violating=424 min_fs=2 min_fr=2 viol_mount_on_abandoned=40 viol_other=384 by_fs=0:0/18;1:0/152;2:4/371;3:21/615;4:22/621;5:39/714;6:80/1076;7:44/891;8:52/782;9:64/922;10:35/707;11:29/384;12:30/163;13:4/48;14:0/8
name=f12d_rb_crash_mount_rb arm=CR2 runs=7464 violating=328 min_fs=2 min_fr=2 viol_mount_on_abandoned=16 viol_other=312 by_fs=0:0/18;1:0/152;2:4/371;3:19/615;4:20/617;5:35/716;6:58/1068;7:38/879;8:46/796;9:32/914;10:35/707;11:23/392;12:14/163;13:4/48;14:0/8
```

读法：

- **前缀末两种读法对丙与 CJ 都没有影响**：f12a 两种读法下丙、CJ、CR、CR2 一次都不中，两种口径都是（第 1117–1118、1361–1362 行与各自的 `_slot_only`）。CJ2 同样一次都不中，但有 210 条脚本对 CJ2 无效而对丙有效（第 1213、1457 行）。我单独核了第一条（`c143r3_debug.out` 的 d2 节）：回退尝试留在环里的那条记录（txg 5）让 CJ2 在普通挂载时从 6 起步，新实例少写了一个根，之后那次回退在 `d1+r2` 下读得出的候选只剩头 12 出现之前的根，脚本按定义无效——不是挂不上，是回退候选集变了。
- **连续多次回退不给丙新添一条路**：三次回退里丙最少 2（`_slot_only` 3），CJ / CJ2 5 / 3（9 / 5）；丙中的那几段都是 2.0 的形状（后一次回退那一刻藏住最新 3 个 txg 的根），例：`c143r3_debug.out` 的 d5 节，第三次回退藏 3 个根槽，(12, 3, 9) 被第二次回退之后建过的对象与第三次回退之后的新对象各用一次。背靠背两次回退里丙、CJ、CJ2 一次都不中（第 2191–2193 行）。回退—崩溃—挂载—回退里丙 2、CJ / CJ2 5 / 3；`_slot_only` 下丙一次都不中（第 2695 行），而甲 / 乙 / G 要 7 / 3（第 2692–2694 行）。
- 甲 / 乙在 f12a 两种读法下都是 1 个故障（第 1114、1358 行），G 3 个；与第二轮 s4 同形。

⇒ 攻击面三这一轮没有打中丙：丙在这四个子族里的最少故障没有低于 2.0 的界（第二轮口径 2、`_slot_only` 3），每一段丙中的脚本甲也中（`hit=C not_hit=A` 在 f12 的八个格里都是 0，例如第 1224、1888、2677 行）。

### 2.5 攻击面四：G 的余量

G 在各族的最少故障（2.1 总表那一列）：第二轮口径 2（f1、f5、f11、f12b）、3（f12a）、4 / 3（f12c）、4 / 2（f12d）；`_slot_only` 3、3、4、3、3、7 / 5、7 / 3。G 中而丙不中的脚本在 16 个格里没有一个是 0；G 中而 AJ 不中的在 16 个格里全是 0（整行抄）：

```
148:name=f1_cj_shift pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=2 arm_AJ_min_fs=2
297:name=f1_cj_shift_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=3
586:name=f5_rb_crash_mount pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=2 arm_AJ_min_fs=2
848:name=f5_rb_crash_mount_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=3
979:name=f11_clones pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=2 arm_AJ_min_fs=1
1097:name=f11_clones_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=4 arm_AJ_min_fs=2
1227:name=f12a_attempt_redo rbp=rold pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=1
1346:name=f12a_attempt_redo rbp=rold_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=3
1471:name=f12a_attempt_redo rbp=scan pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=1
1590:name=f12a_attempt_redo rbp=scan_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=3
1891:name=f12b_three_rollbacks pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=2 arm_AJ_min_fs=2
2176:name=f12b_three_rollbacks_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=3 arm_AJ_min_fs=3
2295:name=f12c_back_to_back pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=4 arm_AJ_min_fs=4
2414:name=f12c_back_to_back_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=7 arm_AJ_min_fs=7
2680:name=f12d_rb_crash_mount_rb pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=4 arm_AJ_min_fs=2
2803:name=f12d_rb_crash_mount_rb_slot_only pair hit=G not_hit=AJ scripts=0 min_fs=None min_fr=None arm_G_min_fs=7 arm_AJ_min_fs=7
```

G 中而丙不中、故障最少的那一段（f1，第二轮口径，从第 137 行起整行抄到下一条 `name=` 行之前）：

```
example name=f1_cj_shift pair hit=G not_hit=C fs=2 fr=2 script=[p(1) rb(1,d1+j1,0) u(2)]
  violations=[((12, 2), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 creates (12,2,bg4) wm=12:3
    ROLLBACK u=d1+j1: R_old=txg3 i1; readable ring max txg3; k_bad=21; first new root txg4 i2; records from c4; wm=12:2
    txg4 i2 c4 root->disk1 wm=12:2
    txg5 i2 c5 root->disk0 wm=12:2
    txg6 i2 c6 root->disk0 creates (12,2,bg6) wm=12:3
    txg7 i2 c7 root->disk1 creates (12,3,bg7) wm=12:4
```

`_slot_only` 下同形的那一段是 `p(1) rb(1,r1+j1,0) u(2)`，3 个故障（第 286 行）。

读法：txg 4 在头 12 里建了 inode 2（根在盘 1），那时本实例早已覆盖两块盘，fsync 返回过（`violations` 里的 `True`）。回退到 txg 3 时盘 1 整块读不出、盘 0 上 c4 那份记录也读不出：2 个故障。G 能读到的根与记录头里全池号水位最大是 2，新对象又拿到 inode 2——按号判，一个已发布、已确认的号被重发。丙在同一段历史上：第一个新根 4，暖机后新对象落在 6，(12, 2, 6) ≠ (12, 2, 4)。

⇒ **G 的余量就是「抬过全池号水位的最新那次发布，它的根与两份记录副本」**：藏住这三样（第二轮口径 2 个故障、`_slot_only` 3 个）G 就中。对活着的头，G 与 AJ 同一个载体（G 中 ⇒ AJ 中，16 格全是 0）；G 比 AJ 强的只在头销毁、journal 覆写与回退尝试崩了再重做那几格（f12a 里 AJ 1、G 3）。这一形与第二轮 H14（普通崩溃后最新的根与它的记录两份副本读不出、一次已确认的写丢掉）同一个物理形状，差别在于这里没有写丢——被抛弃的对象是管理员有意抛弃的，重发它的号是 U1 那一格的违例，丙在同一段历史上不中。**按跑前第三条反向接受条款（G 在 U1 被分辨臂的历史打中 ⇒ G 出局），按字面触发。**

### 2.6 我自己提的收严 CK / CR：被我自己的模型打穿

这两条都只在我的模型上量过；下面是这一轮我拿同一套族攻它们的结果，算「被攻过一轮、被打穿」。

**CK（第一个新根 txg = 可读根最大 + 1 + 读不出可读根的根槽数）**：f1 一次都不中（第 14 行），可 f5 里有 1219 条脚本对 CK 无效而对丙有效（第 557 行，整行抄）：

```
name=f5_rb_crash_mount arm=CK runs=5208 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/33;1:0/128;2:0/289;3:0/421;4:0/532;5:0/565;6:0/579;7:0/571;8:0/513;9:0/383;10:0/288;11:0/206;12:0/179;13:0/165;14:0/140;15:0/103;16:0/72;17:0/35;18:0/6
name=f5_rb_crash_mount arm=CK2 runs=5208 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/33;1:0/128;2:0/289;3:0/421;4:0/532;5:0/565;6:0/579;7:0/571;8:0/513;9:0/383;10:0/288;11:0/206;12:0/179;13:0/165;14:0/140;15:0/103;16:0/72;17:0/35;18:0/6
name=f5_rb_crash_mount arm=CK tree_id_reissued_runs=0 tree_id_reissued_min_fs=None invalid_where_C_valid=1219
```

`c143r3_debug.out` 的 d1 节是第一条：`rb(1,none,0) u(1) crash mount(r4) u(1)`。CK 回退时从 txg 3 跳到 24，而根环的槽位是 txg 的函数（`22-单元原子性怎么合成.md` 第 1013 行所在的已定项 16：区域内槽位 = `(txg div R) mod S`），跳过去之后新实例的根落在第 0 代根与 txg 1、2 的根所在的槽上，把它们覆写掉；之后崩溃、4 个根槽读不出时环里一个可读根都不剩，挂不上。丙在同一段脚本上环里还有 txg 0–6，挂得上。⇒ **跳号会把根环的轮转从「覆写最旧的」变成「覆写最新的」**，缩了回退候选集，严格比丙差。f12b 里 CK 另被 120 条打中、最少 3 个故障（第 1608 行）。

**CR（改成恰好跳 R·S = 24，写的槽与丙完全相同）**：轮转那个毛病没了（f5 里 CR / CR2 无效 0 条），f1、f11、f12a、f12c 一次都不中；可是（整行抄）：

```
name=f5_rb_crash_mount arm=CR runs=6427 violating=170 min_fs=6 min_fr=4 viol_mount_on_abandoned=0 viol_other=170 by_fs=0:0/33;1:0/128;2:0/291;3:0/455;4:0/607;5:0/697;6:4/750;7:10/748;8:14/686;9:28/543;10:32/427;11:20/303;12:16/227;13:10/174;14:10/141;15:10/103;16:10/73;17:6/35;18:0/6
name=f5_rb_crash_mount arm=CR2 runs=6427 violating=898 min_fs=2 min_fr=2 viol_mount_on_abandoned=105 viol_other=793 by_fs=0:0/33;1:0/128;2:13/291;3:50/455;4:88/607;5:121/697;6:132/750;7:126/748;8:116/686;9:94/543;10:66/427;11:37/303;12:18/227;13:10/174;14:7/141;15:8/103;16:7/73;17:5/35;18:0/6
name=f12b_three_rollbacks arm=C runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CR runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12b_three_rollbacks arm=CR2 runs=3830 violating=565 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=565 by_fs=0:0/54;1:0/180;2:13/364;3:79/498;4:86/582;5:101/610;6:86/508;7:88/378;8:34/268;9:27/184;10:12/102;11:27/78;12:12/24
name=f12d_rb_crash_mount_rb arm=CR runs=7472 violating=424 min_fs=2 min_fr=2 viol_mount_on_abandoned=40 viol_other=384 by_fs=0:0/18;1:0/152;2:4/371;3:21/615;4:22/621;5:39/714;6:80/1076;7:44/891;8:52/782;9:64/922;10:35/707;11:29/384;12:30/163;13:4/48;14:0/8
name=f12d_rb_crash_mount_rb arm=CR2 runs=7464 violating=328 min_fs=2 min_fr=2 viol_mount_on_abandoned=16 viol_other=312 by_fs=0:0/18;1:0/152;2:4/371;3:19/615;4:20/617;5:35/716;6:58/1068;7:38/879;8:46/796;9:32/914;10:35/707;11:23/392;12:14/163;13:4/48;14:0/8
name=f12d_rb_crash_mount_rb pair hit=CR2 not_hit=C scripts=8 min_fs=5 min_fr=5 arm_CR2_min_fs=2 arm_C_min_fs=2
```

- 三次回退里 CR 与丙逐条相同（都是 565 条违例，第 1898、1899 行两个方向的 `pair` 都是 0 条）：每次回退都整体平移 24，第二次回退之后建的对象离「第三次回退那一刻读得出的最大 txg」隔着一段空当，固定跳 24 罩不住它（d5 节：第三次回退藏 3 个根槽，CR 在 (12, 3, 57) 上撞，与丙在 (12, 3, 9) 上撞是同一形）。
- CR2 在普通挂载也跳：f5 最少 2，与丙相同；f12d 里 8 条 CR2 中而丙不中（d4 节：`f(1) rb(1,none,0) u(1) crash mount(d0+r2) rb(1,d0+r2,0) u(1)`，5 个故障）。d3 节是最短的形状：回退时 CR 跳到 28，崩溃后挂载藏 4 个根槽，挂载选回 txg 2、新实例 CR2 再跳 24 又回到 28，(12, 2, 30) 被重发。

⇒ 「按读不出的槽数把 txg 往后推」这一族收严的前提是 txg 在环里连续；而跳号本身就在 txg 序列里留下空当，下一次回退或挂载的「读不出几个槽」就界不住藏住的 txg。**要界住藏住的 txg，只能从某个记着它的地方读出来**（记录——就是 CJ；或者一个新的持久字段）。这一条交主 agent，建议另记一笔（第十节第 2 条）。CK / CR 两个形态我撤回，不再当候选。

### 2.7 f1y：同一段历史上丙一次都不中的前缀，全都带一次根槽写失败

f1y 是 f1x 的明细，印在输出最后（f1x 那几行不变；第 2815 行起）。整行抄：

```
name=f1y_detail C_never_hit_prefixes=158 of_which_prefix_has_a_root_write_failure=158
name=f1y_detail fewer_fs pre=[f(0) p(1) f(0) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(0) p(1) f(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail fewer_fs pre=[f(0) p(1) f(1) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(0) p(1) f(1) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail fewer_fs pre=[f(0) f(1) p(0) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(0) f(1) p(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail fewer_fs pre=[f(1) p(1) f(0) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(1) p(1) f(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail fewer_fs pre=[f(1) p(1) f(1) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(1) p(1) f(1) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail fewer_fs pre=[f(1) f(1) p(0) f(0) p(1)] kk=1 C_min=[9, 9] CJnotC_min=(8, 6) script=[f(1) f(1) p(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)]
name=f1y_detail_slot_only C_never_hit_prefixes=172 of_which_prefix_has_a_root_write_failure=172
```

- 丙在扫描空间里一次都不中的 158 个前缀（`_slot_only` 172 个），**每一个都含至少一次根槽写失败**——与 2.2 的机制相符：写失败的那次发布留下一条有记录、没有根的 txg，CJ 的第一个新根能放在它后面一格，丙放不了。
- CJ 按槽计比丙少要 1 个故障的 6 个前缀（第 2817–2822 行）同样都带根槽写失败，而且都是两次；那里丙最少 9、CJ 中而丙不中最少 8 / 6——两边都远在 2.0 那一形的 2 / 3 之上。
- 按区间计 CJ 更少的例子在第 2823–2828 行（`_slot_only` 第 2830–2835 行）：CJ 那一段按槽计反而更多（例如 16 / 9 对丙的 10 / 10），少的只是「同一块盘上连续一段记录槽算一个故障」那一项。

同一段脚本上丙与 CJ 的日志（第 2836–2859 行，整行抄）：

```
example name=f1y_detail arm=C fs=6 fr=4 script=[p(0) p(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)] violations=[]
    txg6 i1: record c6; root write to disk0 FAILS
      switch -> i2
    txg7 i2 c7 root->disk1 wm=12:2
    txg8 i2 c8 root->disk0 wm=12:2
    txg9 i2 c9 root->disk0 creates (12,2,bg9) wm=12:3
    ROLLBACK u=d0+r1+j3: R_old=txg4 i1; readable ring max txg4; k_bad=22; first new root txg5 i3; records from c5; wm=12:2
    txg5 i3 c5 root->disk0 wm=12:2
    txg6 i3 c6 root->disk0 wm=12:2
    txg7 i3 c7 root->disk1 wm=12:2
    txg8 i3 c8 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i3 c9 root->disk0 creates (12,3,bg9) wm=12:4
example name=f1y_detail arm=CJ fs=6 fr=4 script=[p(0) p(0) f(0) p(1) rb(1,d0+r1+j3,0) u(2)] violations=[((12, 2, 9), [2, 3], True)]
    txg5 i1 c5 root->disk0 wm=12:2
    txg6 i1: record c6; root write to disk0 FAILS
      switch -> i2
    txg7 i2 c7 root->disk1 wm=12:2
    txg8 i2 c8 root->disk0 wm=12:2
    txg9 i2 c9 root->disk0 creates (12,2,bg9) wm=12:3
    ROLLBACK u=d0+r1+j3: R_old=txg4 i1; readable ring max txg4; k_bad=22; first new root txg7 i3; records from c5; wm=12:2
    txg7 i3 c5 root->disk1 wm=12:2
    txg8 i3 c6 root->disk0 wm=12:2
    txg9 i3 c7 root->disk0 creates (12,2,bg9) wm=12:3
    txg10 i3 c8 root->disk1 creates (12,3,bg10) wm=12:4
```

⇒ 攻击面一的结论：CJ / CJ2 把第一个新根往后推，确实会让新对象落到丙永远落不到的出生代上；可达，前提是被抛弃时间线里有过一次根槽写失败，外加把之后几次发布的根与记录都藏住（这一轮最少 6 / 4，`_slot_only` 10 / 6）。

## 三、U2：今天就存在、只认号、不随根回退、有条款定义的消费者

判据（背景材料第四节 U2 那一行）：写出一个只认号的消费者在某条臂下失效的读写序列，并指得出它今天住在哪条条款里。第二轮攻方腿第三节那张 14 行的表我没有重做一遍；这一轮只做两件事：① 现查 kb 在第二轮之后有没有新增按号的结构（`grep -rn 'inode 号'` 覆盖 `decisions/`、`invariants.md`、`checks-owed.md`、`verification-build.md`、`milestone-first-txn.md`、`first-txn-layout.md`：08 号 19 处、05 号 8 处、18 号 3 处、09 号 1 处、26 号 1 处、invariants 6 处、checks-owed 6 处、milestone 5 处、first-txn-layout 4 处，逐条读了上下文）；② 把第二轮表里没有的、身份里「含号」的结构逐个过一遍——它们都不是只认号，但它们的唯一性是不是悄悄靠着「号不复用」，丙要答。

| 结构 | 身份里有什么 | 住在哪 | 丙下判 |
|---|---|---|---|
| inode 叶容器的身份（出生树, 类型 2, 容器号, 容器出生代），右半容器号 = 触发分裂的新记录的 inode 号，出生代 = 那次发布的 txg | 号 + 出生代 | `08-核心索引结构.md` 第 369 行「容器身份不需要分配器」、第 372 行「唯一性的射程是一条已发布历史内」（D8（核心索引结构） 已定项 6）；`18-块里携带什么信息.md` 第 830 行（容器号那一行） | 触发分裂的记录是这次发布新建的对象，它的出生代就是这次发布的 txg ⇒ 容器身份撞 ⟺ 那个对象的 (树, 号, 出生代) 撞。丙保得住后者就保得住前者。**但条款的推理链要改**：第 372 行写「每个容器号都是某条记录首次插入时的 inode 号，号在树内不复用 ⇒ 同一棵树里两个容器不会同号」，丙之后同号的容器可以有两个（出生代不同），唯一性改由出生代那一段承担 |
| 跨头首次 COW 的重生：新身份 = (本头树 ID, 类型 2, 原容器号, 本次发布代号) | 号 + 出生代 | 同一段 | 同上：出生代 = 本次发布 txg，丙的保证（回退后新 txg 严格大于每个已发布的 txg）罩得住 |
| 墓碑与退役记录（对象 ID + 对象出生代 + 区间；退役四元组） | 号 + 出生代 | `18-块里携带什么信息.md` 第 749 行 | 带出生代；丙之后这一段从「按 day-1 契约保留」变成承重 |
| deadlist 条目 | 形态未定；题面写「按版本身份记……版本身份就是中央映射 key」 | `05-快照-空间记账机制.md` 第 516、518 行（D5（快照 / 空间记账机制） 未定项 12） | 中央映射 key 不含 inode 号（第二轮表）；deadlist 树 day-1 进树表、随根回退 ⇒ 不是只认号的消费者。未定项 12 定案时要复查 |
| checker 的 I-9.6「水位 > 该树全部墓碑记录的对象 ID」 | 只看号 | `invariants.md` 第 262 行 | checker 按可达遍历读不失效；按扫描读（把被抛弃时间线还没回收的墓碑单元也算进来）丙在 0 故障回退之后会红。它是 checker 的措辞，不是运行时消费者（第二轮已提），丙要改这一句 |
| 扫描重建时同号两版 | 容器按 (出生树, 容器号, 容器出生代) 分组，数据单元按五元组（含对象出生代） | `08-核心索引结构.md` 已定项 6「重建」那一段；`checks-owed.md` 第 154 行（C152（收窄后未发布号重发的扫描重建复核）） | 分组键里都有出生代 ⇒ 丙下同号两版分得开；被抛弃的那版由实例表的已发布谓词滤掉。C152 那条检查只覆盖「未发布的号被重发」，丙之后要加「已发布的号在回退后被重发、出生代不同」这一格，否则扫描重建对丙的这一新情形零覆盖（推理，没建模） |
| 实例表不可读时的级 1 重建按号接 extent（第一轮 H7） | 只看号 | `18-块里携带什么信息.md` 第 879 行「表不可读时」 | 普通崩溃之后三条臂都混接，不分辨臂（第一轮已判） |
| 对外暴露的 inode 号 | 只看号 | 没有条款：第一版不挂载（`17-实现分层与第三方管道.md` 第 95 行） | 今天没有消费者（第二轮辩方腿已判） |
| `deleted_inodes`、批量删除意图（`logged_ops`） | 「只存号」 | `08-核心索引结构.md` 第 380 行附近；`checks-owed.md` 第 124 行（C118（`deleted_inodes` 树的形态无落点）） | 形态没定，读写序列写不出来。C118 落定时若只记号、又不随根回退，丙在 0 故障回退之后会让它作用到新对象上 |
| `milestone-first-txn.md` 第 122 行「从 inode 树按 inode 号查到那条记录」 | 只看号 | 里程碑步 7 的验收 | 查的是活树，随根回退，不失效 |
| `26-后台整理与放置回收.md` 第 413 行「把 inode 号的唯一性范围外推到映射 key 而没写那一步外推」 | — | 一轮论证的材料自查记录 | 不是消费者 |

⇒ **U2 结论**：这一轮现查仍然没有一个同时满足「只认号 ∧ 不随根回退 ∧ 今天有条款定义」的消费者；跑前反向接受条款第一条（丙被一段分辨臂的历史打中）从 U2 这一格不触发。新找出的是两处**推理链靠着「号不复用」的已定条款**（容器身份唯一性那一句、C152 的射程），丙不让它们失效，但要改它们的依据句（第六节）。

## 四、U3：代价——G 改第一个事务的哪些字节，CJ / CJ2 在回退与挂载时多读什么

记号：J = journal 环的记录槽数 = 环长 ÷ 4096（默认环 768 MiB ⇒ 196608；环长是 mkfs 参数，约束环 ≤ 设备容量 ÷ 4，`23-journal的角色与格式.md` 第 1270 行附近已定项 19 ③）；R·S = 根槽数（第一版 24）。

| 形 | 格式字节 | 改第一个事务的字节 | 回退时多读 | 每次挂载多读 | 发号路径 |
|---|---|---|---|---|---|
| 丙 | 0 | 否 | 0 | 0 | 0 |
| 丙 + CJ | 0 | 否 | 要环里全部自证通过的记录的 `checkpoint_txg` = J 个记录头。`23-journal的角色与格式.md` 第 1206 行「回退 = 一次恢复」，恢复必须先全环扫描、逐条验证（同文件第 811 行）⇒ 这一遍本来就读，额外 0。若实现把回退写成不扫环（它不施加任何记录，P1 读法下也用不着记录），CJ 要补一遍：J × 4096 字节，读一份副本、另一份只在这一份读不出时读 | 0 | 0 |
| 丙 + CJ2 | 0 | 否 | 同 CJ | 普通挂载本来就全环扫描，额外 0；切换不重扫（内存里的 txg 已不小于这次挂载见过的一切） | 0 |
| G | 根记录 +8（371 → 379，512 槽余量 141 → 133）；记录头 +8（307 → 315），新根段 188 → 196；一条 4096 记录装 ⌊(4096 − 315) ÷ 56⌋ = 67 个点名项，与今天相同 | **是**，见下 | 0（根与记录头本来就读） | 0 | 0 |
| CK / CR（我的线索，已撤回，见 2.6） | 0 | 否 | 0 | 0 | 0 |

**G 改第一个事务的哪些字节**（按 `first-txn-layout.md` 现查）：

1. 七、发布（第 346–365 行）：根记录多一个 8 字节字段。第一个事务写出的根一共 6 份——mkfs 在三个区域槽 0 各种一份第 0 代根，暖机 txg 1、2 两份，第一个事务 txg 3 一份；每份的偏移表从新字段起往后移 8，`format-const: ROOT_RECORD_BYTES = 371`（`22-单元原子性怎么合成.md` 第 501 行）变 379，自证校验和的值随之全变。
2. 六、journal（第 300–342 行）：w1、w4、t9 三条记录的头各多 8 字节（第 342 行「合计 307 字节」变 315，`format-const: JOURNAL_HEADER_BYTES = 307` 在 `23-journal的角色与格式.md` 第 270 行），新根段那一行（第 311、338 行）从 188 变 196；「施加一条记录 = 把所选根的这四个字段换成记录里的」（同文件第 694 行）变成五个字段；t9 的反向链与三条记录的 `header_csum` 全变。
3. **一格没有定义的值**：mkfs 的第 0 代根、暖机两次的根和 w1 / w4 两条记录写出时还没有任何可写头（`first-txn-layout.md` 第 358 行那一格同类的树 ID 水位写的是 mkfs 11），G 写 0 还是 1（0 是无效号）全仓没有一句；t9 那一份是 2（第一个事务用了 inode 1）。
4. 要配一条与 I-7.8 同形的不变量（`invariants.md` 第 54 行：根环里全部根记录的「树 ID 水位」取 max，必须大于池中出现过的最大树 ID，checker 全盘扫描算「出现过的」）：G 取 max 必须大于池中出现过的最大 inode 号——而「出现过的」要扫码 3 类型 2 容器里的记录与每个数据单元五元组的对象 ID。

**跳号本身的代价**（推理，没建模）：凡是把第一个新根往后推的形态都要付——CJ / CJ2 多跳过被抛弃时间线里读得出、而根读不出或从没写成的那几个 txg（最常见的是回退前在飞那次发布的记录，比今天多跳 1 格）；我撤回的 CK / CR 每次跳 17–24 格（`c143r3_model.out` 第 112、114 行的 `jump_mean`）。第二轮攻方腿第四节那一笔「K 代点删在 txg 跳号时留下删不掉的行」（`05-快照-空间记账机制.md` 第 284 行附近「当前代 − K」的点删）会被放大：跳 Δ 格留下至多 Δ − 1 代的行（每代每个在维护的统计量一行，第一个事务 15 行 × 34 字节）。任何跳号类的形态（包括 CJ / CJ2）要不漏行，点删要改成「删掉 ≤ 当前代 − K 的全部代」（每次多删的代数有界于跳距）——这是 D5（快照 / 空间记账机制） 已定项 2 的一句改动；没有它，每跳一次就永久留下至多 min(跳距, K) − 1 代的行。

## 五、U4：各形配什么会红的检查，判别力自证

四形都是跨挂载的性质，单镜像 checker 看不见（被抛弃的对象按实例表判未发布、不在任何可达的树里），都要多次挂载的录制流——与 C331（择根倒挂压过已确认的写）、C332（回退实例两个根都读不出时回退被撤销） 同一个前置。判定都一样：回退或挂载之后建的每个对象，身份（按各自判据）与此前任何一个写成了根的对象都不同。

| 形 | 注入 | 必须红 / 必须绿 | 判别力自证 |
|---|---|---|---|
| 丙 | 回退那次扫描藏住 R_old 之后最新的 3 个根（`_slot_only` 口径 3 个槽；第二轮口径是整块盘 0 + 盘 1 上那一个根槽），被抛弃时间线最新那次发布建过对象 | 今天的丙必须红（它的最少故障就是这一形，第二节） | 把注入减到 2 个槽必须转绿——这一格验的是「严格暖机的两格间隙」确实在（第二节 2.1 的引理）；把暖机改成宽读法（用户对象紧跟回退根），2 个槽必须转红 |
| 丙 + CJ | 同上，再加那几个 txg 的记录两份副本读不出（第二节 f1 的 5 / 3 那一段） | 必须红 | 去掉记录那一半必须转绿，而同一注入下丙（不带 CJ）仍红——证明 CJ 真的在读记录；把 CJ 换回「可读根最大 + 1」，只留根那一半必须转红 |
| 丙 + CJ2 | 回退之后崩溃，重挂时同一类注入（第二轮 H13） | 必须红 | 同上，另加：把普通挂载的新实例 txg 改回「从重放之后那一根往上数」，只注入根那一半必须转红（第一轮 H5 的形状） |
| CJ / CJ2 的载体不退回 | 录制流上不注入读故障，每次扫描记下「环里读得出的最大 checkpoint_txg」 | 这个量在无读故障的扫描之间单调不减 | 造一个从更旧的前缀末开始写、且在读取最大值之前就覆写了最新记录的写者，必须红 |
| G | 第二节 f1 的 `p(1) rb(1,d1+j1,0) u(2)`（第二轮口径 2 个故障；`_slot_only` 是 `r1+j1`，3 个） | **必须红**：它就是 G 的最少故障历史，今天就打得中 | 自证分得清「G 关掉了头销毁」与「G 什么都关得掉」：第二轮攻方腿 H9 那一段（盘 0 + 一条记录的一份副本）必须绿，这一段必须红 |

写不出判别力自证的：没有。但丙那一行的自证有一个前提——暖机次数是格式常量（`22-单元原子性怎么合成.md` 第 1014 行），归属一改（加盘或开条带表时）间隙跟着变，那条检查的注入量要按新几何重算（第二轮 s8）。

## 六、条款：各形要改哪几句 kb 正文

行号全是 kb 文件自己的行号（`grep -n` 现查）。只列承重的句子；每一处改成什么由写回的人定，这里写「要动什么」。

**丙（任何一种组合都要动的）**

| 文件:行 | 现在写的 | 要动什么 |
|---|---|---|
| `08-核心索引结构.md:378` | 「**已发布的 inode 号单调不复用**」及后面的依据（`deleted_inodes` 回收窗口里 extent key 三段逐字相同） | 改成「已发布的 (inode 号, 对象出生代) 不复用」；依据句要改：回收窗口那一格在丙之下靠「`deleted_inodes` 与 extent 树都随根回退」挡，这是一条新前提，要写出来（第三节 C118 那一行） |
| `08-核心索引结构.md:74`、`:334` | 索引行与「一句话」里的「inode 号单调不复用」 | 同上 |
| `08-核心索引结构.md:369`、`:372` | 容器身份唯一性靠「号在树内不复用 ⇒ 同一棵树里两个容器不会同号」 | 丙之后同号的容器可以有两个，唯一性改由容器出生代那一段承担；分裂右半的出生代 = 那次发布的 txg，这一句要升成承重的依据 |
| `05-快照-空间记账机制.md:370` | 第 12 项那段 C143 的注 | 改成丙的新前提与它的依据（回退之后新对象的出生代严格大于每个已发布的出生代——这一句只在「回退那一刻最新的根读得出」时成立，第二节把它的余量写死） |
| `09-加密.md:536` | 对象出生代「退化为深度防御：若实现 bug 意外复用了号……」 | 丙之后它不再是深度防御，是唯一性本身；「若实现 bug 意外复用了号」改成「号在回退之后按设计会被重发」 |
| `18-块里携带什么信息.md:749` | 墓碑里的出生代「理由自 2026-09-05 起是『与五元组同口径、扫描时逐字段对照』」 | 丙之后它承重（分辨同号两个已发布对象的墓碑） |
| `invariants.md:262` | I-9.6「> 该树全部墓碑记录的对象 ID」 | 写明「已发布的（按实例表判）墓碑」；否则 checker 按扫描读在丙的 0 故障回退之后判红 |
| `checks-owed.md:154` | C152 只造「未发布的号被重发」 | 加一格「已发布的号在回退后被重发、出生代不同」 |
| `checks-owed.md:146` | C143 那一行 | 按定案改写或还清 |

**丙 + CJ 另动**

| 文件:行 | 现在写的 | 要动什么 |
|---|---|---|
| `23-journal的角色与格式.md:1206` 与索引行 `:691` | 「第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1」 | 改成「max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」；同时写明回退也做全环扫描（否则 CJ 要多读一遍环，第四节） |

**丙 + CJ2 另动**：同上一句，再在 `23-journal的角色与格式.md:1235`（「计数器全池接着走」那一条）旁加一句「新实例（普通挂载、切换、回退）的第一次发布 txg ≥ 同一个最大值 + 1」。它碰到 C331（择根倒挂压过已确认的写，`checks-owed.md:303`）那一行的修法候选「记录扫描水位」，要一起写。

**G 另动**（字节，见第四节）：`22-单元原子性怎么合成.md:490` 那张根记录字段表加一行、`:501`–`:502` 的 371 → 379；`23-journal的角色与格式.md:261` 头 307 → 315、`:270` 的 format-const、`:694` 新根段 188 → 196 与「这四个字段」；`first-txn-layout.md` 六（第 300–342 行）、七（第 346–365 行）；`invariants.md` 加一条与第 54 行 I-7.8 同形的不变量；第 12 项仍按头维护，判据仍按号——而第二节说明按号的判据在 2 个故障（`_slot_only` 3 个）就被打穿。

## 七、打中之后先问四句

编号接前两轮（第一轮 H1–H8、第二轮 H9–H14），这一轮从 H15 起。第四句是 `evidence-discipline.md` 那张表的第四种形态：跑前条款给的改法，在打中的那几格上还中不中。

| # | 打中 | 分辨臂吗 | 做决定那一刻看得到判别它的东西吗 | 满足哪条判据的哪个分句 | 跑前条款的改法在这几格上还中不中 |
|---|---|---|---|---|---|
| H15 | CJ / CJ2：落点被推到「只有记录、没有根的 txg + 1」，新对象的出生代正好是被抛弃对象的出生代（2.2、2.7；最少 6 / 4，含一次根槽写失败） | 分辨：同一段历史上丙在扫描空间里一次都不中（158 个前缀），或比 CJ 多要 1 个故障（6 个前缀） | 看不到：藏住的是 txg 7–9 的根与记录，CJ 看得到的只有 c6；丙在这段历史上不中不是因为它看到了什么，是因为它只看根、落点集合里没有 9 | U1「丙按 (号, 出生代)」（CJ / CJ2 沿用丙的判据）+ 这一轮点名的「CJ / CJ2 把第一个新根往后推之后，新对象的出生代会不会正好落在被抛弃对象的出生代上」；反向接受条款第二条的「同一类」按「同一段历史」读时满足第二个合取项，按「同一种攻击形状」读时不满足（2.2 的表） | 改法是「CJ / CJ2 出局，丙原样」。158 个前缀上丙原样不中 ⇒ 起作用；6 个前缀上丙原样仍中、只是多要 1 个故障 ⇒ 对那 6 格只是抬了门槛 |
| H16 | G：抬过全池号水位的最新那次发布，它的根与两份记录副本读不出（2.5；2 个故障，`_slot_only` 3 个） | 分辨：16 个格里 G 中而丙不中都不是 0 | 看不到：那个号只在读不出的根与记录里；丙不需要看到它——暖机那两格间隙让丙的落点避开 | U1「G 按号」；反向接受条款第三条「G 在 U1 被分辨臂的历史打中」 | 改法是「G 出局」；同一批格上丙、CJ 都不中 ⇒ 起作用 |
| H17 | 我的线索 CK：跳号让根环轮转覆写最新的根，之后 4 个根槽读不出就挂不上（2.6，f5 里 1219 条对丙有效、对 CK 无效） | 分辨：丙在同一段脚本上挂得上 | 不适用（是 CK 自己造出的状态） | 不在 U1 里：它伤的是挂载与回退候选集，按我自己的线索判它出局 | 我撤回 CK |
| H18 | 我的线索 CR / CR2：跳 24 在 txg 里留下空当，下一次回退或挂载藏住的 txg 超出「读得出的最大 + 24」（2.6；f12b 与丙逐条相同，f12d 里 CR2 中而丙不中 8 条） | CR 在 f12b 不分辨（与丙同中同不中）；CR2 在 f12d 分辨（比丙差） | 看不到：空当里的 txg 只有读不出的根带着 | U1「丙按 (号, 出生代)」（CR 沿用丙的判据） | 我撤回 CR / CR2 |

**不分辨臂、另记的**：树 ID 重发（第一轮 H6，f11 十条臂一样 155 条、最少 1 个故障）；故障计数的口径（第十节第 1 条）；丙在 2.0 那一形里的 2 / 3 个故障——那一形丙中的每一段甲 / 乙也中（`hit=C not_hit=A` 16 格全 0），CJ 把它抬到 5 / 3（9 / 5）。

**按跑前条款，我的判断**（交主 agent 评判，不是判决）：

- 丙：这一轮在 U1 与 U2 都没有被一段分辨臂的历史打中；跑前第一条反向接受条款不触发。
- 丙 + CJ / CJ2：H15 按「同一段历史」读触发第二条 ⇒ CJ / CJ2 出局、丙原样；按「同一种攻击形状」读不触发。两种读法都交上去。它要的故障（6 / 4 起，含一次根槽写失败；`_slot_only` 10 / 6 起）远多于丙在 2.0 那一形里的 2 / 3——交用户时「丙 + CJ 在多数历史上把门槛抬高、在一小类历史上把丙原来碰不到的点变成碰得到」要一起摆。
- G：H16 触发第三条 ⇒ G 出局。
- 我自己提的 CK / CR / CR2：被我自己的模型打穿，撤回。它们只在我的模型上量过，打穿它们的也只是我的模型——两件事都只算线索。

## 八、什么现象会推翻我的每一条结论

| 结论 | 推翻它的观测 |
|---|---|
| 1 丙的余量是「连续 3 个 txg 的根读不出」，0 / 1 / 0 下第二轮口径 2、`_slot_only` 3（2.0） | 任何一族里出现丙中而故障更少的脚本；或第一版有一条让用户事务进入新实例覆盖两块盘之前那几次发布的路径（宽读法，第二轮 2.7 已逐条查过）；或归属不再写死 0 / 1 / 0（`22-单元原子性怎么合成.md` 第 1013、1014 行自己写着「归属改了这两个常量要重算」） |
| 2 丙中的每一段甲 / 乙也中 | 一段 `hit=C not_hit=A` 不为 0 的历史；这一轮 16 个格都是 0 |
| 3 CJ 的落点集合严格比丙大，多出的是「只有记录、没有根的 txg + 1」（H15） | f1y 的 158 / 172 个丙一次都不中的前缀全含根槽写失败；在更大的前缀空间（前缀长于 5、或加上克隆、切换之外的动作）里出现一个不含写失败、丙一次都不中而 CJ 中的前缀，机制就是别的，要重找；或条款让写失败的那次发布的记录不进 CJ 的最大值（例如 CJ 只取「有同 txg 根」的记录——那是一条新臂，没被攻过）；或第一版的持久顺序改成根先于记录（`16-发布语义.md` 已定项 7 今天是记录先于根） |
| 4 反向接受条款第二条按「同一段历史」读触发、按「同一种攻击形状」读不触发 | 主 agent 或辩方腿给出第三种读法，或指出条款原意就是其中一种（背景材料第五节只写了「丙在同一类里」） |
| 5 G 的余量 = 最新那次抬过全池号水位的发布的根与两份记录副本（H16），G 出局 | 一条让全池号水位多一处不随那次发布一起读不出的持久载体的条款（例如也写进每块盘的超级块槽）——那是 G 的一个新形态，要重攻；或模型把「记录两份镜像、任一份在即在」（`23-journal的角色与格式.md` 第 691 行）建错了 |
| 6 U2：今天没有同时「只认号、不随根回退、有条款定义」的消费者 | C118 定成只记号、不随根回退的结构；出现对外句柄 / 挂载面的条款；或第三节表里某一行的「随根回退」被条款改掉 |
| 7 U3：CJ / CJ2 零额外读 | 实现把回退写成不扫环（那时 CJ 要补读一遍 J 个记录头）；或恢复不再要求全环扫描（`23-journal的角色与格式.md` 第 811 行） |
| 8 G 改第一个事务 6 份根与 3 条记录头的字节 | `first-txn-layout.md` 七 / 六的写清单变了（例如 mkfs 不再三区域各种一份第 0 代根） |
| 9 CK / CR / CR2 被打穿（H17、H18） | 根环槽位不再是 txg 的函数（例如按计数器轮转）；或跳号之后的空当能从某个持久量读回——那就不是这两条臂了 |
| 10 `_slot_only` 那套数是第一版更贴切的口径 | 第一版允许在一块盘读全失败、写却成功的时候做可写回退（`18-块里携带什么信息.md` 第 879 行的可写判定只看独占打开与可写设备数，不看读） |
| 11 CJ2 那 210 条无效脚本不是挂不上，是回退候选集变了 | 在那 210 条里找到一条挂载本身返回失败的（我只核了第一条，`c143r3_debug.out` d2 节） |

## 九、复跑命令、跑的经过、文件 sha256

复跑（只用 Python 3.12 标准库；输出只写 stdout；`-B` 不留 `__pycache__`；本机另有会话在跑测量，一律压到最低优先级）：

```
cd research/prompts/c143-r3-opus-model
nice -n 19 python3 -B c143r3_model.py > "${TMPDIR:-/tmp}/c143r3_model.rerun.out"
cmp "${TMPDIR:-/tmp}/c143r3_model.rerun.out" c143r3_model.out && echo IDENTICAL
nice -n 19 python3 -B c143r3_debug.py > "${TMPDIR:-/tmp}/c143r3_debug.rerun.out"
cmp "${TMPDIR:-/tmp}/c143r3_debug.rerun.out" c143r3_debug.out && echo IDENTICAL
```

模型是确定性的（没有随机源，输出按排序打印），复跑一次只说明没有隐藏状态。留存那一次在本机 `nice -n 19` 下 7 分 29 秒（同时有别的会话的进程在跑）；`c143r3_debug.py` 复跑一次，与留存逐字节相同。

跑的经过（源码是原地改的，旧版本没有另存；每次改了什么照实写）：

| 次 | 结果 | 改了什么 |
|---|---|---|
| 1 | 跑完（2 分 36 秒）。那时的臂到 CK / CK2 为止，没有 f1x；f5 里 CK / CK2 的 runs 比别的臂少 1219 | 查了第一条（今天是 `c143r3_debug.out` 的 d1 节）：跳号把较旧的根覆写了。加 CR / CR2 与「对这条臂无效、对丙有效」的计数；加 f1x（f1 的逐前缀比较只扫到 3 个根槽，丙在 131 个前缀上「没中」可能是被扫描上沿截的） |
| 2 | 一启动就 `NameError`：f1x 追加在 `if __name__ == "__main__"` 之后 | 把那两行挪到文件末尾 |
| 3 | 跑完（7 分 14 秒），2814 行 | — |
| 4（留存） | 加 f1y（只打印 f1x 的明细，印在输出最后），7 分 29 秒，2860 行；前 2813 行与第 3 次逐字节相同（两份各取 `head -n 2813` 再 `cmp`） | — |

开发中另有三个一次性的 `python3 -c` 片段（核 CK 为什么无效、CR2 为什么中、CJ2 为什么无效），内容后来整理成 `c143r3_debug.py`，d1–d5 节是它们的留存版。

文件 sha256 与行数（`sha256sum`、`wc -l` 原样输出）：

```
76fc64e1f5bc25dfd346ab15affe37843951d79b1d630d9eca3ccf2c3949a7e7  c143r2_base.py
b1a75e51b680b3713e27a700b791321736a9e5e2cd6dfb9762d727b242bc9d9c  c143r3_model.py
f8ae22052680c3f012def2ea842845efd5b3d198931353d58d3f7a2f4a5b41b2  c143r3_model.out
391ad436c8e3f56415ff7ce5e858fac81f0e2c7c1fb4c70189f5ad60a63566b8  c143r3_debug.py
3b3d9a275e4a0b7b2ca19287f4909edab93adf1c2dc776f6e0c86f601f82494e  c143r3_debug.out
76fc64e1f5bc25dfd346ab15affe37843951d79b1d630d9eca3ccf2c3949a7e7  ../c143-r2-opus-model/c143r2_model.py
   777 c143r2_base.py
   572 c143r3_model.py
  2860 c143r3_model.out
   100 c143r3_debug.py
   154 c143r3_debug.out
  4463 total
```

`c143r2_base.py` 与第二轮原件的 sha256 相同：拷贝之后一个字没改。

## 十、没建模的、射程、建议另记的账

**没建模的**（每条写它会不会改结论）：

- 丢失写（设备应答了 FUA 却没落盘，槽里还是旧的合法根）：模型里读不出的槽都「读得出来是坏的」。丢失写对丙与 CJ 的计数方式与「整块盘读不出」不同——槽看起来合法，k_bad 数不到它。它不改 2.0 的引理（仍要连续 3 个 txg 的根丢），只改「一个故障」的物理含义；我没把它单列一族。
- 根环一个槽持续读不出（C335 那一类）：模型里一次扫描一种故障，跨扫描按不同目标的并集计（第二轮口径）；持续坏槽在多次挂载之间被重复利用、只记一次，这一点偏向少算故障，对攻方有利。
- 切换的 reload 读法（第一轮 H2）没重跑；按第二轮辩方腿，那一格改记条款缺口。
- 一个可写头加至多一个克隆头；克隆只从头 12 克；S = 8；前缀至多 5 次发布；两块盘。
- K 代点删、F、准入、真实分配器都没建；第四节那一笔 txg 跳号留行是推理。

**射程**：各族的最少故障数是这些枚举空间里的最小值，不是定理；2.0 的引理是推理，模型在它覆盖的空间里没有反例。「丙中 ⇒ 甲中」「CJ 中 ⇒ AJ 中」是全部 sweep 里 0 例外的观测。`_slot_only` 那一套数只是把「扫描时整块盘读不出」换成逐槽计，不是第一版口径的定案——哪个口径是真的，取决于第一版允不允许在一块盘读全失败、写却成功的时候做回退（第一节）。

**建议另记的账**（这一轮打出来的、不分辨臂或不在 C143 格里的；立不立由主 agent 判）：

1. **故障计数的口径没有条款**：前三轮各族的「最少几个故障」都把「整块盘在回退扫描里读不出」算 1，而第一版 2 盘掉一块只能只读挂载（`18-块里携带什么信息.md` 第 879 行）。按逐槽计，丙 2 → 3、CJ 5 / 3 → 9 / 5、G 2 → 3、AJ 2 → 3，甲不变（1）。三轮的余量比较都建在第一种口径上。
2. **回退之后的新实例改写 txg 的收严，不能用「读不出的根槽数」来界**：我这一轮提的 CK / CR 两个形态都被我自己的模型打穿（第七节）——根环的槽位是 txg 的函数，跳号本身会让之后的发布覆写最新的根（CK），或者在 txg 序列里留下一段空当，让下一次回退的「读不出几个槽」界不住藏住的 txg（CR）。凡是想用「跳号」替代「读记录」的收严，都要先答这两问。
3. 第二轮攻方腿第四节那一笔（K 代点删在 txg 跳号时留下删不掉的行）照旧；CJ / CJ2 让跳号更大（最多到被抛弃时间线的长度），任何跳号类的收严都放大它。
4. 容器身份唯一性（`08-核心索引结构.md` 第 372 行）与 C152 的射程靠着「号不复用」写的推理链，丙之后要换依据（第三、六节）。

## 附录：本报告当论证支点的 kb 行（整行，`sed -n` 机械抽取，没有手抄）

**`18-块里携带什么信息.md:879`**

```text
四元组 (出生树 0, 类型 4, 容器号 = 片序号从 0 起, 出生代 0)；出生树 0 与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数；不属于任何树、不进容器索引，由根记录直接持有物理指针（D22（单元原子性怎么合成） 已定项 7）。一片 ⌊(32768 − 136) / 88⌋ = 370 条记录（含链指针，数据行 369；136 是含 nonce / MAC / 算法类型预留位的码 3 头，D18（块里携带什么信息） 已定项 14 / 已定项 16）；mkfs 写出一片空表，记录数 1（唯一一条是「无下一片」的链指针记录）。**行怎么写**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）：按实例代号唯一、后写覆盖，只在恢复、实例切换与回退时写，与第一个新根同一次发布——**每次可写挂载都写行**（实例 0 不写；写行那次发布是新实例的第一次发布，新实例的每一个根引用的实例表都已含 [max(所选根的实例, 1), 新实例) 的行；写行之前不推抬 F 的空发布，写行那次的元数据走切换预留，D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）：恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行——所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)，严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）——jsn = 实例代号 32 位 + 计数器 48 位（D23（journal 的角色与格式） 已定项 9），重放按 jsn 严格连续、断号即止 ⇒ 任何一次重放都跨不过实例边界 ⇒ **同一次恢复写出的那批行里，只有所选根那个实例的 W 可能非 0；同一次切换写出的那批行里，只有被照旧事务的写序主人的 W 可能非 0；其余恒 0**（限定在「同一次写出的那批」上：后来的恢复只给 [新的所选根实例, 新实例) 写行、不回头改旧行，所以表上允许有多行 W ≠ 0，它们来自不同次的恢复或切换）（E104（扫描重建的现行版本判定） `recovery_own_txns_global_w` 世界：把一个全局量写给范围里的每一个实例时，恢复实例自己的孤儿复活 3）——恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格，之后的记录接不上就一条都不施加、W = 0）；一条都没施加的写 (i, 所选根 txg, 0)；回退在 R_old 指着的那一版表上写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方），回退行在 kind 0 记录的 flags 字节里置 bit0 标「回退」、**不许被后来的恢复覆盖**（落在 R_old 上的那次恢复会给 [所选根的实例, 新实例) 每个实例写行，正好盖到它；C124（回退行与重放下界没有会红的检查）；⚠️ 表随根分版本，落在 R_old 上的恢复读的是 R_old 那一版表、回退行不在里面，这一句按条文没有输入，C332（回退实例两个根都读不出时回退被撤销））；实例切换（D23（journal 的角色与格式） 已定项 14 的注）同崩溃行，只是**所选根取被重发的那个在飞 checkpoint 所基于的根**——旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old；切换不重放，行只落在 [这个根的实例, 新实例)，每个实例一行、取下面第一条适用的：① 被重发的那次发布自己写的行原样重发（写行那次的恢复行、回退那次的回退行与中间实例行）；② 被照旧事务的写序实例 k 写 (k, k 最后发布的根的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)；③ 所选根那个实例写 (i, 这个根的 txg, 0)；④ 其余写 (i, 0, 0)——被重发的 checkpoint 里没有事务号非 0 的记录时 ② 一条都不适用（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；每次写行 COW 重写整条链。W 是精确的前缀：事务号在实例内单调、记录按事务号顺序追加（第一版串行提交下恒成立），一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加（D23（journal 的角色与格式） 已定项 7 的注）。**已发布谓词（全局）**：i_now = 挂载根的实例，单元写序 (i, n)、诞生代号 b——i < i_now 且无行 ⇒ 已发布；有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W（在所选根里，或被重放施加；E104（扫描重建的现行版本判定） `lost_root` 世界：T_pub 按字面取在飞 txg − 1 时槽坏掉的那个 checkpoint 整个判成已发布，复活 2 + 错 1），码 2 / 3 ⇒ b ≤ T_pub（重放之后那个根之后的固定点单元一律未发布，恢复实例重写它们；重放施加进来的那几次发布的固定点单元由记录里的新根段引用（D23（journal 的角色与格式） 已定项 15），按 T_pub = 重放之后那个根的 txg 判已发布（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；那 6 字节事务号在码 2 / 3 上不参与任何判定，I-1.8（归并后版本全序） 对码 3 的全序键是 (诞生代号, 实例代号)）；i == i_now ⇒ b ≤ 挂载根的 txg；i > i_now ⇒ 判损坏。「无行 ⇒ 已发布」只对同一时间线成立。**回收**：行可删当且仅当整轮清扫对**池中每一个落点**都得出了判定（读成功并按行判、已抹头、已被覆盖）且其中没有该实例的未发布单元、那次清扫里没有任何一次读失败（读不到不等于不存在，扇区维），且全部设备在线（设备维），且根环里没有该实例发布的根——根环每个槽都读成功才算「没有」，读不出的槽按「可能有该实例的根」算（2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方：一个暂时读不出的根槽能让行被删，槽恢复后被抛弃的根回到回退候选集，而它引用的单元已被清扫抹头；代价是一个根槽持续读不出时行永远删不掉，C335（根槽持续读不出时实例表只增不减））。量词取「池中每一个」不取「该实例写过的每一个」：已抹头与已被覆盖的落点说不出它原来属于谁，按后者写的话要枚举的集合与要销毁的信息是同一份（被抛弃的根留在环里时，它的行是回退候选判据的输入，E104（扫描重建的现行版本判定）：不看根环则被抛弃的根整批回到候选集）；zoned 第一版不回收、链长无上界（C121（zoned 上实例表链长无上界））。**表不可读时**：只读挂载；扫描重建停在级 1——表没了就一条行都没有，全部旧实例的全部单元落进「无行 ⇒ 已发布」，被回退抛弃的整段时间线也在内，所以不是记歧义。实例表单元自己不走 I-1.2（块头写序已发布） 的谓词：它的现行版本由根记录持有的物理指针唯一确定，指针指不到或校验不过就是表不可读。**作废一个 checkpoint 必须换实例代号**：当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布——这是失败表能封闭的依据（D23（journal 的角色与格式） 已定项 14 的注）。**实例代号**住超级块（E100（超级块的三段几何） 段四那 4 字节）。可写挂载的顺序：先判这次能不能可写（可写设备数够 w 的下限 ∧ **独占打开池中过半的设备**——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是本项 ∧ 实例表可读 ∧ 实例切换的预留拿得到），再取新代号 = max(**这次挂载独占打开成功的那个集合**里各超级块的代号（每块盘两槽里全部自证过的槽——校验和过且 fsid 与本池相同；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份超级块、**全或无**（写超级块与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**（旧代号 = 取号之前这个集合里全部自证过的槽中最大的实例代号；回卷写发生在任何单元之前），回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」由 I-7.7（超级块实例代号不低于根环） 的第 ② 句判成立（较大的号没有任何根、记录、单元带着），不另开例外，下一次挂载按 max + 1 修复；取号之后那道屏障（D23（journal 的角色与格式） 已定项 16）在任一块盘上报错同样判全或无失败，不许只重发屏障就继续；取号的写入集合与 max 的取值集合取同一个——写「全部可见」而只独占一部分时，集合外那些盘写不写得进去不由这个挂载说了算，两个挂载能互相逼成只读。**过半是准入门槛，不是写入范围**：写入集合取独占打开成功的全部设备，只写过半会把集合外的健康盘落下、它们的代号从此是旧的，与真正错过取号的盘分不开。**「可见」= 独占打开成功且超级块读得通**；I-7.7（超级块实例代号不低于根环） 的第 ② 句限定到同一个集合），之后才动任何单元；只读挂载不取号；从 1 起、0 无效。**前提**：同一时刻只有一个可写挂载——由过半独占打开保证；多主机共享存储上两个可写挂载会算出同一个代号、谓词分不出它们，随 D9（加密） 已定项 8 那一类宣布不防。取号那一刻两路见证不完整可见时不额外拦截，与整卷回滚同格；I-7.6（根环区域落在互不相同的盘上） 只在 devs ≥ R 时让根环成为独立见证，第一版 (R = 3, devs = 2) 不成立。错过取号的盘分两类：**打开过又掉了、且缺号期间池里确实发布过**的，回归时整盘作废、只读到重同步完成；**从未被这次挂载独占打开的**，或缺号期间池里一次发布都没有的，不作废——它没有分叉的物理可能（第八轮反推腿 3.1 / 7.2）；第一版 2 盘、掉一块只能只读挂载 ⇒ 分叉盘回归不可达；D2（RAID 条带策略） 已定项 3 / 4 允许在线加盘 ⇒ 第三块盘一加就可达，C120（分叉盘回归的判定与重同步）在允许加第三块盘之前到期，与过半规则同时。
```

**`22-单元原子性怎么合成.md:229`**

```text
| 8 | **固定结构的放置** | **已定（2026-09-02，用户定案 + E87（固定结构的放置））：超级块每盘一份（更新走 ≥2 槽轮换）；journal 环两盘镜像（D2（RAID 条带策略）已定项 6 的 w≥2 下限对 journal 记录写同样生效）；mkfs 把第 0 代根种进全部根环区域。** 正文见 D22（单元原子性怎么合成）「已定项 8」 **状态：已定。** |
```

**`16-发布语义.md:207`**

```text
**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布，第一版几何至多 R = 3 次。前置：C314（回退可以复用被抛弃的根引用的单元） 先还。
```

**`16-发布语义.md:372`**

```text
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算（C282（环里最旧根没有定义） 要的定义） |
```

**`22-单元原子性怎么合成.md:1013`**

```text
4. **根环三个区域的设备归属第一版写死 0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域把设备身份写进超级块的「根环逐区域设备身份」12 字节（D2（RAID 条带策略） 已定项 7）。⚠️ **它不是 mkfs 参数**（2026-09-14 用户定案）：第一版 devs = 2、R = 3，按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界（取满 min(R, devs) = 2 个不同值、没有一块盘背超过 ⌈3/2⌉ = 2 个区域），只有「两个区域在一块盘、一个在另一块」这一种形状；再按 E87（固定结构的放置） 逐字「两盘上 R = 3 必有一盘背两个区域」，区域编号一定之后 0 / 1 / 0 是唯一的规范写法。把它做成 mkfs 参数，买到的是「哪块盘背两个」这一格自由度，而那一格在两盘同构下没有可观测差别，代价是多一个要在挂载时校验、要在加盘时重算的参数。
```

**`22-单元原子性怎么合成.md:1014`**

```text
5. **暖机空发布 2 次、第一个事务 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、FIRST_TRANSACTION_TXG = 3，登记在 [first-txn-layout.md](../first-txn-layout.md) 八）。依据：D16（发布语义） 已定项 8 要求「推到本实例的根覆盖两块盘为止」，而在 0 / 1 / 0 的归属下 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定，写成常量比写成运行时谓词少一条只有一个取值的分支。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。
```

**`23-journal的角色与格式.md:1206`**

```text
**显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限；表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。
```

**`23-journal的角色与格式.md:1235`**

```text
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的「48 位撑 3202 年」本来就按全卷寿命算。
```

**`23-journal的角色与格式.md:811`**

```text
**必须先全环扫描、逐条验证，再择最长合法前缀。**
```

**`23-journal的角色与格式.md:694`**

```text
15. **崩在记录持久之后、根槽持久之前，恢复施加什么（2026-09-13，用户定案；两条指针 2026-09-14 随 D19（块指针的结构与宽度预算） 已定项 7 各加 3 字节）：由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节（实例表单元指针照所选根），头 95 → 277 → 307，4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。** D23（journal 的角色与格式） 正文里没有已定项 15 的单独小节，条款全文就是已定项索引表第 15 行。 **状态：已定。**
```

**`08-核心索引结构.md:378`**

```text
**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)。**已发布的 inode 号单调不复用**（D6（快照实现模型） 判据 9 的戒律，2026-09-06 随判据 9 一同收窄射程、用户定案）：
```

**`08-核心索引结构.md:386`**

```text
克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。
```

**`06-快照实现模型.md:240`**

```text
- 靠 D5（快照 / 空间记账机制） 同日补的两条：还有自己活快照的可写头不许销毁；分叉点闸的 `children` 数克隆。
```

**`09-加密.md:536`**

```text
| **对象出生代（inode generation）** | 按五元组 / AAD 的 day-1 契约保留（D18（块里携带什么信息） 已定项 3 / 已定项 11）。inode 号单调不复用（D8（核心索引结构） 已定项 6）之后，「复用后旧文件与新文件的 `(树, 对象, 偏移)` 完全相同」是历史理由；它退化为深度防御：若实现 bug 意外复用了号，这一段仍能在 AAD / nonce 两层把新旧对象分开，是 I-6.1（nonce 不重用） 失效时（崩溃窗口 / nonce 水位回滚）的第二道闸 | 密文侧 |
```

**`05-快照-空间记账机制.md:370`**

```text
**第 12 项（2026-09-05，随 D8（核心索引结构） 已定项 6 加）**：完备性口径照第 10 / 11 项的先例——它不在 D3（空间分配） 已定项 4 的准入不等式里，进来的理由是「分配判定要它」。⚠️ **2026-09-06 立账 C143（inode 号水位在回退后会退回去重发）**：它住记账，而 D23（journal 的角色与格式） 已定项 14 逐字「全部记账统计量的现行值从 R_old 那棵账重新载入」⇒ 回退后水位退回旧值、已发过的号被重发，撞 D8（核心索引结构） 已定项 6「inode 号单调不复用」。同一天给树 ID 水位定的「住超级块取 max」**搬不过来**——该项带树维、条数随可写头数长，而超级块是定长槽。
```

**`22-单元原子性怎么合成.md:490`**

```text
| **树 ID 水位** | **8** | D8（核心索引结构） 已定项 8 ②（2026-09-06 用户定案）：`max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，每次发布随根记录重写 |
```

**`invariants.md:54`**

```text
| I-7.8 | 根记录树 ID 水位不低于全池最大树 ID | **根环里全部根记录的「树 ID 水位」字段取 max，必须 > 该池中出现过的最大树 ID。**「出现过的」由 checker 全盘扫描算出：全部单元头的索引节点类身份段里那个树 ID（D18（块里携带什么信息） 已定项 7），并上树表条目里的树 ID；**不许与运行时共用同一段代码**（D13（验证路线） 已定项 5）。取 max 的范围是「根环里**全部**根记录」不是「回退候选集」——被抛弃时间线的根仍然承载它那一代发过的号，形态与 D23（journal 的角色与格式） 已定项 14 给 checkpoint_txg 取值那句同源。**判别力**：抓两类——① 某次发布没更新水位；② 水位被实现成「本 checkpoint 发出的最高号」而不是**累计高水位**（后者的镜像是连发 R + 1 个 checkpoint 各建一棵树，第一棵的号会随根轮出环而被重发）。⚠️ **只许写成不等式**：水位是「下一个可用号」的上界，写成等式会把正常镜像判红 | 已实现（2026-09-14，池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；层 0 每个崩溃状态都判） ⚠️ **checker 口径注（2026-09-13）**：「出现过的最大树 ID」要把树表里根指针为零的条目也数进去（livelist 6、稀疏旁表 7 day-1 只有树表条目、没有任何单元，D6（快照实现模型） 已定项 2 / D5（快照 / 空间记账机制） 已定项 6），只数单元头会漏。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：扫描方向只数诞生代号不超过根环里最大 checkpoint_txg 的码 2 节点——崩在发布之前的那个事务写出的孤儿节点不算「出现过」；按字面数它们，层 0 全量 262165 个状态里 261891 个判红（水位 11 而盘上已有树 ID 11–15 的孤儿节点），见 records 2026-09-13-总审核 十一·七 |
```

**`invariants.md:262`**

```text
| I-9.6 | 水位大于两处最大号 | 每个可写头的「inode 号水位」统计量 > 该树 inode 树内最大 key，> 该树全部墓碑记录的对象 ID；checker 遍历侧算出的最大值与记账里的水位是两条独立路径 | 未实现，前置：树的种类 |
```

**`checks-owed.md:146`**

```text
| C143 | inode 号水位在回退后会退回去重发 | **树 ID 水位那个洞对 inode 号水位逐字成立，而 2026-09-06 给树 ID 定的根记录臂同样搬不过去。**洞：inode 号水位是 D5（快照 / 空间记账机制） 已定项 4 第 12 项，住记账；D23（journal 的角色与格式） 已定项 14 逐字「全部记账统计量的现行值从 R_old 那棵账重新载入」⇒ 回退之后水位退回旧值 ⇒ 已经发过的 inode 号被重发，撞 D8（核心索引结构） 已定项 6「inode 号单调不复用」。⚠️ **2026-09-06 用新射程回扫过，这条不消失**：同日把 D6（快照实现模型） 判据 9 收窄成「**已发布的**快照标识单调不回收」，而**回退抛弃的是已经发布过的时间线**（那些 checkpoint 的根写成了才谈得上回退到它之前）⇒ 被重发的 inode 号出现在**已发布**的单元里，正落在新射程之内。⚠️ **另一半是开着的问题**：那么 D8（核心索引结构） 已定项 6「inode 号单调不复用」要不要按同一条理由句也收窄成「已发布的」？崩溃丢弃（不是回退）留下的孤儿单元里的 inode 号，按 D6（快照实现模型） 判据 9 的新射程是可以重发的。**这是用户定案要答的，今天没答。**搬不过去的理由：该项**带树维**（逐字「**带**（只为可写头维护）」）⇒ 条数随可写头数长，而根记录与超级块都是定长槽（E100（超级块的三段几何） 出路臂的全部价值就是让它「与设备数无关的常数」，走间接层后恒 420 字节） | 造一个「发出 inode 号 N、崩在发布前、回退到 R_old、再发一个 inode」的镜像，断言新发的号 **≠ N** 且严格大于回退前发过的任何号。判别力自证：把水位改成从记账重载，检查必须由绿转红 | 载体定案（要用户定案）：候选是树表条目里每头一个字段（与 2026-09-06 刚定的诞生 txg / `previous_snapshot_txg` 同处），或让记账里的单调量在回退时豁免重载（改 D23（journal 的角色与格式） 已定项 14）；两条都是格式级 | 2026-09-06 D8（核心索引结构） 已定项 8 定树 ID 水位时逐字核 E100（超级块的三段几何） 段四与 D5（快照 / 空间记账机制） 已定项 4 第 12 项的带树维列，坐实方案不可移植 |
```

**`checks-owed.md:303`**

```text
| C331 | 择根倒挂压过已确认的写 | 较旧实例的全部根暂时读不出时，新实例的第一次发布从更旧的根往上数 txg；那些根以后又读得出，txg 更高的它们被择新选中（D22（单元原子性怎么合成） 第 487 行按 checkpoint_txg 择新、平局才看实例代号），新实例已确认的写被压过去（第一轮攻方腿 6 个故障；C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 的写回管的是谓词，挡不住择根）。D23（journal 的角色与格式） 已定项 14 注 3「新实例从前缀末 + 1 接着写」还会让新实例的第一条记录落在读不出那条根的记录槽上，把它唯一的记录见证盖掉（今天的规则不拿那条记录当见证，所以今天看不出后果） | 多次挂载的崩溃点重放里注入「较新实例的全部根暂时读不出、之后又读得出」，逐状态判：已确认返回的写，在之后每一次挂载选的根上都读得回；判别力自证：今天的择新规则在这个注入下必须红 | 修法（计数器从全环最大 + 1 起、超级块带 txg、或记录扫描水位）要另过三方；多次挂载的录制流 | 2026-09-14 C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 第一轮攻方腿（`research/prompts/c329-c330-r1-main-verification.md` 第 47 行）、第二轮攻方腿（`c329-c330-r2-main-verification.md` 第 44 行）、第三轮辩方腿复核第二轮出局判决的那一格 |
```

**`08-核心索引结构.md:369`**

```text
**容器身份不需要分配器**：inode 号单调不复用 ⇒ 插入永远落在最右的叶 ⇒ 分裂只发生在最右叶，**分裂点取末尾**——
```

**`08-核心索引结构.md:372`**

```text
**唯一性的射程是一条已发布历史内**：每个容器号都是某条记录首次插入时的 inode 号，号在树内不复用 ⇒ 同一棵树里两个容器不会同号，同一次发布内「分裂 → 合并 → 再分裂」也造不出同身份两版。
```

**`05-快照-空间记账机制.md:284`**

```text
**丢弃满足 `.claude/rules/fs-design.md` 第一格**：要丢的那一代**算得出来**（当前代 − K），
```

**`18-块里携带什么信息.md:749`**

```text
**记录粒度**：对象整个删除写**一条区间记录**（对象 ID + 对象出生代 + 全区间；出生代是 2026-09-04 随已定项 11 补的；理由自 2026-09-05 起是「与五元组同口径、扫描时逐字段对照」——inode 号单调不复用（D8（核心索引结构） 已定项 6）之后「复用判别」是历史理由，字段按 day-1 契约保留）；truncate / punch 这类
```

**`22-单元原子性怎么合成.md:502`**

```text
合计 **371 字节**，512 槽内余量 141——槽宽是运行时探测值，余量给将来字段。
```

**`23-journal的角色与格式.md:261`**

```text
4. **记录头能不能被约束在单个原子单元内（2026-08-29）：能。头 307 字节（十个字段 78 + 已定项 7 / 8 / 13 三笔已定增量 17 + 已定项 15 新根段 188 + fsid 8 + MAC 16，2026-09-14）。** **状态：已定。**
```

**`checks-owed.md:154`**

```text
| C152 | 收窄后未发布号重发的扫描重建复核 | **2026-09-06 把 D6（快照实现模型） 判据 9 与 D8（核心索引结构） 已定项 6 的射程收窄成「已发布的标识单调不复用」，依据是「孤儿不在任何树里、扫描重建两道闸按写序择新」。而 D8（核心索引结构） 已定项 6 同一段的末尾写着「『未发布的等于没发生』只对常规挂载路径成立，**扫描重建看的是盘**」——两句挨得很近，方向看起来相反。**主 agent 判它们不是同一个命题**（那句管的是水位在一个代段里取最大值而不是后者胜），但这个判断本身没走三方，也没有实测。** | 造一个镜像：checkpoint N 发出 inode 号 M 与树 ID T、写出带它们的单元头、根没写成就崩；重挂后重发 M 与 T 并写出新对象；然后**只走扫描重建**（不读根）重建整个池。断言：重建结果里 M 与 T 各自只对应一个对象，且是新的那个。判别力自证：把单元头里的写序抹掉，重建必须由绿转红（写序是唯一区分孤儿与活版的字段） | 扫描重建实现 + checker；在那之前可以先做纯内存模型（形态照 E104（扫描重建的现行版本判定）） | 2026-09-06 收窄射程时主 agent 自己现查 D8（核心索引结构） 已定项 6 原文发现的残留疑点，不是攻击腿打的 |
```

**`checks-owed.md:304`**

```text
| C332 | 回退实例两个根都读不出时回退被撤销 | 管理员回退之后，回退实例的根在两块盘上都读不出（2 个故障）时，恢复选到被抛弃时间线上的根（择根不看实例表，回退行救不了），或落回 R_old 并施加 R_old 之后被抛弃的记录，回退被静默撤销、之后已确认的写丢掉；与暖机只防单故障同一类。D18（块里携带什么信息） 已定项 11「回退行不许被后来的恢复覆盖（落在 R_old 上的那次恢复会给 [所选根的实例, 新实例) 每个实例写行，正好盖到它）」与 D23（journal 的角色与格式） 已定项 14「重放下界遇回退行截断」按表随根分版本读都没有输入：落在 R_old 上的恢复读的是 R_old 那一版表，回退行不在里面 | 多次挂载的崩溃点重放里，回退之后注入「回退实例的根全部读不出」，逐状态判：挂载根不在被回退抛弃的时间线上；判别力自证：今天的择根在这个注入下必须红 | 择根要不要看实例表、回退要不要多一份持久见证，没有条款；多次挂载的录制流 | 2026-09-14 C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 第二轮攻方腿（`research/prompts/c329-c330-r2-main-verification.md` 第 46 行）、第三轮攻方腿第三节（`c329-c330-r3-opus-output.md`） |
```

**`18-块里携带什么信息.md:830`**

```text
| 53 | 容器号 | 8 | 分配规则按打包记录类型分（2026-09-05）：类型 1（墓碑）树内单调分配（C103（容器索引无落点） 那套，回退不破身份）；类型 2（inode 记录）= 触发分裂的那条新记录的 inode 号，不要分配器（D8（核心索引结构） 已定项 6）。两类都是发布无关，容器被 COW 重写时不变；4 字节按 E44（序号位宽的本机实测与代价） 的 2785 发布/秒 17 天回绕 |
```

**`first-txn-layout.md:311`**

```text
| 新根段：树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 | 188 | w1 / w4：树表 50178、映射 0、水位 11、F 0；t9：树表 50248、映射 50247、水位 19、F 0 | D23（journal 的角色与格式） 已定项 15 | 已定（2026-09-13 用户定案，两条指针 2026-09-14 各加 3） |
```

**`first-txn-layout.md:342`**

```text
合计 **307 字节**，4096 记录余 3789，装 67 个点名项。
```

**`first-txn-layout.md:358`**

```text
| 树 ID 水位 | 8 | mkfs 11，发布后 **19**（deadlist 树 ID 18 day-1 注册，D5（快照 / 空间记账机制） 已定项 11） | D8（核心索引结构） 已定项 8 ② / 已定项 11；D22（单元原子性怎么合成） 已定项 7 | 已定 |
```


## 补记：交付规则上的偏差

- 模型源码 `c143r3_model.py` 的第一段（第 1–121 行）用一次写入新建，之后五段用追加（每段 58–105 行）。后来四处修补（加 CR / CR2 与「对这条臂无效」的计数、把 f1x 接进入口、把 `if __name__ == "__main__"` 那两行挪到末尾、加 f1y）用 Python 做定点替换：每处先断言旧串恰好命中一次，再回写并回读确认新串在。每处新增的内容都在 40 行以内，但回写的是整份文件——与第二轮报告补记记的是同一种偏差。
- `c143r2_base.py` 是用 `cp` 拷进来的，不是我写的；`c143r3_debug.py`（100 行）一次写入新建。
- 报告第一段用 `set -o noclobber` 排他新建，之后每次追加都在 150 行以内（最多的一次 118 行）。
