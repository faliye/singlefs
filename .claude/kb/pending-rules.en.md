# Pending rules

[中文](pending-rules.md) · **English** · [日本語](pending-rules.ja.md)

Clauses that cannot be turned into failing checks and therefore cannot enter rules are kept in `pending-rules.md` as pending (`.claude/singlefs-ai-sop/rules/sop-first.md` "Boundaries": keep what cannot be turned into checks in kb as pending, do not write them as rules).
Each entry records the original text, where it originally came from, why it was withdrawn, and what is missing to make it a rule. The original text is preserved verbatim in code blocks as it stood when withdrawn.

## Search for existing before creating verification harness or test entry point — Pending

| Item | Contents |
|---|---|
| Where it was | Shared SOP `rules/sop-first.md` section "Search for existing before creating verification harness or test entry point", since 0.0.64 (2026-10-11 release) |
| Withdrawn | 2026-10-16, user decided "withdraw all clauses that cannot be made into checks and rely only on review to kb pending" |
| Status | Pending: not a rule, does not bind execution |
| What is missing to become a rule | A check that can go red: go red if a new entry script has no `# gate-similar:` in file header, and verify whether named harnesses exist; "whether two harnesses judge the same thing" still cannot be judged |

```text
## 立验证装置或测试入口之前，同样先找已有的

射程：验证装置（枚举、对拍、跑真设备或外部工具的那一套代码）、测试入口（`test-script-discipline.md` 说的入口脚本），以及跑它们的门禁阶段与快慢两档。

1. 先列已有的装置与入口各判什么：判哪件事、哪一批对象（哪几条流、哪个枚举域）、用哪个判法、在哪个阶段跑。
2. 有判同一件事、同一批对象的，新的并进已有的那一套：换一种算法、换到加速硬件、加一条流、加一个域，都写成它的加速路径或新项，用它的名字、它的入口、它的阶段与它的快慢两档，不另起名字、不另立阶段、不另分一套快档与全量。加速路径留参照路径、两边对拍，照 `test-script-discipline.md`「用上现有的硬件加速」。
3. 只有一部分重叠（几条流、一段域相同）的，重叠的那一部分并进去，其余作为同一套里的新项。
4. 仓里已经并存两套判同一件事的，先合成一套，再跑它的剪枝跑或全量；合之前不跑两套各自的全量。
5. 非单独立不可，在新入口的文件头写 `# gate-similar: <已有的装置或入口> <为什么不并进它>`；一个像的都没有，写 `# gate-similar: 无 <查过哪些>`。

「立验证装置或测试入口之前，同样先找已有的」门禁还不判：新入口的文件头写没写 `gate-similar`、两套装置判的是不是同一件事，都靠 review。
```

## When fixing a bug, thoroughly inspect places with similar logic and code across the entire codebase — Pending

| Item | Contents |
|---|---|
| Where it was | Shared SOP `rules/code-discipline.md` section "When fixing a bug, thoroughly inspect places with similar logic and code across the entire codebase", since 0.0.66 (2026-10-15 release) |
| Withdrawn | 2026-10-16, same withdrawal decision as "Search for existing before creating verification harness or test entry point" |
| Status | Pending: not a rule, does not bind execution |
| What is missing to become a rule | A check that can go red: go red if bug-fix commit message lacks the three items "what was searched, how many hits, how many places modified"; whether search was thorough or judgment correct still cannot be judged |

```text
## 修了 bug，要严查全库里逻辑和代码相似的地方

射程：改了一处缺陷，不分代码、脚本、门禁、文档，也不分语言版本。

**判据：修完之后，说得出全库里还有哪几处和它是同一个模式，每一处是改了，还是查过确认没有这个缺陷。**

1. 把缺陷拆成两种形态：**代码形态**（同样的写法、同一个函数或正则、抄过的同一段）和**逻辑形态**（同样的假设、同样的边界、同一个口径，写法不同也算）。
2. 两种形态都全库搜。代码形态按写法 grep；逻辑形态按「谁在做同一件事」找：同一个概念的别的实现、同一类输入的别的处理、判同一条判据的别的检查，含测试、示例、各语言版本。输出不截断，先数再看（`verify-before-claiming.md`「截断过的输出也是窄的那一句」）。
3. 每个命中逐个判。有同一缺陷的，在同一次改动里一起改；没有的，写明为什么不受影响。
4. 每个顺带修掉的相似处，各配一条先证明会红的测试（`show-me-test.md`）。
5. 提交信息写：搜了什么（关键字或命令）、命中几处、改了几处、没改的几处各自的原因。搜不到别的相似处，写「搜了 <什么>，0 处」，不写「没有」。

缺陷在口径层（同一个量在两套装置里算法不同）的，相似处就是另一套装置，照 `show-me-test.md`「装置里，还是口径上」立跨装置的检查。
```

## Thoroughly fix discovered problems without citing sunk costs as an excuse — Pending

| Item | Contents |
|---|---|
| Where it was | Shared SOP `rules/code-discipline.md` section "Thoroughly fix discovered problems without citing sunk costs as an excuse", written into upstream workspace on 2026-10-16, unreleased |
| Withdrawn | 2026-10-16, same withdrawal decision as "Search for existing before creating verification harness or test entry point" |
| Status | Pending: not a rule, does not bind execution |
| What is missing to become a rule | Checks that go red can only cover a small literal fraction: go red if arguments or output conventions of shared scripts change while callers in consumer projects still follow old conventions; "whether fix is thorough" and "whether refactoring is warranted" cannot be judged |

```text
## 查出来的问题要彻底修，不拿沉没成本当理由

射程：review、门禁、自证、别人报来的问题，核实成立的每一条。

- **核实成立就修，在这一次改动里修完。** 不写「没在这一次动」「下次再说」「先这样」。修不了的（要使用者定的、要别的仓改的）明说卡在哪、交给谁，不默认放着。
- **判据是怎么改才对，不是已经写了多少。** 已经写好的代码、已经发出去的版本、改动面大，都不是不改或只打补丁的理由；结构错了就重构，不在错的结构上加绕过的分支。
- **不走捷径。** 不放宽检查、不加豁免、不改用例的预期让它变绿、不把报错吞掉；改的是出错的原因，不是它露出来的症状；修完照「修了 bug，要严查全库里逻辑和代码相似的地方」查相似处。
- **改了共享的东西，同一次改动里把用它的地方一起改。** 脚本的参数、输出、文件名与目录约定变了，本仓的调用方、各语言版本、使用者项目里的调用方与样本一起改到位，再发版；只改上游、下游还按旧约定跑的，算没改完。
- 修的是缺陷本身，当场修；为防它再犯而新立的门禁，立的时机照 `sop-first.md`「边界」一节。
```

## Pin positive controls per unit, three formal runs match — Pending

| Item | Contents |
|---|---|
| Where it was | Three clauses in shared SOP `rules/test-discipline.md` section "Positive controls must run against every tested arm", since 0.0.64 (2026-10-11 release) |
| Withdrawn | 2026-10-16, same withdrawal decision as "Search for existing before creating verification harness or test entry point" |
| Status | Pending: not a rule, does not bind execution |
| What is missing to become a rule | A check that can go red: compare formal run lists across pre-run registration, unit test unit table, and rerun registration, go red if missing anywhere; whether control readings per cell suffice still cannot be judged |

```text
- **对照按单元钉，不并起来判。** 正式跑切成几格（档、族、域这类）的，每一格判了哪几条判据，那一格自己的正式跑里就要有那几条判据的对照读数；单元测试按格写死各格的期望集，不拿几格的读数取并集去判。
- **登记时逐格写出对照由哪一跑担。** 跑前登记写判据集合时，按格列出每条判据的对照落在哪一条正式跑；某一格没有一条正式跑担得起，就补一跑，或者那一格不判那条判据。
- **登记、单元测试、复跑三处的正式跑逐条对得上。** 登记里的每条正式跑，在单元测试的单元表与复跑登记里都找得到；少一处就是那一格没人对照。
```
