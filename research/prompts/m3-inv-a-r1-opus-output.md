# m3-inv-a-r1 云端攻方腿报告（2026-09-30）

接续：上一条攻方腿撞限额中断，它落盘的副本、两份 diff、原型用例与日志核过之后接着用（副本与冻结副本只差 `walk.rs`、`recovery.rs` 两个文件，`diff -rq` 现核）；它没写报告，这一份全是这一条腿写的。它跑过的量在这一条腿里全部重跑了一遍，下面的数都取这一次的日志。

## 复跑命令与文件 sha256

```
bash research/prompts/m3-inv-a-r1-opus-model/rerun.sh <草稿目录>        # 源树默认仓根，要是 HEAD fcc4d340 的 crates/
```

副本 = 冻结副本 `/tmp/claude-1000/m3-ring-r1/tree/` 加三份 diff（`patch -p1` 打得上，现核）。**下面全部的数都是副本上的数，不算入库装置上的数。** checker 档包自己的测试一个没跑；原型包 `inv-a-proto` 只把 `singlefs-checker-tier` 当库用，目标名不带 `layer0`。

```
fa61dda8e68af45ad74096fc2ca6c3451d444dde5aa8b913f95ce6fbbb562e58  rerun.sh
52aee96be2e97230801754f2c13fa003dc208fbe560538ef134bcc156608ce26  checker-walk.diff
add2a8a294fba50a105bd798abe3025736aac57cfae456b4d86ed9efdc2bbb30  core-recovery.diff
6bba5495ec48ce159b3bb3576e80f6991665cb13184d2bc50a656274fa01596d  workspace-cargo.diff
8ec113ac04a5f4360b02ca24286f09ac148506f9e2e8b0bfd199ceba3848c77e  inv_a_watermark_replay_and_system_configuration.rs
839ae14414c890a1784c549b0ffae3198eecb0e8ee323c42a68a10648c966878  inv-a-proto/Cargo.toml
67eeddff4dc42f2dbe913d84cb6543279d1e13b0be1f4b82e218451311e32e58  inv-a-proto/src/lib.rs
cf82cb39ee81296fc6f1ac10417e80790bbd4de5b2f03e3e9b08ed2173af79c1  inv-a-proto/tests/inv_a_crash_states.rs
668ed70fc5a083bb84f070844e9ef844f85365f81581647c360c3027b5eb261e  logs/e2.log
f65669ce5ba0775a6dbd7bfccfb96884fff6781f58b7ef4f8452e97a2c9f30d7  logs/e3.log
1ea2a29eecb285c456ce827e7a9cb86175530c8634312385ffe82fe5da297912  logs/e4.log
5724a9d79899d56165d8d0ddc0133735fd71f145fd05d6411b55153dab31439c  logs/e5.log
c53e22359b3664d182d570f3a277225cfeb7ad4e9600fc94fe5a3b7be5ea23cb  logs/harness-all-targets-scan.lines
2bece768bb4be40008b9eb4fbcb858c72c7a0999ddbdbdfb8f421434a0f36c74  logs/harness-core-roots-only.log
38f6e58681cda67f3b188d76417c799870d3ec2cf9ea07b2bab939e31fe6c51e  logs/harness.log
993fe72dfe36fd503d88bf7937729b16f2a978ecc91313289bd81081efae0b13  logs/progress.md
```

三份 diff 各做什么：`checker-walk.diff` 只加观测（记录新根段水位、「并进记录」读法、候选 I-7.8b 的判定、全进程累计），不改任何一条已有判定；`core-recovery.diff` 加一个环境变量开关 `INV_A_CORE_WATERMARK_ROOTS_ONLY`（core 取水位只看根环，C342 退回去的样子），不设时行为不变；`workspace-cargo.diff` 把原型包加进工作区。

名字：下文「只取根」= checker 今天的 I-7.8；「并进记录」= 把环里自证过的记录新根段水位也取进 max 的读法（正文 A5 问的那个改法）；「I-7.8b」= 这条腿自己提的候选「每条有效根的水位 ≥ 环里每条 (实例, txg) 比它早的记录新根段水位」，**只在我的模型上量过、被攻过零轮**。

## 各格判定一览

| 格 | 被攻的对象 | 判定 | 依据（节） |
|---|---|---|---|
| A1-a | R-独 的 I-8.4「施加两次与一次得同一个根」 | 两种读法结局相反：同一起点再施加一次恒等（按构造恒真，坏镜像造不出）；把施加出来的那一版喂回去再施加，在 X1 上不等，而 X1 是 core 照常挂、checker 0 违例的镜像。X1 要一处根槽介质损坏加一次崩溃，不在跑前判据列的两类合法镜像里 | 一·1 |
| A1-b | R-换 的字面「盘上状态相同」 | 打中：挂一次与「崩在取号之后再挂一次」，实例代号、行、两盘各 28 个扇区都不同，读出的内容相同。要先定等价关系才有判定 | 一·2 |
| A1-c | R-独 的只读重放少复刻一条规则 | 打中（一半量过一半推的）：checker 今天那一步只接反向链的重放在 E4 的 2300 个状态上把 I-3.10 判红，core 在这些状态上照常挂；起点含介质损坏，同样不在两类合法镜像里 | 一·3 |
| A4-a | V-盘 与「core 能挂」冲突这句前提 | 前提只对只读恢复成立：一块盘两槽全废时 `recover` 读得出文件，`mount_writable` 拒。跑前判据「与 core 的挂载行为冲突」没说哪个入口，两个入口结论相反 | 二·1 |
| A4-b | V-盘、V-池、V-报 三个候选 | 共同漏一格（不分辨臂）：任何一个槽坏了可写挂载就拒，三个候选都不报；这一格在合法崩溃态里出现（E3、E4 各 15 个状态）。V-盘 在量过的崩溃态上不误红（没有一块盘 0 份自证的状态） | 二·2 |
| A4-c | C466 候选措辞「各盘自证槽里的 device_count 相同、等于盘表」 | 「自证槽」取全部槽时在 core 两个入口都挂得上的 Y4 上红；「等于盘表」在只读恢复挂得上的 Y6 上红。两份都是手改镜像，不算误红，只分辨措辞 | 二·3 |
| A5-a | 「只取根」在「根槽读不出而记录在」的合法镜像上误红 | **没造出**。E2 全量 327711 个崩溃状态里 262155 个是这个形状，三种读法 0 违例；另六组形状也 0。这是一次抽样 | 三·1 |
| A5-b | 改 I-7.8 措辞「并进记录」 | 打中（漏红）：既有坏镜像（中间实例行那一份）与「根轮出环」坏镜像上「只取根」红、「并进记录」不红 | 三·2 |
| A5-c | 「只取根」自己 | 打中（漏红）：只把最新那条根的水位压低时 checker 49 条 0 违例，core 的 `recover` 报 I-7.8 失败；C342 退回去的 core 重发树 11..18，checker 也 0 违例 | 三·3 |
| A5-d | 「根轮出环」坏镜像 | 造得出、只红 I-7.8：建一次树之后经会话入口覆盖写 24 次把发树那条根轮出去，再把环里每条根的水位压到 11。第一版不能连建多棵树，也不必 | 三·4 |
| A3 | I-7.5 判定宽度取 io_min 或取声明的 pbs | 打中：mkfs 收下的三组几何里根槽不是 io_min 的整数倍（24 / 24）或不是声明 pbs 的整数倍（12 / 24）；事实表「推的」那一格只在设备块小于声明 pbs 时成立，设备块 4096 时 mkfs 中途失败 | 四 |
| A6 | extent 偏移非 0 | 量到现状：实例表指针 extent 偏移改成 1，core 两个入口照常、checker 0 违例；同一条指针 MAC 首字节改成 1，checker 报 I-2.4、core 照常 | 五 |
| A2 | I-8.1 改写 | 没跑，只有一段推的 | 五 |

## 一、A1（I-4.1 与 I-8.4）

被攻的条文：`.claude/kb/invariants.md:87`：「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」；`.claude/kb/invariants.md:165`：「任一崩溃点重放后，系统配置指向的树全部可完整遍历」。

### 一·1　R-独 的 I-8.4：「施加两次」有两种读法，结局相反

历史 X1（`a1_replay_twice_across_the_in_flight_cap`）：环长 1 MiB（在飞上限 85），实例 1 建文件 A（txg 3），顺序写 B（txg 4，45 条记录），顺序写 C（txg 5，45 条记录）崩在记录落盘之后、根槽之前；再把 B 的根槽改坏一个字节。许可这一步的代码：`crates/singlefs-core/src/recovery.rs:3109`：「for record in above.into_iter().take(in_flight_limit) {」——水位之上 90 条只取前 85 条，B 的 45 条整体施加，C 只进来 40 条、末条没到、不施加。

`logs/harness.log` 第 30 行原样：

```
INV-A X1 所选根 (InstanceGeneration(1),CheckpointTxg(3))；施加一次 → (InstanceGeneration(1),CheckpointTxg(4)) prefix_applied=45 above_water=90；从那一版再施加 → (InstanceGeneration(1),CheckpointTxg(5)) prefix_applied=45；从所选根重做 → (InstanceGeneration(1),CheckpointTxg(4)) prefix_applied=45；一次与两次相同=false
```

- 读法甲「同一份镜像、同一条所选根，再算一遍」：`replay_journal` 只读、入参相同，两遍恒等（上面「从所选根重做」那一项）。这是按构造恒真，写不出只红它的坏镜像，照跑前判据归 R-换 或 R-形。
- 读法乙「把施加出来的那一版当起点再施加」：X1 上一次得 (1, 4)、两次得 (1, 5)。同一份镜像 `recover` 读得出文件、池级 checker 0 违例（`logs/harness.log` 第 29、31 行）。按读法乙写 I-8.4，它在 core 认的镜像上红。

四句：① 不分辨 R-换 与 R-形，只打 R-独 的读法乙。② 判别它的东西（水位之上的记录数与在飞上限）checker 在镜像上看得到。③ 满足的是跑前判据「R-独 的坏镜像造不出（I-8.4 在单份镜像上恒真）」的反面，但只在读法乙下；读法甲下那一句成立。④ 跑前条款给的两个去向（R-换、R-形）都不碰这一格。

限度：X1 含一处根槽介质损坏，不是跑前判据列的「层 0 两条流的崩溃状态、`checker_known_bad_images.rs` 的好镜像」。只靠崩溃造不造得出「水位之上多于在飞上限」：一次发布的记录数被发布路径压在在飞上限以内，第二次发布的记录在前一次的根槽 FUA 之后才写，所以要前一条根读不出——这一句是推的，没枚举。X1 算不算合法镜像交主 agent。

### 一·2　R-换 的字面在合法崩溃态上不成立

历史 X2（`a1_recovery_and_row_write_once_versus_twice`）：新池新建文件做完的镜像上 ① 可写挂载一次；② 可写挂载崩在取号两写与其后的屏障之后（层 0 枚举域里的一个崩溃状态：整段落、后面的段不落），再可写挂载一次。`logs/harness.log` 第 21、23 行原样：

```
INV-A X2 挂一次: 实例 InstanceGeneration(2) 行 [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(3), applied_transaction_high_water: 0 }] 写行 txg CheckpointTxg(4)；崩过再挂: 实例 InstanceGeneration(3) 行 [InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(3), applied_transaction_high_water: 0 }, InstanceRow { instance: InstanceGeneration(2), selected_root_txg: CheckpointTxg(0), applied_transaction_high_water: 0 }] 写行 txg CheckpointTxg(4)
INV-A X2 逐扇区不同（盘, 扇区数）: [(0, 28), (1, 28)]
```

两边恢复读出的内容相同（3000 字节），checker 都 0 违例。R-换 写成「恢复 + 写行那一整段做两次与做一次盘上状态相同」时，第二次挂载多取一个实例代号、多写一行 (2, 0, 0)，字面必红。要它能判，得先写出等价关系（比什么：读出的用户可见内容、有效根的树表与映射根；不比什么：实例代号、实例表行、系统配置世代号、根槽位置）。这个等价关系是新东西，被攻过零轮。

四句：① 只打 R-换，分辨臂。② 差别全在盘上，看得到。③ 对上的是第五节第一条「一条措辞在任一合法镜像上判红 ⇒ 那条措辞出局」，对象是 R-换 的字面措辞，不是 R-换 这条路。④ 跑前条款没给 R-换 的改法。

### 一·3　R-独 的重放少一条规则就在 core 认的镜像上红

E4（`e4_mount_stream_over_a_record_whose_named_data_unit_is_gone`）：新建文件的单元与记录落了、根没落，那次发布的数据单元两份清零；core 点名验证不过、不施加那条记录。其上可写挂载那条流的 4128 个崩溃状态全量枚举，`logs/e4.log` 第 83 行原样：

```
INV-A-E E4 记录在、数据单元清零 枚举完：states=4128（观察者 4128），耗时 1824.3 秒；枚举器自己的判定：oracle 违例 0、不看 journal 那一遍违例 0、checker 判违例的 {"I-3.10": 2300}、恢复失败的状态 0、点名验证跑过 4110 / 不过 4107
```

2300 个状态上 checker 判 I-3.10 违例，同一批状态 core 恢复 0 失败、可写挂载成功、挂完之后 0 违例（第 88 行那一格）。I-3.10 今天走的正是事实表说的那一步重放（`tree_table_pointer_of_the_version_the_next_mount_applies_first`，只复刻反向链）。这是量过的：**一份不做点名验证的 checker 重放，在记录点名的单元读不出的镜像上与 core 分叉并判红**。R-独 照 D23 已定项 14 写全五条之后这一格该消失——那是推的，没实现没跑。X1 对应在飞上限那一条（不复刻时 checker 施加到 C、core 停在 B），同样是推的。

限度：E4 的起点含介质损坏（数据单元两份清零），不在两类合法镜像里；I-3.10 这 2300 个红不归这一轮的格，照实报给主 agent，没动它。

### 一·4　I-4.1 的对象

「系统配置指向的树」今天没有对象（事实表第一节）。量到的：E2 的 327711 个崩溃状态里点名验证跑过 262158 个、不过 0 个，恢复失败 0 个（`logs/e2.log` 第 107 行）；即纯崩溃态上「core 施加之后那一版」与「只接反向链的一步重放那一版」没分叉，分叉只出现在带介质损坏的 E4。所以 I-4.1 的判别力全落在带介质损坏的镜像上，措辞里的对象要写成「恢复施加前缀之后的有效根那一版」，并写明前缀规则是哪几条；写成「记录新根段指着的树」会把 E4 这类判红。

推翻条件：在只含崩溃的状态上找到一份镜像，core 的 `effective_root` 与「所选根之后第一次发布的记录带提交标记且过反向链」给出的版本不同，一·4 第二句就不成立。

## 二、A4（I-8.5 改写与系统配置冗余）

被攻的条文：`.claude/kb/invariants.md:88`：「全部 tail 槽位中自证校验和通过的记录集合非空，且其中除代号最大者外至少还有一条更早的」。镜像都从「mkfs → 取号 → 暖机 → 新池新建文件」做完的那一份改出来（`a4_system_configuration_redundancy_images`）；盘 0、盘 1 两槽都是世代号 4（tail 2）与 5（tail 3）。

### 二·1　「一块盘全废 core 照样挂得上」只对只读恢复成立

| 镜像 | `recover`（只读） | `mount_writable` | 池级 checker |
|---|---|---|---|
| Y0 干净 | 读出文件 | Ok | 0 违例，不适用 4 条 |
| Y1 盘 1 两槽清零 | 读出文件 | Err `NewerStateStillUnreadableAfterOneReread` | 0 违例，不适用 5 条 |
| Y1b 盘 0 两槽清零 | 读出文件 | Err 同上 | 0 违例，不适用 5 条 |
| Y2 盘 1 世代号大的一槽清零 | 读出文件 | Err 同上 | 0 违例，不适用 4 条 |
| Y2b 盘 1 世代号小的一槽清零 | 读出文件 | Err 同上 | 0 违例，不适用 4 条 |

全是量过的，`logs/harness.log` 第 54、56–59 行。Y1 那一行的拒因原样（同一行里）：

```
mount_writable=Err NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: SelectedVersionAgainstTheWitness { selected_version: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, witness: NewerPublishWitness { witnessed_journal_counter: 3, comparison: SomeSystemConfigurationSlotUnreadOrUnverified } }, …
```

许可它的代码：`crates/singlefs-core/src/mount.rs:404`：「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」，同一处的文档 `crates/singlefs-core/src/mount.rs:434`：「撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载（同一判决 L1）。」。只读那一半：`crates/singlefs-core/src/recovery.rs:1005`：「这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。」。checker 里那句注释只说了只读这一半：`crates/singlefs-checker/src/walk.rs:5740`：「而它是一个挂得上的合法镜像——故障注入让一块盘的两个槽先后写失败就造得出它，判定会被静默放过」。

四句：① 这一格只碰 V-盘（它是唯一把「一块盘全废」判违例的）。② 系统看得到。③ 跑前判据写的是「V-盘 与 core 的挂载行为冲突 ⇒ 出局」：对只读恢复这一分句成立，对可写挂载不成立（可写挂载比 V-盘 还严）。判据没说哪个入口，属于「打中归错了判据」那一类，我不替主 agent 选。④ 条款给的改法「同时改 core」：改只读恢复去拒，是条款变更；不改 core，V-盘 判红的镜像里只读读得出文件。

### 二·2　三个候选共同漏的一格，出现在合法崩溃态里

Y2、Y2b 两份：只坏一个槽，可写挂载就拒，三个候选都不报（V-盘 要整盘两槽全废，V-池、V-报 同）。这一格不是只有手改镜像才有——挂载流的崩溃状态全量枚举里，系统配置槽原地覆写撕裂（第三态）的状态上可写挂载同样拒。`logs/e3.log` 第 80–82 行原样（E4 同样三行、同样的数，`logs/e4.log` 第 85–87 行）：

```
INV-A-E E3 只有单元落了       3 × 崩溃态[自证槽 1+1] 只取根=H 并进记录=H I-7.8b=H 违例=[] → 可写挂载 Err NewerStateStillUnreadableAfterOneReread/SomeSystemConfigurationSlotUnreadOrUnverified
INV-A-E E3 只有单元落了       6 × 崩溃态[自证槽 1+2] 只取根=H 并进记录=H I-7.8b=H 违例=[] → 可写挂载 Err NewerStateStillUnreadableAfterOneReread/SomeSystemConfigurationSlotUnreadOrUnverified
INV-A-E E3 只有单元落了       6 × 崩溃态[自证槽 2+1] 只取根=H 并进记录=H I-7.8b=H 违例=[] → 可写挂载 Err NewerStateStillUnreadableAfterOneReread/SomeSystemConfigurationSlotUnreadOrUnverified
```

从它读出三件事：

- E3、E4 各 4128 个崩溃状态里各 15 个有槽没自证，15 个全部可写挂不上、checker 0 违例。这是 C331 取甲定下的代价，不是新缺陷；但它是「合法崩溃态 + 可写挂不上 + 三个候选都沉默」的一格。三个候选在这一格上不分辨（一起漏），照「打中不分辨臂」办：不拿它判臂，另记一笔。
- 任何把措辞收严成「每块盘两槽都自证」的写法在这 15 个合法崩溃态上误红，出局。C367 已经说过原文后半句不成立，这里是同一件事的计数。
- V-盘「每块盘自证过的集合非空」在 E3、E4 全量与 E2 的 2089 个取样状态上没有一个状态出现某块盘 0 份自证（粗键只有 1+1、1+2、2+1、2+2 四种），即没量到 V-盘 在崩溃态上误红。两槽交替写、中间隔着屏障时崩溃造不出一块盘两槽全坏，这一句是推的。

V-报（checker 返回里加一类「降级」）我没实现。按代码看它要动 `InvariantVerdict` 的每个消费者（层 0 计数把每条不变量分成评估过、违例、不适用三格，`crates/singlefs-checker-tier/src/crash.rs` 的 `Layer0Tally`），这一句是推的。若「降级」按槽报而不是按盘报，它就是三个候选里唯一盖得住二·2 这一格的；按槽报是我提的，被攻过零轮。

### 二·3　C466 的候选措辞「各盘自证槽里的 device_count 相同、等于盘表」

| 镜像（都重封过校验和） | `recover` | `mount_writable` | 池级 checker |
|---|---|---|---|
| Y3 盘 1 两槽 device_count 改成 3 | Failed `SystemConfigurationsDisagree` | Err 同因 | 0 违例 |
| Y7 盘 1 世代号大的一槽改成 3 | Failed 同上 | Err 同因 | 0 违例 |
| Y4 盘 1 世代号小的一槽改成 3 | 读出文件 | Ok | 0 违例 |
| Y6 两块盘四个槽都改成 3（池里两块盘） | 读出文件 | Err `CallerParametersDisagreeWithTheSelectedSystemConfiguration`（`DeviceCountDiffersFromTheSystemConfiguration`） | 0 违例 |

`logs/harness.log` 第 62–65 行。Y3、Y7 坐实 C466 的现状（core 挂不上、checker 全绿）。Y4、Y6 分辨措辞：

- 「自证槽」取每块盘全部自证槽 ⇒ Y4 红，而 core 两个入口都挂得上（core 每块盘只拿世代号大的那一份比）。取每块盘世代号最大的那一份 ⇒ 与 core 同。
- 「等于盘表」⇒ Y6 红，只读恢复读得出文件、可写挂载拒（拒的是调用方给的盘表，不是镜像自己）。checker 拿到的盘数是镜像读者交的，不是调用方的盘表，「盘表」在 checker 里指什么要写明。

Y3–Y7 都是手改镜像，合法历史里 device_count 不变，所以这两条都不算「合法镜像上误红」；它们决定的是 checker 与 core 在坏镜像上同不同步。

### 二·4　与 H6（C463）对不对得上

我没读二·4 那一轮的材料（不在这一轮的背景里），只能给今天实现的事实供对照：C463 行写的是 `.claude/kb/checks-owed.md:383`：「挂得上，且报告里要有一格点名盘 1 的系统配置全废」；今天只读恢复挂得上、可写挂载已经拒（二·1）。V-池 / V-报 另开的那个出口若只在 checker 里，core 的 `RecoveryReport` 里仍没有那一格——两边各要一格还是共用，这一轮的材料定不了。

推翻条件：找到一条可写挂载入口在某个系统配置槽自证不过时仍然成功，二·1、二·2 就不成立。

## 三、A5（树 ID 水位）

两边的取法：checker `crates/singlefs-checker/src/walk.rs:6346`：「.map(|(_, _, root)| root.tree_identifier_watermark)」只取根环；core `crates/singlefs-core/src/recovery.rs:1212`：「.map(|record| record.new_tree_identifier_watermark)」并上环里的记录，可写挂载再与要接的那一版取大（`crates/singlefs-core/src/mount.rs:1147`：「ring.max(version_to_build_on.tree_identifier_watermark)」）。

### 三·1　误红：没造出（一次抽样）

要 checker 误红，得有一份合法镜像：「出现过的最大树 ID」≥ 根环里的最大水位。单元头那一半只数诞生代号不超过根环最大 txg 的节点（`crates/singlefs-checker/src/walk.rs:2419`：「if birth_txg <= newest_published_txg」），再排除实例表判得出没发布过的（`crates/singlefs-checker/src/walk.rs:2387`：「&& birth_txg > row.published_checkpoint_txg」）。「根槽读不出而记录在」的那次发布的节点诞生代号高于环里最大 txg，先被第一道挡掉；等下一次可写挂载把它施加、写行，那条写行的根带的水位已经并过记录。所以结构上两边取法的差落不到 checker 的判定上——这一段是推的，下面是量的。

量过的形状与范围（每个崩溃状态三种读法都判；「观察者」= 再可写挂载一次判一次，挂出树表 0 条的再新建文件判一次，这几步是放开的用户动作）：

| 组 | 起点与历史 | 状态数 | 只取根 / 并进记录 / I-7.8b 判违例 | 观察者 |
|---|---|---|---|---|
| E2 | 新建文件的单元与记录落了、根没落；其上可写挂载那条流，每段都展开、带第三态 | 327711（全量） | 0 / 0 / 0；其中记录水位高于根环水位的状态 262155 个 | 取样 1/160，2089 个，全绿 |
| E3 | 新建文件只有单元落了；其上可写挂载那条流 | 4128（全量） | 0 / 0 / 0 | 每个状态都做，全绿（15 个可写挂载被拒，见二·2） |
| E4 | 同 E2 起点、数据单元两份清零（介质损坏）；其上可写挂载那条流 | 4128（全量） | 0 / 0 / 0（I-3.10 红 2300，见一·3） | 每个状态都做，挂完全绿 |
| E5 ① | 新建文件那次发布自己：单元段只取空与整段，其余各段整写子集 | 9（不是全量） | 0 / 0 / 0 | 全绿 |
| E5 ② | 同上，单元段 24 写的随机子集，种子 1 | 400（取样） | 0 / 0 / 0 | 全绿 |
| W3 | W1 起点上可写挂载那条流逐前缀截断（上一条腿的装置，只截前缀） | 63 | 0 / 0 / 0 | 再挂一次，全绿 |
| 扫描 | 副本上 `singlefs-harness` 全部测试目标里每一次 `check_pool_image` | 955 次判定 | 违例清单为空的 716 次里三种读法 0 红 | — |

E2 那一行的原样（`logs/e2.log` 第 106 行）：

```
INV-A-E E2 记录在、根没落 工作线程上每个状态的池级 checker：走到 I-7.8 的 327711 次，只取根判违例 0，并进记录判违例 0，候选 I-7.8b 判违例 0，记录里的水位高于根环里的 262155；观察者取样 1/160
```

取样与缩减，照定义写明：

- 新建文件那次发布自己的全量是 16777223 个状态（`closed_form_state_count`，`logs/e5.log` 第 6 行），超过约 10⁶，**没跑全量**；E5 两组不是层 0 的枚举域，只当补充取样。这条流是层 0 在册的两条流之一，「只取根」在它上面的判定由入库的层 0 跑法管，我没重做。
- E2 的观察者：先跑的一小段 25 个状态 10.3 秒（0.41 秒一个），全量 327711 个要约 37 小时，超过 40 分钟，缩成按状态序号散列取 1/160（2089 个，实跑 750 秒）。缩的只是「崩溃之后用户再做什么」这一串；每个崩溃状态的恢复与池级 checker 一个没落，由枚举器的工作线程判、从全进程累计里读。
- 批次：b1 三件并行各 2 线程（E2、E3、E4），估 28 分钟一件；其中 E2 发现要 37 小时，停掉（退出码 101，是我停的），E3 实跑 1770 秒、E4 实跑 1824 秒。b2 一件被内存包装拒（退出码 251，`auto` 而峰值表里没有这条命令），没跑、不算结果。b3 一件（E2，6 线程），估 16 分钟，实跑 750 秒。
- 这条腿与上一条腿合起来跑的全量：327711 + 4128 × 3（E3、E4、E3 不带观察者的试跑）= 340095 个状态，加上停掉的两次部分跑，远不到 10⁷。上一条腿没调过枚举函数。

没试的形状写在「没打中的形状」一节。照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，这一格只是一次抽样。

### 三·2　「并进记录」的改法在两份坏镜像上漏红

- 既有坏镜像（`checker_known_bad_images.rs` 的中间实例行那一份，W4 在内存盘上照搬）：`logs/harness.log` 第 70 行原样

```
INV-A W4 根环水位全压到 11（改了 5 条根）: violated=["I-7.8"] I-7.8=Violated("根环水位最大 11，盘上出现过的最大树 ID 15") roots_max=11 records_max=Some(19) highest_seen=15 with_records_I-7.8=Holds I-7.8b=根 (3, 6) 水位 11 < 更早的记录 (2, 5) 新根段水位 19
```

  全目标扫描里同一形状出现 3 次（`grep -c 'stock78=V with_records78=H' logs/harness-all-targets-scan.lines` 输出 `3`，三行的违例清单都是 `["I-7.8"]`）。把 I-7.8 的措辞改成并进记录，这份在册的坏镜像不再红。
- 「根轮出环」坏镜像（三·4）：`with_records_I-7.8=Holds`，同样不红——40 次覆盖写之后 txg 3 那条记录还在 journal 环里。

四句：① 分辨臂：只打「改措辞并进记录」这一支，不打「不改」。② 记录在镜像上，checker 看得到。③ 对上第五节第二条的反面：这条改法让一份已有的只红它的坏镜像变绿。④ 跑前条款给的去向是「造出误红 ⇒ 改措辞」，误红没造出，这一支本来就不触发；我报的是「就算想改，改成并进记录有代价」。

### 三·3　「只取根」自己漏的两格

① 只把最新那条根的水位压低（一次发布没带上累计水位的样子）。`logs/harness.log` 第 258、259 行原样：

```
INV-A R2 最后一份镜像只把最新那条根的水位压到 11: violated=[] I-7.8=Holds roots_max=19 records_max=Some(19) highest_seen=18 with_records_I-7.8=Holds I-7.8b=根 (1, 43) 水位 11 < 更早的记录 (1, 3) 新根段水位 19
INV-A R2 只压最新根之后违例 []；recover=Failed Some((InstanceGeneration(1), CheckpointTxg(43))) InvariantViolated { invariant: "I-7.8", detail: "树 ID 不低于水位" } effective=Some((InstanceGeneration(1), CheckpointTxg(43))) prefix_applied=0 verification_failed=0
```

checker 49 条 0 违例，core 的只读恢复以 I-7.8 失败（`crates/singlefs-core/src/recovery.rs:3541`：「invariant: "I-7.8",」，它拿所选根自己的水位比树表条目）。环里取 max 的读法要等带高水位的旧根全部轮出去才红：R 臂里第 1 到第 23 次发布之后都不红，第 24 次才红（`logs/harness.log` 第 223–231 行）。

② C342 退回去的 core（`INV_A_CORE_WATERMARK_ROOTS_ONLY=1`）在 W5 历史上重发树 11..18，`logs/harness-core-roots-only.log` 第 22 行原样：

```
INV-A W5c 实例 2 新池新建文件之后（mutant=true）: violated=[] I-7.8=Holds roots_max=19 records_max=Some(19) highest_seen=18 with_records_I-7.8=Holds I-7.8b=根 (2, 4) 水位 11 < 更早的记录 (1, 3) 新根段水位 19
```

两种 max 读法都不红，只有 I-7.8b 红。这个变异今天由 harness 的一条用例杀（`crates/mutations.tsv` 第 325 行点名的 `with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_from_its_journal_record`），不靠 checker。W5 的起点含介质损坏（数据单元两份清零）。

四句：① 不分辨「改不改措辞」两支（两种 max 读法一起漏），是两支共用的前提「只比环里的 max」的病。② 看得到。③ 不对应第五节任何一条——跑前判据只问误红，没问漏红；照实报。④ 无对应改法。

改法各修哪一格（I-7.8b 是我提的，只在我的模型上量过、被攻过零轮）：

| 镜像 | 只取根（今天） | 并进记录 | I-7.8b |
|---|---|---|---|
| W4 环里每条根压到 11 | 红（量过，harness.log 第 70 行） | 不红（量过，同行） | 红（量过，同行） |
| R2 轮出环之后每条根压到 11 | 红（量过，第 256 行） | 不红（量过，同行） | 红（量过，同行） |
| R2 只压最新根 | 不红（量过，第 258 行） | 不红（量过，同行） | 红（量过，同行） |
| W5 core 只取根环、重发 11..18 | 不红（量过，harness-core-roots-only.log 第 22 行） | 不红（量过，同行） | 红（量过，同行） |
| E2、E3、E4、E5 的合法崩溃态 | 0 红（量过） | 0 红（量过） | 0 红（量过） |
| 被比的那条记录已被 journal 环盖掉之后 | 同上各行 | 同上各行 | 不红（推的：没有记录可比） |

I-7.8b 不替代 I-7.8：单元头与树表那一半它不看。

### 三·4　「根轮出环」坏镜像的造法

第一版建不了「R + 1 个 checkpoint 各建一棵树」：会话入口的三种改动都不建树，树表 0 条时直接拒（`crates/singlefs-core/src/mounted_session.rs:261`：「PoolVersion::WithoutFile(_) => return Err(UserChangeRefused::NoFileVersionToChange),」），树只在新池新建文件那一次发（11..18）。不必多建：建一次、把发树那条根轮出去就够。

造法（`a5_roots_rotating_out_through_the_session`，量过）：mkfs → 可写挂载 → 新池新建文件（txg 3，水位 19）→ 经 `MountedSession::publish_user_change` 覆盖写 40 次。第 24 次之后 txg 3 那个根槽被 txg 27 盖掉（`crates/singlefs-core/src/root_ring.rs:115`：「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」，3 个区域 × 8 槽）。合法镜像在 11 个取样点上全绿（第 244–254 行）。再把环里 24 条根的水位都改成 11、重封，`logs/harness.log` 第 256 行原样：

```
INV-A R2 最后一份镜像根环水位全压到 11（改了 24 条根）: violated=["I-7.8"] I-7.8=Violated("根环水位最大 11，盘上出现过的最大树 ID 18") roots_max=11 records_max=Some(19) highest_seen=18 with_records_I-7.8=Holds I-7.8b=根 (1, 24) 水位 11 < 更早的记录 (1, 3) 新根段水位 19
```

只红 I-7.8。「判定恒真」的变异照常是把 `watermark > highest_seen` 换成真；在册的水位坏镜像该已杀它（我没核变异表），这一份多杀不了别的变异——它多出来的只是「发树的根不在环里」这一条前提，checker 眼里与 W4 同形。

装置的一处坑，别照抄：R 臂（`a5_roots_rotating_out_of_the_ring`）直接调 `publish_without_units` 连发，两条臂从第 23 次起都多红一条 I-3.1（`logs/harness.log` 第 222、241 行），包括水位照抄 19 的那一臂。那是绕过会话入口造出来的历史，不是用户够得着的；走会话入口的 R2 没有这条红。R 臂只用来读「第几次发布之后 I-7.8 才红」。

推翻条件：在只含崩溃的状态上找到一份镜像「只取根」判红而 core 挂得上，三·1 翻面；找到一份合法崩溃态 I-7.8b 判红，三·3 的表最后两行翻面。

## 四、A3（I-7.5 根槽按判定宽度对齐）

`a3_mkfs_root_slot_alignment`：内存盘的设备物理块与参数里声明的 pbs、io_min 分开取，看 mkfs 收不收、24 个根槽偏移对声明的 pbs 与 io_min 取模。全是量过的（`logs/harness.log` 第 33–52 行）：

| 设备块 | 声明 pbs | io_min | 槽距 | mkfs | 不是声明 pbs 整数倍的槽 | 不是 io_min 整数倍的槽 | mkfs 之后 checker |
|---|---|---|---|---|---|---|---|
| 4096 | 4096 | 3072 | 6144 | Err `BlockDevice(Unaligned { offset: 6144, … })`，失败前已发 18 步 | 12 | 24 | — |
| 4096 | 4096 | 12288 | 12288 | Ok | 0 | 24 | 0 违例 |
| 4096 | 4096 | 8192 | 8192 | Ok | 0 | 0 | 0 违例 |
| 512 | 512 | 4096 | 4096 | Ok | 0 | 0 | 0 违例 |
| 512 | 4096 | 3072 | 6144 | Ok | 12 | 24 | 0 违例 |
| 512 | 512 | 3072 | 6144 | Ok | 0 | 24 | 0 违例 |
| 512 | 4096 | 12288 | 12288 | Ok | 0 | 24 | 0 违例 |

第五行原样（第 45 行）：

```
INV-A Z 设备块=512 pbs=4096 io_min=3072 槽距=6144 mkfs=Ok 24 个根槽里偏移不是 pbs 整数倍的 12 个、不是 io_min 整数倍的 24 个；前 4 个 [(1048576, 0, 1024), (1054720, 2048, 1024), (1060864, 0, 1024), (1067008, 2048, 1024)]
```

打中的三处：

1. 事实表「推的：io_min = 3072、pbs = 4096 时……mkfs 不拒」只在设备自己的块小于声明的 pbs 时成立（第五行）。设备块 4096 时 mkfs 不是「拒」也不是「收」，是写到第二个系统配置槽时被块设备的对齐判定打断，前面 18 步已经发出去（第一行）——盘上留下半份格式。这不归 I-7.5，照实报。
2. 判定宽度取「io_min 大于 pbs 时按 io_min」（invariants.md I-7.5 那一行的射程注）：io_min 不是 2 的幂时，区域起点 1 MiB 不是它的整数倍，mkfs 收下的镜像上 24 个槽全红（第二、六、七行）。除非 mkfs 同时拒这类 io_min，或者式子改成「相邻槽的间距是判定宽度的整数倍」而不是「起始偏移是整数倍」。后一个改法是我提的，只在这张表上对过数（槽距是 io_min 的整数倍，七行都成立）、被攻过零轮；它丢掉的是「区域起点本身没对齐」那一格。
3. 判定宽度取声明的 pbs：第五行 12 个槽红，而这份镜像 mkfs 收了、checker 今天 0 违例。声明的 pbs 大于设备真实的块时 mkfs 没有任何一处拦（挂载只比「盘上与参数相等」，事实表第四节）。

非 2 的幂的 io_min 真设备报不报得出来，我没查外部资料，未核实。

四句：① 分辨「取 pbs / 取 io_min / 取大」三种宽度：取 pbs 只在第五行红，取 io_min 或取大在第二、五、六、七行红。② checker 拿得到声明值，拿不到探测值（C388），所以第五行「声明与设备不符」checker 自己分不出，只能按声明判。③ 对上第五节第一条，但前提是「mkfs 收下的镜像」算不算合法镜像——跑前判据只列了层 0 两条流与好镜像，这几组几何不在其中，交主 agent。④ 跑前条款没给改法。

## 五、A6 与 A2

A6（`a6_nonzero_extent_offset_in_a_pointer_head`，量过，`logs/harness.log` 第 267、268 行）：把每条有效根里实例表指针的 extent 偏移（指针偏移 32）改成 1 并重封，`recover` 读出文件、`mount_writable` 成功、checker 0 违例；对照，同一条指针 MAC 首字节改成 1，checker 报 `I-2.4: 实例表指针：指针头部 MAC 16 / nonce 12 有非 0 字节，加密关着时它们恒 0`，core 两个入口照常。读者那一行：`crates/singlefs-core/src/pointer.rs:140`：「reader.skip(2); // extent 偏移：第一版写 0，读到非 0 怎么处置没有条款，不判」。

从对照读出一件与「core 读者要不要同时改成拒」有关的事：MAC / nonce 非 0 今天也只有 checker 判、core 的这条读路径不拒（上面那一行 MAC 对照里 core 两个入口都成功）。所以「同 MAC / nonce 一样」按今天的实现读，是 checker 判 I-2.4、core 不拒；要 core 拒，是比 MAC / nonce 今天更严的一档。只试了根记录里的实例表指针一处；树表条目、码 2 子指针、extent 数据指针没试。合法镜像上误红的风险：实现第一版恒写 0，E2–E5 的崩溃态与这条改法无关，没量。

A2：没跑模型。只有一句推的：改写候选比的是同一个系统配置槽里的三个字段与同一个槽里的环长，四个数同受一份槽校验和保护、同由 mkfs 一次写出，判得出的只有「槽自证过而字段彼此不符」的手改镜像；「环装不装得下一次发布」它判不出，那一半今天在发布路径的拒绝里。算不算同义反复由主 agent 对着 test-discipline 那一条判，我不判。

## 没打中的形状

- A5 误红：试过上表七组。没试的：管理员回退之后的崩溃态（回退只在带文件的现行版上做，且两版树 ID 不同时拒，`crates/singlefs-core/src/mount.rs:5129`：「if target_version.tree_identifiers != current.tree_identifiers {」，推的：够不着水位）；抬 F 与卸载那一串的崩溃态；三块以上的盘；journal 环小到记录被盖掉的几何；两次以上「记录在、根没落」叠起来的历史（要中间夹介质损坏）。
- A5：想过「发布中途写失败之后不重挂接着发」能不能把孤儿节点数进去——带单元的发布失败会冻结（`crates/singlefs-core/src/transaction.rs:1151`：「allocator.freeze_publish(FrozenPublish {」），树表 0 条时卸载一个字节不写（`crates/singlefs-core/src/mount.rs:2080`：「return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {」），按代码够不着；没跑。
- A4：V-盘 在崩溃态上误红没造出（E2 取样、E3、E4 全量里没有一块盘 0 份自证的状态）。
- A1：只靠崩溃让「喂回去再施加」不等，没造出（推的够不着，没枚举）。

## 这条腿自己的限度

- 全部数字出自副本（冻结副本加三份 diff），不是入库装置上的数；要引得在入库装置上重做。
- 三·1 是一次抽样；E2 的观察者只看了 1/160；新建文件那次发布自己的 16777223 个状态没跑全量。
- E3、E4 的日志是加全进程累计之前的那一版二进制跑的（观察者每个状态都做，数从观察者的粗键里读）；E2 是加了之后跑的。`logs/harness-all-targets-scan.lines` 是上一条腿用更早的副本（只有观测、没有全进程累计）跑的，那次有两个目标因副本里没有 `litmus/` 目录失败，与改动无关。
- I-7.8b、「按槽报降级」、「相邻槽间距」三条是我提的，各自只在我的模型上量过或只对过一张表，被攻过零轮。
- X1、E4、W5 的起点含介质损坏，算不算跑前判据里的「合法镜像」我不判。
- 二·4 那一轮的 H6 条款没读，A4 最后一问只给了实现事实。

## 没做什么

- 不判正推腿的各格，不替主 agent 采纳，没读禁读清单里的文件。
- checker 档测试、层 0 在册的流、变异表、门禁一个没跑；没改 `crates/`，没做 git 写操作。
- A2 没建模型；A6 只试一处指针；R-独 的完整只读重放没实现（一·3、一·4 里标了推的）。
- 一·3 的 I-3.10 在 E4 上 2300 个红、四节第 1 条 mkfs 中途失败留半份格式，都不归这一轮的格，只报不修。
