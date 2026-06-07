# Benchmark

This benchmark compares `SpmcWaker<SYNC, CACHED>` with `futures::task::AtomicWaker`; [`diatomic_waker::DiatomicWaker`](https://docs.rs/diatomic-waker/latest/diatomic_waker/struct.DiatomicWaker.html) is also included for completeness. It has been run on an Intel(R) Xeon(R) CPU E3-1275 v5.

## Results

```
Timer precision: 15 ns
comparison                     fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                  │               │               │               │         │
│  ├─ AtomicWaker              12.69 ns      │ 12.76 ns      │ 12.73 ns      │ 12.73 ns      │ 100     │ 12800
│  ├─ DiatomicWaker            10.84 ns      │ 68.77 ns      │ 10.88 ns      │ 11.46 ns      │ 100     │ 25600
│  ├─ SpmcWaker                2.337 ns      │ 16.6 ns       │ 2.345 ns      │ 2.682 ns      │ 100     │ 102400
│  ├─ SpmcWaker<false, false>  2.209 ns      │ 16.71 ns      │ 2.217 ns      │ 2.566 ns      │ 100     │ 102400
│  ├─ SpmcWaker<true>          2.592 ns      │ 16.63 ns      │ 2.602 ns      │ 2.947 ns      │ 100     │ 102400
│  ╰─ SpmcWaker<true, false>   2.095 ns      │ 27.93 ns      │ 2.176 ns      │ 3.376 ns      │ 100     │ 102400
├─ register_overwrite                        │               │               │               │         │
│  ├─ AtomicWaker              47.14 ns      │ 491 ns        │ 48.52 ns      │ 58.75 ns      │ 100     │ 3200
│  ├─ DiatomicWaker            58.33 ns      │ 217.5 ns      │ 58.55 ns      │ 60.28 ns      │ 100     │ 3200
│  ├─ SpmcWaker                38.03 ns      │ 123 ns        │ 39.75 ns      │ 40.53 ns      │ 100     │ 6400
│  ├─ SpmcWaker<false, false>  37.72 ns      │ 120.4 ns      │ 39.22 ns      │ 40.66 ns      │ 100     │ 6400
│  ├─ SpmcWaker<false>         38.13 ns      │ 183.5 ns      │ 39.69 ns      │ 41.95 ns      │ 100     │ 6400
│  ╰─ SpmcWaker<true, false>   38.45 ns      │ 121.7 ns      │ 39.22 ns      │ 40.05 ns      │ 100     │ 6400
├─ register_wake                             │               │               │               │         │
│  ├─ AtomicWaker              38.41 ns      │ 162.5 ns      │ 38.5 ns       │ 40.58 ns      │ 100     │ 6400
│  ├─ DiatomicWaker            39.81 ns      │ 121.1 ns      │ 39.94 ns      │ 40.75 ns      │ 100     │ 6400
│  ├─ SpmcWaker                32.74 ns      │ 175.3 ns      │ 32.83 ns      │ 35.06 ns      │ 100     │ 6400
│  ├─ SpmcWaker<false, false>  31.13 ns      │ 112.5 ns      │ 32.34 ns      │ 33.13 ns      │ 100     │ 6400
│  ├─ SpmcWaker<false>         30.58 ns      │ 109.9 ns      │ 31.39 ns      │ 32.61 ns      │ 100     │ 6400
│  ╰─ SpmcWaker<true, false>   32.2 ns       │ 181.8 ns      │ 32.28 ns      │ 33.79 ns      │ 100     │ 6400
├─ wake_empty                                │               │               │               │         │
│  ├─ AtomicWaker                            │               │               │               │         │
│  │  ├─ t=1                   13.38 ns      │ 53.5 ns       │ 14.09 ns      │ 14.52 ns      │ 100     │ 12800
│  │  ├─ t=2                   13.99 ns      │ 21.41 ns      │ 15.14 ns      │ 15.58 ns      │ 100     │ 12800
│  │  ╰─ t=4                   17.05 ns      │ 181.8 ns      │ 33.3 ns       │ 52.52 ns      │ 100     │ 1600
│  ├─ DiatomicWaker                          │               │               │               │         │
│  │  ├─ t=1                   10.48 ns      │ 12.29 ns      │ 10.52 ns      │ 10.61 ns      │ 100     │ 25600
│  │  ├─ t=2                   18.75 ns      │ 35.14 ns      │ 22.63 ns      │ 22.98 ns      │ 100     │ 25600
│  │  ╰─ t=4                   9.427 ns      │ 89.74 ns      │ 39.24 ns      │ 42.48 ns      │ 100     │ 6400
│  ├─ SpmcWaker                              │               │               │               │         │
│  │  ├─ t=1                   4.646 ns      │ 11.82 ns      │ 4.66 ns       │ 4.78 ns       │ 100     │ 51200
│  │  ├─ t=2                   4.644 ns      │ 4.769 ns      │ 4.695 ns      │ 4.697 ns      │ 100     │ 51200
│  │  ╰─ t=4                   4.532 ns      │ 13.41 ns      │ 5.147 ns      │ 5.864 ns      │ 100     │ 51200
│  ├─ SpmcWaker<false, false>                │               │               │               │         │
│  │  ├─ t=1                   0.63 ns       │ 115.2 ns      │ 0.661 ns      │ 1.814 ns      │ 100     │ 12800
│  │  ├─ t=2                   0.144 ns      │ 3.21 ns       │ 0.151 ns      │ 0.183 ns      │ 100     │ 409600
│  │  ╰─ t=4                   0.154 ns      │ 0.565 ns      │ 0.328 ns      │ 0.343 ns      │ 100     │ 409600
│  ├─ SpmcWaker<false>                       │               │               │               │         │
│  │  ├─ t=1                   0.264 ns      │ 0.327 ns      │ 0.265 ns      │ 0.268 ns      │ 100     │ 409600
│  │  ├─ t=2                   0.257 ns      │ 0.281 ns      │ 0.265 ns      │ 0.266 ns      │ 100     │ 409600
│  │  ╰─ t=4                   0.276 ns      │ 0.861 ns      │ 0.539 ns      │ 0.552 ns      │ 100     │ 204800
│  ╰─ SpmcWaker<true, false>                 │               │               │               │         │
│     ├─ t=1                   4.804 ns      │ 30.46 ns      │ 6.365 ns      │ 6.162 ns      │ 100     │ 51200
│     ├─ t=2                   4.661 ns      │ 6.466 ns      │ 6.273 ns      │ 6.27 ns       │ 100     │ 25600
│     ╰─ t=4                   5.384 ns      │ 11.22 ns      │ 6.177 ns      │ 6.309 ns      │ 100     │ 25600
╰─ wake_empty_spin                           │               │               │               │         │
   ├─ AtomicWaker                            │               │               │               │         │
   │  ├─ t=1                   50.61 ns      │ 458.9 ns      │ 56.58 ns      │ 59.48 ns      │ 100     │ 3200
   │  ├─ t=2                   52.83 ns      │ 340.8 ns      │ 55.28 ns      │ 58.26 ns      │ 100     │ 3200
   │  ╰─ t=4                   53.36 ns      │ 191.1 ns      │ 113.1 ns      │ 107.3 ns      │ 100     │ 1600
   ├─ DiatomicWaker                          │               │               │               │         │
   │  ├─ t=1                   51.24 ns      │ 514.9 ns      │ 52.92 ns      │ 63.26 ns      │ 100     │ 3200
   │  ├─ t=2                   48.8 ns       │ 53.95 ns      │ 51.66 ns      │ 51.39 ns      │ 100     │ 3200
   │  ╰─ t=4                   47.3 ns       │ 118.8 ns      │ 85.88 ns      │ 81.17 ns      │ 100     │ 3200
   ├─ SpmcWaker                              │               │               │               │         │
   │  ├─ t=1                   56.24 ns      │ 3.709 µs      │ 57.74 ns      │ 94.5 ns       │ 100     │ 400
   │  ├─ t=2                   45.34 ns      │ 46.38 ns      │ 45.68 ns      │ 45.7 ns       │ 100     │ 6400
   │  ╰─ t=4                   45.7 ns       │ 55.99 ns      │ 46.78 ns      │ 47.92 ns      │ 100     │ 3200
   ├─ SpmcWaker<false, false>                │               │               │               │         │
   │  ├─ t=1                   41.03 ns      │ 236.6 ns      │ 41.16 ns      │ 44.14 ns      │ 100     │ 6400
   │  ├─ t=2                   40.92 ns      │ 226.7 ns      │ 41.67 ns      │ 43.44 ns      │ 100     │ 3200
   │  ╰─ t=4                   40.88 ns      │ 98.42 ns      │ 41.48 ns      │ 43.29 ns      │ 100     │ 6400
   ├─ SpmcWaker<false>                       │               │               │               │         │
   │  ├─ t=1                   40.66 ns      │ 247.2 ns      │ 41.59 ns      │ 43.42 ns      │ 100     │ 6400
   │  ├─ t=2                   40.67 ns      │ 41.75 ns      │ 41.1 ns       │ 41.14 ns      │ 100     │ 6400
   │  ╰─ t=4                   40.77 ns      │ 46.38 ns      │ 41.49 ns      │ 42.73 ns      │ 100     │ 6400
   ╰─ SpmcWaker<true, false>                 │               │               │               │         │
      ├─ t=1                   45.8 ns       │ 457 ns        │ 47.74 ns      │ 51.66 ns      │ 100     │ 3200
      ├─ t=2                   45.33 ns      │ 189.1 ns      │ 45.7 ns       │ 47.96 ns      │ 100     │ 6400
      ╰─ t=4                   45.61 ns      │ 56.58 ns      │ 46.8 ns       │ 48.05 ns      │ 100     │ 32000
```
