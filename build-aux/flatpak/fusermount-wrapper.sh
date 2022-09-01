#!/bin/sh
echo "Running fusermount wrapper, redirecting to host..."

# Set DBUS_SESSION_BUS_ADDRESS env variable so the flatpak-spawn --host portal call works.
# The correct value is listed in the /.flatpak-info file.
dbus_address=$(cat /.flatpak-info | grep DBUS_SESSION_BUS_ADDRESS)
export $dbus_address

# Test if the `fusermount` command is available
flatpak-spawn --host fusermount --version &> /dev/null
retval=$?

if [ $retval -eq 0 ]; then
  binary="fusermount"
else
  # Some distros don't ship `fusermount` anymore, but `fusermount3` like Alpine
  echo "Unable to execute fusermount, trying fusermount3"
  binary="fusermount3"
fi

# The actual call on the host side
if [ -z "$_FUSE_COMMFD" ]; then
    FD_ARGS=
else
    FD_ARGS="--env=_FUSE_COMMFD=${_FUSE_COMMFD} --forward-fd=${_FUSE_COMMFD}"
fi
exec flatpak-spawn --host $FD_ARGS $binary "$@"