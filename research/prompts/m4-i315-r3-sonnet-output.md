# m4-i315-r3 辩方腿（Sonnet）报告（2026-10-22）

轮名 m4-i315-r3（第三轮，最终轮）。立场：给自证加代号核对、丁 两份互比、两者叠用、甲二按（盘, 槽）比各造最强版本、替它们辩护，复核第二轮判决 `research/prompts/m4-i315-r2-main-verification.md` 够不够得着、是不是同样打中留下的候选。本报告里的数都是这一次在 HEAD（841b3beab）导出树的副本上真跑出来的；按「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」（`.claude/rules/three-way-inference.md:106`），副本上的数不进 kb。

## 〇、开工核对、副本与复跑

- 快照：`research/prompts/m4-i315-r3-snapshot/tree-sha256.txt` 四份全 OK（`walk.rs`、`fault_injection.rs`、`crash_judge_checker_scan.wgsl`、`image.rs`）。`kb-sha256.txt` 13 份里 9 份 OK，4 份对不上，原样输出：

```
sha256sum: .claude/kb/decisions/03-丙.en.md: No such file or directory
.claude/kb/decisions/03-丙.en.md: FAILED open or read
.claude/kb/decisions/13-verification-route.en.md: FAILED
.claude/kb/decisions/13-検証ルート.ja.md: FAILED
.claude/kb/decisions/13-验证路线.md: FAILED
sha256sum: WARNING: 1 listed file could not be read
sha256sum: WARNING: 3 computed checksums did NOT match
```

  三份 `13-验证路线` 是别的会话改过的（`git diff --cached` 只有一行里把一个路径缩写成文件名，与这一格无关）；`03-丙.en.md` 在 HEAD 里没有这个文件。我读的 kb 条款都取工作区此刻的行，引用处写那份文件自己的行号。
- 副本：`git archive HEAD` 导出 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default` 到草稿目录，四份被判文件的 sha256 与开工快照逐份相同。选 HEAD 而不是主工作区此刻的树，是因为主工作区里别的会话有 19 项未提交或未跟踪的 `crates/` 改动；第二轮两腿用的是主工作区的树，两边的数逐数对上（第三节），说明那些改动没碰这一格。
- 负载：开工时 `ps` 只看到别的会话的 `multi-host-run.sh --peer` 一条；之后别的会话的 `cargo test`（`attack_probe3`）、`crash-point-replay.sh --quick`、变异复跑的 `--quick` 在跑，没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。内存包装的配额常被别的会话占满：放大档比重 snapshot 一档含排队用了 43 分钟，两次排队超过 10 分钟的我停掉重起（把包装上限从 16G 改成 6G / 10G，峰值表按上限与历史峰值取小，排得进去）。
- 模型：`research/prompts/m4-i315-r3-sonnet-model/`。在 HEAD 导出树上先后打三个补丁（`apply_arms_r3.py` 九个臂与外给隔离清单、`apply_defects_r3.py` 实现缺陷开关、`apply_common_r3.py` 固定盘面搭建的两个入口），再放进三个探针目标（`r3_probe.rs` 随机历史、`r3_worlds.rs` 固定盘面十二个世界、`r3_fixtures.rs` 入库固定盘面）。
- 复跑命令（整条经内存包装，线程取派发提示的 31，工作目录仓根；`--frozen` 给 HEAD 的导出树）：

```
D=~/.local/share/singlefs/work/m4-i315-r3-rerun && mkdir -p $D/frozen && git archive HEAD crates Cargo.toml Cargo.lock .cargo .claude/heavy-thread-default | tar -x -C $D/frozen
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=辩方腿模型 rerun.sh 里 cargo test -p singlefs-harness 的 r3_probe、r3_worlds、r3_fixtures、fault_injection_fast_tier 四个测试目标; 范围=4 个测试目标; 为什么=端到端复跑 m4-i315-r3 辩方腿模型，核每档摘要与 out/ 逐行相同' RERUN_THREADS=31 bash scripts/hosts/run-with-memory-cap.sh 12G bash research/prompts/m4-i315-r3-sonnet-model/tools/rerun.sh --repo "$PWD" --frozen $D/frozen --work $D/work
```

- 这条命令我端到端跑过一遍（88 分 41 秒，含排队与别的会话拖慢磁盘），原样输出里的比对行：

```
name=in_repo_fast_tier 以「已知红」收尾 {}、新发现 0
name=rerun_compare config=fast-20x24 same_as_out=true
name=rerun_compare config=k21 same_as_out=true
name=rerun_compare config=leak_once-60x31-o2000 same_as_out=true
name=rerun_compare config=extra_leak-20x24 same_as_out=true
name=rerun_compare config=one_device-20x24 same_as_out=true
name=rerun_compare config=leak_every-20x24 same_as_out=true
name=rerun_compare config=worlds same_as_out=true
name=rerun_compare config=fixtures same_as_out=true
```

  放大档与比重档要加 `RERUN_FULL=1`（挂钟长，没有端到端跑，逐片跑过），见第十一节。
- 模型目录里每个文件的 sha256（`sha256sum tools/* out/*` 在模型目录里跑出来的）：

```
29df1a15dd60050f7f4d4ad5340c661546b2c093ebf9648b50c4823f2981a61d  tools/apply_arms_r3.py
3d4ee795290d4470beaef2d912f0b25e2292740463f0d964427beaca15eb554b  tools/apply_common_r3.py
a05e23643d9ccb5ed82cc3b1b80aac355ba6daee82c3c279fe886256c750bc0c  tools/apply_defects_r3.py
a1fb0217c7bf94817cb12d769753886677b05a2e3069f49a26f2bf38bb0f025f  tools/r3_fixtures.rs
82d2104a57490eb96d968fe871b8a2c9219fb09e38ed6e20a06bb33db830d1be  tools/r3_fixtures_table.py
a5abab9dac694b5e94cdce600855dbde79de91c8a71f545c6e59839df8648dab  tools/r3_probe.rs
c529b4832f07485bbf2896e95b00dc57e31a813d4ea2d4bb30b9caba4b6d64e6  tools/r3_summarize.py
a747956378eb030adb5ecc9fc8d76e1ebcc1d9416ef0fa79b87c6779d1eed892  tools/r3_worlds.rs
37dc528cfbb8526c21eeefd143bc7c03b0e25b8ea0376ebfc09b0a74d8a989bb  tools/r3_worlds_score.py
dea5b73ce7a08b22683ac3b5492972266eacad7edb743636ff73d66e5c9d351d  tools/r3_worlds_table.py
309317f65f646e4576b25d0932f44824854e649c24686bf7bb89d998f2c7ccbb  tools/rerun.sh
2228368543b02da1d7088635c9b98dba1508407f7b2fd52310d77a366336528c  tools/run_big.sh
6e01034b1261e8899939261b8028d7560c1e44b1fd3110169e4a2ce775bf6c40  tools/run_slice.sh
7e18305bfd8d06af9aa88316105b3e6e955f231523939ef77e027f5a4a5dcbbf  out/big-all.sum
83cb21e82d8de0fad991701e1879ccf080d722905c15cb24936c69c528a59e3f  out/big-o1000.sum
79d3d86d7174c86562a6aa7da19561038e865f88594c487080b057db5b50e6a0  out/big-o1050.sum
a9f8183311bda6b6a9273cc8583140e21284e25b97ffe0402634f43953636869  out/big-o1100.sum
a330f5c924a783881b4af4c41b5a98ec49213ba17f495c0d501c774f866c74ae  out/big-o1150.sum
dc0e0a406700551ff59c4923b1560435b6405cf0743d4b0698acba19a35594f6  out/big-o1200.sum
bf1561a77fa5d00b4412648f94aef63230798801092d0ddc66c1f538a12e5283  out/big-o1250.sum
207c0363bec3794415ed107c9235f19895b69e555fc3ce0d7d6dc902942e2fbe  out/extra_leak-20x24.sum
0fe47d33d9497981745503c502f0f79bc7dae8f9d5690c858799746b20291efc  out/extra_leak-60x31-o2000.sum
93ef07d7727651dbd33846741dff35ef9359f9328a32232b2a8a0e6ff019ed99  out/fast-20x24.sum
fe72651c1c974ea70125835a6eae5f90c15e1d97539be66b66581eb063e80a8f  out/fixtures.sum
01c9f365da6f3ec511720c2a83a8ffe437ed423733d7a13b743adc089f39cfaa  out/fixtures-table.txt
d8efd93f1c0fca81ee0d92454b2afbcabbb71db1b68fd1c84a2f68a30de8125f  out/k21.sum
7433f708c67de0a9eb6582b7142eec7c425425525e0894beeb2bcf6923e7698b  out/leak_every-20x24.sum
b82dac99ba126693233e3d0ea79ee13f4716cf85ae3b69e1d62d7b8356c4ca5a  out/leak_every-60x31-o2000.sum
3cb056b07a2efe8c40485a1b3ed899fdab366d242b49b77c848b3c9ecf218ad3  out/leak_once-60x31-o2000.sum
93ef07d7727651dbd33846741dff35ef9359f9328a32232b2a8a0e6ff019ed99  out/one_device-20x24.sum
e3d86a96a005c5d961c0285c44b756cfd5c9da182e1a2fc494b8f72fca017640  out/random-histories-N.txt
91046d56bb449c6c1a504ddf151d7c5f8d18e8fa082319e6b20a1bf85331c0e8  out/w-compaction.sum
e772b66ec899efe67ecc482c33beeadc9175f91d43014803b463cf3d367bce75  out/worlds-score.txt
44da90d921386bde7292f4428c0e0f8742750a829e8a9397c9766f4d40982113  out/worlds.sum
c0c007bcafd59443ce0cfb6fecbf1c236eb0abaca90beebd8b3e64992714b0d0  out/w-reuse.sum
64480a2f3ff6865957a63eae7cb1f4c1fd36c4b4a68a368969b42a4972986d35  out/w-rollback.sum
be5cdd50069128ae42d64f1210647d329039495bbf66e2578bf0040a47b3c586  out/w-snapshot.sum
```

## 一、判决条目逐条复核（站得住 / 够不着 / 同样打中别的臂）

下面「臂」的名字：today 今天（退回自证）、exempt 当隔离豁免（只当参照）、gen 自证加代号核对、genplus gen 再加头里 fsid 等于本池与已发布谓词、cmp 丁（比整单元 CRC-32C）、both gen 与 cmp 都过、bothplus genplus 与 cmp 都过、jrn journal 点名项比（第二节，清单漏掉的那一个）。归类器的名字：name 今天按不变量名比、j2n 甲二按名比、j2s 甲二按（盘, 槽）比（插回去的是最终那张盘）、j2t 甲二按（盘, 槽）比（每步镜像各插回去一张）。模型里的摘要在 `out/` 下，文件名对着档名。

| 判决条目（第二轮判决的原句） | 辩方判定 | 证据 |
|---|---|---|
| K2 行：「退回自证在 K2-1 那一形上误红，乙二合入之后仍是这样」（`research/prompts/m4-i315-r2-main-verification.md:34`） | 站得住 | HEAD 导出树 `k21` 档：today 臂 name 归类 `L+N{I-3.1,I-3.10,I-3.11,I-3.15}`，其余八个臂都是 `L+L`；300 段 today 臂 N=1（同一点）；比重 reuse 25 段 today 臂又撞一点（盘 0 那一份头里诞生代号 7、记录分配代 26，盘 1 那一份诞生代号 26），其余臂 0 |
| 同一行：当隔离豁免、丁、自证加代号核对、甲二都清掉这一形 | 站得住，同样清掉的还有 genplus、both、bothplus、jrn | 除 today 外的八个臂在 `k21`、300 段、快档、reuse / rollback / compaction / snapshot 四档比重上 N 都是 0（`out/random-histories-N.txt`）；甲二按名比（j2n）在 today 臂下也清掉这一点 |
| 同一行：「两腿都没造出让丁或自证加代号核对误红的合法历史」（`research/prompts/m4-i315-r2-main-verification.md:34`） | 随机历史上站得住；字节面上够不着 | 随机历史 300 段加四档比重 0 点分歧；固定盘面造出来了：cmp 在 4 个世界误红，gen 在 6 个，both 在 3 个，bothplus 在 2 个（第四节表）。第二轮辩方那句「读码推为造不出」不对，是我自己上一轮写的，这里更正 |
| 同一行（丁的盲区：两块盘同时丢写、都是同一个旧住户）：「一次只吞一次写的故障注入造不出」（`research/prompts/m4-i315-r2-main-verification.md:34`） | 站得住（harness 一次注入只吞一个盘上的写）；盲区形盘面造出来了，同样打中叠用 | 固定盘面 `ForeignUnitOnBothDevices` 等四个「两盘同一旧住户」世界 cmp 三条红；both 在 `SameIdentityOnBothDevices`、`RemadePoolSameBirthOnBothDevices`、`EarlierInstanceLeftoverOnBothDevices` 三个世界也红 |
| K2 第一轮更正：「快档 19 个吞写点里 7 点在有位置项的那一格红 I-3.11、I-3.15」（`research/prompts/m4-i315-r2-main-verification.md:35`） | 站得住 | 快档 19 点里插回去那张（不带外给清单）盘有 I-3.x 违例的 7 点，九个臂逐点相同（`out/fast-20x24.sum` 的底日志逐点比过）；红在有位置项那一支，被判这一格的候选碰不到 |
| K3：「当隔离豁免放过整整一类：单元完好、记录没释放、位置项已转出根环」（`research/prompts/m4-i315-r2-main-verification.md:36`） | 站得住；留下的候选同类上不放过，没有同样打中 | `leak_once` 60 步 × 31 段：exempt L24 / N9，其余臂（含 gen、cmp、both、jrn）与 today 同为 L4 / N39；固定盘面 `LeakWithBothCopiesIntact` 覆盖写 24 与 30 次：exempt 三条成立，其余八个臂三条红 |
| 同一行（只留一块盘的记录）：「那一种缺陷，所有候选都放过」（`research/prompts/m4-i315-r2-main-verification.md:36`） | 站得住，同样打中本轮每一个臂与每一种归类器 | `one_device` 20 步 × 24 段：九个臂 × 四种归类器摘要都是 `{'-': 7, 'L': 24}`，与无缺陷逐格相同 |
| 同一行（自证加代号核对放过泄漏之后槽被别的住户占了）：「但那一形 I-3.10 红」（`research/prompts/m4-i315-r2-main-verification.md:36`） | 站得住；同样打中 exempt、genplus、both、bothplus、jrn | `LeakThenForeignUnit` 覆盖写 24 / 30 次：这些臂 I-3.1、I-3.11、I-3.15 成立、I-3.10 红；today、cmp 三条红。同一张盘的字节与合法旧住户同形，I-3.10 在合法旧住户世界里同样红，分不开 |
| 同一行：「五个 checker 候选里只有丙分得开」（`research/prompts/m4-i315-r2-main-verification.md:36`） | 本轮不判；复做对上 | `ZeroedHeaderOnDeviceZero`（合法隔离）九个臂三条都成立，与泄漏加一份头坏逐字节同形；jrn 同样分不开：坏的那一份与点名项校验和对不上正是隔离的定义 |
| K3 甲二：「甲二按不变量名比，会把一个与被吞的写无关的泄漏归成说谎的设备」（`research/prompts/m4-i315-r2-main-verification.md:37`） | 站得住 | `leak_once` today 臂：name N39，j2n N37（2 点被归成说谎的设备） |
| C3：甲二收严成「每条违例点名的（盘, 槽）不红才归说谎的设备」（`research/prompts/m4-i315-r2-main-verification.md:45`） | 按原句字面（插回去最终那张盘）够不着；每步各插回去一张才够得着 | j2s 与 j2n 同为 N37，同样那 2 点：泄漏只在中间某步的镜像上红，最终盘 `after_names={}`，插回去也看不到；j2t N40 且六档缺陷 × 八个臂 0 翻。j2s 与 exempt 叠用时 `leak_every` 60 步 × 31 段翻 22 点（N47 → N25），j2t 不翻 |
| C2：丁「入库写法比整单元 CRC，以便与 GPU 一致，这一版还没跑过」（`research/prompts/m4-i315-r2-main-verification.md:44`） | 现在跑过，站得住 | cmp 臂就是比整单元 CRC-32C：`leak_once` 与 today 同数（L4 / N39，同攻方逐字节比那一版的数），`k21` 与固定盘面清掉 K2-1 |
| K4：「丁与自证加代号核对是「对得上」更严的读法，不与那一句冲突，定义行补一句即可」（`research/prompts/m4-i315-r2-main-verification.md:38`） | 站得住（补句措辞第六节，推的） | 第六节；jrn 更贴字面：点名项里的位置条目就是 D19 已定项 4 的 14 字节位置条目 |
| 不取当隔离豁免的两条理由：「又与用户 2026-09-25 的口径冲突」（`research/prompts/m4-i315-r2-main-verification.md:47`）加放过一整类泄漏 | 两条都站得住，都没有同样打中留下的候选 | 口径冲突：第六节表，exempt 字面冲突，gen / cmp / 叠用 / jrn 的豁免都只发生在 checker 自己核出不符之后；放过一整类：上面 K3 一行，留下的臂在该类上全红。留下的臂各有自己的另一类毛病：对合法旧住户误红（第四节表里加粗的「红」）、对「只留一块盘的记录」放过，那是它们的盲区，不是这一类 |

## 二、K1 候选的定义、最强版本与清单完整性

被判那一格今天两处同写法：`crates/singlefs-checker/src/walk.rs:8461`「None => check_unit(&unit).is_ok(),」、`crates/singlefs-checker/src/walk.rs:16239`「None => check_unit(&unit).is_ok(),」（位置项校验和取走过的各版，查不到时退回自证）。下表每个候选只改这一个分支的比对物；臂的实现是副本补丁 `tools/apply_arms_r3.py` 里 `defense_copy_matches_without_a_location_entry` 一个函数。

| 候选（臂名） | 判什么 | 输入从哪来 | 我造的最强版本 |
|---|---|---|---|
| 自证加代号核对（gen / genplus） | 这一份自证过，而且单元头里诞生代号等于罩住它的未释放记录的分配代；genplus 再要头里 fsid 等于本池、头里写序按已发布谓词判成已发布 | 头里诞生代号：`birth_txg_in_a_usable_unit_header`（`crates/singlefs-checker/src/walk.rs:7362`）；记录分配代：I-3.10 已有的那个等式；fsid 与写序也是头里已有的字段，已发布谓词是 `holds_for`（`crates/singlefs-checker/src/walk.rs:8609`） | genplus：多两样头里的字段，不动盘上字节；仍不分辨「同 fsid、同实例、同 txg、只有载荷不同」的旧住户，且已发布谓词在实例表里没有那个实例的行时一律判已发布（`crates/singlefs-checker/src/walk.rs:8618`「None => true,」），我造的旧实例遗留盘面没有行，它没起作用 |
| 丁 两份互比（cmp） | 这一份自证过，且另一块盘同槽那一份整单元 CRC-32C 相同；另一份读不出或不同算对不上 | 另一块盘同槽的字节，池里本来就有 | 入库写法比整单元 CRC，与 GPU 一致（GPU 里每份的整单元校验和已在 `s.c0` / `s.c1`）；多块盘时取同一单元的每个别的副本，我的副本只有两块盘，取「别的盘里第一块」 |
| 两者叠用（both / bothplus） | gen 与 cmp 都过 / genplus 与 cmp 都过 | 同上两样 | bothplus 是最强版：两样的各自最强版本相与。相与只会多豁免，不会少抓：泄漏、两份完好的世界与 `leak_once` 上与 today 同判 |
| 甲二按（盘, 槽）比（j2s / j2t） | 归类器在插回去那张盘上把实现交出、与被吞的写有重叠的单元当隔离单元再判；仅当违例点名的（盘, 槽）在插回去那张盘上仍红才算实现缺陷 | 实现的隔离清单（副本里取自 `copies_of_a_released_unit_failing_the_release_checksum_check` 那一处的线程局部日志）；违例点名的槽取自 checker 日志（I-3.1 / I-3.11 取未豁免的单元，I-3.15 取判红的记录），别的不变量没有槽信息，退回按名 | j2t：被判的是每步镜像的违例，就用同一步的镜像插回去那张盘比，不用最终盘比（`tools/r3_probe.rs` 的 `landed_prefix_length` 一个函数，15 行） |
| journal 点名项比（jrn / jrngen，**清单漏的**） | journal 环里有点名项指着这一（盘, 槽）、点名项的出生 txg 等于记录分配代时，拿点名项位置条目里的整单元校验和比；没有这样的点名项退回自证（jrn）或 gen（jrngen） | 点名项：D23 已定项 17「位置条目 14 × 2 + 类型 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1」（`.claude/kb/decisions/23-journal的角色与格式.md:36`）；位置条目就是 D19 已定项 4 的「设备 ID 4 字节、物理偏移 6 字节（16 KiB 槽号）、密文校验和 4 字节 ⇒ 位置条目 14 字节」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:80`） | checker 今天已经扫描并解出这些点名项（`crates/singlefs-checker/src/walk.rs:9938` 的 `judge_named_entries_against_the_unit_headers` 判 I-8.11），副本里把它们按（盘, 槽）收成表（`defense_note_journal`，13 行），不动格式 |

### 清单有没有漏

有：journal 点名项。依据：

- 第二轮两腿都只看了根环里的位置项（根环 3 区 × 每区 8 个槽：`crates/singlefs-format/src/lib.rs:286`「pub const ROOT_RING_REGIONS: u64 = 3;」、`crates/singlefs-format/src/lib.rs:304`「pub const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;」，覆盖写 24 次转出去，固定盘面里覆盖写 20 次还走有位置项那一支、24 次走没有位置项那一支）。journal 环默认 805306368 字节（`crates/singlefs-format/src/lib.rs:260`「pub const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;」），每条记录 4096 字节，约 196608 条，约是根环槽数的八千倍；点名项带的位置条目含整单元校验和，所以它给的是 D19 已定项 5 写者读盘核用的那个判据本身，只是换了一处取。
- 复做里它的覆盖：300 段无缺陷历史里回退点 95 次，95 次都有点名项（`out/big-all.sum` 的底日志 NOENT 行：`journal=Some(true)` 34 次、`Some(false)` 61 次，没有 `None`）；`leak_once` 里 48 次回退点有 6 次 `journal=None`，都是诞生代号 0 的单元（实例表槽、树表槽，mkfs 写的，不进 journal），这几次退回自证，与 today 同判（日志在草稿目录，`out/leak_once-60x31-o2000.sum` 的 N 数同 today）。
- 它的窗口有限：journal 环写满一圈就覆盖旧记录，而隔离单元与泄漏单元在盘上永远是「已分配、没根引用」，所以环转过一圈之后点名项没了，jrn 退回它的后备（推的，没量转圈之后的盘）。

### 考察后不成立的比对物（盘上已有、不动格式，但这一格用不上）

| 比对物 | 为什么用不上 |
|---|---|
| deadlist 条目 | 条目形态「key = 头的树 ID 8 + 纪元 8 + 中央映射 key 27 = 43 字节，value 空」（`.claude/kb/decisions/05-快照-空间记账机制.md:22`），没有位置也没有校验和；也还没实现 |
| 当版中央映射 | 隔离那一次发布「逻辑上照样释放（映射条目去掉）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:109`），没有条目可比 |
| 分配记录两盘对称 | 这是 C4 那条欠账的判据，分不开旧住户与泄漏 |
| 单元头里的映射 key 对映射 | I-3.15 已经在用（有 key 指着这一槽时走另一支），隔离之后条目已去掉，没有 key 可对 |
| 根环里没走过的根 | `roots_not_walked_yet` 已经补走（`crates/singlefs-checker/src/walk.rs` 的 `LocationEntryChecksumsForQuarantine`），「None」就是补走之后还没有 |

## 三、K4 后一半：第二轮判决第一、二节引的数在入库的 harness 档上复做

**哪一层是入库的、哪一层不是。** 入库的是 HEAD（841b3beab）导出树上的 harness 与池级 checker：`inject_faults_into_history_handing_out_the_images`、`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`、`check_pool_image` 一行没改。不入库的是臂的开关、探针目标和缺陷开关（副本补丁）。所以下面的数是「入库代码 + 副本补丁」上量的，不是登记过的入库装置的新一次跑；入库装置要主 agent 登记后跑（第十一节命令）。入库快档那一条原样跑过，不带任何补丁：

```
test fault_injection_fast_tier_returns_errors_instead_of_panicking ... ok
以「已知红」收尾 {}、新发现 0
```
（96 个注入点，13.2 秒；它不跑池级 checker，池级那一遍归 checker 档，所以下表快档一行靠探针。）

| 判决引的数 | 判决里的原样 | 这一轮的数 | 对上 |
|---|---|---|---|
| 快档 24 段 | 「快档各臂四列都是 {'-': 7, 'L': 24}」（`research/prompts/m4-i315-r2-main-verification.md:25`） | `fast-20x24`：九个臂 × 四种归类器都是 `{'-': 7, 'L': 24}`（19 个吞写点；N 全 0） | 对上 |
| 快档 19 点里 7 点「盘上完好的隔离」 | 「快档 19 个吞写点里 7 点在有位置项的那一格红 I-3.11、I-3.15」（`research/prompts/m4-i315-r2-main-verification.md:35`） | 插回去那张盘（不带外给清单）有 I-3.x 违例的点 7 个，九个臂逐点相同；带外给清单的插回去盘 0 个 | 对上（原句括注「腿报的」，这里是复做） |
| K2-1 种子 7463871032432356395、注入点 0 | today 臂「today=L[I-3.10]+N[I-3.1,I-3.10,I-3.11,I-3.15]」（`research/prompts/m4-i315-r2-opus-output.md:79`，判决 `research/prompts/m4-i315-r2-main-verification.md:34` 写「攻方复跑了第一轮那颗种子」） | `k21`：today 臂 name `L+N{I-3.1,I-3.10,I-3.11,I-3.15}`，j2n / j2s / j2t `L+L`；其余八个臂 name 都是 `L+L` | 对上 |
| `leak_once` 放大档（60 步 × 31 段、种子偏移 2000） | 「TALLY selfcheck today {'L': 4, 'N': 39}」（`research/prompts/m4-i315-r2-main-verification.md:18`）、「TALLY exempt today {'L': 24, 'N': 9}」（`research/prompts/m4-i315-r2-main-verification.md:18`）、「TALLY compare today {'L': 4, 'N': 39}」（`research/prompts/m4-i315-r2-main-verification.md:19`） | today name `{'L': 4, 'N': 39}`，exempt name `{'L': 24, 'N': 9}`，cmp name `{'L': 4, 'N': 39}`；gen、genplus、both、bothplus、jrn 同 today | 对上 |
| 300 段（60 步 × 300 段、种子偏移 1000，六片各 50 段） | 「TALLY today {'L': 239, '-': 49, 'N': 1}」（`research/prompts/m4-i315-r2-main-verification.md:21`），exempt 与 gen 摘要逐字相同 | today name `{'-': 49, 'L': 239, 'N': 1}`，其余八个臂 `{'-': 49, 'L': 240}`；j2t 在九个臂下 N 全 0 | 对上 |
| 固定盘面 `LeakWithBothCopiesIntact` | exempt 三条成立、gen 三条违例（`research/prompts/m4-i315-r2-main-verification.md:22`） | 覆盖写 24 与 30 次：exempt 三条成立；today、gen、genplus、cmp、both、bothplus、jrn、jrngen 三条违例；覆盖写 0 / 8 / 20 次（位置项还在环里）九个臂都是 I-3.11、I-3.15 违例 | 对上 |
| 「攻方 one_device 摘要与无缺陷那份 sha256 相同」 | `research/prompts/m4-i315-r2-main-verification.md:36` | `one_device-20x24` 九个臂 × 四种归类器与无缺陷同为 `{'-': 7, 'L': 24}`（格式不同，不比 sha256，逐格相同） | 对上 |

复做不了的：GPU 判器上的对拍（要 checker 档，按规则不起）；第二轮判决里「腿报的」几格（回退复活逐版 key 数 (1,1)、(0,0)、(1,1)）这一轮没复做。主工作区别的会话有 19 项未提交或未跟踪的 `crates/` 改动（`git status --short crates` 数出来），我用 HEAD 导出树避开它们；数与 r2 两腿在主工作区树上量的逐数相同，说明那些改动没碰这一格。

## 四、K2 合法盘与说谎设备的盘上会不会误红

### 随机历史（故障注入，归类器按 name 报）

被归成新发现（N）的失败条数，格里是「按名比 / j2t」两种归类器下的数；其余臂（exempt、gen、genplus、cmp、both、bothplus、jrn、jrngen）在每一档都是 0 / 0，只有 today 臂有数（`out/random-histories-N.txt` 逐档逐臂）。

| 档 | 吞写点 | today 臂 N | 其余八个臂 N |
|---|---|---|---|
| 快档 20 步 × 24 段（比重 broad） | 19 | 0 / 0 | 0 / 0 |
| K2-1 那颗种子（60 步，种子偏移 1282） | 1 | 1 / 0 | 0 / 0 |
| **放大档 60 步 × 300 段（种子偏移 1000，六片）** | 211 | **1 / 0** | **0 / 0** |
| 比重 reuse（60 步 × 25 段，偏移 3000） | 17 | 1 / 0 | 0 / 0 |
| 比重 rollback（60 步 × 40 段） | 28 | 0 / 0 | 0 / 0 |
| 比重 compaction（60 步 × 25 段） | 17 | 0 / 0 | 0 / 0 |
| 比重 snapshot（50 步 × 20 段） | 13 | 0 / 0 | 0 / 0 |

放大档里各候选剩几点新发现：exempt、gen、genplus、cmp、both、bothplus、jrn、jrngen 都是 0；today 剩 1 点（K2-1 那一点，同一颗种子）。比重 compaction_rollback、broad_compaction、writes_by_inode、truncation 没跑（机器上别的会话占着内存配额，snapshot 一档含排队就用了 43 分钟）。

### 第二轮判决点名的各形

| 形 | 结果 |
|---|---|
| K2-1（复用槽、位置项转出根环） | 见上表：今天误红，其余八个臂清掉 |
| 第二轮辩方固定盘面五个世界（覆盖写 24 与 30 次） | 下表前五行加 `LeakWithBothCopiesIntact` / `LeakThenForeignUnit`：与第二轮辩方的格逐格相同（today 三条红、exempt 与 gen 的行同第二轮） |
| 两块盘同时丢写、两份是同一个旧住户 | 造出来了：`ForeignUnitOnBothDevices`、`SameIdentityOnBothDevices`、`RemadePoolSameBirthOnBothDevices`、`EarlierInstanceLeftoverOnBothDevices`：cmp 四个全红；both 在 SameIdentity / RemadePool / EarlierInstance 三个「两盘」世界红；bothplus 只在 SameIdentity 与 EarlierInstance 两个「两盘」世界红；jrn 全静 |
| 旧住户诞生代号恰等于记录分配代 | 造出来了（下表）：gen 在 6 个世界误红，genplus 在 4 个，both 在 3 个，bothplus 在 2 个 |
| 隔离之后管理员回退复活 | 入库固定盘面三张（甲 隔离着、乙 写回原样未回退、丙 回退到 A）九个臂逐格相同（`out/fixtures-table.txt`）；rollback 比重 28 点 N 为 0。这一形走有位置项那一支，九个臂只改没位置项那一支 |
| 整理搬迁与隔离单元相邻 | compaction 比重 17 点 N 为 0；没有专门造「整理碰到隔离单元」的固定盘面（没做） |
| 快照保住的旧版 | snapshot 比重 13 点 N 为 0；快照引用的单元仍有位置项指着，走有位置项那一支（读码，臂只改 `crates/singlefs-checker/src/walk.rs:8461` 与 `:16239` 那两处 `None` 臂） |
| 重挂之后 | 随机历史里的可写挂载与注入后重开都在上面各档里；实现的隔离清单重挂之后为空（`crates/singlefs-core/src/recovery.rs:4130`「quarantined_after_release_checksum_mismatch: Vec::new(),」），只影响甲二的清单，副本的日志跨重挂不清，生产版要自己累积，这一项没做 |

### 固定盘面（覆盖写 24 与 30 次，位置项已转出根环，走没位置项可比那一格）

「红」= I-3.1、I-3.11、I-3.15 三条同红，「静」= 三条都成立；覆盖写 24 与 30 次两档逐格相同。

| 世界（槽上留的是什么） | 合法？ | today | exempt | gen | genplus | cmp | both | bothplus | jrn |
|---|---|---|---|---|---|---|---|---|---|
| `ForeignUnitOnBothDevices`：别池覆盖写出来的同一个完整单元（诞生代号 4），两盘相同 | 合法 | 红 | 静 | 静 | 静 | **红** | 静 | 静 | 静 |
| `ForeignUnitOnDeviceZero`：同上，只盘 0 | 合法 | 红 | 静 | 静 | 静 | 静 | 静 | 静 | 静 |
| `ZeroedHeaderOnDeviceZero`：盘 0 头扇区写零 | 合法隔离 | 静 | 静 | 静 | 静 | 静 | 静 | 静 | 静 |
| `DifferentForeignUnitsOnTheTwoDevices`：两盘各一个不同的旧住户 | 合法 | 红 | 静 | 静 | 静 | 静 | 静 | 静 | 静 |
| `SameIdentityOnBothDevices`：同 fsid、同实例、诞生代号也是 3，只有载荷不同，两盘相同 | 合法 | 红 | 静 | **红** | **红** | **红** | **红** | **红** | 静 |
| `SameIdentityOnDeviceZero`：同上，只盘 0 | 合法 | 红 | 静 | **红** | **红** | 静 | 静 | 静 | 静 |
| `RemadePoolSameBirthOnBothDevices`：fsid 不同的池（重新 mkfs）留下的诞生代号 3 的单元，两盘相同 | 合法 | 红 | 静 | **红** | 静 | **红** | **红** | 静 | 静 |
| `RemadePoolSameBirthOnDeviceZero`：同上，只盘 0 | 合法 | 红 | 静 | **红** | 静 | 静 | 静 | 静 | 静 |
| `EarlierInstanceLeftoverOnBothDevices`：主池的 A 由实例 2 写，槽上留着实例 1 同 txg 写的单元，两盘相同 | 合法 | 红 | 静 | **红** | **红** | **红** | **红** | **红** | 静 |
| `EarlierInstanceLeftoverOnDeviceZero`：同上，只盘 0 | 合法 | 红 | 静 | **红** | **红** | 静 | 静 | 静 | 静 |
| `LeakWithBothCopiesIntact`：记录没释放、两份完好 | 缺陷，应红 | 红 | **静（放过）** | 红 | 红 | 红 | 红 | 红 | 红 |
| `LeakThenForeignUnit`：泄漏之后槽被同一个旧住户占了（两盘相同） | 缺陷但字节与合法同形，I-3.10 另红 | 红 | 静 | 静 | 静 | 红 | 静 | 静 | 静 |

记分（`out/worlds-score.txt`，合法世界十个 × 覆盖写两档 = 20 格；`jrngen` 与 `jrn` 逐格相同）：today 不红 2/20、exempt 20/20 但放过缺陷世界、gen 8/20、genplus 12/20、cmp 12/20、both 14/20、bothplus 16/20、jrn 20/20；缺陷世界（泄漏、两份完好）除 exempt 外都红。

## 五、K3 坏盘上会不会放过

### 实现缺陷档（随机历史，缺陷开关在副本的 `transaction.rs` 释放循环里）

格里是被归成新发现（N）的失败条数，越多说明抓到的越多。「其余六臂」指 gen、genplus、cmp、both、bothplus、jrn，六个臂逐格相同。

| 缺陷（档） | today 臂：name / j2n / j2s / j2t | exempt 臂：name / j2n / j2s / j2t | 其余六臂：name |
|---|---|---|---|
| `leak_once`（60 步 × 31 段，偏移 2000，24 个吞写点） | 39 / 37 / 37 / **40** | 9 / 5 / 5 / 24 | 39，各归类器同 today 臂 |
| `extra_leak`（同一次发布里另泄漏一个无关单元；60 步 × 31 段） | 15 / 15 / 15 / 15 | 5 / 5 / 5 / 6 | 15，各归类器同 today 臂 |
| `extra_leak`（20 步 × 24 段） | 7 / 7 / 7 / 7 | 7 / 7 / 7 / 7 | 7 |
| `leak_every`（txg 模 7 余 3 的发布泄漏；60 步 × 31 段） | 47 / 47 / 47 / 47 | 47 / 47 / **25** / 47 | 47 |
| `leak_every`（20 步 × 24 段） | 30 / 30 / 30 / 30 | 30 / 30 / **29** / 30 | 30 |
| `one_device`（隔离只留核出对不上那一份；20 步 × 24 段） | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 | 0（与无缺陷逐格相同） |

读法：

- 泄漏档里，gen、genplus、cmp、both、bothplus、jrn 与 today 逐格同数：这些臂对「单元完好、两份相同、记录没释放」不放过（第二轮判决要的就是这一点，复做对上）；exempt 放过一大片（39 → 9）。
- `one_device` 九个臂 × 四种归类器全放过：checker 按单元判隔离，没有一条不变量要求两块盘的记录对称，这是各候选共用的前提，不分辨臂。
- 甲二按名比（j2n）把 `leak_once` 的 2 点归成说谎的设备（39 → 37，对上第二轮攻方的 2 点）。**按（盘, 槽）比、插回去最终那张盘（j2s）同样 37**：这 2 点的泄漏只在中间某步的镜像上红（`first_red={"I-3.11", "I-3.15"}`），最终盘 `after_names={}`，插回去也没有东西可比；这是 C3 原句的缺口，不是按名与按槽的差别。每步镜像各插回去一张（j2t）40，比 today 臂的 39 还多一点：`leak_once` 里另一个点（种子 7463871032432357143、注入点 0，吞写点是一次 1 MiB 写，没有隔离清单）的泄漏同样只在中间步红，今天的按名比、j2n、j2s 都把它归成说谎的设备，j2t 抓到了。
- j2s 与 exempt 臂叠用时 `leak_every` 翻 22 点（47 → 25）：exempt 下最终盘上泄漏单元被豁免，三条成立，观察到的中间步违例就被当成「插回去不红」。j2t 在六档缺陷 × 八个臂都不翻。
- 没造的：实现把一个与被吞的写重叠的完好单元错误地隔离（清单当权威，会把它洗成说谎的设备）、清单跨重挂拿不全、回退那条路交回的字段（`crates/singlefs-core/src/mount.rs:8645` 的 `quarantined_user_visible_copies`，HEAD 的行号）。

### 固定盘面上的坏盘

- 泄漏、两份完好、位置项已转出根环（`LeakWithBothCopiesIntact`，覆盖写 24 与 30 次）：exempt 三条成立（放过），其余八个臂三条红；覆盖写 0 / 8 / 20 次时九个臂都是 I-3.11、I-3.15 红。
- 泄漏之后槽被别的住户占了（`LeakThenForeignUnit`）：exempt、gen、genplus、both、bothplus、jrn 三条成立，I-3.10 红；today、cmp 三条红（两盘是同一个占用者，cmp 看不出）。字节与合法旧住户同形，I-3.10 在合法旧住户世界里同样红，所以这一形各候选都分不开，靠 I-3.10 报出「有不一致」，分不出是泄漏还是说谎的设备。
- 泄漏加一份头坏：与合法隔离逐字节同形（`ZeroedHeaderOnDeviceZero` 九个臂三条都成立），各臂都放过；jrn 也分不开（坏的那一份与点名项校验和对不上，正是隔离的定义）。
- 实现隔离路径上的缺陷：见上表。只留一块盘的记录全放过；多留一个无关单元（`extra_leak`）与同次发布的无关泄漏、单次泄漏（`leak_once`）都被抓到，甲二按名比有 2 点漏，按（盘, 槽）比每步插回去（j2t）不漏。

## 六、K4 前一半：条款文意与补进定义行的那一句

### 与 I-3.1、I-3.11 定义行是否相容

定义行两处同文：`crates/singlefs-checker/invariants/05-空间记账.md:8` 与 `:18`。豁免条件写的是「它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上」（`crates/singlefs-checker/invariants/05-空间记账.md:8`），后半句写的是「每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定」（`crates/singlefs-checker/invariants/05-空间记账.md:8`），括注里的用户口径是「核出坏的才豁免」（`crates/singlefs-checker/invariants/05-空间记账.md:8`）。

| 候选 | 与后半句 | 理由 |
|---|---|---|
| exempt | 字面冲突 | 没位置项可比时把读得出且自证过的一份也豁免，等于「没核出坏也豁免」，要改用户已定的口径（第二轮已判，复做的 `LeakWithBothCopiesIntact` 就是它放过的那一类） |
| gen / genplus | 相容 | 豁免只发生在 checker 自己核出头里诞生代号（fsid、写序）与记录对不上的时候；等式就是 I-3.10 已有的那一条，同一份文件里已经承认 |
| cmp | 相容 | 豁免只发生在 checker 自己核出两份不同的时候；它不知道哪一份是写者写的，但条款本来按单元豁免，不按份 |
| both / bothplus | 相容 | 豁免条件是两样各自条件的并集，每一样都是「核出坏」 |
| jrn | 相容，最贴字面 | 点名项里的位置条目就是 D19 已定项 4 的位置条目（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:80`），比的校验和就是写者读盘核用的那个（D19 已定项 5 硬规则 1，`.claude/kb/decisions/19-块指针的结构与宽度预算.md:109`）；只是位置条目从 journal 环取、不从根环取 |

`crates/singlefs-checker/src/walk.rs:8460` 的注释「这一格条款没写，照改之前的读法退回自证」：条款没写比什么，但前半句「checker 自己读出来读不出或校验和对不上」没限定是哪个校验和，退回自证、自证加代号核对、两份互比都是它的字面读法；jrn 是把「位置项」取得更宽，仍在字面里。所以这一句注释「对一半」（第二轮攻方）与「成立」（第二轮辩方）不矛盾：条款没写拿谁比，写了「对得上就照判」。

### 补进定义行的那一句（推的措辞，交书记员；I-3.1、I-3.11 各补一句，I-3.15 行「隔离的单元」那一格已写「照 I-3.1 那一份隔离判据」，随它，不另补）

- gen / genplus：「走过的版本与根环里别的根都没有一条位置项指着这一份时，这一份读得出且自证过，还要头里诞生代号等于罩住它的未释放记录的分配代（I-3.10 的等式；genplus 再要头里 fsid 等于本池、头里写序判成已发布）才算对得上；等式不成立算核出对不上。」
- cmp：同一个条件，后半换成「还要另一块盘同槽那一份的整单元 CRC-32C 与这一份相同才算对得上；另一份读不出或不同算核出对不上」。
- 叠用：两样都要过。
- jrn：「……journal 环里有点名项指着这一（盘, 槽）、出生 txg 等于记录分配代时，以点名项位置条目里的整单元校验和为准，与这一份对不上算核出对不上；没有这样的点名项时按（后备）。」后备取哪一个（自证 / gen / bothplus）是岔路的一部分。
- `crates/singlefs-checker/src/walk.rs:8460` 那句注释随之改成指向定义行。

D13 在 `.claude/kb/decisions/13-验证路线.md` 里故障注入、说谎设备那几句这一轮没引：开工快照里 kb 清单 13 份有 4 份对不上（`13-验证路线` 三份被别的会话改了一行路径写法，`03-丙.en.md` 在 HEAD 里没有这个文件），都与这一格无关；`sha256sum -c` 的原样输出见第〇节。

## 七、K5 改法与代价

「副本行数」是我的副本补丁里该段的实测行数（含日志，生产写法更小）；「估」全是推的，没有入库实现。按 `crates/` 今天的实现。

| 处 | gen / genplus | cmp | 叠用 | jrn | 甲二按（盘, 槽）比（j2t） |
|---|---|---|---|---|---|
| `crates/singlefs-checker/src/walk.rs` 两处 `None` 臂（`:8461`、`:16239`） | 两处换成一个共用函数，传进记录的分配代：I-3.1 那一份加一张（盘, 槽）→ 分配代的表，I-3.15 那一份从 `account.records` 取；副本行数：gen 5 行比较 + 两处管道约 25 行；估 +25–35；genplus 再加头里 fsid 与已发布谓词（副本 19 行 + 取池事实 3 行 + 钩子 6 行），估再 +25 | 副本 11 行；估 +12–15 | 两样相与，几乎不加行 | 副本：按（盘, 槽）收点名项表 13 行 + 查表 16 行 + 钩子；估 +35–45 | 不改 checker 的判，只要 checker 交回违例点名的（盘, 槽）：副本里靠日志 3 处共约 12 行加 `sites_of` 21 行，生产要结构化交回，估 +30–60 |
| GPU 内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`，一个函数供两组（`:1908`） | 函数多收一个记录分配代，读头里诞生代号（码 1 偏移 75、码 2 偏移 52 + 2k、码 3 偏移 73，同 `birth_txg_in_a_usable_unit_header`），估 +15–25；genplus 的 fsid 已是该函数的参数 `fsid_low`，已发布谓词内核里有没有可复用的没读，估再 +20–35 | 读另一块盘同槽的整单元校验和（`s.c0` / `s.c1` 已有）比，估 +10–15 | 同左相加 | 内核里没找到 journal 点名项表，要走读内核逐盘扫 journal 时收（盘, 槽, 出生 txg, 校验和）并传给扫描内核，估 +60–120（没读通走读内核，最粗的一格） | 不改内核（归类在 harness 侧） |
| 对拍逐位比 | CPU 与 GPU 同改，对拍照旧逐位比 | 同 | 同 | 同，且多一张表的对拍 | 不涉及 |
| 不变量定义行 | I-3.1、I-3.11 各补一句（第六节） | 同 | 同 | 同，另写后备 | 不动 |
| 故障归类器 `crates/singlefs-harness/src/fault_injection.rs`（`:2576` 按名比那一处） | 不动 | 不动 | 不动 | 不动 | 清单收齐（发布、删快照、整理、回退四条路，跨重挂累积，回退那个字段是 `crates/singlefs-core/src/mount.rs:8645`，HEAD 行号）估 +40–80；每步插回去一张：副本里 `landed_prefix_length` 15 行加用法 15 行，估 +30；两项都包括「仍红」按（不变量, 盘, 槽）认的 19 行 |
| 必绿 / 必红世界 | 必绿：K2-1 那一形；必红：`LeakWithBothCopiesIntact`（副本里 `build_world` 约 80 行含十二个世界的框架）；它的盲区形要先定记「已知盲区」还是「必绿」（岔路） | 同，盲区形是两盘同一旧住户 | 同 | 同，另一个必红：点名项存在而单元被换 | 必红：一个无关泄漏 + 一个吞写隔离同在一段历史里，归类要报新发现（`leak_once` 类） |
| 变异行 | 两条：拿掉比较退回自证 → 必绿世界红；比较恒真 → 必红世界红 | 同 | 同，另一条拿掉一半 | 同，另一条：查表恒找不到 | 两条：按名比退回 → 无关泄漏用例红；最终盘比退回 → 中间步泄漏用例红 |

没读到的：GPU 内核里已发布谓词的位置、走读内核里 journal 的收法、这些行数在 GPU 上的真实代价。

## 八、交用户的岔路（候选、翻面观测、够判条件；不带倾向）

标「零轮」的是这一轮才出现、没被攻过的形态；「已量」是副本补丁上量的、要主 agent 在入库装置上重做才能引。

| 岔路 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| 「没有位置项可比」那一格 checker 怎么判 | ① 退回自证（今天）；② 自证加代号核对 gen；③ gen 加 fsid 与已发布谓词 genplus（零轮）；④ 丁 cmp；⑤ 叠用 both / bothplus（bothplus 零轮）；⑥ journal 点名项比 jrn，后备另选自证 / gen / bothplus（零轮） | K2-1 类误红的频率：已量 300 段 1 点、reuse 比重 25 段 1 点、快档 0 点。两块盘同时丢同一次写算不算说谎的设备的合法形：已量 cmp 在四个「两盘同一旧住户」世界红，随机历史 0 点。同 txg 的旧住户（重新 mkfs 的池、早先实例遗留、同实例同 txg 重试）要不要挡：已量 gen 6 个世界红、genplus 4 个、bothplus 2 个、jrn 0 个。journal 点名项的覆盖：已量 300 段无缺陷回退点 95/95 都有，mkfs 写的单元（诞生代号 0）没有；环转过一圈之后没量。点名项里的位置条目算不算定义行里的「位置项」；同（盘, 槽, 出生 txg）有两条校验和不同的点名项时 jrn 的误红：没造 | 候选在入库装置上各复跑一遍第三节那几格（快档、K2-1、`leak_once`、300 段、固定盘面）；选「要挡」同 txg 旧住户的，补一档「早先实例遗留」盘面（实例表里有那个实例的行）量已发布谓词起不起作用；选 jrn 的，补 GPU 内核收表的成本数与环转圈之后的盘；「位置项」读不读到点名项由用户定 |
| 故障归类器怎么接实现的隔离清单 | ① 今天按名比；② 甲二按名 j2n；③ 甲二按（盘, 槽）比、插回去最终那张盘 j2s；④ 甲二按（盘, 槽）比、每步镜像各插回去一张 j2t（零轮）；⑤ 不接清单，只靠 checker 那一侧 | `leak_once` 24 点：①、②、③ 各漏 1–3 点（N 39 / 37 / 37），④ 不漏（N 40）。③ 与 exempt 臂叠用 `leak_every` 翻 22 点，④ 在六档缺陷 × 八个臂 0 翻。没造：实现把重叠的完好单元错误地隔离、清单跨重挂拿不全、回退那个字段 | ④ 的实现版（清单跨重挂累积并收回退字段、每步插回去、checker 交回违例点名的槽）在入库装置上带 `leak_once` / `extra_leak` / `leak_every` 各跑一遍 0 翻；补一档「错误隔离重叠单元」缺陷；若选 ⑤，要定「无关泄漏混进说谎的设备那一类」记欠账 |

## 九、什么现象会推翻我的结论

- 「cmp 的盲区只在两盘同一次丢写」被推翻：一次注入只吞一个盘上的写的随机历史里 cmp 臂 N 大于 exempt 臂（今天 300 段加四档比重都是 0）。
- 「gen / genplus / cmp / both / bothplus / jrn 在随机历史上不误红」被推翻：入库装置上任何一档这些臂的 N 大于 0（今天 `out/random-histories-N.txt` 全 0）。
- 「j2t 不翻」被推翻：任一缺陷档里 j2t 的 N 少于按名比的 N（今天六档缺陷 × 八个臂没有）。
- 「按（盘, 槽）比的缺口在最终盘」被推翻：`leak_once` 那 2 点的泄漏在最终盘上也看得到（主工作区改了别处使 `after_names` 不再为空），则 j2s 与 j2t 同数。
- 「jrn 在十个合法世界全静」被推翻：造出一张盘，同（盘, 槽, 出生 txg）有两条校验和不同的点名项且旧住户那条在环里，jrn 红。
- 「genplus 的已发布谓词没起作用」被推翻：造一张实例表里有早先实例的行、行里已发布 txg 低于旧单元 txg 的盘，genplus 在 `EarlierInstanceLeftover` 两个世界静。
- 「第二轮判决那几格对上」被推翻：入库装置上登记后跑出的数与第三节表不同。

## 十、没做什么

- 没在登记过的入库装置上跑：臂、探针、缺陷开关都是 HEAD 导出树上的副本补丁，数不进 kb。
- 没起 checker 档、GPU 判器、崩溃点重放、QEMU；GPU 一侧的行数是读 `crash_judge_checker_scan.wgsl` 估的。
- 没造的盘面：真的崩溃恢复留下的「早先实例遗留」（实例表里有行）、同（盘, 槽, 出生 txg）两条点名项、journal 环转过一圈、实现错误隔离重叠的完好单元、清单跨重挂累积与回退字段的实现。
- 没跑的比重：compaction_rollback、broad_compaction、writes_by_inode、truncation；没跑三块盘以上的池（cmp 在多盘取「别的盘里第一块」）。
- 回退复活逐版 key 数（第二轮「腿报的」）没复做；D13 三份被别的会话改过，没引。
- 同（盘, 槽）同 txg 重试的可达性（要发布失败后同进程重试再被吞一次写）是读码推的，没验。
- 没做 git 写操作，没改主工作区任何文件；只写了报告、`research/prompts/m4-i315-r3-sonnet-model/` 与草稿目录。
- 不替主 agent 撤销或维持判决，只交复核结果。

## 十一、要主 agent 跑的命令

1. 入库装置上重做（跑前登记要写明：核实第二轮判决第一、二节哪几句，对着第三节哪几格）。HEAD 导出树当冻结副本，整条经内存包装，线程取登记值 31，工作目录仓根；放大档与比重档要 `RERUN_FULL=1`，挂钟长（我这一台上不带 `RERUN_FULL=1` 的一遍用了 88 分 41 秒，含排队和别的会话占着磁盘的拖慢；放大档六片每片 2 到 5 分钟（不排队时），snapshot 比重一档含排队 43 分钟），要 `run_in_background` 起、不设超时：

```
D=~/.local/share/singlefs/work/m4-i315-r3-rerun && mkdir -p $D/frozen && git archive HEAD crates Cargo.toml Cargo.lock .cargo .claude/heavy-thread-default | tar -x -C $D/frozen
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=辩方腿模型 rerun.sh 里 cargo test -p singlefs-harness 的 r3_probe、r3_worlds、r3_fixtures、fault_injection_fast_tier 四个测试目标; 范围=4 个测试目标; 为什么=端到端复跑 m4-i315-r3 辩方腿模型，核每档摘要与 out/ 逐行相同' RERUN_FULL=1 RERUN_THREADS=31 bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-i315-r3-sonnet-model/tools/rerun.sh --repo "$PWD" --frozen $D/frozen --work $D/work
```

看输出里每一档 `name=rerun_compare config=<档> same_as_out=true`（快档、k21、leak_once、extra_leak、one_device、leak_every、worlds、fixtures 八格不带 `RERUN_FULL=1` 也跑；放大档 `big-all`、`extra_leak-60x31-o2000`、`leak_every-60x31-o2000`、`w-rollback`、`w-reuse`、`w-compaction`、`w-snapshot` 七格要 `RERUN_FULL=1`），以及 `name=in_repo_fast_tier 以「已知红」收尾 {}、新发现 0` 一行。我这一台上不带 `RERUN_FULL=1` 的八格已端到端复跑，八格都是 `same_as_out=true`（原样输出在第〇节）；带 `RERUN_FULL=1` 的七格没有端到端复跑（逐片用 `run_slice.sh` 跑过，摘要就是 `out/` 里那几份）。
2. 上面一条是副本补丁上的数，进 kb 之前要把「臂」做成入库装置的新一次跑；入库装置的写法归实现员，这一轮不写。

## 十二、草稿目录清理

仓副本与编译目录已删，不再用：HEAD 导出树的副本 `~/.local/share/singlefs/work/m4-i315-r3-defense/tree`（含 `target`，1.3G）、冻结副本 `~/.local/share/singlefs/work/m4-i315-r3-defense/frozen-head`（35M）、端到端复跑的 `~/.local/share/singlefs/work/m4-i315-r3-defense/rerun-work`（含它自己的 `tree/target`，1.3G）；要复现走模型目录的 `tools/rerun.sh` 重建。另外删了我被超时杀掉的那一次入库固定盘面探针留在系统临时目录里的 14 个镜像文件（文件名里带那个进程号）。

留下的：同目录下 `logs/`（各档原始日志；`logs/old/` 是旧版探针的日志，作废、不引）、`sums/`、`progress.md`、`tools/`（模型源码的工作底稿，与模型目录同名文件相同）。原始日志不入库的原因：副本上量的数，按规则须由主 agent 在入库装置上重做才能引，摘要已进模型目录 `out/`。
