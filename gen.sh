for i in {1..8}
do
	cargo run --release -- $1.jpg $i $1-$i.jpg
done
