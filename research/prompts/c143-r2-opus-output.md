# C143（inode 号水位在回退后会退回去重发） 第二轮：攻方腿（Opus）报告

立场：反推 / 攻。假设主 agent 的倾向（丙，可能带 CJ / CJ2）是错的，也假设 AJ / AJ2 与甲、乙各有死角，换第一轮没攻过的面造可达历史去打穿它们。

读过的材料：任务书 `research/prompts/c143-r2-opus.md`；背景材料 `research/prompts/_c143-r2-background.md` 全文（正文、小节清单、kb 原文附录）；第一轮判决 `research/prompts/c143-r1-main-verification.md`；第一轮攻方腿报告 `research/prompts/c143-r1-opus-output.md` 与它的模型 `research/prompts/c143-r1-opus-model/c143_model.py`（只读，没有改，也没有 import）。附录不够的地方现查了 kb 原文；下文引 kb 一律写 kb 文件名加那份文件自己的行号（`grep -n` / `sed -n` 现查得来，不从背景材料里数），当论证支点的那几行整行抄在报告末尾的附录（`sed` 机械抽取）。没有读这一轮别的腿的产出（`research/prompts/c143-r2-*-output.md`），没有改仓里任何别的文件。

模型：`research/prompts/c143-r2-opus-model/c143r2_model.py`（只用 Python 3.12 标准库，确定性，没有随机源），原始输出 `research/prompts/c143-r2-opus-model/c143r2_model.out`。复跑命令、跑的经过与 sha256 在第八节。

## 一、模型实现了什么、照的是哪一行

第一轮的模型里 journal 记录一直都在；这一轮的攻击面全在 journal 上，所以模型重写了一份，把环、两份副本、覆写、读故障与单元复用都建进来。下表只写模型里怎么做和照的是 kb 哪一行，不转述条款。

| 模型里怎么做 | 照的行 |
|---|---|
| 根环 3 个区域，槽位 `(txg mod 3, (txg div 3) mod S)`，S = 8（第一版取值；s3 另用 S = 4）；区域归属 0 / 1 / 0 | `22-单元原子性怎么合成.md:1005`、`:1013`；`first-txn-layout.md:43` |
| mkfs 在三个区域的槽 0 各放一份 txg 0 的根 | `22-单元原子性怎么合成.md:229` |
| 每次发布先写单元、再写记录、最后写根槽；「已发布」= 根槽写成；「已确认」= 根槽写成、且那一刻本实例写成的根已覆盖两块盘 | `16-发布语义.md:287`、`:207` |
| 严格暖机：新实例（挂载、切换、回退）先推空发布，直到本实例的根覆盖两块盘，用户事务排在后面；宽读法（用户事务紧跟新实例的第一个根）只当对照 | `16-发布语义.md:207`；`22-单元原子性怎么合成.md:1014` |
| journal 环 J 个记录槽，两块盘各一份；计数器为 c 的记录落在槽 `(c − 1) mod J`；每次发布写 nrec 条记录（默认 1，空发布也写 1 条）；每条记录带 (实例代号, 计数器)、checkpoint_txg，和新根段指到的那一版状态（树表 + 记账） | `22-单元原子性怎么合成.md:229`；`23-journal的角色与格式.md:697`、`:698`、`:694` |
| 一条记录两份副本任一份读得出就算在 | `23-journal的角色与格式.md:691`（已定项 14 索引行末尾「两份镜像何时算『记录在』」那一句） |
| 普通恢复：先扫全环；选 (txg, 实例) 最大的可读根；从它覆盖的最后一条记录往后，只施加同一实例、计数器连续、整次发布齐全、指到的单元还在（点名单元验得过）的记录；新实例从前缀末 + 1 接着写；新实例代号 = max(超级块, 根环) + 1；写行 | `23-journal的角色与格式.md:811`、`:1233`、`:1235`、`:691`；`18-块里携带什么信息.md:879` |
| 管理员回退：候选集按最新可读根指着的那一版实例表判；第一个新根 txg = 可读根的最大 txg + 1；水位按臂取；写回退行与中间实例行；回退的第一条记录落在「R_old 覆盖的最后一条记录 + 1」（字面读法，记 P1）。另一种读法 P2 = 从 R_old 往后整条可读链的末尾 + 1，只在 s4 当对照 | `23-journal的角色与格式.md:1206`、`:1235` |
| 回退那次发布崩在记录持久之后、根之前：回退不生效，下一次普通挂载与没发起回退时相同 | `23-journal的角色与格式.md:1206` |
| 单元复用，取对 AJ 最坏的读法：没写成根的发布（孤儿）的单元，下一个实例一分配就被复用；回退之后，被抛弃的根一离开根环、或者在回退那一刻读不出，它引用的单元就被复用（影子账只查「环里每一个可读根」）；同一条时间线上已离开根环、且比环里最旧有效根更旧的根，它的单元被复用 | `23-journal的角色与格式.md:1206`（影子账那一句）；`16-发布语义.md:372` |
| 克隆：新头的树 ID 取树 ID 水位，新头的水位抄 origin 的运行时水位；可以销毁一个没有快照的可写头 | `08-核心索引结构.md:386`；`06-快照实现模型.md:240` |

**臂**（只在「回退 / 挂载时 inode 号水位与第一个新根 txg 怎么取」上不同）：

| 臂 | 做什么 | 判据 |
|---|---|---|
| A（甲 / 乙） | 水位 = max(R_old 那一格, 每个可读根里该头那一格)。甲读树表条目、乙读记账行，两者在每个根里随每次发布重写，模型里是同一个计算（第一轮同一口径） | (树, 号) |
| AJ | A，再取 journal 里 txg > R_old、读得出、新根段指到的单元还在的每条记录里该头的水位 | (树, 号) |
| AJ2 | AJ 推广到每次普通挂载：新实例的水位再取全部读得出、单元还在的记录里该头的最大值 | (树, 号) |
| C（丙） | 水位从 R_old 重载；第一个新根 txg = 可读根最大 + 1 | (树, 号, 出生代) |
| CJ | C，第一个新根 txg = max(可读根的 txg, 读得出的记录的 checkpoint_txg) + 1 | (树, 号, 出生代) |
| CJ2 | CJ 推广到每次普通挂载：新实例的 txg 从同一个最大值往上数 | (树, 号, 出生代) |
| G（我这一轮提的线索，只在我的模型上量过，被攻过零轮） | 甲 / 乙的一个变形：根记录与记录头各带一个全池「inode 号水位」（全部可写头水位的最大值，只增不减），回退与每次挂载把每个头的水位抬到读得出的根与记录里它的最大值 | (树, 号) |

切换一律按 keep 读法（在飞 checkpoint 原样重发，回退时算出的最大值留着）；第一轮 H2 的 reload 读法这一轮没重跑。

**U1 的判定**与第一轮相同：一个身份（A / AJ / AJ2 / G 按 (树, 号)，C / CJ / CJ2 按 (树, 号, 出生代)）先后被两个不同对象带进写成了的根里，且至少一个是在某次回退之后建的，记一次违例；同时记前一个对象当时是否已确认（`violations` 里的 `True`）。s9 是例外，它不要求有回退（见第二节 2.8）。

**故障计数**（这一轮的口径）：整块盘在一次读扫描里读不出记 1；一个根槽读不出记 1；一条记录在一块盘上的副本读不出记 1（输出里的 `fs`，按槽计）；同一块盘上连续的一段记录槽只记 1（`fr`，按区间计，给「一段坏块」这种一次性的故障用）；一次根槽写失败记 1；同一个物理目标（盘、根槽、某块盘上的某个记录槽）在历史里的几次扫描都读不出，只记一次；崩溃不算故障。脚本记号：`rb(k,u,sw)` = 回退到第 k 新的候选，`u` = 这次读扫描的故障（`d0` 盘 0 整块读不出；`rN` 最新 N 个根槽；`jN` 最新 N 条记录在还剩的那块盘上的副本，两块盘都在时两份一起；`rtX` / `jtX` 指定 txg 的根 / 记录），`sw` = 回退那次根槽写失败；`mount(u)` 普通挂载；`rbx(u)` 回退那次发布崩在记录之后；`rbm(u,sw)` 回退到 `mark` 标下的那个根；`p(n)` / `f(n)` 一次发布、建 n 个对象（`f` 为根槽写失败）；`u(n)` n 次用户发布；`clone` / `ucx(n)` / `destroy` 建克隆头、在克隆头里建对象、销毁克隆头。每条脚本前面都有第一次挂载与第一个事务（txg 1–3），不写出来。

## 二、U1：各形最短的「已发布身份被重发」历史

### 2.0 总表

格子里是这一族里最少的故障数：按槽计 `fs`，与按区间计 `fr` 不同时写成 `fs / fr`；「—」= 这一族里没有违例；「没跑」= 这一族里那条臂与左边某条臂是同一个计算（AJ2 / CJ2 只在普通挂载时与 AJ / CJ 不同，s1–s4 的回退之后没有普通挂载）。每格的出处是输出里那一族的 `arm=` 行，整行抄在 2.1–2.6。

| 族（输出节） | A 甲 / 乙 | AJ | AJ2 | G（线索） | C 丙 | CJ | CJ2 |
|---|---|---|---|---|---|---|---|
| s1 被抛弃时间线里销毁了一个可写头，回退时藏根 / 记录 | 1 | **2** | 没跑 | 3 / 2 | — | — | 没跑 |
| s4 回退那次发布崩在记录之后，再重做（字面读法 P1，孤儿单元复用） | 1 | **1** | 没跑 | — | — | — | 没跑 |
| s3 R_old 靠根槽写失败多留几圈，journal 环在窗口里转过一圈（S = 4，小环） | 1 | 随 J 涨，见 2.3 | 没跑 | — | — | — | 没跑 |
| s2 回退那一刻最新的根与记录读不出 | 1 | 2 | 没跑 | 2 | 2 | **5 / 3** | 没跑 |
| s5 回退之后崩溃、重挂时读故障（含回退之后切换一次的变体） | 2 | 2 | 2 | 2 | 2 | 2 | **5 / 3** |
| s6 第二次回退时读故障 | 1 | 2 | 2 | 2 | 2 | 5 / 3 | 5 / 3 |
| s8 几何表，归属 0 / 1 / 0、严格暖机（解析枚举，不是模拟） | 1 | 2 | — | — | 2 | 5 / 3 | — |

跨臂的四个数（每一族都有这一行，下面整行抄）：

- `hit=CJ not_hit=AJ` 在 s1、s2、s4（八个变体）、s5、s6、s9 全是 0；`hit=CJ2 not_hit=AJ2` 在 s5、s6、s9 全是 0；`hit=C not_hit=A` 全是 0。⇒ 在我试过的全部形状里，CJ / CJ2 中的每一条脚本 AJ / AJ2 也中，丙中的每一条甲 / 乙也中。
- 反过来 `hit=AJ not_hit=CJ`：s1 1824 条、s2 48418 条、s4 最多 324 条、s5 3969 条、s6 3710 条。
- ⚠️ `hit=CJ not_hit=C` 不是 0（s2 1158 条、s6 23 条；CJ2 对 C 在 s5 是 75 条）：CJ 把第一个新根往后推了几格，新实例第一个用户对象的出生代换了一个落点，有时正好落在被抛弃对象的出生代上，而丙原来的落点没落上。⇒ CJ 不是逐条脚本都比丙强，它强在要的最少故障数（2 → 5 / 3）。

### 2.1 s1：被抛弃时间线里销毁了可写头——AJ 的载体有界，2 个故障就退回

脚本空间：0–2 次空发布；建克隆头 19；0–2 次空发布；标 R_old；在 19 里建 1 个对象；销毁 19；0–3 次空发布；回退到 R_old；在复活的 19 里建 1 个对象。回退那次读扫描的故障：盘 0 整块 / 盘 1 整块 / 都在，再加上 R_old 之后第 1–4 次发布的根里任取至多 2 个、同样的记录里任取至多 2 个（藏在还剩的那块盘上的副本；两块盘都在时两份一起藏）。`c143r2_model.out` 第 9–13、15、17、21、23 行（整行抄）：

```
name=s1_destroyed_head arm=A runs=8712 violating=5016 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=5016 by_fs=0:0/198;1:510/864;2:1158/1914;3:1587/2466;4:1059/1722;5:315/810;6:387/738
name=s1_destroyed_head arm=AJ runs=8712 violating=1824 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=1824 by_fs=0:0/198;1:0/864;2:294/1914;3:675/2466;4:465/1722;5:183/810;6:207/738
name=s1_destroyed_head arm=G runs=8712 violating=66 min_fs=3 min_fr=2 viol_mount_on_abandoned=0 viol_other=66 by_fs=0:0/198;1:0/864;2:0/1914;3:33/2466;4:24/1722;5:0/810;6:9/738
name=s1_destroyed_head arm=C runs=8712 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/198;1:0/864;2:0/1914;3:0/2466;4:0/1722;5:0/810;6:0/738
name=s1_destroyed_head arm=CJ runs=8712 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/198;1:0/864;2:0/1914;3:0/2466;4:0/1722;5:0/810;6:0/738
name=s1_destroyed_head hit=CJ not_hit=AJ scripts=0
name=s1_destroyed_head hit=AJ not_hit=CJ scripts=1824
name=s1_destroyed_head hit=CJ not_hit=G scripts=0
name=s1_destroyed_head hit=G not_hit=AJ scripts=0
```

AJ 最短的一段（第 43–57 行，整行抄）：

```
example name=s1_destroyed_head arm=AJ min_by=fs fs=2 script=[clone mark ucx(1) destroy rbm(d0+jt1,0) ucx(1)]
  violations=[((19, 2), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    clone of 12 -> tree 19 (wm 2)
    txg4 i1 c4 root->disk1 wm=12:2,19:2
    txg5 i1 c5 root->disk0 creates (19,2,bg5) wm=12:2,19:3
    destroy head 19
    txg6 i1 c6 root->disk0 wm=12:2
    ROLLBACK u=d0+jt5: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; records from c5; wm=12:2,19:2
    txg5 i2 c5 root->disk0 wm=12:2,19:2
    txg6 i2 c6 root->disk0 wm=12:2,19:2
    txg7 i2 c7 root->disk1 wm=12:2,19:2
    txg8 i2 c8 root->disk0 creates (19,2,bg8) wm=12:2,19:3
```

逐步：txg 4 建克隆头 19（根落盘 1），它就是 R_old；txg 5 在 19 里建 inode 2、水位变 3（根落盘 0），那时本实例早已覆盖两块盘，fsync 返回过（`True`）；txg 6 销毁 19（盘 0）。回退扫描时盘 0 整块读不出（txg 5、6 的根都在盘 0，记录在盘 0 的那份也读不出），盘 1 上 txg 5 那条记录的副本也读不出：两个故障。19 的水位 3 只由 txg 5 的根和 txg 5 的记录带着，txg 6 起的根与记录里已经没有 19。AJ 取到的最大值因此是 R_old 那一格的 2，复活的 19 又发出 inode 2。丙跑同一段：可读根最大 txg 4，第一个新根 txg 5，严格暖机推 5、6、7，新对象落在 txg 8，(19, 2, 8) ≠ (19, 2, 5)；CJ 更远：盘 1 上 txg 6 那条记录读得出，新根从 7 起。

**为什么这是结构性的**（推理，s1 / s2 的数与它相符）：甲 / 乙 / AJ 问的是「每个头」到现在为止发过的最大号。头活着时，它之后的每个根、每条记录都带着不小于那个数的水位，要打穿 AJ 就得把 R_old 之后到最新的那一整段都藏掉。头一销毁，这条传播就断了，要藏的只剩「R_old 之后、销毁之前」那几次发布；这一段有界、不再长，而且会被轮转掉（第一轮 H3）、被覆写掉（2.2、2.3）。丙 / CJ 问的是全池的 txg，之后的任何根、任何记录都带着不小于它的值，头销毁不影响。G（全池号水位）同理不受头销毁影响：它在 s1 里的 66 条违例全是「销毁就是最后一次发布、之后的根与记录也全被藏」，与 s2 里的「最新的根与记录读不出」是同一种形状，`hit=G not_hit=AJ` 为 0。

这一形可达的条件：19 没有自己的快照（「还有自己活快照的可写头不许销毁」，`06-快照实现模型.md:240`，挡不住它）；R_old 里有 19（克隆建在 R_old 之前或就是 R_old）。

### 2.2 s4：回退那次发布崩了、再重做——AJ 的见证被回退自己的记录盖掉，1 个故障

机制（条款指路 + 推理，模型坐实）：回退与它的第一个新根同一次发布，崩在根之前就当没发起过（`23-journal的角色与格式.md:1206`「崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同」）。可是那次发布写出的记录留在环里，而且落点是「前缀末 + 1」（`23-journal的角色与格式.md:1235`）。回退不施加 R_old 之后的任何记录，前缀按字面停在 R_old 覆盖的最后一条记录（P1），于是回退的第一条记录正好盖在被抛弃时间线里 R_old 之后的第一条记录上。那次回退按 AJ 算出的最大水位写进了它自己新根段指到的单元，可那是一次没写成根的发布，单元是孤儿（`03-空间分配.md:180` 的孤儿回收条件：写序实例 < 当前实例 ∧ 按实例表判未发布），下一次挂载的新实例一分配就能复用。重做回退时，AJ 要的见证两处都没了：原来那条记录被盖掉，替身记录指到的单元被复用。剩下的只有根环，而根环里带那个水位的根藏一个就够。

脚本空间：s1 的前缀（克隆、标 R_old、在 19 里建 1 个对象、销毁 19、0–2 次空发布），接回退那次发布崩在记录之后（`rbx`）、一次普通挂载、0–2 次空发布、重做回退、在 19 里建 1 个对象。故障只加在重做那次扫描上（`redo_only`），或者两次扫描都有（`attempt_and_redo`，同一个物理目标只记一次）。P1、单元复用、故障只在重做时，`c143r2_model.out` 第 270–274、276、278 行（整行抄）：

```
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=A runs=648 violating=324 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=324 by_fs=0:0/72;1:162/306;2:99/171;3:63/99
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=AJ runs=648 violating=324 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=324 by_fs=0:0/72;1:162/306;2:99/171;3:63/99
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=G runs=648 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/72;1:0/306;2:0/171;3:0/99
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=C runs=648 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/72;1:0/306;2:0/171;3:0/99
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=CJ runs=648 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/72;1:0/306;2:0/171;3:0/99
name=s4_crash_redo rbp=rold reuse=True faults=redo_only hit=CJ not_hit=AJ scripts=0
name=s4_crash_redo rbp=rold reuse=True faults=redo_only hit=AJ not_hit=CJ scripts=324
```

八个变体里 AJ 那一行（第 271、329、347、366、385、444、462、481 行，整行抄；C、CJ、G 在八个变体里都是 `violating=0`，见各变体的 `arm=C` / `arm=CJ` / `arm=G` 行）：

```
name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=AJ runs=648 violating=324 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=324 by_fs=0:0/72;1:162/306;2:99/171;3:63/99
name=s4_crash_redo rbp=rold reuse=True faults=attempt_and_redo arm=AJ runs=648 violating=324 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=324 by_fs=0:0/36;1:81/153;2:117/249;3:126/198;5:0/12
name=s4_crash_redo rbp=rold reuse=False faults=redo_only arm=AJ runs=648 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/72;1:0/306;2:0/171;3:0/99
name=s4_crash_redo rbp=rold reuse=False faults=attempt_and_redo arm=AJ runs=648 violating=108 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=108 by_fs=0:0/36;1:0/153;2:45/249;3:63/198;5:0/12
name=s4_crash_redo rbp=scan reuse=True faults=redo_only arm=AJ runs=648 violating=108 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=108 by_fs=0:0/36;1:0/153;2:45/261;3:63/198
name=s4_crash_redo rbp=scan reuse=True faults=attempt_and_redo arm=AJ runs=648 violating=324 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=324 by_fs=0:0/36;1:81/153;2:117/261;3:126/198
name=s4_crash_redo rbp=scan reuse=False faults=redo_only arm=AJ runs=648 violating=108 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=108 by_fs=0:0/36;1:0/153;2:45/261;3:63/198
name=s4_crash_redo rbp=scan reuse=False faults=attempt_and_redo arm=AJ runs=648 violating=108 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=108 by_fs=0:0/36;1:0/153;2:45/261;3:63/198
```

| 前缀末读法 | 孤儿单元复用 | 故障只在重做时 | 两次扫描都有 |
|---|---|---|---|
| P1（字面：R_old 覆盖的最后一条 + 1） | 是（最坏） | **1** | **1** |
| P1 | 否 | —（0 条） | 2 |
| P2（R_old 往后整条可读链的末尾 + 1） | 是 | 2（与 s1 同形，崩溃没有添东西） | **1** |
| P2 | 否 | 2 | 2 |

P2 那一格的 1：回退那次扫描时 txg 5 的根槽读不出，影子账只查「环里每一个可读根的账」（`23-journal的角色与格式.md:1206`），读不出的那个被抛弃根引用的单元当场就能被回退那次发布复用；重做时那条记录还在（P2 没盖它），可它指到的单元已经没了，而根槽还读不出（同一个目标，只记一次）。⇒ 两种前缀末读法下 AJ 都有 1 个故障的历史；只有「P1 且孤儿单元一次都没被复用」这一格它挺住。

P1、复用、故障只在重做时，AJ 那一段（第 307–326 行，整行抄）：

```
example name=s4_crash_redo rbp=rold reuse=True faults=redo_only arm=AJ min_by=fs fs=1 script=[clone mark ucx(1) destroy rbx(none) mount(none) rbm(d0+jt1,0) ucx(1)]
  violations=[((19, 2), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    clone of 12 -> tree 19 (wm 2)
    txg4 i1 c4 root->disk1 wm=12:2,19:2
    txg5 i1 c5 root->disk0 creates (19,2,bg5) wm=12:2,19:3
    destroy head 19
    txg6 i1 c6 root->disk0 wm=12:2
    ROLLBACK ATTEMPT u=none: R_old=txg4 i1; readable ring max txg6; first new root txg7 i2; records from c5; wm=12:2,19:3
    txg7 i2: record c5 durable, CRASH before root
    MOUNT u=none: selects txg6 i1, replays to txg6 (prefix end c6), new i3 from txg7, wm=12:2
    txg7 i3 c7 root->disk1 wm=12:2
    txg8 i3 c8 root->disk0 wm=12:2
    ROLLBACK u=d0+jt5: R_old=txg4 i1; readable ring max txg7; first new root txg8 i4; records from c5; wm=12:2,19:2
    txg8 i4 c5 root->disk0 wm=12:2,19:2
    txg9 i4 c6 root->disk0 wm=12:2,19:2
    txg10 i4 c7 root->disk1 wm=12:2,19:2
    txg11 i4 c8 root->disk0 creates (19,2,bg11) wm=12:2,19:3
```

读法：回退尝试算出 19 的水位是 3（`wm=12:2,19:3`），写到 c5 就崩了——c5 原来是 txg 5 那条记录，唯一带着「19 在 txg 5 发过 inode 2」的那条。普通挂载选 txg 6 的根，新实例 i3 从 c7 接着写，它的第一次发布就可以复用 i2 那次尝试的孤儿单元。重做回退时盘 0 整块读不出（txg 5 的根在盘 0），`jt5` 已经找不到 txg 5 的记录（c5 现在是 i2 那条），所以只算 1 个故障；AJ 取到 2，inode 2 被重发。丙、CJ 在同一段上：可读根最大 txg 7，第一个新根 txg 8，新对象落在 txg 11，出生代与 txg 5 不同；CJ 另外还读得到 i2 那条记录的 checkpoint_txg 7，结论一样。

### 2.3 s3：journal 环在一次回退的窗口里转过一圈

脚本：第一次挂载、建克隆头 19、一次空发布（标 R_old）、在 19 里建 1 个对象（记录落在 c6）、销毁 19；然后每一圈都发布到下一个 txg 落在 R_old 那个根槽上，让那次根槽写失败（写失败的槽按旧内容算，`16-发布语义.md:372`，R_old 因此再留一圈），共 `laps` 圈；接着回退到 R_old，在 19 里建 1 个对象。没有读故障。S = 4（一圈 12 次发布），J 取 12 到 192 与「大」，每次发布写 1 条或 4 条记录。`c143r2_model.out` 第 207、208、211、213、218、223、228、233、236、238、243、248、253、258、263 行（整行抄；`first_violation=(圈数, 故障数, c_H, 回退前的计数器)`）：

```
name=s3_wrap S=4 nrec=1 J=big arm=A first_violation=(1, 1, 6, 20) record_of_H_at=c6 trail=0:-(c_rb=8) 1:V(c_rb=20) 2:V(c_rb=32) 3:V(c_rb=44) 4:V(c_rb=56) 5:V(c_rb=68) 6:V(c_rb=80) 7:V(c_rb=92) 8:V(c_rb=104) 9:V(c_rb=116)
name=s3_wrap S=4 nrec=1 J=big arm=AJ first_violation=none<=9 record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:-(c_rb=56) 5:-(c_rb=68) 6:-(c_rb=80) 7:-(c_rb=92) 8:-(c_rb=104) 9:-(c_rb=116)
name=s3_wrap S=4 nrec=1 J=big arm=CJ first_violation=none<=9 record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:-(c_rb=56) 5:-(c_rb=68) 6:-(c_rb=80) 7:-(c_rb=92) 8:-(c_rb=104) 9:-(c_rb=116)
name=s3_wrap S=4 nrec=1 J=12 arm=AJ first_violation=(1, 1, 6, 20) record_of_H_at=c6 trail=0:-(c_rb=8) 1:V(c_rb=20) 2:V(c_rb=32) 3:V(c_rb=44) 4:V(c_rb=56) 5:V(c_rb=68) 6:V(c_rb=80) 7:V(c_rb=92) 8:V(c_rb=104) 9:V(c_rb=116)
name=s3_wrap S=4 nrec=1 J=24 arm=AJ first_violation=(2, 2, 6, 32) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:V(c_rb=32) 3:V(c_rb=44) 4:V(c_rb=56) 5:V(c_rb=68) 6:V(c_rb=80) 7:V(c_rb=92) 8:V(c_rb=104) 9:V(c_rb=116)
name=s3_wrap S=4 nrec=1 J=48 arm=AJ first_violation=(4, 4, 6, 56) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:V(c_rb=56) 5:V(c_rb=68) 6:V(c_rb=80) 7:V(c_rb=92) 8:V(c_rb=104) 9:V(c_rb=116)
name=s3_wrap S=4 nrec=1 J=96 arm=AJ first_violation=(8, 8, 6, 104) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:-(c_rb=56) 5:-(c_rb=68) 6:-(c_rb=80) 7:-(c_rb=92) 8:V(c_rb=104) 9:V(c_rb=116)
name=s3_wrap S=4 nrec=1 J=192 arm=AJ first_violation=none<=9 record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:-(c_rb=56) 5:-(c_rb=68) 6:-(c_rb=80) 7:-(c_rb=92) 8:-(c_rb=104) 9:-(c_rb=116)
name=s3_wrap S=4 nrec=1 J=192 arm=CJ first_violation=none<=9 record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=20) 2:-(c_rb=32) 3:-(c_rb=44) 4:-(c_rb=56) 5:-(c_rb=68) 6:-(c_rb=80) 7:-(c_rb=92) 8:-(c_rb=104) 9:-(c_rb=116)
name=s3_wrap S=4 nrec=4 J=big arm=AJ first_violation=none<=9 record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=53) 2:-(c_rb=98) 3:-(c_rb=143) 4:-(c_rb=188) 5:-(c_rb=233) 6:-(c_rb=278) 7:-(c_rb=323) 8:-(c_rb=368) 9:-(c_rb=413)
name=s3_wrap S=4 nrec=4 J=12 arm=AJ first_violation=(1, 1, 6, 53) record_of_H_at=c6 trail=0:-(c_rb=8) 1:V(c_rb=53) 2:V(c_rb=98) 3:V(c_rb=143) 4:V(c_rb=188) 5:V(c_rb=233) 6:V(c_rb=278) 7:V(c_rb=323) 8:V(c_rb=368) 9:V(c_rb=413)
name=s3_wrap S=4 nrec=4 J=24 arm=AJ first_violation=(1, 1, 6, 53) record_of_H_at=c6 trail=0:-(c_rb=8) 1:V(c_rb=53) 2:V(c_rb=98) 3:V(c_rb=143) 4:V(c_rb=188) 5:V(c_rb=233) 6:V(c_rb=278) 7:V(c_rb=323) 8:V(c_rb=368) 9:V(c_rb=413)
name=s3_wrap S=4 nrec=4 J=48 arm=AJ first_violation=(2, 2, 6, 98) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=53) 2:V(c_rb=98) 3:V(c_rb=143) 4:V(c_rb=188) 5:V(c_rb=233) 6:V(c_rb=278) 7:V(c_rb=323) 8:V(c_rb=368) 9:V(c_rb=413)
name=s3_wrap S=4 nrec=4 J=96 arm=AJ first_violation=(3, 3, 6, 143) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=53) 2:-(c_rb=98) 3:V(c_rb=143) 4:V(c_rb=188) 5:V(c_rb=233) 6:V(c_rb=278) 7:V(c_rb=323) 8:V(c_rb=368) 9:V(c_rb=413)
name=s3_wrap S=4 nrec=4 J=192 arm=AJ first_violation=(5, 5, 6, 233) record_of_H_at=c6 trail=0:-(c_rb=8) 1:-(c_rb=53) 2:-(c_rb=98) 3:-(c_rb=143) 4:-(c_rb=188) 5:V(c_rb=233) 6:V(c_rb=278) 7:V(c_rb=323) 8:V(c_rb=368) 9:V(c_rb=413)
```

读法：甲 / 乙（A）1 圈就中，就是第一轮 H3。AJ 要等 txg 6 那条记录被盖掉，条件是 `c_rb − 1 ≥ c_H + J`：J = 12 时 1 圈、24 时 2 圈、48 时 4 圈、96 时 8 圈、192 时 9 圈之内不中；每次发布写 4 条记录时圈数降到 1 / 1 / 2 / 3 / 5。每一圈是一次根槽写失败，所以「圈数」就是故障数；如果那个槽持续写不进、失败处置的探针写落在别处照样写得进，每圈都判瞬时失败、走切换（`23-journal的角色与格式.md:691` 失败表那一段），那就是一个持续的故障。丙、CJ、G 全部不中：CJ 要的最大 txg 永远在最新的记录里，环的覆写先盖最旧的。第一版默认环要多少圈，算在第四节。

### 2.4 s2：回退那一刻最新的根与记录读不出——CJ 的载体只能被读故障打中

脚本空间：第一轮 sweep1 那个空间（前缀至多 5 次发布，每次从 `p(0) p(1) f(0) f(1)` 里选），回退到第 1 / 2 新的候选，回退那次读扫描的故障 = 盘 0 / 盘 1 / 都在 × 最新 0–3 个根槽 × 最新 0–5 条记录（还剩那块盘上的副本；两块盘都在时两份一起），回退后两次用户发布。严格暖机；宽读法只在前缀 ≤ 3 上跑，当对照。`c143r2_model.out` 第 76–84、88、90 行（整行抄）：

```
name=s2_newest_strict arm=A runs=169428 violating=78732 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=78732 by_fs=0:0/125;1:118/871;2:667/3060;3:2019/7061;4:4196/12174;5:6777/16850;6:9074/20177;7:10440/21746;8:10714/21264;9:9893/18666;10:8237/14806;11:6098/10964;12:4022/7878;13:2557/5612;14:1748/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=s2_newest_strict arm=AJ runs=169428 violating=51493 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=51493 by_fs=0:0/125;1:0/871;2:55/3060;3:390/7061;4:1242/12174;5:2626/16850;6:4243/20177;7:5876/21746;8:7215/21264;9:7678/18666;10:6907/14806;11:5315/10964;12:3627/7878;13:2415/5612;14:1732/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=s2_newest_strict arm=G runs=169428 violating=46000 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=46000 by_fs=0:0/125;1:0/871;2:55/3060;3:390/7061;4:1242/12174;5:2546/16850;6:3951/20177;7:5129/21746;8:5993/21264;9:6413/18666;10:6034/14806;11:4805/10964;12:3295/7878;13:2275/5612;14:1700/3926;15:1204/2504;16:696/1264;17:240/416;18:32/64
name=s2_newest_strict arm=C runs=169428 violating=9144 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=9144 by_fs=0:0/125;1:0/871;2:2/3060;3:33/7061;4:274/12174;5:699/16850;6:1075/20177;7:1298/21746;8:1393/21264;9:1376/18666;10:1187/14806;11:804/10964;12:454/7878;13:221/5612;14:134/3926;15:110/2504;16:68/1264;17:16/416;18:0/64
name=s2_newest_strict arm=CJ runs=169428 violating=3075 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=3075 by_fs=0:0/125;1:0/871;2:0/3060;3:0/7061;4:0/12174;5:2/16850;6:18/20177;7:113/21746;8:356/21264;9:606/18666;10:659/14806;11:496/10964;12:315/7878;13:182/5612;14:134/3926;15:110/2504;16:68/1264;17:16/416;18:0/64
name=s2_newest_strict hit=C not_hit=A scripts=0
name=s2_newest_strict hit=CJ not_hit=AJ scripts=0
name=s2_newest_strict hit=CJ not_hit=C scripts=1158
name=s2_newest_strict hit=AJ not_hit=CJ scripts=48418
name=s2_newest_strict hit=CJ not_hit=G scripts=0
name=s2_newest_strict hit=G not_hit=AJ scripts=0
```

CJ 最短的一段与丙在同一个前缀上最短的一段（第 131–161 行，整行抄）：

```
example name=s2_newest_strict arm=C min_by=fs fs=2 script=[p(0) p(0) p(0) p(0) p(1) rb(1,d0+r1,0) u(2)]
  violations=[((12, 2, 8), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 wm=12:2
    txg5 i1 c5 root->disk0 wm=12:2
    txg6 i1 c6 root->disk0 wm=12:2
    txg7 i1 c7 root->disk1 wm=12:2
    txg8 i1 c8 root->disk0 creates (12,2,bg8) wm=12:3
    ROLLBACK u=d0+r1: R_old=txg4 i1; readable ring max txg4; first new root txg5 i2; records from c5; wm=12:2
    txg5 i2 c5 root->disk0 wm=12:2
    txg6 i2 c6 root->disk0 wm=12:2
    txg7 i2 c7 root->disk1 wm=12:2
    txg8 i2 c8 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i2 c9 root->disk0 creates (12,3,bg9) wm=12:4
example name=s2_newest_strict arm=CJ min_by=fs fs=5 script=[p(0) p(0) p(0) p(0) p(1) rb(1,d0+r1+j3,0) u(2)]
  violations=[((12, 2, 8), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 wm=12:2
    txg5 i1 c5 root->disk0 wm=12:2
    txg6 i1 c6 root->disk0 wm=12:2
    txg7 i1 c7 root->disk1 wm=12:2
    txg8 i1 c8 root->disk0 creates (12,2,bg8) wm=12:3
    ROLLBACK u=d0+r1+j3: R_old=txg4 i1; readable ring max txg4; first new root txg6 i2; records from c5; wm=12:2
    txg6 i2 c5 root->disk0 wm=12:2
    txg7 i2 c6 root->disk1 wm=12:2
    txg8 i2 c7 root->disk0 creates (12,2,bg8) wm=12:3
    txg9 i2 c8 root->disk0 creates (12,3,bg9) wm=12:4
```

读法：被抛弃时间线在 txg 8 建了 inode 2（盘 0）。CJ 那一段：盘 0 整块读不出（txg 5、6、8 的根，以及每条记录在盘 0 的副本），盘 1 上最新的根（txg 7）读不出，盘 1 上最新 3 条记录（c6、c7、c8）的副本读不出。按槽计 5 个故障，按区间计 3 个（整块盘、一个根槽、盘 1 上连续三个记录槽）。读得出的最大是 txg 4 的根与 c5 那条记录（txg 5），CJ 的第一个新根取 6，暖机推 6、7，新对象落在 txg 8，(12, 2, 8) 被重发。丙那一段只要前两样（2 个故障）：第一个新根 5，暖机推 5、6、7，同样落在 8。

**CJ 的载体为什么在没有读故障时不退回**（推理，s1–s6 的数与它相符）：它要的是环里最大的 `checkpoint_txg`。① 环的覆写按计数器从小到大，先盖最旧的记录；② 新实例从前缀末 + 1 覆写，在回退里盖的是 R_old 之后最旧的那几条，在择根倒挂的挂载里盖的是较新实例的记录，可是那一刻 CJ2 已经把它们的最大值读进了新实例的第一个 txg，新写的记录 txg 都更大；③ 回退尝试崩了留下的那条记录，自己就带着 T_first ≥ 当时的最大值 + 1。三处都只会让环里的最大值不减。所以要打穿 CJ，只能把最新几个 txg 的根与记录两份副本一起读不出，而那几条记录在两块盘上占同一段连续的槽。

### 2.5 s5 / s6：回退之后的挂载、切换、第二次回退（AJ2 / CJ2 那一格）

s5：前缀（空、`p(1)`、`p(1) p(1)`、`f(1)`、`p(1) f(1)`、一次写 4 条记录的发布、它再接 `p(1)`）→ 干净的回退（第 1–3 新的候选）→ 回退后 1–3 次用户发布，或者先一次根槽写失败的用户发布（回退之后切换一次）再一次 → 崩溃 → 普通挂载，读故障 = 盘 × 最新 0–4 个根槽 × 最新 0–6 条记录 → 1 次用户发布。s6：同一批前缀与回退后的发布，然后第二次回退（第 1–3 新的候选，读故障 = 盘 × 最新 0–3 个根槽 × 最新 0–5 条记录）→ 1 次用户发布。`viol_mount_on_abandoned` = 违例的那一段里，普通挂载选中了被回退抛弃的时间线上的根（回退被撤销，C332 那一类）。第 501–509、514、638–646、651 行（整行抄）：

```
name=s5_rb_then_crash_mount arm=A runs=6427 violating=5037 min_fs=2 min_fr=2 viol_mount_on_abandoned=504 viol_other=4533 by_fs=0:0/33;1:0/128;2:52/291;3:226/455;4:425/607;5:574/697;6:642/750;7:651/748;8:612/686;9:508/543;10:388/427;11:274/303;12:188/227;13:145/174;14:135/141;15:103/103;16:73/73;17:35/35;18:6/6
name=s5_rb_then_crash_mount arm=AJ runs=6427 violating=5037 min_fs=2 min_fr=2 viol_mount_on_abandoned=504 viol_other=4533 by_fs=0:0/33;1:0/128;2:52/291;3:226/455;4:425/607;5:574/697;6:642/750;7:651/748;8:612/686;9:508/543;10:388/427;11:274/303;12:188/227;13:145/174;14:135/141;15:103/103;16:73/73;17:35/35;18:6/6
name=s5_rb_then_crash_mount arm=AJ2 runs=6427 violating=4653 min_fs=2 min_fr=2 viol_mount_on_abandoned=421 viol_other=4232 by_fs=0:0/33;1:0/128;2:31/291;3:154/455;4:311/607;5:464/697;6:587/750;7:640/748;8:611/686;9:508/543;10:388/427;11:274/303;12:188/227;13:145/174;14:135/141;15:103/103;16:73/73;17:35/35;18:6/6
name=s5_rb_then_crash_mount arm=G runs=6427 violating=4653 min_fs=2 min_fr=2 viol_mount_on_abandoned=421 viol_other=4232 by_fs=0:0/33;1:0/128;2:31/291;3:154/455;4:311/607;5:464/697;6:587/750;7:640/748;8:611/686;9:508/543;10:388/427;11:274/303;12:188/227;13:145/174;14:135/141;15:103/103;16:73/73;17:35/35;18:6/6
name=s5_rb_then_crash_mount arm=C runs=6427 violating=1068 min_fs=2 min_fr=2 viol_mount_on_abandoned=105 viol_other=963 by_fs=0:0/33;1:0/128;2:13/291;3:50/455;4:88/607;5:121/697;6:136/750;7:136/748;8:130/686;9:122/543;10:98/427;11:57/303;12:34/227;13:20/174;14:17/141;15:18/103;16:17/73;17:11/35;18:0/6
name=s5_rb_then_crash_mount arm=CJ runs=6427 violating=1068 min_fs=2 min_fr=2 viol_mount_on_abandoned=105 viol_other=963 by_fs=0:0/33;1:0/128;2:13/291;3:50/455;4:88/607;5:121/697;6:136/750;7:136/748;8:130/686;9:122/543;10:98/427;11:57/303;12:34/227;13:20/174;14:17/141;15:18/103;16:17/73;17:11/35;18:0/6
name=s5_rb_then_crash_mount arm=CJ2 runs=6427 violating=525 min_fs=5 min_fr=3 viol_mount_on_abandoned=80 viol_other=445 by_fs=0:0/33;1:0/128;2:0/291;3:0/455;4:0/607;5:10/697;6:32/750;7:63/748;8:75/686;9:88/543;10:80/427;11:55/303;12:35/227;13:24/174;14:17/141;15:18/103;16:17/73;17:11/35;18:0/6
name=s5_rb_then_crash_mount hit=C not_hit=A scripts=0
name=s5_rb_then_crash_mount hit=CJ not_hit=AJ scripts=0
name=s5_rb_then_crash_mount hit=CJ2 not_hit=AJ2 scripts=0
name=s6_two_rollbacks arm=A runs=6126 violating=4728 min_fs=1 min_fr=1 viol_mount_on_abandoned=0 viol_other=4728 by_fs=0:0/54;1:86/190;2:243/413;3:434/601;4:539/721;5:607/774;6:610/792;7:589/703;8:471/549;9:353/404;10:238/304;11:192/243;12:167/179;13:124/124;14:63/63;15:12/12
name=s6_two_rollbacks arm=AJ runs=6126 violating=3940 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=3940 by_fs=0:0/54;1:0/190;2:32/413;3:193/601;4:372/721;5:538/774;6:596/792;7:589/703;8:471/549;9:353/404;10:238/304;11:192/243;12:167/179;13:124/124;14:63/63;15:12/12
name=s6_two_rollbacks arm=AJ2 runs=6126 violating=3940 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=3940 by_fs=0:0/54;1:0/190;2:32/413;3:193/601;4:372/721;5:538/774;6:596/792;7:589/703;8:471/549;9:353/404;10:238/304;11:192/243;12:167/179;13:124/124;14:63/63;15:12/12
name=s6_two_rollbacks arm=G runs=6126 violating=3940 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=3940 by_fs=0:0/54;1:0/190;2:32/413;3:193/601;4:372/721;5:538/774;6:596/792;7:589/703;8:471/549;9:353/404;10:238/304;11:192/243;12:167/179;13:124/124;14:63/63;15:12/12
name=s6_two_rollbacks arm=C runs=6126 violating=630 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=630 by_fs=0:0/54;1:0/190;2:8/413;3:39/601;4:59/721;5:84/774;6:92/792;7:94/703;8:84/549;9:68/404;10:44/304;11:23/243;12:11/179;13:13/124;14:11/63;15:0/12
name=s6_two_rollbacks arm=CJ runs=6126 violating=230 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=230 by_fs=0:0/54;1:0/190;2:0/413;3:0/601;4:0/721;5:6/774;6:21/792;7:42/703;8:46/549;9:46/404;10:21/304;11:13/243;12:11/179;13:13/124;14:11/63;15:0/12
name=s6_two_rollbacks arm=CJ2 runs=6126 violating=230 min_fs=5 min_fr=3 viol_mount_on_abandoned=0 viol_other=230 by_fs=0:0/54;1:0/190;2:0/413;3:0/601;4:0/721;5:6/774;6:21/792;7:42/703;8:46/549;9:46/404;10:21/304;11:13/243;12:11/179;13:13/124;14:11/63;15:0/12
name=s6_two_rollbacks hit=C not_hit=A scripts=0
name=s6_two_rollbacks hit=CJ not_hit=AJ scripts=0
name=s6_two_rollbacks hit=CJ2 not_hit=AJ2 scripts=0
```

CJ2 最短的一段（第 621–635 行，整行抄）：

```
example name=s5_rb_then_crash_mount arm=CJ2 min_by=fs fs=5 script=[rb(1,none,0) u(1) crash mount(d0+r1+j3) u(1)]
  violations=[((12, 2, 6), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    ROLLBACK u=none: R_old=txg3 i1; readable ring max txg3; first new root txg4 i2; records from c4; wm=12:2
    txg4 i2 c4 root->disk1 wm=12:2
    txg5 i2 c5 root->disk0 wm=12:2
    txg6 i2 c6 root->disk0 creates (12,2,bg6) wm=12:3
    CRASH (nothing in flight)
    MOUNT u=d0+r1+j3: selects txg1 i1, replays to txg3 (prefix end c3), new i3 from txg4, wm=12:2
    txg4 i3 c4 root->disk1 wm=12:2
    txg5 i3 c5 root->disk0 wm=12:2
    txg6 i3 c6 root->disk0 creates (12,2,bg6) wm=12:3

```

读法：
- **CJ2 的 5 / 3** 与 2.4 同形，只是发生在挂载上：回退之后新实例 i2 在 txg 6 建了 inode 2；重挂时盘 0 整块、盘 1 上 i2 的根（txg 4）、盘 1 上最新 3 条记录读不出，恢复一路退到第一次挂载的 txg 1 那个根，按 i1 自己的记录重放到 txg 3，新实例从 txg 4 往上数，又走到 6。丙与 CJ（CJ 只在回退时起作用）在同一段上只要前两样：第 593 行那一段 `mount(d0+r1)`，2 个故障。这正是第一轮 H5 / C331（择根倒挂压过已确认的写）那一形；CJ2 把它从 2 抬到 5 / 3，AJ2 没抬。
- **AJ2 / G 的 2**（第 565、579 行那一段 `mount(d0+j1)`）：重挂时盘 0 整块读不出、盘 1 上最新那条记录（c6）读不出，恢复选 i2 在盘 1 上的 txg 4，重放到 c5 为止，i2 在 txg 6 已确认的 inode 2 丢了，号又被发一次。这是一次已确认写被 2 个故障丢掉，号被重发是后果；它与第二节 2.8 的 s9（普通崩溃，没有回退）同一个机制，只是碰巧发生在回退之后。丙不中（出生代 8 ≠ 6）。
- s6 的第二次回退与 2.4 同形：A 1 个故障（第 673 行那一段 `rb(1,d0,0)`），AJ / AJ2 / G 2 个，丙 2 个，CJ / CJ2 5 / 3。
- 「回退之后切换一次」那几条脚本（`MIDS` 里带 `f(1)` 的）没有产出更短的历史：切换只把 txg 往大推，按 keep 读法水位不动。

### 2.6 s8：加盘或改归属之后丙的余量（解析枚举，不是模拟）

一条时间线，把最新 L 个 txg 的根藏掉（J 臂再加每条记录的两份副本）；新实例从 T_first 起推空发布，直到本实例的根覆盖两块盘（单盘池覆盖两个区域），第一个用户对象落在 T_first + w。丙中 ⟺ L ≥ w + 1（严格）；宽读法下用户对象紧跟回退根，丙中 ⟺ L ≥ 2；A / AJ 中 ⟺ L ≥ 1。代价按 2.0 的口径；`warmup=` 是新实例从各个区域起步各要推几次才覆盖。`c143r2_model.out` 第 777–786 行（整行抄）：

```
name=s8_geometry own=0/0/0 devs=1 warmup=[2] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=-1] C_strict:fs=3,fr=3[L=3,d=-1] CJ_strict:fs=6,fr=4[L=3,d=-1] C_lenient:fs=2,fr=2[L=2,d=-1] CJ_lenient:fs=4,fr=3[L=2,d=-1]
name=s8_geometry own=0/1/0 devs=2 warmup=[2, 3] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=1,fr=1[L=2,d=0] CJ_lenient:fs=3,fr=2[L=2,d=0]
name=s8_geometry own=0/1/0 devs=3 warmup=[2, 3] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=1,fr=1[L=2,d=0] CJ_lenient:fs=3,fr=2[L=2,d=0]
name=s8_geometry own=0/1/2 devs=3 warmup=[2] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=3,fr=3[L=3,d=-1] CJ_strict:fs=6,fr=4[L=3,d=0] C_lenient:fs=2,fr=2[L=2,d=-1] CJ_lenient:fs=4,fr=3[L=2,d=0]
name=s8_geometry own=0/1/0/1 devs=2 warmup=[2] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=2,fr=2[L=2,d=-1] CJ_lenient:fs=4,fr=3[L=2,d=0]
name=s8_geometry own=0/0/1/1 devs=2 warmup=[2, 3] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=1,fr=1[L=2,d=0] CJ_lenient:fs=3,fr=2[L=2,d=0]
name=s8_geometry own=0/1/2/0 devs=3 warmup=[2, 3] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=1,fr=1[L=2,d=0] CJ_lenient:fs=3,fr=2[L=2,d=0]
name=s8_geometry own=0/1/2/3 devs=4 warmup=[2] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=3,fr=3[L=3,d=-1] CJ_strict:fs=6,fr=4[L=3,d=0] C_lenient:fs=2,fr=2[L=2,d=-1] CJ_lenient:fs=4,fr=3[L=2,d=0]
name=s8_geometry own=0/1/0/1/0 devs=2 warmup=[2, 3] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=1,fr=1[L=2,d=0] CJ_lenient:fs=3,fr=2[L=2,d=0]
name=s8_geometry own=0/1/2/0/1 devs=3 warmup=[2] A:fs=1,fr=1[L=1,d=-1] AJ:fs=2,fr=2[L=1,d=0] C_strict:fs=2,fr=2[L=3,d=0] CJ_strict:fs=5,fr=3[L=3,d=0] C_lenient:fs=2,fr=2[L=2,d=-1] CJ_lenient:fs=4,fr=3[L=2,d=0]
```

读法：
- 第一版 0 / 1 / 0：丙 2、CJ 5 / 3、宽读法下丙 1，与 s2 的模拟一致（2.4）。
- **只加盘、根环归属不动**（0 / 1 / 0，devs = 3）：同上。暖机只要求本实例的根覆盖两块盘（`16-发布语义.md:207`），新盘上没有根区域，余量不变。
- **根环铺到三块盘**（0 / 1 / 2）：丙 3、CJ 6 / 4。没有一块盘背两个相邻区域，一块盘整块读不出只藏得住 1 个 txg。
- 余量由两样定：一块盘上最长的一串相邻区域 m（整块盘读不出一次藏掉 m 个 txg），与新实例要推几次 w。R = 3、两块盘时必有一块盘背两个相邻区域（`22-单元原子性怎么合成.md:1013` 第 4 句自己推出只有这一种形状）⇒ 丙 2；R = 4 的 0 / 1 / 0 / 1 也是 2（整块盘 + 一个槽藏掉「盘 0、盘 1、盘 0」三个 txg）。
- 单盘池（0 / 0 / 0）：整块盘读不出就没有根可挂，只能按槽藏，丙 3、CJ 6 / 4（记录只有一份）。
- 这张表绑在「暖机覆盖两块盘」这一条上；暖机次数随归属变，条款自己写着「归属改了这两个常量要重算（加盘或开条带表时）」（`16-发布语义.md:208`、`22-单元原子性怎么合成.md:1014`）。

### 2.7 第一版有没有路径让用户事务进暖机那几次发布

按条款逐条查（条款指路 + 推理，模型里 s5 的切换变体与它相符）：

1. **挂载与回退**：新实例的第一次发布是写行那次（回退是写回退行那次），之后「连推空发布」直到覆盖两块盘（`16-发布语义.md:207`）；空发布 = 事务号 0、不承载事务的记录（`23-journal的角色与格式.md:698` 第 ① 条）；第一版暖机次数是格式常量（`22-单元原子性怎么合成.md:1014`）；回退在暖机完成之前不向管理员确认。第一版是纯用户态库、串行提交（`17-实现分层与第三方管道.md:95`；`23-journal的角色与格式.md:1250` 引的 D13（验证路线） 已定项 2）。⇒ 这两条路上用户事务进不了暖机那几次发布。「打开」在恢复与暖机做完之前不交出可写句柄，是我按库的形态推的，条款没有逐字写。
2. **实例切换**：新实例的第一次发布是「重发在飞 checkpoint」（`23-journal的角色与格式.md:691`「实例切换 = 挂载内做一次恢复：取新实例代号、写行、重发在飞 checkpoint」），号 ≤ W 的事务照旧、号 > W 的按新写序重做；切换之后才接暖机（`17-实现分层与第三方管道.md:186`「切换 / 回退之后要接暖机」）。⇒ **这一次发布里有用户事务，而且在本实例覆盖两块盘之前**：按字面，宽读法在切换这条路上可达。
3. **它缩不缩丙的余量**：不缩。照旧的事务在切换之前就建好了对象，出生代是失败那次发布的 txg（数据单元已经带着它写出去了，而 I-9.10 要求记录与数据单元的出生代一致，`invariants.md:266`）；重做的事务拿新写序，出生代不小于重发那次的 txg。两种都不小于「切换前那个实例覆盖两块盘之后」的 txg。回退实例自己的暖机里没有用户事务（第 1 条），所以回退之后第一个用户对象的出生代 ≥ T_first + 2。

⇒ 宽读法在第一版里对 C143 不承重；s2 的宽读法那几行（第 162–169 行）只当对照：丙 1 个故障、CJ 3 / 2。推翻这一条的观测写在第七节。

### 2.8 s9（旁支，不是回退）：普通崩溃后最新的根与它的记录读不出

没有回退；判定改成「任意两个写成了根的对象身份相同」。s9 严格暖机第 790–796 行、宽读法丙那一行第 870 行、甲那一段第 817–828 行（整行抄）：

```
name=s9_plain_crash_strict arm=A runs=154 violating=86 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=86 by_fs=0:0/4;1:0/12;2:4/24;3:16/27;4:17/26;5:20/24;6:8/12;7:9/9;8:3/7;9:5/5;10:3/3;11:1/1
name=s9_plain_crash_strict arm=AJ runs=154 violating=86 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=86 by_fs=0:0/4;1:0/12;2:4/24;3:16/27;4:17/26;5:20/24;6:8/12;7:9/9;8:3/7;9:5/5;10:3/3;11:1/1
name=s9_plain_crash_strict arm=AJ2 runs=154 violating=86 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=86 by_fs=0:0/4;1:0/12;2:4/24;3:16/27;4:17/26;5:20/24;6:8/12;7:9/9;8:3/7;9:5/5;10:3/3;11:1/1
name=s9_plain_crash_strict arm=G runs=154 violating=86 min_fs=2 min_fr=2 viol_mount_on_abandoned=0 viol_other=86 by_fs=0:0/4;1:0/12;2:4/24;3:16/27;4:17/26;5:20/24;6:8/12;7:9/9;8:3/7;9:5/5;10:3/3;11:1/1
name=s9_plain_crash_strict arm=C runs=154 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/4;1:0/12;2:0/24;3:0/27;4:0/26;5:0/24;6:0/12;7:0/9;8:0/7;9:0/5;10:0/3;11:0/1
name=s9_plain_crash_strict arm=CJ runs=154 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/4;1:0/12;2:0/24;3:0/27;4:0/26;5:0/24;6:0/12;7:0/9;8:0/7;9:0/5;10:0/3;11:0/1
name=s9_plain_crash_strict arm=CJ2 runs=154 violating=0 min_fs=None min_fr=None viol_mount_on_abandoned=0 viol_other=0 by_fs=0:0/4;1:0/12;2:0/24;3:0/27;4:0/26;5:0/24;6:0/12;7:0/9;8:0/7;9:0/5;10:0/3;11:0/1
name=s9_plain_crash_lenient arm=C runs=154 violating=25 min_fs=3 min_fr=2 viol_mount_on_abandoned=0 viol_other=25 by_fs=0:0/4;1:0/12;2:0/24;3:1/27;4:4/26;5:5/24;6:5/12;7:4/9;8:2/7;9:1/5;10:2/3;11:1/1
```

```
example name=s9_plain_crash_strict arm=A min_by=fs fs=2 script=[p(1) crash mount(d1+j1) u(2)]
  violations=[((12, 2), [2, 3], True)]
    txg1 i1 c1 root->disk1 wm=
    txg2 i1 c2 root->disk0 wm=
    txg3 i1 c3 root->disk0 creates (12,1,bg3) wm=12:2
    txg4 i1 c4 root->disk1 creates (12,2,bg4) wm=12:3
    CRASH (nothing in flight)
    MOUNT u=d1+j1: selects txg3 i1, replays to txg3 (prefix end c3), new i2 from txg4, wm=12:2
    txg4 i2 c4 root->disk1 wm=12:2
    txg5 i2 c5 root->disk0 wm=12:2
    txg6 i2 c6 root->disk0 creates (12,2,bg6) wm=12:3
    txg7 i2 c7 root->disk1 creates (12,3,bg7) wm=12:4
```

读法：txg 4 建 inode 2（根在盘 1），fsync 返回过；崩溃后重挂时盘 1 整块读不出、c4 在盘 0 上的副本也读不出：2 个故障，恢复落回 txg 3，这次已确认的写丢了，号 2 被重发。甲 / 乙 / AJ / AJ2 / G 都中；丙、CJ、CJ2 在严格暖机下不中（新对象出生代 6 ≠ 4），宽读法下 3 / 2 个故障才中。它的病根是「一次已确认的写被两个故障丢掉」（暖机只防单故障，C332 那一行的措辞），号被重发是后果，不是 C143 那个载体的问题；列在这里是因为 s5 里 AJ2 / G 的 2 个故障历史就是它（2.5）。

## 三、U2：今天就存在的、只认号的消费者；按 inode 号作 key 又不随根回退的结构

判据（背景材料第四节 U2 那一行）：写出一个只认号的消费者在某条臂下失效的具体读写序列，而且这个消费者要指得出它今天住在哪条条款里。

先说把整张表压扁的一件事（推理，条款指路）：丙只在一处比甲 / 乙少保证——一个已发布、已确认的号，在一次 0 故障的回退之后被新对象再用一次，新旧对象的出生代不同。盘上凡是随根回退的结构，回退之后都回到 R_old 那一版，被抛弃的对象不在里面（`23-journal的角色与格式.md:1206`「全部记账统计量的现行值从 R_old 那棵账重新载入」；实例表随根分版本，`18-块里携带什么信息.md:879`）。所以要找的消费者必须同时满足三件事：① 只看号、不看出生代；② 不随根回退（住在根环之外，或者在盘外）；③ 今天有条款定义它。

查法：在 `decisions/`、`invariants.md`、`checks-owed.md`、`verification-build.md`、`first-txn-layout.md`、`milestone-first-txn.md` 里 `grep -rn`「只存号 / 按 inode 号 / inode 号作 / 按号作 / key = inode / inode 号为 / 对象 ID 作 / 按对象号 / (树 ID, inode) / (树, inode)」，命中 7 处，逐条读了上下文；再把第一轮与材料列过的消费者、这一轮新建的 journal 结构逐个过一遍。

| 候选 | 只认号？ | 随根回退？ | 住在哪 | 判 |
|---|---|---|---|---|
| inode 树的 key = inode 号，唯一性范围 (树 ID, inode) | 是 | 随根（树表 → inode 树） | `08-核心索引结构.md:378` | 回退后 inode 树是 R_old 那一版，被抛弃对象不在里面，不失效 |
| extent key `(locality_id, inode, offset)` | 是（不带出生代） | 随根（每头一棵 extent 树） | `08-核心索引结构.md:132` | 同上 |
| `deleted_inodes`「只存号」 | 是 | 没定义：没有树 ID、没进树表 | `08-核心索引结构.md:380`；`checks-owed.md:124`（C118） | 今天没有形态，读写序列写不出来 |
| 批量删除的意图（`logged_ops` 树） | 没定义 | 没定义 | `checks-owed.md:124`（C118 同一行） | 同上 |
| livelist 共享树：条目 key = 头的树 ID 8 + 中央映射 key 27 | 否：码 1 的映射 key = 类标签 + 出生树 + 出生 txg + 写序，不含 inode 号 | 随根（树表 day-1 注册） | `06-快照实现模型.md:247`；`19-块指针的结构与宽度预算.md:98` | 不认号 |
| 稀疏旁表：条目 key = 中央映射 key 27 | 否 | 随根 | `05-快照-空间记账机制.md:244` | 不认号 |
| 中央映射树 | 否 | 根住根记录，随根 | `19-块指针的结构与宽度预算.md:98` | 不认号 |
| journal 记录的点名项（位置条目 × 2、类标签、出生树、出生 txg、key 尾段、flags） | 否：一个字段都不含 inode 号 | 不随根（留在环里） | `23-journal的角色与格式.md:696` | 不带号 |
| 超级块 | 字段表里没有与 inode 号有关的字段（`first-txn-layout.md` 第 95–152 行那一段 grep「inode」零命中） | 不随根 | `first-txn-layout.md` 超级块字段表；`22-单元原子性怎么合成.md` 已定项 9 | 不适用 |
| 墓碑区间记录（对象 ID + 对象出生代 + 区间） | 否：带出生代 | 随写者树 | `18-块里携带什么信息.md:749` | 不失效 |
| I-9.6「水位 > 该树全部墓碑记录的对象 ID」 | 是（只看对象 ID） | checker 遍历侧看的是可达的墓碑 | `invariants.md:262` | 按遍历侧读不失效。有人按扫描侧读（把被抛弃时间线还没回收的墓碑单元也算进来），丙在 0 故障回退之后会红——要写明「已发布的墓碑」（第一轮已提），它是 checker 的措辞问题，不是运行时消费者 |
| checker 的 I-9.10（按 inode 号把数据单元接到 inode 记录上，再比出生代；`crates/singlefs-checker/src/walk.rs:740` 起） | 按号接、再比出生代 | 单镜像、只看可达 | `invariants.md` I-9.10 那一行 | 可达集里没有被抛弃对象；它比的正是出生代 |
| 级 1 扫描重建（实例表读不出）按号接 extent | 是 | 看的是盘 | `18-块里携带什么信息.md:879`「表不可读时」那一句 | 第一轮 H7：普通崩溃之后三条臂都混接，不分辨臂 |
| 对外暴露的 inode 号（句柄） | 是 | 盘外 | 没有条款：第一版不挂载（`17-实现分层与第三方管道.md:95`）；kb 全仓 grep「NFS / 文件句柄 / st_ino / i_generation」只命中 `18-块里携带什么信息.md:97` 一处，那是引 ext4 的校验和种子 | 今天没有消费者（第一轮 H8，主 agent 已判「今天没有」） |
| 库操作面的调用者手里的对象标识（D17 已定项 5「对外是一组操作（mkfs、打开、写 extent、发布 / fsync、恢复）」） | 条款没说用什么标识对象 | 盘外 | `17-实现分层与第三方管道.md`「已定项 5」正文 | 管理员回退是挂载时带外做的一次恢复；调用者跨回退持有号，今天没有条款定义，与对外句柄同一格 |

⇒ **U2 结论**：今天没有一个同时满足三件事的消费者；「按 inode 号作 key、又不随根回退」的结构今天也没有（`deleted_inodes` 与批量删除的意图住哪全仓没定，C118）。跑前反向接受条款第二条（丙在 U2 被一个今天就存在的只认号的消费者打中 ⇒ 丙出局）这一轮不触发。

两句要写明，免得被读成打中：

- 将来的盘外观察者（挂载面、调用者持久化的号）一出现，丙在 0 故障的回退之后就让它看到号被重发；甲 / 乙（不带 AJ）在第一轮 H1 那种 1 故障历史里同样会，AJ 在第二节 s1 / s4 的 1–2 故障历史里也会。那时要的条款是「对外身份 = (号, 出生代)」，三条臂都要，第一轮已列为丙的配套。
- C118 落定时要复查：如果 `deleted_inodes` 或批量删除的意图只记号、又住在不随根回退的地方（例如只住 journal、不进树表），丙在 0 故障的回退之后会让它作用到新对象上，甲 / 乙不会。丙的配套「按号作 key 的结构随根回退，或 key 带出生代」就是给这一格的。

## 四、U3：代价，以及 AJ / CJ 在回退与挂载时要读的 journal 量

记号：J = journal 环的记录槽数 = 环长 ÷ 4096（第一版默认环 768 MiB ⇒ J = 196608，`first-txn-layout.md:45`；环长是 mkfs 参数、约束「环 ≤ 设备容量 ÷ 4」，`23-journal的角色与格式.md:1270` ⇒ J ≤ 设备容量 ÷ 16384）；R·S = 根槽数（第一版 3 × 8 = 24）；d_tt = 树表层数（≤ 109 棵树 1 层，≤ 109² 棵 2 层，`08-核心索引结构.md:466`）；H = 可写头数；d_acct = 记账树高；N_pub(T_old) = 环里还读得出记录的、txg > T_old 的发布次数；r = 一次发布写的记录条数（空发布 1）。一次节点读 16 KiB。

| 臂 | 格式字节 | 改第一个事务的字节 | 回退时多读 | 每次挂载多读 | 发号路径 |
|---|---|---|---|---|---|
| 甲 | 树表条目预留 24 里切 8，而这 24 已被认购满（第一轮判决第 4 条）；不挤掉认购者就要加宽 148 → 156，每层 ⌊16253 / 156⌋ = 104 棵（今天 109） | 是 | 每个可读根一次树表下探：≤ R·S × d_tt 次节点读 | 0 | 0 |
| 乙 | 0 | 否 | ≤ R·S × H × d_acct 次记账点查 | 0 | 0 |
| 丙 | 0 | 否 | 0 | 0 | 0 |
| CJ | 0 | 否 | 要环里全部自证通过的记录的 `checkpoint_txg`，即 J 个记录头。「回退 = 一次恢复」（`23-journal的角色与格式.md:1206`），恢复必须先全环扫描、逐条验证（`:811`）⇒ 这一遍本来就读，额外 0。若实现把回退写成不扫环（它不施加任何记录），CJ 要补一遍：J × 4096 字节（读一份副本，另一份只在这一份读不出时读） | — | 0 |
| CJ2 | 0 | 否 | 同 CJ | 0：普通挂载本来就全环扫描；切换不用重扫（内存里的 txg 已不小于这次挂载见过的一切） | 0 |
| AJ | 0 | 否 | 甲：N_pub(T_old) × d_tt；乙：N_pub(T_old) × (d_tt + H × d_acct)。按发布去重（同一次发布的几条记录新根段相同）。要读的是记录新根段指到的单元，单元被复用了就读不到（这正是 s1 / s4 那几段历史的缺口） | — | 0 |
| AJ2 | 0 | 否 | 同 AJ | 每次挂载：不在重放链上的、读得出的记录所在的发布，同一个式子；最坏 N_pub = J ÷ r | 0 |
| G（线索） | 根记录 +8（371 → 379，512 槽余量 141 → 133）；记录头 +8（307 → 315，一条 4096 记录装 ⌊(4096 − 315) ÷ 56⌋ = 67 个点名项，与今天相同） | 是（根记录与第一个事务三条记录的头） | 0（根记录与记录头本来就读） | 0 | 0 |

**N_pub(T_old) 的量级**（按条款推，s3 在小环上量过同一个关系）：平常 R_old 在根环里，T_max − T_old ≤ R·S − 1 = 23，AJ 多读 ≤ 23 × d_tt 个节点。可是「写失败的槽按旧内容算」（`16-发布语义.md:372`）能让 R_old 在环里多留任意多圈：每圈一次根槽写失败；或者同一个槽持续写不进、而失败处置的探针写落在「目标设备的固定落点」上照样写得进，于是每圈都判成瞬时失败、走切换（`23-journal的角色与格式.md:691` 失败表那一段）。那时 txg > T_old 的发布数没有上界，环里读得出的记录最多 J 条 ⇒ AJ 最坏读 (J ÷ r) × d_tt 个节点：默认环、空发布、2 层树表是 196608 × 2 × 16 KiB ≈ 6 GiB，乙再乘 (1 + H × d_acct ÷ d_tt)。J 跟着 mkfs 选的环长走，而环长的上限是设备容量 ÷ 4 ⇒ **在允许的几何下，AJ / AJ2 回退与挂载要读的量随盘容量涨**（U3 的触发观测）。CJ / CJ2 的额外读量不随任何东西涨。

**journal 环在一次回退窗口里转过一圈要多少圈根环**（s3 的量，S = 4、每圈 R·S = 12 次发布）：H 那条记录的计数器 c_H = 6，回退前计数器走到 c_rb；H 被盖掉的条件是 `c_rb − 1 ≥ c_H + J`。一次发布 1 条记录时每圈约 12 条，4 条时约 45 条（整行在第二节 2.3）。推到第一版（推理，模型只在 J ≤ 192 上跑过）：S = 8、每圈 24 次发布、J = 196608。全是空发布要约 196608 ÷ 24 ≈ 8192 圈；每次发布写满时，有效 T_dirty = min(2 GiB, 环长 ÷ 3) = 256 MiB（`23-journal的角色与格式.md:1272`），按 32 KiB 数据单元约 8192 个、一条记录 67 个点名项算约 123 条记录，全是 16 KiB 节点时约 245 条，于是约 196608 ÷ (24 × 245) ≈ 34 圈。每圈一次根槽写失败 ⇒ 34 个到 8192 个写故障，或者一个持续写不进的槽。

**顺带一笔代价（不是 CJ 特有；按条款推，没有建模，没核实现）**：记账「只保留最近 K 代」的丢弃是点删「当前代 − K」那一代（`05-快照-空间记账机制.md:284`），代 = checkpoint 号、按发布走（`05-快照-空间记账机制.md:261`）。txg 一跳过，被跳过的那几个 txg 永远不会成为「当前代」，于是「那几个 txg − K」那几代的行没有任何一次发布去点删：跳 Δ 格留下至多 Δ − 1 代的行，每代每个在维护的统计量一行（第一个事务 15 行 × 34 字节），永远没人删。今天已经会跳：根槽写失败推一格（Δ = 2）、回退新根取环里最大 + 1（Δ 到 R·S 量级）。CJ / CJ2 让 Δ 可以更大，最大到被抛弃时间线的长度。要不要立账由主 agent 判。

## 五、打中之后先问三句

逐个写：它分不分辨臂；被判的系统在做决定那一刻看不看得到判别它的东西；它满足的是哪条判据的哪一个分句。编号接第一轮（H1–H8），这一轮从 H9 起。

| # | 打中 | 分辨臂吗 | 做决定那一刻看得到判别子吗 | 满足哪条判据的哪个分句 |
|---|---|---|---|---|
| H9 | AJ：被抛弃时间线里销毁了可写头，盘 0 整块 + 盘 1 上一条记录的副本读不出，2 个故障（2.1） | 分辨：同一段历史丙、CJ、G 都不中（s1 `hit=AJ not_hit=CJ` 1824 条，`hit=CJ not_hit=AJ` 0 条） | 看不到：那个水位只在 txg 5 那一版的树表 / 记账单元里（盘上还在，被 R_old 钉着没复用），指到它的根与记录都读不出；回退的决定过程不扫单元，找不到它。丙、CJ 要的是全池 txg，txg 6 的记录在盘 1 上读得出 | U1「各臂在回退之后会不会重发一个已发布的身份（甲、乙按号）」+ 这一轮点名的「两块盘上同一条记录的槽同时读不出」（这里是一块盘整块，加另一块盘上那一个槽） |
| H10 | AJ：回退那次发布崩在记录之后，重做时 1 个故障（2.2） | 分辨：丙、CJ、G 在八个变体里都不中 | 看不到：P1 下原来那条记录被回退尝试自己的记录盖掉，替身记录指到的单元按孤儿被复用；P2 下是尝试那一刻读不出的根不受影子账保护，它的单元被复用。重做时剩下的只有根环 | U1「记录被新实例从前缀末 + 1 覆写」（P1）；P2 那一格是同一个载体（记录指到的单元）被「影子账只查可读根」拿走 |
| H11 | AJ：journal 环在一次回退窗口里转过一圈，小环上 1–8 圈（2.3） | 分辨：丙、CJ、G 都不中 | 看不到：那条记录已被覆写 | U1「journal 环在一次回退的窗口里转过一圈」 |
| H12 | CJ：最新 3–4 个 txg 的根与两份记录副本读不出，5 / 3 个故障（2.4、s6） | **不分辨**：每条 CJ 中的脚本 AJ 也中、G 也中（s2 `hit=CJ not_hit=AJ` 0、`hit=CJ not_hit=G` 0；s6 同）；丙在这一类里要的更少（2） | 看不到：那几个 txg 的根与记录全读不出；能找回最大 txg 的只剩全盘扫描单元头的诞生代号 | U1「丙按 (号, 出生代)」+「两块盘上同一条记录的槽同时读不出」+ 第一轮的「回退那一刻最新的根暂时读不出」 |
| H13 | CJ2：回退之后崩溃、重挂时同一类读故障，5 / 3（2.5） | **不分辨**：s5 / s6 `hit=CJ2 not_hit=AJ2` 0 | 同 H12 | U1「回退之后的挂载与切换（CJ2 / AJ2）」 |
| H14 | 甲 / 乙 / AJ / AJ2 / G：普通崩溃（没有回退）后最新的根与它那条记录的两份副本读不出，2 个故障，一次已确认的写丢掉、它的号被重发；丙、CJ、CJ2 在严格暖机下不中（2.8） | 形式上分辨（丙不中），但**不在 C143 的格里**：没有回退；病根是一次已确认写被 2 个故障丢掉 | 看不到：丢掉的那一代在盘上只剩单元 | 前提二「已发布的 inode 号单调不复用」的字面，不是 U1 的哪个分句；归哪一笔账由主 agent 判 |

**按跑前条款，我的判断**（交主 agent 评判，不是判决）：

- **AJ / AJ2 的载体被打中（退回）**，而且 H9、H10、H11 三形都分辨臂 ⇒ 反向接受条款第三条（「AJ / CJ 的载体被打中（退回）⇒ 那个收严出局，原臂按第一轮判决」）按字面触发：AJ 出局，AJ2 是它在挂载上的推广，同一个载体，一并出局；甲 / 乙回到第一轮判决 2（按判据字面在 U1 不成立）。
- **CJ / CJ2 的载体没有退回**：这一轮试的三种「载体本身退回」的形状（环在窗口里转过一圈、两块盘上同一条记录读不出、新实例从前缀末 + 1 覆写）里，只有第二种打得中 CJ，而且要同时把那几个 txg 的根也藏掉；打中的每一条同时打中 AJ 与 G（H12、H13 不分辨臂）⇒ 按「打中不分辨臂」处理，病根另记（第九节第 3 条）。CJ / CJ2 不出局；它们把丙在这一类里要的最少故障从 2 抬到 5（按槽）/ 3（按区间）。
- **丙本身**：这一轮没有一段分辨臂的历史打中它。s1–s6、s9 里 `hit=C not_hit=A` 全是 0；丙的 2 个故障历史（2.4、2.5）与第一轮 H4 / H5 同形。第一轮判决 1「丙中的每一条脚本甲也中」在带记录故障的空间里仍然成立。
- **收严只算线索**：G 是我这一轮提的，只在我的模型上量过，被攻过零轮。它关掉 H9–H11（头销毁与覆写都够不着全池的号水位），可它的判据是按号，余量是 0：s2 / s5 / s6 里 2 个故障就中，与 AJ 对活着的头一样。它还要改第一个事务的字节（第四节）。

## 六、U4：AJ / CJ / CJ2 与线索 G 各配什么会红的检查

四者都是跨挂载的性质，单镜像 checker 看不见（被抛弃的对象按实例表判未发布、不在任何可达的树里），都要多次挂载的录制流，与 C331（择根倒挂压过已确认的写）、C332（回退实例两个根都读不出时回退被撤销） 同一个前置。判定都一样：回退或挂载之后建的每个对象，身份（按各自判据）与此前任何一个写成了根的对象都不同。

| 臂 | 注入 | 判别力自证 |
|---|---|---|
| AJ / AJ2（若不按第五节出局） | ① 被抛弃时间线里销毁一个可写头，回退时盘 0 整块 + 盘 1 上那条记录的副本读不出（H9）；② 回退那次发布崩在记录之后，普通挂载一次，重做回退时带那个水位的根所在的盘整块读不出（H10）；③ 小环上 R_old 靠根槽写失败多留几圈（H11） | ①②③ 在今天的 AJ 下必须红；把 ① 里的「销毁」拿掉必须转绿（分得清「载体有界」与「注入没到读扫描」）；把 ② 改成 P2 且孤儿单元不复用，必须转绿 |
| CJ | 回退时盘 0 整块 + 盘 1 最新的根槽 + 盘 1 上最新 3 条记录的连续一段读不出（H12，s2 第 147 行那一段） | 今天的 CJ 必须红；去掉那段记录故障必须转绿，而同一注入下丙（不带 CJ）仍红——证明 CJ 真的在读记录；把 CJ 换回「可读根最大 + 1」，只注入盘 0 整块 + 一个根槽（2 个故障）必须转红 |
| CJ2 | 回退之后崩溃，重挂时同一类注入（H13，s5 第 621 行那一段） | 同上，另加：把普通挂载的新实例 txg 改回「从重放之后那一根往上数」，只注入盘 0 + 一个根槽必须转红（第一轮 H5 的形状） |
| CJ / CJ2 载体不退回 | 录制流上不注入读故障，逐次扫描记下「环里读得出的最大 checkpoint_txg」 | 这个量在无读故障的扫描之间单调不减；自证：造一个从更旧的前缀末开始写、且在读取最大值之前就覆写了最新记录的写者，必须红 |
| G（线索） | 同 AJ 的 ①②③，再加 s2 的「盘 1 整块 + 盘 0 上最新那条记录的副本」 | ①②③ 必须绿、最后那一条必须红（G 的余量是 0，自证分得清「G 关掉了头销毁」与「G 什么都关得掉」） |

## 七、什么现象会推翻我的每一条结论

| 结论 | 推翻它的观测 |
|---|---|
| 1 AJ 的载体有界（H9）：一个被销毁的头的最大水位只由 R_old 之后、销毁之前那几次发布的根与记录带着 | 一条条款让头销毁之后仍有随后续每次发布重写的载体带着它的水位（例如销毁时把它并进一个全池量，或树表条目在被抛弃的根离开环之前保留）；或回退拒绝复活在更新的根里已销毁的头 |
| 2 H10：回退尝试崩了再重做，AJ 1 个故障 | 条款把回退的「前缀末」定成整条可读链的末尾（P2），同时影子账也护回退那一刻读不出的环里根；或回退尝试的记录不落进环；或孤儿单元在被抛弃的根离开根环之前不许复用。三条缺一条，1 个故障的历史仍在（2.2 的表） |
| 3 H11：环在一次回退窗口里转过一圈 | 一条条款给环长一个相对根环覆盖范围的下界，或让根槽写失败的槽退出回退候选集（第一轮 R3 那一条）；或模型把「写失败的槽按旧内容算」（`16-发布语义.md:372`）建错了 |
| 4 CJ / CJ2 的载体没有读故障时不退回 | 一段没有读故障、而环里读得出的最大 checkpoint_txg 变小的可达历史；s1–s6 里没有 |
| 5 CJ 最少 5 / 3；CJ 中的每一条 AJ、G 也中 | 一段 CJ 中而 AJ 不中的历史（s2 约 17 万条脚本、s1 / s4 / s5 / s6 / s9 里 0 条）；或 journal 镜像的条款变了（副本多于两份、或与根同盘同偏移），余量要重算 |
| 6 s8 余量表 | 暖机规则不再是「覆盖两块盘」；归属不再写死 0 / 1 / 0；或两块盘同一偏移的记录副本会作为一个故障一起坏（同一批盘、同一个固件缺陷）——那时 CJ 按区间计的数要从 3 降到 2 |
| 7 宽读法在第一版里对 C143 不承重 | 一条让带新出生代的用户事务进入新实例覆盖两块盘之前那几次发布的路径：异步 API、「打开」在暖机之前交出可写句柄，或切换重做的事务拿到比切换前覆盖点更小的出生代 |
| 8 U2：今天没有只认号又不随根回退的消费者 | C118 定成一个只记号、不随根回退的结构；出现对外句柄 / 挂载面的条款；或第三节表里某一行的「随根回退」被条款改掉 |
| 9 U3：CJ / CJ2 零额外读 | 实现把回退写成不扫环（那时 CJ 补一遍 J 个记录头）；或恢复不再要求全环扫描（`23-journal的角色与格式.md:811`） |
| 10 U3：AJ / AJ2 最坏读量随 J 涨 | 一条把 R_old 与最新根之间的发布数限死的条款（R_old 不能靠写失败多留） |
| 11 K 代点删在跳号时留行（第四节，推理） | 条款或实现把丢弃写成「删掉全部 ≤ 当前代 − K 的代」、或跳号时补删；这一条我没核实现 |

## 八、复跑命令、跑的经过、文件 sha256

复跑（只用 Python 3.12 标准库；本机一次约 107 秒；输出只写 stdout；`-B` 不留 `__pycache__`）：

```
cd research/prompts/c143-r2-opus-model
python3 -B c143r2_model.py > "${TMPDIR:-/tmp}/c143r2_model.rerun.out"
cmp "${TMPDIR:-/tmp}/c143r2_model.rerun.out" c143r2_model.out && echo IDENTICAL
```

模型是确定性的（没有随机源，输出按排序打印），复跑一次只说明没有隐藏状态。最终版复跑一次与留存产物逐字节比，`IDENTICAL`。

跑的经过（源码是原地改的，旧版本没有另存；每次改了什么照实写）：

| 次 | 结果 | 改了什么 |
|---|---|---|
| 1 | 跑到 s5 时 `KeyError: 12`：重挂读故障多到选中了还没有头 12 的根（txg 0–2），用户发布找不到头 | 这类脚本记为无效（`user()` 返回 False），与第一轮把「回退到还没有头 12 的根」记为无效同一个处理 |
| 2 | 跑完。s4 有一格报「AJ 1 个故障」，诊断发现是计数错：回退尝试那次扫描藏掉的记录槽，被尝试自己的记录盖掉，重做那次扫描找不到它，于是 3 个故障的历史被记成 1 个。另外 s2 的前缀上限 4 让丙看起来最少 3——2 个故障那一形要 5 次发布的前缀（第一轮就是这个长度） | 故障计数改成「整段历史里被藏过的不同物理目标」（盘、根槽、某块盘上的某个记录槽），同一个目标只记一次；s2 前缀上限改成 5 |
| 3（最终） | 与第 2 次比，s2、s4 之外的输出逐行相同（diff 只差最后一行的行数计数） | — |

一次 `python3 -m py_compile` 在模型目录里留下过 `__pycache__/`，已删；仓里别的文件没有动。

文件 sha256 与行数（`sha256sum`、`wc -l` 原样输出）：

```
76fc64e1f5bc25dfd346ab15affe37843951d79b1d630d9eca3ccf2c3949a7e7  c143-r2-opus-model/c143r2_model.py
53a8043b1706168803456bc3d5ba1030d6c341efc302c487bc1dcd02a23a3049  c143-r2-opus-model/c143r2_model.out
   777 c143-r2-opus-model/c143r2_model.py
   976 c143-r2-opus-model/c143r2_model.out
  1753 total
```

## 九、没建模的、射程、建议另记的账

**没建模的**（每条写它会不会改结论）：

- journal 记录一份副本写失败：模型里两份总是一起写成。建进来 CJ 会更弱（一份没写成、另一份读不出就漏），可 AJ 读的是同一条记录，「CJ 中 ⇒ AJ 中」不受影响。
- 读故障按 txg 或「最新几条」挑，不是跨扫描持续的物理坏槽；跨两次扫描时按不同目标的并集计（s4），偏向少算。
- F、准入、defer 与真实分配器：单元复用取了对 AJ 最坏的读法（能复用就当场复用），只影响 AJ 那几形的「单元还在不在」；不复用的变体在 s4 的表里。
- 一个可写头加至多一个克隆头；S = 8（s3 为 4）；s3 的 J 只到 192；前缀至多 5 次发布；两块盘。
- 切换的 reload 读法（第一轮 H2）没重跑；AJ2 在切换时按 keep 读。
- 实例表行按第一轮的简化写；「在飞记录数上限」、只读挂载、独占打开过半设备都没建。
- K 代点删没建（第四节那一笔是推理）。

**射程**：各族的最少故障数是这些枚举空间里的最小值，不是定理；「CJ 中 ⇒ AJ 中」「丙中 ⇒ 甲中」是全部 sweep 里 0 例外的观测，推理支撑在 2.4 与第一轮 2.4。s8 是解析枚举，只建了「最新 L 个 txg 连续藏掉」这一种形状。

**建议另记的账**（这一轮打出来的、不分辨臂或不在 C143 格里的；立不立由主 agent 判）：

1. 回退的「前缀末」没有定义。`23-journal的角色与格式.md:1235` 说新实例从前缀末 + 1 接着写，回退又不施加任何记录；按字面（P1）回退的第一条记录盖在被抛弃时间线 R_old 之后第一条记录上，回退尝试一崩就把它抹掉（H10）。凡是靠被抛弃时间线的记录做见证的东西（AJ，以及将来对被抛弃时间线的任何审计）都受它影响。
2. 影子账只查「环里每一个可读根的账」（`23-journal的角色与格式.md:1206`）：回退那一刻读不出的被抛弃根，它引用的单元可以被回退那次发布当场复用；那个根之后又读得出、被选中时（C332 那一类），它指到的是被复用的单元。H10 的 P2 格就是它。与 C314（回退可以复用被抛弃的根引用的单元） 同族。
3. 回退与挂载只看得到读得出的见证，而每个 txg 的根只有一份、记录两份（H12、H13 与第一轮 H4、H5 同一个病根）。线索（被攻过零轮）：回退，以及 CJ2 下的挂载，遇到两份都读不出的记录槽时拒绝进行，或者先报告再由管理员决定。
4. 普通崩溃后最新的根与它记录的两份副本读不出（2 个故障），一次已确认的写丢掉、它的号被重发（H14 / s9）：属于「暖机只防单故障」那一类。
5. K 代点删在 txg 跳号时留下永远删不掉的行（第四节）。

## 附录：本报告当论证支点的 kb 行（整行，`sed -n` 机械抽取，没有手抄）

**`23-journal的角色与格式.md:1206`**

```text
**显式例外：管理员回退（2026-09-05，随 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。** 回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（2026-09-13，D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限；表的底版：2026-09-14 用户定案，C330（中间实例那一行的 T_pub 取所选根的 txg） 三轮三方）；第一个新根的 checkpoint_txg = 根环里全部根记录 txg 的最大值 + 1；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账（影子账；2026-09-13 用户定案，C314（回退可以复用被抛弃的根引用的单元） 取 F-A，E150（回退复用被抛弃的根引用的单元） 两格零违例、多隔离 2–3 个单元到被抛弃的最新根被轮转覆写为止；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进超级块的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行（2026-09-14 用户定案，C322（取号那一步的屏障怎么放没有条款） 三轮三方）；回退深度由候选集定（txg ≥ F_生效，2026-09-13 D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。E104（扫描重建的现行版本判定）：不写回退行时全规则臂复活 3，新根取 T_old + 1 时压不过被抛弃的根。
```

**`23-journal的角色与格式.md:1235`**

```text
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的「48 位撑 3202 年」本来就按全卷寿命算。
```

**`23-journal的角色与格式.md:697`**

```text
18. **「journal 在飞记录数上限」的取值与语义；tail 字段的编码（2026-09-13 用户定案；语义 2026-09-14 用户定案写死）：在飞记录数上限 = 环槽数 ÷ F（I-8.1（环几何够大） 的安全系数），第一版 196608 ÷ 3 = 65536；**语义 = 一次恢复重放的前缀最多 N 条**，不是 XFS 那种「最后 N 条内的坏校验和算撕裂、之外算损坏拒绝挂载」；**坏校验和一律断链即止**，第一版不区分撕裂与损坏、不因此拒绝挂载。tail 8 字节存 jsn 的 48 位计数器，靠超级块自己的两槽轮换取冗余，不另设 tail 槽；记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`。** 正文见 D23（journal 的角色与格式）「已定项 18」。 **状态：已定。**
```

**`23-journal的角色与格式.md:698`**

```text
19. **空发布记录的事务号与提交标记；previous_hash 里 header_csum 的处理；环长与切分纪律（2026-09-13 用户定案；② 与 ③ 2026-09-14 用户定案改写）：① 事务号 0 保留给不承载事务的记录（空发布），提交标记写 1，不进实例表 W 的 max，不进「丢掉提交标记没出现的尾巴」判定；② `previous_hash` = CRC32C(**本实例内逻辑前一条**记录的头，其中 `header_csum` 那 32 字节按零参与)，**本实例第一条恒 0**，写路径不读盘；③ 环长是 **mkfs 参数，默认 768 MiB**，约束 **环 ≤ 设备容量 ÷ 4**（越界拒绝 mkfs）；超级块仍只存环长字段本身；挂载时有效 `T_dirty` = min(`T_dirty` 字段, 环长 ÷ F)；超级块里 `T_dirty` 仍写 2 GiB，默认 768 MiB 环下有效值 = min(2 GiB, 256 MiB) = 256 MiB；单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）。** 正文见 D23（journal 的角色与格式）「已定项 19」。 **状态：已定。**
```

**`23-journal的角色与格式.md:811`**

```text
**必须先全环扫描、逐条验证，再择最长合法前缀。**
```

**`22-单元原子性怎么合成.md:229`**

```text
| 8 | **固定结构的放置** | **已定（2026-09-02，用户定案 + E87（固定结构的放置））：超级块每盘一份（更新走 ≥2 槽轮换）；journal 环两盘镜像（D2（RAID 条带策略）已定项 6 的 w≥2 下限对 journal 记录写同样生效）；mkfs 把第 0 代根种进全部根环区域。** 正文见 D22（单元原子性怎么合成）「已定项 8」 **状态：已定。** |
```

**`22-单元原子性怎么合成.md:1013`**

```text
4. **根环三个区域的设备归属第一版写死 0 / 1 / 0**（区域 0 → 盘 0、区域 1 → 盘 1、区域 2 → 盘 0），mkfs 逐区域把设备身份写进超级块的「根环逐区域设备身份」12 字节（D2（RAID 条带策略） 已定项 7）。⚠️ **它不是 mkfs 参数**（2026-09-14 用户定案）：第一版 devs = 2、R = 3，按 D22（单元原子性怎么合成） 已定项 14 的鸽笼下界（取满 min(R, devs) = 2 个不同值、没有一块盘背超过 ⌈3/2⌉ = 2 个区域），只有「两个区域在一块盘、一个在另一块」这一种形状；再按 E87（固定结构的放置） 逐字「两盘上 R = 3 必有一盘背两个区域」，区域编号一定之后 0 / 1 / 0 是唯一的规范写法。把它做成 mkfs 参数，买到的是「哪块盘背两个」这一格自由度，而那一格在两盘同构下没有可观测差别，代价是多一个要在挂载时校验、要在加盘时重算的参数。
```

**`22-单元原子性怎么合成.md:1014`**

```text
5. **暖机空发布 2 次、第一个事务 txg 3 是格式常量**，不是运行时谓词（`format-const` WARM_UP_EMPTY_PUBLISHES = 2、FIRST_TRANSACTION_TXG = 3，登记在 [first-txn-layout.md](../first-txn-layout.md) 八）。依据：D16（发布语义） 已定项 8 要求「推到本实例的根覆盖两块盘为止」，而在 0 / 1 / 0 的归属下 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定，写成常量比写成运行时谓词少一条只有一个取值的分支。⚠️ 归属改了这两个常量要重算（加盘或开条带表时）。
```

**`16-发布语义.md:207`**

```text
**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布，第一版几何至多 R = 3 次。前置：C314（回退可以复用被抛弃的根引用的单元） 先还。
```

**`16-发布语义.md:372`**

```text
| 环里最旧有效根 | 盘上全部根槽里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg；写失败的槽按旧内容算，在飞、没持久的发布不算（C282（环里最旧根没有定义） 要的定义） |
```

**`08-核心索引结构.md:386`**

```text
克隆头：新头的水位 = origin 那一刻的运行时水位（含还在 write buffer 里没落盘的那条），建克隆那次发布必须为新树写出这条水位行；两个头此后各自分配，号会相交，唯一性靠 (树 ID, inode)。
```

**`06-快照实现模型.md:240`**

```text
- 靠 D5（快照 / 空间记账机制） 同日补的两条：还有自己活快照的可写头不许销毁；分叉点闸的 `children` 数克隆。
```

**`17-实现分层与第三方管道.md:95`**

```text
| 5 | **第一版的运行载体** | **已定（2026-09-03，用户定案）：纯用户态库 + 块设备抽象（文件与 `O_DIRECT` 裸设备两个后端），第一版不挂载；FUSE 等目录与 POSIX 面出现后再加，内核模块暂不做。** 正文见 D17（实现分层与第三方管道）「已定项 5」 **状态：已定。** |
```


## 补记：交付规则上的一处偏差

模型源码的第一段（`c143r2_model.py` 第 1–165 行）是一次工具调用写进去的，超过了任务书「每一次工具调用写进文件的内容不超过 150 行」的上限；源码其余五段与报告的九段都在 150 行以内。源码后来的三处修补（第八节「跑的经过」第 1、2 次）用 Python 做定点替换，每处先断言旧串恰好命中一次，再由脚本回写整份文件。
