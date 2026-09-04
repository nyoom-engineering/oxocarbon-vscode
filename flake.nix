{
  description = "Oxocarbon theme compiler and converters";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        inherit (pkgs) rustPlatform;
        src = builtins.path {
          path = ./.;
          name = "oxocarbon-vscode";
        };

        rustTools = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
        ];
      in
      rec {
        packages.default = rustPlatform.buildRustPackage {
          pname = "oxocarbon-themec";
          version = "0.1.0";
          inherit src;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "oxocarbon-themec" ];
          cargoTestFlags = [ "-p" "oxocarbon-themec" "-p" "oxocarbon-utils" ];
        };

        apps.default = {
          type = "app";
          program = "${packages.default}/bin/oxocarbon-themec";
        };

        devShells.default = pkgs.mkShell {
          packages = rustTools ++ (with pkgs; [
            gnumake
            pkg-config
            nodejs_24
            vsce
            jq
            python3
          ]);
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      }
    );
}


