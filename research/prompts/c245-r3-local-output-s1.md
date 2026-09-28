1 2 writes, 6.9% 2 writes, 6.9% (2/29)*100 (S2) S2 changes system_configuration_slot count
2 1 sequence point, 25% 1 sequence point, 25% 1/4 (P3) P3 changes sequence points count
3 8192 bytes 8192 bytes C1 and C3 (slot size and checksum) A fact row stating writes are only for content size
4 21:63 21:500 E1 and E2 For pbs=4096 it does not differ from given 21:500
5 cannot be computed cannot be computed E1 rule (20000 checkpoints), missing fact on tail_inline dependency A fact row explaining how tail_inline's blocks-to-replay changes with checkpoint frequency
