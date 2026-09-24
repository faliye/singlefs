G1.max reject outright because the device capacity divided by 4 is a hard constraint enforced at creation time per FACT 1, and out-of-range values must be rejected to prevent violating this constraint. This would be refuted by: a user-supplied G1.max value exceeding the device capacity divided by 4 being accepted without rejection.

G1.min reject outright because the minimum ring length must satisfy FACT 4's invariant requiring ring size >= F times worst-case per-transaction usage (36 KiB per FACT 6), and out-of-range values must be rejected to prevent violating this invariant. This would be refuted by: a user-supplied G1.min value below 36 KiB being accepted without rejection.

G1.same-or-different no such fact exists because FACT 3 explicitly allows separate fields in the same decision item to follow different rules, and there is no fact indicating the ring bounds must share a rule. This would be refuted by: the design document stating that ring bounds must always follow the same rule, contradicting FACT 3.

G2.a no problem because ring size 36 KiB equals F times worst-case per-transaction usage (3 * 12 KiB = 36 KiB), satisfying the invariant exactly. This would be refuted by: ring size 36 KiB being less than 3 times 12 KiB.

G2.b no problem because field X is computed as (36 KiB / 4096) / 3 = 9 / 3 = 3 records, which is a valid integer. This would be refuted by: field X having a non-integer value when ring size is 36 KiB.

G2.c yes because 36 KiB (36864 bytes) is exactly 9 * 4096 bytes, so it divides evenly into whole records. This would be refuted by: 36 KiB not being a multiple of 4096 bytes.

G2.d no problem because the clamping rule (effective T_dirty = min(stored T_dirty, ring length / F)) produces a valid effective value of 12 KiB when ring length is 36 KiB. This would be refuted by: ring length divided by F being invalid for effective T_dirty calculation.

G3.Intersection breaks down in both edge cases because it produces an empty range (e.g., user min > default max results in [user min, default max] where lower bound exceeds upper bound). This would be refuted by: G3.Intersection producing a non-empty valid range in either edge case.

G3.UserWins breaks down in both edge cases because it replaces default bounds outright without checking consistency, creating invalid ranges (e.g., user min > default max results in [user min, default max] with lower bound > upper bound). This would be refuted by: G3.UserWins producing a non-empty valid range in either edge case.

G3.DefaultWins works for both edge cases because it rejects user-supplied bounds that widen would the default range, preserving a valid range (e.g., user min > default max is rejected, keeping default range). This would be refuted by: G3.DefaultWins accepting a user-supplied bound that widens the default range.

G4.W no because field W stores the current ring length itself and cannot hold a distinct minimum value. This would be refuted by: field W storing a value unrelated to the current ring length.

G4.X no because field X is derived solely from W and F and cannot independently store a distinct minimum. This would be refuted by: field X storing a value not computable from W and F.

G4.Y no because field Y is derived solely from W and F and cannot independently store a distinct minimum. This would be refuted by: field Y storing a value not computable from W and F.

G4.Z no because field Z is a compile-time constant and cannot store any user-configurable value. This would be refuted by: field Z storing a user-configurable minimum ring length.

G5 no because none of the existing fields can store a distinct user-supplied minimum ring length; the minimal new field needed is a dedicated field for the user-supplied minimum ring length. This would be refuted by: the existing on-disk configuration area already containing a field capable of storing a distinct user-supplied minimum ring length.
