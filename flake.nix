{
  description = "A blazingly fast client SDK for Gameforge Auth APIs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {flake-parts, ...} @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
      ];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        fenix = inputs'.fenix.packages;
        toolchain = fenix.stable.withComponents [
          "cargo"
          "rustc"
          "rust-src"
          "clippy"
          "rustfmt"
        ];
      in {
        devShells.default = pkgs.mkShell {
          name = "gf-auth";
          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
            rustup
            # toolchain // Replace rustup when RustRover supports rustfmt outside rustup
          ];
        };
      };
    };
}
