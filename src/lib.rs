//! PIC12F629/675 Microcontroller Simulator
//! 
//! This library provides a complete software simulation of the PIC12F629/675
//! 8-bit microcontroller, including:
//! 
//! - CPU core with 35 instructions
//! - 1024 words of program memory
//! - 128 bytes of data RAM
//! - 8-level hardware stack
//! - GPIO ports
//! - Timers
//! - Interrupts
//! 
//! Reference: PIC12F629/675 Data Sheet (DS41190G)

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