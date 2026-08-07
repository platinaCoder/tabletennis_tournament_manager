{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
	packages = with pkgs; [
		rustc
		cargo
		rust-analyzer
		rustfmt
		clippy
    trunk
    llvm
    lld
    bubblewrap
	];

	shellHook = ''
		echo "Rust dev shell loaded"
	'';
}
