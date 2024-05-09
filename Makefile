build:
	cargo build --release

run: build
	./target/release/memvecdb

clean:
	cargo clean