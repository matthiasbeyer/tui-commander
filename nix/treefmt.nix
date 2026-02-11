{ pkgs, ... }:
{
  projectRootFile = "flake.nix";
  settings.excludes = [
    ".gitignore"
    ".gitlint"
    "LICENSE"
    "Cargo.toml"
  ];
  programs = {
    rustfmt.enable = true;
    nixfmt.enable = true;
    yamlfmt.enable = true;
    mdformat.enable = true;
  };
}
