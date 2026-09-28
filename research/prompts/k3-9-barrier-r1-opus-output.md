# k3-9-barrier-r1 云端攻方（W1）报告

轮名 k3-9-barrier-r1，攻击面 W1（删掉冻结副本 `crates/singlefs-core/src/transaction.rs` 第 1058 行那道屏障，恢复照常 `Consult`，找会出错的崩溃状态）。2026-09-28。
报告分段写入。这一段是中途先落盘的初步结论（看门狗整点询问时写的）；定稿的「各格判定一览」、复跑命令与 sha256 在后面几段。

## 中途落盘的初步结论（第一段）

- 删掉屏障的那一臂上，全部在第一截（崩溃镜像只读恢复）判的状态里，恢复结局都没有一格出错：落回上一版或者施加到这一版，没有施加半次，也没有落到已确认那一版之下。W3 网格里「施加半次」那一列，这一臂一个状态都没有。
- 打中了一格判据：在「第一个文件 / 覆盖写 / 顺序写」这几次发布崩溃之后，第二截（可写挂载）、第三截（挂载途中再崩）、挂载后再发一次布再崩，这几截上池级 checker 的 I-3.10 判红。一处原文：`I-3.10：盘 0 槽 1472 的分配记录（未释放、跨 1 槽）分配代 3，它罩住的单元头里的诞生代号是 4`。同一批状态上恢复落到的那一版，oracle 判的是对的。带屏障的那一臂跑同一个场景，I-3.10 是 0。
- 机理（读代码推的，还在用对照核）：checker 按 `.claude/kb/invariants.md` I-3.10 射程 ④，把「下一次挂载会先施加的那一版」（与最新根同实例、txg + 1、带提交标记的那条记录）的分配记录树也放进读集，而且不复刻点名单元验证。屏障删掉以后，记录落盘、单元没落盘这种状态零故障就能走到；于是挂载的点名验证不过、落回上一版，分配器把那一版的空闲槽发给写行那次发布写新单元。结果记录里那棵分配记录树的代（旧 txg）和槽里新单元的诞生代对不上。
- 还在跑的对照：带屏障那一臂在挂载期间读不出一份单元（一个读故障），同一格红不红。结果定下这一格分不分辨臂。

## 各格判定一览（定稿）

臂的叫法：「带屏障」是冻结副本原样；「删屏障」是副本上设 `K39_DROP_UNIT_RECORD_BARRIER`，跳过第 1058 行「    writer.perform_commit_step(CommitStep::Barrier)?;」（`crates/singlefs-core/src/transaction.rs:1058`）。恢复一律 `Consult`。所有数都是副本上量的，不算入库装置上的数；产物在模型目录 `runs/` 下。

| 格 | 判定 | 一句话依据 |
|---|---|---|
| W1-① 一次发布多条记录，共享内生块只在末条点名 | 恢复结局没打中 | seq2、seq3 在删屏障臂上各手摆 1187、4931 个第一截状态。施加只出现在记录与单元全落的 8、26 个状态上；没有施加半次；层 0 oracle、池级 checker、记录核对器都是 0。第二、三截各 7 个起点，挂载那一段全量展开 671967 个状态，同样全 0 |
| W1-② 第三截：挂载途中再崩 | 恢复结局没打中；池级 checker 打中（I-3.10，机理同 ④） | 删屏障臂 first 场景：挂载那一段 88263 个状态里 I-3.10 红 2016 个；挂载之后再发一次布的手摆状态里 over、seq2、first 分别红 27、57、129。带屏障臂同样的场景与挂载段（over、seq2、first），I-3.10 都是 0 |
| W1-③ 点名单元读故障、单盘掉，与缺席叠加 | 没打中 | 5 个场景、8 种读故障模型：删屏障臂出现的每种结局签名（落到哪一版、读回哪一版的内容、有没有落到已确认那一版之下），带屏障臂也都出现过，逐场景 `comm -13` 为空 |
| W1-④ 记录已落，单元一份落了一份没落 | **池级 checker 打中（I-3.10），零故障、第一截就红** | floorreuse4 历史（第一个文件 → 覆盖写 4 次 → 抬 F → 覆盖写，被判的是最后这次）：删屏障臂手摆 271 个状态，I-3.10 红 48 个；带屏障臂 73 个状态，红 0 个。最短的一格只少一次写：复用槽上那个数据单元在盘 0 的那一份 |
| 丢已确认的写 / 施加半次 / 挂载接受坏状态 | 没打中 | 删屏障臂上没有一个状态落到已确认那一版之下，读回的内容都认得出是哪一版的；可写挂载没有接受过一次坏状态 |
| 三条判据红不红 | 池级 checker 红，只红 I-3.10；理想模型（层 0 多版本 oracle）与记录核对器不红 | 见上 |

### 打中之后的四句（照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）

1. **分不分辨臂**：在零故障的历史上分辨。同一段历史、同一类状态，带屏障臂 0 个红，删屏障臂有红。可是带屏障臂只要在挂载期间读不出一份单元（一次读故障），同一个 I-3.10 就红：`runs/s4-first-readfault-control.log` 的 TALLY 行是 `checker_violated={"I-3.10": 765}`。所以这道屏障买到的是「I-3.10 射程 ④ 那条前提在零故障时成立」，不是一条对用户的语义。
2. **被判的系统当时看不看得到判别它的东西**：看得到。恢复在施加前逐份核点名单元，核不上就不施加（「        let all_verified = !verify_named_units」（`crates/singlefs-core/src/recovery.rs:3161`）、「            report.verification_failed += 1;」（`crates/singlefs-core/src/recovery.rs:3177`）），落回上一版，结局是对的。checker 也看得到：记录点名项带着每一份的 CRC，盘上单元的字节就在那里。只是它有意不复刻这一步，「/// 读它只多判 I-3.10 一格，它的分配记录与它罩住的单元是同一次发布写出来的，不论施加与否两个数都同源。」（`crates/singlefs-checker/src/walk.rs:3548`）。删掉屏障之后，这句的「同源」在零故障状态上就不成立了：记录落了，槽里还是复用之前的旧单元。
3. **满足的是判据字面的哪一个分句**：满足 W1 格里「池级 checker……红」那一半，不满足「丢已确认的写、施加半次发布、挂载接受坏状态」里的任何一句。跑前判据第五节「乙成立」要把它破的东西指到一条已定句。这里能指到的只有 `.claude/kb/invariants.md` 第 143 行（I-3.10 那一行）射程 ④ 的两处：「记录与单元是同一次发布写出来的，两个数因此同源」，以及「再加下一次挂载会先施加的那一版」。那一行自己写着「被攻过零轮」。D16 已定项 7、D23 已定项 14、15、17、22 里，没有一句被这些状态破掉。按判据表，这一格属于「打中的是 checker 自己的前提」：红来自 checker 对「下一次挂载会施加哪一版」的判法，不来自文件系统的结局。
4. **每个改法在打中的那几格上还中不中**：见下表。

| 改法 | 删屏障、零故障那几格（floorreuse4 第一截；first、over、seq2 挂载段与挂载后发布） | 带屏障、挂载期一次读故障那一格（first） | 标 |
|---|---|---|---|
| 甲：删屏障，别的不动 | 中（48/271；2016；27；57；129） | 中（765） | 量过 |
| 乙：保留屏障（今天的样子） | 不中（0/73；first、over、seq2 挂载段与后缀全 0） | 中（765） | 量过 |
| 攻方自提：checker 取「下一次挂载会先施加的那一版」时，照恢复前缀第四条先核那次发布每个点名项的两份（`patches/walk.rs.diff`，环境变量 `K39_CHECKER_VERIFIES_NAMED`） | 不中（删屏障 floorreuse4 0/271；first 第三截 0/88263，后缀 0/1225；over 第三截 0/671967，后缀 0/1225） | 不中（I-3.10 0/1728） | 量过（副本） |
| 改 `.claude/kb/invariants.md` 第 143 行射程 ④ 的读法，把它收到「点名单元都验过的那一版」 | 不中 | 不中 | 推的（和上一行是同一件事的文字那一半） |

攻方自提的那一条只在我的模型上量过，被攻过零轮。它会不会让 I-3.10 漏掉本该抓的坏镜像：在副本上开着它跑 harness 档 `checker_known_bad_images`，44 条全过。其中专钉射程 ④ 的那条是 `an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount`，开和关各跑一次都过（`runs/badimage-on.log`、`runs/badimage-off.log`、`runs/badimage-all-on.log`）。

## 复跑命令与文件 sha256

模型目录 `research/prompts/k3-9-barrier-r1-opus-model/`：`crate/` 是原型 crate，`patches/` 是对冻结副本的三处改动（diff），`scripts/` 是跑法与八个批次，`runs/` 是全部原始输出，`README.md` 写建副本与复跑的步骤。

最短复现（第一截、零故障、池级 checker I-3.10 红），在建好、编好的副本根目录跑：

```
K39_DROP_UNIT_RECORD_BARRIER=1 SINGLEFS_LAYER0_THREADS=10 nice -n 19 bash research/scripts/run-with-memory-cap.sh 4G target/release/k39-barrier-attack stage1 floorreuse4 384 16 32
```

同一条命令去掉 `K39_DROP_UNIT_RECORD_BARRIER=1`，就是带屏障那一臂。复跑时 `run-with-memory-cap.sh` 写仓根下的全路径。原样输出（`runs/stage1-floorreuse4-detail-mutated.log`）里的两行：

```
CHECKER-RED-FEW-WITHHELD seg0 kinds=uuuuuuuuuuuuuuuuuuuurr mask=0111111111111111111111 withheld=["unit_write@dev0+705"]
TALLY floorreuse4 states=271 violations=0 ignored_violations=0 checker_violated={"I-3.10": 48} record_root_without_record=0 record_claimed_state_missing_unit=0 verification_failed_states=198 root_persisted_states=3 failed_states=0 first_violation=None checker_first_violation={"I-3.10": "盘 1 槽 1410 的分配记录（未释放、跨 2 槽）分配代 11，它罩住的单元头里的诞生代号是 0"}
```

（`dev0+705` 是按 32 KiB 数的偏移，对应槽 1410。这一次数据单元跨两槽，落在 mkfs 那一代单元换下、抬 F 之后回收的槽上。）

带屏障臂同一命令的 TALLY 行（`runs/stage1-floorreuse4-control.log`）：

```
TALLY floorreuse4 states=73 violations=0 ignored_violations=0 checker_violated={} record_root_without_record=0 record_claimed_state_missing_unit=0 verification_failed_states=0 root_persisted_states=3 failed_states=0 first_violation=None
```

最短历史：mkfs → 可写挂载（实例 1）→ 第一个文件 → 覆盖写 4 次（都整次落盘）→ 抬 F 到现行 txg − 3 → 覆盖写 P。崩在 P 的单元写与记录写同一段里：两条记录（两块盘各一份）都落了，槽 1410 上 P 的数据单元只有盘 0 那一份没落。结局是恢复核点名单元，盘 0 那一份对不上，P 整条不施加，落回抬 F 那一版，读回的内容对。池级 checker 按 I-3.10 射程 ④，把 P 的记录当成「下一次挂载会先施加的那一版」，读它新根段里树表指着的分配记录树。槽 1410 那条分配记录的代是 11，槽里还是 mkfs 那一代的旧单元（诞生代号 0），判红。

文件 sha256（`sha256.txt` 本身不在内）：

```
ca65cf25179d914eb69fd643d44e2cd525fef20a8402b75cb95ef12f7ff0c741  ./crate/Cargo.toml
e85f7e5b3bfa51c8880956bb0261dcda704adf2423b7966bfda9d40bf84cd55f  ./crate/src/common.rs
eb4bdfaeed8cca31c1eb97e64bfa9d5f71da6f6db341c813ac3219540462298d  ./crate/src/faults.rs
fa07ce60db114ca11858ec6468bc62dbb6a3d612b121a650c7d5d929c534b802  ./crate/src/main.rs
c82f939286f47672137151d21184460a36aa6659c452e9d9ce6875dac40e1d02  ./crate/src/scenario.rs
cb96cf138de96f9180de427876fcc993a3bcb57b4110d077748450d21048d956  ./crate/src/third_cut.rs
7f8a1e2c3775ba2eb896ef919c918bf8d9eb590ca09ef7ba32a010ac34ea3789  ./patches/transaction.rs.diff
e88bbc7176ef3dc786fc4e00aff34c531eb65edbff75414e655439742bad374f  ./patches/walk.rs.diff
56cf34049da7420d9b94580ce1aa18319328ff4099e220f2d2fc92f1c53f8915  ./patches/workspace-Cargo.toml.diff
68f3c74dc7be081ad314053168c344ec3ee34bf85f08b37c2eceaae691c39381  ./README.md
328551e6e842cdd093d6a9619265db8e789b7f04da3aa3450725b034364031f5  ./runs/badimage-all-on.log
02e6ad4637a9de9ed963e5f700dda37217d49b5189f5ab71c30e763eea882cb0  ./runs/badimage-off.log
c7894322e289743f0afa911cde8042a964220c4dd57dba155e0cbefa0228f4c4  ./runs/badimage-on.log
c9de34deda9a6e63f6ff6bbcd154ef2d111656c31c5a2ae00a12856665cb2ff2  ./runs/batch1.out
6cd2f6d939fcdbd0912b94d4fc97bc5e32a731845f71d56881aa6b5515d0c0a7  ./runs/batch2.out
9475868c3826d5572cf7d20a3d23fa6ecf4f38a532452a2e54d808185bb139c0  ./runs/batch3.out
9475868c3826d5572cf7d20a3d23fa6ecf4f38a532452a2e54d808185bb139c0  ./runs/batch4.out
2b2edf512a675dbca88930a4577b437b7775af68877219062952b4d3c96dae22  ./runs/batch5.out
c4eba9737977e61902e5b1e72dc6e2c4004b1f6cac0d66b8d4f511994d6989c5  ./runs/batch6.out
7cb98dc8f8bfeb559e76f6c18f839abf9d589dbc3baa16e9ba19d7aeaf5d494a  ./runs/batch7.out
22b2dd446312c454135f22c5c115184d1a6c6f381d4c4e99cc27c0501bec47cf  ./runs/batch8.out
57d40d7c2d736083b546045ae00686cb7d64ae78178041c79a8f7dee5985a988  ./runs/faults-first-control.log
b07d65577bc19812ffed799c711e96472d4dd7fddd8631ee4e847d7f6a3321b7  ./runs/faults-first-mutated.log
70a0b0da8100188fea67e4993ede2b4cc8b28f163f7d07677e8f31f44444b03a  ./runs/faults-over-control.log
9fe50fcab42942bbff28d18453541a9807b245907ad057c7e4d07c3d73293bc9  ./runs/faults-over-mutated.log
b1a1ab657c7a9b009f37879d91fe649f37c7a558dbba14cfde0f383ac9d7f447  ./runs/faults-rollback-control.log
e7c8495857470daa0a0fef70d465b59266c52191461bd3e3d3a4d9a7b0a925a7  ./runs/faults-rollback-mutated.log
108b5c0eb9ff82cdb2748aeda9ca7913e9a0654f738d84bd3ce35e83ae5a0065  ./runs/faults-seq2-control.log
21ef1191bf3eb4179d7cd248e2d66fd719a2a2b9666e51162387a4d9b452fd6d  ./runs/faults-seq2-mutated.log
da88ee6ba72fd91f84d61996665f918680fe204df8ee1316657fed9fd97d6ed9  ./runs/faults-seq3-control.log
3ffacff9056c8a7b88ef490b2880cd2fc135125a241b9b9edd0d99c38f8a07d4  ./runs/faults-seq3-mutated.log
eaecf4c7bd5613305d7113e2454bdea8e67fb6594345f0673971d02a973883f8  ./runs/full1-rollback-control.log
1534d1c0cd89993b950d0463208ad471a3e15753f17ffe1b771dd2403cc7f984  ./runs/full1-rollback-mutated.log
2da177718b0fa5db0a0fa6afcef2142a250f5ecfe2e7212aff5456740e389d7b  ./runs/layer0-state-totals.txt
78f7cc784a65bab9449d46866dbb48e302b69b648f51c9fb177bb1d4d7e76c46  ./runs/s2-first-mutated.log
847b803c7a36a56e83ff0e85002ea0788df9eeadb35970fbd5728db14c413243  ./runs/s2-over-control.log
59fc78e0e79d7feda5f5e828f7bca2d17973790f719ce94d301cbd1571d72af6  ./runs/s2-over_i2-mutated.log
746814057aa5f453997646b16ab38d72ce9db91ad1480bfcf81e41b68bc868a4  ./runs/s2-over-mutated.log
de42d5402ec0e01848709fe9d30e2749198d7ab0f050f05357ec03df0a1b4379  ./runs/s2probe-rollback-mutated.log
7ef21632c7513d66328aacdc935594caa5cd55c6cd6a8a3da08b66162dc3837c  ./runs/s2-seq2-mutated.log
7ed2d704fa6d25b7be2e24ff9dc99f7667d70c1ad8be9aab87b8815fdd4d559d  ./runs/s2-seq3-mutated.log
38dcdae3fae805770c75073912cbcbf92ddf70c1b86a69b2e8b8dfed03dd4976  ./runs/s4-first-control.log
7cd2272599853d4140f87ea24aa9f13fa430046696e18b44491d2b458bce73fa  ./runs/s4-first-mutated.log
45f202667f052570da85d1dfba373547e067f663f61eba503253bd105f92aab3  ./runs/s4-first-readfault-control.log
991b8c49a19a28feba74fa3d3f3c8cf92b0d9b00efee666b2a1dee650a3069da  ./runs/s4-over-mutated.log
91abeeea241d2a75a2a9c615e41af8e21049c3a5ef09855e75273f9f24f0cc47  ./runs/s4-over-readfault-control.log
e2bf59d754d9610511d7bc1a490f6289822c618d5dd4a8af66413b238997d4d4  ./runs/s4-seq2-control.log
c9823bf5aaf55d9d112f80ab6797be335d7c5031e309c9c742d904983a55fd4c  ./runs/s5-first-readfault-control.log
6e53fc281edf1c1f37bcde75cdfeb2826882d7011bc3c460585a0a19dd511d98  ./runs/s5-over-readfault-control.log
99f107ff8e18e0ab63365b28917cc498481e91c1c405a858d471fb98425a1f01  ./runs/s8-first-fix-mutated.log
8adc47b64abb17d4c13d960b314d58d19de026ce4a1fa63f6c1f3c9d23725f48  ./runs/s8-first-readfault-fix-control.log
189fb0f20dae1fd207dd9e7ffece914a66a6728361ac661a824eac82cf8da3ac  ./runs/s8-floorreuse4-fix-control.log
849881394634b90204289ccd4154f59129a83892716dee2ce6382f8afbd99cd7  ./runs/s8-floorreuse4-fix-mutated.log
907521ad3408b4c53e66993f2044f0b4295c730aa65aed89be5a98958cbc9100  ./runs/s8-over-fix-mutated.log
23d3794bb4e03443b490a680ccd23b1bdc3a9e0c1b2edc6ab972fb7bcb8ecc57  ./runs/stage1-first-control.log
418ea7a6eb411d367f443db4e66f6fcea58259013ce6cfcc030933dcc60815d9  ./runs/stage1-first-mutated.log
14bd6a81fe6784f92a8065858f2d4d8d1e742541d80b543b47e0c9173392c0a7  ./runs/stage1-floorreuse4-control.log
01dc8477c9c05ca5fa7e320bf07e70701f8309f86a75ea3d21f8ad26e6ad373b  ./runs/stage1-floorreuse4-detail-mutated.log
90f72ae9167dd3f15c42122201654e3417dfd6aa285f1092f8ca62d3e2e6f17d  ./runs/stage1-floorreuse4-mutated.log
0fbfd0a0c5bdffb96a200982f52b4794847c5c4cb032fa645d0247cb3cae96e8  ./runs/stage1-floorreuse8-control.log
f1647480bd11dd97c5a714eef4fe995ae559b5b8536d73777448b6177a7f57a6  ./runs/stage1-floorreuse8-mutated.log
0580da6f6165cd11998254b76dce4eae32a515aa0218cf6f1dabbe5d43fab557  ./runs/stage1-over-control.log
d5acb5e15b7ed59823c0bdf9e161a8e409f66847f40ea0bcfbe3ba1869826d61  ./runs/stage1-over_i2-control.log
bb827e56e65373c514d2a8c2127c4a72428d648e832f50013d39d7923bd6c6d3  ./runs/stage1-over_i2-mutated.log
ebce55655937e5c2626440a85263607cb5465862f58e6c700a7a60e99ca067e1  ./runs/stage1-overmany30-control.log
59ed5657b6eab8f4fd141b85944caa1782fb2f34349d2ab661e3fc376e305264  ./runs/stage1-overmany30-mutated.log
5b0708bed39a86a03fb86a5b36070dd6c3aeefbe13f7434ae55ba5a4048ca54b  ./runs/stage1-overmany60-control.log
cdeb3291677b766fa2a686338d3111902032f77b8035bd33bdfbd3a7f0d8b52f  ./runs/stage1-overmany60-mutated.log
d8db406d1f9adf4cd6116b0b56c430654fd25a8718a4cf4b305d1a714815fddd  ./runs/stage1-over-mutated.log
e5c5c8fd5a163912666c1e81820affa28c81f485ebca7ad4a149569a73edf659  ./runs/stage1-seq2-control.log
97f3aa8aeb00c4f4721c7a83f83f95baa3dcb0a0fdc50753f0c367aecdc7f7c6  ./runs/stage1-seq2-mutated.log
7036ac2377237fe2d8f095f5a304b454b894c3f9283862dfb1b628b9740be548  ./runs/stage1-seq3-control.log
1bdd3fcc442cf4a6bdf65b21d1d3297d659b74e7aa1a87338da5b6a0c00a3d87  ./runs/stage1-seq3-mutated.log
5b5cf7f833097c1e7cfc57215984ba3d2754bfc6239a171af205f3571f3e2d7f  ./runs/survey-384-control.log
155c2bed9d2600d345dd2e26dee83bc39257339898401fe9b0163e04ad3667b5  ./runs/survey-384-mutated.log
71668c6398c3e3c493baef51e671eb6b745e67d65808c9b0fcd788f7a1daebfa  ./scripts/batch1.sh
7ecb39f70c5c5563145658f80bd7fc1c864a0348c6286962912bcda0ed5eabc3  ./scripts/batch2.sh
7ba1d92aae0dcdef8cb9f9fa367613a7935105ffd8273076114a390c819d7d7f  ./scripts/batch3.sh
1e5584afcfa939b641dc17fbeac5692fa4161a1aa8660fee3f71492efa1a2642  ./scripts/batch4.sh
f537a9f209989222b1033b7853fe17c0071c6a3ed7730f4e63843508d3dde2eb  ./scripts/batch5.sh
41211f89b6d3f7fd4fccb453a9b7b0c708da1d732775b963e94da8b3fba738b8  ./scripts/batch6.sh
d51488de916fce50f9d3f0b46e56e12771977602eeb0232f56eb0bd588a3dae3  ./scripts/batch7.sh
2b852add7300677dfde76c0f62cffb39b38d7d5181445f196b31141715abc5f6  ./scripts/batch8.sh
182eb2f516ff135f3add2cfeacbf1bd97f2c61496bece8bb67d4432e06ae50fb  ./scripts/run-arm.sh
```

## 按攻击面分节

### 装置与取样（四个方向共用）

- 盘都是内存稀疏盘（`SparseBlockDevice`），两块。几何主要取 `UnitAreaOf384Slots`（单元区 384 槽，环 6 MiB）。起点历史每个场景只跑一次，被判那一次发布之前的池只建一次；之后每个状态从那份 `MemoryPool` 叠写。
- 场景（`crate/src/scenario.rs`）：`first`（被判的是第一个文件）、`over`（覆盖写，一条记录）、`seq2` / `seq3`（顺序写 2 / 3 个数据单元，2 / 3 条记录，共享内生块只在末条点名）、`over_i2`（重挂到实例 2 之后覆盖写）、`rollback`（挂着时回退那次向前发布）、`floorreuse4` / `floorreuse8`（抬 F 之后覆盖写，复用回收的槽）。
- 段形（`runs/survey-384-*.log`）：带屏障臂上，覆盖写的单元段是 20 个写、记录段 2 个；删屏障臂上两段并成 22 个写，全量 4194308 个状态。seq2 并成 28 个写（268435460 个），seq3 并成 32 个写（4294967300 个）。每段历史都超过约 10⁶，照定义「那段不跑」全量，改成手摆（`common.rs` 的 `masks_for_segment`）：记录写的每个持久子集，乘以｛单元全落、每个单元两份都缺、每一份单独缺、全缺、只缺盘 0、只缺盘 1、数据之外全缺、数据全缺、随机 32 个｝。不超过 16 个写的段全枚举。手摆的不是层 0 全量，合计 19412 个状态。
- 层 0 域全量（`enumerate_layer0_selecting_versions_observing_each_state`，每段都展开，含系统配置槽原地覆写的撕裂态）用在两处：`rollback` 那次发布（删屏障臂 16393 个状态、带屏障臂 4108 个）；第二、三截里挂载那一段（每个起点 288 到 98337 个状态，`LAYER0_PARALLEL_FINISHED` 行）。合计 4667120 个（`runs/layer0-state-totals.txt`），跑前每一次都用 `layer0_state_count_with_torn_in_place_overwrites` 算过、不超过 10⁶。
- 缩了什么：第二、三截的起点不是全部第一截状态，是按类轮流挑的，每个场景 5 到 7 个（类 0A/0B/0O/0R/0S/1/2 各一个起，候选数见各日志的 `CLASS` 行，例如 over 的 0R 类 132 个里挑 1 个）。估时：rollback 探针 426117 个状态 358 秒，约 1190 个/秒；按它估批 3 约 2.9M 个状态、约 40 分钟，实跑约 20 分钟（批 3 的六件 WALL 合计 1218 秒）。
- 并行：批 1、2、6、7 每批 10 或 4 件、每件单线程，退出码照定义写 `.rc`，数过件数（`runs/batch*.out`）。批 3、4、5、8 顺序跑，每件内部 10 线程。没给批设限时。

### ① 多记录发布、共享内生块只在末条点名

- 第一截（`runs/stage1-seq2-mutated.log`、`runs/stage1-seq3-mutated.log`）：落到那次发布（(1,4)）的只有记录与单元全落的状态（seq2 8 个，seq3 26 个）；其余都落回 (1,3)，其中 753 / 2763 个是点名验证不过。末条不在、只落了前几条、共享块全缺、数据块全缺，这几类都落回上一版。看代码的原因：前缀要一直走到带末条标志的那一条才施加，一条核不上就整次不施加（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`）。
- 第二、三截（`runs/s2-seq2-mutated.log`、`runs/s2-seq3-mutated.log`，各 7 个起点、挂载段合计 671967 个状态）：oracle、记录核对器、池级 checker 在挂载段上全 0。seq2 挂载之后再发一次布，手摆状态里 I-3.10 红 57 个，机理同 ④。
- 读故障叠加（`runs/faults-seq2-*.log`、`runs/faults-seq3-*.log`）：没有新签名。

### ② 第三截：崩溃后挂载途中再崩

- 第一截落回上一版之后，可写挂载取实例 2，写行那次发布的 txg 取环里最大 + 1。被判那次发布没施加的记录占着一个 txg，于是 first 场景里写行那一版是 (2,4)。写行那次发布的分配器从上一版重建，把那次发布写过的槽当空闲再发出去（first 场景是槽 1472）。挂载途中崩在写行的单元落了、根没落的状态上：恢复择 (1,2)，被判那次的记录还在水位之上，核点名单元对不上，不施加，结局对；池级 checker 读那条记录的分配记录树，槽 1472 的分配代 3 对上诞生代号 4 的写行单元，判红。`runs/s4-first-mutated.log` 的原样发现行：`LAYER0_FINDING event=new finding=1 pass=pool_checker violated=I-3.10 segment=1 publish=instance2_txg4_root_write12 first_state=9 first_violation=I-3.10：盘 0 槽 1472 的分配记录（未释放、跨 1 槽）分配代 3，它罩住的单元头里的诞生代号是 4`。
- 挂载整段落盘之后（`after-mount` 那几行签名），checker 不红：最新根换成实例 2 的，那条记录不再是「最新根同实例、txg + 1」。红只在挂载途中，或者挂载之后下一次发布途中的崩溃状态上。
- over、seq2、seq3 的挂载段全 0，红出在挂载之后那次发布：它复用了被判那次的槽，槽里是被判那次落了的单元（诞生代号是被判那次的 txg），和新记录里的分配代对不上。
- 后缀放开扫：挂载之后｛什么都不做、再关掉重挂一次、发一次布｝三种都跑了。重挂那一种（`runs/s2-over_i2-mutated.log`）两次挂载合起来一段历史 1146929 个状态，超过 10⁶，4 个起点都 `SKIP`，没跑。

### ③ 点名单元读故障、单盘掉，与缺席叠加

- `faults` 模式：第一截每个手摆或全枚举的状态上，换 8 种读法跑恢复——不加故障、被判那次落了的单元盘 0 那份读不出、盘 1 那份读不出、盘 1 整块掉、盘 0 整块掉、上一版的根槽读不出、上一版的记录两份都读不出、上一版的根槽与记录都读不出。rollback 删屏障臂 130584 次恢复、带屏障臂 32688 次，其余场景见各日志 `FAULT-RECOVERIES` 行。
- 删屏障臂出现的每个签名（落到哪一版、读回哪一版的内容、有没有低于已确认那一版），带屏障臂也都有。「盘 0 整块掉」两臂都落到 mkfs 之后那一版（低于已确认那一版），「盘 1 整块掉」两臂都走读失败（I-1.1）。这两种出现在两臂上，不分辨臂，也不在 W1 的射程里。我的故障模型把掉的盘从 `device_identities` 里拿掉，这不一定是挂载会遇到的样子，没核。
- 挂载期间读不出一份单元（带屏障臂，`K39_MOUNT_READ_FAULT`）：挂载落回上一版，复用槽，I-3.10 红 765（见四句第 1 条）。有 1 个起点被拒可写（`WritableMountRefusedByDevicesWithoutTheSelectedVersion`），这是读故障一支自己的结局，两臂共有，不是这道屏障带出来的。

### ④ 记录已持久，单元一份落了一份没落

- 删屏障臂上，一份落一份没落的状态（类 0B）在所有场景的第一截都落回上一版：两份都验过才施加，恢复结局没打中。
- 被判那次发布的槽上若之前有旧单元（抬 F 之后回收复用，或崩溃后挂载复用），没落的那一份留下带可用单元头的旧单元，I-3.10 就红，而且零故障、第一截就红（上面的最短复现）。被判那次的单元都落在全新的槽上时，没落的那一份槽里读不出单元头，I-3.10 按射程 ③ 报不适用，所以机理调查那段历史（每次发布都落在新槽上）看不到这一格。

## 给主 agent 的两条旁注

- C592 那一行写「单独拿掉……今天没有判据会红」。这句只对机理调查那一段写死的历史成立：那段历史里每次发布都落在全新的槽上。历史里有复用回收槽（抬 F，或者崩溃后挂载复用）时，拿掉这道屏障，池级 checker 在第一截就红。随机历史的崩溃注入流里有抬 F 这一步（`HistoryOperation::RaiseRollbackFloor`），推的判断是：checker 档那条战役在变异 C 下有机会红。它属于重型测试，我没跑，要跑得请主 agent 去问。
- 候选甲若成立（屏障多余），跟着要改的是 checker 或 `.claude/kb/invariants.md` 第 143 行射程 ④，不是文件系统。候选乙若成立，它买的东西只能写成「I-3.10 射程 ④ 的前提在零故障时成立」，同一个前提在带屏障、挂载期一次读故障时照样不成立（765）。

## 没打中的形状

| 形状 | 取样 | 结果 |
|---|---|---|
| 单记录发布（first、over、over_i2），记录落而单元缺，任意组合 | 每场景手摆 271 个（删屏障）/ 73 个（带屏障） | 全部落回上一版或施加到这一版，oracle、记录核对器 0；这些场景的第一截 I-3.10 也是 0（推的原因：被判那次的槽都是新槽，没落的那一份读不出单元头，射程 ③ 不判；没逐状态核「不适用」计数） |
| 多记录发布（seq2、seq3）：只落前几条记录、末条不在、共享块全缺、数据块全缺、单份缺 | 1187 / 4931 个手摆 | 没有施加半次，oracle、checker、记录核对器 0 |
| 挂着时回退那次向前发布（rollback），删屏障后单元与记录一段 | 层 0 全量 16393 个（带屏障 4108） | 0 |
| 第三截：崩溃镜像上可写挂载、挂载段全量 | over、seq2、seq3、rollback、first 在删屏障臂上共 31 个起点；各批合计（含带屏障对照与复跑）挂载段 4.6M 个状态 | oracle、记录核对器 0；checker 只有 first 的 I-3.10（见 ②） |
| 挂载之后再发一次布，发布那一段手摆 | 每起点 49 到 175 个 | 恢复结局 0；checker 只有 I-3.10（over 27、seq2 57、first 129） |
| 读故障 8 种 × 全部第一截状态 | 约 22 万次恢复 | 删屏障臂没有带屏障臂之外的结局 |
| 暂时读不出点名单元（第一截） | 同上两种「单元一份读不出」 | 两臂都落回上一版 |
| 关掉重挂两次（后缀 remount） | 4 个起点 | 状态数 1146929 超过 10⁶，没跑 |
| 更多覆盖写之后根环转圈复用（overmany） | K = 20、25、28、30、60 | 小盘上第 20 次覆盖写被空间准入拒（`SpaceAdmissionRefused`），没造出来；改用抬 F 复用（floorreuse） |

## 这条腿自己的限度

- 带文件的发布在删屏障臂上并成 22 到 32 个写的一段，全量超过 10⁶，只手摆。手摆覆盖每个单元两份都缺、每一份单独缺、各种记录子集，外加随机，但它不是层 0 全量。「恢复只看记录可读与每个点名单元两份在不在，所以手摆的类罩全了结局」这句是推的。
- 第二、三截的起点每个场景只挑了 5 到 7 个（候选 73 到 4931 个）。挑法是按类轮流取，不是全量。
- 原型自己的版本表在分叉的时间线上会误报：删屏障臂 first 场景第一版（`runs/s2-first-mutated.log`）、带屏障臂读故障几次（`runs/s4-*-readfault-control.log`、`runs/s5-*`、`runs/s8-first-readfault-fix-control.log`）里 oracle 报的违例，都是被判那一版该不该算进版本表判错了。依据：逐条签名看，读回的内容都认得出是哪一版的，也没有落到已确认那一版之下（`unknown-content` 与 `below_floor=true` 的签名计数见各日志；s5 的 3 条 `unknown-content` 出在我把被判那一版从表里删掉之后）。这些数不拿来支撑任何结论。池级 checker 的判定不读版本表，不受影响。
- 「盘整块掉」的故障模型不一定是挂载真会遇到的样子（见 ③）。
- 攻方自提的 checker 改法只在副本、我的场景上量过，被攻过零轮。它会不会在别的合法镜像上漏报，只拿 44 条已知坏镜像用例核过。
- 只用了 384 槽小盘一种几何，没在 4 GiB 盘上重做。

## 没做什么

- 没跑 checker 档（`singlefs-checker-tier`）的任何测试或装置二进制，没跑层 0 用例、没跑 54 / 55 / 57 / 59 / 87 号。原型是另一个 crate（`k39-barrier-attack`），把 checker 档当库调枚举函数；它的测试目标与二进制名字不带 `layer0`，流在原型里自己造。
- 没判 W2、W3，没替主 agent 采纳。副本上量的数不算入库装置上的数。
- 没改仓里 `crates/` 与 kb 的任何文件。

## 状态数账（给接手的腿）

层 0 域全量合计 4667120 个状态（`runs/layer0-state-totals.txt` 逐文件列），手摆 19412 个，读故障恢复约 22 万次。每一次全量都在 10⁶ 以内，合计在 10⁷ 以内。
