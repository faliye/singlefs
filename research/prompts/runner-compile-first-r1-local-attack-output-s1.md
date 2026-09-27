1
allow
context: neither hook is wired to Read tool
if this call was rejected by any hook, which would not happen as no hooks are wired for Read

2
allow
F12 F10
if the command was rejected by bash-command-detector.sh despite the destination path being outside crates/

3
allow
F3 F8
if the target path did not match F8's pattern but the hook rejected it

4
allow
F6 F3 F2
if the call was rejected by decide_scope or decide_compile_first despite matching F6 and not being .rs

5
allow
F5 F3 F2
if the target path did not match F5's pattern but the hook rejected it

6
allow
F7 F3
if the call was rejected by decide_scope despite matching F7

7
allow
F14 F16
if the command was rejected by bash-command-detector.sh despite the self-test expecting no rejection

8
allow
F14
if the command was rejected by bash-command-detector.sh despite no recognized write operations

9
allow
F4 F3 F2
if the call was rejected by decide_scope or decide_compile_first despite matching F4 and agent_type not experiment-runner

10
allow
F3
if the call was rejected despite no agent_type field
