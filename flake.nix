{
  inputs = {
    harbor-rs.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=fac8049316846e0ef1c1e6acd92aed7a337b333a";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    plinth = {
      url = "git+https://codeberg.org/caniko/plinth.git?ref=refs/heads/trunk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, harbor-rs, rust-overlay, plinth }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          website = plinth.lib.${system}.mkProjectSite {
            pname = "timer-data-website";
            domain = "timer-data.tartanoglu.com";
            configPath = ./website/plinth-project.toml;
          };
        in {
          inherit website;
          site = website;
        });
      apps = forAllSystems (system: {
        deploy-pages = plinth.lib.${system}.mkDeployPagesApp {
          domain = "timer-data.tartanoglu.com";
        };
      });
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };

          toolchain = harbor-rs.lib.mkToolchain { inherit pkgs; toolchainProfile = "stable"; };
          rustToolchain = toolchain.rustToolchain;
        in
        {
          default = pkgs.mkShell {
            buildInputs = [ rustToolchain ];
          };
        }
      );
    };
}
