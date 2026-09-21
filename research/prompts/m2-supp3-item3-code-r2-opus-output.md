# 增补 3 第 3 件（崩溃注入）代码轮第二轮：云端攻方（Opus）报告

攻击面 K1（段内真子集抽得够不够）、K3（记录核对器接上之后判过什么）。判的是工作区今天那一版
`crates/singlefs-harness/src/crash_injection.rs`（1415 行）与它的两份用例。
**全部数都是副本上量的**（仓拷进草稿目录，只在副本上改、副本上跑），按
`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节，主 agent 要在入库装置上重做一次才能引。

## 一、各格判定一览

| 格 | 问的是 | 判定 | 一句话依据（副本实测） |
|---|---|---|---|
| K1-① | 快档一轮里 `withholds_a_write_before_a_persisted_one()` 为真的崩溃状态几个 | **修补有效，40 / 93** | 快档 93 个崩溃状态里 40 个段内有洞（43%）；第一轮这一类是零 |
| K1-② | 同段镜像对里被摆出「只持久后一份」的几对 | **37 对 / 2782 对（1.3%）** | 候选段里同段镜像对 2782 对（与第一轮量到的 2782 逐字相同），一轮快档摆出「只持久后一份」的 37 对、「只持久前一份」48 对 |
| K1-③ | 今天仍然零 / 接近零概率的子集形态 | **打中：长段上「只扣下一两个写」整类** | 崩溃状态落在段长 10 的段上时扣下数最少是 3，落在段长 18 的段上时最少是 6；段长 18 的段里「恰好扣下一对镜像、其余 16 个写全持久」这一个状态的抽中概率是 9 / 262143，一轮快档的期望命中约 4 × 10⁻⁴ |
| K1-④ | 这个洞是抽样分布造成的还是枚举域造成的 | **抽样分布** | `draw_a_proper_subset`（`crash_injection.rs:869`）在段内取均匀掩码 ⇒ 扣下数服从 Binomial(n, ½)，长段上质量全压在「扣下一半」那一带 |
| K3-① | 快档一轮里核对器跑过几次 | **93 次**（= 崩溃状态数） | `crash_injection.rs:698` 每个崩溃状态加一次，`tests/second_transaction_supplement_three_crash_injection.rs:156-159` 的断言钉住它等于崩溃状态数 |
| K3-② | `root_without_record` 在干净构建上判红几次 | **0 次** | 快档 93、偏回退档 68、写死历史 104、抬 F 那段 121，合计 386 个崩溃状态上全 0 |
| K3-③ | `claimed_state_missing_unit` 在干净构建上判红几次 | **0 次** | 同上 386 个崩溃状态上全 0 |
| K3-④ | 「这一格与没接逐字相同」成不成立 | **打中（硬）** | 把 `:706` 的 `&& record_check == RecordCheck::default()` 删掉（计数照增、`check_records_against` 照调，只丢掉判据），干净构建上这个测试二进制 7 条用例 + `--lib crash_injection` 5 条用例**全绿** |
| K3-⑤ | 变异表第 183 行钉住的是什么 | **只钉住计数器** | 它点名的必红用例 `crash_points_are_reproducible_proper_subsets_sorted_by_segment` 判的是 `record_checks == crash_points`；K3-④ 那个变异保留计数器，这条用例照样绿 |
| K3-⑥ | 举一个能让 `root_without_record` 判红的崩溃状态 | **举出来了** | 施加 `crates/mutations.tsv` 第 67 行（步 3：零单元发布在记录与根之间少一道屏障）：写死那段历史的第 6 段 `persisted_within_the_segment = [false, false, true]`（两份 journal 记录扣下、根槽 FUA 持久），`violations` 空、模型不反对，**只有记录核对器判得出** |
| K3-⑦ | 那个判红的状态靠的是不是 K1 这一轮新开的域 | **是** | `[false, false, true]` 正是「后发的写先持久」那一类，第一轮的前缀截断摆不出来 ⇒ K1 的修补是 K3 唯一一次判红的前提 |
| K3-⑧ | 记录核对器在崩溃注入这条路上唯一一次判红，会不会被别的东西接走 | **会一半** | 变异 67 叠上 K3-④ 那个变异：三条红的用例掉到一条，剩下那条红在 `tests/second_transaction_supplement_three_crash_injection.rs:344-347` 的**计数断言** `record_root_without_record == 0`，不是红在判据链 |
| K3-⑨ | `claimed_state_missing_unit` 有没有任何东西盯着 | **一样都没有，而且疑似不可达** | 两份用例里没有一条断言提到它；把带单元的发布那一道屏障也拿掉（`transaction.rs:2129` 写着的持久顺序里「单元 → 屏障 → journal 记录」那一道），快档与三段写死历史上它**仍然全是 0**，红的是写死历史那条用例的 `crash_points >= 100`（掉到 86）——一条与判据无关的计数断言。**把并起来的那个 12 个写的段整段全枚举（12347 个崩溃状态）也还是 0** ⇒ 不是抽样抽不到，机理没查清（3.6 给了两个候选，都没验） |

⚠️ 按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算没被攻过」：
本报告第四节自己提的三条改法**被攻过零轮**，而且只在副本上量过。

## 一之二、复跑命令与模型目录里每个文件的 sha256

模型目录 `research/prompts/m2-supp3-item3-code-r2-opus-model/`。复跑：

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ <副本>/
bash research/prompts/m2-supp3-item3-code-r2-opus-model/reproduce.sh <副本>
```

`reproduce.sh` 五步：干净构建上的量具 → 干净构建上的整个测试二进制 → 变异 A → 变异 67 → 变异 C（含深枚举）。
⚠️ **每次还原变异之后要让 cargo 真的重编**（脚本里用 `cp`／`python` 写回，会带上新的 mtime；
用 `mv 文件.orig 文件` 会保住旧 mtime，cargo 不重编，跑出来的是上一个变异的二进制——见第六节第 7 条）。

| sha256 | 文件 |
|---|---|
| `bb0cd6a0be858443919c2d4f1ffd598c312d3c953770f797a23748e231b62c7a` | `probe1-fast-tier-subset-coverage.rs` |
| `5af05e779eac18503e948dd3c01d49d19f56369b0569d6800b9503c71e02c4a9` | `probe2-deeper-enumeration.rs` |
| `5df02eec2924842f9d86de011fb41a01a58e9692a61e3e5b2d12ae41046eb1c9` | `reproduce.sh` |
| `a1b328f99ba2ef3aa178c6b4d27c29bbdcd4f97babd69aa6122141dee01a7c77` | `run-baseline-crash-injection.txt` |
| `95df98043a2e3beab8e623c16c763d943ec4ffef212c3015b1e3f36809152f64` | `run-fresh-copy-baseline-crash-injection.txt` |
| `31a3b766a218e9205eecf3a492900fdc0ab02bc6330fc3d7da9fb1fb2390567e` | `run-fresh-copy-mutation-A.txt` |
| `0821bf3028338900f7a9e6d7fab71debe00f2184ecea8eda13c6567a9fc5bd98` | `run-fresh-copy-probe1.txt` |
| `63a570f1a1c1977646e878f0587705df447f988c7a90506a282946dc00c3ae15` | `run-mutation-67-crash-injection.txt` |
| `9250f04d2331ccbef558f32151a2a8420dc81e4591423674d997fd29e6b5b89d` | `run-mutation-67-plus-A-crash-injection.txt` |
| `4e561b8ef2b650e2067aef0b2737873957d5a1400e25312042235acb8f941fa4` | `run-mutation-C-crash-injection.txt` |
| `c67658fada3be03c29009ed660139aa7e1682b867d883a74d3c5961ee9ad07ce` | `run-mutation-C-deeper-enumeration.txt` |

被判的源码这一轮没被动过：收尾时在副本里跑 `sha256sum -c research/prompts/m2-supp3-item3-code-r2-start-snapshot.sha256`，
`crates/` 那 10 项全 OK（只有 `.claude/kb/checks-owed.md` 对不上，是别的会话在改，与这一轮无关）。

## 二、K1：段内真子集抽得够不够

### 2.1 这一轮新开的域真的被抽到了（两个数）

副本上按快档同一组参数（种子基 `7_463_871_032_432_355_113` 起 24 段、每段 24 步、`Sampled { crash_points_per_history: 4 }`、
`GenerationWeights::BROAD`、不跑每步 checker、两块 4 GiB 盘）跑一轮，量具原样输出
（同一组数在两份互不相干的副本上各跑出来一次，逐字相同；存档
`research/prompts/m2-supp3-item3-code-r2-opus-model/run-fresh-copy-probe1.txt`）：

```
── 攻方副本量具：快档 24 段 × 24 步 × Sampled{4} ──
崩溃状态 93 个；段内有洞（后发的写先持久）40 个
记录核对器：跑了 93 次；root_without_record 判红 0 次；claimed_state_missing_unit 判红 0 次
被摆到过的（种子, 段号）91 个
段长直方图（段长 → 崩溃状态数）：{1: 27, 2: 43, 4: 1, 10: 8, 18: 14}
段内扣下几个写的直方图（扣下数 → 崩溃状态数）：{1: 59, 2: 11, 3: 3, 4: 2, 5: 1, 6: 2, 7: 3, 8: 3, 9: 4, 10: 3, 12: 2}
扣下比例分桶：{"扣下四分之一到四分之三": 23, "扣下恰好 1 个": 59, "扣下恰好 2 个": 11}
同段镜像对：全部段里 2830 对；被摆到过的段里合计 169 对（按崩溃状态累计，未去重）
镜像对在崩溃状态上的形态（按崩溃状态累计）：{"两份都扣下": 46, "两份都持久": 42, "只持久前一份": 48, "只持久后一份": 37}
镜像对按种类（按崩溃状态累计）：{"journal_record": 27, "unit_write": 146}
被摆出「只持久后一份」的镜像对（按 (种子,段,对) 去重）：37 对；「只持久前一份」：48 对
「恰好一对镜像两份都扣下、这一段别的写全持久」的崩溃状态：5 个，按段长分 {2: 5}
候选段合计 1432 段；候选段段长直方图 {1: 443, 2: 518, 4: 79, 10: 219, 18: 173}
候选段里的同段镜像对合计 2782 对；候选段的真子集总数 45577958
按段长分的「扣下几个写」直方图：{1: {1: 27}, 2: {1: 32, 2: 11}, 4: {3: 1}, 10: {3: 2, 4: 2, 5: 1, 6: 1, 7: 2}, 18: {6: 1, 7: 1, 8: 3, 9: 4, 10: 3, 12: 2}}
```

- **`withholds_a_write_before_a_persisted_one()` 为真的崩溃状态：40 个 / 93 个（43%）。** 第一轮 K1-b 打中的是这一整类零概率，
  今天不但摆得出，还占了四成 —— 这一半修补**站得住**。装置自己打的报告同意这个数（`崩溃状态 93 个：…段内有洞（后发的写先持久）的 40 个`）。
- **同段镜像对里被摆出「只持久后一份」的：37 对。** 分母用候选段（与 `draw_crash_points` 同一条判定：整段的写都落在
  mkfs 之后、最后一个跑完的操作之前）里的同段镜像对数 **2782 对** —— 与第一轮判决 K1-b 那一格记的 2782 **逐字相同**，
  两轮各自独立数出来的同一个数。覆盖率 37 / 2782 = **1.3%**。
  另外「只持久前一份」48 对、「两份都持久」42 次、「两份都扣下」46 次（后两项按崩溃状态累计、未去重）。

### 2.2 打中：长段上「只扣下一两个写」这一整类仍然接近零概率

抽子集的实现是 `crash_injection.rs:869` 的 `draw_a_proper_subset`：段长 ≤ 63 时
`let mask = source.below((1u64 << segment_length) - 1)`（`:871`），即在 [0, 2ⁿ−1) 上取**均匀掩码**。
均匀掩码等价于「每个写独立以 ½ 持久」，于是**段内扣下的写数服从 Binomial(n, ½)**：质量全压在 n/2 附近，
两端（扣下 0～2 个、扣下 n−2～n 个）指数地小。实测正好是这个形状：

| 段长 n | 候选段里有几段 | 快档摆到几个崩溃状态 | 实测扣下数的取值 | 「扣下 ≤ 2 个」这一类的抽中概率 |
|---|---|---|---|---|
| 1 | 443 | 27 | {1} | 1（唯一的真子集） |
| 2 | 518 | 43 | {1, 2} | 1 |
| 4 | 79 | 1 | {3} | 10 / 15 |
| 10 | 219 | 8 | {3, 4, 5, 6, 7} | 55 / 1023 ≈ 5.4% |
| 18 | 173 | 14 | {6, 7, 8, 9, 10, 12} | 171 / 262143 ≈ 0.065% |

⇒ **段长 10 与 18 的段上，一轮快档 22 个崩溃状态里，扣下数最少的是 3（n=10）与 6（n=18）；
「只扣下一个写」「只扣下一对镜像」在长段上一次都没出现过。**

具体到一个能写成构造的形态（K1 要的「今天仍然接近零概率的子集形态」）：

> **形态 S**：一次带单元的发布，它的单元写段有 18 个写（9 对镜像）。
> 崩溃状态取「其中**恰好一对镜像的两份都没持久，另外 16 个写全持久**」。

- 它属于今天的枚举域（`CrashPoint` 摆得出），也属于层 0 的枚举域（D13（验证路线） 已定项 4）。
- 抽中概率：段长 18 的段里这样的状态有 9 个，真子集 2¹⁸−1 = 262143 个 ⇒ 单次抽样命中 9 / 262143 ≈ 3.4 × 10⁻⁵。
  一轮快档抽 96 次（24 段 × 4），落进段长 18 的段的比例按候选段计是 173 / 1432 ≈ 12% ⇒ **期望命中 ≈ 4 × 10⁻⁴**，
  即两千多轮快档才碰得到一次。
- 实测：一轮快档里「恰好一对镜像两份都扣下、这一段别的写全持久」的崩溃状态 5 个，**全部落在段长 2 的段上**
  （段长 2 时这个形态就是空子集，必然摆得到），**段长 4 / 10 / 18 的段上一个都没有**。
- `EveryProperSubsetOfShortSegments` 那一档也救不了它：写死那段历史用的是 `segment_writes: 4`
  （用例第 305 行），只有 ≤ 4 个写的段整段枚举，10 与 18 的段仍然只抽 8 个。

**这个形态不是随便挑的**：它正是记录核对器第二条判据（`claimed_state_missing_unit`：某次发布的某个单元全部副本缺席）
唯一可能判红的形状。K1 的抽样分布与 K3 的恒 default 是**同一件事的两头**（见第三节）。

### 2.3 这一格的推翻条件

- 换一个种子基之后 `crash_points_withholding_a_write_before_a_persisted_one` 掉到 0，或者 40 / 93 这个比例大幅变动 ⇒ 2.1 要重量。
- 谁能举出一个快档能抽到的、扣下数 ≤ 2 且段长 ≥ 10 的崩溃状态 ⇒ 2.2 的「接近零概率」不成立。
- `draw_a_proper_subset` 改成按扣下数分层抽（先抽扣下几个、再抽扣下哪几个）⇒ 2.2 整节作废。

## 三、K3：记录核对器接上之后判过什么

### 3.1 跑过几次、判红几次（干净构建，副本实测）

`crates/singlefs-harness/src/crash_injection.rs:697` 每个崩溃状态调一次
`check_records_against(&image, writes_up_to_this_segment, report.effective_root)`，
`:698` 计数加一，`:706` 把 `record_check == RecordCheck::default()` 当这个崩溃状态通过。
在干净副本上跑整个 `second_transaction_supplement_three_crash_injection` 二进制（7 条用例，大档 `#[ignore]`），
四段报告里记录核对器那一行的原样输出：

| 哪一段 | 崩溃状态 | 记录核对器跑了 | `root_without_record` | `claimed_state_missing_unit` |
|---|---|---|---|---|
| 偏向抬 F 之后回退的取样点 | 68 | 68 次 | **0 次** | **0 次** |
| 快档 | 93 | 93 次 | **0 次** | **0 次** |
| 写死的一段历史，段内真子集全枚举 | 104 | 104 次 | **0 次** | **0 次** |
| 抬 F 落进回退空档那一段历史 | 121 | 121 次 | **0 次** | **0 次** |

原样一行（快档那一段）：

```
崩溃状态上 checker 跑了 93 次；记录核对器跑了 93 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次
```

⇒ **两类在崩溃状态上都恒返回 default**（合计 386 个崩溃状态，零例外）。
这本身不是错——判据本来就该在对的代码上全绿；问题在于**报告读不出「这一格有没有判别力」**，
而背景材料第三节 K3 自己问的就是这一句：「要是某一类在崩溃状态上恒返回 default，这一格与没接逐字相同，而报告不会说」。

### 3.2 打中（硬）：把判据删掉，全部用例照样绿

副本变异 **A**（保留 `check_records_against` 的调用与计数器，只把结论从通过条件里拿掉）：

```
-        if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
-        {
+        if violations.is_empty() && disagreement.is_none()
+        {
```

在干净副本上施加它之后跑：

```
cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection
→ test result: ok. 7 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 15.63s
cargo test -q -p singlefs-harness --lib crash_injection
→ test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 16.36s
```

⇒ **12 条用例全绿。** `crash_injection.rs:706` 那个合取项今天**一格判别力都没有**：
删掉它与留着它，两份跑出来的判定逐字相同。
这一跑在两份互不相干的副本上各做了一次（第二次的存档是
`research/prompts/m2-supp3-item3-code-r2-opus-model/run-fresh-copy-mutation-A.txt`，同样 7 + 5 全绿）。

四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：

1. **分不分辨臂**：分辨。两条臂是「`:706` 带着记录核对器的结论」与「不带」，它们在今天全部用例上的观测**完全相同** ——
   这正是「不分辨」这件事本身被一个会红的实验坐实了（实验红不了 ⇒ 臂分不开）。
2. **被判的系统当时看不看得到判别它的东西**：看得到。`record_check` 就在手里（`:696-704` 算出来、两个计数器也加了），
   是通过条件把它丢了之后没有任何用例去读那两个计数器（除了写死历史那条用例读 `record_root_without_record`，见 3.4）。
3. **满足判据字面的哪一个分句**：背景材料 K3 的第三句 ——「某一类在崩溃状态上恒返回 default，这一格与没接逐字相同」。
   今天两类都恒 default，所以两类都落进这一句。
4. **跑前条款给的每个改法在这几格上还中不中**：第一轮攻方的改法 B（把记录核对器接进来）在这几格上**中**：
   接是接上了，判据链却接在一个今天恒真的条件上；它给报告添了两个恒 0 的计数器，没添判别力。

### 3.3 变异表第 183 行钉住的是计数器，不是判据

`crates/mutations.tsv` 第 183 行（用户 2026-09-20 定案第 3 条）的替换文是
`        let record_check = RecordCheck::default();` —— 它**连同 `tally.record_checks += 1;` 一起删掉**，
点名的必红用例是 `crash_points_are_reproducible_proper_subsets_sorted_by_segment`，
那条用例判的是 `first.tally.record_checks == first.tally.crash_points`（`crash_injection.rs:1314-1317`）。
⇒ 它红，红的原因是**计数器归零**，不是「少判出了什么」。变异 A 把计数器留着、只丢判据，这条用例就绿了（3.2 实测）。

**这两条合起来**：今天钉住「记录核对器接进来了」的唯一一条检查，是一条**数它跑了几次**的断言；
「它判得出东西」这一半，一条检查都没有。

### 3.4 举一个能让 `root_without_record` 判红的崩溃状态

要它判红，得让「某次发布的根槽写已在盘上，而那次发布的 journal 记录一份都不在」
（`crash.rs:544-549`）。今天的持久顺序（`transaction.rs:2129` 的注释：单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA）
把记录与根切在两段里，**更早的段整段持久**这条规则于是保证了「根在 ⇒ 记录也在」——所以干净构建上它恒 0，这是对的。

把那道屏障拿掉（`crates/mutations.tsv` 第 67 行「步 3：零单元发布在记录与根之间少一道屏障」，原样施加），
记录与根并进同一段，段内真子集立刻摆得出这个状态。副本上跑同一个测试二进制，写死那段历史那条用例判红，
原样输出（截到关键几行）：

```
写死的这段历史上崩溃状态判红：[
    CrashPointFinding {
        signature: RecordCheck {
            aspects: [
                "root_without_record",
            ],
        },
        seed: HistorySeed(
            0,
        ),
        crash_point: CrashPoint {
            segment_index: 6,
            persisted_within_the_segment: [
                false,
                false,
                true,
            ],
            step: Operation(
                0,
            ),
            operation_kind: Some(
                CloseAndMountWritable,
            ),
        },
        observation: FailureObservation {
            ...
            violations: [],
            panic: None,
            ...
            model_disagreement: None,
            ...
            record_check: RecordCheck {
                root_without_record: true,
                claimed_state_missing_unit: false,
            },
        },
```

**这个崩溃状态**：写死那段历史（`second_transaction_supplement_three_crash_injection.rs:286-300` 那一段）的第 6 段，
段内三个写 = 两份 journal 记录 + 一个根槽 FUA，持久集 `[false, false, true]` ——
**两份记录都扣下、后发的根槽先持久**。三件事值得写进判决：

1. **`violations` 是空的、`model_disagreement` 是 None**：池级 checker 与理想模型都判绿，
   **只有记录核对器判得出**。第一轮 K4-4 说的「这两类正是随机历史才摆得出的形态」在这里坐实了。
2. **这个状态属于 K1 这一轮新开的那一类**：`[false, false, true]` 就是
   `withholds_a_write_before_a_persisted_one()` 为真的形状，第一轮的前缀截断一个都摆不出来
   ⇒ **K1 的修补是 K3 这唯一一次判红的前提**，两条改法是锁死在一起的，不能只采一条。
3. 三段历史上它各判红 1、2、1 次（偏回退档 1、写死历史 2、抬 F 那段 1），而**快档 94 个崩溃状态上仍然是 0 次**
   ——快档抽不到它。

### 3.5 叠加实验：这一次判红，判据链其实还是被计数断言接走的

在变异 67 之上再叠变异 A（判据丢掉、计数留着）：

```
cargo test -q -p singlefs-harness --test second_transaction_supplement_three_crash_injection
→ test result: FAILED. 6 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 15.44s
→ crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:344:5:
→ assertion `left == right` failed: 今天的代码上没有「根在而它那次发布的记录一条都不在」的状态
```

- 单施加变异 67：**3 条用例红**（都红在 `new_findings.is_empty()`）。
- 变异 67 + 变异 A：**1 条用例红**，红在用例第 345 行那条**计数断言** `record_root_without_record == 0`。

⇒ 记录核对器在这条路上今天的全部判别力，都可以由那一条计数断言替代；
`:706` 的判据链只在「别的用例也想认出它」时才多出两条红。**判别力实际挂在用例里写死的一条 `== 0` 上。**

### 3.6 `claimed_state_missing_unit`：一样都没有盯着，而且更难摆出来

- 两份用例里**没有任何一条断言**提到 `record_claimed_state_missing_unit`
  （`grep -n record_claimed_state_missing_unit crates/singlefs-harness/tests/second_transaction_supplement_three_*.rs` 命中 0 次；
  全仓 `crates/singlefs-harness/tests/*.rs` 里这个名字命中 9 次，9 次全在层 0 那三份用例里）。
  它连 3.5 那条「计数断言」这一层兜底都没有。
- 它判红要的形状（`crash.rs:550-563`）：恢复实际走的根 txg ≥ 某次发布，而那次发布写出的某个单元**全部副本都不在盘上、
  而且没被更晚的写盖过**。今天的持久顺序把单元与记录也切在两段里 ⇒ 干净构建上恒 0（这是对的）。
- 副本变异 **C**（把带单元的发布那一道屏障也拿掉：`transaction.rs` 里
  「单元写 → `CommitStep::Barrier` → journal 记录」那一道，这是 `transaction.rs:2129` 写着的持久顺序自己的第一道）——
  这是一个真的持久顺序缺陷。跑同一个测试二进制：

```
崩溃状态上 checker 跑了 93 次；记录核对器跑了 93 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次   ← 快档
→ test result: FAILED. 6 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 9.74s
→ crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:319:5:
→ 这段历史摆得出的崩溃状态不少于 100 个：86
```

  **`claimed_state_missing_unit` 在四段报告上全是 0**，一次都没判出这个缺陷；
  唯一红的那条断言是「崩溃状态 ≥ 100 个」（用例第 319-323 行）（并段之后能全枚举的短段变少，掉到 86）——
  与判据毫无关系的一条规模断言，纯属撞上。

- **再往深里枚举，它还是 0**。变异 C 之下把段内真子集枚举到段长 14（`EveryProperSubsetOfShortSegments { segment_writes: 14, sampled_in_longer_segments: 8 }`），
  写死那段历史的 12 个写的段（10 个单元写 + 2 个 journal 记录，并段之后正是这个形状）**整段全枚举**，
  **12347 个崩溃状态**，原样输出：

（下面第一行把「种类」那一串中间的 8 个 `unit_write` 略去了，原样一行在模型目录的运行记录里；后两行是原样）

```
段 16：12 个写，候选 true，种类 ["unit_write", …略去 8 个…, "unit_write", "journal_record", "journal_record"]
枚举到段长 12：崩溃状态 12347 个；记录核对器 root_without_record 0 次、claimed_state_missing_unit 0 次；新发现 0 条
枚举到段长 14：崩溃状态 12347 个；记录核对器 root_without_record 0 次、claimed_state_missing_unit 0 次；新发现 0 条
```

  ⇒ **不是「抽样抽不到」，是这条判据在崩溃注入这条路上今天疑似结构性不可达。** 全枚举里当然含着
  「某个单元的两份副本都扣下、那两份 journal 记录持久、别的写全持久」这个状态，它照样没判红。
  机理**没查清**，两个候选（都只是读代码推的、没验）：① 恢复在单元缺席时不肯把那条记录施加上去，
  于是 `report.effective_root` 抬不到那次发布的 txg，`crash.rs:550` 的前置就不成立；
  ② `written_over_later`（`crash.rs:531-540`）按整条记录流前缀算「被后面的写盖过」，把这些副本豁免掉了。
  **这一格留给主 agent 现查**，本腿只给出「全枚举也判不红」这个可核的事实。

## 四、这条腿自己提的三条改法（被攻过零轮，只在副本上量过）

| # | 改法 | 修哪一格 | 代价 / 连带 |
|---|---|---|---|
| 甲 | 给两条判据各加一条**靶向用例**：写死一段历史 + 写死一个崩溃状态（段号 + 持久集），断言那一格判红。形态照层 0 那一路的 `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`（`crates/mutations.tsv` 第 191 行点名的那条） | K3-④、K3-⑨ | 今天 `CrashPointDraw` 只有 `Sampled` 与 `EveryProperSubsetOfShortSegments` 两支，**没有「就摆这一个状态」的入口**，要加第三支；加了之后 `crates/mutations.tsv` 才有地方钉「判据被丢掉就红」 |
| 乙 | `draw_a_proper_subset` 改成**按扣下数分层抽**：先抽「扣下几个」（至少把 1、2 与 n−1 各留一个配额），再抽「扣下哪几个」 | K1-③、K3-⑨ | 落在 `crates/mutations.tsv` 第 182 行的射程里（那一行钉的是「段内只截前缀」），改了要重验；**这个周期已经量过的判红全部要在新分布上重量**，代价与换种子基同量级 |
| 丙 | `crates/mutations.tsv` 加一行「保留计数器、只丢判据」（就是第 3.2 节那个变异 A），点名必红用例 | K3-④ | **今天加不进去**：没有任何一条用例会因为它红（3.2 实测 12 条全绿）⇒ 丙**必须与甲一起做**，单做丙只会给变异表加一行永远绿的行，门禁 59 号会整道红 |

三条改法的每一格标「量过 / 推的」：

| 格 | 甲 | 乙 | 丙 |
|---|---|---|---|
| K1-③（长段上扣下 ≤2 个整类零覆盖） | 推的（靶向用例只钉一个点，不改抽样分布） | 推的（按代码推：分层抽必然给「扣下 1 个」配额，没实现没跑） | 不修 |
| K3-④（`:706` 的判据无判别力） | 推的（没实现没跑；要先有「摆指定状态」的入口） | 推的 | 推的 |
| K3-⑨（`claimed_state_missing_unit` 零覆盖零检查） | 推的 | 推的 | 不修 |
| 「今天的判别力挂在哪」 | 量过（3.5：变异 67 + A 之后只剩用例第 345 行那条计数断言） | 量过（2.2 的分布与 3.6 的变异 C 实测） | 量过（3.2、3.3 两次跑） |

⚠️ 甲、乙、丙**都没有实现、没有跑过**，只有它们要修的那几格是量过的。
按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严…算没被攻过」，交用户时要标「零轮」。

## 五、没打中的形状（试过什么、取样范围多大）

| 试的形状 | 取样范围 | 结果 |
|---|---|---|
| 干净构建上让 `root_without_record` 判红 | 四段报告合计 386 个崩溃状态（快档 93、偏回退 68、写死历史 104、抬 F 那段 121） | 一次都没有。**这不算打中**：干净代码上它本来就该恒绿，这一格量的是「有没有判别力」，不是「有没有 bug」 |
| 干净构建上让 `claimed_state_missing_unit` 判红 | 同上 386 个 | 一次都没有，同上 |
| 拿 `crash.rs:550` 只比 txg 不比实例这一点构造**假阳**（回退之后两条时间线的 txg 撞上，被抛弃那一支的单元缺席被判成「恢复自称新态而单元缺席」） | **只读代码、没跑**：`effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)` 丢掉了 `InstanceGeneration` | 没构造出来。干净构建上偏回退档 68 个、抬 F 那段 121 个崩溃状态都没红，说明这一批历史里 txg 没撞上；**要不要立一笔欠账，这条腿没有数支持** |
| 让快档自己抓住变异 67 | 快档 94 个崩溃状态 | 抓不住（`root_without_record` 仍是 0）。抓住它的是三段写死 / 偏向档的历史 |
| 让变异 C（单元与记录之间少一道屏障）被任何一条 oracle 抓住 | 四段报告合计 309 个崩溃状态（69 + 93 + 86 + 61），另加把并起来那个 12 个写的段全枚举的 12347 个 | 三条 oracle（checker、模型、记录核对器）**一条都没抓住**；红的是一条规模断言（`crash_points >= 100`） |
| 大档（500 段 × 40 步 × 8 个崩溃状态）上这几个数长什么样 | **没跑**（这条腿没量它的挂钟代价） | 不知道；快档的分布结论（2.2）是按均匀掩码推的，段长分布换一批历史会变 |
| 第一轮攻过的角度（K1-a 屏障验收、K1-c 枚举域比例、K1-d 起点段、K1-f 冷启动、K1-g 断言、K4-*、K5、K6） | —— | **按派发提示没有重攻**；本报告只在 3.4、3.6 里把变异 67 当工具用，判的是记录核对器，不是屏障那一格 |

## 六、这条腿自己的限度

1. **全部数都是副本上量的。** 副本是 `rsync -a --exclude target --exclude .git` 拷出来的，除了本报告点名的三个变异与两份量具用例，
   源码与工作区逐字相同（收尾时拿 `sha256sum -c research/prompts/m2-supp3-item3-code-r2-start-snapshot.sha256`
   在副本里核过：`crates/` 那 10 项全 OK，只有 `.claude/kb/checks-owed.md` 因为别的会话在改而对不上，与这一轮无关）。
   按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」，
   这些数**不进 kb**，主 agent 要在入库装置上重跑（第四节前面那张表里的命令原样可跑）。
2. **`crash_points = 93` 这个数与装置自己报的一致，但它是这个测试周期的种子基上的一次观测。**
   种子基一换（`SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 的文档注释说下一个周期要重抽），2.1、2.2 的全部绝对数都要重量；
   只有 2.2 的**分布形状**（Binomial(n, ½)）是按 `crash_injection.rs:869-873` 的代码推的，不依赖种子基。
3. **「没打中」的那几格只抽了一次样**，按「一条腿只抽一次样不算一次观测」，它们不足以支撑否定结论；
   本报告没有把任何一格「没打中」当成结论用（第五节第一、二行明写了为什么不算打中）。
4. **没判 K2、K4、K5、K6**，按分工表那四格归别的腿；也没判正推腿与辩方腿的任何一格。
5. **`crash.rs` 的两条判据本身没被攻。** 本报告判的是「它们在崩溃注入这条路上有没有判别力」，
   没判「它们写得对不对」——层 0 那一路有自己的用例与变异行（`crates/mutations.tsv` 第 191 行）罩着，不在这一轮的攻击面里。
6. **三条改法（甲乙丙）没实现、没跑**，只有它们要修的那几格量过。
7. **自己踩过一次「还原了却没重新编」的坑，记在这里，因为它会咬到复跑的人。**
   用 `mv 文件.orig 文件` 把变异还原回去时，被还原的文件带着**旧的 mtime**，cargo 按 mtime 判要不要重编，
   于是**不重编**——`diff` 与 `sha256sum` 都说源码干净，跑出来的却还是上一个变异的二进制。
   这条腿因此一度量到一组与干净构建不同的数（段长 {1, 2, 4, **12**, **20**}、段内有洞 39、崩溃状态 93）。
   识破它的办法是**另拷一份全新的副本重量**：新副本上原样复现了干净构建那一组数
   （段长 {1, 2, 4, **10**, **18**}、段内有洞 40、候选段 1432、镜像对 2782），两组差的正是变异 C 的签名
   （journal 记录那 392 个长度 2 的段并进了前面的单元段：10 → 12、18 → 20）。
   ⇒ **本报告采用的全部干净构建的数都在新副本上复现过**（`run-fresh-copy-*.txt` 三份），
   那一组受污染的数已作废、不进报告正文；还原变异一律用 `cp` 或还原后 `touch`，不要用 `mv`。

## 七、没做什么

- **没跑 K2、K4、K5、K6**（按分工表归别的腿），也没读别的腿这一轮的提示与产出（禁读清单里的两类文件一个都没开）。
- **没重攻第一轮攻过的角度**（K1-a 到 K6 逐格）；变异 67 只当工具用，判的是记录核对器有没有判别力，不是屏障那一格。
- **没跑大档**（500 段 × 40 步 × 8 个崩溃状态）、没量它的挂钟代价；快档一轮在副本上约 15 秒（整个测试二进制），
  段内枚举到段长 14 的那一次 1454 秒。
- **没改工作区任何文件**：改动只落在草稿目录里的两份副本上；写进仓的只有这份报告与模型目录
  `research/prompts/m2-supp3-item3-code-r2-opus-model/`（11 个文件，第一之二节有 sha256）。
- **没跑门禁阶段**：`.claude/gate.d/stage-owners.tsv` 里没有登记给 `three-way-attack` 的阶段
  （`awk -F'\t' -v me=three-way-attack …` 一行都没输出）。
- **没入库**：副本上量的数不进 kb，也没写进任何实验页；要引它，主 agent 按第一之二节的命令在入库装置上重做一次。
- **没查清**：`claimed_state_missing_unit` 为什么全枚举也判不红（3.6 给了两个候选机理，都只是读代码推的）；
  `crash.rs:550` 只比 txg 不比实例这一点能不能构造假阳（没跑）。
