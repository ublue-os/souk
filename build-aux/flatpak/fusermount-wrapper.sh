#!/bin/sh
echo "Running fusermount wrapper, redirecting to host..."
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/flatpak/bus

# Test if the `fusermount` command is available
echo "Checking if fusermount is available on host system..."
flatpak-spawn --host fusermount --version
retval=$?

if [ $retval -eq 0 ]; then
  echo "Using fusermount."
  binary="fusermount"
else
  # Some distros don't ship `fusermount` anymore, but `fusermount3` like Alpine
  echo "Unable to execute fusermount, trying to use fusermount3."
  binary="fusermount3"
fi

# The actual call on the host side
if [ -z "$_FUSE_COMMFD" ]; then
    FD_ARGS=
else
    FD_ARGS="--env=_FUSE_COMMFD=${_FUSE_COMMFD} --forward-fd=${_FUSE_COMMFD}"
fi
exec flatpak-spawn --host $FD_ARGS $binary "$@"