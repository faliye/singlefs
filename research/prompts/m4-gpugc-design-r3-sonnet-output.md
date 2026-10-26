# m4-gpugc-design-r3 辩方腿（Sonnet）报告（2026-10-24）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 L1 L5 C412 C906 N0 -->

立场：复核第二轮判决 `research/prompts/m4-gpugc-design-r2-main-verification.md` 的八格，替被判出局的一方（候选 A、B、D，以及被「挂着」的几处推荐）找站得住的理由，逐条判「站得住 / 够不着 / 同样打中别的臂」。本轮禁读的攻方第三轮报告与模型目录我没读。不起 GPU、不跑门禁、不跑 checker 档、不编译；全部数从 `research/results/` 下的产物与 `crates/` 主工作区现读现数。

## 复跑

模型全是读第 4 次重跑三份产物的 python（只读、不编译、不起 GPU），放在草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r3-sonnet/`（派发提示没给模型目录，所以没有进仓）。复跑命令：

```bash
bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r3-sonnet/run_all.sh <新的空输出目录>
```

开工快照核对（`research/prompts/m4-gpugc-design-r3-snapshot/sha256sums.txt`）：

```text
$ sha256sum -c research/prompts/m4-gpugc-design-r3-snapshot/sha256sums.txt 2>&1 | awk '{print $NF}' | sort | uniq -c
     23 OK
```

草稿目录里每个文件的 sha256（`out/` 是我这一次复跑的输出；这一条腿中途停下，续做的腿把草稿目录换成 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r3-sonnet-2/`，下面 13 个脚本与 `run_all.sh` 和 17 个输出的哈希在续做里重跑核过逐字相同，续做新增的脚本、输出与更新的复跑命令见文末附）：

```text
34dea9972cceed20c88c4ef750cad5e8f9635695a501806d48c8b8ed7db4cedc  arms.py
f1c33d9fe00a1e97fa2a94db730a81cdf5d16fb172b5edcdaa12e5f6f9fb0cb4  cnt.py
f5ac3d8e37e47ea43744f04ece396cee43eb193f6b832f6401f28c86fc20de7e  gap.py
3b08cb3687565b4fa2be172975e761c7cbc4ed398df910546e2d8d26be75e76c  j4.py
d3309997f660c125ee1068bea6d64aaa1cc59d95f22a7a9300c626cff0d53b82  maj.py
db14ee88a5d7f6d4f524bd3f2115f168934ace0c303e20b1377355be80bc53f0  pivot.py
3e6591bc3678cd4efe4231e21536552e12433cc31e9bedcefd8b72535fb84ff5  pts.py
5513a2096fb2454ad27451796ffc9c20c5cb4f6073b2a440c2a1567a815dbcc6  r5.py
9be0d0dfaab4e1b42c8c9a698edc41d350bba4cd539a221146c39e38d80a3a6c  rr.py
ee19f9b24b77f4e11a86c1bd44887a66c958f3d4546c0f3b3f60dac7192b2cc6  same.py
6852e184f9e2844c2f9e659bd37f12b3c5afc8021cc8219870b8be3651e49ec2  stage.py
b42ca4dbdbb96664b269ab5ec0970d44881138546a11ce84b77edbb815c096ba  tab.py
bead7136547c44f0b45d909d19322a6d3897b6f30bd222dafb5dab00c609bbd1  wall.py
8860cdcb7abe369f4316f773fb4190a3106c6a9d7e703a4e33640d434ea1caaa  run_all.sh
b76e4d65d2aa5ea28b09b96a4780ff5adefa113939f3607e6a612a25e643a490  arms_judgements.txt
4cee532de3eb353e1ce9ea0987aa18fe7c014fa22eb041a4f43cbfa119ded498  cnt_incl_excl.txt
55ed7a8debb771e0b4493aedbcde5fa6b5e9a8deca0fff7d629860305e2e92ee  cpu_stage_medians.txt
b547af89606827e1e8fd5b8a2d97a9e3f98d025e4a94e673c7c1513829682a0b  j4_points.txt
789b6a990aa35d2da8dc8c6333f8c262cfea66ea88fd76a65563e9461f555378  majority_reading.txt
eb322d5c99b9730bf0faa079330ff0a5638c96d80d606727eacbed32579504e5  pts_forms.txt
fcd958bea5dc9e44633dce1379bbe34a74a678cd04f104d91a824ea16d9f9c29  round5_over_round1.txt
7fa81adcc2fb3717de4e63243c3e922649ab538485971fad013af18429e349c5  round_robin.txt
1acdfd2ca6d7e5ea66cbd9a804cd41b69a0ca0096c3e565de78a5d5af4443b3c  same_crt.txt
ceae5726d45e96c436dd4b153114a860a155482f787e368cd3031ca164d9d5c5  tab_all_gaps_incl_today_kernel.txt
4920e707bce1f6e98d79737a068b408e7a3cb74ca90c053659317f92375863a5  tab_all_gaps.txt
01dc17ba0a88933e934d5871f788911a88392f4e69de4e1f7d5a9a8661b1beb5  tab_attack_gap50.txt
489d48b4d4f3b27c346283ff9dbe74bdc54c8377c65da918ab154d2925b98173  tab_gap0.txt
e9541faf83f8bcbbab382a7dc4feca3d57e51c4ca2d51f79963aaa736b9c80f8  tab_gap50.txt
1ae374f9a5a691163170efb64b66b542215799a688b06cb68fe79561f1ac254e  tab_r3crt_gap0.txt
82e9c7d84dbaf244eb27c853d1e933e4ddefbb401cc08b0239e1b91082b52240  tab_r3crt_gap50.txt
a910dbe33fd94f8630fbc0ba36c7eab775f7e2655f4152f283637b599b248e6c  wall.txt
```

## 判定一览（八格；每格的证据与推翻条件在后面各节）

| 判决条目 | 辩方判定 | 一句话依据 |
|---|---|---|
| G1-① 前半「A、B、D 两张卡都出局」 | **拆开**：A、D 在卡 1 上、在间隔 50 ms 上、在任一硬件 CRC 写法上站得住；卡 0、间隔 0、今天的软件 CRC 写法下**够不着**（A 不赔 44/72，D 不赔 50/72）；B **够不着**，且**同样打中 C** | 第 4 次重跑两条臂（受控臂、复现臂）与第 3 次重跑的卡 0 读数不一致；B 与 C 的每次提交主机 CPU 逐字相同（1040/1040），B 标「下界不赔」而 C 没标 |
| G1-① 后半「C 的赢依赖 CPU 模式今天的两处慢路径」 | 间隔 50 ms 下**站得住**（卡 0 两处都换、不计建索引：不赔 0/36）；间隔 0 下**翻了**（卡 0 不赔 55–63/72），剩下的赢落在丙（选段打分），不在那两处 | 阶段中位：换成指令 CRC 与按身份查之后，选段打分 370 µs 一条意图原样留在 CPU 一侧 |
| G1-③ 今天的形态下开 GPU 就进前台路径 | **站得住**；**同样打中** A–D 全部 | 每次提交挂钟 5.30–7.64 ms；但产品路径今天恒不起跑（`crates/singlefs-core/src/compaction_plan.rs:865`：「产品路径恒不起跑」），前台路径指的是测试开关强制的那一支 |
| G2 攒批形态 | 「跨批攒不成」**站得住**（更硬）；「卡 0 的拟合不可信」要**窄化**成「间隔 50 ms 的固定部分不可信」；只打中乙-2 与丙，**够不着**乙-1 | 受控臂与复现臂 `category=merge_across_intents` 0 行；卡 0 的拟合 9 条只 3 条不线性且全在间隔 50 ms |
| G3 多卡分活 | **打中**（第 4 次重跑复现）；**同样打中**一切用到卡 1 的分法 | 轮转把卡 0「不赔」的 101 格里 22 格翻成赔或不稳定；`knob=card` 984 对翻 756 对 |
| G4 ① | **站得住**，第 4 次重跑没碰 | 结构性（输入准备共用），不靠任何读数 |
| G4 ② | **打中**，但只是条款字面不全；不翻任何候选，**同样打中**两种模式 | `transaction.rs` 第 10138 行的逐条核计划与留存条目的 CRC 比对在发布处照做，与谁算的校验和无关 |
| G4 N | **没打中，也判不了**：零次不分辨 N 取几 | 单侧 95% 上界 1.428497e-6；第 4 次重跑没重量第二段 |
| G5 | **打中**，条件是 G8 选「挪出」；今天的代码里到不了 | 会话 `&mut self` 独占，切换落不进一批中间；停下状态在代码里不存在 |
| G6 | **站得住**（「fsync 在等」那半句）；三个默认值本身是条款自己留给 C412 与 E10 的 | D26 已定项 13 射程句；第 4 次重跑没碰 |
| G6 改法「挪出」 | **够着一半**：挪出后只在批与批之间看得见 | 同 G8 |
| G7 | **够不着**：没有实例，条款已把「同一个驱动与着色器编译器」列为知情的共同前提 | 攻方自己写明 crates 上构造不出 |
| G8 | **预算与跨趟两样不是挪出特有的**（凡不经用户改动起跑的放法都中，含调用线程里的定时兜底）；第三样（准入与抬 F 的次序）在单次 `&mut self` 调用下**到不了**，要先把 API 拆开 | `crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」；`mounted_session.rs` 第 1926、2704 行；核心层零线程原语 |

读这张表的三个口径提醒（正文第三节没写，会让同一个数读出两个结论），展开在下一节：① 第三节的透视把间隔 0 与 50 ms 并在一起数；② 第三节 C 的 120 格含 12 格「今天的写法」（`form=today_single_kernel`），登记写明它只报、不当第 1 行的读数（`research/prompts/e185-r3-prereg.md:300`：「不当第 1 行的读数」）；③ 「建索引计入」那几种写法让 CPU 模式比今天更慢，不是任何人会采用的替换。

## 一、关的第 1 条：八格逐格

### 0. 读法口径（先说清，后面每格都用）

第三节的透视取 `crt_source=r4_drift_control_controlled threshold_source=direct_from_the_r4_product threshold_kind=machine_scaled clock=thread`，我用同一取法（`out/tab_all_gaps_incl_today_kernel.txt` 与 `out/cnt_incl_excl.txt`）现数，第三节报的数逐个对得上（第四节的表）。三处读法在第三节里没分开：

1. **间隔 0 与 50 ms 并着数。** 登记写间隔 0 是「连发」、50 ms 是「产品里两批之间隔着一批的 CPU，推的」（`research/prompts/e185-r3-prereg.md` 第 55 行）。产品里两批之间的间隔至少是一批的 CPU，第三段量到 18–338 ms（第二轮攻方 `research/prompts/m4-gpugc-design-r2-opus-output.md` 第 319–323 行引的第三段第 149、1253 行），这一点我没有另量。r4 同样只量了 0 与 50 ms 两点，更长的间隔没量（岔路单第 3 节补行）。
2. **C 的 120 格含 12 格「今天的写法」（`form=today_single_kernel`）。** 同一个候选 A、B、D 都是 108 格。登记写明那一行只报：`research/prompts/e185-r3-prereg.md:300`：「不当第 1 行的读数」。去掉之后 C 的数是：写法 `today` 卡 0 不赔 101、不稳定 7（第三节 113/7），卡 1 不赔 51、赔 43、不稳定 14（第三节 59/47）；`both_three_way_without_build` 卡 0 不赔 55、赔 5、不稳定 48，卡 1 不赔 10、赔 91、不稳定 7。方向都不变，数不一样。
3. **「建索引计入」那几种写法让 CPU 模式比今天更慢。** `both_three_way_with_build` 在 P = 8192、f = 0.75、K = 8 的 C 门槛是 `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:3080`：「threshold_over_today=3.109185」；阶段中位（`out/cpu_stage_medians.txt`）里每次扫描现建索引 2004 µs，比今天的线性找 665 µs 还贵。它在这一点上把 C 的门槛抬高三倍多，是让 CPU 模式更慢的替换，不是任何人会采用的优化；用它得出的 C 全不赔（例 `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:101109`：「candidate=every_stage index=0 threshold_kind=machine_scaled clock=thread lines=120 not_losing=120」）只能当上界，不能拿来为 C 辩护。有意义的两端是 `today` 与 `*_without_build`（索引随版本增量维护）。

另外两条背景，决定了读哪条 c_rt：
- 第 4 次重跑两条臂在卡 0 上互相印证，与第 3 次重跑不印证。复现臂（不钉核，同第 3 次重跑的跑法）与受控臂（钉核、暖机）A 在间隔 0 的不赔格数分别是 64 与 44（共 72 格），D 是 68 与 50；第 3 次重跑自己的 c_rt 判出来 A、D 都是 0（`out/tab_r3crt_gap0.txt`）。复现臂的轮次比是 `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13795`：「quantity=reproduction_card_zero_round_ratio arm=reproduction index=0 value=1.056747 judgement=drift_not_reproduced」，第 3 次重跑同一量是 0.528（`out/round5_over_round1.txt`）。第 3 次重跑的卡 0 读数照旧没有解释（起跑前卡 0 上有别的计算进程，第二轮攻方第 156 行引过 `.out.err` 第 4 行）。
- 不稳定格数：第 3 次重跑卡 0 1564 行里 896 行，复现臂 1564 行里 208 行，受控臂 1704 行里 535 行。三个数的出处：`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:108396`：「name=unstable_count_r3 index=0 lines=1564 unstable=896」；`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13797`：「arm=reproduction index=0 lines=1564 unstable=208」；`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13798`：「arm=controlled index=0 lines=1704 unstable=535」。受控臂（钉核、暖机）比复现臂更不稳，不是更稳。

### G1　GPU 上算哪几段

**判决原文**（二·表 G1 行的判定列）：`research/prompts/m4-gpugc-design-r2-main-verification.md:32`：「候选 A、B、D 在登记读法下两张卡都出局；C 的赢依赖 CPU 模式今天的两处慢路径，那两处换掉之后符号没量过」。

**表一　候选 C 与 A、D 在不同读法下的不赔格数**（受控臂，`machine_scaled`、线程钟，去掉 `today_single_kernel`；括号是该格的总格数；全表在 `out/tab_gap0.txt`、`out/tab_gap50.txt`、`out/tab_all_gaps.txt`）：

| CPU 写法 | 卡 0 间隔 0：C ／ A ／ D | 卡 0 间隔 50 ms：C ／ A ／ D | 卡 1 间隔 0：C | 卡 1 间隔 50 ms：C |
|---|---|---|---|---|
| `today`（软件 CRC、线性找） | 72 ／ 44 ／ 50（72） | 29 ／ 0 ／ 0（36） | 51（72） | 0（36） |
| `hardware_crc_three_way` | 68 ／ 0 ／ 0 | 18 ／ 0 ／ 0 | 33 | 0 |
| `identity_index_without_build` | 71 ／ 44 ／ 50 | 17 ／ 0 ／ 0 | 24 | 0 |
| `both_single_without_build` | 63 ／ 0 ／ 0 | 0 ／ 0 ／ 0 | 12 | 0 |
| `both_three_way_without_build` | 55 ／ 0 ／ 0 | 0 ／ 0 ／ 0 | 10 | 0 |
| `both_three_way_with_build`（只当上界） | 72 ／ 0 ／ 0 | 36 ／ 0 ／ 0 | 58 | 18 |

**前半「A、B、D 两张卡都出局」。**
- A、D：在卡 1 上（任一写法）、在间隔 50 ms 上（两张卡、任一写法）、在任一硬件 CRC 写法上（两张卡、两种间隔），一格不赔（表一；`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:100773`：「candidate=scan_checksum_only index=0 threshold_kind=machine_scaled clock=thread lines=108 not_losing=44」，属 `today`，换成硬件 CRC 即 0）。**站得住**。卡 0、间隔 0、软件 CRC 且用 r4 的 c_rt：A 不赔 44/72、D 不赔 50/72，**够不着**；这一格的前提是 CPU 模式留在软件 CRC 上。
- B：**够不着，同样打中 C。** B 与 C 的每次提交主机 CPU 逐字相同（`out/same_crt.txt`：「B vs C groups 1040 identical 1040」），只差门槛；r4 的汇总行里 B 写「not_losing_lower_bound」、C 写「not_losing」，而登记自己写 C 也只到下界：`research/prompts/e185-r3-prereg.md:292`：「那一点 C 只到…下界不赔…乙、丙的输入准备没量」。两边的上传字节都只含单元字节，没含乙的映射表、丙的分配记录。判决把 B 判出局、C 留作「赢」，靠的是标签不同，不是 c_rt 不同。换成两处都换的写法后 B 与 C 的门槛只差 0.007 ms（P = 1024、f = 0.25、K = 8：`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:3428`：「candidate=every_stage saved_per_intent_milliseconds=0.397163 submissions_per_intent=2 threshold_milliseconds=0.198994」，对 `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:3430`：「candidate=scan_checksum_and_choice saved_per_intent_milliseconds=0.382632 submissions_per_intent=2 threshold_milliseconds=0.191713」）和 0.029 ms（P = 8192、f = 0.75：`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:3464`：「candidate=every_stage saved_per_intent_milliseconds=0.554113 submissions_per_intent=4 threshold_milliseconds=0.138816」，对 `research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:3466`：「candidate=scan_checksum_and_choice saved_per_intent_milliseconds=0.438851 submissions_per_intent=4 threshold_milliseconds=0.109941」）。

**后半「C 的赢依赖 CPU 模式今天的两处慢路径」。**
- 间隔 50 ms：**站得住。** 两处都换、不计建索引，卡 0 不赔 0/36（`out/tab_gap50.txt`：`both_three_way_without_build card0 … C[36]:L2,U34`），卡 1 也是 0/36。攻方第二轮在它模型上推的这一句，入库装置上的读数**撑住**（第二节）。
- 间隔 0：**翻了。** 只换 CRC，C 在卡 0 仍不赔 68/72；只换查找，71/72；两处都换、不计建索引，55–63/72。这时 C 与 B 一样靠的是丙：阶段中位里选段打分每条意图 370 µs（`out/cpu_stage_medians.txt` 的 `choose`；例 `research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out:1778`：「choose_cpu_nanoseconds=384982.000000」），两处替换都不碰它。查找一项在 P = 1024 上本来就小：线性找 106–110 µs、按身份查 89 µs（f = 0.25）。所以第三节那句「卡 0 仍多半不赔」是间隔 0 与 50 ms 并着数的结果；拆开之后，它在间隔 0 上成立、在间隔 50 ms 上不成立，而间隔 0 是登记写的「连发」。
- 第三节的解释（「这一节第二条」）因此**不能直接拿来反驳攻方那一句**：攻方说的是间隔 50 ms。

**这一句判决够不够得着、是不是同样打中替代方案：** 前半对 A、D 是「条件成立」，对 B 够不着；后半在间隔 50 ms 上打中 C（同样打中 B，因为两者只差一个丙以外的 0.007–0.029 ms），在间隔 0 上够不着。结论不翻成「GPU 模式赢」：C 的不赔仍是下界（乙、丙输入没量），卡 1 的间隔 50 ms 在除「建索引计入」外的每一种写法下一格都不赔。

**什么现象会推翻我这一段：** 在入库装置上补量乙的映射表与丙的分配记录的展平 CPU 与上传，B、C 在间隔 0 的卡 0 不赔格数掉到接近 0（那就是前半对 B 的判决也成立）；或补量间隔 100、300、1000 ms 之后卡 0 的 C 在 `both_*_without_build` 下回到有不赔格（那就是间隔 50 ms 那一点是端点假象，攻方那一句也被推翻）。

### G1-③　今天的形态下开 GPU 就进前台路径

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:32`：「G1-③（今天的形态下开 GPU 就进前台路径）」。

**站得住，且同样打中 A、B、C、D 全部**——它和哪个候选算哪几段无关，只看 GPU 调用发生在哪条线程。
- 代码：用户改动发成之后，返回之前起一趟整理：`crates/singlefs-core/src/mounted_session.rs:1927`：「if compaction_starts_after_a_user_change(allocator.compaction().switches.run_forcing) {」。
- r4 的每次提交挂钟仍是这个量级：受控臂与复现臂，提交单元数 ≤ 24、共用输入与分开输入两种形态，单次 5.30–7.64 ms（`out/wall.txt`：卡 0 最小 5.303、最大 6.672，卡 1 最小 5.456、最大 7.637），第 3 次重跑同口径 5.32–7.74 ms。D24 已定项 2 第 2 条写「PCIe 往返是几十微秒量级」（`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`：「PCIe 往返是几十微秒量级」），实测差两个数量级；主因是核逐位串行（G2 一节），不是 PCIe。

**但前台路径这个前提本身有一个范围要写明**：产品路径今天恒不起跑整理。`crates/singlefs-core/src/compaction_plan.rs:865`：「第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制」；`crates/singlefs-core/src/mounted_session.rs:1952`：「批大小条款没写，产品路径在签任何意图之前停下」。所以「前台路径」指的是测试开关强制的那一支；产品要不要有这条前台路径，是七·3 接水位时才定的事。这对 A–D 一视同仁，不改变判决，但写进 kb 时要带这句范围，否则读者会以为今天的产品里 GC 已经在前台跑。

**推翻条件**：七·3 接水位时把整理的起跑放到调用线程之外，G1-③ 就变成 G8 的选项，不再是现状描述；或实测某个核把单次提交压到毫秒以下，「进前台」的代价就从 5–8 ms 掉到可接受，判决剩下的只有字面（D24 已定项 2 第 2 条写的是「不许」，不是「代价高」）。

### G2　攒批形态

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:33`：「跨批攒成不了，第一段读数里跨意图只 1 格；拟合在卡 0 上不可信，卡 1 上可用」。

两半分开判。

**「跨批攒成不了」站得住，r4 把它加硬。** 受控臂与复现臂共 648 行 `merging_shape`，`category=merge_across_intents` 0 行（第 3 次重跑是 432 行里 1 行）：命令与输出在第四节。卡 0 的 `merging_shape` 全是 `one_submission_per_batch` 或 `unstable`，没有 `one_submission_per_intent`。这是因为 r4 卡 0 的固定部分只有 0.033–0.090 ms（间隔 0；`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:9973`：「fixed_milliseconds=0.038806 per_mebibyte_milliseconds=0.042704」），没有什么可摊的。

**「拟合在卡 0 上不可信」要窄化，不能原样保留。** 第 3 次重跑卡 0 的 9 条拟合 7 条 `fit_not_linear=true`；r4 受控臂卡 0 的 9 条只有 3 条，复现臂卡 0 的 9 条也是 3 条，**这 6 条全在间隔 50 ms**，间隔 0 的 6 条全线性（命令在第四节）。受控臂卡 1 的 9 条有 1 条（间隔 50 ms、阻塞等）。所以：
- 间隔 0 的拟合在两张卡上都可用，卡 0 固定部分 0.033–0.090 ms、每 MiB 0.042–0.095 ms，卡 1 固定部分 0.025–0.068 ms、每 MiB 0.484–0.972 ms（同一产物第 9973–9990 行的 `fixed_milliseconds`、`per_mebibyte_milliseconds`）。
- 间隔 50 ms 的固定部分不可信，**两张卡、两条臂一致**，而且不是漂移：每轮的固定部分不单调。`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:9974`：「fit_not_linear=true fixed_milliseconds_per_round=0.264609,0.111201,0.044824,0.235334,0.258486」，复现臂 `:2959`：「fit_not_linear=true fixed_milliseconds_per_round=0.284952,0.314035,0.113775,0.211240,0.243865」。第 3 次重跑攻方把拟合不线性归到卡 0 的轮次漂移上，这一点 r4 **不支持**：复现臂没复现漂移（1.056747），拟合不线性却照样在。每 MiB 一项则稳（间隔 50 ms 卡 1 的每轮 1.446650–1.453052 ms）。

**同样打中别的臂吗：** 打中乙-2（跨意图预取）与丙（放宽 R），**够不着乙-1（同一条意图内一次交）**：受控臂卡 1 间隔 50 ms 有 54 行 `one_submission_per_intent`，间隔 0 有 26 行（第四节）。乙-2 与丙另被三条代码约束挡着，那三条 r4 不碰：每批之前都重扫（`crates/singlefs-core/src/mounted_session.rs:2975`：「每批之前都重扫」）、一趟发布预算（`crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」）、放宽 R 要改 `crates/singlefs-core/src/logged_operations.rs:303` 拒收的槽数字段。乙-1 是否真能用，看岔路单第 4 行（窗口开头验一次字节）先定。

**推翻条件**：卡 0 或卡 1 在间隔 ≥ 100 ms 上量到跨意图类别 ≥ 1 行（那「跨批攒不成」就只是间隔 0 与 50 ms 两点的结论）；或间隔 50 ms 的拟合在重复量下线性（那固定部分不可信就是这一次的噪声）。

### G3　多卡分活

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:34`：「按窗口轮转让 C 有 4 格由不赔翻成赔、23 格翻成不稳定」。

**打中，第 4 次重跑复现。** 用 `drift_control_point` 的受控臂行，同一格取两张卡每轮 c_rt 的平均（轮转的分法，与攻方同一个假设），门槛两张卡相同（`out/round_robin.txt`，去掉 `today_single_kernel`）：候选 C 卡 0 单用不赔的 101 格里，轮转后 11 格赔、11 格不稳定、79 格仍不赔（52 + 14 + 13）；其中 14 格卡 1 单看是赔、平均之后不赔。第 3 次重跑是 132 格里 4 + 23。分母不同，方向与量级相同。两张卡判定不同这一项，r4 也更强：`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:11179`：「knob=card clause=two_cards_judge_differently pairs=984 flipped=756」（第 3 次重跑是 1096 对翻 630 对）。卡间每 MiB 差 11–33 倍：同一产物 `:9973`（卡 0 间隔 0 每 MiB 0.042704）对 `:9982`（卡 1 同格 0.483803），`:9974`（卡 0 间隔 50 ms 0.043744）对 `:9983`（卡 1 同格 1.449579）；卡间链路 `:131` 的「link_peak_bytes_per_second=63015384615」对 `:3966` 的「link_peak_bytes_per_second=15753846154」是一个可能的原因，这一份产物分不开。

**同样打中别的臂：** 凡用到卡 1 的分法都中，不独属于「按窗口轮转」。绕得开的只有两类：只用卡 0，或挂载时在每张卡上量 c_rt、赔的卡不用。前者与 D26 已定项 13 第 2 条字面冲突（`.claude/kb/decisions/26-后台整理与放置回收.md:246`：「不给就用本机全部显卡」），后者要产品里有「挂载时量 c_rt」这一步，今天没有，是判别子运行时观测不到的那一形。「已验厂商独显」「PCI 总线号定次序」不绕开它（卡 1 也是 NVIDIA、也是独显）。

**没碰：** 本机第三张卡（AMD 集显）。`cards_started=2`，r4 一次提交也没在它上面起过；`lspci` 仍是三张：`79:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Device 13c0 (rev c1)`（第四节命令）。

**推翻条件**：卡 1 在间隔 50 ms 上换一条拷贝路径后 C 有不赔格（那卡间差异是驱动路径，不是卡的性质，轮转的代价掉下来）；或产品的分活不是等权轮转而按每张卡的 c_rt 加权（那 22 格翻面就是等权假设的产物）。

### G4　两份核比对与 N

**① 站得住，第 4 次重跑没碰。** 判决：`research/prompts/m4-gpugc-design-r2-main-verification.md:35`：「① 打穿（第一轮中，第二轮辩方复核站得住）」。依据是结构，不靠任何读数：七·3 那条同输入对拍把同一份准备好的输入喂给两种模式，输入表准备出错时两边一起错、对拍抓不到；两种模式各读这一版再比才抓得到。条款自己把这一类列成共同前提：`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「两份共用的那几样（同一个驱动与着色器编译器、同一份输入表的准备、对规则的同一种误读）出错时两份照样一致」。**同样打中**所有两核共用输入准备的方案（岔路单第 5 行的甲），**够不着**乙（输入也各备一份）与丙（各一份且分在两张卡）。推翻它的现象：七·3 的对拍改成从这一版起算之后，在带输入准备缺陷的实现上仍是绿（那就是改法不够，不是 ① 不成立）。

**② 打中，但只是条款字面不全，不翻任何候选，两种模式同样中。** 条款：`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「交 CPU 改位图（CPU 只改位图，不再抽样复核）」。代码：发布处对计划里每一项照做结构性的逐条核——键在不在这一版、是不是钉住的角色、源落点是不是映射位置且跨度合它的类，留存的映射条目另在 CPU 上读盘比校验和：`crates/singlefs-core/src/transaction.rs:10133`：「字节从盘上按映射位置读、核位置项里的校验和，两份取先核得上的那一份」，`crates/singlefs-core/src/transaction.rs:10250`：「.filter(|copy| crc32_castagnoli(copy) == location.unit_checksum)」；调用点 `crates/singlefs-core/src/transaction.rs:9924`：「relocations.push(relocation_checked_against_the_version(」。这一遍与谁算的校验和无关，GPU 模式与 CPU 模式都走，所以「CPU 只改位图」不是 CPU 在发布处做的全部；它也不是对 GPU 结果的复核（不抽样、不重算 GPU 判过的校验和），不与「不再抽样复核」冲突。要补的是条款一句话写明发布处的计划核与留存条目的校验和仍在 CPU 上；留存条目的这一遍校验和在 r4 的三阶段（`crc`、`scan_increment`、`release_check_crc`）里不在，两种模式都付，不进候选的门槛差。**推翻条件**：找到一个 GPU 确认过的条目在发布处被这一遍核拒掉（那说明 GPU 的「确认」不足以让 CPU 只改位图）。

**N 与计数住哪：没打中，也判不了。** 第 4 次重跑没重量第二段；第二段的数（零次不一致，单侧 95% 上界 1.428497e-6）只罩两份 CRC 核（`research/prompts/m4-gpugc-design-r2-opus-output.md:640-642` 写明罩到与罩不到的）。算术（第四节）：1 TiB 一遍期望的不确定条目可多到 47.93 条（N = 1 的上界）、N = 2 时 6.8e-5 条、N = 3 时 9.8e-11 条，**但这是随机不一致的算法**；确定性不一致（同一条目每轮都不一致）的 N 个轮次是白付的，N 越大越贵，第二段 `determinism_replay` 没出现它也不能排除它。所以零次读数不分辨 N 取 1 还是 2：这是用户岔路，不是数据能翻的；见第三节岔路单的 N 行。计数住内存：重挂清零，第二轮攻方的「重挂后第一轮 N 取 1」是它的实现读法（零轮）。

### G5　控制接口

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:36`：「先请求切到 cpu、这一批 GPU 随后出错、GC 停下」。

**打中，但条件是 G8 选「挪出」；在今天的代码里到不了。** 条款两句：`.claude/kb/decisions/26-后台整理与放置回收.md:245`：「从下一批开始生效；一批之内不换」，`.claude/kb/decisions/26-后台整理与放置回收.md:248`：「恢复要人切回 `cpu` 或修好 GPU 再切」。S、E、B 这一序列让「切回 cpu」发生在出错之前、GC 停着、没有条款说停着时再收到一次 cpu 切换算不算恢复。这是条款字面的缺口，补一句（停着时收到切换请求就解除停下）即可，与第 1 条的「从下一批开始生效」不冲突（解除之后的起跑就是下一批）。
- 在今天的代码里到不了：会话方法是 `&mut self`（`crates/singlefs-core/src/mounted_session.rs:1956`：「pub fn run_compaction<Device: BlockDevice>(」；`crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」），切换请求落不进一批中间；「GC 停下」也不是持久状态：失败那一支只把这一趟交回（`crates/singlefs-core/src/mounted_session.rs:2785`：「CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),」）、记一笔观测（`crates/singlefs-core/src/mounted_session.rs:2632`：「compaction_of_the_run.observations.last_stop = Some(*stop);」），下一次越线照样起。
- **同样打中**一切「进门读一次开关、出错后停下」的接口，不独属于正推的会话方法；绕得开的是「停着时切换请求当场解除」，它不违第 1 条（一批之内不换）。
- r4 没碰（纯条款与代码）。**推翻条件**：G8 定成「留在调用线程」且停下态不进代码——此时 S、E、B 不可达，G5 退回「没打中」。

### G6　三个默认值

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:37`：「那一条空真」。

**站得住，且只对一件事成立。** 被打穿的是「前台有 fsync 在等」这个判别子：整理在 `publish_user_change(&mut self,…)`（`crates/singlefs-core/src/mounted_session.rs:1849`）这一次调用里同步跑，没有并发的 fsync 可等；这是 D26 已定项 13 第 6 条的字面（`.claude/kb/decisions/26-后台整理与放置回收.md:250`：「前台有 fsync 在等时不发下一批」）在今天的形态下观测不到。r4 没碰。

**判决没打到的：** 三个默认值本身。条款射程自己把两个留给没量的实验：`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「定时兜底的间隔、写带宽上限的默认值（等 C412（整理的挂钟代价没量过） 与 E10（老化：结构抗老化 vs 事后整理））」；C412 的欠账行今天仍写「前台抢 I/O 与产品默认路径的挂钟仍没有数」（`.claude/kb/checks-owed.md:291`）。显存上限不给时怎么办，条款自己定了（`.claude/kb/decisions/26-后台整理与放置回收.md:247`：「不给就不设上限，后台任务跑的时候卡上能用多少用多少」）。所以「打穿」应读成「不能靠这个判别子给兜底间隔与写带宽上限定默认」，不是「这三个值错了」。

**改法「挪出调用线程」够着一半，且同样打中 CPU 模式。** 第二轮攻方的话：挪出之后判别子看得到，但只在批与批之间看得见。批与批之间才看是第 6 条自己的字面（「不发下一批」），CPU 模式一样；GPU 模式多出来的只有一次飞着的提交，r4 单次 5.30–7.64 ms（`out/wall.txt`），比一批的 CPU（第二轮攻方引的第三段 18–338 ms）小一个数量级。**推翻条件**：量到前台 fsync 的延迟分布在「批间才看」下超过用户可接受线（岔路单第 2、9 行的 Q17 口径），那默认值要由这个分布定，不由 C412 一个数定。

### G7　GPU 代码住哪

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:38`：「checker 档把 `wgpu` 钉成 `=30.0.1`，产品与 checker 档只能共用 naga 与驱动这条翻译链」。

**够不着（推的，没有实例）。** 事实先对上：`crates/singlefs-checker-tier/Cargo.toml:27`：「wgpu = { version = "=30.0.1", default-features = false, features = ["vulkan", "wgsl", "std"], optional = true }」，`./Cargo.lock:452`：「name = "naga"」下一行 version 是 30.0.1。够不着的理由三条：
1. 条款自己知情：`.claude/kb/decisions/26-后台整理与放置回收.md:254`：「GPU 路径默认写成 WGSL、经 `wgpu` 起（checker 档的崩溃判器已经这样用」；第 256 行把「同一个驱动与着色器编译器」列为两份核的共同前提，路由给 GPU 与 CPU 两份实现的对拍和崩溃点重放。共用翻译链是这条路径设计时就接受的残余，不是新发现的矛盾。
2. 攻方自己写明 crates 上构造不出实例（`research/prompts/m4-gpugc-design-r2-opus-output.md:669` 一段）；要出事需要同时满足「naga 把某种写法译错」与「对拍的小输入没覆盖那种形状」，两个条件当前都没有证据。
3. **同样打中所有替代方案**：换版本、另开工作区、直写 SPIR-V 都消不掉 Vulkan 驱动这一层；只能分散 naga 这一段。

**同类先例（找得到）：** 工作区里已有一份单独锁版本的 GPU 栈：工作区根 `./Cargo.toml:15`：「exclude = ["scripts/experiment-apparatus"]」，`crates/singlefs-checker-tier/Cargo.toml` 的注释写明 wgpu 版本「钉成 E163（GPU多卡算单元校验和） 装置用的那一份（scripts/experiment-apparatus/Cargo.toml）」。所以「产品的 GPU 包另开工作区、自带锁」有先例，代价是两份锁；是否要，是 G7 剩下的唯一岔路。

**第二轮辩方提的「所有门槛按软件 CRC 算」：** r4 已经量了（本报告 G1 一节）：它是真的，且量级大——CRC 换成指令后 A、D 在所有格上全赔。这一点 G7 不再需要，并进 G1。

**推翻条件**：在 naga 30.0.1 上构造出一个核形状，GPU 两份核一致地译错、而 GPU 与 CPU 对拍的小输入没覆盖（那是实例，G7 即刻打中）。

### G8　GC 挪不挪出调用线程

**判决原文**：`research/prompts/m4-gpugc-design-r2-main-verification.md:39`：「两边都有代价：不挪出违 D24 已定项 2 第 2 条，挪出要补三样保证。挪出的主机 CPU 代价（c_rt 不因挪线程而变，辩方）没有数」。

先交代一件现场变化：开工快照里 `crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-checker-tier/Cargo.toml` 三份，我这次复核时 `sha256sum -c` 对不上（别的会话在改，命令在第四节），其中 `compaction_plan.rs` 的行号整体后移（`.max(1)` 从第二轮报的第 815 行到 `crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」）。本报告里凡 `crates/` 的行号都是这一次现查的，前一条腿写的两处旧行号我已改过。

**三样保证逐样判**（挪出之后要补的，第二轮攻方 `research/prompts/m4-gpugc-design-r2-opus-output.md` G8-①②③）：

1. **发布预算 `.max(1)`：打中，但不是挪出特有，凡起跑不经 `publish_user_change` 的放法都中。**
   - 式子与归零：预算 `3S − 4 − 已过去的发布数`，至少 1（`crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」）；已过去的发布数只在用户改动里清零（`crates/singlefs-core/src/mounted_session.rs:1926`：「compaction.publishes_since_the_last_user_change = 0;」），每次整理发布加一（`crates/singlefs-core/src/mounted_session.rs:2736`：「compaction.publishes_since_the_last_user_change += 1;」）。预算要护的东西写在 `crates/singlefs-core/src/compaction_plan.rs:852`：「/// 一次整理运行的发布预算（判决 G8 `B_gc`，正推 O11：整理的发布不改用户可见状态，连发会把回退候选集挤成 1 个状态）：」。
   - 今天就有一个不经用户改动的入口：`crates/singlefs-core/src/mounted_session.rs:1956`：「pub fn run_compaction<Device: BlockDevice>(」，连着调它两次，第二次预算至少还是 1。它今天只有测试与装置在调（`crates/singlefs-harness/src/history.rs:6452`：「let run_result = mounted_session.run_compaction(devices);」），产品里没有调用者，所以洞没露出来。
   - 辩方的点：这个洞的成因是「预算只在用户改动时归零，起跑却可以不经用户改动」，与跑在哪条线程无关。任何让整理在两次用户改动之间再起的放法（线程里的定时器、调用方的周期调用）都中；反过来，不经用户改动的起跑方式如果只在 `publish_user_change` 里顺带查间隔，预算已经在 1926 行归零，洞补上了。所以「挪出要补这一样」应读成「定时兜底要补这一样」，而定时兜底是 D26 已定项 13 第 5 条要的（`.claude/kb/decisions/26-后台整理与放置回收.md:249`：「5. **什么时候起跑**：每次发布之后按已定项 1 的三水位查一次；另有定时兜底，每隔一段时间再查一次，间隔是挂载参数。两种模式起跑规则相同。」）。
   - 这里露出一处条款与代码的张力，不论挪不挪都在：预算的用意是整理发布不连发，定时兜底的用意是闲置时也要继续推。两者放在一起，兜底触发的那一趟该不该有 `.max(1)`、该不该与用户改动共用一个预算，条款没写。登记成第三节的补行。
   - 推翻条件：找到一条今天的形态（只经 `publish_user_change` 起跑）下连着整理发布超过 `3S − 4` 次的序列（那就是预算在今天也护不住，不是挪出才有）；或者找到不需要定时兜底、却必须让整理在两次用户改动之间再起的需求。

2. **「不增全空段就不签下一条」是一趟里的局部变量：打中，但今天就护不住跨趟，挪出只是去掉了用户改动这道速度闸。**
   - 变量在一趟里新建、一趟里消费：`crates/singlefs-core/src/mounted_session.rs:2704`：「let mut last_completed_intent: Option<CompactionIntent> = None;」，判它的在 `crates/singlefs-core/src/mounted_session.rs:3044`：「if let Some(completed) = *last_completed_intent {」。停下的原因只记成一笔观测，起跑时不读（`last_stop` 全仓只有一处赋值，命令在第四节：`crates/singlefs-core/src/mounted_session.rs:2632`：「compaction_of_the_run.observations.last_stop = Some(*stop);」）。
   - 第二轮攻方自己的模型就显示今天的形态也护不住：`research/prompts/m4-gpugc-design-r2-opus-output.md:687`：「name=g8_loop batches_per_intent=2 policy=today user_change_every=1 ticks=200 publishes=500 max_consecutive_gc_publishes=2 budget_line=20 root_ring_slots=24 over_budget_line=false intents_issued_after_an_intent_without_yield=99」。用户改动每拍一次时，200 拍里签了 99 条跟在不增全空段的意图后面的新意图；今天的速度被用户改动的速度管着，挪出之后管它的东西没了。
   - 辩方的点：「保证丢了」不准确，今天本来就没有；准确的说法是「今天的上限靠用户改动的节拍，挪出后需要把它存进整理状态」。补法是把上一条完成的意图的全空段数存进会话的整理状态（第二轮攻方自己提的，零轮）。
   - 推翻条件：能在今天的形态下造出「用户改动密集、意图连签、全空段不增」被某处挡住的序列（那就是我没找到的跨趟记忆）。

3. **整理批不过准入与抬 F 的次序：在单次 `&mut self` 调用下到不了；条款与代码对不上（C906）与挪不挪出无关。**
   - 抬 F 再判整段在同一次调用里：`crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」带 `&mut self`，里面调 `crates/singlefs-core/src/mounted_session.rs:2024`：「fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(」，发成之后才起整理（第 1926 行归零、第 1927 行判起不起跑）。整个过程持着会话的独占借用，后台线程拿不到会话，不论是 `Mutex` 还是通道。
   - 第二轮攻方的序列（「用户写 → 准入拒 → 推抬 F → 后台整理这时拿到会话」）要求会话在抬 F 与再判之间被放开，这只有把用户改动这一条路本身拆成几步才成立。「挪出」的两种做法（占着会话等 GPU；等 GPU 时放开会话）都不要求拆它，所以这一样不是挪出的必然代价，是「把 API 拆开」的代价。
   - 与挪出无关的那一半是真的：整理批的发布不判准入，注释至今写着搬迁副本「不进需求，所以两判都不判」（`crates/singlefs-core/src/transaction.rs:9667`：「/// （固定点，空间归 checkpoint 保留池），搬迁副本不进重写的角色、不进需求，所以两判都不判；合取 7 免（带搬迁的发布不是前台写入，」），而条款要求计入（`.claude/kb/decisions/26-后台整理与放置回收.md:100`：「2. **整理不是释放路径，它走常规准入**，批次最坏占用计入「已承诺预留」。」）。C906 仍开着（`.claude/kb/checks-owed.md:585`），第二轮判决已把它标成与 GPU 无关。
   - 核心层确实没有线程原语：`crates/singlefs-core/src` 下 `std::thread`、`Mutex`、`RwLock`、`mpsc`、`Condvar` 一处没有，唯一的 `AtomicU64` 在 `#[cfg(test)]` 里（`crates/singlefs-core/src/block_device.rs:1019`：「use std::sync::atomic::{AtomicU64, Ordering};」，上一个 `cfg(test)` 在第 1016 行）。挪出要在核心层之外新增线程与所有权层，这是代价，不属于三样保证。
   - 推翻条件：某个「挪出」做法不拆 `publish_user_change` 却能让整理插进抬 F 与再判之间（那就是我没看到的共享路径）。

**「放开会话」那一种（第二轮攻方 G8-④）**：陈旧的计划在发布处被逐条核挡掉，代价是白花一次 GPU 往返与重扫，不是正确性。拒绝点在 `crates/singlefs-core/src/transaction.rs:10225`：「RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition {」，这一趟交回的失败支在 `crates/singlefs-core/src/mounted_session.rs:2785`：「CompactionRunOutcome::Failed(CompactionRunFailure::Batch(refused)),」。

**第 4 次重跑碰到的只有一样**：前台挂钟。受控臂与复现臂单次提交 5.30–7.64 ms（`out/wall.txt`，单元 ≤ 24；命令在第四节），一次越线后的用户改动至多两次提交（签发加一批），即 10.6–15.3 ms（算术：2 × 5.303、2 × 7.637），与第二轮辩方报的 5.50–14.76 ms 同一个量级，没有被推翻。这笔账在今天的产品里还没有人付：`crates/singlefs-core/src/compaction_plan.rs:865`：「第一版三条水位恒 0（关），产品路径恒不起跑；只供测试的开关强制」。

**这一格我的判定**：两边都有代价这句话**站得住**；「挪出要补三样」要改写成「预算与跨趟两样是任何让整理脱离用户改动起跑的放法都要补的（含定时兜底），第三样要先拆 API 才会碰到」。**同样打中**不挪出的做法里「想要定时兜底」的那一支。挪出的主机 CPU 增量没有数这一句**站得住**，第 4 次重跑没碰。

## 二、关的第 2 条：第二轮攻方报告的反例与推的数，逐条对第 4 次重跑

对象：`research/prompts/m4-gpugc-design-r2-opus-output.md`。判法：第 4 次重跑（受控臂、复现臂、写法段、重判）**推翻**它的数或归因、**撑住**、还是**没碰**（没重量，或它本来就是代码与条款字面）。凡写「撑住（更硬）」都是数比它报的更偏向它那一边。「第 3 次」指它读的第 3 次重跑第一段；第 4 次重跑的数是我从产物现数的（命令在第四节）。它报的 15 个模型我在原模型目录上复跑，15 行 `identical=true`（第四节）。

先交代两条贯穿的事实，后面不再重复：
- 第 4 次重跑里，**复现臂**（不钉核，同第 3 次的跑法）和**受控臂**（钉核、暖机）在卡 0 上互相印证，与第 3 次的卡 0 读数不印证。第二轮攻方的好几个数（A、D 在卡 0 的「出局」、C 在间隔 0 的「几乎全赔」）是在第 3 次的卡 0 读数上得的，换成第 4 次的读数就变。`research/prompts/m4-gpugc-measure-r4-forks.md` 第 2 行已定后续用受控臂的读数。
- 第 3 次起跑前卡 0 上有别的计算进程：`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err:4`：「卡 0 上有别的计算进程 [76757]，等它（第 0 次看）」；第 4 次控漂移段的 `.err` 里同样的话 0 处（命令在第四节），登记失败条款 L5 触发（这一天复现不出）。是不是这一条造成第 3 次的卡 0 读数，这份产物分不开（推的）。

| 攻方条目（报告行） | 第 4 次重跑的读数 | 判 |
|---|---|---|
| 改法一～四（104、132、142、152）：第三张卡、对拍分块、重挂交 CPU、已知答案自检 | `cards_started=2`，AMD 集显一次提交也没有；其余是条款字面与推的序列。`lspci` 仍三张（第四节） | **没碰** |
| 二.1 口径（167）：「14–38 格」含今天的写法 | 第三节 C 卡 0 的 120 格里 12 格是 `form=today_single_kernel`：`today` 写法 120 格不赔 113、去掉后 108 格不赔 101（`out/cnt_incl_excl.txt`） | **撑住** |
| 二.2 C 的不赔是下界、与 B 同义（201） | A、C 每次提交主机 CPU 1040 组逐字相同，B、C 同样 1040 组（`out/same_crt.txt`）；重判汇总里 B 写 `not_losing_lower_bound`、C 写 `not_losing`；第 4 次登记 `research/prompts/e185-r4-prereg.md:53` 仍只给 B 标下界 | **撑住（更硬）** |
| 二.2 C 的乙、丙输入没进 c_rt、补上后 C 在卡 1 翻面（c_inputs，推的布局） | 写法段的阶段行没有映射表与分配记录的字段（`grep -ciE 'mapping_table|allocation_record|upload_bytes'` 0 处），没量 | **没碰** |
| 二.3 读法：多数轮、中位读下 A、D 在卡 0 有格不赔（237） | 受控臂 `today`、两种门槛、去掉今天的核：多数轮读 A 卡 0 不赔 140/216、D 156/216（攻方 12、24），卡 1 A、D 仍 0；**逐字全轮一致的读法下 A 卡 0 就有 44 格、D 50 格不赔**（`out/cnt_incl_excl.txt`）。「A、B、D 没有一格不赔」不需要换读法就不成立 | **方向撑住（读法决定结论），数被推翻** |
| 二.4 卡 0 五轮随轮次往下走，第 5 轮 ÷ 第 1 轮中位 0.643（270） | `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13795`：「quantity=reproduction_card_zero_round_ratio arm=reproduction index=0 value=1.056747 judgement=drift_not_reproduced」；受控臂 `:13794`：「quantity=controlled_card_zero_round_ratio arm=controlled index=0 value=1.079304 judgement=drift_gone」。受控臂第 3 轮比 `:13776`：「median_round_three_over_round_one=0.777432」仍有一个偏离轮 | **走势推翻**（第 4 次两条臂都没有）；**不稳仍在**（受控臂卡 0 不稳 535/1704，复现臂 208/1564，第 3 次 896/1564，数在第四节） |
| 二.4 校准看不见 c_floor 的漂移 | 没有新的校准对照，原因没分开 | **没碰** |
| 二.5 登记 J4 逐点读：A、D 卡 0 不是出局是不当已判；C 卡 0 只有 3 个点跨全部旋钮不赔（290） | `out/j4_points.txt`：`today` 写法 A、D 卡 0 12 点全翻面、卡 1 12 点全赔（撑住）；C 卡 0 跨全部旋钮不赔的点 3 个变成 8 个（f = 0.75 的 P = 1024 两点与 P = 8192 三点、f = 0.25 的 P = 8192 三点）；两处都换（`both_three_way_without_build`）后 C 卡 0 一个点都不是全不赔，卡 1 12 点全翻面或全赔 | A、D **撑住**；C 的点数 **推翻（3 → 8，今天的写法）**，两处都换后 **撑住** |
| 二.6 间隔 0 不是产品形态、产品最短间隔 18–338 ms；间隔是翻得最多的旋钮（317） | 间隔仍只量 0 与 50 ms；一批 CPU 18–338 ms 来自第三段，没重量。旋钮翻面：`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:12263`：「knob=gap clause=flips_with_a_knob pairs=864 flipped=489」，同文件 `:11179`：「knob=card clause=two_cards_judge_differently pairs=984 flipped=756」。卡旋钮比间隔更高；非卡旋钮里间隔仍最高（其余 190、141、111、148、53、1，第四节） | 间隔的端点问题 **没碰**；「间隔翻得最多」**撑住（除卡外）** |
| 二.6 间隔 50 ms：C 卡 1 几乎全赔、卡 0 多半不赔 | 受控臂卡 1 间隔 50 ms C 36 格里赔 30、不稳 6、不赔 0（`out/arms_judgements.txt`）；卡 0 不赔 29、不稳 7 | **撑住（更硬）** |
| 二.7 门槛 ρ 取段首还是段末，C 有 21 格 `not_losing` 掉成 `unstable`（343） | `research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:12265`：「knob=threshold_machine clause=machine_ratio_decides pairs=1344 flipped=1」（第 3 次 1342 对翻 214 对）；复现臂 `:3635` 0/782。本批 ρ 取 1.002，`research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:9`：「crt_source=r4_drift_control_controlled threshold_source=direct_from_the_r4_product ratio=1.002076」 | **推翻**（门槛机器不再翻面） |
| 二.8 只量两张卡；卡间每字节差 7.9 倍、间隔 50 ms 差 23.6 倍（366） | 卡 0 间隔 0 每 MiB 0.042704 对卡 1 0.483803（11.3 倍），间隔 50 ms 0.043744 对 1.449579（33.1 倍），行号见 G3 一节 | **撑住（差更大）**；第三张卡 **没碰** |
| 二.9 提交的字节与真实 GC 一批对不上（370） | 形态仍 `shared`、`split`、`today_single_kernel`（`out/pts_forms.txt`），池与单元没变 | **没碰** |
| 二.10 岔路单的「5368 行」（376） | r3 产物，没重量；我现数 2684 行、`judgement=unstable` 1054 行、`floor_judgement=unstable` 475 行（第四节） | **没碰（数对得上）** |
| 二.11 第三段各项份额之和大于整次调用（390） | 问题单第 3 行撤掉不量，第三段没重量；现数 8 行汇总里 7 行余项为负、份额和 1.167–2.517（第四节） | **没碰（数对得上）** |
| G1-① 乙段（区域扫描线性找）占 C 门槛 41–62%（P = 8192）、9–20%（P = 1024）（418） | 写法 `today`、受控臂 c_rt 下重算门槛：P = 8192 乙占 41.5%–62.7%，P = 1024 占 8.8%–17.8%（`out/r2_claims.txt`） | **撑住** |
| G1-① 按身份查（取 P = 1024 的乙）后卡 0 的 99 格不赔剩 37 格 | 量到的 `identity_index_without_build`：卡 0 C 不赔 88/108（`today` 101/108），按身份查在 P = 8192、f = 0.25、K = 4 上 107.8 µs 对线性找 275.4 µs（`out/cpu_stage_medians.txt`）；推的替换值与量到的相近，但不赔格数没掉到 37 | 替换值 **撑住**，格数 **推翻** |
| G1-② 指令 CRC 比软件八表快 3.98 倍（本机探针，未入库）（461） | 入库的阶段中位：软件 82.2 µs 对单路指令 17.2 µs（4.78 倍）、三路 6.1 µs（13.5 倍）（`out/cpu_stage_medians.txt`，P = 1024、f = 0.25）；A、D 在 `hardware_crc_single` 下两张卡 216/216 全赔 | **撑住（更硬）**，并且现在有了能引的入库数 |
| G1-③ 两处都换后间隔 50 ms 下 C 一格不赔都没有（488） | `both_three_way_without_build` 间隔 50 ms：卡 0 C 36 格赔 2、不稳 34、不赔 0，卡 1 36 格全赔（`out/tab_gap50.txt`） | **撑住** |
| G1-③ 同一句在间隔 0：卡 0 剩 18/144、卡 1 剩 6/144 | 受控臂 c_rt：卡 0 间隔 0 不赔 55/72、卡 1 10/72（`out/tab_gap0.txt`）；换成第 3 次的 c_rt 重判（`out/tab_r3crt_gap0.txt`）卡 0 只剩 4/72，与攻方一致 | 在第 4 次的读数下 **推翻**；它依赖用哪一份 c_rt |
| G2-① 卡 0 拟合不线性跟着漂移走（515） | 第 3 次卡 0 的 9 条拟合 7 条不线性；第 4 次受控臂卡 0 3 条、复现臂卡 0 3 条，**全在间隔 50 ms**，间隔 0 的 6 条全线性；复现臂没有漂移，间隔 50 ms 的不线性照在 | 「跟着漂移走」**推翻**；间隔 50 ms 的固定部分不可信 **另有来由** |
| G2-② 墙钟 5.3–6.4 ms 是核的串行深度（541） | 提交单元 ≤ 24 的每次提交挂钟 5.30–7.64 ms，核没换 | **撑住** |
| G2-③ 跨意图 1 格；A 照意图内一次交不赔（547） | `merge_across_intents` 0 行（648 行里），卡 1 间隔 50 ms 54 行、间隔 0 26 行是意图内一次交；A 照乙-1 判的 a_per_intent 是推的模型 | 跨意图 **撑住（更硬）**；a_per_intent **没碰** |
| G3-① 轮转让 C 4 格翻赔、23 格翻不稳（595） | 受控臂：C 卡 0 单用不赔的 101 格里 11 格赔、11 格不稳（`out/round_robin.txt`）；两张卡判定不同 `:11179` 984 对翻 756 对（第 3 次 1096 对翻 630） | **撑住** |
| G3-②③ 编译器缺陷、BIOS 开关集显（616、626） | 条款字面与推的序列 | **没碰** |
| G4 零次上界只罩两份 CRC 核、1 TiB 期望 47.93 条（636） | 第二段没重量；算术我重算（第四节）：N = 1 为 47.93，N = 2 为 6.847e-05，N = 3 为 9.781e-11 | **没碰（数对得上）** |
| G5、G6、G7（644、660、664） | 条款字面、代码字面与推的序列 | **没碰**（判法在第一节各格） |
| G8-①②③ 三样保证（678、718、724） | 代码行我现查，行号在 G8 一节；`last_stop` 起跑时不读（第四节） | **没碰**（但归因有改，见 G8 一节） |
| G8-⑤ `run_compaction` 当后台入口，用户改动最坏等 6768 ms（747） | 第三段一批 CPU 没重量；算术 20 × 338.405 = 6768.1 | **没碰** |
| G8-⑥ 前台每次提交 5.5–7.7 ms、两次 11.0–15.4 ms（751） | 单次 5.30–7.64 ms，两次 10.6–15.3 ms（`out/wall.txt`） | **撑住** |
| 透视的岔路 读法甲：卡 0 五轮是否可交换，翻面观测是第 5 轮 ÷ 第 1 轮（408） | 这一行的翻面观测已出：受控臂 1.079、复现臂 1.057，走势不复现 | 观测 **已出**，该行的候选「照现在的五轮读 ／ 重量、轮次打乱」按岔路单格式该改状态（第三节） |

**净结论**：第二轮攻方对 **A、D 在卡 0 的出局、C 在间隔 0 的几乎全赔、卡 0 漂移、ρ 取法、拟合归因** 这五处，第 4 次重跑推翻或大幅改写；对 **A、D 在卡 1 的出局、间隔 50 ms 下两处都换后 C 无一格不赔、C 的下界标签、A–C 同一 c_rt、硬件 CRC 的倍数、轮转的代价、卡间差、每次提交挂钟、跨批攒不成** 撑住或更硬；对 **第三张卡、条款字面类的反例、第二段与第三段的数** 没碰。**什么现象会推翻这张表**：把受控臂卡 0 在另一天重量一次，A、D 在 `today` 写法下的不赔格数掉回 0（那第 4 次的受控臂读数本身是这一天的偶然）。

## 三、关的第 3 条：未定项清单的完整性

清单的现状：岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 的表有 11 行（`grep -c '^| [0-9]* |'` 数出 11），C906 在 `.claude/kb/checks-owed.md:585`，岔路单里一处也没提（`grep -c 'C906'` 0 处）。再加上第二轮攻方每格带的「交用户定」小表（`research/prompts/m4-gpugc-design-r2-opus-output.md:408`、`:791`、`:800` 起）和第 4 次重跑问题单 `research/prompts/m4-gpugc-measure-r4-forks.md` 第 1 行，对一遍：**清单不完整**，缺下面 10 行；另有 5 行的状态已被第 4 次重跑改写，不改它们，交用户时给的会是过时的够判条件。

### 3.1 漏的补成表行（候选、翻面观测、够判条件；不写倾向）

| 编号 | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| 补 1 CPU 模式拿哪一种写法当基线 | 甲 今天的写法（`crates/singlefs-core/src/checksum.rs:3` 的软件 slicing-by-8，区域扫描线性找）；乙 CRC 换硬件指令；丙 区域扫描按身份查；丁 两处都换，丁再分「索引随版本增量维护（不计建索引）」与「每次扫描现建（计入）」 | 候选 A 在卡 0 的不赔格数：甲 44/108、乙 0/108；C 在卡 0：甲 101/108、乙 88/108、丙（不计建索引）88/108、丁（不计建索引）55/108、丁（计入）108/108 全不赔（建索引让 CPU 模式更慢，只能当上界，不能当 C 的辩护，见第一节读法口径 3）；全表 `out/tab_all_gaps.txt` | 用户或主 agent 定哪一种是支持 CPU 模式的人认的那一臂（判据见 `.claude/singlefs-ai-sop/rules/evidence-discipline.md@不许先有结论再建模型` 里的稻草人对照臂）；建索引的增量维护代价要有数（现在只有每次扫描现建的 2004 µs） |
| 补 2 CRC 换指令并不并进七·3（与岔路单第 10 行同类） | 并进七·3 ／ 不并，留在里程碑四优化项；并的话 checker 档那份按位实现不变（`crates/singlefs-core/src/checksum.rs:3` 写明两份对同一段字节必须给同一个数），核心层这份换指令 | A、D 在 `hardware_crc_single` 下两张卡 216/216 全赔（`out/tab_all_gaps.txt`）；C 的门槛不变、CPU 模式的 CRC 一项由 82.2 µs 到 17.2 µs（P = 1024、f = 0.25、K = 8） | 换了之后核心层的 CRC 单测与对拍绿；用户答并不并 |
| 补 3 间隔取哪个范围当产品形态 | 甲 只看量到的 0 与 50 ms 两点；乙 补量 100、300、1000 ms，只读间隔不短于一批 CPU 的格（第三段一批 CPU 18–338 ms，来自第二轮攻方 `research/prompts/m4-gpugc-design-r2-opus-output.md:321`） | 两处都换后 C 在卡 0：间隔 0 不赔 55/72、间隔 50 ms 0/36（`out/tab_gap0.txt`、`out/tab_gap50.txt`）；间隔再长 C 回到有不赔格与否 | 第一段补量长间隔；第三段一批 CPU 在 f、P 更多点上量到（今天只有四个点的 5 轮） |
| 补 4 全部显卡下怎么分活、赔的卡怎么办 | 甲 全部卡等权轮转；乙 只用点名的卡（卡 0）；丙 挂载时在每张卡上量 c_rt，赔的卡不用；丁 按 c_rt 加权。另：「全部」的个数随 BIOS 开关集显变化时，显存上限列表（D26 已定项 13 第 3 条）对不上怎么办（拒绝挂载 ／ 以点名为准） | 轮转让 C 卡 0 单用不赔的 101 格里 11 格赔、11 格不稳（`out/round_robin.txt`）；两张卡判定不同 984 对翻 756 对（`research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:11179`）；卡 1 在间隔 50 ms 上 C 一格不赔（建索引计入的写法除外） | 本机三张卡都量第一段（AMD 集显没量）；用户答 `.claude/kb/decisions/26-后台整理与放置回收.md:246`：「不给就用本机全部显卡」要不要改窄 |
| 补 5 GPU 代码放哪（G7） | 甲 产品 GPU 包进同一工作区，与 checker 档共用 `wgpu =30.0.1`（`crates/singlefs-checker-tier/Cargo.toml:27`）与 naga；乙 另开工作区带自己的锁（先例：工作区根 `Cargo.toml` 的 `exclude = ["scripts/experiment-apparatus"]`，装置那份锁与 checker 档钉成同一版）；丙 直写 SPIR-V 不经 naga | naga 30.0.1 上能不能造出一个核形状：两份核一致译错、对拍小输入没覆盖 | 造得出一例；或用户定共用翻译链是可接受的残余（乙可以钉另一版 naga 分散这一段，丙躲开 naga，三者都躲不开 Vulkan 驱动那一层） |
| 补 6 控制接口：GC 停着时再收到切换请求 | 甲 进门读开关，停着时的 `cpu` 切换请求不解除停下；乙 停着时收到切换请求当场解除（不违第 1 条「从下一批开始生效」，解除后的起跑就是下一批） | S、E、B 序列（先请求切到 `cpu`、这批 GPU 随后出错、下一批开头）下 GC 能不能再起（条款 `.claude/kb/decisions/26-后台整理与放置回收.md:248` 写「恢复要人切回」，S 发生在 E 之前） | 只有 G8 选挪出、停下成为持久状态时才可达；条款补一句，或一条走这一序的用例 |
| 补 7 定时兜底与发布预算的关系 | 甲 兜底触发的一趟与用户改动共用同一个预算，用完就等下一次用户改动；乙 兜底的一趟另有预算（周期内至多若干次）；丙 兜底的一趟照现在的 `.max(1)`（`crates/singlefs-core/src/compaction_plan.rs:862`）给 1 次 | 闲置池上连着的整理发布次数对 `3S − 4`（第二轮攻方模型：200 拍连着 213 次，线是 20，`research/prompts/m4-gpugc-design-r2-opus-output.md:690`） | 一条闲置池上定时触发连着超过 `3S − 4` 次就红的用例；用户答兜底要不要受预算管 |
| 补 8 起跑三水位的阈值与一批的单元数 K | 水位三个值（全空聚簇段数、碎片度、不可回收字节）与 K：条款都没写，`crates/singlefs-core/src/compaction_plan.rs:865` 水位恒 0，批大小产品路径在签任何意图前停下；装置量的点是 K = 4、8、16 | C 的门槛随 K 变：P = 8192、f = 0.75，K = 4 为 0.575046，K = 8 为 0.690835，K = 16 为 0.744155 ms（`out/r2_claims.txt`） | 等 C412 与 E10（D26 已定项 13 依据一段写明数值等它们）；用户定 K |
| 补 9 整理批要不要过常规准入（C906） | 甲 整理批照条款走常规准入，最坏占用计入已承诺预留；乙 改条款：整理批的空间来自 checkpoint 保留池，不走常规准入，并写清它与已承诺预留的关系（原文 `.claude/kb/checks-owed.md:585`） | 带整理批把可用压到需求之下的用例，准入拒不拒（甲）；注释 `crates/singlefs-core/src/transaction.rs:9667` 至今写「两判都不判」 | 用户定哪一边对；改的一边落进代码或条款，加那一条用例 |
| 补 10 判定怎么读（「登记读法」到底是哪一种） | 甲 逐格 5 轮全相同才算不赔（登记 R1–R3 的字面）；乙 多数轮；丙 登记 J4 逐点（任何旋钮翻面的点不当已判）；丁 只读间隔不短于一批 CPU 的格 | A 在卡 0 `today` 写法：甲 44/108，乙（间隔 0、50 ms）64 + 6 = 70/108（`out/majority_reading.txt`），丙 12 点全翻面、已判 0 点（`out/j4_points.txt`） | 用户或主 agent 定一种当最终读法，第一轮判决的「出局」才有统一含义 |

### 3.2 已有行的状态更正（第 4 次重跑之后）

| 行 | 现在写的 | 第 4 次重跑之后应写的 |
|---|---|---|
| 第 1 行 | 「够判：第 3 次重跑第一段……跑完」 | 够判条件里 c_rt 要取受控臂（`research/prompts/m4-gpugc-measure-r4-forks.md` 第 2 行定）；候选 A、B、D「两张卡都出局」只在卡 1、间隔 50 ms、硬件 CRC 写法下成立（第一节 G1）；C 的乙、丙输入准备没量，仍是下界；补 1、补 3 的选择先于这一行 |
| 第 3 行 | 「`merging_shape` 432 行」「实验那一半够判」 | 受控臂与复现臂 648 行 `merge_across_intents` 0 行；间隔 50 ms 的固定部分在两张卡上不稳（卡 0 受控臂与复现臂各 3 条不线性，卡 1 1 条），间隔 0 的拟合可用 |
| 第 6 行 | 「够判」 | 不变；补一句：零次读数不分辨 N 取 1 还是 2（随机不一致的 N = 1 期望 47.93 条／TiB，N = 2 为 6.847e-05），确定性不一致的代价是 N 越大越贵 |
| 第 8 行 | 「软件 Vulkan 适配器会不会被当成一张卡」 | 本机列出三张 PCI 显卡（两张 NVIDIA、一张 AMD 集显，第四节命令），没有一张是软件适配器；`lvp_icd.json` 是否被 `wgpu` 枚举进来仍没列；与补 4 合并问 |
| 第 10 行 | 「开着：……要换一种是划分的拆法重量」 | 拆法已撤（问题单第 3 行不量）；写法丙量到按身份查之后扫描一项（P = 8192、f = 0.25、K = 4：线性找 275.4 µs 对按身份查 107.8 µs）；剩下的是并不并，与补 2 同形 |
| 第 11 行 | 乙「补齐三样保证」 | 三样里预算与跨趟两样是任何让整理脱离用户改动起跑的放法都要补的，第三样要先拆 API 才会碰到（G8 一节）；补 7、补 9 是它的子项 |

**这一条的结论**：清单漏 10 行（补 1–补 10）、5 行的状态过时（第 1、3、8、10、11 行）、第 6 行要补一句；第 4、5、7、9 行第 4 次重跑没碰，第 2 行只添了每次提交挂钟的数（5.30–7.64 ms）。**什么现象会推翻**：用户或主 agent 说补 1–补 10 里某一行已经在别处（例如问题单、决策页）定了；那一行改成指过去的出处，不算漏。

## 四、关的第 4 条：正文第三节与第二轮判决的数，逐个从产物现数

命令全在草稿目录的 `verify4.sh`、`run_extra.sh`、`run_all.sh`（哈希在文末附），只读 `research/results/` 与主工作区，不编译、不起 GPU。下面每块先写命令（`$` 起头的行），后面是这一次真跑出来的原样输出。这一节引的行号都是这次现查的。

### 4.1 正文第三节三份产物与漂移读数

`verify4.sh` 的第一组命令（`$` 起头的行是命令，其下为输出，未改动）：

```text
$ wc -l 三份第 4 次重跑产物
   13808 research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out
    2362 research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out
  108438 research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz
  124608 total
$ grep -n "E7RESULT name=verdict" 三份
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13807:E7RESULT name=verdict segment=host_cpu_drift_control constant_or_anchor_mismatches=0 stop_clause_findings=0 arms_measured=2 cards_started=2 card_failures=0 card_failure_items=none voided_forms_by_the_positive_controls_or_probes=0
research/results/e185-gc-batch-wall-clock-split-cpu-writings-r4-2026-10-24.out:2361:E7RESULT name=verdict segment=cpu_writings constant_or_anchor_mismatches=0 stop_clause_findings=0 positive_control_failures=0 positive_control_failure_items=none
research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:108437:E7RESULT name=verdict segment=baseline_rejudge stop_clause_findings=0 rejudged_lines=96696 voided_writings=none drift_control_end=measured
$ grep -nE "name=(drift_judgement|failure_clause)" 控漂移段
13794:E7RESULT name=drift_judgement quantity=controlled_card_zero_round_ratio arm=controlled index=0 value=1.079304 judgement=drift_gone
13795:E7RESULT name=drift_judgement quantity=reproduction_card_zero_round_ratio arm=reproduction index=0 value=1.056747 judgement=drift_not_reproduced
13796:E7RESULT name=drift_judgement quantity=controlled_card_one_round_ratio arm=controlled index=1 value=1.000256 judgement=drift_gone
13800:E7RESULT name=drift_judgement quantity=controlled_unstable_card_zero_over_card_one arm=controlled index=none value=8.629032 judgement=card_zero_less_stable
13803:E7RESULT name=failure_clause clause=L4 card_one_judgement=drift_gone warm_ups_reaching_the_cap=false,false triggered=false
13804:E7RESULT name=failure_clause clause=L5 triggered=true
$ grep -n clause=L1 重判（both_three_way_without_build、受控臂）
108411:E7RESULT name=failure_clause clause=L1 writing=both_three_way_without_build threshold_source=direct_from_the_r4_product crt_source=r4_drift_control_controlled changed_to_losing=78 triggered=true
108413:E7RESULT name=failure_clause clause=L1 writing=both_three_way_without_build threshold_source=ratio_on_the_r2_product crt_source=r4_drift_control_controlled changed_to_losing=78 triggered=true
$ grep -nE "name=unstable_count" 控漂移段与重判
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13797:E7RESULT name=unstable_count arm=reproduction index=0 lines=1564 unstable=208
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13798:E7RESULT name=unstable_count arm=controlled index=0 lines=1704 unstable=535
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out:13799:E7RESULT name=unstable_count arm=controlled index=1 lines=984 unstable=62
research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:108396:E7RESULT name=unstable_count_r3 index=0 lines=1564 unstable=896
research/results/e185-r4-baseline-rejudge-2026-10-24.out.gz:108397:E7RESULT name=unstable_count_r3 index=1 lines=1120 unstable=158
```

对照：

| 正文第三节说 | 现数 | 对上？ |
|---|---|---|
| 控漂移段 13808 行；写法段 2362 行；重判 108438 行 | 13808、2362、108438 | 对上 |
| 三条 verdict 行（`constant_or_anchor_mismatches=0 stop_clause_findings=0 arms_measured=2 cards_started=2 card_failures=0 …`；`positive_control_failures=0`；`rejudged_lines=96696 voided_writings=none drift_control_end=measured`） | 控漂移段 `:13807`、写法段 `:2361`、重判 `:108437`，字段逐字相同 | 对上 |
| 复现臂 1.056747 `drift_not_reproduced`；受控臂卡 0 1.079304、卡 1 1.000256 都 `drift_gone`；`controlled_unstable_card_zero_over_card_one` 8.629032；L5 触发 | `:13795`、`:13794`、`:13796`、`:13800`、`:13804` | 对上 |
| L1 `writing=both_three_way_without_build … crt_source=r4_drift_control_controlled changed_to_losing=78 triggered=true` | `:108411`（直接门槛）与 `:108413`（第二轮门槛）两行都是 78、`triggered=true` | 对上 |
| 控漂移段不稳格数：受控臂卡 0 535/1704（我据此写的 G1 一节） | `:13798` | 对上 |

### 4.2 正文第三节的透视（24 行）

命令：`python3 -I sumrows.py | sort`，直接读 `baseline_rejudge_summary` 行，取 `crt_source=r4_drift_control_controlled threshold_source=direct_from_the_r4_product threshold_kind=machine_scaled clock=thread`。输出：

```text
both_single_with_build         A card0 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_single_with_build         A card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_single_with_build         B card0 lines=108 not_losing=0 not_losing_lower_bound=60 losing=4 unstable=44
both_single_with_build         B card1 lines=108 not_losing=0 not_losing_lower_bound=8 losing=92 unstable=8
both_single_with_build         C card0 lines=120 not_losing=120 not_losing_lower_bound=0 losing=0 unstable=0
both_single_with_build         C card1 lines=120 not_losing=89 not_losing_lower_bound=0 losing=24 unstable=7
both_single_with_build         D card0 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_single_with_build         D card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_three_way_without_build   A card0 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_three_way_without_build   A card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_three_way_without_build   B card0 lines=108 not_losing=0 not_losing_lower_bound=48 losing=14 unstable=46
both_three_way_without_build   B card1 lines=108 not_losing=0 not_losing_lower_bound=5 losing=93 unstable=10
both_three_way_without_build   C card0 lines=120 not_losing=64 not_losing_lower_bound=0 losing=6 unstable=50
both_three_way_without_build   C card1 lines=120 not_losing=10 not_losing_lower_bound=0 losing=103 unstable=7
both_three_way_without_build   D card0 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
both_three_way_without_build   D card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
today                          A card0 lines=108 not_losing=44 not_losing_lower_bound=0 losing=14 unstable=50
today                          A card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
today                          B card0 lines=108 not_losing=0 not_losing_lower_bound=84 losing=0 unstable=24
today                          B card1 lines=108 not_losing=0 not_losing_lower_bound=18 losing=78 unstable=12
today                          C card0 lines=120 not_losing=113 not_losing_lower_bound=0 losing=0 unstable=7
today                          C card1 lines=120 not_losing=59 not_losing_lower_bound=0 losing=47 unstable=14
today                          D card0 lines=108 not_losing=50 not_losing_lower_bound=0 losing=8 unstable=50
today                          D card1 lines=108 not_losing=0 not_losing_lower_bound=0 losing=108 unstable=0
```

| 正文说 | 现数 | 对上？ |
|---|---|---|
| `today`：C 卡 0 120 格不赔 113、不稳 7；卡 1 不赔 59、赔 47 | `lines=120 not_losing=113 … unstable=7`；卡 1 `not_losing=59 … losing=47 unstable=14` | 对上（卡 1 另有不稳 14，正文没提，不矛盾） |
| `today`：A 卡 0 不赔 44；D 卡 0 不赔 50；卡 1 上 A、D 全赔 | A 卡 0 `not_losing=44`、D 卡 0 `not_losing=50`，卡 1 都 `losing=108` | 对上 |
| `both_three_way_without_build`：C 卡 0 不赔 64、赔 6、不稳 50；卡 1 不赔 10、赔 103 | 同（卡 1 另有不稳 7） | 对上 |
| 同写法：A、D 两张卡全赔；B 卡 0 只剩下界不赔 48 | A、D 四行都 `losing=108`；B 卡 0 `not_losing_lower_bound=48` | 对上 |
| `both_single_with_build`：C 卡 0 120 格全不赔，卡 1 不赔 89 | `not_losing=120`；卡 1 `not_losing=89` | 对上 |

正文第三节的数全部对得上。**对不上的是读法，不是数**：第三节的 C 数含 12 格 `today_single_kernel`（第一节读法口径 2），并且把间隔 0 与 50 ms 并着数、`both_single_with_build` 是让 CPU 模式更慢的写法（口径 3）。

### 4.3 第二轮判决报出的数

命令（`verify4.sh` 第三组，`$` 起头的行是命令）：

```text
$ lspci | grep -iE "vga|display|3d"
01:00.0 VGA compatible controller: NVIDIA Corporation Device 2b85 (rev a1)
10:00.0 VGA compatible controller: NVIDIA Corporation Device 2d04 (rev a1)
79:00.0 VGA compatible controller: Advanced Micro Devices, Inc. [AMD/ATI] Device 13c0 (rev c1)
$ grep -c host_cpu_per_submission_point 第 3 次第一段
2684
$ grep name=host_cpu_per_submission_point | grep -c " judgement=unstable"
1054
$ ... | grep -c "floor_judgement=unstable"
475
$ grep -c "judgement=" 两个字段合起来的个数（岔路单旧写的 5368）
5368
$ grep -n "position=end" 第一段终点校准
8004:E7RESULT name=calibration_baseline segment=host_cpu_per_submission position=end runs=21 median_nanoseconds=5493404.0 chase_median_nanoseconds=102919583.000000 anchors_hold=true ratio_of_the_end_to_the_start=0.876478 drifted=true
$ grep -c "name=function_split_summary" 第三段；余项为负的行数
8
7
$ grep -n 第二段上界
109:E7RESULT name=disagreement_summary index=0 form=shared natural_unit_times=1048560 expected_natural_unit_times=1048560 natural_unit_times_sufficient=true first_round_unreadable=0 first_round_kernel_disagreements=0 first_round_any_difference=0 first_round_ra
175:E7RESULT name=disagreement_summary index=0 form=split natural_unit_times=1048560 expected_natural_unit_times=1048560 natural_unit_times_sufficient=true first_round_unreadable=0 first_round_kernel_disagreements=0 first_round_any_difference=0 first_round_rat
176:E7RESULT name=disagreement_card_summary index=0 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
246:E7RESULT name=disagreement_summary index=1 form=shared natural_unit_times=1048560 expected_natural_unit_times=1048560 natural_unit_times_sufficient=true first_round_unreadable=0 first_round_kernel_disagreements=0 first_round_any_difference=0 first_round_ra
312:E7RESULT name=disagreement_summary index=1 form=split natural_unit_times=1048560 expected_natural_unit_times=1048560 natural_unit_times_sufficient=true first_round_unreadable=0 first_round_kernel_disagreements=0 first_round_any_difference=0 first_round_rat
313:E7RESULT name=disagreement_card_summary index=1 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
$ 第二轮判决要用的代码行
    let entries_pointing_here: Vec<&CentralMappingEntry> = mapping_entries
        .iter()
        .filter(|entry| entry.points_at(record.device, record.slot))
        .collect();
    let header_key_is_mapped = mapping_entries
        .iter()
        .any(|entry| entry.key == header.mapping_key);
    if entries_pointing_here.is_empty() && !header_key_is_mapped {
862:        .max(1)
2704:    let mut last_completed_intent: Option<CompactionIntent> = None;
9667:/// （固定点，空间归 checkpoint 保留池），搬迁副本不进
3://! 实现用 slicing-by-8；checker
100:2. **整理不是释放路径，它走常规准入**，批次最
```

第一段挂钟范围（判决 G1 一格写「一次提交挂钟 5.46–8.05 ms」）：命令 `python3 -I w805.py`，原样输出：

```text
[4, 8, 12, 16, 20, 24, 32, 512]
4 50 card1 min 5.457 max 6.824
8 100 card1 min 5.457 max 6.982
16 200 card1 min 5.457 max 7.301
24 300 card1 min 5.457 max 7.737
32 350 card1 min 5.457 max 8.782
48 350 card1 min 5.457 max 8.782
64 350 card1 min 5.457 max 8.782
100 350 card1 min 5.457 max 8.782
128 350 card1 min 5.457 max 8.782
```

开工快照与判决引文的复核（`verify4.sh` 第四组、`cite-check.py` 复跑第二轮两份腿报告）：

```text
$ sha256sum -c 第三轮开工快照（代码与 kb）
crates/singlefs-core/src/compaction_plan.rs: FAILED
crates/singlefs-core/src/transaction.rs: FAILED
crates/singlefs-checker-tier/Cargo.toml: FAILED
sha256sum: WARNING: 3 computed checksums did NOT match
      3 FAILED
      1 match
     20 OK
.claude/kb/checks-owed.md: FAILED
.claude/kb/experiments/185-GC一批搬迁的挂钟拆分.md: FAILED
sha256sum: WARNING: 2 computed checksums did NOT match
$ sha256sum -c 第二轮开工快照
crates/singlefs-core/src/compaction_plan.rs: FAILED
crates/singlefs-core/src/transaction.rs: FAILED
crates/singlefs-checker-tier/Cargo.toml: FAILED
research/prompts/m4-gpugc-design-r1-forks.md: FAILED
sha256sum: WARNING: 4 computed checksums did NOT match
      4 FAILED
      1 match
     17 OK
$ git status --short 这三份代码（别的会话的改动）
MM crates/singlefs-checker-tier/Cargo.toml
MM crates/singlefs-core/src/compaction_plan.rs
M  crates/singlefs-core/src/transaction.rs
$ python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-design-r2-sonnet-output.md research/prompts/m4-gpugc-design-r2-opus-output.md --root . --background research/prompts/_m4-gpugc-design-r2-background.md
  （末行）核了 77 处引文、24 处裸行号，7 处对不上，25 处没判（其中裸行号没有引文可比 24 处）
  对不上的 7 处：每处是 报告行 → 它引的旧行号 → 那句原文现在的行号（原样输出存在草稿目录的 verify4_f_citecheck_r2.txt，这里不照抄整行，免得这份报告把旧行号当成自己的引文被核）：
    r2 辩方报告第 141 行 → 818 → 865；第 141 行 → 825 → 872；第 167 行 → 806 → 853；第 288 行 → 806 → 853；第 549 行 → 818 → 865
    r2 攻方报告第 680 行 → 805 → 852；第 722 行 → 785 → 832
```

攻方模型原目录复跑（命令：`nice -n 19 bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m4-gpugc-design-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <草稿目录>/opus-r2-rerun`，退出码 0）：

```text
name=rerun_compare model=pivot identical=true
name=rerun_compare model=pivot_wait identical=true
name=rerun_compare model=readings identical=true
name=rerun_compare model=decompose identical=true
name=rerun_compare model=judged_points identical=true
name=rerun_compare model=identity_index identical=true
name=rerun_compare model=c_inputs identical=true
name=rerun_compare model=a_per_intent identical=true
name=rerun_compare model=ratio_end identical=true
name=rerun_compare model=round_robin identical=true
name=rerun_compare model=drift identical=true
name=rerun_compare model=foreground_wall identical=true
name=rerun_compare model=hardware_crc identical=true
name=rerun_compare model=combined identical=true
name=rerun_compare model=g8 identical=true
name=rerun_probe name=crc_speed implementation=software_eight_tables bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=5573585 bytes_per_second=3010130105
name=rerun_probe name=crc_speed implementation=sse42_instruction bytes=16777216 runs=21 crc32c=0x3430be3b anchor_holds=true thread_cpu_median_nanoseconds=1516753 bytes_per_second=11061271018
$ grep -n "policy=background_timer user_change_every=once" opus-r2-rerun/g8.out
5:name=g8_loop batches_per_intent=2 policy=background_timer user_change_every=once ticks=200 publishes=214 max_consecutive_gc_publishes=213 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=70
17:name=g8_loop batches_per_intent=6 policy=background_timer user_change_every=once ticks=200 publishes=218 max_consecutive_gc_publishes=217 budget_line=20 root_ring_slots=24 over_budget_line=true intents_issued_after_an_intent_without_yield=30
```

对照：

| 判决说 | 现数 | 对上？ |
|---|---|---|
| 第一段 `host_cpu_per_submission_point` 2684 行；losing 1137、not_losing 398、not_losing_lower_bound 95、unstable 1054 | 2684；`out/numcheck.txt` 里同一组四个数 | 对上 |
| 岔路单旧写的「5368 行、含 unstable 1529 行」是把 `floor_judgement` 一起数了 | 两个字段合起来 5368；1054 + 475 = 1529 | 对上 |
| C 与 A 同条件下每次提交主机 CPU 672 组逐字相同 | `A vs C groups 672 identical 672`（`out/numcheck.txt`） | 对上 |
| 候选 A、机器比例门槛、线程钟，第 5 轮 ÷ 第 1 轮中位：卡 0 0.528、卡 1 0.958（各 108 格） | `out/round5_over_round1.txt`：0.528、0.958、各 108 | 对上 |
| 辩方报终点校准比起点快 12.4% | `ratio_of_the_end_to_the_start=0.876478`，即快 12.35%，取整 12.4 | 对上 |
| 第三段八行汇总：第一行 0.888（`remainder_share_median=0.110173`），其余七行 1.167–2.517、余项为负（例 2.517 与 `-1.516256`） | `out/numcheck.txt`：0.888 与 0.110173；七行 1.628、1.309、2.517、1.167、1.578、1.675、2.183；2.517 那行余项 -1.516256；余项为负 7 行 | 对上 |
| 本机三张显卡（两张 NVIDIA、一张 AMD 集显），ICD 有 `lvp_icd.json` 在内 9 份 | `lspci` 三行如上；`ls /usr/share/vulkan/icd.d/` 九个文件，含 `lvp_icd.json`、`radeon_icd.json` | 对上 |
| `region_scan.rs` 第 286–289、309–312 行线性找；`mounted_session.rs` 第 2704 行局部变量；`checksum.rs` 第 3 行；`transaction.rs` 第 9663–9670 行注释；D26 第 100 行 | 行号现查都在（上面输出）；`transaction.rs` 注释现在落在第 9667 行 | 对上 |
| `compaction_plan.rs` 第 812–816 行一趟发布预算 | 现在是 `crates/singlefs-core/src/compaction_plan.rs:862`：「.max(1)」，式子在 859–863 行；别的会话改了这份文件，行号后移 47 | **行号对不上，内容在** |
| 开工快照 21 份与 kb 那份全 OK | 现在 17 OK、4 FAILED（`compaction_plan.rs`、`transaction.rs`、`singlefs-checker-tier/Cargo.toml` 被别的会话改过，`m4-gpugc-design-r1-forks.md` 是主 agent 按判决改的）；第三轮快照 23 份里 20 OK、3 FAILED（同三份代码） | **不再对得上（原因在判决之后）** |
| 两条腿引文「核了 77 处引文、24 处裸行号，对上 76 处，没判 25 处」 | 现在 77 处、24 处，**7 处对不上**，全是 `compaction_plan.rs` 的行号后移 | **不再对得上（原因同上）** |
| 攻方 15 个模型复跑 `identical=true` | 我在原模型目录复跑 15 行 `identical=true`，退出码 0 | 对上 |
| 攻方「单路 SSE4.2 比 slicing-by-8 快 3.98 倍」（本机探针，判决只当线索） | 我这次探针 5573585 ns 对 1516753 ns，3.67 倍；入库的阶段中位 4.78 倍（`out/cpu_stage_medians.txt`）；三个数随机器状态在 3.7–4.8 之间 | 同量级，**入库数是 4.78** |
| 第一段挂钟 5.46–8.05 ms | 最小值 5.457 对上；单元 ≤ 24 的最大 7.737，单元 ≤ 32 的最大 8.782；8.05 是 32 单元两格的值（8.051、8.05），不是任何一个单元上限下的最大值 | **上端复现不出**（判决只把它当 G1 一格的量级，不影响结论） |
| 攻方模型 g8：200 拍连着 213 次，线是 20 | 复跑 `g8.out` 第 5 行：`publishes=214 max_consecutive_gc_publishes=213 budget_line=20` | 对上 |
| 辩方：不挪出前台每次净增 5.50–14.76 ms，占一批 4.5%–118% | 第二轮辩方的推导我没拿到（口径不明），只核得出单次提交 5.30–7.64 ms、两次 10.6–15.3 ms | **未核** |
| 辩方：乙换成 P = 1024 后 C 卡 0 的 not_losing 从 52/54 掉到 25/54（代理数） | 判决已说不进 kb；第 4 次重跑量到的按身份查：C 卡 0 不赔 101 → 88/108 | 代理数 **被入库数取代** |

### 4.4 前面各节引到的其余数

控漂移段的旋钮翻面、拟合、重判的门槛机器比、`.err`、核心层线程原语与 `last_stop`（`verify4.sh` 第五组，`$` 起头的行是命令）：

```text
$ grep -c "卡 0 上有别的计算进程" 两份 .err
research/results/e185-gc-batch-wall-clock-split-host-cpu-drift-control-r4-2026-10-24.out.err 命中 0 行
research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out.err 命中 4 行
$ grep -rn last_stop crates/singlefs-core/src
crates/singlefs-core/src/compaction_plan.rs:547:    pub last_stop: Option<CompactionRunStop>,
crates/singlefs-core/src/mounted_session.rs:2632:            compaction_of_the_run.observations.last_stop = Some(*stop);
$ grep 线程原语 crates/singlefs-core/src
crates/singlefs-core/src/block_device.rs:1019:    use std::sync::atomic::{AtomicU64, Ordering};
crates/singlefs-core/src/block_device.rs:1021:    static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
$ grep -rn "run_compaction(" 调用点（非 tests 目录）
crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:3491:        let run = session.run_compaction(devices);
crates/singlefs-checker-tier/src/bin/e185_gc_batch_wall_clock_split.rs:18516:        let Ok(run) = session.run_compaction(&mut devices) else {
crates/singlefs-harness/src/history.rs:6452:        let run_result = mounted_session.run_compaction(devices);
$ grep -ciE "mapping_table|allocation_record|upload_bytes" 写法段
0
$ grep -nE "knob=(gap|threshold_machine|card) " 控漂移段受控臂与复现臂
3183:E7RESULT name=knob_flip_summary arm=reproduction quantity=host_cpu_per_submission knob=card clause=two_cards_judge_differently pairs=0 flipped=0
3634:E7RESULT name=knob_flip_summary arm=reproduction quantity=host_cpu_per_submission knob=gap clause=flips_with_a_knob pairs=380 flipped=264
3635:E7RESULT name=knob_flip_summary arm=reproduction quantity=host_cpu_per_submission knob=threshold_machine clause=machine_ratio_decides pairs=782 flipped=0
11179:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=card clause=two_cards_judge_differently pairs=984 flipped=756
12263:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=gap clause=flips_with_a_knob pairs=864 flipped=489
12265:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=threshold_machine clause=machine_ratio_decides pairs=1344 flipped=1
$ grep -nE "knob=(input_form|wait_|clock)" 受控臂
11370:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=input_form clause=flips_with_a_knob pairs=792 flipped=190
11512:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=wait_blocking_and_sleep_100_microseconds clause=the_wait_method_decides pairs=792 flipped=141
11624:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=wait_blocking_and_sleep_1_millisecond clause=the_wait_method_decides pairs=888 flipped=111
11773:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=wait_sleep_100_microseconds_and_sleep_1_millisecond clause=flips_with_a_knob pairs=792 flipped=148
12319:E7RESULT name=knob_flip_summary arm=controlled quantity=host_cpu_per_submission knob=clock clause=thread_clock_underestimates pairs=912 flipped=53
$ grep -nE "name=machine_ratio " 重判
7:E7RESULT name=machine_ratio crt_source=r3_host_cpu_per_submission threshold_source=direct_from_the_r4_product ratio=1.195306
8:E7RESULT name=machine_ratio crt_source=r3_host_cpu_per_submission threshold_source=ratio_on_the_r2_product ratio=1.199658
9:E7RESULT name=machine_ratio crt_source=r4_drift_control_controlled threshold_source=direct_from_the_r4_product ratio=1.002076
10:E7RESULT name=machine_ratio crt_source=r4_drift_control_controlled threshold_source=ratio_on_the_r2_product ratio=1.005724
$ grep -n "name=fixed_and_per_byte_fit arm=controlled index=[01] form=shared wait=blocking"
9973:E7RESULT name=fixed_and_per_byte_fit arm=controlled index=0 form=shared wait=blocking gap_milliseconds=0 fixed_milliseconds=0.038806 per_mebibyte_milliseconds=0.042704 maximum_relative_residual=0.043583 points=8 fit_not_linea
9974:E7RESULT name=fixed_and_per_byte_fit arm=controlled index=0 form=shared wait=blocking gap_milliseconds=50 fixed_milliseconds=0.199325 per_mebibyte_milliseconds=0.043744 maximum_relative_residual=0.415834 points=8 fit_not_line
9982:E7RESULT name=fixed_and_per_byte_fit arm=controlled index=1 form=shared wait=blocking gap_milliseconds=0 fixed_milliseconds=0.042993 per_mebibyte_milliseconds=0.483803 maximum_relative_residual=0.169842 points=8 fit_not_linea
9983:E7RESULT name=fixed_and_per_byte_fit arm=controlled index=1 form=shared wait=blocking gap_milliseconds=50 fixed_milliseconds=0.130051 per_mebibyte_milliseconds=1.449579 maximum_relative_residual=0.476888 points=8 fit_not_line
```

攒批形态与拟合的计数（命令 `python3 -I shape.py`，读控漂移段的 `merging_shape` 与 `fixed_and_per_byte_fit` 行）：

```text
merging_shape rows 648 merge_across_intents rows 0
('controlled', '0', '0', 'one_submission_per_batch') 144
('controlled', '0', '50', 'one_submission_per_batch') 58
('controlled', '0', '50', 'unstable') 14
('controlled', '1', '0', 'one_submission_per_batch') 104
('controlled', '1', '0', 'one_submission_per_intent') 26
('controlled', '1', '0', 'unstable') 14
('controlled', '1', '50', 'fit_not_linear') 2
('controlled', '1', '50', 'one_submission_per_intent') 54
('controlled', '1', '50', 'unstable') 16
('reproduction', '0', '0', 'one_submission_per_batch') 144
('reproduction', '0', '50', 'one_submission_per_batch') 60
('reproduction', '0', '50', 'unstable') 12
fit rows 27
('controlled', '0', '0', 'false') 6
('controlled', '0', '50', 'true') 3
('controlled', '1', '0', 'false') 6
('controlled', '1', '50', 'false') 2
('controlled', '1', '50', 'true') 1
('reproduction', '0', '0', 'false') 6
('reproduction', '0', '50', 'true') 3
$ grep "^name=merging_shape" 第 3 次第一段：跨意图行数与总行数（命令里取 category 字段）
1
432
```

每次提交挂钟（命令 `python3 -I wall.py`，提交单元 ≤ 24；`today_single_kernel` 另列）与卡 0 的轮次比（`python3 -I r5.py`）：

```text
r4 drift-control
('controlled', '0', 'shared', '0') 91 min 5.303 median 5.758 max 6.413
('controlled', '0', 'shared', '50') 91 min 5.416 median 5.968 max 6.672
('controlled', '0', 'split', '0') 92 min 5.304 median 5.784 max 6.476
('controlled', '0', 'today_single_kernel', '0') 30 min 5.321 median 5.628 max 5.927
('controlled', '1', 'shared', '0') 90 min 5.456 median 6.147 max 6.718
('controlled', '1', 'shared', '50') 90 min 5.618 median 6.886 max 7.637
('controlled', '1', 'split', '0') 90 min 5.524 median 6.446 max 7.082
('controlled', '1', 'today_single_kernel', '0') 30 min 5.532 median 6.119 max 6.722
('reproduction', '0', 'shared', '0') 91 min 5.303 median 5.758 max 6.367
('reproduction', '0', 'shared', '50') 92 min 5.387 median 6.016 max 6.594
('reproduction', '0', 'split', '0') 92 min 5.303 median 5.786 max 6.399
('reproduction', '0', 'today_single_kernel', '0') 31 min 5.327 median 5.680 max 5.932
r3 first segment
('r3', '0', 'shared', '0') 92 min 5.317 median 5.860 max 6.622
('r3', '0', 'shared', '50') 91 min 5.521 median 6.095 max 6.787
('r3', '0', 'split', '0') 91 min 5.351 median 5.924 max 6.714
('r3', '0', 'today_single_kernel', '0') 30 min 5.350 median 5.738 max 6.299
('r3', '1', 'shared', '0') 90 min 5.457 median 6.163 max 6.867
('r3', '1', 'shared', '50') 90 min 5.572 median 6.930 max 7.737
('r3', '1', 'split', '0') 90 min 5.540 median 6.472 max 7.176
('r3', '1', 'today_single_kernel', '0') 30 min 5.553 median 6.177 max 6.819
card 0 108 median round5/round1 = 0.528
card 1 108 median round5/round1 = 0.958
r4 ('controlled', '0') 108 median round5/round1 = 1.079
r4 ('controlled', '1') 108 median round5/round1 = 1.000
r4 ('reproduction', '0') 108 median round5/round1 = 1.057
```

阶段中位（命令 `python3 -I stage.py`，读写法段 `cpu_writing_stage` 行，单位 µs，所有调用与轮次的中位）：

```text
('0.25', '1024', '16') {'crc_today': 82.2, 'crc_hardware_single': 17.2, 'crc_hardware_three_way': 6.1, 'scan_today': 108.6, 'scan_identity_index_lookup': 89.2, 'identity_index_build': 224.9, 'choose': 370.4} (us, median over all calls/rounds)
('0.25', '1024', '4') {'crc_today': 81.8, 'crc_hardware_single': 17.2, 'crc_hardware_three_way': 6.0, 'scan_today': 105.6, 'scan_identity_index_lookup': 89.2, 'identity_index_build': 225.9, 'choose': 369.3} (us, median over all calls/rounds)
('0.25', '1024', '8') {'crc_today': 82.2, 'crc_hardware_single': 17.2, 'crc_hardware_three_way': 6.1, 'scan_today': 109.9, 'scan_identity_index_lookup': 89.3, 'identity_index_build': 224.7, 'choose': 370.5} (us, median over all calls/rounds)
('0.25', '8192', '16') {'crc_today': 82.2, 'crc_hardware_single': 17.3, 'crc_hardware_three_way': 6.1, 'scan_today': 287.9, 'scan_identity_index_lookup': 108.6, 'identity_index_build': 1986.4, 'choose': 387.6} (us, median over all calls/rounds)
('0.25', '8192', '4') {'crc_today': 81.9, 'crc_hardware_single': 17.3, 'crc_hardware_three_way': 6.0, 'scan_today': 275.4, 'scan_identity_index_lookup': 107.8, 'identity_index_build': 2006.5, 'choose': 389.8} (us, median over all calls/rounds)
('0.25', '8192', '8') {'crc_today': 82.1, 'crc_hardware_single': 17.3, 'crc_hardware_three_way': 6.1, 'scan_today': 277.7, 'scan_identity_index_lookup': 107.7, 'identity_index_build': 1987.3, 'choose': 386.9} (us, median over all calls/rounds)
('0.75', '1024', '16') {'crc_today': 245.4, 'crc_hardware_single': 51.3, 'crc_hardware_three_way': 17.9, 'scan_today': 318.3, 'scan_identity_index_lookup': 261.9, 'identity_index_build': 228.1, 'choose': 368.6} (us, median over all calls/rounds)
('0.75', '1024', '4') {'crc_today': 164.8, 'crc_hardware_single': 34.3, 'crc_hardware_three_way': 11.9, 'scan_today': 206.9, 'scan_identity_index_lookup': 175.9, 'identity_index_build': 230.6, 'choose': 371.0} (us, median over all calls/rounds)
('0.75', '1024', '8') {'crc_today': 205.1, 'crc_hardware_single': 42.8, 'crc_hardware_three_way': 15.0, 'scan_today': 266.2, 'scan_identity_index_lookup': 219.2, 'identity_index_build': 229.2, 'choose': 371.8} (us, median over all calls/rounds)
('0.75', '8192', '16') {'crc_today': 245.6, 'crc_hardware_single': 51.7, 'crc_hardware_three_way': 18.0, 'scan_today': 757.0, 'scan_identity_index_lookup': 277.8, 'identity_index_build': 2003.0, 'choose': 381.8} (us, median over all calls/rounds)
('0.75', '8192', '4') {'crc_today': 164.8, 'crc_hardware_single': 34.6, 'crc_hardware_three_way': 12.1, 'scan_today': 556.5, 'scan_identity_index_lookup': 194.0, 'identity_index_build': 2020.6, 'choose': 385.1} (us, median over all calls/rounds)
('0.75', '8192', '8') {'crc_today': 205.0, 'crc_hardware_single': 43.1, 'crc_hardware_three_way': 15.0, 'scan_today': 665.3, 'scan_identity_index_lookup': 236.8, 'identity_index_build': 2004.4, 'choose': 384.4} (us, median over all calls/rounds)
```

轮转与 N 的算术（命令 `python3 -I rr.py`；`python3 -I numcheck.py` 的最后五行）：

```text
every_stage
   ('not_losing', 'losing', 'losing') 11
   ('not_losing', 'losing', 'not_losing') 14
   ('not_losing', 'losing', 'unstable') 11
   ('not_losing', 'not_losing', 'not_losing') 52
   ('not_losing', 'unstable', 'not_losing') 13
   ('unstable', 'losing', 'losing') 4
   ('unstable', 'losing', 'unstable') 3
## N 算术
N=1 expected_uncertain_per_TiB_pass=47.93
N=2 expected_uncertain_per_TiB_pass=6.847e-05
N=3 expected_uncertain_per_TiB_pass=9.781e-11
units_per_TiB 33554432
```

### 4.5 这一条的结论

- 正文第三节报的数（三份产物行数与 verdict 行、漂移读数、24 行透视、L1）**逐个从产物现数，全部对上**；对不上的只有读法（第一节读法口径 1–3），不是数。
- 第二轮判决报出的数共 17 项（4.3 的表）：**10 项对上**；**3 项不再对得上，原因都在判决之后**（别的会话改了 `compaction_plan.rs`、`transaction.rs`、`singlefs-checker-tier/Cargo.toml`，第二、三轮开工快照与两份腿报告的 7 处引文行号随之后移）；**3 项是推出来或口径不明**（辩方 5.50–14.76 ms 与 4.5%–118% 的推导我没拿到，「5.46–8.05 ms」的上端复现不出，P = 1024 的代理数被入库数取代）；**1 项量级对得上但数不同**（指令 CRC 倍数 3.98 对 3.67 与 4.78）。
- 没有一个数对出来与结论相反；但「5.50–14.76 ms」与「4.5%–118%」要在写进 kb 之前由第二轮辩方的推导补回来，我这里核不了。
- **什么现象会推翻这一节**：产物被重写或换文件（三份第 4 次重跑产物的 sha256 变了，4.1 的行号与 4.2 的数都要重数）。

## 五、四条的结论一览

| 条 | 结论 | 在哪一节 |
|---|---|---|
| 1 八格 | G1-①：前半对 A、D 只在卡 1、间隔 50 ms、硬件 CRC 写法下成立，对 B 够不着；后半在间隔 50 ms 站得住、间隔 0 翻了。G1-③ 站得住（范围要写明：产品路径今天恒不起跑）。G2 跨批攒不成站得住、拟合要窄化到间隔 50 ms。G3 打中且同样打中一切用到卡 1 的分法。G4 ① 站得住；② 打中但只是条款字面不全；N 判不了。G5 打中，条件是 G8 选挪出。G6 站得住（只对「fsync 在等」那半句），改法够着一半。G7 够不着。G8 三样里两样不是挪出特有，第三样要先拆 API | 判定一览、第一节 |
| 2 第二轮攻方的反例与数 | 5 处被推翻或大幅改写（A、D 在卡 0 的出局、C 在间隔 0 的几乎全赔、卡 0 漂移、ρ 取法、拟合归因），其余撑住或更硬，条款字面类没碰 | 第二节 |
| 3 未定项清单 | 不完整：漏 10 行（补 1–补 10），5 行状态过时，第 6 行要补一句 | 第三节 |
| 4 数能不能核 | 第三节的数全对上；判决的 17 项里 10 对上、3 项因代码在判决后被别的会话改过不再对得上、3 项推出来或口径不明、1 项数不同 | 第四节 |

## 附：续做的复跑命令与每个文件的 sha256

草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r3-sonnet-2/`；全部只读 `research/results/` 与主工作区，不编译、不起 GPU。复跑（`<草稿>` 指上面那个目录，输出目录给新的空目录）：

```bash
bash <草稿>/run_all.sh <输出目录甲>                      # 先生成 pts.pkl，再出第一批 17 个输出
bash <草稿>/run_extra.sh <输出目录甲>                    # 第二批 5 个输出（r2_claims、numcheck、sumrows、shape、wall_r3_ranges）
bash <草稿>/verify4.sh /home/fy5090/code/singlefs <输出目录乙>   # 第四节 6 个输出
```

```text
34dea9972cceed20c88c4ef750cad5e8f9635695a501806d48c8b8ed7db4cedc  arms.py
f1c33d9fe00a1e97fa2a94db730a81cdf5d16fb172b5edcdaa12e5f6f9fb0cb4  cnt.py
f5ac3d8e37e47ea43744f04ece396cee43eb193f6b832f6401f28c86fc20de7e  gap.py
3b08cb3687565b4fa2be172975e761c7cbc4ed398df910546e2d8d26be75e76c  j4.py
d3309997f660c125ee1068bea6d64aaa1cc59d95f22a7a9300c626cff0d53b82  maj.py
9b120bcb616f7e89bfaafae55aef19c492bafa31a2f02ef63424daf4c0071110  numcheck.py
db14ee88a5d7f6d4f524bd3f2115f168934ace0c303e20b1377355be80bc53f0  pivot.py
3e6591bc3678cd4efe4231e21536552e12433cc31e9bedcefd8b72535fb84ff5  pts.py
b5196e6693294e16ab9619cd584ca65d7f8eae3b6726d5d884c08021c103f476  r2claims.py
5513a2096fb2454ad27451796ffc9c20c5cb4f6073b2a440c2a1567a815dbcc6  r5.py
9be0d0dfaab4e1b42c8c9a698edc41d350bba4cd539a221146c39e38d80a3a6c  rr.py
ee19f9b24b77f4e11a86c1bd44887a66c958f3d4546c0f3b3f60dac7192b2cc6  same.py
8e04e416e778140cc701298fb3297446366e5c813e8cdd8c4a168ca4c385918d  shape.py
6852e184f9e2844c2f9e659bd37f12b3c5afc8021cc8219870b8be3651e49ec2  stage.py
10aba0687af63bd5fed526bed38d4ccb8b948dc45daff56c586fbceb659d88cc  sumrows.py
b42ca4dbdbb96664b269ab5ec0970d44881138546a11ce84b77edbb815c096ba  tab.py
5dcb6343406d67cc0438162d1b5eb1df77ad0d555569f1075f9efe9f3b24300a  w805.py
bead7136547c44f0b45d909d19322a6d3897b6f30bd222dafb5dab00c609bbd1  wall.py
8860cdcb7abe369f4316f773fb4190a3106c6a9d7e703a4e33640d434ea1caaa  run_all.sh
f6d953ec9caeadc4fc2c55c51d2ad26f0fadf168b7f6e32bbf3a1e06475c6450  run_extra.sh
8270330491fea9b075470b775e4db61c9cdd2d2dff6bcb71f9281cec519f90cb  verify4.sh
b76e4d65d2aa5ea28b09b96a4780ff5adefa113939f3607e6a612a25e643a490  arms_judgements.txt
4cee532de3eb353e1ce9ea0987aa18fe7c014fa22eb041a4f43cbfa119ded498  cnt_incl_excl.txt
55ed7a8debb771e0b4493aedbcde5fa6b5e9a8deca0fff7d629860305e2e92ee  cpu_stage_medians.txt
b547af89606827e1e8fd5b8a2d97a9e3f98d025e4a94e673c7c1513829682a0b  j4_points.txt
789b6a990aa35d2da8dc8c6333f8c262cfea66ea88fd76a65563e9461f555378  majority_reading.txt
34ba3e4d96be267a0906f7b55236e794e8633bbb607409cd3b8f251a4ee2a259  numcheck.txt
eb322d5c99b9730bf0faa079330ff0a5638c96d80d606727eacbed32579504e5  pts_forms.txt
172b140859159f3ef9128d91bd5ddd5756beeacbeb77802a5b07a27b539c5001  r2_claims.txt
fcd958bea5dc9e44633dce1379bbe34a74a678cd04f104d91a824ea16d9f9c29  round5_over_round1.txt
7fa81adcc2fb3717de4e63243c3e922649ab538485971fad013af18429e349c5  round_robin.txt
1acdfd2ca6d7e5ea66cbd9a804cd41b69a0ca0096c3e565de78a5d5af4443b3c  same_crt.txt
597d0ea254892901c8dc2d31d2aa10d5295af4aa431a8d6bb9d8448f41df0c5b  shape.txt
d62592cc938ce0c07e9ec0a6a1db203965e36168e741833a2710991c51dca5e6  sumrows.txt
ceae5726d45e96c436dd4b153114a860a155482f787e368cd3031ca164d9d5c5  tab_all_gaps_incl_today_kernel.txt
4920e707bce1f6e98d79737a068b408e7a3cb74ca90c053659317f92375863a5  tab_all_gaps.txt
01dc17ba0a88933e934d5871f788911a88392f4e69de4e1f7d5a9a8661b1beb5  tab_attack_gap50.txt
489d48b4d4f3b27c346283ff9dbe74bdc54c8377c65da918ab154d2925b98173  tab_gap0.txt
e9541faf83f8bcbbab382a7dc4feca3d57e51c4ca2d51f79963aaa736b9c80f8  tab_gap50.txt
1ae374f9a5a691163170efb64b66b542215799a688b06cb68fe79561f1ac254e  tab_r3crt_gap0.txt
82e9c7d84dbaf244eb27c853d1e933e4ddefbb401cc08b0239e1b91082b52240  tab_r3crt_gap50.txt
0bdf98a322b25015e81a08612f9cd3a97176e26b022d2adb0618f4f4ef1a09fb  verify4_a_verdicts.txt
2606207c7832eb8db4c2e48d328c356bff079d343215b14ab099284d318a9868  verify4_b_pivot.txt
ff6650317895ac6c2769c120e6b05fc28b609c930c3787fc14c078b0392a9546  verify4_c_round2_numbers.txt
3ccb7c9a14efc3370994c038f59ef3caf7099699e4166584b996726c2926b9cb  verify4_d_snapshots.txt
c836e4746b1e0cdc90029f6a23680b4aa4ea95c594f8ffd7d6925b08015735cd  verify4_e_misc.txt
4989bc1fd26704fe31ebe4bf25dfe90d69c9e24c5fd7887051ca121279697494  verify4_f_citecheck_r2.txt
e492282c7d850fdf0b9a47b2bdf5b85890202cd6e08a51a94262181248e1c3a8  wall_r3_ranges.txt
a910dbe33fd94f8630fbc0ba36c7eab775f7e2655f4152f283637b599b248e6c  wall.txt
```

`verify4_d_snapshots.txt`、`verify4_c_round2_numbers.txt`、`verify4_f_citecheck_r2.txt` 读的是主工作区此刻的代码与 kb，别的会话再改，这三份的哈希会变；其余只读三份第 4 次重跑与三份第 3 次重跑产物，哈希不随主工作区变。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没改岔路单、判决、kb、决策页。
- 没读禁读清单里的攻方第三轮报告与模型目录。
- 没起 GPU 计算、没跑门禁、没跑 checker 档、没编译 Rust；只用 `lspci` 与 `nvidia-smi -L` 列了一次显卡，并按第二轮攻方的复跑命令在原模型目录复跑了它的 15 个模型（含它那份 C 探针，gcc 编在草稿目录）。
- 没量新数：间隔 ≥ 100 ms、第三张卡（AMD 集显）、乙与丙的输入准备、第二段与第三段都没重量；第 4 次重跑没覆盖的格我写「没碰」。
- 没核出：第二轮辩方 5.50–14.76 ms 与 4.5%–118% 的推导、攻方 432 组口径（我只复现了判决自己的 672 组）、判决 G1 一格「5.46–8.05 ms」的上端。
- 开工快照里三份代码与两份 kb 在判决之后被别的会话改过（4.3），我引 `crates/` 行号取的是主工作区此刻的现状，前一条腿写的两处旧行号（`.max(1)` 与「产品路径恒不起跑」）已改；没追查别的会话改了什么。
- 前一条腿的草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r3-sonnet/`（含 66 MB 的 `pts.pkl`）不是我认领的，我没删，交主 agent 处理。
- 清理：交回前删掉我这条腿自己的 `opus-r2-rerun`（108K，含 gcc 编出的探针二进制）与 `pts.pkl`（64M，可由 `run_all.sh` 重生成）。
