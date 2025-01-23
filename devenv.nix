{pkgs, ...}: {
  packages = with pkgs; [
    git
    qemu
    pkgsCross.avr.buildPackages.gcc
    avrdude
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "rust-src"
      "rust-std"
      "llvm-tools"
    ];

    targets = [
      "x86_64-unknown-none"
    ];
  };

  env.AVR_CPU_FREQUENCY_HZ = "16_000_000";
}
