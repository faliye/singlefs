Q1
C1.1 K22 2 CONFIRMED
This would be refuted by: K22 showing exit code 0
C1.2 K16 K21 2 2 CONFIRMED
This would be refuted by: K16 showing exit code 0 or K21 showing exit code 0
C1.3 K15 2 CONFIRMED
This would be refuted by: K15 showing exit code 0
C1.4 K3 2 CONFIRMED
This would be refuted by: K3 showing exit code 0
C1.5 K18 0 CONFIRMED
This would be refuted by: K18 showing exit code 2

Q2
C2.1 K25 2 CONFIRMED
This would be refuted by: K25 showing exit code 0
C2.2 K2 2 CONFIRMED
This would be refuted by: K2 showing exit code 0
C2.3 K12 2 CONFIRMED
This would be refuted by: K12 showing exit code 0

Q3
C3.1 K23 2 CONFIRMED
This would be refuted by: K23 showing exit code 0
C3.2 K9 2 CONFIRMED
This would be refuted by: K9 showing exit code 0
C3.3 K1 2 CONFIRMED
This would be refuted by: K1 showing exit code 0

Q4
C4.1 K24 2 CONFIRMED
This would be refuted by: K24 showing exit code 0
C4.2 K8 2 CONFIRMED
This would be refuted by: K8 showing exit code 0

Q5
C5.1 K7 K26 2 2 CONFIRMED
This would be refuted by: K7 showing exit code 0 or K26 showing exit code 0
C5.2 K4 2 CONFIRMED
This would be refuted by: K4 showing exit code 0
C5.3 K6 2 CONFIRMED
This would be refuted by: K6 showing exit code 0
C5.4 K19 2 CONFIRMED
This would be refuted by: K19 showing exit code 0
C5.5 K18 0 CONFIRMED
This would be refuted by: K18 showing exit code 2

Q6
C6.1 K10 2 CONFIRMED
This would be refuted by: K10 showing exit code 0
C6.2 K27 2 CONFIRMED
This would be refuted by: K27 showing exit code 0
C6.3 K11 0 CONFIRMED
This would be refuted by: K11 showing exit code 2

Q7
C7.1 K20 2 CONFIRMED
This would be refuted by: K20 showing exit code 0

Q8
C8.1 K13 2 CONFIRMED
This would be refuted by: K13 showing exit code 0
C8.2 K14 2 CONFIRMED
This would be refuted by: K14 showing exit code 0

Q9
NONE NOT APPLICABLE UNTESTED
This would be refuted by: a K item showing a wrapped command like bash -c herd7 with exit code 0

Q10
K17 2 CONTRADICTED
This would be refuted by: K17 showing exit code 0

Q11
NONE NOT APPLICABLE CONFIRMED
This would be refuted by: a K item showing a gate stage script that should be judged by name but the code reads its contents

Q12
NONE NOT APPLICABLE UNTESTED
This would be refuted by: a K item showing a gate stage like 50 with exit code 2

Q13
C10.1
K17 exit code 2 but the code allows it (returns None) when it should reject it

Q14
NONE

Q15
C1.1 layer0-stage
C1.2 layer0-cargo
C1.3 layer0-cargo
C1.4 layer0-binary
C1.5 None
C2.1 qemu-stage
C2.2 qemu-system
C2.3 vm-bench
C3.1 herd7-stage
C3.2 lkmm
C3.3 herd7
C4.1 crates-mutation-stage
C4.2 crates-mutation-mutate
C5.1 full-cargo
C5.2 full-cargo
C5.3 full-cargo
C5.4 check-sh
C5.5 None
C6.1 gate-sh
C6.2 gate-staged
C6.3 None
C7.1 replay-all-stage
C8.1 e152
C8.2 e152
C9.1 NOT IN SOURCE
C10.1 NOT IN SOURCE
C11.1 classify function's name checks
C12.1 None

Q16
layer0-stage C1.1
layer0-cargo C1.2
layer0-binary C1.4
qemu-stage C2.1
qemu-system C2.2
vm-bench C2.3
herd7-stage C3.1
lkmm C3.2
herd7 C3.3
crates-mutation-stage C4.1
crates-mutation-mutate C4.2
full-cargo C5.1
check-sh C5.4
gate-sh C6.1
gate-staged C6.2
replay-all-stage C7.1
e152 C8.1
