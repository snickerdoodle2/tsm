{
  description = "Tmux Session Manager";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs @ {
    flake-parts,
    rust-overlay,
    crane,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
            p.rust-bin.stable.latest.default
        );
      in {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };
        formatter = pkgs.alejandra;
        packages = rec {
          default = binary;
          binary = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
          };
        };
        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              just
              (rust-bin.stable.latest.default.override
                {
                  extensions = ["rust-analyzer" "rust-src"];
                })
            ];
          };
        };
      };
    };
}
