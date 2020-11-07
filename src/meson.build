# Configuration file
conf = configuration_data()
conf.set_quoted('NAME', name)
conf.set_quoted('PKGNAME', meson.project_name())
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('PROFILE', profile)
conf.set_quoted('VCS_TAG', vcs_tag)
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)


# Source code itself
sources = files(
  'backend/package/installed_info.rs',
  'backend/package/mod.rs',
  'backend/package/package_kind.rs',
  'backend/package/remote_info.rs',
  'backend/transaction_backend/host_backend.rs',
  'backend/transaction_backend/mod.rs',
  'backend/transaction_backend/sandbox_backend.rs',
  'backend/flatpak_backend.rs',
  'backend/mod.rs',
  'backend/legacy_package.rs',
  'backend/package_action.rs',
  'backend/transaction.rs',
  'backend/transaction_mode.rs',
  'backend/transaction_state.rs',

  'database/connection.rs',
  'database/models.rs',
  'database/mod.rs',
  'database/package_database.rs',
  'database/queries.rs',
  'database/schema.rs',

  'ui/package_widgets/releases_box.rs',
  'ui/package_widgets/mod.rs',
  'ui/package_widgets/project_urls_box.rs',
  'ui/package_widgets/screenshots_box.rs',

  'ui/pages/explore_page.rs',
  'ui/pages/installed_page.rs',
  'ui/pages/mod.rs',
  'ui/pages/package_details_page.rs',
  'ui/pages/search_page.rs',

  'ui/about_dialog.rs',
  'ui/mod.rs',
  'ui/package_action_button.rs',
  'ui/package_row.rs',
  'ui/package_tile.rs',
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
    localedir,
    profile
  ]
)

