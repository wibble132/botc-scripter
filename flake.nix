{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      naersk,
      ...
    }:
    let
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        inherit overlays;
      };

      rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      naersk' = pkgs.callPackage naersk {
        cargo = rust;
        rustc = rust;
      };
    in
    {
      packages.x86_64-linux = {
        default = naersk'.buildPackage {
          src = ./.;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
          cargoBuildOptions = opts: opts ++ [ "--features" "static-data" ];
        };
        dynamic = naersk'.buildPackage {
          src = ./.;
          cargoBuildOptions = opts: opts ++ [ "--features" "static-data" ];
        };
      };

      devShells.x86_64-linux.default =
        with pkgs;
        mkShell {
          buildInputs = [
            rust
          ];

          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

          # Link the toolchain locally so dev tools can keep a static path when it changes
          shellHook = ''
            ln -sfn ${rust} .toolchain
          '';
        };
    };
}
