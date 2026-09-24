# m2-presumed-clauses-r1 云端攻方（Opus）：K2（P3）与 K4（P6）

2026-09-23，时刻按 UTC 记（东京 JST = UTC+9）。前一条同立场的腿没落任何文件，这一份从头做。禁读的两份没读。

## 复跑

仓副本：`rsync -a --exclude target --exclude .git` 自工作区（HEAD 3b60f09 加当时没提交的改动）拷到 `/tmp/claude-1000/presumed-r1-opus/repo/`。副本上的数都是**副本装置**上的数，不是入库装置上的数。

- 副本源文件 sha256（拷出时）：`mount.rs` 98f1bd68dd9431fd1f2da388976e50791c0079e28b457332cb11e8da754c1858（与工作区当时同）；`recovery.rs` 8ea6074d34207b9fc719eb5f1e63c4575d6673aeea8140e0a15b1f31d81fef84（工作区随后又多了 `replay_journal` 里三行注释，逻辑没动）。
- 副本改了三处，补丁在模型目录 `copy-side/`：`checker-compile-fix.patch`（工作区的 `singlefs-checker/src/walk.rs` 当时编不过：`member.is_commit` 字段不存在，副本改成读 `commit_marker`，只为能编译）；`mount-arms.patch`（两个只在副本上的环境变量臂：`PRESUMED_R1_WARM_UP_CAP` 截暖机次数、`PRESUMED_R1_ROW_TXG_JUMP` 把写行 txg 往后挪）；`recovery-arms-full.patch`（`PRESUMED_R1_NAMED_ANY` 点名单元任一份验过即过、`PRESUMED_R1_CHAIN_HEAD=max|stop|max-stop` 三条链首臂）。不设环境变量时副本的行为与工作区相同。
- 新用例：`copy-side/presumed_r1_opus_k2.rs`、`copy-side/presumed_r1_opus_k4.rs`，放进副本的 `crates/singlefs-harness/tests/`。

```sh
cd /tmp/claude-1000/presumed-r1-opus/repo
export CARGO_TARGET_DIR=/tmp/claude-1000/presumed-r1-opus/target TMPDIR=/tmp/claude-1000/presumed-r1-opus/tmp
nice -n 19 cargo test --release -p singlefs-harness --test presumed_r1_opus_k2 --test presumed_r1_opus_k4 --no-run
bash /tmp/claude-1000/presumed-r1-opus/run-k2-arms.sh          # K2 七条臂并行，约 13 分钟（UTC 15:30–15:43）
for arm in "" max stop max-stop; do env ${arm:+PRESUMED_R1_CHAIN_HEAD=$arm} nice -n 19 target/release/deps/presumed_r1_opus_k4-* --nocapture --test-threads=1; done
rustc -O --edition 2021 <模型目录>/presumed_r1_opus_model.rs -o /tmp/claude-1000/presumed-r1-opus/model && /tmp/claude-1000/presumed-r1-opus/model
```

模型目录 `research/prompts/m2-presumed-clauses-r1-opus-model/` 每个文件的 sha256：

```text
2fdd6769fe304bd30bc7e552d076b110cba7bc2c65b97e3f2627b0e4142453d3  ./copy-side/checker-compile-fix.patch
0855986ad73b55313e870c535e86d5f040e3109a423be4222ae5d2b74a60d807  ./copy-side/mount-arms.patch
21e2550b1b08d6a93832a869cd223089e5901b1214c329eb88279e620cbba42e  ./copy-side/presumed_r1_opus_k2.rs
5861006985e532f823cfb4b66454d5cc78a13896909884fa18947c2cfc463c87  ./copy-side/presumed_r1_opus_k4.rs
658a536eddd7e24e48ccbd035ef38450bbedbf35928e22ad4926e04b467584ab  ./copy-side/recovery-arms-full.patch
4801872f959aa9b8c009038f3a36536526cecd2652fd2faae9ef48eeceb900ed  ./copy-side/run-k2-arms.sh
52bd0970bc7c5fc2a3297badeec495e43d772cafb7716ecb225aa547c22eee87  ./logs/k2-A1-today.log.gz
d8721673483ea79ea86dffdf0aa51f9b0e56d2dc1607ffbf669641d21b68475f  ./logs/k2-A2-any.log.gz
2e4874dea03eab93d7be70fb5156d18bd02411d820efdd6106bd2517ca676f82  ./logs/k2-A3-any-cap0.log.gz
b17e0e39b738433cd41c840db673bfa94cc7b3f2999938d55ace3be10d805ed5  ./logs/k2-A4-any-cap1.log.gz
04cad07a47433ac89390f83d464693b6b3028bbe6b903d8255c6b170d5c7ec2e  ./logs/k2-A5-any-jump.log.gz
43db3f2212887a70f9cbc9b33a82ceb69afd5837ef222311ddf8188578731c19  ./logs/k2-A6-cap0.log.gz
739ee8642b2cfb375bb2b5d78247ee1fd197cff09baa1c873b8beaf3d0bebfce  ./logs/k2-A7-jump.log.gz
71f4f6fb6c65166fde4bef0abd72b3e632b6fcde0d50f117e2d7228b05938bdd  ./logs/k4-max.log
96a70844b58f2f642ffbfc8be26469d6ed0d17132c18793e28ed171499826413  ./logs/k4-max-stop.log
e03706b5dc50700dd3d9e32ef0e9406c6cea72bdfd23d06eccd2badea0be2c9f  ./logs/k4-stop.log
6e0660657b3514d2a19f6b442228593e7a5503609cc8233dcf7c53d2520ad77c  ./logs/k4-today.log
de343fbbd4882dacbcf88456fac96a1e1cf4484f189992854070e58529ede041  ./logs/model.log
da610ee9b10f8bf6f43de61cc20e83b02f123f6be940d254765d069a9619404a  ./logs/run-k2-arms.out
9ff6a7d3bdfbd9be6533f39e53a12098097f5c70e9e1fae398a918c38f34ec26  ./presumed_r1_opus_model.rs
```

`presumed_r1_opus_model.rs` 只用 std，不引 `crates/` 一行，是第二条路：K2 的余数算术与 K4 的链首控制流另写一遍，与副本装置对数。开跑前 `ps` 只看到另一个会话的 `cargo test --release --bin e142-first-txn-dry-run`（变异用，`/tmp/singlefs-mutate-target`），没有性能测量在跑，没等锁。

## 各格判定一览

| 格 | 判定 | 打中的历史（最少故障） | 区不区分候选 | 细节 |
|---|---|---|---|---|
| K2 不够 | **构造不出** | 在副本上扫了 44 次后续挂载 / 回退 / 回退之后再挂、18 次只重开、11332 个「挂载中途崩了」的崩溃状态再挂：今天的现算每次都覆盖到两块盘；点名单元验证放成「任一份」之后，丢任一块盘都读回刚确认的那一版（0 次丢） | — | K2 第一、二节 |
| K2 多推 | **打中（零故障）**：写行 txg ≡ 2 (mod 3) 时推 2 次，第 1 次落在已覆盖的盘 0 上，是白推；只重开不写时稳态每次挂载都落在这一档 | 零故障，最常见的「A → B → 重开」就是（txg 5 → 6 白推），在库里的 `second_transaction_step_three_second_instance.rs` 文件头也自己写着「txg 6 落盘 0 白费」 | 区分：写行 txg 往后挪的候选（`jump`）0 次白推、同样安全；**但它只对普通挂载合条款**，回退那一句写的是「= max + 1」 | K2 第三节 |
| K2 旁见（不计进 K2 判定） | **今天的恢复在丢一整块盘时，把刚确认的那一版丢掉，暖机推几次都一样** | 零崩溃、一个故障：发一版数据（fsync 已返回）之后，丢掉它根槽所在那块盘 | **不区分**：七条臂里凡是点名单元验证要两份都过的，每次挂载都丢一次；病根在 `replay_journal` 要求点名单元的两条位置条目**都**验得过 | K2 第四节 |
| K4-a | **打中（零故障）**：锚点读得出，但所选根那次发布有 N ≥ 2 条记录时，今天拿同 txg 的**第一条**当锚，一条都不施加 | 零故障：P0 两条记录、根 10 落盘；P1 两条记录都落盘、根 11 没落盘就崩 | 区分：`max` 臂（锚取同 txg 里 jsn 最大的）196 格全修 | K4 第二节 |
| K4-b | **打中（2 个介质故障加一次崩溃）**：所选根那条读不出、下一次发布 N ≥ 2 条时，「只接 txg = 根 + 1 那条」会跨过断号接上去（多接） | P0 一条记录两份都读不出；P1 两条记录，崩溃时只有第二条落盘 | `stop` 臂不多接，但同一支 147 格少施加；今天的格式下**判别子观测不到** | K4 第三节 |

K2 这一格：「推到本实例的根覆盖每块盘为止」这条规则在三种起点上都没有「不够」，但写成条款时带着「从 max + 1 逐个加一」这个起点，就把 1/3 的挂载（只重开时是每一次）多推一次空发布写进了条款。K4 这一格：**站不住**，并触发跑前条款第 3 行。

## K2（P3）　后续挂载与回退之后暖机几次

### 一、装置：三种起点，用户动作放开扫

`copy-side/presumed_r1_opus_k2.rs`，四条用例，每条挂载打一行：写行 txg、暖机 txg 与各落哪块盘、白推几次（落在已被本实例的根覆盖的盘上）、最后覆不覆盖两块盘、这次挂载写了多少字节 / 几道屏障 / 几次 FUA。每次挂载之后**再发一版数据**（`publish_overwrite`，fsync 已返回的那一版），然后冷恢复两种故障：F1 = 只有那一版的根槽读不出；F2 = 丢掉盘 0 或盘 1（那块盘换成一块全 0 的盘）。判「读回的是不是那一版」。暖机要买的就是这一件（`.claude/kb/decisions/16-发布语义.md` 第 174 行、第 187 行，附录 A3、A4）。

| 用例 | 扫了什么（由用户决定、放开扫的那几步） | 挂载数 |
|---|---|---|
| `k2_subsequent_mounts_sweep` | 实例 1 在第一个文件之后再发 0–5 版再关（写行 txg 覆盖三种余数），之后「重开、发一版」连做 4 轮 | 24 |
| `k2_remount_only_chain` | 同上前缀 0–2 版，之后只重开、不写，连 6 轮 | 18 |
| `k2_rollback_sweep` | 实例 1 发到 txg 3 + k（k = 1–5），回退到 (1, 3) 或 (1, 3 + k − 1)，发一版、查；再正常重开一次、发一版、查 | 20 |
| `k2_crash_during_mount_then_remount_sweep` | 一次可写挂载（前缀 0、1、2 版）或回退（前缀 2、3 版）的写按屏障与 FUA 切段；每个段前缀，加段内全部真子集（段宽 ≤ 10，本次每段都 ≤ 10）当一个崩溃状态；崩溃状态上再可写挂载、发一版、查 | 11332 个崩溃状态 |

只固定前缀与故障；「挂载之后发几版」在前两条用例里放开扫了，「回退到哪」扫了两个目标，崩溃点取遍段前缀加段内子集。

### 二、「不够」：构造不出

七条臂的总账（`logs/run-k2-arms.out` 原样，只摘 SUMMARY 行）：

```text
A2-any exit=0
K2-SUMMARY [cap=- jump=-] crash-then-remount states=11332 remount_errors=0 disk_loss_failures=0 mounts_by_row_mod3=[3095, 5135, 3102] white_by_row_mod3=[0, 0, 3102]
K2-SUMMARY [cap=- jump=-] rollback mounts=20 disk_loss_failures=0
K2-SUMMARY [cap=- jump=-] subsequent mounts=24 white_publishes=2 disk_loss_failures=0
A3-any-cap0 exit=0
K2-SUMMARY [cap=0 jump=-] crash-then-remount states=5170 remount_errors=0 disk_loss_failures=1041 mounts_by_row_mod3=[2061, 2068, 1041] white_by_row_mod3=[0, 0, 0]
K2-SUMMARY [cap=0 jump=-] rollback mounts=20 disk_loss_failures=8
K2-SUMMARY [cap=0 jump=-] subsequent mounts=24 white_publishes=0 disk_loss_failures=8
A4-any-cap1 exit=0
K2-SUMMARY [cap=1 jump=-] crash-then-remount states=10305 remount_errors=0 disk_loss_failures=0 mounts_by_row_mod3=[3095, 4115, 3095] white_by_row_mod3=[0, 0, 3095]
A5-any-jump exit=0
K2-SUMMARY [cap=- jump=1] crash-then-remount states=10305 remount_errors=0 disk_loss_failures=0 mounts_by_row_mod3=[5170, 5135, 0] white_by_row_mod3=[0, 0, 0]
K2-SUMMARY [cap=- jump=1] rollback mounts=20 disk_loss_failures=0
K2-SUMMARY [cap=- jump=1] subsequent mounts=24 white_publishes=0 disk_loss_failures=0
```

`disk_loss_failures` 这个数 = F2 丢了几次 + 1000 × F1 丢了几次；七条臂所有汇总行都小于 1000，所以 F1（只坏数据那一版的根槽）在七条臂上都是 0 次丢。（崩溃用例里的状态数随臂变：暖机次数不同，挂载那条录制流的段数就不同。）

- **A2（今天的暖机现算，点名单元任一份验过即过）**：三种起点加崩溃再挂，一次都没丢。
- **阳性对照 A3（一次暖机都不推）**：只在写行 txg ≡ 2 的挂载上丢，而且那一档每次都丢：崩溃用例 1041 = `mounts_by_row_mod3` 的第三格 1041；后续挂载与回退各 8 次，逐行核过全是 `row_mod3=2`（`logs/k2-A3-any-cap0.log.gz` 里这两条用例的 44 行：`row_mod3=2` 16 行、每行 `F2_failures=1`；`row_mod3=0` 14 行、`row_mod3=1` 14 行，全是 `F2_failures=0`）。所以这道检查分得出「推得不够」。
- 纯 std 模型（第二条路）逐余数对上：`logs/model.log` 原样两行——

```text
K2-MODEL arm=Today named_any=true by_first_txg_mod3: mounts=[9, 9, 9] publishes_per_mount_total=[18, 18, 27] white=[0, 0, 9] disk_loss_LOST=[0, 0, 0]
K2-MODEL arm=Cap(0) named_any=true by_first_txg_mod3: mounts=[9, 9, 9] publishes_per_mount_total=[9, 9, 9] white=[0, 0, 0] disk_loss_LOST=[0, 0, 9]
```

卡在哪一步：两盘、区域归属写死 0 / 1 / 0、区域 = txg mod 3（`16-发布语义.md` 第 147 行，附录 A1）之下，连续三个 txg 必有一个落盘 1、两个落盘 0，所以「逐个加一推到覆盖两块盘」至多推 2 次，「至多区域数（3）次」那道上限在第一版几何里永远碰不到；本实例的根每次都在两块盘上各有一条，而新实例的写行 txg 取 `max(根环, 环里记录) + 1`，比环里任何一条旧根都大。要「不够」，得让某块盘在暖机里一次都轮不到——第一版几何下不存在；三盘或别的归属 mkfs 当场拒（`make_filesystem.rs` 断言 2 块盘、`RegionDevicesNotTheFirstVersionLayout`）。

### 三、「多推」：打中，零故障

今天的现算从 `start.first_txg = max + 1` 起逐个加一。写行 txg ≡ 2 时写行落盘 0、下一个 txg ≡ 0 仍落盘 0、再下一个 ≡ 1 才落盘 1——中间那次空发布没有覆盖任何新盘。副本原样（`logs/k2-A1-today.log.gz`）：

```text
K2 [cap=- jump=-] chain extra=0 round=2 inst=4 chosen=(3,7) row=8@d0 row_mod3=2 warm=[9@d0,10@d1] warm_count=2 white=1 covered_both=true mount_bytes=517632 mount_barriers=7 mount_fua=3
```

同一条链里写行 txg ≡ 0 的挂载（`logs/k2-A7-jump.log.gz`，副本上 `jump` 臂）：

```text
K2 [cap=- jump=1] chain extra=0 round=2 inst=4 chosen=(3,7) row=9@d0 row_mod3=0 warm=[10@d1] warm_count=1 white=0 covered_both=true mount_bytes=369664 mount_barriers=5 mount_fua=2
```

白推的那一次空发布在副本上量到：多 517632 − 369664 = 147968 字节写、多 2 道屏障、多 1 次 FUA，并占掉根环 24 槽里的一槽（`16-发布语义.md` 第 193 行把「空发布占根环槽位，把旧的非空根更早挤出环」列为暖机的代价，附录 A4）。

多常见：
- 崩溃再挂那 11332 个状态里写行 txg ≡ 2 的 3102 个，白推恰好 3102 次（每个一次）。
- 只重开、不写的链进入稳态之后**每一次**都落在 ≡ 2：写行 ≡ 2 → 暖机 ≡ 0、≡ 1 → 下一次写行 = ≡ 1 的那个加一 = ≡ 2。原样（`logs/run-k2-arms.out`）：

```text
K2-CHAIN [cap=- jump=-] extra=0 publishes_per_mount=[2, 2, 3, 3, 3, 3]
K2-CHAIN [cap=- jump=-] extra=1 publishes_per_mount=[3, 3, 3, 3, 3, 3]
K2-CHAIN [cap=- jump=-] extra=2 publishes_per_mount=[2, 3, 3, 3, 3, 3]
K2-CHAIN [cap=- jump=1] extra=0 publishes_per_mount=[2, 2, 2, 2, 2, 2]
```

纯模型同一条链：`K2-MODEL-CHAIN arm=Today previous_last_txg=4 publishes_per_mount=[3, 3, 3, 3, 3, 3] white_per_mount=[1, 1, 1, 1, 1, 1]`（`logs/model.log` 原样）。只重开时每次挂载吃 3 个根槽而不是 2 个：24 槽的根环 8 次挂载就整圈转完，`jump` 臂要 12 次。

每一步许可它的那一句：写行 txg 取 max + 1——`.claude/kb/decisions/23-journal的角色与格式.md` 第 360 行（附录 A9，注 3 那一句，整行见附录）：新实例第一次发布的 txg 写的是「≥」；之后每次发布加一——`16-发布语义.md` 第 147 行（附录 A1）；推到覆盖为止——`16-发布语义.md` 第 187 行（附录 A4）。

四句：
1. 分不分辨候选：分辨。`jump` 臂（副本 `mount-arms.patch`：普通挂载的写行 txg 往后挪，挪到暖机计划只要 1 次为止）白推 0 次、每次挂载 2 次发布，F1 / F2（任一份口径下）照样 0 次丢（A5 原样见上）。
2. 系统看不看得到判别子：看得到，写行 txg 与区域归属在取号之前就算得出（`warm_up_publish_txgs` 是纯算）。
3. 满足的是哪个分句：派发提示与正文第三节 K2 那一句「或多推（白推一次空发布）」。**它不是跑前条款第 2 行里「少施加、多施加、判红或写出与另一候选不同的字节」的哪一种**——没有少施加任何记录；它写出与 `jump` 候选不同的字节（多一次空发布、写行 txg 不同）。
4. 改法在打中的格上还中不中：`jump` 在普通挂载的这些格上 0 次白推（量过，副本）。**但在回退那一格它不合条款**：`23-journal的角色与格式.md` 第 363 行（附录 A11）写回退第一个新根的 txg 用的是等号（整行见附录 A11）；副本上 `jump` 只动了普通挂载，所以回退用例里 ≡ 2 的那几次照样白推（`logs/k2-A1-today.log.gz` 原样：`K2 [cap=- jump=-] rollback extra=1 target=(1,3) inst=2 chosen=(1,4) row=5@d0 row_mod3=2 warm=[6@d0,7@d1] warm_count=2 white=1 …`）。另一个候选「截成至多 1 次」（`cap=1`）在副本上也 0 次丢（A4），但它在 ≡ 2 那一档推完暖机时本实例的根只在盘 0 上，是靠之后那一版数据的根落盘 1 才补齐；回退在那一刻就会向管理员确认，不合 `16-发布语义.md` 第 187 行「不向管理员确认回退」，不当候选。

推翻条件：有一条已定条款要求写行 txg 恰好等于 max + 1（普通挂载也是等号），或者要求挂载之后的空发布数是一个与余数无关的常数——那时白推是条款的直接后果，不是「现算规则」多推。

### 四、旁见（不计进 K2 的判定）：今天的恢复在丢一整块盘时丢掉刚确认的那一版，与暖机推几次无关

点名单元验证照今天的代码（两条位置条目都要验得过）跑的三条臂，丢一次盘就丢那一版，每次挂载恰好一次（`logs/run-k2-arms.out` 原样）：

```text
K2-SUMMARY [cap=- jump=-] crash-then-remount states=11332 remount_errors=0 disk_loss_failures=11332 mounts_by_row_mod3=[3095, 5135, 3102] white_by_row_mod3=[0, 0, 3102]
K2-SUMMARY [cap=- jump=-] rollback mounts=20 disk_loss_failures=20
K2-SUMMARY [cap=- jump=-] subsequent mounts=24 white_publishes=2 disk_loss_failures=24
K2-SUMMARY [cap=0 jump=-] crash-then-remount states=5170 remount_errors=0 disk_loss_failures=5170 mounts_by_row_mod3=[2061, 2068, 1041] white_by_row_mod3=[0, 0, 0]
K2-SUMMARY [cap=0 jump=-] subsequent mounts=24 white_publishes=0 disk_loss_failures=24
K2-SUMMARY [cap=- jump=1] crash-then-remount states=10305 remount_errors=0 disk_loss_failures=10305 mounts_by_row_mod3=[5170, 5135, 0] white_by_row_mod3=[0, 0, 0]
```

一格的原样（`logs/k2-A1-today.log.gz`，按故障拆行，字段一字未改）：

```text
test k2_subsequent_mounts_sweep ... K2 [cap=- jump=-] sub extra=0 round=0 inst=2 chosen=(1,3) row=4@d1 row_mod3=1 warm=[5@d0] warm_count=1 white=0 covered_both=true mount_bytes=369664 mount_barriers=5 mount_fua=2 data=(2,6)
F1_data_root_slot:ok effective=Some((2, 6)) applied=1
lose_d0:LOST chosen=(InstanceGeneration(2), CheckpointTxg(4)) effective=Some((2, 4)) applied=0 above_water=2 verification_failed=1 | without_named_verification:ok effective=Some((2, 6)) applied=2
lose_d1:ok chosen=(InstanceGeneration(2), CheckpointTxg(6)) effective=Some((2, 6)) applied=0 above_water=0 verification_failed=0 | without_named_verification:ok effective=Some((2, 6)) applied=0
F1_failures=0 F2_failures=1
```

历史：mkfs、第一个文件（txg 3）、关、重开（实例 2 写行 txg 4 落盘 1、暖机 txg 5 落盘 0，两块盘都覆盖了）、发一版数据 txg 6（落盘 0，fsync 返回）、盘 0 整块掉了。零崩溃、一个故障。幸存盘 1 上本实例的根 (2, 4) 被择到，水位之上 2 条记录（txg 5、6，两份镜像都在盘 1 上一份）都在，**施加第一条时点名单元验证失败**（`verification_failed=1`）：`replay_journal` 里是 `named.locations.iter().all(…)`，要求点名单元的两条位置条目**都**读得出、校验和都对，而其中一条在掉了的盘 0 上。同一张盘面关掉点名验证就读回 txg 6。

条款那一边：`.claude/kb/decisions/16-发布语义.md` 第 174 行（附录 A3）「同一实例里退一代，靠 journal 重放追得上」，而暖机（第 187 行，附录 A4）买的正是「根所在的盘掉了之后退到本实例的另一条根、靠重放追上」。「施加前逐项验证点名单元」（`23-journal的角色与格式.md` 第 347 行，附录 A6）字面没说一个单元的两份要都过还是任一份过；「记录在」那一句写的是「任一份自证校验和过即在」（同文件第 354 行，附录 A9），管的是记录不是单元。

四句：
1. 分不分辨臂：**不分辨**。今天的暖机、一次不推、写行 txg 往后挪，三条臂每次挂载都丢一次；它不能拿来判 K2 的任何候选。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「打中不分辨臂」那一形，共用的前提是「点名单元两份都要验过」，另记一笔，K2 只看任一份口径下的格（第二、三节）。
2. 系统当时看不看得到判别子：看得到——读得出的那一份校验和对，读不出的那一份是「读不出」而不是「读出来不对」。
3. 满足的是哪个分句：不属于 K2 的任何一句；它属于「施加前逐项验证点名单元」这条已定分项在丢盘下的读法。
4. 改法：副本 `PRESUMED_R1_NAMED_ANY=1`（任一份验过即过）在同一批格上 0 次丢（A2、A4、A5 原样见第二节）。**只在我的副本上量过、被攻过零轮**；「任一份过」会不会放过 E77 要挡的那种静默嫁接（一份是新内容、另一份是旧内容），我没有量。

推翻条件：入库装置上同一段历史（重开、发一版、丢掉那一版根槽所在的盘）读回了那一版——那就是我的「丢盘」模型（把那块盘换成全 0 的盘）与真实降级读不是一回事。

## K4（P6）　所选根自己那条记录读不出时链首从哪接

### 造的是什么形态（条款允许、今天的 `crates/` 写不出）

- 一次发布切成 N 个事务 N 条记录，每条记录是它自己那个事务（带提交标记），共享的提交内生块只在最后一条点名（用户 2026-09-23 定并行线一，派发提示转述；C491 那一行字面还写「没定」）。
- 同一次发布的 N 条记录共享同一个 checkpoint_txg。依据 `.claude/kb/decisions/23-journal的角色与格式.md` 第 510 行（整行见附录 A5）：「同一 checkpoint 窗口内的多条 journal 记录**共享同一个 txg**」。
- 一次发布的 N 条记录在同一段里写：`.claude/kb/decisions/16-发布语义.md` 第 170 行（附录 A2）定的顺序是「COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」，N 条记录之间没有屏障，崩溃时这一段里任意子集都可能已落盘。
- 今天的实现一次发布只写一条记录（`transaction.rs` 的 `UndecidedClauseBlockingMoreThanOneDataUnit::STILL_UNDECIDED_TODAY` 在任何落盘动作之前拒绝多单元），所以这一格在副本上手造：

装置（副本，`copy-side/presumed_r1_opus_k4.rs`）：两块 4 GiB 稀疏盘上真跑 `make_filesystem`；每条记录用 `JournalRecord::to_bytes` 写进两份镜像的 `record_offset(jsn)`，写之前 `JournalRecord::parse` 往返逐字段相等；每条记录点名的单元真写进单元区、CRC-32C 真算（数据单元 32 KiB，最后一条另点名两个 16 KiB 的共享单元）；然后走真代码 `scan_journal` → `replay_journal`（点名单元验证开着）。所选根 (1, 10) 由装置直接交给 `replay_journal`，**不经 `choose_root`**。三次发布 P0（txg 10，n0 条）、P1（txg 11，n1 条）、P2（txg 12，n2 条），n0、n1 ∈ {1, 2, 3}，n2 ∈ {0, 1, 2}（n2 = 0 表示崩在 P1 的根槽之前、P2 没开始）；读不出的记录集合取遍全部子集，共 1372 格。

判的对象：`replay_journal` 施加的第一条记录（链首）。对照是「锚点已知时的条款链」：锚点 = P0 的最后一条（「所选根覆盖的最后一条记录」，`23-journal的角色与格式.md` 第 358 行，附录 A9），往后 jsn 严格连续、断号即止（同文件第 524 行，附录 A7）。每格还算最少要几个故障：P0 的记录读不出每条 2 个（两份镜像）；n2 = 0 时 P1 缺的是崩溃造成的（0 个）；n2 > 0 时根 11 写过、要读不出（1 个），P1 缺的每条 2 个，P2 缺的是崩溃造成的。

### 今天的做法，1372 格的总账（`k4-today.log` 原样）

```text
K4-TALLY [today] all-P0-torn(no-anchor) OVER(接上不该接的) = 84
K4-TALLY [today] all-P0-torn(no-anchor) match = 147
K4-TALLY [today] all-P0-torn(no-anchor) match-none = 63
K4-TALLY [today] anchor-readable UNDER(少施加) = 196
K4-TALLY [today] anchor-readable match = 147
K4-TALLY [today] anchor-readable match-none = 343
K4-TALLY [today] anchor-torn-other-P0-readable UNDER(少施加) = 196
K4-TALLY [today] anchor-torn-other-P0-readable match-none = 196
K4-SUMMARY [today] cells=1372
```

对照组：n0 = n1 = 1（每次发布一条记录，今天的形态）那 28 格，今天的做法全部 `match` 或 `match-none`（`k4-today.log` 里 `n=(1,1,*)` 14 格 match、14 格 match-none）。两处打中只在 N ≥ 2 时出现：K4-a 只在 n0 ≥ 2，K4-b 只在 n1 ≥ 2。

第二条路：纯 std 模型（`presumed_r1_opus_model.rs`，不引 `crates/` 一行）照 `replay_journal` 的控制流另写一遍，四条臂的 8 行总账与副本装置逐行相等（`model.log` 里 `K4-MODEL-TALLY` 32 行）。

### K4-a　锚点读得出，但所选根那次发布有 N ≥ 2 条：链首认错、一条都不施加（零故障）

原样一行（`k4-today.log`）：

```text
K4 [today] n=(2,2,0) torn=[] branch=anchor-readable applied=[] clause_known_anchor=[102, 103] verdict=UNDER(少施加) min_faults=0 verification_failed=0 effective_txg=10
```

历史：实例 1 发布 P0（txg 10，两个数据单元 ⇒ jsn 100、101 两条记录），根 10 落盘；再发布 P1（txg 11，两条记录 jsn 102、103），两条记录都落盘之后、根 11 落盘之前崩溃。零介质故障。

每一步许可它的那一句：P0、P1 各两条记录——并行线一（派发提示转述的用户 2026-09-23 定案）加 `.claude/kb/checks-owed.md` 第 275 行 C310（事务切分纪律与记录数口径打架） 里的「并行线一按 N 个数据单元 N 条记录写」（整行见附录 A10）；两条共享 txg——第 510 行；崩在记录之后、根槽之前——第 170 行的顺序；恢复要做什么——`23-journal的角色与格式.md` 第 399 行（附录 A8）「由记录重建那次发布的根」。

今天的代码：`replay_journal` 里 `root_own_record_counter` 用 `find` 取「同实例、checkpoint_txg 相等」的**第一条**（jsn 100），期望下一条是 101；水位之上第一条是 102 ⇒ 断号即止，一条都不施加，恢复停在根 10，与第 399 行要的「重建根 11」相反。条款口径下锚点是 P0 的**最后一条**（101），链从 102 接。

四句：

1. 分不分辨臂：分辨。`max` 臂（锚点取同 txg 里 jsn 最大的那条）在 anchor-readable 这一支 343 格 match、0 格 UNDER；`today` 与 `stop` 两臂各 196 格 UNDER。
2. 系统当时看不看得到判别子：看得到。jsn 最大的那条同 txg 记录就在扫描结果里。
3. 满足的是哪个分句：跑前条款第 2 行「站不住」的「在那条历史上今天的做法少施加」。**不是**第 3 行：第 3 行要「所选根那条读不出」，这一格锚点读得出。它打的是正文第二节 P6 那一行今天做法的前半句「链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后」：N ≥ 2 时「那条」不唯一。
4. 改法在打中的格上还中不中：`max`、`max-stop` 两臂在这 196 格上 0 格还中（量过，副本 `k4-max.log`、`k4-max-stop.log`）；`stop` 臂 196 格照中（量过，`k4-stop.log`）。

副本上改法的原样一行（`k4-max.log`）：

```text
K4 [max] n=(2,2,0) torn=[] branch=anchor-readable applied=[102, 103] clause_known_anchor=[102, 103] verdict=match min_faults=0 verification_failed=0 effective_txg=11
```

### K4-b　锚点读不出、下一次发布有 N ≥ 2 条：只看 txg = 根 + 1 会跨过一个断号接上去（2 个介质故障加一次崩溃）

这一格就是跑前条款第 3 行点名的那一格。原样两行（`k4-today.log`）：

```text
K4 [today] n=(1,2,0) torn=[100, 101] branch=all-P0-torn(no-anchor) applied=[102] clause_known_anchor=[] verdict=OVER(接上不该接的) min_faults=2 verification_failed=0 effective_txg=11
K4 [today] n=(1,2,0) torn=[101] branch=anchor-readable applied=[] clause_known_anchor=[] verdict=match-none min_faults=0 verification_failed=0 effective_txg=10
```

历史：P0 是一次只有一条记录的发布（jsn 100，txg 10；例如挂载时最后一次暖机空发布，或一次只写一个单元的发布），根 10 落盘。P1 是一次两个数据单元的发布（jsn 101、102，txg 11，共享单元只在 102 点名），崩在 P1 的记录段里：102 的两份镜像落盘了，101 的两份没落盘（第 170 行：同一段内任意子集）。再加 jsn 100 的两份镜像读不出（2 个介质故障）。

今天的代码：P0 那条读不出、同 txg 的一条都没有 ⇒ 走「没锚点」那一支，水位之上第一条是 102，它的 txg 是 11 = 10 + 1 ⇒ 接上、施加、根重建成 txg 11。同一段历史只要 100 读得出（第二行），链就在 101 那个断号上停住。**断号在不在，今天的判法只看 txg；一次发布一条记录时「txg = 根 + 1」等价于「jsn = 锚点 + 1」，N ≥ 2 时不等价。** 这正是 `23-journal的角色与格式.md` 第 524 行那句空洞的反例（「5851 错接到 5849 后面」）换了个位置出现。

多一个故障时的代价（`k4-today.log` 里另一条用例的原样两行）：再把 101 点名的那个数据单元两份都改坏，今天的接法施加 P1、那个坏单元一次都没被读：

```text
K4-UNIT [today] anchor-torn torn_mask=0b11 corrupt_unit_named_by=101 applied=1 verification_passed=1 verification_failed=0 effective_txg=11 last_applied_jsn=102
K4-UNIT [today] anchor-readable torn_mask=0b10 corrupt_unit_named_by=101 applied=0 verification_passed=0 verification_failed=0 effective_txg=10 last_applied_jsn=0
```

即「施加前逐项验证点名单元」（`23-journal的角色与格式.md` 第 347 行，附录 A6；六条口径连第六条整段在 A6）对只在那条缺失记录里点名的单元失效：它没有别的记录点名。⚠️ 我的装置里记录的新根段抄的是 mkfs 的树表，**「重建出来的树指着那个坏单元、读回什么」没有量**，只量到「那个单元没被验、P1 被施加」。

84 格 OVER 的最少故障数分布（`k4-today.log` 里按 `min_faults=` 数）：2 个 4 格、4 个 4 格、5 个 12 格、6 个 4 格、7 个 24 格、9 个 24 格、11 个 12 格。

四句：

1. 分不分辨臂：分辨，但换来另一种错。`today` 与 `max` 两臂各 84 格 OVER；`stop` 与 `max-stop`（没锚点就一条都不接）0 格 OVER，但同一支里今天接对了的 147 格变成 UNDER（`k4-stop.log`、`k4-max-stop.log` 原样见下）。那 147 格里有 21 格是 n1 = 1——今天的一条一记录形态——`stop` 在那里把步 3 三方第一轮之后定下的「只接 txg + 1 那一条」退回到一条都不接。
2. 系统当时看不看得到判别子：**记录头与根记录这一层看不到。** 147 格 match 与 84 格 OVER 在决定那一刻可见的东西逐条相同：水位之上第一条可读记录的 txg 都是根 + 1、它的前一条都读不出（它的反向链指向的那一条也读不出；`replay_journal` 今天本来也不核反向链）；今天的记录头里没有「这是本次发布第几条」，根记录里没有「覆盖到的最后一条 jsn」。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判别子观测不到」那一形：要求这一支「既不多接、也不少施加」的判据在今天的格式下互相矛盾，任何只用今天可见量的接法只能满足一半。（单元头里的写序是一条部分判别路径，见「零轮形态」，没实现、没量。）
3. 满足的是哪个分句：跑前条款第 3 行「攻方在「N ≥ 2 条记录、所选根那条读不出」的历史上量到跳过或多接」——量到的是**多接**；也满足第 2 行「多施加」。
4. 改法在打中的格上还中不中：`stop` / `max-stop` 在 84 格上 0 格还中、代价是同一支 147 格 UNDER（量过，副本）；`max` 在 84 格上 84 格照中（量过，副本）。

```text
K4-TALLY [stop] all-P0-torn(no-anchor) UNDER(少施加) = 147
K4-TALLY [stop] all-P0-torn(no-anchor) match-none = 147
K4-TALLY [max] all-P0-torn(no-anchor) OVER(接上不该接的) = 84
K4-TALLY [max-stop] all-P0-torn(no-anchor) UNDER(少施加) = 147
```

### 不计打中的一支

`anchor-torn-other-P0-readable`（P0 的最后一条读不出、P0 别的记录读得出）196 格，四条臂一样全是 UNDER 或 match-none：系统看得到 P0 前几条，看不出缺的那条是不是 P0 的最后一条，锚点已知的条款链在这里不可观测。四条臂同中，按「打中不分辨臂」不拿它判臂。

### K4 这一格落哪一种

**站不住。** 今天的做法原样写成条款，在 N ≥ 2 下两处打中：K4-a 零故障少施加（改法 `max` 在副本上量过全修），K4-b 两个介质故障加一次崩溃多接（今天的格式下没有能同时不多接、不少施加的接法）。K4-b 触发跑前条款第 3 行：这一格的条款要跟并行线一一起写。

推翻条件：有一条已定条款让同一次发布的 N 条记录**不**共享 checkpoint_txg（例如非最后一条写 txg 0 或各自一个号），或者要求同一次发布的记录之间有屏障、按 jsn 顺序落盘——前者让 K4-a、K4-b 的装置前提不成立，后者让 K4-b 那个「101 没落、102 落了」的崩溃状态不可达（那时 K4-b 最少要 4 个介质故障）。

## 零轮形态（我自己提的改法，不是任何一格的结论）

下表每一行都**只在我的模型上量过、被攻过零轮**。「量过」= 副本装置上的原样输出（日志在模型目录 `logs/`）；「推的」= 按代码推、没实现、没跑。

| # | 改法 | 修哪一格 | 在打中的格上 | 代价 / 没量的 | 状态 |
|---|---|---|---|---|---|
| Z1 | 普通挂载的写行 txg 取「≥ max + 1 里第一个让暖机只推 1 次的」（副本 `PRESUMED_R1_ROW_TXG_JUMP=1`） | K2 多推 | 白推 0 次，只重开的链每次 2 次发布；F1 / F2（任一份口径）0 次丢 | 回退那一句是等号（附录 A11），回退照样白推；txg 跳号对记账代、F、根环轮转以外的消费者有没有影响没查 | 量过（A5、A7） |
| Z2 | 点名单元两条位置条目任一条验过即过（副本 `PRESUMED_R1_NAMED_ANY=1`） | K2 旁见 | 11332 + 24 + 20 格 0 次丢 | 会不会放过 E77 要挡的那种一份新一份旧的嫁接，没量 | 量过（A2） |
| Z3 | 锚点取同实例、同 txg 里 jsn 最大的那条（副本 `PRESUMED_R1_CHAIN_HEAD=max`） | K4-a | anchor-readable 这一支 196 格 UNDER → 0 格，343 格 match；别的支与今天逐格同 | 锚点读不出、P0 别的记录读得出时仍然一条都不接（196 格，四条臂同） | 量过（`k4-max.log`） |
| Z4 | 没锚点就一条都不接（副本 `stop`；与 Z3 合起来是 `max-stop`） | K4-b | 84 格 OVER → 0 格 | 同一支 147 格从 match 变 UNDER，其中 21 格是 n1 = 1——今天的一条一记录形态，步 3 三方第一轮之后才修成「接 txg + 1」的那一格退回去 | 量过（`k4-stop.log`、`k4-max-stop.log`） |
| Z5 | 根记录带「所覆盖的最后一条记录的 jsn」（C77（重放起点未定义） 里「根带 jsn 水位」那条出路） | K4-a、K4-b 两格 | 推的：链首不再依赖锚点那条记录读不读得出，锚点已知时的条款链就是今天可判的链 | 改格式：根记录多一个字段；根记录的预留位够不够没查 | 推的 |
| Z6 | 记录头带「本条是本次发布第几条」（或本次发布共几条） | K4-b | 推的：没锚点时要求水位之上第一条是 txg + 1 那次发布的第 1 条，101 缺时 102 被拒 | 改格式；K4-a 另要 Z3；P1 的第 1 条缺、后面几条在时仍然一条都不接（那是对的） | 推的 |

K4-b 在今天的格式下没有一个只用可见量就「既不多接、也不少施加」的接法（K4 第三节第 2 句），所以 Z4 与 Z5 / Z6 之间是一个真岔路：Z4 不改格式、在一条一记录的形态上倒退；Z5 / Z6 改格式。

## 没打中的形状

K2「不够」：
- 写行 txg 的三种余数 × 三种起点（后续挂载 24 次、回退 10 次加回退之后再挂 10 次、只重开 18 次）：0 次没覆盖两块盘（`covered_both=false` 只在截断暖机的对照臂上出现）。
- 挂载中途崩了再挂：一次可写挂载（前缀 0、1、2 版）与一次回退（前缀 2、3 版）的每个段前缀加段内全部真子集，今天的臂 11332 个状态，0 次再挂失败（`remount_errors=0`），0 次丢（任一份口径）。
- 想过没跑成的：降级可写挂载（一块盘缺席时 `all_devices` 只剩一块，覆盖判据变宽）——第一版 mkfs 断言 2 块盘，挂载侧没有降级可写的路，条款上降级只读，没造；三盘或别的区域归属——mkfs 当场拒；暖机途中根槽写失败——今天的挂载在那一步整体返回错误、不向谁确认，重挂是新实例重新算，没有要判的「次数」；回退之后旧的更高 txg 根在挂载时暂时读不出、之后又读得出——那是 C331（择根倒挂压过已确认的写） 的形状，与暖机次数无关，没造。

K4：
- n0 = n1 = 1（今天的一条一记录）28 格：今天的接法 0 格打中（对照组）。
- 锚点读不出、P0 别的记录读得出：196 格四条臂同样，没把它算成打中（K4 第三节）。
- 回退那条路（`mount_rollback` 里 `own_record` 也用 `find` 取同 txg 的第一条、读不出就拿环里 jsn 最大那条顶）：只看了代码，没造 N ≥ 2 的回退历史，不算打中也不算没打中。
- 取样范围：n0、n1 ∈ {1, 2, 3}，n2 ∈ {0, 1, 2}，每格读不出的记录取遍全部子集；所选根固定 (1, 10)，单实例，没有回退行，在飞上限没碰到（最多 8 条）。

## 这条腿自己的限度

- 副本不是入库装置：副本编不过的那一处（`walk.rs` 的 `is_commit`）是我按字段名改的，没核它的语义；K2 的三条环境变量臂与 K4 的三条链首臂是我写的，只有「不设环境变量时与工作区相同」这一句靠补丁的字面保证，没跑门禁。
- K4 的记录是手造的：新根段抄 mkfs 的树表，事务号从 1 起连续，共享单元两个；所选根直接交给 `replay_journal`，没经 `choose_root`、没经实例表、没经第六条（按发布边界截短）。所以 K4 只量了**链首**，K4-b 那一版被施加之后读回什么没量。
- K4-b 的「101 没落、102 落了」靠的是「同一次发布的 N 条记录之间没有屏障」（附录 A2 的顺序句）；并行线一的写路径今天不存在，将来它若在记录之间加屏障，K4-b 的最少故障数会从 2 变 4。
- K2 的「丢盘」模型是把那块盘换成全 0：读回全 0、自证全不过；真实的降级读（设备报错）走不走同一条代码路径没核。
- 纯 std 模型与副本装置在 K2 的余数算术、K4 的 8 行总账上逐行相等；两者共用我对条款的读法（同一次发布共享 txg、锚点 = 最后一条），读法错了两条路一起错。

## 没做什么

- 不判 K1、K3，不判正推那几格，不替主 agent 采纳。
- 副本上的数没在入库装置上重做；没跑门禁、没跑变异。
- 没造 N ≥ 2 的回退历史，也没量「任一份口径」会不会放过嫁接。
- 没派本地腿，没重复抽样（这是云端单次观测；「没打中」那几条按规则一次不算数）。
- 副本、构建目录与没压缩的日志留在 `/tmp/claude-1000/presumed-r1-opus/`（`repo/`、`target/`、`k2-arms/`、`k4-*.log`），没入库，因为写范围只到模型目录；压缩过的日志已拷进模型目录 `logs/`。

## 附录：引到的 kb 原文（整行抄，行号是 kb 文件自己的行号，2026-09-23 UTC 15:5x 重取——同一天别的会话在改这两份 kb，行号在这一轮里漂过一次）

**A1** `.claude/kb/decisions/16-发布语义.md` 第 147 行：

```markdown
**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。
```

**A2** `.claude/kb/decisions/16-发布语义.md` 第 170 行：

```markdown
**定案**：一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽；fsync 等根槽持久之后才返回。恢复重放在施加任何记录之前，必须逐项验证点名单元的校验和。**系统配置槽在根槽持久之后再更新，每个 checkpoint 一次**，它是发布序列的最后一步、进层 0 枚举，一次发布的段序列因此唯一。**fsync 返回条件多一句**：新实例在本实例写成的根覆盖两块盘之前，fsync 不返回、不向管理员确认回退（做法与次数在已定项 8）。
```

**A3** `.claude/kb/decisions/16-发布语义.md` 第 174 行：

```markdown
- **「fsync 等根槽持久之后才返回」的射程**：返回之后这一代只由那一个根槽罩着（根槽不镜像）。同一实例里退一代，靠 journal 重放追得上；**每个新实例（每次可写挂载、恢复、切换、回退）的第一个根在下一次发布之前是单点**——那个槽读不出或它所在的盘掉了，恢复退到上一个实例的根，重放按严格前缀停在实例边界（D23（journal 的角色与格式） 已定项 14 的注 1），这次挂载里 fsync 已返回的事务丢，一个故障就够。要不要让新实例先把根写到两块盘上再确认，是已定项 8。
```

**A4** `.claude/kb/decisions/16-发布语义.md` 第 187 行：

```markdown
**定案**：新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退；做法是连推空发布，第一版几何至多 R = 3 次。**第一版实付 2 次，而且它是格式常量不是运行时谓词**：根环三个区域的设备归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7），于是 txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘 ⇒ 次数由归属唯一确定。登记成 `format-const` WARM_UP_EMPTY_PUBLISHES = 2 与 FIRST_TRANSACTION_TXG = 3（[layout/01-first-txn.md](../layout/01-first-txn.md) 八），条款在 D22（单元原子性怎么合成） 已定项 16 第 5 句。
```

**A4** `.claude/kb/decisions/16-发布语义.md` 第 189 行：

```markdown
**射程**：定的是新实例确认之前要做什么，不定实例切换本身的判据（D23（journal 的角色与格式） 已定项 14）、不定空发布写出多少单元（已定项 9）。四样已知边角：
```

**A4** `.claude/kb/decisions/16-发布语义.md` 第 193 行：

```markdown
- **代价**：改已定项 7 的 fsync 返回条件（新实例确认前多一个条件）；第一个事务的 checkpoint_txg 从 1 变 3，多两条空记录与两次根写；D28（挂载期承诺量） 已定项 3 的切换预留加「每次切换至多 R = 3 次空发布」，按 D28（挂载期承诺量） 已定项 4 现算的 c_max 取（池规模 9 块 ⇒ 27 块）；空发布占根环槽位，把旧的非空根更早挤出环。
```

**A5** `.claude/kb/decisions/23-journal的角色与格式.md` 第 510 行：

```markdown
- 理由：D5（快照 / 空间记账机制） 是「一个 checkpoint 一个 txg 号」⇒ 同一 checkpoint 窗口内的多条 journal 记录**共享同一个 txg**，彼此分不出先后，D22（单元原子性怎么合成） 要求的「被实际检查的世代号」就失去意义。
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 344 行：

```markdown
**前缀判定的完整口径是六条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引）：
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 345 行：

```markdown
jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 346 行：

```markdown
提交标记齐全的事务才施加（D23（journal 的角色与格式）已定项 7）、
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 347 行：

```markdown
施加前逐项验证点名单元（D16（发布语义）已定项 7，E77（发布的持久顺序）证明承重）、
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 348 行：

```markdown
**所选根的实例在实例表里有回退行时，该实例的记录只施加到回退行的 W 为止**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2：
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 349 行：

```markdown
否则一次落在 R_old 上的普通恢复会把被回退抛弃的那段时间线整段重放回来，管理员的回退被静默撤销）。
```

**A6** `.claude/kb/decisions/23-journal的角色与格式.md` 第 350 行：

```markdown
⚠️ **第六条：施加的单位是一次发布——合法前缀停在一次发布的两个事务之间时，那次发布整体不施加，前五条判出来的前缀再按发布边界截短一次；引用这五条时连这一条一起引。**（D16（发布语义） 已定项 4）
```

**A7** `.claude/kb/decisions/23-journal的角色与格式.md` 第 524 行：

```markdown
**定案**：**前缀判据是 `jsn == expected`，不等即止。** 只用「`jsn` > 上次已应用」过滤**不够**：若序列里有空洞（5849 写成、5850 没写、5851 写成），「>」会把 5851 错接到 5849 后面，**破坏因果序**。还要一个**显式的在飞记录数上限**（取值与语义见已定项 18）。前缀判定的完整口径（六条）在已定项 14。
```

**A8** `.claude/kb/decisions/23-journal的角色与格式.md` 第 399 行：

```markdown
**定案**：**由记录重建那次发布的根：记录头加「新根段」= 树表单元指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 188 字节（实例表单元指针照所选根），4096 记录装 67 个点名项。施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义。已定项 14 条 2「严格大于所选根就施加」原样成立。**
```

**A9** `.claude/kb/decisions/23-journal的角色与格式.md` 第 354 行：

```markdown
两份镜像何时算「记录在」：**任一份自证校验和过即在**（崩溃点重放里的检查由层 0 全量做出——jsn 3 只有一份持久的两个状态都读回文件）。
```

**A9** `.claude/kb/decisions/23-journal的角色与格式.md` 第 358 行：

```markdown
1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。
```

**A9** `.claude/kb/decisions/23-journal的角色与格式.md` 第 360 行：

```markdown
3. **计数器全池接着走**：新实例从前缀末 + 1 接着写、不归零。定长环的槽位由 jsn 决定（槽位与序号解耦那条出路已被 E36（槽位映射那一维） 判掉），归零会让新实例的记录落到旧前缀还要用的槽上；已定项 9 的 48 位本来就按全卷寿命算。⚠️ **checkpoint_txg 也一样**：新实例（普通挂载、切换、回退）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1；普通挂载与回退本来就全环扫描，切换不为算这个下界重扫 journal 环（内存里的 txg 已不小于这次挂载见过的一切；切换的所选根要读盘择，见实例切换那一句）。它与 C331（择根倒挂压过已确认的写） 的修法候选「记录扫描水位」是同一个量，C331（择根倒挂压过已确认的写） 还清时一起写。代价：跳号比今天大（最多到被抛弃时间线的长度），D5（快照 / 空间记账机制） 已定项 2 的点删随之改成「删掉 ≤ 当前代 − K 的全部代」。
```

**A10** `.claude/kb/checks-owed.md` 第 275 行：

```markdown
| C310 | 事务切分纪律与记录数口径打架 | D16（发布语义） 已定项 5 末段逐字「事务切分纪律把一次写请求按单元切成若干事务各自取号，一个事务最多写一个单元的用户数据」，加 D23（journal 的角色与格式） 已定项 7 一条记录一个事务号 ⇒ 一次带 8 个数据单元的 fsync 至少 8 条记录、1 MiB 写 32 条记录（E142（第一个事务的干跑） 产物 `journal_per_mebibyte`，两盘 262144 字节 journal 占写出数据 12%）；而 D23（journal 的角色与格式） 已定项 12 按「目标负载 12 项事务恰占 1 条记录」算 5.9 倍余量，E75（记录尺寸与环几何） 的环下界也按它算；D19（块指针的结构与宽度预算） 已定项 6 码 1 映射 key 的唯一性又全押在切分纪律上（同一事务写两个数据单元 key 相同，产物 `distinct_keys=1`） ⚠️ 2026-09-13 总审核定案：D23（journal 的角色与格式） 已定项 19 ③ 与 D16（发布语义） 已定项 9 把切分纪律与记录数口径合到一处，条款已写，会红的检查仍欠 | E75（记录尺寸与环几何） 加一条臂按切分纪律算 D25（目标负载优先级） 目标负载的记录数与环下界，与 D23（journal 的角色与格式） 已定项 12 那格不等即红 | 用户定案：切分纪律是不是「一事务一单元」；若是，D23（journal 的角色与格式） 已定项 12 的余量与 I-8.1（环几何够大） 的下界重算，若不是，D19（块指针的结构与宽度预算） 已定项 6 码 1 的 key 要重开。**用户 2026-09-13 定案：切分纪律是一事务一单元**（D16（发布语义） 已定项 5 末段照旧）。剩下的是重算：E143（一事务一单元下的 journal 账） 按 D25（目标负载优先级） 六个负载点算出今天条款下每次 fsync 的记录数等于数据单元数（seq 批 1 是 8 条、批 10 是 80 条），journal 恒占用户数据 12.5%；T_dirty 满窗 65 536 条、256 MiB，是 64 MiB 环的 4 倍，I-8.1（环几何够大） 按「任一事务」读只要 12 KiB、按「满窗装进环」读要 768 MiB 或 T_dirty 压到 170.7 MiB；D23（journal 的角色与格式） 已定项 12 那句「12 项恰占 1 条、5.9 倍余量」按的是被否的旧读法，要改；改法（改 D23（journal 的角色与格式） 已定项 12 的余量口径、改环长或 T_dirty、还是让一条记录装多个事务）2026-09-16 用户定案：一条记录装一个事务连同它的元数据（第一个事务今天的形态），里程碑「第二个事务」并行线一按 N 个数据单元 N 条记录写，E143（一事务一单元下的 journal 账） 的账照它重算；D23（journal 的角色与格式） 已定项 12 那句余量口径仍要改，检查仍欠；会红的检查那一半，用户 2026-09-23 判不收口、挪到后面的里程碑（里程碑「第二个事务」收口表第 29 行） | E142（第一个事务的干跑） G8 / G18，2026-09-13 |
```

**A11** `.claude/kb/decisions/23-journal的角色与格式.md` 第 363 行：

```markdown
**显式例外：管理员回退**（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）。回退 = 一次恢复：管理员带外从回退候选集里选一个旧根 R_old——候选集 = 根环里按实例表判仍然有效、且 txg ≥ F_生效（D16（发布语义） 已定项 1）的根，(i, T) 可选 ⟺ 实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti（被抛弃时间线的根一条都选不中）；不施加 R_old 之后的任何记录；取新实例代号；在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)（r_old 诞生代号 ≤ T_old 的单元全部已发布、之后的一个都不算，不需要事务号上限）；第一个新根的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1（回退本来就全环扫描、逐条验证，额外读 0；被抛弃时间线里根槽写失败留下的「只有记录、没有根」的 txg 因此也被跳过；新实例的第一次发布同样从这个最大值 + 1 起，见注 3）；defer 队列、分配器游标、全部记账统计量的现行值从 R_old 那棵账重新载入。**被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽——查账的集合取窄读法：按主语「被抛弃时间线的根」读，仍被有效根引用的槽不在其内——「有效根」指回退候选集里的根（按实例表判仍然有效 ∧ txg ≥ F_生效），每次挂载与每次抬 F 按当时的候选集重算；宽读法（凡被环里任一可读根引用过的槽都隔离）会让抬 F 在第一版 24 槽里买不到任何东西（影子账；隔离量怎么进准入不等式见 C318（影子账隔离的单元没进准入不等式））。** 回退与它的第一个新根同一次发布：回退行、那次发布的单元与回退根在那次发布之前都不生效，崩了就重做——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；留在盘上的是取号写进系统配置的新号（取号先于那次发布持久，D23（journal 的角色与格式） 已定项 16）以及那次发布已落盘而不生效的单元与记录，新号让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行；回退深度由候选集定（txg ≥ F_生效，D16（发布语义） 已定项 1）：平时是整个根环，盘紧时可缩到最近 4 个可退到的状态，每块盘上最新的持久有效根都在内。同一次定案把事务按单元切分 ⇒ 崩溃原子性的单位是一个单元、不是一次写请求（仓里没有任何决策承诺过写请求级原子性，这是一次对外语义的收缩）。
```
