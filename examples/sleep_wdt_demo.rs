use pic_simulator::Simulator;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     PIC12F629/675 SLEEP & WDT Demo                         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Demo 1: CLRWDT instruction
    println!("═══ Demo 1: CLRWDT Instruction ═══\n");
    
    let mut sim = Simulator::new();
    sim.reset();
    
    println!("WDT enabled by default");
    println!("Initial WDT counter: {}", sim.cpu().wdt().get_counter());
    
    // Run some cycles
    let program = vec![
        0x0000,  // NOP
        0x0000,  // NOP
        0x0000,  // NOP
        0x0064,  // CLRWDT
    ];
    
    sim.load_program(&program);
    
    for i in 0..3 {
        sim.step().unwrap();
        println!("After step {}: WDT counter = {}", i+1, sim.cpu().wdt().get_counter());
    }
    
    println!("\nExecuting CLRWDT...");
    sim.step().unwrap();
    println!("After CLRWDT: WDT counter = {}", sim.cpu().wdt().get_counter());
    
    // Demo 2: SLEEP instruction
    println!("\n\n═══ Demo 2: SLEEP Instruction ═══\n");
    
    sim.reset();
    
    let program2 = vec![
        0x0063,  // 0x000: SLEEP
        0x0000,  // 0x001: NOP (after wake-up)
        0x0000,  // 0x002: NOP
    ];
    
    sim.load_program(&program2);
    
    println!("Executing SLEEP instruction...");
    sim.step().unwrap();
    
    println!("CPU is now sleeping: {}", sim.cpu().is_sleeping());
    
    let status = sim.cpu().read_register(0x03);
    println!("STATUS register: 0x{:02X}", status);
    println!("  TO (bit 4) = {}", if status & 0x10 != 0 { "1" } else { "0" });
    println!("  PD (bit 3) = {}", if status & 0x08 != 0 { "1" } else { "0" });
    
    println!("\nTrying to execute more instructions while sleeping...");
    for i in 0..5 {
        sim.step().unwrap();
        if !sim.cpu().is_sleeping() {
            println!("Woke up after {} sleep cycles!", i+1);
            break;
        }
        println!("  Still sleeping... (cycle {})", i+1);
    }
    
    // Demo 3: Wake from sleep by interrupt
    println!("\n\n═══ Demo 3: Wake from Sleep by Interrupt ═══\n");
    
    sim.reset();
    
    // Enable Timer0 interrupt
    sim.cpu_mut().write_register(0x0B, 0xA0); // GIE=1, T0IE=1
    sim.cpu_mut().write_register(0x01, 0xFE); // TMR0 near overflow
    
    let program3 = vec![
        0x0063,  // 0x000: SLEEP
        0x0000,  // 0x001: NOP (after wake-up)
        0x2801,  // 0x002: GOTO 0x001
        0x0000,  // 0x003: NOP
        0x2804,  // 0x004: GOTO ISR (Interrupt vector)
        0x108B,  // 0x005: BCF INTCON, 2  (Clear T0IF)
        0x0009,  // 0x006: RETFIE
    ];
    
    sim.load_program(&program3);
    
    println!("Timer0 set to 0xFE (will overflow soon)");
    println!("Interrupts enabled");
    println!("\nExecuting SLEEP...");
    
    sim.step().unwrap();
    println!("CPU sleeping: {}", sim.cpu().is_sleeping());
    
    println!("\nWaiting for Timer0 interrupt to wake up...");
    for i in 0..10 {
        sim.step().unwrap();
        if !sim.cpu().is_sleeping() {
            println!("✓ Woke up by interrupt after {} cycles!", i+1);
            println!("PC = 0x{:04X}", sim.cpu().get_pc());
            break;
        }
    }
    
    // Demo 4: WDT timeout during sleep
    println!("\n\n═══ Demo 4: WDT Timeout During Sleep ═══\n");
    
    sim.reset();
    
    // Configure WDT with short timeout for demo
    // Set prescaler to 1:1 (shortest timeout)
    sim.cpu_mut().set_status_bit(5); // Bank 1
    sim.cpu_mut().write_register(0x81, 0x08); // OPTION: PSA=1 (WDT gets prescaler)
    sim.cpu_mut().clear_status_bit(5); // Bank 0
    
    let program4 = vec![
        0x0063,  // SLEEP
    ];
    
    sim.load_program(&program4);
    
    println!("Entering sleep with WDT enabled...");
    println!("WDT timeout period: {} cycles", sim.cpu().wdt().get_timeout_period());
    
    sim.step().unwrap();
    println!("CPU sleeping: {}", sim.cpu().is_sleeping());
    
    println!("\nWaiting for WDT timeout...");
    
    let mut cycles = 0;
    while sim.cpu().is_sleeping() && cycles < 20000 {
        sim.step().unwrap();
        cycles += 1;
        
        if cycles % 5000 == 0 {
            println!("  WDT counter: {} / {}", 
                sim.cpu().wdt().get_counter(),
                sim.cpu().wdt().get_timeout_period()
            );
        }
    }
    
    if !sim.cpu().is_sleeping() {
        println!("✓ Woke up by WDT timeout after {} cycles!", cycles);
    }
    
    println!("\n✓ All SLEEP & WDT demos complete!");
}