# m3-w3-r3 攻方腿（Opus）报告：K2、K3 与前两轮数字的复核（2026-10-08）

轮名 m3-w3-r3（最终轮）；背景材料 `research/prompts/_m3-w3-r3-background.md`（正文 `research/prompts/_m3-w3-r3-body.md`，附录 `research/prompts/_m3-w3-r3-appendix.md`）；前两轮判决 `research/prompts/m3-w3-r1-main-verification.md`、`research/prompts/m3-w3-r2-main-verification.md`。
冻结树 `~/.local/share/singlefs/work/m3-w3-r1/tree/`（只读）；草稿 `~/.local/share/singlefs/work/m3-w3-r3-opus/`；模型 `research/prompts/m3-w3-r3-opus-model/`。
这一份里的数全是我自己写的抽象模型上量的（不依赖 `crates/`，也不是入库装置）；我提的定义与改法只在我的模型上量过、被攻过零轮。

（本文件分段追加；「各格判定一览」是最后一段写完后换进来的。）

## 各格判定一览

| 格 | 形态 | 判定 |
|---|---|---|
| K2 维持 D23 已定项 14：「容许跳升只来自被抛弃的根」顺带放过真重发 | C2aL、C3rL、C4g、C2i（C2ig） | **没打中**：四个形态里没有要容许的判词（C2aL 回退跳升那次本来留行；C3rL、C4g、C2i 没有「盖住水位」那一句）；十个字母长 7 七条臂（三条会重发）漏 0 |
| 同上 | 「容许」写成 I-9.30 ③ 的例外（字面 C2a / C3r-cover 才要） | **打中（双故障）**：照字面核来源（tolB）在产品取法上仍误报约一半（81686 / 163700）；不核来源（tolC）不误报，但两者都把「回退之后台阶行不写」当跳升放过，叠一次真重发就漏（V1：tolB 洞事件 48、tolC 持久镜像洞 120；去掉 cur 项的取法：tolB 洞事件 25）；strict 0 |
| K2 改成只取没被抛弃的根：C143「回退那次取 max、不重发」退回去 | 第二种取法 | **没打中**：在每个量到的产品状态上它等于「只取 cur 内存里的」（`rollback_kept_ring_above_cur=0`）；C143 那一格靠的是 cur 内存项，去掉 cur 项才重发（3362 / 3509 次，滤不滤被抛弃都一样）；仓副本 B 历史全域 196629 张 0 误报、跳升 4 → 0 |
| 同上：实例表读不出、根读不出 | 第二种取法 | **没打中**：cur 的实例表读不出今天的代码先拒整次回退；根读不出不改结果；若另给「表读不出当全部没被抛弃」的退路，跳升与字面 C2a 的误报回来（21114）。代价：合法跨抛弃重发 +11.6%（48356 → 53960） |
| K3 克隆头 | C4g、C2i、C3rL 不分头 / 不带树 ID 的读法 | **打中（误报）**：C4g 只带一个 W 62532 / 35130、C2i 每实例一条 158198、C3rL 不带树 ID 52687（`clone-0`）；按头 / 带树 ID 0。今天的 I-9.29 字面不分树，同样误报 46518（不归 W3） |
| 同上 | C2aL | **打中一形读法（误报）**：新头水位取 origin 现行而克隆那次不留台阶行，29974；留行 0 |
| 同上 | 按头的 C4g、C2i（C2ig） | **打中（漏，V1 类臂）**：克隆头继承来的那一段没有谱系值，洞事件漏 219 / 236 / 219（`clone-5`）；C2aL、C3rL 不漏；补法 C4g-inh（按快照那一版的谱系判，零轮）三组 0 |
| K3 副本坏一份（w = 2） | 三个形态 | **没打中**：读历史是「任一份校验和对上」，坏一份读另一份；坏一份改变的只是时间线（等于崩溃抛弃，模型两个字母一直在扫） |
| K3 新实例第一个 txg 被改坏、撞上被抛弃发布的出生代 | 三个形态 | **没打中**：撞号造不出来（同 txg 的新根落在被抛弃的根同一槽、把它盖掉），`identity_collisions=0`；改坏只在 journal 留下「换实例 txg 不增」，I-8.12 今天不判，全仓无红（不归 W3） |
| 前两轮的数能不能核 | 第二轮攻方模型、辩方模型、攻方原型 `probe` | **全部对上**：在不共用源码的模型上 P 臂（9863012 / 10836269 / 48356 / 55040 / 163700 / 231204）、阈值 0、V1（洞事件 138372）、V3（C4g 洞漏 12056）逐个相同；原型 `probe` 59 行逐行相同 |

## 复跑命令与模型文件

在仓根，`W` 取一个空目录，每段一条前台命令，编译与跑都经 `research/scripts/run-with-memory-cap.sh`（编译 6G、跑 4G），线程 5：

```
M=research/prompts/m3-w3-r3-opus-model; W=<空目录>
for p in build-model model-verify model-y1 model-clone-first build-proto probe-base build-proto-kept probe-kept compare-quick; do SINGLEFS_REPO=$PWD bash $M/rerun.sh $W $p; done
# 长段（可选，约 17 分钟）：kept-shard-0 … kept-shard-19，再 kept-merge，再 compare（多比 expected/kept-merge-of-20.out）
```

`build-proto` 拷冻结树进 `$W/repo`、对 `research/prompts/m3-w3-r3-snapshot/tree-sha256.txt` 核，再把第二轮攻方的原型 crate 原样放进去（取自 `research/prompts/m3-w3-r2-opus-model/proto/`，环境变量 `R2_MODEL_DIR` 可改）；`build-proto-kept` 在副本上打 `proto-patch/patch-Y1-kept.diff`。`crates/` 与冻结树都没动。
`compare` 对模型只比 `W3R3MODEL`、`W3R3COUNT`、`W3R3EXAMPLE` 三种行，对原型只比 `W3R2` 开头的行（去掉 `elapsed_s` 与 DONE 行），排序后比。设 `W3R3_TRACE=1` 时模型多打每个最短反例的逐步状态（`W3R3TRACE` 行，不进比较）。

模型目录每个文件的 sha256（`find . -type f | LC_ALL=C sort | xargs sha256sum`）：

```
3c24ef169f015628bf4c8e4b81784e4e60212785a70ba2788d74ce4aa563c3c6  ./expected/clone-0.out
51608f489bfb9d022f905bd067ee5696a308f3a14eb8ffb059e8d6a003a8ecb9  ./expected/clone-1.out
3edcdf8bc1aa998c8e2c79485ea1f396f596e605dfab83ae7cf9e9478eecde94  ./expected/clone-2.out
c050dd5a83135eb390dc07a0fa4264899833349c2cbc98495450788d879eaf94  ./expected/clone-3.out
885ab9b0722a3c5c5fec84b3e461f0cb17685a1e6b2acf710836f600c51f962f  ./expected/clone-4.out
2d2f201f104ba3387a0ed6b550f513b345f5502805b2fa881a508d9c074f8658  ./expected/clone-5.out
460b392022679275046e9f119f24dbbc3c7d35434701517e4c5963c5a5d79ec0  ./expected/clone-6.out
0b8ca48b5e112c83c266d6fcca9390c66b477a13b8358a96f17d3c854c1fda91  ./expected/clone-7.out
a9c94144d487dcc975761f3670c2d2770bbb2f7fadfde13302eaa3b356128b7d  ./expected/first-0.out
f314f032df55356a8a6aa87ed4b1a442604bcf8304f5af41560848ba55b4e9a4  ./expected/first-1.out
22f2ed5a2914cda9763776a4bcf68cb5c7f12f1f171e31252ef607285be4d5a4  ./expected/kept-merge-of-20.out
b9bae4f073a5b0de628557afb2931ef8bef7281ffa124392aa74d93df25b20d5  ./expected/probe-base.out
f9e36206147938fbdea7160d54e1b82579846201c7c909246b3ce2aa5e9bc2f0  ./expected/probe-kept.out
10f5581cc8ef0ae51bfb18fe9efbba0993139bd239351e683970213c185c14ab  ./expected/verify-0.out
b8a0e9ebdcd582766fb4655339d4cf961a5f81623e1aad9d202a4471cfcc8bee  ./expected/verify-1.out
ae613ffa278119f94f5fb0643e52dccca77255c0f9a2961670eb9f0910fba1ad  ./expected/verify-2.out
40b8e8230f170fa25d53114a21ab1a458a955847b5cd18fe510f68d8f9ca5941  ./expected/verify-3.out
1b15f461a80a1b00955995b8f5341b8e61397f5e08571ebfb04bb6a2994b35e5  ./expected/verify-4.out
1d4695a41c022e54dfe5c97879c54dae8be47a7af7aa905c9a8032fac3a5e12a  ./expected/y1-0.out
01c5076511a5ce08fac52d24a7096a21757f325d0751f8335b069052a11fc0ba  ./expected/y1-1.out
fee5a41a403bbfa92571d67f2bf167981758018d894913f844f19e5e92219060  ./expected/y1m-0.out
6b0dfc326ee8694af430d935e629405b392d1006cdb939192249414eec8390b9  ./expected/y1m-1.out
5fbd62db1aca717a0c1ca695d1703701de8a37b3444911944387460fe4e10803  ./expected/y1m-2.out
8f031a0fbf7edab62506074b2fa5967f6d4f337a4f8855126420a54c3bfba63e  ./expected/y1x-0.out
0416b8e9ed9c542a752ddf0b90101a3be2d0597f228d71414254d79cf5931a2d  ./expected/y1x-1.out
05acc33027fdca720a1713e3462ea5eaf0fff6a6dd4869d61e8053ec48be4787  ./expected/y1x-2.out
4fb861e13a6b1e1fc6e89fc4d79a918ec9490c8e325fa0e4aaa39775a69c8175  ./expected/y1x-3.out
fd226f138026cb20874c27392e3a5134e071671a35e202021502175e4b396fc6  ./expected/y1x-4.out
45b86b600c65cfa2a8e946a7597597a3ba79daea94753d5dd17ea70f39d98a93  ./expected/y1x-5.out
b23fb6ca7eea67e7aaf478fdd3a5684e9bcbcc30ea72172d509915088099aaa4  ./expected/y1x-6.out
2584e4811f4fe83089cbc5c4d1543fd5554ec12230dbf3c52839d9fd6080b4d0  ./model/Cargo.toml
f053e4d1b32e6a5451be6f8058a5743b3c501a6b4203eac36b23aeb6d3ada76f  ./model/src/main.rs
0e0b625a2e368c56e83a0a16554f8526e83826954e60f56c2e1e9062ce214a8d  ./proto-patch/patch-Y1-kept.diff
4db8c8116c10a688b12d5093b9e321e657fe12394b3ea7a468869a1a10076e45  ./rerun.sh
```

## 一、两套装置

- **抽象模型**（`model/`，从头重写，不拷前两轮的源码，不依赖 `crates/`）：条文走法按 D23 已定项 14 与前两轮攻方模型文档里写的字面重写——发号、删、回退 1–3、抬 F、空发布、挂载、崩溃恢复抛弃最新那条根、建到一半崩溃、切换重做 / 报错、清扫；墓碑随现行那一版、不回收；谱系记录 (T_rb, T_old) 之外按头带 H_cur、|log(cur)|、H_new。起点固定在新池新建文件之后，用户步全部穷举（环 4；十二个字母长 7，或十个字母长 7、九个字母长 7）。第七节先用它独立重算前两轮的数，全部逐字对上之后才往下用。新加的维：
  - 回退取水位五种规则：`MaxCurAllRing`（今天的条款与代码，Y1 维持）、`MaxCurKeptRing`（Y1 改成只取没被抛弃的根）、`MaxAllRingOnly` / `MaxKeptRingOnly`（去掉 cur 内存项，C143 第三轮之前那种取法的两种滤法）、`Target`（V1）；
  - 三个新字母：回退那一刻 cur 的根槽读不出、cur 的实例表读不出（三种处置：拒 / 当全部没被抛弃 / 环里一条不取）、崩溃之后最新那条根与它的记录都读不出；
  - 两个可写头：`clone` 建第二个头（从 cur 或从较旧的一版克隆；新头水位取快照的或 origin 现行的；克隆那次写不写台阶行），`create1` 在第二个头上建；回退目标的树集合与 cur 不同时拒（今天的代码同样拒，见第四节）；
  - journal：每次发布一条记录，新实例第一条记录接在读得出的最大号之后（D23 已定项 14 注 3）；新实例第一个 txg 两种：照代码取 max(读得出的根, 读得出的记录) + 1，与改坏成「所选根 + 1」。
- **仓副本原型**：第二轮攻方腿的原型原样（`research/prompts/m3-w3-r2-opus-model/proto/`），放进冻结树副本；流是它自己造的，层 0 枚举域照它（每一段都展开，P 那次发布 65548 个状态）。我只做两件事：照原命令重跑它的 `probe`（核第二轮的数），与在副本的 `mount.rs` 上打 Y1 第二种取法的补丁之后，重跑 `probe` 与 B 历史的全域二十片。
- 候选（判最新那条可读的根）：
  - 三个零轮形态照第二轮判决的定义：C2aL（每次水位变了的发布留台阶行 + I-9.30 ①②③ + 谱系带 cur 水位判等于），C3rL（C3rs + 谱系带 |log(cur)| 判等于），C4g（谱系带回退前 cur 的水位 W，判「水位 < W，或 (号, 出生代) 号 < W 且出生代 > T_rb」），C2i（每 (实例, 头) 一条 (最高水位, 第一次到它的 txg)），C2ig = C2i 或 C4g；
  - 两个头时各候选的「不分头」读法：C4g-maxW（谱系只带一个 W，取各头最大）、C4g-firstW（只带头 0 的）、C2i-inst（每实例一条，不分头）、C3rL-pooled（发号记录不带树 ID）、I-9.29-pooled（今天 I-9.29 的字面：同一个 inode 号，不分树）；与我加的 C4g-inh、C2i-inh（克隆头另按快照那一版的谱系值与 C2i 条目判）；
  - 要容许跳升的几种弱读法：C2a-create（只在建 inode 的发布留台阶）配 ③ 的三种读法——strict（照判）、tolB（「跳升只来自被抛弃的根」照字面核：环里有一条按最新根实例表被抛弃、水位 ≥ 现水位的根就不红）、tolC（有一条谱系记录的 H_new ≥ 现水位就不红）；C3r-cover 同三种。
  - 现有检查照第二轮辩方的口径：I-9.29t（候选集各版的树加谱系上的墓碑）、I-9.6t（树内最大号一半加墓碑一半）；「洞」= 两条都不红。
- 真相：每个头这条时间线上发过的 (号, 出生代)（回退那次并上 R_old 的树）；同号两个出生代就是同一条时间线上的重发。合法镜像（没有重发、没有发过的号 ≥ 水位）上红叫误报；重发那一步或「被重发的对象还活着」的镜像上不红叫漏。V 臂（`Target`、两种去掉 cur 项的、V3）上的「误报」列没有意义（水位被改低，合法的定义在那几臂里不成立），只读漏的列。

## 二、K2：维持 D23 已定项 14（容许跳升只来自被抛弃的根）

被判的那一格：「| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）；代码同一个取法：「let ring_roots = readable_roots(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7738`）、「ring.max(current.inode_number_watermark())」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`）。

### 1. 四个最强形里没有「容许」那一句可钻

- C2aL 在回退跳升那次本来就留一行台阶（「每次水位变了的发布」），③ 照判、不需要容许；C3rL 用「树里每条都在记录里」，没有「记录盖住水位」那一句；C4g、C2i 只判「水位 < W / M」与「号 < W 且出生代 > 那个代」，跳升只会让水位更高，两句都只在水位往下走时红。所以「容许跳升」对这四个形态不是一句判词，没有东西可以顺带放过。
- 量的（环 4、长 7，十个字母，含回退那一刻 cur 的根读不出、实例表读不出、根与记录都读不出三个新字母；用户步全部放开）：四个最强形在七条臂上漏 0（`expected/y1x-0.out` … `-6.out`，其中 `-4`、`-5`、`-6` 是三条会重发的臂：去掉 cur 内存项的两种取法与 V1，洞事件 1143 / 1218 / 7109 个）：

```
$ cd research/prompts/m3-w3-r3-opus-model/expected; for f in y1x-4 y1x-5 y1x-6; do for c in C2aL C3rL C4g C2ig; do grep -E "^W3R3COUNT .* candidate=$c " $f.out | sed -E 's/.*candidate=//; s/ red_on_collision=0//' | sed "s/^/$f /"; done; done
y1x-4 C2aL false_positive=46168 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-4 C3rL false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-4 C4g false_positive=216 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-4 C2ig false_positive=216 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-5 C2aL false_positive=45349 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-5 C3rL false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-5 C4g false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-5 C2ig false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-6 C2aL false_positive=65763 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-6 C3rL false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-6 C4g false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1x-6 C2ig false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
```

（这三条是 V 类臂，「误报」列不读，见第一节。）判：**没打中**（四个最强形，一次抽样）。

### 2. 「容许」写成一句判词时（只有字面 C2a 与 C3r-cover 要它）

把正文「历史核对容许跳升只来自被抛弃的根」写成 I-9.30 ③ 的例外，我试了两种读法：

| 读法 | 产品取法（今天）上的误报 | 叠一个「历史写错」变异之后，真重发漏不漏 |
|---|---|---|
| strict（照判 ③，不容许） | 163700 / 10836269（`expected/verify-0.out`）；29121 / 976584（`y1x-0`） | 0（`y1m-0`、`y1m-2`） |
| tolB（照字面核：环里还有一条被抛弃、水位 ≥ 现水位的根才容许） | 81686（`verify-0`）、14275（`y1x-0`）：约一半，跳升的来源根转出环或被盖掉之后就核不了 | V1：事件漏 64、洞事件漏 48（`y1m-0`）；去掉 cur 项的取法：洞事件漏 25（`y1m-2`） |
| tolC（有一条谱系记录的 H_new ≥ 现水位就容许，不核来源） | 0 | V1：事件漏 48（都不是洞）、持久镜像漏 168、其中洞 120（`y1m-0`）；去掉 cur 项的取法：持久镜像漏 70、其中洞 55（`y1m-2`） |

「历史写错」变异：有过一次回退之后字面 C2a 的台阶行不再写（`rows_create_stop_after_a_rollback`，K3「把那份历史写错或不写，它必须红」那一类）。strict ③ 在这一变异上当场红（历史盖不住水位），容许的两种读法把它当成「跳升」放过，之后叠上一次真重发就漏。

```
$ cd research/prompts/m3-w3-r3-opus-model/expected; for f in y1m-0 y1m-2; do grep -E '^W3R3COUNT .* candidate=C2a-create-(strict|tolB|tolC) ' $f.out | sed -E 's/.*candidate=//; s/ red_on_collision=0//' | sed "s/^/$f /"; done
y1m-0 C2a-create-strict false_positive=137601 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1m-0 C2a-create-tolB false_positive=127769 event_missed=64 residual_missed=60 hole_event_missed=48 hole_residual_missed=60
y1m-0 C2a-create-tolC false_positive=115139 event_missed=48 residual_missed=168 hole_event_missed=0 hole_residual_missed=120
y1m-2 C2a-create-strict false_positive=237609 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
y1m-2 C2a-create-tolB false_positive=205919 event_missed=25 residual_missed=35 hole_event_missed=25 hole_residual_missed=35
y1m-2 C2a-create-tolC false_positive=143939 event_missed=0 residual_missed=70 hole_event_missed=0 hole_residual_missed=55
```

最短反例（tolB 漏一个洞事件；V1 臂加「台阶行回退之后不写」变异；复跑：模型 `W3R3_TRACE=1 w3r3-model y1m 0`，取 `W3R3TRACE candidate=C2a-create-tolB kind=hole_event_missed` 那几行，下面去掉了 rows_all、log、issued、c2i、journal 几列）：

```
$ grep 'W3R3TRACE candidate=C2a-create-tolB kind=hole_event_missed' y1m-0-trace.out | sed -E 's/^W3R3TRACE candidate=[^ ]* kind=[^ ]* //; s/ (rows_all|log|issued)=\[[^]]*\]//g; s/ c2i=\[[^]]*\]//; s/ journal=\[[^]]*\]//'
step=start newest=1:3 F=0 table=[] h0{wm=2 tree=[1@3] rows_create=[3:2]} lineage=[] ring=[1:3:wm[2]]
step=0 op=empty newest=1:4 F=0 table=[] h0{wm=2 tree=[1@3] rows_create=[3:2]} lineage=[] ring=[1:4:wm[2],1:3:wm[2]] red=[]
step=1 op=rollback1 newest=1:5 F=0 table=[] h0{wm=2 tree=[1@3] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2]] ring=[1:4:wm[2],1:5:wm[2],1:3:wm[2]] red=[]
step=2 op=create newest=1:6 F=0 table=[] h0{wm=3 tree=[1@3,2@6] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2]] ring=[1:4:wm[2],1:5:wm[2],1:6:wm[3],1:3:wm[2]] red=[C2a-create-strict,C2a-create-tolB,C2a-create-tolC]
step=3 op=rollback1 newest=1:7 F=0 table=[] h0{wm=2 tree=[1@3] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2],7<-5|cur6:H[3]:n[2]:Hnew[2]] ring=[1:4:wm[2],1:5:wm[2],1:6:wm[3],1:7:wm[2]] red=[I-9.6,C2aL,C3rL,C3rL-pooled,C4g,C4g-maxW,C4g-firstW,C2i,C2i-inst,C2ig,C3r-cover-strict,C3r-cover-tolB,C3r-cover-tolC,C3r-nocover,I-9.6t,C4g-inh,C2i-inh,C2ig-inh]
step=4 op=create newest=1:8 F=0 table=[] h0{wm=3 tree=[1@3,2@8] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2],7<-5|cur6:H[3]:n[2]:Hnew[2]] ring=[1:8:wm[3],1:5:wm[2],1:6:wm[3],1:7:wm[2]] red=[I-9.29,I-9.29-pooled,C2aL,C3rL,C3rL-pooled,C4g,C4g-maxW,C4g-firstW,C2i,C2i-inst,C2ig,C2a-create-strict,C2a-create-tolB,C2a-create-tolC,C3r-cover-strict,C3r-cover-tolB,C3r-cover-tolC,C3r-nocover,I-9.29t,C4g-inh,C2i-inh,C2ig-inh]
step=5 op=crash-hide newest=2:9 F=0 table=[1:7] h0{wm=2 tree=[1@3] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2],7<-5|cur6:H[3]:n[2]:Hnew[2]] ring=[1:8:wm[3],2:9:wm[2],1:6:wm[3],1:7:wm[2]] red=[I-9.6,C2aL,C3rL,C3rL-pooled,C4g,C4g-maxW,C4g-firstW,C2i,C2i-inst,C2ig,C3r-cover-strict,C3r-cover-tolB,C3r-cover-tolC,C3r-nocover,I-9.6t,C4g-inh,C2i-inh,C2ig-inh]
step=6 op=create newest=2:10 F=0 table=[1:7] h0{wm=3 tree=[1@3,2@10] rows_create=[3:2]} lineage=[5<-3|cur4:H[2]:n[1]:Hnew[2],7<-5|cur6:H[3]:n[2]:Hnew[2]] ring=[1:8:wm[3],2:9:wm[2],2:10:wm[3],1:7:wm[2]] red=[C2aL,C3rL,C3rL-pooled,C4g,C4g-maxW,C4g-firstW,C2i,C2i-inst,C2ig,C2a-create-strict,C2a-create-tolC,C3r-cover-strict,C3r-cover-tolB,C3r-cover-tolC,C3r-nocover,C4g-inh,C2i-inh,C2ig-inh]
```

第 3 步 V1 那一次回退把水位从 3 拉回 2，第 4 步重发 2 号（2@8 对 2@6），第 5 步崩溃恢复抛弃 (1, 8)、第 6 步再建 2@10：这时 I-9.29t、I-9.6t 都不红（洞），台阶行只剩 `3:2`（变异），③ 本该红（水位 3 盖不住），而环里被抛弃的 (1, 8) 水位 3，tolB 照「跳升只来自被抛弃的根」把它容许了。

判（K2 维持那一支）：
- 「会不会顺带放过一形真重发」：**对 C2aL、C3rL、C4g、C2i 没打中**——它们没有要容许的那一句；**对「容许」写成判词的两种读法打中（双故障）**：tolB 在产品取法上照样误报（来源根转出环就核不了），tolC 不误报，但两者都把「历史写错」当跳升放过，再叠一次真重发就漏（上表，洞事件最多 48 个）。这是 ③ 的弱读法被打中，不是三个零轮形态被打中。
- 推出来的一句（推的）：维持 D23 已定项 14 时，正文那句「历史核对容许跳升只来自被抛弃的根」不该写进不变量；应写成写者纪律「回退那次跳升也留一行台阶」（C2aL 的定义已经这样），③ 照字面判、不开例外。

## 三、K2：改成只取没被抛弃的根

### 1. 在模型上：等于「只取 cur 内存里的」，C143 那一格不退回去

- 产品取法下，按 cur 的实例表没被抛弃、读得出的根，水位从来不高于 cur 内存里的：十二个字母长 7 两组、十个字母长 7 四组，`rollback_kept_ring_above_cur=0`（`expected/verify-0.out`、`verify-1`、`y1x-0` … `y1x-3` 的第 1 行）。所以第二种取法在每一个量到的状态上都等于「max(cur 内存里的)」，环里那一读只对被抛弃的根起作用——与第二轮攻方「跳升全部来自被抛弃的根」是同一件事的另一面。
- C143 定案那一格靠的是 **cur 内存那一项**，不是环那一项：回退那一刻 cur 的根读不出时，留着 cur 项的三种取法（维持、只取没被抛弃的、实例表读不出时两种处置）重发 0；把 cur 项去掉的两种（全部根 / 只取没被抛弃的根）各重发 3362 / 3509 次、洞事件 1143 / 1218 个——滤不滤被抛弃的根不改变这件事：

```
$ cd research/prompts/m3-w3-r3-opus-model/expected; for f in y1x-0 y1x-1 y1x-2 y1x-3 y1x-4 y1x-5; do head -1 $f.out | grep -oE "(wm_rule|table_fallback|images|events|hole_events|rollback_kept_ring_above_cur|reissues_across_abandoned_roots)=[A-Za-z0-9]+" | tr "\n" " " | sed "s/^/$f /"; echo; done
y1x-0 wm_rule=MaxCurAllRing table_fallback=Refuse images=976584 images=976584 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=2381 rollback_kept_ring_above_cur=0 
y1x-1 wm_rule=MaxCurKeptRing table_fallback=Refuse images=976584 images=976584 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=2966 rollback_kept_ring_above_cur=0 
y1x-2 wm_rule=MaxCurKeptRing table_fallback=AllKept images=1919749 images=1919749 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=3978 rollback_kept_ring_above_cur=0 
y1x-3 wm_rule=MaxCurKeptRing table_fallback=CurOnly images=1919749 images=1919749 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=4235 rollback_kept_ring_above_cur=0 
y1x-4 wm_rule=MaxAllRingOnly table_fallback=Refuse images=976584 images=931000 events=3362 images=7838 hole_events=1143 images=6674 reissues_across_abandoned_roots=2309 rollback_kept_ring_above_cur=8370 
y1x-5 wm_rule=MaxKeptRingOnly table_fallback=Refuse images=976584 images=929120 events=3509 images=7902 hole_events=1218 images=6708 reissues_across_abandoned_roots=2875 rollback_kept_ring_above_cur=8601 
```

（`grep -oE` 把 `legal_images`、`residual_images`、`hole_residual_images` 都截成了 `images=…`：每行三个 `images=` 依次是 images、legal_images、residual_images，最后一个是 hole_residual_images。）

### 2. 「实例表读不出、根读不出怎么办」

- **cur 的实例表读不出**：今天的代码在算水位之前就拒了整次回退——「.map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7634`），候选集本来就要这张表。所以第二种取法不必为它另立一条；若有人另给一个退路，模型里两种退路：「当全部没被抛弃」把跳升带回来（字面 C2a 的 ③ 误报 21114，`expected/y1x-2.out`），「环里一条不取」0（`y1x-3`）；两种都不重发。
- **根读不出**：回退那一刻 cur 的根读不出，cur 内存项兜住（上一小节）；被抛弃的根读不出，两种取法都不算它；没被抛弃的根读不出，它的水位本来不高于 cur（`rollback_kept_ring_above_cur=0`），不算它不改结果。代码里一条根的那一版重建不出来就跳过：「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7456`）。
- 一处要连带写明的不对称（推的）：树 ID 水位在同一处仍取环里全部根，代码注释（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1341` 那一行，那一行自己带书名号，不整行搬进引号）写的理由是被抛弃时间线上的根也算、它们仍承载那一代发过的号——树的身份就是树 ID，没有出生代可分；inode 号有出生代分，所以 Y1 只改 inode 号水位是自洽的，但条款那一格今天把两个水位写在同一句里，改的时候要拆开。

### 3. 在仓副本上量过（实现真走的版本链）

在冻结树副本的 `mount.rs` 回退那一处只多一个滤（`proto-patch/patch-Y1-kept.diff`，16 行；回退水位只取按 cur 的实例表没被抛弃的根），第二轮攻方的原型原样重跑：

- `probe`（四个点名状态逐步打印）：基线与第二轮 `expected/probe-AB.out` 逐行相同；打补丁之后状态 524287 上回退的水位从 4 变成 3（`c1`）与 2（`1`），四种弱读法都不再红，池级 checker 照旧 0 红：

```
$ diff <(grep "^W3R2TRACE state=524287" probe-base.out) <(grep "^W3R2TRACE state=524287" probe-kept.out)
7,8c7,8
< W3R2TRACE state=524287 effective=(1, 3) script=c1 key=(2, 9) wm=4 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2), (8, 3)] rows_all=[(3, 2), (8, 3), (9, 4)] log=[(1, 3), (2, 8)] lineage=[(9, 7, 8, 3, 2, 4)] red=[C2a-create:I930-3,C3r:I930-3,C2a-create:I930-123,C2aL-all:pair(Tcur,Hnew)] real=[]
< W3R2TRACE state=524287 effective=(1, 3) script=1 key=(2, 8) wm=4 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2)] rows_all=[(3, 2), (8, 4)] log=[(1, 3)] lineage=[(8, 6, 7, 2, 1, 4)] red=[C2a-create:I930-3,C3r:I930-3,C2a-create:I930-123,C2aL-all:pair(Tcur,Hnew)] real=[]
---
> W3R2TRACE state=524287 effective=(1, 3) script=c1 key=(2, 9) wm=3 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2), (8, 3)] rows_all=[(3, 2), (8, 3)] log=[(1, 3), (2, 8)] lineage=[(9, 7, 8, 3, 2, 3)] red=[] real=[]
> W3R2TRACE state=524287 effective=(1, 3) script=1 key=(2, 8) wm=2 tree=[(1, 3)] legal=true cross=false rows_create=[(3, 2)] rows_all=[(3, 2)] log=[(1, 3)] lineage=[(8, 6, 7, 2, 1, 2)] red=[] real=[]
```

- B 历史（P 那次发布的根槽 FUA 持久、系统配置轮换没持久，下一次可写挂载整次读 P 的根槽与 P 写的单元读回全 0，挂上之后读得出；然后 `c1`）全域 65548 个状态、二十片：打补丁之后 196629 张镜像全合法，没有一行 `w3b_fp_*`，也没有 `w3b_rollback_jumps`；第二轮同一段（未打补丁）是四种弱读法各误报 4、跳升 4：

```
$ grep -E "^W3R2(LAYER0|COUNT)" kept-merge-of-20.out | sed -E "s/width=[^ ]* exp=[^ ]* creates_before=- p_count=2 scripts=x shard=[^ ]* //"
W3R2LAYER0 states=65548 observed_states=65548 violations=0 root_persisted_states=9 failed_states=0 checker_violated={} first_violation=None
W3R2COUNT w3b_cross_abandoned_reissue_images=65543
W3R2COUNT w3b_images=196629
W3R2COUNT w3b_legal_images=196629
W3R2COUNT w3b_mount_ok=65543
W3R2COUNT w3b_mount_refused_newerstatestillunreadableafteronereread=5
W3R2COUNT w3b_p_abandoned_states=65543
W3R2COUNT w3b_p_abandoned_with_readable_root_states=4
W3R2COUNT w3b_rollbacks=65543
W3R2COUNT w3b_states=65548
$ grep -E "^W3R2COUNT .*w3b_(fp_|rollback_jumps)" research/prompts/m3-w3-r2-opus-model/expected/ab-merge-of-20.out | sed -E "s/width=[^ ]* exp=[^ ]* creates_before=- p_count=2 scripts=x shard=[^ ]* //"
W3R2COUNT w3b_fp_c2a_create_i930_123=4
W3R2COUNT w3b_fp_c2a_create_i930_3=4
W3R2COUNT w3b_fp_c2al_all_pair_tcur_hnew_=4
W3R2COUNT w3b_fp_c3r_i930_3=4
W3R2COUNT w3b_rollback_jumps=4
```

  这是副本上的数：一个历史（`c1`）、一种故障（B），不是入库装置；二十片的分片、合并照第二轮原型自己的做法（每片约 41–98 秒，合计约 17 分钟）。A 历史（只有一次崩溃）在纯崩溃下没有被抛弃的根，第二种取法与今天逐字相同，没另跑全域（`probe` 里 A 的四个点名状态两边相同）。

### 4. 代价与判

- 第二种取法的代价（量过，模型）：合法的跨抛弃重发变多——十二个字母长 7、清扫阈值「从不」48356 → 53960（+11.6%，`expected/verify-0.out` 对 `y1-0.out` 第 1 行），阈值 0 35042 → 36326（+3.7%，`verify-1` 对 `y1-1`）。这些都是 D8 已定项 6 射程允许的（被崩溃恢复抛弃的那一段里发过的号），没有一张非法镜像；多出来的是今天的跳升顺手避开的那几个号。一次回退少一遍「重建环里每条根那一版」的读（推的，没量读的量）。
- 判（K2 改法那一支）：**没打中**——C143 定案「回退那次取 max、不重发」那一格不退回去：它靠 cur 内存项，第二种取法留着它；实例表读不出时代码先拒；根读不出不改结果。四个零轮形态在第二种取法下照样 0 误报 0 漏（`y1-0`、`y1-1`、`y1x-1` … `-3`），字面 C2a 与 C3r-cover 的 ③ 也不再要容许（误报 163700 → 0，`verify-0` 对 `y1-0`）。
- 推翻条件：找到一个产品可达的状态，按 cur 的实例表没被抛弃、读得出的根，水位高于 cur 内存里的（模型里 `rollback_kept_ring_above_cur` 不为 0），那第二种取法就不等于「只取 cur」，要重新判它会不会把这一段号重发。

## 四、K3：克隆头（两个可写头，号会相交，唯一性靠 (树 ID, inode)）

`crates/` 今天没有克隆；回退目标的树 ID 集合与 cur 不同时整次拒：「if target_version.tree_identifiers != current.tree_identifiers {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7680`）。所以只能按条文在模型里走：两个头共用一条根链与一个全池的谱系树，回退是全池的（每个头的树取 R_old 的、水位按头取 max），模型照代码拒跨克隆的回退。九个字母（`create`、`create1`、`clone`、`rollback1`、`rollback2`、`empty`、`mount`、`crash-hide`、`delete`）长 7、环 4，八组。

### 1. 不分头的读法误报（产品取法，打中）

```
$ for f in clone-0 clone-1; do for c in I-9.29-pooled C3rL-pooled C4g-maxW C4g-firstW C2i-inst C2aL C3rL C4g C2i; do grep -E "^W3R3COUNT .* candidate=$c " $f.out | sed -E "s/.*candidate=([^ ]*) false_positive=([0-9]+) event_missed=([0-9]+).*/$f \\1 fp=\\2 ev=\\3/"; done; done
clone-0 I-9.29-pooled fp=46518 ev=0
clone-0 C3rL-pooled fp=52687 ev=0
clone-0 C4g-maxW fp=62532 ev=0
clone-0 C4g-firstW fp=35130 ev=0
clone-0 C2i-inst fp=158198 ev=0
clone-0 C2aL fp=0 ev=0
clone-0 C3rL fp=0 ev=0
clone-0 C4g fp=0 ev=0
clone-0 C2i fp=0 ev=0
clone-1 I-9.29-pooled fp=28637 ev=0
clone-1 C3rL-pooled fp=30696 ev=0
clone-1 C4g-maxW fp=30262 ev=0
clone-1 C4g-firstW fp=21552 ev=0
clone-1 C2i-inst fp=104829 ev=0
clone-1 C2aL fp=0 ev=0
clone-1 C3rL fp=0 ev=0
clone-1 C4g fp=0 ev=0
clone-1 C2i fp=0 ev=0
$ grep -h "W3R3EXAMPLE" clone-0.out | grep -E "candidate=(I-9.29-pooled|C3rL-pooled|C4g-maxW|C4g-firstW|C2i-inst) kind=false_positive" | sed -E "s/.*candidate=//"
I-9.29-pooled kind=false_positive ops=[clone,create,create1]
C3rL-pooled kind=false_positive ops=[clone,create,create1]
C4g-maxW kind=false_positive ops=[clone,create,rollback1]
C4g-firstW kind=false_positive ops=[clone,create,rollback1]
C2i-inst kind=false_positive ops=[clone,create]
```

（clone-0：从 cur 克隆，492488 张镜像全合法；clone-1：从回退候选集里最新的那一版克隆，394014 张全合法。）读法：`[clone,create,rollback1]` 两头都从水位 2 起，头 0 建一个到 3，回退那条谱系记录 H_cur = [3, 2]；只带一个 W 的读法取 3，头 1 的水位 2 < 3 当场红。`[clone,create,create1]` 两头各发 2 号（出生代不同），不带树 ID 的发号记录与今天 I-9.29 的字面（「同一个 inode 号只许对应一个对象出生代」，不分树）都判成「同号两个出生代」。

判：C4g 与 C2i 的定义里「谱系带 W」「每实例一条」**必须按头**，C3rL 的 (号, 出生代) **必须带树 ID**；按头的写法（C2aL、C3rL、C4g、C2i 那四行）在这两组上 0 误报。代价（算术，推的）：谱系记录 16 → 16 + 8k 字节（k 是回退那一刻的可写头数；或每头一条 (T_rb, T_old, 树 ID, W) 32 字节，一个容器 ⌊(32768 − 136) ÷ 32⌋ = 1019 条，今天 16 字节 2039 条）；C3rL 每条发号记录 16 → 24 字节（2039 → 1359 条一容器），或每头一棵发号树。附带：今天的 I-9.29 有了克隆也要按树判（它红在 `[clone,create,create1]` 上），这一条不归 W3，记在这里交主 agent。

### 2. 克隆那次的台阶行（C2aL，打中一形读法）

新头的水位取 origin 现行的（不取快照那一版的）而克隆那次不留台阶行时，C2aL 在 `[create,clone]` 上就误报（③：新头的历史是快照的，盖不住新头的水位）；留那一行就 0：

```
$ for f in clone-2 clone-3; do head -1 $f.out | grep -oE "clone=[^ ]*" | tr "\n" " "; grep -E "^W3R3COUNT .* candidate=C2aL " $f.out | sed -E "s/.*candidate=([^ ]*) false_positive=([0-9]+) event_missed=([0-9]+).*/\\1 fp=\\2 ev=\\3/"; done; grep -h W3R3EXAMPLE clone-3.out | grep "candidate=C2aL kind=false_positive" | sed -E "s/.*candidate=//"
clone=OlderVersion/OriginCurrent/row=true C2aL fp=0 ev=0
clone=OlderVersion/OriginCurrent/row=false C2aL fp=29974 ev=0
C2aL kind=false_positive ops=[create,clone]
```

判：C2aL 的定义「每次水位变了的发布留一行」要明写**含克隆那次**（新头的水位与它继承的历史最高行不同时）。与第二轮「回退跳升那次也留一行」同一个根子：凡是不经建 inode 改水位的发布都要留行。

### 3. 继承来的时间线：按头的 C4g、C2i 漏（打中，V1 类臂）

D8 已定项 6 的「时间线」是所选根的祖先链；克隆头的祖先链在克隆之前就是 origin 那个头的（含它被管理员回退抛弃的那一段）。按头的 C4g 只认「回退那一刻就有这个头」的谱系值，新头继承来的那一段一条都不认；C2i 的新头条目从克隆那一刻才起。V1 在克隆之前发生、克隆又从那段退回的版本起，新头的第一次建就重发：

```
$ for f in clone-5 clone-6 clone-7; do head -1 $f.out | grep -oE "(wm_rule|clone|events|hole_events)=[A-Za-z0-9/=]+" | tr "\n" " "; echo; for c in C2aL C3rL C4g C2i C2ig C4g-inh C2i-inh C2ig-inh; do grep -E "^W3R3COUNT .* candidate=$c " $f.out | sed -E "s/.*candidate=([^ ]*) false_positive=[0-9]+ event_missed=([0-9]+) residual_missed=([0-9]+) hole_event_missed=([0-9]+) hole_residual_missed=([0-9]+).*/  \\1 ev=\\2 res=\\3 hev=\\4 hres=\\5/"; done; done
wm_rule=Target clone=OlderVersion/Snapshot/row=true events=9785 hole_events=4379 
  C2aL ev=0 res=0 hev=0 hres=0
  C3rL ev=0 res=0 hev=0 hres=0
  C4g ev=220 res=577 hev=219 hres=470
  C2i ev=237 res=2770 hev=236 hres=2289
  C2ig ev=220 res=577 hev=219 hres=470
  C4g-inh ev=0 res=0 hev=0 hres=0
  C2i-inh ev=0 res=1798 hev=0 hres=1425
  C2ig-inh ev=0 res=0 hev=0 hres=0
wm_rule=TargetHead0Only clone=OlderVersion/Snapshot/row=true events=8886 hole_events=4330 
  C2aL ev=0 res=0 hev=0 hres=0
  C3rL ev=0 res=0 hev=0 hres=0
  C4g ev=220 res=580 hev=219 hres=473
  C2i ev=237 res=2713 hev=236 hres=2268
  C2ig ev=220 res=580 hev=219 hres=473
  C4g-inh ev=0 res=0 hev=0 hres=0
  C2i-inh ev=0 res=1737 hev=0 hres=1400
  C2ig-inh ev=0 res=0 hev=0 hres=0
wm_rule=TargetHead0Only clone=Current/Snapshot/row=true events=10969 hole_events=4900 
  C2aL ev=0 res=0 hev=0 hres=0
  C3rL ev=0 res=0 hev=0 hres=0
  C4g ev=19 res=350 hev=19 hres=349
  C2i ev=35 res=2784 hev=35 hres=2361
  C2ig ev=19 res=350 hev=19 hres=349
  C4g-inh ev=0 res=0 hev=0 hres=0
  C2i-inh ev=0 res=2009 hev=0 hres=1589
  C2ig-inh ev=0 res=0 hev=0 hres=0
```

最短反例（C4g 漏一个洞事件；复跑：`W3R3_TRACE=1 w3r3-model clone 5`，取 `W3R3TRACE candidate=C4g kind=hole_event_missed` 那几行；下面去掉了台阶行、发号记录、c2i、环、journal 与判红名单几列）：

```
$ grep 'W3R3TRACE candidate=C4g kind=hole_event_missed' clone-5-trace-final.out | sed -E 's/^W3R3TRACE candidate=[^ ]* kind=[^ ]* //; s/ (rows_all|rows_create|log)=\[[^]]*\]//g; s/ (c2i|journal|ring)=\[[^]]*\]//g; s/ red=\[.*//'
step=start newest=1:3 F=0 table=[] h0{wm=2 tree=[1@3] issued=[1@3]} lineage=[]]
step=0 op=create newest=1:4 F=0 table=[] h0{wm=3 tree=[1@3,2@4] issued=[1@3,2@4]} lineage=[],1:3:wm[2]]
step=1 op=rollback1 newest=1:5 F=0 table=[] h0{wm=2 tree=[1@3] issued=[1@3,2@4]} lineage=[5<-3|cur4:H[3]:n[2]:Hnew[2]],1:5:wm[2],1:3:wm[2]]
step=2 op=rollback1 newest=1:6 F=0 table=[] h0{wm=3 tree=[1@3,2@4] issued=[1@3,2@4]} lineage=[5<-3|cur4:H[3]:n[2]:Hnew[2],6<-4|cur5:H[2]:n[2]:Hnew[3]],1:5:wm[2],1:6:wm[3],1:3:wm[2]]
step=3 op=clone newest=1:7 F=0 table=[] h0{wm=3 tree=[1@3,2@4] issued=[1@3,2@4]} h1{wm=2 tree=[1@3] issued=[1@3,2@4]} lineage=[5<-3|cur4:H[3]:n[2]:Hnew[2],6<-4|cur5:H[2]:n[2]:Hnew[3]],1:5:wm[2],1:6:wm[3],1:7:wm[3, 2]]
step=4 op=create1 newest=1:8 F=0 table=[] h0{wm=3 tree=[1@3,2@4] issued=[1@3,2@4]} h1{wm=3 tree=[1@3,2@8] issued=[1@3,2@4,2@8]} lineage=[5<-3|cur4:H[3]:n[2]:Hnew[2],6<-4|cur5:H[2]:n[2]:Hnew[3]],1:5:wm[2],1:6:wm[3],1:7:wm[3, 2]]
```

第 1 步 V1 把头 0 的水位从 3 拉回 2（2@4 已发过），第 2 步又回到 3；第 3 步从回退候选集里最新的那一版（txg 5，水位 2，它的祖先链上发过 2@4）克隆出头 1；第 4 步头 1 建 2@8——头 1 的祖先链上 2 号第二个出生代。今天的 I-9.29t、I-9.6t 都不红（洞）；C4g 按头只认「回退那一刻有头 1」的谱系值，两条谱系记录都在克隆之前，头 1 一条都不认，所以不红；C2aL、C3rL 不漏，因为它们的历史是随快照那一版抄进新头的（台阶 `4:3` 在，水位 2 当场 ① 红；发号记录里 2@4 在）。
补法（我提的，被攻过零轮）：克隆头另按快照那一版的谱系记录判（C4g-inh），用 origin 头那一列的 W；C2i 同理把快照那一版 origin 头的条目带进新头（C2i-inh）。上表 C4g-inh、C2ig-inh 三组都 0；C2i-inh 照旧漏持久镜像（与单头时 C2i 漏持久镜像同一个毛病）。头条目今天的字段表里有「origin 链接 / 克隆时刻」8 字节（`.claude/kb/decisions/08-核心索引结构.md:264`），快照那一版的根随快照留着，C4g-inh 要读的东西不必新加格式（推的：快照留住的是不是含谱系树的整条根，没查）。
判别它的东西系统当时看得到：克隆那一刻快照那一版的谱系树就在快照的根下。

（ring 那一列的 sed 没剥干净：行尾残留的 `],1:5:wm[2],…]` 是环里各根的「实例:txg:wm[各头水位]」。）

## 五、K3：副本坏一份（w = 2）

- 三个形态的历史（C2aL 的台阶行在记账树、C3rL 的发号树、C4g / C2i 与 C2aL、C3rL 的谱系那一份在谱系容器）都是带两条位置条目的普通单元。产品与 checker 读它们都是「逐条试，整单元校验和等于条目里的才算读到」：「for location in locations {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:385`）、「if crc32_castagnoli(&bytes) == location.unit_checksum {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:393`）。一份坏了读另一份，读到的逐字节相同；旧内容的那一份过不了条目里的校验和，读不成「另一个版本」。所以坏一份不改变任何一个形态判什么（推的，按代码）。两份都坏是走读失败，照 checker 的「两份都读不到对得上的」报。
- 坏一份能改变的是**时间线**，不是历史怎么读：
  - 一次只有记录持久、根没持久的发布，点名单元有一份坏了，前缀第四条「两份都验过才施加」让它整次不施加——与崩溃恢复抛弃同一个结局。这一形在模型里就是 `crash-hide`（被抛弃）与 `crash-hide-root-and-record` 两个字母，第二、三节的每一组都在扫；
  - 维持 D23 已定项 14 时，被抛弃那条根那一版的账有一处两份都读不出，回退算水位时那条根整条跳过（「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7456`）——跳不跳升取决于被抛弃根的介质好坏；第二节 tolB 那一读法要事后核跳升的来源，来源根再坏一份读不出就核不了、误报。第二种取法下这一读根本不做。
- 判：**没打中**。扫了什么：按代码（读的两处），与模型里两个抛弃字母（它们在时间线上的效果与「坏一份不施加」相同）；没有在仓副本上注入坏一份的单元。

## 六、K3：新实例第一个 txg 被改坏，与被抛弃发布的出生代撞上

今天的取法：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1317`）。模型里两条臂：照代码取 max(读得出的根, 读得出的记录) + 1，与改坏成「所选根 + 1」；九个字母（`create`、`delete`、`rollback1`、`empty`、`mount`、`crash-hide`、`crash-hide-root-and-record`、`switch-redo`、`switch-error`）长 7、环 4、journal 8 条：

```
$ for f in first-0 first-1; do head -1 $f.out | grep -oE "(first_txg|images|legal_images|events|hole_events|reissues_across_abandoned_roots|identity_collisions|images_with_cross_instance_txg_not_increasing)=[A-Za-z0-9]+" | tr "\n" " "; echo; for c in C2aL C3rL C4g C2i C2ig I-9.29t I-9.6t; do grep -E "^W3R3COUNT .* candidate=$c " $f.out | sed -E "s/.*candidate=([^ ]*) false_positive=([0-9]+) event_missed=([0-9]+).*/  \\1 fp=\\2 ev=\\3/"; done; done
first_txg=Correct images=2182621 legal_images=2182621 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=21116 identity_collisions=0 images_with_cross_instance_txg_not_increasing=0 images_with_cross_instance_txg_not_increasing=0 
  C2aL fp=0 ev=0
  C3rL fp=0 ev=0
  C4g fp=0 ev=0
  C2i fp=0 ev=0
  C2ig fp=0 ev=0
  I-9.29t fp=0 ev=0
  I-9.6t fp=0 ev=0
first_txg=SelectedPlusOne images=2173713 legal_images=2173713 events=0 images=0 hole_events=0 images=0 reissues_across_abandoned_roots=0 identity_collisions=0 images_with_cross_instance_txg_not_increasing=0 images_with_cross_instance_txg_not_increasing=1088952 
  C2aL fp=0 ev=0
  C3rL fp=0 ev=0
  C4g fp=0 ev=0
  C2i fp=0 ev=0
  C2ig fp=0 ev=0
  I-9.29t fp=0 ev=0
  I-9.6t fp=0 ev=0
```

读法：
- 「(号, 出生代) 逐字撞上」一次都没有（`identity_collisions=0`，两条臂都是）。原因在根环的落点只看 txg：「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/root_ring.rs:114`）、「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/root_ring.rs:115`），新实例的第一个 txg 若等于被抛弃那次发布的 txg，它的根就写在那条被抛弃的根的同一槽里、把它盖掉；新实例在这个 txg 上只发写行那一次（不建 inode），之后建的出生代都更大。所以「同号同代」只可能停在被抛弃那次发布写下的孤儿单元里，任何一张镜像上都不会有两条根各带一个——三个形态与现有检查都没有东西可看，这一形也不在 W3 的射程里（被抛弃的那一段，不是同一条时间线）。改坏之后 `reissues_across_abandoned_roots` 从 21116 变 0，就是被抛弃的根都被同 txg 的新根盖掉了。
- 照代码那一臂里，`crash-hide-root-and-record`（被抛弃那条根与它的记录在挂载那一刻都读不出，系统配置没见证它）也让第一个 txg 等于被抛弃那次的 txg（读不出的那条不算进 max），结局与改坏那一臂同：盖掉、不撞。这是今天的代码在多故障下就走得到的形态（推的：没在仓副本上造）。
- 改坏留下的唯一一处单镜像痕迹在 journal：被抛弃那次发布的记录还读得出、排在新实例的记录前面，换到更大的实例代号而 txg 没增——改坏那一臂 2173713 张里 1088952 张有这一形，照代码那一臂 0 张。I-8.12（`.claude/kb/invariants.md:101`）今天明写换实例不判这一格，所以改坏这一行**今天没有任何检查会红**，三个形态也不会（它们只看最新那条根下的东西）。

判：**没打中**（三个形态在这一形上没有漏报也没有误报，撞号造不出来）；附带看到「新实例第一个 txg 取错」这一行改坏今天全仓无红（I-8.12 不判换实例），交主 agent，不归 W3。扫了什么：上面两条臂的全部历史；没有在仓副本上改坏 `first_txg_of_new_instance` 跑层 0。

## 七、前两轮的数能不能核（在我从头写的模型上独立重算）

我的模型不拷前两轮的源码，只按它们文档里写的走法重写；下面每个数都是这一份模型自己跑出来的，与前两轮报告里的逐个比。

| 出处（前两轮报告） | 报的数 | 我的模型 | 对不对得上 |
|---|---|---|---|
| 第二轮攻方，P 臂环 4 长 7 阈值「从不」（`research/prompts/m3-w3-r2-opus-output.md` 第三节那一行 `W3R2MODEL`） | 历史 9863012、镜像 10836269、跨抛弃重发 48356、跳升只来自被抛弃的根 55040、来自时间线上的根 0、切换 1946516 | `expected/verify-0.out` 第 1 行同一组 | 全对上 |
| 同上阈值 0 | 9985685、10964563、35042、46545、0、清扫 2584263、切换 1957758 | `verify-1` 第 1 行 | 全对上 |
| 第二轮攻方，字面 C2a 配 ③、(T_cur, H_new) 那一对的误报 | 163700、231204（阈值「从不」）；152330（阈值 0） | `verify-0`、`verify-1` 的 `C2a-create-strict`、`pair(Tcur,Hnew)` | 全对上 |
| 第二轮辩方，V1 臂环 4 长 7 | 镜像 10836269、合法 7914147、事件 340776、I-9.29 不红的 144904、洞事件 138372、持久镜像 443952、其中洞 352888；C4g 漏 0 / 0；C2i 持久镜像漏 17324、其中洞 13486 | `verify-2` | 全对上 |
| 第二轮辩方，V3 臂 | 镜像 10825489、合法 5323861、事件 648336、洞事件 76552、持久镜像 873440、其中洞 190236；C4g 误报列 10218、漏事件 508048、洞事件 12056、持久镜像 767560、其中洞 149416；C2i 误报列 62346、漏 0 / 0 | `verify-4` | 全对上 |
| 第二轮攻方原型 `probe`（仓副本，同一份原型、同一条命令） | `expected/probe-AB.out` | 我在自己的副本里重编重跑 | 59 行逐行相同 |
| 第二轮攻方代价算术 | 一片记账叶余 462 行；谱系 24 字节一容器 1359 条；P 那条流闭式 65543，多 4 写 1048583 | `python3` 重算 | 477 − 15 = 462；⌊32632 ÷ 24⌋ = 1359；1 + Σ(2^段 − 1)：[16,2,1,2] 65543、[20,2,1,2] 1048583 |

```
$ cd research/prompts/m3-w3-r3-opus-model/expected; for f in verify-0 verify-1 verify-2 verify-4; do head -1 $f.out | grep -oE " (full_length_histories|images|legal_images|events|residual_images|hole_events|hole_residual_images|reissues_across_abandoned_roots|rollback_jumps_from_abandoned_roots_only|rollback_kept_ring_above_cur|sweeps|switches)=[0-9]+" | tr -d "\n" | sed "s/^/$f/"; echo; for c in C2a-create-strict "pair\\(Tcur,Hnew\\)" C4g C2i I-9.29t; do grep -E "^W3R3COUNT .* candidate=$c " $f.out | sed -E "s/.*candidate=//; s/ red_on_collision=0//; s/^/  /"; done; done
verify-0 full_length_histories=9863012 images=10836269 legal_images=10836269 events=0 residual_images=0 hole_events=0 hole_residual_images=0 reissues_across_abandoned_roots=48356 rollback_jumps_from_abandoned_roots_only=55040 rollback_kept_ring_above_cur=0 sweeps=0 switches=1946516
  C2a-create-strict false_positive=163700 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  pair(Tcur,Hnew) false_positive=231204 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  C4g false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  C2i false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  I-9.29t false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
verify-1 full_length_histories=9985685 images=10964563 legal_images=10964563 events=0 residual_images=0 hole_events=0 hole_residual_images=0 reissues_across_abandoned_roots=35042 rollback_jumps_from_abandoned_roots_only=46545 rollback_kept_ring_above_cur=0 sweeps=2584263 switches=1957758
  C2a-create-strict false_positive=152330 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  pair(Tcur,Hnew) false_positive=218538 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  C4g false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  C2i false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  I-9.29t false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
verify-2 full_length_histories=9863012 images=10836269 legal_images=7914147 events=340776 residual_images=443952 hole_events=138372 hole_residual_images=352888 reissues_across_abandoned_roots=47880 rollback_jumps_from_abandoned_roots_only=65068 rollback_kept_ring_above_cur=395112 sweeps=0 switches=1946516
  C2a-create-strict false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  pair(Tcur,Hnew) false_positive=0 event_missed=340776 residual_missed=443952 hole_event_missed=138372 hole_residual_missed=352888
  C4g false_positive=0 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  C2i false_positive=0 event_missed=0 residual_missed=17324 hole_event_missed=0 hole_residual_missed=13486
  I-9.29t false_positive=0 event_missed=138764 residual_missed=367088 hole_event_missed=138372 hole_residual_missed=352888
verify-4 full_length_histories=9852680 images=10825489 legal_images=5323861 events=648336 residual_images=873440 hole_events=76552 hole_residual_images=190236 reissues_across_abandoned_roots=20356 rollback_jumps_from_abandoned_roots_only=50002 rollback_kept_ring_above_cur=692822 sweeps=0 switches=1945620
  C2a-create-strict false_positive=89298 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  pair(Tcur,Hnew) false_positive=190940 event_missed=647120 residual_missed=870216 hole_event_missed=76480 hole_residual_missed=188428
  C4g false_positive=10218 event_missed=508048 residual_missed=767560 hole_event_missed=12056 hole_residual_missed=149416
  C2i false_positive=62346 event_missed=0 residual_missed=0 hole_event_missed=0 hole_residual_missed=0
  I-9.29t false_positive=0 event_missed=79720 residual_missed=372556 hole_event_missed=76552 hole_residual_missed=190236
```

判：第二轮两条腿在模型上报的关键数，在一份不共用源码的模型上全部逐个对上；第二轮攻方的原型 `probe` 在我自己的副本里重编重跑逐行相同。对不上的一处只在我自己这边：第一次只按树判 I-9.29、I-9.6（没加墓碑那两半）时，洞事件 V1 是 144024、V3 是 122592，加上第二轮辩方写的墓碑两半之后才是 138372、76552（那一次的输出文件被同名覆盖了，这句只是读数记录，不当证据）——「洞」口径的数对墓碑那两半敏感，代价表按洞数时要用带墓碑的口径。
没核的：第二轮攻方二十片全域 A + B（约 35 分钟，我只全域重跑了打过补丁的 B）、第二轮辩方的装置段（E175 装置 G-long、F2 两臂）、第一轮的任何数。

## 八、每个打中先答四句

| 打中 | 分不分辨臂（候选） | 被判的系统当时看不看得到判别它的东西 | 满足的是判据字面的哪一个分句 | 跑前条款给的改法在这几格上 |
|---|---|---|---|---|
| ③ 的容许读法 tolB、tolC 在「历史写错 + 真重发」上漏（第二节） | 分辨：同一组镜像上 strict ③ 与 C2aL、C3rL、C4g、C2i 都不漏 | 看得到：strict ③ 判的就是同一张镜像上的台阶行与水位；容许读法自己丢掉了这一判 | 第五节「一形漏报」（同一条时间线上重发而判不红），且是双故障：要先有「历史写错」那一类变异，K3 第二轮已立「写错或不写必须红」 | 跑前条款只有「交用户表写明那一形与代价」：这一形只打中 ③ 的容许读法，三个零轮形态不受；改法是「不写容许，写回退跳升也留行」（C2aL 已经这样） |
| 克隆：不分头的 C4g（maxW / firstW）、C2i-inst、不带树 ID 的 C3rL 误报（第四节 1） | 分辨：同一组镜像上按头的 C4g、C2i、带树 ID 的 C3rL 0 误报 | 看得到：各头的水位与树 ID 都在最新那条根下 | 第五节「一形误报」（合法镜像判红） | 改法是定义收严（按头、带树 ID），代价见第四节 1；这几格上收严之后 0 |
| 克隆：克隆那次不留台阶行时 C2aL 误报（第四节 2） | 分辨：留那一行的同一组 0 | 看得到：克隆那次发布自己知道新头的水位 | 「一形误报」 | 定义里写明「含克隆那次」；这一格上 0（`clone-2`） |
| 克隆：按头的 C4g、C2i 漏继承来的那一段（第四节 3） | 分辨：同一组 C2aL、C3rL 不漏 | 看得到：快照那一版的谱系记录在快照的根下（推的：快照留的是整条根） | 「一形漏报」；只在 V1 类臂上（产品取法上回退不会把水位拉低，克隆的快照不会是退回过的版本） | C4g-inh、C2i-inh（我提的，零轮）：C4g-inh 三组 0；C2i-inh 照旧漏持久镜像 |
| 今天的 I-9.29 有了克隆就误报（第四节 1，附带） | 不归 W3 | 看得到 | I-9.29 的字面不分树 | 不归这一轮 |

## 九、改法各修哪一格（都只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 标 | 依据 |
|---|---|---|---|
| Y1 取第二种（回退的 inode 号水位只取按 cur 的实例表没被抛弃的根，cur 内存项留着） | 第二轮那一跳带来的 ③ 误报；本轮 tolB / tolC 不再需要 | 量过（模型 + 仓副本） | 模型 `verify-0` 对 `y1-0`：163700 → 0；仓副本 B 历史全域：四种弱读法各 4 → 0、跳升 4 → 0（`expected/kept-merge-of-20.out`） |
| 维持 Y1 时不写「容许」判词，写成「回退跳升那次也留一行」 | 第二节 tolB / tolC 的漏 | 量过 | `y1m-0`、`y1m-2`：strict 那一读法漏 0 |
| C4g 的谱系值、C2i 的条目按头带 | 第四节 1 的误报 | 量过 | `clone-0`、`clone-1` 按头的四行 0 误报 |
| C3rL 的发号记录带树 ID（或每头一棵） | 第四节 1 的误报 | 量过 | 同上 `C3rL` 那一行 |
| C2aL 克隆那次留台阶行 | 第四节 2 | 量过 | `clone-2` 0、`clone-3` 29974 |
| C4g-inh / C2i-inh：克隆头另按快照那一版的谱系值与 C2i 条目判 | 第四节 3 的漏 | 量过（只在模型） | `clone-5`、`-6`、`-7`：C4g-inh、C2ig-inh 0；C2i-inh 持久镜像照漏 |
| 树 ID 水位那一格与 inode 号水位拆开写 | 第三节 2 的不对称 | 推的 | 按 `recovery.rs:1341` 那一行与 C342 推 |
| I-8.12 判换实例时 txg 严格增（不归 W3） | 第六节「改坏第一个 txg 全仓无红」 | 推的 | 模型 `first-1` 里 1088952 张有那一形、`first-0` 0 张；没写判词、没跑 |

## 十、没打中的形状

- **K2 维持：四个最强形被「容许」顺带放过**：十个字母长 7（含 cur 根读不出、实例表读不出、根与记录都读不出）七条臂、三条会重发的臂，四个最强形漏 0（第二节 1）；十二个字母长 7 两组（`verify-0`、`-1`）0 误报 0 漏。
- **K2 第二种取法：C143 那一格退回去**：留着 cur 内存项的四组（`y1x-0` … `-3`）重发 0；全部十二字母两组（`y1-0`、`-1`）0；仓副本 B 历史全域 196629 张 0 误报、池级 checker 0 红。
- **K2 第二种取法：实例表读不出、根读不出造出漏或误报**：模型三种处置都不重发；只有「读不出当全部没被抛弃」把字面 C2a 的误报带回来（21114），四个最强形 0。
- **K3 克隆：按头的 C2aL、C3rL 漏或误报**：八组（产品取法四组、第二种取法一组、V1 类三组）0 误报 0 漏，只要克隆那次在水位不同时留台阶行。
- **K3 克隆：第二种取法与克隆的交叉**：`clone-4` 按头四行 0。
- **K3 副本坏一份**：按代码两处读法，与模型里两个抛弃字母（第五节）。
- **K3 新实例第一个 txg 撞上被抛弃发布的出生代**：两条臂九个字母长 7，`identity_collisions=0`；三个形态与现有检查在两条臂上都 0 误报 0 漏（第六节）。
- 取样范围：环 4（第二轮报过环 6、8 的趋势，我没重扫）；长 7；用户步全部穷举，只固定起点（新池新建文件之后）与各组的字母表；克隆只建一个第二头、不销毁头、不打快照之外的别的头操作。

## 十一、这条腿自己的限度

- 模型是条文状态机，不是 `crates/`：写行、暖机、取号屏障、F 的生效、回退候选集「带文件」那一判都简化成一次发布；墓碑只在头 0；克隆是全池回退拒跨克隆之下的最简两头，删头、头的快照、跨头共享单元都不在。
- 每个格只抽了一次样（没有 s2）；「没打中」的格照 `.claude/rules/three-way-inference.md` 不算「没打中」。
- 仓副本只跑了 Y1 第二种取法这一个补丁、B 这一种故障、`c1` 这一段用户步；A 历史只在 `probe` 的四个点名状态上比过；没有在副本上实现 C2aL、C3rL、C4g 的写者（与第二轮一样只有影子），克隆、副本坏一份、第一个 txg 改坏三格都只在模型上。
- 取样与估时（定义第 3b 条）：模型各组 1–40 秒（单线程，`expected` 里没有耗时字段）；仓副本 B 历史全域先跑一片估时（51 秒，3280 个状态），按约 17 分钟（< 40 分钟）分 20 片、每批 3–4 片（估 150–205 秒）跑完，实跑每片 41–81 秒；批不设限时。层 0 状态数：原型那条流的闭式是 65543、加原地覆写的撕裂态 65548（原型自己的输出 `states=65548`；`python3` 算 1 + Σ(2^段 − 1) 对 [16, 2, 1, 2] 得 65543）；这条腿起的全量枚举 5 次 × 65548（草稿里基线 `probe`、补丁 `probe`、补丁 B 二十片合计一次；空目录复跑里基线 `probe`、补丁 `probe`）加空目录里的 kept-shard-7 3280 个，合计 331020 个，单次 65548，都在约 10⁶ / 10⁷ 之内。
- 交回前在空目录 `~/.local/share/singlefs/work/m3-w3-r3-opus/rerun-empty/` 从头跑了 build-model、model-clone-first、model-verify、model-y1、build-proto、probe-base、build-proto-kept、probe-kept、kept-shard-7 与 compare-quick：末行 `compared=29 status=0`；kept-shard-7 的账本与草稿里那一次逐字节相同（去掉耗时）。二十片全量是在草稿目录里用同一份二进制跑的，`expected/kept-merge-of-20.out` 就是那一次 merge 的输出；复跑目录里没把二十片重跑完。
- 机器负载：开工时 `ps` 看到别的会话的 cargo（`cargo test --offline -p singlefs-harness --test opus_r3_leaf_death_images …`、`cargo test --release --offline -p singlefs-harness --test opus_s2_probe --test probe_jk --no-run`），没有性能测量在跑；我的编译与跑一律 `nice -n 19`、线程 5、经内存包装，没等锁；一次 `model-y1` 被内存包装排了 68 秒。空目录复跑里 `probe-base` 与 `build-proto-kept` 那一条前台命令跑了 274 + 35 秒，超过前台 240 秒被工具挪到后台，等它自己跑完（退 0），没停。

## 十二、没做什么

- 没跑任何重型测试（checker 档包的测试二进制、门禁、变异整表都没跑）；原型 crate 是第二轮的，只依赖 checker 档的库、在自己的二进制里调层 0 枚举，测试目标名不带 layer0。
- 没判 K1（正推腿的格）；没替主 agent 采纳任何改法；副本上的数不是入库装置上的数。
- 没重攻前两轮攻过的角度（E175 V1 那两次、op92 / op95、A / B 两种变异本身、辩方对出局候选的复核、崩溃恢复抛弃再建本身、清扫删行、实例切换、A → B → A、「被盖住」对「等于」、从 cur 读）。
- 没在仓副本上造「副本坏一份」的单元、没改坏 `first_txg_of_new_instance`、没实现克隆。
- 草稿目录 `~/.local/share/singlefs/work/m3-w3-r3-opus/` 里的仓副本与编译目录（`proto-w/repo`、`proto-w/target`、`rerun-empty/repo`、`rerun-empty/target`、`rerun-empty/target-model`、`gen/target-model`、`target-model`）交回前删；留着 `out/`（开发时各组模型输出与带 `W3R3TRACE` 的三份）、`gen/out/`（`expected` 的来源）、`proto-w/` 里的二十片账本与原始输出（`l0kept/`、`kept-shard-*.out`、`probe-*.out`、`patch` 前后的 `mount.rs`），主 agent 要直接读原始输出时不必重跑。这些没进 `research/results/`：本轮产物按交付写进模型目录 `expected/`。
