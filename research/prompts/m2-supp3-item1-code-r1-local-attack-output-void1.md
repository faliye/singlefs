Z5-1a sector 10
Would be wrong if: the written sector numbers include 10 and old_read does not copy sector 10 into the output buffer

Z5-1b sector 10
Would be wrong if: the written sector numbers include 10 and new_read does not copy sector 10 into the buffer

Z5-1c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-1d Would be wrong if: the written sector numbers include 10 and old_read does not copy sector 10, or new_read does not copy sector 10, or the buffers differ
Would be wrong if: the statement in Z5-1d is not a concrete observation that would prove the answers wrong

Z5-2a sector 12
Would be wrong if: the written sector numbers include 12 and old_read does not copy sector 12 into the output buffer

Z5-2b sector 12
Would be wrong if: the written sector numbers include 12 and new_read does not copy sector 12 into the buffer

Z5-2c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-2d Would be wrong if: the written sector numbers include 12 and old_read does not copy sector 12, or new_read does not copy sector 12, or the buffers differ
Would be wrong if: the statement in Z5-2d is not a concrete observation that would prove the answers wrong

Z5-3a none
Would be wrong if: the written sector numbers include 13 and old_read copies any sector in the range [10,12)

Z5-3b none
Would be wrong if: the written sector numbers include 13 and new_read copies any sector in the range [10,12)

Z5-3c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-3d Would be wrong if: the written sector numbers include 13 and old_read copies any sector in the range [10,12), or new_read copies any sector in the range [10,12), or the buffers differ
Would be wrong if: the statement in Z5-3d is not a concrete observation that would prove the answers wrong

Z5-4a none
Would be wrong if: the written sector numbers include 9 and old_read copies sector 9 into the output buffer

Z5-4b none
Would be wrong if: the written sector numbers include 9 and new_read copies sector 9 into the buffer

Z5-4c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-4d Would be wrong if: the written sector numbers include 9 and old_read copies sector 9, or new_read copies sector 9, or the buffers differ
Would be wrong if: the statement in Z5-4d is not a concrete observation that would prove the answers wrong

Z5-5a none
Would be wrong if: the written sector numbers are empty and old_read copies any sector

Z5-5b none
Would be wrong if: the written sector numbers are empty and new_read copies any sector

Z5-5c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-5d Would be wrong if: the written sector numbers are empty and old_read copies any sector, or new_read copies any sector, or the buffers differ
Would be wrong if: the statement in Z5-5d is not a concrete observation that would prove the answers wrong

Z5-6a 10 11 12
Would be wrong if: the written sector numbers include 10, 11, 12 and old_read does not copy all three sectors

Z5-6b 10 11 12
Would be wrong if: the written sector numbers include 10, 11, 12 and new_read does not copy all three sectors

Z5-6c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-6d Would be wrong if: the written sector numbers include 10, 11, 12 and old_read does not copy all three sectors, or new_read does not copy all three sectors, or the buffers differ
Would be wrong if: the statement in Z5-6d is not a concrete observation that would prove the answers wrong

Z5-7a none
Would be wrong if: length_bytes is 0 and old_read copies any sector

Z5-7b none
Would be wrong if: length_bytes is 0 and new_read copies any sector

Z5-7c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-7d Would be wrong if: length_bytes is 0 and old_read copies any sector, or new_read copies any sector, or the buffers differ
Would be wrong if: the statement in Z5-7d is not a concrete observation that would prove the answers wrong

Z5-8a 10 1009
Would be wrong if: the written sector numbers include 10 and 1009 and old_read does not copy both sectors

Z5-8b 10 1009
Would be wrong if: the written sector numbers include 10 and 1009 and new_read does not copy both sectors

Z5-8c yes
Would be wrong if: the output buffers from old_read and new_read are not byte-for-byte identical

Z5-8d Would be wrong if: the written sector numbers include 10 and 1009 and old_read does not copy both sectors, or new_read does not copy both sectors, or the buffers differ
Would be wrong if: the statement in Z5-8d is not a concrete observation that would prove the answers wrong

Z1-A1 the documented precondition is that the previous journal record must have checkpoint number 2 and journal sequence number 2
Would be wrong if: the doc text does not state that the previous journal record's checkpoint number and journal sequence number must be 2

Z1-A2 the generator enforces that a writable session is currently open in memory
Would be wrong if: the generator text does not state it checks only for a writable session being open

Z1-A3 the generator's guard is narrower than the documented precondition because the generator only checks for a writable session while the doc requires specific checkpoint and journal sequence numbers; the doc states "only accepts being called on the version whose checkpoint number is 2 and whose journal sequence number is 2" while the generator text says "checks exactly one thing: that a writable session is currently open"
Would be wrong if: the doc text does not require checkpoint 2 and journal sequence 2, or the generator checks more than just a writable session, or the specific quoted words do not match

Z1-A4 Would be wrong if: the generator's guard is not narrower than the documented precondition, or the specific words in the doc and generator texts do not support the difference statement
Would be wrong if: the statement in Z1-A4 is not a concrete observation that would prove the answers wrong

Z1-B1 the documented precondition is that the previous version must be a file-carrying version, as the function releases previous version's storage placements and the parameter is typed as TransactionOutput
Would be wrong if: the doc text does not state the previous version must be a file-carrying version

Z1-B2 the generator enforces that a writable session is open and the current in-memory version carries a file
Would be wrong if: the generator text does not state it checks both a writable session and a file-carrying version

Z1-B3 the generator's guard is narrower than the documented precondition because the doc does not explicitly require a file-carrying version but the generator checks for it; the doc says "the function's parameter for the previous version is typed as a TransactionOutput, which is the type produced by a publish that carries a file" while the generator text says "checks two things: first, that a writable session is open... second, that the current in-memory version is the variant that carries an existing file"
Would be wrong if: the doc text does not imply the previous version must carry a file, or the generator does not check for a file-carrying version, or the specific quoted words do not match

Z1-B4 Would be wrong if: the generator's guard is not narrower than the documented precondition, or the specific words in the doc and generator texts do not support the difference statement
Would be wrong if: the statement in Z1-B4 is not a concrete observation that would prove the answers wrong

Z1-C1 the documented precondition is that the current version must have zero entries in the tree table, as the function is used for zero-unit publishes and the documentation states "when the tree table has zero entries, a publish writes zero units"
Would be wrong if: the doc text does not state the tree table must have zero entries

Z1-C2 the generator enforces that a writable session is open and the current in-memory version has zero entries in the tree table
Would be wrong if: the generator text does not state it checks both a writable session and zero tree table entries

Z1-C3 the generator's guard is narrower than the documented precondition because the doc does not explicitly state the zero-tree-table precondition but the generator checks for it; the doc says "one settled decision item is cited by name as the justification for when this function is meant to be used: that when the tree table has zero entries, a publish writes zero units" while the generator text says "checks two things: first, that a writable session is open... second, that the current in-memory version is the variant whose tree table has zero entries"
Would be wrong if: the doc text does not imply zero tree table entries, or the generator does not check for zero tree table entries, or the specific quoted words do not match

Z1-C4 Would be wrong if: the generator's guard is not narrower than the documented precondition, or the specific words in the doc and generator texts do not support the difference statement
Would be wrong if: the statement in Z1-C4 is not a concrete observation that would prove the answers wrong

Z1-D1 the documented precondition states no precondition on caller state
Would be wrong if: the doc text states any precondition on caller state

Z1-D2 the generator enforces no precondition check before calling mount_writable
Would be wrong if: the generator text states it checks any precondition precondition before calling mount_writable

Z1-D3 the generator's guard exactly matches the documented precondition because the doc states no precondition and the generator performs no checks
Would be wrong if: the doc text states any precondition, or the generator checks any condition before calling mount_writable

Z1-D4 Would be wrong if: the doc text states a precondition or the generator performs any checks before calling mount_writable
Would be wrong if: the statement in Z1-D4 is not a concrete observation that would prove the answers wrong

Z1-E1 the documented precondition states no explicit precondition on caller state, only listing error conditions related to the target root
Would be wrong if: the doc text states any precondition on caller state

Z1-E2 the generator enforces that there is at least one readable root in the ring, but does not check if the target root is in the ring, candidate set, or has a readable journal record
Would be wrong if: the generator text does not state it checks for at least one readable root in the ring but leaves other checks to mount_rollback

Z1-E3 the generator's guard is narrower than the documented precondition because the doc lists error conditions for invalid targets but does not require caller checks, while the generator checks only for readable roots in the ring but not the specific target validity; the doc states "regarding errors: the target root is not in the root ring; the target root is not in the rollback candidate set..." while the generator text says "if there is not a single readable root in the ring, the step is marked not applicable... the generator calls mount_rollback with that target without first checking, itself, whether the target is in the ring, whether it is in the candidate set, or whether its own journal record can be read"
Would be wrong if: the doc text does not list target-related errors, or the generator checks more than just readable roots in the ring, or the specific quoted words do not match

Z1-E4 Would be wrong if: the generator's guard is not narrower than the documented precondition, or the specific words in the doc and generator texts do not support the difference statement
Would be wrong if: the statement in Z1-E4 is not a concrete observation that would prove the answers wrong

Z1-F1 the documented precondition states the new floor must not exceed a ceiling bounded by the newest valid root on each device
Would be wrong if: the doc text does not state the new floor must not exceed a ceiling bounded by the newest valid root

Z1-F2 the generator enforces that a writable session is open, the current version carries a file, and the target is at or above the current floor (computed as current floor + random modulo (checkpoint - floor + 3))
Would be wrong if: the generator text does not state it checks for writable session, file-carrying version, and target at or above current floor

Z1-F3 the generator's guard is wider than the documented precondition because the doc states the new floor must not exceed a ceiling, but the generator allows targets up to current checkpoint + 2 which may exceed the ceiling; the doc says "the new floor exceeds a ceiling (the ceiling being, in general terms, bounded by the newest valid root on each device, which is itself at most the current checkpoint number)" while the generator text says "the generator's own precondition is that the target is only ever taken at or above the current floor, never below it... the formula's maximum output (current checkpoint number plus 2) can exceed the ceiling described in the entry point's own error list"
Would be wrong if: the doc text does not mention the ceiling constraint, or the generator does not compute a target that may exceed the ceiling, or the specific quoted words do not match

Z1-F4 Would be wrong if: the generator's guard is not wider than the documented precondition, or the specific words in the doc and generator texts do not support the difference statement
Would be wrong if: the statement in Z1-F4 is not a concrete observation that would prove the answers wrong

Z1-G1 the documented precondition states no precondition on caller state
Would be wrong if: the doc text states any precondition on caller state

Z1-G2 the generator enforces no precondition check before calling recover
Would be wrong if: the generator text states it checks any precondition before calling recover

Z1-G3 the generator's guard exactly matches the documented precondition because the doc states no precondition and the generator performs no checks
Would be wrong if: the doc text states a precondition, or the generator checks any condition before calling recover

Z1-G4 Would be wrong if: the doc text states a precondition or the generator performs any checks before calling recover
Would be wrong if: the statement in Z1-G4 is not a concrete observation that would prove the answers wrong