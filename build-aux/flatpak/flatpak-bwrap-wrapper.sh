#!/bin/sh
echo "Running flatpak-bwrap wrapper, redirecting to host..."

# Inspect which fds are currently opened, and forward them to the host side.
echo "Open file descriptors:"
ls -l /proc/$$/fd

fds=""
for fd in $(ls /proc/$$/fd); do
  case "$fd" in
    0|1|2|3|255)
      ;;
    *)
      fds="${fds} --forward-fd=$fd"
      echo "Forwarding fd $fd"
      ;;
  esac
done

# Set DBUS_SESSION_BUS_ADDRESS env variable so the flatpak-spawn --host portal call works.
# The correct value is listed in the /.flatpak-info file.
dbus_address=$(cat /.flatpak-info | grep DBUS_SESSION_BUS_ADDRESS)
export $dbus_address

# Test if the `bwrap` command is available
flatpak-spawn --host bwrap --version &> /dev/null
retval=$?

if [ $retval -eq 0 ]; then
  binary="bwrap"
else
  echo "Unable to execute bwrap, falling back to flatpak-bwrap"
  binary="flatpak-bwrap"
fi

# The actual call on the host side
exec flatpak-spawn --host $fds $binary "$@"