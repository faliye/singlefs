# m3-w3-r2 辩方腿（Sonnet）报告：K1（2026-10-08）

轮名 m3-w3-r2；背景材料 `research/prompts/_m3-w3-r2-background.md`（正文 `research/prompts/_m3-w3-r2-body.md`）；被复核的判决 `research/prompts/m3-w3-r1-main-verification.md`；第一轮腿报告 `research/prompts/m3-w3-r1-opus-output.md`、`research/prompts/m3-w3-r1-sonnet-output.md`。
冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`（只读）；草稿 `~/.local/share/singlefs/work/m3-w3-r2-sonnet/`；模型 `research/prompts/m3-w3-r2-sonnet-model/`。
立场：替被第一轮判出局的一方辩护，只答 K1。副本模型上量出的数不是入库装置上的数；我提的形态只在我的模型上量过、被攻过零轮。

## 一句话

K1 各项（详见第二节表，数在第三、四节，推翻条件在第六节）：

1. C1 出局：站得住。C1a 站得住；C1b（环里最大的）也站得住，但「两腿一致」只对 C1a 成立，C1b 的漏是这一轮第一次量。
2. 只记峰值的 C2 出局：站得住（C2c 漏 op95）。最强形态 C2d 在第一轮打中它的格上不中，判决拿来出局它的两处例子 I-9.29 是红的；站得住的理由要换成持久镜像（事件之后的镜像里重发的对象还活着，现有检查也不红）。
3. 每实例一行的 C2 出局：站得住（C2b 两格都漏）。最强形态 C2i 同 C2d，漏得少。
4. C4 出局：攻方腿定义的 C4 站得住；最强形态 C4g（谱系记录多带回退前水位，判加「(号, 出生代) 号 < W 且出生代 > T_rb」）站不住：第一轮打中 C4 的格不中，模型与装置上洞事件、持久镜像都漏 0，P 臂误报 0。C4g 只管回退引起的水位退回，V3 臂漏 12,056/76,552。
5. C2a 与 C3r 被 K3 打中的两形：同样打中所有留下的候选（单个存储、同一个写者的都是 9,534/9,534），挂起不分候选。
6. V2：「全漏」站不住。它是同一条时间线上的重发，但现有 I-9.6 在每一个 V2 事件和持久镜像上都红，洞是 0，不在 W3 要补的块里。

（本文件分段追加。）

## 复跑

模型目录 `research/prompts/m3-w3-r2-sonnet-model/`（`README.md` 写了每个文件是什么）。在仓根，`W` 取一个空目录，每段一条前台命令（各在 4 分钟以内），编译与跑都经 `research/scripts/run-with-memory-cap.sh`（编译 6G、跑 4G），线程 6，只跑 harness 档：

```
M=research/prompts/m3-w3-r2-sonnet-model; W=<空目录>
for p in derive build-model build-P build-V1 quick model-main model-defense model-mutation model-ringwin model-replay model-deep \
         dev-g-long-v1-idx24 dev-g-long-v1 dev-f2-v1 dev-g-long-p dev-f2-p-1 dev-f2-p-2 compare; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
```

交回前在空目录 `~/.local/share/singlefs/work/m3-w3-r2-sonnet/rerun1/` 从头跑通一次，最后一段 `compare` 末行 `compared=37 status=0`；`expected/` 是更早一次独立跑出来的输出（旧二进制、同逻辑）整理的，不是这一次跑的，所以这一次是对着它比。
抽象模型是攻方腿的 `timeline-model`（`research/prompts/m3-w3-r1-opus-model/timeline-model/`）加补丁；装置是 E175 装置源码原样加攻方腿的影子模块再加我的几种形态。`rerun.sh derive` 从攻方腿的三份文件重新生成我的三份并逐字节比。
副本装置上的数不是入库装置上的数；我加的几种形态只在这两套模型上量过、被攻过零轮。

## 一、我加了什么

### 1. 模型里改的（相对攻方腿的模型，`rerun.sh derive` 的五个补丁脚本）

- 墓碑（跟现行那一版走、不回收）与谱系树 (T_rb, T_old)（跟现行那一版走）：删除那一步追加 (号, 出生代, 死亡 txg)，回退那一步追加 (T_rb, T_old)，崩溃恢复取所选那一版的。依据 D18 已定项 24 第 4、5 条与 D23 已定项 14。
  - 「4. **回收**：第一版不回收墓碑，统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记；10⁶ 次 unlink 的墓碑约 76.7 MiB（两份）。」（`.claude/kb/decisions/18-块里携带什么信息.md:575`）
  - 「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）
  - 代码：回退发布的树表条目里只有三种换成目标那一版的，别的（含墓碑树、谱系树）取现行那一版：「|| entry.kind == TREE_KIND_DELETED_INODES」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7512`）；回退那次追加谱系记录：「let lineage_record = LineageRecord {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7800`）；记录今天 16 字节：「pub struct LineageRecord {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/records.rs:184`）。
- 「现有检查」两条改成产品口径：I-9.29t = 候选集各版的 inode 树加「谱系上」的墓碑（死亡 txg 在这一版父链上）合起来同号只许一个出生代；I-9.6t = I-9.6 的树内最大 key 一半加墓碑一半（水位 > 最新根墓碑树里全部对象号）。攻方腿的 I-9.29、I-9.6 只看树，仍并列在表里。
  - 「每一版的 inode 记录的出生代与那一版谱系上墓碑的出生代都算」（`.claude/kb/invariants.md:317`）
  - 「**两半都判**：水位 > 树内最大 key；墓碑那一半（2026-10-05 起）水位 > 所选根（最新的那条根）墓碑树里已发布的」（`.claude/kb/invariants.md:294`）
  - 代码：「judgements.judge("I-9.6", watermark > largest_inode, || {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11676`）与墓碑那一半「judgements.judge("I-9.6", watermark > largest_tombstone_object, || {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11699`）。
- 「洞」= 现有检查（I-9.29t 与 I-9.6t）都不红。第一轮表里的数是「候选单独红不红」；这一轮另报「现有检查都不红的那些事件 / 镜像里，候选红不红」，也就是 W3 真要补的那一块。装置上用真 checker 的 I-9.29 与 I-9.6 判定。
- 「持久镜像」= 事件（同一条时间线上的重发）之后的镜像里，被重发的那个 (号, 出生代) 还活在最新根的 inode 树里。模型里叫 `residual`，装置里叫 `residual_steps`。
- 加了第三个阳性对照臂 V3（恢复时新实例的水位比所选根的少 1，与回退无关）；`W3_OPS`、`W3_LONGEST`、`W3_RING` 三个环境变量把历史拉到超过环长；`replay` 模式按给定序列在给定环长上逐步打印。

### 2. 候选的定义（单镜像上怎么判红）

| 名字 | 对应第一轮 | 写什么、谁写、什么时候写 | 红的条件 |
|---|---|---|---|
| C1 | 候选一（攻方腿的 C1） | 实例表行预留字节记 M = 恢复所选那一版的水位 | 水位 < 某行的 M |
| C1b | 候选一（正推腿的 C1b） | 行第一次写的那一刻，M = 根环里读得出的、该实例且 txg ≤ T 的根的水位最大值 | 水位 < M，或 inode 树里一条 (号, 出生代) 号 < M 且出生代 > T |
| C1bk | C1b 加一条 | 同 C1b，清扫删行时那一行的 M 仍留着（上界） | 同 C1b |
| C2b | 候选二「每实例最后一代」 | 每个实例一行，后写覆盖 | 水位 < 某行 |
| C2c | 只记峰值 | 一个峰值数 | 水位 < 峰值 |
| C2d | 只记峰值（连同代） | (峰值, 第一次到它的代) | 水位 < 峰值，或 (号, 出生代) 号 < 峰值且出生代 > 那个代 |
| C2i | 每实例一行（正推腿的 C2inst） | 每实例 (最高水位, 第一次到它的代) | 同 C2d，每个实例各判一次 |
| C4 | 候选四（攻方腿的 C4） | 谱系记录多带回退前 cur 的水位，取最大 | 水位 < 最大值 |
| C4g | 候选四最强形态（我加的） | 谱系记录多带 (回退前 cur 的水位 W)，16 → 24 字节，随现行那一版，回退那次追加一条（推的：不动别的发布） | 水位 < W，或 (号, 出生代) 号 < W 且出生代 > T_rb |
| C2ig | 我加的组合 | C2i 与 C4g 各存各的，红一处即红 | 任一红 |
| C2a / C3r | 留下的两个 | 历代 (代, 水位) 行 / 根下只追加的 (号, 出生代)，回退那次从 cur 带 | 同第一轮 |

## 二、K1 各项判一览

先说两个口径，后面的表都按它们读：

- 第一轮表里「红不红」是候选单独红不红（比如 C1 在 G-long V1 的 476 个事件里红 70 个）。W3 补的是现有检查（I-9.29、I-9.6）看不见的那一块，所以这一轮同时报「洞」：现有检查都不红的事件 / 镜像里候选红不红。装置 G-long V1 32 段里洞事件只有 2 个（第 24 段 op92、op95），F2 V1 64 段里 0 个，洞的主体是事件之后的持久镜像（G-long 221 步、F2 37 步）。
- 判决 `research/prompts/m3-w3-r1-main-verification.md:26` 的「C4a」是正推腿的根记录带 (最高水位, 第一次到它的 txg)，就是我这里的 C2d；攻方腿的 C4（谱系带回退前水位）是我这里的 C4。

| 判决条目 | 辩方判定 | 证据（量在第三、四节，可复跑） |
|---|---|---|
| C1 出局：「C1 实例表行记水位（所选根的 / 恢复时环里最大的）」「**出局**，两腿一致」（`research/prompts/m3-w3-r1-main-verification.md:23`） | 站得住。「两腿一致」只对 C1a（行记所选那一版的水位）成立：攻方腿的 C1 就是 C1a，C1b（环里最大的）是正推腿提的、攻方腿没攻过，C1b 的漏是这一轮第一次量。 | C1a：洞事件 0/2，持久洞步 0/221（G-long）与 0/37（F2）；同两格 C2a、C3、C4g、C1b、C2i 都 2/2，所以分得出臂。判别子（回退前的高水位）不在行里：行里存的是恢复所选那一版的水位，回退已经把它压低。字面落在「同一条时间线上重发而判不红」。C1b：同两格 2/2 不中，但模型里洞事件漏 16,262/138,372（回退之后没挂载过的 25,838 个里漏 13,026，挂载过的 112,534 个里漏 3,236）；装置持久洞步漏 9/221 与 15/37；环窗口的起点是历史长度 = 环长 + 3（第三节第 4 小节），环长 24 上一条合法序列 C1b 不红而 C2a、C2d、C2i、C3r、C4g 红。 |
| 只记峰值的 C2 出局：「只记峰值漏 op95」（`research/prompts/m3-w3-r1-main-verification.md:24`） | 站得住（C2c）。最强形态 C2d（峰值连同代）在第一轮打中它的格上不中；判决拿来出局它的两处例子（「越过旧峰值之后不红」）I-9.29 都是红的，不是洞；站得住的理由要换成持久镜像。 | C2c：洞事件 1/2（op95 不红），持久洞步 3/221。C2d：洞事件 2/2，持久洞步装置 G-long 4/221、F2 0/37，模型持久镜像漏 120,864/352,888（34%）。两处旧例子的 I-9.29 状态：第一轮攻方腿 `research/prompts/m3-w3-r1-opus-model/expected/device-crafted-V1.out` 第 19 行是 `i9_29=violated`，正推腿 `research/prompts/m3-w3-r1-sonnet-model/out/d-V1.out` 第 48 行也是 `i9_29=violated`。 |
| 每实例一行的 C2 出局：「每实例一行」（`research/prompts/m3-w3-r1-main-verification.md:24`） | 站得住。C2b（每实例最后一代）两格都不中；最强形态 C2i（每实例最高水位连同代）在那两格上中，漏在持久镜像上，漏得比 C2d 少。 | C2b：洞事件 0/2，持久洞步 0/221。C2i：洞事件 2/2；持久洞步 G-long 216/221、F2 26/37；模型持久镜像漏 13,486/352,888（3.8%）。 |
| C4 出局：「C4（谱系带回退前水位）漏 op95」（`research/prompts/m3-w3-r1-main-verification.md:26`） | 对攻方腿定义的 C4（只判水位 < 回退前最大水位）站得住；对最强形态 C4g（谱系记录带回退前水位，判加「(号, 出生代) 号 < W 且出生代 > T_rb」那一半）站不住：第一轮打中 C4 的 op95 在 C4g 上中，我量到的所有格 C4g 都没漏、没误报。C4g 有自己的限度（下面三条）。 | C4：洞事件 1/2，持久洞步 3/221。C4g：洞事件 2/2、持久洞步 221/221（G-long）与 37/37（F2）；模型洞事件漏 0/138,372、持久镜像漏 0/352,888（环 4、6、8 各一组都是 0）；P 臂误报：模型 3 组 33,245,604 张镜像 0（其中合法的跨被抛弃时间线重发 154,778 次），装置 G-long P 64 段 9,664 步与 F2 P 256 段 15,616 步 0。限度：① 只管回退引起的水位退回，V3（恢复时水位少算 1）臂洞事件漏 12,056/76,552（C2ig = C2i 加 C4g 漏 0）；② 回退那次从 R_old 带 / 不写，同样不红（见下一行）；③ 谱系记录 16 → 24 字节是推的，没量。 |
| C2a 与 C3r 被 K3 打中那两形（回退从 R_old 带历史、旁证挤出环）是不是同样打中所有留下的候选 | 是。凡是把历史存在根下 / 谱系里、由同一个写者写的候选（C2a、C3r、C2d、C2i、C4g、C2ig）在 V1 + A、B、C 三组上洞事件漏的数一模一样（9,534/9,534）；交叉核的 C2ax、C3rx 只在被回退抛弃那一段的根还在环里时红，挤出环之后同样漏。 | 第三节第 5 小节的 K3 表。挂起不把 C2a、C3r 和 C4g 分开；K3 要的是「历史写错或不写必须红」，要两处独立来源互核（C2ay 一类），单个存储做不到。 |
| V2 那一形：「V2 那一形（第二种水位开关）两个都全漏」（`research/prompts/m3-w3-r1-main-verification.md:52`） | 「全漏」站不住：V2 是同一条时间线上的重发（与 V1 一样是产品路径走不到的阳性对照），但现有 I-9.6 在每一个 V2 事件那一步和之后每一张镜像上都红，洞是 0。它不在 W3 要补的块里；C3r 能抓到它是额外的好处，不分辨候选。 | 模型 V2 臂：事件 1,523,216 / 1,526,840 个、持久镜像 1,986,492 / 2,024,072 张，现有检查都不红的 0 / 0。E175 V2 臂 5,225 个事件全是「还在」且 I-9.29 红。机理：V2 建 inode 不加水位，新建的对象号 ≥ 水位，I-9.6 树内最大 key 那一半当场红；对象被删后墓碑留着（第一版不回收墓碑），墓碑那一半红。 |

## 三、抽象模型上的数（命令与原样输出）

读法：`FP` 是合法镜像上判红的张数；`evt_miss` 是事件（同一条时间线上的重发）那一步候选不红的次数；`evt_miss_exist` 是其中现有检查（I-9.29t 与 I-9.6t）都不红的那些（洞事件）里候选不红的次数，`exist_after_mount`、`exist_no_mount` 把它按「自最近一次回退起挂载过没有」分开；`resid` 是持久镜像（事件之后被重发的对象还活着）里候选不红的张数，`resid_exist` 是其中现有检查都不红的；`mut_undet` 是变异组用的，这里是 0。头一行是这一组的总数（环 4 槽、历史长 7、V1 臂、清扫阈值「从不」）。

### 1. V1 臂：第一轮打中的几个候选与它们的最强形态

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29t,I-9.6t,C1,C1b,C2b,C2c,C2d,C2i,C4,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-0.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-0.out arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=7 images=10836269 legal_images=7914147 events=340776 events_where_I-9.29_holds=144904 events_where_existing_checks_silent=138372 events_existing_silent_mounted_since_the_last_rollback=112534 events_existing_silent_not_mounted_since_the_last_rollback=25838 residual_images=443952 residual_where_existing_checks_silent=352888
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29t    0    138764          138372             112534           25838 367088       352888          0         0
  I-9.6t     0    313424          138372             112534           25838 421416       352888          0         0
  C1         0    335560          136804             111238           25566 438352       350392          0         0
  C1b        0     71294           16262               3236           13026 127938        98822          0         0
  C2b        0    335560          136804             111238           25566 438352       350392          0         0
  C2c        0    297712          124116             100734           23382 396848       328928          0         0
  C2d        0         0               0                  0               0 142136       120864          0         0
  C2i        0         0               0                  0               0  17324        13486          0         0
  C4         0    297712          124116             100734           23382 396848       328928          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
```

同一组在环 6、环 8 上（C1b 挂载之后的漏随环长掉到 0，没挂载过的漏不掉；C2d、C2i 的持久镜像漏随环长慢慢降但不到 0；C4g、C2a、C3r 一直 0）：

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-1.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-1.out arm=RollbackTakesTheTargetsWatermark mutation=None ring=6 sweep_threshold=never longest=7 images=11444142 legal_images=8295946 events=366236 events_where_I-9.29_holds=56488 events_where_existing_checks_silent=54264 events_existing_silent_mounted_since_the_last_rollback=39524 events_existing_silent_not_mounted_since_the_last_rollback=14740 residual_images=454280 residual_where_existing_checks_silent=158272
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  C1b        0     71672            8202                  0            8202 128658        46726          0         0
  C2d        0         0               0                  0               0 144680        50280          0         0
  C2i        0         0               0                  0               0  17540         5952          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-2.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-2.out arm=RollbackTakesTheTargetsWatermark mutation=None ring=8 sweep_threshold=never longest=7 images=11444772 legal_images=8296576 events=366236 events_where_I-9.29_holds=44848 events_where_existing_checks_silent=44704 events_existing_silent_mounted_since_the_last_rollback=31688 events_existing_silent_not_mounted_since_the_last_rollback=13016 residual_images=454280 residual_where_existing_checks_silent=109108
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  C1b        0     71672            8058                  0            8058 128658        32674          0         0
  C2d        0         0               0                  0               0 144680        27488          0         0
  C2i        0         0               0                  0               0  17540         3914          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
```

### 2. P 臂：合法镜像上的误报（含合法的跨被抛弃时间线重发）

三组合计 33,245,604 张镜像，全部是合法镜像（`legal_images` = `images`），其中跨被抛弃时间线的合法重发 48,356 + 35,042 + 71,380 = 154,778 次；这几个候选 `FP` 全是 0（25 个候选 × 3 组的 `FP` 合计也是 0）。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-main-0.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-main-0.out arm=Product mutation=None ring=4 sweep_threshold=never longest=7 images=10836269 legal_images=10836269 events=0 events_where_I-9.29_holds=0 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=0 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  C1b        0         0               0                  0               0      0            0          0         0
  C2d        0         0               0                  0               0      0            0          0         0
  C2i        0         0               0                  0               0      0            0          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-main-1.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-main-1.out arm=Product mutation=None ring=4 sweep_threshold=0 longest=7 images=10964563 legal_images=10964563 events=0 events_where_I-9.29_holds=0 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=0 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  C1b        0         0               0                  0               0      0            0          0         0
  C2d        0         0               0                  0               0      0            0          0         0
  C2i        0         0               0                  0               0      0            0          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-5.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-5.out arm=Product mutation=None ring=8 sweep_threshold=0 longest=7 images=11444772 legal_images=11444772 events=0 events_where_I-9.29_holds=0 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=0 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  C1b        0         0               0                  0               0      0            0          0         0
  C2d        0         0               0                  0               0      0            0          0         0
  C2i        0         0               0                  0               0      0            0          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
```

### 3. V2 臂与 V3 臂

V2（发布不加新建个数）：现有检查的 I-9.6（只看树的那一条，攻方腿的原口径）和 I-9.6t 的 `evt_miss`、`resid` 都是 0，洞是 0；I-9.29 单独漏的 172,652 个事件，I-9.6 都接得住。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29,I-9.6,I-9.29t,I-9.6t,C1b,C2a,C3r,C4g research/prompts/m3-w3-r2-sonnet-model/expected/model-main-4.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-main-4.out arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=never longest=7 images=10775745 legal_images=1789285 events=1523216 events_where_I-9.29_holds=172652 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=1986492 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29     0    172652               0                  0               01048980            0          0         0
  I-9.6      0         0               0                  0               0      0            0          0         0
  I-9.29t    0    104732               0                  0               0 904624            0          0         0
  I-9.6t     0         0               0                  0               0      0            0          0         0
  C1b        0   1523216               0                  0               01986492            0          0         0
  C2a        0   1523216               0                  0               01986492            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
  C4g        0   1523216               0                  0               01986492            0          0         0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29,I-9.6,I-9.29t,I-9.6t,C1b,C2a,C3r,C4g research/prompts/m3-w3-r2-sonnet-model/expected/model-main-5.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-main-5.out arm=PublishDoesNotAddTheNewInodes mutation=None ring=4 sweep_threshold=0 longest=7 images=10903742 legal_images=1850236 events=1526840 events_where_I-9.29_holds=202368 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=2024072 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29     0    202368               0                  0               01323912            0          0         0
  I-9.6      0         0               0                  0               0      0            0          0         0
  I-9.29t    0    119668               0                  0               01156017            0          0         0
  I-9.6t     0         0               0                  0               0      0            0          0         0
  C1b        0   1526840               0                  0               02024072            0          0         0
  C2a        0   1526840               0                  0               02024072            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
  C4g        0   1526840               0                  0               02024072            0          0         0
```

V3（恢复时新实例的水位比所选根的少 1，与回退无关）：C4g 漏 12,056/76,552 个洞事件，C2ig（C2i 加 C4g）漏 0；这一臂的 `FP` 列没有意义（水位被改低，「合法」的定义在这一臂里不成立），不读。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29t,I-9.6t,C1,C1b,C2c,C2d,C2i,C3r,C4,C4g,C2a,C2ig research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-4.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-defense-4.out arm=RecoveryLowersTheWatermark mutation=None ring=4 sweep_threshold=never longest=7 images=10825489 legal_images=5323861 events=648336 events_where_I-9.29_holds=127848 events_where_existing_checks_silent=76552 events_existing_silent_mounted_since_the_last_rollback=76552 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=873440 residual_where_existing_checks_silent=190236
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29t    0     79720           76552              76552               0 372556       190236          0         0
  I-9.6t     0    604264           76552              76552               0 430748       190236          0         0
  C1      52474    598192           73504              73504               0 430052       189660          0         0
  C1b     62346         0               0                  0               0      0            0          0         0
  C2c     52474    598192           73504              73504               0 430052       189660          0         0
  C2d     61282         0               0                  0               0 223440        86264          0         0
  C2i     62346         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
  C4      9214    642336           73792              73792               0 818616       189876          0         0
  C4g     10218    508048           12056              12056               0 767560       149416          0         0
  C2a     62346         0               0                  0               0      0            0          0         0
  C2ig    62346         0               0                  0               0      0            0          0         0
```

### 4. C1b 的环窗口：回退之后环转了一圈才挂载

只用 create、rollback1、empty、mount 四个字母（`W3_OPS`），V1 臂，把历史拉到超过环长。每一行：这一组里现有检查都不红的事件，按挂载过没有分开的数（斜线前后），C1b 在这两类里漏的数，C4g 漏的数，C2d、C2i 持久镜像里漏的数。每个环长，历史长度 = 环长 + 2 时挂载之后的漏是 0，= 环长 + 3 起不再是 0（4:6 → 4:7，6:8 → 6:9，8:10 → 8:11）；环长 24 的产品上没有这样枚举，走了一条具体序列（下一小节）。

```
$ （每个文件取 C1b、C4g 两行的 evt_miss_exist / exist_after_mount / exist_no_mount / resid_exist，W3_OPS=create,rollback1,empty,mount，V1 臂，清扫阈值从不）
ring:depth  events_existing_silent(after_mount/no_mount)  C1b 漏(after_mount/no_mount)  C4g 漏  C2d_resid_exist  C2i_resid_exist
   4:6           21/21             0/12       0/0         69         44
   4:7          114/114            4/58       0/0        491        282
   4:8          546/546           32/254      0/0       2985       1556
   6:8          189/189            0/48       0/0       1812        813
   6:9         1026/1026          16/232      0/0      11840       4961
  6:10         4914/4914         128/1016     0/0      67257      26381
  8:10         1701/1701           0/192      0/0      36453      12661
  8:11         9234/9234          64/928      0/0     229667      76161
```

环长 24 上走两条合法序列（`rerun.sh model-replay`）。第一条回退之后 23 次空发布让环转了一圈、挂载、再建一个：C1b、C1、C2c、C4 与现有检查都不红，C2a、C2d、C2i、C3r、C4g 红。第二条建、回退、重发、再建一个让水位越过旧峰值、再 24 次空发布让现有检查忘掉：C2d、C2i 也不红，只剩 C2a、C3r、C4g 红。

```
$ （expected 里的逐步打印行按步号排，取 step ≥ 25 的、只留操作与判红名单；V1 臂、环长 24；现有检查 I-9.29 / I-9.29t / I-9.6 / I-9.6t 不在名单里 = 不红）
序列 event：create,rollback1,empty×23,mount,create
  step=25 op=mount red=[C2a,C2c,C2d,C2i,C3r,C4,C4g,C2ax,C3rx,C2ay,C2dg,C3rg,C2ag,C2ig,C1bg,C4gk]
  step=26 op=create red=[C2a,C2d,C2i,C3r,C4g,C2ax,C3rx,C2ay,C2dg,C3rg,C2ag,C2ig,C1bg,C4gk]
序列 persist：create,rollback1,create,create,empty×24
  step=25 op=empty red=[C2a,C3r,C4g,C2ax,C3rx,C2ay,C2dg,C3rg,C2ag,C2ig,C1bg,C4gk]
  step=26 op=empty red=[C2a,C3r,C4g,C2ax,C3rx,C2ay,C2dg,C3rg,C2ag,C2ig,C1bg,C4gk]
  step=27 op=empty red=[C2a,C3r,C4g,C2ax,C3rx,C2ay,C2dg,C3rg,C2ag,C2ig,C1bg,C4gk]
```

### 5. K3 的两形是不是同样打中所有留下的候选

变异：A = 回退那次历史取 R_old 那一版的，B = 历史只照抄这一版自己的，C = 历史不写（攻方腿的定义，环 4、清扫阈值 0、长 6，每组 978,878 张镜像）。`P+A` 等三列是「历史被改过的镜像里不红的 / 全部」，`V1+A` 等三列是 V1 臂洞事件（共 9,534 个）里候选不红的个数。C2a 与 C3r 的数与第一轮攻方腿的 `model-mutation.out` 一致（593,287、217,753 等）。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/mut_table.py research/prompts/m3-w3-r2-sonnet-model/expected
mutations: P+A:HistoryFromTheTarget/Pr images=978878 events_existing_silent=0 P+B:HistoryCopiesThisVersion/Pr images=978878 events_existing_silent=0 P+C:HistoryNotWritten/Pr images=978878 events_existing_silent=0 V1+A:HistoryFromTheTarget/Ro images=978878 events_existing_silent=9534 V1+B:HistoryCopiesThisVersion/Ro images=978878 events_existing_silent=9534 V1+C:HistoryNotWritten/Ro images=978878 events_existing_silent=9534
candidate                   P+A                   P+B                   P+C            V1+A            V1+B            V1+C
C1b                         0/0                   0/0         961526/961526            1130            1130            9534
C1bk                        0/0                   0/0         961526/961526            1064            1064            1064
C2a               593287/593287         978878/978878         978878/978878            9534            9534            9534
C2ax              155355/593287              0/978878              0/978878            6338               0               0
C2ay                   0/593287              0/978878              0/978878               0               0               0
C2d               164784/164784         798106/798106         978878/978878            9534            9534            9534
C2i               408214/408214         976024/976024         978878/978878            9534            9534            9534
C2ig              488811/488811         978241/978241         978878/978878            9534            9534            9534
C3r               217753/217753         338149/338149         978878/978878            9534            9534            9534
C3rx               76091/217753         112490/338149              0/978878            8378            8278               0
C4                196430/196430                   0/0         593287/593287            9534            8646            9534
C4g               336805/336805         488244/488244         593287/593287            9534            9534            9534
```

读法：把历史存在根下或谱系里、由同一个写者写的所有候选（C2a、C2d、C2i、C3r、C4g、C2ig）在 V1 + A、B、C 上都是 9,534/9,534；交叉核的 C2ax、C3rx 在 V1 + A 上仍漏 6,338、8,378（被回退抛弃那一段的根挤出环之后看不见旁证）；只有 C2ay（历代行与谱系两处互核）三组都是 0。C1b 的行不跟根走，A、B 少漏（1,130）、C 一样漏。所以 K3 的两形不分候选，挂起不能拿来把 C2a、C3r 排在 C4g 前面。

## 四、E175 装置上的数（真代码走出来的根链，真 checker 的 I-9.29 与 I-9.6）

读法：每个候选一行。`q1_red` 是同一条时间线上的重发事件里红的个数（第一轮表里的数，候选单独红）；`gone_red` 是其中持有旧出生代的根已不在候选集（Q9 = Gone）的事件里红的个数；`hole_red` 是现有检查（真 I-9.29 与 I-9.6）都不红的事件里红的个数；`residual_red` 是事件之后被重发的对象还活着的步里红的步数，`residual_hole_red` 是其中现有检查都不红的步里红的步数。头一行 `hole_events` 与 `residual_hole_steps` 是分母。
这四个文件是 `rerun.sh dev-*` 各段的摘要（`expected/device-*.summary` 的前 13 行；后面还有事件行与逐步打印，不贴）。G-long V1 的 `q1_red` 一列与第一轮攻方腿的 `awk -f sum.awk` 合计一致（c1 70、c2a 476、c2c 233、c2d 476、c4 233）。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/agg.py（research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-V1.summary 的前 13 行）
histories=32 q1_events=476 gone_events=6 hole_events(I-9.29 与 I-9.6 都不红)=2 residual_steps=3234 residual_hole_steps=221 steps_judged=4832
cand   q1_red  gone_red  hole_red  residual_red  residual_hole_red  red_steps
c1          70         0         0           393                  0        574
c2a        476         6         2          3234                221       4070
c2b         70         0         0           393                  0        574
c2c        233         1         1          1350                  3       1943
c2d        476         6         2          1823                  4       2659
c3         476         6         2          3234                221       4095
c4         233         1         1          1350                  3       1943
c1b        423         6         2          2993                212       3718
c2i        476         6         2          3080                216       3916
c4g        476         6         2          3234                221       4070
c2ig       476         6         2          3234                221       4070
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/agg.py（research/prompts/m3-w3-r2-sonnet-model/expected/device-F2-V1.summary 的前 13 行）
histories=64 q1_events=336 gone_events=5 hole_events(I-9.29 与 I-9.6 都不红)=0 residual_steps=1693 residual_hole_steps=37 steps_judged=3904
cand   q1_red  gone_red  hole_red  residual_red  residual_hole_red  red_steps
c1          32         0         0           147                  0        334
c2a        336         5         0          1693                 37       2619
c2b         32         0         0           147                  0        334
c2c        164         2         0           722                  0       1476
c2d        336         5         0           987                  0       1913
c3         336         5         0          1693                 37       2645
c4         164         2         0           722                  0       1476
c1b        260         5         0          1342                 22       2049
c2i        336         5         0          1473                 26       2399
c4g        336         5         0          1693                 37       2619
c2ig       336         5         0          1693                 37       2619
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/agg.py（research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-P.summary 的前 13 行）
histories=64 q1_events=0 gone_events=0 hole_events(I-9.29 与 I-9.6 都不红)=0 residual_steps=0 residual_hole_steps=0 steps_judged=9664
cand   q1_red  gone_red  hole_red  residual_red  residual_hole_red  red_steps
c1           0         0         0             0                  0          0
c2a          0         0         0             0                  0          0
c2b          0         0         0             0                  0          0
c2c          0         0         0             0                  0          0
c2d          0         0         0             0                  0          0
c3           0         0         0             0                  0          0
c4           0         0         0             0                  0          0
c1b          0         0         0             0                  0          0
c2i          0         0         0             0                  0          0
c4g          0         0         0             0                  0          0
c2ig         0         0         0             0                  0          0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/agg.py（research/prompts/m3-w3-r2-sonnet-model/expected/device-F2-P.summary 的前 13 行）
histories=256 q1_events=0 gone_events=0 hole_events(I-9.29 与 I-9.6 都不红)=0 residual_steps=0 residual_hole_steps=0 steps_judged=15616
cand   q1_red  gone_red  hole_red  residual_red  residual_hole_red  red_steps
c1           0         0         0             0                  0          0
c2a          0         0         0             0                  0          0
c2b          0         0         0             0                  0          0
c2c          0         0         0             0                  0          0
c2d          0         0         0             0                  0          0
c3           0         0         0             0                  0          0
c4           0         0         0             0                  0          0
c1b          0         0         0             0                  0          0
c2i          0         0         0             0                  0          0
c4g          0         0         0             0                  0          0
c2ig         0         0         0             0                  0          0
```

第 24 段的两个洞事件（`research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-V1-idx24.summary` 里原样）：

- 「position=op92 inode=7 new_birth=77 older_births=[51] q9=Gone i9_29_at_step=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=1 w3_c2d=1 w3_c3=1 w3_c4=1 w3_c1b=1 w3_c2i=1 w3_c4g=1 w3_c2ig=1」（`research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-V1-idx24.summary:25`）
- 「position=op95 inode=8 new_birth=78 older_births=[52] q9=Gone i9_29_at_step=holds w3_c1=0 w3_c2a=1 w3_c2b=0 w3_c2c=0 w3_c2d=1 w3_c3=1 w3_c4=0 w3_c1b=1 w3_c2i=1 w3_c4g=1 w3_c2ig=1」（`research/prompts/m3-w3-r2-sonnet-model/expected/device-G-long-V1-idx24.summary:26`）

C4g 在 op95 上红的原因（同一文件逐步打印里 op71 的回退追加了 (T_rb, W) = (53, 9)）：重发的 8 号出生代 78 > 53 且 8 < 9。W 是回退前 cur 的水位 9，不是回退后的水位，所以「水位又回到峰值」（op95 之后水位 9 = 峰值 9）对它没有影响。

## 五、第一轮每处打中，逐处答四句

| 打中 | 分不分辨臂（别的候选在同一格上） | 判别子看不看得到 | 字面落在哪一句 |
|---|---|---|---|
| C1a 漏 op92、op95 | 分辨：同两格 C2a、C3、C4g、C1b、C2i 都 2/2 | 看不到：行里存的是恢复所选那一版的水位，回退已把它压低；回退前的高水位只在已转出环的根里 | 「漏报（同一条时间线上重发而判不红）」，候选单独判与并上现有检查判都成立 |
| C2b 漏 op92、op95 | 分辨：同上 | 看不到：每实例最后一代被同实例后面的一代覆盖 | 同上 |
| C2c、C4 漏 op95 | 分辨：同一步 C2a、C3、C4g、C2d、C2i 都红 | C2c 看不到：重发 8 号之后水位 9 = 峰值 9，只记数的读法与合法镜像逐字段相同；C4 同（只判水位 < 回退前最大水位）。C4g 看得到：记录里带的是 W 和 T_rb，判号 < W 且出生代 > T_rb | 同上 |
| C2d、C4a「越过旧峰值之后不红」 | 分辨：同一张持久镜像上 C2a、C3、C4g 都红，C2d 只在事件那一步红 | 看不到：(峰值, 代) 被新值盖掉，旧峰值的信息没了 | 第一轮拿来出局它的两处例子 I-9.29 是红的（不是洞）；成立的字面是「事件之后的镜像里重发的对象还活着而判不红」，这一句只在持久镜像上成立（模型 120,864 张，装置 G-long 4/221 判红） |
| C1m、C3f、C3fb 误报 | 本轮没有复核（不在 K1 点名的项里） | — | — |

### 6. 深一层（长 8），字母表收窄到 8 个

字母表 create、delete、rollback1、rollback2、raiseF、mount、crash-hide、switch-redo（`W3_OPS`），长 8，环 4，清扫阈值「从不」；P 臂（5,184,012 张合法镜像）和 V1 臂。删除多了，墓碑和崩溃恢复抛弃都在里面。C4g 在 P 臂 0 误报，V1 臂洞事件漏 0、持久镜像漏 0；C1b、C2d、C2i 照旧漏。

```
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29t,I-9.6t,C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-deep-0.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-deep-0.out arm=Product mutation=None ring=4 sweep_threshold=never longest=8 images=5184012 legal_images=5184012 events=0 events_where_I-9.29_holds=0 events_where_existing_checks_silent=0 events_existing_silent_mounted_since_the_last_rollback=0 events_existing_silent_not_mounted_since_the_last_rollback=0 residual_images=0 residual_where_existing_checks_silent=0
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29t    0         0               0                  0               0      0            0          0         0
  I-9.6t     0         0               0                  0               0      0            0          0         0
  C1b        0         0               0                  0               0      0            0          0         0
  C2d        0         0               0                  0               0      0            0          0         0
  C2i        0         0               0                  0               0      0            0          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
$ python3 research/prompts/m3-w3-r2-sonnet-model/tools/table2.py I-9.29t,I-9.6t,C1b,C2d,C2i,C4g,C2a,C3r research/prompts/m3-w3-r2-sonnet-model/expected/model-deep-2.out
research/prompts/m3-w3-r2-sonnet-model/expected/model-deep-2.out arm=RollbackTakesTheTargetsWatermark mutation=None ring=4 sweep_threshold=never longest=8 images=5184012 legal_images=3184872 events=281672 events_where_I-9.29_holds=123268 events_where_existing_checks_silent=121336 events_existing_silent_mounted_since_the_last_rollback=94310 events_existing_silent_not_mounted_since_the_last_rollback=27026 residual_images=537080 residual_where_existing_checks_silent=440672
  candidate  FP  evt_miss  evt_miss_exist  exist_after_mount  exist_no_mount  resid  resid_exist  mut_undet  mut_imgs
  I-9.29t    0    121344          121336              94310           27026 482912       440672          0         0
  I-9.6t     0    263912          121336              94310           27026 490792       440672          0         0
  C1b        0     69622           19916               5824           14092 171810       141072          0         0
  C2d        0         0               0                  0               0 258928       228920          0         0
  C2i        0         0               0                  0               0  34596        29002          0         0
  C4g        0         0               0                  0               0      0            0          0         0
  C2a        0         0               0                  0               0      0            0          0         0
  C3r        0         0               0                  0               0      0            0          0         0
```

## 六、什么现象会推翻上面的结论

- 「C1（C1a、C1b）出局站得住」：C1a 在第 24 段 op92 或 op95 上红（`device-G-long-V1-idx24.summary` 里 `w3_c1` 不是 0）；或 C1b 在模型里洞事件漏 0 且环长 24 上那条合法序列（`model-replay-ring24-event.out`）C1b 红；或造出入库装置上的 C1b 写者，挂载之后环转一圈的历史里 M 仍是回退前的水位。
- 「C2c、C2b 出局站得住」：C2c 在 op95 上红（`device-G-long-V1-idx24.summary:26` 里 `w3_c2c` 是 1），或 C2b 在 op92、op95 上红。
- 「C2d、C2i 站得住，理由是持久镜像」：装置 G-long V1 持久洞步 `residual_hole_red_c2d` 等于 221（现在 4），或模型持久镜像漏 `resid_exist` 在环 4、6、8 上降到 0。
- 「C4（攻方腿定义）站得住」：op95 上 `w3_c4` 是 1。
- 「C4g 不该出局」：①找到 P 臂上 C4g 红的一张合法镜像（模型 33,245,604 + 5,184,012 张、装置 25,280 步里现在是 0）；②找到 V1 臂上现有检查都不红而 C4g 不红的事件或持久镜像（模型 138,372 + 121,336 个洞事件、352,888 + 440,672 张持久镜像、装置 258 个持久洞步与 2 个洞事件里现在是 0）；③入库的谱系记录从 16 字节加到 24 字节被某条已定项或冻结层挡住（`.claude/kb/decisions/18-块里携带什么信息.md:576` 把记录写成 16 字节、一容器 2039 条；我没查冻结层，推的）。
  C4g 的限度站不住的现象：造出一个不经回退的水位退回（V3 之外）而 C4g 单独就红，那它的覆盖比我量的宽。
- 「K3 两形不分候选」：找到一种单个存储的候选（C2a、C2d、C2i、C3r、C4g 之外）在 V1 + A、B、C 上洞事件漏数不是 9,534；或 C2a 与 C3r 的数和 C4g 的不一样。
- 「V2 不在 W3 要补的块里」：找到一张 V2 臂镜像，I-9.29 与 I-9.6 都不红而被重发的对象在最新根里（模型 3,050,056 个事件里现在是 0）；或墓碑被回收（第一版不回收，`.claude/kb/decisions/18-块里携带什么信息.md:575`）之后 V2 删完再重发的形态现有检查看不见。

## 七、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果。
- 没写 `crates/` 里的写者与读者：C1b、C2i、C4g 在装置里是影子记录（把候选的写规则套到真代码走出的根链上，`parent_missing = 0`、`row_root_unknown = 0`），不是盘上字节；模型里的墓碑、谱系树是我按 D18 已定项 24 与 D23 已定项 14 的条文写的状态机，删除把「意图」与「回收」合成一步。数全是副本装置与自写模型上的数，进 kb 之前要在入库装置上重做。
- 没量代价（K4 不归这条腿）：C4g 的 16 → 24 字节、C1b 的每次挂载多读、C2i 的每实例行数都是推的。
- 环长 24 的产品上没有枚举，只有环长 4、6、8、12 的穷举和环长 24 的两条具体序列；历史长度 7 或 8 或（收窄字母表）到 11。没有造出一条 24 次发布的装置定向段（正推腿试过四种没做成，我没再试）。
- 装置 P 臂没有造出跨被抛弃时间线的重发（E175 的 `sum_q3b` 全 0），C4g 对那一形的误报只在模型上量（154,778 次合法重发，0 误报）。
- 没复核 C1m、C3f、C3fb 的误报（不在 K1 点名的项里）；没重跑攻方腿的定制族 5,220 段与 K3 的长段。
- 没跑任何重型测试，没跑门禁与 lint；装置快档两臂各 `28 passed; 0 failed; 3 ignored`。
- 「没打中」只抽了一次：模型是确定性的，不是抽样，但 C4g 的 0 漏是我自己写的模型与影子上的零轮结果；按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」那一节，攻方腿对 C4g 再攻之前不算「没被打中」。

## 八、草稿产物与删掉的东西

- 入库的只有模型目录 `research/prompts/m3-w3-r2-sonnet-model/`（58 个文件，1.5 MB）和这份报告。草稿目录 `~/.local/share/singlefs/work/m3-w3-r2-sonnet/` 现 26 MB：`out/`（开发时各组模型输出，含带 W3TRACE 的全量）、`dev/out*`（装置各段原始输出）、`rerun1/out`（空目录从头跑通那一次的输出，7.7 MB）、`expect-src/`（拿来整理 expected 的那批开发时输出）、`model-src/`、`model-new/`（不含编译目录）、`tools/`、`seg/`。没入库的原因：原始输出里的逐步打印行太大，入库的 `expected/` 已去掉 W3TRACE 行、装置只留摘要；主 agent 要看全量就到这个目录读，要重出就跑 `rerun.sh`。
- 删了的编译目录、工作树与仓副本（`du -sh` 先记的大小）：`~/.local/share/singlefs/work/m3-w3-r2-sonnet/dev/repo` 22M、`dev/target` 506M、`dev/bin` 252M（开发时的冻结树副本、编译目录、两个装置二进制）；`rerun1/repo` 22M、`rerun1/target` 506M、`rerun1/bin` 252M、`rerun1/target-model` 688K（空目录从头跑通那一次的同样四样）；`target-model` 680K、`target-model2` 688K（开发时模型的编译目录）。原因：交回闸要求删，要复核跑 `rerun.sh`。
- 装置二进制是在冻结树副本上编的；每次编完核过 `tree-sha256.txt`（`rerun.sh build-V1` 末尾打「冻结树副本已还原」）。
- `rerun.sh` 最后一次改动是加 `model-deep` 一段（长 8 的 P、V1 两组），那一段和之后的 `compare` 在 `rerun1` 里单独跑过（`compared=37 status=0`）；其余段是改之前从头跑通的那一次。

## 九、交回前核对

模型目录每个文件的 sha256（`find . -type f | sort | grep -v '^./expected/' | xargs sha256sum`；`expected/` 37 个文件连起来的 sha256 是 3236ff2c19bfea40b8c20efea28f7392aebbed2c726399855931e41e765ba6c1）：

```
9c4921844aba33da9383bab2b7f6e2fa839001eb28d43cc79b5863afe6cf0e39  ./compare.sh
7a963b5b838d9db058b1d1c9f9bf5f317c7d3882d9ef74e1aa58cd6b4da95903  ./make_device.py
04791741aa819209917045a8ffb456bae2fbe9f7010bb954433d0707b7095f53  ./README.md
b5eddfa10da5a4846c0f5cd636082a95a60ec80aeb998c9955658551fd49d502  ./rerun.sh
6e443248d2142b608a9da47006d865053985fc4c9d7e8d2b1b0755105897707a  ./timeline-model/Cargo.lock
45f5f9e0d78123de821b39e79861f580e762cb6c6b9f6f0b9cc7d220b48b9b86  ./timeline-model/Cargo.toml
7295182540b025a59c82ea1721520eec40e9691d3f9b776dc560532610eeec25  ./timeline-model/src/main.rs
45e86f2b309bdc3ab16310a751531e76d4e560fd9b7bd6877c1c4bc4713d8936  ./tools/agg.py
b4d7680995859223df94bd6e5e9fa83905f9a5a022a1842077115f3ca07648f4  ./tools/derivation/mk_make_device.py
28b2ea0664c1a9b57460e0215cbcb4580cd797076a2f81bc7e79693a5745d95c  ./tools/derivation/patch_model2.py
43a63c1b59d9117ddb69fe836e00fbd7dea13734e6c6c9ddea7534e3f14044e2  ./tools/derivation/patch_model3.py
c30848b0be658d7efe581e40a9b6ee6c3cb8a6ce2ea376a33e5cb209efede1f7  ./tools/derivation/patch_model4.py
c5d052cac960eebefdf853cb5b2fc36f62e7c5158a604c6d93588f3ce9025e29  ./tools/derivation/patch_model.py
21f8f22804d721ab296ba3e55b0d75f1d2b204fc5a1cd3c4bb761696dd8c2887  ./tools/derivation/patch_module.py
66912ff8d72a8a93d97442e67aeda21a9db99518fca9764ba06892fed944f1c7  ./tools/dev_run.sh
afb842adcdb29df20048cb47a8bd5d57f163da72d6046bf7ca8d5cce6bf46e5b  ./tools/mut_table.py
684791e19c20ed63e99e1fe346f0a87295c4768d536959acdf1623b80d1fe93b  ./tools/par.sh
9ba691709dfce8c572148d2b75f180619da4b30f2b29ab2384f0efb0ca2dc4ef  ./tools/ringwin.sh
d88506b6c2a59aa1c6a0c541c8f9233f707b08e1e5daf9ba3741d1c4188d7a16  ./tools/summarize.py
972922008fe2d8adbb9367acfa520cb4a6f7dae9955b35b87a6693c2f2fb568f  ./tools/table2.py
bd8033092394f059c5994c5fa5c59d67dd1f3dd50c4f149e3061d10d12eaacef  ./w3r2_module.rs
```

交回前跑 `python3 research/scripts/cite-check.py research/prompts/m3-w3-r2-sonnet-output.md` 的末行：✓ 核了 16 处引文，对上 16 处，没判 0 处（逐处列在上面）
