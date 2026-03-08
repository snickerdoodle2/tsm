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
      build-workflows = pkgs.runCommand "copy-workflows" {} ''
        mkdir -p $out/.github/workflows
        cp -r ${config.githubActions.workflowsDir}/* $out/.github/workflows/
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

        jobs.build = {
          runsOn = "ubuntu-latest";

          steps = [
            {
              name = "Checkout";
              uses = "actions/checkout@v4";
            }
            {
              name = "Build";
              run = "nix build";
            }
          ];
        };
      };
    };
  };
}
