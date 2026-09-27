t mod 3 = 0:
txg t+1, region 1, disk 1
txg t+2, region 2, disk 0
Total empty publishes: 2

t mod 3 = 1:
txg t+1, region 2, disk 0
txg t+2, region 0, disk 0
txg t+3, region 1, disk 1
Total empty publishes: 3

t mod 3 = 2:
txg t+1, region 0, disk 0
txg t+2, region 1, disk 1
Total empty publishes: 2

1. The smallest total number of empty publishes is 2, achieved in the t mod 3 = 0 and t mod 3 = 2 cases. The largest total is 3, achieved in the t mod 3 = 1 case. This answer would be falsified if the t mod 3 = 1 case had a total count of 2 or less.
