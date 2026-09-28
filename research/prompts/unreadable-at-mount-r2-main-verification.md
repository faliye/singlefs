# 挂载时读不出的两格：第二轮判决（2026-09-28）

正文 `research/prompts/_unreadable-at-mount-r2-body.md`；背景材料 `research/prompts/_unreadable-at-mount-r2-background.md`；快照 `research/prompts/unreadable-at-mount-r2-snapshot/`；冻结副本 `/tmp/claude-1000/unreadable-at-mount-r2/tree/crates/`；前一轮判决 `research/prompts/unreadable-at-mount-r1-main-verification.md`；岔路单 `research/prompts/unreadable-at-mount-r2-forks.md`。

<!-- doc-lint:not-numbers V1 V2 V3 V4 Y1 Y2 Y3 Y4 K1 K2 K3 L1 L2 L3 L4 -->

## 〇、快照与核查

- 快照 `sha256sum -c`：kb 12 份里 `.claude/kb/invariants.md`、`.claude/kb/checks-owed.md` 两份对不上，是别的会话在腿跑着时改的；腿引到这两份的行，逐处现查仍在（C331、C393、C32、C33、C75 各行）。crates：主工作区与 `crates-sha256.txt` 逐个相同。
- `python3 research/scripts/cite-check.py` 对四份腿报告，`--root .`：「核了 14 处引文，对上 14 处，没判 0 处」。
- 辩方引的四处现查：`16-发布语义.md` 第 179 行（已定项 7 持久顺序，「fsync 等系统配置轮换持久之后」在行内）；`22-单元原子性怎么合成.md` 第 416 行（「撕裂形态的推理**在本机无法证伪**，那一整块记为推理，不是结论」）；`21-权威态与派生态的分界.md` 第 202 行（行内有「挂载只读、重建停在级 1 并显式报告」那一句）；`18-块里携带什么信息.md` 第 313 行（「只读挂载不取号」在行内）。都对上。
- 攻方的数，从它模型目录 `research/prompts/unreadable-at-mount-r2-opus-model/` 的产物逐字找，摘整行：
  - `c331-summary-with-dev.txt` 第 11–12 行：`== candidate=jiaeio scenarios=1959 key_mismatch=0 verdicts={'refused': 1190, 'writable_no_loss': 649, 'writable_LOST_both': 9, 'writable_LOST_older_acknowledged': 111}` / `lost_cells=120 over_refused_cells=284 today_lost_now_not_lost=120`
  - 同文件第 56–57 行：`== candidate=jiaeiodev … verdicts={'refused': 1004, 'writable_no_loss': 835, 'writable_LOST_both': 9, 'writable_LOST_older_acknowledged': 111}` / `lost_cells=120 over_refused_cells=98 today_lost_now_not_lost=120`
  - `trajectory-summary.txt`：撕裂轮换 13 个状态里落点带 `T` 的 5 个（`111T1`、`111T0`、`1110T`、`111TT`、`1111T`），`jia: modes=RRRRRRx6 first_writable=none`、`jiaeio: modes=WWWWWWx6 first_writable=1`（主 agent 逐个 `grep`）。
  - 同文件 `[single_slot] … slot=NewestOnDevice0 read_back=DeviceError`：`jia: modes=RRRRRRx6`、`jiaeio: modes=RRRRRRx6`、`base`、`a`、`b` 都是 `WWWWWWx6`。
  - 同文件 `[c393_double] … account=AllocationRecordTreeRoot`：`a: modes=RRRRRRx6 first_writable=none … abandoned_out_of_ring_from=none`，`b: modes=WWWWWWx6 first_writable=1 … abandoned_out_of_ring_from=3,4,none`。
  - 三份轨迹表里 `refusals_unchanged=False` 0 行（主 agent `grep -c`）：拒可写与只读挂载前后盘面逐字节不变，攻方报 7 份副本 2016 条轨迹。
- 复跑没做：攻方七份副本各要编一次，主 agent 没在草稿目录重编重跑；上面只核了「产物里有这一行」。

## 一、本地腿样本

| 样本 | 判定 |
|---|---|
| `research/prompts/unreadable-at-mount-r2-local-attack-output-s1.md` | 干净 |
| `research/prompts/unreadable-at-mount-r2-local-attack-output-s2.md` | 参考：第 118、153 行逐字「config slots have unreadable or fail self-verify」缺词，主 agent 通读判带损坏；那两句不用，其余照「打中了」那一行当线索 |
| `research/prompts/unreadable-at-mount-r2-local-attack-output-void1.md` | 参考：损坏闸退出码 5（`reuses` 单字母替换 6 处，第 31–45 行） |

s1 的 25 格里 23 格与攻方量的轨迹逐格相同（撕裂三行：甲 只读、其余可写；树表行：(a)、(b) 只读、其余可写；分配记录树行：(a) 只读、(b) 可写）。另 2 格（树表行、分配记录树行的甲）s1 判只读、s2 判可写，攻方那一列 `jia: MISSING`（它的甲 副本造不出 C393 的起点，造法里那次挂载被甲 拒了）：没有量数，这两格不进任何判定。void1 与 s2 的其余格与 s1 同向。

## 二、判决

### L1　「一次故障之后永远只读」量实（K1、K3 那两条推断）

- 甲：撕裂轮换的 5 个状态连挂 6 次、6 种用户动作下每次只读，拒可写都在取号之前、盘上逐字节不变。第一轮判决 K1 那句「推的」改成量的。
- (a)：被抛弃根的账持续读不出的 40 格全部永远只读，被抛弃根一直留在根环里。K3 那句同样改成量的。
- 辩方指出这两句压着的「只读挂载不写盘」在 kb 里没有条款（`18-块里携带什么信息.md` 第 313 行只定「只读挂载不取号」；C32、C33、C75 写着「只读今天只是一句承诺」）。攻方在副本上量了：2016 条轨迹里只读挂载与拒可写前后盘面都没变。所以这一轮对今天的代码成立，对条款不成立——条款上它仍是一句承诺，已有欠账记着，这一轮不新开。
- 辩方补的限定词照录：`22-单元原子性怎么合成.md` 第 416 行「撕裂形态的推理在本机无法证伪」。这一轮攻方的撕裂状态是层 0 枚举取出来的状态（D 流第 0 段之后的 13 个），不是推的，但只取了整条流 2.68×10⁸ 个状态里的 13 个。

### L2　C331：没有一条既不丢写、又不在单故障上永远只读的候选

| 候选 | 丢写 | 多拒 | 撕裂轮换 | 一槽系统配置持续设备读错（单故障） |
|---|---|---|---|---|
| 今天 | 240 | — | 可写 | 可写 |
| 甲 | 0 | 719 | 永远只读 | 永远只读 |
| 甲-读错 | 120（全是 `Zeros` 形） | 284（全是 `DeviceError` 形） | 可写 | 永远只读 |
| 甲-读错-每盘（攻方提，零轮） | 120（同上） | 98（全是两盘各坏一槽） | 可写 | 可写 |

照跑前判据：甲-读错 丢写只在 `Zeros` 形、撕裂三格不拒 ⇒ 与甲并列进交用户的表。甲-读错-每盘 被攻过零轮，照「多轮」那一节交用户时标「零轮」。V4 那一支：甲 的取号扩展丢 0；甲-读错 的取号扩展丢 10（全是 `Zeros`），`DeviceError` 那一半罩住。辩方提的第三条路「根优先」（根槽 FUA 先于系统配置轮换，所选根本可以绕开见证找到较新的根）没验证，零轮，交用户时单列。

### L3　C393：(b)-从树表现算 过了判据，但树表那一指称和 (a) 一样永远只读

- (a)：复用 0，账持续读不出永远只读（量的）。
- (b)-从树表现算：复用 0（Y2 的 98 格），拒 14 格，全是树表读不出的格；分配记录树那几格可写，被抛弃根在第 3 到第 4 次挂载时离开根环；账读得出时现算的落点集与账里的逐个相同，比了 62 次。辩方判相容（`21-权威态与派生态的分界.md` 第 106、120 行），但缺第 202 行那条先例要的「显式报告」。照判据进表，定义补一句「显式报告」。
- (b)-从映射现算（攻方提，零轮，不要求树表读得出）：98 格全可写、复用 0。交用户时标「零轮」。
- 树表那一指称：(a) 与 (b)-从树表现算 都永远只读，只有零轮的 (b)-从映射现算 不只读。

### L4　旁及：坏扇区不重映射时，被抛弃根离开根环之后的那一形

攻方报：被抛弃根离开根环之后，那个读不出的槽回到空闲池又被占用，第 4 次挂载起可写、只读都挂不上。它不分辨任何一条臂（今天、(a)、(b) 都走得到），不进这一轮的判定；主 agent 没核，记进欠账交收拢。

## 三、交用户

岔路单 `research/prompts/unreadable-at-mount-r2-forks.md`。两轮之后的状态：C331 与 C393 都是取舍，不是有一条臂全赢——代价数都在岔路单里。第三轮只在用户选定之后攻被选的那一条（多为零轮形态），不为别的形态再开。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-09-28 不受影响：第 387 行 N-配置续的两个缺口的代价数在岔路单里，改不改等用户定 |
| D16（发布语义） 已定项 7 | 备料 | 2026-09-28 不受影响：撕裂轮换未被确认的依据取自第 179 行，这一轮不改 |
| D18（块里携带什么信息） 已定项 11 | 备料 | 2026-09-28 不受影响：「只读挂载不取号」不等于「只读挂载不写盘」，这一轮只量了今天的代码，不改条款 |
| D21（权威态与派生态的分界） | 备料 | 2026-09-28 不受影响：第 106、120、202 行判 (a)(b) 相容，(b) 补「显式报告」写进岔路单的定义，这一轮不改正文 |
| D22（单元原子性怎么合成） 已定项 21 | 备料 | 2026-09-28 不受影响：第 416 行方法论标注照录进判决，不改 |
