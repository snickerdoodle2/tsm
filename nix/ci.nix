{inputs, ...}: {
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

          steps = [
            {
              name = "Checkout";
              uses = "actions/checkout@v4";
            }
            {
              name = "Install Nix";
              uses = "cachix/install-nix-action@v17";
              with_ = {
                extra_nix_config = "access-tokens = github.com=\${{ secrets.GITHUB_TOKEN }}";
              };
            }
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
      };
    };
  };
}
