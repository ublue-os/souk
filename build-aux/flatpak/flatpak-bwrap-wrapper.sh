#!/bin/sh
echo "Running flatpak-bwrap wrapper, redirecting to host"

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

export DISPLAY=:0
export DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus

flatpak-spawn --host hash bwrap &> /dev/null
retval=$?

if [ $retval -eq 0 ]; then
  binary="bwrap"
else
  echo "bwrap is not available, fallback to flatpak-bwrap"
  binary="flatpak-bwrap"
fi

exec flatpak-spawn --host $fds $binary "$@"