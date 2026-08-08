{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	packages = with pkgs; [
		rustc
		cargo
		rust-analyzer
		rustfmt
		clippy
		rustup
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
