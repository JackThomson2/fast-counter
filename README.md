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

fast_counter/2   time:   [299.28 us 300.40 us 301.52 us]
                        thrpt:  [108.68 Melem/s 109.08 Melem/s 109.49 Melem/s]

fast_counter/4   time:   [276.78 us 278.90 us 281.02 us]
                        thrpt:  [116.61 Melem/s 117.49 Melem/s 118.39 Melem/s]

fast_counter/8   time:   [194.94 us 199.44 us 204.17 us]
                        thrpt:  [160.49 Melem/s 164.30 Melem/s 168.10 Melem/s]

fast_counter/16  time:   [152.30 us 155.98 us 159.92 us]
                        thrpt:  [204.91 Melem/s 210.08 Melem/s 215.16 Melem/s]
```

Big shoutout to @jimvdl who put the core starting point for this together
