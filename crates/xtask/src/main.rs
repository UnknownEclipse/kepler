use std::{
    env::args,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

// use build::BuildOptions;
// // use clap::{Parser, Subcommand};
// // use color_eyre::{eyre::bail, Result};
// use run::RunOptions;

// // mod build;
// // mod check;
// // mod run;
// // mod test;

const LIMINE_GIT_URL: &str = "https://github.com/limine-bootloader/limine.git";
const KERNEL: &str = "target/x86_64-unknown-none/debug/kernel";
const KERNEL_ISO: &str = "target/x86_64-unknown-none/debug/kernel.iso";
const TARGET: &str = "x86_64-unknown-none";

#[derive(Debug)]
struct Options {
    release: bool,
    target: &'static str,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            release: false,
            target: TARGET,
        }
    }
}

impl Options {
    fn kernel(&self) -> PathBuf {
        let mut path = Path::new("target").join(self.target);
        if self.release {
            path.push("release");
        } else {
            path.push("debug");
        }
        path.push("kernel");
        path
    }

    fn kernel_iso(&self) -> PathBuf {
        let mut path = PathBuf::from("target/x86_64-unknown-none");
        if self.release {
            path.push("release");
        } else {
            path.push("debug");
        }
        path.push("kernel.iso");
        path
    }

    fn update_with_arg(&mut self, arg: &str) {
        match arg {
            "--release" => self.release = true,
            _ => panic!("invalid args: {}", arg),
        }
    }
}

// trait ExitCodeExt {
//     fn check_ok(self) -> Result<()>;
// }

// impl ExitCodeExt for ExitStatus {
//     fn check_ok(self) -> Result<()> {
//         if self.success() {
//             Ok(())
//         } else {
//             bail!("process exited with non-zero exit status")
//         }
//     }
// }

// #[derive(Debug, Parser)]
// #[command(author, version, about, long_about = None)]
// struct Cli {
//     #[arg(long)]
//     release: bool,
//     target: Option<String>,
//     #[arg(short)]
//     package: Option<String>,
//     #[command(subcommand)]
//     command: Option<Commands>,
// }

// #[derive(Debug, Subcommand)]
// enum Commands {
//     Run {},
//     Build {},
//     Test {},
//     Check {},
// }

// fn main() -> Result<()> {
//     main_old().unwrap();
//     return Ok(());
//     let cli = Cli::parse();

//     let build_options = BuildOptions {
//         package: cli.package.as_deref(),
//         release: cli.release,
//         target: cli.target.as_deref().unwrap_or(TARGET),
//     };

//     match cli.command {
//         Some(Commands::Build {}) => {
//             build::build(build_options)?;
//         }
//         Some(Commands::Run {}) => {
//             let options = RunOptions {
//                 build: build_options,
//             };
//             run::run(options)?;
//         }
//         Some(_) => unimplemented!(),
//         None => todo!(),
//     }
//     Ok(())
// }

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
        "build" => build(&options)?,
        "run" => run(&options)?,
        _ => eprintln!("unsupported xtask mode"),
    }

    Ok(())
}
fn build(options: &Options) -> Result<(), Box<dyn Error>> {
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

    dbg!(&command);
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
        options.kernel(),
        "conf/limine.cfg".into(),
        "target/limine/limine.sys".into(),
        "target/limine/limine-cd.bin".into(),
        "target/limine/limine-cd-efi.bin".into(),
    ];
    for f in iso_files {
        let mut to = Path::new("target/iso_root").join(f.file_name().unwrap());

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
        ])
        .arg(options.kernel_iso())
        .spawn()?
        .wait()?;

    Command::new("./target/limine/limine-deploy")
        .arg(options.kernel_iso())
        .spawn()?
        .wait()?;

    Ok(())
}

fn run(options: &Options) -> Result<(), Box<dyn Error>> {
    build(options)?;

    Command::new("qemu-system-x86_64")
        .args(qemu_args())
        .arg(options.kernel_iso())
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
