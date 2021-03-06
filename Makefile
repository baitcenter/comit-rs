RUSTUP = rustup
ECHO = echo

TOOLCHAIN = $(shell cat rust-toolchain)
CARGO = $(RUSTUP) run --install $(TOOLCHAIN) cargo --color always

NIGHTLY_TOOLCHAIN = nightly-2020-01-15
CARGO_NIGHTLY = $(RUSTUP) run --install $(NIGHTLY_TOOLCHAIN) cargo --color always

GIT_HOOKS_PATH = ".githooks"
GIT_HOOKS = $(wildcard $(GIT_HOOKS_PATH)/*)

INSTALLED_TOOLCHAINS = $(shell $(RUSTUP) toolchain list)
INSTALLED_COMPONENTS = $(shell $(RUSTUP) component list --installed --toolchain $(TOOLCHAIN))
INSTALLED_NIGHTLY_COMPONENTS = $(shell $(RUSTUP) component list --installed --toolchain $(NIGHTLY_TOOLCHAIN))
AVAILABLE_CARGO_COMMANDS = $(shell $(CARGO) --list)

# All our targets go into .PHONY because none of them actually create files
.PHONY: init_git_hooks default install_rust install_rust_nightly install_clippy install_rustfmt install_tomlfmt install clean all ci build clippy test doc e2e check_format format check_rust_format check_toml_format check_ts_format

default: init_git_hooks build format

init_git_hooks:
	git config core.hooksPath $(GIT_HOOKS_PATH)

## Dev environment

install_rust:
ifeq (,$(findstring $(TOOLCHAIN),$(INSTALLED_TOOLCHAINS)))
	$(RUSTUP) install $(TOOLCHAIN)
endif

install_rust_nightly:
ifeq (,$(findstring $(NIGHTLY_TOOLCHAIN),$(INSTALLED_TOOLCHAINS)))
	$(RUSTUP) install $(NIGHTLY_TOOLCHAIN)
endif

install_clippy: install_rust
ifeq (,$(findstring clippy,$(INSTALLED_COMPONENTS)))
	$(RUSTUP) component add clippy --toolchain $(TOOLCHAIN)
endif

install_rustfmt: install_rust_nightly
ifeq (,$(findstring rustfmt,$(INSTALLED_NIGHTLY_COMPONENTS)))
	$(RUSTUP) component add rustfmt --toolchain $(NIGHTLY_TOOLCHAIN)
endif

install_tomlfmt: install_rust
ifeq (,$(findstring tomlfmt,$(AVAILABLE_CARGO_COMMANDS)))
	$(CARGO) install cargo-tomlfmt
endif

## User install

download_bc_nodes:
	@./blockchain_nodes/ensure_bitcoind.sh
	@./blockchain_nodes/ensure_parity.sh

install:
	$(CARGO) install --force --path cnd

clean:
	$(CARGO) clean

## Development tasks

all: format build clippy test doc e2e

ci: check_format doc clippy test build e2e

build:
	$(CARGO) build --all --all-targets $(BUILD_ARGS)

clippy: install_clippy
	$(CARGO) clippy --all-targets -- -D warnings

test:
	$(CARGO) test --all

doc:
	$(CARGO) doc

e2e: download_bc_nodes build
	(cd ./api_tests; yarn install; yarn test)

check_format: check_rust_format check_toml_format check_ts_format

MODIFIED_FILES = $(shell git status --untracked-files=no --short)
MODIFIED_RUST_FILES = $(filter %.rs,$(MODIFIED_FILES))
MODIFIED_TOML_FILES = $(filter %.toml,$(MODIFIED_FILES))
MODIFIED_TYPESCRIPT_FILES = $(filter %.ts %.json %.yml,$(MODIFIED_FILES))

format: install_rustfmt install_tomlfmt
ifneq (,$(MODIFIED_RUST_FILES))
	$(CARGO_NIGHTLY) fmt -- --files-with-diff | xargs -I{} git add {}
endif
ifneq (,$(MODIFIED_TOML_FILES))
	$(CARGO) tomlfmt -p Cargo.toml && git add Cargo.toml
	$(CARGO) tomlfmt -p cnd/Cargo.toml && git add cnd/Cargo.toml
	$(CARGO) tomlfmt -p libp2p-comit/Cargo.toml && git add libp2p-comit/Cargo.toml
endif
ifneq (,$(MODIFIED_TYPESCRIPT_FILES))
	(cd ./api_tests; yarn install; yarn run fix)
endif

STAGED_FILES = $(shell git diff --staged --name-only)
STAGED_RUST_FILES = $(filter %.rs,$(STAGED_FILES))
STAGED_TOML_FILES = $(filter %.toml,$(STAGED_FILES))
STAGED_TYPESCRIPT_FILES = $(filter %.ts %.json %.yml,$(STAGED_FILES))

check_rust_format: install_rustfmt
ifneq (,$(STAGED_RUST_FILES))
	$(CARGO_NIGHTLY) fmt -- --check
endif

check_toml_format: install_tomlfmt
ifneq (,$(STAGED_TOML_FILES))
	$(CARGO) tomlfmt -d -p Cargo.toml
	$(CARGO) tomlfmt -d -p cnd/Cargo.toml
	$(CARGO) tomlfmt -d -p libp2p-comit/Cargo.toml
endif

check_ts_format:
ifneq (,$(STAGED_TYPESCRIPT_FILES))
	(cd ./api_tests; yarn install; yarn run check)
endif
