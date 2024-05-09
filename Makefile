build:
	cargo build --release --verbose

run: build
	./target/release/memvecdb

clean:
	cargo clean