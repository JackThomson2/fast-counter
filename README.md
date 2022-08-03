# Fast Counter

This is a simple shareded concurrent counter which can be used in higher contention scenearios for example for a counter in a HashMap. 

This approach appears to scale well to a higher number of cores as shown here compared to a single atomic number which is being updated:

```
atomic_counter/2        time:   [290.27 us 293.65 us 297.26 us]
                        thrpt:  [110.23 Melem/s 111.59 Melem/s 112.89 Melem/s]

atomic_counter/4        time:   [320.62 us 323.01 us 325.27 us]
                        thrpt:  [100.74 Melem/s 101.45 Melem/s 102.20 Melem/s]

atomic_counter/8        time:   [343.33 us 344.14 us 344.98 us]
                        thrpt:  [94.985 Melem/s 95.217 Melem/s 95.442 Melem/s]

atomic_counter/16       time:   [410.49 us 411.71 us 412.99 us]
                        thrpt:  [79.344 Melem/s 79.590 Melem/s 79.827 Melem/s]

------------------------------------------------------------------------------

fast_counter_nightly/2  time:   [314.05 us 315.63 us 317.16 us]
                        thrpt:  [103.32 Melem/s 103.82 Melem/s 104.34 Melem/s]

fast_counter_nightly/4  time:   [292.82 us 294.93 us 296.72 us]
                        thrpt:  [110.44 Melem/s 111.10 Melem/s 111.91 Melem/s]

fast_counter_nightly/8  time:   [209.61 us 215.30 us 221.28 us]
                        thrpt:  [148.08 Melem/s 152.20 Melem/s 156.33 Melem/s]

fast_counter_nightly/16 time:   [157.28 us 160.06 us 163.00 us]
                        thrpt:  [201.04 Melem/s 204.72 Melem/s 208.34 Melem/s]

------------------------------------------------------------------------------

fast_counter_stable/2   time:   [400.89 us 407.77 us 413.33 us]
                        thrpt:  [79.277 Melem/s 80.360 Melem/s 81.739 Melem/s]

fast_counter_stable/4   time:   [369.10 us 372.90 us 376.90 us]
                        thrpt:  [86.942 Melem/s 87.873 Melem/s 88.778 Melem/s]

fast_counter_stable/8   time:   [247.36 us 253.10 us 258.51 us]
                        thrpt:  [126.76 Melem/s 129.47 Melem/s 132.47 Melem/s]

fast_counter_stable/16  time:   [162.17 us 166.01 us 170.13 us]
                        thrpt:  [192.60 Melem/s 197.39 Melem/s 202.06 Melem/s]
```

Big shoutout to @jimvdl who put the core starting point for this together