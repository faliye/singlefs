# C143（inode 号水位在回退后会退回去重发） 第一轮：攻方腿（Opus）报告

立场：反推 / 攻。假设主 agent 倾向的丙（唯一性收窄到 (inode 号, 对象出生代)）是错的，也假设甲（水位进树表条目）、乙（回退不重载单调量）各有死角，去造可达历史打穿它们。

读过的材料：`research/prompts/c143-r1-opus.md`（任务书）、`research/prompts/_c143-r1-background.md` 全文（正文、小节清单、附录）。附录不够的地方现查了 kb 原文，下文引 kb 一律写 kb 文件名加那份文件自己的行号（现查 kb 文件得来，不从背景材料里数）。没有读 `research/prompts/c143-r1-*-output.md`，也没有改仓里任何别的文件；模型只写在 `research/prompts/c143-r1-opus-model/` 下。

模型：`research/prompts/c143-r1-opus-model/c143_model.py`（只用 Python 3.12 标准库），原始输出 `research/prompts/c143-r1-opus-model/c143_model.out`。复跑命令与 sha256 在第十节。

## 〇、先说一个跑前没列、这一轮攻到的量

跑前的判据表把丙的关键前提写成「回退后第一个新根的 txg = 根环最大 + 1，大于被抛弃时间线的每一个 txg」。攻这一句时，要比较的不是「丙会不会中」，是**各条臂分别要几个故障才中**，因为三条臂都靠同一个载体：「根环里可读根的最大值」。甲、乙拿它取水位最大值，丙拿它取 txg 最大值。载体读漏了（最新的根暂时读不出），三条臂一起受影响；区别在于漏读之后各自还有多少余量。所以下文 U1 那一节对每条臂报「最少故障数 + 最短历史」，并统计「丙中而甲不中」的历史有几条。

## 一、模型实现了什么、照的是哪一行

下表只写**模型里怎么做**和**照的是 kb 哪一行**，不转述条款；被当成论证支点的那几行，整行原文在报告末尾的附录（机械抽取，没有手抄）。

| 模型里怎么做 | 照的行 |
|---|---|
| 根环 3 个区域，槽位 `(txg mod 3, (txg div 3) mod S)`；区域 0、2 在盘 0，区域 1 在盘 1 | `22-单元原子性怎么合成.md:249`、`:256`、`:1005` |
| S = 4（下界）；第三节另跑 S = 5 | `22-单元原子性怎么合成.md:257` |
| mkfs 在三个区域的槽 0 各放一份 txg 0 的根 | `22-单元原子性怎么合成.md:229` |
| 每次发布先追加一条记录（记录持久），再写根槽；「已发布」= 根槽写成；「已确认」= 根槽写成、且那一刻本实例写成的根已覆盖两块盘 | `16-发布语义.md:287`、`:207` |
| 暖机两种读法：**严格**（新实例——挂载、切换、回退——先推空发布直到本实例的根覆盖两块盘，用户事务排在后面）、**宽**（不推空发布，用户事务可以紧跟新实例的第一个根，只是 fsync 不返回）。第一次挂载两种读法都先推 2 次（txg 1、2），第一个事务 txg 3 建 inode 1、水位 2 | `16-发布语义.md:207`、`:208`；`05-快照-空间记账机制.md:486` |
| 每个根带着每个可写头的完整状态（水位与对象），空发布也一样 | `16-发布语义.md:192`、`05-快照-空间记账机制.md:371` |
| 根槽写失败：换新实例，txg 推进一格重发；失败的那个槽保留原来的根 | `23-journal的角色与格式.md:691`、`16-发布语义.md:372` |
| 普通恢复：选 (txg, 实例) 最大的可读根；只重放同一实例、txg 连续的记录；新实例代号 = max(超级块, 根环) + 1；新实例的 txg 从重放之后那一根 + 1 起 | `18-块里携带什么信息.md:879`、`23-journal的角色与格式.md:1233`、`checks-owed.md:303` |
| 管理员回退：候选集按最新可读根指着的那一版实例表判；第一个新根 txg = 可读根的最大 txg + 1；水位按臂取；写回退行与中间实例行 | `23-journal的角色与格式.md:1206` |
| 树 ID 水位回退时取可读根的最大值（今天的规则，三条臂一样） | `08-核心索引结构.md:476` |
| 克隆：新头的树 ID 取树 ID 水位，新头的水位抄 origin 的运行时水位 | `08-核心索引结构.md:386` |

**臂**（三条臂只在「回退时 inode 号水位怎么取」这一处不同）：

- **甲 / 乙**：水位 = max(R_old 那一格, 每个可读根里该头那一格)。模型里甲与乙是**同一个计算**：两者都对同一个「可读根集合」取同一格的最大值，只是一个读树表条目、一个读记账树的行；那两样在每个根里都随每次发布重写，某个头在某个根里「有没有这一格」的条件也一样。所以模型只跑 A，U1 的结论对乙原样成立。甲乙的差别在 U3 / U4，以及乙把第 13、14 项一起捆进来那一格（第四节）。
- **丙**：水位从 R_old 重载，判据按 (树, 号, 出生代)。
- **攻方自己给的两个收严**（只在这个模型上量过，被攻过零轮）：**AJ** = 甲 / 乙再把 journal 里 txg > R_old 的每条记录新根段指到的那一版水位也取进 max；**CJ** = 丙把第一个新根的 txg 取 max(可读根的 txg, journal 里全部记录的 txg) + 1。
- **回退那次发布里切换**，甲 / 乙有两种读法：**keep**（在飞 checkpoint 原样重发，回退时算出的最大值留着）与 **reload**（切换是「挂载内做一次恢复」、所选根是 R_old，水位按 R_old 盘上那一版重载）。丙在两种读法下行为相同。

**故障计数**：一次根槽写失败 1；一个根槽读不出 1；最新两个根槽读不出 2；一整块盘在这一次挂载的读扫描里暂时读不出、写仍然写得进 1。崩溃不算故障。

**U1 的判定**：一个身份（甲 / 乙按 (树, 号)，丙按 (树, 号, 出生代)）先后被两个不同对象带进写成了的根里，且后一个对象是在某次回退之后建的，就记一次违例；同时记前一个对象当时是否已确认（fsync 返回过）。

**没建模的**：journal 环的覆写（记录一直在）、F 与准入、影子账与单元复用、事务号与 W（根的有效性只按 T_pub 判）、只读挂载。扫描重建在第三节另有一个小模型。这些没建模的东西会不会改结论，第八节逐条写。

## 二、U1：各条臂最短的「已发布身份被重发」历史

脚本记号：`p0` / `p1` = 一次发布，建 0 / 1 个对象；`f0` / `f1` = 同上，但根槽写失败（切换、txg 推进一格重发）；`cr1` = 建 1 个对象的发布，记录持久之后崩溃；`rb{k}{u}{sw}` = 管理员回退到第 k 新的有效候选，u 是这一次挂载读扫描里的读故障（`none`；`new1`–`new3` = 最新 1–3 个根槽读不出；`d0` / `d1` = 盘 0 / 盘 1 整块这一次读不出；`d0n1` / `d1n1` = 整块盘读不出，再加另一块盘上最新的一个根槽读不出），sw = 1 表示回退那次发布的根槽写失败；`u2` = 两次用户发布，各建 1 个对象（严格读法下排在暖机之后）。每条脚本前面都有第一次挂载与第一个事务（txg 1–3），不写出来。

### 2.1 总表

sweep1 的枚举：前缀至多 6 次发布（每次从 `p0 p1 f0 f1` 里选一个）× 有无 `cr1` × k ∈ {1, 2, 3} × 8 种读故障 × sw ∈ {0, 1}，每条有效脚本对每条臂各跑一遍。`c143_model.out` 第 6–15 行（整行抄）：

```
name=sweep1 warm=lenient arm=A runs=455216 violating=227596 min_faults=1 by_faults=0:0/754;1:692/6560;2:5442/26322;3:19784/62744;4:41324/97908;5:56168/107856;6:53756/85472;7:33938/46672;8:12940/16768;9:3216/3776;10:336/384
name=sweep1 warm=lenient arm=A/reload runs=227608 violating=164650 min_faults=1 by_faults=1:312/754;2:2860/5806;3:11752/20516;4:27244/42228;5:39354/55680;6:40152/52176;7:27936/33296;8:11590/13376;9:3094/3392;10:356/384
name=sweep1 warm=lenient arm=AJ runs=455216 violating=0 min_faults=None by_faults=0:0/754;1:0/6560;2:0/26322;3:0/62744;4:0/97908;5:0/107856;6:0/85472;7:0/46672;8:0/16768;9:0/3776;10:0/384
name=sweep1 warm=lenient arm=C runs=455216 violating=32746 min_faults=1 by_faults=0:0/754;1:40/6560;2:320/26322;3:1958/62744;4:5020/97908;5:7292/107856;6:8748/85472;7:6144/46672;8:2516/16768;9:652/3776;10:56/384
name=sweep1 warm=lenient arm=CJ runs=455216 violating=0 min_faults=None by_faults=0:0/754;1:0/6560;2:0/26322;3:0/62744;4:0/97908;5:0/107856;6:0/85472;7:0/46672;8:0/16768;9:0/3776;10:0/384
name=sweep1 warm=strict arm=A runs=506248 violating=178112 min_faults=1 by_faults=0:0/754;1:692/6564;2:4832/26790;3:16376/66212;4:34036/108820;5:45196/123276;6:39964/97272;7:24584/53040;8:10032/19104;9:2208/4032;10:192/384
name=sweep1 warm=strict arm=A/reload runs=253124 violating=147254 min_faults=1 by_faults=1:312/754;2:2594/5810;3:10310/20980;4:24138/45232;5:36402/63588;6:36110/59688;7:24052/37584;8:10536/15456;9:2544/3648;10:256/384
name=sweep1 warm=strict arm=AJ runs=506248 violating=0 min_faults=None by_faults=0:0/754;1:0/6564;2:0/26790;3:0/66212;4:0/108820;5:0/123276;6:0/97272;7:0/53040;8:0/19104;9:0/4032;10:0/384
name=sweep1 warm=strict arm=C runs=506248 violating=6502 min_faults=2 by_faults=0:0/754;1:0/6564;2:24/26790;3:184/66212;4:718/108820;5:1592/123276;6:2036/97272;7:1388/53040;8:480/19104;9:80/4032;10:0/384
name=sweep1 warm=strict arm=CJ runs=506248 violating=0 min_faults=None by_faults=0:0/754;1:0/6564;2:0/26790;3:0/66212;4:0/108820;5:0/123276;6:0/97272;7:0/53040;8:0/19104;9:0/4032;10:0/384
```

`by_faults=f:x/y` 读作「故障数为 f 的有效脚本 y 条，其中违例 x 条」。

| 臂 | 严格暖机：最少故障 | 宽暖机：最少故障 |
|---|---|---|
| 甲 / 乙（keep） | 1 | 1 |
| 甲 / 乙（reload，只在 sw = 1 的脚本上跑） | 1 | 1 |
| 丙 | **2** | 1 |
| AJ、CJ（攻方收严，只在这个模型上量过） | 没有违例 | 没有违例 |

跨臂比较（第 201 行，整行抄）：

```
name=C_hit_but_A_not count=0 first=[]
```

⇒ 两种暖机读法下的全部有效脚本里，**丙中了、甲（keep）没中的脚本一条都没有**。严格读法下 1、2 个故障的分布（第 149–156 行，整行抄）：

```
name=pattern warm=strict faults=1 arms=- scripts=5560
name=pattern warm=strict faults=1 arms=A scripts=692
name=pattern warm=strict faults=1 arms=A/reload scripts=312
name=pattern warm=strict faults=2 arms=- scripts=20056
name=pattern warm=strict faults=2 arms=A scripts=4116
name=pattern warm=strict faults=2 arms=A+A/reload scripts=692
name=pattern warm=strict faults=2 arms=A+C scripts=24
name=pattern warm=strict faults=2 arms=A/reload scripts=1902
```

### 2.2 甲 / 乙：一个根槽读不出就中

`c143_model.out` 第 50–60 行（整行抄）：

```
example warm=strict arm=A faults=1 script=[p1 rb1d10 u2]
  violations=[((12, 2), [2, 3], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 creates (12,2,bg4) wm=12:3
    ROLLBACK u=d1: R_old=txg3 i1; readable ring max txg3; first new root txg4 i2; wm=12:2
    txg4 i2 root->disk1 wm=12:2
    txg5 i2 root->disk0 wm=12:2
    txg6 i2 root->disk0 creates (12,2,bg6) wm=12:3
    txg7 i2 root->disk1 creates (12,3,bg7) wm=12:4
```

txg 4 落在盘 1、建了 inode 2，那时本实例早已覆盖两块盘，这次 fsync 返回过（`violations` 里的 `True`）。回退扫根环时盘 1 这一次读不出——受影响的只有 txg 4 那一个槽（盘 1 上更早的 txg 1 没有头 12），效果等于「最新的根槽读不出」一个槽故障（第二节末尾的 spot 检查用 `new1` 复跑同一段）。可读根的最大 txg 是 3，甲 / 乙取到的最大水位是 2，回退之后第一个用户对象又拿到 2。丙跑同一段：新对象 (12, 2, 出生代 6)，被抛弃的是 (12, 2, 出生代 4)，按丙的判据不算违例。

### 2.3 甲 / 乙的 reload 读法：回退那次发布里切换，一个故障

第 61–73 行（整行抄）：

```
example warm=strict arm=A/reload faults=1 script=[p1 rb2none1 u2]
  violations=[((12, 2), [2, 3], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 creates (12,2,bg4) wm=12:3
    ROLLBACK u=none: R_old=txg3 i1; readable ring max txg4; first new root txg5 i2; wm=12:3
    txg5 i2: rollback root write FAILS
      switch -> i3 (watermark reloaded from R_old)
    txg6 i3 root->disk0 wm=12:2
    txg7 i3 root->disk1 wm=12:2
    txg8 i3 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i3 root->disk0 creates (12,3,bg9) wm=12:4
```

回退时算出的最大水位是 3；回退那次发布的根槽写失败、切换。按 reload 读法（切换是「挂载内做一次恢复」，所选根是 R_old，水位按 R_old 盘上那一版重载）水位回到 2，inode 2 被重发。这是借用式条款的参数没有绑死（`.claude/rules/fs-design.md`「借用式条款要把被借规则的每个参数都绑死」）：「实例切换 = 挂载内做一次恢复」借来的恢复规则里，「单调量从哪里载入」这个参数在甲 / 乙之下有两个取值，今天的条文（`23-journal的角色与格式.md:691`、`18-块里携带什么信息.md:879`）只绑了「所选根 = R_old」，没绑单调量。按最弱读法记一次输；甲 / 乙可以收严成「回退那次发布里的切换沿用回退时取到的最大值」，只多要求一件事。丙在两种读法下行为相同：切换只会把 txg 往大推（`23-journal的角色与格式.md:691`），出生代不会变小。

### 2.4 丙：严格暖机最少 2 个故障，宽暖机 1 个

严格暖机（第 74–89 行，整行抄）：

```
example warm=strict arm=C faults=2 script=[p0 p0 p0 p0 p1 rb1d0n10 u2]
  violations=[((12, 2, 8), [2, 3], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 wm=12:2
    txg5 i1 root->disk0 wm=12:2
    txg6 i1 root->disk0 wm=12:2
    txg7 i1 root->disk1 wm=12:2
    txg8 i1 root->disk0 creates (12,2,bg8) wm=12:3
    ROLLBACK u=d0n1: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; wm=12:2
    txg5 i2 root->disk0 wm=12:2
    txg6 i2 root->disk0 wm=12:2
    txg7 i2 root->disk1 wm=12:2
    txg8 i2 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i2 root->disk0 creates (12,3,bg9) wm=12:4
```

盘 0 这一次整块读不出（txg 5、6、8 的根都在盘 0），再加盘 1 上最新的 txg 7 那个槽读不出，两个故障。可读根的最大 txg 是 4，第一个新根 txg 5；严格暖机再推 txg 6（盘 0）、txg 7（盘 1）才覆盖两块盘，第一个用户对象落在 txg 8、出生代 8。被抛弃时间线恰好在 txg 8 建过 inode 2（它在 txg 5–7 没建过对象），于是 (12, 2, 8) 被重发。同一段历史上甲也中。

宽暖机（第 38–49 行，整行抄）：

```
example warm=lenient arm=C faults=1 script=[p0 p0 p1 rb1d00 u2]
  violations=[((12, 2, 6), [2, 3], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 wm=12:2
    txg5 i1 root->disk0 wm=12:2
    txg6 i1 root->disk0 creates (12,2,bg6) wm=12:3
    ROLLBACK u=d0: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; wm=12:2
    txg5 i2 root->disk0 wm=12:2
    txg6 i2 root->disk0 creates (12,2,bg6) wm=12:3
    txg7 i2 root->disk1 creates (12,3,bg7) wm=12:4
```

只按「槽」计故障、不许整块盘读不出时的四段 spot 检查（严格暖机；第 475–490 行，整行抄）：

```
name=spot case='A/B one slot fault' arm=A script=[p1 rb1new10 u2] faults=1 violations=[((12, 2), [2, 3], True)]
name=spot case='A/B one slot fault' arm=AJ script=[p1 rb1new10 u2] faults=1 violations=[]
name=spot case='A/B one slot fault' arm=C script=[p1 rb1new10 u2] faults=1 violations=[]
name=spot case='A/B one slot fault' arm=CJ script=[p1 rb1new10 u2] faults=1 violations=[]
name=spot case='C lenient 1-event history under strict' arm=A script=[p0 p0 p1 rb1d00 u2] faults=1 violations=[((12, 2), [2, 3], True)]
name=spot case='C lenient 1-event history under strict' arm=AJ script=[p0 p0 p1 rb1d00 u2] faults=1 violations=[]
name=spot case='C lenient 1-event history under strict' arm=C script=[p0 p0 p1 rb1d00 u2] faults=1 violations=[]
name=spot case='C lenient 1-event history under strict' arm=CJ script=[p0 p0 p1 rb1d00 u2] faults=1 violations=[]
name=spot case='slots only, 2 newest unreadable' arm=A script=[p0 p0 p0 p0 p1 rb1new20 u2] faults=2 violations=[((12, 2), [2, 3], True)]
name=spot case='slots only, 2 newest unreadable' arm=AJ script=[p0 p0 p0 p0 p1 rb1new20 u2] faults=2 violations=[]
name=spot case='slots only, 2 newest unreadable' arm=C script=[p0 p0 p0 p0 p1 rb1new20 u2] faults=2 violations=[]
name=spot case='slots only, 2 newest unreadable' arm=CJ script=[p0 p0 p0 p0 p1 rb1new20 u2] faults=2 violations=[]
name=spot case='slots only, 3 newest unreadable' arm=A script=[p0 p0 p0 p0 p1 rb1new30 u2] faults=3 violations=[((12, 2), [2, 3], True)]
name=spot case='slots only, 3 newest unreadable' arm=AJ script=[p0 p0 p0 p0 p1 rb1new30 u2] faults=3 violations=[]
name=spot case='slots only, 3 newest unreadable' arm=C script=[p0 p0 p0 p0 p1 rb1new30 u2] faults=3 violations=[((12, 2, 8), [2, 3], True)]
name=spot case='slots only, 3 newest unreadable' arm=CJ script=[p0 p0 p0 p0 p1 rb1new30 u2] faults=3 violations=[]
```

读法：一个槽读不出，甲中、丙不中（第 475–478 行）；宽暖机下那段一个事件的丙历史放到严格暖机下，丙不中（第 479–482 行）；最新两个槽读不出，甲中、丙不中（第 483–486 行）；最新三个槽读不出，甲、丙都中（第 487–490 行）。AJ、CJ 在这四段上都不中。

**为什么严格读法下一个故障打不穿丙**（推理；枚举与 spot 检查都与它相符，但它本身没有被机检证明）：严格暖机下，任何实例在建第一个用户对象之前都已经有一个根落在盘 1（`16-发布语义.md:207`）；区域归属 0 / 1 / 0（`22-单元原子性怎么合成.md:1005`）让同一个实例两次盘 1 发布之间至多隔两次盘 0 发布，盘 1 那次写失败就换实例、新实例又要先落一次盘 1。于是一个出生代为 g 的已发布对象，另一块盘上总有一个 txg ≥ g − 2 的根；一个读故障（一块盘或一个槽）至多让可读最大值落后 2，而回退之后的新实例至少要写两个根才覆盖两块盘，第一个用户对象的出生代 ≥ 可读最大值 + 3 > g。要打穿丙得让可读最大值落后 ≥ 3：两块盘各坏一处，或者最新三个槽都读不出。⚠️ 这条推理绑在第一版的 0 / 1 / 0 归属与「暖机至多推 R = 3 次」上；加盘或改归属，余量要重算。

**丙的这些违例都分不出臂**：每一条丙中的脚本甲也中（第 201 行）。按跑前条款「三条臂在 U1 被同一段历史一起打中 ⇒ 打中不分辨臂：病根另记」处理。病根在三条臂共用的载体上：回退只看「根环里可读根的最大值」，而每个 txg 的根只有一份、只落在一块盘上（`22-单元原子性怎么合成.md:1005`）。甲 / 乙拿这个载体取水位，丙拿它取 txg——同一个载体读漏时，甲 / 乙的余量是 0（漏掉的那个根发过的号直接被重发），丙的余量是「第一个新根 + 暖机」那几个 txg。

### 2.5 任务书点名的其余几格

**被抛弃的根先离开根环**，分两种时刻：

- 回退**之前**离开（R_old 还在环里，比它新的被抛弃根却已被覆写）：一次根槽写失败就造得出来（「写失败的槽按旧内容算」，`16-发布语义.md:372`，R_old 那个槽因此多活一圈），再配一个在被抛弃时间线里销毁的可写头，甲 / 乙就读不到那个头的水位。整段历史在第四节（乙那一问）。甲 / 乙中，丙、AJ、CJ 不中，1 个故障。
- 回退**之后**离开：丙不受影响。回退之后每个新根的 txg 都大于回退那一刻的可读最大值，环里的最大 txg 只增不减；被抛弃的根被覆写，只会让环里少几个 txg 更小的根。

**连续两次回退**（sweep2 `two_rollbacks`，严格暖机，第 246–251 行，整行抄）：

```
name=sweep2 warm=strict family=two_rollbacks arm=A runs=188 violating=135 tree_id_reused=0 min_example=(1, 4, 'rb1none0 u2 rb1d10 u1')
name=sweep2 warm=strict family=two_rollbacks arm=AJ runs=188 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=two_rollbacks arm=AJ2 runs=188 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=two_rollbacks arm=C runs=188 violating=6 tree_id_reused=0 min_example=(2, 5, 'p1 rb1none0 u2 rb1d0n10 u1')
name=sweep2 warm=strict family=two_rollbacks arm=CJ runs=188 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=two_rollbacks arm=CJ2 runs=188 violating=0 tree_id_reused=0 min_example=None
```

第二次回退只是 2.2 / 2.4 那两种形状再来一遍：甲 1 个故障（第二次回退时最新的根槽读不出，例子在第 372–385 行），丙 2 个故障（`d0n1`，例子在第 386–402 行）；AJ、AJ2、CJ、CJ2 不中。没有新机制。

**回退之后崩溃**（sweep2 `rb_then_crash_mount`，严格暖机，第 240–245 行，整行抄）：

```
name=sweep2 warm=strict family=rb_then_crash_mount arm=A runs=72 violating=8 tree_id_reused=0 min_example=(2, 6, 'p1 rb1none0 u2 crash mountd0n1 u1')
name=sweep2 warm=strict family=rb_then_crash_mount arm=AJ runs=72 violating=8 tree_id_reused=0 min_example=(2, 6, 'p1 rb1none0 u2 crash mountd0n1 u1')
name=sweep2 warm=strict family=rb_then_crash_mount arm=AJ2 runs=72 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=rb_then_crash_mount arm=C runs=72 violating=6 tree_id_reused=0 min_example=(2, 6, 'p1 rb1none0 u2 crash mountd0n1 u1')
name=sweep2 warm=strict family=rb_then_crash_mount arm=CJ runs=72 violating=6 tree_id_reused=0 min_example=(2, 6, 'p1 rb1none0 u2 crash mountd0n1 u1')
name=sweep2 warm=strict family=rb_then_crash_mount arm=CJ2 runs=72 violating=0 tree_id_reused=0 min_example=None
```

丙那一段（第 336–353 行，整行抄）：

```
example2 family=rb_then_crash_mount arm=C faults=2 script=[p1 rb1none0 u2 crash mountd0n1 u1] tree_reuse=[]
  violations=[((12, 3, 8), [3, 5], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 creates (12,2,bg4) wm=12:3
    ROLLBACK u=none: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; wm=12:3
    txg5 i2 root->disk0 wm=12:3
    txg6 i2 root->disk0 wm=12:3
    txg7 i2 root->disk1 wm=12:3
    txg8 i2 root->disk0 creates (12,3,bg8) wm=12:4
    txg9 i2 root->disk0 creates (12,4,bg9) wm=12:5
    CRASH (nothing in flight)
    MOUNT u=d0n1: selects txg4 i1, replays to txg4, new i3, wm=12:3
    txg5 i3 root->disk0 wm=12:3
    txg6 i3 root->disk0 wm=12:3
    txg7 i3 root->disk1 wm=12:3
    txg8 i3 root->disk0 creates (12,3,bg8) wm=12:4
```

回退完成、新实例 i2 建了 inode 3、4 之后普通崩溃；重挂时盘 0 这一次读不出，i2 在盘 1 上唯一的根（txg 7）也读不出，两个故障。普通恢复选到的是 R_old 自己（txg 4，i1）：它是最新的可读根，按它自己那一版表判有效；重放不跨实例边界，新实例 i3 从 txg 5 往上数（`checks-owed.md:303` C331（择根倒挂压过已确认的写） 那一行写的正是这个形状），i2 发过、已确认的 (12, 3, 8) 被 i3 重发。**四条臂连同只在回退时生效的两个收严 AJ、CJ 一起中**；只有把「取 journal 最大值」推广到每一次挂载的 AJ2、CJ2 不中（第 242、245 行）。按跑前条款这是「打中不分辨臂」，病根是 C331（择根倒挂压过已确认的写）。它比 C331 那一行多写出一个后果：回退之后的一次普通恢复，会让回退之后发过的号（甲 / 乙）或 (号, 出生代)（丙）被重发。

**回退那次发布里切换**：见 2.3（甲 / 乙 reload 读法 1 个故障；丙在两种读法下都不中）。

**回退之后建克隆头**（sweep2 `clone_after_rb` 与 `clone_in_abandoned`，严格暖机，第 228–239 行，整行抄）：

```
name=sweep2 warm=strict family=clone_after_rb arm=A runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_after_rb arm=AJ runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_after_rb arm=AJ2 runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_after_rb arm=C runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_after_rb arm=CJ runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_after_rb arm=CJ2 runs=9 violating=0 tree_id_reused=0 min_example=None
name=sweep2 warm=strict family=clone_in_abandoned arm=A runs=84 violating=25 tree_id_reused=30 min_example=(1, 6, 'p1 clone uc1 rb1d00 clone uc1')
name=sweep2 warm=strict family=clone_in_abandoned arm=AJ runs=84 violating=30 tree_id_reused=30 min_example=(1, 6, 'p1 clone uc1 rb1d00 clone uc1')
name=sweep2 warm=strict family=clone_in_abandoned arm=AJ2 runs=84 violating=30 tree_id_reused=30 min_example=(1, 6, 'p1 clone uc1 rb1d00 clone uc1')
name=sweep2 warm=strict family=clone_in_abandoned arm=C runs=84 violating=0 tree_id_reused=30 min_example=None
name=sweep2 warm=strict family=clone_in_abandoned arm=CJ runs=84 violating=0 tree_id_reused=30 min_example=None
name=sweep2 warm=strict family=clone_in_abandoned arm=CJ2 runs=84 violating=0 tree_id_reused=30 min_example=None
```

在干净的历史上回退之后建克隆，四条臂都不中（第 228–233 行）。被抛弃的时间线里建过克隆、回退时那几个根读不出、回退之后再建一个克隆（第 252–267 行，甲那一段，整行抄）：

```
example2 family=clone_in_abandoned arm=A faults=1 script=[p1 clone uc1 rb1d00 clone uc1] tree_reuse=[19]
  violations=[((19, 3), [3, 4], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 root->disk1 creates (12,2,bg4) wm=12:3
    clone of 12 -> tree 19 (wm 3)
    txg5 i1 root->disk0 wm=12:3,19:3
    txg6 i1 root->disk0 creates (19,3,bg6) wm=12:3,19:4
    ROLLBACK u=d0: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; wm=12:3
    txg5 i2 root->disk0 wm=12:3
    txg6 i2 root->disk0 wm=12:3
    txg7 i2 root->disk1 wm=12:3
    clone of 12 -> tree 19 (wm 3)
    txg8 i2 root->disk0 wm=12:3,19:3
    txg9 i2 root->disk0 creates (19,3,bg9) wm=12:3,19:4
```

回退时盘 0 这一次读不出，建克隆的根（txg 5）与克隆里建对象的根（txg 6）都在盘 0，可读根的树 ID 水位是 19，回退之后的新克隆又拿到树 ID 19——**一个已发布、已确认的树 ID 被重发，三条臂都一样**（第 234–239 行的 `tree_id_reused` 列；AJ / AJ2 管不到它，它们只修 inode 号水位）。这是今天已定的树 ID 水位条款自己的洞：`08-核心索引结构.md:476` 的依据 ㈠ 推到「能被回退跨过的根都还在环里」，而「在环里」不等于「读得出」；I-7.8（根记录树 ID 水位不低于全池最大树 ID）（`invariants.md:54`）判的也是读得出的根记录。甲 / 乙把它放大成 inode 身份 (19, 3) 的重发；丙不中，因为新克隆里对象的出生代（9）与被抛弃克隆里的（6）不同。树 ID 重发本身三条臂都有，按跑前条款另记一笔；它在各臂上的后果不同，第五节的三问分开写。

## 三、U2：前提二今天的每个消费者，在各臂下成不成立

先说一个把整张表压扁的事实（条款指路 + 推理）：回退之后，被抛弃时间线上的对象在盘上的处境与一次普通崩溃丢掉的 checkpoint 里的对象**相同**——两者都靠实例表那一行判「未发布」（回退行与中间实例行、崩溃后的恢复行，`18-块里携带什么信息.md:879`），它们的单元都带写序 (实例代号, 事务号)、与新时间线的单元按实例代号分得开。而普通崩溃丢掉的对象，它的 inode 号今天在三条臂下**都会**被重发（`08-核心索引结构.md:378` 把戒律收窄到「已发布的」正是为此）。⇒ 盘上任何一个只认号的消费者，要是靠实例表与写序分辨时间线，它在丙的回退之后遇到的情形，在甲 / 乙的每一次普通崩溃之后早就遇到过；要是不靠这两样，它在普通崩溃之后就已经坏了。丙在盘上新增的只有一件事：那个号当初**发布过、确认过**——而这件事只有盘外的观察者记得。

| 消费者 | 甲 / 乙 | 丙 | 依据 |
|---|---|---|---|
| 插入永远落最右叶、分裂点取末尾、右半容器号 = 触发分裂的那条记录的号 | 成立 | 成立：回退后新号 ≥ R_old 的水位 > R_old 那棵 inode 树里的全部 key（I-9.6（水位大于两处最大号）），插入照旧落最右叶；新旧两条时间线在同一个号上分裂出的容器，身份四段里出生代不同。出生代也撞上时（2.4 的 2 故障历史），两版靠写序与实例表分，与普通崩溃同一条路 | `08-核心索引结构.md:369`、`:372`；`invariants.md:262` |
| `deleted_inodes` 回收窗口里 extent key 三段逐字相同 | 成立 | 成立：那个窗口要求旧对象 N 的 extent 还在**同一棵** extent 树里；回退后 extent 树是 R_old 那一版，而 R_old 里还在回收窗口的对象，号都小于 R_old 的水位（记录回收完成前留在 inode 树里）。⚠️ 条件：`deleted_inodes` 与批量删除的意图都住在随根回退的树里——C118 今天没定它们住哪，见表后 | `08-核心索引结构.md:378`；`checks-owed.md:124` |
| AAD / nonce 第二道闸 | 成立 | 成立，但出生代那一段从「深度防御」变成「承重」：它要求出生代本身不重发，余量就是 2.4 那几个 txg；在 2.4 的 2 故障历史里新旧对象 AAD 相同（甲 / 乙在同一段历史里号也重发，AAD 一样相同）。nonce 那一道与臂无关：nonce 水位住超级块，回退只载入记账统计量 | `09-加密.md:536`、`:777`；`13-验证路线.md:251` |
| 删除写区间记录（对象 ID + 出生代 + 全区间） | 成立 | 成立：墓碑本来就按 (ID, 出生代) 认对象，与丙的判据同形；被抛弃时间线的墓碑单元按实例表判未发布 | `18-块里携带什么信息.md:749` |
| I-9.6（水位大于两处最大号） | 甲撤第 12 项时主语要改成树表条目那一格；乙不变 | 成立：回退后水位 = R_old 的水位。⚠️「该树全部墓碑记录」要写明是已发布的墓碑，否则 checker 的扫描方向把被抛弃时间线的墓碑算进来会误红 | `invariants.md:262` |
| I-9.11（可写克隆头有水位行） | 甲撤第 12 项时改成「树表条目有水位」；乙不变 | 不变 | `invariants.md:267` |
| 对外暴露的 inode 号 | 0 故障的回退后不重发（2.2 / 2.3 / 2.5 的 1 故障历史里会） | **0 故障的回退后就把确认过的号重发给新文件** | kb 全仓 grep `NFS` / `i_generation` / `st_ino` / `文件句柄`：只有 `18-块里携带什么信息.md:97` 引 ext4 的校验和种子与 `prior-art.md` 两处，没有一条条款定义对外身份 |
| 级 1 扫描重建（实例表读不出）按号把 extent 接到记录上 | 普通崩溃之后就已经混接；回退后 I-9.4 红 | 同左，另加一个重号 | 见下 |

**对外暴露的 inode 号：一个只认号的具体序列（丙，0 故障）**。① NFS 客户端拿到文件 f 的句柄，句柄只含 (树 12, inode N)；f 在 txg g 发布、fsync 返回。② 管理员回退到 g 之前的 R_old。③ 新文件 h 按丙拿到 inode N（出生代 g′ > g）。④ 客户端用旧句柄读，服务端按 (12, N) 找到 h，把 h 的内容当 f 返回。甲 / 乙在同一段 0 故障历史里不重发 N，旧句柄查不到对象、报 ESTALE。句柄里带出生代（例如 Linux 的 `i_generation` 取出生代低 32 位）就分得开：服务端做决定那一刻看得到 h 的记录偏移 8 那个出生代（`08-核心索引结构.md:396`）。⇒ 丙要补一条：对外暴露的对象身份 = (inode 号, 对象出生代)。⚠️ 甲 / 乙也要这一条：2.2 / 2.3 / 2.5 的 1 故障历史里号同样被重发，那时出生代是唯一还分得开的字段。所以它不是丙独有的代价，只是在丙下它扛的是 0 故障的路径。这一格是「要补的条款」还是「打中」，第五节按跑前条款写我的判断。

**C118 那一格是条件式的**：今天全仓没有「按号作 key、又不随根回退」的结构（`deleted_inodes` 与意图住的 `logged_ops` 树都没有树 ID、没进树表，`checks-owed.md:124`）。将来要是有了（例如批量删除的意图只记号、住在不随根回退的地方），丙在 0 故障的回退之后就会让它作用到新对象 N 上，甲 / 乙不会。⇒ 丙要再补一条：凡按 inode 号作 key 的结构必须随根回退，或者 key 里带出生代。

**级 1 扫描重建**（`18-块里携带什么信息.md:879`「表不可读时」那一句：全部旧实例的全部单元落进「无行 ⇒ 已发布」）。小模型 `l1_rebuild` 的三段（第 493–498 行，整行抄）：

```
name=rebuild case=crash join_by_birthgen=False result={'leaves(cno:[inos])': [(1, [1, 2, 3])], 'I-9.4_violations': 0, 'duplicate_keys': 0, 'mixed_extents': 1, 'orphan_extents': 0}
name=rebuild case=crash join_by_birthgen=True result={'leaves(cno:[inos])': [(1, [1, 2, 3])], 'I-9.4_violations': 0, 'duplicate_keys': 0, 'mixed_extents': 0, 'orphan_extents': 2}
name=rebuild case=rollback_AB join_by_birthgen=False result={'leaves(cno:[inos])': [(1, [1, 2, 5]), (4, [4]), (6, [6])], 'I-9.4_violations': 1, 'duplicate_keys': 0, 'mixed_extents': 0, 'orphan_extents': 2}
name=rebuild case=rollback_AB join_by_birthgen=True result={'leaves(cno:[inos])': [(1, [1, 2, 5]), (4, [4]), (6, [6])], 'I-9.4_violations': 1, 'duplicate_keys': 0, 'mixed_extents': 0, 'orphan_extents': 2}
name=rebuild case=rollback_C join_by_birthgen=False result={'leaves(cno:[inos])': [(1, [1, 2, 3]), (4, [4]), (4, [4])], 'I-9.4_violations': 1, 'duplicate_keys': 1, 'mixed_extents': 1, 'orphan_extents': 0}
name=rebuild case=rollback_C join_by_birthgen=True result={'leaves(cno:[inos])': [(1, [1, 2, 3]), (4, [4]), (4, [4])], 'I-9.4_violations': 1, 'duplicate_keys': 1, 'mixed_extents': 0, 'orphan_extents': 2}
```

- `crash`（普通崩溃，三条臂都一样）：崩在 txg 5 的对象 3 留下偏移 0、1 两个单元，恢复后新对象 3 只写了偏移 0；按号接 extent，偏移 1 那个旧单元被接到新对象 3 上（`mixed_extents` 1）——新文件的空洞里读出别的对象的旧数据。按 (号, 出生代) 接就只剩两个孤儿 extent。⇒ 这个消费者今天在**普通崩溃之后**就已经只认号而出错，与回退无关、与臂无关。
- `rollback_AB`：没有混接，但 I-9.4（容器号不超最小 key）（`invariants.md:260`）判红：最右那片叶的现行版本来自新时间线（含新号 5），被抛弃时间线分裂出的容器从号 4 起，两个 key 区间交叠。
- `rollback_C`：同样 I-9.4 红，另加一个重号（inode 4 在两个容器里各一条）与一个混接；按出生代接消掉混接、消不掉重号——inode 树的 key 只有号（`08-核心索引结构.md:378`），同号两条只能留一条，要一条「同号取出生代大的」重建规则。

⇒ 这一格三条臂一起坏（I-9.4），丙多一个重号。`18-块里携带什么信息.md:879` 那句「被回退抛弃的整段时间线也在内，所以不是记歧义」对三条臂都不成立：回退之后最右那片叶在两条时间线里各被 COW 过、身份不变，重建取到的现行版本来自一条时间线，分裂出的容器来自另一条。按跑前条款这一格不分辨臂，另记一笔（第九节）。⚠️ 这个小模型只有一片容量 3 的叶、两次分裂，是构造出来的示例，不是枚举；它说明「会坏」，不说明「多常坏」。

**先例**：C103（容器索引无落点）（`checks-owed.md:109`）对打包记录类型 1 的容器号已经写着「计数器回退不破身份（出生代不同）」——仓里已经有一个号在回退后重发、靠出生代分身份的计数器，与丙同一种身份。

## 四、乙：前提十的点删与根环深度；乙读不到那一行的可达格

**点删本身不会让乙读不到**（条款指路 + 推理，模型里每个根带着完整状态，与之相符）：记账的「代」按发布走（`16-发布语义.md:113`），每次发布点删第 T − K 代（`05-快照-空间记账机制.md:284`），而水位行每次发布都重写成第 T 代（`05-快照-空间记账机制.md:371`；空发布也重写，`16-发布语义.md:192`）。乙在回退时读的是**每个可读根自己那棵记账树**里该头的现行行：根 T 的记账树里必有第 T 代那一行（只要该头在 T 时是可写头），点删删的是 T − K 代，碰不到它。所以 K = 根环槽总数 + 1（`05-快照-空间记账机制.md:296`）对乙的读法不承重。⚠️ 反过来，要是乙被实现成「在最新那棵记账树里按代号去找第 T_old + 1 … T_max 代的行」，那是错的读法：被抛弃时间线的行根本不在新时间线那棵 COW 树里，与 K 无关。乙的条文要写明「读每个可读根自己的记账树」。另一种读不到：F 抬过之后，比 F 旧的根引用的单元可以被重新分配（`16-发布语义.md:371`），读它们的记账树会校验失败——但这些根比 R_old 旧（R_old 的 txg ≥ F_生效）、在同一条时间线上，水位不超过 R_old 那一格，读不到也不丢东西（推理）。

**乙读不到那一行的可达格：R_old 靠一次根槽写失败留在环里，比它新、还带着那个头的被抛弃根却都已离开**（sweep3，`c143_model.out` 第 405–444 行、第 449 行、第 457 行，整行抄）：

```
name=sweep3 S=4 skip=True n_obj=1 arm=A R_old=txg5 R_old_in_ring=True ring_roots_carrying_H2=[5] rollback_ok=True faults=1 violations=[((19, 2), [2, 3], True)]
    txg1 i1 root->disk1 wm=
    txg2 i1 root->disk0 wm=
    txg3 i1 root->disk0 creates (12,1,bg3) wm=12:2
    clone of 12 -> tree 19 (wm 2)
    txg4 i1 root->disk1 wm=12:2,19:2
    txg5 i1 root->disk0 wm=12:2,19:2
    txg6 i1 root->disk0 creates (19,2,bg6) wm=12:2,19:3
    destroy head 19
    txg7 i1 root->disk1 wm=12:2
    txg8 i1 root->disk0 wm=12:2
    txg9 i1 root->disk0 wm=12:2
    txg10 i1 root->disk1 wm=12:2
    txg11 i1 root->disk0 wm=12:2
    txg12 i1 root->disk0 wm=12:2
    txg13 i1 root->disk1 wm=12:2
    txg14 i1 root->disk0 wm=12:2
    txg15 i1 root->disk0 wm=12:2
    txg16 i1 root->disk1 wm=12:2
    txg17 i1: root write to disk0 FAILS
      switch -> i2
    txg18 i2 root->disk0 wm=12:2
    txg19 i2 root->disk1 wm=12:2
    txg20 i2 root->disk0 wm=12:2
    txg21 i2 root->disk0 wm=12:2
    txg22 i2 root->disk1 wm=12:2
    txg23 i2 root->disk0 wm=12:2
    txg24 i2 root->disk0 wm=12:2
    txg25 i2 root->disk1 wm=12:2
    txg26 i2 root->disk0 wm=12:2
    txg27 i2 root->disk0 wm=12:2
    txg28 i2 root->disk1 wm=12:2
    ROLLBACK u=none: R_old=txg5 i1; readable ring max txg28; first new root txg29 i3; wm=12:2,19:2
    txg29 i3 root->disk0 wm=12:2,19:2
    txg30 i3 root->disk0 wm=12:2,19:2
    txg31 i3 root->disk1 wm=12:2,19:2
    txg32 i3 root->disk0 creates (19,2,bg32) wm=12:2,19:3
name=sweep3 S=4 skip=True n_obj=1 arm=AJ R_old=txg5 R_old_in_ring=True ring_roots_carrying_H2=[5] rollback_ok=True faults=1 violations=[]
name=sweep3 S=4 skip=True n_obj=1 arm=C R_old=txg5 R_old_in_ring=True ring_roots_carrying_H2=[5] rollback_ok=True faults=1 violations=[]
name=sweep3 S=4 skip=True n_obj=1 arm=CJ R_old=txg5 R_old_in_ring=True ring_roots_carrying_H2=[5] rollback_ok=True faults=1 violations=[]
name=sweep3 S=4 skip=False n_obj=1 arm=A R_old=txg5 R_old_in_ring=False ring_roots_carrying_H2=[] rollback_ok=False faults=0 violations=None
name=sweep3 S=5 skip=True n_obj=1 arm=A R_old=txg5 R_old_in_ring=True ring_roots_carrying_H2=[5] rollback_ok=True faults=1 violations=[((19, 2), [2, 3], True)]
```

逐步：txg 4 建克隆头 19；txg 5 = R_old；txg 6 在头 19 里建 inode 2（已确认）；txg 7 销毁头 19。之后一直发布到 txg 17——它落在 R_old 那个槽上（区域内槽位 = (txg div 3) mod S，S = 4 时 12 个 txg 轮一圈，`22-单元原子性怎么合成.md:1005`），根槽写失败，切换、txg 推进到 18；失败的槽按旧内容算（`16-发布语义.md:372`）⇒ R_old 留在环里。再发布到 txg 28，环里其余 11 个槽全换成不带头 19 的根。回退到 R_old：候选集按最新根指着的那一版实例表判，i1 那一行是 (i1, 16)，R_old（txg 5）有效。环里带头 19 的根只剩 R_old 自己（`ring_roots_carrying_H2=[5]`），乙（甲同）取到的最大水位就是 R_old 那一格的 2，回退之后头 19 又发出 inode 2。故障数 1。S = 5 同样（第 457 行）；根槽不失败的对照里 R_old 自己被覆写，回退不成立（第 449 行）。

⇒ 这一格不是点删造成的，是**根环不是一段连续的 txg 窗口**：一次根槽写失败让最旧的那个根多活一圈，它与最新的根之间那一段却已经离开。乙（与甲）的「取根环里全部可读根的最大值」默认了「R_old 在环里 ⇒ 比 R_old 新的根都在环里」，这一格把它打穿。丙不受影响（新根 txg 29 大于环里的一切）；AJ 不受影响（journal 里 txg 6 那条记录的新根段指着带头 19 的那一版状态）。⚠️ 这一格要求被抛弃时间线里有一个可写头被销毁；「还有自己活快照的可写头不许销毁」（`06-快照实现模型.md:240`）挡不住它——模型里被销毁的头没有快照。

**乙把第 13、14 项一起捆进来**：第 13 项（清扫水位）取最大值，推理上无害：被抛弃时间线里那一轮清扫能确认的释放代不超过它当时环里最旧的根（`03-空间分配.md:180`），而 R_old 在环里，所以不超过 T_old，那段释放落在两条时间线共有的历史里。第 14 项（最近一次根销毁的代号）取最大值，会把「只在被抛弃时间线里发生过的一次根销毁」带进新时间线——那个根在 R_old 里是活的。它进墓碑回收门「门 = max(死亡代号, 这一项)」（`05-快照-空间记账机制.md:364`），而这扇门的方向（门高了回收得更早还是更晚）kb 里只有这一个式子（全仓 grep「回收门」只命中 `05-快照-空间记账机制.md:363`、`:364` 两行）。⇒ 乙对第 14 项的做法**判不了对错，这一格没核**。乙要么把第 14 项从捆里拿掉，要么先把那扇门的语义写全。模型没建这一格。

## 五、打中之后的三问

逐个写三件事：它分不分辨臂；被判的系统在做决定那一刻看不看得到判别它的东西；它满足的是哪条判据的哪一个分句。

| # | 打中 | 分辨臂吗 | 做决定那一刻看得到判别子吗 | 满足哪条判据的哪个分句 |
|---|---|---|---|---|
| H1 | 甲 / 乙：回退那一刻最新的根槽读不出，1 个故障（2.2） | 分辨：同一段脚本丙判不违例；严格暖机下 1 个故障的违例脚本只有甲 / 乙（第 150、151 行） | 看得到，只是今天的规则不看：读不出的那个根发布时写过一条 journal 记录（`16-发布语义.md:287`），journal 环两盘镜像（`22-单元原子性怎么合成.md:229`），记录的新根段带树表单元指针（`23-journal的角色与格式.md:694`）；AJ 按它取最大值，0 违例 | U1「甲、乙按号」+「至少覆盖：回退那一刻最新的根暂时读不出」 |
| H2 | 甲 / 乙：回退那次发布里切换（reload 读法），1 个故障（2.3） | 分辨：丙在两种读法下都不中 | 看得到：最大值是回退那一刻算出来的、还在内存里；缺的是条文里绑参数的那一句，不是信息 | U1「回退那次发布里切换」；只在最弱读法下成立（keep 读法不中），按「记一次输、只许收严、写明收严在哪」处理 |
| H3 | 甲 / 乙：根环缺口（R_old 靠一次根槽写失败留下，带那个头的被抛弃根都已离开，头已销毁），1 个故障（第四节） | 分辨：丙、AJ、CJ 不中 | 看得到：journal 里那几条记录还在；根环里看不到 | U1「甲、乙按号」+ 任务书点名的「被抛弃的根先离开根环」（回退之前离开那一种） |
| H4 | 丙：最新几个根读不出，严格暖机 2 个故障 / 宽暖机 1 个事件（2.4） | **不分辨**：每条丙中的脚本甲也中（第 201 行，0 条例外） | 看得到：被抛弃时间线最新的那几次发布各有 journal 记录，记录头带 `checkpoint_txg`；CJ 按它取 txg，0 违例 | U1「丙按 (号, 出生代)」+「回退那一刻最新的根暂时读不出」 |
| H5 | 四条臂连同 AJ、CJ：回退之后普通崩溃，重挂时两个读故障（2.5） | **不分辨** | 看得到：回退实例的 journal 记录都在；AJ2 / CJ2 按它取，0 违例 | U1「回退之后崩溃」；病根是 C331（择根倒挂压过已确认的写），不是 C143 那个载体 |
| H6 | 树 ID 被重发（被抛弃时间线里建过克隆、那几个根读不出、回退后再建克隆），1 个故障（2.5） | 树 ID 重发本身**不分辨**（三条臂都有）；它的后果分辨：甲 / 乙把它放大成 (树, 号) 重发，丙不中 | 看得到：journal 记录的新根段就带树 ID 水位 8 字节（`23-journal的角色与格式.md:694`），读记录头即可，零额外读 | 树 ID 那一半落在 D6（快照实现模型） 判据 9 上，不在 C143 的 U1 里；inode 那一半是 U1「甲、乙按号」+「回退之后建克隆头」 |
| H7 | 级 1 扫描重建（实例表读不出）按号接 extent、容器 key 区间交叠（第三节） | **不分辨**：普通崩溃之后三条臂都混接；回退之后三条臂都 I-9.4 红；丙多一个重号 | 看得到：单元头五元组里有出生代，重建可以按 (号, 出生代) 接；重号要一条「同号取出生代大的」规则 | U2「`deleted_inodes` 回收窗口里 extent key 三段逐字相同」那一类（extent key 不带出生代，前提七），消费者是重建路径 |
| H8 | 丙：对外暴露的 inode 号，0 故障（第三节） | 分辨：甲 / 乙在 0 故障的回退之后不重发号 | 看得到：服务端做决定那一刻读得到记录偏移 8 的出生代；前提是对外身份里带它 | U2「对外暴露的 inode 号」 |

**按跑前条款，我的判断**（交主 agent 评判，不是判决）：

- **U1 上丙没有被一段分辨臂的历史打中**。丙的全部违例（H4、H5）都同时打中甲 / 乙，按「三条臂在 U1 被同一段历史一起打中 ⇒ 打中不分辨臂：病根另记」处理。反向接受条款第二条（「丙只在 U1 被『回退那一刻最新的根暂时读不出』一类历史打中 ⇒ 交用户在『丙 + 全池 txg 水位』与甲、乙之间选」）按字面也被触发——H4 正是那一类——但那条条款默认了「这一类历史打中丙而不打中甲乙」，实测是反过来的：这一类历史打甲 / 乙只要 1 个故障（H1），打丙要 2 个（H4）。
- **甲 / 乙在 U1 上被三段丙不中的 1 故障历史打中**（H1、H2、H3）。跑前条款里没有「甲 / 乙被打中而丙不中」这一支。按判据字面，甲 / 乙在「U1 关得住」上不成立；H1、H3 的收严是 AJ（第七节），H2 的收严是把切换那一格的参数写死。
- **U2 上丙唯一分辨臂的是 H8**。它满足「只认号的消费者 + 具体读写序列」的字面，前提是对外句柄不带代号；而 kb 今天没有定义对外身份。更准确的说法是：丙把一条甲 / 乙本来也该有、今天没人写的条款（对外身份带出生代）从深度防御变成了承重。按跑前条款「丙在 U2 被一个只认号的消费者打中 ⇒ 丙出局」，判不判 H8 为打中交主 agent。我的倾向：H8 是三条臂都缺的一条条款，不是只打丙的机制；拿一条大家都没写的条款去否一条臂，而甲 / 乙在 H1 那种 1 故障历史里同样要靠这条条款才分得开新旧对象——判出局之前，要先答「甲 / 乙在 H1 下对外身份靠什么」。

## 六、U3 代价、U4 检查、各臂要改的 kb 正文

### 6.1 U3

| 臂 | 格式字节 | 第一个事务的字节 | 回退时多读什么 | 发号路径 |
|---|---|---|---|---|
| 甲 | 树表条目预留 24 里切 8 字节。⚠️ 这 24 字节已经被认购满：D6（快照实现模型） 已定项 3 写每头一个 livelist 计数器「只能住头的树表条目预留仅剩的 8 字节」（`06-快照实现模型.md:237`），C144（快照谱系的区间标号没有载体） 要每个快照两个数、16 字节（`checks-owed.md:147`），16 + 8 = 24。甲再切 8 就超额认购；「格式宽度不变」只在挤掉一个认购者时成立，不挤就要加宽条目（例如 148 → 156，每层 ⌊16253 / 156⌋ = 104 棵，今天 109；分母照 `08-核心索引结构.md:76` 那一行写死的 16384 − 131） | inode 树那条条目的 8 字节 0 → 2。若同时撤记账第 12 项：记账树 15 行 → 14 行（声明长度 510 → 476，`05-快照-空间记账机制.md:486` 那一行没了），而且记账 key 的树 ID 段就一个用户都没有了（`05-快照-空间记账机制.md:378` 写着它「现在只为第 12 项存在」）。若不撤：两处载体要一条一致性不变量，回退时谁赢要写 | 每个可读根一次树表下探（根记录 → 树表单元，一到两层），至多 R × S 个根；与盘容量无关 | 不读盘 |
| 乙 | 0 | 0 | 每个可读根 × 每个可写头一次记账树点查，至多 R × S × 头数 × 记账树层数；与盘容量无关，随头数涨 | 不读盘 |
| 丙 | 0 | 0 | 0 | 不读盘 |
| CJ（攻方收严） | 0 | 0 | 0 次额外读：`checkpoint_txg` 是记录头里永久要读得懂的字段（`16-发布语义.md:313`），恢复本来就逐条验证环里的记录（第七节给行号） | 不读盘 |
| AJ（攻方收严） | 0 | 0 | 乙：再加 journal 里 txg > R_old 的最新那条记录新根段指到的那一版记账树，每头一次点查；甲：那一版树表一次下探 | 不读盘 |

### 6.2 U4：会红的检查与判别力自证

三条臂的 U1 都是跨挂载的性质（一个身份先后被两次挂载里的两个对象用）。单镜像 checker 看不见被抛弃的对象——它们按实例表判未发布、不在任何可达的树里。⇒ 三条臂都要多次挂载的录制流，与 C331（择根倒挂压过已确认的写）、C332（回退实例两个根都读不出时回退被撤销）、C334（切换的所选根没有会红的检查） 同一个前置。

| 臂 | 检查 | 判别力自证 |
|---|---|---|
| 甲 / 乙 | 录制流里注入：① 回退那一刻最新的根槽读不出；② R_old 那个槽的一次根槽写失败 + 被抛弃时间线里销毁一个可写头；③ 回退那次发布的根槽写失败。逐状态判：回退之后建的每个对象，(树, 号) 与此前任何一个已发布对象都不同 | 按今天的甲 / 乙条文，①②③ 各自必须红（本模型：H1、H3、H2 各 1 个故障就红）；换成 AJ、并把切换那一格写死之后必须转绿。① 不红说明注入没到读扫描那一步 |
| 丙 | 同样三种注入，再加 ④ 盘 0 读不出 + 盘 1 最新根槽读不出（`d0n1`）、⑤ 最新三个根槽读不出。判：回退之后建的每个对象，(树, 号, 出生代) 与此前任何一个已发布对象都不同；并判「回退后第一个新根的 txg > 此前任何一个写成的根的 txg」 | 丙在 ①②③ 下必须绿、④⑤ 下必须红（本模型：2 个 / 3 个故障红）；换成 CJ 后 ④⑤ 必须转绿；把第一个新根的 txg 改成「R_old 的 txg + 1」，① 必须转红 |
| 三条臂共用 | 回退之后普通崩溃，重挂时回退实例的根读不出（2.5）；判同上 | 今天三条臂都红；AJ2 / CJ2 必须转绿 |
| 树 ID | 被抛弃时间线里建克隆、回退时那几个根读不出、回退后再建克隆；判：树 ID 不重发 | 今天三条臂都红；树 ID 水位也从 journal 记录的新根段取 max 之后必须转绿（本模型没跑这一臂，是推理） |

单镜像 checker 能判的只有一半：I-9.6（水位大于两处最大号） 在三条臂下都判得了，但它**判不出重发**——重发的号在新镜像里只出现一次。⇒ 按 U4 的触发条件（写不出判别力自证的臂记一次），单镜像这一层三条臂各记一次，不分辨臂；录制流那一层三条臂都写得出。

### 6.3 各臂要改的 kb 正文

| 臂 | 要改的 kb 正文（文件:行） |
|---|---|
| 甲 | `08-核心索引结构.md:76` 与 `:476` 所在的已定项 8（条目布局：预留 24 切 8 字节）；`06-快照实现模型.md:237` 与 `checks-owed.md:147`（预留的认购要重新分账）；`23-journal的角色与格式.md:1206`（「全部记账统计量的现行值从 R_old 那棵账重新载入」加例外）；若撤第 12 项：`05-快照-空间记账机制.md:362`、`:370`、`:378`、`:486`，`08-核心索引结构.md:381`、`:386`，`invariants.md:262`、`:267` |
| 乙 | `23-journal的角色与格式.md:1206`（重载那一句改成非单调量重载、单调量取最大值，并写明读每个可读根自己的记账树）；`18-块里携带什么信息.md:879` 与 `23-journal的角色与格式.md:691`（回退那次发布里的切换沿用回退时取到的最大值）；第 14 项要不要捆，先要 `05-快照-空间记账机制.md:364` 那扇门的语义 |
| 丙 | `08-核心索引结构.md:378`（戒律改成「已发布的 (号, 出生代) 不复用」）；`:396`（出生代那一列「不再承担复用判别」改掉）；`09-加密.md:536`（「退化为深度防御」改掉）；`18-块里携带什么信息.md:749`（「历史理由」改掉）；新增两句：对外暴露的身份 = (号, 出生代)；按号作 key 的结构随根回退或 key 带出生代（落到 C118（`deleted_inodes` 树的形态无落点） 的形态里） |
| 三条臂都要 | `08-核心索引结构.md:476`（树 ID 水位的依据 ㈠「能被回退跨过的根都还在环里」要加「而且读得出」，或者把 journal 记录新根段里的树 ID 水位一起取 max）；`18-块里携带什么信息.md:879`（「表不可读时……所以不是记歧义」对回退之后、普通崩溃之后都不成立） |

## 七、攻方自己给的收严（只在这个模型上量过，被攻过零轮）

四个收严出自同一个观察：回退要问的是「此前已经发出过什么」，今天三条臂都拿「根环里可读根的最大值」来回答。每个 txg 的根只有一份、只落在一块盘上（`22-单元原子性怎么合成.md:1005`）；每次发布的 journal 记录却住在两盘镜像的环里（`22-单元原子性怎么合成.md:229`），发布顺序保证记录先于根槽持久（`16-发布语义.md:287`），恢复本来就先全环扫描、逐条验证（`23-journal的角色与格式.md:811`），记录头里的 `checkpoint_txg` 是永久要读得懂的字段（`16-发布语义.md:313`），新根段还带着树 ID 水位与树表单元指针（`23-journal的角色与格式.md:694`）。⇒ journal 记录是一个比根环多一份副本、而且恢复时已经读过的见证。

| 收严 | 做什么 | 本模型的结果 | 修哪一格 |
|---|---|---|---|
| CJ | 丙：回退的第一个新根 txg = max(可读根的 txg, 环里全部自证通过的记录的 `checkpoint_txg`) + 1 | sweep1 两种读法 0 违例（第 10、15 行）；sweep2 只剩「回退之后崩溃」中（第 220、244 行） | H4 |
| CJ2 | CJ 推广到每一个新实例的第一次发布（普通挂载、切换、回退都取这个 max） | sweep2 全部族 0 违例（第 209、215、221、227、233、239、245、251 行） | H4、H5；它就是 C331（择根倒挂压过已确认的写） 修法候选里的「记录扫描水位」 |
| AJ | 甲 / 乙：回退时再把 journal 里 txg > R_old 的记录新根段指到的那一版水位取进 max | sweep1 0 违例（第 8、13 行）；sweep3 不中；sweep2 的「回退之后崩溃」与「被抛弃时间线里建克隆」仍中 | H1、H3 |
| AJ2 | AJ 推广到每一次挂载 | 「回退之后崩溃」不中（第 218、242 行）；「被抛弃时间线里建克隆」仍中（第 212、236 行）——树 ID 重发的洞它管不到 | H1、H3、H5 |

⚠️ **这四个都只在我的模型上量过，被攻过零轮**（`.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算『没被攻过』」）。交用户时要写明这一点，并给它们各自的检查记一笔账。

**「那个水位的载体本身会不会退回」**（反向接受条款第二条要核的那一问），按条文推，journal 里的 txg 最大值不会退回：① 回退不施加 R_old 之后的记录，但也不抹它们；新实例从前缀末 + 1 接着写（`23-journal的角色与格式.md:1235`），会一条条覆写被抛弃时间线的记录，而覆写它们的新记录 txg 都不小于回退后第一个新根的 txg——这个 txg 已经取了 journal 的最大值（CJ），所以环里的最大值只增不减。② 记录两盘镜像，一块盘这一次读不出不会让它读漏。③ 记录头带 fsid（`23-journal的角色与格式.md:263`），上一个文件系统留下的记录进不来。④ 取大了无害：txg 本来就允许跳号（根槽写失败时推进一格，`23-journal的角色与格式.md:691`）。⚠️ 这四条是按条文推的；本模型没建 journal 环的覆写与记录读故障（记录一直都在），所以「载体不退回」这一句**没有测过**。推翻它的观测：两块盘上存那条记录的槽同时读不出，或者环在一次回退的窗口里转过了一整圈。

**对跑前条款「丙没有收严的余地」的异议**：条款逐字「丙在 U2 被一个只认号的消费者打中 ⇒ 丙出局，在甲、乙之间按 U3 选（丙没有收严的余地：它本身就是把唯一性放宽的那条臂）」。丙放宽的是「唯一性判据」这一格；它另有几格可以只收严、不放宽：txg 的取法（CJ / CJ2，多要求一个见证）、对外身份（多要求出生代进句柄）、按号作 key 的结构（多要求随根回退或 key 带出生代）。这几样没有一样把唯一性再放宽一个字。⇒「没有收严的余地」按字面不成立；要是它被读成「丙出局之后不许再看这几条」，等于把一条跑前条款写成了不可推翻。这是我的异议，不是判决，交主 agent。

## 八、什么现象会推翻每一条结论

| 结论 | 推翻它的观测 |
|---|---|
| R1：甲 / 乙在「回退那一刻最新的根暂时读不出」上 1 个故障就中（H1） | 一条条款让回退在根环有槽读不出时拒绝进行（像 `18-块里携带什么信息.md:879` 删行那一句用的「根环每个槽都读成功才算」）；或者回退改从 journal 取水位（AJ）；或者读不出的那个根发过的号在别处持久、回退时读得到。任一条成立，H1 的历史就不可达 |
| R2：回退那次发布里切换，甲 / 乙 1 个故障就中（H2） | 条文写明切换重发的是在飞 checkpoint 的原样内容（keep 读法）；H2 本来就只在最弱读法下成立 |
| R3：根环缺口让甲 / 乙 1 个故障就中（H3） | 「写失败的槽按旧内容算」改成「该槽作废、不进回退候选集」；或者候选集要求 R_old 与环里最新的根之间没有缺口；或者可写头销毁时它的水位另有保存 |
| R4：严格暖机下丙最少 2 个故障（H4），宽暖机下 1 个 | 用户事务能在暖机完成之前进新实例的发布（那就是宽读法，丙 1 个故障）；区域归属不再是 0 / 1 / 0 或暖机次数变了（`16-发布语义.md:208` 自己写着加盘、改归属时要重算）；或者比枚举更长的前缀里出现 1 故障的丙违例——枚举到前缀 6、8 种读故障，没见到 |
| R5：丙中的每一条脚本甲也中（C_hit_but_A_not = 0） | 枚举范围外有一条脚本丙中而甲不中。我能想到的方向只有「txg 退回而水位没退回」：txg 与水位走不同载体、只有 txg 那个载体读漏。本模型里两者同一个载体，这一类构造不出来；甲若把水位放进比根环副本更多的地方，要重跑 |
| R6：「回退之后崩溃」三条臂都中，AJ2 / CJ2 不中（H5） | 普通恢复之后新实例的 txg 不是从所选根往上数（C331 那一行的读法被推翻）；或者回退实例的根分布让两个读故障不足以让恢复落回 R_old（本模型按 0 / 1 / 0 归属数） |
| R7：树 ID 重发三条臂都有（H6） | 树 ID 水位的依据 ㈠ 补上「读得出」（例如回退在有槽读不出时拒绝进行），或者树 ID 水位也从 journal 取 max |
| R8：级 1 扫描重建在回退之后三条臂都 I-9.4 红（H7） | 重建规则里有一条让「同一身份的现行版本」与「分裂出的容器」取自同一条时间线（例如先按写序实例聚类、再择新）；那样甲 / 乙不红，丙只剩重号 |
| R9：丙在 U2 上唯一分辨臂的是对外身份（H8） | 找到一个盘上的、按号作 key、又不随根回退的结构（C118 落定之后要复查）；或者找到一条写路径，让丙在同一条时间线里有两个已发布对象同号、同时可达 |
| R10：点删不会让乙读不到那一行 | 有一次发布不重写某个可写头的水位行（与 `05-快照-空间记账机制.md:371`、`16-发布语义.md:192` 相反的条款）；或者乙被规定成只读最新那一棵记账树 |
| R11：journal 里的 txg 最大值不会退回 | 两块盘上存那条记录的槽同时读不出，或者环在一次回退的窗口里转过一整圈；本模型没测，这一条是推理 |

## 九、没建模的、射程、建议另记的账

**模型没建的**（每一条写它会不会改结论）：

- journal 环的覆写与记录读故障：模型里记录一直都在。它影响 AJ / CJ / AJ2 / CJ2 的结论（四个收严都靠 journal），不影响甲 / 乙 / 丙按今天条文的结论（那三条臂回退时不读 journal）。
- F 与准入、影子账、分配器与单元复用：不影响 U1 的身份判定（身份只看水位与 txg）。「最新的根读不出」会不会让影子账漏护被抛弃的根引用的单元，是 C314（回退可以复用被抛弃的根引用的单元） 那一格的问题，本轮没碰。
- 事务号与 W：根的有效性只按 T_pub 判。回退行、中间实例行按条文写；切换时只写了旧实例一行（`18-块里携带什么信息.md:879` 切换那四种行只建了一部分），对 U1 的身份判定不承重。
- 只读挂载、「独占打开过半设备」：模型假定两块盘都在，只是这一次读扫描读不出。`d0` / `d1`（整块盘读不出、写得进）是一个假设；按槽计的等价故障数第二节已经给了（`new1`–`new3`，spot 检查）。
- 前缀 ≤ 6 次发布、S ∈ {4, 5}、两块盘、一个可写头（另有克隆族）：枚举结论的射程就这么大。

**射程**：`C_hit_but_A_not count=0` 是这个枚举空间里的结论，不是定理，第八节 R5 写了推翻它的方向。严格暖机下丙最少 2 个故障，是枚举加推理（2.4），推理绑在 0 / 1 / 0 归属与暖机次数上。第三节的扫描重建小模型是构造出的示例，不是枚举。

**建议另记的账**（本轮打出来、不分辨臂的病根；立不立由主 agent 判）：

1. 回退判「此前发过什么」只看根环里可读根的最大值，而每个 txg 的根只有一份：一个读故障就让树 ID 水位读漏、把已发布的树 ID 重发（H6，三条臂都有；`08-核心索引结构.md:476` 的依据 ㈠ 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 都只对读得出的根成立）。修法候选：回退在有槽读不出时拒绝进行；或者从 journal 记录新根段里的树 ID 水位取 max，读记录头就够。
2. 回退之后的一次普通恢复会让回退之后发过的号或 (号, 出生代) 被重发（H5）。归 C331（择根倒挂压过已确认的写），给它加这一个后果；CJ2 就是它修法候选里的「记录扫描水位」。
3. 级 1 扫描重建（实例表读不出）按号把 extent 接到记录上，普通崩溃之后就会混接；回退之后三条臂都会让 inode 树的 key 区间交叠（H7）。`18-块里携带什么信息.md:879`「所以不是记歧义」对这两种情形都不成立。修法候选：重建按 (号, 出生代) 接 extent，同号只留出生代大的那一条。
4. 乙把第 14 项捆进来那一格判不了：墓碑回收门的方向 kb 里只有一个式子（第四节）。
5. 甲要的 8 字节与树表条目预留的已知认购冲突（6.1 节）。

## 十、复跑命令、跑的经过、文件 sha256

复跑（只用 Python 3.12 标准库；本机一次约 3 分半、峰值常驻内存约 13 MB；输出只写 stdout，`-B` 不在仓里留 `__pycache__`）：

```
cd research/prompts/c143-r1-opus-model
python3 -B c143_model.py > "${TMPDIR:-/tmp}/c143_model.rerun.out"
cmp "${TMPDIR:-/tmp}/c143_model.rerun.out" c143_model.out && echo IDENTICAL
```

模型是确定性的（没有随机源；输出按排序打印）。第二次跑与最终版各复跑一次、与留存产物逐字节比，两次都是 `IDENTICAL`。

跑的经过（旧版本原样留存，没有覆盖）：

| 版本 | 文件 | 改了什么 |
|---|---|---|
| 第一次 | `c143_model_run1.py` / `c143_model_run1.out` | 前缀 ≤ 4 次发布、5 种读故障。第一次运行在「回退到 txg 0–2（那时还没有头 12）」时报 KeyError，改成把这类脚本当无效之后跑出这一份 |
| 第二次 | `c143_model_run2.py` / `c143_model_run2.out` | 前缀 ≤ 6、读故障加 `new3`、`d0n1`、`d1n1`；这几种形状在历史很短时会让根环一个都读不出，改成把这类脚本当无效 |
| 最终 | `c143_model.py` / `c143_model.out` | 加 AJ2、CJ2 两个收严与 spot 检查；sweep1 那部分（第 1–201 行）与第二次跑逐字节相同（`cmp` 过） |

本报告引的模型输出行号都是最终版 `c143_model.out` 的行号。

文件 sha256 与行数（`sha256sum`、`wc -l` 原样输出）：

```
450cf798778d563612a452145aeb2aeb5a2f8704d6a532624fc92de291c810c8  c143-r1-opus-model/c143_model.py
aa14cb03886ae99805c97feb888a24b5d5045bbccc66abe23bbb88052bdae357  c143-r1-opus-model/c143_model.out
97cba2e7245ba2dc812f4a97b7ffb916b0aec24f4b2b3dfee5b95fa4e0b44775  c143-r1-opus-model/c143_model_run1.py
d126afe4d809769a254e2e184dca3c54a01e7cdb38c7c2245c46b662b33762c2  c143-r1-opus-model/c143_model_run1.out
fd38de072940874f30b2ef72033a512acd4d30f3a3131c69027aadbd4d38a540  c143-r1-opus-model/c143_model_run2.py
193370976cb7aaf6281e5d253284a1a6896965b4115a2b4e580a4619f04c30fa  c143-r1-opus-model/c143_model_run2.out
  595 c143-r1-opus-model/c143_model.py
  499 c143-r1-opus-model/c143_model.out
 1094 total
```

## 附录：本报告当论证支点的 kb 行（整行，`sed` 机械抽取，抽取时逐行核过关键词；没有手抄）

**`decisions/23-journal的角色与格式.md:1206`**

```text
**显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限；表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。
```

**`decisions/23-journal的角色与格式.md:691`**

```text
14. **重放的下界（2026-09-02，用户定案）：由所选根给出——只施加 `(实例代号, checkpoint_txg)` 严格大于根的记录；tail 只是扫描起点的优化。** C113（扫描重建时多版单元的现行版本判定无输入） 定案（2026-09-05）加一条显式例外——管理员回退：从回退候选集（根环里按实例表判仍有效、且 txg ≥ F_生效的根）选 R_old，不施加它之后的任何记录、取新实例代号、在 R_old 指着的那一版实例表上写回退行，第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1，回退与第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号，以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退候选集还要 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）：平时 F 不动时深度是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。同一次定案把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。**失败的处置按失败的性质分两支**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 失败表，2026-09-05）。**判别子是当下判得出的量，不是次数**：失败发生时先对**目标设备的固定落点做一次探针写**——写得进去 ⇒ 瞬时失败，走**实例切换**；写不进去 ⇒ 持续失败，走**转只读到下次挂载**。可写设备数掉到 w 的下限以下、切换自己的预留拿不到，同样直接走只读支；连续切换次数越过 **N_switch = 3**（具名可调参数）也转只读，它是兜底的兜底，真正的判别子是探针写。**收敛靠的是切换的第一步就是一次会失败的写**：切换要先取号，取号要写独占打开成功的每一份超级块、全或无 ⇒「写不出去」这类故障最多让切换走一步（探针写先拦，侥幸过了全或无也立刻判失败）。此前写的「只读支不需要分配也不写单元所以必然收敛」说的是只读支自己，不是切换支怎么终止（第八轮反推腿 2.1）。切换要用的块在挂载准入时预留：**它是内存里的一个空间量**，每次挂载重算、崩溃即丢、**不落盘**——分配记录条目只有「已分配」与「已释放 + 释放代」两个态（D3（空间分配） 已定项 7），盘上没法表达第三个态，而落盘表达它本身就要写单元、回到「要写才能写」。预留量按 **(N_switch + 1) × 一次切换的最坏量**（一次挂载允许 N_switch 次切换，且实例表链每切一次长一行、写行要 COW 重写整条链，第 k 次比第 1 次贵；多的一份给这次挂载写行那次发布的元数据，2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）；**这个量的口径与式子 2026-09-13 由用户定案，权威正文在 D28（挂载期承诺量） 已定项 3**：一次切换的最坏量 = 实例表链重写（副本数 2 × 32768 × 片数，按设备算），**固定点重做与号 > W 的数据单元重写不另要空间**——切换是挂载内一次恢复，重建态里失败那次的分配不存在，重做走 checkpoint 的保留池；引这一段时引 D28（挂载期承诺量） 已定项 3。**预留挡的是分配失败那一路，不挡写失败那一路**——写失败由探针写与取号全或无挡（第八轮反推腿 4.1–4.3）。**「分配失败」按它发生在哪里拆三支**（此前把三种事搅成一格，撞了 `.claude/rules/fs-design.md` 那句「释放空间这个操作本身不需要申请空间」：池一满就整卷只读，而释放空间要写盘，重挂又被预留合取挡住，成了不可逆）——前台事务的用户数据分配失败 ⇒ **向调用者返回 ENOSPC**，checkpoint 照常发布、不动挂载状态；后台、可续做、非决策路径（重平衡、墓碑回收、scrub、整理）的分配失败 ⇒ **暂停那个活并记进意图**，不动挂载状态（`.claude/rules/fs-design.md` 第三格「无戒律」、D2（RAID 条带策略） 已定项 4 c「前台不停」）；**切换自己的预留拿不到** ⇒ 才转只读。根槽写失败重发时 checkpoint_txg 推进一格再发（根环区域 = `txg mod R`，D22（单元原子性怎么合成） 已定项 2，不推进就是反复重写同一个槽、把 R 个失败域用成一个）；⚠️ 由此「同一个根槽连续失败」不再是一条判据——槽位随 txg 轮转，N_switch ≤ R 时它永远够不着，留着它读起来像一道闸、实际是空的。**实例切换** = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint——所选根取被重发的那个在飞 checkpoint 所基于的根（旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old），行只落在 [这个根的实例, 新实例)，每个被照旧事务的写序实例 k 写 (k, k 最后发布的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)，其余按 D18（块里携带什么信息） 已定项 11（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方；旧实例发布过根、被照旧的事务都是它自己的时候，就是 (旧实例, 最后发布的 txg, 被重发的那个 checkpoint 里的最大事务号) 这一行），号 ≤ W 的事务照旧（开放 checkpoint 里已完成的事务全部 > W，C287（切换收养开放 checkpoint 的事务后再崩） 按这个 W 封死，2026-09-13 D23（journal 的角色与格式） 已定项 14 的注 4，2026-09-14 用户复核确认）、号 > W 的按新写序重做或向还没返回的调用者报错，固定点单元全部按新实例重写；不转只读、不等下次挂载，代价 = 一条行 + 整条实例表链重写 + 号 > W 的数据单元重写 + 一次固定点重做，频率未测。**重放下界遇回退行截断**：所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止——否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销，盘上没有任何痕迹（C124（回退行与重放下界没有会红的检查））。 两份镜像何时算「记录在」：**任一份自证校验和过即在**（2026-09-14 用户收尾弹窗定案，E142（第一个事务的干跑） 第七次跑记的 G9 收口；崩溃点重放里的检查 2026-09-14 由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件，C328（journal 两份镜像何时算「记录在」没有条款） 同日还清）。 **状态：已定。** ⚠️ **第六条口径（2026-09-13，D16（发布语义） 已定项 4）**：一次发布整体施加或整体不施加——前缀不得停在一次发布的中间；这一条待写进条款正文，引用五条口径时连它一起引。
```

**`decisions/23-journal的角色与格式.md:694`**

```text
15. **崩在记录持久之后、根槽持久之前，恢复施加什么（2026-09-13，用户定案；两条指针 2026-09-14 随 D19（块指针的结构与宽度预算） 已定项 7 各加 3 字节）：由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节（实例表单元指针照所选根），头 95 → 277 → 307，4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。** D23（journal 的角色与格式） 正文里没有已定项 15 的单独小节，条款全文就是已定项索引表第 15 行。 **状态：已定。**
```

**`decisions/23-journal的角色与格式.md:811`**

```text
**必须先全环扫描、逐条验证，再择最长合法前缀。**
```

**`decisions/23-journal的角色与格式.md:1233`**

```text
1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（五条口径第一条原样）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
```

**`decisions/23-journal的角色与格式.md:1235`**

```text
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的「48 位撑 3202 年」本来就按全卷寿命算。
```

**`decisions/16-发布语义.md:207`**

```text
**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布，第一版几何至多 R = 3 次。前置：C314（回退可以复用被抛弃的根引用的单元） 先还。
```

**`decisions/16-发布语义.md:208`**

```text
⚠️ **第一版实付 2 次，而且它是格式常量不是运行时谓词（2026-09-14 用户定案）**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 FIRST_TRANSACTION_TXG = 3（[first-txn-layout.md](../first-txn-layout.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）；「至多 R = 3 次」那句仍是几何上界，不是第一版走得到的次数。没有走三方论证，可推翻。⚠️ 主 agent 推荐的也是这一档。
```

**`decisions/16-发布语义.md:287`**

```text
**一次发布的持久顺序恒为：COW 单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）；
```

**`decisions/16-发布语义.md:313`**

```text
永久要读得懂的只有判定「这条记录不施加」要用的那几个头字段（magic、类型、自述长度、`jsn`、`checkpoint_txg`、头校验和）。**
```

**`decisions/16-发布语义.md:372`**

```text
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算（C282（环里最旧根没有定义） 要的定义） |
```

**`decisions/22-单元原子性怎么合成.md:1005`**

```text
**定案**：超级块槽：每盘恒 2 个槽，槽 i 的设备内偏移 = i × 固定结构槽距（D2（RAID 条带策略） 已定项 19），槽世代号从 1 起、每写一次 +1，下一次写的槽 = 世代号 mod 2，择槽取校验和过且世代号最大的（逐盘计：取号、发布末尾的轮换、取号全或无失败时的回卷三种超级块写都写这块盘上自证过的槽里最大的世代号 + 1；2026-09-14 用户定案取逐盘这一读法，C322（取号那一步的屏障怎么放没有条款） 三轮三方——全池最大 + 1 会让滞后盘一次跳 2、落回它择到的那一槽）。根环：`r × P × chunk` 是设备内偏移，基址 = 超级块「根环起点」字段（第一版 1 MiB），区域 r 的起点 = 基址 + r × P × chunk；区域内槽位 = `(txg div R) mod S`，无状态、只由 txg 与 (R, S) 决定，升格进已定项 2 的参数表；把公式换成 mkfs 逐区域写起点那条留到 zoned 开线前再走三方。
```

**`decisions/22-单元原子性怎么合成.md:229`**

```text
| 8 | **固定结构的放置** | **已定（2026-09-02，用户定案 + E87（固定结构的放置））：超级块每盘一份（更新走 ≥2 槽轮换）；journal 环两盘镜像（D2（RAID 条带策略）已定项 6 的 w≥2 下限对 journal 记录写同样生效）；mkfs 把第 0 代根种进全部根环区域。** 正文见 D22（单元原子性怎么合成）「已定项 8」 **状态：已定。** |
```

**`decisions/18-块里携带什么信息.md:879`**

```text
四元组 (出生树 0, 类型 4, 容器号 = 片序号从 0 起, 出生代 0)；出生树 0 与 D5（快照 / 空间记账机制） 给「全池 / 无归属」行的约定同一个数；不属于任何树、不进容器索引，由根记录直接持有物理指针（D22（单元原子性怎么合成） 已定项 7）。一片 ⌊(32768 − 136) / 88⌋ = 370 条记录（含链指针，数据行 369；136 是含 nonce / MAC / 算法类型预留位的码 3 头，D18（块里携带什么信息） 已定项 14 / 已定项 16）；mkfs 写出一片空表，记录数 1（唯一一条是「无下一片」的链指针记录）。**行怎么写**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2）：按实例代号唯一、后写覆盖，只在恢复、实例切换与回退时写，与第一个新根同一次发布——**每次可写挂载都写行**（实例 0 不写；写行那次发布是新实例的第一次发布，新实例的每一个根引用的实例表都已含 [max(所选根的实例, 1), 新实例) 的行；写行之前不推抬 F 的空发布，写行那次的元数据走切换预留，D28（挂载期承诺量） 已定项 3；2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方）：恢复在所选根指着的那一版表上给 [max(所选根的实例, 1), 新实例) 每个实例写一行——所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)，严格介于所选根的实例与新实例之间的实例写 (i, 0, 0)（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）——jsn = 实例代号 32 位 + 计数器 48 位（D23（journal 的角色与格式） 已定项 9），重放按 jsn 严格连续、断号即止 ⇒ 任何一次重放都跨不过实例边界 ⇒ **同一次恢复写出的那批行里，只有所选根那个实例的 W 可能非 0；同一次切换写出的那批行里，只有被照旧事务的写序主人的 W 可能非 0；其余恒 0**（限定在「同一次写出的那批」上：后来的恢复只给 [新的所选根实例, 新实例) 写行、不回头改旧行，所以表上允许有多行 W ≠ 0，它们来自不同次的恢复或切换）（E104（扫描重建的现行版本判定） `recovery_own_txns_global_w` 世界：把一个全局量写给范围里的每一个实例时，恢复实例自己的孤儿复活 3）——恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格，之后的记录接不上就一条都不施加、W = 0）；一条都没施加的写 (i, 所选根 txg, 0)；回退在 R_old 指着的那一版表上写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方），回退行在 kind 0 记录的 flags 字节里置 bit0 标「回退」、**不许被后来的恢复覆盖**（落在 R_old 上的那次恢复会给 [所选根的实例, 新实例) 每个实例写行，正好盖到它；C124（回退行与重放下界没有会红的检查）；⚠️ 表随根分版本，落在 R_old 上的恢复读的是 R_old 那一版表、回退行不在里面，这一句按条文没有输入，C332（回退实例两个根都读不出时回退被撤销））；实例切换（D23（journal 的角色与格式） 已定项 14 的注）同崩溃行，只是**所选根取被重发的那个在飞 checkpoint 所基于的根**——旧实例发布过根时是它最后发布的根，没有时是这次挂载恢复重放之后的根，回退那次发布里是 R_old；切换不重放，行只落在 [这个根的实例, 新实例)，每个实例一行、取下面第一条适用的：① 被重发的那次发布自己写的行原样重发（写行那次的恢复行、回退那次的回退行与中间实例行）；② 被照旧事务的写序实例 k 写 (k, k 最后发布的根的 txg（没发布过根时 0）, 被照旧的、写序属于 k 的最大事务号)；③ 所选根那个实例写 (i, 这个根的 txg, 0)；④ 其余写 (i, 0, 0)——被重发的 checkpoint 里没有事务号非 0 的记录时 ② 一条都不适用（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；每次写行 COW 重写整条链。W 是精确的前缀：事务号在实例内单调、记录按事务号顺序追加（第一版串行提交下恒成立），一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加（D23（journal 的角色与格式） 已定项 7 的注）。**已发布谓词（全局）**：i_now = 挂载根的实例，单元写序 (i, n)、诞生代号 b——i < i_now 且无行 ⇒ 已发布；有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W（在所选根里，或被重放施加；E104（扫描重建的现行版本判定） `lost_root` 世界：T_pub 按字面取在飞 txg − 1 时槽坏掉的那个 checkpoint 整个判成已发布，复活 2 + 错 1），码 2 / 3 ⇒ b ≤ T_pub（重放之后那个根之后的固定点单元一律未发布，恢复实例重写它们；重放施加进来的那几次发布的固定点单元由记录里的新根段引用（D23（journal 的角色与格式） 已定项 15），按 T_pub = 重放之后那个根的 txg 判已发布（2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；那 6 字节事务号在码 2 / 3 上不参与任何判定，I-1.8（归并后版本全序） 对码 3 的全序键是 (诞生代号, 实例代号)）；i == i_now ⇒ b ≤ 挂载根的 txg；i > i_now ⇒ 判损坏。「无行 ⇒ 已发布」只对同一时间线成立。**回收**：行可删当且仅当整轮清扫对**池中每一个落点**都得出了判定（读成功并按行判、已抹头、已被覆盖）且其中没有该实例的未发布单元、那次清扫里没有任何一次读失败（读不到不等于不存在，扇区维），且全部设备在线（设备维），且根环里没有该实例发布的根——根环每个槽都读成功才算「没有」，读不出的槽按「可能有该实例的根」算（2026-09-14 用户定案，C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方：一个暂时读不出的根槽能让行被删，槽恢复后被抛弃的根回到回退候选集，而它引用的单元已被清扫抹头；代价是一个根槽持续读不出时行永远删不掉，C335（根槽持续读不出时实例表只增不减））。量词取「池中每一个」不取「该实例写过的每一个」：已抹头与已被覆盖的落点说不出它原来属于谁，按后者写的话要枚举的集合与要销毁的信息是同一份（被抛弃的根留在环里时，它的行是回退候选判据的输入，E104（扫描重建的现行版本判定）：不看根环则被抛弃的根整批回到候选集）；zoned 第一版不回收、链长无上界（C121（zoned 上实例表链长无上界））。**表不可读时**：只读挂载；扫描重建停在级 1——表没了就一条行都没有，全部旧实例的全部单元落进「无行 ⇒ 已发布」，被回退抛弃的整段时间线也在内，所以不是记歧义。实例表单元自己不走 I-1.2（块头写序已发布） 的谓词：它的现行版本由根记录持有的物理指针唯一确定，指针指不到或校验不过就是表不可读。**作废一个 checkpoint 必须换实例代号**：当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布——这是失败表能封闭的依据（D23（journal 的角色与格式） 已定项 14 的注）。**实例代号**住超级块（E100（超级块的三段几何） 段四那 4 字节）。可写挂载的顺序：先判这次能不能可写（可写设备数够 w 的下限 ∧ **独占打开池中过半的设备**——任意两个过半集合相交，每设备的独占打开才成为池级互斥；这条与它的代价（4 盘池只剩 2 块可写时只能只读）是 D2（RAID 条带策略） 已定项 13 定的，不是本项 ∧ 实例表可读 ∧ 实例切换的预留拿得到），再取新代号 = max(**这次挂载独占打开成功的那个集合**里各超级块的代号（每块盘两槽里全部自证过的槽——校验和过且 fsid 与本池相同；2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）, 根环里全部根记录的实例代号) + 1 并写进**那个集合**里的每一份超级块、**全或无**（写超级块与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**（旧代号 = 取号之前这个集合里全部自证过的槽中最大的实例代号；回卷写发生在任何单元之前），回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」由 I-7.7（超级块实例代号不低于根环） 的第 ② 句判成立（较大的号没有任何根、记录、单元带着），不另开例外，下一次挂载按 max + 1 修复；取号之后那道屏障（D23（journal 的角色与格式） 已定项 16）在任一块盘上报错同样判全或无失败，不许只重发屏障就继续；取号的写入集合与 max 的取值集合取同一个——写「全部可见」而只独占一部分时，集合外那些盘写不写得进去不由这个挂载说了算，两个挂载能互相逼成只读。**过半是准入门槛，不是写入范围**：写入集合取独占打开成功的全部设备，只写过半会把集合外的健康盘落下、它们的代号从此是旧的，与真正错过取号的盘分不开。**「可见」= 独占打开成功且超级块读得通**；I-7.7（超级块实例代号不低于根环） 的第 ② 句限定到同一个集合），之后才动任何单元；只读挂载不取号；从 1 起、0 无效。**前提**：同一时刻只有一个可写挂载——由过半独占打开保证；多主机共享存储上两个可写挂载会算出同一个代号、谓词分不出它们，随 D9（加密） 已定项 8 那一类宣布不防。取号那一刻两路见证不完整可见时不额外拦截，与整卷回滚同格；I-7.6（根环区域落在互不相同的盘上） 只在 devs ≥ R 时让根环成为独立见证，第一版 (R = 3, devs = 2) 不成立。错过取号的盘分两类：**打开过又掉了、且缺号期间池里确实发布过**的，回归时整盘作废、只读到重同步完成；**从未被这次挂载独占打开的**，或缺号期间池里一次发布都没有的，不作废——它没有分叉的物理可能（第八轮反推腿 3.1 / 7.2）；第一版 2 盘、掉一块只能只读挂载 ⇒ 分叉盘回归不可达；D2（RAID 条带策略） 已定项 3 / 4 允许在线加盘 ⇒ 第三块盘一加就可达，C120（分叉盘回归的判定与重同步）在允许加第三块盘之前到期，与过半规则同时。
```

**`decisions/08-核心索引结构.md:378`**

```text
**key 与 inode 号**：key = inode 号 8 字节，唯一性范围 (树 ID, inode)。**已发布的 inode 号单调不复用**（D6（快照实现模型） 判据 9 的戒律，2026-09-06 随判据 9 一同收窄射程、用户定案）：
```

**`decisions/08-核心索引结构.md:476`**

```text
**② 树 ID 水位住根记录，取根环里全部根记录该字段的 max**（2026-09-06 用户定案；同日先定「住超级块取 max + 区段预留」，当日重比三条臂后重开，见变更史）。逐字形态： `新水位 = max(根环里全部根记录的该字段, 本次 checkpoint 发出的最高树 ID + 1)`，**每次发布随根记录重写**。 ⚠️ **「累计」这两个字是承重的**：读成「本 checkpoint 发出的最高号」（不累计）时，树 ID 会随根轮出环被重发，而**那时被重发的号属于已发布的树**，D6（快照实现模型） 判据 9 正面命中 ⇒ 整条定案不成立。**依据**：**㈠ 回退不打穿它**——D23（journal 的角色与格式） 已定项 14 自己就在用这个装置给别的量取值：「第一个新根的 checkpoint_txg = **根环里全部根记录 txg 的最大值 + 1**」，是「全部」不是「候选」（候选集才排除被抛弃时间线的根）；同条逐字「**回退深度 ≤ 根环深度**」⇒ 能被回退跨过的根都还在环里。**㈡ 写时机天生成立**：根槽 FUA 就是发布本身，checkpoint N 里发布的树，其 ID 必 ≤ 根 N 里的水位——不需要任何预留机制。**㈢ 见证是根环 R = 3 份**，且 I-7.7（超级块实例代号不低于根环） 有现成的同形不变量可抄。**㈣ 每次发布零代价**：根记录本来就每次发布都写，字段表从 186 变 **194** 字节、512 槽余量 326 → 318（2026-09-06 的数；D22（单元原子性怎么合成） 已定项 7 今天是 242 / 余量 270）。 **被否的两条与它们的射程**：**乙 记账臂**——D23（journal 的角色与格式） 已定项 14 逐字「全部记账统计量的现行值从 R_old 那棵账重新载入」⇒ 回退后退回去重发。**「挂载时从树表扫最大值 + 1」（btrfs 那条路）**——被抛弃 checkpoint 里分配过的树 ID 不在已发布的树表里，不能单用。⚠️ **此前把根记录臂与记账臂一起用那句话否掉，是引用射程越位**：那条引用一个字也没说根记录。**这是 2026-09-06 当日重开这条定案的直接理由**，不是「丙更好」。 **丙-2 唯一盖不住的那一格，判为不是错误**：某个 checkpoint 分配了树 ID 42、写出带 42 的单元头，根没写成就崩了 ⇒ 水位随那个根一起丢 ⇒ 下次挂载重发 42，而盘上留着带 42 的孤儿单元。**判它不构成错误的依据（三方论证，主 agent 逐条现查）**：① D18（块里携带什么信息） 已定项 11 逐字「**作废一个 checkpoint 必须换实例代号**」⇒ 孤儿与活版的实例代号必不相等；② I-1.2（块头写序已发布） 逐字「**判不成已发布者必是未发布的撕裂事务垃圾或被抛弃时间线的残留**」；③ 扫描重建的两道闸（已发布谓词、择新键）与孤儿回收条件（D3（空间分配）「孤儿（没有条目）要求头里的写序实例 < 当前实例 ∧ 按实例表的已发布谓词判未发布」）**都不看树 ID**；④ **决定性的一条**：孤儿与活版**共用五元组本来就是常态**——崩溃前后同一个逻辑块两次写出的五元组逐字段相同（同树、同 inode、对象出生代在创建时固定、锚点偏移是文件内偏移），正是 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P1 给三类身份段统一加 10 字节写序要解决的东西⇒ 丙-2 只是把一个**已被专门解决**的碰撞从既有树扩到新建树；⑤ D6（快照实现模型） 判据 9 的理由句动词是「盘上遗留的旧 key 在新快照里**变成可见**」，而可见性作用在**树里的 key** 上，孤儿不在任何树里。⚠️ **由此要动 D6（快照实现模型） 判据 9 的射程**：它的字面「快照标识单调不回收」在孤儿场景下确实被违反，按它自己的理由句收窄成「**已发布**的快照标识单调不回收」（2026-09-06 用户定案，同一次）。 ⚠️ **inode 号水位仍要另给答案**：D5（快照 / 空间记账机制） 已定项 4 第 12 项逐字「**带**（只为可写头维护）」⇒ 每个可写头一个标量、条数随头数长，而根记录也是定长槽 ⇒ 丙-2 同样搬不过去。落点 C143（inode 号水位在回退后会退回去重发）。
```

**`decisions/05-快照-空间记账机制.md:364`**

```text
| 14 | 最近一次根销毁的代号 | 墓碑回收门的下限（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P5：门 = max(死亡代号, 这一项)） | 不带 | 不带 |
```

**`decisions/05-快照-空间记账机制.md:371`**

```text
合并规则取最大值（见已定项 3）。只为可写头维护，随每次发布重写（已定项 2 的 K 代点删要求每行每发布重写，第 8 / 9 项同），只读树（快照）没有这一行。
```

**`decisions/06-快照实现模型.md:237`**

```text
- 批前预留要的「这个头的 livelist 有多少条」不按前缀范围数，要一个每头计数器；它只能住头的树表条目预留仅剩的 8 字节，
```

**`decisions/09-加密.md:536`**

```text
| **对象出生代（inode generation）** | 按五元组 / AAD 的 day-1 契约保留（D18（块里携带什么信息） 已定项 3 / 已定项 11）。inode 号单调不复用（D8（核心索引结构） 已定项 6）之后，「复用后旧文件与新文件的 `(树, 对象, 偏移)` 完全相同」是历史理由；它退化为深度防御：若实现 bug 意外复用了号，这一段仍能在 AAD / nonce 两层把新旧对象分开，是 I-6.1（nonce 不重用） 失效时（崩溃窗口 / nonce 水位回滚）的第二道闸 | 密文侧 |
```

**`checks-owed.md:109`**

```text
| C103 | 容器索引无落点 | D18（块里携带什么信息） 已定项 11 要求打包记录单元由一棵容器索引指向：它是打包单元物理指针（与 MAC）的唯一持有者，别的树按身份引用容器；key = (出生树 ID, 打包记录类型, 容器号, 容器出生代) 四段全进，容器号排在出生代之前，使 (树, 类型) 下最大容器号 O(log N) 可查——容器号分配器的下一个号 = 最大号 + 1，派生态、扫描可重建；计数器回退不破身份（出生代不同）。这棵树住哪、key 编码、它自己的更新代价、它独占的 MAC 重建时怎么重算，全仓无落点。第二轮正推腿：容器号分配器与 inode 号分配器是同一类从未处理过的问题。2026-09-05 起本条只覆盖打包记录类型 1（墓碑）：类型 2 的容器索引就是 inode 树的内部节点、容器号 = 触发分裂的新记录 inode 号（D8（核心索引结构） 已定项 6），这半支收口，检查列的「最大容器号一致」同样只对类型 1 | 容器索引落成分项之后：黄金镜像里造一个容器索引全丢的样本，扫描重建必须从码 3 单元头恢复出全部条目且各 (树, 类型) 的最大容器号一致；把容器号从头里删掉必须让恢复失败；伪造一条指向错误落点的条目必须被载荷校验和或 MAC 抓住 ⚠️ **两条现查过的事实（2026-09-06，主 agent 逐字核 D18（块里携带什么信息） 已定项 11 的码 3 类身份段表）**：① **容器的四个命名字段在码 3 头里全在**——出生树 ID（偏移 43）、打包记录类型（51）、容器号（53）、容器出生代（61）⇒ **扫描重建读得出容器身份**，不缺字段；② **打包记录类型是独立的 2 字节字段**，不是折在单元类型标签那 1 字节里 ⇒ 任何把「类型」塞进 1 字节标签的收口候选，先要回答那 2 字节的值域怎么办。 ⚠️ **2026-09-06 起它挡着的东西多了三样**：D2（RAID 条带策略） 已定项 12 定了条带成员表走容器索引 ⇒ C127（钉住状态在分配记录里没有落点） 的修法 ②、C128（清扫准入对 parity 格两支都判不出） 的修法、以及 **D9（加密） 已定项 9 那条让渡区 A 的签发判据第 ⑦ 条合取**（「不是钉住的格、也不属于任何含活成员的条带」）**全部要先走一次容器索引点查**。⇒ 它从一笔「日后要还」的账变成**三处的直接前置**。 ⚠️ **2026-09-06 试过一次收口，三腿判定不了案**：候选是「容器索引就是中央映射」（容器按对象进 D19（块指针的结构与宽度预算） 已定项 5 那层映射）。**栽在 value 装不下**——D18（块里携带什么信息） 已定项 11 逐字说容器索引是「打包单元物理指针（**与 MAC**）的唯一持有者」，而 MAC 128 位 + nonce 96 位住在指针头部 31 里，中央映射的 value（位置条目 14）只有密文校验和 4；D9（加密） 已定项 4 逐字「nonce 每 extent 存，不现算」⇒ 加密后解不开。**自举也会重演**，收敛在「中央映射自己的节点」那一层，而那一层零条款。逐条经过与剩下的四条候选路写在 `records/2026-09-06-C103容器索引收口轮.md`。 | 先在 D3（空间分配） 或 D8（核心索引结构） 开一条分项 | 2026-09-04 D18（块里携带什么信息） 已定项 11 三轮三方论证（第一轮立账，第二轮正推腿场景二补三句，第二轮反推腿 9.3 补 MAC 那半） |
```

**`checks-owed.md:303`**

```text
| C331 | 择根倒挂压过已确认的写 | 较旧实例的全部根暂时读不出时，新实例的第一次发布从更旧的根往上数 txg；那些根以后又读得出，txg 更高的它们被择新选中（D22（单元原子性怎么合成） 第 487 行按 checkpoint_txg 择新、平局才看实例代号），新实例已确认的写被压过去（第一轮攻方腿 6 个故障；C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 的写回管的是谓词，挡不住择根）。D23（journal 的角色与格式） 已定项 14 注 3「新实例从前缀末 + 1 接着写」还会让新实例的第一条记录落在读不出那条根的记录槽上，把它唯一的记录见证盖掉（今天的规则不拿那条记录当见证，所以今天看不出后果） | 多次挂载的崩溃点重放里注入「较新实例的全部根暂时读不出、之后又读得出」，逐状态判：已确认返回的写，在之后每一次挂载选的根上都读得回；判别力自证：今天的择新规则在这个注入下必须红 | 修法（计数器从全环最大 + 1 起、超级块带 txg、或记录扫描水位）要另过三方；多次挂载的录制流 | 2026-09-14 C329（写行那次发布之前推抬 F 的空发布没有检查）、C330（中间实例那一行的 T_pub 取所选根的 txg） 第一轮攻方腿（`research/prompts/c329-c330-r1-main-verification.md` 第 47 行）、第二轮攻方腿（`c329-c330-r2-main-verification.md` 第 44 行）、第三轮辩方腿复核第二轮出局判决的那一格 |
```

## 历史版本

（这份报告是一轮论证的原样产物，不改写；没有历史。）
