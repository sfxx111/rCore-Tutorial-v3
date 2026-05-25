use std::io::{Result, Write};
use std::fs::{File, read_dir};

fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed=../user/target/");
    insert_app_data().unwrap();
}

static TARGET_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";

fn insert_app_data() -> Result<()> {
    let mut f = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = read_dir("../user/src/bin")
        .unwrap()
        .map(|e| {
            let s = e.unwrap().file_name().into_string().unwrap();
            s.split('.').next().unwrap().to_string()
        })
        .collect();
    apps.sort();

    writeln!(f, "
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}", apps.len())?;

    for i in 0..apps.len() {
        writeln!(f, "    .quad app_{}_start", i)?;
    }
    writeln!(f, "    .quad app_{}_end", apps.len()-1)?;

    for (i, app) in apps.iter().enumerate() {
        writeln!(f, "
app_{0}_start:
    .incbin \"{1}{2}.bin\"
app_{0}_end:
", i, TARGET_PATH, app)?;
    }
    Ok(())
}
