{
  description = "The tui-commander project";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
    unstable-nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (_: _: { } // inputs.self.packages."${system}")
            (import inputs.rust-overlay)
          ];
        };

        callPackage = pkgs.lib.callPackageWith (
          pkgs
          // {
            inherit
              callPackage
              buildInputs
              craneLib
              src
              version
              ;
          }
        );

        unstable = import inputs.unstable-nixpkgs {
          inherit system;
        };

        nightlyRustTarget = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          pkgs.rust-bin.fromRustupToolchain {
            channel = "nightly-2024-12-16";
            components = [ "rustfmt" ];
          }
        );

        nightlyCraneLib = (inputs.crane.mkLib pkgs).overrideToolchain nightlyRustTarget;

        rustTarget = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustTarget;

        tomlInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; };
        inherit (tomlInfo) version;

        pname = "tui-commander";

        src =
          let
            nixFilter = path: _type: !pkgs.lib.hasSuffix ".nix" path;
            extraFiles =
              path: _type:
              !(builtins.any (n: pkgs.lib.hasSuffix n path) [
                ".github"
                ".sh"
              ]);
            filterPath =
              path: type:
              builtins.all (f: f path type) [
                nixFilter
                extraFiles
                pkgs.lib.cleanSourceFilter
              ];
          in
          pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = filterPath;
          };

        buildInputs = [
          pkgs.pkg-config
          pkgs.fontconfig
          pkgs.wayland-protocols
        ];

        nativeBuildInputs = [
          pkgs.notmuch
        ];

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src pname buildInputs;
        };

        tui-commander = craneLib.buildPackage {
          inherit
            cargoArtifacts
            src
            pname
            version
            buildInputs
            nativeBuildInputs
            ;
          cargoExtraArgs = "--all-features -p tui-commander";
        };

        tui-commander-doc = craneLib.cargoDoc {
          inherit
            cargoArtifacts
            src
            pname
            version
            buildInputs
            ;
          cargoExtraArgs = "--document-private-items -p tui-commander";
        };

        rustfmt' = pkgs.writeShellScriptBin "rustfmt" ''
          exec "${nightlyRustTarget}/bin/rustfmt" "$@"
        '';

        customCargoMultiplexer = pkgs.writeShellScriptBin "cargo" ''
          case "$1" in
            +nightly)
              shift
              export PATH="${nightlyRustTarget}/bin/:''$PATH"
              exec ${nightlyRustTarget}/bin/cargo "$@"
              ;;
            *)
              exec ${rustTarget}/bin/cargo "$@"
          esac
        '';

        treefmt = inputs.treefmt-nix.lib.evalModule pkgs ./nix/treefmt.nix;
      in
      rec {
        formatter = treefmt.config.build.wrapper;

        checks = {
          inherit tui-commander;

          tui-commander-clippy = craneLib.cargoClippy {
            inherit cargoArtifacts src pname;
            cargoClippyExtraArgs = "--benches --examples --tests --all-features -- --deny warnings";
          };

          tui-commander-fmt = nightlyCraneLib.cargoFmt {
            inherit src pname;
          };

          formatting = treefmt.config.build.check inputs.self;
        };

        packages = {
          default = packages.tui-commander;
          inherit tui-commander tui-commander-doc;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            customCargoMultiplexer
            rustfmt'
            rustTarget

            pkgs.cargo-insta
            pkgs.cargo-deny
            pkgs.gitlint
            pkgs.statix
          ];
        };
      }
    );
}
