use pic_simulator::{Simulator, Debugger, Executor, Instruction};

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     PIC12F629/675 GPIO Demo                                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Demo 1: Basic Output
    println!("═══ Demo 1: Basic Output ═══\n");
    
    let mut sim = Simulator::new();
    sim.reset();
    
    println!("Initial GPIO state:");
    Debugger::display_gpio(sim.cpu());
    
    // Configure GP0 and GP1 as outputs
    println!("\n→ Configuring GP0 and GP1 as outputs...");
    sim.cpu_mut().write_register(0x85, 0x3C); // TRISIO: GP0,GP1 = output
    
    // Turn on GP0
    println!("→ Setting GP0 = HIGH...");
    Executor::execute(sim.cpu_mut(), Instruction::BSF { f: 0x05, b: 0 });
    
    Debugger::display_gpio(sim.cpu());
    
    // Turn on GP1
    println!("\n→ Setting GP1 = HIGH...");
    Executor::execute(sim.cpu_mut(), Instruction::BSF { f: 0x05, b: 1 });
    
    Debugger::display_gpio(sim.cpu());
    
    // Turn off GP0
    println!("\n→ Setting GP0 = LOW...");
    Executor::execute(sim.cpu_mut(), Instruction::BCF { f: 0x05, b: 0 });
    
    Debugger::display_gpio(sim.cpu());
    
    // Demo 2: Input with External Signals
    println!("\n\n═══ Demo 2: Reading Inputs ═══\n");
    
    sim.reset();
    
    // Configure GP2 as input (default)
    println!("→ GP2 configured as input (default)");
    
    // Simulate button press (external pin goes LOW)
    println!("→ Simulating button press (GP2 = LOW)...");
    sim.cpu_mut().gpio_mut().set_external_pin(2, false);
    
    Debugger::display_gpio(sim.cpu());
    
    // Read GPIO
    let gpio_val = sim.cpu_mut().read_register(0x05);
    println!("\nGPIO read value: 0x{:02X}", gpio_val);
    println!("GP2 is: {}", if gpio_val & 0x04 != 0 { "HIGH" } else { "LOW" });
    
    // Simulate button release
    println!("\n→ Simulating button release (GP2 = HIGH)...");
    sim.cpu_mut().gpio_mut().set_external_pin(2, true);
    
    let gpio_val = sim.cpu_mut().read_register(0x05);
    println!("GP2 is now: {}", if gpio_val & 0x04 != 0 { "HIGH" } else { "LOW" });
    
    // Demo 3: Weak Pull-ups
    println!("\n\n═══ Demo 3: Weak Pull-ups ═══\n");
    
    sim.reset();
    
    println!("→ GP4 as input without pull-up:");
    sim.cpu_mut().gpio_mut().set_external_pin(4, false);
    Debugger::display_gpio(sim.cpu());
    
    println!("\n→ Enabling weak pull-up on GP4...");
    sim.cpu_mut().write_register(0x95, 0x10); // WPU: Enable GP4 pull-up
    
    // With pull-up, should read high even if external is floating
    sim.cpu_mut().gpio_mut().set_external_pin(4, true);
    Debugger::display_gpio(sim.cpu());
    
    // Demo 4: LED Blink Simulation
    println!("\n\n═══ Demo 4: LED Blink Program ═══\n");
    
    sim.reset();
    
    // Program to blink LED on GP0
    let program = vec![
        0x3030,  // 0x000: MOVLW 0x30
        0x0085,  // 0x001: MOVWF TRISIO  ; GP0,GP1,GP2,GP3 = output
        0x1005,  // 0x002: BCF GPIO, 0   ; LED OFF
        0x1405,  // 0x003: BSF GPIO, 0   ; LED ON
        0x1005,  // 0x004: BCF GPIO, 0   ; LED OFF
        0x1405,  // 0x005: BSF GPIO, 0   ; LED ON
        0x2806,  // 0x006: GOTO 0x006    ; Halt
    ];
    
    sim.load_program(&program);
    
    println!("Executing LED blink program:\n");
    
    for i in 0..6 {
        let pc = sim.cpu().get_pc();
        let word = sim.cpu().memory().read_program(pc);
        let asm = Debugger::disassemble(word);
        
        sim.step().unwrap();
        
        println!("Step {}: 0x{:04X} {}", i + 1, pc, asm);
        
        // Show GPIO state after certain operations
        if i == 1 || i == 2 || i == 3 || i == 4 || i == 5 {
            let gpio_val = sim.cpu().read_register(0x05);
            println!("       → GPIO = 0x{:02X}, GP0 = {}", 
                gpio_val, 
                if gpio_val & 0x01 != 0 { "●" } else { "○" }
            );
        }
    }
    
    // Demo 5: Read-Modify-Write
    println!("\n\n═══ Demo 5: Read-Modify-Write ═══\n");
    
    sim.reset();
    
    println!("→ Configuring GP0,GP1,GP2 as outputs, GP3,GP4,GP5 as inputs...");
    sim.cpu_mut().write_register(0x85, 0x38); // TRISIO
    
    println!("→ Setting GPIO = 0b00000111 (GP2,GP1,GP0 = HIGH)...");
    sim.cpu_mut().write_register(0x05, 0x07);
    
    Debugger::display_gpio(sim.cpu());
    
    println!("\n→ Simulating external inputs: GP4=HIGH, GP5=LOW...");
    sim.cpu_mut().gpio_mut().set_external_pin(4, true);
    sim.cpu_mut().gpio_mut().set_external_pin(5, false);
    
    println!("→ Reading GPIO...");
    let gpio_read = sim.cpu_mut().read_register(0x05);
    println!("  Read value: 0x{:02X} = 0b{:06b}", gpio_read, gpio_read);
    
    println!("\n→ Using BTFSS to test GP4 (should skip)...");
    Executor::execute(sim.cpu_mut(), Instruction::BTFSS { f: 0x05, b: 4 });
    println!("  PC after BTFSS: 0x{:04X} (skipped!)", sim.cpu().get_pc());
    
    println!("\n✓ All GPIO demos complete!");
}