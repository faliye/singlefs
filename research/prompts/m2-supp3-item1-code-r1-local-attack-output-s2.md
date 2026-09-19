Z5-1-a
sector 10
Would be wrong if: old_read did not copy sector 10 into the output buffer.

Z5-1-b
sector 10
Would be wrong if: new_read did not copy sector 10 into the buffer.

Z5-1-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-1-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-2-a
sector 12
Would be wrong if: old_read did not copy sector 12 into the output buffer.

Z5-2-b
sector 12
Would be wrong if: new_read did not copy sector 12 into the buffer.

Z5-2-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-2-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-3-a
none
Would be wrong if: old_read copied any sector into the output buffer.

Z5-3-b
none
Would be wrong if: new_read copied any sector into the buffer.

Z5-3-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-3-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-4-a
none
Would be wrong if: old_read copied any sector into the output buffer.

Z5-4-b
none
Would be wrong if: new_read copied any sector into the buffer.

Z5-4-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-4-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-5-a
none
Would be wrong if: old_read copied any sector into the output buffer.

Z5-5-b
none
Would be wrong if: new_read copied any sector into the buffer.

Z5-5-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-5-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-6-a
10, 11, 12
Would be wrong if: old_read did not copy sector 10, 11, or 12 into the output buffer.

Z5-6-b
10, 11, 12
Would be wrong if: new_read did not copy sector 10, 11, or 12 into the buffer.

Z5-6-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-6-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-7-a
none
Would be wrong if: old_read copied any sector into the output buffer.

Z5-7-b
none
Would be wrong if: new_read copied any sector into the buffer.

Z5-7-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-7-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z5-8-a
10, 1009
Would be wrong if: old_read did not copy sector 10 or 1009 into the output buffer.

Z5-8-b
10, 1009
Would be wrong if: new_read did not copy sector 10 or 1009 into the buffer.

Z5-8-c
yes
Would be wrong if: the output buffers from old_read and new_read differed in any byte.

Z5-8-d
Would be wrong if: the output buffers from old_read and new_read differed in any byte.
Would be wrong if: the output buffers were identical but the answer for part d claimed they differed.

Z1-A-1
The documented precondition is that the previous journal record has checkpoint number 2 and journal sequence number 2.
Would be wrong if: the doc text did not mention checkpoint number 2 and journal sequence number 2.

Z1-A-2
The generator enforces that a writable session is open.
Would be wrong if: the generator checked for anything other than a writable session being open.

Z1-A-3
The generator's guard is narrower than the documented precondition because the doc requires checking the previous record's checkpoint and journal sequence numbers, but the generator only checks session open.
Would be wrong if: the doc text did not require checking previous record's checkpoint and journal sequence numbers or the generator checked those numbers.

Z1-A-4
Would be wrong if: the doc text did not require checking previous record's checkpoint and journal sequence numbers or the generator checked those numbers.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-B-1
The documented precondition is that the previous version passed as parameter is a file-carrying version.
Would be wrong if: the doc text did not mention TransactionOutput type or file-carrying version.

Z1-B-2
The generator enforces that the current in-memory version carries a file.
Would be wrong if: the generator checked for something else like session open.

Z1-B-3
The generator's guard exactly matches the documented precondition because the doc specifies the parameter must be a file-carrying version, and the generator checks the current version has a file.
Would be wrong if: the doc text didn't specify parameter type or generator didn't check for file.

Z1-B-4
Would be wrong if: the doc text didn't specify parameter type or generator didn't check for file.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-C-1
The documented precondition is that the tree table has zero entries.
Would be wrong if: the doc text didn't mention tree table zero entries.

Z1-C-2
The generator enforces that the current in-memory version has zero entries in the tree table.
Would be wrong if: the generator checked for file-carrying version.

Z1-C-3
The generator's guard exactly matches the documented precondition because the doc states the function is used when tree table has zero entries, and the generator checks for that.
Would be wrong if: the doc text didn't state tree table zero entries or generator checked something else.

Z1-C-4
Would be wrong if: the doc text didn't state tree table zero entries or generator checked something else.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-D-1
The doc text states no precondition on caller session state.
Would be wrong if: the doc text mentioned any session state requirement.

Z1-D-2
The generator enforces no precondition check before calling mount_writable.
Would be wrong if: the generator checked for session open or closed.

Z1-D-3
The generator's guard exactly matches the documented precondition because both state no precondition.
Would be wrong if: the doc text had a precondition or generator checked for one.

Z1-D-4
Would be wrong if: the doc text had a precondition or generator checked for one.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-E-1
The doc text states no precondition.
Would be wrong if: the doc text required target to be in candidate set.

Z1-E-2
The generator enforces that there is at least one readable root in the ring.
Would be wrong if: the generator didn't check for readable roots.

Z1-E-3
The generator's guard is narrower than the documented precondition because the doc allows any target (errors returned), but the generator requires at least one readable root in the ring.
Would be wrong if: the doc text required target to be in candidate set or generator didn't check for readable roots.

Z1-E-4
Would be wrong if: the doc text required target to be in candidate set or generator didn't check for readable roots.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-F-1
The doc text states no precondition on the target value.
Would be wrong if: the doc text specified target must be >= current floor.

Z1-F-2
The generator enforces that a writable session is open, the current version carries a file, and the target is at or above the current floor.
Would be wrong if: the generator didn't check for session or file or target >= floor.

Z1-F-3
The generator's guard is narrower than the documented precondition because the doc allows any target value (errors returned for exceeding ceiling), but the generator additionally restricts targets to be at or above the current floor.
Would be wrong if: the doc text specified target >= floor or generator didn't check those.

Z1-F-4
Would be wrong if: the doc text specified target >= floor or generator didn't check those.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.

Z1-G-1
The doc text states no precondition on caller state.
Would be wrong if: the doc text mentioned any precondition.

Z1-G-2
The generator enforces no precondition check before calling recover.
Would be wrong if: the generator checked for session state.

Z1-G-3
The generator's guard exactly matches the documented precondition because both state no precondition.
Would be wrong if: the doc text had a precondition or generator checked for one.

Z1-G-4
Would be wrong if: the doc text had a precondition or generator checked for one.
Would be wrong if: the answer for part 4 was not a concrete observation proving the specific answer wrong.
