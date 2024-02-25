// CPU 6802 Flags

#[derive(Debug, PartialEq)]
pub enum Flag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Unused,
    Overflow,
    Negative,
}

// Rest of the code...
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8,
    pub program_counter: u16,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            status: 0,
            program_counter: 0,
        }
    }

    fn select_flag(flag: Flag) -> u8 {
        match flag {
            Flag::Zero => 0b0000_0010,
            Flag::Carry => 0b0000_0001,
            Flag::Interrupt => 0b0000_0100,
            Flag::Decimal => 0b0000_1000,
            Flag::Break => 0b0001_0000,
            Flag::Unused => 0b0010_0000,
            Flag::Overflow => 0b0100_0000,
            Flag::Negative => 0b1000_0000,
        }
    }

    // using for flip the binary
    fn flip_flag(flag: Flag) -> u8 {
        !Self::select_flag(flag)
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;
        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;

            match opscode {
                // LOAD / STORE Operations
                // LDA, LDX, LDY , STA, STX, STY

                // 0xA9 = LDA (Load Accumulator)
                //
                0xA9 => {
                    self.register_a = program[self.program_counter as usize];
                    self.program_counter += 1;

                    // set Zero flag to register A
                    // Zero flag is 0b0000_0010 or 2 (in decimal)
                    if self.register_a == 0 {
                        self.status |= Self::select_flag(Flag::Zero);
                    } else {
                        // FLIP Zero flag to 0b1111_1101 for clear
                        self.status &= Self::flip_flag(Flag::Zero);
                    }

                    if self.register_a & Self::select_flag(Flag::Negative) != 0 {
                        self.status |= Self::select_flag(Flag::Negative);
                    } else {
                        self.status &= Self::flip_flag(Flag::Negative);
                    }
                }

                // END OF LOAD / STORE Operations
                0xAA => {
                    self.register_x = self.register_a;
                    if self.register_x == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    if self.register_x & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                }

                0x00 => {
                    println!("BRK");
                    break;
                }
                _ => {
                    println!("Unrecognized opscode: {}", opscode);
                    break;
                }
            }
        }
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
    }
}
