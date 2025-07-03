use core::serde::Serde;

#[derive(Drop, Serde, Debug)]
struct TaskResult {
    size_including_this_field: felt252,
    applicative_bootloader_hash: felt252,
    aggregator_hash: felt252,
    simple_bootloader_hash: felt252,
    applicative_bootloader_hash_again: felt252,    
    cairo_verifies_hash: felt252,
    prev_block_height: u32,
    result_block_height: u32,
    prev_chain_state_hash: felt252,
    result_chain_state_hash: felt252
}


#[derive(Drop, Serde)]
struct Args {
    task_results: Span<TaskResult>,
}

#[derive(Drop, Serde)]
struct OutputWithArgs {    
    args: Args,
    output: Result,
}

#[derive(Drop, Serde)]
struct Result {
    prev_block_height: u32,
    result_block_height: u32,
    prev_chain_state_hash: felt252,
    result_chain_state_hash: felt252
}

#[executable]
fn aggregate(input: Args) -> OutputWithArgs {
    let task_results = input.task_results;
    
    validate_task_chain(task_results);
    
    let result = build_result(task_results);
    
    OutputWithArgs {
        args: input,
        output: result,
    }
}

fn validate_task_chain(task_results: Span<TaskResult>) {
    let mut i = 1;
    while i != task_results.len() {            
        let current_task = task_results.at(i);
        let previous_task = task_results.at(i - 1);
        
        assert(
            *current_task.prev_chain_state_hash == *previous_task.result_chain_state_hash,
            'Chain state hash mismatch'
        );

        assert(
            *current_task.prev_block_height == *previous_task.result_block_height,
            'Block height mismatch'
        );

        assert(
            *previous_task.applicative_bootloader_hash == 0x2710558d1139041e1f6af4507603e4357f60ee6838a85f8814baa46aca4b1d3,
            'Bad applicative bootloader hash'
        );

        assert(
            *previous_task.aggregator_hash == 0x68bf7c5ed415b332050362df0796f7503e064795b7204abae20e73fcbc85da1,
            'Bad aggregator hash'
        );

        assert(
            *previous_task.simple_bootloader_hash == 0x5ab580b04e3532b6b18f81cfa654a05e29dd8e2352d88df1e765a84072db07,
            'Bad simple_bootloader hash'
        );

        assert(
            *previous_task.applicative_bootloader_hash_again == 0x2710558d1139041e1f6af4507603e4357f60ee6838a85f8814baa46aca4b1d3,
            'Bad applicative recursion hash'
        );

        assert(
            *previous_task.cairo_verifies_hash == 0x22f3bd2fd7a5a131199e56a7e39a430cc287ed3df600a8288f90c1932a5e62e,
            'Bad cairo_verifies hash'
        );
        
        i += 1;
    }
}

fn build_result(task_results: Span<TaskResult>) -> Result {
    if task_results.is_empty() {
        panic!("No task results provided");
    }
    
    let first_task = task_results.at(0);
    let last_task = task_results.at(task_results.len() - 1);

    Result {
        prev_block_height: *first_task.prev_block_height,
        result_block_height: *last_task.result_block_height,
        prev_chain_state_hash: *first_task.prev_chain_state_hash,
        result_chain_state_hash: *last_task.result_chain_state_hash
    }
}
