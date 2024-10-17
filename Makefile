bin:	newfs_hammer2 fsck_hammer2 hammer2
newfs_hammer2:
	cargo build --release --bin newfs_hammer2
fsck_hammer2:
	cargo build --release --bin fsck_hammer2
hammer2:
	cargo build --release --bin hammer2
clean:
	cargo clean --release -p hammer2-utils
clean_all:
	cargo clean
fmt:
	cargo fmt
	git status
lint:
	cargo clippy --release --fix --all
	git status
plint:
	cargo clippy --release --fix --all -- -W clippy::pedantic
	git status
test:
	cargo test --release
test_debug:
	cargo test --release -- --nocapture
install:
	cargo install --path .
uninstall:
	cargo uninstall

xxx:	fmt lint test
