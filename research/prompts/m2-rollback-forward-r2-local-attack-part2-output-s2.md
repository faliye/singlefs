t mod 3 = 0:
txg t+1: region 1, disk 1
txg t+2: region 2, disk 0
Total: 2

t mod 3 = 1:
txg t+1: region 2, disk 0
txg t+2: region 0, disk 0
txg t+3: region 1, disk 1
Total: 3

t mod 3 = 2:
txg t+1: region 0, disk 0
txg t+2: region 1, disk 1
Total: 2

1. The smallest total number of empty publishes is 2, occurring in the t mod 3 = 0 and t mod 3 = 2 cases. The largest total is 3, occurring in the t mod 3 = 1 case. This answer would be falsified if the t mod 3 = 1 case had a total count different from 3.
