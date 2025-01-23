FROM fedora:rawhide

# Install rustup and Souk dependencies
RUN dnf -y install \
  rustup \
  git \
  meson \
  cmake \
  gcc \
  openssl-devel \
  flatpak-devel \
  sqlite-devel \
  dbus-devel \
  gtk4-devel \
  libadwaita-devel \
  update-desktop-database \
  libxmlb-devel

# Install Rust
RUN rustup-init -y
