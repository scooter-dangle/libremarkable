all: library examples

.PHONY: examples
examples:
	cargo build --examples --release --target=armv7-unknown-linux-gnueabihf

bench:
	cargo build --examples --release --target=armv7-unknown-linux-gnueabihf --features "enable-runtime-benchmarking"

library:
	cargo build --release --target=armv7-unknown-linux-gnueabihf

test:
	# Notice we aren't using the armv7 target here
	cargo test

.PHONY: docker
docker:
	docker build --tag remarkable.rs .

./target/armv7-unknown-linux-gnueabihf/release/examples/demo: examples/demo.rs docker
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs cargo build --example $(@F) --release --target=armv7-unknown-linux-gnueabihf
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs arm-linux-gnueabihf-strip $@

./target/armv7-unknown-linux-gnueabihf/release/examples/demo-modified: examples/demo-modified.rs docker
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs cargo build --example $(@F) --release --target=armv7-unknown-linux-gnueabihf
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs arm-linux-gnueabihf-strip $@

./target/armv7-unknown-linux-gnueabihf/release/examples/live: examples/live.rs docker
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs cargo build --example $(@F) --release --target=armv7-unknown-linux-gnueabihf
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs arm-linux-gnueabihf-strip $@

./target/armv7-unknown-linux-gnueabihf/release/examples/libspy.so: examples/spy.rs docker
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs cargo build --example spy --release --target=armv7-unknown-linux-gnueabihf
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs arm-linux-gnueabihf-strip $@

docker-test: docker
	docker run --rm --interactive --tty --workdir /src --volume $(PWD):/src remarkable.rs cargo test

DEVICE_IP ?= "10.11.99.1"
run: ./target/armv7-unknown-linux-gnueabihf/release/examples/demo
	ssh root@$(DEVICE_IP) 'kill -9 `pidof demo` || true; systemctl stop xochitl || true'
	scp $< root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) './demo'

run-modified: ./target/armv7-unknown-linux-gnueabihf/release/examples/demo-modified
	ssh root@$(DEVICE_IP) 'kill -9 `pidof demo-modified` || true; systemctl stop xochitl || true'
	scp $< root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) './demo-modified'

live: ./target/armv7-unknown-linux-gnueabihf/release/examples/live
	ssh root@$(DEVICE_IP) 'kill -9 `pidof live` || true'
	scp $< root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) './live'

run-bench: bench
	ssh root@$(DEVICE_IP) 'kill -9 `pidof demo` || true; systemctl stop xochitl || true'
	scp ./target/armv7-unknown-linux-gnueabihf/release/examples/demo root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) './demo'

spy-xochitl: ./target/armv7-unknown-linux-gnueabihf/release/examples/libspy.so
	ssh root@$(DEVICE_IP) 'systemctl stop xochitl'
	scp $< root@$(DEVICE_IP):~/
	ssh root@$(DEVICE_IP) 'LD_PRELOAD="/home/root/libspy.so" xochitl'

start-xochitl:
	ssh root@$(DEVICE_IP) 'kill -9 `pidof demo` || true; systemctl start xochitl'
	
