# Git revisions for external dependencies
BOOTLOADER_HINTS_REV ?= cec3b568447a2ebc7f1cdee8c1001b5af11ba34b
STWO_REV ?= a8a434b521a588cf80eb3625affb984f9913be6c
CAIRO_EXECUTE_REV ?= e209f4557c535ddb4d2c76b2c6b14d004af99467
SCARB_EJECT_REV ?= 9c9b35870ac35af46e62b725219a8ac925281fe5
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
		cairo-program-runner stwo_run_and_prove

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
	cargo install --git https://github.com/software-mansion-labs/scarb-eject \
		--rev $(SCARB_EJECT_REV)

install-convert-proof-format:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install --force \
		--git https://github.com/starkware-libs/stwo-cairo \
		dev-utils

install-corelib:
	mkdir -p vendor
	rm -rf vendor/cairo
	git clone --single-branch --branch raito \
		https://github.com/m-kus/cairo vendor/cairo

	(cd vendor/cairo && git config --add remote.origin.fetch "+refs/pull/*/head:refs/remotes/origin/pr/*" && git fetch origin --prune && git checkout $(CAIRO_EXECUTE_REV))
	ln -s "$(CURDIR)/vendor/cairo/corelib" \
		packages/assumevalid/corelib

install: install-bootloader-hints install-cairo-execute \
	install-scarb-eject install-corelib

################################## ASSUMEVALID ##################################

assumevalid-build:
	scarb --profile proving build --package assumevalid \
		--no-default-features

assumevalid-eject:
	scarb-eject --package assumevalid \
		--output packages/assumevalid/cairo_project.toml

assumevalid-build-with-syscalls:
	mkdir -p target/proving
	cd packages/assumevalid && \
	cairo-execute \
		--build-only \
		--output-path \
			../../target/proving/assumevalid-syscalls.executable.json \
		--executable assumevalid::main \
		--ignore-warnings \
		--allow-syscalls .

assumevalid-execute:
	@[ -n "$(ARGS_FILE)" ] || (echo "ERROR: ARGS_FILE is required. Usage: make assumevalid-execute ARGS_FILE=path/to/args.json" >&2; exit 1)
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--arguments-file $(ARGS_FILE) \
		--print-resource-usage

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
	@if [ "$$(lsb_release -rs | cut -d. -f1)" -ge "24" ]; then \
		echo ">>> Detected Ubuntu 24.04+, using python3-venv"; \
		sudo apt install -y python3-pip python3-venv; \
	else \
		echo ">>> Detected Ubuntu < 24.04, using python3.11-venv"; \
		sudo apt install -y python3-pip python3.11-venv; \
	fi

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

assumevalid-bridge:
	cargo run -p raito-bridge-node

# Run the raito-assumevalid CLI Prove subcommand
assumevalid-prove:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" cargo run --release -p raito-assumevalid -- \
		$(if $(LOG_LEVEL),--log-level $(LOG_LEVEL)) \
		$(if $(BRIDGE_URL),--bridge-url $(BRIDGE_URL)) \
	    prove \
		$(if $(KEEP_TEMP_FILES),--keep-temp-files) \
		$(if $(OUTPUT_DIR),--output-dir $(OUTPUT_DIR)) \
		$(if $(TOTAL_BLOCKS),--total-blocks $(TOTAL_BLOCKS)) \
		$(if $(STEP_SIZE),--step-size $(STEP_SIZE)) \
		$(if $(LOAD_FROM_GCS),--load-from-gcs) \
		$(if $(SAVE_TO_GCS),--save-to-gcs) \
		$(if $(GCS_BUCKET),--gcs-bucket $(GCS_BUCKET))

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
	systemctl status raito-build-recent-proof.service || true
	systemctl status raito-build-recent-proof.timer || true

build-recent-proof-service-run:
	sudo systemctl start raito-build-recent-proof.service

build-recent-proof-service-stop:
	sudo systemctl stop raito-build-recent-proof.timer

build-recent-proof-service-start:
	sudo systemctl start raito-build-recent-proof.timer
