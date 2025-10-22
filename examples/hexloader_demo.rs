use std::path::Path;
use pic_simulator::{Simulator, Debugger, HexLoader};

fn main() {
    println!("PIC12F629/675 HEX Loader Demo");
    println!("==============================\n");

    // Example 1: Load program from external HEX file
    let path = Path::new("test_program.hex");
    println!("Example 1: Loading HEX from file ({})", path.display());
    println!("---------------------------------------");

    if !path.exists() {
        eprintln!("✗ Error: File {:?} not found!", path);
        eprintln!("💡 Hint: create a file named test_program.hex with valid Intel HEX data, for example:\n");
        eprintln!(
            ":020000040000FA\n\
             :10000000553000A0053E00A1A007A2000000280274\n\
             :00000001FF\n"
        );
        return;
    }

    // Load the file
    let result = HexLoader::load_file(path);
    let program = match result {
        Ok(p) => {
            println!("✓ HEX file parsed successfully!");
            p
        }
        Err(e) => {
            println!("✗ Error parsing HEX: {}", e);
            return;
        }
    };

    // Initialize simulator and load program memory
    let mut sim = Simulator::new();
    sim.reset();
    sim.load_program(&program.program);

    println!("\nLoaded program:");
    Debugger::disassemble_range(sim.cpu(), 0, 8);

    println!("\nExecuting program...");
    for i in 0..6 {
        let pc = sim.cpu().get_pc();
        let word = sim.cpu().memory().read_program(pc);
        let asm = Debugger::disassemble(word);

        sim.step().unwrap();
        println!("Step {}: 0x{:04X} {}", i + 1, pc, asm);
    }

    println!("\nFinal state:");
    println!("  W      = 0x{:02X}", sim.cpu().read_w());
    println!("  [0x20] = 0x{:02X}", sim.cpu().read_register(0x20));
    println!("  [0x21] = 0x{:02X}", sim.cpu().read_register(0x21));
    println!("  [0x22] = 0x{:02X}", sim.cpu().read_register(0x22));

    println!("\n✓ HEX Loader demo complete!");
}
