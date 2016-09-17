for i in {8..16}
do
	cargo run --release -- $1.jpg $i $1-$i.jpg
done
