# m3-birth-floor-r2 攻方腿报告（2026-10-02）

<!-- doc-lint:not-numbers V1 V2 V3 J1 J2 J3 J4 J5 CS1 CS2 TF BI BJ -->

批次：records/2026-09-30-里程碑三开工第一轮.md#5。攻击面：正文 `research/prompts/_m3-birth-floor-r2-body.md` 第四节分工表攻方那一行（J1、J2、J3）。判据照正文第五节。

## 复跑与文件

复跑：`bash research/prompts/m3-birth-floor-r2-opus-model/rerun.sh /tmp/claude-1000/m3-batch5-freeze/tree <草稿目录> [段名…]`。它把冻结副本拷进草稿目录、`patch -p2` 打上 `proto.diff`，经内存包装（4G）、线程上限 7 编 release，按段名跑原型包 `bf-proto` 的几份用例，每段一份日志、末尾打汇总行。段名不给时跑除两段层 0 全量之外的全部（本机串行约 35 分钟，按我各段单跑的耗时加的，推的）；`crash-full-raiseTo8`、`crash-full-rb1` 只在点名时跑（见「要主 agent 跑的命令」）。不跑 checker 档包的任何测试：`bf-proto` 只把 `singlefs-checker-tier` 当库链进来、调它的 `judge_layer0_state_range`；测试目标名里没有 layer0。`proto.diff` 打在一份新拷的冻结副本上，与我的副本逐文件相同（`diff -rq --exclude=target --exclude=Cargo.lock` 无输出）；`rerun.sh` 整条没在新副本上跑过，里面每段的命令与环境变量是我在副本上跑出 `logs/` 的那几条。

```text
5f3b2729c4805a047f84bfe75f8c6774ae8710573ef417d5bdc37c2b2d6ce1d5  ./logs/bad-r1six.log
1c8f12cb5f197a3caf6d8c9665794b698b2bb7eee711823df518d28d55b61f78  ./logs/bad-r2.log
23cd5749b46e67a9b86d19cc04c9e29c7c86e2fb091b4573b2ec2f89ab2b55d6  ./logs/crash-count.log
c129417882c934763b3cb2980c977f55d69d7b82d74733ffa51f80589115d9a5  ./logs/crash-head.log
600c5e84df38da97f8422a0c868c026ea8ffe42d7e37368f51712993b9f40f66  ./logs/crash-tail.log
9c8cea6a227f9cb5dae9c4b9527d5fe2d734f4c37364418e89cc37482b6e7bb2  ./logs/jwrap1-a1.log
ce8a2f250ec09052d8e5e8599a170e55cb8c03fd83708b1b188df3be7529633f  ./logs/jwrap1-a2.log
bd151bf5cc88ccd280f3f804f6b3f9d8509baa8d15227174b00d9153c5909ecf  ./logs/jwrap6-a2.log
dc1e4060e26afff1d2572a3657e1c4d2910f4ab5f17fbde60efa9e769a8a4c60  ./logs/stale-ow-a2.log
fc67d6c9743234c30631605e148803bd8c66450e480adc5a0a0402e6c53a2cbc  ./logs/sweep-A0.log
f6e5a4b9674797df13a8ec3a5c8cf328c326f6eb100533d5b13196ee3e5a0830  ./logs/sweep-A1.log
580fde2ba3ad1bef7b19adcbe15247c75d2c36c4a096c0688c456727ddb53acd  ./logs/sweep-B0.log
64b0f796dd143751e7898a7cfb0cbcfd75e519d575e0a380a41d18baf5d8a692  ./logs/sweep-B1.log
64e0d418a70e113432f0da622b4f842a81c04022a2bd0d7cdebfbdb844aedc79  ./logs/sweep-C0.log
226c4f1beafd8854db32e36c97ba8a27f0ebd44f53a621da8fd8637b84136df1  ./logs/sweep-C1.log
b6d888c4b4443fcfe79a7604afcb4409dccb98752c024e7ce196245c61a19959  ./logs/sweep-C2.log
12f88889273b336d9fd14a504811b2628b9139ba5d10723c21bfec9ad32967a9  ./logs/sweep-raise-A.log
6caf059f683eea7599b989b14c1dfcee3cebee1784afcaf72b35808c47ebea21  ./proto.diff
e2cec2138a21fd07f2a4d19fe758cbcc88403dc055f06510cf033a1693012b67  ./rerun.sh
```

全部数都是冻结副本拷贝上的数，不算入库装置上的数。

装置：`proto.diff` 在第一轮攻方原型（`walk/proto_b.rs`、`crates/bf-proto`）上加 `tree_floor_judgements`，把两样被攻的东西写成谓词，都判在两种候选集上：CS2 = `check_pool_image` 的候选集（根环候选根并上 `versions_applied_only_by_records` 补进来的「由记录施加出来、根槽从没落盘的那一版」，引用集合从它所施加的那条根的记录换上新根段之后走，同第一轮原型），按 txg 升序、同 txg 并成一条（引用集合取并、数不全取任一、用户可见树取并）；CS1 只取根环候选根，作对照。相邻两条 (R⁻, R)，R 引用而 R⁻ 不引用的落点 b 判 (a) birth(b) > R⁻.txg 或 (b)（R 的 extent / inode / `deleted_inodes` 树里根指针非全零、出生 txg 严格小于 R⁻.txg、子树每个单元头诞生代号不大于它，且 b 在那棵子树里）；R 或 R⁻ 数不全报不适用。

| 谓词名 | 是什么 |
|---|---|
| `tf-lenient` / `tf-strict` | 按树继承下界，只判 b 在候选集里第一次出现的那一对（I-3.5 ② 下界那一半）；lenient = 读不出的单元头算压得住（第一轮 V3 的写法），strict = 算压不住 |
| `beta-lenient` / `beta-strict` | β 出现规则：每一对都判（第一次出现与再次出现） |
| `beta-re-*` | β 里「再次出现」那一部分（b 被更早某条候选根引用过） |
| `tfgap-lenient` / `betagap-lenient` | 我提的改法（零轮）：R.txg ≠ R⁻.txg + 1 的那一对不判，见「改法」一节 |

第一轮的原措辞、V1、V3 照旧在同一份原型里（`birth_floor_judgements`），坏镜像那一侧一起报。驱动 `crates/bf-proto`：内存稀疏盘（`SparseBlockDevice`）上 mkfs、取号、暖机、第一个文件（txg 3），起点只建一次、每段历史从它 `fork`；新加 `after_first_file_with_journal_ring`（journal 环长可另给）。

## 各格判定一览

| 格 | 被攻的东西 | 判定 | 依据（副本上量的） |
|---|---|---|---|
| J1 根环旧槽 + journal 环绕回 | 按树继承下界（TF） | **打中：合法镜像上红** | 短环（1 MiB / 6 MiB，mkfs 收的环长）上：回退、顺序写、回退崩在根槽之前、重新挂载；CS2 里出现一条 txg 8 的旧根，与下一条候选根之间 txg 9（回退根）、10 的根槽被根环盖掉、记录被 journal 环盖掉。终态镜像 472 张 TF 红；这 472 张池级 checker 都红 I-3.1。之后一步抬 F 到 8 的层 0 崩溃状态里，前 700 个有 696 个 TF 红而层 0 整套判定全绿（I-3.1 报不适用） |
| J1 根槽读坏（介质坏一个根槽） | TF | 打中，但同一张上池级 checker 先红 | 抬 F 越过回退目标、把回退根的根槽抹 0：24 张 TF 红；每一张池级 checker 都红 I-3.1（多数连 I-3.9） |
| J1 被抛弃的根与回退同在 | TF、β | 没造出 | 今天的崩溃恢复造不出被抛弃的根（可写挂载拒），装置里没有别的路 |
| J1 放开用户动作 | TF、β | 没打中（不带故障） | 3 种前缀 × 回退目标 4 种 × 挂着回退 / 回退崩在根槽之前 2 种 × 回退之后长度 0–2 的全部动作序列，2 184 段；另加强抬 F 的 528 段：终态镜像 TF、β 0 红 |
| J1「压得住」读不出的头 | TF 两种读法 | strict 在一份盘坏的合法镜像上红，lenient 不红；那几张池级 checker 都红 I-2.1 | 继承来的子树里一个单元的一份抹 0 |
| J2 β 再次出现（洞两侧） | β | **打中：洞两侧误判成再次出现**，同一张上池级 checker 都红 I-3.1 | 回退目标还在候选集、回退根根槽读坏：β-re 红 818 张（逐条抹 420、相邻两条一起抹 398），TF 不红；不带故障 0 |
| J2 β 在 J1 那一形上 | β | 红（那一形里是第一次出现，与 TF 同红） | 同 J1 第一行 |
| J3 第一轮六份坏镜像 | 在合入后的池级 checker 上 | ③（整条链改小）与 ④ 里改 inode 叶那一份现在池级 checker 红 I-9.26；其余 ④ 仍只红下界 | `logs/bad-r1six.log` |
| J3 整条链改小到 R⁻.txg | TF 相对 V3 的收严 | TF 红、V3 不红、池级 checker 0 红（连改动计数一起改） | 「严格小于」那一处收严的只红它的坏镜像 |
| J3 整条链改小到 R⁻.txg 之下 | TF | 漏红（判别子观测不到，同第一轮）；池级 checker 0 红 | 连改动计数一起改 |
| J3 β 再次出现的只红它 | β | **造出来了**：池级 checker 0 红、TF 0 红、β 红 | 最新根的 inode 树指回上一版已丢的旧叶，分配记录与中央映射一起改 |

「没打中」都是这一条腿一次抽样的结果；取样范围在「没打中的形状」一节。

## J1　按树继承下界在合法镜像上

### J1-a　根环旧槽 + journal 环绕回（打中）

**历史**（`tests/bf_r2_sweep.rs` 的 `stale_root_slot_after_an_applied_only_version`，取 a=2、r=3、b=22、c=1 那一段；txg 是这段历史上的实数）：mkfs 时 journal 环长取 1 MiB（或 6 MiB，`crates/singlefs-harness/src/history.rs` 小盘那一档用的就是它：「const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 6 << 20;」（`crates/singlefs-harness/src/history.rs:142`）），之后：建 300 个 inode（txg 4，inode 树两片叶）、覆盖写（5）、建 1 个 inode（6，第二片叶换成新的 L2₆）、覆盖写两次（7、8）、回退到从新到旧第 3 条（回退根 txg 9，目标 txg 5，带回旧叶 L2₄）、顺序写 16 单元 22 次（10–31，每次 16 条记录，把环转过去）、回退那次发布崩在根槽之前（txg 32 只有记录）、重新可写挂载（实例 2 的根 33、34）。之后每多一次覆盖写另判一张（d = 0–26）。

一步一步许可它的那几句：
1. 根槽位置由 txg 算：「r == txg mod R 且 s == (txg div R) mod S」（`.claude/kb/invariants.md:71`），R、S 是「pub const ROOT_RING_REGIONS: u64 = 3;」（`crates/singlefs-format/src/lib.rs:245`）与「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」（`crates/singlefs-format/src/lib.rs:263`）。txg 32 与 txg 8 落在同一个槽；32 的根槽从没写过，那个槽里一直是 txg 8 的根。33、34 盖掉了 9、10 的槽。
2. txg 8 的根自证过、没被实例表判抛弃、不低于 F（F 生效值 0），进候选集：「let walked = *index == newest_index || (!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:6766`）。
3. 9、10 的根槽读不出，CS2 本该从记录补回来：「/// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。」（`crates/singlefs-checker/src/walk.rs:5798`）——默认 768 MiB 的环上记录还在，9、10 被补回来，CS2 上 0 红（`logs/stale-ow-a2.log`：CS1 256 张红、CS2 0 张）。短环上 22 次顺序写之后 9（6 MiB）或 9、10（1 MiB）的记录已被环盖掉，补不回来，候选集在 8 与下一条之间出现一个洞。
4. 洞右边那条 R（1 MiB 时是覆盖写的根 txg 11，6 MiB 时是由记录施加出来的 txg 10）引用 L2₄（诞生代号 4），R⁻ = 8 引用的是 L2₆；R 的 inode 树根出生 10 / 11，不严格小于 8，(b) 不成立；(a) 4 > 8 不成立 ⇒ 红。L2₄ 在候选集里第一次出现就在 R 上（目标 txg 5 早已被根环盖掉），TF 与 β 同红。

原样：

```text
R2 stale:CS2:tf-lenient evaluated=188416 violations=88 images=44 first=stale jring=Some(6) sw=Some(72) a=2 r=3 b=22 c=2 d=0 :: TF-lenient：盘 0 槽 1488 类 3 诞生代号 4 ≤ R⁻ txg 8 [实例 1]；R txg 10 [实例 1·由记录施加]；第一次出现；在继承树（不论压不压得住）下 false；R 的树：种类 1 根槽 1556 出生 10；种类 2 根槽 1560 出生 10
R2 stale:pool-checker evaluated=54 violations=44 images=44 first=stale jring=Some(6) sw=Some(72) a=2 r=3 b=22 c=2 d=0 ["I-3.1"]
```

（`logs/jwrap6-a2.log` 第 388、391 行。）三份日志合计：

| 日志 | 段数 × 每段张数 | TF 红的张数 | 其中池级 checker 红 | 池级 checker 红的张数（全部） |
|---|---|---|---|---|
| `jwrap1-a1.log`（1 MiB，a=1，r 2–5，b 16–24，c 1–2） | 72 段；18 段回退目标不在候选集、回退被拒、不判；54 × 27 张 | 172 | 172（I-3.1） | 1 170 |
| `jwrap1-a2.log`（1 MiB，a=2） | 72 × 27 | 256 | 256（I-3.1） | 1 560 |
| `jwrap6-a2.log`（6 MiB，a=2，r=3，b=22） | 2 × 27 | 44 | 44（I-3.1） | 44 |

TF 红只出现在 b 落在 [22 − a, 22]、回退目标是建 1 个 inode 之前的那一版（a=1 时 r 2–3，a=2 时 r 3–4）那几段、d ≤ 21 那几张（旧根 txg 8 的槽在 d = 22 被盖掉，洞跟着没了），与按根槽位置推的一致。

**池级 checker 在这些终态镜像上自己也红 I-3.1**：洞里那几版（回退根 9、顺序写 10）自己新写的单元已被之后的发布释放、进 defer，又因为环里最旧的有效根是旧根 8 回收不了（④「它换下的单元还没到回收的时候」那一句的同一条闸，`crates/singlefs-checker/src/walk.rs:5800`），仍算已分配；而候选集里没有一条根引用它们，遍历少算——「隔离的记录：已分配而没有任何根引用的记录，按单元豁免」（`.claude/kb/invariants.md:143`）只豁免读不出的单元，这里每一份都读得出。这是 I-3.1 在同一个洞上的误红（推的：靠「默认环上记录还在、CS2 补回 9、10 时 I-3.1 0 红」对出来的，没逐槽列出那几个单元）。

**崩溃状态**（`tests/bf_r2_crash.rs`，起点 = 1 MiB 那一段 d=0 的终态，之后一步「抬 F 到 8」，层 0 全量 196 665 个状态，闭式 196 630）：

```text
R2C name=jring=1MiB sw=16 d=0 [raiseTo8] :: tf-red-state:layer0-green evaluated=700 violations=696 states=696 first=jring=1MiB sw=16 d=0 [raiseTo8] state=500 versions=txg 8 [实例 1] | txg 11 [实例 1] | txg 12 [实例 1] | txg 13 [实例 1] | txg 14 [实例 1] | txg 15 [实例 1] | txg 16 [实例 1] | txg 17 [实例 1] | txg 18 [实例 1] | txg 19 [实例 1] | txg 20 [实例 1] | txg 21 [实例 1] | txg 22 [实例 1] | txg 23 [实例 1] | txg 24 [实例 1] | txg 25 [实例 1] | txg 26 [实例 1] | txg 27 [实例 1] | txg 28 [实例 1] | txg 29 [实例 1] | txg 30 [实例 1] | txg 31 [实例 1] | txg 32 [实例 1·由记录施加] | txg 33 [实例 2] | txg 34 [实例 2]
R2C name=jring=1MiB sw=16 d=0 [raiseTo8] :: state:I-3.1=not-applicable evaluated=696 violations=696 states=696 first=jring=1MiB sw=16 d=0 [raiseTo8] state=500
```

（`logs/crash-head.log` 第 33、29 行，状态 0–699；`tf-red-state:layer0-green` 一行的 violations 计的是「TF 红而层 0 整套判定——两种 journal 政策的恢复与两份 oracle、池级 checker、记录核对器——一项不红」的状态数。）这 696 个状态里新 F 已写进系统配置、带新 F 的根还没落，I-3.1 照条款报不适用：「最新根带的 F 低于 F_生效（两者不等）时这一格不判」（`.claude/kb/invariants.md:143`）；它们之上别的不变量都绿，只有 TF 红。末尾 700 个状态（`logs/crash-tail.log`，195 965–196 664，根已落）TF 全红、池级 checker 全红 I-3.1。全量 196 665 个没跑完，见「要主 agent 跑的命令」。

#### J1-a 打中之后的四句

1. **分不分辨臂：不分。** 同一张上 TF 红、β 红；α（删掉 ①、留 ② 的下界）照样红——这里 L2₄ 在候选集里是第一次出现，α 留下的那一半正是 TF。第一轮的原措辞、V1 在这一对上也红（推的：R 的 inode 树根出生 10 / 11 > 8，V1 的豁免条件不成立，照原措辞判；我没在这几段上跑 `birth_floor_judgements`）。同一张终态镜像上池级 checker 也红 I-3.1。病根是各种下界写法共用的前提：**候选集在 txg 序上没有洞，或洞能由 journal 记录补齐**；根环旧槽 + 记录被环盖掉时这个前提不成立。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`「打中不分辨臂」那一行，这个共用前提该另立一笔账。
2. **系统当时看得到判别它的东西吗：一半看得到。** 看得到的：R⁻ 与 R 的 txg 之间有缺口（8 → 10 / 11），旧根 8 坐在 txg 32 该在的槽里。看不到的：缺口里那一版（txg 9）是一次回退、回退到哪一版——那条根与它的记录都被环盖掉了，盘上没有别的地方记着（回退今天不写谱系记录，第一轮判决第二节第 5 行）。所以「这一对之间有洞」可判，「洞里是不是回退」不可判。
3. **满足的是判据字面的哪一句：** 正文第五节「- 按树继承下界在任一合法镜像上红 ⇒ 出局，I-3.5 ② 不立、状态列写明，等谱系记录。」（`research/prompts/_m3-birth-floor-r2-body.md:52`）。镜像由合法调用走出来（mkfs 收的环长、写、回退、回退那次发布崩在根槽之前、重新挂载）；其中 696 个崩溃状态层 0 整套判定全绿，终态那 472 张池级 checker 红 I-3.1。若主 agent 判「池级 checker 红的镜像不算合法镜像」，打中只剩那 696 个状态（它们仍然成立）。同一组镜像也满足「- β 在任一合法镜像上红 ⇒ β 出局，I-3.5 ① 改取 α（删掉 ①）。」（`research/prompts/_m3-birth-floor-r2-body.md:53`），但见第 4 句。
4. **跑前条款给的改法在打中的格上还中不中：**

| 改法 | 来源 | 在 J1-a 那几格上 | 量过 / 推的 |
|---|---|---|---|
| β 出局 ⇒ 改取 α | 正文第五节第二条 | **还中**：α 留着 ② 的下界，这里是第一次出现，红法与 TF 相同 | 量过：`tf-lenient` 就是 α 留下的那一半，同红（`logs/jwrap6-a2.log` 第 388 行） |
| TF 出局 ⇒ ② 不立、等谱系记录；落地后 (b) 收成「R 里有一条 T_rb = R.txg 的谱系记录、树根指针的出生 txg 不大于它的 T_old」 | 正文第五节第一条、J5 | **还中**：R 是回退之后的一次覆盖写（txg 10 / 11），不是回退根，R 里没有 T_rb = R.txg 的谱系记录 | 推的：谱系记录今天没实现 |
| 同上，但谱系记录住在随现行那一版走的谱系树里（「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」，`.claude/kb/decisions/23-journal的角色与格式.md:377`），(b) 读成「R 的谱系树里有一条 T_rb ∈ (R⁻.txg, R.txg] 的记录、b 的诞生代号不大于它的 T_old」 | 我提的读法，零轮 | 不中（推的：L2₄ 出生 4 ≤ T_old 5） | 推的：没实现 |
| 相邻两条 txg 不相邻（R.txg ≠ R⁻.txg + 1）时这一对不判（gap） | 我提的，零轮 | 不中 | 量过（只在我的模型上）：`tfgap-lenient`、`betagap-lenient` 在三份 jwrap 日志与 `crash-head.log`、`crash-tail.log` 上 0 红 |

### J1-b　根槽读坏（介质坏掉一个根槽）

在 J1-d 那 2 712 段历史的每张终态镜像上，把根环里每一条根的槽逐条抹 0（zero1），再把相邻两条一起抹 0（zero2），各判一张。

| 故障 | 张数 | TF 红 | β 再次出现红 | 池级 checker 红 |
|---|---|---|---|---|
| zero1 | 34 628 | 8（都在加强抬 F 那 528 段里） | 420 | 13 890 |
| zero2 | 31 916 | 16（同上） | 398 | 14 710 |

TF 与 β 红的每一张，池级 checker 都红（I-3.1，多数连 I-3.9；`R2_RED` 行里 `pool_red=[]` 的一行都没有）。今天的池级 checker 不把「一个根槽读不出」当合法镜像：抹掉一条根之后它独有的单元没人引用、仍算已分配，I-3.1 红（同 J1-a 那一句豁免条款，只豁免读不出的单元）。TF 红要回退目标已被抬 F 越过（加强抬 F 那一批里才有）；回退目标还在候选集里时红的是 β 的「再次出现」，见 J2。

### J1-c　被抛弃的根与回退同在（没造出）

今天的崩溃恢复造不出被抛弃的根：「这一步不再造出被抛弃的根」（`crates/singlefs-harness/src/history.rs:395`，那一行后半句带嵌套引号，没整行抄；可写挂载在取号之前拒）。装置里没有第二条路，我没造；推的：被抛弃按实例与 txg 一刀切，回退根若被抛弃，它之后同实例的根一起被抛弃，新实例接在被选的那条根上，不会出现「洞右边继承了洞里那条回退根」的一对。

### J1-d　放开用户动作（不带故障：没打中）

`tests/bf_r2_sweep.rs` 的 `tree_floor_and_beta_over_final_images`：前缀三种（A：建 300 个 inode、覆盖写、建 1 个 inode、覆盖写；B：顺序写 3 单元、建 240 个 inode、覆盖写、建 5 个 inode；C：A 再加建 2 个 inode、覆盖写），回退目标从新到旧第 1–4 条，挂着回退 / 回退那次发布崩在根槽之前再重新挂载两种，回退之后的用户动作取字母表 {覆盖写、建 1 个、建 240 个（跨叶）、顺序写 2 单元、抬 F 1 步、抬 F 3 步、重新挂载、回退到第 1 条、回退到第 3 条} 上长度 0–2 的全部序列（91 种），共 2 184 段；另一批（`sweep-raise-A.log`）字母表再加抬 F 5 / 6 / 7 步、只留至少带一次的后缀，528 段（426 段有一步被拒，日志只记了第一条：抬 F 越过上限 `RollbackFloorAboveCeiling`；照被拒之后盘上的样子判）。每段判终态镜像。

结果：2 712 张终态镜像，TF 判 561 292 次、β 判 591 682 次，都是 0 红；池级 checker 0 张红（八份 `sweep-*.log` 的 `none:` 行相加）。这批只判终态，没有枚举崩溃状态（这几段的后缀都带覆盖写或建 inode，单次覆盖写一次发布就超过约 10⁶ 个层 0 状态，第一轮攻方 `logs/count-probe.log` 量过）。

### J1-e　「压得住」读不出的头怎么算

`tests/bf_r2_bad.rs` 的 `one_copy_unreadable_under_an_inherited_tree`：覆盖写 4 次、回退到第 4 条（txg 4）、抬 F 到 5，回退根整棵继承 txg 4 的用户可见树；把继承来的子树里一个单元的盘 1 那一份抹 0（另一份完好）。抹到 txg 4 那四个单元（数据单元、extent 树根、inode 叶、inode 树根）时 TF-strict 红 3 次、TF-lenient 0 次；这四张池级 checker 都红 I-2.1、I-4.8、I-7.4（`logs/bad-r2.log`）。两种读法只在「池级 checker 已经红的单份盘坏」上分开；lenient 那一边漏的形态要两份都读不出（池级 checker 红 I-2.1），我没造出它只靠 lenient 漏掉的坏镜像（推的）。

## J2　β 出现规则在合法镜像上

**不带故障（没打中）：** J1-d 那 2 712 张上 β 0 红、「再次出现」那一部分判 30 390 次 0 红（`none:CS2:beta-re-lenient`）；J1-a 默认环那一批（`stale-ow-a2.log`）CS2 上 0 红。第一轮正推说「非回退根再次引用一个已不被 R⁻ 引用的单元在合法历史里不会出现」，这一批没有反例。

**洞两侧（打中，同一张上池级 checker 也红）：** 回退目标还在候选集里、回退根的根槽读坏时，回退之后那次改动的根 R 与回退之前那条 R⁻ 成了相邻一对；R 照抄回来的旧叶被回退目标引用过、R⁻ 不引用，β 判它「再次出现」并按 (a)/(b) 判红（R 的 inode 树根是新的，不算继承）。原样（`logs/sweep-A0.log` 第 19 行）：

```text
R2_RED A/rb1/[ni1,ow] zero(txg 8 实例 1) fault=zero1 CS2:beta-re-lenient violations=2 pool_red=["I-3.1", "I-3.9"] first=β-re-lenient：盘 0 槽 50268 类 3 诞生代号 5 ≤ R⁻ txg 7 [实例 1]；R txg 9 [实例 1]；再次出现；在继承树（不论压不压得住）下 false；R 的树：种类 1 根槽 50267 出生 5；种类 2 根槽 50314 出生 9
```

818 张 β 再次出现红，TF 都不红（第一次出现在回退目标上，TF 不判那一对），池级 checker 都红 I-3.1。四句：
1. 分不分辨臂：**分 β 与 α**——α 删掉 ①、不判再次出现，这里不红；TF 也不红。
2. 看得到吗：看得到 R⁻、R 之间的 txg 缺口（回退根那一格），看不到缺口里是回退。
3. 字面：「- β 在任一合法镜像上红 ⇒ β 出局，I-3.5 ① 改取 α（删掉 ①）。」（`research/prompts/_m3-birth-floor-r2-body.md:53`）——前提是根槽读坏的镜像算合法镜像；今天的池级 checker 在这些镜像上都红 I-3.1，按「池级 checker 红的不算合法」读，这一格不满足字面。
4. 改法：改取 α 在这几格上不中（量过：`tf-lenient` 0 红）；gap 那一刀也不中（量过：`betagap-lenient` 在八份 sweep 日志里 0 红）。

**J1-a 那一形上 β 也红**（与 TF 同红，那里是第一次出现），见 J1-a。

## J3　坏镜像

### 第一轮攻方六份坏镜像在合入后的池级 checker 上（`logs/bad-r1six.log`）

造法照第一轮（`tests/bf_bad_images.rs`，改了单元的诞生代号，指针、映射 key、分配记录分配代、journal 点名项一起改，校验和沿引用链补到根槽与记录）。冻结副本里的单元落点与第一轮相同（`units_born_at_6` 那一行逐项相同），`BI2_SLOTS` 照用。表里是违例次数，「池级 checker 红」一列是合入后的 checker：

| 份 | 池级 checker 红 | 原措辞 | V1 | V3 | TF-lenient | TF-strict | TF-gap | β |
|---|---|---|---|---|---|---|---|---|
| BI-1 slot=50289 class=2 key_width=10 | - | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1 slot=50290–50293（分配记录树，四份） | I-1.1 | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1 slot=50294 class=2 key_width=22 | - | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1 slot=50295 class=2 key_width=27 | - | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1 slot=50296 class=2 key_width=8 | - | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1b slot=50301 class=2 key_width=10 | - | 10 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1b slot=50302–50305（四份） | I-1.1 | 10 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1b slot=50306 class=2 key_width=22 | - | 10 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1b slot=50307 class=2 key_width=27 | - | 10 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-1b slot=50308 class=2 key_width=8 | - | 10 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50186 class=1 key_width=1 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50277 class=2 key_width=24 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50278 class=3 key_width=2 | **I-9.26** | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50280 class=2 key_width=8 | - | 2 | 0 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50281 class=2 key_width=10 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50282–50285（四份） | I-1.1 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50286 class=2 key_width=22 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50287 class=2 key_width=27 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-3 slot=50288 class=2 key_width=8 | - | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| BI-2 slots=[50186, 50277, 50278, 50280]（整条链 6 → 5） | **I-9.26** | 8 | 0 | 0 | 8 | 8 | 8 | 8 |

（「四份」那几行是逐份一行的合并，四份逐列相同；I-1.1 那几份第一轮就没造干净，不算。）对第一轮第 ④ 行（= BI-3）：改 inode 叶（50278）那一份现在池级 checker 红 I-9.26——「object_birth <= change_count && change_count <= container_birth_txg,」（`crates/singlefs-checker/src/walk.rs:1814`）：inode 记录里的改动计数还是 6、容器诞生代号被改成 5。第 ④ 行其余五份（50186、50281、50286、50287、50288）池级 checker 仍 0 红，仍是只红下界的坏镜像，TF 照红；第一轮的 ②（50280）、②b（50277）同样池级 checker 0 红、TF 红。第一轮第 ③ 行（= BI-2，整条链 6 → 5）现在 I-9.26 也红，原因同上；TF 红它是因为「严格小于」：改到 5 = R⁻.txg，树根出生不严格小于 R⁻.txg，(b) 不成立。

### 新造的四份（`tests/bf_r2_bad.rs`，`logs/bad-r2.log`）

| 份 | 改了什么 | 池级 checker | TF | V3 | β |
|---|---|---|---|---|---|
| J3-a | 不回退的历史（最新根 txg 6、R⁻ txg 5），整条用户可见链（数据单元、extent 树根、inode 叶、inode 树根）6 → 4 | I-9.26 | 0 | 0 | 0 |
| J3-b | 同 J3-a，inode 叶里改动计数 6 的那条记录一起改成 4 | 0 红 | 0 | 0 | 0 |
| J3-c | 同 J3-b，但改到 5（= R⁻.txg） | 0 红 | **8** | 0 | 8 |
| J3-d | 建 300 个 inode、覆盖写、建 1 个 inode、覆盖写（txg 4–7）；txg 7 的 inode 树根里指第二片叶的指针换回 txg 4 那一片，txg 7 的分配记录树里新叶记成已释放（释放代 7）、旧叶记成未释放（分配代 4），txg 7 的中央映射根里新叶那一条换成旧叶那一条 | 0 红 | 0 | 0 | **2（再次出现）** |

原样：

```text
BJ J3-b chain→4+count pool_checker_violated=[]
BJ J3-b chain→4+count CS2:tf-lenient evaluated=96 violations=0 first=-
BJ J3-c chain→5+count pool_checker_violated=[]
BJ J3-c chain→5+count CS2:tf-lenient evaluated=96 violations=8 first=TF-lenient：盘 0 槽 50186 类 1 诞生代号 5 ≤ R⁻ txg 5 [实例 1]；R txg 6 [实例 1]；第一次出现；在继承树（不论压不压得住）下 false；R 的树：种类 1 根槽 50277 出生 5；种类 2 根槽 50280 出生 5
BJ J3-d/step3-mapping pool_checker_violated=[]
BJ J3-d/step3-mapping CS2:beta-re-lenient evaluated=2 violations=2 first=β-re-lenient：盘 0 槽 50256 类 3 诞生代号 4 ≤ R⁻ txg 6 [实例 1]；R txg 7 [实例 1]；再次出现；在继承树（不论压不压得住）下 false；R 的树：种类 1 根槽 50291 出生 7；种类 2 根槽 50294 出生 7
BJ J3-d/step3-mapping CS2:tf-lenient evaluated=114 violations=0 first=-
```

（`logs/bad-r2.log` 第 46、50、61、65、223、225、229 行。）

- **TF 相对 V3 的收严有只红它的坏镜像**：J3-c（「严格小于」那一处）。按树分摊那一处的只红它，第一轮已有（BI-1 / BI-1b：V1 不红、TF 红、池级 checker 0 红，表里那几行合入后照旧 0 红）。正文第五节第三条「按树继承下界写不出一份只红它的坏镜像 ⇒ 下界不立」**没打中**。
- **新的漏红形态**：J3-b——整条用户可见链改到 R⁻.txg 之下、inode 改动计数一起压住，TF、V3、β、池级 checker 都不红。它就是第一轮判决第二节第 5 行那一形（判别子观测不到），这一轮补的是「I-9.26 落地之后只要再压改动计数就照样漏」。「压得住」只读到能读的头那一处，我没造出只靠它漏掉、池级 checker 又不红的坏镜像（J1-e）。
- **β 再次出现那一半有镜像层的只红它**：J3-d，三步造法（换指针、改两条分配记录、换一条中央映射条目），池级 checker 0 红、TF 0 红、β 红。第一轮正推说「没有便宜的造法」，这一份是反例；它的语义是 txg 7 静默丢了 txg 6 建的那个 inode（inode 号水位照旧）。每一步之后池级 checker 红什么也在日志里：只换指针红 I-3.9、I-3.11，再改分配记录剩 I-3.11，再换映射条目 0 红。

## 改法：几个改法各修哪一格（我提的只在我的模型上量过、被攻过零轮）

| 格 | 原样 TF / β | 改取 α | gap：txg 不相邻的一对不判（我提的） | 谱系记录 T_rb = R.txg（J5，跑前给的） | 「压得住」取 strict |
|---|---|---|---|---|---|
| J1-a 根环旧槽 + 环绕回（合法，要不红） | 红 / 红 | 仍红（量过） | 不红（量过：三份 jwrap 日志与 `crash-head.log`、`crash-tail.log` 上 `tfgap-lenient`、`betagap-lenient` 0 红） | 仍红（推的） | 仍红（量过：`tf-strict` 同 `tf-lenient`） |
| J1-b 根槽读坏、回退目标已出候选集（池级 checker 先红） | 红 / 红 | 仍红（量过） | 不红（量过：`sweep-raise-A.log`） | 推的：不中，回退根读不出时 R 不是回退根 | 仍红（量过） |
| J2 根槽读坏、回退目标还在（池级 checker 先红） | 不红 / 红 | 不红（量过） | 不红（量过：八份 sweep 日志 `betagap-lenient` 0 红） | 推的：同上 | — |
| J1-e 单份盘坏在继承子树里（池级 checker 先红） | 不红（lenient） | — | 不红（量过） | — | 红（量过） |
| J3-c 链改到 R⁻.txg（坏，要红） | 红 / 红 | 红（量过） | 红（量过：`bad-r2.log` `tfgap-lenient` 8） | 推的：红 | 红（量过） |
| J3-d 再次出现（坏，要红） | 不红 / 红 | **不红**（量过：`tf-lenient` 0） | 红（量过：`betagap-lenient` 2） | 推的：红 | — |
| BI-1 / BI-1b / ④（坏，要红） | 红 / 红 | 红（量过：TF-lenient 列） | 红（量过：`bad-r1six.log` TF-gap 列） | 推的：红 | 红（量过：TF-strict 列） |
| J3-b 链改到 R⁻.txg 之下（坏） | 不红 / 不红 | 不红（量过） | 不红（量过） | 推的：要谱系记录才可能红，今天没有 | 不红（量过） |

gap 那一刀的代价没量：一个被改小的单元若恰好第一次出现在一个缺口右边，gap 不判它（推的，没造这种坏镜像）；缺口今天只在根槽读坏、被抛弃的根、根环旧槽这几种地方出现。它补的是「候选集无洞」这个共用前提，同一个洞上 I-3.1 也误红（J1-a），I-3.1 那一边要另修，gap 管不到。

## 没打中的形状

| 形状 | 取样 | 打的是哪几格 |
|---|---|---|
| 不带故障的终态镜像，回退之后的用户动作放开 | 2 712 段（J1-d），各判终态一张 | TF、β 两种读法，CS1 / CS2 |
| 默认 768 MiB 环上的根环旧槽 | 120 段 × 27 张（`stale-ow-a2.log`） | CS2 上 TF、β 0 红（9、10 由记录补回）；CS1 256 张红（CS1 不是措辞取的候选集） |
| 被抛弃的根与回退同在 | 没造（J1-c） | — |
| 抬 F 越过回退目标之后的层 0 崩溃状态（不带根槽读坏） | 本轮没跑；第一轮攻方四段 458 857 个状态 V3 0 红，TF 在不带洞的候选集上与 V3 只差「严格小于」与树的取法（推的） | — |
| 「压得住」只靠读不出的头漏红、池级 checker 又不红 | 没造出（J1-e） | J3 |
| 回退之后长度 3 以上的用户动作、多次回退穿插 F 抬升 | 没扫 | — |

每批的段数与估时：终态镜像那一批先跑 14 段（2.6 秒，七线程）估 A 前缀 728 段约 135 秒，实际 233 秒，之后按约 360 段一批分片；jwrap 那一批先跑 1 段（27 张，11.7 秒）定每批 72 段；层 0 那一段先跑 700 个状态（1 MiB 那一段 35.6 秒）估全量 196 665 个约 2.8 小时，超过 40 分钟，没跑全量，只跑了头尾各 700 个（这一份是缩样的探针，不算正式跑；全量交主 agent）。层 0 状态账：这条腿合计 4 200 个崩溃状态（四次 700 个的估时探针，加 `logs/` 里头尾各 700；各日志 `R2C_DONE` 行的 states 相加）；终态镜像 152 372 张（各算一个状态；探索与正式两遍，各日志 `R2_DONE` 行的 images 相加，探索那一遍的日志在草稿目录、没入库），合计在约 10⁷ 以内。

## 要主 agent 跑的命令

1. 全量复跑（不含层 0 全量）：工作目录仓根，`bash research/prompts/m3-birth-floor-r2-opus-model/rerun.sh /tmp/claude-1000/m3-batch5-freeze/tree <新的草稿目录>`；脚本内部经 `research/scripts/run-with-memory-cap.sh` 4G、`research/scripts/capped.sh` 7 线程；串行约 35 分钟（推的）。看每段末尾打出的汇总行，与本报告各表与 `logs/` 逐行比。
2. J1-a 那一段的层 0 全量：同上加段名 `crash-full-raiseTo8`（196 665 个状态，按 700 个 35.6 秒估约 2.8 小时）、`crash-full-rb1`（65 553 个，约 0.9 小时）。看 `R2C name=… :: tf-red-state:layer0-green` 一行的 violations（TF 红而层 0 整套判定全绿的状态数）、`layer0-red-states` 一行、`state:I-3.1=…` 几行。

## 这条腿自己的限度

- J1-a 的终态镜像上池级 checker 都红 I-3.1；只红 TF 的是抬 F 那一步的崩溃状态，而那一段层 0 全量 196 665 个只跑了头尾各 700 个（头 700 个里 696 个只红 TF，尾 700 个 TF 与 I-3.1 同红）。全量交主 agent。
- J1-a 要一条短 journal 环（1 MiB 或 6 MiB）：22 次发布之内把环转过去。默认 768 MiB 的环上，同样的洞要在旧根被盖掉之前写满约 19.7 万条记录，每次发布要几千条记录，我没试能不能走到（推的：顺序写一次一条请求要切几千个一单元事务，extent 叶装不装得下没查）。
- 「短环是不是合法配置」我按「mkfs 收、小盘那一档就用 6 MiB」算合法；主 agent 若另判，J1-a 只剩「判据在什么配置上成立」这一层。
- J1-b、J2 的根槽读坏是介质故障，今天的池级 checker 在这些镜像上都红 I-3.1：它们算不算合法镜像由主 agent 判。
- gap 改法只在我的模型上量过、被攻过零轮；它在缺口右边的漏判代价没量。V1 在 J1-a 那几段上没跑，表里「V1 也红」是推的。
- J3-d 只在一段历史上造了一份；它的三步改造是按池级 checker 每一步的红一项一项补出来的，入库 checker 若另有一条判「inode 号水位以下的号都在树里」，它会被别的不变量红（今天没有，推的）。
- 正式跑那一轮里，从 B 前缀那一片到 jwrap、层 0 探针，我在同一条回复里发了十一条命令，它们并行跑了（每条各自七线程），超过派发提示「线程上限：7」；数是确定性的、不受影响，各段 `seconds=` 的挂钟因此偏长（例如 `sweep-raise-A.log` 1 259 秒，单跑时 101 秒）。
- 「没打中」都只抽了一次样。

## 什么现象会推翻这些结论

- J1-a：在入库装置上把那几段重造一遍，CS2 在 txg 8 与下一条之间没有洞（checker 另有办法补回 9、10）、或 TF 在那几张上不红；层 0 全量里「TF 红而层 0 全绿」的状态数为 0（与头 700 个里的 696 个矛盾）。
- J1-a「打中不分辨臂」：找到一种下界写法，在这几张上不红、又不靠「看 txg 缺口」。
- J2：根槽读坏的镜像被判不合法（今天池级 checker 就红它们），这一格就不满足正文第五节第二条的字面。
- J3-d「β 有只红它」：入库池级 checker 在 J3-d 第三步那张上有任何一条红。
- J3-b「整条链改到 R⁻ 之下照样漏」：回退写了谱系记录、或盘上另有字段记下「这条根是一次回退」。

## 没做什么

- 不判辩方那几格（J4、J5 的措辞与代价），不替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑 checker 档包的任何测试、门禁、QEMU、变异；没改仓里 `crates/` 一个字，改动都在草稿目录的副本上，以 `proto.diff` 入库。
- 没读禁读清单里的文件（这一轮辩方的报告与模型、同批别的三轮的产出）。
- 层 0 全量没跑完（见上）；被抛弃的根那一形没造。

## 删了哪些

- `/tmp/claude-1000/m3-birth-floor-r2-opus/tree`（冻结副本的拷贝，连同里面的编译目录 `target/`，1.2G）：已删。
- `/tmp/claude-1000/m3-birth-floor-r2-opus/patchcheck`（核 `proto.diff` 用的第二份拷贝，50M）：已删。
- 草稿目录里留着探索那一遍的日志、两个小脚本（`bin/cap.sh`、`bin/sum_bad.py`）与 `model/`（入库那几份的草稿），共约 10M，不是仓副本也不是编译目录。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r2-opus-output.md` 末行：

```text
  ✓ 核了 14 处引文，对上 14 处，没判 0 处（逐处列在上面）
```
