{
  description = "Env for sapiens";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustVersion = "latest";
        rustChannel = "nightly";
        #rustChannel = "stable";
        #rustVersion = "1.62.0";
        rust = pkgs.rust-bin.${rustChannel}.${rustVersion}.default.override {
          extensions = [
            "rust-src" # for rust-analyzer
          ];
        };

        testAndLint = pkgs.writeShellScriptBin "testAndLint" ''
          set -euo pipefail
          # Run checks
          cargo validate
          # Run tests
          cargo test
          # Run lints
          cargo lint
        '';

      in
      {
        formatter = nixpkgs.legacyPackages.${system}.nixpkgs-fmt;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
          ] ++ (with pkgs; [
            llvmPackages.bintools
            bashInteractive
            python312
            protobuf
            just
            rust-analyzer
            rustc
            cargo-edit
            cargo-machete
            cargo-insta
            cargo-watch
            cargo-deny
            testAndLint
            watchexec
          ] ++ # if darwin
          (if system == "aarch64-darwin" then [
            darwin.apple_sdk.frameworks.SystemConfiguration
          ] else [ ])
          );
        };
      });
}
