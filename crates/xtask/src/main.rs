use std::{
    env::args,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, CommandArgs},
};

const LIMINE_GIT_URL: &str = "https://github.com/limine-bootloader/limine.git";
const KERNEL: &str = "target/x86_64-unknown-none/debug/kernel";
const KERNEL_ISO: &str = "target/x86_64-unknown-none/debug/kernel.iso";

#[derive(Debug, Default)]
struct Options {
    release: bool,
}

impl Options {
    fn update_with_arg(&mut self, arg: &str) {
        match arg {
            "--release" => self.release = true,
            _ => panic!("invalid args: {}", arg),
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let mode = args().nth(1).unwrap();
    let mut args = args();
    _ = args.next();

    let mode = args.next().unwrap();

    let mut options = Options::default();
    for arg in args {
        options.update_with_arg(&arg);
    }

    match mode.as_str() {
        "build" => build(options)?,
        "run" => run(options)?,
        _ => eprintln!("unsupported xtask mode"),
    }

    Ok(())
}

fn build(options: Options) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("cargo");
    command
        .arg("+nightly")
        .arg("build")
        .arg("-Z")
        .arg("build-std=core,alloc,compiler_builtins")
        .arg("--target")
        .arg("x86_64-unknown-none")
        .arg("--bin")
        .arg("kernel");

    if options.release {
        command.arg("--release");
    }

    command.spawn()?.wait()?;

    if !Path::new("target/limine").exists() {
        // git clone $LIMINE_GIT_URL --depth=1 --branch v3.0-branch-binary target/limine
        Command::new("git")
            .args([
                "clone",
                LIMINE_GIT_URL,
                "--depth=1",
                "--branch",
                "v3.0-branch-binary",
                "target/limine",
            ])
            .spawn()?
            .wait()?;
    }

    _ = fs::create_dir("target/iso_root");

    let iso_files = [
        KERNEL,
        "conf/limine.cfg",
        "target/limine/limine.sys",
        "target/limine/limine-cd.bin",
        "target/limine/limine-cd-efi.bin",
    ];
    for f in iso_files {
        let mut to = Path::new("target/iso_root").join(Path::new(f).file_name().unwrap());

        fs::copy(f, to)?;
    }
    // xorriso -as mkisofs                                             \
    // -b limine-cd.bin                                            \
    // -no-emul-boot -boot-load-size 4 -boot-info-table            \
    // --efi-boot limine-cd-efi.bin                                \
    // -efi-boot-part --efi-boot-image --protective-msdos-label    \
    // target/iso_root -o $KERNEL.iso

    Command::new("xorriso")
        .args([
            "-as",
            "mkisofs",
            "-b",
            "limine-cd.bin",
            "-no-emul-boot",
            "-boot-load-size",
            "4",
            "-boot-info-table",
            "--efi-boot",
            "limine-cd-efi.bin",
            "-efi-boot-part",
            "--efi-boot-image",
            "--protective-msdos-label",
            "target/iso_root",
            "-o",
            KERNEL_ISO,
        ])
        .spawn()?
        .wait()?;

    Command::new("./target/limine/limine-deploy")
        .arg(KERNEL_ISO)
        .spawn()?
        .wait()?;

    Ok(())
}

fn run(options: Options) -> Result<(), Box<dyn Error>> {
    build(options)?;

    Command::new("qemu-system-x86_64")
        .args(qemu_args())
        .arg(KERNEL_ISO)
        .spawn()?
        .wait()?;

    Ok(())
}

fn test() -> Result<(), Box<dyn Error>> {
    todo!()
}

fn qemu_args() -> &'static [&'static str] {
    &[
        "-machine",
        "q35",
        "-cpu",
        "qemu64,+rdrand",
        "-M",
        "smm=off",
        "-D",
        "target/log.txt",
        "-d",
        "int,guest_errors",
        "-no-reboot",
        "-no-shutdown",
        "-serial",
        "stdio",
        "-drive",
        "file=nvm.img,if=none,id=nvm",
        "-device",
        "nvme,serial=deadbeef,drive=nvm",
        "-smp",
        "8",
    ]
}
