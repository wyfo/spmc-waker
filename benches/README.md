# Benchmark

This benchmark compares `SpmcWaker<SYNC, CACHED>` with `futures::task::AtomicWaker`; [`diatomic_waker::DiatomicWaker`](https://docs.rs/diatomic-waker/latest/diatomic_waker/struct.DiatomicWaker.html) is also included for completeness.
As expected, `SpmcWaker` is significantly faster than the alternatives in any meaningful scenario.

The following scenarios are measured:
- `register`: register a waker (with potentially the same waker already in cache)
- `register_already_registered`: register a waker which is already registered
- `register_overwrite`: register a waker while another one is already registered
- `wake`: wake a registered waker
- `wake_cold`: wake a registered waker with `wake_cold` for `SpmcWaker` (same as `wake` for others)
- `wake_cold_empty`: call `wake_cold` with no waker registered on `t` threads

The most important gain is on `wake_cold_empty` with `SYNC=false`, which is not surprising as it compiles to a single atomic load. However, `wake_cold` is also read-only (but still pays a fence) with `SYNC=true` on x86, which is why the benchmark shows no contention impact.

Another visible result is the performance gain with `CACHED=true`. In fact, it saves most atomic RMW operations updating the waker's reference count (except in overwrite). `DiatomicWaker` also uses caching, which explains its better numbers than `AtomicWaker`.

Last but not least, `SYNC=false` uses lighter internal synchronization (at the cost of heavier external synchronization), which gives better performance in `wake`.

## Results

### x86_64 (Intel(R) Xeon(R) CPU E3-1275 v5)

```
comparison                      fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                   │               │               │               │         │
│  ├─ AtomicWaker               17.29 ns      │ 179.8 ns      │ 17.85 ns      │ 17.95 ns      │ 11048   │ 1414144
│  ├─ DiatomicWaker             6.565 ns      │ 114.9 ns      │ 6.807 ns      │ 6.823 ns      │ 16051   │ 2054528
│  ├─ SpmcWaker                 4.861 ns      │ 35.31 ns      │ 4.916 ns      │ 4.951 ns      │ 4489    │ 2298368
│  ├─ SpmcWaker<false, false>   9.717 ns      │ 44.13 ns      │ 9.795 ns      │ 9.826 ns      │ 8818    │ 2257408
│  ├─ SpmcWaker<false>          4.883 ns      │ 30.05 ns      │ 4.918 ns      │ 4.933 ns      │ 5231    │ 2678272
│  ╰─ SpmcWaker<true, false>    9.76 ns       │ 28.08 ns      │ 9.803 ns      │ 9.831 ns      │ 8062    │ 2063872
├─ register_already_registered                │               │               │               │         │
│  ├─ AtomicWaker               12.96 ns      │ 70.13 ns      │ 13.03 ns      │ 13.33 ns      │ 9037    │ 2313472
│  ├─ DiatomicWaker             6.676 ns      │ 38.31 ns      │ 6.703 ns      │ 6.799 ns      │ 5984    │ 3063808
│  ├─ SpmcWaker                 1.99 ns       │ 21.14 ns      │ 2.528 ns      │ 2.435 ns      │ 3712    │ 3801088
│  ├─ SpmcWaker<false, false>   1.722 ns      │ 14.68 ns      │ 1.739 ns      │ 1.774 ns      │ 4005    │ 4101120
│  ├─ SpmcWaker<false>          2.248 ns      │ 14.98 ns      │ 2.787 ns      │ 2.789 ns      │ 3670    │ 3758080
│  ╰─ SpmcWaker<true, false>    2.307 ns      │ 435.1 ns      │ 2.651 ns      │ 2.646 ns      │ 109327  │ 3498464
├─ register_overwrite                         │               │               │               │         │
│  ├─ AtomicWaker               37.65 ns      │ 14.58 µs      │ 39.65 ns      │ 40.23 ns      │ 515702  │ 515702
│  ├─ DiatomicWaker             23.18 ns      │ 316.1 ns      │ 24.21 ns      │ 24.31 ns      │ 30051   │ 1923264
│  ├─ SpmcWaker                 23.96 ns      │ 1.024 µs      │ 25.08 ns      │ 25.18 ns      │ 110827  │ 1773232
│  ├─ SpmcWaker<false, false>   21.2 ns       │ 137 ns        │ 21.61 ns      │ 22.06 ns      │ 17283   │ 2212224
│  ├─ SpmcWaker<false>          23.55 ns      │ 260.2 ns      │ 23.97 ns      │ 24.05 ns      │ 31548   │ 2019072
│  ╰─ SpmcWaker<true, false>    21.13 ns      │ 155.9 ns      │ 21.54 ns      │ 21.69 ns      │ 17388   │ 2225664
├─ wake                                       │               │               │               │         │
│  ├─ AtomicWaker               16.52 ns      │ 246.3 ns      │ 17.1 ns       │ 17.21 ns      │ 36462   │ 2333568
│  ├─ DiatomicWaker             12.31 ns      │ 72.02 ns      │ 12.59 ns      │ 12.66 ns      │ 10154   │ 2599424
│  ├─ SpmcWaker                 11.26 ns      │ 70.44 ns      │ 11.3 ns       │ 11.41 ns      │ 10953   │ 2803968
│  ├─ SpmcWaker<false, false>   12.68 ns      │ 470.2 ns      │ 12.9 ns       │ 12.97 ns      │ 95985   │ 3071520
│  ├─ SpmcWaker<false>          5.307 ns      │ 238.5 ns      │ 5.541 ns      │ 5.537 ns      │ 50485   │ 3231040
│  ╰─ SpmcWaker<true, false>    16.19 ns      │ 156.2 ns      │ 16.28 ns      │ 16.35 ns      │ 23253   │ 2976384
├─ wake_cold                                  │               │               │               │         │
│  ├─ AtomicWaker               16.41 ns      │ 199.6 ns      │ 16.94 ns      │ 17.06 ns      │ 18768   │ 2402304
│  ├─ DiatomicWaker             12.38 ns      │ 141.1 ns      │ 12.65 ns      │ 12.71 ns      │ 20032   │ 2564096
│  ├─ SpmcWaker                 12.77 ns      │ 70.14 ns      │ 12.86 ns      │ 12.91 ns      │ 10504   │ 2689024
│  ├─ SpmcWaker<false, false>   11.77 ns      │ 92.02 ns      │ 11.81 ns      │ 11.88 ns      │ 13712   │ 3510272
│  ├─ SpmcWaker<false>          6.67 ns       │ 32.02 ns      │ 6.69 ns       │ 6.715 ns      │ 6365    │ 3258880
│  ╰─ SpmcWaker<true, false>    17.15 ns      │ 140.9 ns      │ 17.39 ns      │ 17.78 ns      │ 22330   │ 2858240
╰─ wake_cold_empty                            │               │               │               │         │
   ├─ AtomicWaker                             │               │               │               │         │
   │  ├─ t=1                    14.31 ns      │ 79.67 ns      │ 14.35 ns      │ 14.43 ns      │ 23732   │ 6075392
   │  ├─ t=2                    14.38 ns      │ 224.4 ns      │ 18.5 ns       │ 19.17 ns      │ 12794   │ 1637632
   │  ╰─ t=4                    15.9 ns       │ 1.014 µs      │ 97.68 ns      │ 86.85 ns      │ 17248   │ 275968
   ├─ DiatomicWaker                           │               │               │               │         │
   │  ├─ t=1                    8.174 ns      │ 86.92 ns      │ 8.307 ns      │ 8.368 ns      │ 38430   │ 9838080
   │  ├─ t=2                    8.338 ns      │ 104 ns        │ 17.4 ns       │ 17.7 ns       │ 10974   │ 2809344
   │  ╰─ t=4                    9.01 ns       │ 159.6 ns      │ 84.66 ns      │ 78.8 ns       │ 15948   │ 1020672
   ├─ SpmcWaker                               │               │               │               │         │
   │  ├─ t=1                    5.252 ns      │ 45.95 ns      │ 6.108 ns      │ 6.149 ns      │ 25193   │ 12898816
   │  ├─ t=2                    4.783 ns      │ 44.35 ns      │ 6.123 ns      │ 6.239 ns      │ 11984   │ 6135808
   │  ╰─ t=4                    4.834 ns      │ 40.32 ns      │ 6.492 ns      │ 6.797 ns      │ 17052   │ 8730624
   ├─ SpmcWaker<false, false>                 │               │               │               │         │
   │  ├─ t=1                    0.428 ns      │ 9.143 ns      │ 0.431 ns      │ 0.436 ns      │ 25953   │ 53151744
   │  ├─ t=2                    0.428 ns      │ 7.642 ns      │ 0.431 ns      │ 0.442 ns      │ 9814    │ 40198144
   │  ╰─ t=4                    0.47 ns       │ 1.233 ns      │ 0.473 ns      │ 0.476 ns      │ 15052   │ 61652992
   ├─ SpmcWaker<false>                        │               │               │               │         │
   │  ├─ t=1                    0.438 ns      │ 20.08 ns      │ 0.441 ns      │ 0.446 ns      │ 50448   │ 51658752
   │  ├─ t=2                    0.428 ns      │ 5.09 ns       │ 0.431 ns      │ 0.452 ns      │ 9650    │ 39526400
   │  ╰─ t=4                    0.448 ns      │ 3.791 ns      │ 0.474 ns      │ 0.69 ns       │ 11968   │ 49020928
   ╰─ SpmcWaker<true, false>                  │               │               │               │         │
      ├─ t=1                    5.192 ns      │ 51.33 ns      │ 6.108 ns      │ 6.136 ns      │ 25273   │ 12939776
      ├─ t=2                    5.033 ns      │ 79.79 ns      │ 6.154 ns      │ 6.245 ns      │ 13528   │ 3463168
      ╰─ t=4                    5.135 ns      │ 234.7 ns      │ 6.838 ns      │ 7.373 ns      │ 18708   │ 1197312
```

### aarch64 (Apple M3)

```
comparison                      fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ register                                   │               │               │               │         │
│  ├─ AtomicWaker               2.679 ns      │ 6.667 ns      │ 2.923 ns      │ 2.849 ns      │ 2578    │ 5279744
│  ├─ DiatomicWaker             1.113 ns      │ 2.079 ns      │ 1.164 ns      │ 1.167 ns      │ 2482    │ 10166272
│  ├─ SpmcWaker                 0.512 ns      │ 0.762 ns      │ 0.533 ns      │ 0.536 ns      │ 1613    │ 13213696
│  ├─ SpmcWaker<false, false>   1.458 ns      │ 2.73 ns       │ 1.53 ns       │ 1.536 ns      │ 1911    │ 7827456
│  ├─ SpmcWaker<false>          0.284 ns      │ 1.397 ns      │ 0.339 ns      │ 0.335 ns      │ 1466    │ 12009472
│  ╰─ SpmcWaker<true, false>    1.468 ns      │ 11.02 ns      │ 1.611 ns      │ 1.619 ns      │ 2852    │ 11681792
├─ register_already_registered                │               │               │               │         │
│  ├─ AtomicWaker               1.845 ns      │ 5.914 ns      │ 1.886 ns      │ 1.893 ns      │ 5550    │ 11366400
│  ├─ DiatomicWaker             1.133 ns      │ 2.771 ns      │ 1.184 ns      │ 1.185 ns      │ 3278    │ 13426688
│  ├─ SpmcWaker                 1.326 ns      │ 3.442 ns      │ 1.346 ns      │ 1.347 ns      │ 3239    │ 13266944
│  ├─ SpmcWaker<false, false>   0.797 ns      │ 1.275 ns      │ 0.818 ns      │ 0.818 ns      │ 4691    │ 19214336
│  ├─ SpmcWaker<false>          1.214 ns      │ 7.776 ns      │ 1.235 ns      │ 1.275 ns      │ 3510    │ 14376960
│  ╰─ SpmcWaker<true, false>    0.731 ns      │ 1.703 ns      │ 0.812 ns      │ 0.815 ns      │ 2346    │ 19218432
├─ register_overwrite                         │               │               │               │         │
│  ├─ AtomicWaker               3.94 ns       │ 12.93 ns      │ 4.022 ns      │ 4.03 ns       │ 8942    │ 9156608
│  ├─ DiatomicWaker             3.432 ns      │ 6.26 ns       │ 3.473 ns      │ 3.587 ns      │ 5053    │ 10348544
│  ├─ SpmcWaker                 3.432 ns      │ 10 ns         │ 3.737 ns      │ 3.725 ns      │ 4960    │ 10158080
│  ├─ SpmcWaker<false, false>   2.7 ns        │ 17.26 ns      │ 2.944 ns      │ 2.945 ns      │ 6892    │ 14114816
│  ├─ SpmcWaker<false>          2.943 ns      │ 16.79 ns      │ 3.208 ns      │ 3.178 ns      │ 5414    │ 11087872
│  ╰─ SpmcWaker<true, false>    3.452 ns      │ 10.98 ns      │ 3.534 ns      │ 3.524 ns      │ 12325   │ 12620800
├─ wake                                       │               │               │               │         │
│  ├─ AtomicWaker               1.947 ns      │ 5.649 ns      │ 2.15 ns       │ 2.333 ns      │ 6450    │ 13209600
│  ├─ DiatomicWaker             2.374 ns      │ 5.731 ns      │ 2.415 ns      │ 2.413 ns      │ 5803    │ 11884544
│  ├─ SpmcWaker                 1.214 ns      │ 2.679 ns      │ 1.235 ns      │ 1.283 ns      │ 3374    │ 13819904
│  ├─ SpmcWaker<false, false>   1.469 ns      │ 5.07 ns       │ 1.632 ns      │ 1.631 ns      │ 5646    │ 23126016
│  ├─ SpmcWaker<false>          0.726 ns      │ 1.906 ns      │ 0.818 ns      │ 0.817 ns      │ 3632    │ 14876672
│  ╰─ SpmcWaker<true, false>    1.967 ns      │ 7.379 ns      │ 2.476 ns      │ 2.403 ns      │ 9401    │ 19253248
├─ wake_cold                                  │               │               │               │         │
│  ├─ AtomicWaker               1.947 ns      │ 5.527 ns      │ 2.191 ns      │ 2.38 ns       │ 6393    │ 13092864
│  ├─ DiatomicWaker             2.191 ns      │ 8.111 ns      │ 2.415 ns      │ 2.413 ns      │ 5804    │ 11886592
│  ├─ SpmcWaker                 1.703 ns      │ 6.24 ns       │ 1.886 ns      │ 1.87 ns       │ 6086    │ 12464128
│  ├─ SpmcWaker<false, false>   1.591 ns      │ 3.523 ns      │ 1.631 ns      │ 1.629 ns      │ 5667    │ 23212032
│  ├─ SpmcWaker<false>          1.062 ns      │ 2.364 ns      │ 1.082 ns      │ 1.084 ns      │ 3485    │ 14274560
│  ╰─ SpmcWaker<true, false>    2.109 ns      │ 7.053 ns      │ 2.15 ns       │ 2.148 ns      │ 9847    │ 20166656
╰─ wake_cold_empty                            │               │               │               │         │
   ├─ AtomicWaker                             │               │               │               │         │
   │  ├─ t=1                    1.214 ns      │ 4.551 ns      │ 1.723 ns      │ 1.521 ns      │ 8783    │ 35975168
   │  ├─ t=2                    1.397 ns      │ 23.59 ns      │ 6.097 ns      │ 5.365 ns      │ 7594    │ 15552512
   │  ╰─ t=4                    1.375 ns      │ 88.28 ns      │ 3 ns          │ 13.25 ns      │ 19376   │ 2480128
   ├─ DiatomicWaker                           │               │               │               │         │
   │  ├─ t=1                    1.214 ns      │ 4.754 ns      │ 1.356 ns      │ 1.338 ns      │ 18332   │ 37543936
   │  ├─ t=2                    1.458 ns      │ 17.65 ns      │ 2.638 ns      │ 3.059 ns      │ 13662   │ 13989888
   │  ╰─ t=4                    1.375 ns      │ 89.59 ns      │ 15.05 ns      │ 16.17 ns      │ 15920   │ 4075520
   ├─ SpmcWaker                               │               │               │               │         │
   │  ├─ t=1                    1.53 ns       │ 3.534 ns      │ 1.55 ns       │ 1.611 ns      │ 8425    │ 34508800
   │  ├─ t=2                    1.581 ns      │ 11.38 ns      │ 2.15 ns       │ 2.417 ns      │ 14032   │ 14368768
   │  ╰─ t=4                    1.539 ns      │ 43.69 ns      │ 10.98 ns      │ 13.4 ns       │ 17256   │ 4417536
   ├─ SpmcWaker<false, false>                 │               │               │               │         │
   │  ├─ t=1                    0.161 ns      │ 0.736 ns      │ 0.169 ns      │ 0.174 ns      │ 4270    │ 69959680
   │  ├─ t=2                    0.166 ns      │ 0.945 ns      │ 0.284 ns      │ 0.271 ns      │ 7680    │ 62914560
   │  ╰─ t=4                    0.207 ns      │ 2.071 ns      │ 0.281 ns      │ 0.256 ns      │ 8380    │ 137297920
   ├─ SpmcWaker<false>                        │               │               │               │         │
   │  ├─ t=1                    0.161 ns      │ 0.876 ns      │ 0.169 ns      │ 0.183 ns      │ 4182    │ 68517888
   │  ├─ t=2                    0.172 ns      │ 3.671 ns      │ 0.284 ns      │ 0.271 ns      │ 7664    │ 62783488
   │  ╰─ t=4                    0.202 ns      │ 1.306 ns      │ 0.284 ns      │ 0.278 ns      │ 11744   │ 96206848
   ╰─ SpmcWaker<true, false>                  │               │               │               │         │
      ├─ t=1                    1.52 ns       │ 5.884 ns      │ 1.55 ns       │ 1.565 ns      │ 8642    │ 35397632
      ├─ t=2                    1.662 ns      │ 11.32 ns      │ 3.981 ns      │ 3.991 ns      │ 8696    │ 17809408
      ╰─ t=4                    1.542 ns      │ 44.01 ns      │ 10.65 ns      │ 13.03 ns      │ 17448   │ 4466688
```
