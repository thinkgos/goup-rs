{
  description = "A flake for goup-rs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-parts.url = "github:hercules-ci/flake-parts"; # https://flake.parts/
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        {
          self',
          system,
          pkgs,
          lib,
          ...
        }:
        let
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.default;
            rustc = pkgs.rust-bin.stable.latest.default;
          };
        in
        {
          # https://flake.parts/overlays.html#consuming-an-overlay
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
          packages.default = rustPlatform.buildRustPackage (finalAttrs: {
            inherit nativeBuildInputs;

            pname = "goup-rs";
            version = self'.shortRev or "dev";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            # Disable cargo-auditable until https://github.com/rust-secure-code/cargo-auditable/issues/124 is fixed
            auditable = false;

            meta = {
              homepage = "https://github.com/thinkgos/goup-rs";
              description = "goup is an elegant Go version manager write in rust.";
              license = with lib.licenses; [
                asl20
                mit
              ];
              mainProgram = "goup";
            };
          });
          devShells.default = pkgs.mkShell {
            name = "develop-shell";
            # 直接继承 packages 里的依赖
            inputsFrom = [ self'.packages.default ];
            packages = [
            ];
            env = {
            };
            # SHELL = "${pkgs.zsh}/bin/zsh";
            shellHook = ''
              echo "Rust development shell ready! 🦀 $(rustc --version)"
            '';
          };
        };
    };
}
