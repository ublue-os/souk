vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
localedir  = join_paths(get_option('prefix'), get_option('localedir'))

# Configuration file
conf = configuration_data()
conf.set_quoted('NAME', 'FlatpakFrontend')
conf.set_quoted('PKGNAME', meson.project_name())
conf.set_quoted('APP_ID', 'de.haeckerfelix.FlatpakFrontend')
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('PROFILE', 'devel')
conf.set_quoted('VCS_TAG', vcs_tag)
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

sources = files(
  'backend/transaction_backend/host_backend.rs',
  'backend/transaction_backend/mod.rs',
  'backend/transaction_backend/sandbox_backend.rs',
  'backend/backend_message.rs',
  'backend/flatpak_backend.rs',
  'backend/mod.rs',
  'backend/package.rs',
  'backend/package_action.rs',
  'backend/package_transaction.rs',
  'backend/package_transaction.rs',
  'backend/utils.rs',

  'database/connection.rs',
  'database/models.rs',
  'database/mod.rs',
  'database/package_database.rs',
  'database/queries.rs',
  'database/schema.rs',

  'ui/page/explore_page.rs',
  'ui/page/installed_page.rs',
  'ui/page/mod.rs',
  'ui/page/package_details_page.rs',
  'ui/app_buttons_box.rs',
  'ui/app_tile.rs',
  'ui/mod.rs',
  'ui/project_urls_box.rs',
  'ui/releases_box.rs',
  'ui/screenshots_box.rs',
  'ui/transaction_progressbar.rs',
  'ui/utils.rs',
  'ui/window.rs',

  'app.rs',
  'config.rs',
  'error.rs',
  'main.rs',
  'path.rs',
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
