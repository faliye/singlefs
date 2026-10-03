# m3-formal-r3 攻方腿报告（Opus，2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 M4 S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 T1 T2 T3 T4 J0 J1 J2 J3 J4 J5 J6 K10 Z1 Z2 F4 F14 Q7 E1 E2 L6 L9 C6 A6 G2 G4 G5 G8 IA1 V4 F11 -->

## 复跑命令与文件

模型目录 `research/prompts/m3-formal-r3-opus-model/`，复跑步骤在它的 `README.md`。被量的一律是冻结副本 `/tmp/claude-1000/m3-formal-r3/tree/` 的拷贝（`research/prompts/m3-formal-r3-snapshot/crates-sha256.txt` 212 份在拷贝上 `sha256sum -c` 全 OK）：一份原样编（E165 原样跑、池上原型），一份打 `patch_attack.py`（核九个开关、拼根三个开关、生成器两个开关，另数几样量；E165 原有判据一条不改）。Verus 0.2026.09.20.aef82ed，`--no-cheating`。

```
593952dce68797fc16e8e505c75fec0e5d27cc0bfb0b9d9251442a8e2b7d0f53  ./e165_attack_functions.rs
3efbbecf1cf4539dd792045503758c4f49648697bb95ec40214ab8f10c9b7806  ./m3_formal_r3_attack_unmount_marker.rs
a06593253967cb09f03402e9d6ba22ae104a190f633d4f7cab6b34643fd47ce6  ./patch_attack.py
8edfc5aaa056765ecff04833627f035e72225cca214530c397fcc3c8ad0906f7  ./README.md
5d0b195db6ddcef9858402a36e16342f8894543c191484e2f261c3c1c68ad5b9  ./run-all.sh
f5c63131a31c493a2534845465d66fa8859ab5535c97286fed3022b19cc1493b  ./runs/base-equivalence.txt
86f407111d3a21a409b495961fca749599655a6ff6bc5753f868f8984d2f3a95  ./runs/exits.txt
124c891414d549af673199eec8f61d2a3ae81ae3930af79ae90a79beaa7c7262  ./runs/k10-summary.txt
6bce7d78aecdae9ab6145a94994f19d18b53006d780387257f4bf9ee2c0a796d  ./runs/pairs-summary.txt
98c764c44542eba70e62b51dcd6ebc9132ea9dadb18203735f52fd1ed716011e  ./runs/summary-all.txt
e07ad1339cac7cc2fd9fa78dd7f998dec98864791b6c3941199a2534b30bb17c  ./runs/txg-cap-summary.txt
30dcfcdbe2dec5ab67ea70dc67e1d24cbc7c95bcb04424b3e7ddc5eb9f0cae1a  ./runs/unmount-marker.txt
e00ce05cd7cd1fa4c3ff419cf746d62e92e43448c6423dcdb65d78b507eeb701  ./runs/verus-s2-plus-spec-verdicts.txt
a338ebeb6d45ecbaecfa342df631c502ffb988e6b6c9439f5e38fdc2e9c0bfbe  ./runs/verus-s2-plus-v08-rlimit1000.txt
193598a361b9e568cfea318c1c0038ed5a31ce3aa0e95f7c4e82974ec425ec53  ./runs/verus-s2-plus-v08-rlimit200.txt
094105bde1c091e252360690afda76f6c2ddc5ba7e4594267bf0680e4b83b6c5  ./runs/verus-s2-spec-verdicts.txt
55f1889cf9e0a76f5f58c6ad887ab248b102046687a1a6eaa15536a28371fa51  ./runs/verus-today-spec-verdicts.txt
2ad1f7e85c4051d9949d91d2a6ab26af7d2bffa5a22113fd7ccb98379577ead7  ./run-verus.sh
a6be4dc8a3e84431146b07addab06ac78ef3f5c321f2b37f3d2b17cb4ed16a02  ./s2_plus.py
a6e242239585b1657ecdfe7743d4ceea911ce97e9706eebdd4035092aa4f19da  ./s2_strengthen.py
2162fc022e7a81cc89c02b1c5cbb75fbd82d29d888114f48f781309d8cc48c74  ./summarize.py
50afacc26b146a98b6c53d98dc3468d76a22fdc425ee2496b8ae8727c774c5f2  ./verus_variants.py
```

**数全是副本上量的**，不是入库装置上的数。装置等价（`runs/base-equivalence.txt`）：原样编的 E165 跑 `--bounds light --family legal`，876 行 `E7RESULT` 与 `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out` 逐字节相同；`--bounds segment1 --family adversarial --only A` 的 A 域 57 行与 `…-segment1-adversarial-family.out` 里 A 域那 57 行逐字节相同；打了补丁、开关全 0 的那份去掉 `name=attack` 与 `name=done` 两类行之后 875 行与入库产物逐字节相同。核已从 `recovery.rs` 搬进 `singlefs-replay-prefix`，这三处逐字节相同也说明搬家没改这几个域上的结局。

规模：E165 一共跑 25 次、合计 279 965 230 态（每态一到三次 `replay_journal`），挂钟合计 210 秒（8 线程）；先跑一次原样 light legal（4.2 秒）估的全量，不到一分钟一批，没缩取样。池上原型只建一个池、只取一个崩溃状态，没调层 0 的枚举函数。Verus 跑 13 × 3 个变体加 4 次单跑。

## 各格判定一览

「量过」= 这一轮在副本上量的，贴了原样输出；「推的」= 读代码或条款推的，没实现没跑。

| 格 | 被攻的形态 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|---|
| T1·S1 后半句 | 「其余字段等于所选根」 | **打中** | 正常卸载那一串第二次空发布的记录落了、根槽没落（不带故障的崩溃状态），今天的实现施加出来的根卸载记号是 0、所选根是 1。算不算实现缺陷，D23 已定项 15「四个字段换成记录里的」与 D22 已定项 7「其余根写 0」各指一边：条款欠一句 | 量过（池上 1 态；E165 副本 C 域甲切法 911 / 8 006 态） |
| T1·S1 前半句 | 投影 + 盘 | 站住 | 写行根进生成器后今天的实现投影读法 0 违例、整条根 1 007 514；施加时清零分配记录树根、实例表指针取错两个变异都只有「其余字段」那一判抓到 | 量过 |
| T1·S2 | 句法层补连续与链首、表写成 `requires` | **改（收严）** | 今天的 spec 对七·2 的必红世界与表第 36 行都是 14 verified；我写的补强证过原核、把这两个判红，方向对；但补强的两句不含在飞上限与提交标记，Z2 在补强之后照样 14 verified，而它在 E165 合法历史一族上 J4 红（L6 354、L9 51 998、C6 215）；`requires` 升序对这两句不承重 | 量过（副本 Verus） |
| T1·S3 | 唯一前提只挂语义层 | 没打中 | 写侧几条重发路径都换实例代号或逐字节原样 | 推的 |
| T1·S4 | M4 与择根合起来排除丢已落盘的完整发布 | **射程句收严** | M4 本身站住；「连同择根」只在不带故障的崩溃状态上排除得了：S2 取的 E165 合法历史一族里较新根槽读不出时，在飞上限把记录完整的发布截掉（L6 354、L9 51 998、C6 215 态，甲切法合法态 0） | 量过 |
| T1·S5 | M3 单调写成推论 | 没打中 | 合法对上 J5b 0（复现）；推论第 4 步要「恢复只是记录与根槽的函数」，择根只读系统配置的不可变字段，成立 | 量过 + 读代码 |
| T1·S7 | 主语三处 | 没打中 | 三处产品调用都传 `true`、`records` 都是 `scan_journal` 的输出 | 读代码 |
| T1·S8 | 第四格补句挂 K10 | 没打中 | K10 下全表看与缺号不停在 Rust 上 0 / 0（A 到 n = 4、AC 到 n = 3） | 量过 |
| T1·S9 | 同源计数器「条款欠一句」 | 没打中 | 补句不来时事件永不归档，是弱点不是反例 | 推的 |
| T1·S10 | 七·5 判别力样本与判绿 | **打中两处** | (a)「spec 太弱」只列七·2、七·3，S2 补强之后 Z2、表第 516、515 行、第四格、Z1 仍 verified；(b) 必红按「verified, M errors」字样判，rlimit 用尽也是「1 errors」：关掉反向链的变体 rlimit 200 报 13 / 1、rlimit 1000 报 14 / 0 | 量过 |
| T2 版本序 | checkpoint_txg 还是 (实例代号, txg) | 两种写法在 S5 的域上不可分 | 新实例 txg ≥ 环里一切 + 1、实例代号取 max + 1、回退不新增实例；分得开要一次取号时读不出的记录 | 推的（条款） |
| T2 txg 顶格 | 前提「不顶格」还是「报错且一条没施加算相等」 | **分得开** | E1 上今天的实现两种写法都 0；表第 1003 行（顶格不判、第二次调用溢出 panic）：写法 A 0 / 328，写法 B 70 / 398。A 把这一处缺陷排出判定，B 判红 | 量过 |
| T3 | 欠账清单 | 补八笔 | 卸载记号、spec 漏两句、rlimit 冒充必红、verified 数判不出 spec 变弱、S4 的域、合法崩溃状态的口径、择根与重放没有合判、J6 不照选定写法判 | 量过 + 推的 |
| T4 | 第二轮的数 | **全部复现**；一处口径写错 | 1 007 514、264 499、352 608、15 343、K10 下 0 / 0、57 585 320、辩方 n ≤ 4 都在冻结副本上逐数对上；15 343 是乙切法的合法态数，写成「D16 已定项 7 持久顺序下」不对，那个顺序下是 8 389 | 量过 |

候选写法全部只在我的模型上量过或推的，被攻过零轮。

## 一、T1：站住的形态撑不撑得住

### 1.1 S1 后半句「其余字段等于所选根」：今天的实现在一个不带故障的崩溃状态上为假

正文问的是这半句「它是不是把一个实现该做的事写成了条款」（`research/prompts/_m3-formal-r3-body.md:36`）。答：它写的事有一格今天的实现不做，而且不做是有条款撑着的。

**历史**（每一步都是今天的代码允许的）：mkfs → 第一次可写挂载（取号 1、写行、暖机）→ 发布第一个文件版本 → 正常卸载。卸载在「现行那一版带文件」时推一串带卸载记号的空发布，「带新 F、带卸载记号的空发布，直到每块盘上都有一条（第一版几何上 2 或 3 次）。」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/mount.rs:1976`）。每一次都是一次普通发布：单元 → 记录 → 根槽（FUA）→ 系统配置。崩在第二次空发布的记录已落、根槽没落：前缀切在段边界上，是 D13 已定项 4 意义上的崩溃状态，不带任何故障。

**池上量**（原样的核，内存稀疏盘；`research/prompts/m3-formal-r3-opus-model/m3_formal_r3_attack_unmount_marker.rs`）：

- 卸载推了两次：`research/prompts/m3-formal-r3-opus-model/runs/unmount-marker.txt:1`「unmount_publishes=2 markers=[WrittenByTheUnmountSequence, WrittenByTheUnmountSequence] txgs=[4, 5] counters=[4, 5]」
- 恢复：`research/prompts/m3-formal-r3-opus-model/runs/unmount-marker.txt:3`「chosen=(1,4) chosen_marker=WrittenByTheUnmountSequence rebuilt=(1,5) rebuilt_marker=NotWrittenByTheUnmountSequence prefix_applied=1」
- 四个「其余字段」逐个比：`research/prompts/m3-formal-r3-opus-model/runs/unmount-marker.txt:4`「rebuilt_equals_second_publish_root=false other_fields_equal_chosen=(true,false,true,true)」（次序 fsid、卸载记号、实例表指针、分配记录树根）

所选根 (1, 4) 带卸载记号，施加了 (1, 5) 那一条之后的根不带；(1, 5) 真写下的根带。S1 后半句要它等于所选根，今天的实现给的是另一个值。已撤的整条根读法在这一态上同样为假。

**为什么**：拼根那一行「unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/recovery.rs:3147`），注释说由记录施加出来的这一版「不是卸载那一串写下的根」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/recovery.rs:3145`），枚举的文档也写明记号 0 的根里有「以及由记录施加出来的那一版」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/root_record.rs:26`）。

**E165 副本上数**（生成器开关 `ATTACK_GEN=2`：每一版的根都带卸载记号，等于一段全在卸载那一串里的历史）：

- L 域施加过的态全中：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:148`「atk_s1_marker_differs=1007514」
- C 域里只数甲切法（根槽自成一段，D16 已定项 7 的持久顺序）停在某一段的态：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:153`「atk_legal_own_states=8006 atk_legal_s1_marker_differs=911 atk_legal_own_s1_marker_differs=911」；其中「至多三次、每次一条记录」那种卸载形状的态 C 域 20 个里 5 个中（同一行 `atk_unmount_shape_marker_differs=5`）。
- 拼根照抄所选根的卸载记号（`ATTACK_GLUE=2`）之后 S1 判与整条根判都归 0：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:137`「gen2-glue2 L atk_unmount_shape_states=14 states=4031078」（这一行没有 `atk_s1_*`、`atk_full_root_mismatch`，即为 0）。

E165 原装置看不见这件事：生成器里「不在新根段里的字段（实例表指针、分配记录树根、卸载记号、fsid）全历史相同，不进判据。」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:289`）；Verus 核只交回新根段，拼根在胶水里（G8）。

**条款两边**：D23 已定项 15 的定案「施加一条记录 = 把所选根的这四个字段换成记录里的，不需要树语义」（`.claude/kb/decisions/23-journal的角色与格式.md:443`），照字面其余字段（含 flags）都留所选根的，今天的实现是缺陷。D22 已定项 7 的 flags 一行「正常卸载那一串写的根置 1…其余根写 0」（`.claude/kb/decisions/22-单元原子性怎么合成.md:137`），把施加出来的根读成「其余根」，今天的实现对、S1 错。两句都没说施加出来的那一版算哪一种。

**打中之后的四句**：

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨：S1 原句与已撤的整条根读法都中；只要投影、不说其余字段的写法与「其余字段等于所选根，卸载记号除外」的写法不中 |
| 被判的系统看不看得到判别子 | 看得到：所选根的卸载记号就在恢复手里，今天的实现是有意写 0。另一头看不到：(1, 5) 真写下的记号不在记录里，所选根是普通根、施加的是卸载第一次空发布时，照抄所选根给 0、真值是 1，哪种拼法都对不上整条根 |
| 满足的是判据哪一个分句 | 本轮正文第六节第二条（今天的实现在合法历史上报违例、而那不是实现缺陷 ⇒ 撤）——前提「不是实现缺陷」只在 D22 已定项 7 那一读下成立；照 D23 已定项 15 的字面是实现缺陷，落第三条（一留一撤记不一致）的形状。是 S9 那一类「条款欠一句」 |
| 改法在打中的格上还中不中 | 撤掉后半句：不中，但丢了判实例表指针取错的那一判（`ATTACK_GLUE=1` 过 J1–J4、只有这半句抓到，见 1.2）。收严成「其余字段等于所选根；卸载记号例外：施加过至少一次时为 0」：不中（推的：我的判据去掉卸载记号那一项后 gen2 上 `atk_s1_other_fields_differ` 只剩卸载记号这一来源，没单跑）。改实现照抄所选根：不中（量过，上一条 gen2-glue2） |

**我提的写法**（零轮，只在我的模型上量过）：S1 后半句改成「其余字段等于所选根，卸载记号除外：施加过至少一次时卸载记号为 0（D22 已定项 7「其余根写 0」）」，并在 D23 已定项 15 补一句施加出来的根 flags 怎么取。哪一边改由用户定；改实现那一边要另想清楚「卸载第一次空发布施加在普通根上」那一格（真值 1，照抄得 0）。

### 1.2 S1 前半句（投影加盘）：站住，复现第二轮

- 写行发布的根进生成器（`ATTACK_GEN=1`）：今天的实现整条根不等 1 007 514、投影加盘（我的 S1 判）0：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:126`「gen1 L atk_full_root_mismatch=1007514 atk_unmount_shape_states=14」。
- 施加时把分配记录树根清零（`ATTACK_GLUE=3`）：整条根 0、S1 判红：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:115`「atk_s1_alloc_differs=1007514」。
- 实例表指针取成记录里的映射根指针（`ATTACK_GLUE=1`）：J1–J4 全 0、只有「其余字段」这一判红：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:159`「atk_s1_instance_table_differs=1007514」。这就是 1.1 第四句里「撤后半句会丢一判」的依据。
- 「盘」那一半：关掉点名验证那一臂 S1 判与 J2 同数：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:178`「m0 U_verify_off atk_s1_violations=352608」。

### 1.3 S2：补强的方向对，但列的两句漏了能进 spec 的两样

我在核的副本上把 S2 的补强写出来证了一遍（`s2_strengthen.py`，零轮）：`m2_holds` 加「施加的计数器 = 第一条的计数器 + 下标」与链首（锚点读得出时 = 锚点 + 1；读不出时 txg = 根 + 1、序号 1），锚点函数加三条 `ensures`，主循环加两个幽灵量与几条不变式。原核在它上面 14 verified、0 errors。再在它上面加我提的两句（`s2_plus.py`，零轮）：施加条数不超过 `in_flight_limit`；一次整的发布末条带提交标记、不带提交标记的那条之后不换事务号。这一版要 `--rlimit 200` 才证得过（默认 rlimit 报 rlimit 超限）。

13 个核变体（`verus_variants.py`，每个只改一处判定）在三版 spec 上的判定行，与 E165 副本上同一改法的读数：

| 变体 | 今天的 spec | S2 补强 | 补强再加两句（rlimit 200） | E165 合法历史一族（副本） |
|---|---|---|---|---|
| v00 原核 | 14 / 0 | 14 / 0 | 14 / 0 | 全 0 |
| v01 七·2 必红世界（I-8.3 那一判不断） | 14 / 0 | 13 / 1 | 13 / 1 | L 域 J2 264 499 |
| v02 表第 36 行 | 14 / 0 | 13 / 1 | 13 / 1 | L 域 J2 164 809 |
| v03 Z1 一条都不施加 | 14 / 0 | 14 / 0 | 14 / 0 | 只有 J4（L 域 1 007 514） |
| v04 Z2 不看在飞上限 | 14 / 0 | 14 / 0 | 13 / 1 | J3 = J4：L6 354、L9 51 998、C6 215 |
| v05 表第 516 行 | 14 / 0 | 14 / 0 | 13 / 1 | 合法一族 0；对抗一族 A 域 A-结构 25 058 |
| v06 表第 515 行 | 14 / 0 | 14 / 0 | 13 / 1 | 没在副本上开 |
| v07 第四格关掉 | 14 / 0 | 14 / 0 | 14 / 0 | 没在副本上开 |
| v08 反向链关掉 | 14 / 0 | 14 / 0 | 13 / 1（rlimit 超限；rlimit 1000 时 14 / 0） | 没在副本上开 |
| v09 点名验证关掉 | 14 / 0 | 14 / 0 | 14 / 0 | 同形的 U 验证关 J2 352 608 |
| v10 表第 613 行 | 13 / 1 | 13 / 1 | 13 / 1 | — |
| v11 表第 406 行（七·3） | 13 / 1 | 13 / 1 | 13 / 1 | L 域 J2 1 093 324 |
| v12 表第 513 行（锚点两条不报错） | 14 / 0 | 13 / 1 | 13 / 1 | — |

原样输出：`research/prompts/m3-formal-r3-opus-model/runs/verus-today-spec-verdicts.txt:2`「v01-skip-I-8.3-jsn-check rc=0 verification results:: 14 verified, 0 errors」；`research/prompts/m3-formal-r3-opus-model/runs/verus-s2-spec-verdicts.txt:2`「v01-skip-I-8.3-jsn-check rc=1 verification results:: 13 verified, 1 errors」；`research/prompts/m3-formal-r3-opus-model/runs/verus-s2-spec-verdicts.txt:5`「v04-Z2-ignores-in-flight-limit rc=0 verification results:: 14 verified, 0 errors」；`research/prompts/m3-formal-r3-opus-model/runs/verus-s2-plus-spec-verdicts.txt:5`「v04-Z2-ignores-in-flight-limit rc=1 verification results:: 13 verified, 1 errors」。E165 那一列：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:44`「core4 L6 atk_s1_violations=354」、`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:79`「a_structure_violations=25058」。

读法：
- 第二轮判决「主 agent 现跑坐实」的那件事在 r3 的核上照样成立（今天的 spec 对 v01、v02 都 14 / 0），S2 的补强量过能把它们判红、不碰原核的证明。这一半站住。
- **漏的两样**：在飞上限与提交标记都是切片与入参上的句法谓词，S1 的「盘」里写了在飞上限、D23 已定项 14 前缀五条里写了提交标记，S2 的补强一句都没带。Z2 过 S2 补强之后的句法层（14 / 0），在 E165 合法历史一族上 J4 红——本轮正文第六节第一条的形状。它今天被 E165（S1 的「盘」那半句）抓到，不被 Verus 抓到；所以这一格打的是「哪半句进工具」的分工，不是 M2 整体。
- `requires` 升序：s00、s01 两版都没写这条 `requires` 就证过了，补强的两句不靠它（「逐一连续」直接写在计数器上）。它承重的是第四格（核里那段切片扫描的注释自己写了靠升序才与原文同义）与「不跳过」一类说法，那两样今天不在 spec 里。
- v12（锚点两条不报错）在补强之后变红，是链首那句的副产品：链首写成「对每一条锚点 = 它 + 1」。

哪半句只能留给胶水与有界穷举（读代码，没写证明试）：点名单元两份都验（回调，G5）、反向链（回调，G4）、投影之外拼根（G8）、表从盘上怎么来（G1–G3）、M4「不少施加」（要说「前缀只在这些断点停」，断点里有两个回调）、J1 的「时间线上第 j 版」（要幽灵时间线）、几条前提本身。

| 四句 | 答 |
|---|---|
| 分不分辨臂 | 分辨：S2 原列的补强 vs 补强再加两句；Z2、516、515 只在后者上红（量过） |
| 系统看不看得到判别子 | 看得到：`taken`、`in_flight_limit`、`is_commit`、`transaction` 都在核的入参里 |
| 判据哪一个分句 | 第六节第一条（过 S2 句法层字面、在合法历史一族上违反 J4）；限定：语义层照样抓到 |
| 改法在打中的格上 | 「收严：句法层再加两句」不中（量过 v04、v05、v06 红）；代价是 rlimit 要调到 200（量过），见 1.8 |

### 1.4 S3（唯一前提只挂语义层，由原样重发担保）：没打中

今天写侧会让同一个键再出现的几条路都换了实例代号（零单元发布那一支失败后整个挂载报错、下一次换代号；取号之后写行或暖机失败号烧掉）或逐字节原样重发；环回绕盖掉的是计数器小一圈的槽。没找到同一 (实例代号, 计数器) 两份内容的合法路径（推的，读 D23 已定项 14、已定项 16 与 `mount.rs`）。「推进 txg 再发」那条路落地时重核的射程句照留。

### 1.5 S4（M4 与择根合起来）：M4 站住，射程句只在不带故障的状态上成立

M4 本身复现：Z1 在全部合法域 J1–J3 为 0、只有 J4 红；「水位之上两条以上就一条都不施加」在甲切法合法态上 J4 526：`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:37`「atk_legal_j4=911 atk_legal_own_j4=911」（Z1）、`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:73`「atk_legal_j4=526 atk_legal_own_j4=526」。

候选射程句写「它连同所选根怎么择（D22 已定项 7 的最新有效根）一起才排除」（`research/prompts/m3-formal-r2-opus-output.md:185`）。S2 把语义层的前提域取成 E165 合法历史一族——那一族里较新的根槽可以读不出。在那个域上，所选根退到旧的一版、它之上的记录全都完整在盘，在飞上限照样把超出的那几次完整发布截掉；这是 D23 已定项 18 有意的（「这个数的含义是一次恢复重放的前缀最多 N 条」，`.claude/kb/decisions/23-journal的角色与格式.md:494`），M4 的 j\* 也带着上限，所以今天的实现过 M4。截掉的态数就是「不看上限」的 Z2 与今天的实现 J4 不同的态：L6 354、L9 51 998、C6 215（`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:45`「core4 L9 atk_s1_violations=51998」，另两格见 1.3 表）；C6 里甲切法合法态 383 个，一个都不在其中（`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:49`「atk_legal_states=606 atk_legal_own_states=383」，这一行没有 `atk_legal_own_j4`，即 0）。另一处是用户认下的：「代价：两块盘的池上一块盘整块掉了」（`.claude/kb/decisions/23-journal的角色与格式.md:363`）那一格，记录在、点名单元少一份，那一版不施加。

| 四句 | 答 |
|---|---|
| 分不分辨臂 | 不分辨 M4 的写法（M4 两种读法都过）；打的是射程句 |
| 系统看不看得到判别子 | 看不到：被截的那几版的根槽读不出，恢复不知道它们落过盘 |
| 判据哪一个分句 | 都不是 J 违例；是射程句在 S2 自己取的域上外推过头 |
| 改法 | 射程句收严成「在 D13 已定项 4 的不带故障的崩溃状态上，它连同择根排除…；带故障的域里在飞上限截断与 C519 那一格不排除」（零轮） |

### 1.6 S5（单调写成推论）：没打中

合法对集合上 J5b 复现为 0（今天的实现、开关 1、2、3、5、6）：`research/prompts/m3-formal-r3-opus-model/runs/pairs-summary.txt:2`「m0 C cut_own_legal pairs=71579 today_j5a=4969 today_j5b=0」、`research/prompts/m3-formal-r3-opus-model/runs/pairs-summary.txt:110`「core6 C cut_own_legal pairs=71579 today_j5a=2883 today_j5b=0」；「宽」集合上开关 1 红：`research/prompts/m3-formal-r3-opus-model/runs/pairs-summary.txt:19`「core1 C wide pairs=7322544 today_j5a=1215527 today_j5b=21710」。

试的形状：推论第 4 步要「u 与 s 只差 u 多出来的写，而恢复只读记录与根槽」。D16 已定项 7 里根槽（FUA）与系统配置槽同在一段，u 可以比 s 多一槽系统配置、少那次根槽。择根只读系统配置的不可变字段（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/recovery.rs:1157`「&system_configuration.immutable.region_devices,」），扫环与在飞上限只用环长，所以这一步成立（读代码）。限度：推论说的是一次 `replay_journal`；可写挂载的读阶段另读系统配置里的 journal tail 定要不要重读一遍，那一层不在 M3 里（推的）。

### 1.7 S7、S8、S9：没打中

- S7：三处产品调用照旧——可写挂载读阶段传 `true`（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/mount.rs:4361`「let (journal, effective_root) = replay_journal(」），只读挂载同形（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/mounted_read.rs:559`「let (journal, effective_root) = replay_journal(」），`recover` 只在 `Consult` 时传真（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/recovery.rs:4191`「policy == JournalPolicy::Consult,」）；三处的 `records` 都是同一函数里 `scan_journal` 的返回值，产品读者的环槽提示都交 `None`。
- S8：K10 成立的态上全表看、缺号不停与实现逐态相同，在 Rust 上量到 n = 4：`research/prompts/m3-formal-r3-opus-model/runs/k10-summary.txt:8`「atk_k10_states=28459330 atk_k10_as_whole_table=0 atk_k10_as_gap_continues=0 atk_k10_as_gap_stops=233600」，AC 到 n = 3：`research/prompts/m3-formal-r3-opus-model/runs/k10-summary.txt:14`「atk_k10_states=4079970 atk_k10_as_whole_table=0 atk_k10_as_gap_continues=0 atk_k10_as_gap_stops=12544」。K10 态上不动点前提真的违例也是 0（同文件第 1、3、4 行 `atk_k10_j6_premise_holds=0`）。
- S9：只记一个弱点（推的）：「先记账、补句后再判」没有出口——补句一直不来（例如 K10 那一轮判「不立」），事件就一直不进 5 次的门槛，同源性的判定永远开着。建议给账上每一笔挂它等的那一轮。

### 1.8 S10（七·5 判别力样本与判绿）：打中两处

(a) **「spec 太弱」只列了七·2、七·3 两个世界**（`research/prompts/m3-formal-r2-opus-output.md:278`「判别力样本五份」那一句）。1.3 的表：S2 补强之后，Z2、表第 516、515 行、第四格关掉、Z1 都还是 14 / 0，其中 Z2 在 E165 合法历史一族上 J4 红。补强前后原核都是「14 verified」（`research/prompts/m3-formal-r3-opus-model/runs/verus-today-spec-verdicts.txt:1`「v00-original rc=0 verification results:: 14 verified, 0 errors」，S2 补强那份第 1 行同样）：verified 数是函数个数，spec 变强变弱它不动，「verified 数等于登记」判不出 spec 被削弱。要判出来，只能靠每一条 spec 合取项各配一个必红世界。

(b) **必红世界按「verified, M errors」字样判，rlimit 用尽同样是「1 errors」**。核目录的必红表规定期望「不带 N（核里多一个函数 N 就变，不让整表跟着改）」（`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-replay-prefix/verus-mutations.tsv:4`）。我的 S2 再加两句那一版上，关掉反向链的变体（spec 根本不说反向链）：`research/prompts/m3-formal-r3-opus-model/runs/verus-s2-plus-v08-rlimit200.txt:1`「error: while loop: Resource limit (rlimit) exceeded; consider rerunning with --profile for more details」、同文件第 2 行「verification results:: 13 verified, 1 errors」；rlimit 1000 再跑：`research/prompts/m3-formal-r3-opus-model/runs/verus-s2-plus-v08-rlimit1000.txt:1`「verification results:: 14 verified, 0 errors」。照 S10 的字面，它在 rlimit 200 时算「必红世界报了 errors」——是证明脆，不是 spec 抓到。

| 四句 | 答 |
|---|---|
| 分不分辨臂 | (a) 分辨「五份样本」与「每条合取项一个必红世界」；(b) 分辨「看 errors 数」与「看错误种类」 |
| 系统看不看得到判别子 | 看得到：Verus 的错误行写着是 postcondition / invariant / assertion 还是 rlimit |
| 判据哪一个分句 | (a) 七·5 验收「spec 太弱」那一份覆盖面；(b)「必红世界的核必须报 errors」那一句 |
| 改法 | (a) 每条 spec 合取项至少一个必红世界（今天：连续与链首 v01、v02，序号 v10，末条 v11，锚点唯一 v12；加两句后在飞上限 v04、提交标记 v05、v06）；(b) 必红只认 postcondition、invariant、assertion 三类错误行，rlimit、超时一律判「没判」（零轮，都只在我的模型上量过） |

## 二、T2：两格零轮

### 2.1 S5 的版本序：两种写法在 S5 的域上不可分（推的，按条款）

两种版本序只在「时间上靠后的一版 txg 不大于靠前的一版」或「实例代号倒退」时给出相反判定。合法写侧把这两样都堵了：
- 新实例的 txg：「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。
- 新实例的代号：取 max(系统配置, 根环) + 1（D23 已定项 16，`.claude/kb/decisions/23-journal的角色与格式.md:456`；代码 `next_instance_generation_to_acquire` 同一读法，`/tmp/claude-1000/m3-formal-r3/tree/crates/singlefs-core/src/transaction.rs:705`「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」）。
- 同一实例内：每次发布 txg 加一；管理员回退「checkpoint_txg 照常加一，**不新增实例、不取号、不写实例表行、不施加也不删除任何记录**」（`.claude/kb/decisions/23-journal的角色与格式.md:376`）。

所以同一段运行里先后两版 A、B：txg_A < txg_B 且实例代号 A ≤ B，两种序都给 A < B。不带故障时恢复落到的版本都在这条时间线上（被抛弃那一截没走完的发布不施加），两种写法逐对相同。分得开的观测要一次取号读漏了环里 txg 最大的那条记录或根（暂时读不出、之后又读得出），新实例的 txg 就可以不大于旧实例那一版——那是带故障的域，S5 自己写了「域放宽到带故障时重开」。E165 的 X 域把 txg 按全局发布序号往上接，两种序在那个生成器里按构造重合，量不出差别。**两种写法在 S5 的域上不可分，交用户定**；我倾向写 checkpoint_txg（与 D22 已定项 7 择根的主键一致），没有观测撑着。

### 2.2 S6 的 txg 顶格：分得开，写法 B 的违例集合包含写法 A 的

两种写法照第二轮原句实现成计数（只在所选根 txg ≥ u64 最大值 − 1 的 E1、E2 域判；都带 E165 原有的在飞上限前提）：
- 写法 A（辩方）：前提「所选根与施加后的根的 `checkpoint_txg` 小于 u64 最大值」（`research/prompts/m3-formal-r2-sonnet-output.md:124`），前提假的态不判。
- 写法 B（攻方）：「第二次因所选根 txg 顶格报错且一条没施加的，算相等」（`research/prompts/m3-formal-r2-opus-output.md:246`），其余照不动点判。

E1 就是 E165 实验页逐行判定 Q7 那一格（「E1 是 70（第二次调用在 txg 顶格的根上报错）」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:168`）。量到的：

| 实现 | E165 原 J6（前提真） | 写法 A 违例 / 判了 | 写法 B 违例 / 判了 | 别处抓不抓 |
|---|---|---|---|---|
| 今天的实现 | 70 | 0 / 328 | 0 / 398 | — |
| 表第 1003 行（顶格不判，第二次调用加一溢出 panic） | 70 | 0 / 328 | **70** / 398 | E2 上 J0 42 |
| 表第 406 行（每条都当末条） | 114 | 0 / 284 | 0 / 398 | E1 上 J2 44、J4 44 |
| 表第 1004 行（差一格就报顶格） | 0 | 0 / 0 | 0 / 0 | E1 上 J4 70、报错 398 |

原样输出：`research/prompts/m3-formal-r3-opus-model/runs/txg-cap-summary.txt:19`「m0 E1 j6_premise_holds_violations=70 atk_j6_writing_a_judged=328 atk_j6_writing_a_violations=0 atk_j6_writing_b_judged=398 atk_j6_writing_b_violations=0」；`research/prompts/m3-formal-r3-opus-model/runs/txg-cap-summary.txt:15`「core8 E1 j6_premise_holds_violations=70 atk_j6_writing_a_judged=328 atk_j6_writing_a_violations=0 atk_j6_writing_b_judged=398 atk_j6_writing_b_violations=70」；见证 `research/prompts/m3-formal-r3-opus-model/runs/txg-cap-summary.txt:22`「first=returned:txg18446744073709551615:watermark1001:applied1;second=panicked」；表第 406 行 `research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:65`「j2_violations=44 j3_violations=0 j4_violations=44 j6_premise_holds_violations=114」。

读法：
- 今天的 E165 J6 对今天的实现与表第 1003 行都报 70，分不开；两种写法都让今天的实现归 0。
- **写法 A 把表第 1003 行在 E1 上的缺陷排出判定**（那 70 态前提假），写法 B 判红 70。这个变体在 E2 上首调就 panic、J0 照样抓到，所以 A 丢的是 J6 这一判自己的判别力，不是整个变体。
- 表第 406 行在 E1 上多出的 44 态：第一次就落到了 txg 顶格的错版本，第二次报顶格。B 判「相等」、A 不判，J6 在两种写法下都不再红；它由 J2、J4 抓。实验页 Q14 记的「406、414 在 E1、N = 3 上多出 J6 前提真的违例」在两种写法下都会消失，回写时那一句要跟着改。
- 量过的四个实现上 B 的违例集合包含 A 的；推的：B 只比 A 多判「第一次落到顶格」的那几态，A 判的态 B 用同一条判，所以一般也包含。

## 三、T3：未定项清单还缺的（照 `.claude/kb/checks-owed.md` 的三格写，候选，没写进 kb）

| # | 拦什么 | 怎么拦会红 | 缺什么前置 |
|---|---|---|---|
| 1 | 施加出来的根，投影之外的四格与 S1 后半句不一致（今天卸载记号就不一致，1.1） | E165 生成器让卸载记号、分配记录树根随版本变（我的 `ATTACK_GEN` 两位），逐态判「其余字段等于所选根」；池上原型那一态进 harness 档 | D23 已定项 15 与 D22 已定项 7 先定施加出来的根 flags 取什么 |
| 2 | Verus spec 漏在飞上限与提交标记，Z2、表第 516、515 行在核上证得过（1.3） | `verus-mutations.tsv` 加三行，期望是 postcondition / invariant / assertion 错误 | S2 收严成再加两句；rlimit 定一个值 |
| 3 | 必红世界被 rlimit 或超时「杀」掉，算成 spec 抓到（1.8 b） | 必红表的期望改成认错误种类；样本：S2 再加两句那版上关掉反向链、rlimit 200 | 无 |
| 4 | spec 合取项被删、verified 数不变（1.8 a） | 每条合取项一个必红世界，表里写明它对的是哪一项 | S2 定稿 |
| 5 | S4 射程句在带故障的域上外推（1.5） | 射程登记里引 E165 合法历史一族里「记录完整、被在飞上限截掉」的态数（就是 Z2 与今天的实现 J4 不同的态） | 无 |
| 6 | 「D13 已定项 4 的崩溃状态」与 E165 C 域「合法态」口径混用：第二轮的 15 343 取的是乙切法，D16 已定项 7 的顺序下是甲切法 8 389（四、第 4 行） | 判决与 kb 引 C 域合法态数时写明切法；E165 的 `legal_crash_states` 行已经两种都报 | 无 |
| 7 | 择根与重放合起来没有判：S4 说「连同 D22 已定项 7 择根」，E165 C 域的所选根是装置自己按根槽位算的，不调 `choose_root` | C 域把根槽写成真根记录字节、调 `choose_root`，把择旧的变异放进第二段 | 根槽字节的生成器（E165 今天不写根槽） |
| 8 | E165 J6 不照 S6 选定的写法判，E1 70 恒为「违例」 | J6 照选定写法判（我的两组计数就是两种写法的可执行形式） | 用户先定 A / B |

第二轮记的四笔照旧欠着：冻结副本上 `crates/mutations.tsv` 里 `expected_key` 零命中，I-8.3 那一判（今天在核的第 434 行）仍没有变异行。

## 四、T4：第二轮的数在冻结副本上重做

装置：冻结副本原样编的 E165 二进制（不跑 `cargo test`），与打了 `patch_attack.py` 的那份（开关全 0 时与入库产物逐字节相同，见开头）。我自己的计数是照第二轮报告写的定义另写的，没拷第二轮的补丁脚本。

| # | 第二轮的数（撑哪一格） | 这一轮的读数 | 对不对得上 |
|---|---|---|---|
| 1 | 1 007 514：写行根进生成器后整条根不等（S1） | `ATTACK_GEN=1` 下 `atk_full_root_mismatch=1007514`（`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:126`「gen1 L atk_full_root_mismatch=1007514」）；原样装置 L 域 `research/prompts/m3-formal-r3-opus-model/runs/base-equivalence.txt:22`「applied_ge1=1007514」，即施加过的态全中 | 对上 |
| 2 | 264 499 / 164 809：开关 1、2 在 L 域的 J2（S2） | `research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:4`「core1 L atk_s1_violations=264499」；`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:18`「core2 L atk_s1_violations=164809」（同两行 J2 同数） | 对上 |
| 3 | 352 608：关掉点名验证那一臂 J2（S7、S1 的盘） | 原样装置：`research/prompts/m3-formal-r3-opus-model/runs/base-equivalence.txt:23`「j2_violations=352608」 | 对上 |
| 4 | 15 343：C、C6 的「D16 已定项 7 持久顺序下不带故障的崩溃状态」，开关 1、2 在上面 J1–J4 全 0（S2 的域） | 数对上：原样装置的乙切法合法态 C 14 737、C6 606（`research/prompts/m3-formal-r3-opus-model/runs/base-equivalence.txt:31`「n=6 cut_own=6167 cut_with=11443」等 11 行相加）。**口径对不上**：第二轮写「其中停在某一段的那 15 343 个（C 14 737、C6 606）是 D16 已定项 7 持久顺序下不带故障的崩溃状态」（`research/prompts/m3-formal-r2-opus-output.md:110`），而这个数是乙切法（根槽与记录同段）的；D16 已定项 7 记录之后隔一道屏障才写根槽，是甲切法，合法态 C 8 006、C6 383，共 8 389。辩方第二轮已写过「乙（根槽与记录同段）与条款的屏障位置矛盾」（`research/prompts/m3-formal-r2-sonnet-output.md:166`）。结论不变：开关 1、2 在甲、乙两种口径上 J1–J4 都是 0（`research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:9`「core1 C atk_s1_violations=4343」那一行有 `atk_legal_states=14737 atk_legal_own_states=8006`，没有任何 `atk_legal_j*`、`atk_legal_own_j*`——`summarize.py` 不带键时只列不为 0 的 `atk_` 量，即全 0） | 数对上，口径要改 |
| 5 | K10 下 0 / 0（A、AC 的 n = 3；S8） | `research/prompts/m3-formal-r3-opus-model/runs/k10-summary.txt:11`「atk_k10_states=523570 atk_k10_as_whole_table=0 atk_k10_as_gap_continues=0 atk_k10_as_gap_stops=2400」；AC 见 1.7 | 对上 |
| 6 | 57 585 320：A 域 n = 1…4 的态数（S8） | 原样装置 segment1 A 域：`research/prompts/m3-formal-r3-opus-model/runs/base-equivalence.txt:21`「states=57585320 a0_violations=0 failed_states=12435792 as_whole_table_violations=330240」；A 域 57 行与入库产物逐字节相同 | 对上 |
| 7 | 辩方 n ≤ 4 的逐态比较（译本上的；S8） | 在 Rust 上重做：读法不同的态上实现一律等于「缺号不停」——n = 3 全表看 / 缺号不停 1 024、缺号不停 / 缺号即停 2 400，n = 4 依次 174 592、331 200（`research/prompts/m3-formal-r3-opus-model/runs/k10-summary.txt:8`「atk_diff_wc_impl_whole=0 atk_diff_wc_impl_continues=174592 atk_diff_wc_impl_neither=0」）；K10 态 n = 3 523 570、n = 4 28 459 330，缺号即停在 K10 态上 2 400、233 600 | 对上（译本的数在 Rust 上逐个复现） |
| 8 | 单调：合法对集合上 J5b 全 0，开关 1 在「宽」集合 21 710（S5） | 见 1.6，原数逐个对上（含开关 3 的 11 423、开关 10 的 2 883、宽集合 24 583） | 对上 |
| 9 | Z1 在全部合法域 J1–J3 为 0（S4） | `research/prompts/m3-formal-r3-opus-model/runs/summary-all.txt:32`「core3 L atk_unmount_shape_states=14 states=4031078 j1_violations=0 j2_violations=0 j3_violations=0 j4_violations=1007514」，其余域同（R 域 J1 也是 0） | 对上 |

没重做的：第二轮判决「E165 第二段有 7 行变异只被 J4 那一族抓到」那个分类（要把 25 个变体逐个换进核、跑 segment2 的界，我只重做了其中 Z1 与「两条以上就不施加」的同形开关）；第二轮攻方的 `vm2_*` 替身（这一轮直接跑了 Verus，见 1.3）。

## 五、这条腿自己提的（只在我的模型上量过、被攻过零轮）

| 提的 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| S1 后半句加「卸载记号例外：施加过至少一次时为 0」，或 D23 已定项 15 补一句施加出来的根 flags 怎么取（二选一交用户） | 1.1 | 打中那一态量过（池上、E165 副本）；收严之后不中是推的（gen2 上 `atk_s1_other_fields_differ` 与 `atk_s1_marker_differs` 同数、另两格 0） |
| S2 补强的写法（`s2_strengthen.py`）：计数器 = 首条 + 下标、链首两支；不写 `requires` 升序 | 1.3 | 量过：原核 14 / 0，v01、v02、v12 红 |
| S2 再加两句：施加条数 ≤ `in_flight_limit`；整次发布末条带提交标记、不带提交标记的那条之后不换事务号（`s2_plus.py`） | 1.3、1.8 | 量过（rlimit 200）：原核 14 / 0，v04、v05、v06 红；v08 rlimit 200 红、1000 绿 |
| S4 射程句加域：「在不带故障的崩溃状态上」；带故障时在飞上限截断与 C519 那一格不排除 | 1.5 | 截断态数量过；改写本身推的 |
| 七·5：每条 spec 合取项一个必红世界；必红只认 postcondition、invariant、assertion 三类错误行 | 1.8 | 量过（三版 spec 的 13 个变体；v08 两个 rlimit） |
| S6 取写法 B | 2.2 | 量过（四个实现） |
| S9 每笔「条款欠一句」挂它等的那一轮 | 1.7 | 推的 |

## 六、没打中的形状

- **过 S1 投影加盘、却在合法历史一族上违反 J1、J2 或 J4 的恢复**：试了核的九个开关、拼根的三个开关。凡在合法历史一族上 J1、J2、J4 红的（开关 1、2、5、Z1、Z2、开关 6），S1 判同样红或由 M4 抓；拼根的三个开关 J1–J4 全 0，S1 后半句抓到实例表指针与分配记录树根那两个。
- **S2 补强之后仍过、而 E165 合法历史一族上 J1 或 J2 红的核变体**：13 个变体里没有；漏网的 Z2 红在 J3、J4，Z1 红在 J4（1.3 表）。第四格、反向链、点名验证三个变体没在 E165 副本上开，合法历史一族里第四格与反向链按构造走不到（推的）。
- **合法对上不单调**：六个实现的甲、乙合法对、同段对 J5b 全 0；推论第 4 步的「只差系统配置槽」那一格读代码排除。
- **S3 的反例**：没找到同键两份内容的合法写侧路径（推的）。
- **S7**：没找到第四处产品调用或传假的路径。
- **S8**：K10 态上三种读法与实现的关系照旧，到 n = 4（A）、n = 3（AC）。
- **版本序**：没造出 S5 域内两序不同的一对（推的，按条款）。
- 取样范围：E165 `--bounds light`（L 到 9、L6 到 6、L9 到 9、U 到 5、C 到 Nc = 6、C6 到 5、X 到 7、E1 到 4、E2 到 3、R 到 5、对抗一族到 3 个槽），对抗一族 A 另到 n = 4（segment1）；核开关 9 个、拼根 3 个、生成器 2 位，共 22 次跑（合法一族 15、light 对抗一族 6、segment1 对抗一族 A 域 1）；Verus 13 个变体 × 三版 spec；池上原型一个池一个崩溃状态。由用户决定的动作（卸载前写几个文件、写几次）没扫：原型只写了一个文件，卸载推几次空发布由盘数定（量到两次）。没扫这一维不影响 1.1 的结论：任何「卸载推了至少两次」的历史都有那一态（推的）。
- 一次抽样：照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，上面的「没打中」都不支撑「造不出来」。

## 七、这条腿自己的限度

- 数全在副本上。E165 的计数与 S1 判、两种 txg 顶格写法、K10 计数都是我加的，读同一个生成器的真相，与 E165 三条路径不独立；它们只说「在这个生成器上」。
- S2 的补强与再加两句是我写的 spec 与证明，不是第 4 批 4-1 正在做的那一份；再加两句那版要 rlimit 200，证明脆（v08 的 rlimit 那一格就是它）。spec 的写法本身被攻过零轮。
- 池上原型只量了卸载第二次空发布那一态，没走层 0 的枚举、没跑 checker；`rebuilt_equals_second_publish_root=false` 只打了相等与否，除卸载记号之外还有没有别的字段不等没逐个比。
- 「推的」那几处（S3、S9、版本序、Verus 只能留给胶水的那几样）没有实现也没跑。
- 包装的结局（命令都没跑，改了重起，没绕开包装）：第一次编原样装置退出码 252（同会话已有一条，设 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 重起）；编打补丁的那份退出码 252（整机放不下 12G，改成 3G）；第一次 `run-all.sh` 参数次序写错，22 次跑全部退出码 125 / 127，修了脚本重起，重起时退出码 251（峰值表记了那次几毫秒的 6M，设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`）；Verus 一次退出码 251（6G 超过峰值 251M 的两倍，改 500M）、一次 252（`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=0`，改成排队）；池上原型两次退出码 251（6G、2G 超过峰值 1001M 的两倍，改 2000M）。第一次跑原型 panic 在录制流的 C557 检查上（我没记卸载入口），改成 `record_entry` 记入口后过。
- 量的时候 `ps` 看到别的会话在跑 `cargo nextest run … -j 31`、几条 `cargo test -p singlefs-harness`、`broad-mutation one-build`，负载均值 44–78；没有 qemu、vm-bench、e152、fio。挂钟没进结论。

## 八、什么现象会推翻

- 1.1：池上崩在卸载第二次空发布的记录落了、根槽没落，`replay_journal` 施加出来的根卸载记号等于所选根的；或条款定下施加出来的根 flags 恒为 0（那时 S1 原句要收严，不是实现改）。
- 1.3：把 Z2 换进 S2 补强那版核，Verus 报 errors；或在飞上限那句写进 spec 之后原核在默认 rlimit 下证不过、证明要改结构。
- 1.5：E165 合法历史一族里出现一态，所选根之上记录完整的发布被在飞上限截掉、而它的根槽在那一态里读得出。
- 1.8：Verus 的 rlimit 错误不计进「M errors」；或 S2 补强之后某个 spec 合取项被删、verified 数跟着变。
- 2.1：合法写侧有一条不带故障的路让新实例的 txg 不大于环里已有的某一版。
- 2.2：某个变体在 E1 上被写法 A 判红而写法 B 判相等。
- 四、第 4 行：D16 已定项 7 的屏障位置改成根槽与记录同段。

## 九、没做什么

- 没跑 checker 档测试、层 0、门禁、重型测试、`cargo test -p singlefs-checker-tier`；E165 只直接跑二进制。池上原型是 harness 档测试，名字不带 `layer0`，只在我的仓副本里，不进库。
- 没读主仓 `crates/`、没读禁读清单里的文件；没改 kb、没改主仓任何代码，没做 git 写操作。
- 没在入库装置上重做任何一个数；副本上的数不算入库装置上的数。
- 不判辩方那几格（T2 的辩护、第二轮判决够不够得着），不替主 agent 采纳；第五节全部零轮。
- 第二段 25 个变体的分类没在副本上重做（四、末段）。
- 交回前删掉了自己建的编译目录 `/tmp/claude-1000/m3-formal-r3-opus/target`（403M）、`/tmp/claude-1000/m3-formal-r3-opus/target-base`（1.1G）与两份仓副本 `tree`、`tree-base`（各 11M）。草稿目录 `/tmp/claude-1000/m3-formal-r3-opus/` 留着各次跑的原样输出（`runs/`、`runs-attack/`）、Verus 变体的单文件 `lib.rs` 与日志（`verus*/`）、编译日志，都不是编译目录或仓副本；入库的只有模型目录里的脚本、原型与汇总。
