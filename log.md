# Reference solution
`./target/release/rs-1brc ../measurements.txt > ref.txt  57.07s user 2.79s system 703% cpu 8.507 total`

# My solution progress
* baseline
`./target/release/obrc > results.txt  197.53s user 5.86s system 98% cpu 3:27.02 total`
1. don't compare if min & max can't change
`./target/release/obrc > results.txt  192.05s user 7.51s system 99% cpu 3:21.34 total`
2. store records in a hashmap not btreemap
`./target/release/obrc > results.txt  102.73s user 2.96s system 97% cpu 1:47.93 total`
3. no new string allocations in parsing
`./target/release/obrc > results.txt  86.10s user 3.04s system 98% cpu 1:30.10 total`
4. directly parse 'floats' as integers
`./target/release/obrc > results.txt  80.01s user 3.18s system 98% cpu 1:24.30 total`
5. don't parse strings as utf-8 (until printing them at the very end)
`./target/release/obrc > results.txt  48.90s user 2.55s system 98% cpu 52.407 total`
6. better hash functions (fxhash for now, but should revisit)
`./target/release/obrc > results.txt  42.98s user 2.45s system 98% cpu 46.321 total`