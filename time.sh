cargo b --release && time ./target/release/obrc > results.txt
# tr ',' '\n' < results.txt > results_split.txt