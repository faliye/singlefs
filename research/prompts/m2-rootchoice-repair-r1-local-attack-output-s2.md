Judgment A
Part one grounding fact 1
Part two break change fact 1 to replace the first shippable version of this filesystem has no mechanism at all to relocate a root ring slot to a different physical location with the first shippable version of this filesystem has a mechanism to relocate a root ring slot to a different physical location
Part three consequence no other fact directly conflicts with this change

Judgment B
Part one grounding fact 3
Part two break change fact 3 to replace a location that cannot be read does not count as proven absent with a location that cannot be read does count as proven absent
Part three consequence no other fact directly conflicts with this change

Judgment C
Part one grounding fact 1
Part two break change fact 1 to replace every writable mount writes at least one new row into the instance table with some writable mounts do not write a new row into the instance table
Part three consequence fact 2 conflicts because fact 2 states every writable mount writes a row

Judgment D
Part one grounding fact 1 and fact 5
Part two break change fact 1 to replace this second absorbing state no longer requires the pool to be close to full at all with this second absorbing state requires the pool to be close to full at all
Part three consequence fact 6 conflicts because fact 6 states this second absorbing state no longer requires the pool to be close to full at all

Judgment E
Part one grounding fact 1 and fact 7
Part two break change fact 1 to replace once the switch reserve is unobtainable the mount is forced read only with once the switch reserve is unobtainable the mount is not forced read only
Part three consequence fact 7 conflicts because fact 7 states if even one of these fails to hold the mount is forced read only and reserve unobtainable is one of the failures

Judgment F
Part one grounding fact 1
Part two break change fact 1 to replace a row can no longer be deleted because deleting a row is itself a write and requires a writable mount with a row can be deleted even when the mount is read only
Part three consequence no other fact directly conflicts with this change

Judgment 2a-3a
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 10
Part three verdict no conflict found

Judgment 2a-3b
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 10
Part three verdict no conflict found

Judgment 2a-3c
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 8
Part three verdict adopting 2a would change the behavior that 3c is meant to keep unchanged so there is a conflict

Judgment 2b-3a
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 15
Part three verdict no conflict found

Judgment 2b-3b
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 15
Part three verdict no conflict found

Judgment 2b-3c
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 15
Part three verdict adopting 2b would change the behavior that 3c is meant to keep unchanged so there is a conflict

Judgment 2c-3a
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 9
Part three verdict no conflict found

Judgment 2c-3b
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 9
Part three verdict no conflict found

Judgment 2c-3c
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 9
Part three verdict no conflict found

Judgment 2a-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 14
Part three verdict no conflict found

Judgment 2b-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 15
Part three verdict no conflict found

Judgment 2c-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 14
Part three verdict no conflict found

Judgment 3a-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 11
Part three verdict no conflict found

Judgment 3b-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 11
Part three verdict no conflict found

Judgment 3c-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 11
Part three verdict no conflict found

Judgment 4-2b
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 15
Part three verdict no conflict found

Judgment 4-3b
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 11
Part three verdict no conflict found

Judgment 4-5
Part one same byte no shared byte is named in any fact given above
Part two same invariant fact 14
Part three verdict no conflict found
