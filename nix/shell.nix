{
  perSystem = {pkgs, ...}: {
    devShells.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override
          {
            extensions = ["rust-analyzer" "rust-src"];
          })
      ];
    };
  };
}
