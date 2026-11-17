# gpu-judge-service-r2：判决（2026-11-17）

正文 `research/prompts/_gpu-judge-service-r2-body.md`，开工快照 `research/prompts/gpu-judge-service-r2-snapshot/`。这是这一题的第二轮（设计推论轮），第一轮判决 `research/prompts/gpu-judge-service-r1-main-verification.md`。攻的是第一轮判决「三、改法」六条。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（three-way-attack，opus） | `research/prompts/gpu-judge-service-r2-opus-output.md` | 格 1、5、6 推不出（打中），格 2、3 不成立，格 4 第一轮那句成立、显存推不出 |
| 辩方（three-way-defense） | `research/prompts/gpu-judge-service-r2-sonnet-output.md` | 第一轮第 2 格判宽，第 3 格判宽一半，第 10 格线索一判宽、线索二判对，第 4 格判对、说宽一点，第 7 格判宽，第 8 格判对；改法六条里 1–5 站得住，第 6 条改它会换判器组键 |

核查：

- `python3 scripts/agents/snapshot-tree.py gpu-judge-service-r2 --out ~/.local/share/singlefs/work/gpu-judge-service-r2-verify/gpu-judge-service-r2-tree --repository <仓根>`（任务 18540 退 0）：「清单点名 15 份，倒推出 15 份，sha256 对上 15 份」，都取工作区此刻那一份。
- `python3 scripts/agents/cite-check.py research/prompts/gpu-judge-service-r2-opus-output.md research/prompts/gpu-judge-service-r2-sonnet-output.md --root ~/.local/share/singlefs/work/gpu-judge-service-r2-verify/gpu-judge-service-r2-tree --background research/prompts/_gpu-judge-service-r2-background.md`：「核了 90 处引文、89 处裸行号，对上 74 处，没判 105 处」，没有判红的；没判的多是快照树之外的代码文件，下面写主 agent 现读了哪几处。
- 辩方腿自报：在仓根跑带 `research/` 的递归 grep 时输出带出禁读文件一行，它没读那份文件、那一行没进结论。主 agent 认，不影响判。
- 主 agent 现读坐实的两处：
  - 攻方格 3：主工作区 `crates/singlefs-verification/build.rs` 第 125 行读 `repository_root.join("Cargo.toml")` 取 profile 生效值，第 138 行读 `repository_root.join("Cargo.lock")`。服务自成工作区、用自己的锁与 profile 编时，判器实际链进的依赖与构建档可以和键里记的不同。
  - 辩方改法 6：任务 18535 编出的 `judging_units_keys.tsv` 里 `gpu_judge` 那一组的单元有 `MAXIMUM_SCRATCH_LANES`（`crash_judge_dispatch.rs:33`）、`MEBIBYTES_LEFT_FOR_THE_DRIVER`（:36）、`states_in_flight_within`（:475–488）、`states_per_dispatch`（:907–918）、`DispatchWidthSchedule::starting_narrow`（:2023–2037）：定批宽的函数与常量在 GPU 判器那一组的闭包里。
- 没核的：两条腿引的其余代码行，按线索用。两条腿都没写模型、都没有要复跑的读数。

## 二、判决

用户在两条腿交回之后说：「我觉得你打中这6点都比较勉强 比如第六点。 这是服务调度 根据不同的 数据指定不同的批次数就可以了 这是什么难得问题吗」。主 agent 照被打中的是什么核了一遍，按下表判；「打中」只记坐实了的、能分辨做法的那一处，不把「行要定义清楚」这类写成打中。

| 格 | 判 | 依据与去向 |
|---|---|---|
| 1 待判行 | 做法成立，行形照两条腿补 | 攻方指出的两份真相（认领按墙钟过期、行值里的「判着」不过期）与重录无条件写，是行形没写清，不是做法错。照辩方：「判完」不存一位，按今天两个口径函数（复用口径、续点口径）在库上现算；状态另加「判不了」「卡显存不够交回队」两种；行指向录入表，不另抄旋钮 |
| 2 认领写进结论帧 | 做法成立，「对不上拒」那一句改写 | 攻方坐实了照字面会丢后到的红。照辩方的四条落笔：核放在去重之后；核「别人持着且租期没到」，不是「必须持着」；比较对象是写的一方读到的那一份；一份结论有多行时守卫放在最后落的结论块那一行。比较不符时不静默丢，照今天「有一份红整份换」，两份都绿而不同报红。持有者身份按每次进程分（攻方提、零轮） |
| 3 版本对账 | 打中一处，已坐实 | 服务必须是仓根工作区的成员，与验证包同一份 Cargo.lock 与 profile；`.claude/root-entries` 那一行同步改。两组键各自可取（辩方引 `crash_identity.rs:825`、`crash_amplification.rs:21266`），「服务拆不开」不成立。旧服务把旧结论写在新键下那条路径成立：服务启动时算自己的判器组键，待判行里带期待的键，对不上不认领，做成写入准入 |
| 4 旋钮 | 成立 | 旋钮已在录入表与块键里（辩方引 `crash_amplification.rs:6266`），行指向录入表、服务读回后重算块键核对。管线缓存进显存账：交服务第二段 |
| 5 强制现判 | 推不出，按辩方取「轮次加凭据行」 | 两条腿一致：凭据由服务写，读库再存也写得出。今天哨兵的判据本来就是「这个进程交来存过没有」（辩方引 `verdict_store.rs:3091`），服务路径照同一判据不比今天弱；CPU 参照复判改判据，不采。辩方给的第三种（每轮插已知答案的合成块，证判器此刻真在判）零轮，进第三轮 |
| 6 批大小与交替派 | 做法成立，是服务调度 | 按用户的读法：批大小由服务按数据定。坐实的一处是定批宽的函数今天在 GPU 判器那一组的键闭包里，改它就整库重判。改法：批宽由服务算好交给判器，判器只照给的宽度判；那几样移出判器闭包，键换这一次，之后服务里的调度怎么改都不碰键。按写到的字压草稿区要改 WGSL 偏移，是另一件优化，不进服务 |

第一轮判过的格（辩方复核）：第 2、3 格判宽的部分照上表收窄；第 7 格判宽，缺口记三处（清场拒而不杀、任务服务要卡的认法名单、记账只认它自己起的任务），进服务第二段的控制命令那一件；第 8 格补两处（控制命令每台一个单元、状态翻转的写量要量）。

第一段的实现（批 47）里工具员自定的三处：「一波每条流一批」「等过限时装不下就整卡重装」「限时 500 毫秒（推的）」，零轮，进第三轮。

## 三、改法

| # | 改法 | 状态 |
|---|---|---|
| 1 | 服务并进仓根工作区 | 量过（`build.rs` 第 125、138 行现读） |
| 2 | 批宽移出 GPU 判器闭包，由服务算好交给判器 | 量过（任务 18535 的键表现读）；键换这一次 |
| 3 | 待判行：指向录入表；状态待判、判着、判完（现算）、判不了、交回队；期待的判器组键 | 零轮 |
| 4 | 写结论帧里核认领，照辩方四条落笔，两份都绿而不同报红 | 零轮 |
| 5 | 服务启动算自己的判器组键，与待判行对不上不认领 | 零轮 |
| 6 | 强制现判照今天的判据挪进服务（轮次加凭据行） | 零轮 |

零轮的几条写进产品代码之前照「腿提的、判决里标零轮的改法，写进产品代码之前先在探针里对它做 A/B」办；第 1、2 条量过，直接进服务第二段。

## 交用户的

1. 批宽移出判器闭包会让 GPU 判器那一组键换一次、库里这一组的判定整组现判。用户已定不兼容旧东西，这一条照做，只告知。
2. 强制现判取「轮次加凭据行」，不取 CPU 参照复判（后者要动「登记了的那几类只走加速路径」那一条）。
3. 规则条文九处（第一轮判决「交用户的」第 1 条）照旧等第三轮之后一并交。

## 这一轮被判过的文件

- prune-server/src/committer.rs：认领与写帧两处（第 445–456、484–496 行）在改法 4 下怎么改，判了核放在去重之后、守卫放在最后落的结论块
- prune-server/src/protocol.rs：认领协议的形态，判为待判行的状态不放进认领行值
- crates/singlefs-verification/src/verdict_store.rs：「有一份红整份换」与哨兵判据（第 3091 行）在服务路径下照用
- crates/singlefs-verification/src/crash_amplification.rs：旋钮跟着录入表进库、两组版本键各自可取，判为待判行指向录入表即可
- crates/singlefs-verification/src/crash_judge_dispatch.rs：定批宽的函数与常量在 GPU 判器闭包里，判为要移出
- crates/singlefs-verification/src/crash_judge_gpu.rs：打包判表不带旋钮、旋钮编进扫描内核，判为第一轮那句成立
- crates/singlefs-verification/src/gpu_unit_checks.rs：管线缓存只增不删、不进显存账，判为服务第二段要记账
- crates/singlefs-verification/src/judging_units_entries.rs：第 29 行「调度与批的宽度不进键」的说明与闭包实况不符，判为要照改法 2 改到相符
- crates/singlefs-verification/src/gpu_memory_ledger.rs：块队列与显存账，判为服务的显存账另起、与它对拍
- task-server/task_server.py：要卡任务的认法名单与记账（辩方引第 605、614 行），判为服务常驻后要改的三处之二
- research/prompts/gpu-judge-service-r1-main-verification.md：第一轮判决，第 2、3、7 格收窄、第 8 格补两处
- research/prompts/gpu-judge-service-r1-forks.md：第 4 行交替派，混流一组 18516 五轮都大于 1，用户定照做

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-11-17 不受影响：批宽移出判器闭包之后，键的组成规则不变，只是闭包里少了不改判定结局的调度单元 |
| D13（验证路线） 已定项 15 | 支撑 | 2026-11-17 不受影响：判器整套照旧上 GPU，只是由常驻服务派活 |
| D13（验证路线） 已定项 4 | 不影响 | 2026-11-17 不受影响：设备模型与可达子集不动 |
