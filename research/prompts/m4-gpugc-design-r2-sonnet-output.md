# m4-gpugc-design-r2 辩方腿（Sonnet）报告（2026-10-24）

复核对象：`research/prompts/m4-gpugc-design-r1-main-verification.md`（下称「判决」）。草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r2-sonnet`。没读禁读清单里的文件（第二轮攻方的报告与模型目录）。

## 复跑命令与 sha256

纯 python 统计，只读产物，不编译、不起 GPU。本腿没有 `crates/` 的副本、没有编译目录。

```text
cd /home/fy5090/code/singlefs
SINGLEFS_RUN_SCOPE='种类=analysis; 跑什么=辩方腿的纯 python 产物统计脚本; 范围=只读 research/results 下一份 .out 产物、输出写草稿目录; 为什么=复跑腿报告里的计数，不编译不起 GPU' \
  nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 4G bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r2-sonnet/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m4-gpugc-design-r2-snapshot \
  --work <新的草稿目录> --model /home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r2-sonnet
# 另一条单跑（不在 rerun.sh 里）：python3 -I <模型目录>/tools/percard.py <第一段产物>
```

脚本（`<模型目录>/tools/`）：
```text
ed0dc7558c274a68549aa10a0556aa5755d940bf29ea1e99c92d33f4286f44db  bypoint.py
0dde5a96b4df9dedb457a468ccca4cc873e15db33f3310d9c366dba665469532  bywait.py
93dc9daf398f066e9fa357ac1e165bc1d31f3d3b76683af23cdb728320ec6cdb  cbyp.py
56386b138d2e5b66085772091471626b6d8a84b660aca1943e0daeff01707ef1  cells.py
4315960081af63b090a7d82c38e125ae84b6120de2ee5db10542e704e8c4e3d8  cfix.py
c520e8e3abbbde854536ac9d9e31b251e4af9a3aa1a3041927807f024b7c6058  fg.py
ab91d392aabf52e28d199cccd8731c09c00204825e04968252dd49a79e3720f9  flips.py
c6500bb93c4c171e4762dd1e58460b946b6c8eef1bf6d606b2ea137fe1fa1175  percard.py
bcda71b40727da02170b364e5004c42dcef7d805f540491ee9e02f89ac1c50b0  readings.py
82d32f6db47ca92e8c9ac3a25f9d968b61deeb402466314dad904caf14dd8570  tally2.py
4ae34a2e629541087a89691b2b0e76de78b61e83c1c21546837189a34f83cc9d  tally3.py
974d50077d56ba34330ed68701087eca197071f31b1ba97e1000b5d4d479e97f  tally.py
3b45a4d7c9576096699b095dfca909901f28b7931a6ca6484ddad9082ff7ff67  wall.py
a89f5338e1344be96fc6e068458aa8c445c5c3fe965ac38e9cbfe3b3683941a7  rerun.sh
```
复跑一遍（退 0，十二个脚本各出一份 `out/*.out`）的输出 sha256（`run1/out/`）：
```text
a3bbabf1ae8cfd824fc17f134eb59a58713677850a8c82724c2d3abc6e54cd71  bypoint.out
d8433c41e84233153115ddc5f45ab8e6144414d66469425eb520ad23919de027  bywait.out
a985f3a6d223aa2ada59a6b0bc07ea2bdefa1e8872d46d203d3fe6df66cc8bdd  cbyp.out
a98d454338970dc432eef64b5f581970241a25354be9990ffe19a13c00fe64c9  cells.out
85fe14372f9fd6d1bb21ad6c844b6a39ffc9717e71a3fe92080e4bec15c29e90  cfix.out
572ee08724209f2eb657990690e231602be73d8cf1888c06f3adb83ba229d6d3  fg.out
be7bd809733ca8b5c33124a3ef0af51034fb8a743de265d548c653a4324f8980  flips.out
e2b0aad267f876527ff674b366f8905ea7d67ee4a1562644f01eb3f92d9f3076  percard.out
4041059d1a815ed880171edba3efd8dca2fed3ec3625c978e5c318c7d7a11e73  readings.out
3eb53f0c01040886db59df47d2a17f5256feff2b808b721260e698c41c094f1b  tally2.out
93140567c6d89671fad222da76b88f16a6585d6f2a495a1e73928a73fe7659a6  tally3.out
bb7f0f92960344ca1e9a2187b4c88415830d9acb99425e1c166ffec15e1a6179  tally.out
cb8583ec93414dfdbe9560e86a4c516aa4d8d3e46715eff1df8bafd8a522f811  wall.out
```
读的三份产物（`research/results/` 下）：
```text
840b68352e6d849e9a397dd71c3fb9d7a1f30c0a5b7f7f362e620b2a7ef69993  e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out
a56a0572b754551cd5f2f082bdf75f47efb7f71d58d9175fd0489d63b22dd34d  e185-gc-batch-wall-clock-split-recompute-disagreement-r3-2026-10-24.out
31fd33a3a7672b9db7e13b54990ef4533db137a1c0f93bd521b66a07b8596851  e185-gc-batch-wall-clock-split-function-split-r3-2026-10-24.out
```

## 先说读数口径（下面每处都用）

- 第一段产物里候选的名字：A = `candidate=scan_checksum_only`，B = `scan_checksum_and_choice`，C = `every_stage`，D = `release_check_only`。
- `name=host_cpu_per_submission_point` 行现数 2684（线程钟 1776、进程钟 908）。岔路单第 1 行写的「5368 行」和「unstable 1529 行」是把 `judgement` 与 `floor_judgement` 两列一起数的：`judgement=unstable` 1054 行、`floor_judgement=unstable` 475 行，合 1529；行数是 2684 的两倍。
- 一格的 `judgement` 是 5 轮各判一次、5 轮相同才写那个判定，不全相同写 `unstable`（`research/prompts/e185-r3-prereg.md` 第六节开头）。B 的 `not_losing` 写成 `not_losing_lower_bound`（丙的输入没量），不当「不赔」。
- 同一个 (f, K, 候选, 卡, 形态, 等法, 间隔) 在 P = 1024 与 P = 8192 两行里的 `host_cpu_per_submission_milliseconds_per_round` 逐字相同（`cfix.out` 首行：1342 个键全相同），只有门槛不同。
- 门槛两种：`attack_ledger` 与 `machine_scaled`（前者 × 机器换算 1.199658 得后者，产物第 121 行）。主 agent 在登记里定读 `machine_scaled`、两种并列报（`research/prompts/e185-r3-prereg.md` 第一节末「主 agent 定」第 2 条）。

## 总览（逐条的依据在后面各节）

| 关的条 | 结论 |
|---|---|
| 1　G1 | ①：缺的那个数量到了，A、D 在 5 轮全一致的读法下一格不赔都没有，C 在卡 0 上成批不赔；但 C 的赢有四成到六成来自乙（区域扫描）在 P = 8192 随池线性涨的那一段，那一段是判决第三节第 3 条登记的 CPU 侧可修的线性找。②：只打中 D，不打中 A、B、C。③：同样打中 A–D 全部，且被新读数加强（一次往返 5.46–8.05 ms，D24 写的是几十微秒） |
| 1　G2 | 站得住但不决定什么：跨意图攒批只在 432 行里的 1 行（卡 1、间隔 50 ms）出现 |
| 1　G3 | 漏判一处：卡 1 的每 MiB 主机 CPU（0.488–1.464 ms）高于 CPU 算同样字节（约 0.37 ms），「按窗口轮转」把窗口派给一张只赔不赚的卡；但小提交上两张卡的原始 CPU 看不出差，证据量级要说清 |
| 1　G4 | ① 站得住且同样打中所有两核方案；② 对内容错够不着，对「活 / 死」判错够得着；N 取几：第二段零次，数不翻面 |
| 1　G5、G7 | 没漏判；G5 并进 G8；G7 另有一处不在格内的共同因素（CPU 侧 CRC 是 slicing-by-8） |
| 1　G6 | 站得住，不挪出时条款空真，挪出时才有可观测的东西 |
| 2　五条改法 | 没有一条被推翻；两条被第二段撑住，一条（挪出）被一次往返的挂钟撑住但没有代价数 |
| 3　A、B、D | 在登记读法下全出局；能活的读法都落在卡 0、间隔 0、后几轮这一角，D 在 f = 0.75、K = 16 一点 12 行中位全在门槛下 |
| 4　G8 | 不挪出的前台挂钟增量：签发那一趟 11.0–14.8 ms（A/B/C，两次提交），已有活意图那一趟 5.5–7.4 ms（一次提交）；D 一次提交 5.5–7.7 ms |

## 一、判决第二节七格逐格复核

判决里每格的原文在 `research/prompts/m4-gpugc-design-r1-main-verification.md` 第 34–40 行（一格一行）。判定三种：站得住 / 够不着 / 同样打中别的臂，另记「漏判」。

### G1（判决第 34 行，三处）

**G1-①　净收益的符号**。`research/prompts/m4-gpugc-design-r1-main-verification.md:34`：「净收益的符号这一轮判不了，两条腿一致，缺 c_rt，进实验」。
- 判定：当时站得住，同样打中 A–D 四个候选（缺的数对每个候选一样）；第 3 次重跑把缺的数量到了，现在能按候选分开写。
- 第一段 5 轮全一致的读法（线程钟，两种门槛，`tally3.out`，B 的 `not_losing` 含下界）：A 卡 0 全部 losing 或 unstable、`all_nl 0`，卡 1 108 行全 losing；D 同；B 卡 0 `all_nl` 10 / 27 行、卡 1 4 / 10 行（只是下界）；C 卡 0 `all_nl` 59 / 73 行、卡 1 33 / 52 行（门槛 attack_ledger / machine_scaled 前后）。
- 「C 赢的那一部分从哪一段省下」：C 比 B 多的就是乙（`candidate_threshold` 里 C 减 B）。`cbyp.out` 的算术（f、P、K 与每条意图省下的毫秒）：
```text
saved per intent ms: f P K  A  B-A  C-B  C  (C-B)/C
0.25 1024 4 A=0.206 B-A=0.372 C-B=0.069 C=0.648 share_of_C_from_scan_part=0.11
0.25 1024 8 A=0.165 B-A=0.373 C-B=0.059 C=0.596 share_of_C_from_scan_part=0.10
0.25 1024 16 A=0.165 B-A=0.372 C-B=0.053 C=0.591 share_of_C_from_scan_part=0.09
0.25 8192 4 A=0.206 B-A=0.386 C-B=0.472 C=1.064 share_of_C_from_scan_part=0.44
0.25 8192 8 A=0.165 B-A=0.387 C-B=0.386 C=0.937 share_of_C_from_scan_part=0.41
0.25 8192 16 A=0.165 B-A=0.386 C-B=0.388 C=0.938 share_of_C_from_scan_part=0.41
0.75 1024 4 A=1.114 B-A=0.370 C-B=0.361 C=1.846 share_of_C_from_scan_part=0.20
0.75 1024 8 A=0.744 B-A=0.370 C-B=0.238 C=1.352 share_of_C_from_scan_part=0.18
0.75 1024 16 A=0.578 B-A=0.367 C-B=0.184 C=1.129 share_of_C_from_scan_part=0.16
0.75 8192 4 A=1.118 B-A=0.384 C-B=2.459 C=3.961 share_of_C_from_scan_part=0.62
0.75 8192 8 A=0.743 B-A=0.381 C-B=1.628 C=2.752 share_of_C_from_scan_part=0.59
0.75 8192 16 A=0.577 B-A=0.385 C-B=1.202 C=2.164 share_of_C_from_scan_part=0.56
```
  乙占 C 的份额：P = 1024 时 9%–20%，P = 8192 时 41%–62%；乙的绝对量在 P 涨 8 倍时涨 6.5–6.8 倍（f = 0.75、K = 4：0.361 → 2.459 ms）。产物整行（门槛 = 省下的 ÷ 提交次数，machine_scaled 再乘 1.199658）：
```text
E7RESULT name=candidate_threshold fraction=0.75 pool_data_units=1024 batch_units=4 candidate=every_stage saved_per_intent_milliseconds=1.846171 submissions_per_intent=7 threshold_milliseconds=0.263739
E7RESULT name=candidate_threshold fraction=0.75 pool_data_units=8192 batch_units=4 candidate=every_stage saved_per_intent_milliseconds=3.960971 submissions_per_intent=7 threshold_milliseconds=0.565853
```
- 这一段随 P 涨的来源是线性找：`crates/singlefs-core/src/region_scan.rs:288`：「.filter(|entry| entry.points_at(record.device, record.slot))」，`crates/singlefs-core/src/region_scan.rs:311`：「.any(|entry| entry.key == header.mapping_key);」（两处前一行都是 `.iter()`，第 287、310 行）。`.claude/kb/milestone/04-fourth-txn.md:174`：「判据是按函数拆开的一批 CPU 里扫描占几成」。
- 第三段（`function-split`）答的是份额，D26 比的是绝对毫秒（`.claude/kb/decisions/26-后台整理与放置回收.md:251`：「GPU 模式的目的是少占主机 CPU 时间」）。同一点（f = 0.75、P = 8192、K = 8）的两行，扫描 1.46 ms（批）与 0.76 ms（签发）对着 C 在这一点 0.688038 ms（attack_ledger；machine_scaled 再乘 1.199658）的门槛，份额只有 0.004，绝对值却和门槛同量级：
```text
E7RESULT name=function_split_summary fraction=0.75 pool_data_units=8192 batch_units=8 scope=batches rounds=5 call_cpu_nanoseconds_median=338405211.000000 call_share_median=1.000000 clone_cpu_nanoseconds_median=94596956.000000 clone_share_median=0.280496 read_the_version_cpu_nanoseconds_median=469239280.000000 read_the_version_share_median=1.390565 region_scan_cpu_nanoseconds_median=1464437.000000 region_scan_share_median=0.003878 choose_cpu_nanoseconds_median=0.000000 choose_share_median=0.000000 remainder_cpu_nanoseconds_median=-227188750.000000 remainder_share_median=-0.675966
E7RESULT name=function_split_summary fraction=0.75 pool_data_units=8192 batch_units=8 scope=issue rounds=5 call_cpu_nanoseconds_median=86476993.000000 call_share_median=1.000000 clone_cpu_nanoseconds_median=31715083.000000 clone_share_median=0.366594 read_the_version_cpu_nanoseconds_median=155919578.000000 read_the_version_share_median=1.803361 region_scan_cpu_nanoseconds_median=760564.000000 region_scan_share_median=0.008781 choose_cpu_nanoseconds_median=385112.000000 choose_share_median=0.004444 remainder_cpu_nanoseconds_median=-102441596.000000 remainder_share_median=-1.182773
```
  这两行的 `read_the_version_share_median` 大于 1、`remainder_share_median` 为负：重放比真调用贵，份额不是划分，读这一版、克隆两项只能当上界。
- 代理数（不是量的）：把 C 在 P = 8192 的乙换成 P = 1024 的量，当作乙改成按身份查之后的上界，重判（`cfix.out`，c_rt 在两个 P 下逐字相同，所以只换门槛）：
```text
('0', 'attack_ledger', 'losing') 5
('0', 'attack_ledger', 'not_losing') 12
('0', 'attack_ledger', 'unstable') 37
('0', 'machine_scaled', 'losing') 1
('0', 'machine_scaled', 'not_losing') 25
('0', 'machine_scaled', 'unstable') 28
('1', 'attack_ledger', 'losing') 32
('1', 'attack_ledger', 'not_losing') 5
('1', 'attack_ledger', 'unstable') 17
('1', 'machine_scaled', 'losing') 25
('1', 'machine_scaled', 'not_losing') 13
('1', 'machine_scaled', 'unstable') 16
```
  C 在卡 0（machine_scaled）的 `not_losing` 从 52 / 54 掉到 25 / 54，卡 1 从 39 / 54 掉到 13 / 54。结论：「只剩 C」有四成到六成靠乙在大池上的线性找，把线性找修掉（登记在 04-fourth-txn.md 延后项）之后 C 与 B 同形。
- 推翻它的现象：身份索引实现后实量乙增量在 P = 8192 仍不低于 2 ms；或第三段的重放修成可划分之后扫描绝对值不随 P 涨。

**G1-②　发布那一半**。`research/prompts/m4-gpugc-design-r1-main-verification.md:34`：「发布那一半里没有能放 GPU、又不碰共享事务层的部分，两条腿一致」。
- 代码：释放前读盘核住在 `crates/singlefs-core/src/transaction.rs:11224` 的 `prepare_the_version_publish_relocating_units`，别的发布从 `crates/singlefs-core/src/transaction.rs:11152` 的 `prepare_the_version_publish` 进同一个函数。搬走源落点的那一圈是单独一个循环：`crates/singlefs-core/src/transaction.rs:11420`：「搬走的源落点释放之前同样按映射位置项读盘核」，`crates/singlefs-core/src/transaction.rs:11422`：「for (position, relocation) in settled.relocations_in_effect.iter().enumerate() {」，别的发布这一圈空转。
- 判定：站得住，但只打中 D。A、B、C 的核在 `region_scan.rs`、`compaction_plan.rs`，不在 `transaction.rs`，不碰共享事务层。D 的插入点是上面那一圈，改动落在共享函数里，但只经 `relocations` 参数走到。D 的死因不是这一条，是第一段的数（见第三节）。
- 推翻它的现象：把那一圈提成只被 `publish_relocation_batch` 调的函数、共享函数的签名与别的发布的行为逐字不变。

**G1-③　前台路径**。`research/prompts/m4-gpugc-design-r1-main-verification.md:34`：「GC 先挪出调用线程是 G1 任何推荐的前提」。
- 代码：`crates/singlefs-core/src/mounted_session.rs:1927`：「if compaction_starts_after_a_user_change(allocator.compaction().switches.run_forcing) {」，`crates/singlefs-core/src/mounted_session.rs:1935`：「CompactionRunReach::AtMostOneBatchAfterAUserChange,」（判决写 1928–1931，现在是 1927–1936，行号挪了，内容同）。产品路径今天不起跑：`crates/singlefs-core/src/compaction_plan.rs:818`：「第一版三条水位恒 0（关），产品路径恒不起跑」，`crates/singlefs-core/src/compaction_plan.rs:825`：「CompactionRunForcing::ForcedToStartForTests => true,」。
- 判定：站得住，同样打中 A–D 全部（它们都在同一个函数里起提交）；不打中「GPU 不进用户改动之后那一趟」的混合形。被新读数加强：`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`：「PCIe 往返是几十微秒量级，前台读写延迟吃不起这个」，第一段 4–32 个单元的一次提交，每格（卡 × 等法 × 间隔，共用输入）取 5 轮中位后的挂钟是 5.46–8.05 ms（84 格，中位 6.34 ms；`wall.py` 与上面同一份产物，第四节），差两个量级。
- 推翻它的现象：七·3 接上水位之后，用户改动之后那一趟被证明只走 CPU（或水位在用户路径上永不越线）。

G1 附：同一统计里登记读法的那一半（`cfix.out` 前 13 行，C 在 P = 8192、线程钟，键 = 卡、门槛种类、判定）：
```text
keys 1342 with identical c_rt across P=1024/8192: 1342
C at P=8192, as registered:
('0', 'attack_ledger', 'not_losing') 47
('0', 'attack_ledger', 'unstable') 7
('0', 'machine_scaled', 'not_losing') 52
('0', 'machine_scaled', 'unstable') 2
('1', 'attack_ledger', 'losing') 15
('1', 'attack_ledger', 'not_losing') 28
('1', 'attack_ledger', 'unstable') 11
('1', 'machine_scaled', 'losing') 10
('1', 'machine_scaled', 'not_losing') 39
('1', 'machine_scaled', 'unstable') 5
C at P=8192 with the scan part replaced by the P=1024 level (proxy for an identity index):
```

### G2（判决第 35 行）

`research/prompts/m4-gpugc-design-r1-main-verification.md:35`：「每批之前都重扫」「乙-1 在两条腿下都压不下固定开销，净赚与否同样看 c_rt」。三条串行约束的代码依据逐条现查：
- `crates/singlefs-core/src/mounted_session.rs:2975`：「对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）」。
- `crates/singlefs-core/src/transaction.rs:9628`：「签发时上一版已有一条活整理意图（全池至多一条，判决 G1 e1）」。
- `crates/singlefs-core/src/compaction_plan.rs:806`：「`3S − 4 − 已经过去的发布数`，至少 1。」
- 放宽 R 要改的字段：`crates/singlefs-core/src/logged_operations.rs:303`：「if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {」，对着 `.claude/kb/decisions/26-后台整理与放置回收.md:71`：「恒 64（一个聚簇段），别的值拒收」。判决第 35 行写的是「slot_count != CLUSTER_SEGMENT_SLOTS」，实际代码多一层 `u64::from(…)`，内容同，行号对。

判定：约束都是真的。同样打中的臂：乙-2（跨意图预取）与丙（放宽 R）；不打中甲（一批一交）与乙-1（同一条意图内一次交）。

新读数对「跨批攒不成」要不要改判：第 3 次重跑第一段的 `merging_shape`（门槛是 C 的，所以只对 C 说话，对 A、B、D 不说话）：
```text
$ grep 'name=merging_shape' $F | awk '{for(i=1;i<=NF;i++){split($i,a,"=");if(a[1]=="threshold_kind")tk=a[2];if(a[1]=="category")c=a[2]}; print tk, c}' | sort | uniq -c
      1 attack_ledger fit_not_linear
      1 attack_ledger merge_across_intents
     92 attack_ledger one_submission_per_batch
     44 attack_ledger one_submission_per_intent
     78 attack_ledger unstable
    125 machine_scaled one_submission_per_batch
     34 machine_scaled one_submission_per_intent
     57 machine_scaled unstable
$ grep 'name=merging_shape' $F | awk '{for(i=1;i<=NF;i++){split($i,a,"=");if(a[1]=="category_per_round"){n=split(a[2],r,",");for(j=1;j<=n;j++)c[r[j]]++}}} END{for(k in c)print k,c[k]}'
one_submission_per_intent 680
merge_across_intents 26
one_submission_per_batch 1449
fit_not_linear 5
```
唯一一行 `merge_across_intents`（产物第 3988 行整行）：
```text
E7RESULT name=merging_shape fraction=0.75 pool_data_units=1024 batch_units=16 index=1 form=shared wait=sleep_100_microseconds gap_milliseconds=50 threshold_kind=attack_ledger threshold_milliseconds=0.376500 saved_per_intent_milliseconds=1.129000 one_submission_per_intent_units=24 one_submission_per_intent_host_cpu_milliseconds_per_round=1.450668,1.258273,1.257293,1.507853,1.390843 per_byte_part_milliseconds_per_round=1.102815,1.095399,1.094280,1.095610,1.097943 intents_to_merge_at_least_per_round=6,6,7,10,6 fit_not_linear=false category_per_round=merge_across_intents,merge_across_intents,merge_across_intents,merge_across_intents,merge_across_intents category=merge_across_intents
```
读法：432 行里只有这 1 行（卡 1、间隔 50 ms、f = 0.75、P = 1024、K = 16、睡眠 100 µs）要跨意图合并，每轮要 6–10 条意图；`one_submission_per_batch` 217 行说明在 C 的门槛下一批一交（甲）就够。所以判决「跨批攒被三条约束挡住」的结论不必改判，也几乎不被用到。
拟合：18 行 `fixed_and_per_byte_fit` 里 9 行 `fit_not_linear=true`（`grep -c 'fit_not_linear=true'` 数出 9）。每 MiB 的主机 CPU 对照 CPU 算同样字节的量（产物第 122、129 行）：
```text
E7RESULT name=cpu_crc_of_the_same_bytes units=4 runs=21 thread_cpu_median_nanoseconds=46110.000000
E7RESULT name=cpu_crc_of_the_same_bytes units=512 runs=21 thread_cpu_median_nanoseconds=5938355.000000
E7RESULT name=fixed_and_per_byte_fit index=0 form=shared wait=blocking gap_milliseconds=0 fixed_milliseconds=0.193968 per_mebibyte_milliseconds=0.061909 maximum_relative_residual=0.217393 points=8 fit_not_linear=true fixed_milliseconds_per_round=0.193968,0.163527,0.277343,0.170961,0.043057 per_mebibyte_milliseconds_per_round=0.061909,0.072083,0.054108,0.066628,0.048419
E7RESULT name=fixed_and_per_byte_fit index=1 form=shared wait=blocking gap_milliseconds=0 fixed_milliseconds=0.056578 per_mebibyte_milliseconds=0.487637 maximum_relative_residual=0.218780 points=8 fit_not_linear=true fixed_milliseconds_per_round=0.039877,0.113204,0.079952,0.089129,0.065607 per_mebibyte_milliseconds_per_round=0.486705,0.491310,0.481823,0.488662,0.486913
E7RESULT name=fixed_and_per_byte_fit index=1 form=shared wait=blocking gap_milliseconds=50 fixed_milliseconds=0.106233 per_mebibyte_milliseconds=1.463544 maximum_relative_residual=0.449607 points=8 fit_not_linear=true fixed_milliseconds_per_round=0.082673,0.138820,0.171610,0.129309,0.038117 per_mebibyte_milliseconds_per_round=1.465051,1.456726,1.462070,1.464859,1.462944
```
CPU 一侧 4 单元（128 KiB）46110 ns、512 单元（16 MiB）5938355 ns，都约 0.37 ms 每 MiB。卡 0 共用输入、阻塞等、间隔 0 的每 MiB 部分 0.061909 ms，固定部分 0.193968 ms：用这两个数（残差 0.217393，超 20%，只当量级）推，一次提交要到约 0.63 MiB（约 20 个单元）才抵得过固定部分，一条意图 f = 0.75 的 24 个单元刚过，f = 0.25 的 8 个单元不够。卡 1 同口径每 MiB 部分 0.487637 ms，高于 CPU 的 0.37，再攒也不抵。
判定：站得住（约束为真），不决定什么（用到它的只有 1 行）。推翻它的现象：量到第二批跨意图预取在某卡上净赚且不撞三条约束之外的东西。

### G3（判决第 36 行，「没打中」，核有没有漏判）

`research/prompts/m4-gpugc-design-r1-main-verification.md:36`：「没打中，两处脆弱记进第二轮」。两处脆弱是软件适配器（推的，没起 GPU）与故障下一张卡 / 几张卡判定不同。这一轮没有起 GPU，那两处我也没法现查，原样转述。新读数里有第三处，判决没有：
```text
E7RESULT name=gpu_card index=0 card_name=NVIDIA_GeForce_RTX_5090 driver=NVIDIA_595.99.02 free_memory_mebibytes=32094 link_peak_bytes_per_second=63015384615 resident_capacity_units=512 threads_after_the_card_started=6
E7RESULT name=gpu_card index=1 card_name=NVIDIA_GeForce_RTX_5060_Ti driver=NVIDIA_595.99.02 free_memory_mebibytes=15849 link_peak_bytes_per_second=15753846154 resident_capacity_units=512 threads_after_the_card_started=6
E7RESULT name=knob_flip_summary quantity=host_cpu_per_submission knob=card clause=two_cards_judge_differently pairs=1096 flipped=630
```
- 卡 1 的链路峰值是卡 0 的四分之一（`link_peak_bytes_per_second` 15753846154 对 63015384615），每 MiB 主机 CPU 高于 CPU 自己算（上节）。直接数，不经拟合（`wall.out` 里 16 MiB 一档，共用输入、阻塞等、间隔 0 / 50 ms；CPU 算同样 16 MiB 是 5.94 ms）：
```text
$ grep -E '^(0|1) shared blocking (0|50) 512 ' /home/fy5090/.local/share/singlefs/work/m4-gpugc-design-r2-sonnet/run1/out/wall.out
0 shared blocking 0 512 5 wall 7.219 min 6.855 max 7.355 cpu 1.185
0 shared blocking 50 512 5 wall 7.447 min 7.395 max 7.743 cpu 1.356
1 shared blocking 0 512 5 wall 14.033 min 13.963 max 14.127 cpu 7.859
1 shared blocking 50 512 5 wall 31.915 min 31.826 max 31.956 cpu 23.517
```
  卡 1 的线程 CPU 7.859 ms（间隔 0）与 23.517 ms（间隔 50 ms）都高于 CPU 的 5.94 ms；卡 0 是 1.185 与 1.356 ms。所以「按窗口轮转」（正推腿推荐，判决第 36 行转述为「按窗口轮转」）把一半窗口派给一张在大提交上只赔不赚的卡。
- 小提交上看不出：一批 8 个单元、共用输入的线程 CPU 中位卡 0 是 0.116–0.267 ms、卡 1 是 0.148–0.202 ms（`percard.out`），一个格内 5 轮的散布 0.06–0.49 ms，比卡间差大：
```text
thread_cpu_median_ms over valid rounds, gap 0, units 8 / 24(=none) : card0 | card1  [per-round list]
shared blocking | median 0.193 rounds [0.193, 0.218, 0.219, 0.13, 0.061] | median 0.148 rounds [0.148, 0.295, 0.294, 0.148, 0.148]
shared sleep_100_microseconds | median 0.267 rounds [0.225, 0.267, 0.146, 0.344, 0.358] | median 0.202 rounds [0.202, 0.335, 0.174, 0.241, 0.189]
shared sleep_1_millisecond | median 0.116 rounds [0.276, 0.238, 0.072, 0.116, 0.098] | median 0.164 rounds [0.149, 0.243, 0.144, 0.173, 0.164]
split blocking | median 0.198 rounds [0.249, 0.198, 0.279, 0.082, 0.144] | median 0.285 rounds [0.387, 0.274, 0.268, 0.405, 0.285]
split sleep_100_microseconds | median 0.204 rounds [0.316, 0.204, 0.491, 0.117, 0.089] | median 0.312 rounds [0.308, 0.422, 0.299, 0.312, 0.47]
split sleep_1_millisecond | median 0.183 rounds [0.19, 0.165, 0.362, 0.183, 0.06] | median 0.275 rounds [0.339, 0.275, 0.264, 0.351, 0.266]
```
- 「两张卡判定不同」的 630 / 1096 对里，`flips.out` 数出只有 36 对是 `not_losing` 对 `losing` 的硬翻面，594 对牵涉 `unstable`：
```text
('card', 'hard') 36
('card', 'involves_unstable_or_bound') 594
('clock', 'involves_unstable_or_bound') 54
('gap', 'hard') 50
('gap', 'involves_unstable_or_bound') 295
('input_form', 'hard') 5
('input_form', 'involves_unstable_or_bound') 239
('threshold_machine', 'involves_unstable_or_bound') 214
('wait_blocking_and_sleep_100_microseconds', 'hard') 1
('wait_blocking_and_sleep_100_microseconds', 'involves_unstable_or_bound') 227
('wait_blocking_and_sleep_1_millisecond', 'involves_unstable_or_bound') 153
('wait_sleep_100_microseconds_and_sleep_1_millisecond', 'involves_unstable_or_bound') 195
```
判定：漏判一处，量级要说清——大提交上卡 1 的每 MiB 成本高于 CPU 是直接读数；一批只有 4–32 个单元时看不出，两张卡判定不同的大多是 `unstable` 对 `losing`，不是硬翻面。同样打中的臂：「全部显卡」、按窗口轮转，以及任何把大提交派给卡 1 的分法；不打中只用卡 0 的分法。
推翻它的现象：卡 1 换到更宽的链路或别的驱动后，同口径每 MiB 部分不高于 CPU 的 0.37 ms；或一批大小的小提交上多轮读数把两张卡拉开。

### G4（判决第 37 行）

`research/prompts/m4-gpugc-design-r1-main-verification.md:37`：「输入表准备这一类，七·3 写的同输入对拍抓不到」「② 是条款与代码对不上，下一轮核」。

**①　输入表准备这一类**。判定：站得住，同样打中所有「两份核读同一份准备好的输入」的方案（两份核共用输入、各备一份输入但由同一段主机代码展平、分在两张卡，三种都一样）；不打中「输入表也由两条独立代码路径各备」的方案，那一种要多付主机 CPU（岔路单第 5 行乙）。第二段罩到哪、罩不到哪（登记原文 `research/prompts/e185-r3-prereg.md:248`：「每个单元比三样：核一对核二、核一对 CPU、核二对 CPU」）：
- 罩到：同一份已经在内存里的字节，上传、两份核算、读回、与 CPU 在同一份字节上算的值比。上传路径、驱动、着色器编译、核的算术都在里面。
- 罩不到：字节是不是该算的那一段（偏移与长度表怎么由扫描展平、哪些单元进这一批）、真单元头的内容（第二段的字节是 xorshift 随机造的，偶数号单元 32768 字节、奇数号 16384 字节）、一批 4–32 个单元的小批（第二段每批 16384 个单元）、CPU 参照自己出错（参照是 `crates/singlefs-core/src/checksum.rs:3`：「实现用 slicing-by-8」那一份）。
```text
E7RESULT name=configuration segment=recompute_disagreement cards=0,1 batches=64 units_per_batch=16384 recomputes_at_most=3 injected_batches=1..=8 deterministic_injection_unit=7 transient_injection_unit=11 worker_threads=31 card_quota_mebibytes=4096
E7RESULT name=disagreement_card_summary index=0 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
E7RESULT name=disagreement_card_summary index=1 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
E7RESULT name=both_cards_same_input cards=0,1 units=16384 identical=16384 cards_differ=false
```
所以第二段的零次不动这一条：它量的是对拍抓得到的那一类。推翻它的现象：一份「输入表漏掉几条」的注入在同输入对拍里被抓出来（判决引的攻方模型 g4 复跑是 0 处不同）。

**②　「CPU 只改位图，不再抽样复核」对着今天的发布路径**。`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「CPU 只改位图，不再抽样复核」。代码现查：
- 计划与上一版对不上的四种错在任何读写之前交回：`crates/singlefs-core/src/transaction.rs:9609`：「PlannedUnitNotInTheVersion {」，`crates/singlefs-core/src/transaction.rs:9618`：「PlannedSourceIsNotTheMappingPosition {」，`crates/singlefs-core/src/transaction.rs:9625`：「UnitBytesNotAvailable {」，另有钉住那一种；调用点 `crates/singlefs-core/src/transaction.rs:10179`：「return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });」。
- 释放之前再读盘核一次源落点的校验和：`crates/singlefs-core/src/transaction.rs:11361`：「经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和」，搬走的源落点同样（`crates/singlefs-core/src/transaction.rs:11420`）。
- 判定：对「位图把不该搬的单元标成可搬」够不着：计划里的项要对得上上一版映射、钉住、源落点，字节要拿得到，源落点释放前还读盘核。够得着的是「位图把该搬的标成死 / 不搬」，只会让意图完不成（活性），不是损坏；这一条与判决第 37 行写的「条款与代码对不上」一致，但损坏方向被代码挡住。发布路径先写目的地还是先核源落点的次序我没读到，这一点留给下一轮。
- 推翻它的现象：读出发布里目的地先于源落点读盘核落盘，且一份校验和不对的源字节被搬走而没被隔离。

**N 与计数住哪**。`.claude/kb/decisions/26-后台整理与放置回收.md:252`：「N 与几张卡时怎么分在里程碑四设计轮（七·2）定」。第二段每张卡两种输入各 1048560、合 2097120 个单元次首轮零次不一致（单侧 95% 上界 1.428497e-6，产物第 176、313 行），两张卡同输入逐条相同（第 315 行）。登记原文 `research/prompts/e185-r3-prereg.md:298`：「zero 时 N 的取值不由这个量翻面，只报上界」。注入的 8 个确定性单元与 8 个瞬时单元分类全对（第 109 行 `deterministic_injection_holds=true`、`transient_injection_holds=true`，四组同样）：
```text
E7RESULT name=disagreement_summary index=0 form=shared natural_unit_times=1048560 expected_natural_unit_times=1048560 natural_unit_times_sufficient=true first_round_unreadable=0 first_round_kernel_disagreements=0 first_round_any_difference=0 first_round_rate=0.000000000e0 upper_bound_at_95_percent_when_zero=2.856992e-6 both_kernels_agree_and_differ_from_the_cpu=0 kernel_one_differs_from_the_cpu=0 kernel_two_differs_from_the_cpu=0 still_disagreeing_after_one_recompute=0 still_disagreeing_after_two_recomputes=0 still_disagreeing_after_three_recomputes=0 unreadable_recomputes=0 deterministic_injection_outcomes=deterministic,deterministic,deterministic,deterministic,deterministic,deterministic,deterministic,deterministic transient_injection_outcomes=transient,transient,transient,transient,transient,transient,transient,transient deterministic_injection_holds=true transient_injection_holds=true first_round_disagreements_peak_of_a_batch=0 batches_with_first_round_disagreements=0 first_round_disagreements_of_the_last_batch=0 category=zero
```
算术（推的）：上界 1.428497e-6 乘 32 个单元一批得每批至多 4.6e-5 条不确定。计数住内存、重挂清零的代价因此上界很小；零次不能证明「等几轮也不好」的那类持续故障不存在，只有注入的 8 个确定性单元证明分类走得通。判定：N 取几不由这个数翻面；住内存还是盘上不由这个数翻面（要由持续故障的发生率翻，没有这个数）。

### G5（判决第 38 行，「没打中」）

`research/prompts/m4-gpugc-design-r1-main-verification.md:38`：「没打中，脆弱那一处与 G1-③ 同一件事」。现查：
- `crates/singlefs-core/src/mounted_session.rs:2641`：「let switches = allocator.compaction().switches;」，一趟开头读一次。
- 单线程独占：
```text
$ grep -c 'std::thread\|thread::spawn\|Mutex\|RwLock\|Condvar\|mpsc' crates/singlefs-core/src/mounted_session.rs
0
$ grep -rln 'thread::spawn\|std::thread::scope' crates/singlefs-core/src; echo "exit=$?"
exit=1
```
- 判定：没漏判关键的一处。补两处判决没写的：一，一趟的推法有三种（`crates/singlefs-core/src/mounted_session.rs:2542` 起的 `CompactionRunReach` 三个成员：显式入口、用户改动之后、可写挂载续做），判决只谈前两条入口；挂载续做用哪个后端，判决没写，正推腿 G5-c 列过。二，显式入口一趟最多推到发布预算（`crates/singlefs-core/src/compaction_plan.rs:806`：「`3S − 4 − 已经过去的发布数`，至少 1。」）次，开关在 2641 行只读一次，所以「从下一批生效」在显式入口里今天就是「下一趟」，只是 `&mut self` 让控制调用落不进一趟中间，没有可见后果。
- 推翻它的现象：七·3 加了不经 `&mut self` 的控制路径（共享句柄、另一条线程）而开关仍在一趟开头读一次。

### G6（判决第 39 行）

`research/prompts/m4-gpugc-design-r1-main-verification.md:39`：「GC 在用户改动的调用里同步跑，没有并发的 fsync 可等」。条款：`.claude/kb/decisions/26-后台整理与放置回收.md:250`：「前台有 fsync 在等时不发下一批」。代码：用户改动的入口是 `crates/singlefs-core/src/mounted_session.rs:1849`：「pub fn publish_user_change<Device: BlockDevice>(」，`&mut self`，核心层没有线程原语（上面 G5 的两条命令）；`publish_user_change` 之外没有名为 fsync 的入口。
- 判定：站得住。「同样打中所有替代方案」要拆开：不挪出时这条条款空真（没有别的等待者，永远不发「下一批」的前提不成立也不违反），挪出时才有可观测的东西，但要新接口；所以它打中的是「在今天的形态上给这条条款定默认值」这件事，不分臂。
- 写带宽上限压不住的那一半，新读数加强：GPU 一次提交的挂钟（5.46–8.05 ms，第四节）不在写带宽上限管得到的东西里。判决引的攻方模型 g6 输出（`research/prompts/m4-gpugc-design-r1-opus-model/out/g6.out`）：
```text
name=g6_batch arm=memory f=0.25 P=1024 K=4 batches=10 wall_ms_median=12.542 wall_ms_max=12.736 cpu_over_wall_median=1.0000 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.25 P=1024 K=4 batches=10 wall_ms_median=21.671 wall_ms_max=23.604 cpu_over_wall_median=0.6030 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.25 P=8192 K=8 batches=5 wall_ms_median=123.135 wall_ms_max=124.717 cpu_over_wall_median=0.8965 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
```
  500 MB/s 的上限只把一批的写摊成 1.5–2.8 ms（`paced_ms_at_500000000`），一批的挂钟是 12.5–123 ms，GPU 的往返另加 5.5–8 ms 且不在写里。
- 推翻它的现象：七·3 接线之后在用户改动的调用里量到有第二个调用者在等；或写带宽上限被改成按挂钟定而不是按写字节定。

### G7（判决第 40 行，「没打中」）

`research/prompts/m4-gpugc-design-r1-main-verification.md:40`：「真卡对拍归 checker 档、假后端进快档」是正推的推荐；判决认的差是七·3 验收写「对拍进快档」。两处原文现查：`.claude/kb/milestone/04-fourth-txn.md:145`：「对拍进快档」，`.claude/rules/verification.md:18`：「要真设备或外部工具」。判定：站得住，没漏判。
另有一个不在 G7 格内、但同样打中 A–D 全部候选的共同因素：CPU 侧的 CRC 是 slicing-by-8 的查表实现（`crates/singlefs-core/src/checksum.rs:3`：「实现用 slicing-by-8」），约 0.37 ms 每 MiB（上节产物第 122、129 行），所有候选的「省下的 CPU」都是按这个速度算的（门槛 = 省下的 ÷ 提交次数）。CPU 侧换成硬件 CRC 指令会把全部门槛按速度比压低，A、D 更活不了，C 同样；这一条没量，不进判定，进岔路表。推翻它的现象：量到 CPU 侧另一种 CRC 实现的每 MiB 成本。

## 二、第一轮推的五条改法：新读数推翻谁、撑住谁

五条在判决「三、下一步」第 2 条与正文第三节末点名；都是被攻过零轮的改法。一条一行结论，后面是依据的产物整行。

1. **按设备类型筛卡**（攻方 G3-①）：没被推翻，也没被撑住。第一段起的两张卡都是 NVIDIA 独显，没有软件适配器可量（本腿不起 GPU）。新读数补一条它挡不住的：筛完仍剩一张赔的独显（上面 G3）。
```text
E7RESULT name=gpu_card index=0 card_name=NVIDIA_GeForce_RTX_5090 driver=NVIDIA_595.99.02 free_memory_mebibytes=32094 link_peak_bytes_per_second=63015384615 resident_capacity_units=512 threads_after_the_card_started=6
E7RESULT name=gpu_card index=1 card_name=NVIDIA_GeForce_RTX_5060_Ti driver=NVIDIA_595.99.02 free_memory_mebibytes=15849 link_peak_bytes_per_second=15753846154 resident_capacity_units=512 threads_after_the_card_started=6
```
2. **对拍从这一版起算**（攻方 G4-①）：没被推翻，新读数没碰这一类。第二段是同一份字节上两份核对 CPU 的对拍，零次不一致，正是改法说抓不到输入表准备的那一类，它对这条改法既不撑也不推（上面 G4-①）。
```text
E7RESULT name=disagreement_card_summary index=0 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
```
3. **重挂时把不确定的条目直接交 CPU**（攻方 G4-③）：被撑住，只在代价一侧。两张卡 2097120 个单元次零次首轮不一致，每批（32 个单元）至多 4.6e-5 条不确定（推的），重挂清零与直接交 CPU 的主机 CPU 代价上界很小；持续故障的一侧没数。
```text
E7RESULT name=disagreement_card_summary index=1 natural_unit_times=2097120 first_round_kernel_disagreements=0 upper_bound_at_95_percent_when_zero=1.428497e-6
```
4. **挂载时用已知答案自检**（攻方 G4-④）：被撑住，只在能走通一侧。9 字节检验串在两张卡、共用输入与分开输入两种形态上，两份核与期望都是 0xe3069283。没有「坏核被自检抓出」的阴性对照（`name=positive_control row=comparison` 两行测的是比对抓得到一个翻位，不是自检）。
```text
E7RESULT name=positive_control row=check_string index=0 form=shared kernel_one=0xe3069283 kernel_two=0xe3069283 expected=0xe3069283 sequence_numbers_hold=true holds=true
E7RESULT name=positive_control row=check_string index=0 form=split kernel_one=0xe3069283 kernel_two=0xe3069283 expected=0xe3069283 sequence_numbers_hold=true holds=true
E7RESULT name=positive_control row=comparison index=0 form=shared units=8 flipped_entry=2 uncertain_entries=[2] holds=true
E7RESULT name=positive_control row=check_string index=1 form=shared kernel_one=0xe3069283 kernel_two=0xe3069283 expected=0xe3069283 sequence_numbers_hold=true holds=true
E7RESULT name=positive_control row=check_string index=1 form=split kernel_one=0xe3069283 kernel_two=0xe3069283 expected=0xe3069283 sequence_numbers_hold=true holds=true
E7RESULT name=positive_control row=comparison index=1 form=shared units=8 flipped_entry=2 uncertain_entries=[2] holds=true
```
5. **GC 挪出调用线程**（判决 G1-③）：被撑住一半。一次提交的挂钟 5.46–8.05 ms（第四节），前台路径装不下，撑住「不挪出就违 D24 已定项 2 第 2 条字面」。没撑住的一半：它量的是主机线程 CPU，挪到别的线程不改这个数（`.claude/kb/decisions/26-后台整理与放置回收.md:251` 比的就是主机 CPU）；代价数没有，核心层零线程原语（G5 两条命令）。
```text
E7RESULT name=submission_cell index=0 form=shared wait=blocking gap_milliseconds=0 units=4 round=1 attempt=1 thread_cpu_median_nanoseconds=208750.500000 floor_thread_cpu_median_nanoseconds=159055.500000 process_cpu_median_nanoseconds=228870.500000 wait_thread_cpu_median_nanoseconds=48965.000000 wait_wall_median_nanoseconds=5348923.500000 wall_median_nanoseconds=5508723.500000 polls_median=0.000000 input_bytes_written=131072 input_bytes_written_holds=true results_agree_with_the_cpu=true sequence_numbers_hold=true uncertain_entries=0 thread_cpu_maximum_nanoseconds=256500.000000 thread_cpu_percentile_90_nanoseconds=223851.000000 submissions_at_or_above_the_smallest_threshold=31 submissions_at_or_above_the_largest_threshold=0 thread_cpu_first_nanoseconds=135571.000000 thread_cpu_last_nanoseconds=222750.000000 respects_the_physical_floor=true floor_violations=0 wait_inside_the_submission=true comparison_counts_hold=true thread_clock_underestimated=false calibration_ratio_before=1.039980 calibration_ratio_after=1.069092 threads_before=6 threads_after=6 invalid_reason=none
```

## 三、候选 A、B、D 有没有哪一种读法能活

产物：`research/results/e185-gc-batch-wall-clock-split-host-cpu-per-submission-r3-2026-10-24-2.out`。复跑命令见开头；下面每段输出是 `run1/out/` 里对应脚本的原样输出（`tally.py` 逐候选 × 卡 × 钟 × 门槛数 `judgement` 与 `floor_judgement`；`tally3.py` 数每轮；`readings.py` 换读法数；`bywait.py` 按等法 × 间隔拆；`bypoint.py` 按点拆）。

### 3.1　登记读法（5 轮全一致才算）：复现正文透视

线程钟、`judgement` 列（`tally.out` 里 A、B、D 的行，B 的 `not_losing_lower_bound` 单列）：
```text
$ awk '/^## judgement/{f=1;next} /^## floor_judgement/{f=0} f && /thread/ && /^(scan_checksum_only|scan_checksum_and_choice|release_check_only)/' tally.out
release_check_only card 0 thread attack_ledger {'losing': 34, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 74}
release_check_only card 0 thread machine_scaled {'losing': 30, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 78}
release_check_only card 1 thread attack_ledger {'losing': 108, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 0}
release_check_only card 1 thread machine_scaled {'losing': 108, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 0}
scan_checksum_and_choice card 0 thread attack_ledger {'losing': 21, 'not_losing': 0, 'not_losing_lower_bound': 10, 'unstable': 77}
scan_checksum_and_choice card 0 thread machine_scaled {'losing': 8, 'not_losing': 0, 'not_losing_lower_bound': 27, 'unstable': 73}
scan_checksum_and_choice card 1 thread attack_ledger {'losing': 78, 'not_losing': 0, 'not_losing_lower_bound': 4, 'unstable': 26}
scan_checksum_and_choice card 1 thread machine_scaled {'losing': 62, 'not_losing': 0, 'not_losing_lower_bound': 10, 'unstable': 36}
scan_checksum_only card 0 thread attack_ledger {'losing': 62, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 46}
scan_checksum_only card 0 thread machine_scaled {'losing': 46, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 62}
scan_checksum_only card 1 thread attack_ledger {'losing': 108, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 0}
scan_checksum_only card 1 thread machine_scaled {'losing': 108, 'not_losing': 0, 'not_losing_lower_bound': 0, 'unstable': 0}
```
- A、D：两张卡、三种等法、两种门槛，`not_losing` 一格都没有（正文透视成立）。卡 1 上 A、D 全 losing（各 108 行），卡 0 上 losing 与 unstable 各占一部分。
- B：只有 `not_losing_lower_bound`（卡 0 10 与 27 行，卡 1 4 与 10 行，attack_ledger 与 machine_scaled 前后），不是不赔的证明（丙的输入没量）。
- 进程钟（含驱动起的线程）更差：A 卡 0 进程钟 `losing` 48 / 32、`unstable` 36 / 52、`not_losing` 0；D 卡 0 `losing` 16 / 8、`unstable` 58 / 66（`tally.out` 同名行）。

### 3.2　换读法数一遍（A、D、B 在卡 0 / 卡 1 108 行里的活格数）

`readings.py`：每行 = 候选 × 卡 × 门槛，数的是 5 轮数里小于门槛的行数；`median5` 5 轮中位、`min5` 最好的一轮、`max5` 最差的一轮、`round5` 只看最后一轮、`round1` 只看第一轮；第二块用去掉等的那一段的 `floor_` 数。
```text
### host_cpu_per_submission_milliseconds_per_round
A card 0 attack_ledger of 108 median5=2 min5=46 max5=0 mean5=2 round5=36 round1=6
A card 0 machine_scaled of 108 median5=10 min5=62 max5=0 mean5=12 round5=52 round1=10
A card 1 attack_ledger of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
A card 1 machine_scaled of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
B card 0 attack_ledger of 108 median5=60 min5=87 max5=10 mean5=60 round5=76 round1=40
B card 0 machine_scaled of 108 median5=70 min5=100 max5=27 mean5=72 round5=86 round1=61
B card 1 attack_ledger of 108 median5=20 min5=30 max5=4 mean5=16 round5=24 round1=20
B card 1 machine_scaled of 108 median5=39 min5=46 max5=10 mean5=32 round5=40 round1=32
D card 0 attack_ledger of 108 median5=6 min5=74 max5=0 mean5=5 round5=46 round1=14
D card 0 machine_scaled of 108 median5=18 min5=78 max5=0 mean5=12 round5=62 round1=28
D card 1 attack_ledger of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
D card 1 machine_scaled of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
### floor_host_cpu_per_submission_milliseconds_per_round
A card 0 attack_ledger of 108 median5=34 min5=80 max5=4 mean5=30 round5=64 round1=26
A card 0 machine_scaled of 108 median5=40 min5=82 max5=8 mean5=36 round5=70 round1=30
A card 1 attack_ledger of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
A card 1 machine_scaled of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
B card 0 attack_ledger of 108 median5=88 min5=101 max5=60 mean5=88 round5=99 round1=82
B card 0 machine_scaled of 108 median5=94 min5=106 max5=78 mean5=98 round5=102 round1=86
B card 1 attack_ledger of 108 median5=33 min5=36 max5=14 mean5=23 round5=31 round1=28
B card 1 machine_scaled of 108 median5=50 min5=56 max5=26 mean5=42 round5=46 round1=48
D card 0 attack_ledger of 108 median5=28 min5=82 max5=4 mean5=36 round5=62 round1=34
D card 0 machine_scaled of 108 median5=50 min5=84 max5=12 mean5=52 round5=80 round1=36
D card 1 attack_ledger of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
D card 1 machine_scaled of 108 median5=0 min5=0 max5=0 mean5=0 round5=0 round1=0
```
读：
- 「`unstable` 算活」（`min5`，任一轮小于门槛）：A 卡 0 46 / 62 行，D 74 / 78 行，卡 1 零行。这一读法把登记的「5 轮相同才写判定」整个丢了，只是把 `unstable` 当 `not_losing`。
- 中位小于门槛：A 2 / 10，D 6 / 18。
- 只看最后一轮（`round5`）：A 36 / 52，D 46 / 62；只看第一轮：A 6 / 10，D 14 / 28。后几轮活得多，前几轮活得少，见 3.4。
- 地板（`floor_` 块，去掉等的那一段，只在线程钟上有）：A 卡 0 中位小于门槛 34 / 40 行，D 28 / 50 行；卡 1 仍零行。地板只是下界：它假设等那一段的主机 CPU 能做到零，登记 R2 写：c_floor「losing 的点，赔与等法、核快慢都无关」，反过来不得。

### 3.3　按间隔与等法拆：活在哪、死在哪

`bywait.py`（machine_scaled、线程钟、共用输入；`median(c_rt/T)` 是该格全部行的中位比，小于 1 即活）：
```text
A card 0 blocking gap 0 n 24 median5<T 4 min5<T 24 median(c_rt/T)=1.54 median(c_floor/T)=1.21
A card 0 blocking gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=2.81 median(c_floor/T)=2.47
A card 0 sleep_100_microseconds gap 0 n 24 median5<T 0 min5<T 10 median(c_rt/T)=1.72 median(c_floor/T)=1.01
A card 0 sleep_100_microseconds gap 50 n 12 median5<T 0 min5<T 4 median(c_rt/T)=3.23 median(c_floor/T)=2.06
A card 0 sleep_1_millisecond gap 0 n 24 median5<T 6 min5<T 22 median(c_rt/T)=1.14 median(c_floor/T)=0.81
A card 0 sleep_1_millisecond gap 50 n 12 median5<T 0 min5<T 2 median(c_rt/T)=2.54 median(c_floor/T)=1.84
A card 1 blocking gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.22 median(c_floor/T)=2.07
A card 1 blocking gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=4.07 median(c_floor/T)=3.74
A card 1 sleep_100_microseconds gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.50 median(c_floor/T)=2.00
A card 1 sleep_100_microseconds gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=5.31 median(c_floor/T)=4.58
A card 1 sleep_1_millisecond gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.36 median(c_floor/T)=2.15
A card 1 sleep_1_millisecond gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=4.98 median(c_floor/T)=4.65
D card 0 blocking gap 0 n 24 median5<T 4 min5<T 24 median(c_rt/T)=1.34 median(c_floor/T)=1.02
D card 0 blocking gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=2.31 median(c_floor/T)=2.04
D card 0 sleep_100_microseconds gap 0 n 24 median5<T 10 min5<T 20 median(c_rt/T)=1.24 median(c_floor/T)=0.73
D card 0 sleep_100_microseconds gap 50 n 12 median5<T 0 min5<T 8 median(c_rt/T)=1.88 median(c_floor/T)=1.32
D card 0 sleep_1_millisecond gap 0 n 24 median5<T 4 min5<T 24 median(c_rt/T)=1.17 median(c_floor/T)=0.89
D card 0 sleep_1_millisecond gap 50 n 12 median5<T 0 min5<T 2 median(c_rt/T)=1.77 median(c_floor/T)=1.27
D card 1 blocking gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.25 median(c_floor/T)=2.11
D card 1 blocking gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=4.02 median(c_floor/T)=3.94
D card 1 sleep_100_microseconds gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.39 median(c_floor/T)=1.94
D card 1 sleep_100_microseconds gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=4.64 median(c_floor/T)=4.25
D card 1 sleep_1_millisecond gap 0 n 24 median5<T 0 min5<T 0 median(c_rt/T)=2.15 median(c_floor/T)=1.99
D card 1 sleep_1_millisecond gap 50 n 12 median5<T 0 min5<T 0 median(c_rt/T)=4.59 median(c_floor/T)=4.37
```
读：间隔 50 ms（登记里「产品里两批之间隔着一批的 CPU」的推的取值，`research/prompts/e185-r3-prereg.md` 第一节「间隔」一行）下，A、D 在三种等法的全部格中位都不小于门槛（格中位比 1.77–5.31），只有睡眠等里有极少数「最好的一轮」小于门槛（A 共 6 行、D 共 10 行）。间隔 0（连发）下 A 有 10 行、D 有 18 行中位小于门槛，全在卡 0。

### 3.4　哪一个点上 A、D 活得最像样，以及它活在哪几轮

`bypoint.py`（卡 0、间隔 0、线程钟、machine_scaled；每个 (f, K) 12 行 = 3 种等法 × 2 种输入形态 × 2 个 P，`median<T` 与 `min<T` 是 12 行里数的行数，花括号是登记的 `judgement`）：
```text
A f 0.25 K 4 n 12 T ['0.082496', '0.082553'] median<T 0 min<T 6 {'unstable': 6, 'losing': 6}
A f 0.25 K 8 n 12 T ['0.098744', '0.098907'] median<T 0 min<T 10 {'unstable': 10, 'losing': 2}
A f 0.25 K 16 n 12 T ['0.098816', '0.098936'] median<T 0 min<T 10 {'unstable': 10, 'losing': 2}
A f 0.75 K 4 n 12 T ['0.190987', '0.191646'] median<T 2 min<T 10 {'unstable': 10, 'losing': 2}
A f 0.75 K 8 n 12 T ['0.222823', '0.223074'] median<T 4 min<T 10 {'unstable': 10, 'losing': 2}
A f 0.75 K 16 n 12 T ['0.230847', '0.231263'] median<T 4 min<T 10 {'unstable': 10, 'losing': 2}
D f 0.25 K 4 n 12 T ['0.099290', '0.099542'] median<T 0 min<T 10 {'unstable': 10, 'losing': 2}
D f 0.25 K 8 n 12 T ['0.197789', '0.198231'] median<T 2 min<T 12 {'unstable': 12}
D f 0.25 K 16 n 12 T ['0.198305', '0.198533'] median<T 2 min<T 12 {'unstable': 12}
D f 0.75 K 4 n 12 T ['0.099058', '0.099211'] median<T 0 min<T 10 {'unstable': 10, 'losing': 2}
D f 0.75 K 8 n 12 T ['0.197849', '0.197881'] median<T 2 min<T 12 {'unstable': 12}
D f 0.75 K 16 n 12 T ['0.296635', '0.297037'] median<T 12 min<T 12 {'unstable': 12}
```
D 在 f = 0.75、K = 16 一点（两块盘各一份，提交 32 + 16 个单元，每条意图 2 次提交，门槛 0.2966–0.2970 ms）12 行中位全在门槛下，登记判定全是 `unstable`。两行整行（产物第 2919 与 3621 行；后者 4 轮不赔、1 轮赔，两种门槛同判）：
```text
E7RESULT name=host_cpu_per_submission_point fraction=0.75 pool_data_units=1024 batch_units=16 candidate=release_check_only index=0 form=shared wait=blocking gap_milliseconds=0 clock=thread threshold_kind=machine_scaled threshold_milliseconds=0.297037 submission_units=32,16 host_cpu_per_submission_milliseconds_per_round=0.255888,0.198083,0.349498,0.300333,0.072755 judgement_per_round=not_losing,not_losing,losing,losing,not_losing judgement=unstable floor_host_cpu_per_submission_milliseconds_per_round=0.203505,0.150416,0.271306,0.227228,0.056575 floor_judgement_per_round=not_losing,not_losing,not_losing,not_losing,not_losing floor_judgement=not_losing
E7RESULT name=host_cpu_per_submission_point fraction=0.75 pool_data_units=8192 batch_units=16 candidate=release_check_only index=0 form=shared wait=sleep_1_millisecond gap_milliseconds=0 clock=thread threshold_kind=machine_scaled threshold_milliseconds=0.296635 submission_units=32,16 host_cpu_per_submission_milliseconds_per_round=0.217783,0.203651,0.314360,0.085463,0.246081 judgement_per_round=not_losing,not_losing,losing,not_losing,not_losing judgement=unstable floor_host_cpu_per_submission_milliseconds_per_round=0.170740,0.148350,0.224478,0.062378,0.188503 floor_judgement_per_round=not_losing,not_losing,not_losing,not_losing,not_losing floor_judgement=not_losing
```
A 最像样的点是 f = 0.25、K = 4（门槛 0.0826 ms）：这一点 12 行里 6 行 `unstable`，中位全不小于门槛；产物第 975 行（5 轮里只有最后一轮不赔，0.058907 ms 小于 0.082553 ms；前四轮 0.110–0.228 ms）：
```text
E7RESULT name=host_cpu_per_submission_point fraction=0.25 pool_data_units=1024 batch_units=4 candidate=scan_checksum_only index=0 form=shared wait=blocking gap_milliseconds=0 clock=thread threshold_kind=machine_scaled threshold_milliseconds=0.082553 submission_units=8,8,4 host_cpu_per_submission_milliseconds_per_round=0.197957,0.221157,0.228220,0.110429,0.058907 judgement_per_round=losing,losing,losing,losing,not_losing judgement=unstable floor_host_cpu_per_submission_milliseconds_per_round=0.154266,0.172242,0.160558,0.085450,0.037638 floor_judgement_per_round=losing,losing,losing,losing,not_losing floor_judgement=unstable
```

### 3.5　`unstable` 从哪来：卡 0 的 c_rt 后两轮明显低于前三轮，轮次与机器状态混在一起

同一个格（共用输入、阻塞等、间隔 0、8 个单元）5 轮线程 CPU 中位，见 G3 的 `percard.out`：卡 0 是 0.193、0.218、0.219、0.130、0.061 ms，卡 1 是 0.148、0.295、0.294、0.148、0.148 ms。把全部格按轮次取中位：
```text
card 1 round 4 blocking shared gap0 units8 valid thread_cpu_median ns [147600.0] all [147600.0]
card 1 round 5 blocking shared gap0 units8 valid thread_cpu_median ns [148135.5] all [148135.5]
card 0 round 1 median over units8 gap0 cells 229047.75 8
card 0 round 2 median over units8 gap0 cells 204466.0 7
card 0 round 3 median over units8 gap0 cells 278745.5 7
card 0 round 4 median over units8 gap0 cells 130125.5 7
card 0 round 5 median over units8 gap0 cells 89305.5 7
card 1 round 1 median over units8 gap0 cells 307511.0 7
card 1 round 2 median over units8 gap0 cells 294846.0 7
card 1 round 3 median over units8 gap0 cells 267786.0 7
card 1 round 4 median over units8 gap0 cells 312225.0 7
card 1 round 5 median over units8 gap0 cells 266130.5 7
```
卡 0 沿轮次 0.229、0.204、0.279、0.130、0.089 ms（后两轮低），卡 1 持平在 0.27–0.31 ms；A 卡 0 每轮活的行数 [6, 8, 4, 14, 36]（attack_ledger；`tally3.out`）随轮次升。产物自己记了机器状态在跑的时候变过：
```text
E7RESULT name=calibration_baseline segment=host_cpu_per_submission position=end runs=21 median_nanoseconds=5493404.0 chase_median_nanoseconds=102919583.000000 anchors_hold=true ratio_of_the_end_to_the_start=0.876478 drifted=true
E7RESULT name=calibration_trajectory segment=host_cpu_per_submission intents=805 intents_with_a_readable_ratio=805 larger_ratio_peak=1.106210 intents_above_the_voiding_edge=5 voiding_edge=1.10 intents_above_the_lower_reported_edge=107 lower_reported_edge=1.05 intents_above_the_upper_reported_edge=0 upper_reported_edge=1.20 larger_ratio_final=0.920912 wait_longest_seconds=20 waits=5 wait_total_seconds=100
```
终点校准比起点快 12.4%（`ratio_of_the_end_to_the_start=0.876478`，`drifted=true`）。产物自己没法分开「轮次在升」与「卡 0 的状态在变」：格子次序每轮轮换起点（登记第五节「每轮」），但轮次本身没有翻转。

### 3.6　结论（候选 A、B、D）

| 候选 | 登记读法下 | 能活的读法（门槛种类、等法、卡、`unstable` 怎么算） | 翻面观测 |
|---|---|---|---|
| A 只放扫描里的校验和 | 出局：两张卡 `not_losing` 零格；卡 1 全 `losing` | 只在卡 0、间隔 0、`unstable` 当活或只看后几轮时有格（`min5` 46 / 62 行，`round5` 36 / 52 行）；间隔 50 ms 与进程钟下一格不活；地板读法（去掉等的那一段）卡 0 有 34 / 40 行，但它假设等零成本 | 卡 0 在轮次顺序反转或轮数加倍后，间隔 0 与 50 ms 两档、4–8 个单元的 c_rt 是否仍低于 0.07–0.23 ms |
| B 加选段打分 | 只能写下界 `not_losing_lower_bound`，不能判不赔 | 卡 0 中位小于门槛 60 / 70 行、卡 1 20 / 39 行，全是下界；丙的输入展平没量 | 丙的输入展平那一段的主机 CPU（登记 R3 注：「丙的输入是分配记录，展平它的 CPU 没量」） |
| D 只换释放前读盘核 | 出局：`not_losing` 零格；卡 1 全 `losing` | f = 0.75、K = 16 一点（每条意图两次提交）卡 0 间隔 0 的 12 行中位全在门槛下，登记判定 `unstable`（见 3.4 两行整行）；别的点只有 `min5` 活 | 同一点在卡 0 上量 ≥ 10 轮、含顺序反转，不赔的轮数占比；以及 `prepare_the_version_publish_relocating_units` 里那一圈（`crates/singlefs-core/src/transaction.rs:11422`）提出之后，别的发布的行为是否逐字不变 |

读法的不对称：能让 A、D 活的读法（`min5`、`round5`、地板、间隔 0）都是把登记的判据放宽或换了一个更乐观的量；登记读法（5 轮全一致、machine_scaled、线程钟）下它们没活。两种读法下都不活的格：卡 1 全部、间隔 50 ms 的阻塞等。`unstable` 既不支撑「赔」也不支撑「不赔」（我的读法，不是哪条规则的原文）。
推翻 A、B、D「出局」的现象：卡 0 上同一批点在轮次顺序反转、轮数 ≥ 10 的重跑里，后几轮那种低 c_rt 在所有顺序下都出现（卡 0 的低 CPU 是稳态不是漂移）。

## 四、G8　GC 挪不挪出用户改动的调用线程：替「不挪出」辩护

### 4.1　不挪出时的前台挂钟增量（第一段读数）

一次提交的挂钟（共用输入，4–32 个单元，每格取 5 轮中位后的 84 格，`onesub.py`）与扣掉 CPU 自己算同样字节的主机 CPU 之后的净增量（`fg.py` 按 `candidate_threshold` 的提交序列取前台那一趟的提交，`onesub.py` 取单次）：
```text
wall ms of one submission, shared input, 4..32 units: cells=84 min=5.463 max=8.051 median=6.337
units 8 card 0 one submission wall minus the CPU crc of the same bytes: min 5.50 max 6.54
units 8 card 1 one submission wall minus the CPU crc of the same bytes: min 5.53 max 6.80
units 16 card 0 one submission wall minus the CPU crc of the same bytes: min 5.61 max 6.47
units 16 card 1 one submission wall minus the CPU crc of the same bytes: min 5.80 max 7.05
units 24 card 0 one submission wall minus the CPU crc of the same bytes: min 5.68 max 6.33
units 24 card 1 one submission wall minus the CPU crc of the same bytes: min 6.02 max 7.38
units 32 card 0 one submission wall minus the CPU crc of the same bytes: min 5.87 max 6.36
units 32 card 1 one submission wall minus the CPU crc of the same bytes: min 6.24 max 7.68
```
一趟用户改动之后提交几次：`crates/singlefs-core/src/mounted_session.rs:2546`：「所以一趟可以发「收尾 + 签发 + 一批」三次」；`crates/singlefs-core/src/mounted_session.rs:2714`：「if reach == CompactionRunReach::AtMostOneBatchAfterAUserChange && batches_published == 1 {」。登记里每次提交的单元数序列（`research/prompts/e185-r3-prereg.md` 第一节「一个点的提交序列」）：A、B、C 每次发布前重扫一次，签发那一趟 2 次（签发 + 一批，f = 0.25 是 8、8 个单元，f = 0.75 是 24、24 个）、已有活意图那一趟 1 次；D 每批一次。净增量（A/B/C 签发那一趟 / D 或活意图一趟）：
```text
f 0.25 K 4 A/B/C seq (8, 8) card 0 foreground wall increment ms: min 11.00 max 13.09 (n=6)
f 0.25 K 4 A/B/C seq (8, 8) card 1 foreground wall increment ms: min 11.05 max 13.60 (n=6)
f 0.25 K 4 D seq (8,) card 0 foreground wall increment ms: min 5.50 max 6.54 (n=6)
f 0.25 K 4 D seq (8,) card 1 foreground wall increment ms: min 5.53 max 6.80 (n=6)
f 0.25 K 8 A/B/C seq (8, 8) card 0 foreground wall increment ms: min 11.00 max 13.09 (n=6)
f 0.25 K 8 A/B/C seq (8, 8) card 1 foreground wall increment ms: min 11.05 max 13.60 (n=6)
f 0.25 K 8 D seq (16,) card 0 foreground wall increment ms: min 5.61 max 6.47 (n=6)
f 0.25 K 8 D seq (16,) card 1 foreground wall increment ms: min 5.80 max 7.05 (n=6)
f 0.25 K 16 A/B/C seq (8, 8) card 0 foreground wall increment ms: min 11.00 max 13.09 (n=6)
f 0.25 K 16 A/B/C seq (8, 8) card 1 foreground wall increment ms: min 11.05 max 13.60 (n=6)
f 0.25 K 16 D seq (16,) card 0 foreground wall increment ms: min 5.61 max 6.47 (n=6)
f 0.25 K 16 D seq (16,) card 1 foreground wall increment ms: min 5.80 max 7.05 (n=6)
f 0.75 K 4 A/B/C seq (24, 24) card 0 foreground wall increment ms: min 11.36 max 12.66 (n=6)
f 0.75 K 4 A/B/C seq (24, 24) card 1 foreground wall increment ms: min 12.03 max 14.76 (n=6)
f 0.75 K 4 D seq (8,) card 0 foreground wall increment ms: min 5.50 max 6.54 (n=6)
f 0.75 K 4 D seq (8,) card 1 foreground wall increment ms: min 5.53 max 6.80 (n=6)
f 0.75 K 8 A/B/C seq (24, 24) card 0 foreground wall increment ms: min 11.36 max 12.66 (n=6)
f 0.75 K 8 A/B/C seq (24, 24) card 1 foreground wall increment ms: min 12.03 max 14.76 (n=6)
f 0.75 K 8 D seq (16,) card 0 foreground wall increment ms: min 5.61 max 6.47 (n=6)
f 0.75 K 8 D seq (16,) card 1 foreground wall increment ms: min 5.80 max 7.05 (n=6)
f 0.75 K 16 A/B/C seq (24, 24) card 0 foreground wall increment ms: min 11.36 max 12.66 (n=6)
f 0.75 K 16 A/B/C seq (24, 24) card 1 foreground wall increment ms: min 12.03 max 14.76 (n=6)
f 0.75 K 16 D seq (32,) card 0 foreground wall increment ms: min 5.87 max 6.36 (n=6)
f 0.75 K 16 D seq (32,) card 1 foreground wall increment ms: min 6.24 max 7.68 (n=6)
```
读：签发那一趟（A/B/C）净增 11.00–14.76 ms，已有活意图那一趟或 D 的一次提交 5.50–7.68 ms；卡 1 比卡 0 高 0.05–2.1 ms。对照 CPU 模式一批自己的挂钟（攻方 g6 输出，第 1 / 13 / 17 行）：
```text
name=g6_batch arm=memory f=0.25 P=1024 K=4 batches=10 wall_ms_median=12.542 wall_ms_max=12.736 cpu_over_wall_median=1.0000 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.25 P=1024 K=4 batches=10 wall_ms_median=21.671 wall_ms_max=23.604 cpu_over_wall_median=0.6030 write_bytes_median=737792 paced_ms_at_50000000=14.756 paced_ms_at_500000000=1.476
name=g6_batch arm=real_disk f=0.25 P=8192 K=8 batches=5 wall_ms_median=123.135 wall_ms_max=124.717 cpu_over_wall_median=0.8965 write_bytes_median=1262080 paced_ms_at_50000000=25.242 paced_ms_at_500000000=2.524
```
一批 12.5 ms（内存盘、P = 1024）到 123.1 ms（真盘、P = 8192），前台增量占一批挂钟的 4.5% 到 118%。扣掉 B、C 在这一趟里另外省的主机 CPU（`candidate_threshold` 行，每条意图 0.17–3.96 ms，摊到每次提交更少）也改变不了量级。

### 4.2　替「不挪出」站得住的理由（每条带出处与推翻条件）

1. **今天产品路径不起跑**：`crates/singlefs-core/src/compaction_plan.rs:818`：「第一版三条水位恒 0（关），产品路径恒不起跑」，批大小条款没写：`crates/singlefs-core/src/mounted_session.rs:1952`：「批大小条款没写，产品路径在签任何意图之前停下」。前台增量在七·3 接线之前是零。推翻：七·3 把水位接上而不改本节形态。
2. **前台那一趟有上界**：至多一批（上面 2714 行），签发那一趟至多 2 次提交；`crates/singlefs-core/src/mounted_session.rs:1924`：「从这里重新数」 + 预算 `3S − 4 − 已经过去的发布数`（`crates/singlefs-core/src/compaction_plan.rs:806`）。水位越线才起，不是每次用户改动都付。推翻：水位在常见负载下几乎每次改动都越线。
3. **挪出要新建一整套并发，而核心层今天零线程原语**（G5 两条命令）；`.claude/kb/milestone/04-fourth-txn.md:137`：「命令行要到里程碑十的 FUSE」。`run_compaction`（`crates/singlefs-core/src/mounted_session.rs:1956`）是 `&mut self`，当后台入口语义上够（一趟推到停机或预算），并发上不够：它与 `publish_user_change`（`crates/singlefs-core/src/mounted_session.rs:1849`，同样 `&mut self`）互斥靠借用规则，挪出去就要加锁或消息。推翻：七·3 已经为别的原因引入了共享句柄。
4. **混合形**：GPU 只服务不在用户改动返回路径上的入口（显式入口、可写挂载续做），用户改动之后那一批仍走 CPU。`CompactionRunReach` 在一趟开头就定了（`crates/singlefs-core/src/mounted_session.rs:2542`），不违反「一批之内不换」。代价：与 `.claude/kb/decisions/26-后台整理与放置回收.md:245`（第 1 条，一份开关管全部后台任务）相撞，需要用户裁一句。

### 4.3　不挪出这一边不得不认的

- `.claude/kb/decisions/24-后台重活能不能卸给GPU.md:35`：「不许进前台路径。」「PCIe 往返是几十微秒量级，前台读写延迟吃不起这个」。第一段量到的挂钟是 5.46–8.05 ms，约 100 倍；条款依据段还写着「本工程没测过前台路径上的 GPU 往返」（同文件 `:43`）。不挪出而在调用线程上用 GPU，字面违这一条，辩护只能走 4.2 的 1、2、4，不能走「代价小」。
- D26 第 7 条比的是主机 CPU，并写了「挂钟与 CPU 模式持平也做」（`.claude/kb/decisions/26-后台整理与放置回收.md:251`）；挂钟变长不在这句里：它没说变长可不可以。
- 「GC 挪出之后丢掉哪些保证」（攻 G8 的第一条）：代码里绑在用户改动那一次调用上的有三样。预算计数在 `crates/singlefs-core/src/mounted_session.rs:1926` 归零（`compaction.publishes_since_the_last_user_change = 0;`）；整理排在删除续做与抬 F 之后、discard 之前（`crates/singlefs-core/src/mounted_session.rs:1925`：「次序在这次改动的删除续做与为它推的抬 F 之后」）；独占借用保证整理与用户改动互斥。整理在用户改动之后跑，所以那次改动的空间准入已经定了，整理不为它兜底；挪出不丢「这次改动能不能准入」。丢的是前两样的归属。

## 五、交岔路的地方（候选、翻面观测、够判条件；不写倾向）

| # | 候选（各自的定义） | 翻面观测 | 够判条件 |
|---|---|---|---|
| 1 | 读第一段的 `unstable`：甲 登记读法（5 轮全一致才判，machine_scaled，线程钟）；乙 放宽读法（任一轮、最后一轮或地板小于门槛就算活） | 卡 0 上 A、D 的低 c_rt（0.06–0.13 ms，后两轮）在轮次顺序反转后是否仍出现在后两轮、还是跟着轮次号走 | 同一批点在卡 0 上重量 ≥ 10 轮，含一趟轮次顺序反转的；每格报不赔轮数占比 |
| 2 | D（只换释放前读盘核）：甲 出局；乙 只在卡 0、f = 0.75、K = 16 一点留 | 该点 12 行中位在门槛下（0.297 ms）这一条在多轮与反序后是否稳定；别的 11 个点是否仍出局 | 同 1；另要 `crates/singlefs-core/src/transaction.rs:11422` 那一圈提成只被搬迁批调的函数之后，别的发布的行为逐字不变（可对拍） |
| 3 | C 的「唯一成批不赔」：甲 在今天的线性找（`crates/singlefs-core/src/region_scan.rs:288`、`:311`）上判；乙 在乙改成按身份查映射条目之后重量再判 | 乙占 C 省下量的份额（今天 P = 8192 时 41%–62%）改完后掉到多少；代理数下 C 在卡 0 `not_losing` 52 / 54 → 25 / 54 | 身份索引做完，第一段在同一批点上重量，或第三段的重放修成份额相加为 1 |
| 4 | 卡 1 怎么办（「全部显卡」）：甲 全部独显都用；乙 按卡的实测成本筛（每 MiB 主机 CPU 高于 CPU 自己算的卡不用）；丙 只用卡 0 | 卡 1 同口径每 MiB 部分（共用输入、阻塞等、间隔 0）仍 0.4876 ms 高于 CPU 的 0.37 ms 吗；16 MiB 一档线程 CPU 7.859 ms 对 CPU 5.94 ms | 卡 1 换槽位或驱动后重量 16 MiB 一档；用户对 D26 第 2 条「不给就用本机全部显卡」是否包含「赔钱的卡」的裁示 |
| 5 | G8：甲 不挪出，用户改动之后那一趟用 GPU；乙 不挪出，用户改动之后那一趟走 CPU、别的入口用 GPU（要用户裁 D26 第 1 条「一份开关管全部」）；丙 挪到另一条线程或另一个调用入口 | 前台净增量 5.50–14.76 ms 对用户可接受的前台延迟；七·3 接上水位后越线的频度；挪出所需并发的代价数（核心层今天零线程原语） | 水位接线后的越线频度；挪出方案的代价数（建计数模型）；用户对 D24 已定项 2 第 2 条字面（几十微秒）的裁示 |
| 6 | CPU 侧 CRC 的基线：甲 今天的 slicing-by-8（约 0.37 ms 每 MiB）；乙 硬件 CRC 指令 | 门槛按两种实现的速度比缩放后，A、B、C、D 的活格数变成多少 | 本机 CPU 上量两种实现每 MiB 的线程 CPU；把第一段的门槛按新速度重算 |
| 7 | G4② 目的地与源落点核的先后：甲 先核源落点再写目的地；乙 先写目的地 | 一份校验和不对的源字节是否会被搬走而没被隔离 | 读 `publish_relocation_batch` 的次序，并注入一份坏源字节跑一遍（不归本腿） |

## 没做什么

- 没起 GPU、没编译、没跑门禁阶段、没跑 checker 档、没用第二台。G3 的软件适配器那一处与 G4 的发布次序（岔路 7）这一轮没现查。
- 没复跑第一轮攻方的模型（判决已复跑过）；g6 的行只引了输出里的原样行。
- 第三段（`function-split`）只读了汇总行，没有对它的重放做逐行核；`remainder` 为负这件事只指出了，没有查为什么。
- 没替主 agent 撤销或维持判决，只交复核结果。
- 没把第一段 2684 行逐行读：计数全来自脚本，脚本的口径写在「先说读数口径」与各节标题下；`bypoint.py` 与 `fg.py` 只数线程钟、共用输入、machine_scaled。
- 没用过 `cfix.py` 的代理数当量的数：它把乙的绝对量换成 P = 1024 那一档，今天 P = 1024 的乙里本来还带线性找，所以是上界的代理，不是身份索引改完的读数。
- 复跑只覆盖十二个脚本和 `percard.py`、`onesub.py` 两条单跑：`onesub.py` 的 sha256 `62f075b1ed4b00b5937fb1619fc4ef1a5ce199e26fa496a0f55b0997287becc7`，输出 `run1/out/onesub.out` 的 sha256 `6eb5bd26babc51439539a364e2af9c6111dbba712248be4404f34e730ee47330`。
