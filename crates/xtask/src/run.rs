use std::{
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{eyre::bail, Result};

use crate::{
    build::{build, BuildOptions},
    ExitCodeExt,
};

pub fn run(options: RunOptions<'_>) -> Result<()> {
    fetch_limine()?;
    build_limine()?;
    build(options.build)?;

    let mut iso = PathBuf::from("target");
    iso.push(options.build.target);

    if options.build.release {
        iso.push("release");
    } else {
        iso.push("debug");
    }

    iso.push("kernel.iso");

    make_iso(&iso)?;

    let Some((arch, _)) = options.build.target.split_once('=') else {
        bail!("invalid target");
    };

    qemu_run(&iso, arch)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct RunOptions<'a> {
    pub build: BuildOptions<'a>,
}

pub fn qemu_run(iso: &Path, arch: &str) -> Result<()> {
    const QEMU_ARGS: &[&str] = &[
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
    ];

    let qemu = format!("qemu-system-{arch}");

    Command::new(qemu)
        .args(QEMU_ARGS)
        .arg(iso)
        .spawn()?
        .wait()?
        .check_ok()?;
    Ok(())
}

pub fn build_limine() -> Result<()> {
    Command::new("git")
        .arg("fetch")
        .current_dir("target/limine")
        .spawn()?
        .wait()?
        .check_ok()?;
    Command::new("make")
        .current_dir("target/limine")
        .spawn()?
        .wait()?
        .check_ok()?;
    Ok(())
}

pub fn fetch_limine() -> Result<()> {
    if Path::new("target/limine").exists() {
        return Ok(());
    }

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
        .wait()?
        .check_ok()?;

    Ok(())
}

const LIMINE_GIT_URL: &str = "https://github.com/limine-bootloader/limine.git";

fn make_iso(iso: &Path) -> Result<()> {
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
        .arg(iso)
        .spawn()?
        .wait()?
        .check_ok()?;

    Ok(())
}
