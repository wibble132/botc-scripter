{
  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url       = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naersk' = pkgs.callPackage naersk {
          cargo = rust;
          rustc = rust;
        };

      in {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
        };

        devShells.default = with pkgs; mkShell {
          buildInputs = [
            rust
          ];

          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
          shellHook = ''
            ln -sfn ${rust} .toolchain
          '';
        };
      }
    );
}