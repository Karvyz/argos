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

    udev
    inotify-tools
    rsync
  ];

  env.LD_LIBRARY_PATH = lib.makeLibraryPath [
    pkgs.libclang
  ];
}
