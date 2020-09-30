VERSION=1.0.0

all: debug

clean:
	cargo clean

debug:
	cargo build

tag:
	git tag -a v${VERSION}
	git push origin --tags

pub:
	@# internal
	jks build modbusgw

ver:
	sed -i 's/^version = ".*/version = "${VERSION}"/g' Cargo.toml
	sed -i 's/^const VERSION.*/const VERSION: \&str = "${VERSION}";/g' src/main.rs

release: prepare-targets release_x86_64 release_armhf release_win64 check-binaries

prepare-targets:
	./.dev/prepare-targets.sh

release_x86_64:
	cargo build --target x86_64-unknown-linux-musl --release

release_armhf:
	cargo build --target arm-unknown-linux-musleabihf --release

release_win64:
	cargo build --target x86_64-pc-windows-gnu --release

check-binaries:
	./.dev/check-binaries.sh

release-upload: release-upload-x86_64 release-upload-arm release-upload-win64

release-upload-x86_64:
	cd ./target/x86_64-unknown-linux-musl/release && \
		tar --owner=root --group=root -cvf /tmp/modbusgw.linux-x86_64-musl.tar modbusgw
	gzip /tmp/modbusgw.linux-x86_64-musl.tar
	./.dev/release-upload.sh modbusgw.linux-x86_64-musl.tar.gz
	rm /tmp/modbusgw.linux-x86_64-musl.tar.gz

release-upload-arm:
	cd ./target/arm-unknown-linux-musleabihf/release && \
		tar --owner=root --group=root -cvf /tmp/modbusgw.linux-arm-musleabihf.tar modbusgw
	gzip /tmp/modbusgw.linux-arm-musleabihf.tar
	./.dev/release-upload.sh modbusgw.linux-arm-musleabihf.tar.gz
	rm /tmp/modbusgw.linux-arm-musleabihf.tar.gz

release-upload-win64:
	rm -f /tmp/modbusgw.windows-x86_64.zip
	cd ./target/x86_64-pc-windows-gnu/release && \
		zip /tmp/modbusgw.windows-x86_64.zip modbusgw.exe
	./.dev/release-upload.sh modbusgw.windows-x86_64.zip
	rm /tmp/modbusgw.windows-x86_64.zip
