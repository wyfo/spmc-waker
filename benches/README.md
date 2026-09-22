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
│  ├─ AtomicWaker                                                                                         18.25 ns      │ 32.9 ns       │ 21.13 ns      │ 21.69 ns      │ 100     │ 102400
│  ├─ DiatomicWaker                                                                                       6.682 ns      │ 24.16 ns      │ 6.731 ns      │ 7.441 ns      │ 100     │ 204800
│  ├─ SpmcWaker                                                                                           17.57 ns      │ 20.98 ns      │ 17.66 ns      │ 17.74 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      11.32 ns      │ 11.41 ns      │ 11.41 ns      │ 11.37 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       4.924 ns      │ 9.075 ns      │ 4.973 ns      │ 5.589 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            11.12 ns      │ 13.37 ns      │ 11.22 ns      │ 11.24 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  17.08 ns      │ 25.28 ns      │ 17.18 ns      │ 17.31 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    11.51 ns      │ 14.64 ns      │ 11.61 ns      │ 11.68 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     5.51 ns       │ 9.319 ns      │ 5.51 ns       │ 5.576 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          12.88 ns      │ 14.15 ns      │ 12.98 ns      │ 12.97 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  7.414 ns      │ 10.34 ns      │ 7.463 ns      │ 7.529 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   0.92 ns       │ 4.668 ns      │ 0.932 ns      │ 1.004 ns      │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        7.707 ns      │ 15.91 ns      │ 7.805 ns      │ 7.896 ns      │ 100     │ 102400
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              11.9 ns       │ 17.66 ns      │ 11.9 ns       │ 11.99 ns      │ 100     │ 102400
├─ register_already_registered                                                                                          │               │               │               │         │
│  ├─ AtomicWaker                                                                                         12.39 ns      │ 48.62 ns      │ 12.49 ns      │ 12.83 ns      │ 100     │ 102400
│  ├─ DiatomicWaker                                                                                       6.389 ns      │ 18.45 ns      │ 6.438 ns      │ 6.657 ns      │ 100     │ 204800
│  ├─ SpmcWaker                                                                                           12.68 ns      │ 15.61 ns      │ 12.78 ns      │ 12.81 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      2.263 ns      │ 3.825 ns      │ 2.287 ns      │ 2.318 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       2.092 ns      │ 12.37 ns      │ 2.116 ns      │ 2.302 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            13.56 ns      │ 22.35 ns      │ 13.66 ns      │ 13.88 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  13.56 ns      │ 19.62 ns      │ 13.66 ns      │ 13.72 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    2.141 ns      │ 3.996 ns      │ 2.312 ns      │ 2.429 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     1.775 ns      │ 2.763 ns      │ 2.019 ns      │ 2.066 ns      │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          11.12 ns      │ 30.95 ns      │ 11.22 ns      │ 11.99 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  1.885 ns      │ 6.902 ns      │ 2.019 ns      │ 2.087 ns      │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   1.762 ns      │ 17.37 ns      │ 1.824 ns      │ 2.023 ns      │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        6.145 ns      │ 53.06 ns      │ 6.242 ns      │ 7.44 ns       │ 100     │ 204800
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              6.389 ns      │ 15.08 ns      │ 6.438 ns      │ 6.526 ns      │ 100     │ 204800
├─ register_overwrite                                                                                                   │               │               │               │         │
│  ├─ AtomicWaker                                                                                         22.74 ns      │ 34.27 ns      │ 22.94 ns      │ 23.1 ns       │ 100     │ 51200
│  ├─ DiatomicWaker                                                                                       22.35 ns      │ 153 ns        │ 22.55 ns      │ 24.55 ns      │ 100     │ 51200
│  ├─ SpmcWaker                                                                                           22.94 ns      │ 112.5 ns      │ 23.13 ns      │ 29.04 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      21.77 ns      │ 104.3 ns      │ 21.96 ns      │ 25.31 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       21.96 ns      │ 87.59 ns      │ 22.16 ns      │ 22.93 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            22.35 ns      │ 101.4 ns      │ 22.35 ns      │ 25.63 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  22.74 ns      │ 106.5 ns      │ 22.94 ns      │ 26.37 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    21.96 ns      │ 95.01 ns      │ 21.96 ns      │ 24.3 ns       │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     21.37 ns      │ 170.2 ns      │ 21.96 ns      │ 25.48 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          23.72 ns      │ 63.37 ns      │ 23.91 ns      │ 24.38 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  16.39 ns      │ 59.36 ns      │ 16.49 ns      │ 18.3 ns       │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   16.39 ns      │ 36.61 ns      │ 16.49 ns      │ 17.29 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        17.08 ns      │ 51.45 ns      │ 17.18 ns      │ 18.25 ns      │ 100     │ 102400
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              17.57 ns      │ 54.58 ns      │ 18.15 ns      │ 19.73 ns      │ 100     │ 102400
├─ wake                                                                                                                 │               │               │               │         │
│  ├─ AtomicWaker                                                                                         17.18 ns      │ 57.7 ns       │ 17.32 ns      │ 20.89 ns      │ 100     │ 102400
│  ├─ DiatomicWaker                                                                                       11.61 ns      │ 78.21 ns      │ 11.8 ns       │ 13.07 ns      │ 100     │ 51200
│  ├─ SpmcWaker                                                                                           10.24 ns      │ 44.52 ns      │ 10.34 ns      │ 10.76 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      10.34 ns      │ 49.5 ns       │ 10.34 ns      │ 11.12 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       10.53 ns      │ 51.45 ns      │ 10.88 ns      │ 12.6 ns       │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            10.53 ns      │ 47.94 ns      │ 10.63 ns      │ 11.58 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  10.05 ns      │ 83.29 ns      │ 10.34 ns      │ 12.85 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    10.34 ns      │ 47.55 ns      │ 10.34 ns      │ 11.56 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     10.83 ns      │ 50.87 ns      │ 10.93 ns      │ 12.08 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          10.83 ns      │ 50.48 ns      │ 10.93 ns      │ 11.47 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  10.34 ns      │ 12.59 ns      │ 10.34 ns      │ 10.48 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   10.83 ns      │ 43.35 ns      │ 10.93 ns      │ 11.39 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        10.83 ns      │ 11.9 ns       │ 10.93 ns      │ 10.95 ns      │ 100     │ 102400
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              10.24 ns      │ 16.1 ns       │ 10.34 ns      │ 10.48 ns      │ 100     │ 102400
├─ wake_cold                                                                                                            │               │               │               │         │
│  ├─ AtomicWaker                                                                                         17.18 ns      │ 85.14 ns      │ 17.27 ns      │ 19.28 ns      │ 100     │ 102400
│  ├─ DiatomicWaker                                                                                       11.61 ns      │ 48.62 ns      │ 11.71 ns      │ 13.47 ns      │ 100     │ 102400
│  ├─ SpmcWaker                                                                                           10.53 ns      │ 50.18 ns      │ 10.63 ns      │ 11.79 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>      10.34 ns      │ 48.62 ns      │ 10.34 ns      │ 11.16 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>       12.2 ns       │ 49.01 ns      │ 12.39 ns      │ 13.42 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                            12.2 ns       │ 524.7 ns      │ 12.2 ns       │ 18.13 ns      │ 100     │ 6400
│  ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                  10.05 ns      │ 89.93 ns      │ 10.14 ns      │ 12.13 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>    10.24 ns      │ 30.26 ns      │ 10.34 ns      │ 10.76 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>     12.2 ns       │ 32.02 ns      │ 12.29 ns      │ 12.65 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                          12.2 ns       │ 32.12 ns      │ 12.29 ns      │ 12.49 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>  10.05 ns      │ 82.02 ns      │ 10.14 ns      │ 14.2 ns       │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>   11.8 ns       │ 80.55 ns      │ 12.2 ns       │ 15.83 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                        11.9 ns       │ 101.4 ns      │ 12 ns         │ 13.99 ns      │ 100     │ 102400
│  ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                              10.05 ns      │ 11.9 ns       │ 10.05 ns      │ 10.11 ns      │ 100     │ 102400
╰─ wake_cold_empty                                                                                                      │               │               │               │         │
   ├─ AtomicWaker                                                                                                       │               │               │               │         │
   │  ├─ t=1                                                                                              14.05 ns      │ 19.81 ns      │ 14.15 ns      │ 14.19 ns      │ 100     │ 102400
   │  ├─ t=2                                                                                              53.6 ns       │ 388.7 ns      │ 122.3 ns      │ 125.3 ns      │ 100     │ 12800
   │  ╰─ t=4                                                                                              135.6 ns      │ 240.3 ns      │ 198.1 ns      │ 194.5 ns      │ 100     │ 6400
   ├─ DiatomicWaker                                                                                                     │               │               │               │         │
   │  ├─ t=1                                                                                              9.075 ns      │ 137.9 ns      │ 9.856 ns      │ 10.91 ns      │ 100     │ 12800
   │  ├─ t=2                                                                                              20.01 ns      │ 58.68 ns      │ 53.99 ns      │ 53.81 ns      │ 100     │ 25600
   │  ╰─ t=4                                                                                              104.3 ns      │ 134 ns        │ 121.5 ns      │ 121 ns        │ 100     │ 12800
   ├─ SpmcWaker                                                                                                         │               │               │               │         │
   │  ├─ t=1                                                                                              4.826 ns      │ 14.59 ns      │ 4.875 ns      │ 5.25 ns       │ 100     │ 204800
   │  ├─ t=2                                                                                              4.973 ns      │ 7.951 ns      │ 5.022 ns      │ 5.031 ns      │ 100     │ 204800
   │  ╰─ t=4                                                                                              5.412 ns      │ 6.731 ns      │ 5.461 ns      │ 5.465 ns      │ 100     │ 204800
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, false, spmc_waker::registration::Unchecked>                    │               │               │               │         │
   │  ├─ t=1                                                                                              0.231 ns      │ 2.025 ns      │ 0.279 ns      │ 0.327 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.246 ns      │ 0.664 ns      │ 0.285 ns      │ 0.281 ns      │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.292 ns      │ 3.533 ns      │ 0.34 ns       │ 0.631 ns      │ 100     │ 1638400
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true, spmc_waker::registration::Unchecked>                     │               │               │               │         │
   │  ├─ t=1                                                                                              0.231 ns      │ 1.921 ns      │ 0.279 ns      │ 0.353 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.246 ns      │ 0.368 ns      │ 0.285 ns      │ 0.28 ns       │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.285 ns      │ 1.445 ns      │ 0.346 ns      │ 0.368 ns      │ 100     │ 819200
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential, true>                                                          │               │               │               │         │
   │  ├─ t=1                                                                                              0.231 ns      │ 1.445 ns      │ 0.231 ns      │ 0.298 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.243 ns      │ 0.475 ns      │ 0.285 ns      │ 0.28 ns       │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.298 ns      │ 1.482 ns      │ 1.158 ns      │ 0.978 ns      │ 100     │ 819200
   ├─ SpmcWaker<spmc_waker::synchronization::Sequential>                                                                │               │               │               │         │
   │  ├─ t=1                                                                                              0.231 ns      │ 0.884 ns      │ 0.279 ns      │ 0.287 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.243 ns      │ 0.481 ns      │ 0.285 ns      │ 0.283 ns      │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.298 ns      │ 1.47 ns       │ 0.438 ns      │ 0.775 ns      │ 100     │ 819200
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, false, spmc_waker::registration::Unchecked>                  │               │               │               │         │
   │  ├─ t=1                                                                                              4.973 ns      │ 24.01 ns      │ 5.022 ns      │ 5.911 ns      │ 100     │ 204800
   │  ├─ t=2                                                                                              4.973 ns      │ 23.08 ns      │ 5.022 ns      │ 5.764 ns      │ 100     │ 204800
   │  ╰─ t=4                                                                                              5.461 ns      │ 12.29 ns      │ 11.27 ns      │ 8.783 ns      │ 100     │ 102400
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true, spmc_waker::registration::Unchecked>                   │               │               │               │         │
   │  ├─ t=1                                                                                              4.826 ns      │ 17.08 ns      │ 4.973 ns      │ 5.12 ns       │ 100     │ 204800
   │  ├─ t=2                                                                                              4.973 ns      │ 7.854 ns      │ 5.022 ns      │ 5.029 ns      │ 100     │ 204800
   │  ╰─ t=4                                                                                              5.461 ns      │ 17.86 ns      │ 8.537 ns      │ 8.818 ns      │ 100     │ 102400
   ├─ SpmcWaker<spmc_waker::synchronization::Synchronized, true>                                                        │               │               │               │         │
   │  ├─ t=1                                                                                              4.826 ns      │ 45.4 ns       │ 5.022 ns      │ 6.643 ns      │ 100     │ 204800
   │  ├─ t=2                                                                                              4.973 ns      │ 7.854 ns      │ 5.022 ns      │ 5.056 ns      │ 100     │ 204800
   │  ╰─ t=4                                                                                              5.412 ns      │ 5.51 ns       │ 5.461 ns      │ 5.455 ns      │ 100     │ 204800
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, false, spmc_waker::registration::Unchecked>                │               │               │               │         │
   │  ├─ t=1                                                                                              0.212 ns      │ 4.68 ns       │ 0.267 ns      │ 0.378 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.227 ns      │ 0.346 ns      │ 0.279 ns      │ 0.263 ns      │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.273 ns      │ 0.478 ns      │ 0.292 ns      │ 0.312 ns      │ 100     │ 3276800
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true, spmc_waker::registration::Unchecked>                 │               │               │               │         │
   │  ├─ t=1                                                                                              0.215 ns      │ 2.193 ns      │ 0.227 ns      │ 0.294 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.231 ns      │ 0.359 ns      │ 0.276 ns      │ 0.263 ns      │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.285 ns      │ 4.009 ns      │ 0.328 ns      │ 0.442 ns      │ 100     │ 1638400
   ├─ SpmcWaker<spmc_waker::synchronization::Unsynchronized, true>                                                      │               │               │               │         │
   │  ├─ t=1                                                                                              0.224 ns      │ 1.476 ns      │ 0.258 ns      │ 0.268 ns      │ 100     │ 3276800
   │  ├─ t=2                                                                                              0.231 ns      │ 1.793 ns      │ 0.279 ns      │ 0.338 ns      │ 100     │ 3276800
   │  ╰─ t=4                                                                                              0.273 ns      │ 5.974 ns      │ 0.334 ns      │ 0.428 ns      │ 100     │ 819200
   ╰─ SpmcWaker<spmc_waker::synchronization::Unsynchronized>                                                            │               │               │               │         │
      ├─ t=1                                                                                              0.224 ns      │ 1.607 ns      │ 0.267 ns      │ 0.314 ns      │ 100     │ 3276800
      ├─ t=2                                                                                              0.237 ns      │ 2.422 ns      │ 0.285 ns      │ 0.322 ns      │ 100     │ 1638400
      ╰─ t=4                                                                                              0.273 ns      │ 4.039 ns      │ 0.331 ns      │ 0.456 ns      │ 100     │ 3276800
```