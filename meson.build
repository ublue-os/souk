project('souk', 'rust',
          version: '0.0.1',
    meson_version: '>= 0.59.0',
          license: 'GPL-3.0-or-later',
)

gnome = import('gnome')
i18n = import('i18n')

# Dependencies
dependency('sqlite3', version: '>= 3.20')
dependency('openssl', version: '>= 1.0')
dependency('dbus-1')

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gdk-pixbuf-2.0')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>=1.1.0')

# Initialize project variables
name       = 'Souk'
base_id    = 'de.haeckerfelix.Souk'
path_id    = '/de/haeckerfelix/Souk'
app_id     = base_id
vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
profile    = get_option('profile')
datadir    = join_paths(get_option('prefix'), get_option('datadir'))
localedir  = join_paths(get_option('prefix'), get_option('localedir'))
bindir     = join_paths(get_option('prefix'), get_option('bindir'))

# Change app id for development builds
if profile == 'development'
  app_id = '@0@.Devel'.format(app_id)
endif

subdir('data')
subdir('src')
subdir('po')
