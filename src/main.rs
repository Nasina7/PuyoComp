mod decompress;
mod compress;

use decompress::DecompressInstance;
use compress::CompressInstance;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let opt;
    let src;
    let dst;
    if args.len() < 3 {
        println!("PuyoComp by Nasina7");
        println!("Usage: puyocomp [-d|-c] src_file.bin dst_file.bin");
        println!("Args:");
        println!("    -d : Decompress src_file.bin and save it as dst_file.bin");
        println!("    -c : Compress src_file.bin and save it as dst_file.bin");
        println!("If no parameter is provided for dst_file.bin, it will be saved as src_file.bin, overwriting the original file.");
        return;
    } else if args.len() == 3 {
        opt = args[1].clone();
        src = args[2].clone();
        dst = args[2].clone();
    } else if args.len() == 4 {
        opt = args[1].clone();
        src = args[2].clone();
        dst = args[3].clone();
    } else {
        println!("Invalid number of arguments! {}", args[3]);
        return;
    }

    if opt == "-d" {
        let mut dec = DecompressInstance::new(&src);
        let out = dec.decompress();
        let res = std::fs::write(&dst, out);
        match res {
            Result::Err(e) => {
                println!("An error occured while saving the file: {}", e);
                return;
            }
            _ => {}
        }
    } else if opt == "-c" {
        let mut cmp = CompressInstance::new(&src);
        let out = cmp.compress();
        let res = std::fs::write(&dst, out);
        match res {
            Result::Err(e) => {
                println!("An error occured while saving the file: {}", e);
                return;
            }
            _ => {}
        }
    } else {
        println!("Invalid operation passed!");
        println!("Valid operations are '-d' and '-c'");
        return;
    }
}