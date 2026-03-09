{inputs, ...}: let
  init-steps = [
    {
      name = "Checkout";
      uses = "actions/checkout@v6";
    }
    {
      name = "Install Nix";
      uses = "cachix/install-nix-action@v17";
      with_ = {
        extra_nix_config = "access-tokens = github.com=\${{ secrets.GITHUB_TOKEN }}";
      };
    }
  ];
in {
  imports = [
    inputs.github-actions-nix.flakeModule
  ];

  perSystem = {
    pkgs,
    config,
    ...
  }: {
    packages = {
      workflows = pkgs.pkgs.writeShellScriptBin "copy-workflows" ''
        mkdir -p ./.github/workflows
        cp --no-preserve=mode,ownership ${config.githubActions.workflowsDir}/* .github/workflows/
      '';
      check-workflows = pkgs.pkgs.writeShellScriptBin "copy-workflows" ''
        diff ${config.githubActions.workflowsDir} .github/workflows/
      '';
    };

    githubActions = {
      enable = true;

      workflows.ci = {
        name = "CI";

        on = {
          push.branches = ["main"];
          pullRequest = {};
        };

        jobs.nix = {
          runsOn = "ubuntu-latest";

          steps =
            init-steps
            ++ [
              {
                name = "Check flake";
                run = "nix flake check";
              }
              {
                name = "Check formatting";
                run = "nix fmt . -- --check";
              }
              {
                name = "Check whether workflows are up to date";
                run = "nix run .#check-workflows";
              }
            ];
        };

        jobs.rust = {
          runsOn = "ubuntu-latest";
          steps =
            init-steps
            ++ [
              {
                name = "Setup rust cache";
                uses = "actions/cache@v5";
                with_ = {
                  path = ''
                    ~/.cargo/bin/
                    ~/.cargo/registry/index/
                    ~/.cargo/registry/cache/
                    ~/.cargo/git/db/
                    target/
                  '';
                  key = "\${{ runner.os }}-tsm-cargo-\${{ hashFiles('**/Cargo.lock') }}";
                };
              }
              {
                name = "cargo check";
                run = "nix develop .#ci -c cargo check";
              }
              {
                name = "cargo fmt";
                run = "nix develop .#ci -c cargo fmt --check";
              }
              {
                name = "cargo clippy";
                run = "nix develop .#ci -c cargo clippy";
              }
              {
                name = "cargo test";
                run = "nix develop .#ci -c cargo test";
              }
            ];
        };
      };
    };
  };
}
