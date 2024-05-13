build:
	cargo build --release --verbose

run: build
	./target/release/memvectordb

clean:
	cargo clean

test:
	cargo test