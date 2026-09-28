1 2, 6.8966% 2, 6.8966% S2 (system_configuration_slot times 2, total writes 29) If S2 listed a different count for system_configuration_slot writes
2 1, 25% 1, 25% P3 (four sequence points, one barrier after system-configuration slot) If P3 described a different number of sequence points related to system-configuration
3 A:8192, B:1024 A:8192, B:8192 C3 (checksum covers entire slot) If C3 did not state the checksum covers the entire 4096-byte slot
4 21:63 21:500 E1 and E2 If E1 or E2 data was different
5 20000; ratio cannot be computed 20000; ratio cannot be computed E1's rule, P4, P5 If P4/P5 did not state every publish is a checkpoint or if facts existed about tail_inline replay blocks dependence on checkpoint frequency
