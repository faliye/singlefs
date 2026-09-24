RED-CHECK no, NONE (FACT-4,5,6,7,8,9,10; no check detects constant correctness across core and harness)  
REST-CHECK REST-1: no, NONE (FACT-7; GATE-56 checks file naming only, not binary behavior)  
REST-CHECK REST-2: no, NONE (FACT-3 and FACT-7; GATE-56 verifiesans form, not content; three-way review is human)  
REST-CHECK REST-3: yes, GATE-59-SCRIPT checks mutation rows for exact match and test failure (FACT-8)  
FIRST-FINDER NOBODY (no scheduled check or hook detects incorrect constants in shared code)  
SHARED-FILE no, fixing W1 requires new code correctness check; no, fixing W1 requires new code correctness check  

RED-CHECK yes, WRITE-GUARD-HOOK (FACT-4; blocks writes outside pattern like crates/singlefs-core)  
REST-CHECK REST-1: no, NONE (FACT-7; GATE-56 checks file naming only, not binary behavior)  
REST-CHECK REST-2: no, NONE (FACT-3 and FACT-7; GATE-56 rans form, not content; three-way review is human)  
REST-CHECK REST-3: yes, GATE-59-SCRIPT checks mutation rows for exact match and test failure (FACT-8)  
FIRST-FINDER WRITE-GUARD-HOOK (FACT-4; blocks writes to forbidden paths before execution)  
SHARED-FILE no, fixing W2 requires WRITE-SCOPE-TABLE adjustment; no, fixing W2 requires WRITE-SCOPE-TABLE adjustment  

RED-CHECK no, NONE (FACT-11 and FACT-12; planning document lacks field for harness type)  
REST-CHECK REST-1: no, NONE (FACT-7; GATE-56 checks file naming only, not binary behavior)  
REST-CHECK REST-2: no, NONE (FACT-3 and FACT-7; GATE-56 rans form, not content; three-way review is human)  
REST-CHECK REST-3: yes, GATE-59-SCRIPT checks mutation rows for exact match and test failure (FACT-8)  
FIRST-FINDER NOBODY (FACT-11 and FACT-12; no field exists to specify trigger condition)  
SHARED-FILE no, fixing W3 requires EXPERIMENT-DESIGNER-DEF update; no, fixing W3 requires EXPERIMENT-DESIGNER-DEF update  

W1 overturn condition: if a scheduled check exists that compares constant values between core and harness code  
W2 overturn condition: if WRITE-GUARD-HOOK is modified to permit writes to crates/singlefs-core or crates/singlefs-checker  
W3 overturn condition: if EXPERIMENT-DESIGNER-DEF includes a field for "in repo harness" and a gate validates it  
UNKNOWN COUNT 0
