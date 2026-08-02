{ lib, pkgs, ... }:
{
  languages = {
    rust = {
      enable = true;
      channel = "stable";
      components = [
        "rustc"
        "cargo"
        "clippy"
        "rustfmt"
        "rust-analyzer"
      ];
      mold.enable = true;
    };
  };

  packages = with pkgs; [
    openssl
    alsa-lib
    cmake
    rsync
    libcamera
  ];

  env.LD_LIBRARY_PATH = lib.makeLibraryPath [
    pkgs.libclang
    pkgs.libcamera
  ];

  processes.sync = {
    exec = "rsync . karviz@raspberrypi.home:/home/karviz/argos/ -av --delete --no-owner --no-group --no-perms --exclude='target/' --exclude='.git/' --exclude='.devenv/' --exclude='.gitignore'";
    watch = {
      paths = [
        ./src
        ./Cargo.toml
      ];
    };
  };
}
