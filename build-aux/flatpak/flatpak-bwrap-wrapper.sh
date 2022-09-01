#!/bin/sh
echo "Running flatpak-bwrap wrapper, redirecting to host."

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

dbus_address=$(cat /.flatpak-info | grep DBUS_SESSION_BUS_ADDRESS)
export $dbus_address

display=$(cat /.flatpak-info | grep DISPLAY)
export $display

echo "Set env variable: DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS DISPLAY=$DISPLAY"

flatpak-spawn --host hash bwrap &> /dev/null
retval=$?

if [ $retval -eq 0 ]; then
  binary="bwrap"
else
  echo "Unable to execute bwrap, falling back to flatpak-bwrap"
  binary="flatpak-bwrap"
fi

exec flatpak-spawn --host $fds $binary "$@"