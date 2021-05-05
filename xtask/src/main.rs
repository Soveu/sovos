use std::env;
use std::error::Error;
use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::os::unix::process::CommandExt;

type Return = std::result::Result<(), Box<dyn Error>>;

macro_rules! brint {
    ($fmt:expr $(, $($arg:tt)* )?) => {{
        print!(concat!("\x1B[1;96m[XTASK]\x1B[0m ", $fmt) $(,$($arg)* )*);
    }}
}

fn print_help() -> Return {
    print!("Use these commands for xtask:\n\n");
    println!("build");
    println!("run");
    println!("clean [all, kernel, uefi_wrapper]");
    Ok(())
}

fn build(mut current_dir: PathBuf) -> Return {

    assert!(current_dir.is_absolute());
    current_dir.push("kernel");

    brint!("Building kernel\n");

    let status = Command::new("cargo")
        .current_dir(&current_dir)
        .args(&["build", "--release"])
        .status()?;

    brint!("Cargo finished with {}\n", status);
    assert!(status.success());


    assert!(current_dir.pop());

    let mut kernel_path = current_dir.clone();
    kernel_path.push("kernel/target/amd64-kernel-none/release/kernel");
    current_dir.push("uefi_wrapper");

    brint!("Building uefi_wrapper\n");
    let status = Command::new("cargo")
        .current_dir(&current_dir)
        .args(&["build", "--release"])
        .env("SOVOS_KERNEL_PATH", &kernel_path)
        .status()?;

    brint!("Cargo finished with {}\n", status);
    assert!(status.success());
    assert!(current_dir.pop());

    Ok(())

}

fn build_run_directory(current_dir: PathBuf) -> Return {
    brint!("Building FAT directory structure\n");

    let mut boot = current_dir.clone();
    boot.push("fat/EFI/BOOT");
    fs::create_dir_all(&boot)?;
    boot.push("BOOTx64.EFI");

    let mut image = current_dir;
    image.push("uefi_wrapper/target/x86_64-unknown-uefi/release/uefi_wrapper.efi");
    
    if fs::metadata(&boot).is_err() {
        use std::os::unix::fs::symlink;
        brint!("Symlinking {} to {}\n", image.display(), boot.display());
        symlink(&image, &boot)?;
    } else {
        brint!("Symlink exists, skipping ({})\n", boot.display());
    }

    Ok(())
}

fn run(current_dir: PathBuf) -> Return {
    build(current_dir.clone())?;
    build_run_directory(current_dir.clone())?;

    brint!("Running QEMU (execve)\n");
    let qemu_args = [
        "-drive", "if=pflash,format=raw,read-only,file=/usr/share/edk2/ovmf/OVMF_CODE.fd",
        "-drive", "format=raw,file=fat:rw:fat/",
        "-enable-kvm",
        "-cpu", "host",
        "-m", "8G",
        "-nographic",
        "-d", "int,cpu_reset,guest_errors",
        "-no-reboot",
        //"-s", "-S",
    ];
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args(&qemu_args);

    brint!("{:?}\n\n", qemu);
    brint!("To exit QEMU press Ctrl+a x, or Ctrl+a h for help\n");

    return Err(qemu.exec().into());
}

fn clean(mut current_dir: PathBuf, clean_target: &str) -> Return {
    if clean_target.len() == 0 {
        return print_help();
    }

    assert!(current_dir.is_absolute());

    if clean_target == "all" || clean_target == "kernel" {
        current_dir.push("kernel");

        brint!("Cleaning kernel\n");

        let status = Command::new("cargo")
            .current_dir(&current_dir)
            .arg("clean")
            .status()?;

        brint!("Cargo finished with {}\n", status);
        assert!(status.success());
        assert!(current_dir.pop());
    }


    if clean_target == "all" || clean_target == "uefi_wrapper" {
        current_dir.push("uefi_wrapper");

        brint!("Cleaning uefi_wrapper\n");
        let status = Command::new("cargo")
            .current_dir(&current_dir)
            .arg("clean")
            .status()?;

        brint!("Cargo finished with {}\n", status);
        assert!(status.success());
        assert!(current_dir.pop());
    }

    if clean_target == "all" || clean_target == "fat" {
        current_dir.push("fat");
        if fs::metadata(&current_dir).is_ok() {
            brint!("Cleaning fat/ directory\n");
            fs::remove_dir_all(&current_dir)?;
        } else {
            brint!("fat/ directory non-existant, skipping\n");
        }
        assert!(current_dir.pop());
    }

    Ok(())
}

fn main() -> Return {
    let current_dir = env::current_dir().unwrap();
    print!("Current directory: {:?}\n", current_dir.display());

    let args: Vec<String> = env::args().skip(1).collect();
    print!("Args: {:?}\n\n", args);

    let (first_arg, rest) = match args.split_first() {
        Some(x) => x,
        None => return print_help(),
    };

    return match first_arg.as_str() {
        "build" => build(current_dir),
        "run" => run(current_dir),
        "clean" => clean(current_dir, rest.get(0).map(|s| s.as_str()).unwrap_or("")),
        _ => print_help(),
    };
}
