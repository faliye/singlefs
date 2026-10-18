# 事实表丁甲：报告

- 事实表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁甲/facts-丁甲.tsv，数据行 56（`wc -l` 57 行含表头），编号 F800–F855。其中变更类 41 行，「新立：」11 行（F810 F813 F817 F818 F820 F827 F828 F839 F846 F847 F851），「只改措辞」4 行（F852–F855）。
- 候选表：/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁甲/candidates-丁甲.tsv，1120 行（1020 个不同的行；`wc -l` 1121 行含表头）。
- 没建副本，没起编译，没跑 `--changes`。认领了草稿目录。
- 读法：切片文件 changes-丁甲.md 只有字符级碎片，我对切片里列的每个载体另读了 `git diff ce1457c3 -U0 --word-diff=plain`，覆盖：agent-common、agents/*（含 clerk、mutation-triage 的纯新增）、main-agent、kb/INDEX、pitfalls、prior-art、term-renames、tooling、verification-build、vm-harness、rules 下 changelog-format、format-evolution、fs-design、implementation-first、implementation-workflow、mutation-sampling、path-moves、three-way-inference、verification，新增三条规则整份读了。

## 两条命令

```
python3 scripts/kb/stale-candidates.py --check-facts <草稿目录>/facts-丁甲.tsv --base ce1457c3
```
退出码 6（预期：片里没有 H 编号，67 条变更记录全报没罩）。汇总行：
`  ✗ 事实表没罩全：67 条变更记录没罩、0 行正则坏了、0 行检索词在基准零命中、0 行在结束零命中、0 行检索词太宽、0 行多余的 &&、0 行冒充新立、0 行冒充只改措辞、0 行宽词没抽查`
输出末行原样：
`     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；检索词写新旧说法都会提到的概念名词，基准与结束两版都要命中；太宽（超过上限）才用 && 并上第二个概念名词收窄；事实真变了的不许写成只改措辞或新立；命中超过 100 行的先抽 10 行自判相干不相干，精度低就改窄，精度够就在「出处」列写「宽词已抽查：抽 10 行、相干 N 行」`
除「没罩住」之外的红都是 0（上面汇总行逐项为 0）。

```
python3 scripts/kb/stale-candidates.py --facts <草稿目录>/facts-丁甲.tsv --base ce1457c3 --out <草稿目录>/candidates-丁甲.tsv
```
退出码 0，输出末行：
`  ✓ 候选表 /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-丁甲/candidates-丁甲.tsv：1120 行（1020 个不同的行）`

每组候选行数（`awk` 数出）：F800=173 F801=25 F802=7 F803=13 F804=10 F805=13 F806=21 F807=5 F808=33 F809=47 F811=83 F812=49 F814=8 F815=29 F816=7 F819=53 F821=12 F822=32 F823=33 F824=68 F825=5 F826=4 F829=80 F830=4 F831=25 F832=27 F833=17 F834=11 F835=4 F836=5 F837=26 F838=27 F840=17 F841=14 F842=23 F843=24 F844=22 F845=31 F848=7 F849=8 F850=18。

## 只改措辞的载体（F852–F855，不出候选）

- F852 术语改名（层 0 → 崩溃点重放，第一条流 / 第二条流 → 新池新建文件流 / 覆盖写固定脚本流，录 / 判 → 录入段 / 判定段，layer0-* 名 → crash-point-replay-*）：`.claude/agent-common.md`、`.claude/agents/implementation-writer.md`、`three-way-attack.md`、`tooling-writer.md`、`.claude/main-agent.md`、`.claude/kb/term-renames.md`、`.claude/kb/verification-build.md`、`.claude/rules/` 下相关规则。
- F853 路径搬迁（`research/scripts` → `scripts/`，mutate.sh → `scripts/mutation/`，变异表 → `scripts/mutation/tables/`，`kb/invariants.md` → `crates/singlefs-checker/invariants/`，上游 SOP 的 test-script-discipline.md → `.claude/rules/`）：`experiment-designer.md`、`experiment-runner.md`、`mutation-triage.md`、`kb/INDEX.md`、`kb/verification-build.md`、`rules/fs-design.md`、`rules/verification.md`、`rules/format-evolution.md`。
- F854 行号引用换符号锚、删日期尾巴与「今天」、补简称、删论证与「为什么」尾巴、位置指代改写：`kb/prior-art.md`、`kb/vm-harness.md`、`kb/pitfalls.md`、`kb/term-renames.md`、`agent-common.md`、`agents/*.md`、`rules/fs-design.md`、`format-evolution.md`、`path-moves.md`、`implementation-first.md`、`mutation-sampling.md`、`three-way-inference.md`。
- F855 三方论证没有核查员的残留句清理：`agent-common.md`、`agents/three-way-attack.md`、`three-way-defense.md`、`three-way-forward.md`。
- 注意：F853 把 test-script-discipline 路径搬迁算措辞，但那条规则整份是新增（入口脚本不带参数就是剪枝跑全部项等），这一面另写成了变更行 F819。

## 命中超过 100 行的检索词（抽查）

- 只有 F800（`线程上限`，结束那一版 173 行）超过 100；其余最大的是 F811 83 行、F829 80 行、F824 68 行，都不到门槛。
- F800 抽 10 行（固定种子 17）：10 行都在讲线程上限这个量，相干 10 行；其中 8 行是 records 里某次派发写「线程上限各 10」之类的事件句，预期判「事件句不改」。命中分布：records 138 行、.claude/agents 23 行、.claude/kb 8 行、agent-common 2 行、main-agent 1 行、rules 1 行。「出处」列已写「宽词已抽查：抽 10 行、相干 10 行」。
- 精度提示：F800 的 138 行 records 命中多半是事件句；没收窄是因为旧式子（核数 ÷ 重活数）在别处可能换说法，宁宽勿窄。要缩工作量可以让逐行判的人对 records 里的事件句快速过。

## 要主 agent 现查（不在事实表形态里）

1. `.claude/agents/implementation-writer.md` 第 41 行（4a 条）还写「受影响的层 0 流与崩溃枚举用例」，是 3603ace4 之后没改净的旧名，同一行后半已改成崩溃点重放；`.claude/agent-common.md` 第 47 行新规矩正好禁这种写法。
2. `.claude/kb/experiments/` 下 21 个实验页的复跑块里还有 `mutations/e<号>_….tsv` 这种相对路径（例：106、108），而 `scripts/experiments/mutations/` 目录不存在，变异表在 `scripts/mutation/tables/`。这些页不在我这一片，路径是旧的；是否算冻结证据要主 agent 判。
3. 私有主机信息：新规矩只由门禁 doc-text 的 `private-values` 格判「新加的行」，旧行不判。我只核了 `multi-host.env` 里的 `PEER_SSH_HOST` 一个键，值在被 git 跟踪的文件里零命中；其余键没核（值不适合我这边逐个展开），建议主 agent 自己对 `multi-host.env` 的全部取值各 `git grep -F` 一遍。
4. `层 ?0` 在结束那一版的现状载体里仍有 113 行命中（records 100、kb 11、agent-common 1、agents 1），`layer0|Layer0` 有 459 行（kb 245、records 212）。大半应是 term-rename-exempt 登记的冻结证据与改名记录本身，我没逐行核哪些没登记；F852 是「只改措辞」所以不出候选，需要的话主 agent 另跑 `scripts/kb/sweep-term.py --check`。
5. 切片里 `.claude/kb/prior-art.md` 新增了「十、快照下的死块记账、搬被共享的块与 discard 时机」整节（2026-10-12 取得、他家事实表），是纯新增调研，没有哪句旧话变了，所以没写事实行。
6. `.claude/rules/three-way-inference.md` 材料检查新增退出码 7（`@标题` 取法节尾不一致）与 8（清单「不抄」理由点名的节附录里没有），旧话只说退出码 4、6，没写事实行（被引的地方少），主 agent 觉得需要再加。

## 没做什么

- 没跑 `--changes`；片里没有 H 编号，所以 `--check-facts` 的「没罩住」67 条是预期，没处理。
- 没改任何仓内文件，没改 `stale=`；没建副本；没起编译。
- 事实表只覆盖丁甲这一片的载体；别的片（kb 决策、实验页、里程碑、records、技能、门禁、钩子）里同一件事的改动没读，但检索词是全仓搜的，候选里会带出它们的旧句。
- 日常验证崩溃状态数压 20 万量级、GPU 清场与单个 GPU 测试二进制的 `--test-threads 1` 这几句在 verification.md 里有改写，我判为措辞调整没单列。
- 候选表的精度没逐行验过，只抽了 F800 的 10 行；其余 40 组命中都不到 100 行。
- 什么会推翻：若 `git diff ce1457c3` 在上面那些载体里还有我没读到的事实性改动（例如 tooling.md 里被整段删除的本地模型实测表之外的事实变化），事实表就漏了那一行。
