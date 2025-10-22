use pic_simulator::Simulator;

fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║     PIC12F629/675 Timer Demo                               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Demo 1: Timer0 Basic Operation
    println!("═══ Demo 1: Timer0 Basic Counting ═══\n");
    
    let mut sim = Simulator::new();
    sim.reset();
    
    // Configure Timer0: internal clock, 1:4 prescaler
    // OPTION_REG: PSA=0 (assign to TMR0), PS=001 (1:4)
    sim.cpu_mut().set_status_bit(5); // Switch to Bank 1
    sim.cpu_mut().write_register(0x81, 0x01); // OPTION_REG
    sim.cpu_mut().clear_status_bit(5); // Back to Bank 0
    
    // Set TMR0 to 250
    sim.cpu_mut().write_register(0x01, 250);
    
    println!("Timer0 configured with 1:4 prescaler");
    println!("Initial TMR0 value: {}", sim.cpu().read_register(0x01));
    
    // Execute some instructions to let timer tick
    let program = vec![
        0x0000,  // NOP
        0x0000,  // NOP
        0x0000,  // NOP
        0x0000,  // NOP
    ];
    sim.load_program(&program);
    
    for i in 0..20 {
        sim.step().unwrap();
        if i % 4 == 3 {
            let tmr0 = sim.cpu().read_register(0x01);
            println!("After {} cycles: TMR0 = {}", (i+1)*4, tmr0);
        }
    }
    
    // Demo 2: Timer0 Overflow Interrupt
    println!("\n\n═══ Demo 2: Timer0 Overflow ═══\n");
    
    sim.reset();
    
    // Set TMR0 to 254 (will overflow soon)
    sim.cpu_mut().write_register(0x01, 254);
    
    println!("TMR0 set to 254 (near overflow)");
    
    // Configure for no prescaler
    sim.cpu_mut().set_status_bit(5);
    sim.cpu_mut().write_register(0x81, 0x08); // PSA=1 (no prescaler for TMR0)
    sim.cpu_mut().clear_status_bit(5);
    
    for i in 0..5 {
        sim.step().unwrap();
        let tmr0 = sim.cpu().read_register(0x01);
        let intcon = sim.cpu().read_register(0x0B);
        let t0if = (intcon & 0x04) != 0;
        
        println!("Step {}: TMR0 = {}, T0IF = {}", i+1, tmr0, if t0if { "1" } else { "0" });
    }
    
    // Demo 3: Timer1 16-bit Operation
    println!("\n\n═══ Demo 3: Timer1 16-bit Counting ═══\n");
    
    sim.reset();
    
    // Enable Timer1 with 1:1 prescaler
    // T1CON: TMR1ON=1
    sim.cpu_mut().write_register(0x10, 0x01);
    
    // Set Timer1 to 0xFFF0
    sim.cpu_mut().write_register(0x0E, 0xF0); // TMR1L
    sim.cpu_mut().write_register(0x0F, 0xFF); // TMR1H
    
    println!("Timer1 enabled with 1:1 prescaler");
    println!("Initial Timer1 value: 0x{:04X}", 
        ((sim.cpu().read_register(0x0F) as u16) << 8) | 
        (sim.cpu().read_register(0x0E) as u16));
    
    for i in 0..20 {
        sim.step().unwrap();
        let tmr1_low = sim.cpu().read_register(0x0E);
        let tmr1_high = sim.cpu().read_register(0x0F);
        let tmr1 = ((tmr1_high as u16) << 8) | (tmr1_low as u16);
        let pir1 = sim.cpu().read_register(0x0C);
        let tmr1if = (pir1 & 0x01) != 0;
        
        if i % 5 == 4 {
            println!("After {} cycles: Timer1 = 0x{:04X}, TMR1IF = {}", 
                i+1, tmr1, if tmr1if { "1" } else { "0" });
        }
    }
    
    // Demo 4: Timer1 with Prescaler
    println!("\n\n═══ Demo 4: Timer1 with 1:8 Prescaler ═══\n");
    
    sim.reset();
    
    // Enable Timer1 with 1:8 prescaler
    // T1CON: TMR1ON=1, T1CKPS=11 (1:8)
    sim.cpu_mut().write_register(0x10, 0x31);
    
    // Set Timer1 to 0
    sim.cpu_mut().write_register(0x0E, 0x00);
    sim.cpu_mut().write_register(0x0F, 0x00);
    
    println!("Timer1 with 1:8 prescaler");
    println!("Timer should increment every 8 cycles");
    
    for i in 0..40 {
        sim.step().unwrap();
        if i % 8 == 7 {
            let tmr1_low = sim.cpu().read_register(0x0E);
            let tmr1_high = sim.cpu().read_register(0x0F);
            let tmr1 = ((tmr1_high as u16) << 8) | (tmr1_low as u16);
            println!("After {} cycles: Timer1 = {}", i+1, tmr1);
        }
    }
    
    println!("\n✓ All timer demos complete!");
}