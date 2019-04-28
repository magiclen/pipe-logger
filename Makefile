all: ./target/x86_64-unknown-linux-musl/release/pipe-logger

./target/x86_64-unknown-linux-musl/release/pipe-logger: $(shell find . -type f -iname '*.rs' -o -name 'Cargo.toml' | sed 's/ /\\ /g')
	LZMA_API_STATIC=1 cargo build --release --target x86_64-unknown-linux-musl
	strip ./target/x86_64-unknown-linux-musl/release/pipe-logger
	
install:
	$(MAKE)
	sudo cp ./target/x86_64-unknown-linux-musl/release/pipe-logger /usr/local/bin/pipe-logger
	sudo chown root. /usr/local/bin/pipe-logger
	sudo chmod 0755 /usr/local/bin/pipe-logger

uninstall:
	sudo rm /usr/local/bin/pipe-logger

test:
	cargo test --verbose

clean:
	cargo clean
