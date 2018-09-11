all: ./target/release/pipe-logger

./target/release/pipe-logger: $(shell find . -type f -iname '*.rs' -o -name 'Cargo.toml' | sed 's/ /\\ /g')
	LZMA_API_STATIC=1 cargo build --release
	strip ./target/release/pipe-logger
	
install:
	$(MAKE)
	sudo cp ./target/release/pipe-logger /usr/local/bin/pipe-logger
	sudo chown root. /usr/local/bin/pipe-logger
	sudo chmod 0755 /usr/local/bin/pipe-logger
	
test:
	cargo test --verbose

clean:
	cargo clean
