# Fast Counter

This is a simple sharded concurrent counter which can be used in higher contention scenarios for example for a counter in a HashMap. 

This approach appears to scale well to a higher number of cores as shown here compared to a single atomic number which is being updated:

```
atomic_counter/1        time:   [1.5215 ms 1.5311 ms 1.5414 ms]
                        thrpt:  [680.28 Melem/s 684.86 Melem/s 689.19 Melem/s]

atomic_counter/2        time:   [8.1665 ms 8.3089 ms 8.4518 ms]
                        thrpt:  [124.07 Melem/s 126.20 Melem/s 128.40 Melem/s]

atomic_counter/4        time:   [10.192 ms 10.250 ms 10.303 ms]
                        thrpt:  [101.77 Melem/s 102.30 Melem/s 102.88 Melem/s]

atomic_counter/8        time:   [10.021 ms 10.131 ms 10.234 ms]
                        thrpt:  [102.46 Melem/s 103.50 Melem/s 104.64 Melem/s]

atomic_counter/16       time:   [12.362 ms 12.393 ms 12.425 ms]
                        thrpt:  [84.390 Melem/s 84.610 Melem/s 84.823 Melem/s]

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

fast_counter_stable/1   time:   [1.5678 ms 1.5795 ms 1.5913 ms]
                        thrpt:  [658.92 Melem/s 663.88 Melem/s 668.81 Melem/s]

fast_counter_stable/2   time:   [820.95 us 824.81 us 828.56 us]
                        thrpt:  [1.2655 Gelem/s 1.2713 Gelem/s 1.2773 Gelem/s]

fast_counter_stable/4   time:   [429.10 us 430.62 us 432.17 us]
                        thrpt:  [2.4263 Gelem/s 2.4350 Gelem/s 2.4437 Gelem/s]

fast_counter_stable/8   time:   [240.59 us 242.92 us 245.44 us]
                        thrpt:  [4.2723 Gelem/s 4.3165 Gelem/s 4.3583 Gelem/s]

fast_counter_stable/16  time:   [206.01 us 210.24 us 214.74 us]
                        thrpt:  [4.8829 Gelem/s 4.9875 Gelem/s 5.0900 Gelem/s]
```

Big shoutout to @jimvdl who put the core starting point for this together
