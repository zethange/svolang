use std::env;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
enum OpCode {
    IncrementPointer,
    DecrementPointer,
    Increment,
    Decrement,
    Write,
    Read,
    LoopBegin,
    LoopEnd,
}

#[derive(Debug, Clone)]
enum Instruction {
    IncrementPointer,
    DecrementPointer,
    Increment,
    Decrement,
    Write,
    Read,
    Loop(Vec<Instruction>),
}

fn lex(source: String) -> Vec<OpCode> {
    let mut operations = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == 's' && chars[i + 1] == 'v' {
            let mut o_count = 0;
            let mut j = i + 2;

            while j < chars.len() && chars[j] == 'o' {
                o_count += 1;
                j += 1;
            }

            let op = match o_count {
                1 => Some(OpCode::Increment),        // svo
                2 => Some(OpCode::Decrement),        // svoo
                3 => Some(OpCode::LoopBegin),        // svooo
                4 => Some(OpCode::LoopEnd),          // svoooo
                5 => Some(OpCode::DecrementPointer), // svooooo
                6 => Some(OpCode::IncrementPointer), // svoooooo
                7 => Some(OpCode::Write),            // svooooooo
                8 => Some(OpCode::Read),             // svoooooooo
                _ => None,
            };

            if let Some(op) = op {
                operations.push(op);
            }

            i = j;
        } else {
            i += 1;
        }
    }

    operations
}

fn parse(opcodes: Vec<OpCode>) -> Vec<Instruction> {
    let mut program: Vec<Instruction> = Vec::new();
    let mut loop_stack = 0;
    let mut loop_start = 0;

    for (i, op) in opcodes.iter().enumerate() {
        if loop_stack == 0 {
            let instr = match op {
                OpCode::IncrementPointer => Some(Instruction::IncrementPointer),
                OpCode::DecrementPointer => Some(Instruction::DecrementPointer),
                OpCode::Increment => Some(Instruction::Increment),
                OpCode::Decrement => Some(Instruction::Decrement),
                OpCode::Write => Some(Instruction::Write),
                OpCode::Read => Some(Instruction::Read),

                OpCode::LoopBegin => {
                    loop_start = i;
                    loop_stack += 1;
                    None
                }

                OpCode::LoopEnd => panic!("loop ending at #{} has no beginning", i),
            };

            match instr {
                Some(instr) => program.push(instr),
                None => (),
            }
        } else {
            match op {
                OpCode::LoopBegin => {
                    loop_stack += 1;
                }
                OpCode::LoopEnd => {
                    loop_stack -= 1;

                    if loop_stack == 0 {
                        program.push(Instruction::Loop(parse(
                            opcodes[loop_start + 1..i].to_vec(),
                        )));
                    }
                }
                _ => (),
            }
        }
    }

    if loop_stack != 0 {
        panic!(
            "loop that starts at #{} has no matching ending!",
            loop_start
        );
    }

    program
}

fn run(instructions: &Vec<Instruction>, tape: &mut Vec<u8>, data_pointer: &mut usize) {
    for instr in instructions {
        match instr {
            Instruction::IncrementPointer => *data_pointer += 1,
            Instruction::DecrementPointer => *data_pointer -= 1,
            Instruction::Increment => tape[*data_pointer] += 1,
            Instruction::Decrement => tape[*data_pointer] -= 1,
            Instruction::Write => print!("{}", tape[*data_pointer] as char),
            Instruction::Read => {
                let mut input: [u8; 1] = [0; 1];
                std::io::stdin()
                    .read_exact(&mut input)
                    .expect("failed to read stdin");
                tape[*data_pointer] = input[0];
            }
            Instruction::Loop(nested_instructions) => {
                while tape[*data_pointer] != 0 {
                    run(&nested_instructions, tape, data_pointer)
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!("usage: svolang\n\trun <file.svo>\n\ttranslate <file.bf> <file.svo>");
        std::process::exit(1);
    }

    let command = &args[1];

    if command == "run" {
        let filename = &args[2];
        let mut file = File::open(filename).expect("program file not found");
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("failed to read program file");

        let opcodes = lex(source);

        let program = parse(opcodes);

        let mut tape: Vec<u8> = vec![0; 1024];
        let mut data_pointer = 512;

        run(&program, &mut tape, &mut data_pointer);
    } else if command == "translate" {
        let from_filename = &args[2];
        let to_filename = &args[3];

        let mut file = File::open(from_filename).expect("program file not found");
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("failed to read program file");

        let result = source
            .replace("+", "svo")
            .replace("-", "svoo")
            .replace("[", "svooo")
            .replace("]", "svoooo")
            .replace("<", "svooooo")
            .replace(">", "svoooooo")
            .replace(".", "svooooooo")
            .replace(",", "svoooooooo");

        let mut file_write = File::create(to_filename).expect("error create svo file");
        file_write
            .write(result.as_bytes())
            .expect("error write to svo file");
    }
}
