# m2-layer0-scale-r3 云端辩方（Sonnet）报告：N1——替甲二辩护

写于 2026-09-26（JST）。立场：替第二轮判决第二节 M1 行被打中、第三节挂起的甲二辩护，只答 N1
（`research/prompts/_m2-layer0-scale-r3-background.md:11`）。被复核判决 `research/prompts/m2-layer0-scale-r2-main-verification.md`，
它依据的腿报告 `m2-layer0-scale-r2-opus-output.md`（云端攻方，打中 M1 的那一条腿）、`m2-layer0-scale-r2-sonnet-output.md`（第二轮辩方，
另一格 M4）、`m2-layer0-scale-r2-local-attack-output-s1.md`/`-s2.md`、核查员 `m2-layer0-scale-r2-verifier-output.md`。
全部引文行号现查于对应文件（`grep -nF`/`grep -n`），命中 0 次的一律不写成引文、改写「转述」。

**做法与第二轮辩方（M4，纯桌面复核）不同**：N1 明确要求「照第二轮攻方的按扇区豁免换掉今天的豁免之后……逐状态判得一不一样」，
这是一个要真的动手做的判据替换实验，不是复核一句话的出处。因此这份报告**把第二轮攻方提的按扇区豁免真的打进了 `crash.rs` 生产代码**
（一份草稿仓副本，不是入库装置），而不是像第二轮攻方那样只在探针里写一个旁路函数做对照。方法与产物见第二节。

## 一、结论一览

| N1 的问句 | 结论 | 证据 |
|---|---|---|
| 打中的是不是只有记录核对器那一处判据错：按扇区豁免换掉之后，两条流与第二轮随机历史上甲二与全量判得一不一样 | **打中的确实只是这一处判据错**：换成扇区判之后，在这次搜到的每一个样本上（UOOUOMSU 的 σ=79 段、今天 54 号实际全量跑的两条流、第二轮攻方用过的 120 条随机历史 + 7 条回收窗口置 0 的历史）甲二与全量**判得一样**；没有找到第二个机制。见第三节 | 本轮实测，见下 |
| 「只在 COW 部分落盘时显出来」这一类还有没有别的成员 | **没找到，但没证明不存在**：两个独立种子（11、53）合计 270 条随机历史、7 条变异历史，在打了补丁的判据上再搜一遍不单调点，仍是 0；这只是「没打中」的第二次独立观测，不是证明 | 见第四节 |
| 甲二能不能当平时快档、放过的是哪一类 | **有条件地能**：前提是 C561 的扇区判先落地到生产代码（今天还没有），而且它的等价性仍然是「两次搜索都没找到反例」而不是证明——真正的结构性风险是甲二只取 COW∈{∅, 全集} 两点，任何判据在中间取值处的非单调都会被它吞掉，这一点未被证伪、也未被证实穷尽 | 见第五节 |
| 辩不住的地方 | 三处：① 没有针对 I-7.7/I-7.8/I-1.8 专门构造变异（只跑了通用的不单调搜索，没有类似回收窗口置 0 那样的定向变异）；② C507/C513 的专用坏镜像用例没有在打了补丁的判据上重跑（这是本轮 N2 攻方的射程，我没有重复做）；③ 我的补丁是辩方原型，不是「实六」要落地的那份实现，边界情形（非 512 字节对齐的写、池边界的短读）没有压过 | 见第六节 |

## 二、方法：把按扇区豁免打进生产代码

冻结副本 `crates/singlefs-harness/src/crash.rs` 第 706、714 行（现查，与 `.claude/kb/checks-owed.md:480` C561 那一行「落点冻结副本
`crates/singlefs-harness/src/crash.rs` 第 706、714 行那两处豁免」一致）：

```
    let written_over_later = |index: usize| {   // 第 694 行起
        ...
        (index + 1..writes.len()).any(|later_index| {
            ...
            later.device == write.device
                && later_start < end
                && start < later_end
                && in_place(later_index)   // 第 706 行：要求整份还在位
                && reuse_is_not_proven_illegal_by_the_reclaim_predicate(...)
        })
    };
    let copy_is_missing = |copy: usize| !in_place(copy) && !written_over_later(copy);   // 第 714 行
```

补丁（模型目录 `crash-sector-wise.patch`，草稿仓副本 `/tmp/claude-1000/m2-layer0-scale-r3-sonnet/repo/` 上打的）：把 `written_over_later`
与 `copy_is_missing` 换成按扇区（512 字节）判——一份副本的某个扇区缺席 ⟺ 这个扇区上的字节不是它自己的，且没有一次更晚、已持久、
在这个扇区上字节仍在、且过了 `reuse_is_not_proven_illegal_by_the_reclaim_predicate`（C513 的检查）的写。`reuse_is_not_proven_illegal_by_the_reclaim_predicate`
本身原样保留、逐字未改——扇区判只收紧判定粒度，不动 C513 已经接上的那一道回收谓词检查。这与第二轮攻方在探针里写的
`variant_claimed_state_missing_unit`（`m2-layer0-scale-r2-opus-output.md` 第四节首行、`research/prompts/m2-layer0-scale-r2-opus-model/opus_r2_probe.rs:1266` 起）
是同一条判据的两份独立实现：一份在生产代码路径上（这份补丁），一份在探针侧的旁路函数上（第二轮攻方）。

编译：`cargo check --release -p singlefs-harness` 干净通过（草稿仓副本上现查，不是入库装置）。

探针文件：把第二轮攻方的 `opus_r2_probe.rs`（`research/prompts/m2-layer0-scale-r2-opus-model/opus_r2_probe.rs`，第二轮判决明写
「这一轮可以接着用」，`_m2-layer0-scale-r3-background.md:21`）原样拷进 `crates/singlefs-harness/tests/n1_sonnet_probe.rs`，只在文件末尾追加
三个新用例（`n11_exemption_chains_on_the_two_admitted_streams`、`n12_non_monotone_fresh_seed_after_patch`、
`n13_patched_sigma_full_enumeration_and_arm_a2`），不改上面任何一行。第二轮攻方原有的用例（q1、q7、q9、q10 等）在这份打了补丁的仓副本上
原样复跑，得到的是「同一段代码、同一批历史，换了判据之后」的结果，不是另起一套方法。

另建一份未打补丁的基线副本 `/tmp/claude-1000/m2-layer0-scale-r3-sonnet-baseline/repo/`（同一份探针文件，crash.rs 原样），
独立复现第二轮攻方的「打补丁前」的数，核实我读到的旧判据与第二轮攻方引用的是同一份代码（不是我这边转述错了）。

**两份仓副本都是草稿，不进 `research/results/`**：本轮不改生产代码（补丁只在 `/tmp` 副本上，落盘的是 `.patch` 文件与运行日志，
放在模型目录，交回前删掉两份仓副本，见「没做什么」）。

## 三、N1-①：按扇区豁免换掉之后，甲二与全量在哪些样本上判得一样

### 3.1 M1 打中的那一段本身（UOOUOMSU，σ=79）

基线（未打补丁，独立复现）：`q9_sector_wise_exemption_on_sigma`，`m2-layer0-scale-r3-sonnet-model/outputs/q9-baseline-unpatched.log`：

```
Q9 seq=UOOUOMSU sigma=79 sigma_states=262144 today_red=32768 sector_wise_red=0 differ=32768 elapsed_seconds=372.5
```

与第二轮攻方 `m2-layer0-scale-r2-opus-model/outputs/q9.log`（`Q9 seq=UOOUOMSU sigma=79 sigma_states=262144 today_red=32768
sector_wise_red=0 differ=32768 elapsed_seconds=208.7`）逐字一致（只有机器和线程数不同导致的 `elapsed_seconds` 不同）——独立复现了
「打补丁前」的数，确认我读到的旧判据与攻方引用的是同一份代码，不是转述错。

打补丁之后（生产代码路径，不是探针侧旁路函数）：`n13_patched_sigma_full_enumeration_and_arm_a2`（跑了 1370.76 秒，262144 个状态，
使用生产用的 `enumerate_layer0_selecting_versions` 与 `evaluate_state_for_versions`，不是隔离的对照函数），
`m2-layer0-scale-r3-sonnet-model/outputs/n13-sigma-full-and-arm-a2-patched.log`：

```
  N13_SIGMA_FULL seq=UOOUOMSU seg=79 states=262144 expected=262144 record_claimed_state_missing_unit=0 violations=0 ignored=0 root_without_record=0 checker_violations=0 elapsed_seconds=1369.7
  N13_ARM_A2 seq=UOOUOMSU states=334 red=0 elapsed_seconds=1370.6
```

`record_claimed_state_missing_unit` 从 32768（补丁前）变成 0（补丁后），`violations`（oracle）与 `checker_violations` 全程都是 0——
与第二轮攻方「同一段 oracle 与 checker 违例都是 0」（`m2-layer0-scale-r2-opus-output.md:103`「这个状态本身是合法的……同一段 262144 个状态里 oracle 两遍违例 0、checker 违例 0」）一致。
甲二（`ARM_A2`，整条流全部段）红 0——**全量与甲二在这一段上判得一样了**。这是本轮最直接的证据：不是探针侧的一个隔离对照，而是
把修法真的接进 `check_records` 之后，从头跑一遍生产管线得到的数。

**两份独立实现互相印证**：基线上 `sector_wise_red=0`（第二轮攻方探针侧的 `variant_claimed_state_missing_unit`）与打补丁后
`record_claimed_state_missing_unit=0`（我写进 `crash.rs` 的生产代码路径）是两份不同代码、算的同一件事，结果一致——降低了「哪一份实现写错了」
的可能性（两份都错成同一个数的概率远低于两份都对）。

### 3.2 今天 54 号实际全量跑的两条流

`n11_exemption_chains_on_the_two_admitted_streams`（静态扫，只读写表，不读 `check_records`，打不打补丁不影响它，
仍在打了补丁的仓副本上跑了一遍确认）：

```
N11 stream=first_stream writes=41 segments=10 chain_candidates=0
N11 stream=second_stream_to_e writes=425 segments=55 chain_candidates=0
```

`first_stream`（`Sim::first_transaction()`）41 写/10 段，`second_stream_to_e`（`fixed_script(Script::E)`）425 写/55 段——
与第一轮判决 `m2-layer0-scale-r1-main-verification.md:22`「第一条流 41 写 / 10 段……到 E 425 写 / 55 段」的形状逐字对上，确认
这两条正是 54 号 `--full` 今天实际跑的两条流。豁免链候选都是 0——这两条流**从不出现** C561 的两代跨度不对齐复用形状（要连续两次
卸载才放得出这个形状，`m2-layer0-scale-r2-opus-output.md:130`「每条最小的都要两次卸载」），所以补丁打不打对它们完全不改变判定；
`q7_exemption_chains_on_fixed_scripts`（覆盖 B/C/D/E/E18/H1/H2 共 7 条今天与第一轮用过的固定脚本）同样逐条候选 0，
与第二轮攻方 `m2-layer0-scale-r2-opus-model/outputs/q7_exemption_chains_on_fixed_scripts.log` 逐字一致。**这两条流上甲二与全量本来就判得一样，
补丁不改变这一点**——不是因为补丁修好了什么，而是因为这个判据错的触发条件（两代跨度不对齐复用）在今天的固定脚本里根本不出现。

### 3.3 第二轮攻方用过的随机历史

120 条（种子 11，长度 1–10，字母表 O/U/M/R/S）：`q1_search_non_monotone_cow_subsets_over_histories`，打了补丁之后重跑，
`m2-layer0-scale-r3-sonnet-model/outputs/q1-120-seed11-patched.log`：

```
Q1_TOTAL streams=120 streams_with_non_monotone=0 cheap_checked=51720 elapsed_seconds=1270.4
```

`cheap_checked=51720` 与第二轮攻方 `m2-layer0-scale-r2-opus-model/outputs/q1-sweep120.log` 的 `Q1_TOTAL streams=120
streams_with_non_monotone=0 cheap_checked=51720` 逐字一致——同一个种子生成的是同一批 120 条历史（确认了历史生成的确定性，
不是两边各自跑出不同的样本凑巧都是 0），补丁前后都是 0 个不单调点。

7 条回收窗口置 0 的历史（`ZOOOO,OZOOO,UZOOOSO,ZSOSOS,MZOOO,UOZOSOS,ZOUZOO`，C22 那一类真实 bug 的变异替身）：
`q10_forced_zero_reuse_window_arms`，打了补丁之后重跑，`m2-layer0-scale-r3-sonnet-model/outputs/q10-forced-zero-reuse-window-patched.log`：

```
ARMS ZOOOO arm_a_states=66 arm_a_bad=31 arm_a2_states=84 arm_a2_bad=43 full_sampled_cow_states=1368 full_sampled_bad_beyond_arm_a=132
ARMS OZOOO arm_a_states=66 arm_a_bad=23 arm_a2_states=84 arm_a2_bad=32 full_sampled_cow_states=1368 full_sampled_bad_beyond_arm_a=128
ARMS UZOOOSO arm_a_states=146 arm_a_bad=39 arm_a2_states=179 arm_a2_bad=54 full_sampled_cow_states=2434 full_sampled_bad_beyond_arm_a=140
ARMS ZSOSOS arm_a_states=74 arm_a_bad=39 arm_a2_states=95 arm_a2_bad=54 full_sampled_cow_states=1596 full_sampled_bad_beyond_arm_a=132
ARMS MZOOO arm_a_states=94 arm_a_bad=23 arm_a2_states=115 arm_a2_bad=32 full_sampled_cow_states=1577 full_sampled_bad_beyond_arm_a=136
ARMS UOZOSOS arm_a_states=146 arm_a_bad=31 arm_a2_states=179 arm_a2_bad=43 full_sampled_cow_states=2434 full_sampled_bad_beyond_arm_a=148
ARMS ZOUZOO arm_a_states=122 arm_a_bad=51 arm_a2_states=146 arm_a2_bad=63 full_sampled_cow_states=1750 full_sampled_bad_beyond_arm_a=272
```

这 7 行的 `arm_a_bad`（31/23/39/39/23/31/51）与 `arm_a2_bad`（43/32/54/54/32/43/63）**与第二轮攻方打补丁前的
`m2-layer0-scale-r2-opus-model/outputs/q10.log` 逐字相同**（同样的 7 个数字、同样的顺序）——**打了扇区判补丁之后，
甲二在这一类真实 bug（回收窗口置 0，与 C561 是不同的机制）上的判定完全没变**，仍然照样红。这一点对甲二的辩护是正面证据：
扇区判补丁是一次针对性的收紧（只去掉 C561 那一处假红），没有连带放松甲二对别的真实故障类型的抓取能力。

### 3.4 小结：N1-① 的答案

在这一轮搜到的全部样本（M1 打中的那一段、今天实际全量跑的两条流、第二轮攻方用过的 120+7 条历史）上，打了按扇区豁免的补丁之后，
**甲二与全量逐状态判得一样**。没有找到「按扇区豁免会在别的地方产生新的分歧」这一类第二个机制——这与判决表四「按扇区豁免会不会
放过一次真的错误复用」（N2，本轮攻方的射程）是两个不同的问题：N2 问的是扇区判会不会漏掉真错误（假阴性），这里回答的是
「甲二在这个修好的判据下与全量还分不分得开」（甲二本身的等价性）。两者都指向同一份补丁，但检验的方向相反，互不重复。

**推翻条件**：找到任何一条历史，在按扇区判的判据下甲二与全量判得不一样（甲二红而全量绿，或反过来）——本轮没有找到；
若 N2（本轮攻方）在这份或类似的扇区判补丁上找到这样一条历史，这里「打中的只是这一处判据错」的结论就要重新核。

## 四、N1-②：「只在 COW 部分落盘时显出来」这一类还有没有别的成员

第二轮攻方的论证（`m2-layer0-scale-r2-opus-output.md:150`）：COW 写按落点分两类——落在从没被写过的扇区上的（判据单调，
读代码论证，**没有实测反例，也没有实测坐实**）、落在之前写过的扇区上的（只有这一类可能不单调，第二轮攻方在 120 条随机历史 +
1 条已知历史 + 7 条变异历史上搜过，0 个非 record 类的不单调点，`m2-layer0-scale-r2-opus-output.md:163`
「`Q1_TOTAL streams=120 streams_with_non_monotone=0`」）。

本轮独立复核两步：

**第一步**：把第二轮攻方用过的同一批历史（种子 11，120 条）在打了补丁的判据上重跑（第三节 3.3），仍是 0——排除了「换成扇区判之后，
这批历史里冒出新的不单调点」这个可能。

**第二步**：换一个第二轮完全没用过的种子（53），跑 150 条独立生成的历史（不与第二轮的 120 条重叠，`n12_non_monotone_fresh_seed_after_patch`），
`m2-layer0-scale-r3-sonnet-model/outputs/n12-150-seed53-patched.log`：

```
N12_TOTAL seed=53 streams=150 streams_with_non_monotone=0 cheap_checked=77912 elapsed_seconds=2054.4
```

150 条、77912 次判定，0 个不单调点。这个函数（`search_non_monotone`）的「便宜判定」同时核记录核对器两条 **和** 池级 checker 其余各条
（`m2-layer0-scale-r2-opus-output.md:151`「用记录核对器两条 + 池级 checker 判」），所以这一轮也覆盖了 M1 那句「重点是……记录核对器第二条」
之外的 I-7.7、I-7.8、I-1.8（`.claude/kb/invariants.md:57`、`:58`、`:31`）与 checker 的其余判据，不是只测了记录核对器。

**按「一条腿只抽一次样不算一次观测——两次都没打中才记没打中」这条规则**（`.claude/rules/three-way-inference.md:147` 起）核：
第二轮攻方原本的 120 条（种子 11）是对**旧判据**的一次「没打中」；这一轮的重跑（同一 120 条，改判**新判据**）与新种子 53 的 150 条
是对**新（打补丁之后）判据**的两次独立「没打中」——两次都没打中，按规则可以记「没打中」，但这只覆盖了：动作字母表 `{O, U, M, R, S}`、
历史长度 1–10、相交组大小 ≤ 14（`search_non_monotone` 的 `max_group` 参数）这个范围。**没做、也不该被当成已经排除的**：

- 没有像第二轮攻方对 C22（回收窗口置 0）那样，专门为 I-7.7、I-7.8、I-1.8 构造一个**定向变异**去逼出「COW 中间取值才红」的形状——
  两次搜索用的都是通用的随机历史 + 已知的一种变异替身（回收窗口置 0），没有针对性地去找「树 ID 水位」「归并键」这类判据自己的
  漏洞类比物；
- 历史长度上限 10、字母表只有 5 种动作，比 M1 那条历史（`UOOUOMSU`，长 8，也在这个范围内，说明范围至少覆盖得到已知的那一个）
  更长、更复杂的历史组合没有搜过。

**推翻条件**：一个新的变异（类比回收窗口置 0 之于 C22）专门去逼 I-7.7/I-7.8/I-1.8 出现「COW 中间取值判定不同于两端」的历史，
或者更长历史 / 更大动作字母表上的随机搜索找出非 0 的不单调点——任一个都会推翻「除了记录核对器这一处，没有别的成员」这句话。

## 五、N1-③：甲二能不能当平时快档，放过的是哪一类

先澄清一个容易搞反的地方：**今天（判据没打补丁之前），全量在 UOOUOMSU 的 σ=79 段上报 32768 个红、甲二报 0 个红，这不是甲二漏掉了
一个真 bug**——同一段 oracle 与池级 checker 违例都是 0（第三节 3.1），那 32768 个红是判据自己的假阳性（C561）。所以今天字面意义上
「甲二与全量不一致」这件事本身，不能读成「甲二把一个全量抓得到的错放过了」；更准确的说法是「甲二意外地绕开了一个判据 bug，
而这不是它被设计出来要做的事，纯属它只取 COW∈{∅, 全集} 两个端点、没有落在这个判据 bug 的触发区间里」。

**能当快档的条件**：

1. **C561 的扇区判先落地到生产代码**（`.claude/kb/checks-owed.md:480` 「判法先过层 0 规模第三轮（改法被攻过零轮）；实六实现」）——
   本轮这份补丁是辩方原型，不是「实六」。落地之前，今天的判据本身有已知的假阳性，甲二和全量在这一点上的一致与否都建立在
   一个已知错误的判据上，不是稳固的地基。
2. **落地之后，甲二与全量的等价性仍然是「两次搜索没找到反例」，不是证明**——第四节已给出范围与限度。

**甲二能当快档时，放过的是哪一类（结构性，不是已经实测到的）**：甲二只在每段两端（COW 全 ∅、COW 全落）取样，任何判据如果在
COW 的某个**中间**子集上给出与两端不同的结论（第二轮攻方称之为「不单调」），甲二都会把它整类吞掉——不管这个不单调是判据自己的错
（C561 那一类）还是实现的真错误。本轮与第二轮加起来的证据显示：**记录核对器第二条**在「跨代跨度不对齐的复用只落一半」时确实
有这样一个不单调点（C561，打补丁前）；**I-7.7、I-7.8、I-1.8** 与池级 checker 其余各条，经两次独立的随机 + 变异搜索（合计 277 条历史、
约 130000 次判定），没有找到不单调点，但也没有像 C561 那样被专门构造出来过（第四节「没做、也不该被当成已经排除的」）。

**建议的用法（推的，不是已经实测坐实的一个规则）**：甲二可以当**开发中途的快速信号**——它对回收窗口置 0 这一类真实故障（与 C561
不同机制）仍然照红（第三节 3.3），说明它不是对所有故障都不敏感；但它**不能替代提交时的全量**，因为它对「COW 中间取值才红」这一类
判据缺陷或实现缺陷的敏感度是**未经证明的零**，而不是**证明过的零**。`.claude/agent-common.md:53`
「崩溃点测试不衡量时间成本，也不为省时间缩范围」这条约束下，甲二作为约简候选要「证明了等价」才合法（`m2-layer0-scale-r1-main-verification.md:38`
「约简与否是用户的定……甲二的等价还没证出来」）；本轮的结果是把「还没证出来」往前推了一步（多了两个独立种子的零反例、多了一次
真实生产代码路径上的验证），但没有把它变成「证出来了」。

**推翻条件**：任何一个未来实现（层 0 判据的改动、新的判据）在 COW 的中间子集上出现红、而两端都绿的历史——这会直接说明甲二这个
「只取两端」的取样方式本身有结构性缺口，不只是 C561 那一个孤立判据错。

## 六、辩不住的地方——还差什么

1. **没有为 I-7.7、I-7.8、I-1.8 专门构造定向变异**：第二轮攻方对 C22（回收窗口置 0）造过一个专门的变异（`ReuseWindow::ForcedToZero`），
   逼出了一批真实红的历史；这一轮和第二轮都只用了通用随机历史 + 这一个既有变异去测其余三条判据的「COW 中间取值」是否单调，
   没有为这三条判据各自的机制（树 ID 水位如何被压低、归并键如何被撞、系统配置实例代号如何不一致）设计对应的定向变异。
   这是本轮真实的方法缺口，第四节已列出，不重复计入结论的可信度。
2. **C507/C513 的专用坏镜像用例没有在打了补丁的判据上重跑**：`.claude/kb/checks-owed.md:445`（C513）与 `:515`（C507）
   点名的用例（`a_unit_whose_only_later_write_to_the_same_slot_never_landed_is_reported_missing_by_the_record_checker` 与
   `crates/mutations.tsv:306-308` 的三条变异）是 N2（本轮云端攻方）的射程（`_m2-layer0-scale-r3-background.md:12`
   「① 按扇区豁免会不会放过一次真的错误复用：C507 那一格……C513 那一道……」）——本轮的 q10（回收窗口置 0）结果显示扇区判补丁没有
   松动甲二对这一类真实故障的抓取（第三节 3.3），但那是**间接**证据，不是直接重跑 C507/C513 的专用坏镜像用例；没有重复做 N2 的活，
   避免与攻方两条腿重叠（`_m2-layer0-scale-r3-background.md:41`「两条攻方腿不重叠」，虽然我是辩方，但同一条纪律照样适用——不重复
   已经分派给另一条腿的验证）。
3. **补丁是辩方原型**：`SECTOR_BYTES_FOR_REUSE_EXEMPTION = 512` 假设写长总是 512 的整数倍（冻结副本的几何下 16 KiB/32 KiB/4 KiB
   节点与单元确实都是，本轮取样的历史也都满足）；`reader.read(...).unwrap_or_default()` 在越界或读不到时按空字节处理，
   这与今天 `in_place` 的 `.is_some_and(...)` 语义（读不到按 false）方向一致但没有专门造一个跨越池边界的写去压过这条边。
   这些不是本轮找到的反例，是补丁自身还没被压过的边界，交给「实六实现」时要覆盖。

## 七、没做什么

- 没有跑任何名字带 `layer0` 的测试目标、没跑 54 号。
- 没有重跑 C507（`.claude/kb/checks-owed.md:515`）、C513（`:445`）的专用坏镜像用例——这是本轮云端攻方（N2）的射程，见第六节第 2 点。
- 没有针对 I-7.7、I-7.8、I-1.8 各自的机制专门构造定向变异，只用了通用随机搜索与既有的回收窗口置 0 变异，见第六节第 1 点。
- 没有把这份扇区判补丁当成「实六」的最终实现提交或写回 kb——它只是本轮辩方用来做实验的原型，落在草稿仓副本上，
  不改生产代码、不改 `.claude/kb/`。判法要不要照这个形态落地，由主 agent 综合三条腿之后判。
- 没有跑 N3（本地攻方，代价表复算）与 N2（云端攻方）的射程，避免重叠。
- 全量枚举严格控制在约 10⁶ 个状态以内（σ=79 段 262144 个状态是本轮唯一一次全枚举，其余都是 `search_non_monotone` 的
  按组抽样/枚举，单组不超过 2^14=16384 个状态）。

## 附：产物与哈希

- 报告：本文件 `research/prompts/m2-layer0-scale-r3-sonnet-output.md`。
- 模型目录：`research/prompts/m2-layer0-scale-r3-sonnet-model/`（`crash-sector-wise.patch`、`n1_sonnet_probe.rs`、
  `outputs/` 五份日志、`rerun.sh`、`SHA256SUMS`）。
- 草稿仓副本（交回前删）：`/tmp/claude-1000/m2-layer0-scale-r3-sonnet/repo/`（打了补丁）、
  `/tmp/claude-1000/m2-layer0-scale-r3-sonnet-baseline/repo/`（未打补丁的基线）。
