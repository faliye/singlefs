1.1 torn_both-Today: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.2 torn_both-Jia: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: F26 for mount 1; assumption: none; overturned if: all system configuration slots were self-verified

1.3 torn_both-Jia-read-error: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: similar to F26; assumption: none; overturned if: devices had two self-verified slots each

1.4 torn_both-(a): mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.5 torn_both-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.6 torn_d0-Today: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.7 torn_d0-Jia: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: F26 for mount 1; assumption: none; overturned if: all system configuration slots were self-verified

1.8 torn_d0-Jia-read-error: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: similar to F26; assumption: none; overturned if: devices had two self-verified slots each

1.9 torn_d0-(a): mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.10 torn_d0-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.11 torn_d0_d1_persisted-Today: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.12 torn_d0_d1_persisted-Jia: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: F26 for mount 1; assumption: none; overturned if: all system configuration slots were self-verified

1.13 torn_d0_d1_persisted-Jia-read-error: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: similar to F26; assumption: none; overturned if: devices had two self-verified slots each

1.14 torn_d0_d1_persisted-(a): mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.15 torn_d0_d1_persisted-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: each mount writes system configuration slots for the new instance; the torn slot is rewritten properly; C is not involved; facts: F25 for mount 1, F20 for mount 2 and 3; assumption: none; overturned if: the system configuration slot write for mount 1 was torn causing acquisition failure

1.16 tree table-Today: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's tree table was readable

1.17 tree table-Jia: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's tree table was readable

1.18 tree table-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's tree table was readable

1.19 tree table-(a): mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: F28; assumption: none; overturned if: C's tree table was readable

1.20 tree table-(b)-from-tree-table: mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: similar to F28; assumption: none; overturned if: C's tree table was readable

1.21 allocation record tree-Today: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's allocation record tree was readable

1.22 allocation record tree-Jia: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's allocation record tree was readable

1.23 allocation record tree-Jia-read-error: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots and reuses C's data; later mounts write normally; C leaves root ring after mount 1; facts: F27; assumption: none; overturned if: C's allocation record tree was readable

1.24 allocation record tree-(a): mount 1 = R; mount 2 = R; mount 3 = R; after mount 3: STAYS; disk between mounts: no write for any mount; C is not involved; facts: F28; assumption: none; overturned if: C's allocation record tree was readable

1.25 allocation record tree-(b)-from-tree-table: mount 1 = W; mount 2 = W; mount 3 = W; after mount 3: STAYS; disk between mounts: mount 1 writes system configuration slots but does not reuse C's data due to isolation; later mounts write normally; C remains in root ring until overwritten; facts: from (b) behavior; assumption: none; overturned if: C's tree table was unreadable

2.1 none