{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	packages = with pkgs; [
		rustc
		cargo
		rustup
		rust-analyzer
		rustfmt
		clippy
    trunk
    llvm
    lld
    bubblewrap
    pnpm
    sqlx-cli
	];

	shellHook = ''
		echo "Rust dev shell loaded"
	'';
}
