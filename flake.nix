{
  inputs = {
    rs-harbor.url = "git+https://github.com/caniko/harbor-rs.git?ref=trunk&rev=05cc4f162b55fa904b687db1821e2463fa813e50";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    plinth = {
      url = "git+https://github.com/caniko/plinth.git?ref=refs/heads/trunk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rs-harbor, rust-overlay, plinth }:
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

          rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" "rustfmt" "rustc-codegen-cranelift-preview" ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [ rustToolchain ];
          };
        }
      );
    };
}
