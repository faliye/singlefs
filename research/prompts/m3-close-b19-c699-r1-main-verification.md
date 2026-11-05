# m3-close-b19-c699-r1 判决：C699 取号跳过往后怎么走（2026-11-05）

正文 `research/prompts/_m3-close-b19-c699-r1-body.md`；腿报告：正推 `research/prompts/m3-close-b19-c699-r1-sonnet-output.md`、攻方 `research/prompts/m3-close-b19-c699-r1-opus-output.md`（0e859070…）、攻方模型修补 `research/prompts/m3-close-b19-c699-r1-opus-b-output.md`（f318bb7b…）。候选：甲（照原计划加第四条件，sw-read 那一形登成残余）、乙（甲加读坏槽护住）、丙（不实现甲）。

## 一、核查

- 开工快照 `research/prompts/m3-close-b19-c699-r1-snapshot/` 七份 `sha256sum -c` 全 OK（主 agent 2026-11-05 现跑），主工作区就是开工那一刻的树。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-c699-r1-sonnet-output.md research/prompts/m3-close-b19-c699-r1-opus-output.md research/prompts/m3-close-b19-c699-r1-opus-b-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b19-c699-r1-background.md` 末行「✓ 核了 25 处引文、112 处裸行号，对上 25 处，没判 112 处（其中裸行号没有引文可比 112 处；逐处列在上面）」。腿引的 E198 产物行抽核：`research/results/e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out` 第 1422 行原文「E7RESULT name=verdict question=Q19 group=nrb point=main reference_interval_verdicts=holds:6 judgement=attached_not_judged」，与攻方「今天的 I-3.5 在 291 张上一张没红」相符。
- 攻方模型（`research/prompts/m3-close-b19-c699-r1-opus-b-model/`，只在上一份上补了模型 crate 的 `singlefs-format` 依赖）拷到 `~/.local/share/singlefs/work/m3-close-b19-c699-r1-main/opus-b-model` 复跑：6821 编过、第 1 次撞服务给的 4 GiB 上限、第 2 次因 `--work` 目录非空退 2，没出数；换新目录重交 6822 退 0（内存峰值 4.95 GiB），末行「OPUSMODEL name=done mode=full histories=261」。各组（产物 `~/.local/share/singlefs/work/m3-close-b19-c699-r1-main/opus-b-rerun-6822.log`）：
  - `group=nrb-recheck … errors=today_e1:6 … all_three_candidates_e1=0`；`group=rs1-recheck … errors=today_e1:285 … all_three_candidates_e1=0`（与 E198 第一段相同：今天的臂 6、285，甲 0）。
  - `group=sw-read-control … hole_pairs=315 truths=existed_versions_in_between:315 errors=today_e1:315,with_pointer_birth_e1:315`（与 E198 第二段相同）。
  - `group=skr-gap-fault … hole_pairs=153 truths=directly_bridged:140,existed_versions_in_between:13 errors=read_bad_gap_slot_guarded_e2:70,read_bad_or_zero_gap_slot_guarded_e2:140`（乙在真取号跳过上降成不判 70 对，并上全零槽是 140 对；今天的臂与甲 0）。
  - `group=sw-crash-wrapped … hole_pairs=684 truths=existed_versions_in_between:684 errors=read_bad_gap_slot_guarded_e1:54,read_bad_or_zero_gap_slot_guarded_e1:54,today_e1:684,with_pointer_birth_e1:54 accounting_unknown_pairs=261 all_three_candidates_e1=54`（三个候选一起认错 54 对，今天的臂 684）。
  - `group=rs-two-files … hole_pairs=75 truths=existed_versions_in_between:75 errors=today_e1:75 … all_three_candidates_e1=0 pool_checker_reference_interval=holds:80,violated:25 pool_checker_release_generation=holds:70,violated:35`（今天的判法在这一组认错 75 对，今天的池级 checker 在 105 张上 I-3.5 红 25 张、I-3.9 红 35 张；攻方预言 I-3.5 30 张，量到 25；这 25 张红是否全由认错引起，模型没分开，推的）。
  正推腿的乙那一列是推的（E198 没有乙的臂），以攻方模型量到的为准。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| 乙 | **出局（模型上量的）**：读坏的洞内落点槽在合法镜像上造得出（C698 认它是合法镜像），乙把真取号跳过降成不判 70 对（并全零槽 140 对），丢判别力；在 sw-crash-wrapped 上照样认错 54 对 | 第一节 skr-gap-fault、sw-crash-wrapped 两行 |
| 丙 | **出局（模型上量的）**：今天的判法在 nrb、rs1 认错 6、285 对（量的，E198 与第一节）、在 rs-two-files 认错 75 对，那一组今天的池级 checker 红 I-3.5 25 张、I-3.9 35 张；甲在这三组上 0 对 | 第一节 nrb-recheck、rs1-recheck、rs-two-files 三行 |
| 甲 | **照用户 2026-10-04 定案实现（量的）**：在量过的每一组上认错的对数不多于今天的臂（nrb 0 / 6、rs1 0 / 285、rs-two-files 0 / 75、skr-gap-fault 0 / 0、sw-read 315 / 315、sw-crash-wrapped 54 / 684），没有一组比今天差；残余两形是 sw-read（315 对）与 sw-crash-wrapped（54 对），三个候选在这两形上都认错 | 第一节全部 |
| 残余两形 | **另立账**：病根在甲、乙、丙共用的前提「指针出生 txg 等于 R.txg 就说明 R 是新实例的首版」，清扫那一版也满足它（正推 Q3、攻方 Q2.2）。攻方自提的臂丁（看 R 那条根的分配记录里有没有代落在 (R⁻, R) 之间）按推的能分开这两形、也不丢 skr，被攻过零轮、没建模型，登进 `.claude/kb/checks-owed.md` 新一行，要先量才能提成候选 | 攻方 Q2.2 |
| 问题单第 1 行 | **拆两句（正推 Q3，四问照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」）**：按字面验收，甲分不全（sw-read 一形，今天的臂同样错）；判臂只看分辨得出臂的格，判甲。F1 报的事实保留，不拿它判臂 | 正推 Q3；第一节 |
| 乙的两种写法（降成接不上 / 报部分没判）与 C701 | 乙出局，这一问不判 | — |

## 三、交用户的

不交新岔路：甲就是用户 2026-10-04 的定案，这一轮量到它在每一组上都不比今天差、乙与丙各有量到的代价。残余两形与臂丁登欠账，臂丁量过之后若成了要换判法的候选，再交用户。

## 四、被攻过零轮的

臂丁（攻方自提，没建模型）。不为它开轮，登欠账、先量。

## 五、去向

| 事 | 去向 |
|---|---|
| C699 甲的实现（`crates/singlefs-checker/src/walk.rs:865` `hole_in_the_candidate_set_between` 加第四条件与它的调用处，GPU 判器同改；正推给的代价：CPU 三处、GPU 一处，版本行 48 字用了 47、余 1 字够） | 下一批实现员（池级 checker 与 GPU 判器，测试代码，不开代码轮）；验收用 E198 的 nrb、rs1 两组与攻方 rs-two-files 那一组做会红的用例 |
| `.claude/kb/checks-owed.md` C699 那一行 | 写回：前置改成「已定：照甲实现」、去向随下一批；残余两形写进这一行或另立一行（由写回规格定），引这份判决第一节的组行 |
| 残余两形与臂丁 | 新登 `.claude/kb/checks-owed.md`：要拦的输入是 sw-read 与 sw-crash-wrapped 两形，候选是臂丁（零轮、没量），前置是先建量它的装置格 |
| E198 实验页问题单第 1 行 | 写回时照第二节「问题单第 1 行」那一格补一句判臂结论，F1 的判定照留 |
| 腿的复跑脚本拒绝非空 `--work`，与服务的放大上限重试配不上 | 记在调度记录，不另立账（模型复跑脚本是腿的一次性产物） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-05 不受影响：乙借的是它把读坏槽当「可能有根」的思路，乙出局，条款不动 |
