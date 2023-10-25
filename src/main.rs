use datacube::DataCube;

use crate::optimize::control_flow_graph::{ProgramControlFlowGraph, Optimization};

mod errors;
mod datacube;
mod instruction;
mod program;

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
    
    let mut cfg = ProgramControlFlowGraph::new(&program);
    
    // optimization loop
    loop {
        use optimize::block_optimizations::*;
        use optimize::local_optimizations::*;
        
        if cfg.run_optimization_pass(local_optimization(simplify_outgoing_jumps)) {
            println!("simplify_outgoing_jumps"); continue
        } else if cfg.run_optimization_pass(remove_dead_blocks) {
            println!("remove_dead_blocks"); continue 
        } else if cfg.run_optimization_pass(combine_sequential_blocks) {
            println!("combine_sequential_blocks"); continue
        } else if cfg.run_optimization_pass(local_optimization(peephole_optimizations)) {
            println!("peephole_optimizations"); continue
        }
        
        break;
    }
    
    for block in cfg.blocks {
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
