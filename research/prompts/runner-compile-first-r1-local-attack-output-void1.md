1
a) allow
b) Read tool is not wired to any hooks per context, so no hooks run to reject it
c) if the Read call was rejected by a hook, that would prove the verdict wrong

2
a) allow
b) F12 and F10: destination path in /tmp/... is outside crates/ so not rejected by compile_first_verdict
c) if the Bash call was rejected despite destination outside crates/, that would prove the verdict wrong

3
a) allow
b) F2 (e) and F8: target path in /tmp/... doesn't start with crates/ so decide_compile_first allows; write-scope pattern F8 permits /tmp/claude-1000/** 
c) if the Edit call was rejected despite path matching allowed scope, that would prove the verdict wrong

4
a) allow
b) F2 (d) and F6: target file .tsv doesn't end with .rs so decide_compile_first allows; write-scope pattern F6 explicitly allows crates/mutations.tsv
c) if the Edit call was rejected despite matching F6's allowed pattern, that would prove the verdict wrong

5
a) allow
b) F2 (e) and F5: target path in research/... doesn't start with crates/ so decide_compile_first allows; write-scope pattern F5 permits research/e7-index-bench/src/bin/**
c) if the Write call was rejected despite matching F5's allowed pattern, that would prove the verdict wrong

6
a) allow
b) F2 (d) and F7: target file .toml doesn't end with .rs so decide_compile_first allows; write-scope pattern F7 explicitly allows research/e7-index-bench/Cargo.toml
c) if the Edit call was rejected despite matching F7's allowed pattern, that would prove the verdict wrong

7
a) allow
b) F13, F14, and F16: compile-then-swap script invocation is explicitly allowed; non-shell wrapper capped.sh prevents command scanning; self-test confirms no rejection
c) if the Bash call was rejected despite F16's test case expecting pass, that would prove the verdict wrong

8
a) allow
b) F14: python3 script invocation produces no recognized write operations; F10: scratch directory paths outside crates/ would not be rejected even if scanned
c) if the Bash call was rejected despite no recognized write operations being scanned, that would prove the verdict wrong

9
a) allow
b) F2 (a) and F4: agent_type implementation-writer not experiment-runner so decide_compile_first allows; write-scope pattern F4 permits crates/**
c) if the Edit call was rejected despite agent_type not being experiment-runner, that would prove the verdict wrong

10
a) allow
b) F3 (a) and F2 (a): no agent_type field triggers F3's immediate allow; main agent case also satisfies F2's agent_type not experiment-runner condition
c) if the Edit call was rejected despite no agent_type field, that would prove the verdict wrong