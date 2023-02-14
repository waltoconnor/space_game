 with import <nixpkgs> {};
let
#   my-python = pkgs.python3;
#   python-with-my-packages = my-python.withPackages (p: with p; [
#     ipaddress
#     matplotlib
#     scipy
#     numpy
#     reportlab
#     # other python packages you want
#   ]);
  # define packages to install with special handling for OSX
  basePackages = [
    clang
    # python-with-my-packages
    zlib
    pkg-config
    libclang
    cmake
    autoconf
    nodePackages.tailwindcss
    nodejs
  ];

  inputs = basePackages
    ++ lib.optional stdenv.isLinux inotify-tools;

  # define shell startup command
  hooks = ''
  
  '';

in mkShell {
  GODOT4_BIN="${godot_4}/bin/godot";
  LIBCLANG_PATH="${libclang.lib}/lib";
  buildInputs = inputs;
  nativeBuildInputs = with pkgs; [ rustc cargo clang pkg-config libclang ];
  shellHook = hooks;
  RUST_SRC_PATH="${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
