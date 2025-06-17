install-cairo-prove:
	RUSTFLAGS="-C target-cpu=native -C opt-level=3" \
		cargo install \
			--git https://github.com/starkware-libs/stwo-cairo \
			--rev a9fd9934eabb5ca1a06a910ef04ed4c0dae9114c \
			cairo-prove
			
install-cairo-execute:
	cargo install --git https://github.com/m-kus/cairo --rev 9117214e4a3509870c6a6db8e61ddcdaf9ade561 cairo-execute

client-build:
	scarb --profile proving build --package client --target-kinds executable

client-build-with-shinigami:
	sed -i.bak 's/default = \[\]/default = ["shinigami"]/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak
	scarb --profile proving build --package client --target-kinds executable
	sed -i.bak 's/default = \["shinigami"\]/default = []/' packages/consensus/Scarb.toml && rm packages/consensus/Scarb.toml.bak

assumevalid-build:
	sed -i.bak 's/default = \["syscalls"\]/default = \[\]/' packages/utils/Scarb.toml && rm packages/utils/Scarb.toml.bak
	scarb --profile proving build --package assumevalid --target-names main
	sed -i.bak 's/default = \[\]/default = \["syscalls"\]/' packages/utils/Scarb.toml && rm packages/utils/Scarb.toml.bak

assumevalid-execute:
	scripts/data/format_args.py --input_file packages/assumevalid/tests/data/batch_100.json > target/execute/assumevalid/args.json
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--executable-name main \
		--arguments-file target/execute/assumevalid/args.json \
		--print-resource-usage

assumevalid-pie:
	rm -rf target/execute/assumevalid/execution1
	mkdir -p target/execute/assumevalid/execution1
	scripts/data/format_args.py --input_file packages/assumevalid/tests/data/light_169.json > target/execute/assumevalid/args.json
	cairo-execute \
		--layout all_cairo_stwo \
		--args-file target/execute/assumevalid/args.json \
		--prebuilt \
		--output-path target/execute/assumevalid/execution1/raito_1.zip \
		target/proving/main.executable.json

assumevalid-bootload:
	cairo-bootloader --cairo_pies target/execute/assumevalid/execution1/cairo_pie.zip \
		--layout all_cairo \
		--secure_run true \
		--ignore_fact_topologies true \
		--cairo_pie_output target/execute/assumevalid/boot.zip

assumevalid-execute-rec:
	scarb --profile proving execute \
		--no-build \
		--package assumevalid \
		--executable-name agg \
		--arguments-file target/execute/assumevalid/proof.json \
		--print-resource-usage

assumevalid-prove:
	rm -rf target/execute/assumevalid
	mkdir -p target/execute/assumevalid
	scripts/data/format_args.py --input_file packages/assumevalid/tests/data/light_169.json > target/execute/assumevalid/args.json
	cairo-prove prove \
		target/proving/main.executable.json \
		target/execute/assumevalid/proof.json \
		--arguments-file target/execute/assumevalid/args.json \
		--proof-format cairo-serde

assumevalid-prove-rec:
	cairo-prove prove \
		target/proving/agg.executable.json \
		target/execute/assumevalid/proof-rec.json \
		--arguments-file target/execute/assumevalid/proof.json \
		--proof-format cairo-serde

assumevalid:
	$(MAKE) assumevalid-build
	$(MAKE) assumevalid-prove
	$(MAKE) assumevalid-execute-rec


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