.PHONY: build
build:
	cargo build -p libdrbug -p drb

.PHONY: test rust-test-targets asm-test-targets
test: rust-test-targets asm-test-targets
	cargo nextest run

rust-test-targets:
	cargo build -p test-targets

asm-test-targets:
	mkdir -p target/asm
	gcc -o target/asm/reg_write test/asm/reg_write.s -pie
	gcc -o target/asm/reg_read test/asm/reg_read.s -pie
