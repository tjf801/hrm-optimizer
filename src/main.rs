use datacube::DataCube;

mod errors;
mod datacube;
mod instruction;
mod program;

mod basic_blocks;

fn main() -> std::process::ExitCode {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <file path>", argv[0]);
        return std::process::ExitCode::FAILURE;
    }
    
    let asm_file_contents = std::fs::read_to_string(&argv[1]).expect("Failed to read file");
    let mut program = program::Program::from_asm(&asm_file_contents).unwrap();
    
    // for (i, inst) in program.instructions.iter().enumerate() {
    //     println!("{i}. {inst:?}");
    // }
    
    for block in program.split_into_blocks() {
        println!("Block {:?}:", block.id.0);
        
        for inst in block.instructions {
            println!("  {inst:?}");
        }
        
        for (id, flag) in block.outgoing_jumps {
            println!("  -> Block {:?} ({:?})", id.0, flag);
        }
    }
    
    // (NOTE: average perf: 182 steps)
    program.initial_floor = vec![None; 15];
    program.initial_floor[14] = Some(DataCube::from_number(0).unwrap());
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
