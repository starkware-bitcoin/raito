use stwo_cairo_air::{CairoProof, VerificationOutput, get_verification_output, verify_cairo};

#[executable]
fn main(proof: CairoProof) -> VerificationOutput {
    let output = get_verification_output(proof: @proof);

    match verify_cairo(proof) {
        Result::Ok(_) => {},
        Result::Err(err) => panic!("Failed to verify proof: {:?}", err),
    }

    output
}
