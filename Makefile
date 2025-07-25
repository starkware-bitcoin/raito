# Git revisions for external dependencies
BOOTLOADER_HINTS_REV ?= 5648cf0a5a2574c2870151cd178ff3ae4b141824
STWO_REV ?= d09a2cfd2b308b6210906f26320f767cf279abef
CAIRO_EXECUTE_REV ?= 7fbbd0112b5a926403c17fa95ad831c1715fd1b1
########################################## CLIENT ##########################################

client-build:
	scarb --profile proving build --package client --target-kinds executable

client-build-with-shinigami:
	scarb --profile proving build --package client --target-kinds executable --features shinigami


########################################## BINARIES ##########################################

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
	cargo install --git https://github.com/m-kus/cairo --rev $(CAIRO_EXECUTE_REV) cairo-execute

install-scarb-eject:
	cargo install --git https://github.com/software-mansion-labs/scarb-eject

install-corelib:
	mkdir -p vendor
	rm -rf vendor/cairo
	git clone --single-branch --branch m-kus/system-builtin https://github.com/m-kus/cairo vendor/cairo
	(cd vendor/cairo && git checkout $(CAIRO_EXECUTE_REV))
	ln -s "$(CURDIR)/vendor/cairo/corelib" packages/assumevalid/corelib

install: install-bootloader-hints install-stwo install-cairo-execute

########################################## ASSUMEVALID ##########################################

assumevalid-data:
	./scripts/data/generate_data.py \
		--mode light \
		--height 0 \
		--num_blocks 1 \
		--output_file packages/assumevalid/tests/data/blocks_0_1.json
	./scripts/data/generate_data.py \
		--mode light \
		--height 1 \
		--num_blocks 1 \
		--output_file packages/assumevalid/tests/data/blocks_1_2.json

assumevalid-build:
	scarb --profile proving build --package assumevalid --no-default-features

assumevalid-prim-execute:
	mkdir -p target/execute/assumevalid/execution1
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_0_1.json \
		--output-path target/execute/assumevalid/execution1/args.json
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--arguments-file target/execute/assumevalid/execution1/args.json \
		--print-resource-usage

assumevalid-bis-execute:
	mkdir -p target/execute/assumevalid/execution2
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_1_2.json \
		--proof-path target/execute/assumevalid/execution1/proof.json \
		--output-path target/execute/assumevalid/execution2/args.json
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--arguments-file target/execute/assumevalid/execution2/args.json \
		--print-resource-usage

########################################## ASSUMEVALID WITH SYSCALLS ##########################################

assumevalid-eject:
	scarb-eject --package assumevalid --output packages/assumevalid/cairo_project.toml

assumevalid-build-with-syscalls:
	cd packages/assumevalid && \
	cairo-execute \
		--build-only \
		--output-path ../../target/proving/assumevalid.executable.json \
		--executable assumevalid::main \
		--ignore-warnings \
		--allow-syscalls .

assumevalid-clean:
	rm -rf target/execute/assumevalid/execution1
	mkdir -p target/execute/assumevalid/execution1
	rm -rf target/execute/assumevalid/execution2
	mkdir -p target/execute/assumevalid/execution2

assumevalid-prim-bootload: assumevalid-clean
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_0_1.json \
		--output-path target/execute/assumevalid/execution1/args.json
	scripts/data/generate_program_input.py \
		--executable $(CURDIR)/target/proving/assumevalid.executable.json \
		--args-file $(CURDIR)/target/execute/assumevalid/execution1/args.json \
		--program-hash-function blake \
		--output $(CURDIR)/target/execute/assumevalid/execution1/program-input.json
	cairo_program_runner \
		--program bootloaders/simple_bootloader_compiled.json \
		--program_input $(CURDIR)/target/execute/assumevalid/execution1/program-input.json \
		--air_public_input target/execute/assumevalid/execution1/pub.json \
		--air_private_input target/execute/assumevalid/execution1/priv.json \
		--trace_file $(CURDIR)/target/execute/assumevalid/execution1/trace.json \
		--memory_file $(CURDIR)/target/execute/assumevalid/execution1/memory.json \
		--layout all_cairo_stwo \
		--proof_mode \
		--execution_resources_file target/execute/assumevalid/execution1/resources.json

assumevalid-prim-prove:
	adapted_stwo \
		--priv_json target/execute/assumevalid/execution1/priv.json \
		--pub_json target/execute/assumevalid/execution1/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/execution1/proof.json \
		--proof-format cairo-serde \
		--verify

assumevalid-bis-bootload:
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_1_2.json \
		--proof-path target/execute/assumevalid/execution1/proof.json \
		--output-path target/execute/assumevalid/execution2/args.json
	scripts/data/generate_program_input.py \
		--executable target/proving/assumevalid.executable.json \
		--args-file target/execute/assumevalid/execution2/args.json \
		--program-hash-function blake \
		--output target/execute/assumevalid/execution2/program-input.json
	cairo_program_runner \
		--program bootloaders/simple_bootloader_compiled.json \
		--program_input target/execute/assumevalid/execution2/program-input.json \
		--air_public_input target/execute/assumevalid/execution2/pub.json \
		--air_private_input target/execute/assumevalid/execution2/priv.json \
		--trace_file $(CURDIR)/target/execute/assumevalid/execution2/trace.json \
		--memory_file $(CURDIR)/target/execute/assumevalid/execution2/memory.json \
		--layout all_cairo_stwo \
		--proof_mode \
		--execution_resources_file target/execute/assumevalid/execution2/resources.json

assumevalid-bis-prove:
	adapted_stwo \
		--priv_json target/execute/assumevalid/execution1/priv.json \
		--pub_json target/execute/assumevalid/execution1/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/execution2/proof.json \
		--proof-format cairo-serde \
		--verify

########################################## PIPELINE ##########################################

build-simple-bootloader:
	. .venv/bin/activate && cd ../starkware && cairo-compile --proof_mode \
		src/starkware/cairo/bootloaders/simple_bootloader/simple_bootloader.cairo \
		--cairo_path src --output $(CURDIR)/bootloaders/simple_bootloader_compiled.json

setup: install-system-packages create-venv install-python-dependencies

install-system-packages:
	@echo ">>> Updating apt package list and installing system-level Python packages..."
	sudo apt update
	sudo apt install -y python3-pip python3.11-venv # Use -y for non-interactive install

create-venv:
	@echo ">>> Creating Python virtual environment '.venv'..."
	python3 -m venv .venv

install-python-dependencies:
	@echo "Installing Python dependencies into the 'venv' virtual environment..."

	. .venv/bin/activate && pip install -r scripts/data/requirements.txt

data-generate-timestamp:
	@echo ">>> Generating timestamp data..."
	# Ensure the venv is activated for this script as well
	. .venv/bin/activate && cd scripts/data && python generate_timestamp_data.py

data-generate-utxo:
	@echo ">>> Generating UTXO data..."
	# Ensure the venv is activated for this script as well
	. .venv/bin/activate && cd scripts/data && python generate_utxo_data.py

prove-pow:
	@echo ">>> Prove POW..."
	. .venv/bin/activate && cd scripts/data && python prove_pow.py $(if $(START),--start $(START)) --blocks $(or $(BLOCKS),100) --step $(or $(STEP),10) $(if $(SLOW),--slow) $(if $(VERBOSE),--verbose)

# Main data generation target, depending on specific data generation tasks
data-generate: data-generate-timestamp data-generate-utxo
	@echo "All data generation tasks completed."
