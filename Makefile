build:
	cargo build --release --verbose

run: build
	./target/release/memvectordb

clean:
	cargo clean

test:
	cargo test

load_test_post:
	oha -n 100000 -m POST -d '{"collection_name": "collection1", "dimension": 3, "distance": "dot"}' http://127.0.0.1:8000/create_collection

load_test_get:
	oha -n 100000 -m GET -d '{"collection_name": "collection1"}' http://127.0.0.1:8000/get_collection

load_test_put:
	oha -n 100000 -m PUT -d '{"collection_name": "collection1", "embedding": {"id" : "1", "vector" :[0.14, 0.316, 0.433], "metadata": {"key1": "value1", "key2": "value2"}}}' http://127.0.0.1:8000/insert_embeddings

load_test_delete:
	oha -n 100000 -m DELETE -d '{"collection_name": "collection1"}' http://127.0.0.1:8000/delete_collection
