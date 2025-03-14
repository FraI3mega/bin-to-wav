{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (system:
        f {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [rust-overlay.overlays.default self.overlays.default];
          };
        });
  in {
    overlays.default = final: prev: {
      rustToolchain = let
        rust = prev.rust-bin;
      in
        if builtins.pathExists ./rust-toolchain.toml
        then rust.fromRustupToolchainFile ./rust-toolchain.toml
        else if builtins.pathExists ./rust-toolchain
        then rust.fromRustupToolchainFile ./rust-toolchain
        else
          rust.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt"];
          };
    };

    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          pkg-config
          fontconfig
          freetype
        ];
        buildInputs = with pkgs; [
        ];

        packages = with pkgs; [
          fontconfig
          rustToolchain
          openssl
          cargo-deny
          rust-analyzer
          bacon
          cargo-nextest
          cargo-bloat
          cargo-diet
          cargo-dist
        ];

        env = {
          # Required by rust-analyzer
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          PKG_CONFIG_PATH = "${pkgs.freetype.dev}/lib/pkgconfig:${pkgs.fontconfig.dev}/lib/pkgconfig";
          LD_LIBRARY_PATH = "${pkgs.fontconfig.lib}/lib:${pkgs.freetype}/lib:$LD_LIBRARY_PATH";
        };
      };
    });
  };
}
