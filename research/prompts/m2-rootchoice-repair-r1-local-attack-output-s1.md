judgment A
grounding fact 1
break change fact 1 to say the first shippable version has a mechanism to relocate a root ring slot to a different physical location
consequence fact 6

judgment B
grounding fact 3
break change fact 3 to say a location that cannot be read counts as proven absent
consequence fact 1

judgment C
grounding fact 1 fact 2
break change fact 2 to say not every writable mount writes a row
consequence fact 1

judgment D
grounding fact 5 fact 6
break change fact 5 to say rows per shard equals 1000
consequence fact 6

judgment E
grounding fact 7
break change fact 7 to say if the reserve an instance switch would need cannot be obtained but other conditions hold the mount is still writable
consequence fact 7

judgment F
grounding fact 1
break change fact 1 to say deleting a row does not require a writable mount
consequence fact 2

judgment 2a 3a
same byte no shared byte named in any fact given above
same invariant fact 9
verdict no conflict found

judgment 2a 3b
same byte no shared byte named in any fact given above
same invariant fact 9 fact 12
verdict no conflict found

judgment 2a 3c
same byte no shared byte named in any fact given above
same invariant fact 8 fact 11
verdict no conflict found

judgment 2b 3a
same byte no shared byte named in any fact given above
same invariant fact 15
verdict no conflict found

judgment 2b 3b
same byte no shared byte named in any fact given above
same invariant fact 15 fact 12
verdict no conflict found

judgment 2b 3c
same byte no shared byte named in any fact given above
same invariant fact 15
verdict no conflict found

judgment 2c 3a
same byte no shared byte named in any fact given above
same invariant fact 9
verdict no conflict found

judgment 2c 3b
same byte no shared byte named in any fact given above
same invariant fact 9 fact 12
verdict no conflict found

judgment 2c 3c
same byte no shared byte named in any fact given above
same invariant fact 8 fact 11
verdict no conflict found

judgment 2a 5
same byte no shared byte named in any fact given above
same invariant fact 9 fact 14
verdict no conflict found

judgment 2b 5
same byte no shared byte named in any fact given above
same invariant fact 15 fact 14
verdict no conflict found

judgment 2c 5
same byte no shared byte named in any fact given above
same invariant fact 9 fact 14
verdict no conflict found

judgment 3a 5
same byte no shared byte named in any fact given above
same invariant fact 2 fact 14
verdict no conflict found

judgment 3b 5
same byte no shared byte named in any fact given above
same invariant fact 12 fact 14
verdict no conflict found

judgment 3c 5
same byte no shared byte named in any fact given above
same invariant fact 11 fact 14
verdict no conflict found

judgment 4 2b
same byte system configuration structure
same invariant fact 15
verdict no conflict found

judgment 4 3b
same byte no shared byte named in any fact given above
same invariant fact 12
verdict no conflict found

judgment 4 5
same byte no shared byte named in any fact given above
same invariant fact 14
verdict no conflict found
