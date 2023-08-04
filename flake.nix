{
	description = "apply alacritty toml files to a running window";

	# Flake inputs
	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs"; # also valid: "nixpkgs"
		rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
	};

	# Flake outputs
	outputs = { self, nixpkgs, rust-overlay }:
		let
			# Overlays enable you to customize the Nixpkgs attribute set
			overlays = [
				# Makes a `rust-bin` attribute available in Nixpkgs
				(import rust-overlay)
				# Provides a `rustToolchain` attribute for Nixpkgs that we can use to
				# create a Rust environment
				(self: super: rec {
					rustToolchain = super.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
						extensions = [ "rust-analyzer" "rust-src" "rust-docs" ];
					});
					rustPlatform = super.makeRustPlatform {
						cargo = rustToolchain;
						rustc = rustToolchain;
					};
				})
			];

			# Systems supported
			allSystems = [
				"x86_64-linux" # 64-bit Intel/AMD Linux
				#"aarch64-linux" # 64-bit ARM Linux
				#"x86_64-darwin" # 64-bit Intel macOS
				#"aarch64-darwin" # 64-bit ARM macOS
			];

			# Helper to provide system-specific attributes
			forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
				pkgs = import nixpkgs { inherit overlays system; };
			});
		in
		{
			# Development environment output
			devShells = forAllSystems ({ pkgs }: {
				default = pkgs.mkShell {
					# The Nix packages provided in the environment
					packages = (with pkgs; [
						# The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
						# rustdoc, rustfmt, and other tools.
						rustToolchain
					]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);
				};
			});
			packages = forAllSystems ({ pkgs }: {
				default = pkgs.rustPlatform.buildRustPackage {
					name = "ala-apply";
					src = ./.;
					cargoLock = {
						lockFile = ./Cargo.lock;
					};
				};
			});
		};
}
