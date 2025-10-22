pub mod memory;
pub mod cpu;
pub mod instruction;
pub mod executor;
pub mod simulator;
pub mod debugger;
pub mod cli;
pub mod hexloader;
pub mod gpio;
pub mod timer;
pub mod interrupt;
pub mod wdt;
pub mod gui;

pub use memory::Memory;
pub use cpu::Cpu;
pub use instruction::{Instruction, InstructionDecoder};
pub use executor::Executor;
pub use simulator::{Simulator, SimulatorState};
pub use debugger::Debugger;
pub use cli::Cli;
pub use hexloader::{HexLoader, HexProgram, HexRecord};
pub use gpio::{Gpio, PinState};
pub use timer::{Timer0, Timer1, TimerController};
pub use interrupt::{InterruptController, InterruptSource};
pub use wdt::Wdt;

use eframe::egui;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] == "--gui" {
        run_gui();
    } else {
        run_cli();
    }
}

fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("PIC12F629/675 Simulator"),
        ..Default::default()
    };
    
    let _ = eframe::run_native(
        "pic_simulator",
        options,
        Box::new(|cc| Ok(Box::new(gui::SimulatorApp::new(cc)))),
    );
}

fn run_cli() {
    let mut cli = Cli::new();
    cli.run();
}