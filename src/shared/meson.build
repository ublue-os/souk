# Configuration file
conf = configuration_data()
conf.set_quoted('NAME', name)
conf.set_quoted('PKGNAME', meson.project_name())
conf.set_quoted('APP_ID', app_id)
conf.set_quoted('WORKER_APP_ID', '@0@.Worker'.format(app_id))
conf.set_quoted('PATH_ID', path_id)
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('PROFILE', profile)
conf.set_quoted('VCS_TAG', vcs_tag)
conf.set_quoted('LOCALEDIR', localedir)
conf.set_quoted('DATADIR', datadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

run_command(
  'cp',
  join_paths(meson.project_build_root(), 'src', 'shared', 'config.rs'),
  join_paths(meson.project_source_root(), 'src', 'shared', 'config.rs'),
  check: true
)