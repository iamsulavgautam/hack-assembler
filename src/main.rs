use lazy_static::lazy_static;
use regex::Regex;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

lazy_static! {
    static ref DEST_TABLE: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("null", "000");
        map.insert("M", "001");
        map.insert("D", "010");
        map.insert("MD", "011");
        map.insert("A", "100");
        map.insert("AM", "101");
        map.insert("AD", "110");
        map.insert("AMD", "111");
        map
    };
    static ref JUMP_TABLE: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("null", "000");
        map.insert("JGT", "001");
        map.insert("JEQ", "010");
        map.insert("JGE", "011");
        map.insert("JLT", "100");
        map.insert("JNE", "101");
        map.insert("JLE", "110");
        map.insert("JMP", "111");
        map
    };
    static ref COMP_TABLE: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("0", "101010");
        map.insert("1", "111111");
        map.insert("-1", "111010");
        map.insert("D", "001100");
        map.insert("A", "110000");
        map.insert("!D", "001101");
        map.insert("!A", "110001");
        map.insert("-D", "001111");
        map.insert("-A", "110011");
        map.insert("D+1", "011111");
        map.insert("A+1", "110111");
        map.insert("D-1", "001110");
        map.insert("A-1", "110010");
        map.insert("D+A", "000010");
        map.insert("D-A", "010011");
        map.insert("A-D", "000111");
        map.insert("D&A", "000000");
        map.insert("D|A", "010101");
        map
    };
}

fn main() {
    let initial_check: (String, String) = argument_handling();
    let file: String = initial_check.0;
    let file_name: String = initial_check.1;

    let comments_and_whitespaces = Regex::new(r"(//.*)|( )").unwrap();
    let file = comments_and_whitespaces.replace_all(&file, "");
    let mut code: Vec<&str> = file.split(&['\n']).filter(|&r| r != "").collect();
    let code: Vec<String> = symbols(&mut code);

    for i in &code {
        println!("{i}");
    }

    let mut output = match File::create(&file_name) {
        Err(error) => panic!("couldn't create {file_name}.hack: {error}"),
        Ok(file) => file,
    };

    for line in code {
        let binary: String = handle_instruction(&line.to_string());
        match output.write(binary.as_bytes()) {
            Err(error) => panic!("couldn't write to {file_name}.hack: {error}"),
            Ok(_) => continue,
        };
    }

    println!("successfully assembled at: {file_name}");
}

fn symbols(code_with_symbols: &mut Vec<&str>) -> Vec<String> {
    let mut symbol_table: HashMap<&str, usize> = HashMap::new();
    symbol_table.insert("R0", 0);
    symbol_table.insert("R1", 1);
    symbol_table.insert("R2", 2);
    symbol_table.insert("R3", 3);
    symbol_table.insert("R4", 4);
    symbol_table.insert("R5", 5);
    symbol_table.insert("R6", 6);
    symbol_table.insert("R7", 7);
    symbol_table.insert("R8", 8);
    symbol_table.insert("R9", 9);
    symbol_table.insert("R10", 10);
    symbol_table.insert("R11", 11);
    symbol_table.insert("R12", 12);
    symbol_table.insert("R13", 13);
    symbol_table.insert("R14", 14);
    symbol_table.insert("R15", 15);

    symbol_table.insert("SCREEN", 16384);
    symbol_table.insert("KBD", 24576);

    symbol_table.insert("SP", 0);
    symbol_table.insert("LCL", 1);
    symbol_table.insert("ARG", 2);
    symbol_table.insert("THIS", 3);
    symbol_table.insert("THAT", 4);

    let mut line_no: usize = 1;
    let mut label_index: Vec<usize> = Vec::new();

    // reading and removing lables
    loop {
        let mut line: &str = &code_with_symbols[line_no - 1];
        line_no += 1;

        if line.contains("(") {
            line = &line[1..line.len() - 1];
            let address: usize = line_no - 2;

            symbol_table.insert(line, address);
            label_index.push(line_no - 1);
            code_with_symbols.remove(line_no - 2); // line_no starts at 1 and is inc at beginning
        }

        if line_no == code_with_symbols.len() {
            break;
        }
    }

    // reading and inserting variables if new
    let mut no_of_new_variables: usize = 0;
    let mut line_no: usize = 0;
    let mut code_without_symbols: Vec<String> = Vec::new();
    loop {
        let mut line: &str = &code_with_symbols[line_no];
        line_no += 1;

        if line.contains("@") {
            // checking if just an address
            match line[1..].parse::<usize>() {
                Ok(_) => {
                    code_without_symbols.push(line.to_string());
                    continue;
                }
                _ => (),
            }
        }

        if line.contains("@") && (symbol_table.contains_key(&line[1..line.len()]) == false) {
            let address = 16 + no_of_new_variables;
            line = &line[1..];
            symbol_table.insert(line, address);
            let new_inst: String = format!("@{}", address);

            code_without_symbols.push(new_inst);

            no_of_new_variables += 1;
            continue;
        } else if line.contains("@") && (symbol_table.contains_key(&line[1..line.len()]) == true) {
            let address = symbol_table.get(&line[1..]).unwrap();
            let new_inst = format!("@{}", address);

            code_without_symbols.push(new_inst);
            continue;
        }
        code_without_symbols.push(line.to_string());
        if line_no == code_with_symbols.len() {
            break;
        }
    }

    return code_without_symbols;
}

fn argument_handling() -> (String, String) {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("no arguments provided!")
    }

    let file_name: &String = &args[1];

    if file_name == "-h" {
        help();
    }
    let file = load_file(&file_name);

    let file: String = match file {
        Ok(code) => code,
        Err(error) => {
            panic!("error: {:?}", error)
        }
    };
    let output_file = file_name.replace("asm", "").to_owned() + &"hack";
    return (file, output_file);
}

fn handle_instruction(instruction: &String) -> String {
    if instruction.contains('@') {
        return a_instruction(instruction) + "\n";
    }
    return c_instruction(instruction) + "\n";
}

fn a_instruction(instruction: &String) -> String {
    // in the form of "@<address>"
    let instruction = instruction.replace("@", "");
    let address = instruction.parse::<u32>().unwrap();
    let address = binary_converter(address);

    let left_space: usize = 15 - address.len();
    let mut into_binary: String = String::from("0");

    for _ in 0..left_space {
        into_binary.push('0');
    }
    into_binary.push_str(&address);
    return into_binary;
}

fn c_instruction(instruction: &String) -> String {
    // syntax: "<dest>=<comp>;<jump>"
    // binary: 111 a c1 c2 c3 c4 c5 c6 d1 d2 d3 j1 j2 j3

    let mut dest: String = String::from("null");
    let mut comp: String = String::from("0");
    let mut jump: String = String::from("null");
    if instruction.contains("=") && instruction.contains(";") {
        let components: Vec<&str> = instruction
            .split(&['=', ';'])
            .filter(|&r| r != "")
            .collect();
        dest = components[0].to_string();
        comp = components[1].to_string();
        jump = components[2].to_string();
    } else if instruction.contains("=") {
        let components: Vec<&str> = instruction.split(&['=']).filter(|&r| r != "").collect();
        dest = components[0].to_string();
        comp = components[1].to_string();
    } else if instruction.contains(";") {
        let components: Vec<&str> = instruction.split(&[';']).filter(|&r| r != "").collect();
        comp = components[0].to_string();
        jump = components[1].to_string();
    }

    let dest = DEST_TABLE.get(dest.as_str()).unwrap().to_string();
    let comp = comparing_for_comp(comp.as_str());
    let jump = JUMP_TABLE.get(jump.as_str()).unwrap().to_string();

    let into_binary = String::from("111") + &comp + &dest + &jump;
    return into_binary;
}

//fn c_inst_no_dest(instruction: &String) -> Vec<String> {}

fn comparing_for_comp(comp: &str) -> String {
    // comp dictates the value of 'a' depending on the instruction
    let mut a_bit: String = String::from("1");
    match comp {
        "M" => {
            a_bit.push_str(COMP_TABLE.get(&"A").unwrap());
            return a_bit;
        }
        "!M" => {
            a_bit.push_str(COMP_TABLE.get(&"!A").unwrap());
            return a_bit;
        }
        "-M" => {
            a_bit.push_str(COMP_TABLE.get(&"-A").unwrap());
            return a_bit;
        }
        "M+1" => {
            a_bit.push_str(COMP_TABLE.get(&"A+1").unwrap());
            return a_bit;
        }
        "M-1" => {
            a_bit.push_str(COMP_TABLE.get(&"A-1").unwrap());
            return a_bit;
        }
        "D+M" => {
            a_bit.push_str(COMP_TABLE.get(&"D+A").unwrap());
            return a_bit;
        }
        "D-M" => {
            a_bit.push_str(COMP_TABLE.get(&"D-A").unwrap());
            return a_bit;
        }
        "M-D" => {
            a_bit.push_str(COMP_TABLE.get(&"A-D").unwrap());
            return a_bit;
        }
        "D&M" => {
            a_bit.push_str(COMP_TABLE.get(&"D&A").unwrap());
            return a_bit;
        }
        "D|M" => {
            a_bit.push_str(COMP_TABLE.get(&"D|A").unwrap());
            return a_bit;
        }
        &_ => {
            a_bit = String::from("0");
            a_bit.push_str(COMP_TABLE.get(comp).unwrap());
            return a_bit;
        }
    }
}

fn binary_converter(mut number: u32) -> String {
    let mut binary: String = String::new();

    while number != 0 {
        let bit = number % 2;
        number = number / 2;

        binary = bit.to_string() + &binary;
    }

    return binary;
}

fn help() {
    println!("Usage: assembler <file_name>.asm");
    exit(0);
}

fn load_file(file_name: &String) -> Result<String, io::Error> {
    let mut code = String::new();
    File::open(file_name)?.read_to_string(&mut code)?;

    Ok(code)
}
