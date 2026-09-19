Z5-1-a: sector 10
Would be wrong if: old_read did not copy sector 10 when the map contains sector 10 and the range includes it, or copied any sector outside the map's entries in the range [10, 13)

Z5-1-b: sector 10
Would be wrong if: new_read did not copy sector 10 when the map contains sector 10 and the range includes it, or copied any sector outside the map's entries in the range [10, 13)

Z5-1-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-1-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-2-a: sector 12
Would be wrong if: old_read did not copy sector 12 when the map contains sector 12 and the range includes it, or copied any sector outside the map's entries in the range [10, 13)

Z5-2-b: sector 12
Would be wrong if: new_read did not copy sector 12 when the map contains sector 12 and the range includes it, or copied any sector outside the map's entries in the range [10, 13)

Z5-2-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-2-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-3-a: none
Would be wrong if: old_read copied any sector in the range [10, 13) when the map contains only sector 13

Z5-3-b: none
Would be wrong if: new_read copied any sector in the range [10, 13) when the map contains only sector 13

Z5-3-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-3-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-4-a: none
Would be wrong if: old_read copied any sector in the range [10, 13) when the map contains only sector 9

Z5-4-b: none
Would be wrong if: new_read copied any sector in the range [10, 13) when the map contains only sector 9

Z5-4-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-4-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-5-a: none
Would be wrong if: old_read copied any sector in the range [10, 13) when the map is empty

Z5-5-b: none
Would be wrong if: new_read copied any sector in the range [10, 13) when the map is empty

Z5-5-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-5-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-6-a: 10 11 12
Would be wrong if: old_read did not copy sectors 10 11 12 when the map contains them and the range includes them, or copied any sector outside the map's entries in the range [10, 13)

Z5-6-b: 10 11 12
Would be wrong if: new_read did not copy sectors 10 11 12 when the map contains them and the range includes them, or copied any sector outside the map's entries in the range [10, 13)

Z5-6-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 13)

Z5-6-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-7-a: none
Would be wrong if: old_read copied any sector in the range [10, 10) when the map contains sector 10

Z5-7-b: none
Would be wrong if: new_read copied any sector in the range [10, 10) when the map contains sector 10

Z5-7-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 10)

Z5-7-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z5-8-a: 10 1009
Would be wrong if: old_read did not copy sectors 10 1009 when the map contains them and the range includes them, or copied any sector outside the map's entries in the range [10, 1010)

Z5-8-b: 10 1009
Would be wrong if: new_read did not copy sectors 10 1009 when the map contains them and the range includes them, or copied any sector outside the map's entries in the range [10, 1010)

Z5-8-c: yes
Would be wrong if: the output buffers from old_read and new_read had different byte values for any position in the range [10, 1010)

Z5-8-d: the output buffers were not byte-for-byte identical
Would be wrong if: the output buffers were identical when they should not have been

Z1-A-1: The documented precondition is that the current in-memory version has checkpoint number 2 and journal sequence number 2
Would be wrong if: the Doc text did not include the phrase "only accepts being called on the version whose checkpoint number is 2 and whose journal sequence number is 2"

Z1-A-2: The enforced precondition is that a writable session is open
Would be wrong if: the Generator text checked for checkpoint number 2 or journal sequence number 2

Z1-A-3: The generator's guard is narrower than the documented precondition because the Doc text requires specific checkpoint and journal sequence numbers (stated as "only accepts being called on the version whose checkpoint number is 2 and whose journal sequence number is 2"), whereas the Generator text only checks for a writable session being open
Would be wrong if: the Doc text did not state that the function requires checkpoint 2 and journal sequence 2 or the Generator text did not check only for a writable session

Z1-A-4: if the Doc text did not state that the function requires the version to have checkpoint number 2 and journal sequence number 2, or if the Generator text checked for those numbers
Would be wrong if: the Doc text did state that the function requires checkpoint 2 and journal sequence 2 and the Generator text did check for them

Z1-B-1: The documented precondition is that the previous version carries a file
Would be wrong if: the Doc text did not state that the parameter is a TransactionOutput produced by a publish that carries a file

Z1-B-2: The enforced precondition is that the current in-memory version carries a file
Would be wrong if: the Generator text did not check for a current version with a file

Z1-B-3: The generator's guard exactly matches the documented precondition because both require the version to carry a file (Doc text states parameter is TransactionOutput from a file publish, Generator text checks current version has a file)
Would be wrong if: the Doc text did not reference TransactionOutput or the Generator text did not check for a file in the current version

Z1-B-4: if the Doc text did not reference TransactionOutput or the Generator text did not check for a file in the current version
Would be wrong if: the Doc text referenced TransactionOutput and the Generator text checked for a file in the current version

Z1-C-1: The documented precondition is that the current version's tree table has zero entries
Would be wrong if: the Doc text did not state that the function is used for pools with zero tree entries or first mount warm-up publishes

Z1-C-2: The enforced precondition is that the current version's tree table has zero entries
Would be wrong if: the Generator text did not check for zero tree entries

Z1-C-3: The generator's guard exactly matches the documented precondition because both require zero tree entries (Doc text implies it via usage examples, Generator text checks tree table has zero entries)
Would be wrong if: the Doc text did not reference zero tree entries or the Generator text did not check for zero tree entries

Z1-C-4: if the Doc text did not reference zero tree entries or the Generator text did not check for zero tree entries
Would be wrong if: the Doc text referenced zero tree entries and the Generator text checked for zero tree entries

Z1-D-1: No documented precondition about session state
Would be wrong if: the Doc text stated a precondition on session state

Z1-D-2: No enforced precondition
Would be wrong if: the Generator text checked session state before calling mount_writable

Z1-D-3: The generator's guard exactly matches the documented precondition because both state no session state precondition (Doc text says nothing, Generator text sets session to closed unconditionally with no checks)
Would be wrong if: the Doc text stated a session state precondition or the Generator text checked session state

Z1-D-4: if the Doc text stated a session state precondition or the Generator text checked session state
Would be wrong if: the Doc text stated no session state precondition and the Generator text checked session state

Z1-E-1: The documented precondition is that the target root is in the rollback candidate set
Would be wrong if: the Doc text did not state "the caller picks an old root, called R_old, from the rollback candidate set"

Z1-E-2: The enforced precondition is that there is at least one readable root in the ring
Would be wrong if: the Generator text did not check for at least one readable root in the ring

Z1-E-3: The generator's guard is different from the documented precondition because the Doc text requires the target to be in the candidate set, while the Generator text only checks for readable roots in the ring and does not validate the target against the candidate set
Would be wrong if: the Doc text did not mention the candidate set or the Generator text checked for the target being in the candidate set

Z1-E-4: if the Doc text did not mention the candidate set or the Generator text checked for the target being in the candidate set
Would be wrong if: the Doc text mentioned the candidate set and the Generator text did not check for it

Z1-F-1: The documented precondition is that the new floor does not exceed the ceiling
Would be wrong if: the Doc text did not list "new floor exceeds a ceiling" as an error condition

Z1-F-2: The enforced precondition is that a writable session is open and the current version carries a file
Would be wrong if: the Generator text did not check for a writable session or file presence

Z1-F-3: The generator's guard is wider than the documented precondition because the Doc text requires the new floor to be within the ceiling, while the Generator text allows targets exceeding the ceiling but checks only for session and file presence
Would be wrong if: the Doc text did not list ceiling as an error condition or the Generator text checked for the ceiling constraint

Z1-F-4: if the Doc text did not list ceiling as an error condition or the Generator text checked for the ceiling constraint
Would be wrong if: the Doc text listed ceiling as an error condition and the Generator text did not check for it

Z1-G-1: No documented precondition
Would be wrong if: the Doc text stated any precondition on caller state

Z1-G-2: No enforced precondition
Would be wrong if: the Generator text performed any check before calling recover

Z1-G-3: The generator's guard exactly matches the documented precondition because both state no precondition (Doc text has no errors section or precondition, Generator text calls unconditionally)
Would be wrong if: the Doc text stated a precondition or the Generator text checked any state

Z1-G-4: if the Doc text stated a precondition or the Generator text performed any check
Would be wrong if: the Doc text stated no precondition and the Generator text performed no checks
