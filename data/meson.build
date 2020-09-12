gnome = import('gnome')

# Icons #
subdir('icons')

# Desktop file #
desktop_conf = configuration_data()
desktop_conf.set('icon', app_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: files('org.gnome.Store.desktop.in.in'),
    output: 'org.gnome.Store.desktop.in',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(app_id),
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'applications')
)

# Validate desktop file #
desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif


# Appstream metadata #
appdata_conf = configuration_data()
appdata_conf.set('appid', app_id)
appstream_file = i18n.merge_file(
  input: configure_file(
    input: files('org.gnome.Store.metainfo.xml.in.in'),
    output: 'org.gnome.Store.metainfo.xml.in',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(app_id),
  po_dir: '../po',
  install: true,
  install_dir: join_paths(datadir, 'metainfo')
)


# Gschema file #
install_data('org.gnome.Store.gschema.xml',
  install_dir: join_paths(datadir, 'glib-2.0/schemas')
)

# Validate gschema file #
compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif


# DBUS service
service_conf = configuration_data()
service_conf.set('appid', app_id)
service_conf.set('bindir', join_paths(get_option('prefix'), get_option('bindir')))
configure_file(
  input: 'org.gnome.Store.service.in',
  output: '@0@.service'.format(app_id),
  configuration: service_conf,
  install_dir: join_paths(datadir,'dbus-1', 'services')
)


# Resources
gnome.compile_resources(
  app_id,
  'org.gnome.Store.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install: true,
  install_dir: join_paths(datadir, meson.project_name())
)

