// CPU 6802 Flags

#[derive(Debug, PartialEq)]
pub enum Flag {
    Carry = 0b0000_0001,
    Zero = 0b0000_0010,
    Interrupt = 0b0000_0100,
    Decimal = 0b0000_1000,
    Break = 0b0001_0000,
    Unused = 0b0010_0000,
    Overflow = 0b0100_0000,
    Negative = 0b1000_0000,
}

enum AddressingMode {
    //
}

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

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;
        loop {
            let opscode = program.get(self.program_counter as usize);
            // get opscode and go to next instruction
            self.program_counter += 1;

            if let Some(opscode) = opscode {
                match opscode {
                    // -----------------------------
                    // LOAD / STORE Operations
                    // LDA, LDX, LDY , STA, STX, STY
                    // -----------------------------

                    // LDA (Load Accumulator)
                    // Addresing mode
                    // Immediate (0xA9)
                    0xA9 => {
                        self.register_a = program[self.program_counter as usize];
                        self.program_counter += 1;

                        // set Zero flag to register A
                        // Zero flag is 0b0000_0010 or 2 (in decimal)
                        if self.register_a == 0 {
                            self.status |= Flag::Zero as u8;
                        } else {
                            // FLIP Zero flag to 0b1111_1101 for clear
                            self.status &= !(Flag::Zero as u8);
                        }

                        if self.register_a & Flag::Negative as u8 != 0 {
                            self.status |= Flag::Negative as u8;
                        } else {
                            self.status &= !(Flag::Negative as u8);
                        }
                    }

                    // END OF LOAD / STORE Operations
                    // 0xAA => {
                    //     self.register_x = self.register_a;
                    //     if self.register_x == 0 {
                    //         self.status = self.status | 0b0000_0010;
                    //     } else {
                    //         self.status = self.status & 0b1111_1101;
                    //     }

                    //     if self.register_x & 0b1000_0000 != 0 {
                    //         self.status = self.status | 0b1000_0000;
                    //     } else {
                    //         self.status = self.status & 0b0111_1111;
                    //     }
                    // }
                    0x00 => {
                        println!("BRK");
                        break;
                    }
                    _ => {
                        println!("Unrecognized opscode: {:x}", opscode);
                        break;
                    }
                }
            } else {
                println!("Program out of bound");
                break;
            }
        }
    }
}

// -----------------------------
// TEST Section
// -----------------------------

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5); // load 5 to register 5
                                       // The Zero flag should be clear
        assert_eq!(cpu.status & 0b0000_0010, 0);
        assert_eq!(cpu.status & 0b1000_0000, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status, 2); // 2 is zero flag 0b10
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x80, 0x00]);

        assert_eq!(cpu.register_a, 128);
        assert_eq!(cpu.status, 128) //
    }
}
