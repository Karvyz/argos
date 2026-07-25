{ pkgs, ... }:
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
    udev
    inotify-tools
    rsync
  ];
}
