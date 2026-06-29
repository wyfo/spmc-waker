# Benchmark

This benchmark compares `SpmcWaker<S, CACHED>` with `futures::task::AtomicWaker`; [`diatomic_waker::DiatomicWaker`](https://docs.rs/diatomic-waker/latest/diatomic_waker/struct.DiatomicWaker.html) is also included for completeness.
As expected, `SpmcWaker` is significantly faster than the alternatives in any meaningful scenario.

The following scenarios are measured:
- `register`: register a waker (with potentially the same waker already in cache)
- `register_already_registered`: register a waker which is already registered
- `register_overwrite`: register a waker while another one is already registered
- `wake`: wake a registered waker
- `wake_cold`: wake a registered waker with `wake_cold` for `SpmcWaker` (same as `wake` for others)
- `wake_cold_empty`: call `wake_cold` with no waker registered on `t` threads

The most important gain is on `wake_cold_empty` with `S=Sequential` and `S=Unsynchronized`, which is not surprising as it compiles to a single atomic load. However, `wake_cold` is also read-only (but still pays a fence) with default `S=Synchronized` on x86, which is why the benchmark shows no contention impact.

Another visible result is the performance gain with `CACHED=true`. In fact, it saves most atomic RMW operations updating the waker's reference count (except in overwrite). `DiatomicWaker` also uses caching, which explains its better numbers than `AtomicWaker`.

## Results

### x86_64 (Intel(R) Xeon(R) CPU E3-1275 v5)

```
Timer precision: 16 ns
comparison                                                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                                             │               │               │               │         │
│  ├─ AtomicWaker                                         18.31 ns      │ 18.92 ns      │ 18.38 ns      │ 18.39 ns      │ 100     │ 12800
│  ├─ DiatomicWaker                                       6.944 ns      │ 84.99 ns      │ 7.001 ns      │ 8.037 ns      │ 100     │ 25600
│  ├─ SpmcWaker                                           4.78 ns       │ 4.845 ns      │ 4.794 ns      │ 4.795 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      9.995 ns      │ 10.08 ns      │ 10.02 ns      │ 10.02 ns      │ 100     │ 25600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             4.778 ns      │ 15.1 ns       │ 4.793 ns      │ 4.899 ns      │ 100     │ 51200
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    9.995 ns      │ 61.25 ns      │ 10.02 ns      │ 10.9 ns       │ 100     │ 25600
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  6.382 ns      │ 6.433 ns      │ 6.4 ns        │ 6.401 ns      │ 100     │ 25600
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         0.735 ns      │ 0.806 ns      │ 0.767 ns      │ 0.765 ns      │ 100     │ 204800
├─ register_already_registered                                          │               │               │               │         │
│  ├─ AtomicWaker                                         13.1 ns       │ 119.7 ns      │ 13.15 ns      │ 14.24 ns      │ 100     │ 12800
│  ├─ DiatomicWaker                                       6.757 ns      │ 7.038 ns      │ 6.8 ns        │ 6.804 ns      │ 100     │ 25600
│  ├─ SpmcWaker                                           1.606 ns      │ 1.715 ns      │ 1.641 ns      │ 1.641 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      1.465 ns      │ 16.09 ns      │ 1.499 ns      │ 1.663 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             1.599 ns      │ 17.78 ns      │ 1.631 ns      │ 1.791 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    1.46 ns       │ 15.83 ns      │ 1.486 ns      │ 1.63 ns       │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  1.554 ns      │ 14.2 ns       │ 1.576 ns      │ 1.7 ns        │ 100     │ 102400
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         1.6 ns        │ 4.754 ns      │ 1.645 ns      │ 1.675 ns      │ 100     │ 102400
├─ register_overwrite                                                   │               │               │               │         │
│  ├─ AtomicWaker                                         22.14 ns      │ 122.8 ns      │ 22.31 ns      │ 23.32 ns      │ 100     │ 12800
│  ├─ DiatomicWaker                                       23.09 ns      │ 141.3 ns      │ 24.06 ns      │ 26.08 ns      │ 100     │ 12800
│  ├─ SpmcWaker                                           23.27 ns      │ 62.45 ns      │ 23.34 ns      │ 23.99 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      23.03 ns      │ 124.8 ns      │ 23.1 ns       │ 24.52 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             23.26 ns      │ 139.4 ns      │ 23.34 ns      │ 27.51 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    23.04 ns      │ 137.7 ns      │ 23.1 ns       │ 24.51 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  17.32 ns      │ 133 ns        │ 17.67 ns      │ 19.21 ns      │ 100     │ 12800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         17.22 ns      │ 133.4 ns      │ 18.35 ns      │ 19.41 ns      │ 100     │ 12800
├─ wake                                                                 │               │               │               │         │
│  ├─ AtomicWaker                                         15.84 ns      │ 130.7 ns      │ 16.08 ns      │ 18.19 ns      │ 100     │ 12800
│  ├─ DiatomicWaker                                       12.67 ns      │ 112.9 ns      │ 12.74 ns      │ 14.15 ns      │ 100     │ 12800
│  ├─ SpmcWaker                                           11.84 ns      │ 32.3 ns       │ 12.01 ns      │ 12.3 ns       │ 100     │ 25600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      15.23 ns      │ 54.71 ns      │ 15.3 ns       │ 15.7 ns       │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             11.6 ns       │ 69.73 ns      │ 11.7 ns       │ 13.08 ns      │ 100     │ 25600
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    15.03 ns      │ 55.01 ns      │ 15.32 ns      │ 15.7 ns       │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  15.39 ns      │ 130.6 ns      │ 15.44 ns      │ 17.6 ns       │ 100     │ 12800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         11.64 ns      │ 39.57 ns      │ 11.7 ns       │ 12.11 ns      │ 100     │ 25600
├─ wake_cold                                                            │               │               │               │         │
│  ├─ AtomicWaker                                         15.89 ns      │ 131.5 ns      │ 16.08 ns      │ 18.44 ns      │ 100     │ 12800
│  ├─ DiatomicWaker                                       12.69 ns      │ 127.1 ns      │ 12.74 ns      │ 13.88 ns      │ 100     │ 12800
│  ├─ SpmcWaker                                           13.56 ns      │ 70.78 ns      │ 13.9 ns       │ 14.44 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      16.34 ns      │ 208.9 ns      │ 16.59 ns      │ 18.5 ns       │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             13.7 ns       │ 129.5 ns      │ 13.76 ns      │ 15.32 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    14.83 ns      │ 75.15 ns      │ 15.66 ns      │ 16.22 ns      │ 100     │ 12800
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  14.77 ns      │ 116 ns        │ 16.52 ns      │ 17.48 ns      │ 100     │ 12800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         13.73 ns      │ 115.1 ns      │ 13.78 ns      │ 15.3 ns       │ 100     │ 12800
╰─ wake_cold_empty                                                      │               │               │               │         │
   ├─ AtomicWaker                                                       │               │               │               │         │
   │  ├─ t=1                                              14.45 ns      │ 129.8 ns      │ 14.49 ns      │ 17.08 ns      │ 100     │ 12800
   │  ├─ t=2                                              15.06 ns      │ 119.9 ns      │ 27.3 ns       │ 27.77 ns      │ 100     │ 12800
   │  ╰─ t=4                                              18.6 ns       │ 188.4 ns      │ 108.7 ns      │ 97.49 ns      │ 100     │ 1600
   ├─ DiatomicWaker                                                     │               │               │               │         │
   │  ├─ t=1                                              8.55 ns       │ 58 ns         │ 8.825 ns      │ 9.476 ns      │ 100     │ 25600
   │  ├─ t=2                                              12.33 ns      │ 54.13 ns      │ 19.83 ns      │ 19.94 ns      │ 100     │ 25600
   │  ╰─ t=4                                              20.48 ns      │ 3.205 µs      │ 31.1 ns       │ 68.29 ns      │ 100     │ 400
   ├─ SpmcWaker                                                         │               │               │               │         │
   │  ├─ t=1                                              6.206 ns      │ 58.05 ns      │ 6.398 ns      │ 7.433 ns      │ 100     │ 25600
   │  ├─ t=2                                              6.39 ns       │ 6.737 ns      │ 6.484 ns      │ 6.481 ns      │ 100     │ 25600
   │  ╰─ t=4                                              5.413 ns      │ 57.55 ns      │ 6.415 ns      │ 6.719 ns      │ 100     │ 25600
   ├─ SpmcWaker<spmc_waker::sync::Sequential, false>                    │               │               │               │         │
   │  ├─ t=1                                              0.524 ns      │ 6.707 ns      │ 0.527 ns      │ 0.6 ns        │ 100     │ 204800
   │  ├─ t=2                                              0.513 ns      │ 9.004 ns      │ 0.523 ns      │ 0.622 ns      │ 100     │ 204800
   │  ╰─ t=4                                              0.536 ns      │ 7.809 ns      │ 0.9 ns        │ 0.982 ns      │ 100     │ 204800
   ├─ SpmcWaker<spmc_waker::sync::Sequential>                           │               │               │               │         │
   │  ├─ t=1                                              0.513 ns      │ 0.573 ns      │ 0.527 ns      │ 0.533 ns      │ 100     │ 204800
   │  ├─ t=2                                              0.515 ns      │ 7.778 ns      │ 0.519 ns      │ 0.595 ns      │ 100     │ 204800
   │  ╰─ t=4                                              0.548 ns      │ 13.32 ns      │ 0.914 ns      │ 1.043 ns      │ 100     │ 102400
   ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>                  │               │               │               │         │
   │  ├─ t=1                                              6.23 ns       │ 64.65 ns      │ 6.417 ns      │ 8.316 ns      │ 100     │ 25600
   │  ├─ t=2                                              6.198 ns      │ 6.562 ns      │ 6.314 ns      │ 6.315 ns      │ 100     │ 25600
   │  ╰─ t=4                                              6.39 ns       │ 9.132 ns      │ 7.025 ns      │ 7.407 ns      │ 100     │ 25600
   ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>                │               │               │               │         │
   │  ├─ t=1                                              0.512 ns      │ 0.756 ns      │ 0.515 ns      │ 0.533 ns      │ 100     │ 204800
   │  ├─ t=2                                              0.511 ns      │ 4.736 ns      │ 0.517 ns      │ 0.56 ns       │ 100     │ 409600
   │  ╰─ t=4                                              0.534 ns      │ 1.36 ns       │ 0.869 ns      │ 0.907 ns      │ 100     │ 204800
   ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>                       │               │               │               │         │
      ├─ t=1                                              0.525 ns      │ 3.475 ns      │ 0.527 ns      │ 0.565 ns      │ 100     │ 204800
      ├─ t=2                                              0.514 ns      │ 0.545 ns      │ 0.525 ns      │ 0.525 ns      │ 100     │ 204800
      ╰─ t=4                                              0.534 ns      │ 1.359 ns      │ 0.847 ns      │ 0.903 ns      │ 100     │ 204800
```

### aarch64 (Apple M3)

```
comparison                                                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                                             │               │               │               │         │
│  ├─ AtomicWaker                                         2.638 ns      │ 2.821 ns      │ 2.679 ns      │ 2.711 ns      │ 100     │ 204800
│  ├─ DiatomicWaker                                       1 ns          │ 1.356 ns      │ 1.021 ns      │ 1.064 ns      │ 100     │ 409600
│  ├─ SpmcWaker                                           0.629 ns      │ 1.601 ns      │ 0.65 ns       │ 0.71 ns       │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      1.448 ns      │ 2.648 ns      │ 1.499 ns      │ 1.521 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             0.532 ns      │ 0.792 ns      │ 0.548 ns      │ 0.552 ns      │ 100     │ 819200
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    1.458 ns      │ 1.936 ns      │ 1.489 ns      │ 1.513 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  1.438 ns      │ 10.94 ns      │ 1.504 ns      │ 1.765 ns      │ 100     │ 409600
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         0.537 ns      │ 0.695 ns      │ 0.609 ns      │ 0.589 ns      │ 100     │ 819200
├─ register_already_registered                                          │               │               │               │         │
│  ├─ AtomicWaker                                         1.824 ns      │ 2.638 ns      │ 1.845 ns      │ 1.863 ns      │ 100     │ 204800
│  ├─ DiatomicWaker                                       1.102 ns      │ 1.143 ns      │ 1.112 ns      │ 1.117 ns      │ 100     │ 409600
│  ├─ SpmcWaker                                           0.787 ns      │ 0.99 ns       │ 0.807 ns      │ 0.813 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      0.766 ns      │ 0.96 ns       │ 0.777 ns      │ 0.778 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             0.797 ns      │ 0.848 ns      │ 0.802 ns      │ 0.803 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    0.766 ns      │ 1.021 ns      │ 0.787 ns      │ 0.807 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  0.787 ns      │ 2.414 ns      │ 0.817 ns      │ 0.83 ns       │ 100     │ 409600
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         0.787 ns      │ 0.99 ns       │ 0.817 ns      │ 0.826 ns      │ 100     │ 409600
├─ register_overwrite                                                   │               │               │               │         │
│  ├─ AtomicWaker                                         4.021 ns      │ 4.144 ns      │ 4.022 ns      │ 4.037 ns      │ 100     │ 102400
│  ├─ DiatomicWaker                                       3.452 ns      │ 4.388 ns      │ 3.493 ns      │ 3.524 ns      │ 100     │ 204800
│  ├─ SpmcWaker                                           3.94 ns       │ 7.318 ns      │ 4.022 ns      │ 4.145 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      3.208 ns      │ 4.795 ns      │ 3.229 ns      │ 3.29 ns       │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             3.147 ns      │ 6.239 ns      │ 3.239 ns      │ 3.382 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    3.981 ns      │ 4.958 ns      │ 4.225 ns      │ 4.225 ns      │ 100     │ 102400
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  2.903 ns      │ 3.981 ns      │ 2.943 ns      │ 3.103 ns      │ 100     │ 204800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         2.923 ns      │ 6.646 ns      │ 3.401 ns      │ 3.452 ns      │ 100     │ 204800
├─ wake                                                                 │               │               │               │         │
│  ├─ AtomicWaker                                         1.926 ns      │ 7.847 ns      │ 2.496 ns      │ 2.686 ns      │ 100     │ 204800
│  ├─ DiatomicWaker                                       2.414 ns      │ 5.669 ns      │ 2.74 ns       │ 2.795 ns      │ 100     │ 204800
│  ├─ SpmcWaker                                           1.194 ns      │ 3.126 ns      │ 1.245 ns      │ 1.349 ns      │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      2.17 ns       │ 6.626 ns      │ 2.231 ns      │ 2.399 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             1.194 ns      │ 2.18 ns       │ 1.194 ns      │ 1.24 ns       │ 100     │ 409600
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    2.15 ns       │ 2.353 ns      │ 2.19 ns       │ 2.187 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  2.15 ns       │ 4.998 ns      │ 2.19 ns       │ 2.213 ns      │ 100     │ 204800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         1.184 ns      │ 14.92 ns      │ 1.204 ns      │ 1.398 ns      │ 100     │ 409600
├─ wake_cold                                                            │               │               │               │         │
│  ├─ AtomicWaker                                         2.089 ns      │ 5.039 ns      │ 2.109 ns      │ 2.214 ns      │ 100     │ 204800
│  ├─ DiatomicWaker                                       2.597 ns      │ 2.801 ns      │ 2.638 ns      │ 2.632 ns      │ 100     │ 204800
│  ├─ SpmcWaker                                           1.804 ns      │ 1.886 ns      │ 1.845 ns      │ 1.837 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential, false>      2.333 ns      │ 2.516 ns      │ 2.354 ns      │ 2.365 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Sequential>             1.804 ns      │ 1.885 ns      │ 1.845 ns      │ 1.836 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>    2.353 ns      │ 2.821 ns      │ 2.374 ns      │ 2.371 ns      │ 100     │ 204800
│  ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>  2.353 ns      │ 4.327 ns      │ 2.374 ns      │ 2.412 ns      │ 100     │ 204800
│  ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>         1.865 ns      │ 2.333 ns      │ 1.906 ns      │ 1.932 ns      │ 100     │ 204800
╰─ wake_cold_empty                                                      │               │               │               │         │
   ├─ AtomicWaker                                                       │               │               │               │         │
   │  ├─ t=1                                              1.183 ns      │ 1.865 ns      │ 1.194 ns      │ 1.298 ns      │ 100     │ 409600
   │  ├─ t=2                                              1.458 ns      │ 24.04 ns      │ 2.353 ns      │ 5.625 ns      │ 100     │ 102400
   │  ╰─ t=4                                              1.822 ns      │ 54.06 ns      │ 28.35 ns      │ 24.17 ns      │ 100     │ 25600
   ├─ DiatomicWaker                                                     │               │               │               │         │
   │  ├─ t=1                                              1.173 ns      │ 2.638 ns      │ 1.926 ns      │ 1.85 ns       │ 100     │ 204800
   │  ├─ t=2                                              2.231 ns      │ 9.718 ns      │ 6.809 ns      │ 5.861 ns      │ 100     │ 102400
   │  ╰─ t=4                                              1.742 ns      │ 41.53 ns      │ 21.72 ns      │ 21.31 ns      │ 100     │ 51200
   ├─ SpmcWaker                                                         │               │               │               │         │
   │  ├─ t=1                                              1.489 ns      │ 1.601 ns      │ 1.509 ns      │ 1.509 ns      │ 100     │ 409600
   │  ├─ t=2                                              1.947 ns      │ 7.053 ns      │ 3.91 ns       │ 4.024 ns      │ 100     │ 204800
   │  ╰─ t=4                                              2.15 ns       │ 38.28 ns      │ 22.82 ns      │ 21.97 ns      │ 100     │ 51200
   ├─ SpmcWaker<spmc_waker::sync::Sequential, false>                    │               │               │               │         │
   │  ├─ t=1                                              0.12 ns       │ 0.24 ns       │ 0.123 ns      │ 0.129 ns      │ 100     │ 1638400
   │  ├─ t=2                                              0.131 ns      │ 0.273 ns      │ 0.238 ns      │ 0.221 ns      │ 100     │ 819200
   │  ╰─ t=4                                              0.171 ns      │ 0.253 ns      │ 0.243 ns      │ 0.234 ns      │ 100     │ 819200
   ├─ SpmcWaker<spmc_waker::sync::Sequential>                           │               │               │               │         │
   │  ├─ t=1                                              0.126 ns      │ 0.131 ns      │ 0.126 ns      │ 0.127 ns      │ 100     │ 1638400
   │  ├─ t=2                                              0.131 ns      │ 0.248 ns      │ 0.237 ns      │ 0.225 ns      │ 100     │ 1638400
   │  ╰─ t=4                                              0.169 ns      │ 0.248 ns      │ 0.24 ns       │ 0.216 ns      │ 100     │ 1638400
   ├─ SpmcWaker<spmc_waker::sync::Synchronized, false>                  │               │               │               │         │
   │  ├─ t=1                                              1.479 ns      │ 2.191 ns      │ 1.519 ns      │ 1.604 ns      │ 100     │ 204800
   │  ├─ t=2                                              1.702 ns      │ 15.08 ns      │ 2.11 ns       │ 2.932 ns      │ 100     │ 102400
   │  ╰─ t=4                                              1.662 ns      │ 26.56 ns      │ 4.591 ns      │ 8.182 ns      │ 100     │ 25600
   ├─ SpmcWaker<spmc_waker::sync::Unsynchronized, false>                │               │               │               │         │
   │  ├─ t=1                                              0.146 ns      │ 0.894 ns      │ 0.156 ns      │ 0.173 ns      │ 100     │ 819200
   │  ├─ t=2                                              0.161 ns      │ 0.309 ns      │ 0.253 ns      │ 0.234 ns      │ 100     │ 819200
   │  ╰─ t=4                                              0.192 ns      │ 0.314 ns      │ 0.273 ns      │ 0.259 ns      │ 100     │ 819200
   ╰─ SpmcWaker<spmc_waker::sync::Unsynchronized>                       │               │               │               │         │
      ├─ t=1                                              0.146 ns      │ 0.156 ns      │ 0.151 ns      │ 0.152 ns      │ 100     │ 1638400
      ├─ t=2                                              0.192 ns      │ 0.304 ns      │ 0.273 ns      │ 0.265 ns      │ 100     │ 819200
      ╰─ t=4                                              0.192 ns      │ 0.304 ns      │ 0.273 ns      │ 0.259 ns      │ 100     │ 819200```
