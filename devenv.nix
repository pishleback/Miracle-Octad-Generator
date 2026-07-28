{ pkgs, ... }:

{
  languages.rust = {
    enable = true; 
    channel = "stable";
    targets = [ "wasm32-unknown-unknown" ];
  };

  packages = with pkgs; [
    trunk
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

  processes.wasm_debug = {
    exec = "trunk serve --port 8080";
    ready = {
      http.get = {
        port = 8080;
        path = "/";
      };
      initial_delay = 15;
      period = 30;
    };
    restart = {
      on = "always";
      max = null;
    };
  };

  processes.wasm_release = {
    exec = "trunk serve --release --port 8081";
    ready = {
      http.get = {
        port = 8081;
        path = "/";
      };
      initial_delay = 15;
      period = 30;
    };
    restart = {
      on = "always";
      max = null;
    };
  };
}