# Git revisions for external dependencies
BOOTLOADER_HINTS_REV ?= 5648cf0a5a2574c2870151cd178ff3ae4b141824
STWO_REV ?= d09a2cfd2b308b6210906f26320f767cf279abef
CAIRO_EXECUTE_REV ?= 7fbbd0112b5a926403c17fa95ad831c1715fd1b1
################################## CLIENT ##################################

client-build:
	scarb --profile release build --package client --target-kinds executable

client-build-with-shinigami:
	scarb --profile release build --package client --target-kinds executable --features shinigami

################################## BINARIES ##################################

install-bootloader-hints:
	cargo install \
		--git ssh://git@github.com/starkware-libs/bootloader-hints.git \
		--rev $(BOOTLOADER_HINTS_REV) \
		cairo-program-runner

install-stwo:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install --force \
		--git https://github.com/starkware-libs/stwo-cairo \
		--rev $(STWO_REV) \
		adapted_stwo

install-cairo-execute:
	cargo install --git https://github.com/m-kus/cairo \
		--rev $(CAIRO_EXECUTE_REV) cairo-execute

install-scarb-eject:
	cargo install --git \
		https://github.com/software-mansion-labs/scarb-eject

install-convert-proof-format:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install --force \
		--git https://github.com/starkware-libs/stwo-cairo \
		dev-utils

install-corelib:
	mkdir -p vendor
	rm -rf vendor/cairo
	git clone --single-branch --branch m-kus/system-builtin \
		https://github.com/m-kus/cairo vendor/cairo
	(cd vendor/cairo && git checkout $(CAIRO_EXECUTE_REV))
	ln -s "$(CURDIR)/vendor/cairo/corelib" \
		packages/assumevalid/corelib

install: install-bootloader-hints install-stwo install-cairo-execute \
	install-convert-proof-format install-scarb-eject install-corelib

################################## ASSUMEVALID ##################################

assumevalid-build:
	scarb --profile proving build --package assumevalid \
		--no-default-features


assumevalid-eject:
	scarb-eject --package assumevalid \
		--output packages/assumevalid/cairo_project.toml

assumevalid-build-with-syscalls:
	cd packages/assumevalid && \
	cairo-execute \
		--build-only \
		--output-path \
			../../target/proving/assumevalid.executable.json \
		--executable assumevalid::main \
		--ignore-warnings \
		--allow-syscalls .


################################## PIPELINE ##################################

build-simple-bootloader:
	. .venv/bin/activate && cd ../starkware && cairo-compile \
		--proof_mode \
		src/starkware/cairo/bootloaders/simple_bootloader/\
			simple_bootloader.cairo \
		--cairo_path src --output \
			$(CURDIR)/bootloaders/simple_bootloader_compiled.json

setup: install-system-packages create-venv install-python-dependencies

install-system-packages:
	@echo ">>> Updating apt package list and installing system-level Python packages..."
	sudo apt update
	sudo apt install -y python3-pip python3.11-venv \
		# Use -y for non-interactive install

create-venv:
	@echo ">>> Creating Python virtual environment '.venv'..."
	python3 -m venv .venv

install-python-dependencies:
	@echo "Installing Python dependencies into the 'venv' virtual environment..."

	. .venv/bin/activate && pip install \
		-r scripts/data/requirements.txt

data-generate-timestamp:
	@echo ">>> Generating timestamp data..."
	. .venv/bin/activate && cd scripts/data && \
		python generate_timestamp_data.py

data-generate-utxo:
	@echo ">>> Generating UTXO data..."
	. .venv/bin/activate && cd scripts/data && \
		python generate_utxo_data.py

prove-pow:
	@echo ">>> Prove POW..."
	. .venv/bin/activate && cd scripts/data && python prove_pow.py \
		$(if $(START),--start $(START)) \
		--blocks $(or $(BLOCKS),100) \
		--step $(or $(STEP),10) \
		$(if $(SLOW),--slow) \
		$(if $(VERBOSE),--verbose)

build-recent-proof:
	@echo ">>> Building recent proof..."
	. .venv/bin/activate && cd scripts/data && \
		python build_recent_proof.py \
		$(if $(START),--start $(START)) \
		$(if $(MAX_HEIGHT),--max-height $(MAX_HEIGHT)) \
		$(if $(SLOW),--slow) \
		$(if $(VERBOSE),--verbose)

collect-resources-all:
	@echo ">>> Collecting resource usage data (all tests)..."
	cd packages/client && python ../../scripts/data/collect_resources.py \
		$(if $(NOCAPTURE),--nocapture) \
		$(if $(FORCEALL),--forceall)

# Main data generation target, depending on specific data generation tasks
data-generate: data-generate-timestamp data-generate-utxo
	@echo "All data generation tasks completed."

################################## SERVICES ##################################
build-recent-proof-service-status:
	systemctl status build-recent-proof.service || true
	systemctl status build-recent-proof.timer || true

build-recent-proof-service-run:
	sudo systemctl start build-recent-proof.service

