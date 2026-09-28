1.1 torn_both-Today: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F03, F25, F13; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: during mount1, the system configuration write is torn, causing refusal

1.2 torn_both-Jia: mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not rewritten; C does not leave root ring
facts: mount1: F26, F13, F16; mount2: F26, F13, F16; mount3: F26, F13, F16
assumption: none
overturned if: the system configuration slots self-verify, so no refusal

1.3 torn_both-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F15, F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: config slots have device error, causing refusal

1.4 torn_both-(a): mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.5 torn_both-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.6 torn_d0-Today: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F04, F25, F13; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: during mount1, the system configuration write is torn, causing refusal

1.7 torn_d0-Jia: mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not rewritten; C does not leave root ring
facts: mount1: F26, F13, F16; mount2: F26, F13, F16; mount3: F26, F13, F16
assumption: none
overturned if: the system configuration slots self-verify, so no refusal

1.8 torn_d0-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F15, F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: config slots have device error, causing refusal

1.9 torn_d0-(a): mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.10 torn_d0-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.11 torn_d0_d1_persisted-Today: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F05, F25, F13; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: during mount1, the system configuration write is torn, causing refusal

1.12 torn_d0_d1_persisted-Jia: mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not rewritten; C does not leave root ring
facts: mount1: F26, F13, F16; mount2: F26, F13, F16; mount3: F26, F13, F16
assumption: none
overturned if: the system configuration slots self-verify, so no refusal

1.13 torn_d0_d1_persisted-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F15, F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: config slots have device error, causing refusal

1.14 torn_d0_d1_persisted-(a): mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.15 torn_d0_d1_persisted-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not rewritten; C does not leave root ring
facts: mount1: F18, F20; mount2: F18, F20; mount3: F18, F20
assumption: none
overturned if: there is an abandoned root for torn rows, but none specified

1.16 tree table-Today: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F08, F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: during mount1, tree table becomes readable

1.17 tree table-Jia: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: config slots have unreadable or fail self-verify, causing refusal

1.18 tree table-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: config slots have device error, causing refusal

1.19 tree table-(a): mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not relevant; C does not leave root ring
facts: mount1: F28, F08, F23; mount2: F28, F08, F23; mount3: F28, F08, F23
assumption: none
overturned if: tree table becomes readable after reread

1.20 tree table-(b)-from-tree-table: mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not relevant; C does not leave root ring
facts: mount1: F28, F08, F23; mount2: F28, F08, F23; mount3: F28, F08, F23
assumption: none
overturned if: tree table becomes readable after reread

1.21 allocation record tree-Today: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F09, F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: during mount1, allocation record tree becomes readable

1.22 allocation record tree-Jia: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: config slots have unreadable or fail self-verify, causing refusal

1.23 allocation record tree-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring
facts: mount1: F27, F23; mount2: F20; mount3: F20
assumption: none
overturned if: config slots have device error, causing refusal

1.24 allocation record tree-(a): mount 1 = R; mount 2 = R; mount 3 = R
after mount 3: STAYS
disk between mounts: nothing; torn slot not relevant; C does not leave root ring
facts: mount1: F28, F09, F23; mount2: F28, F09, F23; mount3: F28, F09, F23
assumption: none
overturned if: allocation record tree becomes readable after reread

1.25 allocation record tree-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W
after mount 3: STAYS
disk between mounts: mount1 writes new system configuration slots; mount2 writes new system configuration slots; mount3 writes new system configuration slots; torn slot not relevant; C does not leave root ring; isolation in memory for C's slots
facts: mount1: F09, F23, F24; mount2: F20; mount3: F20
assumption: none
overturned if: tree table unreadable, so cannot walk

2.1 no cells where picked under assumption
2.2 no cells where two trajectories seemed possible
