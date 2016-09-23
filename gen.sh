for i in {1..8}
do
	cargo run --release -- $1 $i
done
