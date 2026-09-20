<!-- knowledge-sync -->
# rules-only-howto 阶段同步

阶段：接上游 SOP 0.0.53「规则文件只写怎么做」，把七份项目规则与 `CLAUDE.md` 里的论证、日期与定案记述回扫掉，刷版本戳，给 55 号那句不带参数的 `wait` 标明退出码在哪收（2026-09-20）。基准 `95fd301`，提交 `7d2d25c`，时间 UTC。
触发文件：.claude/rules/fs-design.md、.claude/rules/three-way-inference.md、.claude/rules/mutation-sampling.md、.claude/rules/format-evolution.md、.claude/rules/implementation-workflow.md、.claude/rules/implementation-first.md、.claude/rules/path-moves.md、CLAUDE.md、.claude/gate.d/55-qemu-first-transaction.sh、research/scripts/rules-sweep-audit.py、.singlefs-ai-sop-version
回扫本体由另一个会话做完（`records/2026-09-20-rules只写怎么做.md`），它没写同步记录；这一份由主 agent 补，只判「别处还指不指得到被删掉的内容」。

## 搜索

判据没丢：`python3 research/scripts/rules-sweep-audit.py` 拿 git 里那一版当基准逐片段查它今天还在不在，报出来的每一处逐条判过（是判据就放回正文，是历史或论证才删）；`--selftest` 通过（删一句会红、原样不动不红、改字会红）。
回扫后过闸：`RULES_LINT_DIR=.claude/rules RULES_LINT_FILES="CLAUDE.md .claude/agents/*.md .claude/agent-common.md .claude/main-agent.md .claude/skills/*/SKILL.md" bash .claude/singlefs-ai-sop/scripts/rules-lint.sh .` → 扫了 29 份文件 1625 行，七条判据各 0 命中。
别处指不指得到：全仓搜「规则文件「小节标题」」这种引用（`grep -rno 'rules/[a-z-]*\.md》\?「[^」]*」' .claude records research README.md`）26 处，按小节标题匹配指不到的 5 处；这 5 处逐句拿原文全文搜（`grep -rqF`），4 句还在正文里（回扫把它们从小节标题降成了粗体引子，不是删掉），1 句真删了。
死链：全量门禁的「文档里的链接与「第 N 节」指向到不到得了」阶段，回扫早先留下的两条指向 `.claude/rules-rationale/` 的链接已随规则改写去掉，本轮零命中。

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| research/mutations/e129_thin_rmw.tsv:8 | 按 .claude/rules/mutation-sampling.md「判定为等价的，把等价性写成留档，再换一个真会改行为的补上」， | 改了：那句话随回扫从 `mutation-sampling.md` 删掉（它本来就是共享 SOP 的话），引用改指 `.claude/singlefs-ai-sop/rules/test-discipline.md` 里的原话 |
| .claude/hooks/runner-dispatch-guard.sh:43 | 说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」… | 不改：那句话还在 `three-way-inference.md` 正文里（回扫只把它从小节标题降成粗体引子），引用逐字命中 |
| research/scripts/agent-handover.py:5 | .claude/rules/three-way-inference.md「撞了限额、被停或报错的腿」那一段 | 不改：同上，那句话还在正文里 |
| research/prompts/alloc-basis-r3-opus-model/run-probe.sh:3 | .claude/rules/three-way-inference.md「一条腿在副本装置上量出来的数」 | 不改：句子还在；而且 `research/prompts/` 是原样保存的证据，路径以外不回改 |
| research/prompts/alloc-basis-r2-opus-model/run-probe.sh:3 | .claude/rules/three-way-inference.md「一条腿在副本装置上量出来的数」 | 不改：同上 |
