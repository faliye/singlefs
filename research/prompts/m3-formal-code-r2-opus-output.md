# m3-formal-code-r2 云端攻方（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 L1 L2 L3 L4 L5 V1 W1 H1 G1 U5 U6 F6 -->

腿：云端攻方（Opus）。攻击面：正文第四节分工表「云端攻方」一行，K1–K4，重点 K2、K3；不重复第一轮攻方攻过的角度（胶水差分、择新键摊入、扫环内存、Verus 必红表）。代码按冻结副本 `/tmp/claude-1000/m3-formal-code-r2/tree/` 拷出来的副本读、改、跑（清单 `research/prompts/m3-formal-code-r2-snapshot/crates-sha256.txt` 241 行在拷贝上 `sha256sum -c --quiet` 无输出、退出 0）。下文所有数都是副本上的数，不是入库装置上的数。代码引文的路径写冻结副本的绝对路径：我交回前仓里的工作树已经有人改过（`crates/singlefs-checker-tier/src/crash_injection.rs` 里 K2-c 引的那一行挪到了第 2130 行，`crates/singlefs-reference-filesystem/src/lib.rs` 里 K1 引的那一句已经不在），按工作树核行号对不上。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| K2-a 推的四样（树 ID 水位、两根指针、txg 下界、扫描报告）与其余六样在合法历史上误红 | 没打中 | 11 个窗口第一次崩溃层 0 全量 147 806 态，只挂载、挂载加一次发布各判一次，十样 0 红 | 量过 |
| K2-b 二次崩溃三个相位 | 没打中 | 3 个窗口取 28 个第一次崩溃态，各把那次挂载的写流按层 0 全量枚举，二次崩溃 234 773 态 0 红（含挂载推了抬 F 的一态） | 量过 |
| K2-c 挂载推了抬 F 时的 F 一半 | 半中（漏判，不误红） | 暖机与抬 F 同一相位，暖机那一次发布的 4 107 个二次崩溃态（抬 F 第一个写之前）F 也整段不判 | 量过 |
| K2-d 拒绝合法的复算与 D23 已定项 14 单子 | 没打中 | 五步逐条对上；量到的 165 次拒全是 N-配置，复算全判合法 | 读码 + 量过 |
| K2-e 卸载记号 | 覆盖缺口（读码） | 崩溃注入历史里没有卸载一步，「R1 是卸载写的」那一支在接入处走不到；原型补的 `um` 窗口 24 618 态 0 红 | 读码；补跑量过 |
| K3-a checker 逐档与实现逐档的结局 | **打中** | 13 种手放槽里 5 种结局不同；其中「I-8.1 字段不对」那一种只在逐档路上分叉（checker 全绿、实现整池拒），是 11-5 新带进来的；S 越界、R 越界、incompat + 版本三种普通读法上也分叉（共用前提） | 量过 |
| K3-b mkfs 写零后的钉值 | **打中（漏列）** | crates 的 mkfs 今天 25 步；E170 两份 exact 复跑产物仍钉 `local=23 crates=23`，11-5 漂移表没列 | 25 量过；E170 复跑红推的 |
| K3-c GPU 判器 | 没打中 | 逐档函数与 CPU 同序同判；GPU 只读表里落点，合法流上同值 | 读码（推的） |
| K4-a ② 借槽距与 ③ 逐档分不分得开 | **打中一半** | core 删 ② 被一条用例杀（11-11 的推测不成立）；checker 删 ② 在 harness 档 108 个二进制上全绿 | 量过 |
| K4-b E165 r3 装置 | 没攻 | — | — |
| K1 RefFS 包 | 没攻 | 只读包，记三处与 H1 定案不同 | 读码 |

## 复跑

```bash
M=/home/fy5090/code/singlefs/research/prompts/m3-formal-code-r2-opus-model
bash $M/tools/rerun.sh /tmp/claude-1000/m3-formal-code-r2/tree /tmp/claude-1000/<草稿目录>          # 全部：k3 k2first k2second k4
bash $M/tools/rerun.sh /tmp/claude-1000/m3-formal-code-r2/tree /tmp/claude-1000/<草稿目录> k3       # 只跑 K3（几秒）
```

`rerun.sh` 把冻结副本拷进草稿目录、把 `prototypes/r2probe` 加成工作区成员、经 `run-with-memory-cap.sh 3G` 与 `capped.sh 6` 编 release，再逐件跑；日志写 `<草稿目录>/runs/`，与 `outputs/` 里同名的比计数行（去掉 `elapsed`、包装行与 `e.g.` 例子行）。`outputs/` 里的日志是这一轮先后几版原型跑的：判法与接法每版相同，后加的只是 `R2_SECOND_CAP`、`R2_FIRST_PER_SEGMENT`、`R2_SECOND_BUDGET`、`R2_ONLY_RAISED` 四个开关与 `k2one`、`k3slot0`、`um`、`fms` 几个模式；`k2-mkmnt.log` 出自第一版，没有 `SECOND enumerated …` 那一行。K4 两个变异各跑两批（`k4-targets` 53 个、`k4-rest` 55 个），`outputs/` 里 checker 那一份的第二批分成了 `rest0`–`rest3` 四个日志（`rest0` 头一次缺 `litmus/` 目录红了一条与变异无关的，`rest0b` 是拷进去之后的重跑）。

## 模型目录每个文件的 sha256

`research/prompts/m3-formal-code-r2-opus-model/` 下 47 个文件（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
a944957816ea731d22a5730ce112b13dfd0391a45b0c4ef0129722701486f40a  ./outputs/k2-cold-first.log
96487c7087e5d59e9fab7f8ee3c28c8890df571c3ced6810e46f4a0fd7453c67  ./outputs/k2-fm256m12-first.log
c91a9afa09606400a95d0534399bd44cce4a2b1b3ebcce9b8fc07e17fafa9c79  ./outputs/k2-fms12-first-b.log
5bf700a12038db158bb1b2483963769ab182cb60615600b9a1d23f5124e00695  ./outputs/k2-fms12-first.log
de8ba39fe6ef732fb7d8bc3bb0b17ab02e5845719565690a02d3e322d8187fcd  ./outputs/k2-fms12-second-raised.log
2b44e67329afe4c8de03c3e7b745232b875b96f51b9536234ee038c5f85e795d  ./outputs/k2-fms12-secondsizes.log
e804913ba349c92167c5bf042ec4c86ccaa9d1f6f84b4e17461175cc95290e9e  ./outputs/k2-mkmnt.log
4e872d4d8a48f8e0e8f67e0963f3fb6840a7f7a4059b72e78e2c18fa0ec4bda7  ./outputs/k2-mnt-0-3000-secondsizes.log
c9ca23c0eaba96dbeb38dd54fffb33ee1ad96faad0e204d1c9393ee02dfca37d  ./outputs/k2-mnt-first.log
0d7e98b0de957e4f41f9001174e5277e52c46e552887fcaf1da625a79579503a  ./outputs/k2-mnt-second-seg1.log
07272b3f64a5ea8cc243691554d74a0910da03634f6204d55ad26cf41679d99a  ./outputs/k2one-fms12-0-a.log
323a3130d148aad440de5e81d8859e4c6b02652733bcabf3d312d6e69a9fe989  ./outputs/k2one-fms12-0-b2.log
64f878ce175cc1b14fc3ce2d23dc239d799d522fcae94c4c1a970e4bd317841d  ./outputs/k2one-fms12-0-b.log
7a1275f7830d8fc537d2509dc723651e5c89a80f7b8330ac7806cf9abe4a3de8  ./outputs/k2one-fms12-0-rate.log
c05c16faa3840a0f1db701b42e68fa25a78444f3878c4e18c413afbba2718049  ./outputs/k2-puw-first.log
56b0068912850e98c0d141b3a8276f8ae8dbd886844964fc18d5882f7f148d12  ./outputs/k2-rb-first.log
55d94808f5029a2b6051e683f3d4641fb050a1158d95b76079df773cd69dcc2c  ./outputs/k2-rbm-first.log
e7f4aa087de15c0f7c67eabdccb2946fabeb68c5af7e144dc473d3110f2979ce  ./outputs/k2-rfx3-1-first.log
069f03d493c23cf7461cbdd0a6b5d8fcffcf0300ece95b1527a83f1b57f5668d  ./outputs/k2-rfx4-2-first.log
7e822b3298dcf969a576fe54783948cfe92a0e937727a0af12d108fa4d187bba  ./outputs/k2-um-first.log
7fccd812e0fafe8b1ce2264fb10d139e0c662a0d909f6462c3ecbf7948ddcdfc  ./outputs/k3mkfs.log
c3e49f36945df7f519d23b57b656e64fffe966f0beaaa52918d2209bd1609fbd  ./outputs/k3slot0.log
65f2f5e5e57a6962d2513cd0671a1d781fe018ee779e88f8de3e1af3d978b8f2  ./outputs/k3slot.log
8d1919a4c8db186e167610eba56269c51abd84cee6ce5b4b2d424cc30fa0f07a  ./outputs/k4-checker-borrow.log
765508e83283e6cfabf3baeca441dcd9eca0d553e814fe075de4d85476454b14  ./outputs/k4-checker-borrow-rest0b.log
2d5afd7fcf65514a3738090d11aae1d4bf98578d76b4679207ff12a027419f9a  ./outputs/k4-checker-borrow-rest0.log
a001703f670342998193bbec0565f04ade9ba7ae8992ce21465bf79ed6e62c07  ./outputs/k4-checker-borrow-rest1.log
279486e414bbc665bd1062b4c2d01d921e9ec26e4c2e8c62559a5bfeebeb003e  ./outputs/k4-checker-borrow-rest2.log
814c7058b20c82a2a36087fd9391f065c0671d6993713c150cfeb9eeab10bd75  ./outputs/k4-checker-borrow-rest3.log
2c820d5136ce505a84bb295f5732a46d776d49600bc7a9abbae2c36204e79622  ./outputs/k4-core-borrow.log
2b8349d19e48ff51c90b266c13326dfffee4393e4cfb8a9e5d4bf8ef396c97ba  ./outputs/k4-rest.txt
7de1d60a9e06d5c127e09b4b8dea5e25a629a7a19e7f83f6861c38a8286a1e12  ./outputs/k4-targets.txt
4df8322409f5a22bb2044b28c367ed4788ce1703edf18f8f9e3b737eef8c5fd2  ./outputs/pilot-mnt-0-40-all.log
714f22ad1daef9b2ccebd214098a0df856a51a2eb8dbb40fff4862c882822b90  ./outputs/pilot-mnt-0-600-none.log
04f8ab155819bf6b869e8b4cf9b4f197bff586b211ed4ea0780a891cd17f3f19  ./outputs/scan-fms.log
ae3db32876b9ca2c3dceba7af6fddaeaedefc637e670d4ee51880fb822deb82b  ./outputs/sizes2.err
ba98f91130fcefa373f85b70fd93fdf46064f624a4e5dabe4aed5b26c38bedf5  ./outputs/sizes2.log
369a47be43b5b6ac9e09201b315cfb584e0f92ccb8ddfcacd3f0ecf91182a067  ./outputs/sizes3.err
3c0fc6209a96a8f18126785ed2d6f1586be9a01a7ed79e74c1ceb0a068ec855a  ./outputs/sizes3.log
414bbd8d6f4c629edbef59e1208e74b01ae5d24f07e1a76b0efaa23dc0d41030  ./outputs/sizes.err
6f77235bf358b4bc8743f9c4d5115cbef38defcd9cf9bb0d6e1e620c79ef2a6c  ./outputs/sizes.log
77b6d3494516aed307c15bf056beab69b8e82e543b6c7815fc927d8d213ca9c4  ./outputs/sizes-um.log
706a800ae93de2178a4012632d39db10205f6a6e5a7594032f6a0ce6d907df22  ./probes/mut-checker-borrow.patch
0171853b97a118be11ce051935662a0741b5553d3ba5b0e9926a1639d6215b82  ./probes/mut-core-borrow.patch
9ecf43a1dcb0a92c426a0f61a61c85921ec255c312e13092637d071a542565bd  ./prototypes/r2probe/Cargo.toml
95e269c4e8ac15192b398c559696e9837112ad5740697ac639ebc74a7b5b0834  ./prototypes/r2probe/src/main.rs
1ea3b9deb03a290930fd8358c20bfe3d95b73a3dd49c87b266560ecf8c061768  ./tools/rerun.sh
```

## K2　挂载透明（L2）

### K2-a　「推的」四样与其余六样在合法历史上误不误红（没打中，量过）

装置：`prototypes/r2probe/src/main.rs` 的 `k2` 模式，照崩溃注入接法重写的一份（崩溃注入那几个函数是私有的，接法照 `crates/singlefs-checker-tier/src/crash_injection.rs` 第 1001–1172 行抄，判法本身调入库的 `singlefs_harness::mount_transparency`，一个字没改）：每个崩溃状态上 R1 = `read_recovery_fields(崩溃镜像, TheRecoveredVersion)`；`mount_and_publish_once_on_the_crash_image` 起一次可写挂载加一次发布；挂载被拒的走 `judge_writable_mount_refusal`（盘上逐字节比、只读读回按模型的允许集合判）；做成的，挂载那一段写叠到崩溃镜像上读 R2 判 `MountAlone(floor)`，发布做成的再在挂载加发布之后的池上读 `TheVersionBeforeTheRecoveredOne` 判 `MountAndOnePublish(floor)`。崩溃状态照层 0 的枚举域取（`judge_layer0_state_range` + `full_expansion`，每段都展开、原地覆写取第三态），不是崩溃注入那种每段抽一个。

第一次崩溃的窗口（历史全是原型手写的，起点与前缀不录、窗口那一步录成写流）与结果：

| 窗口 | 前缀 → 窗口那一步 | 层 0 全量态数 | 挂载做成 | 拒（判合法 / 不合法） | 十样判红 |
|---|---|---|---|---|---|
| mkmnt | mkfs → 第一次可写挂载 | 33 | 18 | 15 / 0 | 0 |
| mnt | 第一个文件 → 重挂 | 20 511 | 20 496 | 15 / 0 | 0 |
| rb | 第一个文件、覆盖写 → 回退到次新 | 4 108 | 4 103 | 5 / 0 | 0 |
| rbm | 第一个文件、覆盖写、重挂 → 回退 | 4 108 | 4 103 | 5 / 0 | 0 |
| cold | 第一个文件、覆盖写、冷启动恢复 → 重挂 | 24 618 | 24 598 | 20 / 0 | 0 |
| puw | mkfs、挂载 → 零单元发布 | 13 | 8 | 5 / 0 | 0 |
| rfx3-1 | 第一个文件、三次覆盖写 → 抬 F | 8 223 | 8 208 | 15 / 0 | 0 |
| rfx4-2 | 第一个文件、四次覆盖写 → 抬 F | 12 330 | 12 310 | 20 / 0 | 0 |
| fm256m12 | 256 槽小盘、十二次满单元覆盖写 → 重挂 | 20 511 | 20 496 | 15 / 0 | 0 |
| um | 第一个文件、挂载、正常卸载 → 重挂（R1 那一版带卸载记号） | 24 618 | 24 598 | 20 / 0 | 0 |
| fms12 | 256 槽小盘、十二次满单元覆盖写（前缀关掉空间准入，r5 攻方 fms 同一种造法）→ 重挂 | 28 733 | 28 703（其中挂载推了抬 F 的 20 496） | 30 / 0 | 0 |

合计第一次崩溃 147 806 态、被拒 165 态（`outputs/` 里 12 份日志的 `states_judged` 与 `MOUNT refused … count` 用 awk 求和），每个做成的状态判两个动作，判红 0（日志里 ` RED ` 0 命中）；被拒的 165 态全是 `NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfiguration…)`，复算全判合法。逐窗口原样输出在 `outputs/k2-<窗口>-first.log`（fms12 分两段 `-first.log`、`-first-b.log`），汇总行的形状例：「ALONE judged count=20496」（`research/prompts/m3-formal-code-r2-opus-model/outputs/k2-mnt-first.log:5`）。

实现员推「树 ID 水位先误红」（取根环与记录的 max）。按代码推它在不带故障的历史上走不到：水位只在第一个文件那次发布抬（`transaction.rs` 发十二个号那一处），每次新实例第一次发布取环里根与记录的 max 再接着往上走，所以 R1 那一版的水位低于环里某条根或某条记录，只能是那条根或记录不在 R1 的时间线上、又没被任何一次挂载的 max 收进来——要么一次发布的记录持久了却没被施加（单记录发布在这几个窗口里不会：记录写之前一道屏障，单元先持久），要么有被抛弃的时间线（历史里那一步 `CrashRecoveryAbandoningTheNewestRoot` 在 C554 乙之后被拒，原型里试过 `ab`、`abf` 两段，那一步都报 `NewerStateStillUnreadableAfterOneReread`，不造出被抛弃的根）。多记录发布（点名项 68 个以上）那一支没造（见「这条腿自己的限度」）。

### K2-b　二次崩溃三个相位（没打中，量过）

二次崩溃不照崩溃注入那样每个相位抽一个，是把那次挂载的写流整条按层 0 全量枚举（相位照 `writable_mount_phase_of_each_write` 的规则重写，状态的相位取它那一段里第一个没持久的原写）。挂载那一段写流一条就是 2 万到 3.7 万态，所以第一次崩溃只取窗口每一段状态区间里的第一个（缩的是历史条数，留下的每个第一次崩溃状态，它的二次崩溃一态不落）：

| 窗口 | 取的第一次崩溃态 | 二次崩溃态 | 取号 / 写行 / 暖机 | 判红 |
|---|---|---|---|---|
| mkmnt | 全部 18 个挂上的 | 5 013 | 144 / 4 551 / 300 | 0 |
| mnt | 每段第一个，9 个 | 201 027 | 72 / 147 555 / 53 391 | 0 |
| fms12 | 序号 0（挂载推了抬 F 到 txg 12） | 28 733 | 8 / 16 395 / 12 329 | 0 |

fms12 序号 0 那一个分三次跑完（`outputs/k2one-fms12-0-a.log` 序号 0–14000、`-b2.log` 14000–28733，`-b.log` 是同一段不带段号的先一次）。合计枚举二次崩溃 234 773 态（每个第一次崩溃态那一条写流的全持久终态不是「挂载途中」，不判），判红 0。

### K2-c　挂载推了抬 F 时，暖机那一次发布的 F 也整段不判（半中：不误红，漏判，量过）

二次崩溃的相位只分三段，暖机与之后推的抬 F 同一段：「    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F）。」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:270`）；挂载推了抬 F 时整个 WarmUp 段的 F 交「不判」：「        (FloorRaisedByTheMount::RaisedTo(_), WritableMountPhase::WarmUp) => {」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2028`）。不判的理由写的是抬 F 之后那一截：「    /// 挂载推了抬 F，二次崩溃崩在暖机那一段（抬 F 那几串接在暖机之后）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-harness/src/mount_transparency.rs:254`）。

fms12 序号 0 那一次挂载的写流（`outputs/k2one-fms12-0-b2.log` 的 `mount_segment` 行）：段 0 取号，段 1–4 写行，段 5–8 是暖机那一次发布（单元 12、记录 2、根 1、系统配置 2），段 9 是抬 F 那一串先写的两槽系统配置，段 10–17 是抬 F 推的两次发布。段 5–8 的 4 095 + 3 + 1 + 8 = 4 107 个二次崩溃态崩在抬 F 的第一个写之前，F 生效值按代码就是 R1 的，可这一判把它们也记成「二次崩溃崩在抬 F 开始之后的 F」：段 5 那一行原样「SECOND[warm_up] seg=5 field_not_judged=二次崩溃崩在抬 F 开始之后的 F count=4095」（`research/prompts/m3-formal-code-r2-opus-model/outputs/k2one-fms12-0-b2.log:57`）。

四句：不分辨「判法会不会误红」那一格（它让判法少判，不让它多红）；判别它的东西系统看得到（`MountOutput::space_admission` 交回了抬 F 那几串的发布，暖机的发布数是格式常量）；满足的是 r5 判决 U5 那一行 F 的前半句「挂载没推抬 F 时 … R2 根里的 F = R1 的 F 生效值」——暖机那一次发布在挂载推抬 F 之前，就它而言挂载还没推；改法见文末 K2-1。这一格是漏判的量，不是误红，按「打中不分辨臂」交主 agent 定要不要收。

### K2-d　拒绝合法的复算与 D23 已定项 14 的单子（没打中）

逐条对过：判法复算（`crates/singlefs-harness/src/mount_transparency.rs:960` 那个函数）与 `.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换` 里「可写挂载读到的更新状态读不出」那一段的五步（有一槽读不出或本池自证不过按真；c_见证 = 全部槽 tail 的最大值，0 不判；所选那一版带末条标志的记录读得出则比计数器；读不出则比计数器等于 c_见证 那条记录的 (实例, txg)；两条都读不出按真）一一对上；只多一句「所选那一版没有时交回假」，单子没写这一格。单子上第三种拒——取号那一刻有一槽读不出——实现报成同一个成员（`crates/singlefs-core/src/mount.rs:4046` 那一处映射），判法的「不在单子上」那一支接不到它。合法历史上量到的 165 次拒全是第一种（撕裂的系统配置槽），复算全判合法。

### K2-e　卸载记号那一格在崩溃注入里走不到（读码；原型补跑 0 红）

崩溃注入的历史里没有卸载一步（`HistoryOperation` 的八种里没有，r5 判决第五节第 3 条已经记了「卸载记号那一格要先把卸载加进历史」），所以「R1 那一版是卸载写的」那一支在接入处一态都走不到。原型 `um` 窗口（第一个文件 → 挂载、正常卸载 → 录一次重挂）补了这一支：24 618 态、24 598 次挂载做成，判红 0（`outputs/k2-um-first.log`）。

## K3　C651（L3）

### K3-a　checker 逐档与实现逐档在手放的槽上结局不同（打中，量过）

装置：`prototypes/r2probe/src/main.rs` 的 `k3slot`、`k3slot0` 两个模式。io_min 8192 的新池（槽距 8192，两块 4 GiB 内存稀疏盘，mkfs 写下的槽 1 在 8192、世代号 1），两块盘的槽 0 整槽写 0，再在每块盘某个偏移上放一份「mkfs 写下的槽 1 改几格、重封整槽校验和」的槽；恢复一侧调 `choose_system_configuration`，checker 一侧调 `chosen_system_configurations`、`system_configuration_slot_readings` 与 `check_pool_image`。对照组 `k3slot0` 不抹槽 0，只把盘 0 的槽 0 换成同样改过的一份（普通读法，不逐档）。

原样输出（`outputs/k3slot.log`、`outputs/k3slot0.log`，每行截到恢复与 checker 两段）：

| 放的槽 | 恢复（实现逐档） | checker 逐档 | 普通读法对照（槽 0 换成同样改过的） | 分叉在哪 |
|---|---|---|---|---|
| S = 17、自述槽距 = 所在偏移 12288、世代号 9 | 整池拒（`RootRingSlotsPerRegionOutOfRange`） | 两盘都择 8192 处世代号 1，`checker_red=[]` | 同样分叉：恢复整池拒，checker 全绿 | 两条读法都分叉 |
| S = 17、自述槽距 8192、放在 12288 | 整池拒 | 两盘择 8192、全绿 | 同上 | 两条都分叉 |
| S = 17、自述槽距 8192、放在 4096（在本池槽 1 之前） | 整池拒 | 两盘择 8192、全绿 | 同上 | 两条都分叉 |
| 在飞上限 + 1、自述槽距 8192、放在 12288 | 整池拒（`SystemConfigurationValueRefused`，在飞上限不是环长现算的那个） | 两盘择 8192、全绿 | 两边都判坏：恢复整池拒，checker 报 I-8.1 | **只在逐档路上分叉** |
| 在飞上限 + 1、自述槽距 = 12288、世代号 0 | 整池拒 | 两盘择 8192、全绿 | 两边都判坏（checker 报 I-8.1） | **只在逐档路上分叉** |
| 在飞上限 + 1、自述槽距 = 12288、世代号 9 | 整池拒 | 两盘择 12288 处世代号 9，报 I-8.1 | 两边都判坏 | 不分叉 |
| R = 4、自述槽距 = 12288、世代号 9 | 择 12288 处世代号 9 | 两盘择 8192 处世代号 1、全绿 | 同样分叉：恢复择世代号 9，checker 择世代号 1 | 两条都分叉 |
| R = 4、自述槽距 8192、放在 12288 | 择 8192 | 择 8192 | — | 不分叉 |
| incompat 位不认得 + 格式版本 2 | 择 8192（挂得上） | 两盘择不到，报 I-7.13 | 同样分叉（checker 报 I-7.13，恢复照择） | 两条都分叉 |
| incompat 位不认得 + 加密类型 1 | 择 8192 | 择 8192、全绿 | — | 不分叉 |
| 只改格式版本 2 | 整池拒 | 择不到，报 I-7.13 | — | 不分叉 |
| 世代号相同、自述槽距 = 12288 | 择 8192 | 择 8192 | — | 不分叉（平局取偏移小两边同） |
| 不放（对照） | 择 8192 | 择 8192、全绿 | — | — |

三种病根，按代码现查：

1. **在飞上限（I-8.1 三个字段）只在逐档路上分叉，是这一批新出的。** 实现逐档先把一槽交给 `mountable_slot_or_refusal`（「reader.read(device, DeviceOffsetInBytes(candidate_spacing), slot_bytes),」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/recovery.rs:982`）），那里头一步就判值收不收（「system_configuration_values_this_reader_accepts(&bytes, &system_configuration)」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/recovery.rs:921`），I-8.1 三条在它最后：「journal_ring_geometry_fields_recompute_from_the_ring_length(slot, sizes.journal_ring_bytes)」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/recovery.rs:840`）），带着 `?` 整池拒，之后才核偏移（「if recorded_spacing != candidate_spacing {」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/recovery.rs:991`））。checker 逐档读成 `Usable` 之后先核偏移（「if geometry.slot_spacing != candidate_spacing {」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker/src/image.rs:597`））、再按世代号只留一槽，I-8.1 只在留下的读数上判（GPU 判器同：「if (slot.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(slot)) {」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3421`））。被跳过、被比下去的那一槽带着坏的 I-8.1 字段，checker 一个字都不报。
2. **S 越界、R 越界：两条读法都分叉，病根在两边对这两个字节的分类不同，不是逐档改出来的。** checker 写明「R、S 越界照旧是这一槽不可择。槽距先判：它是根槽宽的上界。」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker/src/image.rs:304`），`RootRingSlotsPerRegionOutsideTheFormatInterval` 不在「读者不收之值」的表里（`crates/singlefs-checker/src/image.rs:490`）；实现解槽时 S 越界整池拒（「.map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/system_configuration.rs:569`）），R 那一字节不读（「region_reader.skip(1 + 1 + 24 + 8);」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-core/src/system_configuration.rs:601`））。
3. **incompat 不认得 + 格式版本不对：两条读法都分叉，病根在判的先后。** checker 先判格式版本（判成「不收之值」、I-7.13），实现先判 incompat（判成不可择、接着找）。

打中之后的四句（只对第 1 种，第 2、3 种按「打中不分辨臂」记到共用前提上）：

1. 分不分辨臂：分辨。被判的是 r6 判决 V1 那一行「11-5 的 checker 已同处置」——同一组输入在普通读法上两边都判坏、在逐档读法上 checker 全绿而实现整池拒，分开的正是 11-5 写的逐档那一段。
2. 系统当时看不看得到判别它的东西：看得到。checker 逐档那一档已经把这一槽读成 `Usable` 连同它的几何，丢掉它的是自己的偏移判与世代号比。
3. 满足的是判据字面的哪一个分句：r6 判决 V1 那一行「条款句补两句（照实现与 D22 已定项 16 第 5 句）：逐档读到自证过而池级字段越界的槽整池拒」。I-8.1 三个字段实现按「读者不收之值」整池拒，它们算不算该句的「池级字段越界」字面没写——D22 已定项 16 第 5 句点名的是槽距与 `physical_block_size`，I-8.1 三条是 m3-inv-a 判决加进实现那一张表的。S 越界那一种则是字面里的（实现的文档写「S 是池级字段」），但它不分辨逐档。
4. 跑前条款给的改法：正文第五节只给「改法进下一批」。我能提的改法见文末改法表 K3-1、K3-2。

### K3-b　mkfs 写零之后的钉值：漏列两份 exact 复跑产物（打中，量过一半）

量过的一半：`k3mkfs` 模式照 E142 / E170 那一条路（`singlefs_harness::scenario::e142_parameters(512, 512)`、两块 4 GiB 内存稀疏盘、`run_new_pool_file_creation`）在冻结副本的拷贝上数 mkfs 占的步，原样：

「name=k3_mkfs mkfs_operations=25 zero_fills=10 barriers=4 zero_fill_ranges=d0@0+1048576,d0@1048576+32768,d0@4194304+32768,d0@7340032+32768,d0@16777216+805306368,d1@0+1048576,d1@1048576+32768,d1@4194304+32768,d1@7340032+32768,d1@16777216+805306368」（`research/prompts/m3-formal-code-r2-opus-model/outputs/k3mkfs.log:1`）

推的一半（没跑实验复跑，它是重型）：E170 的两份 exact 复跑产物钉着改之前的 23。

- 「E7RESULT name=s1 stream=new_pool_file_creation check=new_pool_file_creation_mkfs_operations_equal_build_pool local=23 crates=23 holds=true」（`research/results/e170-o3-spec-executor-2026-10-02-s1-export.out:17`）
- 「E7RESULT name=bad_s1 stream=new_pool_file_creation check=new_pool_file_creation_mkfs_operations_equal_build_pool local=23 crates=23 holds=true」（`research/results/e170-o3-spec-executor-2026-10-02-s3-bad-states-export.out:9`）

两个数都出自 crates 的 mkfs：装置经 `#[path]` 引 harness 的 `tests/common`（「#[path = "../../../singlefs-harness/tests/common/mod.rs"]」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs:27`）），一边是「let mkfs_operation_count = formatted.mkfs_operation_count;」（同文件第 266 行），一边是「let reference = common::build_pool("e170-first-stream-reference");」（同文件第 620 行）。harness 那一份 `build_pool` 的 mkfs 已经钉成 25（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 第 330 行的注释「mkfs 23 → 25 步」，11-5 改的、harness 档绿）。所以这两行复跑出来是 `local=25 crates=25`，`replay.sh` 里这两行按 exact 比（E170 那几行的模式栏是 `exact`），提交时实验复跑红。

11-5 报告的漂移表（`/tmp/claude-1000/m3-formal-11-5/report.md` 第 90–111 行）列了 E142 第 734 行、E158、崩溃注入写下标、故障注入注释，没有 E170（那份报告里 `E170`、`e170` 0 命中）。checker 档那一句「grep `12+1+1+1+4`、`zero_fill×8`、`4114`、`1_610_925_568`、`mkfs 23`、`八次清零` … 零命中」管的是 crates 的源码，管不到 `research/results/` 里的产物行。

四句：分辨的是「checker 档推『从 mkfs 之后起枚举、不变』」这一句的射程（它对层 0 与崩溃放量成立——那两处都按 `mkfs_operation_count` 切，见 K3-b 末尾「没打中的」——对实验产物不成立）；系统看得到（产物里是字面 23）；满足的是正文 K3 格「录制流多出的步对层 0 与崩溃放量钉值的影响」那一分句的「钉值」；改法：E170 两份产物随 E142 一起重出（同一次实验复跑），或者把 `mkfs_operations_equal_build_pool` 那一检查的打印改成只打 `holds=`（推的）。

没打中的（读码）：层 0 与崩溃放量的钉值不随 mkfs 变——崩溃注入的候选段只取 mkfs 之后的写（`draw_crash_points` 按 `marks.after_make_filesystem` 滤）；崩溃放量的点按 `mkfs_operation_count` 起算（`CrashPointRecorder::record_span`）；崩溃放量的复用键里的父镜像摘要只数写过的扇区，而内存盘的写零是删扇区（`SparseDevice::zero_fill` 删掉区间里的扇区），mkfs 之后的基镜像在写零前后逐扇区相同。

### K3-c　GPU 判器 `crash_judge_checker.wgsl` 那一份（没打中，读码）

逐档函数与 CPU 那一份同序同判：4096 起、每 512 一档，读到带不收之值的那一档立即交回（「if (s.reading == 1u) {」），只收可用且自述槽距等于所在偏移的（「if (s.reading == 0u && s.spacing == candidate) {」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:220`）），世代号严格更大才换，一档可用的都没有时见过自证过而不可择的就交回那一类。槽的分类（`checker_read_system_configuration_slot`）R、S 越界也是「不可择」，与 CPU 的 checker 同，所以 K3-a 那几种分叉 GPU 与 CPU 一起分叉（推的，没在卡上跑）。

另一处是 GPU 读得到什么：GPU 只读判器表里的落点，系统配置的落点每盘只有偏移 0 与基镜像槽 0 记的槽距那两处（`crates/singlefs-checker-tier/src/crash_judge_tables.rs:438` 那一处 `push`），逐档读到别的偏移都是「没有这一片」。合法流上 [4096, 1 MiB) 里只有本池那一份槽 1，正好在基镜像槽距那一处，GPU 与 CPU 逐档同值（推的）；带手放残留的镜像上 GPU 看不见残留、CPU 看得见。GPU 判器只判崩溃放量的流（都从 mkfs 起步），这一差在它的射程里走不到（推的）。

## K4　第一轮改法与装置（L4、L5）

### K4-a　② 借槽距与 ③ 逐档分不分得开（core 一侧分得开；checker 一侧分不开，量过）

11-11 报告推「把 ② 整个删掉、借不到时落到 ③ 逐档，这条用例大概不红」（`/tmp/claude-1000/m3-formal-11-11/report.md` 第 68 行，交主 agent 的观察）。我在冻结副本的另一份拷贝上把两边的 ② 各删一次（只让 `.or(…)` 那一项恒为 None，落到 ③），各自编、跑碰系统配置择槽的 53 个 harness 档测试二进制（`--no-fail-fast`；清单是 `crates/singlefs-harness/tests/` 里提到 `choose_system_configuration`、`system_configuration_slot_readings`、`chosen_system_configurations`、`slot_one`、「槽 1」的文件，`outputs/k4-targets.txt`）：

| 变异 | 补丁 | 53 个二进制合计 | 红的那一条 |
|---|---|---|---|
| core：槽 0 不可择的盘不借，落到逐档 | `probes/mut-core-borrow.patch` | passed 527 failed 1 ignored 62 | `a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching`（盘 0 择到 8192 处世代号 5 的旧槽：`left: (DeviceIdentity(0), 8192, 5)`、`right: (DeviceIdentity(0), 4096, 1)`） |
| checker：槽 0 无效的盘不借，落到逐档 | `probes/mut-checker-borrow.patch` | passed 528 failed 0 ignored 62 | 没有 |

合计行是 `outputs/k4-core-borrow.log`、`outputs/k4-checker-borrow.log` 里 `test result` 行用 awk 求和的。

- core 一侧：11-11 的推测不成立。那条用例在 8192 处手放了一份世代号 5 的旧槽，借来的 4096 读不到它、逐档读得到，正好是 11-11 自己写的「别的档上有世代号更大的一槽」那种取样点；`crates/mutations.tsv` 第 917 行「实审 A2a 第 37 条：槽 0 不可择的盘不借池里别的盘槽 0 记的槽距」点名的就是它，替换文是 `.or(None)`，11-5 之后它的落点从「照 4096」变成「落到逐档」，照样红。
- checker 一侧：分不开。那条用例只问恢复，不问 checker；`crates/mutations.tsv` 第 979 行（checker 那一份「不借、照 4096 找槽 1」）的替换文是把借来的槽距换成 4096，11-5 之后「不借」不再落到 4096 而是落到逐档，这一行管不到「删掉 ②」这一种。合法历史上 ② 与 ③ 结局相同（mkfs 写零之后 [4096, 1 MiB) 里只有本池那一份槽 1，推的），能分开的只有手放旧槽的镜像。
- 四句：分辨的是「checker 的 ② 有用例罩」这一句；checker 读到了那份旧槽、看得到；满足 `.claude/rules/mutation-sampling.md` 第三类的形态（取样点上两档同值）；改法：在同一条用例里对同一份镜像再问一次 `chosen_system_configurations`（盘 0 应择 4096 处世代号 1），`crates/mutations.tsv` 补一行 checker「不借、落到逐档」点名它（推的，没实现）。

只跑了这 53 个；harness 档其余约 60 个二进制在 checker 变异下没跑（见「这条腿自己的限度」）。

### K4-b　E165 r3 两段装置（没攻）

没读、没跑。

## K1　RefFS 包（L1）

没攻，只读了包本身，记三处与 H1 定案不同的地方（推的，调用点没接，量不到）：下界只认返回成功的 fsync（「operation.kind == EnteredKind::Fsync && operation.outcome == EnteredOutcome::Succeeded」（`/tmp/claude-1000/m3-formal-code-r2/tree/crates/singlefs-reference-filesystem/src/lib.rs:209`）），H1 定的是发布调用、fsync、回退都抬；回退目标 m 只在「已确认的发布」里数（`crates/singlefs-reference-filesystem/src/lib.rs:282` 那一处 `chain`），H1 要覆盖每一个写根的入口与恢复读出的那一版；恢复之后以读出内容重新起头（`crates/singlefs-reference-filesystem/src/lib.rs:420`），读出的那一版不进 m 的编号。

## 改法表（全部只在我的模型上量过或推的，被攻过零轮）

| 改法 | 修哪一格 | 在打中的那几格上 | 量过 / 推的 |
|---|---|---|---|
| K3-1　checker 逐档读到一槽 `Usable` 时，先按实现那张表判 I-8.1 三个字段（与 I-7.13 那一类同处置：交回「带不收之值」、整池不作保），再核偏移、比世代号 | K3-a 第 1 种 | 两行「在飞上限 + 1」的逐档分叉收掉：checker 报违例，实现整池拒 | 推的 |
| K3-2　S 越界、R 越界、incompat 与格式版本的先后：checker 与实现统一成一边（实现那一边：S 越界整池拒，R 读了判；或 checker 那一边：S 越界不可择）；要连 D22 已定项 16 第 5 句一起改 | K3-a 第 2、3 种 | 普通读法与逐档都收；改哪一边是条款问题 | 推的 |
| K3-3　E170 两份 exact 产物（s1 export、s3 bad-states export）随 E142 一起在提交时的实验复跑里重出，11-5 漂移表补这两行 | K3-b | `local=23 crates=23` → 25 | 推的（25 是量过的） |
| K4-1　`core_review_tree_table_duplicates_and_slot_one_search.rs` 那条借槽距的用例在同一份镜像上加问 `chosen_system_configurations`（盘 0 应择 4096 处世代号 1），`crates/mutations.tsv` 补「checker 不借、落到逐档」一行点名它 | K4-a | checker 删掉 ② 那一种变异从 108 个二进制全绿变成红 | 推的（全绿是量过的） |
| K2-1　二次崩溃的相位把「暖机」与「挂载推的抬 F」拆开：抬 F 的第一个写起才算 `SomeMayHaveLanded`（界可以从 `MountOutput::space_admission` 交回的那几串发布在挂载写流里的位置取） | K2-c | fms12 序号 0 那一次的 4 107 个暖机态从「F 不判」变成按「不变」判 | 推的（4 107 是量过的） |

## 没打中的形状

- 挂载透明误红：11 个窗口的第一次崩溃层 0 全量 147 806 态（宽 384 的九个、宽 256 的两个，含回退、回退前重挂、抬 F、冷启动恢复后重挂、零单元发布、正常卸载后重挂、前缀关准入后挂载自己推抬 F），两个动作各判一次，0 红；二次崩溃 234 773 态 0 红。被抛弃的时间线（`CrashRecoveryAbandoningTheNewestRoot`）在 C554 乙之后那一步被拒，造不出（`ab`、`ab2`、`abf`、`abf2` 四段历史那一步都报 `NewerStateStillUnreadableAfterOneReread`，`outputs/sizes2.err` 里四段历史各自的 `step 1:` 或 `step 2:` 那一行）。
- 拒绝合法：165 次拒全是 N-配置，复算全判合法；单子上另两种拒在不带故障的历史上没走到。
- checker 与实现逐档：平局（世代号相同取偏移小）、自述槽距不等于偏移的照跳、只改格式版本的整池拒、R 越界而槽距不等于偏移、incompat + 加密类型，五种两边同结局（`outputs/k3slot.log`）。
- 层 0 与崩溃放量的钉值：不随 mkfs 写零变（读码，见 K3-b 末段）。
- 没试的：多记录发布（点名项 68 个以上，水位那一格唯一可能误红的形状，按 K2-a 的推法）；带故障的历史（射程之外）；根环转过一圈之后的窗口（`wrap30` 窗口层 0 全量 1 048 588 态，超过单次 20 万态的上限，没跑）；第一个文件、覆盖写那两种窗口（`first`、`ow` 都是 1 048 588 态，同上没跑）；fms12 其余 9 个推了抬 F 的第一次崩溃态的二次崩溃（第一次一口气跑被后台时限停掉，没有输出，见限度）。

## 这条腿自己的限度

- 全部数都在冻结副本的拷贝上量（`sha256sum -c` 清单 241 行无输出、退出 0）；崩溃注入、层 0 用例、E142 / E170 复跑、GPU 判器、变异整表一样没跑。挂载透明的接法是我照 `crash_injection.rs` 重写的一份（崩溃注入那几个函数私有），判法本身是入库那一份；我的接法与入库接法不同的两处：第一次崩溃照层 0 全量枚举（入库是每段抽一个）、二次崩溃照层 0 全量枚举（入库是每个相位抽一个）。
- 原型跑过的态数（`closed_form_state_count` 与含撕裂第三态的层 0 全量，逐窗口在 `outputs/sizes*.log`）：跑完的第一次崩溃 147 806 + 先导 3 600 + 二次崩溃那几次挂的 27，二次崩溃 234 773（另有 fms12 序号 0 的两段先导 600 + 14 733），合计约 40.2 万态。停掉、没有产物的三次：`mnt 0 40 all`（那时的二进制没有二次崩溃的上限，跑了约 4 分钟我停的）、`mnt 0 20511 all`（编译被内存包装拒 252 之后跑的是旧二进制，约 4.5 分钟我停的）、fms12 一口气跑 18 个第一次崩溃态的二次崩溃（预算 19 万态，到后台 30 分钟时限被停）；三次各自跑到多少态没有输出，按先导的速度估在 20 万到 30 万态一次（估计）。单次调用 20 万态的上限：`mnt` 那一次二次崩溃 201 036 态，超了 1 036；上面停掉的三次按估计也超过，是我的装置先没设上限造成的。
- 撕裂第三态只照层 0 的认法（基镜像上罩住旧内容的原地覆写取三态）；崩溃时刻归属、多记录发布、带故障的读写都没造。
- K3-a 的槽都是手放的（改字节、重封整槽校验和）；它说明两份读法在这些输入上结局不同，不说明这些输入在合法历史上出现得了（mkfs 写零之后 [4096, 1 MiB) 只剩本池的槽 1，推的）。
- K4-a 只跑 harness 档 108 个二进制的默认用例（各自 `ignored` 的 62 + 1 条没跑）；checker 档里引系统配置择槽的用例（层 0、坏盘输入）没跑。
- 内存包装没跑成的：`sizes` 第一次 252（整机排不上，3G 降成 1G）；编译一次 251（3G 超峰值两倍，改 `auto`）、一次 252（同会话已有一条在跑，加并行理由）；`sizes rfx…` 一次 251、一次 250（`auto` 取的峰值是别的历史的，改 1500M 加理由）。都按包装给的出路重跑过，命令一行都没跑的那几次不算结果。

## 没做什么

- 没判辩方那几格，没替主 agent 采纳；副本上的数不算入库装置上的数。
- 没跑任何重型测试（checker 档测试二进制、层 0、崩溃注入、E142 / E170 复跑、Verus、门禁）；K3-b 里 E170 复跑会红是推的。
- K1 只读了包；K4-b（E165 r3 两段装置）没读没跑；GPU 判器没在卡上跑。
- 没改仓里任何文件；只写了这份报告与模型目录。
