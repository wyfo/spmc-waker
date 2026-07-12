# Benchmark

This benchmark compares `SpmcWaker<S, CACHING, R>` with `futures::task::AtomicWaker`; [`diatomic_waker::DiatomicWaker`](https://docs.rs/diatomic-waker/latest/diatomic_waker/struct.DiatomicWaker.html) is also included for completeness.
As expected, `SpmcWaker` is significantly faster than the alternatives in any meaningful scenario.

The following scenarios are measured:
- `register`: register a waker (with potentially the same waker already in cache)
- `register_already_registered`: register a waker which is already registered
- `register_overwrite`: register a waker while another one is already registered
- `wake`: wake a registered waker
- `wake_cold`: wake a registered waker with `wake_cold` for `SpmcWaker` (same as `wake` for others)
- `wake_cold_empty`: call `wake_cold` with no waker registered on `t` threads

The most important gain is on `wake_cold_empty` with `S=Sequential` and `S=Unsynchronized`, which is not surprising as it compiles to a single atomic load. However, `wake_cold` is also read-only (but still pays a fence) with default `S=Synchronized` on x86, which is why the benchmark shows no contention impact.

Another visible result is the performance gain with `CACHING=true`. In fact, it saves most atomic RMW operations updating the waker's reference count (except in overwrite). `DiatomicWaker` also uses caching, which explains its better numbers than `AtomicWaker`.

## Results

### x86_64 (Intel Core i7-1065G7)

```
Timer precision: 100 ns
comparison                                                                                                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                                                                                             │               │               │               │         │
│  ├─ AtomicWaker                                                                                         16.3 ns       │ 310.8 ns      │ 18.25 ns      │ 18.97 ns      │ 2553    │ 1307136
│  ├─ DiatomicWaker                                                                                       5.95 ns       │ 53.94 ns      │ 6.78 ns       │ 7.293 ns      │ 838     │ 1716224
│  ├─ SpmcWaker                                                                                           17.08 ns      │ 696.6 ns      │ 17.47 ns      │ 18.92 ns      │ 1455    │ 1489920
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      10.14 ns      │ 64.05 ns      │ 11.32 ns      │ 11.45 ns      │ 1809    │ 1852416
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       4.387 ns      │ 46.57 ns      │ 4.485 ns      │ 4.826 ns      │ 1912    │ 1957888
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            10.05 ns      │ 228.2 ns      │ 10.44 ns      │ 12.28 ns      │ 3005    │ 1538560
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  15.71 ns      │ 118 ns        │ 16.1 ns       │ 17.93 ns      │ 3138    │ 1606656
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    10.14 ns      │ 112.3 ns      │ 10.24 ns      │ 11.55 ns      │ 1862    │ 1906688
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     4.411 ns      │ 23.5 ns       │ 4.558 ns      │ 5.314 ns      │ 469     │ 1921024
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          9.953 ns      │ 89.64 ns      │ 10.24 ns      │ 11.17 ns      │ 1556    │ 1593344
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  6.633 ns      │ 49.21 ns      │ 8.293 ns      │ 9.622 ns      │ 950     │ 1945600
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   0.823 ns      │ 5.986 ns      │ 1.128 ns      │ 1.366 ns      │ 240     │ 1966080
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        7.073 ns      │ 27.72 ns      │ 7.17 ns       │ 8.111 ns      │ 949     │ 1943552
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              10.83 ns      │ 103.4 ns      │ 10.93 ns      │ 12.4 ns       │ 2101    │ 2151424
├─ register_already_registered                                                                                          │               │               │               │         │
│  ├─ AtomicWaker                                                                                         11.12 ns      │ 112.2 ns      │ 12.49 ns      │ 13.5 ns       │ 2062    │ 2111488
│  ├─ DiatomicWaker                                                                                       6.145 ns      │ 28.55 ns      │ 6.194 ns      │ 6.969 ns      │ 1355    │ 2775040
│  ├─ SpmcWaker                                                                                           12.1 ns       │ 96.57 ns      │ 12.29 ns      │ 13.72 ns      │ 2105    │ 2155520
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      1.897 ns      │ 26.79 ns      │ 2.092 ns      │ 2.435 ns      │ 1650    │ 3379200
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       1.848 ns      │ 21.52 ns      │ 2.019 ns      │ 2.308 ns      │ 819     │ 3354624
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            12.1 ns       │ 86.71 ns      │ 12.49 ns      │ 13.64 ns      │ 2062    │ 2111488
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  12.1 ns       │ 103.8 ns      │ 12.2 ns       │ 13.34 ns      │ 2134    │ 2185216
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    1.885 ns      │ 13.7 ns       │ 2.214 ns      │ 2.636 ns      │ 411     │ 3366912
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     1.848 ns      │ 20.03 ns      │ 2.019 ns      │ 2.452 ns      │ 781     │ 3198976
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          12.1 ns       │ 77.53 ns      │ 12.59 ns      │ 14.09 ns      │ 2007    │ 2055168
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  1.872 ns      │ 12.37 ns      │ 1.897 ns      │ 2.224 ns      │ 1039    │ 4255744
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   1.762 ns      │ 11.98 ns      │ 1.824 ns      │ 2.219 ns      │ 411     │ 3366912
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        6.145 ns      │ 47.01 ns      │ 6.291 ns      │ 7.716 ns      │ 1265    │ 2590720
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              6.194 ns      │ 51.84 ns      │ 6.389 ns      │ 7.249 ns      │ 1357    │ 2779136
├─ register_overwrite                                                                                                   │               │               │               │         │
│  ├─ AtomicWaker                                                                                         22.16 ns      │ 212.3 ns      │ 22.94 ns      │ 25.54 ns      │ 3373    │ 1726976
│  ├─ DiatomicWaker                                                                                       21.77 ns      │ 179.7 ns      │ 22.16 ns      │ 24.37 ns      │ 3606    │ 1846272
│  ├─ SpmcWaker                                                                                           22.16 ns      │ 270.5 ns      │ 22.55 ns      │ 25.14 ns      │ 3368    │ 1724416
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      21.77 ns      │ 145.9 ns      │ 23.72 ns      │ 23.89 ns      │ 3806    │ 1948672
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       21.96 ns      │ 213.9 ns      │ 22.16 ns      │ 26.28 ns      │ 3483    │ 1783296
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            21.57 ns      │ 342.4 ns      │ 24.3 ns       │ 24.88 ns      │ 3324    │ 1701888
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  22.74 ns      │ 202 ns        │ 23.13 ns      │ 26.49 ns      │ 3212    │ 1644544
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    21.77 ns      │ 148.5 ns      │ 23.72 ns      │ 24.55 ns      │ 3737    │ 1913344
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     21.96 ns      │ 181.1 ns      │ 23.91 ns      │ 24.76 ns      │ 3694    │ 1891328
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          21.37 ns      │ 238.5 ns      │ 21.96 ns      │ 24.24 ns      │ 3360    │ 1720320
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  16.69 ns      │ 111.2 ns      │ 17.18 ns      │ 19.96 ns      │ 2323    │ 2378752
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   16.49 ns      │ 99.3 ns       │ 17.08 ns      │ 19.55 ns      │ 2014    │ 2062336
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        16.88 ns      │ 229.1 ns      │ 19.03 ns      │ 19.7 ns       │ 3974    │ 2034688
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              17.57 ns      │ 109.5 ns      │ 18.35 ns      │ 20.47 ns      │ 2006    │ 2054144
├─ wake                                                                                                                 │               │               │               │         │
│  ├─ AtomicWaker                                                                                         16.78 ns      │ 93.93 ns      │ 17.27 ns      │ 19.38 ns      │ 2115    │ 2165760
│  ├─ DiatomicWaker                                                                                       11.32 ns      │ 84.75 ns      │ 12.78 ns      │ 12.91 ns      │ 2300    │ 2355200
│  ├─ SpmcWaker                                                                                           10.05 ns      │ 122.5 ns      │ 10.14 ns      │ 11.01 ns      │ 2735    │ 2800640
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      10.05 ns      │ 147.9 ns      │ 10.05 ns      │ 11.28 ns      │ 3238    │ 3315712
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       10.34 ns      │ 83.87 ns      │ 10.63 ns      │ 11.67 ns      │ 2551    │ 2612224
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            10.53 ns      │ 110.2 ns      │ 10.63 ns      │ 11.99 ns      │ 2113    │ 2163712
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  10.05 ns      │ 93.05 ns      │ 10.34 ns      │ 11.7 ns       │ 2630    │ 2693120
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    10.05 ns      │ 161.2 ns      │ 10.34 ns      │ 11.23 ns      │ 3254    │ 3332096
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     10.34 ns      │ 154.8 ns      │ 10.63 ns      │ 11.89 ns      │ 2528    │ 2588672
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          10.44 ns      │ 133 ns        │ 10.93 ns      │ 12.4 ns       │ 2082    │ 2131968
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  10.05 ns      │ 94.23 ns      │ 10.34 ns      │ 11.67 ns      │ 3743    │ 3832832
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   10.53 ns      │ 70.98 ns      │ 11.9 ns       │ 12.35 ns      │ 2437    │ 2495488
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        10.63 ns      │ 71.86 ns      │ 11.9 ns       │ 13.83 ns      │ 2166    │ 2217984
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              10.24 ns      │ 163.1 ns      │ 10.34 ns      │ 11.11 ns      │ 3194    │ 3270656
├─ wake_cold                                                                                                            │               │               │               │         │
│  ├─ AtomicWaker                                                                                         17.18 ns      │ 54.19 ns      │ 18.84 ns      │ 19.13 ns      │ 2121    │ 2171904
│  ├─ DiatomicWaker                                                                                       11.32 ns      │ 86.51 ns      │ 12.68 ns      │ 12.91 ns      │ 2298    │ 2353152
│  ├─ SpmcWaker                                                                                           10.24 ns      │ 83.97 ns      │ 10.53 ns      │ 11.61 ns      │ 2679    │ 2743296
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      10.05 ns      │ 169.8 ns      │ 10.24 ns      │ 11.74 ns      │ 6201    │ 3174912
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       11.9 ns       │ 92.86 ns      │ 12.29 ns      │ 13.52 ns      │ 2358    │ 2414592
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            11.9 ns       │ 149.9 ns      │ 13.37 ns      │ 14.72 ns      │ 1918    │ 1964032
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  10.24 ns      │ 68.74 ns      │ 10.34 ns      │ 11.65 ns      │ 2593    │ 2655232
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    10.24 ns      │ 85.83 ns      │ 10.63 ns      │ 11.69 ns      │ 3157    │ 3232768
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     12.2 ns       │ 71.67 ns      │ 12.59 ns      │ 14.29 ns      │ 2223    │ 2276352
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          12.2 ns       │ 1.549 µs      │ 12.2 ns       │ 14.78 ns      │ 59431   │ 1901792
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  10.05 ns      │ 292.6 ns      │ 10.05 ns      │ 11.46 ns      │ 7652    │ 3917824
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   12.2 ns       │ 2.415 µs      │ 12.2 ns       │ 14.63 ns      │ 65586   │ 2098752
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        11.9 ns       │ 123.8 ns      │ 12.29 ns      │ 13.24 ns      │ 2349    │ 2405376
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              10.05 ns      │ 76.26 ns      │ 11.22 ns      │ 11.63 ns      │ 3039    │ 3111936
╰─ wake_cold_empty                                                                                                      │               │               │               │         │
   ├─ AtomicWaker                                                                                                       │               │               │               │         │
   │  ├─ t=1                                                                                              17.66 ns      │ 85.63 ns      │ 17.86 ns      │ 18.39 ns      │ 9793    │ 5014016
   │  ├─ t=2                                                                                              11.41 ns      │ 529.3 ns      │ 123.1 ns      │ 121.1 ns      │ 11192   │ 1432576
   │  ╰─ t=4                                                                                              18.45 ns      │ 1.809 µs      │ 205.9 ns      │ 204 ns        │ 20792   │ 1330688
   ├─ DiatomicWaker                                                                                                     │               │               │               │         │
   │  ├─ t=1                                                                                              9.66 ns       │ 107.1 ns      │ 10.53 ns      │ 10.48 ns      │ 8374    │ 8574976
   │  ├─ t=2                                                                                              9.856 ns      │ 466.1 ns      │ 46.96 ns      │ 42.13 ns      │ 15348   │ 3929088
   │  ╰─ t=4                                                                                              10.63 ns      │ 483.2 ns      │ 96.57 ns      │ 96.73 ns      │ 22440   │ 2872320
   ├─ SpmcWaker                                                                                                         │               │               │               │         │
   │  ├─ t=1                                                                                              4.973 ns      │ 65.12 ns      │ 7.121 ns      │ 8.101 ns      │ 5078    │ 10399744
   │  ├─ t=2                                                                                              5.364 ns      │ 261.6 ns      │ 5.559 ns      │ 6.49 ns       │ 35130   │ 17986560
   │  ╰─ t=4                                                                                              5.364 ns      │ 81.53 ns      │ 5.461 ns      │ 6.545 ns      │ 30452   │ 31182848
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>                    │               │               │               │         │
   │  ├─ t=1                                                                                              0.328 ns      │ 3.172 ns      │ 0.334 ns      │ 0.354 ns      │ 4359    │ 71417856
   │  ├─ t=2                                                                                              0.328 ns      │ 2.702 ns      │ 0.34 ns       │ 0.38 ns       │ 7674    │ 125730816
   │  ╰─ t=4                                                                                              0.334 ns      │ 4.338 ns      │ 1.14 ns       │ 0.922 ns      │ 11624   │ 95223808
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>                     │               │               │               │         │
   │  ├─ t=1                                                                                              0.292 ns      │ 5.834 ns      │ 0.34 ns       │ 0.402 ns      │ 4098    │ 67141632
   │  ├─ t=2                                                                                              0.292 ns      │ 3.282 ns      │ 0.34 ns       │ 0.388 ns      │ 7348    │ 120389632
   │  ╰─ t=4                                                                                              0.273 ns      │ 41.44 ns      │ 0.346 ns      │ 0.549 ns      │ 16168   │ 132448256
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                                          │               │               │               │         │
   │  ├─ t=1                                                                                              0.288 ns      │ 1.799 ns      │ 0.343 ns      │ 0.336 ns      │ 2218    │ 72679424
   │  ├─ t=2                                                                                              0.243 ns      │ 4.82 ns       │ 0.298 ns      │ 0.369 ns      │ 7678    │ 125796352
   │  ╰─ t=4                                                                                              0.249 ns      │ 9.807 ns      │ 0.346 ns      │ 0.411 ns      │ 20736   │ 169869312
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                                │               │               │               │         │
   │  ├─ t=1                                                                                              0.279 ns      │ 2.519 ns      │ 0.334 ns      │ 0.347 ns      │ 4403    │ 72138752
   │  ├─ t=2                                                                                              0.273 ns      │ 5.925 ns      │ 0.292 ns      │ 0.407 ns      │ 7484    │ 122617856
   │  ╰─ t=4                                                                                              0.322 ns      │ 12.84 ns      │ 0.346 ns      │ 0.428 ns      │ 20092   │ 164593664
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>                  │               │               │               │         │
   │  ├─ t=1                                                                                              4.826 ns      │ 47.3 ns       │ 5.461 ns      │ 5.75 ns       │ 7106    │ 14553088
   │  ├─ t=2                                                                                              5.364 ns      │ 51.45 ns      │ 5.461 ns      │ 6.943 ns      │ 20016   │ 20496384
   │  ╰─ t=4                                                                                              5.364 ns      │ 62.49 ns      │ 5.461 ns      │ 6.031 ns      │ 33168   │ 33964032
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>                   │               │               │               │         │
   │  ├─ t=1                                                                                              5.412 ns      │ 47.79 ns      │ 5.461 ns      │ 5.731 ns      │ 7111    │ 14563328
   │  ├─ t=2                                                                                              5.364 ns      │ 122.8 ns      │ 5.559 ns      │ 6.708 ns      │ 19882   │ 20359168
   │  ╰─ t=4                                                                                              5.364 ns      │ 94.62 ns      │ 5.461 ns      │ 6.087 ns      │ 32652   │ 33435648
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                                        │               │               │               │         │
   │  ├─ t=1                                                                                              5.412 ns      │ 54.24 ns      │ 5.461 ns      │ 5.58 ns       │ 7281    │ 14911488
   │  ├─ t=2                                                                                              5.364 ns      │ 93.45 ns      │ 5.559 ns      │ 7.382 ns      │ 18320   │ 18759680
   │  ╰─ t=4                                                                                              5.412 ns      │ 54.28 ns      │ 5.461 ns      │ 6.184 ns      │ 18736   │ 38371328
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>                │               │               │               │         │
   │  ├─ t=1                                                                                              0.267 ns      │ 8.244 ns      │ 0.322 ns      │ 0.366 ns      │ 4245    │ 69550080
   │  ├─ t=2                                                                                              0.237 ns      │ 6.297 ns      │ 0.322 ns      │ 0.338 ns      │ 7982    │ 130777088
   │  ╰─ t=4                                                                                              0.285 ns      │ 8.659 ns      │ 0.334 ns      │ 0.69 ns       │ 26960   │ 110428160
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>                 │               │               │               │         │
   │  ├─ t=1                                                                                              0.224 ns      │ 5.547 ns      │ 0.322 ns      │ 0.326 ns      │ 4514    │ 73957376
   │  ├─ t=2                                                                                              0.237 ns      │ 62.21 ns      │ 0.322 ns      │ 0.35 ns       │ 7874    │ 129007616
   │  ╰─ t=4                                                                                              0.273 ns      │ 211 ns        │ 0.334 ns      │ 0.381 ns      │ 22252   │ 182288384
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                                      │               │               │               │         │
   │  ├─ t=1                                                                                              0.237 ns      │ 2.19 ns       │ 0.322 ns      │ 0.327 ns      │ 4491    │ 73580544
   │  ├─ t=2                                                                                              0.231 ns      │ 7.909 ns      │ 0.279 ns      │ 0.346 ns      │ 7892    │ 129302528
   │  ╰─ t=4                                                                                              0.273 ns      │ 14.72 ns      │ 0.334 ns      │ 0.392 ns      │ 20972   │ 171802624
   ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                                            │               │               │               │         │
      ├─ t=1                                                                                              0.279 ns      │ 4.149 ns      │ 0.322 ns      │ 0.357 ns      │ 4302    │ 70483968
      ├─ t=2                                                                                              0.237 ns      │ 6.169 ns      │ 0.322 ns      │ 0.336 ns      │ 15236   │ 124813312
      ╰─ t=4                                                                                              0.273 ns      │ 12.67 ns      │ 0.334 ns      │ 0.463 ns      │ 18924   │ 155025408


```