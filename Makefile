########################################## CLIENT ##########################################

client-build:
	scarb --profile proving build --package client --target-kinds executable

client-build-with-shinigami:
	sed -i.bak 's/default = \[\]/default = ["shinigami"]/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak
	scarb --profile proving build --package client --target-kinds executable
	sed -i.bak 's/default = \["shinigami"\]/default = []/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak

########################################## BINARIES ##########################################

install-cairo-execute:
	cargo install --git https://github.com/m-kus/cairo --rev 9117214e4a3509870c6a6db8e61ddcdaf9ade561 cairo-execute

install-cairo-bootloader:
	cargo install --git https://github.com/m-kus/cairo-bootloader --rev 5aa40dc5f084c6406ae9d76b19a115e3a3832b97 cairo-bootloader

install-stwo:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install \
		--git https://github.com/starkware-libs/stwo-cairo \
		--rev f8979ed82d86bd3408f9706a03a63c54bd221635 \
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
