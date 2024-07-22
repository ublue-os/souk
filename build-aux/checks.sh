#!/bin/sh

export LC_ALL=C

# Usage info
show_help() {
cat << EOF
Run conformity checks on the current Rust project.

USAGE: ${0##*/} [CHECK]
EOF
}

# Style helpers
act="\e[1;32m"
warn="\e[1;33m"
err="\e[1;31m"
pos="\e[32m"
res="\e[0m"

# Common styled strings
checking="${act}Checking${res}"
warning="${warn}Warning${res}"
failed="${err}Failed${res}"
ok="${pos}OK${res}"

check_tool_availability() {
    echo ""
    if ! $2 >/dev/null 2>&1; then
        echo -e "$warning $1 is not installed, skipping check."
        retval=1
    else
        echo -e "$checking $1 …"
        retval=0
    fi
    return "$retval"
}

execute() {
    if ! $2; then
        echo ""
        echo -e "$1 result: $failed"
        exit 1
    else
        echo ""
        echo -e "$1 result: $ok"
    fi
}

cargo_fmt() {
    if ! check_tool_availability "cargo-fmt" "cargo fmt --version"; then
        return
    fi
    execute "cargo-fmt" "cargo fmt --all -- --check"
}

cargo_typos() {
    if ! check_tool_availability "typos-cli" "$HOME/.cargo/bin/typos --version"; then
        return
    fi
    execute "cargo-typos" "$HOME/.cargo/bin/typos"
}

cargo_deny() {
    if ! check_tool_availability "cargo-deny" "cargo deny --version"; then
        return
    fi
    execute "cargo-deny" "cargo deny --log-level error check"
}

cargo_clippy() {
    if ! check_tool_availability "cargo-clippy" "cargo clippy --version"; then
        return
    fi
    execute "cargo-clippy" "cargo clippy --all -- -D warnings"
}

potfiles() {
    if ! check_tool_availability "potfiles" "git --version"; then
        return
    fi
    git ls-files 'src/*.rs' 'src/*.ui' 'data/*.ui' 'data/*.desktop.in*' '*.gschema.xml.in' '*.metainfo.xml.in*' > po/POTFILES.in
    execute "potfiles" "git diff --exit-code po/POTFILES.in"
}


# Check arguments
while [[ "$1" ]]; do case $1 in
    cargo_fmt )
        cargo_fmt
        exit 0
        ;;
    cargo_typos )
        cargo_typos
        exit 0
        ;;
    cargo_deny )
        cargo_deny
        exit 0
        ;;
    cargo_clippy )
        cargo_clippy
        exit 0
        ;;
    potfiles )
        potfiles
        exit 0
        ;;
    *)
        show_help >&2
        exit 1
esac; shift; done

# Run
cargo_fmt
cargo_typos
cargo_deny
cargo_clippy
potfiles
