#!/usr/bin/env sh

_INSTALL_PATH=/usr/local/bin
_SERVICE_INSTALL_PATH=/etc/systemd/system
_BIN_NAME=jonsbo_th
_SERVICE_NAME=jonsbo.service

install() {
    echo "Installing $_BIN_NAME..."
    cp target/release/$_BIN_NAME "$_INSTALL_PATH/"
    cp "$_SERVICE_NAME" "$_SERVICE_INSTALL_PATH/"

    systemctl daemon-reload
    systemctl enable --now "$_SERVICE_NAME"
    echo "Done. Your water block should now be displaying your CPU package temperature."
}

uninstall() {
    echo "Uninstalling $_BIN_NAME..."

    systemctl stop "$_SERVICE_NAME" 2>/dev/null
    systemctl disable "$_SERVICE_NAME" 2>/dev/null

    rm -f "$_INSTALL_PATH/$_BIN_NAME"
    rm -f "$_SERVICE_INSTALL_PATH/$_SERVICE_NAME"

    systemctl daemon-reload
    echo "Done. Your water block display should go dark in a few seconds."
}

if [ "$1" = "--uninstall" ]; then
    uninstall
else
    install "$1"
fi
