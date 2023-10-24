use datacube::DataCube;

mod errors;
mod datacube;
mod instruction;
mod program;

mod basic_blocks;
mod optimize;

fn main() -> std::process::ExitCode {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <file path>", argv[0]);
        return std::process::ExitCode::FAILURE;
    }
    
    let asm_file_contents = std::fs::read_to_string(&argv[1]).expect("Failed to read file");
    let mut program = program::Program::from_asm(&asm_file_contents).unwrap();
    program.initial_floor = vec![None; 16];
    program.initial_floor[15] = Some(DataCube::from_number(4).unwrap());
    program.initial_floor[14] = Some(DataCube::from_number(0).unwrap());
    
    // for (i, inst) in program.instructions.iter().enumerate() {
    //     println!("{i}. {inst:?}");
    // }
    // println!("{:?}", program.jump_label_lines);
    
    let mut blocks = program.split_into_blocks();
    
    // optimization loop
    loop {
        use optimize::block_optimizations::*;
        
        refresh_incoming_jumps(&mut blocks);
        
        if simplify_outgoing_jumps(&mut blocks) { println!("simplify_outgoing_jumps"); continue }
        else if remove_dead_blocks(&mut blocks) { println!("remove_dead_blocks"); continue }
        else if combine_sequential_blocks(&mut blocks) { println!("combine_sequential_blocks"); continue }
        
        break;
    }
    
    for block in blocks {
        println!("Block {:?}:", block.id.0);
        
        match &block.incoming_jumps[..] {
            [] => if block.id.0 != 0 { println!("  (DEAD BLOCK)") },
            [jumps @ ..] => {
                println!("  Incoming jumps:");
                for (id, flag) in jumps {
                    println!("    -> Block {:?} ({:?})", id.0, flag);
                }
                println!();
            },
        }
        
        for inst in block.instructions {
            println!("  {inst:?}");
        }
        
        println!("  Outgoing jumps:");
        for (id, flag) in block.outgoing_jumps {
            println!("    -> Block {:?} ({:?})", id.0, flag);
        }
        println!();
        
        println!();
    }
    
    // (NOTE: average perf: 182 steps)
    println!("{:?}", program.simulate(vec![
        DataCube::from_char('A').unwrap(),
        DataCube::from_char('D').unwrap(),
        DataCube::from_char('E').unwrap(),
        DataCube::from_char('C').unwrap(),
        DataCube::from_char('A').unwrap(),
        DataCube::from_char('D').unwrap(),
        DataCube::from_char('E').unwrap(),
        DataCube::from_char('D').unwrap(),
        DataCube::from_char('B').unwrap(),
        DataCube::from_char('E').unwrap(),
    ]).unwrap());
    
    return std::process::ExitCode::SUCCESS;
}
