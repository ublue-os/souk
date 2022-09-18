subdir('shared')

# Setup cargo
cargo = find_program('cargo')
cargo_env = [ 'CARGO_HOME=' + meson.project_build_root() / 'cargo-home' ]
cargo_options = [ '--manifest-path', meson.project_source_root() / 'Cargo.toml' ]
cargo_options += [ '--target-dir', meson.project_build_root() / 'target' ]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: rust_target,
  console: true,
  command: [
    'env',
    cargo_env,
    cargo, 'build',
    cargo_options,
  ]
)

copy_binary = custom_target(
  'cp-binary',
  depends: cargo_build,
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  install: true,
  install_dir: bindir,
  command: [
    'cp', 'target' / rust_target / meson.project_name(), '@OUTPUT@',
  ]
)

copy_worker_binary = custom_target(
  'cp-worker-binary',
  depends: cargo_build,
  build_by_default: true,
  build_always_stale: true,
  output: '@0@-worker'.format(meson.project_name()),
  install: true,
  install_dir: bindir,
  command: [
    'cp', 'target' / rust_target / '@0@-worker'.format(meson.project_name()), '@OUTPUT@',
  ]
)

# docs
cargo_doc = custom_target(
  'cargo-doc',
  build_by_default: false,
  build_always_stale: true,
  output: 'doc',
  console: true,
  command: [
    'env', cargo_env,
    cargo, 'doc',
    cargo_options + [ '--no-deps', '--lib', '--document-private-items'],
  ],
)