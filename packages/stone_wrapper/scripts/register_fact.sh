#!/usr/bin/env bash

# File: https://github.com/HerodotusDev/integrity-calldata-generator/blob/main/cli/verify.sh

CALLDATA_DIR="tests/data/calldata"

# See https://github.com/HerodotusDev/integrity/blob/main/deployed_contracts.md
INTEGRITY_PROXY="0x16409cfef9b6c3e6002133b61c59d09484594b37b8e4daef7dcba5495a0ef1a"

if [ $# -ne 5 ]; then
    echo "Usage: $0 <job_id> <layout> <hasher> <stone_version> <memory_verification>"
    exit 1
fi

string_to_hex() {
    input_string="$1"
    hex_string="0x"
    for ((i = 0; i < ${#input_string}; i++)); do
        hex_char=$(printf "%x" "'${input_string:$i:1}")
        hex_string+=$hex_char
    done
    echo "$hex_string"
}

job_id=$1
layout=$(string_to_hex $2)
hasher=$(string_to_hex $3)
stone_version=$(string_to_hex $4)
memory_verification=$(string_to_hex $5)

send_transaction() {
    local retries=5
    local count=0
    local status=0

    while [[ $count -lt $retries ]]; do
        sncast \
            --wait \
            invoke \
            --contract-address "${INTEGRITY_PROXY}" \
            --function "$1" \
            --calldata "$3 $(<$2)"

        sleep 5 # extra delay to make sure the transaction is registered

        status=$?

        if [[ $status -eq 0 ]]; then
            return 0
        else
            echo "Transaction failed with status $status. Retrying... ($((count + 1))/$retries)"
        fi

        count=$((count + 1))
    done

    echo "Transaction failed after $retries attempts."
    return $status
}

echo ""
echo "Sending verify_proof_initial"
send_transaction "verify_proof_initial" "${CALLDATA_DIR}/initial" "$job_id $layout $hasher $stone_version $memory_verification"

i=1
while true; do
    filename="${CALLDATA_DIR}/step${i}"

    if [[ -e "$filename" ]]; then
        echo ""
        echo "Sending verify_proof_step (${i})"
        send_transaction "verify_proof_step" "$filename" "$job_id"
    else
        break
    fi

    ((i++))
done

echo ""
echo "Sending verify_proof_final_and_register_fact"
send_transaction "verify_proof_final_and_register_fact" "${CALLDATA_DIR}/final" "$job_id"
