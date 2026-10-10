# 根环实现第 10z 批第十、十一块：代码轮第一轮判决（2026-10-08）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 G1 G2 G4 E1 E2 E3 E7 X1 X2 Y1 Y4 H2 R C P Q -->

正文 `research/prompts/_m3-batch10z-jk-code-r1-body.md`。三份腿报告：
- 正推：`research/prompts/m3-batch10z-jk-code-r1-sonnet-output.md`，sha256 7bc03ed4…1025，259 行；
- 攻方：`research/prompts/m3-batch10z-jk-code-r1-opus-output.md`，sha256 779a2e32…7b36，126 行；
- 攻方第二次抽样：`research/prompts/m3-batch10z-jk-code-r1-opus-s2-output.md`，sha256 2369cd44…3e36，161 行。攻方第一次各格多半读码判「没打中」，照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」再抽一次，提示要它造反例。

## 一、核查

| 项 | 结论 |
|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1/tree/` 对 `research/prompts/m3-batch10z-jk-code-r1-snapshot/crates-sha256.txt` `sha256sum -c` 全过；kb 对 `kb-sha256.txt` 全过 |
| 引文 | `cite-check.py` 三份各跑，`--root` 冻结副本与仓根：正推 `✓ 核了 47 处引文，对上 47 处，没判 0 处`，攻方 4 / 4，s2 6 / 6 |
| 主 agent 现读 | 冻结副本 `crates/singlefs-core/src/mount.rs` 第 2155–2171 行：谱系片 `read_unit_via_locations(…).ok()`、`parse_packed_unit(&bytes).ok()` 两处把读不出、解不开吞成「这一片没有」，与「读得出、没有这条根的谱系记录」同一个 `None`；`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` 第 1336–1358 行 `take_in_the_devices_found_idle` 对丢了的设备 `continue`，第 1365–1380 行 `started_card` 的重起只在借到之后才走到 |
| 复跑一 | 草稿目录 `~/.local/share/singlefs/work/m3-batch10z-jk-code-r1-main/`，冻结副本拷贝 `copy/` 加第十块实现员的量法臂与探针（`inputs-sha256.txt` 记了三份输入），`rerun.sh` 28 件 rc 全 0，`summary.txt` 8 组与实现员 `runs/cost-summary.txt` 的 today、implemented 两臂逐字相同：`label=rollback-cost slots=384 arm=today … histories=60 … hit=39`、`slots=1536 arm=today … hit=29`、`slots=384 arm=implemented … hit=0 … iso_mount(mean/max)=42.0/42 iso_after_first_raise=70.0/70` |
| 复跑二 | 同一草稿目录 `copy2/` 加正推腿的环境变量臂与探针 `probe_jk.rs`，`rerun2.sh` 9 件 rc 全 0，`summary2.txt` 与正推 `logs/summary-residual-shapes.txt` 的 implemented、tri 两臂逐字相同：`res-racct+rlin arm=implemented n=4 … hit=4 preroot=4 refused=0 iso_mount=18.0/18`、`res-racct+rlin arm=tri n=4 … hit=0 … refused=0 iso_mount=116.0/116 iso_first_raise=144.0/144`，`rold`、`roldacct` 两形同数；入库回退用例 `units_a_rollback_publish_revived_…` 在 `SINGLEFS_PROBE_ARM=control`（E3）与 `anylin`（E2）下各 `test result: ok. 1 passed` |
| 没复跑 | 攻方 s2 的 s2fb 臂与 1536 槽、回退目标深的数（`research/prompts/m3-batch10z-jk-code-r1-opus-s2-model/runs-summary/k23fb-fallback.txt`）、并发压测，正推的账层探针 `probe_kept_leases_on_a_lost_device`、G1 G2 G4 与 Y1–Y4、下游叫停 `cargo check` 只报警告那一条：都是腿副本上的数，判决只当线索，不进 kb |
| 腿的数 | 都是腿的模型或副本上的数，不进 kb |

被判的源文件：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`、`crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs`。

## 二、逐格判决

| 格 | 三腿 | 判 |
|---|---|---|
| K1-1 丙并中间罩不罩得住 | 三腿都没造出 R 走丙而 C 里哪一槽都没有的单元：正推 72 形扫做成的 33 形里红的 7 形全带「谱系片读不出」（归 K2-3），不带的 26 形 0 红；s2 四种回退目标、两档几何、五故障组各约 160 段 0 红。Q 的记录全读不出那一形已归 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）。多条根同走丙、(P, R) 相差超过一圈两形三腿都没造出 | 站住（两次攻方抽样都没打中） |
| K1-2 放的时机 | 正推 implemented 405 段 refused 0、放早 0 红；s2 iso_end=0。正推旁证：故障 `cfgq,qroot,qlast,racct` 之后段末隔离停在 18 槽不回 0（`long-nn60-none.log`），撤掉两处补法的 none 臂同数 | 站住；旁证不是这一批带进来的，但它是「一直不放」，进下一批先查 |
| K2-1 认回退 | 认错三腿都没造出（正推 anylin 对 implemented 逐格相同）；认漏：读不出与「没有这条谱系记录」同一个结局（主 agent 现读第 2163、2165 行坐实） | 认漏并进 K2-3 |
| K2-2 回退复活罩不罩得住 | 正推 128 形 256 段 0 红、复活集全在 R 那一槽，含 R_old 自己是回退；s2 嵌套回退 18 段 0 红 | 站住 |
| K2-3 认不出回退（交用户的残余） | 三腿都造出：谱系片读不出、R_old 的根槽读不出、R_old 的账读不出，s2 另加「R 的全部单元读回全零」、R_old 已不在环里；红法同一种（复活的数据单元没进 R 那一槽，回收时发出去，I-7.4 红）。主 agent 复跑二坐实三形各 4/4 红。两条不改格式的退路：正推 tri（R 记录点名的打包单元有读不出的，或认出回退而 R_old 或它的账读不出 ⇒ 并现行账里全部已释放的落点），复跑二三形 0 红、384 槽盘挂载 116 槽；s2 s2fb（同样的退法，只并释放代晚于 F 的那一截，认回退的信号另取事务号 0），s2 副本上 0 红、隔离 90–164 槽，R_old 已不在环里时 610–634 槽（1536 槽盘） | **交用户**（第三节） |
| K2-4 多护的那一截 | 三腿：随 R 那一槽写掉放，0 段被拒；D28（挂载期承诺量） 已定项 1 第 27 行第九项字面只写到「比所选那一版新、按实例表有效、读得出的根的账里的落点」，没说到并进来的 R_old（比所选那一版旧）账里的落点 | 站住；第九项字面进 kb 写回 |
| K3 列卡带借据号 | 三腿都没造出多给、多销；「借据先于缓冲丢」仍没有会红的检查；正推 G1（去掉 `gpu_memory_ledger.rs` 第 911 行设备号条件）照绿，生产里等价（推的） | 站住；前提照旧记欠账 |
| K4-1、K4-3 两把锁 | 锁序只有 starting → started；s2 10 线程 × 5000 轮无死锁、无双起；正推去掉 starting 的变异 48/48 红；G2（旧设备改回在 started 锁里丢）照绿 | 站住；G2 进 K5 补用例 |
| K4-2 重起时不等、不 poll | 账多给三腿都没造出。正推：丢了的设备上没等成的借据一直记在账上，账要到重起时才知道设备丢了，而重起只在借到之后才走到；记着的量让后面的借都放不下时，这张卡在这个进程里再也借不出、报本次未跑，卡上其实空着（账层探针 `room_now_after_lost=1000`、借 4000 被拒、`prepare` 之后 29550）。主 agent 现读第 1336–1380 行坐实「重起要先借到」那一环；真卡没试 | **打中（窄）**：不是第十一块带进来的（拆锁前同样交丢了的、同样跳过），违反用户「不压并发」与正文 K6-2「卡放得下却报本次未跑」；进下一批 |
| K5 判别力 | 第十块 10 行变异两腿各自全跑全红。正推另造四处照绿：E1 回退目标取所有有效根、E2 认回退不比 rollback_txg、E3 改回对照臂（并现行账里全部已释放的）、E7 多条走丙的根只并最后一条（主 agent 复跑二坐实 E2、E3）；第十一块账层 10 行全红，G1 G2 G4 照绿；`take_in_the_devices_found_idle` 函数体里面（Y1–Y4）与下游各层的叫停没有用例，下游换成永不叫停 `cargo check` 只报 unused variable 警告，变异跑道不带 `-D warnings`；checker 档 X1 X2 X4 X4b 4 行只在逐字搬运的小程序里红 | **打中**，补用例与变异；E3 最要紧：窄取法的「窄」没有测试钉着 |
| K6 | 多给没造出；K6-2 一形同 K4-2；只销列出来的带来一次列卡到记账之间的瞬时少给，没量到被拒 | 同 K4-2 |

## 三、交用户

H2 丙补洞之后剩的那一形：R 是回退发布、走丙（R 的账读不出而记录齐），而且认不出它回退到哪一版（谱系片读不出，或根环里没有 R_old、R_old 的账读不出）。这时补法交空，复活的单元照旧会被发出去，I-7.4 红。四个候选：
1. 留原样，记欠账（与 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 同族）：红照旧；
2. 认不出时退到宽护（正推 tri）：并上现行账里全部已释放的落点；
3. 认不出时退到宽护的窄一点的那一形（s2 s2fb）：只并释放代晚于 F 的那一截；
4. 认不出时拒可写挂载：要新的错误成员，要先告形式化线（E171 装置引 `MountError` 的成员）。

弹窗原话与用户定案记在第五节。

## 四、下一批（第 10z 批第十二块）

两簇，文件不交：
- core：用户定的残余处置；K1-2 旁证那一形先查原因（`cfgq,qroot,qlast,racct` 之后隔离停在 18 槽）；K5 补 E1、E2、E3、E7 会红的用例与变异；D28 第九项写回。
- GPU：K4-2 丢了的设备上没等成的借据让卡借不出——账要在借之前、列卡那一下就知道设备丢了；K5 补 G1 G2 G4、`take_in_the_devices_found_idle` 函数体、下游各层叫停的用例。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-10-08 改了：丙补两洞之后剩认不出回退那一形，交用户；第 37 行等用户定了与下一批补法定形之后一起写回 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 不受影响：第 390 行与 I-7.4 照旧 |
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-08 改了：第九项字面没说到并进来的 R_old 账里的落点，下一份 kb 写回 |
| D18（块里携带什么信息） 已定项 24 | 备料 | 2026-10-08 不受影响：回退那次发布追加谱系记录照旧，认回退读的就是它 |
| D24（后台重活能不能卸给 GPU） 已定项 2 | 备料 | 2026-10-08 不受影响：判的是 G2 乙补法的实现与测试 |

## 五、用户定案

用户 2026-10-08 弹窗定「认不出时宽护 tri」（主 agent 推荐的就是这一项；弹窗里交代了四个候选、tri 在主 agent 复跑上三形 0 红 0 拒、384 槽盘挂载 116 槽对补法 42 槽、1536 槽与回退目标深时没量、s2 相近一形在 R_old 不在环里时 610–634 槽、删除发布的墓碑打包单元读不出时也会触发多护（推的）、被攻过零轮）。去向：
- 第 10z 批第十二块（core）：照正推 tri 臂（`research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`）的判法进 `mount.rs`：R 的记录点名的打包单元有读不出或解不开的、或认出回退而根环里没有 R_old 或 R_old 的账读不出，R 那一槽并上现行账里全部已释放的落点；先量 1536 槽盘与回退目标深时的隔离数、删除发布的墓碑单元读不出时多护多少，与正推对照臂同一量级以外的停下交主 agent。做完走代码轮。
- 同一块连同第四节 core 那一簇；第 10z 批第十三块（GPU）照第四节 GPU 那一簇。
- D16（发布语义） 已定项 1 第 37 行与 D28（挂载期承诺量） 已定项 1 第九项等第十二块交回、补法定形之后一起写回。
