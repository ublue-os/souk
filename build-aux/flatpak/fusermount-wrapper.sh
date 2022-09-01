#!/bin/sh
echo "Running fusermount wrapper, redirecting to host."

dbus_address=$(cat /.flatpak-info | grep DBUS_SESSION_BUS_ADDRESS)
export $dbus_address

display=$(cat /.flatpak-info | grep DISPLAY)
export $display

echo "Set env variable: DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS DISPLAY=$DISPLAY"

flatpak-spawn --host hash fusermount &> /dev/null
retval=$?

if [ $retval -eq 0 ]; then
  binary="fusermount"
else
  echo "Unable to execute fusermount, trying fusermount3"
  binary="fusermount3"
fi

if [ -z "$_FUSE_COMMFD" ]; then
    FD_ARGS=
else
    FD_ARGS="--env=_FUSE_COMMFD=${_FUSE_COMMFD} --forward-fd=${_FUSE_COMMFD}"
fi
exec flatpak-spawn --host $FD_ARGS $binary "$@"