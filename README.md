# Fast Counter

This is a simple shareded concurrent counter which can be used in higher contention scenearios for example for a counter in a HashMap. 

This approach appears to scale well to a higher number of cores as shown here compared to a single atomic number which is being updated:

```
atomic_counter/2        time:   [282.18 us 285.66 us 289.18 us]
                        thrpt:  [113.31 Melem/s 114.71 Melem/s 116.12 Melem/s]

atomic_counter/4        time:   [324.25 us 326.41 us 328.51 us]
                        thrpt:  [99.749 Melem/s 100.39 Melem/s 101.06 Melem/s]

atomic_counter/8        time:   [345.57 us 346.09 us 346.61 us]
                        thrpt:  [94.539 Melem/s 94.681 Melem/s 94.824 Melem/s]

atomic_counter/16       time:   [414.53 us 415.65 us 416.83 us]
                        thrpt:  [78.612 Melem/s 78.836 Melem/s 79.048 Melem/s]


==============================================
==============================================


fast_counter/2          time:   [370.83 us 377.43 us 383.15 us]
                        thrpt:  [85.522 Melem/s 86.818 Melem/s 88.364 Melem/s]

fast_counter/4          time:   [338.49 us 345.35 us 351.70 us]
                        thrpt:  [93.171 Melem/s 94.882 Melem/s 96.807 Melem/s]

fast_counter/8          time:   [249.25 us 254.46 us 259.47 us]
                        thrpt:  [126.29 Melem/s 128.78 Melem/s 131.47 Melem/s]

fast_counter/16         time:   [163.34 us 169.76 us 176.39 us]
                        thrpt:  [185.77 Melem/s 193.03 Melem/s 200.61 Melem/s]


==============================================
==============================================


fast_counter thread local macro/2
                        time:   [388.31 us 392.67 us 396.95 us]
                        thrpt:  [82.549 Melem/s 83.449 Melem/s 84.387 Melem/s]

fast_counter thread local macro/4
                        time:   [364.32 us 369.14 us 373.44 us]
                        thrpt:  [87.746 Melem/s 88.769 Melem/s 89.943 Melem/s]

fast_counter thread local macro/8
                        time:   [254.32 us 259.57 us 265.15 us]
                        thrpt:  [123.58 Melem/s 126.24 Melem/s 128.84 Melem/s]

fast_counter thread local macro/16
                        time:   [172.06 us 175.66 us 179.73 us]
                        thrpt:  [182.32 Melem/s 186.54 Melem/s 190.44 Melem/s]
```

Big shoutout to @jimvdl who put the core starting point for this together