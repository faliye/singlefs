# m3-unlink-code-r2 云端攻方（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers N1 N2 N3 N4 N5 N6 U9 U10 U11 U12 U13 U14 A1 A2 A3 A4 A5 G1 G2 G3 MU1 MU2 MU3 H1 H2 -->

攻击面：正文第四节分工表云端攻方那一行，K2，重点 N1、N3、N5；N2、N4 只攻到与 N5 相连的那一段。原型全在冻结副本的拷贝 `/tmp/claude-1000/m3-unlink-r2-attack/tree/` 上编、跑，只跑 harness 档（`-p singlefs-harness --test attack_r2_*`、`--release`），checker 档没跑；N5 那几个私有函数从 `crash_injection.rs` 原样抄进 harness 档的原型里调。副本上量出的数不是入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-unlink-code-r2-opus-model/`。复跑（工作目录是冻结副本的一份新拷贝 `X`）：

```
rsync -a --exclude target --exclude .git /tmp/claude-1000/m3-unlink-code-r2/tree/ X/
M=research/prompts/m3-unlink-code-r2-opus-model
mkdir -p X/crates/singlefs-harness/tests/attack_r2_support
cp $M/copied_from_crash_injection.rs X/crates/singlefs-harness/tests/attack_r2_support/copied.rs
cp $M/attack_r2_mount_continuation_probe.rs $M/attack_r2_history_probe.rs $M/attack_r2_random_probe.rs X/crates/singlefs-harness/tests/
# 随机历史那一份要执行器副本里的两行 eprintln：
(cd X/crates/singlefs-harness/src && patch history.rs < $M/history_rs_instrumentation.patch)
cd X
# 一、N5 取界与真值（全网格，不注错）：
ATTACK_SWEEP=0 bash research/scripts/run-with-memory-cap.sh 1G bash research/scripts/capped.sh 6 \
  cargo test --release --offline -p singlefs-harness --test attack_r2_mount_continuation_probe -- --nocapture
# 二、注错扫（N2、U12）：同上，ATTACK_SLOTS=256 ATTACK_OVERWRITES=2 ATTACK_UNITS=14 ATTACK_PRODUCT=0 ATTACK_SWEEP=1
#     与 ATTACK_SLOTS=192 ATTACK_OVERWRITES=0 ATTACK_UNITS=14 ATTACK_PRODUCT=0 ATTACK_SWEEP=1
# 三、穷举后缀：ATTACK_EXECUTIONS=4G|256|240-skip… ATTACK_OVERWRITES=… ATTACK_EXHAUSTIVE_LENGTH=1|2，--test attack_r2_history_probe
# 四、随机历史：bash $M/rand-batch.sh <批名> <盘宽> <attack|wall|broad> <十进制种子基> <种子数> <步数>（脚本里写死了草稿目录的路径，换成 X）
# 五、变异 MU1 / MU2：python3 $M/mutants_u9_ledger.py MU1（脚本里写死了草稿目录的路径），再跑
#     --test unlink_mount_continuation_and_the_emptied_version --test admission_raises_the_floor_before_refusing 与「二」
```

内存包装在这台机上要 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（同会话还有别的腿在跑），上限取 480M–2G；日志都在 `logs/`。

```
8b8e768c1817f2f3ad0c92b59f4e420dee813909f2b2406c479ebb5de3fcaf64  attack_r2_history_probe.rs
74507f3fab9920800169ee6fe01d5eadcc7b2604829d8efbe9a6bf8390e7a283  attack_r2_mount_continuation_probe.rs
0f1d592bbab9c3f5e8d145332c2acd018b65c16f9bbe9a9633512aefcaf13131  attack_r2_random_probe.rs
cee0031a0ac89068e4b7b8d9b4ffa7048971f5ed8ccb3eb912c193a334f8018e  copied_from_crash_injection.rs
984a5cd956c485637a39d533ea902fd760fc43de8c32dce518e0ef7c3d83055b  history_rs_instrumentation.patch
a46959adab2764bd74f3899d704dab6cd26e5751ef254580d2a0032f78a81db4  mutants_u9_ledger.py
c63b2412cff497a311b64699ed491819b7acb74df8ae00139dd214540e6e16fe  rand-batch.sh
36b2a6e9b1ba00419217ae2d9d95aa46927fb937e24a5a836e701aed6bb04592  logs/grid-skip.log
06d22954b078af9a62afd0157e367238ca3ddd8baa3d42a7a13327a216656176  logs/grid-u14.log
cd38906f6de873f4855e59b4653cf2943612c0c956ef708ff2661dd49d7b3848  logs/hist-4G.log
f0b297db465bad828493b90e05524097af9e416c051302ddf9f92dcc6fc9af1b  logs/hist-k.log
d2094f1ed5a81eef92b1a552a3d78e41cab3174179fb1ad7a22a09d4163b565f  logs/hist-narrow-t.log
6b33dfbd7e18c8460c5fbb785f35964daed11be3d5b40f0ca8821ea71452a02d  logs/hist-wall.log
6e433bcbfc87230313270b7aaadb20db7443ced925072e9d87854a34e59f7fe4  logs/mu1-probe.log
75fdb653a2aafd20023030e1c4171a1a58c8fa2b90312ea30309ec03eb3f5ff9  logs/mu1-tests.log
3b39c74d422a9c08d3763862d431e4730a8d1d441b3fc7b54b2fed5d3f1ee557  logs/mu2-probe.log
1c57abab190b471f82526c924d595ca5f89fc1a653e9ba7c7694aa57908942d2  logs/mu2-tests.log
1309fb216799bc0aea855d9ac450d0c8ebf5f43d6e32a0353a5d0d71728dada0  logs/rand-b1.log
69f201b8f3e19bb81ac2fa43964bd7573a5c5a305b6e191c8fff2f9345d54d87  logs/rand-b2.log
3757ed600adc2a78473a4613377bdd998d3a5254e63a6306b3c6a24e2ad80488  logs/rand-b3.log
4ecae86dac23b9a9ce7dbc20090e0696ab6acb966b6c62596cf0a4c1140c2bef  logs/rand-t.log
8cf427eed5943e2db102e0c8e8da9ab6950c0379b7003b00edbbc2ba6fa246b8  logs/rand-w1.log
aecaf960cb9d6e1b3846f43f97de3a4b31d3fc0b82fd939478249fe03ab41735  logs/run3.log
71f593fd7a95d4b422c5be9648071a0aea50d889d2856cd4eedd2d1f016df447  logs/sweep2.log
901653a24fe9439b4ab38a6d6666568968a99765164206143394c7dce2eb9258  logs/sweep3.log
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据 |
|---|---|---|---|
| N5 取界（准入推的、续做推的、两者都推） | 没打中（一次抽样） | 真实录制流上抄来的取界与「第一个带新 F 的系统配置槽写」逐个相等 | 2456 个不同的起点（崩溃镜像 × 挂载开关），48 个续做里推过抬 F（其中 38 个准入也推过）；第一节 |
| N5 U12 划段：续做里抬 F 自己报错那一支 | **打中（量过，只在故障注入下摆得出）** | 错原样交回、不带挂载写账，`crash_injection.rs` 交 None，已落盘的续做各步的写全归暖机段，一次最多 334 个写 | 第一节 H2 |
| N5 B 自报「续做开头先推的抬 F 归暖机」 | 没造出那一形 | 2456 个起点里续做推的抬 F 一次都不在第一步之前 | 第一节 |
| N2 续做交回 `Publish` 时的写账 | **打中（量过）** | 每推过一串抬 F（准入推的也算），错里少那一串先写系统配置的 2 个写；A 只自报了续做推的那一半 | 第二节 H1 |
| N2 抬 F 自己报错那一支的写账 | 打中（量过，A 已自报） | 错里只有抬 F 自己的写（2–39 个），挂载已落盘的 348–386 个写不在 | 第二节 H2 |
| N2 用例注释「份数等于盘上落的发布数」 | 打中不分辨本批（量过） | 注错点落在一次发布的记录之后，只读恢复靠记录多走一次，份数比盘上少 1 | 第二节 H4 |
| N4 U9 用例的判别力 | **打中（量过）** | U9 唯一的用例没推抬 F：写账里删掉准入推的那几次（MU1）或续做推的那几次（MU2），两份 harness 测试照样全绿 | 第三节 H3 |
| N3 删除中的一版上的用户改动 | 没打中（一次抽样） | 854 段不同的历史跑完、模型全对得上；删除中的一版上覆盖写、unlink 拒成 `NoFileVersionToChange`、建 inode 做成都走到了 | 第四节 |
| N1 挂载续做推满仍放不下之后的会话 | 没打中，走不到 | 执行器的历史里一次都没走到 `StillShortAfterTheFloorRaises`，会话走空也一次都没被空间拒；窄盘原型里只有 8 个、14 个单元的文件才停得住，模型只罩一个单元 | 第四节 |
| N4 别的新用例、`KNOWN_RED_FORMS`、LR5、N6 | 没攻 | 时间给了 N1、N3、N5 | 「这条腿自己的限度」 |

## 一、N5：抬 F 的界与续做段

### 取界（没打中）

造法（`attack_r2_mount_continuation_probe.rs`）：不补足环窗口的窄盘（入库用例 `admission_raises_the_floor_before_refusing.rs` 的同一个起池函数），unlink 之前覆盖写 k 次、顺序写把文件长到 u 个单元，这个进程里发意图与 b 批（批大小 1）就退出；重开可写挂载（准入判着或关掉、删除批 1 或产品批），录下挂载那一段流，按建池参数的几何切写表，拿抄来的 `floor_raises_in_the_mount_writes_counting_the_continuation` 取界。真值不经实现的账：写表里第一个所带 F（系统配置槽偏移 481）高于取号那一次系统配置写所带 F 的系统配置槽写。

放开扫的用户动作：单元区 176–352 槽（步长 8）与 192/224/256/288/320 两套、覆盖写 0–10 次、文件 1/2/4/8/14 个单元、崩在第 0、1、u/2、u−1 批之后、挂载删除批 1 或 144、准入判着或关掉。

| 网格 | 行数 | 取界判定 |
|---|---|---|
| `logs/run3.log` | 978 | 「SUMMARY n5_lines=978 n5_hits=0 sweep_lines=0」 |
| `logs/grid-u14.log`（14 个单元，跑到 1158 行时被我外面套的 `timeout 235` 停掉，见限度） | 1158 | 808 个 ok-located、350 个 ok-not-pushed |
| `logs/grid-skip.log`（准入关掉） | 540 | 「SUMMARY n5_lines=540 n5_hits=0 sweep_lines=0」 |

前两张网格去重之后 1916 个起点，48 个续做里推过抬 F（38 个准入也推过），取出的界与真值逐个相等；没有一个 `NotLocatedInTheMountWrites`。每个起点上设备一层数到的写调用数与写表条数相等（`attempts_equal_landed=true` 全部）。准入关掉那 540 个起点上续做一串抬 F 都没推（79 个停在推满仍放不下、停因都是抬 F 那一串被空间拒），B 自报的「续做开头先推的那一串」那一形在三张网格里都没出现。

### 续做里抬 F 自己报错那一支（打中，H2）

造法：同一份崩溃镜像上可写挂载，整池第 n 个写调用报块设备错、不落盘，n 从 1 扫到没崩过的那次挂载的写调用总数（`FailingNth`，每个写调用都算，写零也算）。两个起点：256 槽、覆盖写 2 次、14 个单元（`logs/sweep2.log`，崩在第 0、1、7 批之后，1056 个注错点）；192 槽、覆盖写 0 次、14 个单元（`logs/sweep3.log`）。真值的续做起点 = 没崩过那次挂载的写行 + 暖机 + 准入推的空发布次数。

续做里推的那一串抬 F 自己落盘途中报错时，续做原样交回那个错（`crates/singlefs-core/src/mount.rs:4476`「return Err(cause);」，文档 `crates/singlefs-core/src/mount.rs:4380`「/// 抬 F 自己报了不是空间不够的错：那个错原样（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）。」），挂载的错是 `RaiseFloorSequencePublishFailed` 或 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`；崩溃注入只认 `MountError::Publish`（`crates/singlefs-checker-tier/src/crash_injection.rs:1647`），别的错走 `crates/singlefs-checker-tier/src/crash_injection.rs:1653`「Err(_) => None,」，续做段一个都划不出来，写行之后全归暖机——就是 r1 攻方 A4 那一形，U12 只修了 `Publish` 那一支。

`research/prompts/m3-unlink-code-r2-opus-model/logs/sweep2.log:361`：「n=353 in_continuation=true RaiseFloorSequencePublishFailed persisted=0 landed_roots=14 ledger=2 landed_minus_acq=350 partition=None(crash_injection Err(_)) truth_partition=Some(3) disk_publishes=Some(14) mislabeled_continuation_writes=297」

| 起点 | 注错点落在续做推的抬 F 里 | 续做的写被归进暖机段的注错点 | 一次最多归错几个写 |
|---|---|---|---|
| 256 槽、覆盖写 2、14 单元、崩在第 0 批之后 | 40（38 个 `RaiseFloorSequencePublishFailed`、2 个 `RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot`） | 40 | 334 |
| 192 槽、覆盖写 0、14 单元、崩在第 1 批之后 | 36 | 36 | 210 |

同一批里 `Publish` 那一支（续做的一步落盘途中报错）的划段与真值相等：256 槽那份 832 个、192 槽那份 713 个注错点里，每份只有 3 个点（每个崩溃起点一个：续做第一步的第一个写报错）交 None（那时续做一个写都没落，归错 0 个写）。

可达性：今天的崩溃注入战役在崩溃镜像上挂载用的是不注错的稀疏盘（`crates/singlefs-checker-tier/src/crash_injection.rs:1628`「mount_writable_with_space_admission(parameters, &mut devices, execution.space_admission);」），块设备错走不到；这两个错别的成因（预演与真发分叉、冻结着一次没重发的发布）在这条路上我没造出来。所以 H2 在今天的战役里是一格摆不出来的错划段，判它的是 U12 的验收字面「续做里交回错的那次挂载，续做的写落在 Continuation 段」（`research/prompts/m3-unlink-code-r1-main-verification.md:41`），不是一条已经出过错的战役产物。

## 二、N2：续做交回错时的写账与设备一层数到的写

同一批注错扫（`logs/sweep2.log`、`logs/sweep3.log`）。每个注错点上比三样：错里的写账合计（`writes_before_the_first_publish` + 已落盘各份 + 失败账）与设备一层落下的写减去取号那两次系统配置写（契约把取号那两次除外）；错里的已落盘份数与落下的根槽写数；只读挂载走到的那一版比崩溃镜像多几次。

契约：`crates/singlefs-core/src/mount.rs:720`「/// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。」；先写系统配置那一步有专门的一格，`crates/singlefs-core/src/mount.rs:725`「/// 这一串第一次发布之前、不属于任何一次发布、这个写入口已经发出去的写：抬 F 那一串先把新 F 写进每块盘系统配置那一步」。续做那一格交的是空账：`crates/singlefs-core/src/mount.rs:4445`「writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,」。

### H1：每推过一串抬 F，`Publish` 错里少 2 个写（打中，量过）

| 起点 | 推过的抬 F | 续做里 `Publish` 的注错点 | 设备落下的写 − 写账 |
|---|---|---|---|
| 256 槽、覆盖写 2、14 单元、崩在第 7 批之后 | 准入推 1 串、续做 0 串 | 169 | 169 个点全是 2 |
| 256 槽、覆盖写 2、14 单元、崩在第 0 批之后 | 准入 0 串、续做 1 串 | 368 | 抬 F 之前 295 个点是 0，之后 73 个点是 2 |
| 192 槽、覆盖写 0、14 单元、崩在第 1 批之后 | 准入 1 串、续做 1 串 | 319 | 175 个点是 2，续做那一串之后 144 个点是 4 |
| 两份里崩在第 1 批之后（256 槽）、第 0 批之后（192 槽） | 都没推 | 295、225 | 全是 0 |

`research/prompts/m3-unlink-code-r2-opus-model/logs/sweep2.log:937`：「n=131 in_continuation=true Publish persisted=6 landed_roots=6 ledger=126 landed_minus_acq=128 partition=Some(5) truth_partition=Some(5) disk_publishes=Some(6) mislabeled_continuation_writes=0」

`research/prompts/m3-unlink-code-r2-opus-model/logs/sweep3.log:557`：「n=286 in_continuation=true Publish persisted=13 landed_roots=13 ledger=279 landed_minus_acq=283 partition=Some(4) truth_partition=Some(4) disk_publishes=Some(13) mislabeled_continuation_writes=0」

差的正是每一串先把新 F 写进两块盘系统配置的那 2 个写：准入推的那几串，`writes_of_the_publishes_before_the_continuation` 只收它们的空发布（`crates/singlefs-core/src/mount.rs:4281`「.flat_map(|raised| raised.publishes.iter())」）；续做推的那几串同样只收空发布（`crates/singlefs-core/src/mount.rs:4350`）。实现员 A 自报的是「续做里推过抬 F 之后再出错时，每一串先写系统配置那一步的写不在错里」（背景材料正文第二节）；第一行那一格是准入推的抬 F，续做里一串都没推，照样少 2 个——自报只罩了一半。

### H2 的写账那一半（A 已自报，量过）

续做推的那一串自己报错时，错里只有那一串自己的写：`logs/sweep2.log:361` 上错里 2 个、设备落下 350 个（写行、暖机、11 步续做都落了盘），一直到这一串第 n 次空发布报错，错里 2–39 个、落下 348–386 个。A 自报过这一支（背景材料正文第二节「抬 F 自己报错那一支原样交回、不带这次挂载已落盘的写」），这里只补数。它与 H2 的划段是同一个病根：错里没有挂载的写账，崩溃注入想划也没东西可划。

### H4：「份数等于盘上落的发布数」只在用例挑的那个注错点上成立（打中不分辨本批，量过）

U9 用例的注释：`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:548`「/// 一次发布一份、按先后，份数等于这次挂载落在盘上的发布数（写行、暖机、续做第一步），每一份都有写（代码轮 U9，」。注错点落在一次续做发布的记录写之后（根槽写、轮换的系统配置写）时，只读恢复靠那条记录多走一次，份数比只读挂载看到的少 1：256 槽三个起点上依次 56/368、44/295、28/169 个点；其中一半是根槽写已经落了、轮换那一次系统配置写失败，这一次发布进了失败账而根在盘上。写账合计照样对得上（差 0），这是 `PublishSequenceFailed` 按「落盘阶段走完没有」分份的老口径，写行、暖机那一串一样，不是本批引入的；只是用例注释那句话写成了全称，用例只在「续做第二步的第一个写」那一个点上量（`crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs:599`「failing_write: writes_before_the_second_step + 1,」）。

## 三、N4：U9 用例的判别力（H3，打中，量过）

U9 只有一条用例，造法是 3 个单元的文件、4 GiB 级的池、续做第二步第一个写报错，挂载里一串抬 F 都没推。`crates/mutations.tsv` 第 2134 行的变异是「错里的已落盘写账照旧交空表」。另造两刀（`mutants_u9_ledger.py`），都只删写账里抬 F 那一段：

- MU1：`writes_of_the_publishes_before_the_continuation` 去掉准入推的那几串空发布（`crates/singlefs-core/src/mount.rs:4277-4283` 那一段 `.chain(…)`）。
- MU2：`writes_of_the_persisted_publishes_in_txg_order` 去掉续做推的那几串空发布（`crates/singlefs-core/src/mount.rs:4347-4351`）。

两刀下 `unlink_mount_continuation_and_the_emptied_version`（含 U9 那条 `a_mount_whose_continuation_step_fails_on_a_device_error_hands_back_the_writes_of_every_publish_it_landed`）与 `admission_raises_the_floor_before_refusing` 两份 harness 测试全绿：

- `research/prompts/m3-unlink-code-r2-opus-model/logs/mu1-tests.log:22`：「test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.49s」
- `research/prompts/m3-unlink-code-r2-opus-model/logs/mu1-tests.log:39`：「test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s」
- `research/prompts/m3-unlink-code-r2-opus-model/logs/mu2-tests.log:22`：「test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.45s」
- `research/prompts/m3-unlink-code-r2-opus-model/logs/mu2-tests.log:39`：「test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s」

同一刀下原型的注错扫红（份数比落下的根槽少、写账比落下的写少一大截）：

- MU1，`research/prompts/m3-unlink-code-r2-opus-model/logs/mu1-probe.log:897`：「n=92 in_continuation=true Publish persisted=2 landed_roots=5 ledger=36 landed_minus_acq=89」——准入推的那一串 3 次空发布、51 个写不在账里（原样是 persisted=5、ledger=87，`logs/sweep2.log:898`）。
- MU2，`research/prompts/m3-unlink-code-r2-opus-model/logs/mu2-probe.log:398`：「n=391 in_continuation=true Publish persisted=14 landed_roots=16 ledger=348 landed_minus_acq=388」——续做推的那一串 2 次空发布、38 个写不在账里。

第三刀 MU3（去掉按 txg 排序）我只推了：份数与合计都不变，崩溃注入划段只找第一个写续做结构的那一份，排序改不了它的结论；没有一个用例看次序，算等价，没跑。

别的读 `writes_of_persisted_publishes` 的 harness 测试（`commit_generated_units_fall_back_when_the_segment_runs_out.rs`、`a_zero_unit_publish_whose_post_rotation_barrier_fails_returns_the_mount_error.rs`、`core_review_unit_area_start_and_publish_limits.rs`）判的是写行与暖机那一串，没在 MU1、MU2 下跑（推的：它们的挂载没有续做，两刀碰不到）。

## 四、N1、N3：执行器不再跳过删除中的版本之后

### 穷举后缀（没打中）

`attack_r2_history_probe.rs`：起点「第一个文件之后」，前缀只固定造出删除中的一版那几步与覆盖写次数，后缀从 14 个操作里放开（覆盖写一个单元内 / 空内容、unlink、建 inode、可写挂载、冷启动、挂着时回退到第 0–3 新的根、抬 F 1 或 3 步、零单元发布、崩溃恢复抛弃根）：

| 前缀 | 造出的状态 |
|---|---|
| PA：覆盖写 k 次、unlink、建 inode、回退到第 3 新的根 | 回退到意图那一版，队列带回第一个文件（r1 攻方 A3 那一形） |
| PB：PA 再可写挂载 | 挂载续做把它走空 |
| PC：覆盖写 k 次、unlink | 窄盘上想撞最后一批被空间拒 |
| PD：覆盖写 k 次、unlink、崩溃恢复抛弃根 | 想让下一次挂载接着续做 |

| 日志 | 盘宽 / 准入 | k | 后缀 | 段数 | 结局 |
|---|---|---|---|---|---|
| `logs/hist-4G.log` | 4 GiB | 0 | 长度 ≤ 2 穷举（PA 全 211 段、PB 跑到 82 段时我停了，见限度） | 293 | 全 Completed |
| `logs/hist-narrow-t.log` | 256 判着、240 关掉 | 0、6 | 长度 ≤ 1 穷举 | 240 | 全 Completed |
| `logs/hist-k.log` | 240 判着、240 关掉 | 12、24、40 | 无 | 12 | 全 Completed |
| `logs/hist-wall.log` | 256 关掉、240 关掉、256 判着 | 60、100、140 | 无（只 PC） | 9 | 全 Completed |

4 GiB 那一批里 PA 之后第一步的结局（从日志按前缀长度切出来数的）：覆盖写 30 次全拒成 `UserChangeRefused::NoFileVersionToChange`、unlink 15 次同样、建 inode 15 次全做成、回退 60 次全做成、抬 F 30 次全做成；模型每一段都对得上。PD 的「崩溃恢复抛弃根」在跑过 PD 的两档窄盘（256 判着、240 关掉）上都被 C554 乙拒成 `NewerStateStillUnreadableAfterOneReread`，造不出续做。

### 按种子的随机历史（没打中）

`attack_r2_random_probe.rs`，比重往 unlink、回退、建 inode 偏（文件里 `UNLINK_NEAR_THE_WALL`），另跑入库的 `TOWARD_THE_UNIT_AREA_WALL`。执行器副本加了两行 eprintln（`history_rs_instrumentation.patch`）数走到了哪几支。

| 日志 | 盘宽 / 准入 | 比重 | 步数 | 段数 | 结局 | 删除中的一版上的用户改动 | 挂载续做 |
|---|---|---|---|---|---|---|---|
| `logs/rand-b2.log` | 256 关掉 | 攻方 | 40 | 120 | 全 Completed | 12 次拒成 `NoFileVersionToChange`、3 次做成 | 3 次，都走到了空 |
| `logs/rand-b3.log` | 240 关掉 | 攻方 | 60 | 120 | 全 Completed | 同上 | 同上 |
| `logs/rand-w1.log` | 256 关掉 | 入库的逼近单元区墙 | 150 | 60 | 全 Completed | 0 | 0 |

`rand-b1`、`rand-t` 两份的种子是 `rand-b2` 的子集（见限度：十六进制种子基没解析、落回了默认值），不另算。`rand-b2` 与 `rand-b3` 种子相同、盘宽与步数不同。去重之后两种合起来 854 段不同的历史，0 个新发现、0 次模型对不上。

### 空间不够的那几支走不到（N1 的打不了，量过 + 推的）

上面所有日志里 `NoSpaceAfterRaisingTheFloor`、`StillShort`、`PlacementRefused`、`SpaceAdmissionRefused` 一次都没出现（`grep -c` 每份 0）。覆盖写 140 次再 unlink，256、240 槽两档、准入判着或关掉都照样做成（`logs/hist-wall.log`）：会话里根环转圈之后稳定回收，执行器里的一个单元的文件撞不到墙。

挂载续做停在 `StillShortAfterTheFloorRaises` 要多大的文件，我在窄盘原型上量过（`logs/run3.log`，不补足环窗口的盘，比执行器那几档更窄）：1、2、4 个单元的文件 70、140、278 个起点里一次都没有，8 个单元 270 个起点里 8 次，14 个单元 220 个起点里 49 次。模型只罩一个数据单元的文件（U13 归下一批），所以：

- 执行器 `settle_mount` 里新加的那一支——`crates/singlefs-harness/src/history.rs:3795`「// 队列留着：模型的现行那一版停在删除中，那一次被拒交给这次挂载之后会话里下一次用户改动的走空重判」——在我跑的 854 段里一次都没走到，在今天的模型下也走不到（推的：只量了上面这几档盘宽与比重）。
- N1 的「挂载交回可写之后的会话有没有一段合法历史给出与模型不同的结局」在模型这一侧没法判：判它的历史今天生成不出来。这不是 N1 的实现错，是 N1 没有被模型比过的一格，与 r1 的 A3 同形（那时是删除中的一版上的用户改动从没被比过，U11 修了它；N1 这一支是 U11 之后剩下的）。

窄盘原型（不经模型）上 57 个停住的起点挂载都交回 Ok、停因都是抬 F 那一串被空间拒（`logs/run3.log` 的 StillShort 行；队列留没留着我没另核）；入库用例 `a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_with_the_queue_left_and_its_publishes_on_disk` 钉了停住之后会话里一次覆盖写报空间不够、卸载在预演里拒。

## 五、打中之后的四句

| 打中 | 分不分辨臂 | 被判的系统当时看不看得到 | 满足的是哪一句 | 跑前给的改法在这几格上还中不中 |
|---|---|---|---|---|
| H1 写账少抬 F 先写系统配置的写 | 分辨：只打「续做那一格交空的 `writes_before_the_first_publish`」，写行、暖机那一串与准入推抬 F 报错那一格（`FloorRaiseFailedAfterTheMountsPublishes`）不中 | 看得到：续做函数手里有准入推的与续做推的每一串 `RaisedFloor`，先写系统配置的写账就在里面 | 契约 `crates/singlefs-core/src/mount.rs:720` 那一句；跑前判据「打中 ⇒ 主 agent 现查坐实，改法进下一批」 | 跑前没给具体改法；我提的 G2 中，G1 不中 |
| H2 抬 F 报错那一支划不出续做段、写账缺挂载已落盘的写 | 分辨：`Publish` 那一支的划段全对（1545 个注错点只有 6 个交 None、都归错 0 个写），只有这一支错 | 崩溃注入那一侧看不到：错值里没有挂载的写账，`crash_injection.rs` 无论怎么写都划不出——是「判别子观测不到」那一形，要改的是 `mount.rs` 交回的东西，不是划段函数 | U12 的验收「续做里交回错的那次挂载，续做的写落在 Continuation 段」（`research/prompts/m3-unlink-code-r1-main-verification.md:41`）；写账那一半 A 已自报 | G1 中；G2 不中 |
| H3 U9 用例删掉抬 F 那一段写账照样绿 | 分辨：MU1、MU2 下原型红、入库两份测试绿 | 看得到：用例手里有错值与盘面 | K2 N4「有没有错实现照样过」 | G3 中（原型上量过同一个判法杀两刀）；G1、G2 不碰它 |
| H4 注释的全称句 | 不分辨本批：写行、暖机那一串同一个口径，换哪个实现都一样 | — | K2 N2「写账与设备一层数到的写对不上的形」：合计对得上，只是份数的口径 | G4 只改字 |

## 六、改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| G1：续做里为空间推的抬 F 自己报了不是空间不够的错时，不原样交回，照 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 的样子包一层，带上这次挂载写行、暖机、准入推的抬 F、续做已落盘各步与续做推成的那几串的写账；`crash_injection.rs` 的划段认这个成员 | H2 两半 | 推的（没实现、没跑） |
| G2：续做那一格交 `Publish` 时，把准入与续做推的每一串先写系统配置那一步的写合进 `writes_before_the_first_publish`（或在 `PublishSequenceFailed` 里另立一格「挂载里推的抬 F 先写系统配置的写」），使「落下的写 − 取号两次 = 写账合计」 | H1 | 推的；「落下的写 − 取号两次 − 2 × 推过的串数 = 写账合计」（串数按注错点之前已推成的算）在第二节表里五个起点 1376 个续做里的 `Publish` 注错点上逐点成立是量过的 |
| G3：U9 再补两条用例：准入推过抬 F 的起点（256 槽、覆盖写 2 次、14 个单元、崩在第 7 批之后）与续做里推过抬 F 的起点（崩在第 0 批之后），注错点放在抬 F 之后的某一步续做；判「已落盘份数 = 落下的根槽写数」与写账合计（照 G2 修没修取两种等式之一） | H3 | 判法在原型上量过：MU1、MU2 各被杀（第三节）；做成入库用例是推的 |
| G4：`unlink_mount_continuation_and_the_emptied_version.rs` 第 548 行那句改成「份数等于落盘阶段走完的发布数」，或在用例里写明只在一步的第一个写上量 | H4 | 推的 |

## 没打中的形状

- N5 取界：准入推、续做推、两者都推三种，2456 个不同的起点；取界与真值没有一个对不上，也没有一个取不出（第一节）。
- N5 B 自报那一形（续做开头被空间拒、先推的那一串归暖机）：准入判着与关掉两种挂载都没造出续做第一步之前就推抬 F 的起点。准入关掉时续做一串抬 F 都没推（540 个起点）。
- 续做推满仍放不下而推的那一串真发时部分落盘（`FloorRaiseRefusedForSpace` 里装的是 `RaiseFloorSequencePublishFailed`、前 n − 1 次已落盘）：那几次不在 `MountOutput` 的任何一格里，取界会往后偏、执行器的模型会少比几次。只按代码推的，没造出来——要预演与真发分叉，不注错的盘上我没找到造法。
- N3：删除中的一版上覆盖写、unlink、建 inode，接回退、挂载、抬 F、冷启动、零单元发布，854 段历史 0 个新发现（第四节）。
- N1：在模型这一侧走不到（第四节）。
- MU3（去掉续做写账的 txg 排序）：推为等价，没跑。

## 这条腿自己的限度

- 一次抽样；「没打中」那几格照 `.claude/rules/three-way-inference.md` 要再抽一次。
- N5 那几个函数是从 `crash_injection.rs` 抄进 harness 档原型里调的（`copied_from_crash_injection.rs`，抄的那段原样、只在文件尾加了转手的出口）：判的是抄的那份，冻结副本里那份没跑过（checker 档不跑）。冻结副本里那份改了，抄的不会跟着变。
- 真值是我定的：取界的真值看系统配置槽偏移 481 的 F 第一次变大的那个写；续做段的真值按没崩过那次挂载的写行 + 暖机 + 准入推的空发布次数数根槽写。两条都只在不补足环窗口的窄盘上用过。
- 注错只扫了 3 + 3 个崩溃起点，每个起点从第 1 个写扫到最后一个写；别的起点只取界、不注错。
- `logs/grid-u14.log` 那一批我在外面套了 `timeout 235`（定义写明不给批设限时，这是我做错了），跑到 1158 行被停，没跑完的那几个起点没取样。
- `logs/hist-4G.log` 那一批估时没估准、超过前台 240 秒被转成后台，我按 `proc.py stop` 停了测试二进制（293 段之后），PB 前缀只跑到 82 段、PC、PD 前缀在 4 GiB 上没跑。
- 随机历史的种子基我用十六进制写进环境变量，原型只解析十进制，`rand-b1`、`rand-b2`、`rand-b3`、`rand-t` 落回了同一个默认种子基；表里已按去重计数。
- 层 0 的枚举函数一次都没调：这一条腿原型跑的崩溃状态数是 0。
- 每批的段数与估时：网格每批 540–1158 个起点、估 1–2 分钟；注错扫每批 3 个起点、约 1056 次挂载、估 2 分钟；穷举后缀每批 240–293 段；随机历史每批 30–120 段、估 1–2 分钟。各批挂钟 13 秒到 2 分 18 秒（同机别的会话在跑，nice 19）。

## 顺手看到、不在这一轮判的

- 只挂载那一格的挂载透明取「挂载推没推抬 F」只看准入推的（`crates/singlefs-harness/src/mount_transparency.rs:241`「match output.space_admission.floor_raises().last() {」）：续做里推了抬 F 而准入没推时，它按「F 不变」判。挂载透明在续做上怎么判归下一批（正文第一节「不在这一轮判的」），这里不攻，只记。

## 没做什么

- K1、K3 不归这条腿；N4 里别的新用例、删空 `KNOWN_RED_FORMS`、LR5 的等价论证、N6 删掉的四行都没攻。
- 没跑 checker 档、没跑层 0、没跑整包 harness；MU1、MU2 只在两份 harness 测试二进制上跑过。
- 副本上的数不进 kb；草稿目录里的仓副本与编译目录交回前删掉（见交回）。
