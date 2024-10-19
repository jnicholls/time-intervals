# Time Intervals

Hi Paul,

I took about 1.5 hours this morning to implement a full solution to the problem you presented to me yesterday.

If you recall I mentioned that I was considering a binary search tree or some sort of B+-tree to create a logarithmic data structure for the search. I was also considering cases we didn't discuss such as adding/removing time intervals from the data structure, even though those cases were not explicitly noted.

However, I thought about it after the fact and since mutating the intervals was off the table and we just wanted to optimize for search time, I would just do a binary search on the sorted intervals in my Vec for a logarithmic search time (O(log n)). That's what I've implemented here.

Additionally I did implement my optimization of the search space by combining overlapping (as well as adjacent, since the time resolution I'm using is whole integers) time intervals.

I've stuck with Rust for the implementation. You can find the code in the `src/lib.rs` file. At the bottom are my unit tests, and below is the output of executing the tests via `cargo test`.

If you have any questions or feedback, please let me know.

Thanks,
Jarred

```
❯ cargo test
   Compiling time-intervals v0.1.0 (/Users/jarrednicholls/code/time-intervals)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running unittests src/lib.rs (target/debug/deps/time_intervals-1a90585398a5a8cd)

running 7 tests
test tests::already_sorted ... ok
test tests::gaps ... ok
test tests::overshadowed ... ok
test tests::sparse_overlapping ... ok
test tests::time_interval ... ok
test tests::out_of_order ... ok
test tests::overlapping ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests time_intervals

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
