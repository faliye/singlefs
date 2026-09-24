judgment A: grounding: fact 1 break: change fact 1 from the first shippable version of this filesystem has no mechanism at all to relocate a root ring slot to a different physical location to the first shippable version of this filesystem has a mechanism to relocate a root ring slot to a different physical location consequence: fact 1 would no longer hold true

judgment B: grounding: fact 1 and fact 3 break: change fact 3 from a location that cannot be read does not count as proven absent to a location that cannot be read counts as proven absent consequence: fact 3 would no longer hold true

judgment C: grounding: fact 1 and fact 2 break: change fact 2 from every writable mount writes a row to not every writable mount writes a row consequence: fact 2 would no longer hold true

judgment D: grounding: fact 1 fact 5 fact 6 break: change fact 5 from rows per shard equals the floor of 32768 minus 135 divided by the width of one row minus 1 to rows per shard equals 1000000 consequence: fact 5 would no longer hold true

judgment E: grounding: fact 1 and fact 7 break: change fact 7 to the conjunction that must all hold is the count of writable devices is at or above the floor labeled w and this mount has exclusively opened a strict majority of the devices in the pool and the instance table can be read consequence: fact 7 would no longer hold true

judgment F: grounding: fact 1 break: change fact 1 from deleting a row is itself a write and requires a writable mount to deleting a row does not require a writable mount consequence: fact 1 would no longer hold true

judgment 2a-3a: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2a-3b: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2a-3c: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2b-3a: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2b-3b: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 2b-3c: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 2c-3a: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2c-3b: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 2c-3c: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 2a-5: same byte: no shared on-disk field that both changes write to same invariant: fact 10 verdict: no conflict found

judgment 2b-5: same byte: no shared on-disk field that both changes write to same invariant same invariant: fact 15 verdict: no conflict found

judgment 2c-5: same byte: no shared on-disk field that both changes write to same invariant: fact 9 verdict: no conflict found

judgment 3a-5: same byte: the instance table rows from fact 2 same invariant: fact 2 verdict: no conflict found

judgment 3b-5: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 3c-5: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 4-2b: same byte: no shared on-disk field that both changes write to same invariant: no shared invariant verdict: no conflict found

judgment 4-3b: same byte: no shared on-disk field that both changes write to same invariant same invariant: no shared invariant verdict: no conflict found

judgment 4-5: same byte: the root ring slots from fact 1 same invariant: fact 1 verdict: no conflict found