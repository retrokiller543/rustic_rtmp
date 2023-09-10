-include makeconfig

# Default target to run the program in dev mode
.PHONY: all
all: build

# Setup the project

.PHONY: setup
setup:
	@read -p "Enter binary path [default: ./target/release/project_name]: " binary; \
	read -p "Enter package name [default: project-name]: " name; \
	read -p "Enter package version [default: 1.0.0]: " version; \
	read -p "Enter package maintainer [default: Your Name your.email@email.com]: " maintainer; \
	read -p "Enter package description [default: Description of the project]: " description; \
	echo "BINARY_PATH=$${binary:-./target/release/project_name}" > makeconfig; \
	echo "PACKAGE_NAME=$${name:-project-name}" >> makeconfig; \
	echo "PACKAGE_VERSION=$${version:-1.0.0}" >> makeconfig; \
	echo "PACKAGE_MAINTAINER=$${maintainer:-\"Your Name your.email@email.com\"}" >> makeconfig; \
	echo "PACKAGE_DESCRIPTION=$${description:-\"Description of the project\"}" >> makeconfig;

# Packaging

# Ubuntu .deb packaging
.PHONY: package-ubuntu
package-ubuntu:
	@echo "¤ Building package for Ubuntu"
	@mkdir -p build/ubuntu/$(PACKAGE_NAME)/DEBIAN
	@mkdir -p build/ubuntu/$(PACKAGE_NAME)/usr/local/bin
	@cp $(BINARY_PATH) build/ubuntu/$(PACKAGE_NAME)/usr/local/bin/$(PACKAGE_NAME)
	@echo "¤ Creating control file"
	@echo "Package: $(PACKAGE_NAME)" > build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Version: $(PACKAGE_VERSION)" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Section: base" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Priority: optional" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Architecture: amd64" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Maintainer: $(PACKAGE_MAINTAINER)" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Description: $(PACKAGE_DESCRIPTION)" >> build/ubuntu/$(PACKAGE_NAME)/DEBIAN/control
	@echo "¤ Building package"
	@dpkg-deb --build build/ubuntu/$(PACKAGE_NAME)

# macOS .pkg packaging
.PHONY: package-macos
package-macos:
	@echo "¤ Building package for macOS"
	@mkdir -p build/macos/$(PACKAGE_NAME).pkg/usr/local/bin
	@cp $(BINARY_PATH) build/macos/$(PACKAGE_NAME).pkg/usr/local/bin/$(PACKAGE_NAME)
	@pkgbuild --root build/macos/$(PACKAGE_NAME).pkg --identifier com.yourdomain.$(PACKAGE_NAME) --version $(PACKAGE_VERSION) build/macos/$(PACKAGE_NAME).pkg

.PHONY: clean
clean:
	@echo "¤ Cleaning up"
	@rm -rf build/
	@rm -rf target/

# Cargo helpers and shortcuts

.PHONY: build
build:
	@cargo build --release

.PHONY: dev
dev:
	@cargo run

.PHONY: test
test:
	@cargo test

.PHONY: bench
bench:
	@cargo bench

.PHONY: update
update:
	@cargo update

.PHONY: nightly
nightly:
	@rustup default nightly

.PHONY: stable
stable:
	@rustup default stable

.PHONY: clippy
clippy:
	@cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	@cargo fmt

.PHONY: doc
doc:
	@cargo doc --open

.PHONY: check
check:
	@cargo check

.PHONY: loc
loc:
	@cloc --exclude-dir=target .

.PHONY: test-name
test-name:
	@read -p "Enter test name: " name; \
	cargo test $$name

.PHONY: init
init:
	@cargo init

# Cargo Release helpers

.PHONY: release
release:
	@read -p "Enter release type (major/minor/patch): " type; \
	cargo release $$type

run-release:
	@cargo run --release

version:
	@rustc --version


# Git helpers

.PHONY: commit
commit:
	@git add -A
	@read -p "Enter commit message: " message; \
	git commit -m "$$message";

.PHONY: push
push:
	@read -p "Enter branch name (default: main): " branch; \
	git push origin $${branch:-main};

.PHONY: pull
pull:
	@read -p "Enter branch name (default: main): " branch; \
	git pull origin $${branch:-main};

.PHONY: status
status:
	@git status

.PHONY: log
log:
	@git log

.PHONY: new-branch
new-branch:
	@read -p "Enter new branch name: " branch; \
	git checkout -b $$branch

.PHONY: branches
branches:
	@git branch

.PHONY: merge
merge:
	@read -p "Enter branch name to merge: " branch; \
	git merge $$branch
