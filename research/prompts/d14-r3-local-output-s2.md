1. none. A clause requiring directories to be interleaved at creation would be needed for the benefit of freeing whole segments when deleting a directory to count, as stated in F8.

2. A dedicated user data segment pool reserved exclusively for user data overflow and not used for commit-produced blocks. With this source, a directory could still stay within 3 segments.

3. yes. The home is fixed at creation and never changes, so it cannot be re-evaluated at every allocation, similar to the locality identifier rejection in F3 which also could not be re-evaluated due to being fixed at creation.

4. narrowing. Arm B demands that allocations first try slots within the directory home even if not the lowest free slot, whereas the baseline always selects the lowest free slot on the device without such restriction, reducing flexibility and permitting no new choices.

5. evidence that the threshold measured the wrong quantity. Copy-on-write rewrites inherently scatter slots for every policy, so the contiguous stretches metric may not reflect actual performance improvement as scattering is unavoidable regardless of the policy.

6. F2. The proposed clause states that if a hint is added later it must inherit once at creation and never change, but F2 requires the rule to be re-evaluated at every allocation, which is impossible for a fixed rule.
