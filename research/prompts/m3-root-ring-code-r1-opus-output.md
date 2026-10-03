# m3-root-ring-code-r1 攻方腿报告（Opus，第一轮，2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 R2 B1 B2 E1 E2 -->

被判代码取冻结副本 `/tmp/claude-1000/m3-root-ring-code-r1/tree/`（只读），拷进草稿 `/tmp/claude-1000/m3-root-ring-code-r1-opus/` 再改再跑。引代码一律写冻结副本的绝对路径与它自己的行号（主工作区的 `crates/singlefs-checker/src/walk.rs` 在本轮之中已被别的会话改过，行号与冻结副本不同；其余被引文件两边逐字节相同）。副本上的数都是副本上的数，不算入库装置上的数。只跑了 harness 档能跑的东西：原型 crate 的二进制经 `run-with-memory-cap.sh`；原型调了 `singlefs-checker-tier` 的库函数 `enumerate_layer0_versions`，没跑任何 checker 档测试目标。

## 复跑

```
bash research/prompts/m3-root-ring-code-r1-opus-model/scripts/rebuild-and-run.sh <新草稿目录> build k2e k2e-enum k2b-1 k2b-2 k2b-more-1 k2b-more-2 k2b-fix-1 k2b-fix-2 k1k3k4-1 k4-enum b04 b13 b15
```

脚本把冻结副本拷进 `<草稿目录>/tree`、`patch -p1` 打 `core-and-checker.diff`、放进原型 crate `rr-attack-proto`、编译（`scripts/build.sh`：`run-with-memory-cap.sh 6G` + `capped.sh 15`），每件经 `scripts/run.sh`（`run-with-memory-cap.sh`，上限默认 1G、`capped.sh`，线程默认 1；枚举那几件 4G–6G、7 线程）；并行件经 `scripts/par.sh` / `scripts/fz-batch.sh`，每件退出码写 `out/b<批>-<件>.rc`，开跑前先删这一批的 `.rc`，最后单独 `wait`、数 `.rc` 个数。每批件数乘线程数合计不超过 15。`results/out/` 里的 116 份输出就是这条命令在新草稿目录 `/tmp/claude-1000/m3-root-ring-code-r1-opus/rerun/` 上跑出来的（与我开发时的草稿逐字节相同，层 0 枚举那 5 份只差 `LAYER0_PROGRESS` 进度行的次序与线程数）。

`core-and-checker.diff` 只动冻结副本三处：`Cargo.toml` 加原型 crate；`allocator.rs` 加只读访问器 `proto_unreadable_at_mount_highest_txgs`；`mount.rs` 加测试入口 `proto_unreadable_ring_slots_inside_the_window`（原样调 `unreadable_ring_slots_inside_the_window`）与改法 B1 的开关 `SINGLEFS_PROTO_RAISE_KEEPS_UNREADABLE`（不设时与冻结副本行为相同：`results/out/r*.out` 与 `s*.out` 的 720 段逐行相同）。

模型目录 `research/prompts/m3-root-ring-code-r1-opus-model/` 的文件 sha256（`results/out/` 116 份输出与 `results/fz-summary.txt` 的 sha256 在 `results/SHA256SUMS`，`sha256sum -c` 不是 OK 的 0 行）：

```
01eee5d67f7df0de935e981f1e846b5076d763d5f8378757174962173719121c  ./core-and-checker.diff
c4eeceff7eed74fdd9e2778c6d5e92e6fdc941876ebe039d3b3e02ae80079745  ./results/fz-summary.txt
8ae202c5c510d384920d93933ac3d546c8b665d5042d2321b46a754ea0ff9006  ./results/SHA256SUMS
dac33e0fb5babd22caf1be59ba341fc6e42eb8fad2c9ffaecbc09aebcf30757b  ./rr-attack-proto/Cargo.toml
631cf23e1633df5aa659014bdbf0b7b977cb32ec02b6e5addf94458b972ab1b1  ./rr-attack-proto/src/bin/first_file_record_only.rs
92f170a2d1617a918db187578c386cb1109822daba05342dcd712c21a9f99dbe  ./rr-attack-proto/src/bin/marker_bits_probe.rs
9d13947d92066aa86458028442247cf85b0b087cde6c2bc38ed16995fb9d1aa7  ./rr-attack-proto/src/bin/raise_floor_hidden_slot.rs
87e060edd04b824fd8a1b881fe06210e17c2e7a53736d0f6d5ef7b5e3a9d0b9f  ./rr-attack-proto/src/bin/sysconf_r2_forever.rs
0b9a8620c7bc9841ad6fc99970b93b19710c61c1ccfb68dcef361e7274fb67b3  ./rr-attack-proto/src/bin/unreadable_slot_i31.rs
d6b2d3a9fde0b41c79b6c0babbda08f9f94f4af53b71b02629cff161958d4115  ./rr-attack-proto/src/bin/window_fuzz.rs
8ea588a56d53a49ffc5ef1d920244f6c37a7dfd699f6a93e7ed75192745497bb  ./rr-attack-proto/src/bin/window_loop_equiv.rs
d0b4023a92c8a8e46c4f698a5a5161b056d8e1e10d98c2d09b4253a7430c3303  ./rr-attack-proto/src/lib.rs
8080c495e7f90ff3b4a307bda25bbc2a071c30e2ad3457571c535aaabf1c522d  ./scripts/build.sh
1ab3ff15aee85899d15edd9f3b8515f51c1b20b2bf73cbba8cab18866823a478  ./scripts/fz-batch.sh
706bc60535b9a6a384885897bbd22300581716cdeb22f1d22b341fe7cdcc6086  ./scripts/fz-summary.py
2705e839128b2cf5f1920fd7c9837046e60b05b45a1b267008af875cae65fe75  ./scripts/par.sh
e810934082559df834b00204087c9c325e4eed4d089d79ea914225e571052b54  ./scripts/rebuild-and-run.sh
bc35a517f89ad92790a6b08f62304f70bb11316c4e21f18fa4b311b5d98cb4ee  ./scripts/run.sh
f8140d5db31f7faccf050d57b4023a18510007d4b9338e34c58286c153a31bd7  ./scripts/specs/b04.txt
9254827036a39acb214e6845aec136d760cd0e4c733582291e75f7ee1b3de391  ./scripts/specs/b13.txt
433fd14dca4ca12112e9121ad440bc26a5d055c737ce813be0ac3a902051ae6b  ./scripts/specs/b15.txt
b7b544e18af96fefbb3763eca58d6afed864a45d25e8c5fe056e2ec4ecc61cc5  ./scripts/specs/k1k3k4-1.txt
e7e92ff9cbbf403445c711170123ad754670b9460985d9262c2fb5560d183da8  ./scripts/specs/k2b-1.txt
3752f058221c2abc4c26f040e85dc02f0d7fff93e3712b595cc3845faac4fbf6  ./scripts/specs/k2b-2.txt
08d8f32328e531788842a653b41a7e09aecb3e420c89e535988ad1216b04b60c  ./scripts/specs/k2b-fix-1.txt
1c9e18ed95a4c88d9621f6a748e8f6890f1bc90f396425f7e71d278d86e4625a  ./scripts/specs/k2b-fix-2.txt
5e8b894310b6bc7613d7401e79b8fdc4011d294892854cf097b386bb43a7f2a8  ./scripts/specs/k2b-more-1.txt
50160e028a4f9f44b01614205ad5186b24a3bc1fef4739cc75ad4e561782c53e  ./scripts/specs/k2b-more-2.txt
e0a6925a6d311ed86008c11e4163efdf1fee5422c9728fb6e35aece69957b904  ./scripts/specs/k2e-enum.txt
806e8afcb4743623ce380036fdc5a830435d4e896e8c474619f0235d10c6d9ea  ./scripts/specs/k2e.txt
3f96522909e458dbaab152851613cb2c937bed597ced627a2f85020e4c07a7b8  ./scripts/specs/k4-enum.txt
```

原型由 `research/prompts/m3-ring-r3-opus-model/ring-window-proto/` 的 `lib.rs` 与 `window_fuzz.rs` 拷来（文件头写了改了什么），其余五个二进制是这一轮新写的。

## 各格判定一览

| 格 | 判定 | 依据（量过 / 推的） |
|---|---|---|
| K2-e 挂载时读不出的槽「门槛取最小、根环表记最大」 | **打中**（合法历史上池级 checker 判 I-3.1 红）：挂载那一次一个根环槽读不出（瞬时读错，挂完就好），根环表把它记成 txg 为「它能住的最大 txg」h 的有效根，进程写到锚 A = h + 3S − 1 那一版时回收门槛停在 h，而盘上那一槽并没有 txg h 的根，checker 按盘取窗口里最旧有效根 = h + 1，记账的已分配比遍历多 | 量过：S = 4 / 8、环没转过 / 转过、只在挂载那一次 / 整条命读不出共 7 组都在 A = h + 3S − 1 红、同一段历史不注入读错不红；层 0 枚举 txg 22–25 四次发布各 4108 态：A 那次 9 态、A + 1 那次 4099 态红，对照臂 0；随机历史 720 段里 5 段 I-3.1 红，其中 3 段（种子 30114、30247、30249）查过是这一形（记账多于遍历 131 072 字节，红在 h + 3S − 1），另 2 段带「藏最新根」那一招、I-3.11 同红，没细分 |
| K2-b 抬 F 那一遍 | **打中**（合法历史上 I-3.1 红，方向是少扣：被回收的槽仍被一条窗口里、F 之上的有效根引用）：抬 F 读根环时，挂载那一刻读不出、之后仍读不出的槽当没有根，回收门槛 max(新 F, 窗口里读得出的最旧有效根) 越过那一槽里的根 X；窗口把洞里上一圈的旧根关在外面之后，没有别的根替 X 压门槛 | 量过：窗口最旧端 2–6 个洞 + X 读不出，29 组里 19 组红（不红的 10 组：5 组根本没抬成 F，4 组环还没转过一圈、X 落到新 F 之下，1 组抬 F 那一串要发三次、第三次盖掉了 X），X 读得出的同历史对照 20 组 0；层 0 枚举抬 F 之后三次发布 4108 / 4099 / 0 态红；改法 B1（抬 F 那一遍把根环表里的读不出槽按最大 txg 算进门槛）同几何 0，层 0 三次发布全 0（零轮）。被回收的槽真被复用、checker 判 I-7.4 的那一形没造出来 |
| K2-c I-3.1 丁-defer 门槛 | 打中的两格都落在它上面（K2-e 是 checker 判多、K2-b 是实现回收多），单独造不出别的盘面 | 量过 + 推的 |
| K2-a 挂载时的窗口（锚 N 与读不出槽按最小 txg） | 没打中：挂载那一刻的门槛只更低，写行那次发布记根时按进程内的锚重算、记账行按重算之后的数写 | 推的 + 随机历史 1020 段（790 段带读故障）里没有归到它的红 |
| K2-d 影子账的候选过滤 | 造不出：过滤只让候选集变小、隔离只多不少；隔离位只住内存，不进记账行，池级 checker 不判它 | 推的（按代码） |
| K2-f 头尾两段循环与逐 txg 原写法；`target_for_publish` 的周期 | 没打中：170 326 组输入（S 4..16、N ≤ 60 × first_txg ≤ 90 全域、贴着 u64::MAX 的 26 000 组）0 处不同；周期是 3S（txg ↦ (txg mod 3, (txg div 3) mod S) 在任意 3S 个连续 txg 上是双射） | 量过（测试入口原样调被判函数） |
| K2-g 核心层与池级 checker 两份窗口的边界 | 没打中：两边都写成 txg + 3S > N、加法饱和；txg = N − 3S 两边都在窗口外，N < 3S 两边都整环在内 | 推的（按代码逐字比） |
| K2-h 四处的锚 | 挂载取环里读得出的最新有效根、进程内取这个进程记到的最新根、抬 F 取现行那一版的根、checker 取镜像里最新的根，与 D16 已定项 1 那一行字面相同 | 推的（按代码） |
| K3-a R2 谓词 | 字面相同；撕裂、fsid 不符、全零都落在「判得出」那一边 | 推的 + 量过（k3-b/c/d） |
| K3-b 三处同一处定义，有没有地方还留着取甲 | **打中（旁及，不在 core 的三处里）**：被判文件 `e165_replay_bounded_enumeration.rs` 的读故障族生成器仍按取甲建可写挂载那道闸（自证过的槽不足 4 份就判不出），产物配置行写着它模型的是实现的取甲；core 三处都走 R2 那一处定义 | 推的（按代码；E165 是 checker 档装置，没跑） |
| K3-c R2 加留着的「某块盘一份都没有就拒」会不会让池永远只读 | 没打中条款：要一块盘两槽都坏（两处故障落在同一块盘），已在「每盘一槽」承诺之外；崩溃撕不坏系统配置槽（489 字节都在第一个 512 字节扇区里，撕到 512 处与整写相同） | 量过（k3-b/c/d/f）+ 推的 |
| K3-d 改钉的两条判拒用例 | 判的事换了：取号那条从「一槽读不出就拒」改判「一块盘一份自证过的都没有」（留着的那道拒），不再判 R2 的判不出；见 K3 一节 | 推的（读 diff） |
| K1 新根段三字段与施加 | 施加七项取末条记录、fsid 照所选根，三处装记录都取这次写的根，与字面相同；**卸载记号「位 0 之外有位为 1 当损坏」只有 core 一边**：checker 的 `check_journal_record` 不判它，同一条记录 core 拒、checker 收 | 量过（mb-2/3/80000000）；只在头部校验和对得上的外来字节上走得到，合法崩溃历史走不到 |
| K4 池级 checker 跟上 | 没打中：写行之后第一个文件版本崩在记录已持久根槽没持久、下一次可写挂载施加它，那条挂载流层 0 全量 20 511 态 checker 0 违例（I-3.1 判了 4116 态、I-4.1 16 398 态、I-3.10 20 511 态）；(实例代号, 计数器) 两边同一个键；checker 只依赖 `singlefs-format`，不与 core 共用窗口与键的代码 | 量过 + 推的 |
| 旁及 | ① 读不出的槽在算上限时当没有根，checker 按盘算上限 ⇒ I-7.9 红（C563 那一类，早于 7-2）；② 不带读故障的随机历史 230 段里 24 段 I-7.9 红（m3-ring-r3 判决第 12 行那一件，今天的代码上还在）；③ 候选类命中 13 段全是 C554 那一形（被藏的最新根是候选、下一次挂载写行的单元盖了它的槽） | 量过，没查因 |

## K2　一圈窗口

### K2-e　挂载时读不出的槽：打中

**条款**：`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`：「挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（零轮，只在攻方模型上量过）。」

**实现**（冻结副本）：挂载时的门槛按最小 txg 算（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:1772`：「// 窗口里有效根的最小 txg；挂载时读不出的槽按它在窗口里能住的最小 txg 算（门槛只会更低）。」）；根环表按最大 txg 记（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:1808`：「// 挂载时读不出的槽在根环表里按它在窗口里能住的最大 txg 记成一条有效根（它在进程内的窗口里待到这个进程写到那个槽为止）；」）；进程内每记一条根时，这一项照「有效根」进最小值（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/allocator.rs:1033`：「} => Some(*highest_txg_it_can_hold),」）。checker 的 I-3.1 是等式（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/walk.rs:7352`：「judgements.judge("I-3.1", allocated == Some(walked), || {」），丁-defer 门槛按镜像现算（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/walk.rs:7290`：「// 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 一圈窗口里最旧有效根)（D16（发布语义） 已定项 1，五处同改之一；」）。

**造法**（`unreadable_slot_i31.rs`，每一步都是许可的）：mkfs → 可写挂载 → 第一个文件 → 覆盖写到 txg 11 → txg 12 的覆盖写崩在根槽写之前（记录已持久、根槽没持久，层 0 的合法截点）→ 下一次可写挂载（实例 3）时根环槽 slot(12) = (0,0) 那一次读报块设备错（只在挂载那一次，挂完就好）→ 之后照常覆盖写。表里记成 highest = 12；进程写到 A = 12 + 3S − 1 = 23 那一版时，进程内的门槛 = 12，而盘上 (0,0) 住的是 txg 0（窗口外）、根本没有 txg 12 的根，checker 取窗口里最旧有效根 13。释放代 13 的那几个槽实现还扣着、算在已分配里，checker 的遍历不算它们。

```
$ grep -E '^(HOLE|MOUNTED)' u31-s4-9-mount.out | cut -c1-200
HOLE txg=12 slot=(0,0) device=0 occupant_txg=Some(0) newest_readable=11
MOUNTED instance=3 row_txg=13 occupant_in_table=Some("RootRingSlot { region: 0, slot: 0 }: UnreadableAtMountInsideTheWindow { highest_txg_it_can_hold: CheckpointTxg(12) }, RootRingSlot { region:")
$ grep -E '^AFTER txg=2[2-4]' u31-s4-9-mount.out u31-s4-9-none.out | grep -oE '^[^ ]+ txg=[0-9]+ red=\[[^]]*\]'
u31-s4-9-none.out:AFTER txg=22 red=[]
u31-s4-9-none.out:AFTER txg=23 red=[]
u31-s4-9-none.out:AFTER txg=24 red=[]
u31-s4-9-mount.out:AFTER txg=22 red=[]
u31-s4-9-mount.out:AFTER txg=23 red=["I-3.1"]
u31-s4-9-mount.out:AFTER txg=24 red=[]
```

txg 23 那一行的原文（`results/out/u31-s4-9-mount.out`，原型只印前 160 字）：「盘 0：记账的已分配 Some(2949120)，遍历全部有效根得到 2785280（其中隔离豁免 0、F 之下仍在 defer 的 0）；机理：根环槽数 12、最新根 txg 23、环里自证过的根槽 12 个、最老的自证过的根 txg 0、遍历的候选根槽 11 个、并进遍历的由记」。差 163 840 字节 = 10 个槽。

层 0 全量（1 MiB journal、128 MiB 盘、挂载之后换成空发布，状态数才落在 10⁶ 以内；`ENUM_TXGS=22:23:24:25`，`enumerate_layer0_versions` 每段都展开、每个状态恢复之后跑池级 checker）：

```
$ grep -h '^ENUM_TALLY' u31e-small-E-mount.out u31e-small-E-none.out | grep -oE 'txg=[0-9]+ states=[0-9]+ oracle_violations=[0-9]+|I-3\.1=[0-9/]+' | paste - -
txg=22 states=4108 oracle_violations=0	I-3.1=4108/0/0
txg=23 states=4108 oracle_violations=0	I-3.1=4108/9/0
txg=24 states=4108 oracle_violations=0	I-3.1=4108/4099/0
txg=25 states=4108 oracle_violations=0	I-3.1=4108/0/0
txg=22 states=4108 oracle_violations=0	I-3.1=4108/0/0
txg=23 states=4108 oracle_violations=0	I-3.1=4108/0/0
txg=24 states=4108 oracle_violations=0	I-3.1=4108/0/0
txg=25 states=4108 oracle_violations=0	I-3.1=4108/0/0
```

（前四行注入挂载那一次的读错，后四行不注入；`I-3.1=评估/违例/不适用`。）txg 23 那次发布根落盘之后的 9 态、txg 24 那次发布根落盘之前的 4099 态红，其余 0。

**放开的维度**（`specs/k2e.txt`）：S = 4 / 8；环没转过（洞那一槽从没写过或住着第 0 代根）/ 转过（住着上一圈的根 txg 12、9）；读错只在挂载那一次 / 整条命都在。7 组全在 A = h + 3S − 1 红（h = 12 → 23、24 → 35、12 → 35、33 → 56），对照臂（同一段历史不注入读错）3 组全不红。随机历史（`specs/b13.txt`）720 段里 5 段 I-3.1 红，查过的 3 段是同一形。

**四句**：
1. 分不分辨臂：这一轮没有跑前臂；它分辨的是「记最大」（零轮改法、已实现）与「不记」（7-2 之前）：不记时这一形不红，但 m3-ring-r3 第 4 行那几段候选类命中会回来（那一轮攻方量过）。所以它不是让一边胜出，而是说零轮改法与 I-3.1 的等式合不起来。
2. 被判的系统看不看得到：**看不到**。checker 读的是盘面，读错只是读、不写盘，盘上不留它的任何字节。条款许可实现在「挂载时那一槽报过读错」时多扣、在「没报过」时不许多扣；checker 要判记账对不对，就得知道挂载那一刻那一槽读没读出来，而这件事不在盘上（推的，按读路径不写盘；没逐字节比两臂的盘面）。按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 是「判别子观测不到」：改判据（或改条款），不是改实现去迎合它。
3. 满足判据字面的哪一个分句：I-3.1「已分配统计对得上」的等式那一判；记账一侧照 D16 已定项 1「根环表记最大的」写，遍历一侧照同一项「窗口里自证合法、按实例表判仍然有效的根的最小 checkpoint_txg」按盘现算。两句都照字面，合起来红。
4. 跑前条款给的改法在这几格上还中不中：跑前条款只写「改法进下一批」，没给改法。我提的两个见「几个改法各修哪一格」。

### K2-b　抬 F 那一遍：打中

**条款**：D16 已定项 1 同一行「同取这个窗口的五处」把分配器的回收门槛列为第一处（`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`：「同取这个窗口的五处：分配器的回收门槛、checker 的候选过滤、checker 的 I-3.1（已分配统计对得上） 丁-defer 门槛（max(F_生效, 窗口里最旧有效根)）、影子账的候选过滤、回退候选集」）；「当没有根」那一句的射程是算上限、判回退候选与算 F_生效（同一节：「算准入抬 F 的上限、挂着时管理员回退判候选（根环里有没有目标）与算 F_生效 时读根环：挂载那一刻就读不出或自证不过、这个进程之后也没写过的槽，当没有根」），没有列回收门槛。

**实现**：抬 F 读根环只对「这个进程知道住着根的槽」读坏重读，挂载那一刻就读不出的槽当没有根（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:2712`：「// 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。」）；根环表里那一项不算这个进程知道住着根的槽（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/allocator.rs:1017`：「RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => false,」）；回收门槛只取读得出的根（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:2735`：「!abandoned_by_table(root, &table) && is_inside_the_window(root.checkpoint_txg)」），按它回收（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:2764`：「reclaim_floor(new_floor, oldest_valid_root_in_the_window),」）。进程内记根那一处（K2-e 引的 `allocator.rs:1033`）把读不出的槽按最大 txg 算进门槛，抬 F 这一处没有。

**造法**（`raise_floor_hidden_slot.rs`，S = 4，3S = 12）：mkfs → 可写挂载 → 第一个文件 → 覆盖写到 P → 连着 H 次可写挂载都崩在写行那次根槽写之前（每次留下一个洞：洞那一槽住着上一圈的根）→ 第 H + 1 次可写挂载正常写行（那条根就是 X，紧跟在洞后面）、再发两次空发布、退出不卸载 → 下一次可写挂载起 X 那一槽一直读不出（一时读不出，这条命里都没好）→ 空发布推到锚 A = 第一个洞 + 3S − 1（洞成了窗口最旧的几格、X 紧跟其后）→ 准入抬 F（`raise_rollback_floor_to_the_admission_ceiling`，这一串两次发布盖掉两个洞）→ 再覆盖写几次。

抬 F 时门槛越过 X，释放代落在 (X, 窗口里读得出的最旧根] 的槽被回收；X 在盘上读得出（读错是这个进程一时的）、按实例表有效、txg ≥ 新 F、在窗口里，是候选根，checker 的遍历算它引用的槽，记账却已不算：

```
$ grep -hE '^(HOLES|X |LIFE4|ANCHOR)' rfo3-17-persist.out
HOLES [18, 19, 20]
X txg=21 slot=(0,3) device=0 referenced_slots=28
LIFE4 instance=6 row=26 red=[]
ANCHOR 29 red=[]
$ grep -E '^(AFTER_RAISE|STEP)' rfo3-17-persist.out | sed -E 's/（其中[^"]*//' | cut -c1-330
AFTER_RAISE newest=31 (was 29) x_still_on_disk=Some(21) red=["I-3.1：盘 0：记账的已分配 Some(1146880)，遍历全部有效根得到 1245184"]
STEP 0 txg=32 unit_writes_over_x_refs=[] x_on_disk=Some(21) crash_before_root_red=["I-3.1：盘 0：记账的已分配 Some(1146880)，遍历全部有效根得到 1245184"] after_red=["I-3.1：盘 0：记账的已分配 Some(1343488)，遍历全部有效根得到 1441792
STEP 1 txg=33 unit_writes_over_x_refs=[] x_on_disk=Some(21) crash_before_root_red=["I-3.1：盘 0：记账的已分配 Some(1343488)，遍历全部有效根得到 1441792"] after_red=[]
STEP 2 txg=34 unit_writes_over_x_refs=[] x_on_disk=None crash_before_root_red=[] after_red=[]
```

同一段历史 X 读得出（`rfo3-17-none.out`）四行的 `red` 全是 `[]`。记账比遍历少 98 304 字节 = 6 个槽，一直到 X 被盖掉（txg 33 落盘）为止。

层 0 全量（H = 3、P = 17，抬 F 之后三次空发布；`rfe-sw0` 是被判代码，`rfe-sw1` 打开改法 B1）：

```
$ grep -oE '^ENUM_TALLY step=[0-9] states=[0-9]+ checker_evaluated/violated I-3.1=[0-9/]+ I-7.4=[0-9/]+' rfe-sw0.out rfe-sw1.out
rfe-sw0.out:ENUM_TALLY step=0 states=4108 checker_evaluated/violated I-3.1=4108/4108 I-7.4=4108/0
rfe-sw0.out:ENUM_TALLY step=1 states=4108 checker_evaluated/violated I-3.1=4108/4099 I-7.4=4108/0
rfe-sw0.out:ENUM_TALLY step=2 states=4108 checker_evaluated/violated I-3.1=4108/0 I-7.4=4108/0
rfe-sw1.out:ENUM_TALLY step=0 states=4108 checker_evaluated/violated I-3.1=4108/0 I-7.4=4108/0
rfe-sw1.out:ENUM_TALLY step=1 states=4108 checker_evaluated/violated I-3.1=4108/0 I-7.4=4108/0
rfe-sw1.out:ENUM_TALLY step=2 states=4108 checker_evaluated/violated I-3.1=4108/0 I-7.4=4108/0
```

**放开的维度**（`specs/k2b-*.txt`）：P = 4..7、13..21；洞 2..6 个；抬 F 之后是空发布或覆盖写、3 或 6 次；盘 32 / 128 MiB。X 读不出的 29 组里 19 组红；不红的 10 组：5 组根本没抬成 F（上限不高于现行的 F），4 组环还没转过一圈（洞那一槽没有旧根，非空有效根不足 4 个时上限取最旧有效根，于是新 F 越过了 X、X 不再是候选），1 组抬 F 那一串要发三次、第三次盖掉了 X。X 读得出的同一批历史 20 组 0。

**没造出来的**：被回收的槽真被下一次发布写掉、checker 判 I-7.4 的那一形。上面 29 组里覆盖写之后的单元写一次都没落在 X 引用的槽上（`unit_writes_over_x_refs` 全空）：分配器先从开放聚簇段 bump，X 活着的那几次发布里没轮到回收出来的槽。盘越小越容易轮到，但 20 / 24 MiB 的盘在 mkfs 里就 panic（`rfs-*`，草稿里，没进模型），32 MiB 照样没轮到。所以这一格量到的是「回收集合里有候选根引用的槽」这一层（违反 D16 已定项 1 的可再分配谓词），没量到「真被复用」那一层。

**四句**：
1. 分不分辨臂：分辨。窗口那一条臂（今天的代码）红；7-2 之前的门槛 max(F_生效, 环里最旧有效根) 在这一形上会被洞里住着的上一圈旧根压低、不越过 X（推的：7-2 之前的代码没拷来跑）。这正是 m3-ring-r3 判决第 4 行那一类（「今天的门槛恰好被洞里一条更旧的根压住」），零轮改法只装进了挂载与进程内记根两处，抬 F 这一处漏了。
2. 被判的系统看不看得到：看得到。X 那一项就记在分配器自己的根环表里（`UnreadableAtMountInsideTheWindow`），抬 F 那一遍手里有这张表（同一个函数里用它取 `ring_slots_known_to_hold_a_root_by(allocator)`），只是没拿它算门槛。
3. 满足判据字面的哪一个分句：D16 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」里「窗口里最旧有效根」那一项，加同一行「挂载时读不出的槽……门槛按最小的算」；checker 一侧落在 I-3.1 的等式上（记账少于遍历）。不是「当没有根」那一句：那一句的射程没列回收门槛。
4. 改法在打中的那几格上：跑前条款没给改法；我的改法 B1 在打中的 19 组与层 0 那三次发布上都不中（量过，见改法表）。

### K2-b2　抬 F 的上限不在窗口里：没有读故障的合法历史上准入抬 F 被拒（顺带打中，量过）

上面那批的对照臂（X 读得出、全程没有读故障）里有 5 组抬 F 根本没抬成，拒因相同：

```
$ grep -l 'RollbackFloorCeilingNeedsUnreadableValidRootTreeTable' rf*-none.out | tr '\n' ' '
rf-14-none.out rfo3-21-none.out rf-15-none.out rfo-21-none.out rf-16-none.out 
$ grep -E '^(HOLES|LIFE3|X |LIFE4|ANCHOR|RAISE)' rf-14-none.out
HOLES [15, 16]
LIFE3 instance=4 x_txg=17 newest=21
X txg=17 slot=(2,1) device=0 referenced_slots=28
LIFE4 instance=5 row=23 red=[]
ANCHOR 26 red=[]
RAISE_ERR RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) }, failure: UnitUnreadable { slot: SlotNumber(1162) } }
```

机理：洞那一槽 slot(15) 里住着上一圈的根 txg 3（15 − 12），按实例表有效、txg ≥ F（F 还是 0），但在窗口外（锚 26，窗口 txg > 14）。窗口许可回收它独占的单元，它的树表那一槽（1162）已被合法复用；算上限要给每一条有效根判「非空」、读它的树表，读不出就拒（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:2061`：「pub fn rollback_floor_ceiling<Device: BlockDevice>(」）。这个拒因不是空间不够（`floor_raise_refused_for_space` 判它 false），会话里的准入交回 `UserChangeRefused::FloorRaiseFailedWhilePushingForSpace`，挂载续做的删除走空交回 `MountError::FloorRaiseFailedAfterTheMountsPublishes`（按代码推的；我直接调的抬 F 入口，没造空间压力去走这两条路）。随机历史里也撞到：不带读故障的 `q11`、`q12`、`r10`–`r12` 五件里有 8 段记下这个拒因（`refusals` 里的 `F:RollbackFloorCeilingNeedsUnreadableValid`）。

条款一侧：D16 已定项 1 列的「同取这个窗口的五处」没有「抬 F 的上限」，「抬 F 的上限」那一行照旧在全部有效根上取「第 4 新的非空持久有效根」；实现照字面。窗口让窗口外有效根的单元可以被复用，而上限还要读它们——两句合起来在环上有洞（崩在记录已持久根槽没持久、或换实例跳号）、F 又低于洞里旧根的盘上拒抬 F，直到洞那一槽被盖掉或卸载抬 F（卸载那一串不判上限）。

四句：① 分辨：窗口臂才有（7-2 之前那条旧根就是「环里最旧有效根」、压着门槛、树表不会被复用；推的，7-2 之前的代码没拷来跑）。② 看得到：看得到，上限那一遍手里有锚与 3S。③ 分句：D16 已定项 1「抬 F 的上限」那一行的「第 4 新的非空持久有效根」加「非空」那一段的读法，碰上「一圈窗口里最旧有效根」那一行许可的复用；是条款少列了一处，不是实现偏离字面。④ 改法：跑前条款没给。我想到的 B2（上限与「非空」只在窗口里的有效根上取）没实现、没跑，推的。

### K2-a　挂载时的窗口：没打中

挂载时锚取环里读得出的最新有效根（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:1775`：「.filter(|root| !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg))」那一段之前取 `window_anchor`），读不出的槽按最小 txg 进门槛，两样都只让挂载那一刻的门槛更低（扣得更多）。写行那次发布在装记账行之前按进程内的锚重算门槛（K2-e 引的 `allocator.rs:1033` 那一处），记账行按重算之后的数写，所以挂载那一刻多扣的不会单独落进任何一版记账；崩在写行那次根落盘之前，盘上最新的记账还是上一版的。会红只能经由进程内那张表，那就是 K2-e。随机历史 1020 段（`results/out/q*.out`、`r*.out`；790 段带读故障，其中 535 段读故障整条命都在）没有一段的红归到挂载那一刻的门槛上。

### K2-c　丁-defer 门槛：两处打中都落在它上面

K2-e 是 checker 这一侧按盘取的门槛比实现高（实现多扣），K2-b 是实现这一侧抬 F 时门槛比盘上该有的高（实现少扣）。除此之外，两边门槛不同的来源我按代码只找到「挂载那一刻读不出的槽」这一种：两边都按「按最新根指着的实例表不被抛弃 ∧ 在窗口里」取最小 txg，F 那一半都取 max，被抛弃根都不进最小值。推的。

### K2-d　影子账的候选过滤：造不出

候选 = 有效 ∧ txg ≥ F ∧ 在窗口里（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:1370`：「&& is_inside_the_window(root.checkpoint_txg)」）。过滤只会把候选集变小，候选集变小只让「只被被抛弃根引用的槽」变多、隔离变多；隔离位只住内存、不进记账行，池级 checker 也不判隔离，所以既造不出 checker 红，也造不出「被抛弃根引用的槽被复用」（那要候选集变大）。挂载那一刻读不出的被抛弃根影子账罩不到，是 C554 那一件，与窗口无关。推的（按代码），没造盘面。

### K2-f／g／h　循环、两份窗口的边界、四处的锚：没打中

- 循环：`window_loop_equiv.rs` 经测试入口原样调 `unreadable_ring_slots_inside_the_window`，与逐 txg 走 (N − 3S, first_txg − 1] 的原写法逐输入比。取样：S = 4..16 每一档，N ∈ [0, 60] × first_txg ∈ [0, 90] 全域（每对一次随机 1–3 个读不出的槽、一次全部槽读不出），另加每档 2000 组 N 贴着 u64::MAX、first_txg = min(N + 1..100, u64::MAX) 或恰为 u64::MAX。输出原样：「WINDOW_LOOP_EQUIV cases=170326 mismatches=0」（`results/out/wle.out`）。周期：`target_for_publish` 把 txg 映成 (txg mod 3, (txg div 3) mod S)，在任意 3S 个连续 txg 上是双射，周期恰为 3S（推的，算术）。
- 边界：核心层 `/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/root_ring.rs:130`：「.saturating_add(ROOT_RING_REGIONS * slots_per_region.count())」，checker `/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/walk.rs:5934`：「checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.newest_valid_root_txg」，同一个式子；checker 的 3S 取系统配置里的区域数乘每区槽数，第一版区域数恒 3。txg = N − 3S、N < 3S、txg 贴着 u64::MAX 三格两边逐格相同（推的，按式子）。
- 锚：抬 F 那一遍取现行那一版（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:2728`：「let window_anchor = current.root.checkpoint_txg;」），与 D16 已定项 1「进程内每记一条根时取这条根」相同；挂载、进程内、checker 三处见 K2-a、K2-e 引的行。字面：`.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读`：「N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1；进程内每记一条根时取这条根）」。

## K3　系统配置 R2

**K3-a　谓词**：实现（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/recovery.rs:3308`：「.any(|slots| slots.slots_reporting_a_read_error > 0)」，读错只认读者交空 `/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/recovery.rs:3279`：「None => slots_reporting_a_read_error += 1,」）与条款（`.claude/kb/decisions/22-单元原子性怎么合成.md@已定项 8：固定结构的放置`：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过」）逐字对得上。读得出而自证不过（撕裂、全零）、fsid 不符都只是不进 `self_verified`，落在「判得出」那一边。

**K3-b　三处同一处定义**：读阶段的见证读与取号都调 `system_configuration_witness_is_undecidable`；逐盘核那一遍只判留着的那道拒（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/mount.rs:3487`：「let Some(newest_self_verified) = newest_self_verified else {」），R2 由随后的取号那一读再判一次，拒在第一个写之前。core 里没有留着取甲的判法。**被判文件里留着取甲的是 E165 装置**：读故障族的生成器按「自证过的槽不足全池槽数就判不出」建可写挂载那道闸（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12997`：「if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {」），产物配置行写着它模型的是实现的那一判（同一文件 `/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:15725`：「witness_gate=the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable」）。7-2 动过这份文件（只为新根段三字段编得过，diff 里自己写了 J7 要重新登记），这一判没跟着改。生成器「不调 `mount.rs`」，所以它不会红，只是从此模型的不是实现：至少在「一块盘一槽一时读不出、另一块盘两槽都自证过」那一格（它的挂载开头故障菜单里有单槽读不出），它判「拒可写」而实现判「可写」。推的（按代码；E165 是 checker 档装置，没跑；它的故障菜单里还有哪几格落在两判之间没逐格对）。

**K3-c　R2 加留着的那道拒会不会让池永远只读**：

```
$ for f in k3-d k3-b k3-c k3-f; do echo "-- $f"; grep -E '^(TORN|DEVICE|R2_|ATTEMPT 0|READ_ONLY)' $f.out | cut -c1-170; done
-- k3-d
TORN device=0 slot_offset=0 (写之前这一槽是另一份，之后只落了头 512 字节)
DEVICE 0 self_verified=2 read_errors=0
DEVICE 1 self_verified=2 read_errors=0
R2_UNDECIDABLE false
ATTEMPT 0 writable=Ok instance=3
READ_ONLY Ok
-- k3-b
TORN device=0 slot_offset=0 (写之前这一槽是另一份，之后只落了头 512 字节)
DEVICE 0 self_verified=2 read_errors=0
DEVICE 1 self_verified=1 read_errors=1
R2_UNDECIDABLE false
ATTEMPT 0 writable=Ok instance=3
READ_ONLY Ok
-- k3-c
DEVICE 0 self_verified=1 read_errors=1
DEVICE 1 self_verified=2 read_errors=0
R2_UNDECIDABLE false
ATTEMPT 0 writable=Ok instance=2
READ_ONLY Ok
-- k3-f
TORN device=0 slot_offset=0 (写之前这一槽是另一份，之后只落了头 512 字节)
DEVICE 0 self_verified=2 read_errors=0
DEVICE 1 self_verified=0 read_errors=2
R2_UNDECIDABLE false
ATTEMPT 0 writable=Err NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { first_read: S reread_Some("comparison: S
READ_ONLY Ok
```

`sysconf_r2_forever <P> <撕哪块盘> <读不出的槽> 3`：k3-d 撕盘 0 的轮换写、无读错；k3-b 撕盘 0 + 盘 1 槽 0 一直读不出；k3-c 只有盘 0 槽 0 一直读不出；k3-f 撕盘 0 + 盘 1 两槽一直读不出；每格连做 3 次可写挂载（k3-f 三次都拒、盘上不变，另两次与第一次同一行；被 `cut` 截掉的那一段原样是 `reread_Some("comparison: SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot } } })") disk_unchanged=true`）。

- 撕不坏：层 0 的撕裂粒度是 512 字节的物理块，而系统配置的内容只有 489 字节（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-format/src/lib.rs:248`：「pub const SYSTEM_CONFIGURATION_BYTES: u64 = 489;」），槽里其余字节新旧都是 0；只落头 512 字节的那一写与整写逐字节相同（k3-d / k3-b 里被撕的盘仍两槽自证过）。所以「读得出、自证不过」的系统配置槽在合法崩溃历史里造不出，只能来自介质损坏；被删掉的旧用例 `a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs` 用清零造的「撕裂那一形」也属于这一类（推的）。
- 要让留着的那道拒永远拒，得一块盘两槽都坏（k3-f：盘 1 两槽读不出，R2 判得出，逐盘核按「这块盘一份自证过的都没有」拒，三次都拒、盘上不变，只读照常）。这是两处故障落在同一块盘上，D22 已定项 8「容错承诺是单故障：每盘一槽系统配置坏在承诺里」罩不到，所以不算条款不要的永远只读。一处我记下、没判：两槽的偏移是 0 与槽距（第一版 4096），挨着放，一段 8 KiB 的介质坏块就同时坏两槽（推的）。

**K3-d　改钉的两条判拒用例**：取号那条（`acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs`）原来判的是「一槽读不出就判不出」（取甲），换盘面之后判的是留着的那道拒「盘 1 一份自证过的都没有」（盘 0 两槽都自证过，R2 判得出）；它不再判 R2 的「判不出」那一支，那一支由新文件 `system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs` 判。可写挂载那条整份删掉、由同一份新文件接；新文件的「撕裂轮换」用的是清零（见上一条：合法崩溃历史里造不出）。推的（读 diff）。

K3 四句（只对 K3-b）：① 不分辨臂（没有臂）；② 不适用；③ 落在「有没有一处还留着取甲的写法」，但那一处是装置不是 core 的三处，可能归错格，交主 agent 定；④ 跑前没给改法。

## K1　新根段三字段与恢复施加

- 施加：由记录重建的根，七项与实例代号、txg 取施加的末条记录，fsid 照所选根（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/recovery.rs:3582`：「unmount_marker: record.new_unmount_marker,」那一段）。三处装记录（零单元发布、写行那次、带文件的一版）都写「这次发布写的根上的」那七项（`transaction.rs` 第 1397–1399、1928–1930、8639–8641 行，同一句注释）。与 D23 已定项 15 和 m3-ring-r3 判决第二节第 1、2 行字面相同。推的（按代码）。
- **卸载记号「位 0 之外有位为 1 当记录损坏」三处不是一个口径**：读者在解析时拒（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-core/src/journal.rs:346`：「UnmountMarker::of_unmount_marker_field_of_a_new_root_segment(reader.get_u32())?;」），恢复用同一个解析器所以也拒；池级 checker 的 `check_journal_record`（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/lib.rs:770`：「pub fn check_journal_record(」）不判这 4 字节——那个函数的正文里 `unmount`、`卸载`、`459` 一次都没出现（`sed -n 770,835p … | grep -c -i 'unmount\|卸载\|459'` 输出 `0`），而 checker 读根记录 flags 时是拒的（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/lib.rs:336`：「if flags & !ROOT_FLAG_UNMOUNT_MARKER != 0 {」）。量过（`marker_bits_probe.rs`：txg 9 那次覆盖写崩在根槽之前，把这次发布两块盘上的记录偏移 459..463 改成 2、重封头部校验和）：

```
ARM clean crashed_txg=9 core_parse_accepts=[true, true] checker_parse_accepts=[true, true] recovery_effective_root=Some((2, 9)) checker_red=[]
ARM clean mount=Ok effective_root=(2, 9) row_txg=10 after_checker_red=[]
ARM bit1 crashed_txg=9 core_parse_accepts=[false, false] checker_parse_accepts=[true, true] recovery_effective_root=Some((2, 8)) checker_red=[]
ARM bit1 mount=Ok effective_root=(2, 8) row_txg=9 after_checker_red=[]
```

  core 不收那两条记录，恢复落到 (2, 8)，下一次可写挂载的写行就取 txg 9（与被拒的那条同号）；checker 两份都收，盘面上两臂 checker 都不红。改成 `0x3`、`0x80000000` 结局相同（`results/out/mb-3.out`、`mb-80000000.out`）。只在头部校验和对得上的外来或写坏的字节上走得到（写者只写 0 或 1），合法崩溃历史走不到。四句：① 不分辨臂；② checker 看得到那 4 字节，只是没判；③ 落在 K1 那一句「读者、恢复、池级 checker 三处是不是同一个口径」上；④ 跑前没给改法；改法 K1a（checker 的 `check_journal_record` 照根记录 flags 那一判，位 0 之外有位为 1 就交 `Verdict::NonZeroFlags` 这一类）没实现，推的。
- 头宽、偏移、点名项容量与 `format-const` 标记：没逐个核（分给正推腿），这一条腿没判。

## K4　池级 checker 跟上

- 写行之后第一个文件版本、崩在记录已持久根槽没持久（`first_file_record_only.rs`：mkfs → 可写挂载（实例 1）→ 可写挂载（实例 2，写行）→ 第一个文件 txg 5 崩在根槽写之前 → 可写挂载（实例 3）施加它、写行 txg 6），实例 3 那条挂载流层 0 全量：

```
CRASHED first_file_txg=5 instance=2 red=[]
MOUNT3 effective_root=(2, 5) row_txg=6 red=[]
ENUM mount3 segments=9 closed_form_state_count=20496
ENUM_INV I-3.1 evaluated=4116 violated=0 not_applicable=16395
ENUM_INV I-3.10 evaluated=20511 violated=0 not_applicable=0
ENUM_INV I-4.1 evaluated=16398 violated=0 not_applicable=4113
ENUM_INV I-3.11 evaluated=4116 violated=0 not_applicable=16395
ENUM_INV I-7.4 evaluated=20511 violated=0 not_applicable=0
ENUM_INV I-2.1 evaluated=20511 violated=0 not_applicable=0
ENUM_INV I-8.10 evaluated=20511 violated=0 not_applicable=0
ENUM_ALL_VIOLATED []
ENUM_TALLY states=20511 oracle_violations=12
```

  checker 在 20 511 个状态上一处违例都没有。那 12 处 oracle 违例是我给的版本表漏了实例 3 挂载里的第二次发布（txg 7）那一版，oracle 报「第 7 代根下面没有文件却读出了内容」，与 checker 无关（`ENUM_TALLY` 那一行的 `first_oracle`）。之后再覆盖写 14 次、每次之后盘面跑 checker，全绿（`results/out/ff-s4.out`）。
- (实例代号, 计数器) 键：checker 逐盘按计数器收、跨盘按 (实例, 计数器) 取先读到的一份（`/tmp/claude-1000/m3-root-ring-code-r1/tree/crates/singlefs-checker/src/walk.rs:5412`：「.entry((record.instance, *counter))」），core 扫环按同一个键收、任一份自证过即算；两块盘同计数器坐着不同实例时两边都把两条当两个键收下。要两边收的不一样，得一条记录 core 不收而 checker 收——K1 那一格就是这一种（同一个键下 core 只取另一块盘那一份或一份都不取）。推的（按代码）。
- 独立性：`singlefs-checker` 只依赖 `singlefs-format`（`crates/singlefs-checker/Cargo.toml` 的 `[dependencies]` 只有这一行），窗口（`OneRevolutionWindow`）与键都是 checker 自己写的，没有与 core 共用的代码。

## 几个改法各修哪一格

全是我提的，**只在我的模型上量过、被攻过零轮**（B1）或连量都没量（其余，推的）。

| 改法 | 内容 | K2-e（记最大 ⇒ checker 判多） | K2-b（抬 F 越过读不出的根） | K2-b2（上限读窗口外旧根） | K1（卸载记号位） |
|---|---|---|---|---|---|
| B1 | 抬 F 那一遍取门槛时，把分配器根环表里「挂载那一刻读不出」的那几项按它记的最大 txg 并进窗口里最旧有效根（与进程内记根那一处同一个取法） | 不起作用（量过：`r*.out` 与开了 B1 的 `s*.out` 720 段逐行相同，其中 3 段是这一形） | 修（量过：9 组 H = 2 / 3 / 6 × P = 17 / 18 / 20 原来全红、开了之后 63 行 `AFTER_RAISE` / `STEP` 里 I-3.1 出现 0 次；层 0 三次发布 4108 / 4099 / 0 → 0 / 0 / 0） | 不起作用（推的：不碰上限） | — |
| E1 | 改判据：checker 的 I-3.1 丁-defer 门槛从一个值改成一个区间，下沿 = max(F_生效, 窗口里每个「那一槽没住着它该住的那个 txg 的根」的位置 txg 的最小值, …)，记账落在区间两端的遍历之间就算对得上 | 修（推的） | 不起作用（推的：那一格是记账少于上沿，区间不放宽上沿） | — | — |
| E2 | 改实现：进程内记根时，根环表里那一项若成了窗口里最小的那个，先重读那一槽，读得出就换成读出来的（窗口里的有效根按它的 txg，别的当没有），仍读不出才按最大 txg 算 | 只修挂载那一次瞬时读错的几组（推的）；整条命都读不出（`u31-v2`、`u31-s4-9-persist`）照中，且那时真盘上的 checker 也读不出那一槽 | 不起作用（推的） | — | — |
| B2 | 抬 F 的上限与「非空」只在一圈窗口里的有效根上取 | — | — | 修（推的；条款要改「抬 F 的上限」那一行，算「同取这个窗口」的第六处） | — |
| K1a | checker 的 `check_journal_record` 照根记录 flags 那一判，卸载记号位 0 之外有位为 1 就判这条记录坏 | — | — | — | 修（推的） |

## 旁及（不在攻击面或不算这一轮的打中）

- 读不出的槽在算上限时当没有根（条款「根槽这一次读坏」那一行），checker 按盘算上限：`rf-5-persist`、`rf-6-persist` 两组 I-7.9 红（「实例 5 txg 18 那条根把回退下界 F 从 0 抬到 9，高于它之前的根算出的抬 F 上限 8」，`results/out/rf-5-persist.out`），对照臂不红。C563 那一类、早于 7-2，checker 看不到那一次读错（判别子观测不到）。
- 不带读故障的随机历史 230 段里 24 段 I-7.9 红（`q11`、`q12`、`r10`–`r12`），就是 m3-ring-r3 判决第 12 行记进下一批的那一件，今天的代码上还在，没查因。
- 候选类命中 13 段，第一处命中全是「`cand_txgN/newestN`、那一命的第 1 段」：被藏的最新根是候选、下一次挂载写行的单元盖了它的槽，C554 那一形，与窗口无关。

## 没打中的形状

- K2-a 挂载时的窗口、K2-d 影子账过滤、K2-f 循环、K2-g 边界、K2-h 锚、K3-a 谓词、K3-c 永远只读、K4 第一个文件那一形与键：各节写了试过什么、取样多大。
- 随机历史（`window_fuzz`，判法沿用 m3-ring-r3 攻方的全域静态判法加每一命收尾跑池级 checker）：`b04` 300 段、`b13` 720 段（`b15` 是 `b13` 开 B1 的同一批）；S = 4 / 8；每一命 0..3S + 6 个动作，动作分布两种（覆盖写 / 空发布 / 准入抬 F / 回退 = 60 / 12 / 12 / 16，或 20–25 / 45 / 25–30 / 5–10）；挂载里就崩 25–50%；读故障 0 / 60 / 80% 的挂载藏一个根环槽，藏法分「只在挂载那一次」与「整条命」；1 MiB journal（256 条记录，环绕回）、256 MiB 盘。除上面归了类的，没有别的红。
- 由用户决定的动作放开扫过：K2-e、K2-b 的打中都在放开动作的随机历史与几种不同后缀上复现（K2-e 随机历史里自己撞到 3 段；K2-b 抬 F 之后的后缀换过空发布 / 覆盖写、3 / 6 次）。B1「不中」是在 9 组脚本历史、层 0 三次发布与 720 段随机历史（后者原本就没撞到 K2-b）上看的。

## 跑批、取样与估时

- 盘用 `singlefs_harness::memory_pool::SparseBlockDevice`（内存稀疏盘）；前缀每段历史从内存里那一份 `MemoryPool` 拷。原型调层 0 枚举只经 `singlefs_checker_tier::crash::enumerate_layer0_versions`（每段都展开、每个状态恢复之后跑池级 checker），流都是原型自己造的，测试目标名里没有 `layer0`。
- 估时：先跑一小段。768 MiB journal、4 GiB 盘时随机历史一段约 2 s（种子 2030，`time` 实测 2.0 s），层 0 一态约 60 ms/线程（第一次枚举的进度行：4 线程、64.5 s 做完 4100 态）。按后者，一次覆盖写的单元写那一段有 28 个写，闭式状态数 268 435 463，超过约 10⁶，那段不跑；空发布那一段 16 个写，闭式 65 543 态，4 线程估 17 分钟。于是改成 1 MiB journal、128 MiB 盘、挂载之后换成空发布（每次发布 4103 闭式 / 4108 实计态），复跑里 2 件 × 7 线程做完 32 864 态用了 1 分 52 秒。随机历史改用 1 MiB journal、256 MiB 盘：12 件 × 60 段一批约 35 s。缩的只是几何、发布种类与历史条数，没缩任何一段的崩溃状态。
- 状态总量（层 0 枚举，含复跑）：开发时 82 131 态 + 第一次那件没跑完就停掉的约 12 300 态 + 复跑 78 023 态，合计约 1.7 × 10⁵ 态，单件最多 32 864 态（每件都先用 `closed_form_state_count` 算过、超过 10⁶ 就 `assert` 拒）。
- 偏离：第一次层 0 枚举（`u31e-s4-9-mount-E`，768 MiB journal）跑过前台 240 s 上限，被工具挪到后台；我按 `proc.py stop` 停了那个二进制（它没有子进程），没让它跑完，改成上面的小几何。其余命令都在前台跑完。线程：每批件数乘线程数合计不超过 15。
- 复跑那一遍（`rerun/`）的挂钟：编译 29.5 s；`k2e-enum` 1 分 52 秒；`k2b-1`…`k2b-fix-1` 五批合计 5.9 s；`k2b-fix-2` 加 `k4-enum` 2 分 7 秒；`b04`、`b13`、`b15` 合计 1 分 24 秒。

## 什么现象会推翻这些结论

- K2-e：同一段历史不注入读错也红，或注入了读错而 txg h + 3S − 1 那一版不红；或者有人指出条款「根环表记最大的」本来就不进进程内的门槛（那样实现偏离条款，打中的归属变成实现）。
- K2-b：把 B1 打开之后同几何仍红；或证明 X 那一槽这一条命里一直读不出的历史不合法（读故障不许跨过挂载之后的第一次抬 F）。
- K2-b2：7-2 之前的代码上同一段历史也拒（那就不是窗口带来的）；或「抬 F 的上限」那一行本来就该只在窗口里取、实现漏了（那样归实现）。
- K3-c：造出一段合法崩溃历史让一个系统配置槽读得出而自证不过（我的推论依赖内容 489 字节 < 512 字节物理块）。
- K1：checker 别处（不是 `check_journal_record`）判了新根段卸载记号的保留位、并在 `marker_bits_probe` 那张盘上判红。

## 这条腿自己的限度

- 都是一次抽样；「没打中」那几格照规则要再抽一次才算。
- K2-b 没造出「被回收的槽真被复用、checker 判 I-7.4」的那一形，量到的只是回收集合里有候选根引用的槽。
- K2-b2 只直接调了抬 F 入口，没造空间压力走会话准入与挂载续做的删除走空那两条路，那两条路上的结局是按代码推的。
- K2-e 的读错与 K2-b 的「整条命读不出」都是 `HidingDevice` 在读者一侧注入的；真盘上一直读不出时，checker 自己也读不出那一槽，那时它看到的盘面与我量的不同。
- 7-2 之前的代码没拷来跑：「分辨臂」两处都是推的。
- K1 的头宽、偏移、点名项容量、`format-const` 标记，K5 整格，都没做（分工里归正推腿或不在我的攻击面）。
- E165 那一处只读了代码，没跑装置（checker 档）。

## 要主 agent 跑的命令

没有非跑不可的。要坐实就照「复跑」那一条在新草稿目录里跑（harness 档：只编原型、经内存包装跑原型二进制），比 `results/SHA256SUMS`；层 0 枚举那 5 份要先去掉 `LAYER0_PROGRESS` 行再比。

## 没做什么

- 没跑任何 checker 档测试目标、门禁、变异、E165 / E152 装置；没跑 `cargo test`。
- 没判正推那几格（K5 改钉、K1 的格式常量），没替主 agent 采纳改法。
- 副本上的数不是入库装置上的数。
- B2、E1、E2、K1a 都没实现。

## 草稿与清理

草稿 `/tmp/claude-1000/m3-root-ring-code-r1-opus/`：交回前删掉了两份仓副本与两份编译目录（`tree/`、`target/`、`rerun/tree/`、`rerun/target/`），留下的是日志（`out/`、`rerun/out/`、`specs/`、几份脚本）。模型目录里的东西可以从冻结副本整份重建。

## 引文核对

`python3 research/scripts/cite-check.py research/prompts/m3-root-ring-code-r1-opus-output.md` 末行：

```
  ✓ 核了 30 处引文，对上 30 处，没判 0 处（逐处列在上面）
```

代码引文都写冻结副本的绝对路径（不随主工作区变），kb 引文写 `文件@小节标题`。
