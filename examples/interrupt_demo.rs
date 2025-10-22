use pic_simulator::Simulator;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     PIC12F629/675 Interrupt Demo                           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Demo 1: Timer0 Overflow Interrupt
    println!("═══ Demo 1: Timer0 Overflow Interrupt ═══\n");
    
    let mut sim = Simulator::new();
    sim.reset();
    
    // Program with interrupt service routine
    let program = vec![
        // 0x000: Main program
        0x30FE,  // 0x000: MOVLW 0xFE          ; Load 254
        0x0081,  // 0x001: MOVWF TMR0          ; Set TMR0
        0x30A4,  // 0x002: MOVLW 0xA4          ; GIE=1, T0IE=1, T0IF=1
        0x008B,  // 0x003: MOVWF INTCON        ; Enable interrupts
        0x0000,  // 0x004: NOP                 ; (Interrupt vector)
        0x2808,  // 0x005: GOTO ISR            ; Jump to ISR
        0x0000,  // 0x006: NOP                 ; Main loop
        0x2806,  // 0x007: GOTO 0x006          ; Infinite loop
        
        // 0x008: Interrupt Service Routine
        0x1485,  // 0x008: BSF GPIO, 1         ; Toggle LED
        0x108B,  // 0x009: BCF INTCON, 2       ; Clear T0IF
        0x0009,  // 0x00A: RETFIE              ; Return from interrupt
    ];
    
    sim.load_program(&program);
    
    println!("Program loaded:");
    println!("  Main: Initialize TMR0 and enable interrupts");
    println!("  ISR:  Toggle GPIO pin and clear flag\n");
    
    // Configure Timer0 for no prescaler
    sim.cpu_mut().set_status_bit(5); // Bank 1
    sim.cpu_mut().write_register(0x81, 0x08); // OPTION_REG: PSA=1
    sim.cpu_mut().clear_status_bit(5); // Bank 0
    
    // Execute main program initialization
    for i in 0..4 {
        let pc = sim.cpu().get_pc();
        sim.step().unwrap();
        println!("Step {}: PC=0x{:04X}", i+1, pc);
    }
    
    println!("\nNow in main loop, waiting for TMR0 overflow...");
    
    let mut step_count = 0;
    let mut interrupt_count = 0;
    
    for _ in 0..20 {
        let pc_before = sim.cpu().get_pc();
        sim.step().unwrap();
        let pc_after = sim.cpu().get_pc();
        step_count += 1;
        
        // Check if we jumped to ISR
        if pc_before == 0x006 && pc_after == 0x004 {
            interrupt_count += 1;
            println!("\n🔔 Interrupt #{} triggered at step {}!", interrupt_count, step_count);
            println!("   Jumped from PC=0x{:04X} to ISR at PC=0x{:04X}", pc_before, pc_after);
            
            // Execute ISR
            for j in 0..3 {
                let pc = sim.cpu().get_pc();
                sim.step().unwrap();
                let intcon = sim.cpu().read_register(0x0B);
                println!("   ISR step {}: PC=0x{:04X}, INTCON=0x{:02X}", j+1, pc, intcon);
            }
            
            println!("   Returned to main loop at PC=0x{:04X}", sim.cpu().get_pc());
        }
        
        if interrupt_count >= 2 {
            break;
        }
    }
    
    // Demo 2: Timer1 Overflow Interrupt
    println!("\n\n═══ Demo 2: Timer1 Overflow Interrupt ═══\n");
    
    sim.reset();
    
    let program2 = vec![
        // Main program
        0x30C0,  // 0x000: MOVLW 0xC0          ; GIE=1, PEIE=1
        0x008B,  // 0x001: MOVWF INTCON
        0x3001,  // 0x002: MOVLW 0x01          ; TMR1IE=1
        0x008C,  // 0x003: MOVWF PIE1
        0x30FF,  // 0x004: MOVLW 0xFF
        0x008E,  // 0x005: MOVWF TMR1L         ; Set Timer1 = 0xFFF0
        0x30F0,  // 0x006: MOVLW 0xF0
        0x008F,  // 0x007: MOVWF TMR1H
        0x3001,  // 0x008: MOVLW 0x01
        0x0090,  // 0x009: MOVWF T1CON         ; Enable Timer1
        0x0000,  // 0x00A: NOP                 ; (Interrupt vector at 0x004)
        0x2810,  // 0x00B: GOTO ISR
        0x0000,  // 0x00C: NOP                 ; Main loop
        0x280C,  // 0x00D: GOTO 0x00C
        
        // ISR
        0x1405,  // 0x010: BSF GPIO, 0         ; Toggle LED
        0x108C,  // 0x011: BCF PIR1, 0         ; Clear TMR1IF
        0x0009,  // 0x012: RETFIE
    ];
    
    sim.load_program(&program2);
    
    println!("Timer1 interrupt program loaded");
    println!("Timer1 set to 0xFFF0 (will overflow soon)\n");
    
    // Execute initialization
    for _ in 0..10 {
        sim.step().unwrap();
    }
    
    println!("Initialization complete, now in main loop\n");
    
    // Wait for Timer1 interrupt
    for i in 0..30 {
        let pc_before = sim.cpu().get_pc();
        sim.step().unwrap();
        let pc_after = sim.cpu().get_pc();
        
        if pc_before == 0x00C && (pc_after == 0x004 || pc_after == 0x00B) {
            println!("🔔 Timer1 interrupt at step {}!", i+1);
            
            let tmr1l = sim.cpu().read_register(0x0E);
            let tmr1h = sim.cpu().read_register(0x0F);
            println!("   Timer1 = 0x{:02X}{:02X}", tmr1h, tmr1l);
            
            break;
        }
    }
    
    // Demo 3: Interrupt Priority
    println!("\n\n═══ Demo 3: Multiple Interrupt Sources ═══\n");
    
    sim.reset();
    
    // Enable both TMR0 and TMR1 interrupts
    sim.cpu_mut().write_register(0x0B, 0xE4); // GIE=1, PEIE=1, T0IE=1
    sim.cpu_mut().write_register(0x8C, 0x01); // TMR1IE=1
    
    // Set both timers near overflow
    sim.cpu_mut().write_register(0x01, 0xFE); // TMR0 = 254
    sim.cpu_mut().write_register(0x0E, 0xFE); // TMR1L = 0xFE
    sim.cpu_mut().write_register(0x0F, 0xFF); // TMR1H = 0xFF
    
    // Enable Timer1
    sim.cpu_mut().write_register(0x10, 0x01);
    
    println!("Both TMR0 and Timer1 enabled and near overflow");
    println!("Watching for interrupt flags...\n");
    
    let program3 = vec![
        0x0000,  // NOP
        0x2800,  // GOTO 0x000
    ];
    sim.load_program(&program3);
    
    for i in 0..10 {
        sim.step().unwrap();
        
        let intcon = sim.cpu().read_register(0x0B);
        let pir1 = sim.cpu().read_register(0x0C);
        let t0if = (intcon & 0x04) != 0;
        let tmr1if = (pir1 & 0x01) != 0;
        
        if t0if || tmr1if {
            println!("Step {}: T0IF={}, TMR1IF={}", 
                i+1, 
                if t0if { "1" } else { "0" },
                if tmr1if { "1" } else { "0" }
            );
        }
    }
    
    println!("\n✓ All interrupt demos complete!");
}