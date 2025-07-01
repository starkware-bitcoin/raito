# Default program hash function
PROGRAM_HASH_FUNCTION ?= blake

########################################## CLIENT ##########################################

client-build:
	scarb --profile proving build --package client --target-kinds executable

client-build-with-shinigami:
	sed -i.bak 's/default = \[\]/default = ["shinigami"]/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak
	scarb --profile proving build --package client --target-kinds executable
	sed -i.bak 's/default = \["shinigami"\]/default = []/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak

########################################## BINARIES ##########################################

install-bootloader-hints:
	cargo install \
		--git ssh://git@github.com/starkware-libs/bootloader-hints.git \
		--rev a0b20e8ac527d3591455743b88f60bc6df2c1c28 \
		cairo-program-runner

install-stwo:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install \
		--git https://github.com/starkware-libs/stwo-cairo \
		--rev 671e94dac5d13dbc2059f9dd10d9802c705ffaef \
		adapted_stwo

########################################## ASSUMEVALID ##########################################

assumevalid-build:
	sed -i.bak 's/default = \["syscalls"\]/default = \[\]/' packages/utils/Scarb.toml && rm packages/utils/Scarb.toml.bak
	scarb --profile proving build --package assumevalid
	sed -i.bak 's/default = \[\]/default = \["syscalls"\]/' packages/utils/Scarb.toml && rm packages/utils/Scarb.toml.bak

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

assumevalid-execute: assumevalid-clean
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_0_1.json \
		--output-path target/execute/assumevalid/execution1/args.json
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--arguments-file target/execute/assumevalid/execution1/args.json \
		--print-resource-usage

assumevalid-clean:
	rm -rf target/execute/assumevalid/execution1
	mkdir -p target/execute/assumevalid/execution1

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
	../dovki/work_dir/adapted_stwo \
		--priv_json target/execute/assumevalid/execution1/priv.json \
		--pub_json target/execute/assumevalid/execution1/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/execution1/proof.json \
		--proof-format cairo-serde \
		--verify

assumevalid-pie: assumevalid-clean
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_0_1.json \
		--output-path target/execute/assumevalid/execution1/args.json
	cairo-execute \
		--layout all_cairo_stwo \
		--args-file target/execute/assumevalid/execution1/args.json \
		--prebuilt \
		--output-path target/execute/assumevalid/execution1/cairo_pie.zip \
		target/proving/assumevalid.executable.json

assumevalid-bootload:
	stwo-bootloader \
		--pie target/execute/assumevalid/execution1/cairo_pie.zip \
		--output-path target/execute/assumevalid/execution1

assumevalid-prove:
	adapted_stwo \
		--priv_json target/execute/assumevalid/execution1/priv.json \
		--pub_json target/execute/assumevalid/execution1/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/execution1/proof.json \
		--proof-format cairo-serde \
		--verify

assumevalid-execute-rec:
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

assumevalid-pie-rec:
	scripts/data/format_assumevalid_args.py \
		--block-data packages/assumevalid/tests/data/blocks_1_2.json \
		--proof-path target/execute/assumevalid/execution1/proof.json \
		--output-path target/execute/assumevalid/execution2/args.json
	cairo-execute \
		--layout all_cairo_stwo \
		--args-file target/execute/assumevalid/execution2/args.json \
		--prebuilt \
		--output-path target/execute/assumevalid/execution2/cairo_pie.zip \
		target/proving/assumevalid.executable.json

assumevalid-bootload-rec:
	stwo-bootloader \
		--pie target/execute/assumevalid/execution2/cairo_pie.zip \
		--output-path target/execute/assumevalid/execution2

assumevalid-prove-rec:
	adapted_stwo \
		--priv_json target/execute/assumevalid/execution2/priv.json \
		--pub_json target/execute/assumevalid/execution2/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/execution2/proof.json \
		--proof-format cairo-serde \
		--verify

replicate-invalid-logup-sum:
	mkdir -p target/execute/assumevalid/invalid-logup-sum
	cairo-execute \
		--layout all_cairo_stwo \
		--args-file  packages/assumevalid/tests/data/invalid-logup-sum-arguments.json \
		--prebuilt \
		--output-path target/execute/assumevalid/invalid-logup-sum/cairo_pie.zip \
		target/proving/assumevalid.executable.json
	stwo-bootloader \
		--pie target/execute/assumevalid/invalid-logup-sum/cairo_pie.zip \
		--output-path target/execute/assumevalid/invalid-logup-sum
	adapted_stwo \
		--priv_json target/execute/assumevalid/invalid-logup-sum/priv.json \
		--pub_json target/execute/assumevalid/invalid-logup-sum/pub.json \
		--params_json packages/assumevalid/prover_params.json \
		--proof_path target/execute/assumevalid/invalid-logup-sum/proof.json \
		--proof-format cairo-serde \
		--verify

########################################## PIPELINE ##########################################

setup: install-system-packages create-venv install-python-dependencies

install-system-packages:
	@echo ">>> Updating apt package list and installing system-level Python packages..."
	sudo apt update
	sudo apt install -y python3-pip python3.11-venv # Use -y for non-interactive install

create-venv:
	@echo ">>> Creating Python virtual environment 'venv'..."
	python3 -m venv venv

install-python-dependencies: create-venv
	@echo "Installing Python dependencies into the 'venv' virtual environment..."

	. venv/bin/activate && pip install google-cloud-storage
	. venv/bin/activate && pip install -r scripts/data/requirements.txt

data-generate-timestamp:
	@echo ">>> Generating timestamp data..."
	# Ensure the venv is activated for this script as well
	. venv/bin/activate && cd scripts/data && python generate_timestamp_data.py

data-generate-utxo:
	@echo ">>> Generating UTXO data..."
	# Ensure the venv is activated for this script as well
	. venv/bin/activate && cd scripts/data && python generate_utxo_data.py

prove-pow:
	@echo ">>> Prove POW..."
	. venv/bin/activate && cd scripts/data && python prove_pow.py --blocks 100

# Main data generation target, depending on specific data generation tasks
data-generate: data-generate-timestamp data-generate-utxo
	@echo "All data generation tasks completed."
