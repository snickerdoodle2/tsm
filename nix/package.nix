{inputs, ...}: {
  perSystem = {pkgs, ...}: let
    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (
      p:
        p.rust-bin.stable.latest.default
    );
  in {
    packages = rec {
      default = binary;
      binary = craneLib.buildPackage {
        src = craneLib.cleanCargoSource ./..;
      };
    };
  };
}
