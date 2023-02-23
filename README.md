# Souk

## Development

Souk uses a multi-process architecture, with a separate "worker" process that is activated via DBus. 
To develop / compile Souk with GNOME Builder, it is recommended to set the following settings (requires GNOME Builder 43 or higher):

1. Open Builder "Run" menu -> "Select Run Command..."
2. Click on "Command" in sidebar -> "Create Command"
3. Enter "Souk with worker" as name
4. Enter `sh -c 'souk & NO_INACTIVITY_TIMEOUT=1 souk-worker'` as Shell command
5. Enable "Use Subshell" option
6. Click on "Save"
7. Now open "Application" from sidebar, and select "Souk with worker" as "Run Command"

With these settings the worker process is started together with the main process and no DBus activation will occur.
