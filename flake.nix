{
  description = "Oxocarbon VS Code theme compiler (Rust) with dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustPlatform = pkgs.rustPlatform;
      in
      rec {
        packages.default = rustPlatform.buildRustPackage {
          pname = "oxocarbon-themec";
          version = "0.1.0";
          src = builtins.path { path = ./.; name = "source"; };
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        apps.default = {
          type = "app";
          program = "${packages.default}/bin/oxocarbon-themec";
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rust-analyzer
            gnumake
            nodejs_24
            vsce
            jq
          ];
        };
      }
    );
}


