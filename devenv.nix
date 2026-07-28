{ pkgs, ... }:

{
  languages.rust = {
    enable = true; 
    channel = "stable";
  };

  packages = with pkgs; [
    libX11
    libXcursor
    libXrandr
    libXi
    libxcb
    libxkbcommon
    wayland
    wayland-protocols
    mesa
    libGL
    libglvnd
    vulkan-loader
  ];

  env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.mesa
    pkgs.libGL
    pkgs.libglvnd
    pkgs.wayland
    pkgs.libxkbcommon
  ];
}