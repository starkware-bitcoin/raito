use core::serde::Serde;

#[derive(Drop, Serde, Debug)]
struct TaskResult {
    size_including_this_field: felt252,
    program_hash: felt252,
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
            *previous_task.program_hash == 0xd17d641e5e09ae0cc460fb6a495cbd3d5bd93550a745ad7d98bbedf8d40a6d,
            'Bad program hash'
        );

        assert(
            *current_task.prev_chain_state_hash == *previous_task.result_chain_state_hash,
            'Chain state hash mismatch'
        );

        assert(
            *current_task.prev_block_height == *previous_task.result_block_height,
            'Block height mismatch'
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
