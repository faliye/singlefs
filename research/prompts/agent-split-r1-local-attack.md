You are one of four independent reviewers of an execution plan for a software project. Your stance is ATTACK: assume the plan is wrong and look for concrete sequences that break it. Answer in English. Do not use any markdown emphasis such as bold or italic. Number your answers 1 to 4. For each answer, either give a concrete sequence of steps (who dispatches whom, with what input, which file is written or read, what goes wrong) or say plainly that you could not construct one and list the shapes you tried.

Settled facts (do not dispute them; use them):

S1. Project rule on three-way inference: every inference goes through three legs. An inference means a design judgment, a trade-off conclusion, or any basis used to overturn or establish an entry in the project's decision log. The three legs are a local model, a cloud model called Sonnet, and a cloud model called Opus. "Not duplicated" means the three legs get different angles, not just different model names; each leg is assigned a different stance, for example forward derivation, backward reasoning (assume the conclusion is wrong and look for what should then be observed), or counterexample search.

S2. The main agent judges; the judgment is not a vote. The main agent's job is to judge, not to act as a fourth inference.

S3. Project evidence rule: a check path is a second, independent way of making the same judgment, and the two paths must not share the same code, the same sample, or the same tool. The check path itself must be proven to go red: inject a known fault that the second path should catch and see whether it actually catches it.

S4. Observation by the main agent: a general-purpose subagent, forbidden to use tools, reproduced verbatim five sentences that each appear only in one of: the project instruction file, the shared rules, the project rules, the user-level instruction file, and the memory index. This indicates that this subagent type has those instruction files in its context; only five sentences were sampled, one per source, and not every line was checked. Custom subagents defined in project files were not tested.

S5. Today each round's leg prompts are written by hand. Cloud legs write reports into files under research/prompts and are told not to read other legs' outputs of the same round; nothing in the tooling stops them from reading any file.

The plan under review, P:

P1. Split the work into 15 project-local subagents with fixed definitions. Each definition fixes its input, its write scope, its output format, and a section on what it did not do. The main agent keeps: talking to the user; writing each round's questions, criteria, pre-registered clauses and the observation line on what the implementation looks like today; writing the verdict and deciding adoption; the judgment part of settling or overturning a design decision; choosing whom to dispatch with what input, and spot-checking reports after reading them; git operations; and the end-of-round progress report.

P2. The three-way family has six agents in the first batch: a forward leg (Sonnet), an attack leg (Opus), a defense leg (Sonnet), a local attack leg and a local defense leg (each is a Sonnet subagent that translates the question into an English prompt without markdown emphasis, checks each paraphrase against the source and adds any missing qualifier, runs the local model in the foreground at least twice, keeps the void copy and reruns when the corruption gate exits with code 5, delivers the samples, word counts and gate records, and neither adds reasoning for the local model nor judges whether its answer is right; the local defense leg looks for reasons why the side that was judged out or attacked still stands), and a verifier. The verifier checks, in each leg report, every knowledge-base citation (file name, line number and quoted text), every product line and every rerun command: whether the line exists in that knowledge-base file, whether the quote is verbatim, whether the rerun is byte-identical, whether the sha256 matches, and whether a line number was mistakenly taken from the background material. It returns a check table and does not state whether a finding should be adopted.

P3. Dispatch is flat: only the main agent dispatches, dependent steps run serially (materials builder, then legs, then verifier), independent legs run in parallel, and no agent definition is given the tool for dispatching agents.

P4. User decision: every agent that lands goes through an adversarial review before use. Step 1: write the definition, its write-scope hook arguments and its gate samples together; every delivery rule in the definition points to the rule text. Step 2: adversarial review; the material carries the full definition and the rule text it rests on; the forward leg checks whether what the definition asks is what the rules require; the attack leg looks for which step goes wrong or out of scope when the agent is dispatched per its definition; the local attack and local defense legs each look for one point; findings are written back and attacked again; a finding counts only when a majority of three rounds breaks through. Step 3: one real trial run, archived with the before-and-after write-scope comparison.

P5. Rollout order: step 0 builds a gate stage for definition files, a write-scope hook and a before-and-after comparison script, each with samples that must go red, and probes a minimal custom subagent for three things: whether it inherits the rules and the memory, whether a hook attached to it takes effect, and whether it can get around not being given the dispatch tool. Batch 1 is forward leg, attack leg, defense leg, local attack, local defense, verifier, gate triage, repository sweep. Batches 2 and 3 come later.

P6. The plan credits two isolation benefits to splitting. First: legs do not share context; reading files still has to be forbidden by the definition. Second: an experiment designer agent does not have prior conclusions in its context unless it reads them itself, and it must list what it read.

Answer these four items:

1. Bootstrap. Under P4 and P5, the batch 1 leg definitions must themselves pass an adversarial review, and that review is carried out by legs. Give a concrete ordering of the first reviews: which legs review the first leg definitions, and are those legs the definitions under review or ad hoc prompts? Construct a sequence where a defect in a leg definition is hidden in its own review because the reviewing leg carries the same defect, or say you could not.

2. Verifier independence. Under S3 and S4, the verifier and the legs it checks may inherit the same rule text into their context and may be the same model. Is the verifier a second path that shares no code, sample or tool with the legs? Construct a concrete defect in a leg report that both the leg and the verifier would miss for the same reason, or say you could not. Then name the injected fault that would prove the verifier goes red.

3. Isolation claims in P6. Name concrete observations, obtainable in step 0 or in the first trial run, that would show each of the two isolation benefits does not hold. For each, say how the observation could be collected without trusting the subagent's own report.

4. For each of your answers 1 to 3, state what observation would refute your answer.
