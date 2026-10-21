# m3-close-sysconf-r2 攻方腿（Opus）报告（2026-10-19）

立场：照第四节分工表「云端攻方」那一行，攻第一轮挂起的形态甲、乙、丙与 S3、S6 的再抽一次。代码只读树副本 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree`，引代码写那份副本的绝对路径与行号；kb 行号是 kb 文件自己的（仓根 `.claude/kb/`）。凡写「量过」的都是攻方副本上的数，不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-close-sysconf-r2-opus-model/`。复跑（整条经内存包装；`PROBE_SEED_TARGET` 可选，指到一个用同一份 `Cargo.lock` 编过的 release 编译目录，脚本只从里面拷第三方产物，免得在 2G 里现编 RocksDB）：

```
PROBE_SEED_TARGET=<一个 release 编译目录的上一层> bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-sysconf-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree \
  --work <新的草稿目录>
```

它把冻结副本拷进草稿目录，补上仓根的 `Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，打 `two-rounds.patch`（两处：`transaction.rs` 加形态甲那一臂，环境变量 `SINGLEFS_PROBE_ACQUISITION=two_rounds` 打开，回卷逐份或每盘一次由 `SINGLEFS_PROBE_ROLLBACK=per_written|per_device` 选；`crash.rs` 把崩溃点重放逐状态走的 `visit_crash_state_range_of_the_read_back_flow_with_slices` 从 `pub(crate)` 改成 `pub`），加探针 crate `opus-r2-probe`（`probe-crate/`），`cargo build --release -j 1`，按 runs 表跑七份，每份的 `probe=` 行与 `out/` 里同名文件逐份比、打 `same` / `differs`。

```
949ede5d6cdf8e8d00caa608ee7eb3173727c8f47017aeb07dcf2cf40035185e  ./out/acq-one_round.txt
f47cf550438017a193949617d687f01ea19cb9b08290de420bfa82e6f97a2262  ./out/acq-two_rounds.txt
326245cd7b85bea2bfd1529c89fb3daf6cca24214006080fa0473cdd837eb371  ./out/alpha-one_round.txt
2e25ab544f2e38c35b0a24b8dfb83bcfa9563b45f45a3640c35a974e7721d081  ./out/alpha-two_rounds.txt
8cc5bdf8ff2c12cd1a056a1eee6ebef9da9aaf3e40509989ef396b205b70743c  ./out/rollback-one_round.txt
73045ac0c070d9d28e787fa2d620d14a4c8dff3f6852c277fe3d7d33232b7f31  ./out/rollback-two_rounds-per_device.txt
376b3bf3b2f102e070a80845c6a726d22d815e310ff5a3018e9c2c4704a213d3  ./out/rollback-two_rounds-per_written.txt
1795a2b280921a92297d07712dc3bd0c358f1db63864e312849757df1a7bb2c7  ./probe-crate/Cargo.toml
6e52daea84e7c443a32af17a4f886ded6c43b8344eb9ceb5c764d9c6abf1baf4  ./probe-crate/src/main.rs
6d08ef81cd82f7ab7c4ce299241c114f4cc9064fe4a2d95826d83124eefe277f  ./probe-crate/src/probe_alpha.rs
3103b1f158c425de733436aff06c9acaa2e04df6b17528d34c11764e0464ae2a  ./probe-crate/src/probe_body.rs
5a5c98eebe082c1609fd1acfc89528743c42a5f7cc3cbafaf00f8f82cc8ea944  ./tools/rerun.sh
c5539556402a59f891238022c923d473a4938be8492e9b29f64c3d0677d7ce89  ./two-rounds.patch
```

（`sha256sum` 在模型目录里跑。`out/` 是我用草稿目录里编好的同一份源码跑出来的；`rerun.sh` 在一个新草稿目录里端到端跑过一次，结果见文末「复跑记录」。）

## 各格判定一览

| 格 | 对象 | 判定 | 量过 / 推的 |
|---|---|---|---|
| T1 | 形态甲取号写两轮，「已写出的各份回卷」照字面逐份回卷 | 打中：第二轮第二块盘的取号写报错（单故障），回卷对盘 0 连发两次写、中间没有屏障，同段崩溃把盘 0 两槽一起撕坏；37 个崩溃状态里 3 个下一次可写挂载拒（`SystemConfigurationUnreadableAfterTheReadStage`），盘 0 两槽全废，落进 C463 那一形，之后每次可写挂载都拒 | 量过（副本） |
| T1 | 形态甲，回卷每块盘只写一次 | 同一处报错之后 19 个状态全挂得上 | 量过（副本） |
| T1 | 形态甲，两轮之间崩、第二轮写一半崩、取号之后第一次轮换之前每盘再坏一槽 | 没打中：三个起点各 20519 个崩溃状态 × 12 条故障臂，撞号 0；今天的一轮在同样的取样上每个起点 16386 个状态撞号 | 量过（副本） |
| T1 | 世代号整体加 1 之后现有判法会不会判红新中间态 | 没打中：取号流全部崩溃状态的原始镜像与挂上之后的镜像，池级 checker 红 0（两条臂都是）；回卷那一形的 3 个状态 I-7.16 红，那是盘 0 两槽全废本身 | 量过（副本） |
| T1 | 崩溃点重放状态的增量 | 每条可写挂载流多一段两次原地覆写：闭式 +3，带撕裂态 +8（20511 → 20519） | 量过（副本） |
| T2 | 离线出路 α 落在今天的一轮上 | 打中（重现第一轮打中一，换成 α）：α 写回的号是 1，下一次可写挂载取 2，与只写了单元的实例 2 撞号 | 量过（副本） |
| T2 | α 落在形态甲上 | 撞号没打中（取 3）；持续读错（写之后照样读错）与写不进去两种盘，α 之后照旧拒，出不了只读 | 量过（副本） |
| T2 | α 断言错一位：断言比真实版本新一版（T+1）、α 不查根环 | 打中：α 写下两槽，原本根一恢复可读就挂得上的盘面变成永远拒（`NewerStateStillUnreadableAfterOneReread`） | 量过（副本） |
| T2 | α 查根环（断言那一版的根读得出才写） | 断言 T+1 一个字节不写；但断言真实的 T 也拒（那一版的根正读不出），断言 T−1 照写 | 量过（副本） |
| T2 | R2 收掉的那一形（每盘一槽坏 + 最新根读不出） | 没打中丢写：最新根读不出时挂载由 journal 记录重建出 txg 5，α 断言 T−1 也没让它丢 | 量过（副本，一个起点） |
| T3 | 形态乙「缺的份数不到 2」 | 打中（推的）：最新那次发布的单元写到一半取下的快照，缺一份、那一版的根住在另一块盘上，今天拒、加上这一格就放行 | 推的 |
| T3 | 形态乙三种盘面（环转过一圈、记录只在一块盘上、旧快照在发布途中取下） | 没打中 | 推的（第一轮量过的 41 条照引） |
| T3 | 坏副本没被补写那三条红 | 不分辨臂，是单元副本修复那一侧的账 | 推的 |
| T4 | 形态丙的闸：空盘、别池的盘、对调过的盘 | 没打中 | 推的 |
| T4 | 形态丙的闸：同 fsid 的克隆盘（带着所选那一版的全部单元，另带克隆之后自己写的根与记录） | 打中（推的）：闸只问「有没有那一版」，不问「有没有比那一版新的东西」，补写之后克隆的根与记录成了本池可见的 | 推的 |
| T4 | 补写失败按取号失败回卷，叠在形态甲逐份回卷上 | 打中：与 T1 第一行同一处，两槽全废的盘补写失败再崩，回到两槽全废 | T1 那一格量过，叠加推的 |
| T5 | S3「自证不过」 | 没打中：fsid 不符的槽一块盘一槽，三个起点 × 两条臂 × 全部崩溃状态，撞号 0 | 撞号量过；丢确认过的写没有判据，没量 |
| T5 | S6「不收」 | 没打中：没有读故障时新实例的第一条记录编在两块盘上全部读得出的记录最大计数器之后，残留坐不到它前一格 | 推的；多记录发布的崩溃状态超过约 10⁶，没跑 |

最脆弱的一条：形态甲的回卷。照「已写出的各份回卷」逐份写，同一块盘两槽在同一段里被覆写，这正是代码里三处注释（`transaction.rs` 第 672、781、1138 行）说要防的「覆写旧槽时新槽已持久」；单故障加一次断电就造出 C463 的两槽全废盘。改成每块盘只回卷一次（或两次回卷写之间加屏障）在量过的那几格上都挂得上，但只在我的模型上量过、被攻过零轮。

## 取样与方法（这一轮的攻击面之二：产生结论的方法）

第一轮的探针只走了一个起点、一个切点（第一条 journal 记录写之前那一个前缀）。这一轮改成：
- 盘：`SparseBlockDevice` 两块，单元区 384 槽那一档小盘（`tests/common_admission_pool/mod.rs` 的 `parameters_of_the_384_slot_small_devices`，经 `#[path]` 引进原型，不是崩溃枚举用例）。每个起点只建一次池，之后每段历史从内存那一份拷。
- 起点（前缀，放开扫的用户步骤）：`k0` 第一个文件之后直接崩；`k2` 再覆盖写两次之后崩；`r1` 覆盖写一次、崩、可写挂载（取 2）、再覆盖写一次之后崩（下一次取 3）。第四个起点 `k0_pub`（挂上之后用户再覆盖写一次）没跑：它的崩溃状态数一条臂就是 1 069 098（`probe=acq_counts prefix=k0_pub … states_with_torn=1069098`，草稿目录 `count-one_round.log`），超过一段约 10⁶ 的上限。
- 被录的那一步：从起点做一次可写挂载（取号、写行发布、暖机直到本实例的根覆盖每块盘），录写流。
- 崩溃状态：崩溃点重放的同一个计划（副本里把 `visit_crash_state_range_of_the_read_back_flow_with_slices` 改成 `pub`，展开取 `EveryProperSubset`，原地覆写取三态），每个状态都展开、一个不落。我没有调 `enumerate_crash_states` 本身：它的判器是按文件内容的 oracle，我要在每个状态上另做「再坏一槽、再可写挂载、比号」，所以走它下面同一份 `CrashStatePlan`。状态数用 `crash_state_count_with_torn_in_place_overwrites` 先算，与逐状态走的总数相同（`states_judged` 等于 `states_with_torn`）。
- 每个状态上判的：原始镜像跑池级 checker；不加故障可写挂载一次，挂上之后再跑池级 checker；十二条故障臂（盘 0、盘 1 各坏一槽的四种组合 × 翻一个字节 / 持续读错，加单块盘一槽换成别池 fsid 的四种）各算一次取号式子 `next_instance_generation_to_acquire`，状态里有本实例的非系统配置写而式子算出的号不大于本实例的号时，才真做可写挂载，挂上且号不大于本实例的号记 `COLLIDE`。
- 回卷：同三个起点里的 `k0`、`r1`，在整池第 n 次系统配置写上注入写报错（只那一次 / 从那一次起每次），n 走遍取号的写，录失败那次挂载的写流（报错的写不进流），枚举它的全部崩溃状态，每个状态不加故障可写挂载一次。
- 估时：先跑每条臂 `k0` 的前 3100 个状态（`PROBE_STATE_LIMIT=3100`），31 线程 4.0 秒与 7.0 秒（草稿目录 `sample-*.log` 的 `timing` 行），按它估全量六段约 5 分钟，没缩。实跑 `acq-one_round` 196 秒、`acq-two_rounds` 157 秒，其余五份各 0–1 秒（草稿目录 `measure-all.out`）。
- 合计：取号流 123 410 个状态，回卷流 320 个状态，远低于约 10⁷。

## T1　形态甲：取号写两轮

实现（副本 `two-rounds.patch`）：`write_the_witnessed_acquisition` 把逐盘一写、一道屏障做两遍；每次写仍走 `write_system_configuration_slot`，世代号取本盘自证槽最大值加一、槽号取世代号 mod 2，所以第二轮落在第一轮没碰的那一槽。今天写哪一槽：「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:625`）。回卷沿用 `roll_back_acquisition`，它对传进来的每一项各写一次：「for &index in written {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:680`），写之前一道屏障、写与写之间没有屏障。「已写出的各份回卷」照字面就是逐份传进去（`per_written`）；另一臂每块盘只传一次（`per_device`）。

### 打中：逐份回卷让单故障加一次断电造出两槽全废的盘（量过，副本）

盘面与历史（起点 `k0`，`r1` 结果相同）：
1. 第一个文件之后进程退出，可写挂载。
2. 取号第一轮两块盘都写成、屏障过；第二轮盘 0 写成，盘 1 那一写设备报错（整池第 4 次系统配置写，单故障）。
3. 回卷：一道屏障，之后照 `written = [盘 0, 盘 1, 盘 0]` 发三次回卷写，盘 0 两槽都被覆写，三次写在同一段里。
4. 这一段里断电。

结局：「probe=rollback prefix=k0_fail4_once acquisition=two_rounds rollback=per_written fault=next_mount_no_fault outcomes={"SystemConfigurationUnreadableAfterTheReadStage": 3, "mounted": 34}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/rollback-two_rounds-per_written.txt:40`），例子「probe=rollback prefix=k0_fail4_once acquisition=two_rounds rollback=per_written example=next_mount/SystemConfigurationUnreadableAfterTheReadStage ordinal=20 sysconf=d0[bad,bad] d1[g6i1t3,g7i2t3]」（`research/prompts/m3-close-sysconf-r2-opus-model/out/rollback-two_rounds-per_written.txt:43`）。这 3 个状态的原始镜像池级 checker 红 I-7.16：「probe=rollback prefix=k0_fail4_once acquisition=two_rounds rollback=per_written raw_checker_red_states_by_invariant={"I-7.16": 3}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/rollback-two_rounds-per_written.txt:41`）。可写挂载拒不写盘、只读挂载不写盘，盘 0 两槽从此全废，与 C463 同一个盘面；自愈要形态丙。
对照：
- 每块盘回卷一次：「probe=rollback prefix=k0_fail4_once acquisition=two_rounds rollback=per_device fault=next_mount_no_fault outcomes={"mounted": 19}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/rollback-two_rounds-per_device.txt:40`）。
- 今天的一轮：回卷流最多 5 个状态，全挂得上（`out/rollback-one_round.txt` 的 `next_mount_no_fault` 各行）。
- 「回卷两轮时写失败」（从第 n 次起每次写都报错）：回卷报 `RollbackFailed`，n = 2、3、4 的结束盘面与全部崩溃状态都挂得上（同一文件 `_onward` 各行），新号留在盘上、下一次取号跳过它。

四句：
- 分不分辨臂：分。只中逐份回卷；每盘一次与今天的一轮在同一段历史上不中（量过）。
- 系统看不看得到：看不到。盘 0 两槽都撕了，读得出的只有「这块盘一份自证过的都没有」，与换进一块空盘、别池的盘逐条相同，所以只能拒。
- 判据的哪一句：单故障（一次写报错）加一次断电之后可写挂载永远拒；对第一轮判决挂起的形态甲那一段「任一轮报错，已写出的各份回卷成取号之前的最大号」照字面。
- 改法：每盘一次在这 37 格上 0 拒（量过）；回卷写之间加屏障（推的，没实现）。两样都只在我的模型上量过或推过，被攻过零轮。

### 没打中：两轮之间崩、第二轮写一半崩、取号之后第一次轮换之前每盘再坏一槽（量过，副本）

形态甲：「probe=acq prefix=k0 acquisition=two_rounds rollback=per_written fault=flip/d0s0/d1s0 outcomes={"formula_above_acquired": 20519}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-two_rounds.txt:3`）；十二条故障臂、三个起点全是 `formula_above_acquired`（同一文件，`COLLIDE` 0 行）。
今天的一轮，同样的取样：「probe=acq prefix=k0 acquisition=one_round rollback=per_written fault=flip/d0s0/d1s0 outcomes={"COLLIDE": 16386, "formula_above_acquired": 4125}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-one_round.txt:3`），`k2`、`r1` 各 16386。所以第一轮只在一个切点上量的 C555 撞号，在今天的写流上是 20511 个状态里 16386 个（有本实例的单元写落了、轮换还没把新号写进另一槽的那些），三个起点相同；形态甲把它们全关掉。
两轮之间崩、第二轮只落一块盘，都在这 20519 个状态里：这时还没有任何单元写，再坏一槽重取同一个号也没有东西可撞（`footprint_states=20502` 之外的 17 个状态就是这一类）。

### 现有判法与状态数（量过，副本）

- 池级 checker：两条臂、三个起点，原始镜像与挂上之后的镜像红 0：「probe=acq prefix=k0 acquisition=two_rounds rollback=per_written raw_checker_red_states_by_invariant={}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-two_rounds.txt:16`）、「probe=acq prefix=k0 acquisition=two_rounds rollback=per_written post_mount_checker_red_states_by_invariant={}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-two_rounds.txt:17`）。I-7.7、I-7.15、I-7.16 与择系统配置都在这一套里，取号之后两槽世代号差一、槽号是世代号 mod 2，I-7.16 的「两个槽都可择时世代号相差不超过 1」照旧成立。
- 状态数：「probe=acq_counts prefix=k0 acquisition=one_round rollback=per_written acquired=2 writes=38 sysconf_writes=6 segments=9 closed_form_states=20496 states_with_torn=20511 segment_shape=2s2,14s0,2s0,1s0,2s2,12s0,2s0,1s0,2s2」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-one_round.txt:1`），「probe=acq_counts prefix=k0 acquisition=two_rounds rollback=per_written acquired=2 writes=40 sysconf_writes=8 segments=10 closed_form_states=20499 states_with_torn=20519 segment_shape=2s2,2s2,14s0,2s0,1s0,2s2,12s0,2s0,1s0,2s2」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-two_rounds.txt:1`）。多出的就是一段「每块盘一次原地覆写」：闭式 +(2^D − 1)，带撕裂态 +(3^D − 1)，D 是盘数；两块盘 +3 与 +8。崩溃点重放里每条含可写挂载的流各多这么多（推的：按段的闭式算，其余流没逐条数）。
- 牵连面里我没量的：世代号整体加 1 之后写死世代号的用例与 layout 字节表（第一轮正推列的 9 个文件）。我只跑了自己的原型，没跑现有 harness 用例。

## T2　形态甲之上的离线出路 α

α 的条文只有「调用方断言版本，只重写不自证的槽，写的实例代号取各盘自证槽、根环、journal 环的最大值」。我在原型里照它实现（`probe-crate/src/probe_alpha.rs` 文件头写了取法）：世代号取同盘另一槽加一，tail 取断言那一版的位置；`checked` 一臂先读断言那一版的根槽，读不出或不是它就一个字节不写，另一臂不查。journal 环那一项没读：两个盘面上环里都没有比根环、系统配置更高的实例（推的）。这是我对条文的读法，条文没写「查什么」，所以下面凡涉及「查」的结论都只对这两种读法。

### 打中一：α 落在今天的一轮上照样撞号（量过，副本）

盘面（第一轮打中一那一形，换成全量取样里的一个状态）：起点 `k0`，可写挂载取 2，切在第一条 journal 记录写之前，两块盘带实例 2 的那一槽持续读错、写之后读得出（`remap`）。
「probe=alpha_c555 acquisition=one_round rollback=per_written behavior=remap before_alpha=refused NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir alpha=wrote_2_slots instance=1 after_alpha=mounted instance=2 effective_txg=3 collides_with_units_of_instance_2=true checker_red=[]」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-one_round.txt:2`）。
形态甲上同一个盘面：「probe=alpha_c555 acquisition=two_rounds rollback=per_written behavior=remap before_alpha=refused NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { fir alpha=wrote_2_slots instance=2 after_alpha=mounted instance=3 effective_txg=3 collides_with_units_of_instance_2=false checker_red=[]」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-two_rounds.txt:2`）。
所以 α 不撞号只因为形态甲把新号写进了另一槽；α 自己的「取最大值」读不到只写了单元的实例。两者是绑在一起的：α 不能先于形态甲单独上。

### 打中二：持续读错、写不进去的槽，α 之后仍出不了只读（量过，副本）

两条臂结局相同，形态甲那一份：
- 写之后照样读错：「probe=alpha_c555 acquisition=two_rounds rollback=per_written behavior=persistent before_alpha=refused NewerStateStillUnreadableAfterOneReread…alpha=wrote_2_slots instance=2 after_alpha=refused NewerStateStillUnreadableAfterOneReread…collides_with_units_of_instance_2=false checker_red=[]」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-two_rounds.txt:3`）。
- 写不进去：「probe=alpha_c555 acquisition=two_rounds rollback=per_written behavior=unwritable before_alpha=refused NewerStateStillUnreadableAfterOneReread…alpha=write_failed_after_0_slots after_alpha=refused NewerStateStillUnreadableAfterOneReread…collides_with_units_of_instance_2=false checker_red=[]」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-two_rounds.txt:4`）。
C596 的盘面本身就是「持续读错」（第一轮用的就是每次读都报错的那一种）。α 只重写，重写不改变「这一槽报读错」，R2 照旧判不出。形态甲之后两槽内容相同（取号两写带同一个见证 tail），读错那一槽藏着的不比另一槽多，但 R2 的式子只数「有槽报读错」：「.any(|slots| slots.slots_reporting_a_read_error > 0)」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4311`），看不到这一点。
四句：分辨臂（只中「只重写、不改 R2」的 α；推的改法「α 之后把那一槽记成报废、R2 不再数它」没实现）；系统看得到读错，看不到读错那一槽原来是不是取号写；判据第四种（单故障之后永远只读）；改法见上，零轮。

### 打中三：断言错一位（新一版），不查的 α 把可恢复的盘面写成永远拒（量过，副本）

盘面：起点 `k2`（最新版 txg 5，根住盘 0），每盘最新那一槽读错（写之后读得出），txg 5 的根槽持续读错，α 之后再可写挂载；最后把读错全撤掉再挂一次（根「一时」读不出之后恢复）。
- 不查、断言 T+1：「probe=alpha_r2_closed acquisition=two_rounds rollback=per_written assertion=T_plus_1 checked=false before_alpha=refused NewerStateStillUnreadableAfterOneReread…alpha=wrote_2_slots instance=1 mount_root_unreadable=refused NewerStateStillUnreadableAfterOneReread…then_root_readable_mount=refused NewerStateStillUnreadableAfterOneReread…checker_red_end=[] took_instance=None」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-two_rounds.txt:11`）。读错撤掉之后照样拒：α 写下的见证 tail 指向一条不存在的记录。
- 同一盘面不跑 α，读错撤掉之后挂得上（`checked=true` 一臂 α 拒写，后面那次挂载取 2、txg 5：同一文件第 6、10 行）。
- 查根环、断言真实的 T：α 拒（那一版的根正读不出），「alpha=refused_root_of_the_asserted_version_unreadable」（同一文件第 6 行）。断言 T−1：两臂都写（第 8、9 行），这一盘面上没丢 txg 5（下一段）。
四句：分辨臂（只中「不查」的读法；「查根环」的读法不中这一格、但拒掉了真实断言）；系统看得到断言那一版的根读不出，看不到它是「读不出」还是「不存在」——除非再去 journal 环里找那一版的记录；判据：派发提示「断言错一位时会不会一个字节都不写」，答案是不查就写；改法：查断言那一版「根或记录有一样读得出、且是它」才写（推的，没实现、零轮）。

### 没打中：R2 收掉的那一形，在这一个起点上没丢确认过的写（量过，副本，一个起点）

同一盘面，根读不出时挂载照样拿到 txg 5：「probe=alpha_r2_closed acquisition=two_rounds rollback=per_written assertion=T_minus_1 checked=true before_alpha=refused NewerStateStillUnreadableAfterOneReread…alpha=wrote_2_slots instance=1 mount_root_unreadable=mounted instance=2 effective_txg=5 then_root_readable_mount=mounted instance=3 effective_txg=7 checker_red_end=[] took_instance=Some(2)」（`research/prompts/m3-close-sysconf-r2-opus-model/out/alpha-two_rounds.txt:8`）。txg 5 那次发布的 journal 记录还读得出，恢复由记录重建那一版的根（D23（journal 的角色与格式） 已定项 15 那一条），所以「最新根一时读不出」单独不够丢写，还要那条记录也读不出。我没造「根与末条记录同时读不出」的盘面（三个故障），所以 R2 收掉的那一形里 α 会不会把拒变成丢写，这一轮没量，算没打中、不算排除。

## T3　形态乙

今天缺单元那一半只比 tail：「if newest_system_configuration_journal_tail >= version_journal_position {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5802`），落后的才逐个读「那一版」的单元，单元清单是那一版那次发布写下的单元（空字节的跳过：「.filter(|unit| !unit.bytes.is_empty())」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5807`））。形态乙把前一判换成「本盘环槽里有那一版末条记录」；第一轮攻方的臂（`research/prompts/m3-close-sysconf-r1-opus-model/tools/patch_behind_arms.py`）按 (实例代号, 计数器) 认那条记录。

### 三种盘面（推的；第一轮量过的 41 条照引，没重跑）

| 盘面 | 形态乙怎么判 | 结论 |
|---|---|---|
| 环转过一圈 | 那一槽坐着下一圈的记录，计数器不等，判落后，接着读单元 | 不放宽，与今天同 |
| 记录只在一块盘上 | 没有记录的那块盘判落后、读单元；单元写在记录之前那道屏障之前，根已落就说明单元已落，缺只能是损坏 | 与今天同 |
| 旧快照在发布途中取下 | 单元与记录都在、根没落：缺单元那一半放行，缺根那一半照旧拒（第一轮量过：`record_units` 41 条全绿） | 不中 |

### 打中一：「缺的份数不到 2 不算缺」放行最新那次发布写到一半时取下的快照（推的）

盘面：最新那一版 P 的发布写了 k ≥ 2 个单元，同一段里没有屏障，崩溃点重放的全量里有「k 个里落了 k−1 个」的状态（今天的取号流就有 14 次单元写在同一段，`segment_shape` 的第二段 `14s0`）。某块盘在这样的状态下被取下（快照），之后池在另一块盘上照常做完 P（根住在另一块盘上），快照盘再换回来。
- 今天：落后（tail 在 P 之前），读 P 的单元，缺 1 份，拒。
- 加上「缺的份数不到 min(2, 单元总数) 不算缺」：缺 1 份 < 2，缺单元那一半放行；缺根那一半只在 P 的根写在这块盘上时才读（第一轮判决 S2 那一行），根在另一块盘上，放行。
- 结局：一块没有任何故障、只是旧快照的盘被当成跟上了，P 的那一份单元从此只剩一份，再来一个单故障就丢；这正是 C585 要拒的「旧快照」，只是取下的时刻早一点。
四句：分辨（只中加了「份数不到 2」的那一臂）；系统看得到「缺 1 份」，看不到它是坏了一份（C593）还是根本没写到（旧快照），两种情形在单元那一半上逐条相同——要分只能再读记录（形态乙那一判本身：快照取在单元写到一半时，末条记录必定还没写）；判据：派发提示「会不会让 C585 的旧快照重新放行」；改法：「份数不到 2」只在「本盘环槽里有那一版末条记录」成立时才用（推的，没实现，零轮）。

### 打中二（从没被看过的地方）：形态乙按 (实例代号, 计数器) 认记录，而今天的取号会重发同一个实例代号（推的）

形态乙的那一判假定 (实例代号, 计数器) 认得出唯一一条记录。今天的一轮在 C555 那一形上撞号（上面 T1 量过：每个起点 20511 个状态里 16386 个），同一个实例代号会写出两段历史；第二段从环里读得出的最大计数器加一起编号（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:8074` 那一处），第一段只写了单元、没写记录，所以在 C555 那一形上两段的记录键不重。要让两段键相同，第一段得写过记录、之后那几条记录在每块盘上都读不出——这要再加故障，单故障下我没造出来。所以这一格只记成依赖：形态乙的正确性依赖实例代号不重发，应当与形态甲一起定，不能单独先上。

### 坏副本没被补写那三条红该落在哪（推的）

第一轮量过：C593 形挂上之后红 I-2.1、I-4.8、I-7.4，与「系统配置跟得上、只缺一份单元」的对照逐条相同。红的原因是可写挂载与之后的发布都不重写一份读得出而校验和不对的单元副本，这与落后判法、与系统配置都无关，形态乙的每条臂都一样（不分辨臂）。它属于「读到坏副本之后补写或隔离」那一条线，第一轮判决写的是 D19 读盘核隔离单份副本那一侧；我同意另记一笔，不拿它判形态乙。

## T4　形态丙

闸：本盘带所选那一版的全部单元（清单非空、逐个读到、字节与验得过的那一份相同），根槽那一半不缺；补写失败按取号失败回卷。

| 盘 | 闸怎么判 | 结论（推的） |
|---|---|---|
| 空盘 | 单元读不到 | 拒，不中 |
| 别池的盘（fsid 不同） | 单元字节对不上（头里有 fsid） | 拒，不中 |
| 两块盘对调了位置 | 换过去的那块盘若两槽全废、单元又恰好在同一槽号上，闸可能放行；但另一块盘的系统配置自证得过、它记的本盘设备号与位置对不上，挂载在别处拒（`OwnDeviceNumbersDifferFromTheDeviceTable` 那一判） | 不中（要两块盘都两槽全废，是两个故障） |
| 同 fsid 的克隆盘：从本池 dd 或虚机快照克隆出去、在别处可写挂载过的那块盘，所选那一版之后它自己又写过 | 所选那一版的单元在克隆盘上（COW 没覆写它们），那一版的根若住在另一块盘上，根槽那一半不读这块盘：闸放行 | 打中 |

### 打中：克隆盘带着同一版全部单元，闸放行，补写之后克隆那一边的根与记录成了本池可见的（推的）

历史：本池停在版本 V；它的一块盘被整盘克隆（同 fsid），克隆在别处挂上、取号、发布了几次（实例 N+1，txg 大于 V 的根与记录写在克隆盘的根环与 journal 环里）；本池那块盘两槽系统配置坏了（单故障），管理员把克隆盘换进来，它两槽也恰好坏了（或被当成坏盘清过系统配置那一段）。
- 闸只问「这块盘有没有所选那一版」，不问「这块盘上有没有比所选那一版新、却不属于本池历史的东西」。补写之后这块盘「可见」，它根环里那几条 fsid 相同、自证得过的根就进了择根，journal 环里实例 N+1 的记录进了扫环；本池下一次取号是 max(…, 根环里的 N+1) + 1，与克隆那一边分开不了。
- 今天：这块盘两槽全废，取号之前那一判就拒（第一轮引过的 `transaction.rs:893` 一段），克隆那一边的东西进不来。
四句：分辨（只中加了闸的形态丙；今天拒）；系统看得到判别子：这块盘的根环与 journal 环里有没有比所选那一版新的根、记录（同 fsid 而不在本池那块好盘上），前提是去读；判据：派发提示「对调的盘恰好带着同一版全部单元的那一形」；改法：闸加一条「这块盘的根环与 journal 环里没有比所选那一版新的根与记录」（推的，没实现，零轮）。这一形要两件事（一次单故障，加一次换进克隆盘的运维动作），与 C585 的旧快照同一类「换盘」事件。

### 补写之后到第一次轮换之间的池级 checker

没量：形态丙没有在副本里实现。推的：补写写的是所选那一版的见证 tail 与现行实例，与取号写同一份内容，I-7.7 的 ② 句「各盘的实例代号不等」在第二轮补写写完之前成立、由那一句收；I-7.16 只看自证槽之间的世代号差，补写那块盘两槽从全废变成一槽自证，这一判不适用。

### 与形态甲叠加

形态丙的「补写失败按取号失败回卷」若沿用形态甲逐份回卷，T1 打中的那一格照搬过来：补写之后再一次写报错、回卷连写同一块盘两槽、同段断电，回到两槽全废（T1 量过，叠加推的）。

## T5　S3、S6 再抽一次

### S3「自证不过」：单故障加一次断电，没打中撞号（撞号量过，丢写没量）

故障臂 `fsid/d<盘>s<槽>`：一块盘的一槽换成 fsid 改了一个字节、重封校验和的系统配置（读得出、整槽校验和过、fsid 不是本池），在取号流的每个崩溃状态上各做一次，四个位置、两条臂、三个起点。结局全是 `formula_above_acquired`，例如今天的一轮：「probe=acq prefix=k0 acquisition=one_round rollback=per_written fault=fsid/d0s0 outcomes={"formula_above_acquired": 20511}」（`research/prompts/m3-close-sysconf-r2-opus-model/out/acq-one_round.txt:7`）。单故障只盖掉一块盘的一槽，另一块盘带新号的那一槽还在，取号式子看得到；只有两块盘同一时刻都失去带新号的那一槽（C555 的两个故障）才撞。
丢确认过的写：我的原型没有「哪一版已确认」的判据（取号流里只有挂载自己的几次空发布与写行发布），这一半没量。推的：要丢写还得最新根与它的记录同时读不出（T2 最后一段量到单是根读不出时记录照样重建出那一版），超出单故障。

### S6「不收」：没有读故障的崩溃状态里不出现（推的）

新实例第一条记录的计数器取扫环读得出的最大计数器加一：「let next_counter = records」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:8074`）。扫环两块盘每个记录槽都读，读得出、解得开的都进表，不按前缀截：「if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:4351`）。checker 判别的实例的那一格：「Some(_record_of_another_instance) => ExpectedBackChain::Value(0),」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker/src/walk.rs:9058`）。
要 checker 在新实例 B 的记录 (B, k) 上红，某块盘计数器 k−1 那一格要坐着别的实例 A 的记录，而 B 自己从不大于 k−1 的计数器起写。B 从「读得出的最大计数器 + 1」起，那条 A 的记录在 B 挂载时若读得出，B 的起点就大于 k−1，矛盾；所以它在 B 挂载那一刻必须读不出。没有读故障时读不出只有两种：撕裂（之后不会变得读得出）、根本还没写（A 在 B 之前，不会在 B 之后才写）。环转过一圈的旧记录计数器差一整圈，checker 按计数器找前一条找不到它（第一轮已列）。所以没有读故障的崩溃状态里这一格不出现。
没跑的原因：要造这一形得有跨两条记录的发布（一条记录装 64 个点名项，经会话发布要 65 个以上单元），那一段单元写在同一段里，全量状态数是 2^130 量级，远超一段约 10⁶；写入口上压小点名项的开关只在直接用 `PoolWriter` 时装得上，挂载与会话自己开写入口，装不上。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| T1 | 两轮之间崩、第二轮写一半崩、取号之后到第一次轮换之前每盘一槽翻坏或持续读错、单块盘一槽 fsid 不符 | 起点 `k0`、`k2`、`r1`，每个起点可写挂载写流的全部 20519 个崩溃状态 × 12 条故障臂；撞号只在取号式子不大于本实例号时才真挂载判 |
| T1 | 回卷每盘一次；回卷写本身报错（`_onward`） | 起点 `k0`、`r1`，第 1 到第 4 次系统配置写报错，各自写流的全部崩溃状态 |
| T1 | 新中间态被现有判法判红 | 同上全部状态的原始镜像与挂上之后的镜像，池级 checker 全部条目 |
| T2 | 形态甲之上 α 撞号 | 一个盘面（`k0`，切在第一条记录之前），三种盘的行为 |
| T2 | R2 收掉的那一形丢写 | 一个起点（`k2`）、根读不出，断言 T、T−1、T+1 × 查与不查 |
| T3 | 环转圈、记录只在一块盘上、发布途中快照 | 推的；第一轮量过的 41 条 |
| T4 | 空盘、别池的盘、对调的盘 | 推的 |
| T5 | S3 单故障撞号 | 与 T1 同一套状态，fsid 臂 |
| T5 | S6 没有读故障时出现 | 推的（代码读法），没造 |

放开的用户步骤：挂载之前的历史（直接崩、再覆盖写两次、崩后挂一次再写一次）；挂上之后用户再写一次那一个起点因状态数超过约 10⁶ 没跑；挂载之后的读写挂载顺序（先只读挂载再可写挂载）没扫，推的：只读挂载不写盘，状态不变。

## 这条腿自己的限度

- 只在一档几何上量：两块盘、单元区 384 槽的小盘。没扫三块盘以上（取号写两轮的段宽随盘数 D 涨，撕裂态按 3^D）。
- 故障臂只在式子算出的号不大于本实例号时才真做可写挂载；式子更大的那些格记成「不撞」，没核它们是不是挂得上（持续读错那几臂在形态甲下多半照样拒，T2 打中二是其中一格）。
- 「撞号」只认「挂上、取到的号不大于这一次录的实例号、且这个状态里落了本实例的非系统配置写」；更早的实例被重发我没判（三个起点都没造出更早实例被重发的前缀）。
- α 的「查」是我选的读法（读断言那一版的根槽）；条文没写查什么。
- 形态乙、形态丙没有在副本里实现，T3、T4 全是推的。
- 我提的改法（形态甲回卷每盘一次或两次回卷写之间加屏障；α 只在「根或记录读得出且是它」时写、写后把读错槽记成报废；形态乙的「份数不到 2」只在本盘有末条记录时用；形态丙的闸加「没有比所选那一版新的根与记录」）只在我的模型上量过或推过，被攻过零轮。「每盘一次」量过；其余推的。

| 改法 | T1 逐份回卷 3 格 | T2 不查的 α 断言 T+1 | T2 持续读错 | T3 份数不到 2 放行快照 | T4 克隆盘 |
|---|---|---|---|---|---|
| 回卷每盘一次 | 不中（量过） | 不涉及 | 不涉及 | 不涉及 | 补写失败那一叠加不中（推的） |
| α 查根或记录 | 不涉及 | 不中（推的；只查根那一读法量过拒 T+1，也拒了真实的 T） | 照中（推的） | 不涉及 | 不涉及 |
| α 写后报废读错槽 | 不涉及 | 不涉及 | 不中（推的） | 不涉及 | 不涉及 |
| 份数不到 2 只在有末条记录时用 | 不涉及 | 不涉及 | 不涉及 | 不中（推的） | 不涉及 |
| 闸加「没有更新的根与记录」 | 不涉及 | 不涉及 | 不涉及 | 不涉及 | 不中（推的） |

## 复跑记录

`rerun.sh` 在新草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r2-opus/rerun-work` 端到端跑过一次（整条经 `run-with-memory-cap.sh 2G`，排队 100 秒后起跑；`PROBE_SEED_TARGET` 指到本机另一个同 `Cargo.lock` 的 release 编译目录，只拷了第三方产物，singlefs 各包与探针在新目录里现编），末尾原样：

```
name=rerun_done work=/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r2-opus/rerun-work compare_with=/home/fy5090/code/singlefs/research/prompts/m3-close-sysconf-r2-opus-model/out/<名>.txt
run=acq-one_round same
run=acq-two_rounds same
run=rollback-one_round same
run=rollback-two_rounds-per_written same
run=rollback-two_rounds-per_device same
run=alpha-one_round same
run=alpha-two_rounds same
```

编译：探针依赖 checker 档包（崩溃点重放的计划在那里），所以是 `cargo build` 出二进制、经内存包装直接执行，没有 `cargo test` / `cargo run`。2G 上限下 `-j 6` 撞了上限（`run-with-memory-cap: 撞了内存上限 2G（… Result=oom-kill，命令退出码 143）`，草稿目录 `build1.log`），之后一律 `-j 1`。第一次数状态时内存总池占满，排队十多分钟，我按进程号逐个停掉了自己那条排队的命令（`proc.py stop`，自下而上，停完再列一遍确认没剩），之后合成一条脚本一次排队跑完。

## 没做什么

- 不判辩方那几格（T6）；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没读禁读清单里的文件。
- 没跑 checker 档包的任何测试、没跑重型测试、没跑现有 harness 用例；只编、跑了自己的原型。
- 没实现形态乙、形态丙，没造 S6 的盘面，没扫三块盘以上与别的几何；`k0_pub` 起点（挂上之后用户再写一次，1 069 098 个状态）没跑。
- 交回前删了：草稿目录里的仓副本 `repo/`（34M）与编译目录 `target/`（6.6G），复跑目录里的仓副本 `rerun-work/tree/`（34M）与编译目录 `rerun-work/target/`（6.6G）。草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r2-opus/` 只留日志、`meas/`、`rerun-work/` 的输出与两份驱动脚本，共 388K；`meas/*.txt` 与模型目录 `out/` 逐份相同（`out/` 就是从它拷的）。
