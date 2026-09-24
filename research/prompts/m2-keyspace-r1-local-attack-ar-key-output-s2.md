leaf_cap floor((16384 - 135) / 20) = floor(16249 / 20) = floor(812.45) = 812 This would be refuted by: FACT 1's node size, FACT 2's header size, or FACT 3's leaf-entry width changing

fanout floor((16384 - 135) / 96) = floor(16249 / 96) = floor(169.26041666666666) = 169 This would be refuted by: FACT 1's node size, FACT 2's header size, or FACT 6's internal-entry width changing

h_4gib L(1) = ceiling(211968 / 812) = ceiling(261.044335) = 261 L(2) = ceiling(261 / 169) = ceiling(1.544378698224852) = 2 L(3) = ceiling(2 / 169) = ceiling(0.011834319526627219) = 1 height is 3 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 4GiB slot count changing

n_4gib 261 + 2 + 1 = 264 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 4GiB slot count changing

h_64gib L(1) = ceiling(4144128 / 812) = ceiling(5103.6059) = 5104 L(2) = ceiling(5104 / 169) = ceiling(30.1994) = 31 L(3) = ceiling(31 / 169) = ceiling(0.1834319526627219) = 1 height is 3 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 64GiB slot count changing

n_64gib 5104 + 31 + 1 = 5136 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 64GiB slot count changing

h_1tib L(1) = ceiling(67058688 / 812) = ceiling(82584.5911) = 82585 L(2) = ceiling(82585 / 169) = ceiling(488.668639) = 489 L(3) = ceiling(489 / 169) = ceiling(2.893491124260355) = 3 L(4) = ceiling(3 / 169) = ceiling(0.01775147928994083) = 1 height is 4 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 1TiB slot count changing

n_1tib 82585 + 489 + 3 + 1 = 83078 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 1TiB slot count changing

h_16tib L(1) = ceiling(1073691648 / 812) = ceiling(1322280.3547) = 1322281 L(2) = ceiling(1322281 / 169) = ceiling(7824.14852) = 7825 L(3) = ceiling(7825 / 169) = ceiling(46.30177514792899) = 47 L(4) = ceiling(47 / 169) = ceiling(0.2781065088757396) = 1 height is 4 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 16TiB slot count changing

n_16tib 1322281 + 7825 + 47 + 1 = 1330154 This would be refuted by: FACT 7's leaf capacity, FACT 8's fanout, or FACT 9's 16TiB slot count changing

wc_k1 level 1: min(1, 1322281) = 1 level 2: min(1, 7825) = 1 level 3: min(1, 47) = 1 level 4: 1 total = 1 + 1 + 1 + 1 = 4 This would be refuted by: FACT 10's level chain for 16TiB or FACT 11's worst case rules changing

wc_k10 level 1: min(10, 1322281) = 10 level 2: min(10, 7825) = 10 level 3: min(10, 47) = 10 level 4: 1 total = 10 + 10 + 10 + 1 = 31 This would be refuted by: FACT 10's level chain for 16TiB or FACT 11's worst case rules changing

wc_k100 level 1: min(100, 1322281) = 100 level 2: min(100, 7825) = 100 level 3: min(100, 47) = 47 level 4: 1 total = 100 + 100 + 47 + 1 = 248 This would be refuted by: FACT 10's level chain for 16TiB or FACT 11's worst case rules changing
