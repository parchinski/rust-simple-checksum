// I Bryant Parchinski (br552182) affirm that
// this program is entirely my own work and that I have neither developed my code with any
// another person, nor copied any code from any other person, nor permitted my code to be copied
// or otherwise used by any other person, nor have I copied, modified, or otherwise used programs
// created by others. I acknowledge that any violation of the above terms will be treated as
// academic dishonesty.

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Expected 2 arguments: <filename> and <checksumSize>");
        process::exit(1);
    }

    let input_file_name = &args[1];

    let checksum_bit_size: u32 = match args[2].parse() {
        Ok(size) => {
            if size != 8 && size != 16 && size != 32 {
                eprintln!("Valid checksum sizes are 8, 16, or 32");
                process::exit(1);
            }
            size
        }
        Err(_) => {
            eprintln!("Checksum size must be a number");
            process::exit(1);
        }
    };
    
    let file_contents = match read_file(input_file_name) {
        Ok(file_contents) => file_contents,
        Err(err) => {
            eprintln!("Error reading {}: {}", input_file_name, err);
            process::exit(1);
        }
    };
    
    // print the input file text in 80 char chunks
    for chunk in file_contents.chunks(80) {
        // print regardless of valid utf8 found this to be super cool
        println!("{}", String::from_utf8_lossy(chunk));
    }

    // figure out how much padding or X's to add to the file contents
    let padding_size = match checksum_bit_size {
        8 => 0, // no padding needed
        16 => if file_contents.len() % 2 != 0 { 1 } else { 0 }, // add 1 byte if file_len%2 is odd
        32 => (4 - (file_contents.len() % 4)) % 4, // this one was ... we don't talk about it
        _ => unreachable!()
    };

    // add the X's to the file contents to prepare for checksum
    let mut padded_contents = file_contents.clone();

    // add padding_size Xs to the end of byte vector
    padded_contents.extend(vec![b'X'; padding_size]);

    // calculate the checksum with padding added to the vector for clean checksum generation
    let final_checksum = calculate_checksum(&padded_contents, checksum_bit_size);

    // print the checksum
    match checksum_bit_size {
        8 => println!("{:2} bit checksum is {:02x} for all {:4} chars", 
                     checksum_bit_size, final_checksum & 0xFF, file_contents.len()),
        16 => println!("{:2} bit checksum is {:04x} for all {:4} chars",
                      checksum_bit_size, final_checksum & 0xFFFF, file_contents.len()),
        32 => println!("{:2} bit checksum is {:08x} for all {:4} chars",
                      checksum_bit_size, final_checksum, file_contents.len()),
        _ => unreachable!()
    }
}

// Take a file path as string slice 
// Return an io result containing a vector of bytes
fn read_file(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?; // open file return if err
    let mut file_contents = Vec::new(); // create a new vector of bytes
    file.read_to_end(&mut file_contents)?; // read contents into vector return if err
    Ok(file_contents)
}

// Take a slice of bytes as file_data and a u32 integer size
// return a u32 checksum
fn calculate_checksum(file_data: &[u8], size: u32) -> u32 {
    let mut checksum: u32 = 0;

    match size {
        8 => {
            // Process one byte at a time for 8-bit checksum
            for &byte in file_data {
                checksum = checksum.wrapping_add(byte as u32);
                checksum &= 0xFF; // Mask after each addition for 8-bit
            }
        }
        16 => {
            // Process two bytes at a time for 16-bit checksum
            for chunk in file_data.chunks(2) {
                let value = if chunk.len() == 2 {
                    ((chunk[0] as u32) << 8) | (chunk[1] as u32)
                } else {
                    (chunk[0] as u32) << 8 // Handle last byte if odd length
                };
                checksum = checksum.wrapping_add(value);
            }
            checksum &= 0xFFFF; // Final mask for 16-bit
        }
        32 => {
            // Process four bytes at a time for 32-bit checksum
            for chunk in file_data.chunks(4) {
                let value = match chunk.len() {
                    4 => ((chunk[0] as u32) << 24) | ((chunk[1] as u32) << 16) |
                         ((chunk[2] as u32) << 8) | (chunk[3] as u32),
                    3 => ((chunk[0] as u32) << 24) | ((chunk[1] as u32) << 16) |
                         ((chunk[2] as u32) << 8),
                    2 => ((chunk[0] as u32) << 24) | ((chunk[1] as u32) << 16),
                    1 => (chunk[0] as u32) << 24,
                    _ => unreachable!()
                };
                checksum = checksum.wrapping_add(value);
            }
        }
        _ => unreachable!()
    }
    
    checksum
}
