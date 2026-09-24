G1.max reject outright because the device capacity divided by four is a hard constraint enforced at filesystem creation and any user-supplied maximum exceeding this bound is invalid and must be rejected to maintain system integrity. This would be refuted by a user-supplied G1.max value exceeding the device capacity divided by four being accepted without rejection.

G1.min reject outright because the minimum ring length must satisfy FACT 4's invariant requiring it to be at least F times the true worst-case per-transaction usage (36 KiB) and any user-supplied minimum below this is invalid and must be rejected to prevent invariant violation. This would be refuted by a user-supplied G1.min value below 36 KiB being accepted without rejection.

G1.same-or-different no such fact exists because FACT 3 explicitly allows different handling rules for fields within the same decision item and no specific fact about ring bounds mandates identical treatment for maximum and minimum settings. This would be refuted by a specific fact stating that ring bounds must use the same handling rule.

G2.a no problem because FACT 4's invariant (ring size >= F times worst-case usage) holds exactly when ring size equals 36 KiB (36 KiB >= 3 * 12 KiB). This would be refuted by ring size of 36 KiB being less than F times worst-case per-transaction usage.

G2.b no problem because field X (in-flight record limit) computes to exactly 3 records (36 KiB / 4096 bytes per record / 3 = 3) which is a valid whole number. This would be refuted by field X not being an integer when ring is 36 KiB.

G2.c yes because 36 KiB (36864 bytes) divides evenly by 4096 bytes per record to produce exactly 9 records with no remainder. This would be refuted by 36 KiB not dividing evenly into 4096-byte records.

G2.d no problem because the T_dirty clamping rule (effective T_dirty = min(stored T_dirty, ring length divided by F)) produces a valid effective value of 12 KiB when ring is 36 KiB (36 KiB / 3 = 12 KiB) and no inconsistency arises. This would be refuted by effective T_dirty exceeding ring length divided by F when ring is 36 KiB.

G3.Intersection breaks down in both edge cases because the effective range becomes empty (e.g., user min > default max results in [user min, default max] which is invalid). This would be refuted by effective range being non-empty when user min > default max.

G3.UserWins breaks down in both edge cases because the user-supplied bound replaces the default outright without checking the opposite side, leading to invalid ranges (e.g., user max < default min results in [default min, user max] which is invalid). This would be refuted by effective range being valid when user max < default min.

G3.DefaultWins handles both edge cases correctly by rejecting user-supplied bounds that would widen the default range beyond its original constraints (e.g., user min > default max or user max < default min triggers rejection). This would be refuted by user-supplied min larger than default max being accepted instead of rejected.

G4.W no because field W stores the current ring length and cannot independently hold a distinct user-supplied minimum ring length value as it is defined solely as the ring's current size. This would be refuted by field W being capable of storing a distinct user-supplied minimum ring length value.

G4.X no because field X is computed purely as (ring length in bytes) divided by 4096 divided by F and cannot store any value not derivable from field W and F. This would be refuted by field X being capable of storing a distinct user-supplied minimum ring length value.

G4.Y no because field Y is computed purely as field X multiplied by 4096 and cannot store any value not derivable from field W and F. This would be refuted by field Y being capable of storing a distinct user-supplied minimum ring length value.

G4.Z no because field Z is a fixed compile-time constant equal to 3 and cannot store any user-configurable value. This would be refuted by field Z being capable of storing a distinct user-supplied minimum ring length value.

G5 no because none of the existing fields can store a distinct user-supplied minimum ring length value, so the minimal new field needed is a dedicated user-configured minimum ring length field. This would be refuted by existing on-disk fields collectively being capable of recording a user-supplied minimum ring length value distinct from the ring's current length.
