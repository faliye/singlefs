# m3-b44-code-r3 判决：代码欠账批 4-4 修红批合入的产品代码，第三轮（2026-10-18）

被判的对象：W13–W16 对 `crates/singlefs-core/src/` 下 `block_device.rs`、`deleted_inodes.rs`、`mounted_session.rs`、`mount.rs`、`recovery.rs`、`transaction.rs`、`unit.rs` 的改动（7 个文件 +1178/−183，全文 `research/prompts/_m3-b44-code-r3-diff.md`）。正文 `research/prompts/_m3-b44-code-r3-body.md`，冻结副本 `~/.local/share/singlefs/work/m3-b44-code-r3/repo/`。

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 L6 J1 J2 J3 W13 W14 W15 W16 U1 U2 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本与主工作区各 `sha256sum -c --quiet research/prompts/m3-b44-code-r3-snapshot/crates-sha256.txt`；`kb-sha256.txt` 同法 | 三样全对上（118 份 crates、kb 快照），腿跑着的时候被判的文件没变，不用倒推 |
| 正推 `research/prompts/m3-b44-code-r3-sonnet-output.md`（392 行）、攻方 `research/prompts/m3-b44-code-r3-opus-output.md`（290 行，sha256 5b6ce1a1…） | `cite-check.py <两份> --root <冻结副本> --root <仓根> --background research/prompts/_m3-b44-code-r3-background.md` | 「✓ 核了 79 处引文、288 处裸行号，对上 79 处，没判 288 处（其中裸行号没有引文可比 288 处）」；承重的裸行号主 agent 在冻结副本里现读：`transaction.rs` 第 1085 行注释「取号写覆写的是这块盘上较旧的那一槽」，`unit.rs` 第 44–45 行 `unit_filesystem_identifier` 只取 fsid 的前 8 字节，`make_filesystem.rs` 第 531 行「fsid 是调用方给的（…mkfs 自己不生成）」 |
| 正推模型 | 拷进会话草稿目录，`run-with-memory-cap.sh 14G` 跑 `tools/rerun.sh`（副本里两条 cargo 补了逐行范围声明，别的不动） | 退 0，除用时与窗口序数外与腿的 `out/rerun.stdout` 逐行相同。L2：「PROBE2 outcome=Err FloorRaiseFailedAfterTheMountsPublishes(… RollbackFloorCeilingNeedsUnreadableValidRootTreeTable …」「PROBE2 disk_changed=true writes_of_the_mount=420」，同窗口报读错时「Err Publish(PublishSequenceFailed { cause: ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(…」。L1 共用那一份：被删文件 4、12、20 个单元时两盘合计 2863104、6402048、9359360 字节 |
| 攻方模型 | 同上（副本里那一条 `cargo test --no-run` 补了范围声明） | 退 0，`out/` 下 9 份（fault-free-trace、follow-up、pair-summary、window-summary、sysconf、sysconf-writes、counterfactual、fsid-fixtures、rehearsal-endings）与腿的逐份相同。L1 ②：「before system-configuration dev=0 slot0:generation=10 … slot1:generation=9」，无故障第一个系统配置写「dev=0 slot=1」，`plan=w:303:1:garbage` 下第一个写「dev=0 slot=0」 |
| 正文里一处主 agent 的错 | 正文没照抄 W15 报告引的「D18（块里携带什么信息） 已定项 11」，说它与逐盘核对不上 | 错了：那一项从 `.claude/kb/decisions/18-块里携带什么信息.md` 第 253 行到第 355 行，第 316、320 行都在它正文里（正推腿报出，主 agent 核了下一个 `####` 在第 356 行）；不影响腿的判定，腿现查了原文 |

## 二、逐格判

| 格 | 攻方 | 正推 | 判 |
|---|---|---|---|
| L1 | 打中两形（量过，副本）。① 盘 0 两个系统配置槽在取号之前头一次读回坏字节：坏字节记进共用那一份，只进内存那一趟的取号读记下的那一份、停在取号、读错账空，照旧交真做；真做取号读盘读到好的，在改坏那份上走到 C841 时已落盘 19 次发布（W13 之前同一故障取号之前就交 C841、盘不变）。② 第 303 次读回坏字节一次，取号写算「写哪一槽」那一读走共用那一份，覆写了盘 0 最新那一槽（世代 10），与 `transaction.rs` 第 1085 行相反。没打中：无故障时写出的字节与 W13 之前逐字节相同（两版摘要 826326e3fd072b73） | 条款一致（每一次读走哪一路逐处列了）；共用那一份没有上界，真做续做读写的块也记（量的：每多一个数据单元多 64 KiB），判会撑爆合法挂载、交用户 | **① 打中，坐实**（主 agent 复跑）：反例落在岔路一那一行没写的地方——读成功而自证不过的字节在共用那一份里怎么办；交用户，岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路一。**② 打中，坐实**：取号写算写哪一槽的那一读就是「取号写之前那几次读」的一部分（它决定覆写哪一槽，D18 已定项 11 第 320 行那一段与 `transaction.rs` 第 1085 行的两槽轮换前提都要它读此刻的盘），代码把它切回了共用那一份，违反已有条款；修红批。**内存**：并进第二轮岔路二（用户选「先量再定」）的量，E179（预演内存随删除规模） 要把共用那一份一起量 |
| L2 | 打中条款没写的那一形（量过）：一份树表两份都读回坏字节时 ⓑ 不拒（坏字节不进读错账），真做取同一份坏字节、落盘之后交 `FloorRaiseFailedAfterTheMountsPublishes`。量了 ⓑ 偏拒：没改坏那份宽 2 的读错窗口 411 个拒 7 个、宽 1 拒 0 个；关掉这一拒这 7 个全做成、盘与无故障相同；同 7 个窗口落在改坏那份上不拒就是 C841 落盘之后才报；拒的那一刻两份盘分不出 | 拒后两盘不变、再挂照常：一致；21 处调用方逐处查过、成立；反例（量的）：读成功而字节坏时那一拒不触发，真做落盘 420 个写之后才报错 | **坏字节那一形打中，坐实**（两条腿各自量出、主 agent 复跑），与 L1 ① 同根，并进岔路一。**ⓑ 偏拒**：拒的那一刻两份盘观测不到差别（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判别子观测不到」），改判据交用户：岔路单岔路二 |
| L3 | 部分打中：坏字节一旦记下，整次挂载都取它，只有当场重读那几处穿到盘上；L1 ② 与 L2 那两读都没有当场重读。W16 没改的两处：根环槽单次读坏没打中；默认实现走故障注入那一层 | 改到的六处一致；没改的两处 U1 不用改，U2（E2，`mount.rs` 第 1374 行）读的是记下的字节、字面不一致方向安全（读码，没造镜像）；默认实现走故障注入层：一致 | 打中的部分与 L1 ①、L2 同根，并进岔路一；U2 记进下一轮清单（读码推的，没造镜像） |
| L4 | 没攻（只读码） | 一致：四条指针全进了判，谓词与写者断言同一个函数；附带一句：点名项的位置条目不在这一判里（不 panic，但 I-2.5 射程里有它） | 正推对得上、攻方没攻，记一次观测（不记「没打中」）；点名项位置条目那一句进修红批：先查恢复里有没有别处按 I-2.5 判它，没有就补 |
| L5 | 打中，判别子看不到：单元头只带 fsid 的低 8 字节，仓里当「另一个池」用的 `singlefs-earlier` 与 `singlefs-rev-a2b` 记进单元头都是 7366656c676e6973；mkfs 不生成 fsid；别池镜像本身没造（推的）；误拒本池的情形没找到 | 一致：fsid 取自开头择定的系统配置（`mount.rs` 第 7797 行），不另读盘；别池拒、本池放各有用例 | **打中，坐实**（主 agent 现读 `unit.rs` 第 44–45 行与 `make_filesystem.rs` 第 531 行）：W14 照用户定案做对了，但判别子观测不到——单元头格式本来只带 8 字节、fsid 由调用方给。改判据或格式交用户：岔路单岔路三 |
| L6 | 没攻 | ② `!= 8` 与写者条目宽 8 对得上；⑥ 越界一判与 `parse_data_unit` 同一个界；注释三处写宽：`mounted_session.rs` 第 562–563 行把取号列进「都取记下的那一份」，`mount.rs` 第 5302、7104 行「读同一份字节」对 W16 的重读不成立，「读错」不分报错与字节坏 | 三处注释进修红批 |

## 三、处置

**修红批**（并进下一批实现员，照「写与验分两段」）：

1. L1 ②：取号写算写哪一槽（与轮换写）的那一读穿到盘上，算进「取号写之前那几次读」（攻方改法表「丁」）。用例照攻方原型那一形（第 303 次读回坏字节一次，取号写不许覆写最新那一槽），变异「这一读走共用那一份」让它红。
2. L4 附带：恢复里有没有别处按 I-2.5 判 journal 点名项的位置条目；没有就并进 J1 那一判，补用例与变异。
3. L6：三处注释改成今天代码做的事。
4. 岔路一、二、三用户定了之后的改法。

**交用户**：岔路单 `research/prompts/m3-b44-code-r3-forks.md` 三条岔路。各候选都被攻过零轮；代价数只有窄盘一种几何上量的（ⓑ 偏拒 7/411，宽 2；共用那一份每单元 64 KiB），别的是推的。

**进 E179（预演内存随删除规模）**：共用那一份（`DeviceReadOnceInThisMount` 记下的读）随删除规模的内存，与只进内存的盘一起量；派下一段执行员之前由设计员补进跑前登记。

**推的、没造、进下一轮清单**：U2（E2 记根时读记下的字节）；L1 ② 取号写写到一半崩溃会怎样（要在 checker 档枚举，攻方原型做不了）。

**第三轮之后停**：这一轮是 m3-b44 代码轮的第三轮，照 `.claude/rules/three-way-inference.md`「第三轮之后停，不开第四轮」；修红批改完之后的形态与三条岔路的定案都算被攻过零轮，写进交用户表，不再为它们开第四轮。

## 这一轮被判过的文件

- `crates/singlefs-core/src/block_device.rs`
- `crates/singlefs-core/src/deleted_inodes.rs`
- `crates/singlefs-core/src/mounted_session.rs`
- `crates/singlefs-core/src/mount.rs`
- `crates/singlefs-core/src/recovery.rs`
- `crates/singlefs-core/src/transaction.rs`
- `crates/singlefs-core/src/unit.rs`

## 回看决策

不涉及决策：这一轮判的是代码对不对得上用户在第二轮岔路单上的定案、C834 比 fsid 的定案与已有条款，没有拿依据推翻或确立 `.claude/kb/decisions.md` 里的分项（回看：2026-10-18 不受影响：三条岔路交用户，岔路三选「头里带满 16 字节」或「mkfs 生成 fsid」时再看要不要改 D18 与 D15 的分项）
